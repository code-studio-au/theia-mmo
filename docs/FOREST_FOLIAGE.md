# Static forest foliage adaptation

P1-10/P2-08 continuation, 2026-10-20. The prerequisites are hash-bound Unity
sources, explicit model/fileID matching, prefab/LOD selection and the ordinary
Bevy GLB/PBR path. This slice converts the actual background-tree cards and a
source vine LOD1 card. It does not assemble the forest or replace the town map.

## Source-specific material contract

The source package's basic vegetation shader selects leaf versus trunk using
vertex blue, applies a componentwise colour floor, and clips source texture alpha
against `_Cutoff`. It uses separate metallic/smoothness values for the two
branches. Treating it as an ordinary opaque albedo material loses those features.

`tools/unity_foliage.py` accepts only the reviewed basic shader GUID and exact
SHA-256 `f2f800520bffd6081ef699ded2f16e773e209b717cad6695b8786b97b499e9e0`.
The 17 shader payloads and metadata are hash-bound separately under ignored
`build/synty-enchanted-shader-source`, from the same archive as the assets. They
are read as source data, never compiled or executed. The full vegetation shader,
water, triplanar, crystal and particle shaders remain outside this profile.

The chosen adaptation uses a **linear PBR colour workflow**: decode source sRGB
RGB, take max with the corresponding serialized RGB floor, re-encode/quantize to
PNG, and retain original alpha bytes unchanged. A source Unity project/editor
colour-space reference is unavailable; this is an explicit workflow, not a claim
of exact Unity colour management or BRDF output. Source mesh normals replace
custom normal-map output; the unapplied normal references and approximation are
recorded in each conversion manifest. Emission/frosting must be disabled. The
reviewed basic shader does not use the stored wind/noise properties found in
these materials; the full vegetation shader's animation is still unimplemented.

Binary, constant-per-face vertex blue selects a leaf or trunk standard material.
Fractional selectors or faces crossing the branch threshold reject because they
need fragment-level selection. Vertex colours are consumed rather than multiplied
into exported albedo a second time. Both materials use source cutoffs, double-sided
MASK rendering and repeat/bilinear/nearest-mip sampling. Metallic and perceptual
roughness derive from the selected branch, with roughness `1 - smoothness`.
Texture transforms, unknown shader bytes/variants, unsupported source effects and
unqualified sampler/import settings reject. Source images are bounded at 4096².
No normal map, wind or effect is silently claimed as preserved.

## Source mesh and LOD selection

`tools/convert_unity_foliage.py` uses the static prefab resolver and checked FBX
fileID candidates, selects the exact named imported mesh and removes other FBX
LODs. It accepts one identity-placed static renderer, repeated source material
binding and qualified unit/axis import settings; deformed meshes reject. General
Unity-placement/FBX handedness conversion remains separate. The default planner
still selects highest detail. An explicit preview `--lod-level` can select another
serialized LOD without changing that default; absent levels reject.

The background-tree prefab uses the basic shader directly. The vine's highest
LOD uses another material family, so this profile deliberately selects its actual
**LOD1** card. It does not substitute that card for LOD0 in the scene plan. A fresh
full-scene default-plan run exactly matches the prior result, including all 18,480
emitted records, 423 unused overrides and 209 explicit unresolved placements.

## Executed checks

| Actual asset | Triangles | Materials / embedded images | Raw source image |
|---|---:|---:|---|
| Background-tree cards | 160 | 2 / 2 | 2048×2048 TGA |
| Vine LOD1 card | 8 | 1 / 1 | 1024×1024 TGA |

`tools/verify_unity_foliage.py` independently rereads source renderer/material/LOD
bindings. It compares branch-bound triangle positions, winding and UVs at 1e-5
quantization, and every derived RGBA pixel using a separately calculated scalar
colour table. Alpha stays exact: the tree image has 3,570,418 texels below cutoff
and 623,886 retained; the vine has 878,919 / 169,657. These are raw texture texels,
not visible pixels, instance counts or performance measurements.

