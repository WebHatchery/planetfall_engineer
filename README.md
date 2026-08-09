# Planetfall Engineer

Playable vertical slice for a Rust + Macroquad planetary-engineering puzzle
game. Launch opens a title screen with a new-campaign route, campaign board,
and verification grounds for the all-fluid laboratory and every device bay. A
verification visit launched from the title returns to that board; a visit
launched from a campaign preserves and returns to the current campaign session.
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
- Floodgates now control their authored surface edge at 0%, 50%, and 100%;
  `J/K/H` set the selected gate, and the inspector readout shows setting and
  active state alongside queued-plan budget.
- Authored campaign sources are fixed-rate and deterministic: L01 starts with
  its 20 `vU`/tick west-side meltwater source stopped, and tutorial completion
  or skip enables it after the player has shaped the route.
- Deterministic placement validation for bounds, overlap, protected terrain,
  budget, and rotation, plus renderer-independent device showcase fixtures.
- Build-plan queue with budget reservation, atomic commit, cancellation, and
  renderer-independent queued-plan state. Runtime controls: `B/P/O` queue a
  channel/pipe/pump, `Enter` commits, `Backspace` cancels, `C` removes the
  selected device, and `F2` opens the ten-device showcase browser.

## Phase 4/5 Mission Slice

- Mission phases (`Briefing`, `Active`, `Success`, `Failure`, `Debrief`) with
  stability windows, terminal-state guards, checkpoints, failure notices, and
  campaign unlock progression for exactly L01, L02, and L03.
- Event-driven L01 tutorial state with twelve ordered steps, stable
  `tutorial_locked` admissions, skip behavior, saved progress fields, and
  deterministic tutorial tests.
- The active L01 step is rendered as a readable field prompt with the required
  action, so a new player can follow the authored path without external notes.
- Camera changes, selection, inspection, time controls, terrain edits, device
  queues, plan commits, and floodgate settings pass through tutorial
  admission, so the playable path cannot bypass locked L01 steps.
- The guided path has the player raise three runoff barriers before enabling
  L01's authored source; skip leaves the puzzle untouched but unlocks all tools.
- Runtime controls: `F3` shows the campaign sequence, `F4` skips the tutorial,
  `F6` records a checkpoint, `F7` exercises failure recovery, and `F8` reports
  the ten-scenario deterministic campaign suite. `F12` restores the last checkpoint or restarts
  the current authored mission when no checkpoint exists. `F10` loads the
  current campaign's deterministic reference-material fixture.
- `N` selects the next unlocked authored campaign level, rebuilding its map and
  camera while preserving campaign unlocks and best-time progress.

Campaign constructors now author the exact L01/L02/L03 map sizes, budgets,
ambient temperatures, protected zones, basins/shelves, and reference tick
ranges from `docs/06_FIRST_THREE_LEVELS.md`. A clean launch opens authored L01
(`32×20`); L02 and L03 remain campaign progression content rather than generic
template maps. L02/L03 campaign starts enable their authored sources, and L03
begins with its documented water/lava reaction pocket so the live campaign can
be observed without replay-only material injection.

Save schema version 3 includes simulation, device, mission/tutorial,
checkpoint, and campaign progression state in the deterministic hash. F5/F9
round-trip tests prove that mission progress and best completion data survive
save/load. Loading also rejects malformed data, unsupported future schema
versions, installed-content version mismatches, and world/simulation dimension
mismatches with actionable recovery notices while leaving the live session
untouched.

The campaign replay harness runs L01, L02, and L03 from authored constructors,
checks terminal stability, resumes from a midpoint snapshot, and compares the
uninterrupted/resumed final hashes. It now covers reference and alternate
success routes, all three authored hazard failures, and the L03 insufficient-
water recovery state. `F11` reports the complete ten-scenario result.
The midpoint is serialized and deserialized through serde before continuation,
so the replay gate exercises the same persistence shape as a real checkpoint.
Campaign and device reports now also require exact material conservation across
surface, airborne, and pending terrain products; the authoritative balance
error is exposed by the simulation and covered by reaction tests.
L03 objective progress now reports formed firebreak products rather than the
water input, matching its authored reaction objective and reference replay.
Formed rock and vitrified terrain remain in the authoritative material ledger
after they raise the terrain, so the reference balance remains exactly zero.

The showcase index names exactly one executable map for each enabled device:
`device_channel`, `device_pipe`, `device_pump`, `device_floodgate`,
`device_reservoir`, `device_spillway`, `device_flow_turbine`, `device_sensor`,
`device_filter`, and `device_rune_relay`.

