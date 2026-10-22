# FBX-backed static model variants

P1-10/P2-08 source-placement continuation, 2026-10-20. This adds the four
Enchanted Forest branch variants to the existing static source-coordinate plan.
It requires checked FBX identities, hash-bound prefab/scene metadata and the
ordinary hierarchy/override resolver. Geometry import, handedness, materials and
collision still have their own acceptance gates.

## Inherited identity contract

The branch variants inherit objects whose IDs do not appear as full local YAML
records. Their object aliases are `(instanceID XOR sourceObjectID) & 0x7fff...ffff`.
A [Unity engineer documents this variant-ID rule](https://discussions.unity.com/t/how-is-fileid-generated-for-m_correspondingsourceobject/754235).
`tools/unity_prefab_ids.py` handles signed 64-bit inputs, rejects nulls/collisions,
and separates hierarchy-object aliases from mesh subassets. Candidate IDs must
match actual serialized references; the derivation is not a general promise of
Unity importer equivalence or stable public API behavior.

The actual stripped GameObject IDs and component owners match that rule in all
four source branch variants. All **2,864 outer scene modifications** across their
**179 placed instances** bind: 358 GameObject and 2,506 Transform targets. No
variant audit error or unmatched target remains. Mesh/material/subasset references
retain their original source GUID/fileID; they are not assigned by file order.

## Bounded source-plan compilation

The new compiler accepts only one static FBX model with one mesh, no imported
animation, visibility or blend shapes, and disabled model material import. A
source renderer material override must exist explicitly. Stripped object
references and locally added collider ownership are validated before compilation.
Unknown source overrides, ambiguous components, structural changes and other
model profiles reject. Added collider records are preserved as source data;
this does not cook or qualify collision.

Crucially, every final root position/quaternion/scale component must appear in
the **outer scene instance's overrides**. All 179 actual branch placements supply
those ten components. Missing, duplicate or object-valued TRS overrides reject.
The compiler never fills missing pose components from an assumed imported FBX
root transform, scale or axis policy. Pose matrices remain in Unity source
coordinates; model geometry conversion is still separate. The material/name
inheritance and scene active/renderer/material overrides feed the existing
static hierarchy resolver, using the source-derived aliases.

Only this supported profile is materialized. Other nested or partially overridden
model instances remain errors; this is not a general Unity prefab importer.
The existing flat-source unused-target rule does not discard inherited targets.

## Actual scene and independent checks

`build/synty-forest-static-variants-plan.json` now resolves **18,725 of 18,755**
placed prefab instances and emits **18,659 static records**. The new records are
179 branch placements. All original 18,480 records, 423 unused flat-target
overrides, inline-error results and the remaining 30 particle-override errors
are exactly unchanged. Counts are source plans, not Bevy entities or draw calls.
Other recorded unimplemented scene/prefab components remain separate; 30 errors
does not mean only 30 visual elements remain before full-scene equivalence.

`tools/verify_unity_model_variants.py` imports neither compiler nor planner. It
rereads model/prefab/scene sources, independently computes aliases and authored
TRS with mathutils, follows ordinary and stripped scene-parent chains, and
checks mesh/material/transform identity. It also checks the expected category's
placement set, rejecting omitted or duplicate branch placements. All 179 pass;
maximum absolute matrix-entry difference is **0.0000147762358**, within the
existing 0.0001 f32-mathutils versus f64-source threshold. This reference does not
prove FBX vertex coordinates, Unity Editor parity, collision or whole-scene
absence of omissions. The old 18,480 records retain their preceding independent
reference evidence and now pass exact unchanged-record comparison.

Four asset tests cover actual signed-ID anchors, null/range/collision rejection,
stripped/source/owner mismatches, explicit TRS compilation and retained collider
records. Two separate Blender reference tests check category completeness,
geometry identity, poses, material aliases, NaN, stripped nonuniform/rotated
parents and inactive-parent overrides. Fresh **115 distinct Python tests** pass
(108 asset, four GLB, one prior Blender reference and two new model reference
tests), plus fmt and denied-warning workspace/all-target Clippy.

The initial new reference only supported ordinary scene parents and rejected
actual stripped-parent cases; the qualified reference supports and tests both.
An early synthetic material-reference test exposed an unnecessarily exact pointer
`type` dictionary comparison; matching now uses source GUID/fileID. No source or
planner geometry/identity acceptance was loosened for those reference corrections.

No Rust runtime, binary, dependency pin, renderer or default-map change occurred.
No new native GPU or Rust test rerun is claimed for these offline source-plan
tools; prior map/native and Rust evidence remains separate. All proprietary
sources/plans stay ignored and changes local. The goal/roadmap remains active.

## Reproduce with private sources

```sh
build/asset-tools/bin/python tools/unity_prefab_ids.py build/synty-enchanted-source \
  --dependencies build/synty-enchanted-dependencies.json \
  --report build/forest-new-variant-identities.json
build/asset-tools/bin/python tools/unity_prefabs.py build/synty-enchanted-source \
  --scene Assets/PolygonNatureBiomes/PNB_Enchanted_Forest/Scene/Demo_01.unity \
  --output build/forest-new-model-plan.json
build/blender-tools/bin/python tools/verify_unity_model_variants.py \
  build/synty-enchanted-source build/forest-new-model-plan.json \
  --report build/forest-new-model-reference.json
build/blender-tools/bin/python tools/test_unity_model_variant_reference.py
```

Evidence: `build/synty-forest-variant-identities-qualified.json`,
`build/synty-forest-static-variants-plan.json`,
`build/synty-forest-static-variants-reference-qualified.json`,
`build/synty-forest-model-plan-regression.json` and
`build/synty-forest-{variant-unit-tests,model-reference-tests,model-python-assets,model-python-glb,model-prior-reference-tests,model-fmt,model-clippy}.log`.
Next: source geometry/importer coordinate qualification and full vegetation
materials before actual scene assembly, then collision/multiplayer/camera
qualification before replacing the working town launcher.
