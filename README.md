# Planetfall Engineer

Phase 0 building site for a Rust + Macroquad planetary-engineering puzzle game.
The project uses `macroquad-toolkit` for the shared runtime, UI, persistence,
camera, and data-loading patterns used by the Rust games in this workspace.

## Technical Build Specification

Implementation scope, deterministic simulation rules, content contracts,
verification maps, the first three campaign levels, interactive tutorial, work
packages, and release gates are indexed in [`docs/README.md`](docs/README.md).
Those documents are the engineering contract; the GDD remains the source of
product intent.

## Foundation Already Wired

- Ashfall Basin survey grid with selectable terrain tiles.
- Data-driven survey, excavation, and spillway actions.
- Toolkit camera, notifications, mission save/load, and migration scaffolding.
- Placeholder art and UI language ready for fluid routing and terrain simulation.

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

The next foundation pass should replace the action economy with explicit
terrain, fluid, and structure state while keeping the existing toolkit seams.
