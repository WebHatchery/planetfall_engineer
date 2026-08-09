# Planetfall Engineer Technical Documentation

This directory is the implementation contract for the first playable slice of
*Planetfall Engineer*. It translates the design intent in
[`../Planetfall_Engineer_GDD.md`](../Planetfall_Engineer_GDD.md) into bounded,
testable engineering work.

## Authority and vocabulary

When documents disagree, use this order:

1. `AGENTS.md` and `CODE_STANDARDS.md` for repository rules.
2. These technical documents for implementation behavior and acceptance.
3. `Planetfall_Engineer_GDD.md` for product intent and later-game direction.
4. The current template code only as a reusable starting point.

`MUST`, `MUST NOT`, `SHOULD`, and `MAY` are normative. A task is not complete
until every linked acceptance criterion passes. IDs are stable references for
code, tests, map data, commits, and later issue tracking.

## Document map

| Document | Purpose |
| --- | --- |
| [01_PROJECT_SCOPE.md](01_PROJECT_SCOPE.md) | In/out boundaries, slice deliverables, constraints, and definition of done |
| [02_TECHNICAL_ARCHITECTURE.md](02_TECHNICAL_ARCHITECTURE.md) | Runtime ownership, module boundaries, update order, determinism, and saves |
| [03_SIMULATION_SPEC.md](03_SIMULATION_SPEC.md) | Terrain, fabrication, power, fluids, devices, and exact tick rules |
| [04_CONTENT_FORMATS.md](04_CONTENT_FORMATS.md) | Data contracts, stable IDs, validation, and mission authoring |
| [05_VERIFICATION_MAPS.md](05_VERIFICATION_MAPS.md) | Fluid/economy laboratories and one isolated map per device |
| [06_FIRST_THREE_LEVELS.md](06_FIRST_THREE_LEVELS.md) | Resource/power-constrained content and acceptance for L01–L03 |
| [07_INTERACTIVE_TUTORIAL.md](07_INTERACTIVE_TUTORIAL.md) | Event-driven L01 fabrication and L02 power onboarding |
| [08_IMPLEMENTATION_PLAN.md](08_IMPLEMENTATION_PLAN.md) | Ordered engineer-ready work packages and milestone commits |
| [09_QUALITY_GATES.md](09_QUALITY_GATES.md) | Automated, capture, performance, and publisher release gates |
| [10_DELIVERY_ROADMAP.md](10_DELIVERY_ROADMAP.md) | Whole-project release boundaries and post-slice entry criteria |
| [11_3D_VISUAL_TECHNICAL_SPEC.md](11_3D_VISUAL_TECHNICAL_SPEC.md) | From-start orthographic 3D rendering, assets, picking, and visual acceptance |

## Slice decisions already locked

- The authoritative world is a deterministic square-cell heightfield rendered
  as genuine orthographic 3D from the first foundation milestone. Terrain,
  fluids, machines, placement, picking, verification maps, and campaign maps
  MUST use the 3D path in `11_3D_VISUAL_TECHNICAL_SPEC.md`; presentation still
  MUST NOT own simulation state.
- The first slice contains exactly three campaign levels: `L01`, `L02`, and
  `L03`. Sandbox laboratories are verification content, not campaign levels.
- Slice fluids are water, lava, steam, and toxic slurry. Steam is represented
  as a cell material despite being gaseous. The remaining GDD materials keep
  reserved IDs and join the all-fluid laboratory when implemented.
- Slice placeables are channel, pipe, pump, floodgate, reservoir, spillway,
  flow turbine, sensor, filter, and rune relay. Terrain excavation, raising,
  and sealing are tools, not placeable devices.
- R0 construction is constrained by finite `fabU` fabrication stock recovered
  directly from authored planetary deposits. Recovery is a map-wide field-tool
  action, not a worker, hauling, crafting, factory, or production-chain system.
- R0 powered devices draw deterministic `eU` from one mission-wide field grid.
  Authored geothermal fixtures and flow turbines are the only slice producers;
  cables, batteries, fuel logistics, population needs, and base building remain
  outside the slice.
- Simulation advances only in fixed ticks. Frame rate and fast-forward change
  how many ticks execute, never the result of an individual tick.
- No avatar is introduced. “Move around” in onboarding means pan, zoom, and
  rotate the 3D engineering camera plus move the survey cursor, matching the
  GDD player role.

## Change discipline

Any behavior change MUST update its owning document in the same commit. New
fluids require a fluid definition, pairwise interaction tests, and an update to
`lab_fluids_all`. New devices require a device definition, an isolated showcase
map, and map acceptance. New campaign mechanics require a first-use tutorial or
an explicit statement that they are intentionally discoverable.
Changes to fabrication or power require exact ledger rules, updates to
`lab_field_economy`, affected device showcases, every campaign reference stream,
save/content versions, and a first-use onboarding event.

## Branch and commit discipline

- `master` is the canonical branch for this project.
- Major updates MUST be committed as focused, complete changes before the next
  major update begins.
- Each commit MUST include the owning documentation and its verification
  evidence when behavior or scope changes.
