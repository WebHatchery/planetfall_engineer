# Engineer Implementation Plan

## 1. Execution contract

Work packages are ordered. An implementation agent MUST finish a package's
tests and acceptance before starting a dependent package. It MAY split a package
into smaller commits, but MUST NOT combine later content with an unproved
foundation. Focused tests may run during work; each milestone ends with
`publish.ps1` with no parameters.

Before adding project-local runtime, input, camera, assets, UI, persistence, or
capture behavior, inspect `macroquad-toolkit`. General reusable capability goes
there first. Do not broaden unrelated code while satisfying a package.

Each completed package records:

- requirement IDs addressed and files changed;
- tests/reference replays added or updated;
- `publish.ps1` result at milestone boundaries;
- any documented deviation, with the affected acceptance updated first;
- a commit following `rust_management/docs/COMMIT_STYLE.md`.

## 2. Milestone A — Deterministic world foundation

### WP-A1 Replace template domain state

Implement integer units, `CellPos`, definitions, cell/world state, stable entity
IDs, row-major storage, and immutable/read-only world views. Remove points,
regenerating energy, fog, and generic action behavior once no longer used.

Acceptance: world construction validates dimensions/bounds; coordinates and
occupancy tests pass; no authoritative floats; save serde round trip is exact.

### WP-A2 Content registries and validator

Implement §04 definitions, embedded registries, deterministic collected errors,
RLE map decoding, reference validation, and reserved fluid/device records.

Acceptance: valid minimal content loads native/test; a fixture for every §04
validation class fails with field-qualified errors; WebGL uses embedded data.

### WP-A3 Fixed tick, commands, snapshots, and replay

Implement accumulator, pause/speed modes, command admission/results, ordered
tick shell, state hash, versioned snapshot, checkpoint, and command replay.

Acceptance: render-frame partition does not alter final hash; pause executes
zero ticks; 2x/4x run exact additional ticks; save/load/replay hashes agree.

### Milestone A commit

Commit after A1–A3 and run the publisher. Do not begin fluid rules until this
commit is clean.

## 3. Milestone B — Terrain and material simulation

### WP-B1 Terrain editing and surface flow

Implement terrain actions, placement validation, mixture storage, proposal/
limit/apply phases, source/drain fixtures, overlays' derived facts, and mass
ledger per §§03.2–03.4.

Acceptance: downhill, equal-head, competing-neighbor, ridge/notch, capacity,
boundary, source-backpressure, and terrain-edit tests pass.

### WP-B2 Steam, heat, and interactions

Implement airborne diffusion, heat exchange, boiling/condensation, cooling lava,
water/lava, lava/slurry, dilution, pending terrain products, and event emission.

Acceptance: exact table-driven reaction quantities at 1, 120, 250, 1,000, and
capacity-limited `vU`; phase transitions; no-reaction pairs; deterministic hash.

### WP-B3 All-fluid laboratory

Author `lab_fluids_all`, automation, assertions, live laboratory HUD, evidence
capture, and headless scenario. Reserved bays remain explicit.

Acceptance: every §05.2 slice assertion passes, lab reset is exact, mass balance
holds, automatic/free modes work, and the capture is readable at 1280x720.

### Milestone B commit

Commit terrain/fluids/laboratory together after the reference hash is stable;
run the publisher and record its test/build/capture output.

## 4. Milestone C — Planning and device framework

### WP-C1 Build plan UX and device runtime

Implement build registry/palette, ghost/rotation, validity reasons, queued plan,
budget reservation, commit, selection/inspect, common device entity state,
network connections, and powered-device phase.

Acceptance: all command rejection codes, footprint rotations, overlap, budget,
queue removal, atomic commit, and renderer-independent state tests pass.

### WP-C2 Simple flow devices

Implement channel, floodgate, spillway, and reservoir behavior and settings.
Author their four showcase maps with automation/assertions/captures.

