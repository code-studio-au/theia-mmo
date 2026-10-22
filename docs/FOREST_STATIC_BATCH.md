# Enchanted Forest Standard-material batch

P1-10/P2-08 continuation, 2026-10-20. This extends the
[qualified foliage batch](FOREST_FOLIAGE_BATCH.md) to built-in opaque Standard
materials from the same immutable source plan. Full scene collision, navigation,
resource budgets and multiplayer/default-map adoption remain separate acceptance
work. Keep the current town and opt-in forest-ground regression paths.

## Conversion contract

`preview_unity_static.py` retains its original simple-prefab interface and adds an
explicit draw-plan/instance/renderer path. The canonical path requires the existing
opaque Standard profile, actual source overrides and the highest-detail renderer.
It binds the Unity mesh fileID through reviewed ModelImporter IDs and selects that
specific Blender mesh. Nested/structurally changed prefabs, unsupported components,
deformed meshes and mixed material slots reject. Existing texture/scalar, culling,
blend/depth, emission and sampler requirements remain in force; transparency,
normal maps and other active Standard features are not silently discarded.

Scene placement already includes imported FBX model-node transforms. Canonical
mesh output therefore uses Blender's baked file-axis/unit conversion and clears
object-node placement before export. `unity_fbx_static.py` rejects nonidentity FBX
geometric transforms until separately qualified. This preserves original mesh-local
geometry instead of baking node placement twice. Independent ufbx checks compare
canonical positions for each successful binding.

The first real pebble trial exposed a source FBX node scale of 1.2966154173460949
being included in a supposedly canonical prototype. That trial is rejected.
The corrected pebble prototype differs from independent ufbx positions by at most
1.84e-8 metres. Its independent geometry/UV/normal and exact source palette checks
pass. This distinction matters for actual scene scale and later object collision.

`verify_unity_static.py` independently checks raw Standard source eligibility,
actual source mesh/material override bindings, exact embedded image pixels,
scalars/alpha/culling/sampling and source triangle normals. New reference geometry
uses exact f32 cyclic positions with the existing bounded UV/normal export policy;
legacy stages retain their earlier explicitly quantized comparison. The reference
implementation imports no material authoring adapter or draw selector.

`batch_unity_foliage.py --include-standard` enables this path explicitly while
preserving foliage-only behavior. It still validates the complete ordinary and
model-variant source plan and runs prototype/reference/ufbx/placement checks per
binding. Unsupported groups remain in the batch report. Final assembly uses the
terrain-only base because successful Standard conversion now includes ruins;
reusing the previous ground-plus-ruins base would duplicate those placements.
The independent final oracle checks every source renderer and omitted category,
base preservation, material/image semantics and mirrored attributes/winding.

## Scope and limits

The Standard adapter preserves the previously documented source scalar/albedo/
emission adaptation, not Unity shader, lightmap, BRDF or project colour-space
parity. Other shader families, transparent glass, mixed material slots and
unsupported foliage profiles remain explicit work. Per-renderer shadow/lightmap flags are not yet applied by standalone GLB inspection.
Existing foliage motion, tangent and ground-colour/normal limitations remain. Decorative source
geometry outside the ground rectangle does not expand playable world bounds.
This is an offline static visual scene, not an authoritative collision package.

## Reproduce

Use new output directories and the prerequisite pinned Blender/asset environments.

```sh
build/blender-tools/bin/python tools/batch_unity_foliage.py \
  build/synty-enchanted-source build/synty-enchanted-shader-source \
  build/synty-forest-static-variants-plan.json build/forest-new-static-batch \
  --probe build/ufbx-0.23.1/ufbx_geometry --include-standard
build/asset-tools/bin/python tools/assemble_foliage_batch.py \
  build/forest-new-static-batch build/synty-forest-terrain-v1/terrain.glb \
  build/forest-new-static-scene
build/asset-tools/bin/python tools/verify_foliage_batch_scene.py \
  build/forest-new-static-batch build/forest-new-static-scene \
  --report build/forest-new-static-scene/reference.json
```

This remains the full source plan with explicit unresolved classifications, not a
filtered plan that silently drops unsupported records. The four FBX-backed branch
variants account for 179 source draws and have no ordinary-prefab witness. Their
source poses were independently qualified earlier; canonical renderer/material
conversion for them remains separate from this ordinary-prefab slice.

## Offline qualification results

The initial complete run is retained in `build/synty-forest-static-batch-v1/`.
Two bindings (six placements) initially failed the source-unit-normal assumption:
`SM_Env_Tree_Trunk_01` has four triangle-corner samples of length down to
0.9973743201; `SM_Env_Log_01` has one of length 0.3333343863. Blender exports unit
normals. The independent Standard oracle now compares normalized source directions,
requiring finite nonzero source vectors and unit exported vectors. Geometry and
UVs are unchanged. Maximum direction errors for these two bindings are 8.37e-5
and 7.69e-5, within the existing 0.00018 export bound. Their independent ufbx
position errors are 1.61 micrometres and 0.353 micrometres. The corrected complete
stage is `build/synty-forest-static-batch-v2/`: prior qualified prototype/placement
bytes were copied unchanged, and those six placements received fresh reference,
ufbx and full placement checks. The first-pass rejected evidence remains intact.

