# Verification Map Specification

## 1. Purpose

Verification maps are executable technical fixtures with a player-visible
presentation. They serve three jobs: manual behavior inspection, deterministic
scenario testing, and automated capture evidence. They do not unlock campaign
progress, award grades, or contain narrative dependencies.

Every map MUST provide:

- the production orthographic 3D terrain/fluid/device renderer and production
  ray-picking/placement path; no 2D laboratory renderer or test-only picking;
- deterministic initial state and fixed 3D camera target/yaw/zoom framing;
- a short instruction panel listing behavior and controls under test;
- labeled inputs, output/measurement zones, and expected result ranges;
- reset, pause, single-tick, 1x, 2x, and 4x controls;
- live tick, state hash, source/sink totals, and mass-balance readout;
- an automatic scenario command stream and a free-control mode;
- pass/fail assertions shown in the map and exercised headlessly;
- one stable 1280x720 evidence capture after its automatic scenario, plus
  rotation captures where another yaw exposes behavior hidden by terrain.

Verification assertions are data records interpreted by tested assertion kinds,
not arbitrary scripts. Required kinds are `zone_volume_range`,
`zone_material_present`, `zone_material_absent`, `transfer_total_range`,
`device_state`, `terrain_height_range`, `contamination_range`, `mass_balance`,
`deposit_state`, `fabrication_balance`, `power_balance`, and `state_hash`.

## 2. All-fluid laboratory: `lab_fluids_all`

### 2.1 Layout

The 64x48 stepped 3D map has a 16x16 central interaction field and nine 10x8
perimeter bays, one for every GDD fluid ID. Slice-disabled fluids appear as
sealed, labeled empty bays with `NOT IMPLEMENTED` status. Enabling a fluid MUST
fill and activate its existing bay in the same change; the map ID and bay
position do not change.

| Bay | Fluid ID | Slice state |
| --- | --- | --- |
| North-west | `water` | Active |
| North | `brine` | Reserved |
| North-east | `lava` | Active |
| West-north | `steam` | Active |
| East-north | `cryofluid` | Reserved |
| West-south | `acid` | Reserved |
| East-south | `toxic_slurry` | Active |
| South-west | `nutrient_solution` | Reserved |
| South-east | `aetheric_condensate` | Reserved |

Each active surface-fluid bay contains the same terrain sequence so behavior is
comparable:

1. a 4x4 sealed, level source pool at 2,000 `vU` depth;
2. a three-cell slope dropping 250 `hU` per cell;
3. a one-step 500 `hU` ridge with a one-cell notch;
4. adjacent sealed and unsealed 2x2 catch basins;
5. a drain after the measurement line.

The steam bay replaces the surface pool with a sealed 4x4 hot chamber, a two-
cell opening, a warm corridor, a cold 2x2 condensation pad, and a water catch
basin. Terrain never blocks open steam unless its cell is `gas_blocked`.

The central field has four separately resettable lanes:

- `water_lava`: equal 1,000 `vU` pulses meet on level sealed terrain;
- `lava_slurry`: equal 1,000 `vU` pulses meet on contaminated terrain;
- `steam_condense`: steam crosses warm-to-cold ambient zones;
- `non_reaction`: water and slurry mix/dilute without disappearing.

### 2.2 Automatic scenario

The automation runs bays one at a time to keep evidence readable:

1. Reset; release each active bay for 100 ticks; assert its transfer limit,
   downhill/diffusion behavior, retention, and contamination effect.
2. Reset; run every central interaction lane for 80 ticks; assert products,
   reactant decrease, terrain products, and mass accounting.
3. Reset; run all active bays and lanes together for 300 ticks at 4x; assert
   invariants and deterministic final hash.

The UI capture uses step 3 at tick 180, centered on the interaction field with
the fluid legend and assertion panel visible. Capture yaw quarters 0 and 2 to
prove pools, cliffs, liquid depth, steam height, and reactions remain readable
from opposing views. Each active bay includes at least three elevation steps so
the shared top/cliff mesh, fluid sides, occlusion fade, and picking are exercised.

