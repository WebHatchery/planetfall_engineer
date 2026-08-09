# Content Formats and Validation

## 1. Goals

Campaign and verification content MUST be data-authored and fail as a group
with actionable diagnostics. JSON is used because the template already embeds
and validates JSON with `serde_json` and macroquad-toolkit registries. Runtime
code MUST NOT contain level-specific coordinates or tutorial prose.

## 2. Asset layout

```text
assets/data/
  game_config.json
  fluids.json
  interactions.json
  terrain_materials.json
  devices.json
  tutorials.json
  missions.json
  maps/
    campaign_l01_first_flow.json
    campaign_l02_holding_line.json
    campaign_l03_firebreak.json
    lab_fluids_all.json
    device_<device_id>.json
assets/models/
  devices/<device_id>.glb
  ruins/<asset_id>.glb
assets/textures/
  terrain_atlas.png
  fluid_atlas.png
  models/<asset_id>.png
assets/shaders/
  world_lit.vert
  world_lit.frag
  fluid.vert
  fluid.frag
```

Definitions are embedded at compile time for native/WebGL parity. Art assets
remain in the project asset pack and are referenced by logical asset ID.

## 3. Stable ID rules

- IDs use lowercase ASCII snake case and match `^[a-z][a-z0-9_]*$`.
- IDs are permanent once included in a published save. Rename through an
  explicit migration alias, never by silently changing a string.
- Prefix authored campaign levels `campaign_lNN_`; verification maps use
  `lab_` or `device_`.
- Map entity IDs are unique within a map and stable across cosmetic edits.
- Objective IDs start with `primary_`, `optional_`, or `failure_`.
- Tutorial step IDs begin with the tutorial ID, for example
  `tutorial_l01_move_camera`.

## 4. Definition contracts

Rust structs MAY use stronger enums/newtypes, but serialized fields and meaning
must follow these contracts. Unknown fields are errors during development and
tests (`deny_unknown_fields`); release data is produced by the same validation.

### 4.1 Fluid

```json
{
  "id": "water",
  "display_name": "Water",
  "phase": "surface",
  "default_temp_dk": 2930,
  "max_transfer_vu_per_tick": 400,
  "default_contamination_bp": 0,
  "render_asset_id": "fluid_water",
  "overlay_color": "#3A9DFFFF",
  "enabled_in_slice": true
}
```

Required reserved IDs are `water`, `brine`, `lava`, `steam`, `cryofluid`,
`acid`, `toxic_slurry`, `nutrient_solution`, and `aetheric_condensate`.
Only `water`, `lava`, `steam`, and `toxic_slurry` may be enabled in the slice.

### 4.2 Device

```json
{
  "id": "pump",
  "display_name": "Pump",
  "category": "transport",
  "fabrication_cost_fu": 12,
  "power_demand_eu_per_tick": 3,
  "power_class": "transport",
  "footprint": [[0, 0]],
  "allowed_rotations": [0, 90, 180, 270],
  "placement_rule": "surface_foundation",
  "behavior": { "kind": "pump", "max_rate_vu": 250, "pressure_add_pu": 100 },
  "model": {
    "asset_id": "device_pump_glb",
    "scale_bp": 10000,
    "yaw_offset_degrees": 0,
    "local_aabb": { "min": [-0.45, 0.0, -0.45], "max": [0.45, 0.9, 0.45] },
    "ports": [
      { "id": "inlet", "position": [0.0, 0.25, -0.5], "direction": "north" },
      { "id": "outlet", "position": [0.0, 0.25, 0.5], "direction": "south" }
    ]
  },
  "showcase_map_id": "device_pump"
}
```

`behavior.kind` selects a tested implementation. Arbitrary scripting and
user-authored expressions are out of scope. Every enabled definition requires
a matching primary showcase map. `model` follows §11's GLB/static-model subset;
its AABB and ports are validated against footprint and allowed rotations.

### 4.3 Map

```json
{
  "schema_version": 1,
  "id": "campaign_l01_first_flow",
  "display_name": "First Flow",
  "kind": "campaign",
  "size": { "width": 32, "height": 20 },
  "ambient_temp_dk": 3030,
  "height_bounds_hu": { "min": -2000, "max": 5000 },
  "terrain": {
    "encoding": "rows_rle_v1",
    "height_rows": ["32*1000"],
    "material_rows": ["32*ash"]
  },
  "flags": [],
  "zones": [],
  "sources": [],
  "resource_deposits": [
    {
      "id": "wreckage_cache",
      "cell": [5, 6],
      "yield_fu": 28,
      "asset_id": "deposit_wreckage"
    }
  ],
  "power_sources": [],
  "authored_devices": [],
  "scheduled_events": [],
  "camera_start": {
    "target_cell": [16, 10],
    "yaw_quarter": 0,
    "zoom_level": 3
  }
}
```

`rows_rle_v1` tokens are `<count>*<value>` separated by commas. Each decoded
row MUST equal map width and row count MUST equal height. A plain JSON array
encoding MAY be accepted for tests but published maps SHOULD use RLE.

Map `kind` is `campaign`, `fluid_lab`, `field_economy_lab`, or
`device_showcase`. The map owns physical starting state, finite deposit
locations/yields, and authored planetary power fixtures. Deposit cells cannot
overlap devices, protected cells, sources, or one another. Briefings, starting
fabrication stock, objectives, allowed build kit, and tutorial assignment
belong to the mission definition.

### 4.4 Mission

