# Applying recorded Unity mesh frames

P1-10/P2-08 continuation, 2026-10-21. The user resumed work after the safe stop.
The [actual Editor reference](FOREST_UNITY_REFERENCE.md) covers every FBX-backed
scene binding and identifies ten converted bindings / thirteen placements whose
old canonical mesh frame differs from Unity's imported mesh frame. This slice
corrects that geometry before continuing remaining materials and collision.

## Contract and prerequisites

Use hash-bound original FBXs/metadata, the existing canonical prototype and an
actual Editor report collected by the reviewed read-only collector. Check the
original prototype against independent ufbx first. Recheck the selected model's
persistent mesh/transform IDs, metadata bytes and actual Editor/ufbx point sets.
The render-space correction is
`reflectX * inverse(UnityWorld) * reflectX * FbxNodeWorld`.
No bounding-box centring or artist-specific offset table supplies this matrix.

The required ten bindings have translation-only corrections. The tool also
accepts strictly positive uniform scale, which preserves shading directions.
Rotation, nonuniform/mirrored scale and singular/nonfinite frames reject pending
their own geometry/shading qualification. Coordinates stay bounded to ±4096 m.
This is an offline source-asset frame operation, separate from f64 global-origin
rebasing or authoritative world movement.

`reframe_unity_static.py` makes a new stage. It appends corrected float32 POSITION
data and updates only position accessors/buffer length. Original binary bytes,
all other vertex streams, triangle winding/indices, node poses, materials and
embedded image bytes stay unchanged. Old geometry/qualification sidecars are
renamed with `prior-`; tools expecting the old frame therefore fail closed.
Conversion metadata retains the original source/scene/material fields and adds
an explicit frame receipt binding predecessor, actual Editor report and matrix.

`verify_unity_reframe.py` imports no correction authoring code. It independently
checks the predecessor's source geometry/materials, derives the correction from
the actual reference, checks per-vertex arithmetic and complete invariant graph/
binary streams, then compares corrected used point sets directly with actual
Editor mesh vertices. The source material adapter and generated normals/tangents
retain their earlier stated limits; Editor point sets do not certify Unity's
triangulation, normals, UV generation, lighting or BRDF. Standard and the pinned
triplanar profile have explicit predecessor material checks; triplanar also
requires its shader source stage and exact external PNG/settings preservation.

`reframe_unity_batch.py` creates a separate batch, retains unaffected prototype/
placement bytes exactly, rebuilds affected source placements and checks every
converted prototype against the actual Editor point sets. It withholds the new
`batch.json` until all corrections and complete prototype coverage pass. Prior
stages remain available; runtime/default-map adoption is separate.

## Acceptance

- Ten corrected bindings / thirteen source draws, with all other 106 bindings'
  prototype/placement bytes unchanged. Source plan, category counts and unresolved
  records unchanged; no missing or extra source renderer.
- Every converted prototype agrees with the actual Editor point sets within the
  existing 0.1 mm bound. Independent source-pose, normals/tangents, winding,
  material/image and full combined-scene coverage checks pass.
- Reassembled scene passes Khronos and actual native Bevy/RADV Vulkan validation,
  with independently computed bounds and visual inspection. Retain town and
  farmhouse encrypted multiplayer regressions.
- Fresh Python tests, fmt and denied-warning Clippy. Rust runtime code/binaries
  are unchanged by this offline slice; no new Rust or native Windows claim.

Four new tests cover translation/uniform scale, invariant shading/topology bytes,
material/UV/normal/tangent/pose/winding faults, invalid frame rejection and bounded
positions. Three additional tests cover pixel-space edge clearance and intersecting
front surfaces; they test geometric sample eligibility without using pixel colours.

## Executed source and scene evidence

`build/synty-forest-frame-batch-v1/` corrects **ten bindings / thirteen source draws**.
All other **106 prototype/placement pairs remain byte-identical**. Every one of
the **116 converted prototypes** matches actual Editor mesh-local point sets,
maximum **7.69 micrometres**. Corrected prototypes individually differ by at most
0.954 micrometres. Their source topology/UV/normal/tangent/material/image streams
are unchanged. The source plan and 580 unresolved draws are unchanged.

The new standalone `build/synty-forest-frame-scene-v1/forest.glb` contains the same
**18,079 source draws / 18,302 primitive instances / 5,171,337 triangles / 121
meshes / 142 materials / 31 images**. Independent full-scene coverage, source
matrices, normals/tangents, winding and shared-image/material checks pass, with
maximum position arithmetic difference 9.95e-14 m. Ground bytes stay unchanged.
The former batch/scene are retained. This replaces the inspected static subset,
not the normal multiplayer map or complete forest content.

