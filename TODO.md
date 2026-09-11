# Planetfall Engineer TODO

This list records the follow-up work identified by the review against
`AGENTS.md`, `CODE_STANDARDS.md`, and the current technical delivery plan.
Behavior changes should update their owning document and verification evidence
in the same commit.

## Code structure and ownership

- [ ] Migrate the legacy test modules out of `src/**/tests.rs` and into the
  crate-level `tests/` directory. The affected modules are campaign, content,
  data, devices, game_build, game_terminal, game_title, game_verification,
  mission, replay, simulation, state, and verification. Add `src/lib.rs` as
  the public test seam, leave `main.rs` responsible for the executable entry
  point, and preserve the existing behavior coverage during the migration.
- [ ] Split production files that have crossed the 600-line planning threshold:
  `src/game.rs` (768 lines), `src/simulation.rs` (773), `src/ui.rs` (732),
  `src/devices.rs` (683), and `src/game_render.rs` (655). Keep each extracted
  module cohesive and use named module files.
- [ ] Break up functions over the 100-line absolute limit: `Game::update` in
  `src/game.rs` (292 lines), `draw_hud` in `src/ui.rs` (401),
  `DeviceSystem::tick` in `src/devices.rs` (239),
  `SimulationWorld::flow_surface` in `src/simulation.rs` (105),
  `ContentRegistry::validate` in `src/content.rs` (104), and
  `draw_authored_markers` in `src/game_render.rs` (106). Keep the split by
  responsibility instead of moving code only to lower a line count.
- [ ] Introduce an explicit UI action/intents layer. Rendering and pointer
  hit-testing should return actions, while one dispatcher owns mission,
  simulation, save, and verification mutations. This should consolidate the
  currently distributed input mutation in `game.rs`, `game_build.rs`,
  `game_title.rs`, `game_terminal.rs`, and `game_verification.rs`.
- [ ] Add module-level `//!` documentation to the migrated integration test
  modules and keep `tests/code_standards.rs`'s comment aligned with the current
  total-line source gate, which includes tests, comments, and whitespace.

## Data, UI, and browser behavior

- [ ] Make authored content data-driven under `assets/`. Move duplicated map
  sizes, budgets, schedules, objectives, tutorial prompts, and player-facing
  strings out of `campaign.rs`, `mission.rs`, `ui.rs`, and `ui_tutorial.rs`
  into typed JSON loaded through the toolkit. Make the content registry the
  authoritative source and extend semantic validation for IDs, references,
  dimensions, balance values, and tutorial steps.
- [ ] Audit every runtime `unwrap` and `expect` in rendering, devices,
  placement, simulation, and replay paths. Replace recoverable failures with
  `Option`/`Result` and actionable notices; document the few remaining
  invariant or startup-only failures.
- [ ] Replace ad-hoc repeated button drawing and hit-testing with shared
  `macroquad-toolkit` widgets and input helpers where they fit, adding a
  toolkit capability when the behavior is reusable across games.
- [ ] Make the pause overlay fully touch-accessible. It currently displays
  instructions for resuming, saving, loading, and resetting, but the paused
  update path returns before the visible time controls are processed and the
  overlay has no direct buttons for those actions. Add visible actions and
  logical-coordinate tests at the required viewport sizes.
- [ ] Reconcile `game_page.json` and all player-facing shortcut text with the
  actual controls and their visible touch equivalents. The metadata currently
  advertises `S / L` for save/load while the runtime uses `F5 / F9`, and it does
  not explain the visible controls used by browser players.
- [ ] Recheck every required overlay at 1280x720, 1024x768, and 800x600 after
  the UI refactor, including title selection, tutorial, build palette, pause,
  verification, debrief, recovery, and camera interactions. Keep the logical
  hit regions aligned with the rendered controls.
- [ ] Remove the exact duplicate verification capture
  `docs/verification/ui_gameplay.png`, which has the same content hash as
  `docs/verification/ui_gameplay_1280x720.png`, and keep the stable
  `<screen>_<width>x<height>.png` naming convention.
- [ ] Replace the direct `fs::read_to_string` and `serde_json::from_str` asset
  parsing in `tests/asset_registry.rs` with the toolkit loading path, or record
  an explicit test-only exception in the standards if raw inspection is
  required for this packaging check.

## Tests and release gates

- [ ] Review feature suites against the five-case target before adding more
  tests. Current suites exceed it for campaign (8), devices (14), mission
  (11), simulation (14), and state (7); consolidate related cases with
  table-driven assertions or document why distinct coverage is necessary.
- [ ] Ensure the publisher path enforces formatting, Windows and WASM builds,
  warnings-denied Clippy, `cargo test`, the total-line source gate, and content
  validation, then record a successful no-argument `publish.ps1` run at each
  completed milestone.
- [ ] Complete the pending Milestone F work in `docs/08_IMPLEMENTATION_PLAN.md`:
  finite fabrication stock and deposits (WP-F1), deterministic planetary field
  power and brownouts (WP-F2), and campaign retuning, onboarding, saves, and
  recovery evidence (WP-F3). Refresh the release captures and catalog thumbnail
  after those constraints are implemented.
