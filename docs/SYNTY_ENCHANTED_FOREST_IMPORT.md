# Enchanted Forest import target

P1-10/P2-08 investigation; eventual P2-06/P2-07 adoption, 2026-10-20.
The user selected **Enchanted Forest instead of Elven Realm**. Enchanted Forest
is now the active environment import target. Elven Realm's source audit and
pillar preview remain historical evidence; finding its missing LFS blobs is no
longer a prerequisite or an active request to the user.

## Available source

Local archive: `/home/steven/Downloads/Synty Models/Biomes/Enchanted Forest - Nature Biome v1.01.unitypackage`.
Inventory contains 667 payloads: 225 FBXs, 173 prefabs, 97 PNGs, 81 materials,
32 asset files, 31 TGAs, 17 shaders, five terrain layers, two Unity scenes,
two nested Unity packages, one JPG and one C# file. Total inventoried payload
size is approximately 403.77 MiB. Counts are source files, not runtime instances.

The hash-bound stage `build/synty-enchanted-source` extracts 647 selected data
payloads and their adjacent metadata. Shader/C# source and nested packages are
not selected or executed. **No Git LFS pointers** occur in the selected payloads;
all 225 model files and all selected images have actual payloads. This removes
the specific Elven missing-blob obstacle, not every scene-conversion requirement.

## Scene audit and remaining work

The selected built-in-render-pipeline scene is
`Assets/PolygonNatureBiomes/PNB_Enchanted_Forest/Scene/Demo_01.unity`.
`Demo_URP.unity` is an available alternative, not the selected scene. Recursive
audit visits 436 assets: 118 prefabs, 154 FBXs, 55 materials, 67 PNGs, 25 TGAs,
16 asset files and the scene. Its 39,746 serialized records include 18,755 prefab
instances and 19,743 transforms. These do not establish resolved render counts.

Fourteen external GUIDs are outside the selected stage. Eight are present in the
archive (seven shaders and LightMotion.cs) but intentionally unselected; six
are absent from the archive, in an override object reference, a particle material
and four material texture properties. Their runtime significance needs explicit
assessment; zero LFS pointers does not prove scene equivalence or completeness.

A trial of the existing Elven-only simple-prefab preview rejects the forest
ruins material because it lacks the assumed `_BaseColor` property. No preview
was produced and no default white material was substituted. Explicit source
material mapping is needed before qualifying a forest GLB; Unity shaders cannot
be loaded directly as Bevy materials.

Next acceptance slices:

1. Resolve model/fileID bindings, prefab hierarchy/overrides and LOD selection
   against source data; preserve actual demo placements and flag unresolved IDs.
2. Implement a declared source-material conversion to standard Bevy PBR, with
   separately stated approximations for foliage, crystals, water and effects.
   Independently compare geometry, UVs, textures and nested transforms.
3. Qualify a rendered forest preview on the native GPU, then assembled scene
   resources. Prepare identity-bound world collision and independently qualify
   authority movement, camera/access and multiplayer before launcher adoption.

The normal town launcher and its regression fixtures remain operational until
the replacement passes. The Base Locomotion character work remains separate.
No forest scene, collider, navigation or playable-map adoption is claimed here.
All proprietary source/assets and reports stay ignored under `build/`.

## Reproduce the source audit

```sh
build/asset-tools/bin/python tools/unity_scene.py build/synty-enchanted-source \
  --scene Assets/PolygonNatureBiomes/PNB_Enchanted_Forest/Scene/Demo_01.unity \
  --report build/enchanted-new-audit.json
build/asset-tools/bin/python tools/unity_model_ids.py build/synty-enchanted-source \
  --dependencies build/enchanted-new-audit.json \
  --report build/enchanted-new-model-ids.json
```

Private evidence: `build/synty-enchanted-inventory.json`, source `extraction.json`,
`build/synty-enchanted-dependencies.json`, `build/synty-enchanted-lfs.json` and
`build/synty-enchanted-preview.log` (the rejected preview).