Acceptance: VM-CHANNEL, VM-FLOODGATE, VM-RESERVOIR, and VM-SPILLWAY pass.

### WP-C3 Networks and active devices

Implement pipe connectivity/capacity/pressure, pump, turbine power, sensor sample
and delayed output, filter, and rune relay. Author six showcase maps.

Acceptance: all remaining §05 device-map cases pass; validator proves exactly
one showcase per enabled device; no showcase includes another placeable device.

### Milestone C commit

Commit only after all ten showcases pass in one deterministic suite and the
publisher completes.

## 5. Milestone D — Mission framework and player interface

### WP-D1 Mission lifecycle

Implement briefing, active, success/failure, stability windows, objectives,
fixed scheduled events/warnings, soft reset, debrief, unlock progression, and
active/campaign save behavior.

Acceptance: failure wins precedence on same tick; stability resets correctly;
events fire once; checkpoints restore hash; terminal missions admit no commands.

### WP-D2 Engineering UI

Implement world rendering with height/fluid/device cues, camera/cursor, build
palette, inspector, objective/time/budget strip, alerts, overlays/legends, pause
menu, debrief, UI scale, reduced motion, and clickable paths for all actions.

Acceptance: layouts and controls pass §09 resolutions; no required action is
keyboard-only; non-color cues exist; camera never changes simulation hash.

### WP-D3 Tutorial engine

Implement data-driven tutorial predicates, command gating, focus, hints, skip,
checkpoints, binding-aware copy, and L01 sequence T01–T12.

Acceptance: all §07 automated tutorial tests pass before authoring final L01.

### Milestone D commit

Commit the reusable mission/UI/tutorial framework and run the publisher.

## 6. Milestone E — First three levels

### WP-E1 Author and prove L01

Build exact L01 state/objectives, tutorial integration, reference solution,
failure/recovery, briefing/debrief, and captures.

Acceptance: all §06.2 and §07 criteria pass from clean save and tutorial replay.

### WP-E2 Author and prove L02

Build L02, surge warnings/effect, optional sensor objective, reference solution,
recovery checkpoint, and captures.

Acceptance: all §06.3 criteria; solution succeeds with stated spare budget;
sensor-free solution also completes.

### WP-E3 Author and prove L03

Build L03, reaction explanation, lava surge, turbine/relay objectives, reference
solution, water exhaustion recovery, restoration state, and captures.

Acceptance: all §06.4 criteria; solution respects budget/water reserves and
exact interaction accounting.

### Milestone E commits

Each campaign level is a major content milestone and MUST receive its own commit
after its reference replay and focused tests pass. Run the full publisher after
L01 and L02 commits if shared code changed; always run it after L03.

## 7. Milestone F — Slice hardening

### WP-F1 Determinism and persistence audit

Replay every map twice, compare hashes, save/load at representative ticks and
tutorial steps, test content-version mismatch, and audit all iteration order.

### WP-F2 Performance and extreme-state audit

Run §09 worst-case maps, capacity/pressure/heat/contamination bounds, resize and
WebGL sessions, rapid pause/speed/placement input, and one-hour simulated soak.

### WP-F3 Release evidence and handoff

Refresh all captures, verify catalog thumbnail is a title-screen capture, update
README status, run `publish.ps1`, and record shipped/deferred scope against §01.

### Milestone F commit

Commit the completion audit only when all §09 gates pass and no required work
package remains. The body must state exact publisher results and honest deferrals.

## 8. Prohibited shortcuts

- Do not special-case campaign coordinates in Rust.
- Do not make visual particles authoritative material.
- Do not advance tutorials from prompt dismissal when an action/event is required.
- Do not make reference tests assert only “mission completed”; assert resources,
  terminal tick range, objective state, mass ledger, and final hash.
- Do not create a generic script engine for ten fixed slice device behaviors.
- Do not satisfy a showcase with screenshots alone; it is an executable fixture.
- Do not mark a work package complete on `cargo check` when its milestone requires
  publisher validation.
