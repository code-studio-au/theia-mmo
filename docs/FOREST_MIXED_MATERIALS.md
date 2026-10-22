# Mixed source material slots and crystal previews

P1-10/P2-08/P2-10 continuation, 2026-10-21. Nine crystal/geode bindings /
**25 source draws** now join the qualified visual subset: **18,650 source draws /
nine unresolved**. Two source slots remain distinct, and actual Editor submesh
triangle assignments are checked directly. The crystal appearance is an explicit
static standard-PBR preview, with source refraction/view-dependent effects omitted.
This does not complete collision, lighting, resources or runtime map adoption.

Prerequisites were the immutable source plan, actual Editor mesh/indices exports,
qualified canonical/frame path and combined catalog. Acceptance requires raw
source renderer/material overrides, complete per-slot geometry/UV/normal/image
checks, actual Editor triangle slots, exact placements, unchanged predecessor
assets, native rendering and negative cases after metadata hashes are rebound.

## Implemented behavior

`convert_unity_mixed.py` accepts exactly one pinned crystal and one qualified
opaque Standard slot, in their source order. It preserves the source face material
indices and marks exported materials with `theia_source_slot`. Canonical geometry
uses the qualified baked file-axis/unit path, excluding imported model-node pose.
The Standard branch retains its source albedo/emission PNG roles and surface
properties. Other material families/features remain rejected.

`unity_crystal.py` pins shader GUID `006c4dbb387622342a1397597c7b3e66` and SHA-256
`ca667b1dc83a0f02e1059694197ff954ee2b065b0e8c78601f35a42aad2d6aa7`.
The preview samples its shallow/deep colour interpolation at normal incidence,
uses source opacity/metallic/roughness/emission and exports ordinary double-sided
glTF BLEND. Saved texture bindings absent from this pinned shader are recorded,
not accidentally applied. The actual source uses falloff 1.16, opacity 0.339,
metallic 0.678 and roughness 0.411. View-dependent tint, GrabPass/refraction,
chromatic aberration, source final-alpha behavior and Unity colour-space/BRDF
parity are explicitly unimplemented. This is a portable PBR adaptation rather
than a source shader port; no runtime Rust/WESL or engine/plugin pin changes.

`verify_unity_mixed.py` imports no author. It derives actual raw renderer/scene
overrides, verifies both material identities, computes independent PBR factors,
checks Standard import/sampler/image roles and pixels, and compares complete
per-slot source triangles/UVs/normal directions. `verify_unity_submesh_slots.py`
separately compares the corrected prototype with the actual Editor's submesh
triangle multisets using exact f32 positions and calibrated winding. This avoids
treating Blender material indices alone as proof of Unity slot fidelity.

`batch_unity_mixed.py` selects only exact eligible bindings from the preserved
base, qualifies their frames/slots/source placements, and retains explicit
unresolved reasons. Previously qualified groups can be reused with fresh checks.
The group coverage checker freshly rechecks actual Editor slots for mixed inputs
and rejects overlap with all earlier converted groups.

## Actual geode triangulation correction

Initial strict checks rejected five geode draws despite matching slot counts.
Independent FBX inspection found 99 source quads in Geode 02 and one in Geode 01.
Actual Editor and Blender use different diagonals for three/one of those quads;
the differing triangles have the same source corner sets. No error limit is relaxed.

`unity_editor_topology.py` matches source triangles/quads to recorded Editor slot
triangles in the verified frame, retaining original source positions, corner UVs,
active/render UV-layer selection and original corner normal references. Ambiguous,
missing or unsupported topology rejects. A Blender custom-normal round trip then
exceeded the existing normal bound, so `unity_corner_streams.py` writes the original
normal/UV corner streams directly into ordinary glTF after the topology edit.
Triangle corners remain separate; old material/image binary bytes stay unchanged.
This avoids re-encoding normals and keeps the original bound. Both corrected
geodes now have maximum normal direction error below **4.65e-8**. The seven
unaffected prototype/placement pairs remain byte-identical to their first batch.

## Executed qualification

All nine prototypes pass raw material/geometry/UV/normal/image checks, independent
ufbx/frame checks and complete actual Editor submesh triangles. Actual Editor
point-set error is zero observed. Exact source placements contribute **18,055
triangles**, with arithmetic error below 3.56e-15 m. Maximum normal direction error
across all nine profiles is **8.26e-5**, below the unchanged 0.00018 bound.

`build/forest-crystal-scene-v1/` preserves all predecessor inputs and adds these
25 placements: **18,915 rendered primitives / 6,110,220 triangles / 183 materials /
73 glTF images / 152 mesh definitions**. Actual Standard primitives total 18,387;
the four triplanar profiles still apply to 528 primitives using ten extension
images. Embedded Standard images remain duplicated; no minimum-residency claim.

Native Linux Bevy 0.20 checks pass on **RX 7900 XTX / RADV NAVI31 Vulkan**: initial
crystal prototype/all-five group, both corrected geode prototypes/groups, combined
overview/focus and one closer combined crystal view. All nine native cases have
active Khronos validation, completed pipelines, expected counts/UV channels, no
renderer errors/panics/tangent failures and bounds within **7.50 micrometres**.
The initial crystal and closer combined captures were inspected. An earlier
combined focus was occluded by a trunk; lighting/camera composition and translucent
sorting/refraction parity remain open. Source wind/highlight limitations remain.

Khronos checks all nine prototypes/nine placement files and the combined scene:
**19 files, zero errors, seven retained combined runtime-tangent warnings**.
Prototypes/groups have zero warnings. Five real-source faults reject after hashes
are rebound: swapped slots, reversed/dropped triangle, crystal alpha mode and
swapped Standard albedo/emission images. Fresh fmt, denied-warning workspace/
all-target Clippy and **189 Python tests** pass (180 asset + four GLB + five Blender
references/topology tests). The unchanged client retains its 35 Rust tests, legacy
render and encrypted town/farmhouse evidence; no fresh multiplayer/Windows claim.

Combined scene SHA-256:
`5527691cdebedf90239e7dd882ce375e65780171b7e72aa9c7560a4fece84c65`.
Client remains `3526f43349e7bb02cb93c3469be23040012a9ffd1c077674869781e9475a99b0`.
Evidence: `build/forest-crystal-batch-v3/`, `build/forest-crystal-coverage-v1.json`,
`build/forest-crystal-scene-v1/`, `build/forest-crystal-prototype-native-v1-20261021/`,
`build/forest-crystal-native-v1-20261021/`,
`build/forest-crystal-focused-native-v2-20261021/`,
`build/forest-crystal-faults-v1.json`, `build/forest-crystal-gltf-v1.json` and
matching logs. Earlier rejected topology/normal attempts stay diagnostic only.

Remaining source families: seven water draws, one skydome and one emissive willow,
across six bindings. Use explicit standard-Bevy adaptations where practical and
record omitted effects. Lighting, rebase/resources/mips, source object collision
and runtime adoption remain separate gates. Keep town and opt-in forest ground
available; inspect the new Elven folder after forest completion. Work stays local
and all generated/licensed artifacts remain ignored.