The existing source model-ID audit matches **264 distinct asset/fileID
references**, with no unmatched references or model-reader errors in this audited
closure. This is identity evidence, not instantiated scene equivalence. Report:
`build/synty-enchanted-model-ids.json`. Fresh fmt, denied-warning Clippy and
**91 Python tests** pass. Rust runtime/binaries are unchanged; this source and
documentation switch does not claim a new GPU qualification.


## Qualified Standard material and ruins preview — 2026-10-20

The source-material rejection above is now resolved for explicitly identified
opaque built-in Standard materials. `tools/unity_materials.py` produces a reusable
property plan, and `preview_unity_static.py --standard-material` uses ordinary
Blender/glTF PBR output. It maps `_MainTex`/white `_Color`, scalar metallic,
perceptual roughness as `1 - _Glossiness`, and enabled white/black emission with
its source map. Source PNGs must be sRGB, repeat/bilinear, mip-enabled and without
anisotropy or bias adaptations. Unknown shaders, alpha modes, nonwhite tints,
texture transforms, active normal/detail/metallic maps, disabled reflection
features and unsupported keywords reject. Unsupported effects are not silently
removed. This is a declared PBR adaptation, not Unity BRDF or lighting parity.

Unity documents the source properties in its
[Standard material reference](https://docs.unity.com/en-us/engine/6000.3/manual/materials-and-shaders/built-in/shader-built-in-birp/shader-standard-shader/standard-shader-material-parameters).
The source smoothness-to-roughness mapping is an explicit adaptation; equivalent
full shader output is unproven. Backface culling follows the opaque source.
Blender's default trilinear sampler is finalized to repeat/bilinear with nearest
mip (glTF 9985), preserving every embedded BIN byte. See the
[glTF sampler specification](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html#reference-sampler).
The existing explicitly approximate Elven base-colour path remains a regression.

`SM_Prop_Ruins_01` now converts with 4,026 imported source vertices and 7,844
triangles, one mesh/material, and separate 4096×4096 albedo and emission PNGs.
Its source-imported Y-up dimensions are approximately 11.438×10.061×11.487 metres.
The independent verifier compares all triangle positions, winding and UVs at
1e-5 quantization; both source images match exported RGBA pixels exactly. It
independently reads the source renderer/material and verifies actual glTF texture
slots, emission, surface scalars, opacity, culling and sampler values. Tests reject
swapped image slots, wrong roughness/emission, NaN, changed UV set, double-sided
output and trilinear sampling. Property-plan tests reject unsupported active
features, malformed/duplicate data and nonfinite/out-of-range scalars.
Khronos validation reports zero errors, warnings, informational messages or hints.

One native Bevy inspector case passes on **AMD Radeon RX 7900 XTX / RADV NAVI31,
Vulkan**, with active Khronos validation, compiled pipelines and visually
inspected capture. Ordinary Bevy reports one material, two images and 7,844
triangles. This is Linux execution and an isolated asset, not an assembled forest,
normal-map adoption, runtime material-equivalence proof or Windows execution.

Field-level profiling accepts 12 of 55 source materials in the demo dependency
closure. Forty-one identify other shaders and two use unsupported Standard
opacity/pass settings. This is property-plan eligibility; it does not qualify all
of those materials' image dependencies, models or placements. Next: explicit
custom foliage/material adaptation and instantiated source hierarchy/LOD/override
qualification, then full-scene and collision gates already listed above.

```sh
build/blender-tools/bin/python tools/preview_unity_static.py build/synty-enchanted-source \
  --prefab Assets/PolygonNatureBiomes/PNB_Enchanted_Forest/Prefabs/Props/SM_Prop_Ruins_01.prefab \
  --output build/enchanted-new-ruins --standard-material
build/asset-tools/bin/python tools/verify_unity_static.py \
  build/synty-enchanted-source build/enchanted-new-ruins
```

Qualified stage: `build/synty-enchanted-ruins-pbr-qualified`; native report/capture:
`build/synty-enchanted-ruins-qualified-native-20261020/ruins/`. Earlier PBR trial
output precedes corrected culling/sampling and is not this qualified stage.
Fresh fmt, denied-warning Clippy and **95 Python tests** (91 asset plus four GLB)
pass; the original Elven single-pillar conversion/source oracle also passes.
Rust code, dependency pins, application binaries and normal map stay unchanged;
Rust tests retain the previous 160-test checkpoint and were not rerun for these
offline tools. All assets/evidence stay ignored and changes local.

## Static hierarchy/LOD continuation

The [static source-placement planner](UNITY_STATIC_PLACEMENTS.md) now resolves
18,546 placed prefabs and independently checks 18,480 emitted records. It preserves
explicit unresolved variants/effects and source coordinates; it does not assemble
the map or qualify FBX axes. See that contract for exact tests and remaining gates.

## Static foliage material continuation

The [source-specific basic foliage adapter](FOREST_FOLIAGE.md) now converts and
natively renders the actual tree-card prefab and a vine LOD1 card. Derived colours,
alpha, branch geometry and standard PBR properties are checked independently.
Custom normal-map/full vegetation and assembled-scene gates remain open.

## Model-variant continuation

[Inherited alias and complete-TRS qualification](UNITY_MODEL_VARIANTS.md) now
resolves all 179 branch placements without guessing imported model poses. The
source plan emits 18,659 static records; geometry axes/materials and full-scene
adoption remain open. Remaining particle errors and omitted component families
are explicit.


## Geometry and first assembly continuation

[Independent mesh-local calibration and ordinary glTF assembly](UNITY_GEOMETRY.md)
now qualify five actual ruins placements plus an original affine fixture. Two
source LOD-centre mismatches remain explicit. Full ground uses binary Unity
TerrainData; its decoding, full vegetation, scene coverage and collision still
precede runtime adoption.


## Actual ground continuation

[Binary TerrainData conversion](UNITY_TERRAIN.md) now preserves all 263,169 source
height samples and five painted layers. Full-grid ground and ground-plus-five-
ruins pass independent checks, Khronos validation and native Bevy Vulkan captures.
The colour bake is an explicit approximation; terrain normal maps, full forest
materials/assembly and bounded authoritative collision still precede adoption.


## Ground collision and opt-in multiplayer continuation

[Four full-sample ground packages](FOREST_TERRAIN_COLLISION.md) now pass complete
independent geometry/collision/UV/normal coverage, actual Avian movement and two
native encrypted two-client seam cases. The development launcher supports this
explicit chunk-world option; the default town remains. Full vegetation/materials,
authored object collision and complete-map adoption remain open.


## Highest-detail full-shader continuation

[Static full vegetation PBR](FULL_FOREST_FOLIAGE.md) now qualifies the actual LOD0
small tree and fern, source cutouts, explicit normal maps and three native cases.
Three exact-binding trees and 612 ferns join the visual subset. The basic card
regenerates byte-identically. Four used full-shader profiles, motion/highlights,
other shader families and full scene/collision/adoption remain open.


The [bulk foliage continuation](FOREST_FOLIAGE_BATCH.md) supersedes the earlier
small subset: 15,156 exact source draws, shared images and two native final-scene
views now qualify. Remaining shader/material/collision/adoption work stays open.


The [Standard-material continuation](FOREST_STATIC_BATCH.md) adds 2,744 source
prop/rock draws to the visual subset, now 17,900 total. The source plan remains
unchanged; remaining variants/materials/collision/adoption work stays explicit.


The [canonical variant continuation](FOREST_VARIANT_GEOMETRY.md) now adds all
179 branch placements. The combined scene has 18,079 qualified source draws;
580 draws and full collision/resources/adoption remain open.


The [triplanar material prerequisite](FOREST_TRIPLANAR.md) now passes native
source-projection and independent diagnostic pixel checks. Its exposed FBX
mesh/root-pivot offset still needs calibration before full-scene placement;
material-prototype success does not advance the combined draw count or adoption.