The completed stage has **112 bindings / 17,900 source draws**, comprising the
prior 46 foliage bindings / 15,156 draws and **66 Standard bindings / 2,744 draws**.
The 40 unresolved bindings retain **759 source draws**. Their binding reasons are
23 other shader families, nine mixed material-slot bindings, four FBX-backed
variants, two unsupported foliage texture transforms, one transparent Standard
material and one active foliage emission/frosting profile. No source category was
removed from the original 18,659-draw plan.

`build/synty-forest-static-scene-v2/forest.glb` contains **5,151,519 triangles,
18,123 primitive instances, 117 source meshes, 138 materials and 31 shared image
variants**. This source-detail inventory is not a rendering budget or capacity
claim. The terrain-only base avoids the previous five-ruins duplication.
Khronos glTF Validator 2.0.0-dev.3.10 checks all 112 prototypes plus the final scene:
**zero errors**, 14 existing generated-tangent warnings and 117 informational
messages. The tangent warnings retain the qualified standard Bevy MikkTSpace
policy. All final external image resources are read. No quaternion repair is
needed in the new assembled file because pose generation now normalizes at export.

The legacy single-prefab ruins conversion remains byte-identical at SHA-256
`b9872ee03251e38980cfa2d929f148861043d40ee8488cf4a18b84ca24b8a617`.
Both legacy ruins and Elven pillar source oracles pass. Six additional real-stage
fault injections refresh file hashes, then verify that changed positions, UVs,
normals, surface scalars, source mesh IDs and source material paths still reject.
These private checks are additional to the normal unit test count.

All 66 successful Standard bindings receive a fresh independent source/material/
geometry/normal recheck under the final direction-normalization policy. Complete
final-scene verification covers all 17,900 source renderer IDs and all omitted
categories, with maximum source-pose arithmetic difference 9.95e-14 metres.
The largest canonical ufbx position difference across all 112 prototypes is
12.48 micrometres, below the existing 100-micrometre bound. Shared PNG files total
25,901,162 encoded bytes. The earlier two source LOD-centre calibration mismatches
remain explicit; no compensating transform or Unity Editor parity claim is added.

Fresh **150 Python tests** pass (143 asset, four GLB, one ordinary-prefab reference,
two model-variant reference); formatting and denied-warning workspace/all-target
Clippy pass. Source rechecks cover all 647 asset and 17 shader payload/metadata
pairs. No Rust application code, engine/plugin pins or runtime binaries changed;
prior Rust/networked-ground qualification remains separate. Everything stays local.

Final scene SHA-256:
`19b53090b45390217255c564d8d99d986a336ae0c83a41114166fba2206999b0`.
Completed batch report SHA-256:
`314ec06ee16f6a60bdf5ef7d08b94987c5877af9a0f6c3dd0195b1502f45f6b5`.
Source plan SHA-256 remains
`5639797b45f43f5a57a6e0e6f4c9b22fe7a50be7e2dc9ce152bdadc6059e8e9d`.

Ignored evidence includes `build/forest-standard-{all-reference,normal-completion}.log`,
each prototype's source/geometry/ufbx/placement reports, final `assembly.json` and
`reference.json`, `build/forest-static-gltf-validation-v2.json`,
`build/forest-standard-faults-v1.json`, `build/forest-standard-tests-final.log`,
`build/forest-standard-{reference-tests,glb-tests,clippy,source-recheck}.log` and
`build/forest-batch-source-recheck.json`.


## Native Bevy evidence

Two final native Linux cases (overview and closer forest view) pass in
`build/forest-static-batch-native-v1-20261020/`. The actual Bevy 0.20 client runs
on AMD Radeon RX 7900 XTX / RADV NAVI31 Vulkan in an owned private Wayland
compositor. Khronos validation activation is confirmed by loader trace. No VUID,
validation/renderer errors, panic or tangent-generation failure occurs. Pipelines
finish with no pending/failed entries and at least three clear frames.

Native inventory matches all 18,123 primitives, 5,151,519 triangles, 138 materials,
31 images and 117 source meshes. Actual transformed vertex bounds differ from
independent source calculations by at most 0.0151 mm. Both 1280×720 screenshots
were visually inspected; the closer view adds source rocks, trunks and mushrooms
to the qualified foliage and ground. The inspector's standard reference plane,
capsule and lighting are presentation aids, not source geometry or an authored
Unity lighting match. This is not a performance or native Windows qualification.

Client SHA-256 remains
`8c7baaff5e39a0848975ca3f4b019ceb231ce4ab631b29fc36419151be9bfaaf`.
The captures/reports and `build/forest-static-batch-native-v1.log` remain ignored.
Continue with the four model variants and remaining shader/material families,
then object collision and explicit resource/multiplayer acceptance before default
adoption. The complete roadmap remains active.


The [canonical variant continuation](FOREST_VARIANT_GEOMETRY.md) now adds all
179 branch placements. The combined scene has 18,079 qualified source draws;
580 draws and full collision/resources/adoption remain open.
