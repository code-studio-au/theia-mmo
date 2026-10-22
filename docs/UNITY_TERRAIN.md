# Enchanted Forest TerrainData conversion

P1-10/P2-06/P2-07/P2-08 prerequisite, 2026-10-20. The current slice preserves and
renders the actual ground; it does not import the complete forest or create a
runtime collision package. Prerequisites are the hash-bound source/demo metadata
and the [calibrated source coordinate convention](UNITY_GEOMETRY.md).

## Decoded source

The demo has one enabled static root Terrain and a matching enabled
TerrainCollider referencing `Enchanted_Forest_Terrain.asset`, GUID
`493a093e8e1d66540810f9da20f0ca7c`, fileID 15600000. Its binary Unity version is
2020.3.40f1. A separate offline Python environment pins UnityPy 1.25.0, NumPy
2.3.4, PyYAML 6.0.3, Pillow 12.0.0 and trimesh 4.10.0. No Unity scripts, external
binary dependencies, Unity Editor or runtime engine dependencies are loaded.
[UnityPy's type-tree reader](https://github.com/K0lb3/UnityPy/blob/master/UnityPy/files/ObjectReader.py)
provides typed decoding; our reader verifies source hashes and GUID/fileID/type
bindings explicitly.

The terrain is **513 × 513 samples**, 150 × 150 metres, with source origin
(-75, -20, -75), X/Z sample spacing 0.29296875 metres and vertical scale 100.
All 263,169 unsigned source samples are retained exactly. Heights divide by
**32766**, not 32767 or 65535; Unity documents that normalization in its
[terrain paint implementation](https://github.com/Unity-Technologies/UnityCsReference/blob/master/Modules/Terrain/Public/PaintContext.cs).
The source's complete hierarchy of **1,365 patch minimum/maximum pairs** agrees
with that conversion and row-major grid convention to 1.408e-8 normalized height.
The independent verifier also binds the entire height array to a unique
count-prefixed sequence in the raw serialized TerrainData object bytes.

Two embedded linear RGBA32 control textures contain five painted layers, ordered:
multicolour leaves, dirt, mud, leaves and moss. All five external binary GUIDs
resolve to actual source layer fileIDs and texture metadata. The 512 × 512 × 5
base control bytes remain exact. Three unused channels must be zero. Source
weight sums range from 244 to 273; they are preserved without silently normalizing
or discarding the fifth layer. There are no populated terrain tree instances,
detail patches or holes. Unpopulated detail prototypes remain source data;
populated details/trees or any holes reject this bounded profile.

## Static rendering representation

All height samples become a standard embedded glTF mesh: 263,169 vertices and
524,288 triangles. Apply the source X reflection consistently with model
placements. Geometry spans X/Z ±75 metres and Y -16.322407 to 9.005677 metres.
The representation chooses a fixed regular grid diagonal and derives geometric
normals. It does not reproduce Unity's adaptive terrain tessellation or prove
TerrainCollider parity between samples.

An explicit 4096 × 4096 base-colour bake uses all five source images, their repeat/
bilinear sampling, tile sizes/offsets, linear sRGB blending and painted control
weights. Control pixels interpolate across their first/last texel centres;
source diffuse UVs use terrain-local metres, and glTF/image vertical orientation
is explicit. Unity's [terrain shader source](https://github.com/Unity-Technologies/Graphics/blob/master/Packages/com.unity.render-pipelines.universal/Shaders/Terrain/TerrainLitPasses.hlsl)
is reference for terrain sampling conventions, not an equivalence claim for this
source's built-in material. Output uses ordinary rough nonmetallic glTF PBR.
Unsupported active mask/specular/remap/sampler profiles reject.

This is a **finite-resolution base-colour preview**. It reduces the source tiled
texture detail and does not reproduce multi-pass terrain lighting or normal maps.
All five authored normal-map references/files are hash-bound and recorded but
not rendered. Full terrain material adoption requires reviewing those differences;
a successful preview does not waive that gate. Changing bake resolution alone
does not establish visual parity.

## Executed acceptance

The independent verifier imports neither the terrain reader nor the converter.
It checks all source heights/raw bytes, all patch extrema, every output position,
fixed-grid coverage/diagonal/winding, UVs, raw layer weights, source texture
bindings and 685 independent scalar colour samples. Maximum position error is
**0.000000605 metres** from f32 storage; sampled colour-channel error is **zero**.
These samples verify the stated bake algorithm, not Unity shader equivalence.

Both the ground and a combined scene containing the already-qualified five ruins
placements pass Khronos glTF validation with zero errors, warnings, information
messages or hints. Two actual Bevy inspector cases pass on **AMD Radeon RX 7900
XTX / RADV NAVI31, Vulkan**, Linux, with active Khronos validation, compiled
pipelines, no VUID/errors/panics and visually inspected 1280 × 720 captures.
Native actual transformed vertex bounds agree within 0.000000605 metres.
Ground reports one material/image/mesh; the combined scene reports six mesh
instances, two shared meshes, two materials, three images and 535,828 triangles.
The inspector floor and blue capsule are inspection aids, not imported terrain
collision or a multiplayer player.

Fresh fmt, denied-warning workspace/all-target Clippy and **127 Python tests**
pass: 120 asset, four GLB, one prefab reference and two model-variant reference.
Six new terrain cases cover asymmetric sample orientation/winding/UVs,
normalization/patch disagreement, holes/truncation/invalid scales, control formats/
missing channels/streams, bilinear repeat/clamp/colour and serialized GUID identity.
The Rust application source/binary, dependency lock and default town are unchanged;
previous Rust tests were not rerun for these offline tools. No native Windows,
collision, multiplayer or whole-map adoption claim follows. All changes stay local.

## Evidence and reproduction

Private ignored artifacts:

- `build/synty-forest-terrain-v1/`: GLB, source-height/control bytes, bake and
  conversion report with 32 source data/metadata hash bindings.
- `build/synty-forest-terrain-reference-v1.json`: independent verification.
- `build/synty-forest-{terrain,terrain-ruins}-validation-v1.json`: Khronos reports.
- `build/synty-forest-terrain-ruins-v1.{glb,json}`: merged scene and input/output hashes.
- `build/forest-terrain-native-v1-20261020/{terrain,terrain-ruins}/`: native reports,
  logs and screenshots.
- `build/unity-terrain-tools-freeze.txt`: actual complete offline environment.
- `build/forest-terrain-{python-tests,clippy}.log`: current checks.

Ground GLB SHA-256:
`efd983710e32dd7c9fd88441fe4048744bb228e1f57e47effd0fc2b04ae9eb68`.
Combined GLB SHA-256:
`124ca26c121e6b2f94beba4ca2d3f0326709993d17e8cec28077589fcfbbe325`.

```sh
python3 -m venv build/new-unity-terrain-tools
build/new-unity-terrain-tools/bin/pip install -r tools/unity-terrain-requirements.txt
build/new-unity-terrain-tools/bin/python tools/convert_unity_terrain.py \
  build/synty-enchanted-source \
  --scene Assets/PolygonNatureBiomes/PNB_Enchanted_Forest/Scene/Demo_01.unity \
  --output build/new-forest-terrain --base-color-size 4096
build/new-unity-terrain-tools/bin/python tools/verify_unity_terrain.py \
  build/synty-enchanted-source build/new-forest-terrain \
  --report build/new-forest-terrain-reference.json
```

Use fresh output paths. Source assets are private/local and must already be staged.
Next: qualify matching terrain collision in bounded packages while retaining all
height samples, and complete full vegetation/material/source-scene assembly.
The full grid exceeds the current 131,072-triangle collision-profile limit;
partition it rather than silently decimating or increasing runtime budgets.
Actual spawns, collision completeness and native multiplayer checks precede
replacement-map adoption. Preserve current town regressions meanwhile.


## Ground collision continuation

[Full-sample bounded packages](FOREST_TERRAIN_COLLISION.md) now preserve every
source ground triangle through standard collision/shared-image contracts. Real
Avian and encrypted native two-client seam checks qualify the ground-only slice.
The full forest, material approximations and authored object collision remain
separate adoption gates.
