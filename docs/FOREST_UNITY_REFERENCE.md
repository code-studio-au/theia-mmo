# Unity mesh/root reference prerequisite

Subsequent 2026-10-21 work applies the recorded frames to thirteen existing
placements and the 85-instance moss experiment, with independent/native checks.
See [mesh-frame continuation](FOREST_MESH_FRAMES.md). The checkpoint below
preserves the original reference collection and safe-stop evidence.

P1-10/P2-08 continuation, 2026-10-20. The forest moss placement exposes a gap
between verified FBX mesh-local geometry and Unity's actual model import. The
[triplanar prototype](FOREST_TRIPLANAR.md) remains an inspector experiment until
this is resolved. Serialized LOD bounds match 52/54 groups, but do not cover the
problematic moss model. The two old centre mismatches have identity FBX mesh-node
transforms; neither establishes a general root-translation correction.

The user's **Unity Editor 6000.6.6f1** installation now runs this collector in
native Linux batch mode. Actual C# compilation and the initial thirteen-model
export pass. Unity Hub and the separate Unity CLI alone do not provide the
imported mesh data required by this check.

## Reference collection

`tools/unity_editor_reference.py stage` creates a fresh project **inside ignored
`build/`**. It copies only explicitly selected, hash-checked FBX data and original
metadata, the repository's reviewed `TheiaMeshReference.cs`, an empty package
manifest and the exact requested Editor version. No source scenes, scripts,
materials or package code are staged. Original metadata also remains outside
`Assets/` for comparison after import. Sources are never edited.

