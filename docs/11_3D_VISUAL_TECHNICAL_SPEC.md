# 3D Visual and Interaction Technical Specification

## 1. Visual target

The game and every verification map MUST render as a genuine 3D world from the
first playable foundation. The target is a readable orthographic engineering
diorama with the qualities shown by the supplied Timberborn-style reference:

- stepped, block-built terrain with visible top and cliff faces;
- compact machines with strong silhouettes and visible ports/state changes;
- liquids occupying channels and basins at readable physical heights;
- a warm, restrained terrain palette with fluids separated by hue and motion;
- an angled camera that exposes height, routing, and machine relationships;
- clean screen-space UI framing the world instead of flattening it into a board.

This is a style direction, not permission to reproduce another game's assets,
UI composition, icons, branding, or exact building designs. Planetfall Engineer
uses expedition machinery, volcanic materials, and alien rune structures from
its own GDD.

## 2. Locked rendering decisions

- World rendering uses Macroquad `Camera3D` with depth testing and orthographic
  projection. Perspective projection is not the default gameplay view.
- The simulation remains the deterministic cell heightfield in §03. Rendering
  converts that state to 3D meshes; scene objects never become authoritative.
- Terrain, fluids, devices, placement ghosts, selection, and objective markers
  occupy 3D world space. A 2D minimap or overlay panel MAY supplement them but
  cannot be the primary map.
- The slice uses a fixed directional sun, ambient fill, face shading, fog, and
  simple contact/blob shadows. Dynamic shadow maps, global illumination,
  reflections, and physically based rendering are out of scope for R0.
- All campaign, fluid-laboratory, and device-showcase acceptance is performed
  against the same 3D renderer and picking path used by the player.

## 3. World-space mapping

Simulation and rendering axes are fixed:

```text
simulation x -> world +X (east)
simulation y -> world +Z (south)
world +Y     -> elevation
```

Conversions:

```text
CELL_WORLD_SIZE       = 1.0
HEIGHT_WORLD_PER_1000 = 0.5
world_x = cell_x + 0.5
world_z = cell_y + 0.5
world_y = height_hu * 0.0005
fluid_top_y = world_y + depth_vu * 0.0005
```

The top of a cell at zero elevation is `Y=0`. Terrain extends downward to the
map's visual skirt floor. Logical 250 `hU` terrain edits therefore move a top
surface by 0.125 world units. Presentation MAY tween between old/new geometry
for up to 0.2 seconds, but picking and flow use the new state immediately.
Reduced-motion mode snaps directly.

## 4. Orthographic engineering camera

### 4.1 Camera state

The shared toolkit MUST expose a serializable orthographic engineering camera
or upgrade `render3d::IsometricCamera` to support this contract:

```text
target: Vec3
yaw_quarter: 0..3
current_yaw_radians: f32
pitch_radians: 0.61548       // approximately 35.264 degrees
ortho_vertical_span: f32
target_ortho_vertical_span: f32
viewport: optional rectangle
map_bounds_xz: Aabb2
```

Default yaw is 45 degrees looking north-east across the map. Rotate-left/right
changes the target quarter by exactly 90 degrees and visually eases for 0.2
seconds. Logical picking uses the current rendered camera matrix throughout the
tween. Pitch is fixed during R0. Zoom changes orthographic vertical span through
discrete 12, 16, 22, 30, 40, and 54 world-unit levels; it does not change field
of view or introduce perspective distortion.

### 4.2 Controls

- Pan: WASD, middle/right drag, or visible edge controls.
- Rotate: Q/E, rotate buttons, or drag gesture when bindings permit.
- Zoom: wheel, +/- bindings, and visible zoom controls.
- Focus selection/objective: binding plus inspector button.
- Camera target is clamped with a half-cell margin around map X/Z bounds.
- UI captures input before world camera controls.

Camera movement is presentation-only and excluded from simulation hashes. User
settings MAY persist the preferred yaw quarter and zoom level; active mission
saves do not depend on them.

## 5. Screen-ray picking

