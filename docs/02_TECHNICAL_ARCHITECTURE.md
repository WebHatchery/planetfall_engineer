# Technical Architecture

## 1. Architectural goals

The runtime MUST be deterministic, data-authored, renderer-independent, and
small enough to test without opening a window. Simulation state owns truth;
rendering, sound, notifications, and tutorial presentation consume emitted
facts and MUST NOT infer or mutate gameplay outcomes.

## 2. Target module ownership

Named files are targets, not a demand for empty scaffolding. Create a module
when its first behavior is implemented. Do not create `mod.rs` files.

```text
src/
  main.rs                  window setup and outer frame loop only
  game.rs                  app state orchestration and screen transitions
  app_state.rs             title/campaign/mission/debrief state enum
  input.rs                 raw input -> frame commands
  data.rs                  embedded registries and validation entry point
  data/
    definitions.rs         serde content definitions
    validation.rs          cross-reference and invariant validation
  simulation.rs            public simulation facade
  simulation/
    world.rs               cell storage, coordinates, terrain and occupancy
    fluids.rs              fluid transfer and phase updates
    interactions.rs        pairwise material reactions
    resources.rs           deposits, fabrication stock, spending, and refunds
    power.rs               field-grid generation, demand, and allocation
    devices.rs             device dispatcher and shared device state
    objectives.rs          objective evaluation and stability windows
    events.rs              deterministic scheduled mission events
    snapshot.rs            save/replay snapshot and stable-state hashing
  mission.rs               mission lifecycle and command admission
  mission/
    loader.rs              map/mission construction from definitions
    commands.rs            serializable player intent
    tutorial.rs            tutorial state machine
    replay.rs              command stream runner
  render3d.rs              3D render facade and immutable snapshot bridge
  render3d/
    terrain_mesh.rs        dirty-chunk top/cliff mesh generation
    fluid_mesh.rs          derived liquid surfaces/sides and steam volumes
    devices.rs             model instances, pipes, ports, ghosts, state visuals
    picking.rs             ray hits -> cells, edges, devices, and ports
    overlays.rs            3D grade/flow/hazard/selection geometry
    materials.rs           shaders, texture atlases, model/material handles
  ui.rs                    UI facade and shared layout
  ui/
    hud.rs                 objectives, fabrication, power, time, alerts
    build_palette.rs       selection and placement preview
    inspector.rs           selected-cell/device facts
    overlays.rs            overlay selection and screen-space legend
    tutorial.rs            prompt presentation and focus treatment
```

No non-test `.rs` file may reach 800 lines. Split by cohesive ownership before
the limit. Tests stay inline at the bottom of their owner or in `foo/tests.rs`
when the test module exceeds the repository threshold.

## 3. Dependency direction

```text
embedded JSON -> validated definitions -> mission loader -> simulation state
                                                        ^          |
input -> frame command -> mission command admission ----+          v
UI/render <------------- read-only view model + simulation events/hash
save/replay <----------- versioned snapshot + admitted command stream
```

- `simulation` MUST depend only on content value types, serde-compatible state,
  and standard deterministic collections/algorithms.
- `simulation` MUST NOT read input, wall-clock time, frame time, textures,
  window size, filesystem state, or persistence APIs.
- `ui` MUST submit commands. It MUST NOT directly change world state.
- `render3d` consumes immutable snapshots and presentation events. It MUST NOT
  own cells, fluid quantities, device settings, objectives, or placement truth.
- `mission` owns whether a command is legal while paused, running, completed,
  failed, or tutorial-gated.
- Device implementations MUST use the same transfer/reaction APIs as terrain
  flow; no device may edit unrelated cells as a rendering shortcut.

## 4. Time model and frame order

Constants:

```text
SIM_TICKS_PER_SECOND = 10
SIM_DT_SECONDS       = 0.1
MAX_TICKS_PER_FRAME  = 8
```

Frame order MUST be:

1. Capture raw input once and translate it to ordered `FrameCommand`s.
2. Apply presentation-only commands such as 3D camera and overlay changes; use
   the exact current camera/viewport to ray-pick any pointer world target.
3. Admit gameplay commands in their captured order; return a result for each.
4. Add scaled frame time to an accumulator when running.
5. Execute fixed simulation ticks while the accumulator permits, capped at
   eight per render frame. Retain excess time; do not increase tick duration.
6. Evaluate objectives and tutorial triggers after each completed tick.
7. Build a read-only view model, rebuild dirty derived 3D meshes, render the 3D
   world, restore the virtual 2D UI camera, and render HUD once.

Campaign orchestration remains in `game.rs`; player-facing placement preview
and verification transitions live in `game_placement.rs` and
`game_verification.rs` so the runtime controller stays below the source-file
size gate.

Pause sets simulation time scale to zero. Queued builds may be authored while
paused but only become world entities on an explicit `CommitPlan` command.
Committing is allowed while paused; devices do not operate until a later tick.

Save loading validates the schema version, installed content version, and both
world dimensions before constructing a session. Malformed, future, mismatched,
or dimension-incompatible saves return an actionable error; the caller keeps
the current session until the player chooses recovery. The state module tests
the exact round trip and each rejection path.

## 5. Command contract

All state-changing player intent MUST be represented by a serializable command:

