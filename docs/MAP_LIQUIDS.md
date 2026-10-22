# Static liquid surfaces

Subsequent work: the [full-tile development world](DEVELOPMENT_WORLD.md) now uses
this visual assembly for outdoor multiplayer movement with conservative collision
and water exclusions. This document records the original visual import checks.

The exported map now includes 30 static liquid patches using the source heights
and cell visibility masks. They use a generic translucent blue PBR material for
geometry inspection. This does not implement swimming, buoyancy, liquid collision,
source-game liquid textures, refraction, animated waves or gameplay liquid rules.
This full assembled map remains a visual preview. A separate
[terrain-only walking patch](TERRAIN_WALKING.md) now qualifies authoritative ground
height; it contains no liquids.

## View or reproduce

Prepared local output and a view from the open side of the lake:

```sh
cargo run --locked -p theia-client -- --inspect-asset build/qualification/map-liquids/map-with-liquids.glb --focus 55,22,2490,100,-1.5707963,0.9
```

Focus accepts `x,y,z,radius` with optional `yaw,pitch` in radians. The four-value
form keeps the existing default orientation. Yaw wraps; pitch stays within the
normal orbit limits. Home restores the selected framing/angles. Saved reports
record camera angles separately from whole-scene bounds and mesh counts.

To reproduce from the [static map assembly](MAP_OBJECTS.md), using the existing
offline Python environment and new output filenames:

```sh
mkdir -p build/map-liquids
build/asset-tools/bin/python tools/add_map_liquids.py /home/steven/wow.export/maps/2999/liquid_31_27.json build/map-assembly/map.glb build/map-liquids/map-with-liquids.glb > build/map-liquids/conversion.json
```

The tool also writes a small `map-with-liquids-surfaces.glb` for isolated geometry
inspection. Outputs and source files cannot be overwritten. Source hashes are
checked before/after generation, and the original map buffer is copied unchanged
into the combined GLB. No extra textures or runtime format are needed.

## Geometry interpretation

The primary reference is wow.export's
[liquid renderer at pinned commit `c2fd7bde36a712be78a5da896c995b84fbfa2545`](https://github.com/Kruithne/wow.export/blob/c2fd7bde36a712be78a5da896c995b84fbfa2545/src/js/map-viewer/LiquidRenderer.js#L420).
In the terrain OBJ's Y-up coordinates, a vertex at row/column is:

```text
step = (1600/3) / 16 / 8
x = terrainChunkPosition[1] - (column + xOffset) * step
y = vertexData.height[row * (width+1) + column]
z = terrainChunkPosition[0] - (row + yOffset) * step
```

Height samples are absolute. The exporter's
[synthesized `worldPosition` height](https://github.com/Kruithne/wow.export/blob/c2fd7bde36a712be78a5da896c995b84fbfa2545/src/js/3D/exporters/ADTExporter.js#L1510)
adds the terrain chunk height and must not be added again to the vertex heights.
The source renderer uses the vertex array directly. All referenced heights here
are approximately 21.192062–21.192066 source units.

Mask bits are row-major, least-significant bit first. An absent/empty bitmap means
all cells; a nonempty truncated bitmap fails. Trailing padding bytes are ignored.
Only vertices referenced by visible cells are emitted, so unused zero samples do
not create false geometry/bounds. Each cell becomes two upward-facing triangles,
using the source renderer's diagonal. Area-weighted normals are generated with
the existing NumPy dependency. Missing heights are accepted only for an explicitly
constant min/max level; incomplete or nonfinite samples fail rather than invent
a surface. Slot identities, grid offsets and dimensions are checked too.

The 256 chunk slots contain 30 nonempty instances, all with raw `liquidType=5`
and `liquidObject=18472`. Those are retained as source IDs, not interpreted as a
gameplay category. Resolving original materials/categories requires source liquid
database data not provided in this JSON. The preview colour and fixed opacity are
deliberate substitutes for this visual geometry check.

## Executed results — 2026-10-16

| Item | Result |
|---|---|
| Visible liquid patches / cells | 30 / 1,372 |
| Added vertices / triangles | 1,825 / 2,744 |
| Combined triangles / mesh instances | 256,296 / 2,786 |
| Materials / embedded images | 492 / 491 |
| Combined bytes | 704,120,480 |
| Source liquid bounds | X about 0–108.334, Y about 21.192, Z 2308.333–2658.334 |
| glTF validation | Zero errors/warnings; existing empty smoke-node info and 2,393 optional buffer-target hints |

Combined SHA-256:
`7c2600576961e87fc3748769f69a5ac74503479bdb87cd6f3d02ce0afd29854a`.
The map is one exported tile, so the liquid ends at its boundary; surrounding tiles
are not fabricated. The source OBJ/PNG/map/liquid exports remain unchanged and
outside Git.

This work exposed a capture timing issue: CPU scene instantiation and a fixed
two-second warmup did not ensure asynchronous render pipelines had compiled.
Some early captures contained only opaque terrain or only the transparent water.
Those captures are retained as rejected visual evidence.

The inspector now observes Bevy's pipeline cache through the normal
[`RenderApp`/`MainWorld` extraction pattern](https://github.com/bevyengine/bevy/blob/v0.20.0/examples/showcase/loading_screen.rs).
It resets readiness when framing a scene, then requires compiled pipelines, no
pending/failed entries and three clear render observations before capturing.
The existing asset/pose checks, minimum warmup and bounded capture timeout remain.
Reports include the actual pipeline readiness counts. The interactive renderer
keeps Bevy's asynchronous compilation; this change gates inspector captures only.

Native RX 7900 XTX / RADV checks use actual Khronos Vulkan validation in an owned
private Wayland display. The final open-lake capture shows terrain, trees and water
together; a matching view of the map without water verifies an actual image change.
A wolf run-animation capture checks the inspector regression path. Final captures
must have no pending/failed pipelines and no VUIDs, validation errors or panics.
The matching-image check finds 139,474 changed pixels outside the HUD, including
139,398 with a substantial blue-channel increase. Reproduce it on a display:

```sh
target/debug/theia-client --inspect-asset build/qualification/map-liquids/map-with-liquids.glb --focus 55,22,2490,100,-1.5707963,0.9 --capture build/liquid-check
target/debug/theia-client --inspect-asset build/qualification/map-objects/map.glb --focus 55,22,2490,100,-1.5707963,0.9 --capture build/liquid-baseline
build/asset-tools/bin/python tools/verify_liquids.py build/liquid-check build/liquid-baseline
```

No performance, Windows or source-shader-fidelity claim follows from these checks.

Twenty-seven Rust tests, four GLB repair tests and eighteen offline asset tests
pass with formatting, denied-warning all-target Clippy and build. New tests cover
mask bit order, cell offsets, absolute and sloped heights, generated normals,
excluded zero samples, invalid input, overwrite refusal, camera angle handling
and capture readiness/reset behaviour.

Ignored `build/qualification/map-liquids/` contains source hashes, generated GLBs,
validator output, test/build/lint logs, pinned upstream references, rejected
pre-readiness captures and final native reports/screenshots. These checks establish
visual liquid import. Server-owned terrain movement, reviewed environment clearance,
water gameplay and the original liquid material remain separate work.
