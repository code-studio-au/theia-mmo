# Enchanted Forest foliage batch

P1-10/P2-08 continuation, 2026-10-20. Enchanted Forest is the selected replacement
for Elven Realm. This slice batches qualified static foliage from the actual demo
scene; it does not yet adopt the complete forest as the normal multiplayer map.
Prerequisites are the [source placement plan](UNITY_STATIC_PLACEMENTS.md),
[model variants](UNITY_MODEL_VARIANTS.md), [geometry calibration](UNITY_GEOMETRY.md),
[full vegetation profile](FULL_FOREST_FOLIAGE.md) and qualified ground/ruins.

## Source selection and acceptance

`unity_draw_bindings.py` selects an exact instance/renderer from the hash-bound
18,659-draw plan. It applies the actual source scene's material overrides and
requires the source-selected highest-detail LOD. It rejects structural/nested
prefab changes and ambiguous bindings. A separate reference implementation reads
raw scene records and checks the selected material slots; the whole-plan
mathutils and model-variant oracles independently establish source poses.

`batch_unity_foliage.py` groups exact mesh/material references, validates source
shader eligibility and chooses an ordinary source witness for each group. Every
converted prototype passes independent geometry/UV/winding, material/pixel/normal,
ufbx canonical-position and complete source-placement checks. Unsupported groups
retain their source references, draw counts and reasons. Lower LODs and fallback
materials are not substituted.

The full profile accepts the serialized `_ALPHATEST_ON`, `_NORMALMAP` and
`_EMISSION` flags only under the existing explicit cutout/normal and zero-emission
requirements. Empty YAML keyword values are normalized. Unknown flags reject.
The stale saved `_CameraFadeParams` is recorded as unused by the exact pinned
shader; its nonfinite value does not participate in rendering. Active properties
retain finite-value checks. These changes do not claim Unity include/platform
variant equivalence.

One exported large-tree leaf vertex had a zero tangent despite a nondegenerate
sole incident triangle. `foliage_tangents.py` derives that tangent from the original
positions/UV derivatives and normal, appending only a replacement tangent stream.
Ambiguous or degenerate cases reject. An independent linear-system calculation
checks its direction and handedness. Source geometry, UVs, normals and indices
remain unchanged. The earlier assembled v1 scene failed this check and is not
qualified evidence. Missing tangent streams use ordinary Bevy MikkTSpace generation.

`assemble_foliage_batch.py` joins successful bindings with qualified ground/ruins
and extracts shared PNG variants. Standalone `shared/...` paths remain inside the
inspector asset root. `verify_foliage_batch_scene.py` independently checks complete
converted/unresolved coverage, every source pose, winding, mirrored tangent sign,
all source attributes, material/image semantics and preservation of the base.
This produces a standard Bevy/glTF inspection asset, not a collision package.

## Remaining work

Other shader families, distinct per-renderer material slots, unsupported texture
transforms and active emission remain explicit conversion work. Static foliage
omits wind, bending, highlights and runtime LOD/crossfade. Eight selected background
cards have source cast-shadows disabled; their per-renderer flag is not yet applied
by the standalone GLB inspection path. Project colour-space, compression, mip/filter,
normal tangent and BRDF differences retain the prior profile's limits. Ground
still uses its documented finite colour bake and geometric normals.

Object collision/navigation, final resource budgets and complete native encrypted
multiplayer qualification precede map adoption. Ground packages already fill the
current per-package triangle allowance; do not silently add object meshes, raise
caps or decimate source ground. The existing town and opt-in forest-ground launcher
remain available. Licensed assets, generated scenes, captures and private source
reports stay in ignored `build/`; all changes remain local.

## Executed offline evidence

Completed batch `build/synty-forest-foliage-batch-v5/` qualifies **46 bindings and
15,156 source draws**. Its 106 unresolved bindings contain **3,503 draws**:
71 bindings outside the selected shader stage, 23 other shader families, nine
mixed material-slot bindings, two unsupported texture transforms and one active
emission/frosting profile. These are binding counts, not per-reason draw counts.
Four large-tree material variants each required one independently checked tangent
repair, covering 22 placements.

Final standalone scene `build/synty-forest-foliage-scene-v3/forest.glb` contains
**2,517,360 triangles, 15,384 primitive instances, 48 source meshes, 73 materials
and 17 shared image variants**. Shared encoded PNG bytes total 18,134,928. The
independent final oracle covers every converted source renderer and every omitted
binding; maximum world-position arithmetic difference is 9.95e-14 metres.
Decorative source foliage extends beyond the 150-metre ground: full visual bounds
are approximately X [-206.6073,180.0553], Y [-16.3224,87.8874],
Z [-198.7772,177.3804]. These are visual bounds, not authoritative playable bounds.