```rust
enum MissionCommand {
    MoveSurveyCursor { to: CellPos },
    SelectCell { at: CellPos },
    RecoverDeposit { deposit_id: String },
    QueueTerrainEdit { at: CellPos, edit: TerrainEdit },
    QueueDevice { device_id: String, anchor: CellPos, rotation: Rotation },
    RemoveQueued { queue_id: u32 },
    CommitPlan,
    ConfigureDevice { entity_id: u32, setting: DeviceSetting },
    SetTimeControl { mode: TimeControl },
    ResetCheckpoint,
}
```

The actual Rust types MAY be split, but their serialized meaning MUST remain.
Each admission returns `Accepted` or one stable rejection code, including
`out_of_bounds`, `occupied`, `invalid_surface`, `invalid_connection`,
`protected_cell`, `deposit_depleted`, `insufficient_fabrication`,
`power_unavailable`, `tutorial_locked`, and
`mission_not_active`. UI text maps from codes; simulation logic does not own
localized prose.

The build palette keeps a selected device and quarter-turn rotation in the
presentation shell. Its translucent footprint ghost is derived from the same
device footprint and anchor used by queue admission; `Z` changes the rotation,
and the resulting `QueuedPlan` carries it through commit into the simulation.

## 6. Determinism rules

- Use integer or fixed-point quantities for authoritative terrain height,
  material volume, temperature, pressure, contamination, costs, and tick time.
- Never branch authoritative logic on `f32` render values.
- Iterate cells in row-major order and entities by ascending stable entity ID.
- Any competing transfers MUST be proposed from the start-of-phase snapshot,
  proportionally limited, then applied together.
- Use ordered maps/vectors where iteration affects outcomes. Hash collections
  are allowed only for membership tests that never determine ordering.
- Random events require a saved deterministic RNG seed and draw counter. The
  first slice's authored events SHOULD not require randomness.
- A state hash includes authoritative cells, devices, objectives, tick,
  fabrication stock/reservations, deposit depletion, power ledger/allocation,
  scheduled events, and tutorial progress; it excludes camera and animation.

## 7. Simulation tick phases

Every tick executes these phases in this exact order:

1. Apply scheduled mission events for this tick.
2. Inject enabled authored sources at their current rates.
3. Apply accepted device setting changes pending for this tick.
4. Build current supply from authored planetary sources plus the prior tick's
   turbine output; allocate it by power class and stable entity ID.
5. Sample powered sensors from the prior completed world state.
6. Compute powered device intake/transfer proposals.
7. Compute open-terrain fluid transfer proposals.
8. Limit and atomically apply all transfer proposals.
9. Resolve material interactions in row-major cell order.
10. Apply heat exchange and phase changes.
11. Apply powered filters, drains, turbines, reservoirs, and relay post-transfer
    effects; turbines record generation available on the next tick.
12. Recalculate derived pressure, power, alerts, and overlay fields.
13. Evaluate failure boundaries, objectives, and stability counters.
14. Increment tick and emit ordered simulation events.

Exact rules and quantities are in `03_SIMULATION_SPEC.md`.

## 8. State separation

`Definition` data is immutable after validated load. `MissionState` is saved and
hashed. `PresentationState` is neither saved nor hashed unless user preference
requires persistence.

| State | Examples | Saved | Hashed |
| --- | --- | --- | --- |
| Definitions | fluid/device/mission records | No | Definition version only |
| Mission | cells, devices, fabrication/deposits, power, tick, objectives, events | Yes | Yes |
| Tutorial | current step, completed steps, prompt cooldown | Yes | Yes |
| Campaign | unlocked level, completion, preferences | Yes | No mission hash |
| Presentation | camera tween, particles, hovered cell | No | No |
| User settings | UI scale, reduced motion, bindings | Separate | No |

The world-space conversion, orthographic camera, mesh ownership, model asset
contract, picking, render order, and acceptance tests are normative in
`11_3D_VISUAL_TECHNICAL_SPEC.md`. Reusable orthographic-camera, screen-ray,
ray/AABB, and GLB-loading support belongs in `macroquad-toolkit::render3d` rather
than project-local copies.

## 9. Persistence and migration

- Use macroquad-toolkit versioned save slots; do not add direct filesystem save
  access because WebGL must remain supported.
- Save envelope fields: `schema_version`, `content_version`, `campaign`, and
  optional `active_mission`.
- `active_mission` includes mission ID, content revision, authoritative state,
  tutorial state, admitted command history since checkpoint, and state hash.
- Autosave only after a completed tick or while paused with no tick in flight.
- Loading validates versions, data references, bounds, nonnegative quantities,
  and stored hash before replacing the live session.
- Migration functions are pure transforms. Unsupported future saves return an
  explanation and leave the current session unchanged.

## 10. Error and event handling

Expected invalid player commands return rejection codes. Invalid authored data
fails startup in development/tests with all collected validation errors. A
release build shows one blocking data-load screen rather than panicking inside
the frame loop. Runtime invariants such as negative volume are assertion/test
failures and MUST NOT be silently clamped except where the simulation spec
explicitly defines saturation.

Simulation events use stable kinds such as `fluid_entered_cell`,
`device_activated`, `material_reacted`, `threshold_crossed`,
`deposit_recovered`, `fabrication_spent`, `power_brownout`,
`objective_changed`, and `mission_terminal`. Tutorials, notifications, sound,
and effects subscribe to these facts; none may detect success by scraping UI.
