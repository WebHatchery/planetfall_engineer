# First Three Campaign Levels

## 1. Sequence contract

The initial slice contains exactly these campaign missions. They introduce one
problem before its tool, allow pause at all times, and use fixed events rather
than randomness. Coordinates below are normative anchors; surrounding terrain
may be art-tuned without changing routes, capacity, or reference replays.

All three maps use the production orthographic 3D renderer, stepped terrain,
fluid surfaces/sides, machine models, shadows, 3D placement/picking, and four
camera yaw quarters from §11. Reference solutions are executed through world
hits and admitted commands; a 2D debug grid is not acceptable gameplay evidence.

Each deployment opens on a blocking field briefing that names the new concept,
the primary objective, and a visible `BEGIN OPERATION` action. Commands and
simulation remain paused until the player clicks that action or presses Enter.

| Seq. | ID | Name | New concepts | Target first-play time |
| ---: | --- | --- | --- | ---: |
| 1 | `campaign_l01_first_flow` | First Flow | camera, cursor, inspect, excavation, channel, gate, run/observe | 8–12 min |
| 2 | `campaign_l02_holding_line` | The Holding Line | pipe, pump, reservoir, spillway, sensor, surge recovery | 15–20 min |
| 3 | `campaign_l03_firebreak` | Firebreak Protocol | lava, water/lava reaction, steam, turbine, rune relay | 20–25 min |

## 2. L01 — First Flow

### Purpose

Prove the core interaction in one screen: move around, inspect a slope, place a
route, run time, see water respond, then operate a gate to stabilize a basin.
`07_INTERACTIVE_TUTORIAL.md` is mandatory on first play and skippable on replay.

### Authored state

- Map: 32x20 stepped ash terrain, ambient 3,030 dK; camera target (16,10), yaw
  quarter 0, zoom level 3. The opening view shows a high north-west meltwater
  terrace, a winding descending cut through a central ridge, the sealed basin,
  a protected beacon marker, and the south-east waste sink.
- `meltwater_source`: (4,4), starts stopped; 180 `vU`/tick after tutorial run.
- Natural route slopes south-east toward `waste_sink` at (27,16).
- `restoration_basin`: 4x4 zone x=22..25, y=7..10, sealed, capacity 12,000 `vU`.
- Ridge x=12..18, y=4..11 at 1,850 `hU`; the authored cut descends from the
  source terrace through (12,8)..(21,8), reaching 250 `hU` before the basin.
- Protected survey beacon: (10,6); protected cells cannot be edited.
- Authored closed floodgate on basin inlet edge (21,8)->(22,8).
- Budget: 40 credits. Build kit: channel, floodgate, excavate.
- Time controls: pause, 1x, and 2x; 4x remains locked.

### Objectives and boundaries

- `primary_route_water`: at least 6,000 `vU` water in restoration basin.
- `primary_hold_basin`: all mandatory predicates true for 100 ticks.
- `failure_beacon_flood`: beacon cell depth >=1,500 `vU` for 10 ticks.
- Soft warning: water entering waste sink; it reduces no budget and cannot
  hard-fail, but resets the stability timer while basin volume is insufficient.

### Reference solution

Excavate the five route cells once each (20 credits), place channels on four of
them (8), commit, run until inlet is wet, open the authored gate to 50%, then
close it when basin reaches approximately 6,500 `vU`. Expected completion is
ticks 480–750 after first run. Other routes remain valid within budget.

### Acceptance

- First-time flow follows every required tutorial step without external text.
- The source remains stopped until the tutorial requests observation.
- At least two distinct valid channel alignments can reach the basin.
- Closing the gate visibly stops inflow on the following tick and the overlay
  changes direction/magnitude accordingly.
- Excavation rebuilds the visible 3D ridge face; channel placement is aligned to
  the modified top surface and remains pickable from every yaw quarter.
- Failure checkpoint restarts before the source is enabled, preserving completed
  camera/cursor tutorial steps but requiring the build/observe steps again.

## 3. L02 — The Holding Line

### Purpose

Teach contained transport and safety capacity. The player lifts water from a
low aquifer into a reservoir, supplies a restoration trench, and survives one
forecast meltwater surge using a spillway or sensor-assisted control.

### Authored state

- Map: 40x24; basin floor 0 `hU`, plateau 2,000 `hU`.
- `aquifer`: x=3..7, y=14..18, starts with 8,000 `vU`, then 100 `vU`/tick.
- `plateau_reserve`: valid 2x2 reservoir pad centered at (20,8).
- `restoration_trench`: x=31..36, y=7..9, sealed, target capacity 10,000 `vU`.
- `camp_zone`: x=24..28, y=14..18, protected; safe depth below 500 `vU`.
- `emergency_drain`: east boundary at (39,18).
- Preinstalled disconnected pipe trunks leave twelve marked player-build gaps
  across the aquifer-to-reservoir and reservoir-to-trench routes.
- Forecast surge at tick 900: aquifer becomes 400 `vU`/tick for 300 ticks, with
  alerts at ticks 600 and 800, then returns to 100.
- Budget: 105 credits. Build kit: prior tools plus pipe, pump, reservoir,
  spillway, and sensor. Terrain tools: excavate, raise, seal.
- All time controls unlocked after the first successful pump transfer.

### Objectives and boundaries

