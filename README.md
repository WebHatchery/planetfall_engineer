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