The toolkit MUST provide reusable `screen_ray(Camera3D, screen_pos, viewport)`
and ray/AABB helpers. Implement by inverting the exact view-projection matrix,
unprojecting the pointer at normalized device depths -1 and +1, and normalizing
the far-minus-near direction. Picking MUST account for the active 3D viewport
after virtual-UI gutters are removed.

World picking order is nearest positive ray distance:

1. interactive device collision boxes;
2. placement ports/edges when the selected tool requires them;
3. exposed terrain top/side AABBs;
4. authored objective/source markers;
5. no hit.

Hidden geometry cannot win. Terrain picking tests cell column AABBs from highest
visible elevation down, returning cell, hit face (`top`, `north`, `east`,
`south`, `west`), world point, and distance. Placement resolves from this hit:

- cell devices and terrain edits require a top face;
- gates/spillways require a vertical edge or the closest top-face edge;
- pipe rotation/port previews use hit cell plus current rotation;
- clicking visible liquid resolves to its supporting cell unless a device hit
  is nearer.

Hover and click on the same unmoved pointer/camera MUST return the same target.
Picking may use acceleration, but its result is deterministic for a fixed
camera and world snapshot.

## 6. Terrain mesh

### 6.1 Geometry

Terrain uses chunked indexed meshes generated from heightfield state. Default
chunk size is 8x8 cells. For each cell emit:

- one top quad at its world elevation;
- one vertical side quad per cardinal edge whose neighbor is lower or absent;
- a bottom skirt only around outer map boundaries;
- no internal face between equal-height adjacent cells.

Side quads span the exact height difference, including 250 `hU` increments.
Split a tall side vertically at every 1,000 `hU` boundary so the texture scale
remains consistent. Mesh vertices include position, atlas UV, face-normal data,
material tint, and ambient-occlusion factor. Chunk meshes MUST remain below the
Macroquad `u16` index limit.

Terrain edits dirty their containing chunk and any cardinal neighbor chunk that
shares the edited boundary. Dirty meshes rebuild after commands and before the
next render, never during the simulation transfer loop.

### 6.2 Materials and lighting

Use a texture atlas with separate top/side tiles for ash, rock, basalt, sealed
lining, vitrified slag, moss, and protected ruin material. UV density is one
tile per world unit; large surfaces repeat rather than stretch.

A lightweight custom material uses vertex normals for one directional sun and
ambient fill. North/east/south/west cliff faces MUST remain distinguishable at
all yaw quarters. Vertex ambient occlusion may darken internal corners, capped
so flow arrows and device ports remain readable. The renderer displays a flat
colored placeholder cube and one actionable asset ID when an art asset fails.

## 7. Fluid rendering

Fluid geometry is derived after the completed simulation tick:

- Surface liquids render a top plane at `fluid_top_y`, inset 0.03 world units
  from cell edges to prevent z-fighting. Exposed sides descend to terrain top or
  the adjacent lower fluid surface.
- Neighboring cells of the same visible material and near-equal surface height
  are batched into a chunk fluid mesh. Geometry rebuilds only for dirty fluid
  chunks; UV/time animation occurs in the shader.
- Water is blue/teal with directional ripples; lava is orange-red with emissive
  cracks; toxic slurry is purple with slow viscous motion. Each also differs in
  edge pattern and surface speed for non-color readability.
- Flow direction arrows are slightly raised world-space decals/billboards whose
  direction and size come from last-tick transfers. They remain legible after
  all four camera rotations.
- Steam uses a translucent low-density volume surface plus pooled billboard
  wisps seeded deterministically from cell/tick for appearance only. Particles
  never determine steam volume or picking.
- Reaction contact points show brief steam/rock/vitrification effects derived
  from `material_reacted` events. Reduced-motion uses a static burst marker.

The slice MAY render only the dominant surface material per cell, but inspect
and overlays MUST reveal every stored material and quantity. Blending choices
cannot alter simulation mixing/reaction state.

## 8. Device models and asset pipeline

### 8.1 Asset contract