The initial strict normal signature caught Blender's standard exporter rounding
components to four decimals and renormalizing. This is confirmed in installed
Blender glTF `primitive_extract.py::__get_normals` and `ROUNDING_DIGIT = 4`.
The independent check matches cyclic source triangles, normalizes their unit
normals and bounds the vector difference at 0.00018, derived from that rounding
step. Maximum observed errors are 0.000076603 (tree) and 0.000054680 (vine).
Positions/winding/UV and pixel gates retain their previous checks. Custom
normal-map output remains unapplied, independently of this small export loss.

Both GLBs pass Khronos validation with zero errors, warnings, informational
messages or hints. Two native Bevy inspector cases pass on **AMD Radeon RX 7900
XTX / RADV NAVI31 Vulkan**, active Khronos validation and compiled pipelines.
Both captures were visually inspected for actual cutouts. The tree GLB has one
mesh with two material primitives, correctly reported as two Bevy mesh instances.
The initial reused single-primitive fixture assertion failed and is marked
rejected; the corrected source-bound case passes. The vine case has one primitive.
No native Windows, Unity Editor reference, normal-map/lighting equivalence,
assembled scene, gameplay collision or default-map adoption is claimed.

Fresh fmt, denied-warning workspace/all-target Clippy and **109 distinct Python
tests** pass (104 asset, four GLB, one Blender reference test). Ten additional
private fault injections reject wrong alpha mode/cutoff/NaN, culling, surface,
sampling, branch names, geometry, image graph and normals. Source-reference tests
also cover all byte colour values/alpha, unsupported shader effects and ambiguous
selectors. The old default scene plan is unchanged. Rust code, runtime binaries,
engine/plugin pins and default launcher are unchanged; prior Rust/map-native
qualification remains separate, with no new Rust test rerun for these tools.
Offline Blender dependencies now explicitly pin Pillow 12.0.0 and NumPy 2.5.4;
the independent asset oracle uses its existing NumPy 2.3.4 environment.

## Local reproduction and evidence

```sh
build/blender-tools/bin/python tools/convert_unity_foliage.py \
  build/synty-enchanted-source build/synty-enchanted-shader-source \
  --prefab Assets/PolygonNatureBiomes/PNB_Enchanted_Forest/Prefabs/SM_Env_Vines_01.prefab \
  --lod-level 1 --output build/forest-new-vine
build/asset-tools/bin/python tools/verify_unity_foliage.py \
  build/synty-enchanted-source build/synty-enchanted-shader-source build/forest-new-vine
```

Qualified stages: `build/synty-forest-{card,vines}-pbr-qualified/`. Their GLB bytes
exactly match the GPU-tested v2-stage bytes. Native captures/reports:
`build/synty-forest-foliage-qualified-native-v2-20261020/cards/` and
`build/synty-forest-vine-card-qualified-native-20261020/vines/`.
Evidence includes `build/synty-forest-{card,vines}-qualified-verify.log`,
`build/synty-forest-{card,vines}-gltf-validation.json`,
`build/synty-forest-foliage-{faults,lod-regression,python-assets,python-glb,prefab-reference,fmt,clippy}.log`
and `build/synty-forest-foliage-faults.json`. Earlier trials lack the additional
normal reference checks; the qualified stages include them. All source/derived
assets and reports remain ignored; changes stay local.

Next: the full vegetation/material family used by highest-detail meshes, FBX
variant/geometry coordinate qualification and actual scene assembly, followed by
world collision and normal multiplayer/camera checks before town replacement.


## Highest-detail full-shader continuation

[Static full vegetation PBR](FULL_FOREST_FOLIAGE.md) now qualifies the actual LOD0
small tree and fern, source cutouts, explicit normal maps and three native cases.
Three exact-binding trees and 612 ferns join the visual subset. The basic card
regenerates byte-identically. Four used full-shader profiles, motion/highlights,
other shader families and full scene/collision/adoption remain open.
