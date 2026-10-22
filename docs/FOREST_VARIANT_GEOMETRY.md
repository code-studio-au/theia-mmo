# Enchanted Forest model-variant geometry

P1-10/P2-08 continuation, 2026-10-20. This joins the previously qualified
[179 model-variant source poses](UNITY_MODEL_VARIANTS.md) to actual canonical
branch geometry and the [combined static scene](FOREST_STATIC_BATCH.md).
These are four single-FBX branch variants, not arbitrary nested Unity prefabs.
Complete scene materials, authored object collision, resource review and normal
multiplayer/default adoption remain separate requirements.

## Binding and reuse contract

`unity_draw_bindings.select_draw` has an explicit model-variant opt-in. Ordinary
foliage callers retain their prior behavior. The Standard canonical converter opts
in and uses the existing reviewed compiler: one static FBX mesh, checked inherited
IDs and source material, complete authored outer TRS, no unsupported structural
changes or deformation. Source scene overrides are applied after inherited IDs
are resolved. No imported default/root pose is invented.

`verify_unity_static.variant_renderer` independently derives renderer/mesh IDs and
source materials from raw variant records, FBX identities and importer metadata.
It imports neither the variant compiler nor draw selector. The existing independent
raw-scene reference checks outer material overrides and the hash-bound plan; the
whole model-variant reference oracle verifies all source poses/category completeness.
Each branch prototype also receives exact geometry/UV/normal/pixel checks,
independent ufbx positions and complete placed-instance geometry/winding checks.

`extend_unity_variants.py` validates a prior final scene and prototype/placement
hashes before copying its qualified artifacts into a new batch. It rechecks every
raw source payload/metadata pair and independently verifies the full variant source
category. It converts only unresolved FBX-backed Standard bindings, retaining the
complete original source plan and every remaining unresolved classification. New
outputs receive the same per-prototype checks as an ordinary batch. It records the
prior batch/scene hashes and checks input/source manifests again before completion.
Final assembly/source checks cover reused and newly converted records together.
The normal full-batch command also supports these Standard variants directly.

Tests check explicit opt-in, inherited renderer/mesh IDs, raw nested material versus
outer scene overrides, a rebound material-plan mismatch, wrong renderer IDs,
missing authored TRS and structural-change rejection. Previous ordinary-prefab,
full-foliage, source-normal, mirrored-pose and image contracts remain in effect.
The initial actual Branch_01 prototype agrees with independent ufbx positions
within 4.32e-8 metres and passes independent geometry/UV/normal/palette checks.

## Reproduce

Use fresh output directories and the pinned offline environments/probe.

```sh
build/blender-tools/bin/python tools/extend_unity_variants.py \
  build/synty-enchanted-source build/synty-forest-static-batch-v2 \
  build/synty-forest-static-scene-v2 build/forest-new-variant-batch \
  --probe build/ufbx-0.23.1/ufbx_geometry
build/asset-tools/bin/python tools/assemble_foliage_batch.py \
  build/forest-new-variant-batch build/synty-forest-terrain-v1/terrain.glb \
  build/forest-new-variant-scene
build/asset-tools/bin/python tools/verify_foliage_batch_scene.py \
  build/forest-new-variant-batch build/forest-new-variant-scene \
  --report build/forest-new-variant-scene/reference.json
```

All source/output assets and private proof reports stay in ignored `build/`.
The user queued a new Elven Realms source-folder inspection under Downloads /
Synty / Biomes **after Enchanted Forest is complete**. Do not treat the reported
folder as verified complete or switch away from unfinished forest work.

## Executed evidence

`build/synty-forest-variant-batch-v1/` adds four bindings with 48, 36, 39 and
56 placements: **179 total**. Their independent canonical ufbx position errors
range from 2.28e-8 to 4.33e-8 metres. Fresh source-category verification covers
all 179 poses. An additional byte comparison confirms the prior 112 converted
records and every prior preview/placement GLB, geometry reference and conversion
record remain exactly unchanged. The original full source plan is unchanged.

Final scene `build/synty-forest-variant-scene-v1/forest.glb` now has **18,079 source
renderers, 18,302 primitive instances, 5,171,337 triangles, 121 source meshes,
142 materials and 31 shared image variants**. The independent final oracle covers
every converted source renderer and all unresolved categories; maximum source-pose
arithmetic difference remains 9.95e-14 metres. No image variant or quaternion
repair is added. These source-detail counts are not production budgets/capacity.

The remaining **580 draws / 36 bindings** are retained explicitly: 23 other shader
families, nine mixed material-slot bindings, two unsupported foliage texture
transforms, one transparent Standard binding and one active foliage emission/
frosting profile. The next material audit identifies triplanar cliffs/rocks as the
largest outstanding shader family. Its reported draw/material pairs can overlap
mixed slots and must not be mistaken for unique source draw counts. Static source
coverage still excludes documented particle/behavior gaps; full forest adoption is
not established by resolving the model-variant category.

Khronos glTF Validator 2.0.0-dev.3.10 checks 116 prototypes and the final scene:
**zero errors, 14 existing generated-tangent warnings and 117 informational
messages**. Standard Bevy MikkTSpace remains the explicit absent-tangent policy.
All external scene images are read. Fresh **152 Python tests** pass: 145 asset,
four GLB, one ordinary-prefab reference and two model-variant reference tests.
Formatting, denied-warning workspace/all-target Clippy and whitespace checks pass.
All source payload/metadata pairs are rechecked at extension entry and exit.

Two native Linux cases (overview and closer forest view) pass in
`build/forest-variant-native-v1-20261020/` on **AMD Radeon RX 7900 XTX / RADV NAVI31
Vulkan**, with the actual Bevy client and private owned Wayland compositor.
Loader traces confirm active Khronos validation. No VUID, validation/renderer
error, panic or tangent-generation failure occurs. All pipelines finish with zero
pending/failed entries and at least three clear frames. Native inventory exactly
matches the final scene; actual transformed vertex bounds differ from the
independent oracle by at most **0.0151 mm**. Both 1280×720 captures were visually
inspected. The standard inspector lighting/reference plane/capsule are not authored
Unity lighting/source content. No native Windows or performance claim follows.

Scene SHA-256:
`57427527b260aeae02e07973ad284a77c50472411ce29c611a15df0aa1b58289`.
Batch report SHA-256:
`2fa35a470004a562f20abb4bbe4fc13a7bd26488c29f327216dd07586704edfa`.
Source plan SHA-256:
`5639797b45f43f5a57a6e0e6f4c9b22fe7a50be7e2dc9ce152bdadc6059e8e9d`.
Client SHA-256 remains
`8c7baaff5e39a0848975ca3f4b019ceb231ce4ab631b29fc36419151be9bfaaf`.
Rust application code/binaries/pins are unchanged; prior Rust, town and networked
ground results remain separate. No new full Rust test or multiplayer-forest claim.

Ignored evidence includes each new prototype's source/pixel/ufbx/placement reports,
`reused-scene-reference.json`, `variant-reference.json`, final `assembly.json` and
`reference.json`, `build/forest-variant-reuse-reference.json`,
`build/forest-variant-gltf-validation-v1.json`,
`build/forest-variant-{tests-final,reference-tests,glb-tests,clippy,native-v1}.log`,
and `build/forest-unresolved-materials-after-variants.json`.


The [triplanar material prerequisite](FOREST_TRIPLANAR.md) now passes native
source-projection and independent diagnostic pixel checks. Its exposed FBX
mesh/root-pivot offset still needs calibration before full-scene placement;
material-prototype success does not advance the combined draw count or adoption.
