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
  ui.rs                    UI facade and shared layout
  ui/
    world_view.rs          terrain/fluid/device draw pass and hit testing
    hud.rs                 objectives, budget, time, alerts
    build_palette.rs       selection and placement preview
    inspector.rs           selected-cell/device facts
    overlays.rs            overlay selection and legend
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
2. Apply UI-only commands such as camera and overlay changes.
3. Admit gameplay commands in their captured order; return a result for each.
4. Add scaled frame time to an accumulator when running.
5. Execute fixed simulation ticks while the accumulator permits, capped at
   eight per render frame. Retain excess time; do not increase tick duration.
6. Evaluate objectives and tutorial triggers after each completed tick.
7. Build a read-only view model and render once.

Pause sets simulation time scale to zero. Queued builds may be authored while
paused but only become world entities on an explicit `CommitPlan` command.
Committing is allowed while paused; devices do not operate until a later tick.

## 5. Command contract

All state-changing player intent MUST be represented by a serializable command:

```rust
enum MissionCommand {
    MoveSurveyCursor { to: CellPos },
    SelectCell { at: CellPos },
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
`protected_cell`, `insufficient_budget`, `tutorial_locked`, and
`mission_not_active`. UI text maps from codes; simulation logic does not own
localized prose.

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
- A state hash includes authoritative cells, devices, objectives, tick, budget,
  scheduled events, and tutorial progress; it excludes camera and animation.

## 7. Simulation tick phases

Every tick executes these phases in this exact order:

1. Apply scheduled mission events for this tick.
2. Inject enabled authored sources at their current rates.
3. Apply accepted device setting changes pending for this tick.
4. Sample sensors from the prior completed world state.
5. Compute powered device intake/transfer proposals.
6. Compute open-terrain fluid transfer proposals.
7. Limit and atomically apply all transfer proposals.
8. Resolve material interactions in row-major cell order.
9. Apply heat exchange and phase changes.
10. Apply filters, drains, turbines, reservoirs, and relay post-transfer effects.
11. Recalculate derived pressure, alerts, and overlay fields.
12. Evaluate failure boundaries, objectives, and stability counters.
13. Increment tick and emit ordered simulation events.

Exact rules and quantities are in `03_SIMULATION_SPEC.md`.

## 8. State separation

`Definition` data is immutable after validated load. `MissionState` is saved and
hashed. `PresentationState` is neither saved nor hashed unless user preference
requires persistence.

| State | Examples | Saved | Hashed |
| --- | --- | --- | --- |
| Definitions | fluid/device/mission records | No | Definition version only |
| Mission | cells, devices, budget, tick, objectives, events | Yes | Yes |
| Tutorial | current step, completed steps, prompt cooldown | Yes | Yes |
| Campaign | unlocked level, completion, preferences | Yes | No mission hash |
| Presentation | camera tween, particles, hovered cell | No | No |
| User settings | UI scale, reduced motion, bindings | Separate | No |

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
`objective_changed`, and `mission_terminal`. Tutorials, notifications, sound,
and effects subscribe to these facts; none may detect success by scraping UI.
