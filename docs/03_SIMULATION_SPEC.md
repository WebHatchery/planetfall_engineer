# Simulation Specification

## 1. Authority and units

This document defines gameplay physics for the first slice. It is deliberately
an engineering model, not scientific fluid dynamics.

The cell grid is rendered as genuine 3D. Simulation `x/y` map to world `X/Z`,
and integer elevation maps to world `Y` exactly as specified in §11. Rendering,
mesh interpolation, particles, and camera state never feed back into these rules.

| Quantity | Authoritative representation | Meaning |
| --- | --- | --- |
| Position | `CellPos { x: i16, y: i16 }` | Zero-based square grid, origin at top-left |
| Elevation | `i16 height_mu` | 1,000 height units (`hU`) = one visible height step |
| Material | `u32 volume_mu` | 1,000 volume units (`vU`) = one cell at one-step depth |
| Temperature | `i32 temp_dk` | Decikelvin; display conversion is presentation only |
| Pressure | `u32 pressure_pu` | Abstract pressure units, valid only in networks/storage |
| Contamination | `u16 contamination_bp` | Basis points, 0 to 10,000 |
| Time | `u64 tick` | 10 ticks per simulated second |
| Cost | `u32 credits` | Whole mission credits |

All multiplication MUST widen before division. Integer division rounds toward
zero. Residual quantity remains at the source; it is never discarded.

## 2. World cell

Each cell contains immutable authoring flags and mutable simulation state:

```text
CellDefinition
  base_height_hu: i16
  terrain_material_id: string
  protected: bool
  source_id: optional string
  objective_zone_ids: [string]

CellState
  height_hu: i16
  sealed: bool
  reinforcement: 0..10000
  ground_contamination_bp: 0..10000
  surface: sorted list of (fluid_id, volume_vu, temp_dk, contamination_bp)
  airborne: sorted list of (fluid_id, volume_vu, temp_dk)
  pending_rock_vu: u32
  pending_vitrified_vu: u32
  device_entity_id: optional u32
  queued_plan_ids: sorted list of u32
```

Surface and airborne lists are sorted by fluid ID after every tick. Slice
content caps each phase at four entries. Total volume in either phase MUST NOT
exceed its `8,000 vU` cell capacity; excess surface material is routed during
flow proposal and excess steam remains blocked at its source. A map boundary is
a solid wall unless an authored `drain` source/sink marks that cell.

Derived values are recalculated, not saved:

```text
surface_head_hu = height_hu + total_surface_volume_vu
depth_hu        = total_surface_volume_vu
free_capacity   = 8000 - total_surface_volume_vu
```

Because `hU` and `vU` share the 1,000-per-step scale, head comparison is exact.

## 3. Terrain actions

Terrain commands target one cell and are rejected on protected or occupied
cells unless the device explicitly supports the edit.

| Action | Cost | Result on commit |
| --- | ---: | --- |
| Excavate | 4 | `height_hu -= 250`, minimum authored map floor |
| Raise | 5 | `height_hu += 250`, maximum authored map ceiling |
| Seal | 3 | sets `sealed = true` |

The slice does not simulate erosion or infiltration. Sealing matters to device
placement, objectives, and contamination containment: toxic slurry occupying
an unsealed cell sets that cell's `contamination_bp` to at least the slurry's
concentration and remains contaminated after the fluid leaves. A filter or
level-authored restoration zone may clear it; ordinary water does not.

Terrain edits are instantaneous when a plan is committed. A committed raise is
rejected if the resulting surface volume would exceed capacity; the player must
drain the cell first. Chemical rock production may raise occupied terrain and
routes displaced excess through normal overflow on the next tick.

## 4. Surface flow

Surface fluid flows only through cardinal neighbors. For each adjacent pair,
compute one proposal from the tick phase's immutable start snapshot:

```text
delta_hu = source.surface_head_hu - destination.surface_head_hu
```

No proposal is made when `delta_hu <= 1`, a device/wall blocks the edge, or the
destination has no capacity. Otherwise:

```text
raw_vu = delta_hu / 4
limit_vu = min(source total volume, destination free capacity,
               source mixture max_transfer_vu)
proposal_vu = min(raw_vu, limit_vu)
```

When a source proposes to multiple lower neighbors, allocate its total allowed
outflow proportional to each positive `delta_hu`; distribute integer remainder
by neighbor order North, East, South, West. The transferred mixture uses the
source's material volume proportions; remainder uses ascending fluid ID.

`source mixture max_transfer_vu` is the lowest transfer limit among materials
present at the source. This prevents a trace of water from making slurry flow
like water. Initial limits per tick:

| Fluid | Phase | Max transfer `vU` | Default temperature | Contamination |
| --- | --- | ---: | ---: | ---: |
| `water` | surface | 400 | 2,930 dK | 0 bp |
| `lava` | surface | 120 | 12,730 dK | 0 bp |
| `toxic_slurry` | surface | 180 | 3,000 dK | 10,000 bp |
| `steam` | airborne | 300 | 3,930 dK | n/a |

Authored sources inject after scheduled events and before flow proposals. If an
injection would exceed capacity, inject to capacity and emit
`source_backpressure`; do not delete the remainder or accumulate source debt.
The mass ledger counts only material actually injected into the world.
Authored drains remove up to their rate at the post-transfer phase.

## 5. Airborne steam

Airborne material ignores terrain height but not cells flagged `gas_blocked`.
Steam proposes transfer to cardinal neighbors whose steam volume is at least
two units lower. It uses the same proportional competing-transfer algorithm
with `delta = source steam volume - destination steam volume` and
`raw_vu = delta / 5`.

