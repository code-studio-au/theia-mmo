# Elven Realm import checkpoint

Superseded as the active environment target on 2026-10-20 by the user-selected
[Enchanted Forest](SYNTY_ENCHANTED_FOREST_IMPORT.md). This record preserves
historical evidence; missing Elven sources are no longer an active prerequisite.

P1-10/P2-08, 2026-10-20. The user's Synty folder contains an assembled Elven
Realm demo as well as individual FBX models. This checkpoint qualifies source
inspection and one static base-colour preview. It does not export the complete
scene, install a new playable map or establish collision/navigation fidelity.

## Source/dependency audit

The explicit extractor stages 2,276 selected payloads and their 2,276 adjacent
`.meta` files under ignored `build/synty-elven-source`. Payload and metadata hashes,
source archive hash and original paths are recorded; selected code is never run.
`--include-meta` requires complete metadata, rejects destination overlap and
counts metadata against the same 512 MiB aggregate extraction budget.

`tools/unity_scene.py` reads bounded Unity YAML records, preserves signed source
IDs and stripped markers, and rejects duplicate IDs/keys, YAML aliases and object
constructors. It follows hash-bound extracted dependencies recursively. This is a
dependency resolver, not a prefab-instantiation implementation.

The actual scene has 33,891 records, including 15,416 serialized prefab instances
and 15,528 transforms. Its dependency closure visits 1,626 staged assets, including
525 prefab documents, 572 FBXs and 96 materials. These are serialization/source
counts, not resolved render instances, draw calls, collision bodies or residency.

The visited YAML contains 15,625 prefab-source edges; 378 inherit from FBX model
prefabs. All 378 lack an `internalIDToNameTable` in their model metadata. Do not
infer a multi-mesh object's identity from enumeration order. A removed component
also occurs; additions, nested references, active flags and LOD selection must be
handled by a future instantiated export rather than discarded.

Twelve serialized external GUIDs are outside the selected staging set. Eleven
are absent from the supplied Elven archive inventory; one is the deliberately
unselected `SimpleRotate.cs`. Contexts include a mesh-collider mesh, a particle
shape mesh, shader/script dependencies and material texture properties. Some
properties may be unused by their active shaders; these counts do not establish
twelve missing runtime requirements. Report unresolved references and decide
explicit source replacements/omissions before claiming scene equivalence.
Unity built-in GUID references are counted separately.

## Qualified representative preview

`tools/preview_unity_static.py` accepts one simple active static prefab, identity
root transform, one unambiguous FBX mesh and repeated binding to one material.
It verifies extracted source hashes, GUIDs, model import units/axis settings,
sRGB PNG and repeat/linear texture settings. Nested model variants, arbitrary
transforms and ambiguous multi-mesh fileIDs reject instead of guessing. The
single-mesh case proves source association, not Unity's hashed-ID algorithm.

The actual `SM_Bld_Pillar_01` converts through Blender 5.2.2 LTS to standard GLB:
204 imported source vertices, 404 triangles, 786 split render vertices, one mesh,
one standard material and one 4096×4096 source palette. Exported bounds are about
0.646×3.006×0.646 metres, Y-up. The existing blue 1.8 m comparison capsule remains
part of the inspector only.

The required `--base-color-preview` flag explicitly permits a material
approximation: source base colour, metallic 0 and roughness 0.8. Normal,
metallic/smoothness, emission and shader-specific effects are omitted. This is
not a complete reconstruction of `PolygonElven_Mat_01_A` or Unity shader output.
No scene handedness/hierarchy mapping is established by this identity-prefab case.

`tools/verify_unity_static.py` independently reads the GLB and compares all
triangle positions, winding and UVs against the Blender-imported FBX reference at
10⁻⁵ coordinate/UV quantization. Exported palette RGBA pixels match the source
exactly. This tests export fidelity from the imported source; it is not a Unity
Editor reference comparison. Khronos validation reports **zero errors, warnings
and informational messages**.

One actual standard Bevy inspector capture passes on **AMD Radeon RX 7900 XTX,
RADV NAVI31, Vulkan**, with active Vulkan validation, compiled pipelines and no
validation errors. The screenshot was visually inspected. This is Linux native
evidence; no Windows execution or full-map performance claim follows.

## Reproduce with local private sources

```sh
build/asset-tools/bin/python tools/unity_scene.py build/synty-elven-source \
  --scene Assets/PolygonElvenRealm/Scenes/Demo_01.unity \
  --report build/elven-new-audit.json
build/blender-tools/bin/python tools/preview_unity_static.py build/synty-elven-source \
  --prefab Assets/PolygonElvenRealm/Prefabs/Buildings/SM_Bld_Pillar_01.prefab \
  --output build/elven-new-preview --base-color-preview
build/asset-tools/bin/python tools/verify_unity_static.py \
  build/synty-elven-source build/elven-new-preview
```