New batch SHA-256:
`422ce9929d660a681be102249604dc15d67b4d07aa2124841f99c5cbb80cde54`.
New combined GLB SHA-256:
`805be979b958c898509115228412a7fa0df33eb25ab574615b710acc4fd3cfb3`.

Khronos validator 2.0.0-dev.3.10 checks all 116 prototypes and the combined scene:
**zero errors**, the same fourteen standard runtime-tangent warnings, and unused
accessor informational messages from preserving predecessor binary data.
Three actual native Bevy 0.20 views (overview, forest and corrected building/
door detail) pass active Vulkan validation on **RX 7900 XTX / RADV NAVI31**, Linux,
in an owned private Wayland compositor. Actual bounds differ from the independent
reference by at most **0.0151 mm**; pipelines finish without pending/failed entries,
VUIDs, renderer errors, panics or tangent-generation failures. Forest and building
views were visually inspected. Lighting is the existing inspector adaptation,
not imported Unity lighting.

Encrypted two-client **town and farmhouse** regressions pass with the retained
normal binaries/maps. This does not establish forest multiplayer/object collision.
Evidence: `build/forest-frame-native-v1-20261021/`,
`build/forest-frame-map-regressions-v1-20261021/`,
`build/forest-frame-gltf-validation-v1.json`, and the new batch/scene receipts.

## Corrected moss group

`build/forest-triplanar-moss-frame-v1/` applies the recorded mesh frame to the
actual 196-triangle moss prototype. Independent actual Editor positions match
within **2.76 micrometres**. `build/forest-triplanar-moss-frame-group-v1/` contains
all **85 exact-binding source placements / 16,660 triangles / one shared mesh**.
All source instance IDs/matrices match the original plan and the preserved
unqualified experiment. Independent actual Editor vertices under all source
matrices match within **10.56 micrometres**; GLB source-matrix arithmetic differs
by at most 1.43e-14 m. Corrected Y bounds are **[-1.6092, 13.6529] m**, replacing
the unqualified experiment's implausible [-105.84, 96.13] m range.

Canonical/group lit, unlit-colour and decoded-normal cases all render through the
existing opt-in Bevy material extension. The final group view focuses on a cluster
of fourteen instances while all 85 receive the shared profile. Final six native
cases pass active Vulkan/pipeline/bound checks; maximum bound differences are
1.32 micrometres for the prototype and 3.26 micrometres for the group. Both lit
views were visually inspected. Both GLBs pass Khronos with **zero errors/warnings**.

Independent final colour/decoded-normal checks cover **146 + 146 prototype** and
**110 + 110 group** pixels: **512 total**, maximum **one sRGB8 level**, against the
unchanged four-level limit. Earlier group overview checks admitted MSAA silhouette
pixels with <0.27-pixel edge clearance. A closer capture exposed an intersecting
surface changing the nearest triangle within one pixel. Those rejected results
remain preserved. The final oracle requires a geometric one-pixel edge margin
and the same nearest front triangle at footprint corners; neither rule depends
on observed/expected colours. A dedicated crossing-plane test covers the latter.
The final capture offers adequate interior coverage; no error bound is relaxed.

Moss GLB SHA-256:
`da85781f0331c543a95759baed79dbb47c283bc76365404ddbfeed4b977ac1d0`.
85-instance GLB SHA-256:
`e106952eb69b9617c3d3891eb2f85463bcf1ce46d863ae1b4fcfd0886013ca54`.

Evidence: `build/forest-triplanar-frame-native-v1-20261021/prototype/`,
`build/forest-triplanar-frame-native-v2-20261021/group/`,
`build/forest-frame-*-pixels-v3.json`,
`build/forest-frame-triplanar-gltf-validation-v1.json`, and group
`editor-world-reference.json`. Three private faults reject changed material
settings, changed external texture bytes and rebound altered positions.

## Checks and remaining work

Fresh **169 Python tests** pass (162 asset, four GLB, three Blender prefab/variant),
plus the three private fault cases. Fresh fmt and denied-warning workspace/
all-target Clippy pass. Rust runtime code, Cargo.lock and binaries are unchanged;
the earlier Rust suites are retained, not claimed as newly executed. Client SHA:
`448c900042af0f0ecca810110a56e63f3f0460952a2f63e3a57b483bdee4d4f6`.
No new native Windows, performance, production streaming or gameplay claim.

The main corrected subset remains **18,079 draws / 580 unresolved**. The 85 moss
draws are qualified as an origin-zero, single-profile inspector group; they have
not been added to the main scene. Multiple material profiles, rebased texture
phase, resources/mips, remaining material families, object collision and runtime
adoption remain. Continue those gates and inspect the new Elven Realms folder
only after forest completion. All work stays local.
