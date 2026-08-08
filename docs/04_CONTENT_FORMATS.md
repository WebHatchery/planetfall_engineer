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
  "cost_credits": 12,
  "footprint": [[0, 0]],
  "allowed_rotations": [0, 90, 180, 270],
  "placement_rule": "surface_foundation",
  "behavior": { "kind": "pump", "max_rate_vu": 250, "pressure_add_pu": 100 },
  "asset_id": "device_pump",
  "showcase_map_id": "device_pump"
}
```

`behavior.kind` selects a tested implementation. Arbitrary scripting and
user-authored expressions are out of scope. Every enabled definition requires
a matching primary showcase map.

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
  "authored_devices": [],
  "scheduled_events": [],
  "camera_start": { "center": [16, 10], "zoom_bp": 10000 }
}
```

`rows_rle_v1` tokens are `<count>*<value>` separated by commas. Each decoded
row MUST equal map width and row count MUST equal height. A plain JSON array
encoding MAY be accepted for tests but published maps SHOULD use RLE.

Map `kind` is `campaign`, `fluid_lab`, or `device_showcase`. The map owns only
physical starting state. Briefings, budgets, objectives, allowed build kit, and
tutorial assignment belong to the mission definition.

### 4.4 Mission

```json
{
  "id": "campaign_l01_first_flow",
  "map_id": "campaign_l01_first_flow",
  "sequence": 1,
  "budget_credits": 40,
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
grant unlimited budget only through `budget_credits: null`.

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
`set_device_enabled`, and `show_message`. An effect that changes simulation
state cannot be encoded as `show_message`.

### 4.7 Tutorial

Tutorials are ordered steps with event predicates and UI directives. Exact
behavior and required step kinds are in `07_INTERACTIVE_TUTORIAL.md`.

## 5. Validation rules

The loader MUST collect and return every detectable error in deterministic
file/ID/field order. At minimum it validates:

1. JSON/schema versions and unknown fields.
2. Unique IDs within and across registries where namespaces overlap.
3. Every referenced fluid, device, map, zone, source, objective, tutorial,
   asset, connection, entity, and unlock.
4. RLE dimensions, coordinates, footprints, rotations, and height bounds.
5. Nonnegative rates/costs/volumes, contamination <=10,000, valid colors, and
   temperatures within the definition type range.
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

## 6. Content versioning

`schema_version` changes when serialized shape changes. `content_version`
changes when an authored starting state or rule changes enough to invalidate a
replay. Cosmetic text/assets do not invalidate replays. Save migration maps old
IDs and either upgrades active mission state or safely returns the player to
the latest completed campaign checkpoint.

Every reference solution stores mission ID, content version, admitted command
stream, expected terminal tick range, and final state hash. Content changes
that alter a hash MUST update the reference replay in the same commit.
