# Whole-Project Delivery Roadmap

## 1. Purpose

This roadmap bounds the complete product described by the GDD while keeping the
current engineering queue unambiguous. Only Release 0 is implementation-ready
from these documents. Later releases are authorized scope envelopes, not a
license to invent missing level content or mechanics in code.

## 2. Release boundaries

| Release | Required product result | Engineering entry gate |
| --- | --- | --- |
| R0 — First playable slice | L01–L03, orthographic 3D world, four-fluid simulation, finite planet-sourced fabrication, planetary power, ten 3D devices, tutorial, verification suite | Current `docs/` contract approved |
| R1 — Volcanic MVP | 6–8 total Act I missions, grading, polished volcanic presentation/audio, complete accessibility/save/restart | R0 quality gates pass; each added mission has an authored spec and replay |
| R2 — Frozen Archive | Act II content, brine and cryofluid, freeze/thaw/storage rules, cold-rated equipment | R1 stable; exact fluid matrices and device deltas accepted |
| R3 — Poisoned Inheritance | Act III content, acid and expanded contamination/neutralization/corrosion | R2 stable; corrosion and purification specs accepted |
| R4 — Drowned Engines | Act IV content, nutrient solution, ecology targets, large/tidal reservoirs | R3 stable; tide and ecology state models accepted |
| R5 — Glyphs of Genesis | Act V content, aetheric condensate, rune networks, final multi-stage restoration | R4 stable; resonance rules and final mission specs accepted |
| R6 — Launch hardening | 24–30 principal missions total, 8–12 optional contracts if authored, balance/accessibility/localization/release audit | All acts complete; no unresolved simulation contract |
| Post-launch candidates | Editor, modding, larger sandbox, co-op evaluation | Explicit new scope; never required for launch |

R1 mission count includes the three R0 levels. The GDD's suggested launch range
is a content ceiling/target, not permission to create placeholder missions. A
release may not enter implementation until its mechanics and authored missions
have the same specificity as R0.

## 3. Technical work that carries through every release

The following contracts are permanent unless a versioned architecture decision
replaces them:

- deterministic heightfield simulation and fixed-tick command replay;
- finite mission-local construction stock and deterministic planet-sourced
  power without colony, worker, hauling, or production-chain simulation;
- genuine orthographic 3D terrain, fluids, devices, picking, placement, and map
  captures from R0 onward, separated from authoritative simulation state;
- data-authored maps, definitions, events, tutorials, and objectives;
- versioned saves/content plus deterministic migrations or checkpoint fallback;
- one permanent all-fluid laboratory and one isolated showcase per device;
- native Windows and WebGL publisher validation;
- module/source-size and macroquad-toolkit reuse rules.

Later presentation may add higher-detail materials, models, shadows, weather,
or a cinematic perspective camera, but the product does not defer its 3D world.
Every renderer change MUST consume existing snapshots/commands and pass the same
replays; scene meshes never own fluid, device, or objective truth.

## 4. Fluid expansion gate

Before enabling any reserved fluid, an engineer requires an approved technical
delta containing all of:

1. phase, integer units, flow/transport limit, temperature and contamination;
2. behavior on level, sloped, sealed, unsealed, hot, and cold terrain;
3. explicit reaction or non-reaction with every enabled fluid;
4. device compatibility/rating changes and player-readable warning states;
5. source, drain, objective, save, overlay, inspect, and alert representation;
6. activated `lab_fluids_all` bay, central interaction lanes, assertions, replay,
   reference hash, and capture;
7. campaign first-use mission and interactive onboarding event, if player-facing.

The GDD's one-line interaction descriptions are intent, not sufficient numeric
implementation rules. Missing values block the fluid; an agent MUST NOT choose
them silently.

## 5. Device expansion gate

Before enabling a new device, its technical delta MUST specify stable ID,
category, footprint, rotations, placement surfaces, cost/unlock, input/output
ports, capacity/rate, settings, power, material ratings, failure behavior,
inspect fields, overlay cues, save state, deterministic tick phase, and every
rejection reason. The same change adds unit tests and `device_<id>` per §05.

No device is added solely to satisfy a category count. It must solve an authored
mission problem and be understandable in its isolated showcase.

## 6. Mission expansion gate

Every post-slice campaign mission requires, before code or map production:

- ID, sequence, learning purpose, prior knowledge, and new concepts;
- exact size, sources, zones, protected boundaries, authored devices,
  fabrication deposits/starting stock, power sources/demand, build kit, terrain
  actions, events/warnings, and time controls;
- numeric primary/optional/failure objectives and stability windows;
- reference solution with expected resources and terminal tick range;
- soft/hard recovery and checkpoint behavior;
- first-use tutorial triggers for new player-facing mechanics;
- deterministic replay assertions and required evidence captures.

A prose theme such as “ice mission” is not an implementable mission. If any
field is missing, the work remains product specification, not engineering.

## 7. Launch scope exclusions

Unless separately approved, launch excludes multiplayer, public editor, deep
mod API, procedural world generation, live-service contracts, daily challenges,
online accounts, user-generated-content hosting, and complex autonomous ecology.
The architecture should avoid needless obstruction but MUST NOT build hooks,
servers, schemas, or UI for those candidates during R0–R6.

## 8. Project completion definition

The full launch project is complete when R0–R5 behavior/content is shipped,
the final authored mission count lies within the approved 24–30 range, the
campaign introduces each enabled fluid/device through a specified mission, all
laboratory/showcase/reference replays pass, save migrations cover supported
published versions, Windows/WebGL publishers pass, accessibility/localization
checks pass, and the R6 release audit records no unresolved launch blocker.
