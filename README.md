# Planetfall Engineer

Playable foundation with the first terrain and material-simulation slice for a
Rust + Macroquad planetary-engineering puzzle game.
The project uses `macroquad-toolkit` for the shared runtime, UI, persistence,
camera, and data-loading patterns used by the Rust games in this workspace.

## Technical Build Specification

Implementation scope, from-start orthographic 3D rendering, deterministic
simulation rules, content contracts, verification maps, the first three
campaign levels, interactive tutorial, work packages, and release gates are
indexed in [`docs/README.md`](docs/README.md). Those documents are the
engineering contract; the GDD remains the source of product intent.

## Phase 1 Foundation

- Deterministic integer heightfield with stable row-major coordinates and state hashes.
- Fixed 10 Hz simulation accumulator with pause, 1x, 2x, and 4x controls.
- Genuine depth-tested orthographic 3D stepped terrain with four quarter-turn views.
- Survey cursor, camera pan/zoom, checkpoint save/load, and engineering HUD.
- Embedded configuration and toolkit asset-loading seams retained for later content.

## Phase 2 Terrain and Materials

- Integer terrain actions: excavate, raise, and seal, with protected-cell and
  fluid-capacity rejection.
- Water, lava, toxic slurry, and steam with bounded surface/airborne volume,
  downhill/diffusion transfer, source backpressure, contamination, heat, and
  water/lava and lava/slurry reactions.
- Deterministic material events, mass ledger, terrain products, and focused
  simulation tests.
- Executable `lab_fluids_all` fixture with four active bays, five reserved
  bays, interaction lanes, replay hashing, and an `F1` automation report.

## Phase 3 Device Framework

- Stable entities for all ten slice devices with costs, footprints, rotations,
  health, power, settings, storage, and active-state reporting.
- Active device behavior includes rotated pump transfer, reservoir capacity,
  spillway threshold/rate, filter contamination removal, turbine power output,
  sensor threshold state, and rune-relay flow/power gating.
- Deterministic placement validation for bounds, overlap, protected terrain,
  budget, and rotation, plus renderer-independent device showcase fixtures.
- Build-plan queue with budget reservation, atomic commit, cancellation, and
  renderer-independent queued-plan state. Runtime controls: `B/P/O` queue a
  channel/pipe/pump, `Enter` commits, `Backspace` cancels, `C` removes the
  selected device, and `F2` runs all ten showcase checks.

## Phase 4/5 Mission Slice

- Mission phases (`Briefing`, `Active`, `Success`, `Failure`, `Debrief`) with
  stability windows, terminal-state guards, checkpoints, failure notices, and
  campaign unlock progression for exactly L01, L02, and L03.
- Event-driven L01 tutorial state with twelve ordered steps, stable
  `tutorial_locked` admissions, skip behavior, saved progress fields, and
  deterministic tutorial tests.
- Runtime controls: `F3` shows the campaign sequence, `F4` skips the tutorial,
  `F6` records a checkpoint, `F7` exercises failure recovery, and `F8` reports
  a tutorial command admission. `F10` loads the current campaign's deterministic
  reference-material fixture.

Campaign constructors now author the exact L01/L02/L03 map sizes, budgets,
ambient temperatures, protected zones, basins/shelves, and reference tick
ranges from `docs/06_FIRST_THREE_LEVELS.md`. A clean launch opens authored L01
(`32×20`); L02 and L03 remain campaign progression content rather than generic
template maps.

Save schema version 3 includes simulation, device, mission/tutorial,
checkpoint, and campaign progression state in the deterministic hash. F5/F9
round-trip tests prove that mission progress and best completion data survive
save/load.

The campaign replay harness runs L01, L02, and L03 from authored constructors,
checks terminal stability, resumes from a midpoint snapshot, and compares the
uninterrupted/resumed final hashes. `F11` reports the three-reference result.

The showcase index names exactly one executable map for each enabled device:
`device_channel`, `device_pipe`, `device_pump`, `device_floodgate`,
`device_reservoir`, `device_spillway`, `device_flow_turbine`, `device_sensor`,
`device_filter`, and `device_rune_relay`.

## 3D Vertical-Slice Presentation

The orthographic world now renders authoritative surface depth, airborne steam,
device footprints, active/inactive machine state, and selected-device outlines
as derived depth-tested geometry. Camera movement, quarter-turn rotation,
survey selection by keyboard or mouse ray, terrain edits, materials, and device
placement remain on the same production 3D path. The shared toolkit owns
viewport-aware screen rays and AABB hits.

Release evidence is captured under `docs/verification/` at 1280×720,
1024×768, and 800×600. `catalog_thumbnail.png` is a current title-screen
capture and is included by `publish.ps1`.

The template avoids browser-incompatible filesystem access. Static data is
embedded with `include_str!()`, runtime browser assets go through Macroquad or
toolkit async loaders, and save data uses macroquad-toolkit persistence.

## Run

```powershell
cargo run
```

## Test

```powershell
cargo test
```

Devices, campaign missions, tutorial flow, and release evidence remain in the
subsequent implementation-plan milestones.