Khronos glTF Validator 2.0.0-dev.3.10 checks 46 prototypes and the final scene with
**zero errors**. Fourteen warnings are exclusively generated tangent space (seven
prototype primitives and their seven assembled equivalents). Standard Bevy
MikkTSpace is the selected native path; cross-implementation tangent parity
remains unproven. There are 101 informational messages, including external GLB
image URIs, unused UVs and unused tangents. All 17 external resources are read.
The validator caught 13 quaternion components at ±1.0000000000000002; final
assembly normalizes only that bounded unit rounding, recorded by node ID. Future
pose factorization now normalizes at generation too. Source-pose verification
passes unchanged. Earlier scene-v2 retains its rejected validation result.

Fresh **149 Python tests** pass: 142 asset, four GLB, one ordinary-prefab reference
and two model-variant reference tests. These include rebound source material/
plan rejection, complete omitted-category coverage, changed placement/mirrored
tangent rejection, sole-triangle tangent derivation, invalid/ambiguous basis
rejection and bounded quaternion normalization. Fresh formatting and denied-warning
workspace/all-target Clippy pass. The basic card conversion remains byte-identical
at SHA-256 `dbf3b32a367f9f9737e0f2e5ca529bcbc55beacf44829499824d70c04051da24`.
All 647 source payload/metadata pairs and 17 shader payload/metadata pairs pass a
fresh independent hash/GUID recheck. Rust application code, binaries and pins are
unchanged in this slice; earlier Rust/ground-network qualification is separate.

Final scene SHA-256:
`6ff1e0a32af91a89d079e1a434e61f662f84070f442626c7358755c6e3505019`.
Source plan SHA-256:
`5639797b45f43f5a57a6e0e6f4c9b22fe7a50be7e2dc9ce152bdadc6059e8e9d`.
Completed batch report SHA-256:
`92f6a68a94588903220d549cdb0dd1e626e2e985682b185f0db0c7a27a7ac1ea`.

## Reproduce locally

Use fresh output directories; commands require the pinned isolated asset/Blender
environments and ufbx probe described in the prerequisite contracts.

```sh
build/blender-tools/bin/python tools/batch_unity_foliage.py \
  build/synty-enchanted-source build/synty-enchanted-shader-source \
  build/synty-forest-static-variants-plan.json build/forest-new-batch \
  --probe build/ufbx-0.23.1/ufbx_geometry
build/asset-tools/bin/python tools/assemble_foliage_batch.py \
  build/forest-new-batch build/synty-forest-terrain-ruins-v1.glb build/forest-new-scene
build/asset-tools/bin/python tools/verify_foliage_batch_scene.py \
  build/forest-new-batch build/forest-new-scene --report build/forest-new-scene/reference.json
target/debug/theia-client --inspect-asset build/synty-forest-foliage-scene-v3/forest.glb \
  --focus 0,5,0,12,1.0,0.25
```

Ignored evidence includes each prototype's conversion, independent geometry/pixel,
ufbx and assembled-source reports; final `assembly.json` and `reference.json`;
`build/forest-batch-gltf-validation-v3.json`, `build/forest-batch-tests-final.log`,
`build/forest-batch-clippy.log`, and `build/forest-batch-source-recheck.json`.

## Native Bevy evidence

The final scene-v3 bytes pass two native Linux cases (overview and closer forest
view) in `build/forest-batch-native-v2-20261020/`. Both run the actual Bevy 0.20
client on **AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan**, in a private owned
Wayland compositor. Loader traces confirm active Khronos validation. No Vulkan
VUID/validation errors, renderer errors, panic or tangent-generation failures occur.
Pipelines finish with no pending/failed entries and at least three clear frames.
Native inventory matches the complete offline inventory, and actual transformed
vertex bounds differ by at most **0.0151 mm** from the independent source oracle.
Both 1280×720 screenshots were visually inspected; the closer view shows source
forest trunks, cutout foliage, ferns and terrain under ordinary inspector lighting.
The inspector adds its standard reference plane/capsule; those are not imported
source geometry. Inspector lighting is not an authored Unity lighting match.

Client SHA-256 remains
`8c7baaff5e39a0848975ca3f4b019ceb231ce4ab631b29fc36419151be9bfaaf`.
These checks establish this static inspection subset on this Linux GPU. They do
not establish native Windows execution, rendering performance/capacity, networked
foliage, authored object collision or complete scene adoption.


The [Standard-material continuation](FOREST_STATIC_BATCH.md) adds 2,744 source
prop/rock draws to the visual subset, now 17,900 total. The source plan remains
unchanged; remaining variants/materials/collision/adoption work stays explicit.