After transfer, each steam entry exchanges heat with that cell's terrain toward
the terrain's authored ambient temperature by `abs(delta_temp) / 20` per tick.
At or below `WATER_BOIL_DK = 3730`, up to `200 vU` steam per tick condenses 1:1
into surface water, limited by surface capacity. Blocked condensate remains
steam. Steam above 6,000 `vU` emits a high-pressure alert; the slice has no
steam explosion failure unless a mission declares that boundary.

## 6. Surface heat exchange

After interactions, every surface material temperature moves toward the cell's
volume-weighted surface average by `abs(delta_temp) / 16`, then toward ambient
by `abs(delta_temp) / 100`. Minimum movement is one dK when delta is nonzero.
Heat exchange changes temperature only; material conversion occurs through the
explicit rules below.

Water at or above 3,730 dK converts to steam at up to `150 vU` per tick, 1:1.
Lava below 9,000 dK converts to `basalt` terrain at up to `80 vU` per tick. Each
1,000 `vU` accumulated rock product raises the cell by 1,000 `hU`; sub-step
rock product is stored in the cell snapshot as `pending_rock_vu`.

## 7. Material interactions

Interactions operate in row-major cell order after transfers and before heat
exchange. Each interaction consumes at most its per-tick limit. Reactants and
products are integer and deterministic.

### SI-01 Water and lava

```text
reaction_vu = min(water_vu, lava_vu, 250)
consume reaction_vu water
consume reaction_vu lava
produce reaction_vu steam at 4,730 dK
add reaction_vu to pending_rock_vu
```

Every reaction emits `material_reacted(water_lava, cell, reaction_vu)`. It also
reduces remaining lava temperature by 300 dK, minimum 9,000 dK, and raises
remaining water temperature by 300 dK, maximum 3,730 dK.

### SI-02 Lava and toxic slurry

```text
reaction_vu = min(lava_vu, toxic_slurry_vu, 120)
consume reaction_vu lava
consume reaction_vu toxic_slurry
add reaction_vu * 2 to pending_vitrified_vu
reduce cell contamination by reaction_vu * 5 bp, saturating at zero
```

Each accumulated 1,000 `pending_vitrified_vu` raises terrain by 500 `hU` and
sets its material to `vitrified_slag`. This emits
`material_reacted(slurry_vitrified, ...)` and creates no airborne contamination.

No other slice fluid pair reacts. Water dilutes slurry without changing either
fluid identity: `new_slurry_bp = old_slurry_bp * slurry_vu /
(slurry_vu + water_vu)`. Water contamination remains unchanged.

## 8. Device contract

Every device has an immutable definition and an entity state containing stable
ID, anchor, rotation, health, power state, settings, internal material, and
network connections. Devices run by ascending entity ID.

| ID | Footprint | Cost | Required slice behavior |
| --- | --- | ---: | --- |
| `channel` | 1 cell | 2 | Passable surface cell; placement excavates 100 `hU`; transfer cap +150 |
| `pipe` | 1 cell | 3 | Connects N/E/S/W pipe network; sealed from surface; 600 `vU` capacity |
| `pump` | 1 cell | 12 | Moves up to 250 `vU`/tick inlet -> outlet; +100 `pU`; toggle and target rate |
| `floodgate` | 1 edge | 8 | Open fraction 0/25/50/75/100%; closed blocks surface flow |
| `reservoir` | 2x2 | 16 | Stores 8,000 `vU`; accepts/releases through configured connection |
| `spillway` | 1 edge | 6 | One-way surface transfer up to 500 `vU` above 2,000 `vU` upstream depth |
| `flow_turbine` | 1 cell | 14 | Water/steam flow generates one power per 100 `vU` passing, max 4/tick |
| `sensor` | 1 cell | 5 | Samples adjacent target; threshold toggles one linked device next tick |
| `filter` | 1 cell | 14 | Moves 160 `vU`/tick; removes 2,500 bp contamination per pass |
| `rune_relay` | 2x2 | 20 | Activates while receiving 2 power/tick and adjacent flow >=100 `vU`/tick |

Devices that transfer material propose in phase 4 and are limited with all
other proposals. They cannot create volume. A transfer requiring power reserves
its authored power before proposing; unavailable power produces no transfer.
The slice uses abstract map-wide power supply/demand, recalculated each tick;
power cables and batteries are out of scope.

Pipe networks hold one compatible surface fluid mixture. Connecting a different
fluid is allowed; interaction occurs at receiving cells, not invisibly inside a
pipe. Pressure is `min(1000, filled_capacity_ratio_bp / 10 + pump_added_pu)`.
At pressure over the pipe rating, the device warns but does not rupture in the
slice unless a mission explicitly enables rupture.

## 9. Objectives and terminal state

Objective predicates inspect post-tick state only. Types required in the slice:

- `zone_volume`: material volume in zone at/above threshold.
- `zone_flow`: material transfer across marked boundary in rolling 10 ticks.
- `zone_contamination`: maximum/average basis points below threshold.
- `device_active`: named authored device/entity operating.
- `protected_integrity`: no protected cell exceeded material/heat limits.
- `tutorial_complete`: all mandatory tutorial steps complete.

A primary objective begins its stability counter when all mandatory predicates
are true, resets to zero on any false tick, and completes at its authored tick
target. Failure predicates are evaluated before success in the same tick.
After terminal state, no simulation ticks or gameplay commands are admitted.

## 10. Required invariants

Tests MUST assert after every simulated tick:

- positions and entity references are valid;
- volumes, costs, pressure, and contamination remain in range;
- material consumed plus stored plus drained plus products matches authored
  sources within the explicit reaction conversion table;
- no cell exceeds capacity after overflow has had one tick to resolve;
- no device occupies an invalid footprint;
- objective counters match their predicates;
- re-running the same definition, seed, and commands yields the same hash.