Authored machine assets use binary glTF 2.0 (`.glb`) under `assets/models/` with
embedded geometry and textures. The shared toolkit MUST own asynchronous GLB
loading that checks the existing ZIP `AssetPack::bytes` path before loose
Macroquad `load_file`, matching texture/audio behavior on WebGL. Parse bytes
with a minimal glTF dependency because this removes material model-loading
complexity across games. Shader text uses the same pack-first raw-byte access.
This shared loader dependency is explicitly justified for WP-A0; do not add a
second project-local model parser or loader crate.
R0 accepts only:

- indexed triangle primitives;
- `POSITION`, `NORMAL`, and `TEXCOORD_0` attributes;
- one base-color texture/tint per primitive;
- static node transforms;
- no skinning, skeletal animation, morph targets, or runtime lights.

Model definitions include `model_asset_id`, world scale, yaw offset, local AABB,
port transforms, optional state-part nodes, and optional emissive state color.
Loading failure produces the placeholder cube without blocking simulation.

### 8.2 Readability requirements

- Every device silhouette is identifiable at the default zoom from all four
  yaw quarters.
- Input/output ports use physical geometry plus color-independent arrow/shape
  markers. Rotation preview shows both before placement.
- Operating, off, blocked, unpowered, warning, and failed states are visible on
  the model and repeated in inspect text.
- Pipes join adjacent port transforms without visible gaps. Elbows, straights,
  ends, and junctions select meshes from network topology after each commit.
- A placement ghost is translucent, depth-tested, and tinted valid/invalid;
  invalid state also uses a crosshatch and rejection icon.
- Contact shadows anchor machines to terrain. Models must not visually float or
  hide their occupied cell/edge.

Procedural cubes/prisms MAY stand in during Milestones A–C, but they still render
in 3D with final footprints, ports, picking boxes, and states. Campaign level
acceptance requires authored or approved slice-quality models for every device
visible in that level.

## 9. 3D overlays and occlusion

- Grade overlay tints terrain top faces and draws height labels on selected
  cells; it does not replace terrain geometry.
- Flow/heat/contamination overlays tint fluid/terrain and supply a screen-space
  legend. Critical values also use symbols/patterns.
- Selection outlines the picked 3D cell/device and draws its footprint.
- Protected cells use world-space corner markers visible above fluids.
- Objective zones use low vertical beacons and top-face boundaries.
- When terrain or a machine occludes the selection, the occluder fades to at
  least 35% opacity while hovered/selected. R0 does not require arbitrary
  cutaway planes; 90-degree camera rotation must expose every playable cell.
- World labels billboarding toward the camera must not scale below readable UI
  size; dense labels collapse until hover/inspect.

## 10. Render order

Each frame renders:

1. set the 3D camera and draw sky/background;
2. opaque terrain chunks;
3. opaque device models and pipe meshes;
4. opaque/alpha-tested world markers;
5. fluid surfaces and translucent steam/effects back-to-front by chunk;
6. selection, placement, flow arrows, and objective overlays;
7. restore the 2D virtual UI camera;
8. HUD, inspector, palette, tutorial, notifications, and debug counters.

State mutation is forbidden during all draw passes.

## 11. 3D acceptance tests

The R0 renderer is not complete until:

- a stepped 8x8 fixture emits only exposed top/side faces at exact heights;
- terrain dirtying rebuilds the edited and boundary-neighbor chunks only;
- screen rays hit known cells/devices at all four yaw quarters, every zoom level,
  and all required window sizes;
- edge placement resolves the intended gate/spillway edge from all rotations;
- fluid surfaces match authoritative depth and never z-fight at zero/maximum;
- pipe topology selects correct straight/elbow/junction geometry;
- placeholder models preserve footprint, port, state, and picking behavior;
- every campaign and verification reference capture is a 3D gameplay frame;
- camera movement/rotation, mesh rebuilds, particles, and animation never change
  simulation state or replay hash;
- §09 3D draw-call, mesh, picking, resize, and performance gates pass on Windows
  and WebGL.
