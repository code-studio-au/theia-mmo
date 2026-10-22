# Static Unity source placement plans

P1-10/P2-08, Enchanted Forest continuation, 2026-10-20. This slice requires the
hash-bound source/dependency audit and explicit model/fileID references. It adds
working source-hierarchy and override resolution for ordinary static prefabs.
It is an offline precursor to assembled GLB/world content, not a renderer,
instantiated full scene, physics import or active game-map replacement.

## Implemented contract

`tools/unity_prefabs.py` resolves source parent/child links, GameObject component
ownership, inherited active flags, renderer enablement, mesh/material references,
local TRS and supported instance overrides. Matrices use f64 source arithmetic,
including nested rotation and nonuniform scale; they remain in **Unity source
coordinates**. FBX importer geometry/axes and glTF handedness are separate gates.
Input uses the existing bounded YAML reader and hash/metadata-bound `Sources`.
Hierarchy depth is bounded at 128 and expanded draw records at 262,144. Cycles,
inconsistent links, unresolved parents/LOD references and nonfinite/invalid
rotations reject. A failed instance contributes no partial draw records.

Static planning chooses the first serialized, highest-detail LOD. It checks
renderer membership and excludes lower-detail alternatives without changing
ordinary non-LOD renderers. This is an explicit offline selection, not runtime
LOD thresholds, crossfading, performance optimization or an MMO residency claim.
See Unity's [LOD Group reference](https://docs.unity3d.com/Manual/class-LODGroup.html).

Overrides support local position/quaternion/scale, authoring Euler hints/root
order, names/layers/active flags, renderer/collider enablement, cast-shadow flags
and existing material slots. Supported flags are deliberately bounded. Unknown
properties on existing objects, foreign source GUIDs, array resizing, structural
additions/removals, nested/model prefabs and particle-system overrides remain
explicit errors. Instance preparation retains original source records unchanged.
Scene-parent composition follows ordinary and stripped Transform references back
to their source instance/object, with cycle checks across those relationships.
Unity documents the serialized relationship in its
[prefab YAML reference](https://docs.unity.com/en-us/engine/7000.0/manual/working-with-scenes/text-scene-format/yaml-prefab-serialization).

**Unused override handling is narrow.** A target with the current prefab GUID but
an absent local fileID is recorded as unused only after proving that the prefab
contains no nested/model/stripped inheritance. Its value is not applied or used as
a replacement. Unknown properties on existing records continue rejecting. Nested
variants are rejected before this classification because their IDs can be inherited.
Unity documents invalid-target data as [unused overrides](https://docs.unity.com/en-us/engine/6000.7/manual/working-with-gameobjects/prefabs/override/unused-overrides).
No source overrides or files are deleted.

## Actual Enchanted Forest execution

`build/synty-forest-static-instance-plan-v2.json` processes the actual Demo_01
scene's 18,755 placed prefab instances. It resolves **18,546 instances** and emits
**18,480 static draw records**, including two inline scene renderers. It selects
16,184 placed LOD groups and excludes 5,276 lower-LOD renderer references. These
are offline source-plan counts, not Bevy entities, draw calls or loaded meshes.

The report retains **423 unused flat-prefab target overrides**. It also retains
**209 unresolved placements**: 179 placements of the four FBX-backed branch
variants and 30 placements with unsupported particle overrides. No fallback mesh,
material, guessed model-root transform or silent complete-scene claim is made.
Components outside the implemented static subset are separately counted, including
128 CapsuleCollider occurrences and remaining particle components; scene lights,
settings, camera, audio and scripts are likewise recorded as unimplemented here.
They are not imported into gameplay or executed. All-source material/geometry
qualification and omission-free full-scene acceptance remain open.

The first strict run is retained as `build/synty-forest-static-instance-plan.json`:
18,122 instances resolved, 633 unresolved. A separate target audit proved 423
missing IDs belong to flat sources; the remaining 2,506 missing-ID modifications
belong to nested variants and retain their unresolved classification. The v2
result uses the documented unused-target semantics, not a broad error bypass.

## Independent reference and acceptance

`tools/verify_unity_prefabs.py` imports no planner implementation. It rereads
hash-bound source YAML, independently binds emitted renderer/mesh/material/LOD
and active-state references, and compares every emitted world matrix against
Blender's `mathutils.Matrix.LocRotScale` and parent composition. All **18,480**
records pass. The maximum absolute matrix-entry difference is
**0.0000227346793**, within the explicit 0.0001 threshold for the f32 mathutils
reference versus f64 planning. This is not a Unity Editor comparison, an omitted
instance proof, an FBX-axis/vertex comparison or a rendered scene qualification.

Seven asset tests cover nonuniform rotated parent/child points, LOD selection,
immutable material/transform overrides, ancestor inactivity, renderer disablement,
stripped scene-parent bindings, cycles, broken ownership, invalid references and
numeric/unsupported cases. One separate Blender-environment reference test uses
hand-authored expected matrices and rejects changed positions/materials, lower
LODs, inactive scene parents and NaN. Fresh **103 distinct Python tests** pass:
98 asset tests, four GLB tests and that separate reference test. Fresh fmt and
denied-warning workspace/all-target Clippy pass. Rust runtime, binaries and pins
are unchanged; prior Rust/native qualifications retain their separate scopes.
No new GPU execution is required for these non-rendering source-plan tools.

```sh
build/asset-tools/bin/python tools/unity_prefabs.py build/synty-enchanted-source \
  --scene Assets/PolygonNatureBiomes/PNB_Enchanted_Forest/Scene/Demo_01.unity \
  --output build/forest-new-plan.json
build/blender-tools/bin/python tools/verify_unity_prefabs.py \
  build/synty-enchanted-source build/forest-new-plan.json \
  --report build/forest-new-reference.json
build/blender-tools/bin/python tools/test_unity_prefab_reference.py
```

Evidence: plan v2 and `build/synty-forest-static-reference-qualified.json`,
`build/synty-forest-override-target-analysis.json`,
`build/synty-forest-prefab-{unit-tests,reference-tests,python-assets,python-glb,fmt,clippy}.log`.
The earlier reference v2 report predates the additional scene-parent-active/NaN
checks; the `qualified` report includes them. All private source/plans remain
ignored under `build/`, all repository changes local.

Next acceptance: resolve FBX-backed variant object bindings/transform semantics;
adapt custom forest foliage/materials; qualify geometry/importer coordinates with
asymmetric source fixtures; then assemble and independently compare actual scene
geometry/placements/materials. Source collision and normal multiplayer/camera
qualification still precede replacing the town launcher.

## Model-variant continuation

The [bounded model-variant compiler](UNITY_MODEL_VARIANTS.md) now binds the four
branch variants using inherited aliases and complete outer TRS overrides. Its
179 new records pass an independent category/reference check; all 18,480 older
records remain exactly unchanged. The current source plan resolves 18,725 prefab
instances, with 30 particle-override errors. Earlier counts above remain the
historical checkpoint; geometry/importer and full-scene gates remain open.