The embedded `content_registry.json` now validates the four enabled fluids,
reserved fluid IDs, ten device/showcase references, three campaign map sizes
and budgets, the twelve-step tutorial namespace, and all fourteen authored map
IDs before the game starts.

## 3D Vertical-Slice Presentation

The orthographic world now renders authoritative surface depth, airborne steam,
device footprints, active/inactive machine state, and selected-device outlines
as derived depth-tested geometry. Camera movement, quarter-turn rotation,
survey selection by keyboard or mouse ray, terrain edits, materials, and device
placement remain on the same production 3D path. The shared toolkit owns
viewport-aware screen rays and AABB hits.

The live HUD now identifies the authored campaign level and mission phase and
shows objective progress plus the current tutorial step alongside the selected
cell and device readout. The engineering readout explicitly reports selected
surface depth, material heat, contamination, and device status.
It also names the deterministic alert level (`CLEAR`, `ADVISORY`, `WARNING`,
or `CRITICAL`) so forecast pressure and hazard escalation are readable without
depending on color alone. `Y` cycles terrain-grade, flow, heat, and
contamination overlays derived from the authoritative simulation; the active
overlay is named in the HUD and uses the same orthographic world path as play.
The authored L02 source rises from 100 to 400 `vU`/tick at simulation tick 900
and returns at 1,200; L03 rises from 100 to 220 at tick 1,000 and returns at
1,250. These events run in the same authoritative tick path as the replay
harness.

Terminal success and failure now open a readable debrief/recovery panel with
the objective result, failure reason when present, and the valid next action.
Authored hazard timers now enforce the L01 protected-beacon, L02 camp-zone,
and L03 ancient-foundation failure predicates, with failure taking precedence
over same-tick success.
Escape opens a pause menu that freezes commands and simulation while exposing
resume, save, load, and mission-reset actions; the `pause_menu` capture records
the production overlay.
The `failure_recovery` capture scene exercises the terminal recovery panel and
is stored with the campaign and showcase evidence.
The verification suite also runs a 64×48 four-material, 10,000-tick synthetic
soak twice and requires identical final hashes with zero material-balance error.

Release evidence is captured under `docs/verification/` at 1280×720,
1024×768, and 800×600. `catalog_thumbnail.png` is a current title-screen
capture and is included by `publish.ps1`. The capture harness accepts the
`lab_fluids_all` scene name to render the automatic laboratory through the same
orthographic 3D world path; F1 returns to the campaign during free play.
Capture scenes named `device_channel` through `device_rune_relay` seed the
corresponding executable showcase; F2 opens the channel showcase in free play
and V cycles the ten authored device maps without losing the campaign session.
The HUD build palette lists all ten placeables and its mouse hit regions queue
the same validated plans as the keyboard bindings, including visible rejection
notices for tutorial, budget, bounds, protected-cell, and overlap failures.
The selected build has a translucent 3D footprint ghost; `Z` cycles its quarter
turn rotation, and the queued plan carries that rotation into the authoritative
device entity and directional behavior. The readout shows its cost and
`READY`/`BLOCKED` reason for bounds, protection, overlap, or budget before queue.
Visible `ROTATE`, `COMMIT`, and `CANCEL` buttons mirror the keyboard actions so
the core placement loop is fully mouse-reachable.
Visible `PAUSE`, `1X`, `2X`, and `4X` buttons likewise drive the fixed-tick time
controls at the reference and 800×600 viewport sizes.
Verification maps expose their current PASS/FAIL result, exact balance error,
and compact injected/drained/reacted/product ledger in the readout. F6 resets
the active lab or device fixture and F8 advances exactly one verification tick
while in verification mode,
making the test maps inspectable without relying on an automatic run.

The verification set also includes opposing-quarter camera captures and a
flow-overlay capture, proving that the field remains readable under camera and
presentation changes.

The initial orthographic zoom is derived from the authored map dimensions, so
the 32×20 L01 world remains fully framed at the required viewport sizes.

The template avoids browser-incompatible filesystem access. Static data is
embedded with `include_str!()`, runtime browser assets go through Macroquad or
toolkit async loaders, and save data uses macroquad-toolkit persistence.

## Run

```powershell
cargo run
```

Start a new campaign to enter L01's guided onboarding and its short field
briefing. Every stage begins with a readable mission brief and a visible Begin
Operation control. The title-screen
campaign board shows which of L01–L03 is unlocked, while Verification Grounds
opens the laboratory and all ten device showcases without disturbing a campaign
in progress.

## Test

```powershell
cargo test
```

The current repository is a playable vertical slice with authored campaign
missions, interactive tutorial flow, deterministic test maps, replay evidence,
and a Windows/WebGL publisher path. Further content expansion remains outside
this slice.