Use the isolated converter dependencies in `tools/synty-conversion-requirements.txt`.
All output directories/reports must be new where required. The actual qualified
stage is `build/synty-elven-pillar-canonical-preview`, and the native report/capture
is under `build/synty-elven-pillar-qualified-native-20261020/pillar`.

## Remaining acceptance

1. Resolve model-prefab fileIDs and nested hierarchy against a verified imported
   model/Unity reference; apply stripped references, overrides, removals/additions,
   inactive objects and highest-detail LOD selection with observable rejection of
   unresolved visual changes.
2. Qualify coordinate handedness and nested/nonuniform transforms with asymmetric
   source fixtures. Keep shared geometry/images when assembling the actual demo.
3. Implement deliberate standard Bevy materials and declared approximations for
   water, vegetation, triplanar and other Unity shader effects; investigate actual
   unresolved serialized references before claiming completeness.
4. Independently compare assembled placements/geometry/material bindings and run
   native full-scene inspection with measured resources. Qualify collision,
   navigation and normal game adoption separately.

The character preview remains separate. The core world-edge, actual-town client
resources and handover lifecycle gates remain open. No task/phase completion or
new active map is claimed. All private source/output assets stay ignored; no
Rust client/server or engine pins changed in this checkpoint.

## Model identity and missing payload continuation — 2026-10-20

The initial absence of a named metadata table is no longer a blanket obstacle to
source identification. `tools/unity_model_ids.py` reads bounded binary FBX model,
mesh and hierarchy identities and checks modern signed 64-bit XXHash candidates
against the actual serialized `(source asset, fileID)` pairs. The derivation uses
[a Unity engineer's explanation](https://discussions.unity.com/t/fbx-submesh-fileids/803882),
not a stable Unity API. It must continue rejecting unmatched IDs rather than
assigning a candidate based on source order. Model-path and connected-model mesh
names are checked against this source, not claimed as universal importer rules.
The source's older candle-flame FBX instead uses its hash-bound explicit metadata
ID/name table, with exact source mesh-name matching.

The FBX reader handles 32/64-bit record offsets, signed IDs, cycles, duplicate
paths/IDs and bounded scalar identity properties. It does not decompress geometry
or evaluate source code. Display-layer `CollectionExclusive` membership is distinct
from transform parentage, as also reflected in
[Autodesk's collection API](https://help.autodesk.com/cloudhelp/2019/ENU/FBX-Developer-Help/cpp_ref/class_fbx_collection_exclusive.html).
Actual multi-mesh telescope inspection confirms eleven source model/mesh bindings
and nested paths; its Blender import hierarchy was separately inspected. This is
identity evidence, not full-scene transform or shader qualification.

Current results match **1,109 distinct asset/fileID references**: 517 Mesh,
222 MeshRenderer, 185 Transform and 185 GameObject. The 62 unmatched references
remain explicit: 44 refer to unavailable model payloads and 18 lack a matching
candidate in otherwise readable source models. The latter mostly occur in prefab
override targets; they must be reconciled before asserting complete instantiation.
Counts describe distinct serialized references, not the 15,416 placed instances.

Crucially, **32 selected source payloads are Git LFS pointers**, comprising
30 `.fbx` and two `.png` files, declaring 1,429,558 missing bytes in total. The
actual demo dependency closure reaches **26 of those FBXs and one PNG**. The
archive contains the short pointer text, not the referenced asset bytes. Hash
verification of that text cannot establish available geometry or image data.
The original archive hash remains unchanged. Full scene-equivalent export needs
the missing blobs from a complete source download; no replacement models were
silently substituted and no source files were changed.

`Sources.lfs_pointers()` reports the path, GUID, pointer hash, expected blob SHA-256
and declared size. The dependency report marks which are in the demo closure.
Private reports: `build/synty-elven-dependency-vetted.json` and
`build/synty-elven-model-ids-vetted.json`. The earlier
`build/synty-elven-lfs-pointers.json` preserves the full missing-payload list.
Pinned offline dependencies now include PyYAML 6.0.3 and xxhash 3.6.0 in both
appropriate requirements files. No runtime dependency was introduced.

Reproduce the identity check after producing a new dependency report:

```sh
build/asset-tools/bin/python tools/unity_model_ids.py build/synty-elven-source \
  --dependencies build/elven-new-audit.json --report build/elven-new-model-ids.json
```

Fresh formatting, denied-warning Clippy and **85 Python tests** pass. No Rust
runtime or rendering change requires a new GPU run in this continuation; the
preceding pillar/character native captures remain their own scoped evidence.
The full-scene asset gate stays open. Core world-edge/client-resource work can
continue while missing source payloads are located.
