# Quality Gates

## 1. Completion rule

A milestone is acceptable only when its relevant focused tests pass and
`publish.ps1` with no parameters succeeds from the project root. The publisher
is the required integrated validation path; a local dev run does not replace it.

## 2. Static and unit gates

The test/publisher path MUST enforce:

- formatting check;
- compilation for all project targets exercised by the publisher;
- Clippy with warnings denied for project code;
- `cargo test` including the shared source-size gate;
- every non-test `.rs` file below 800 lines;
- content load and cross-reference validation;
- no enabled fluid/device without required verification coverage;
- shared-toolkit 3D camera, screen-ray, AABB, and GLB-loader tests;
- 3D terrain/fluid mesh, model/port, picking, and render-snapshot tests.

The implementation SHOULD add one test owner per behavior module rather than a
single integration-test crate, following `AGENTS.md` inline/child test rules.

## 3. Deterministic scenario suite

The following maps run headlessly through their full automatic command stream:

- `lab_fluids_all`;
- all ten `device_<id>` showcase maps;
- L01 guided reference solution and one alternate route;
- L02 reference, sensor-free completion, and unmitigated failure;
- L03 reference, insufficient-water recovery, and foundation failure.

For every scenario the suite MUST:

1. construct twice from definitions and compare initial hashes;
2. replay the same commands twice and compare per-checkpoint and final hashes;
3. assert every map-defined condition plus terminal tick range;
4. assert no invariant event was emitted;
5. save/load at its midpoint and prove the loaded continuation has the same
   final hash as uninterrupted play;
6. report source, drain, reaction, stored, and remaining material ledgers.

Reference hashes are expected data. A changed hash fails until the behavioral
change and new expected state are reviewed together.

## 4. Conservation tolerance

Integer volume rules permit exact accounting. For each material family and
scenario:

```text
initial + authored_source + reaction_products
- authored_drains - reaction_consumption
= world_surface + world_airborne + device_storage + pending_terrain_product
```

The allowed unexplained difference is exactly zero `vU`. Presentation values
may round for display; tests use authoritative units. Heat is not conserved by
the abstract ambient exchange and is excluded from mass balance.

The runtime exposes `SimulationWorld::mass_balance_error()` for scenario and
showcase assertions; it includes surface, airborne, and pending rock/vitrified
products and is required to equal zero by the campaign and device replay tests.

## 5. UI and capture matrix

Automated deterministic captures MUST exist for title/mission selection, every
verification map, each campaign briefing, active campaign gameplay, each
level's main hazard, success debrief, and one failure/recovery state.

Every map capture MUST come from the production orthographic `Camera3D` path
with visible stepped terrain, fluid height, and machine geometry. The all-fluid
lab requires yaw quarters 0 and 2; each campaign level and device map requires
its authored default yaw plus any opposing yaw needed to expose occluded logic.
A 2D grid/debug capture cannot satisfy evidence.

Run required screens at:

| Size | Required result |
| --- | --- |
| 1280x720 | Reference layout; all captures |
| 1024x768 | No clipping/overlap; gameplay and tutorial captures |
| 800x600 | No inaccessible controls; gameplay, build palette, pause, debrief |

At every size:

- required buttons are wholly on-screen or reachable in a visible scroll area;
- tutorial prompt does not cover its focus target;
- objective, current time mode, budget, and critical alert remain visible;
- overlay legend and inspector can close without losing selection;
- text does not overlap or truncate IDs/values needed for engineering decisions;
- mouse hit regions match rendered controls after scaling;
- screen-ray hits match visible 3D cell/device/edge after UI viewport scaling;
- reduced-motion mode has no required information only in animation;
- water/lava/slurry/steam and alert levels have non-color distinctions.

The deterministic UI-unit suite maps logical click coordinates to every title
entry, the fluid laboratory, all ten device bays, all ten build-palette entries,
Rotate/Commit/Cancel, and Pause/1X/2X/4X. A control with no matching action or
an overlapping/off-panel coordinate fails that suite.

Mission alert state is authoritative and text-labeled at four levels: clear,
advisory, warning, and critical. Hazard failure timers remain separate from
the display level, so a warning can be inspected before a terminal failure.
The engineering readout labels selected surface depth, heat, contamination, and
device status as authoritative values.
The L02 and L03 authored source-rate surges are applied at their documented
ticks and are covered by schedule tests plus deterministic scenario replay.
Active L02/L03 capture scenes run those authored sources through the production
session update loop rather than a replay-only fixture.
Campaign tests also advance both authored constructors through their terminal
success state with only those live sources and scheduled events.

Captures live under `docs/verification/` with stable names
`<map_or_screen>_<width>x<height>.png`. `catalog_thumbnail.png` at repository
root MUST be a current title-screen capture before release publishing.

Current evidence includes the title screen, the verification-grounds menu, the three campaign briefings, `lab_fluids_all`, all
ten `device_*` showcases, gameplay at the three required viewport sizes, and a
`failure_recovery` terminal panel capture.
The `pause_menu` capture proves the frozen-state recovery overlay and its
resume/save/load/reset actions.

Verification maps are also inspectable in free play: F6 resets the active
fixture, F8 advances one tick, and the engineering readout reports PASS/FAIL,
exact balance error, and the injected/drained/reacted/product ledger.

The active build selection also renders a translucent 3D footprint ghost;
`Z` rotates it through all four quarter turns before queue/commit.
The HUD provides the selected device cost and a text validity reason, while a
blocked preview changes to a red ghost and remains non-committable.
The palette's visible Rotate, Commit, and Cancel buttons share the scaled mouse
regions used by the keyboard-equivalent commands.
The time-control row exposes Pause, 1X, 2X, and 4X through the same scaled
mouse interaction path; the gameplay evidence includes both 1280×720 and
800×600 views.