- `primary_supply_trench`: 6,000–9,000 `vU` water in restoration trench.
- `primary_reserve`: reservoir contains at least 2,000 `vU`.
- `primary_survive_surge`: camp depth stays below 500 `vU` through tick 1,250.
- Stability: all three true for 150 ticks after the surge ends.
- Hard failure: camp depth >=1,500 `vU` for 20 ticks.
- Optional `optional_automatic_safety`: a sensor changes a linked flow setting
  at least once during the surge. It does not gate completion.

### Reference solution

Place reservoir on the plateau, route pipe from aquifer fixture to one pump and
the reservoir, then from its release fixture to the trench. Add a spillway from
the aquifer overflow route to the emergency drain. A sensor adjacent to the
aquifer may close the pump or open the spillway fixture at 3,000 `vU` depth.
Hold 2,500 `vU` reserve before tick 900 and release after the surge. Expected
completion is ticks 1,400–1,800.

### Acceptance

- Pump preview explains inlet/outlet rotation and rejects a backward connection.
- Reservoir storage, release, and spillway threshold are readable in inspect.
- Surge alerts identify source, time remaining, predicted exposed zone, and a
  suggested inspect action without prescribing a single solution.
- The documented solution fits budget with at least 15 credits spare.
- At least one completion is possible without the optional sensor.
- The reservoir, raised pipes, pump ports, spillway crest, and sensor link are
  readable 3D silhouettes at default zoom from opposing yaw quarters.

## 4. L03 — Firebreak Protocol

### Purpose

Combine terrain routing with the first memorable material reaction. The player
meters water into lava to create a basalt firebreak, captures resulting steam
through a flow turbine fixture, and powers a dormant rune relay.

### Authored state

- Map: 48x30 volcanic caldera, ambient 3,230 dK.
- `lava_vent`: (5,15), 100 `vU`/tick initially; surge to 220 at tick 1,000 for
  250 ticks, alerts at ticks 700 and 900.
- `lava_route`: authored trough toward protected `ancient_foundation` zone at
  x=35..40, y=12..18.
- `water_cistern`: x=8..11, y=3..6, starts with 12,000 `vU`, no replenishment.
- `reaction_shelf`: x=20..25, y=12..16, sealed and 500 `hU` below approach.
- `steam_capture`: x=24..30, y=8..11, cold edge at x=30.
- One authored flow-turbine fixture socket at (27,9); player places the turbine.
- `climate_relay_socket`: 2x2 pad x=38..39, y=5..6.
- A cold condensate channel runs from (30,9) toward the relay pad, providing the
  relay's required adjacent flow when steam condenses successfully.
- Budget: 160 credits. Build kit: channel, pipe, pump, floodgate, reservoir,
  spillway, flow turbine, sensor, and rune relay. Filter remains lab-only.
- Terrain tools: excavate, raise, seal.

### Objectives and boundaries

- `primary_firebreak`: at least 3,000 `pending/formed rock vU` produced in the
  reaction shelf and no lava in ancient foundation.
- `primary_power`: turbine generated at least 40 cumulative power units.
- `primary_relay`: rune relay active continuously for 100 ticks.
- Stability: all mandatory objectives true and foundation lava-free for 150
  ticks after the lava surge ends.
- Hard failure: foundation receives any lava for 20 ticks.
- Soft failure: exhaust water before forming 3,000 rock; player can reset to the
  pre-release checkpoint rather than restart briefing.

### Reference solution

Excavate a controlled reaction pocket on the shelf, pipe/pump cistern water to
a 25% floodgate outlet above it, and channel lava into the opposite side. Place
the turbine in its socket and the rune relay on its pad. Meter water until at
least 3,000 combined rock is formed; steam passes through the turbine, then
condenses into the relay's adjacent flow channel. Preserve at least 2,000 `vU`
water for the surge. Expected completion is ticks 1,500–2,100.

### Acceptance

- First water/lava contact pauses once, explains steam/rock products from the
  emitted reaction event, then returns to the player's previous time control.
- Heat and flow overlays make the reaction location and steam route legible.
- The reaction cannot delete volume outside `SI-01` conversion rules.
- Relay inspect states independently show power and adjacent-flow requirements.
- The documented solution fits budget with at least 10 credits spare and water
  supply with at least 1,000 `vU` spare under deterministic playback.
- Lava depth, newly formed basalt height, airborne steam, turbine operation,
  condensate flow, and relay activation are simultaneously readable in the 3D
  firebreak capture without relying on the inspector alone.

## 5. Campaign progression and save behavior

- Only L01 is unlocked on a clean save. Success unlocks the next sequence.
- Completing a mission autosaves before debrief. Replaying cannot revoke unlocks.
- Each level stores best completion, optional flags, command count, simulated
  ticks, and final content version; no numeric grade is required for the slice.
- Active saves restore the exact mission tick and tutorial state. A content
  version mismatch returns to the latest mission-start checkpoint with a clear
  notice and never silently runs an incompatible state.

The runtime objective strip evaluates L03 firebreak progress from formed and
pending rock/vitrified products, rather than counting the water input used to
make them. Its deterministic reference stream supplies two reaction pockets,
replays through the same fixed-tick engine, and requires exact material
conservation after terrain products are formed into the heightfield.

Capture scenes `campaign_l01_first_flow`, `campaign_l02_holding_line`, and
`campaign_l03_firebreak` seed each authored briefing through the production
orthographic renderer; their current 1280×720 evidence is stored under
`docs/verification/` with the corresponding map dimensions visible in the HUD.
