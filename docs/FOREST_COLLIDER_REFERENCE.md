# Forest authored collision reference

P1-10/P2-06/P2-07/P2-08 continuation, 2026-10-21. This supplies actual source
collider geometry, hierarchy and native Unity query evidence. It does not yet
publish authored object collision into Avian runtime packages or adopt the forest
as the normal development map. The qualified [ground packages](FOREST_TERRAIN_COLLISION.md)
and [visual/lighting preview](FOREST_LIGHTING.md) remain separate baselines.

## Qualified source evidence

The source's raw prefab colliders reference 79 distinct meshes. Existing visual
FBX evidence covers 23 of them. Two new native Linux Unity 6000.6.6f1 batches
export 39 dedicated collision FBXs; all mesh-local point sets match the independent
ufbx/frame oracle, maximum error 3.67 micrometres. Fifteen serialized asset
containers export another 17 Mesh subassets. A separate bounded decoder verifies
every float32 position and uint16 triangle index exactly against the original
packed source bytes, including persistent IDs and submesh slices.

The complete actual Editor catalog now contains 270 meshes. All original metadata
stays byte-identical. Serialized mesh geometry is not a claim that PhysX and Avian
convex cooking produce the same shapes.

An additional isolated project stages all 647 selected package data files and
their original metadata. Executable source files, shaders, assemblies and packages
are excluded. Only the reviewed collector runs. It opens the original forest
scene in edit mode, never saves it, checks every input before/after and reports
zero resolved source MonoBehaviours; two missing script components remain data.

Actual scene collider inventory:

| Kind | Present | Active and enabled |
|---|---:|---:|
| MeshCollider | 2,598 | 2,585 |
| BoxCollider | 49 | 49 |
| CapsuleCollider | 128 | 128 |
| TerrainCollider | 1 | 1 |
| Total | 2,776 | 2,763 |

`verify_unity_scene_collision.py` independently compares the actual component
identities, source GUIDs, prefab instances, active ancestry, relevant overrides,
world matrices, primitive dimensions and mesh/cooking bindings with raw source
documents. It imports neither the visual planner nor a collision author.
Independent mathutils composition agrees within **0.0000457764 metres**.
The four single-model branch variants preserve their added colliders and require
all ten final transform components explicitly; inherited aliases are checked.

The collector performs six targeted native Collider.Raycast probes for every
active/enabled collider: **16,578 probes, 15,184 hits and 1,394 misses**. These
are real Editor physics queries, not inferred mesh intersections. Their purpose
is a cross-backend reference; successful collection does not prove Avian parity.

Four rebound reference faults—world pose, mesh GUID, active state and capsule
radius—fail the independent source check. New guards cover packed geometry,
script exclusion, output containment, relevant overrides and incomplete model
variant transforms. The current offline suites pass **213 Python tests**.

## Findings that constrain runtime import

- The visible built-in water plane's MeshCollider is disabled. A separate enabled
  invisible `Water_Collision` plane points downward: the actual upward ray hits
  its underside while the downward ray from above misses. Treating all triangles
  as ordinary two-sided walkable ground would change its behaviour.
- The 79 raw prefab mesh references expand to about 727,386 triangles before
  active-state filtering and convex cooking. These are source counts, not admitted
  bodies, memory measurements or runtime capacity.
- All 49 boxes have effectively orthogonal world bases; 25 have nonuniform scale.
  All 128 capsules have effectively orthogonal bases; three have nonuniform scale.
  Mesh colliders include 11 mirrored instances and substantial shear. Baking their
  full transforms is a candidate representation to qualify against native probes.
- Each retained full-grid terrain package already contains 131,072 triangles,
  filling its current allowance. Uniform stride-two grid reduction produces
  roughly three metres of maximum sample error in this source; larger strides
  are worse. That measured candidate is rejected. Preserve terrain samples while
  measuring package representation/capacity; do not silently raise limits or
  remove authored obstacles.

## Local artifacts and next acceptance

Ignored evidence under `build/` includes:

- `forest-collision-editor-fbx-v1/{0,1}/Reference/`: actual dedicated FBXs and
  independent frame inspection. The first attempted invocation used the wrong
  collector entry point; its failure log remains. Corrected `Run` invocations
  produced the qualified reports without changing the collector or inputs.
- `forest-collision-editor-serialized-v1/Reference/`: actual serialized meshes
  and exact packed-byte inspection.
- `forest-collision-editor-scene-v1/Reference/`: actual complete scene report and
  native targeted ray probes.
- `forest-collision-scene-reference-verification-v1.json`: complete independent
  raw source comparison.
- `forest-collision-reference-faults-v1`: four rejected rebound faults.
- `forest-collision-terrain-lod-measurement-v1.json`: rejected coarse-grid sample
  error measurements, not continuous/capsule movement qualification.

The next gate is a native headless Bevy/Avian comparison of source meshes,
convexes, boxes and capsules against the recorded probes. Terrain retains its
separate gate. Distinguish winding/backface policy, convex cooking, scale/shear
and query tolerances before accepting a representation. Then measure actual
package/body/triangle/CPU costs and authoritative movement/camera behaviour.
None of the current work establishes navigation or MMO-scale capacity.

All execution here is Linux; no native Windows collision evidence is claimed.
Changes and licensed artifacts remain local.