```json
{
  "id": "campaign_l01_first_flow",
  "map_id": "campaign_l01_first_flow",
  "sequence": 1,
  "starting_fabrication_fu": 0,
  "allowed_device_ids": ["channel", "floodgate"],
  "allowed_terrain_edits": ["excavate"],
  "tutorial_id": "tutorial_l01",
  "briefing_text_id": "briefing_l01",
  "objectives": [],
  "checkpoint_policy": "on_tutorial_milestone",
  "unlocks_on_success": ["campaign_l02_holding_line"]
}
```

Verification missions use `sequence: null`, have no campaign unlock, and may
grant unlimited fabrication only through `starting_fabrication_fu: null`.
Campaign missions MUST use a finite integer and MUST author enough recoverable
yield for their reviewed reference and recovery routes.

### 4.5 Objective

```json
{
  "id": "primary_fill_basin",
  "kind": "zone_volume",
  "zone_id": "restoration_basin",
  "fluid_id": "water",
  "comparison": "at_least",
  "value": 6000,
  "stability_ticks": 100,
  "mandatory": true,
  "visible": true
}
```

Failure objectives use the same predicate fields plus
`terminal_when: "predicate_true"`. Objective evaluation must not parse prose.

### 4.6 Scheduled event

Events use either a fixed tick or a predicate trigger, never both.

```json
{
  "id": "storm_surge",
  "trigger": { "tick": 600 },
  "warning_ticks": [300, 500],
  "effect": { "kind": "set_source_rate", "source_id": "rain_inflow", "rate_vu": 350 },
  "duration_ticks": 300
}
```

Slice effect kinds are `set_source_rate`, `inject_material`, `set_drain_rate`,
`set_device_enabled`, `set_power_source_output`, and `show_message`. An effect
that changes simulation state cannot be encoded as `show_message`.

### 4.7 Tutorial

Tutorials are ordered steps with event predicates and UI directives. Exact
behavior and required step kinds are in `07_INTERACTIVE_TUTORIAL.md`.

### 4.8 Resource deposit and planetary power source

A resource deposit is a one-shot map fixture with `id`, `cell`, positive
`yield_fu`, `asset_id`, and optional `label_text_id`. Runtime adds only a saved
`depleted` flag. It has no inventory, owner, extraction rate, recipe, or worker.

An authored power source is a non-placeable planetary fixture:

```json
{
  "id": "geothermal_tap",
  "cell": [6, 16],
  "output_eu_per_tick": 3,
  "enabled": true,
  "asset_id": "power_geothermal_tap"
}
```

It contributes current-tick field-grid supply while enabled. A turbine is a
placeable producer governed by its device behavior and MUST NOT be duplicated
as an authored power source. Both definitions expose a stable world marker and
power-overlay label.

## 5. Validation rules

The loader MUST collect and return every detectable error in deterministic
file/ID/field order. At minimum it validates:

1. JSON/schema versions and unknown fields.
2. Unique IDs within and across registries where namespaces overlap.
3. Every referenced fluid, device, map, zone, source, objective, tutorial,
   asset, connection, entity, and unlock.
4. RLE dimensions, coordinates, footprints, rotations, and height bounds.
   Camera targets/yaw/zoom, model AABBs, and port transforms must also be valid.
5. Nonnegative rates/costs/volumes/deposit yields/power values, contamination
   <=10,000, valid power classes, valid colors, and temperatures within the
   definition type range.
6. No overlapping authored devices or placement on protected/invalid cells.
7. Every campaign mission has a reachable unlock chain from `L01` with no
   cycle; exactly one slice campaign mission has sequence 1, 2, and 3.
8. Every enabled device has one `device_showcase` map whose primary device ID
   matches it.
9. `lab_fluids_all` has a labeled bay for every enabled fluid and a central
   interaction field containing every enabled interaction pair.
10. Tutorial target IDs and permitted commands are valid for their mission.
11. Every mandatory objective is syntactically satisfiable: referenced source
    and target material exist, stability is positive, and its threshold fits
    zone/device capacity. This is not a solver proof.
12. Every enabled device resolves a static GLB model or an explicitly approved
    3D placeholder, and every terrain/fluid material resolves its shader/atlas
    assets. A missing asset fails validation tests even though runtime retains a
    diagnostic placeholder.
13. Every campaign map has at least one positive-yield deposit, no duplicate or
    overlapping deposit, and total authored plus starting fabrication equal to
    the amount asserted by its reference ledger.
14. Every powered campaign device can resolve the mission-wide field grid;
    power-source IDs are unique, turbine output is not authored as current-tick
    supply, and each reference stream declares expected generated/allocated/
    curtailed/deficit totals.

## 6. Content versioning

`schema_version` changes when serialized shape changes. `content_version`
changes when an authored starting state or rule changes enough to invalidate a
replay. Cosmetic text/assets do not invalidate replays. Save migration maps old
IDs and either upgrades active mission state or safely returns the player to
the latest completed campaign checkpoint.

Every reference solution stores mission ID, content version, admitted command
stream, expected terminal tick range, and final state hash. Content changes
that alter a hash MUST update the reference replay in the same commit.

## 7. Current slice registry

The current implementation embeds `assets/data/content_registry.json` and
validates its cross-references before constructing the runtime. The resource-
and-power retrofit MUST advance its schema/content version and cover the
deposit/power fields before campaign saves may resume. The current registry
covers the four enabled slice fluids plus five reserved IDs, all ten device
showcase references, three campaign missions, twelve legacy `tutorial_l01`
steps, and three campaign plus eleven verification map IDs. The retrofit target
is fourteen L01 steps, five L02 power steps, and twelve verification map IDs.
Runtime behavior remains in
the bounded simulation/device modules; the registry owns the stable content
contract and fails with collected diagnostics when it is malformed.
