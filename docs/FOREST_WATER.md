# Exact water geometry with an explicit static PBR preview

P1-10/P2-08/P2-10 continuation, 2026-10-21. Four exact bindings / seven source
water draws now join the visual subset: **18,658 source draws / one unresolved**.
The remaining binding is the emissive willow. Geometry/placements qualify; water
shader fidelity, swimming, object collision and normal map adoption remain open.

Prerequisites were the immutable highest-detail plan, qualified actual Editor FBX
positions/indices and the combined catalog. Acceptance requires a separately proven
built-in Plane, raw source bindings/poses, exact Editor triangle corners/winding,
explicit material adaptation, full composition, native checks and rehashed faults.

## Actual built-in Plane reference

One inline scene renderer references Unity built-in mesh GUID
`0000000000000000e000000000000000`, file ID `10209`; it has no package FBX.
`TheiaBuiltinPlaneReference.cs` creates a temporary Plane in an empty ignored
project, verifies its persistent identity, reads positions/normals/UV0/indices
through read-only mesh data and destroys the temporary object. No source scenes,
scripts or packages are imported. Request/collector/version hashes bind its report.

Native Linux Unity **6000.6.6f1** compiled/executed that collector in
`build/forest-water-builtin-editor-v1/`: **121 vertices / 200 triangles**. Source
scale 15 places it across 150 × 150 metres at Y=0. The source MeshCollider is
explicitly disabled; this visual does not establish a walkable water collider.
`unity_builtin_plane_reference.py` checks identity, byte budgets, finite streams,
unit normals, indices, bounds, request and unchanged collector. Collector SHA-256:
`1bc351ce5447c694af6061b47f6ea9aab2dc169c81f6d9aff0ae70320cae6455`.

## Implemented adaptation and limits

`convert_unity_water.py` consumes exact Editor positions/indices for two FBX water
meshes and that built-in mesh. Calibrated X reflection/reversed winding, separate
triangle corners and geometry-derived flat normals are explicit. Imported source
normal/UV parity is not claimed. Source materials use ordinary glTF BLEND,
backface culling and the inspector's source shadow flags. Six FBX placements disable
shadow casting and retain receiving; the built-in Plane retains both source flags.

`unity_water.py` pins the two actual shader GUID/SHA-256 pairs:

- `a14ab9faa19eaeb4b8b41c78635b23bd` /
  `ab01bd729b9d7e76281573b27b72e22bfec1ca5e72047c554e7f7b3b4c4f702b`.
- `276a39a5a210b124885f0eff3e80f1f3` /
  `d1798bf9cc2e6e4622cb1f6812c08f951778bcc380b8f0d0565fa37c0c7da6b8`.

The preview uses the shader's pre-screen-effects tint at normalized depth one and
its one-metre alpha without foam. Roughness is one minus clamped source water
smoothness × reflection power; metallic is zero with standard dielectric specular.
Scrolling water has alpha 0.702 / roughness 0.710744. The built-in water has alpha
0.841 / roughness zero after clamping its source smoothness 2. Saved source
properties and unapplied textures remain recorded. Source refraction, scrolling,
depth fade, foam, caustics, ripple/normal maps, specular and Unity lighting parity
are unimplemented. Nonzero vertex wave amplitude and unsupported variants reject.
This is a static material approximation, not completion of P5-05 water rendering.

`verify_unity_water.py` imports no water author. It checks exact source group
membership and raw renderer/material/active/LOD/parent poses through the independent
mathutils oracle, all source shadow overrides, pinned shaders and independently
computed material factors. It then checks exact Editor triangle corners/winding,
normalized geometry-derived normals and every source placement. Metadata hashes
alone are insufficient. Shared-binding groups with inconsistent shadow flags reject.

## Executed qualification

All four groups under `build/forest-water-source-v1/` pass independent source,
Editor and placement checks: **2,600 placed triangles**. Position arithmetic error
is below 7.11e-15 m; independent f32 matrix difference is below 1.68e-6.
Four rehashed faults reject transparency, winding, derived normals and an altered
source-plan pose; the latter fails the raw source matrix oracle by one metre.

`build/forest-water-combined-v1/preview.glb` SHA-256 is
`b5c978de019c13cfe0a203dffdaf034694e41e68cd01ad56539119dee472ac73`.
Independent four-input ownership/stream/material/frame checks pass. Combining water
before the catalog uses fifteen Standard inputs, preserving its sixteen-input cap.

`build/forest-water-scene-v1/preview.glb` SHA-256 is
`d717999b9e8433851652cf6579eb2249dba6a997f49086014978c234b93f2e87`:
**18,923 primitives / 6,112,980 triangles / 188 materials / 74 glTF images /
157 mesh definitions**. Complete predecessor composition checks pass; four
triplanar profiles still apply to 528 primitives using ten extension images.
Actual Standard instances total 18,395, including seventeen colour-UV1 instances.
The combined scene has one unlit mesh, seven disabled shadow casters and one
disabled receiver. `build/forest-water-coverage-v1.json` binds source coverage.

Native Linux **RX 7900 XTX / RADV NAVI31 Vulkan**: all four prototypes/four placed
groups and three full-scene views pass. These eleven cases have active Khronos
validation, completed pipelines, exact material/shadow counts and expected bounds;
maximum native bound error is below 0.00001183 m. Screenshots were inspected;
canopy views remain dark/occluded and do not qualify lighting or source water
shader fidelity. Ten Khronos GLBs have zero errors: nine have zero warnings and
the combined forest retains seven runtime-tangent warnings.

Fresh fmt, workspace/all-target denied-warning Clippy and **197 Python tests**
pass (188 asset, four GLB and five Blender). No Rust/runtime/binary change during
this slice: retain the preceding 38 client tests, three legacy material modes and
encrypted town/farmhouse regressions on client
`75d46a365d6357c872223875647a8c27363331404aeaf5f5ba8356a473e9ff45`.
Evidence roots under ignored `build/`: `forest-water-native-v1-20261021`,
`forest-water-scene-native-v1-20261021`, `forest-water-focus-native-v1-20261021`,
`forest-water-source-verification-v1.json`, `forest-water-faults-v1.json` and
`forest-water-gltf-v1.json`. Windows execution is not claimed. Licensed inputs,
outputs and account data stay out of Git; all changes stay local.

Next: emissive willow, then lighting, resource/rebase/mip gates and authored object
collision before forest runtime adoption. Elven folder inspection remains queued
after forest completion.