### 2.3 Slice acceptance

- Water enters the lower catchment and passes the ridge only through its notch.
- Lava follows the same gradient no faster than 120 `vU`/tick per source edge.
- Slurry follows the gradient no faster than 180 `vU`/tick and contaminates
  only its unsealed basin.
- Steam spreads independent of elevation and condenses on the cold pad.
- Water/lava produces steam and rock; lava/slurry produces vitrified terrain;
  water/slurry remains material-conserving and has weighted contamination.
- Disabled bays contain no runtime fluid and clearly report reserved status.

## 3. Field-economy laboratory: `lab_field_economy`

The 32x20 fixture proves the R0 constraint loop without campaign narrative. Its
west lane contains a 20 `fabU` wreckage deposit, terrain edit pad, and channel
pad. Automation recovers the deposit, reserves/cancels a mixed plan, commits a
raise plus channel, verifies exact spending, removes the unoperated channel for
a full refund, operates and removes its replacement for a 75% refund, then
proves the depleted deposit cannot pay twice.

The east lane contains a 3 `eU/tick` geothermal fixture, a controlled water
flow through a turbine, and labeled safety/transport/process/interface test
loads. Automation proves current authored supply, one-tick-delayed turbine
output, whole-device priority allocation, transition-only brownout events,
curtailment, deficit, toggle recovery, and save/load of the exact ledger.

The automatic scenario MUST finish with these independently asserted accounts:

```text
starting_fab + recovered + refunds - committed_spend
= available_fab + reserved_fab

authored_generation + prior_tick_turbine_generation
= allocated + curtailed
deficit = max(0, enabled_demand - current_supply)
```

Free mode exposes visible `RECOVER`, queue/commit/cancel, dismantle, device
toggle, reset, and single-step paths. The 1280x720 capture shows the deposit,
depleted marker, fabrication ledger, geothermal source, turbine flow, power
priority, and one unpowered test load without depending on color.

## 4. Device showcase contract

Each enabled device owns `device_<device_id>.json`. The primary device may
appear more than once when its own behavior requires comparison. No other
placeable device definition may appear in that map. Authored fluid sources,
drains, terrain walls, planetary power fixtures, objective zones, and labeled measurement
fixtures are allowed and MUST be styled as test fixtures, not machines.

Every device scenario places/configures the primary machine once through 3D
ray picking before automation continues. Its capture MUST show the model,
footprint, ports, operating state, contact shadow, connected flow, and selected
3D outline at default zoom. Placeholder geometry is acceptable only until that
device's campaign-first-use milestone.

Every showcase follows a common 32x18 shell:

- left third: input/source and starting condition;
- center third: empty valid placement pad plus one pre-placed primary device;
- right third: output/measurement zone;
- bottom strip: expected behavior, current measurements, assertion results;
- build kit: only the primary device, with unlimited fabrication;
- automatic scenario: demonstrates inactive/default, active, and one edge case;
- free mode: player may place/remove/configure only that device through the
  production 3D placement ghost and hit-testing path.

Every powered-device showcase uses an authored planetary power fixture rather
than implicit free power. Its automatic scenario includes one tick below full
demand and asserts `UNPOWERED`, then restores sufficient supply. Passive-device
showcases still assert zero demand. Every scenario reports fabrication spending
and the five-field power ledger even when those values are zero.

## 5. Required slice device maps

### VM-CHANNEL — `device_channel`

Water faces a shallow ridge. One pre-cut channel crosses the ridge; an adjacent
blank route accepts player channels. Assert channel placement lowers 100 `hU`,
increases transfer cap by 150 `vU`, and redirects water without creating or
destroying it. Edge case: reject protected bedrock.

### VM-PIPE — `device_pipe`

