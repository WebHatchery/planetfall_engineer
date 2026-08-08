# Planetfall Engineer

Phase 1 foundation for a Rust + Macroquad planetary-engineering puzzle game.
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

Phase 2 will add explicit terrain edits, fluids, material interactions, and
verification laboratory content, followed by devices and campaign missions.
