# Static map buildings and props

The Ruins of Lordaeron reference tile now has a single assembled GLB containing
its terrain and static object placements, including nested building props. Source
exports remain outside Git and untouched. This is an inspector scene; multiplayer
movement, collision and NPC gameplay are not connected to this map yet. Liquid
geometry now has a separate [derived preview](MAP_LIQUIDS.md).

## Run or reproduce

Prepared local output:

```sh
cargo run --locked -p theia-client -- --inspect-asset build/qualification/map-objects/map.glb
cargo run --locked -p theia-client -- --inspect-asset build/qualification/map-objects/map.glb --focus 295,40,2250,70
```

The second command focuses on the town. `--focus x,y,z,radius` changes camera
framing only; full scene bounds/counts remain in `asset.json`, and the report's
`view` field distinguishes explicit focus from automatic framing. Hold RMB to
orbit, wheel to zoom and Home to reset the selected view. Add `--capture <directory>`
to save the real PNG/report. Large PNG sets take time to load; loading/capture
remain bounded and dependency failures are not replaced with success reports.

From a fresh checkout, first prepare terrain using the [terrain workflow](MAP_CHECK_2026_10_16.md).
Then run the offline assembler, with new output filenames:

```sh
mkdir -p build/map-assembly
build/asset-tools/bin/python tools/assemble_map.py /home/steven/wow.export/maps/2999/adt_31_27_ModelPlacementInformation.csv build/map-preview/terrain.glb build/map-assembly/map.glb > build/map-assembly/assembly.json
```

The assembler also writes `map-objects.glb`, preserving one copy of each unique
model part with multiple scene-node references. It joins this with the existing
terrain GLB by rebasing ordinary glTF indices and copying embedded buffers.
Terrain PNGs are not decoded/re-encoded during the join. No custom runtime format,
Blender dependency or engine renderer was introduced.

## Placement provenance

Transforms were derived from wow.export's
[Blender importer at commit `c2fd7bde36a712be78a5da896c995b84fbfa2545`](https://github.com/Kruithne/wow.export/blob/c2fd7bde36a712be78a5da896c995b84fbfa2545/addons/blender/io_scene_wowobj/import_wowobj.py#L2195).
The upstream code is a reference for interpretation, not a runtime dependency.
Tests cover known axis rotations and quaternion order; native views check the
actual assembled content. No Blender execution is claimed.

For an ADT row, `MAX = 51200/3`. World translation in the original terrain OBJ's
Y-up frame is `(MAX-PositionX, PositionY, MAX-PositionZ)`. Both M2 and WMO rows use
Euler angles, even though a `RotationW` column exists. The rotation is the upstream
Blender XYZ rotation `(RotationZ, RotationX, 90+RotationY)`, conjugated from Blender's
Z-up frame into the terrain OBJ's Y-up frame. Uniform `ScaleFactor` is retained.

Nested WMO rows use **WXYZ quaternions**, building-local positions and their own
uniform scale. They are children of the building placement, so building rotation
and scale apply to the contents. Only the selected doodad set and
`Set_$DefaultGlobal` are included. This tile's exported sidecars already contain
only the relevant sets. Duplicate ADT identities, unsupported types, nonfinite
transforms, missing dependencies and invalid scales fail explicitly.

Six building anchors were also compared with nearby terrain vertices. Their
height differences range from approximately -3.28 to +2.20 source units; this is
an approximate alignment diagnostic, not an exact surface/clearance test. Authored
heights and foundations were preserved rather than automatically snapped.

## Qualified content — 2026-10-16

| Item | Executed result |
|---|---|
| ADT placements | 846: 840 M2 and six WMO |
| Nested building placements | 624 across inn, stable, blacksmith, town hall and farmhouse |
| Distinct model files | 183, with all required OBJ/MTL/JSON/texture dependencies present |
| Visible object placements | 1,469; one empty smoke-emitter export has no geometry |
| Unique object mesh parts | 468, shared between repeated placements |
| Combined source meshes / instantiated meshes | 724 / 2,756, including terrain |
| Combined triangles | 253,552: 65,504 terrain plus 188,048 instanced object triangles |
| Materials / embedded images | 491 / 491 |
| Assembled bytes | 704,046,796 |
| glTF validation | Zero errors/warnings; one empty-node info; 2,322 optional buffer-target hints |

Checked GLB SHA-256:
`ad6de06e4845a975d9806269e0accc88cfac1ede22221d774bd2a82e7676fac4`.
The original terrain buffer is preserved exactly. Source model triangle counts
are checked before/after conversion; input hashes are checked before/after assembly.
The one nonrendering smoke emitter retains an empty placement node for provenance;
there is no fabricated particle geometry or active smoke effect.

Material opacity comes from exported metadata: opaque, alpha mask with cutoff 0.5,
or standard PBR alpha blend. Foliage uses its authored alpha-cutout mode. Objects
use rough, nonmetallic, double-sided PBR materials. Source blend mode 4 is
approximated with standard BLEND; additive glow, layered WoW shaders, point lights
and particle effects are not reproduced. No animation is imported for map objects;
buildings, doors, foliage and props remain static.

Twenty-five Rust tests, four GLB repair tests and ten offline asset tests pass,
along with formatting, denied-warning all-target Clippy and build. New tests cover
ADT axis mapping, pitch/roll, nested WMO quaternion/parent transforms, set selection,
invalid inputs and GLB index/buffer rebasing while retaining shared mesh references.
Existing asset tests also verify missing-texture failures and overwrite refusal.

Native default and focused map captures run through standard Bevy on RX 7900 XTX /
RADV in a private headless Wayland display. Actual Khronos Vulkan validation is
active, with no VUIDs, validation errors or panics. A wolf run-animation capture
checks the existing inspector path after adding focus. Screenshots are inspected;
these are correctness runs, not performance or streaming benchmarks. Windows and
exact source-game material fidelity remain unqualified.

Ignored evidence: `build/qualification/map-objects/` holds assembly/input hashes,
derived GLBs, full validator output, transform/alignment diagnostics, test/lint/
build logs, pinned upstream reference files and `native/{assembled-map,town,wolf}/`
reports/logs/PNGs. Inputs and large third-party assets are not added to Git.

Update: [liquid geometry and capture readiness](MAP_LIQUIDS.md) now have a separate
qualified preview. This report's original static-object evidence is retained.

A subsequent [terrain-only patch](TERRAIN_WALKING.md) now qualifies server-owned
height/slope movement and sampled terrain camera clearance. Reviewed static object
clearance and broader map collision are still required before using this full
assembly for multiplayer.
The flat procedural world and separate human/wolf/farmhouse scene remain available.