L01 renders each incomplete tutorial step as a field prompt with an instruction
and required action; the prompt disappears only after the authoritative event
advances the tutorial state.

The replay owner covers the ten campaign scenarios listed above, including
alternate success paths, authored hazard failures, and the L03 insufficient-
water recovery path. Each report includes midpoint continuation and the four
material ledger counters; midpoint continuation is performed after a serialized
serde round trip rather than an in-memory clone.

## 6. 3D functional gates

- Orthographic projection remains perspective-free at all six zoom levels.
- Pan clamps to map bounds; four quarter rotations return to the initial camera
  matrix within float presentation tolerance.
- Known-point ray fixtures select the same intended cell/device at all four yaw
  quarters, six zoom levels, and three required window sizes.
- Terrain mesh emits no internal equal-height faces, exact stepped side heights,
  scaled UVs, and valid `u16` indices; boundary edits dirty adjacent chunks.
- Fluid top/side geometry matches authoritative height/depth and does not
  z-fight terrain at 0, 1, 250, 1,000, and 8,000 `vU`.
- Device model AABBs/ports rotate with footprint; pipe topology has no open gap
  where connected; placeholder and authored models share picking/state behavior.
- Opaque/translucent render order, selection outline, placement ghost, occlusion
  fade, and HUD restoration are visually correct at every yaw.
- Model, shader, atlas, or GLB failure names the asset and shows a 3D diagnostic
  placeholder without changing simulation state.

## 7. Manual interaction checks

At each content milestone, perform and record:

- pan, zoom, all four 3D yaw quarters, survey selection, edge/device picking,
  occlusion fade, focus, and inspect with mouse and keyboard equivalents;
- queue/rotate/cancel/commit valid placement and each common invalid placement;
- pause, 1x, 2x, 4x, rapid toggling, and window focus loss;
- reset checkpoint, restart mission, save, load, and content mismatch notice;
- tutorial normal path, delayed hints, skip, replay, and checkpoint recovery;
- resize while palette, inspector, tutorial prompt, and pause menu are open;
- WebGL persistence across page reload through toolkit save slots.

No manual check substitutes for deterministic behavior tests.

## 8. Performance gates

Use a release build with capture/diagnostic overlays disabled except the
performance counter.

### QG-P1 Reference desktop

At 1280x720 on the development machine, `lab_fluids_all` with all slice bays
and interactions active for 10 simulated minutes MUST maintain:

- median render rate >=60 FPS at 1x;
- 99th-percentile frame time <=33.3 ms;
- no simulation backlog lasting more than 1 real second;
- no growing memory trend after the first minute;
- at most 500 world draw submissions and 300,000 world vertices in the fully
  active laboratory reference frame, reported by diagnostic counters;
- no terrain/fluid chunk rebuild when authoritative geometry did not change.

### QG-P2 WebGL floor

At 1280x720 in a supported desktop browser, the same scenario MUST maintain
median >=30 FPS at 1x and keep input responsive while the laboratory runs.
The same draw/vertex limits apply because WebGL uses identical authored content.

### QG-P3 Deterministic load

A 64x48 synthetic worst-case map with four surface entries per cell, maximum
enabled device count, and active objectives runs 10,000 ticks headlessly with
no panic, overflow, NaN/Inf presentation conversion, invalid invariant, or hash
difference between runs. Wall-clock time is reported but not fixed across CI
hardware. Per-tick code SHOULD reuse buffers and avoid allocations proportional
to active transfers after warm-up.

If a performance target fails, record map, build, platform, duration, median,
p99, backlog, and suspected owner. Do not reduce deterministic tick count or
silently drop simulation work to improve rendering.

The current automated soak covers the deterministic-load portion of QG-P3: two
64×48 worlds with four material entries per cell execute 10,000 ticks and must
finish with equal serialized state and zero balance error. Interactive FPS,
frame-time, memory-trend, and draw-submission counters remain capture/runtime
measurements rather than claims made by this headless test.

## 9. Failure-path gates

Tests or manual checks MUST cover:

- malformed/missing embedded data produces one actionable blocking error;
- corrupted, older supported, and unsupported-future saves preserve the current
  session until the user chooses a recovery action;
- full destination, depleted source, unavailable power, and disconnected
  device produce legible inactive/rejection states;
- hard failure and success on one tick resolves to failure;
- reset/restart cannot duplicate budget, material, unlock, or tutorial progress;
- terminal state cannot continue simulating behind debrief;
- source injection at capacity emits backpressure and does not lose ledger mass.

## 10. Release checklist

- [ ] Scope completion criteria in `01_PROJECT_SCOPE.md` are all true.
- [ ] All three campaign and eleven verification maps validate and replay.
- [ ] All maps render/pick through §11's orthographic 3D path; required yaw
      captures, mesh/model validation, and Windows/WebGL 3D gates pass.
- [ ] Tutorial reference, alternate inputs, skip, save/load, and reset pass.
- [ ] Static, unit, deterministic, failure, UI, and performance gates pass.
- [ ] `publish.ps1` succeeds with Windows and WebGL outputs.
- [ ] Evidence captures and `catalog_thumbnail.png` reflect the current build.
- [ ] README describes the implemented slice rather than the starter template.
- [ ] Working tree contains no accidental build/save artifacts.
- [ ] Final milestone commit follows the catalog commit standard and reports
      exact verification plus any non-blocking deferred polish.
