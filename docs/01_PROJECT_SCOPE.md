# Project Scope: First Playable Slice

## 1. Objective

Deliver a publishable three-level playable slice proving this loop:

> inspect terrain, move around the map, place an engineering intervention,
> run time, understand its visible effect, adapt, and hold a stable result.

The slice is an engineering proof, not the full MVP from GDD §25. It MUST be
complete enough that a new player can finish all three levels without external
instructions and an engineer can add a fluid or device through data plus a
bounded implementation module.

## 2. Required deliverables

### PS-01 Runtime foundation

- Native Windows and WebGL builds through `publish.ps1`.
- Deterministic fixed-tick terrain/fluid/device simulation.
- Top-down heightfield map, camera pan/zoom, survey cursor, inspect selection,
  pause, 1x, 2x, and 4x time controls.
- Data-loaded fluids, devices, interactions, tutorials, and missions.
- Save/load of campaign progress and active mission state.
- Restart mission and reset-to-last-checkpoint actions.

### PS-02 Player-facing systems

- Terrain-grade, flow/depth, heat, contamination, and device-status overlays.
- Build palette, ghost placement, validity reasons, cost preview, rotate/cancel,
  and queued placement while paused.
- Objective strip with live progress and a visible stability timer.
- Four alert levels with color-independent icon and text treatment.
- Briefing, tutorial prompts, pause menu, success debrief, and failure/recovery.

### PS-03 Simulation content

- Four slice fluids: water, lava, steam, and toxic slurry.
- Ten slice placeables: channel, pipe, pump, floodgate, reservoir, spillway,
  flow turbine, sensor, filter, and rune relay.
- Three terrain actions: excavate, raise, and seal.
- Required interactions: downhill flow; retention; pressure transport; water +
  lava -> rock + steam; steam cooling -> water; slurry transport and filtering;
  lava/slurry -> vitrified inert ground.

### PS-04 Authored content

- First three campaign levels defined in `06_FIRST_THREE_LEVELS.md`.
- Event-driven interactive tutorial defined in `07_INTERACTIVE_TUTORIAL.md`.
- `lab_fluids_all`, containing one central interaction field and a pool/source
  bay for every implemented liquid or mobile material.
- One isolated showcase map for every placeable device. Every later device MUST
  add its showcase map in the same change.

### PS-05 Verification

- Unit and scenario tests for the tick engine and every slice interaction.
- Content-schema and cross-reference validation.
- Deterministic replay hashes for every verification and campaign map.
- Automated UI captures at 1280x720, 1024x768, and 800x600.
- Full `publish.ps1` success after meaningful implementation milestones.

## 3. Explicitly out of scope

The following MUST NOT be added to the slice unless this scope is deliberately
revised first:

- True volumetric fluid dynamics, particle-based fluid authority, or erosion.
- A free-moving player avatar, character combat, inventory, or crafting.
- Freeform mesh sculpting. Terrain edits operate on cells with integer height.
- Full 3D orbit rendering. The logic/renderer boundary must permit it later.
- Fluids beyond the four slice types. Reserved data may exist, behavior may not.
- Devices beyond the ten slice placeables.
- Missions four through eight, other biomes, procedural maps, editor, mods,
  multiplayer, online services, telemetry, achievements, or leaderboards.
- Audio production, cinematics, final art, localization content, or voice-over.
- Efficiency grading beyond pass/fail plus optional-objective flags.

## 4. Product constraints

| Constraint | Slice requirement |
| --- | --- |
| View | 16:9 reference at 1280x720; usable at 800x600 and common browser sizes |
| Input | Mouse + keyboard; every required action has a visible clickable path |
| Mission size | Up to 48x32 cells in campaign; 64x48 in verification maps |
| Simulation | 10 fixed ticks/second at 1x; tick result independent of render FPS |
| Fast-forward | 2x and 4x; no dropped or enlarged simulation ticks |
| Stability | No NaN/Inf; nonnegative volumes; mass error under quality-gate bound |
| Performance | Quality targets are specified in `09_QUALITY_GATES.md` |
| Persistence | Versioned serde payload through macroquad-toolkit save slots |
| Accessibility | UI scale, remapping-ready actions, non-color state cues, reduced motion |

## 5. Compatibility contract with the template

The current survey/action economy is scaffolding, not retained game design.
Engineers MAY replace `PlayerState.points`, regenerating energy, fog reveal, and
the generic `ActionDef` with the systems in these documents. They MUST preserve
useful shared-toolkit seams: virtual UI, camera/input helpers, notifications,
asset loading, capture mode, persistence, and source-size validation.

Before implementing a local replacement for shared runtime, camera, input,
assets, persistence, UI, or capture behavior, inspect `macroquad-toolkit`. If
the capability is generally useful across RustGames, upgrade the toolkit and
consume it here.

## 6. Slice completion criteria

The slice is complete only when all statements are true:

1. A clean save launches `L01`; the tutorial guides camera movement, survey
   cursor movement, placement, time advance, observation, and completion.
2. `L01`, `L02`, and `L03` are completable from their authored starting states
   with at least one documented reference solution each.
3. Every fluid behavior can be observed in `lab_fluids_all` and every device in
   exactly one primary showcase map without unrelated device dependencies.
4. Restarting any map produces the same initial state; replaying the same
   command stream produces the same end-state hash.
5. Invalid placement, budget exhaustion, overflow, heat, and contamination are
   explained before or when they block the player.
6. Save/load during pause and active simulation preserves state to documented
   quantization tolerances and resumes deterministically.
7. All gates in `09_QUALITY_GATES.md` pass through `publish.ps1`.
8. No touched non-test Rust file reaches 800 lines, and module ownership matches
   `02_TECHNICAL_ARCHITECTURE.md`.

## 7. Scope-change rule

A scope addition requires all of: an updated deliverable or explicit deferral,
an acceptance criterion, a data/runtime owner, verification content, estimated
work-package placement, and an explanation of what leaves scope if schedule is
fixed. Unspecified polish does not block the slice; violated behavioral or
quality contracts do.
