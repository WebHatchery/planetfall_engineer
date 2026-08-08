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

## 2. Milestone A — Deterministic 3D world foundation

### WP-A0 Upgrade shared 3D foundations

Extend `macroquad-toolkit::render3d` with §11's orthographic engineering camera,
viewport-aware screen-ray unprojection, ray/AABB helpers, and WebGL-safe static
GLB loading/diagnostics. Extend pack-first raw-byte lookup so models and shader
text load from the existing ZIP `AssetPack` before loose files. Keep APIs
game-neutral and cover camera matrices, all-yaw ray hits, viewport offsets,
AABBs, malformed assets, packed/loose assets, and native/WebGL compilation. Do
not create project-local substitutes for these shared needs.

Acceptance: toolkit tests pass; an orthographic stepped-cube fixture can pan,
zoom, rotate, and pick the same intended cube at every yaw/zoom; a static GLB
loads from bytes or produces the documented placeholder diagnostic.

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

### WP-A4 Establish the production 3D world path

Implement §11 world conversion, orthographic camera/viewport, chunked stepped
terrain mesh, fixed lighting/material, 3D survey reticle, cell/edge/device
picking, selection outline, placement ghost shell, 2D HUD restoration, dirty
chunk rebuild, and deterministic capture framing. Use procedural 3D placeholders
until authored models arrive; do not retain the template's 2D grid as gameplay.

Acceptance: §11 terrain, camera, picking, resize, render-order, placeholder, and
hash-isolation tests pass on an 8x8 stepped fixture. The first foundation capture
must visibly be a 3D diorama, not a flat grid with height colors.

### Milestone A commit

Commit after A0–A4 and run the publisher. Do not begin fluid rules until this
commit is clean.

## 3. Milestone B — Terrain and material simulation

### WP-B1 Terrain editing and surface flow

Implement terrain actions, placement validation, mixture storage, proposal/
limit/apply phases, source/drain fixtures, overlays' derived facts, and mass
ledger per §§03.2–03.4. Connect terrain mutation to dirty 3D chunk rebuilds and
world-space grade/flow overlays without putting simulation state in meshes.

Acceptance: downhill, equal-head, competing-neighbor, ridge/notch, capacity,
boundary, source-backpressure, and terrain-edit tests pass.

### WP-B2 Steam, heat, and interactions

Implement airborne diffusion, heat exchange, boiling/condensation, cooling lava,
water/lava, lava/slurry, dilution, pending terrain products, and event emission.
Implement §11 liquid top/side chunk meshes, animated material shaders, steam
volumes/billboards, flow decals, and reaction effects as derived presentation.

Acceptance: exact table-driven reaction quantities at 1, 120, 250, 1,000, and
capacity-limited `vU`; phase transitions; no-reaction pairs; deterministic hash.

### WP-B3 All-fluid laboratory

Author `lab_fluids_all`, automation, assertions, live laboratory HUD, evidence
capture, and headless scenario. Reserved bays remain explicit.

Acceptance: every §05.2 slice assertion passes, lab reset is exact, mass balance
holds, automatic/free modes work through 3D picking, and opposing-yaw captures
are readable at 1280x720.

### Milestone B commit

Commit terrain/fluids/laboratory together after the reference hash is stable;
run the publisher and record its test/build/capture output.

## 4. Milestone C — Planning and device framework

### WP-C1 Build plan UX and device runtime

Implement build registry/palette, ghost/rotation, validity reasons, queued plan,
budget reservation, commit, selection/inspect, common device entity state,
network connections, and powered-device phase.

All hover, selection, edge choice, footprint, rotation, ports, and ghosts use
the production 3D ray-hit path and depth-tested world presentation.

Acceptance: all command rejection codes, footprint rotations, overlap, budget,
queue removal, atomic commit, and renderer-independent state tests pass.

### WP-C2 Simple flow devices

Implement channel, floodgate, spillway, and reservoir behavior/settings plus
their 3D models/state visuals. Author four 3D showcase maps with production
picking, automation, assertions, and captures.

Acceptance: VM-CHANNEL, VM-FLOODGATE, VM-RESERVOIR, and VM-SPILLWAY pass.

### WP-C3 Networks and active devices

Implement pipe connectivity/capacity/pressure, topology meshes, pump, turbine
power, sensor sample/delayed output, filter, rune relay, and their 3D model/state
visuals. Author six 3D showcase maps.

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

Finish the §11 slice-quality 3D art/material pass, camera/cursor, occlusion fade,
build palette, inspector, objective/time/budget strip, alerts, 3D overlays plus
legends, pause menu, debrief, UI scale, reduced motion, and clickable paths.

Acceptance: layouts, 3D captures, picking, and controls pass §09 resolutions and
all yaw/zoom cases; no action is keyboard-only; non-color cues exist; camera,
animation, and mesh rebuilds never change simulation hash.

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
WebGL sessions, all-yaw picking, 3D draw/mesh budgets, rapid camera/pause/speed/
placement input, and one-hour simulated soak.

### WP-F3 Release evidence and handoff

Refresh all captures, verify catalog thumbnail is a title-screen capture, update
README status, run `publish.ps1`, and record shipped/deferred scope against §01.

### Milestone F commit

Commit the completion audit only when all §09 gates pass and no required work
package remains. The body must state exact publisher results and honest deferrals.

## 8. Prohibited shortcuts

- Do not special-case campaign coordinates in Rust.
- Do not implement a 2D map as a temporary gameplay path; start with the shared
  orthographic 3D camera, terrain mesh, and picking fixture in Milestone A.
- Do not make visual particles authoritative material.
- Do not advance tutorials from prompt dismissal when an action/event is required.
- Do not make reference tests assert only “mission completed”; assert resources,
  terminal tick range, objective state, mass ledger, and final hash.
- Do not create a generic script engine for ten fixed slice device behaviors.
- Do not satisfy a showcase with screenshots alone; it is an executable fixture.
- Do not mark a work package complete on `cargo check` when its milestone requires
  publisher validation.