The collector uses Unity Editor's
[MeshUtility.AcquireReadOnlyMeshData](https://docs.unity3d.com/2022.3/Documentation/ScriptReference/MeshUtility.AcquireReadOnlyMeshData.html),
which permits read-only access without enabling the model's Read/Write flag.
It records actual signed persistent mesh/transform IDs as decimal strings, used
triangle indices, mesh vertices/bounds, model hierarchy and explicit row-major
local/world matrices. The report binds actual Editor version/platform, source
hashes, current metadata hashes, request and collector hashes. It does not run a
game scene, change importer settings or export materials, animation, normals/UVs,
lighting or collision.

`inspect` checks those bindings against the original extracted source, checks
source-derived mesh IDs against actual Editor IDs, and rejects malformed bounds,
indices, duplicate identities, inconsistent matrices, missing parents and cycles.
A pinned independent ufbx probe compares **both directions of used point sets**
within 0.1 mm after the previously calibrated X reflection. It reports three
diagnostic alternatives separately: mesh-local geometry, full geometry-to-world,
and root translation expressed in the node's local frame. Identity roots can
match multiple alternatives; the tool never selects a correction automatically.
It additionally compares geometry using the actual recorded Unity node frame:
`UnityLocal = inverse(UnityWorld) * reflectX * FbxGeometryWorld * FbxPoint`.
The mesh and its transform must both match source-derived persistent IDs.
This handles frame translation/rotation/scale without guessing a mesh centre.
Point sets do not establish topology, winding, normals or material equivalence.

Metadata rewritten by a newer Editor is reported as **not byte-identical**.
Review actual setting differences before using those findings to change
conversion. A changed metadata hash does not prove equivalent import behaviour.

## Initial actual reference and reproduction

`build/forest-editor-reference-v1/` contains **13 actual models**: all four moss
lumps, six pebble models, the portal and trunk involved in the old LOD-bound
mismatches, and one ruins model. This tests the suspected root offset alongside
the earlier scale and identity-root cases. Stage receipt:
`build/forest-editor-reference-stage-v1.json`.

Request SHA-256:
`bf7fd74688e284c0fff9d9460281179d3b276c4c5903dad21a7c864247fc8a4e`.
Collector SHA-256:
`0ccce450f462f00817365722086a4172d149c6df9bb9291766f510e1f106c67e`.

The installed binary at
`/home/steven/Unity/Hub/Editor/6000.6.6f1/Editor/Unity` executed in batch mode.
The collector compiled, exited zero and logged the completion marker. The
bound initial report contains **122,367 vertices / 60,274 triangles / 15 meshes**.
All thirteen metadata files remain byte-identical to the originals. Every mesh
matches the independent FBX geometry expressed in the actual Unity frame, with
maximum point-set difference **1.537 micrometres**.

The first three moss lumps match root translation expressed in the local frame;
the fourth requires the actual Unity root frame (X translation -0.23970115 m).
Both trunk LODs require subtracting their Unity child Z translation 0.75383973 m
from mesh-local vertices. Six pebble meshes retain the existing mesh-local
geometry; applying their FBX scale twice remains incorrect. The portal meshes
also match the existing mesh-local data, so their old LOD-centre discrepancy is
not explained by these root corrections.

Only **nine of fifteen** meshes match the unadjusted mesh-local alternative.
The existing static batch contains **two trunk placements** affected by the
new frame evidence. They remain in the preserved experiment; no corrected
replacement is promoted in this slice. The moss experiment remains unqualified
as assembled world geometry.

Final initial inspection: `build/forest-editor-reference-inspection-v3.json`,
SHA-256 `acee6c5518e0e63e4d4f9477a1f33a6f2f6e38c9853d98a018b73d2ef295f387`.
Actual Editor output SHA-256:
`fa8068dac1a714d0b50e63b42b0934c3df1b385a6e3c0ac74a73103d7a9f5bdd`.
Startup logs include a nonfatal access-token update diagnostic; local collection
completed. This is a nographics/null-device Editor import, not a Unity GPU check.

The original execution command follows. Use a fresh stage/output for a new run;
the collector rejects overwriting an existing report.

```sh
"$unity_editor_path" -batchmode -nographics \
  -projectPath "$PWD/build/forest-editor-reference-v1" \
  -executeMethod TheiaMeshReference.Run \
  -theiaReferenceRequest "$PWD/build/forest-editor-reference-v1/Reference/request.json" \
  -theiaReferenceOutput "$PWD/build/forest-editor-reference-v1/Reference/imported.json" \
  -logFile "$PWD/build/forest-editor-reference-v1/Reference/editor.log"

build/asset-tools/bin/python tools/unity_editor_reference.py inspect \
  build/synty-enchanted-source build/forest-editor-reference-v1 \
  --probe build/ufbx-0.23.1/ufbx_geometry \
  --report build/forest-editor-reference-inspection-v1.json
```

Use a fresh project/report when changing collector, source selection, Editor
version or importer settings. Do not rebind a failed report to altered inputs.
No installation, account login or licence activation is performed by these tools.

## Executed checks and next acceptance

The remaining **99 scene models** subsequently completed in four bounded groups
(32/32/32/3) before the safe stop. All five Editor runs compiled/exited zero and
all five independent comparisons completed. Combined coverage is **112 FBXs /
214 meshes**, including every **118 FBX-backed mesh binding** used by the source
scene plan. Built-in Unity meshes remain outside this FBX audit. All 112 metadata
files remain byte-identical; all 214 recorded-frame point sets match, with global
maximum difference **11.02 micrometres** against the 100-micrometre bound.

The unadjusted mesh-local policy matches **195/214** meshes. Comparison with the
preserved static batch identifies **ten converted bindings / thirteen draws**
requiring frame corrections: roots/clusters, tree-house building/door, the trunk
and a table. The initial two trunk placements are part of this wider count.
The other differing source meshes include unresolved moss geometry and LODs.
Do not generalize the old mesh-local qualification to Unity importer equivalence.

The hash-bound aggregate and exact affected keys live in
`build/forest-editor-reference-summary-v1.json`. Four additional projects and
their request/raw/inspection/log receipts live in
`build/forest-editor-scene-reference-v1/{0,1,2,3}/Reference/`;
execution receipts are `build/forest-editor-scene-reference-execution-v1.json`.
All jobs are terminal. The user requested **“Stop when safe to do so”** before
converter correction work began; stop here with these references preserved.

Five new Python tests exercise rotated/scaled-root policy discrimination,
hash-bound staging and source-script exclusion, model identity/hierarchy/index/
bounds rejection, JSON limits and ten report-binding faults. The integrated
asset suite passes **155 tests**, plus four GLB tests and three independent
Blender prefab/variant-reference tests: **162 distinct Python tests**. Fresh
`cargo fmt --check` and denied-warning workspace/all-target Clippy pass. An initial
attempt to run the Blender reference tests in the asset-only environment failed
on missing `bpy`/module lookup; the correct isolated environments pass. Logs are
`build/editor-reference-{python-tests,blender-tests,glb-tests,clippy}-v1.log`.

The Python tests use synthetic reports; actual Editor execution is separate
evidence above. No Rust/runtime/rendering change, binary rebuild, new native GPU
run or map regression run is claimed here. The static forest experiment retains
**18,079 draws / 580 unresolved**, now with known frame corrections pending. The
normal town and opt-in forest ground remain available.

On resumption, apply explicit source-bound converter frame changes, independent replacement geometry/placement
checks and native rendering/map regressions before promotion. Complete remaining
material families, object collision and resource/runtime gates before forest
adoption. Inspect the user's new Elven Realms folder only after forest completion.