A raised dry barrier separates a water source and sink. A prebuilt straight
pipe route demonstrates connection highlighting; the blank route requires
rotated segments. Assert capacity 600 `vU` per segment, network connectivity,
no surface leakage, and disconnected-segment warning. No pump is present;
fixture pressure provides flow.

### VM-PUMP — `device_pump`

A low pool and high output basin touch opposite sides of a placement pad.
Assert off/100% settings, 250 `vU` maximum transfer, uphill transport, +100
`pU`, 3 `eU/tick` demand, no transfer at 2 `eU`, conservation, and backpressure
at a full destination. Fixture inlet and outlet adapters are map edges, not
pipe devices.

### VM-FLOODGATE — `device_floodgate`

Two identical channels receive water; one has an authored gate. Cycle all five
open fractions and assert zero transfer when closed plus monotonic transfer at
25/50/75/100%. Edge case: gate direction rotation changes the blocked edge.

### VM-RESERVOIR — `device_reservoir`

A pulsed fixture source feeds a 2x2 placement pad and measured drain. Assert
8,000 `vU` capacity, configured accept/hold/release states, 1 `eU/tick` demand
while accepting/releasing, no powered effect at zero supply, no overflow below
capacity, and backpressure when full. The scenario ends with stored + drained
volume equal to injected volume.

### VM-SPILLWAY — `device_spillway`

A source raises upstream water above a crest. Assert no transfer below 2,000
`vU` depth, one-way transfer above it, maximum 500 `vU`/tick, and blocked reverse
flow. Compare a protected downstream zone that remains dry before threshold.

### VM-FLOW-TURBINE — `device_flow_turbine`

A fixture channel crosses the turbine pad. Run rates of 99, 100, 399, 400, and
600 `vU`/tick; assert generated power 0, 1, 3, 4, and 4 per tick respectively,
and unchanged material volume. Edge case: zero flow generates zero power.
Generation from tick N is unavailable to consumers until tick N+1.

### VM-SENSOR — `device_sensor`

A rising fixture pool is adjacent to a sensor; its link target is a labeled
binary test lamp/flow shutter fixture, not a placeable device. Assert sampling
of the prior completed state, threshold comparison, one-tick delayed output,
clear link visualization, 1 `eU/tick` demand, safety-priority allocation, and
deterministic response when crossing both ways.

### VM-FILTER — `device_filter`

Ten-thousand-bp slurry enters and exits through fixture adapters. Assert maximum
160 `vU`/tick, 2,500 bp removal per pass, 3 `eU/tick` demand, no volume loss,
no effect when off or underpowered, and saturation at zero after repeated
passes. A water pulse demonstrates that clean water remains clean.

### VM-RUNE-RELAY — `device_rune_relay`

Fixture power and adjacent water flow can be toggled independently. Assert the
relay is active only with at least 2 `eU/tick` and 100 `vU`/tick adjacent flow,
its 2x2 footprint/rotation placement, activation event emission, and immediate
deactivation after either condition is absent on a completed tick.

## 6. Growth rule

A new device change is incomplete unless it adds its definition, behavior
tests, `device_<id>` map, automatic command stream, assertions, evidence capture
target, and index entry here. A new fluid change is incomplete unless it
activates its reserved bay (or adds a named new bay if outside the GDD set),
defines terrain behavior, covers every pairwise interaction, and updates the
laboratory replay hash.

The current slice exposes `lab_fluids_all` through F1 and the capture scene
`lab_fluids_all`: its automatic 300-tick result is loaded into the production
orthographic 3D world, remains selectable and inspectable, and F1 returns to
the preserved campaign session. The lab report requires reserved-bay emptiness,
reaction events, deterministic hash output, and zero material-balance error.
The constraint retrofit adds `lab_field_economy` to the same verification menu
and capture harness before any campaign consumes `fabU` or `eU`. F2 continues
to open validated device showcases, V cycles them in free play, and every
report includes exact material, fabrication, and power ledgers.
