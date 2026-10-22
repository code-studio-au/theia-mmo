# Forest collision geometry in Avian

P1-10/P2-06/P2-07/P2-08 continuation, 2026-10-21. Actual headless Bevy/Avian now
matches the independently verified source geometry for **2,762 active object
colliders / 16,572 targeted ray cases**, maximum distance difference
**0.000265122 metres**. Terrain retains its [separate full-grid gate](FOREST_TERRAIN_COLLISION.md).
This is a representation/query prerequisite, not runtime package admission,
authoritative movement, camera, navigation or normal forest-map adoption.

The [actual source reference](FOREST_COLLIDER_REFERENCE.md) supplies all component
identities, active flags, world matrices and Editor geometry. Its collider data
remains immutable. No source scripts execute.

## Two different comparisons

The strict actual-Unity ray comparison remains a diagnostic failure. Its initial
world-baked representation has 142 differences; local mesh centering leaves 141.
All box and capsule cases match. Observed mesh differences include:

- One-sided source ray filtering, including the downward invisible water plane.
  The lantern's native front-face hits differ from nearer two-sided intersections
  by as much as 0.372 metres.
- Native queries on sheared meshes differ from their fully transformed source
  triangle surface. The matrix itself is independently verified against the raw
  source; this is not fixed by changing its pose.
- Native convex results differ from the original serialized hull surface. On the
  mismatching subset, direct source-surface intersection differs from native hits
  by up to 13.84 mm; Avian follows those source surfaces within 0.093 mm.

These findings do not establish why every native cooking difference occurs.
They do establish that strict PhysX-query equality and original source-geometry
equality are distinct gates. The strict test remains available and is not relabeled
as a success.

## Independent source-geometry acceptance

`qualify_collision_geometry.py` uses float64 exhaustive triangle intersections;
it imports neither Avian/Parry nor PhysX. All 17 convex inputs prove to be closed
convex volumes: every edge has two incident faces, all vertices are referenced,
Euler characteristic is two, volume rank is three and all vertices lie behind
every outward face plane. Their original surfaces can therefore serve as the
independent convex-hull oracle without a second hull-cooking dependency.

Standard Avian two-sided triangle semantics are explicit in this comparison.
Box/capsule expectations retain the actual native Editor probes. Mesh geometry
uses mirrored-X right-handed positions, reversed triangles and complete source
affine transforms. Geometry stays local to each mesh body, with a separate static
body placement. All 49 boxes and 128 capsules use standard analytic shapes,
including the measured nonuniform scales.

The native test creates a real headless Bevy/Avian world and queries its live BVH.
Each ray filters to its exact source collider entity. Another object's hit cannot
satisfy the comparison. Hit/miss, distance and point agreement must pass; distance
and point tolerances remain 2 mm. Triangle normals must agree with a source face
within the contact neighbourhood. For convex edge ambiguity, the returned normal
must define a valid source support plane within that same 2 mm, rather than equal
one arbitrary legacy facet normal.

All 16,572 cases pass. Four rehashed/rebound faults still fail in actual Avian:
expected distance, expected hit state, primitive normal and translated mesh
geometry. The mesh fault produces 88 failed cases. Metadata hash checks alone
would accept those newly bound files; the geometry test rejects them.

The test is intentionally opt-in because source data is licensed/private. Both
comparisons live in `crates/network/tests/unity_collision_reference.rs`:

- `actual_unity_object_rays_match_a_native_avian_world`: retained strict legacy
  diagnostic; expected differences remain.
- `avian_object_rays_match_independent_source_geometry`: qualified source geometry
  acceptance; requires the explicit fixture, oracle and fresh report paths.

## Evidence and limits

Ignored local artifacts:

- `build/forest-collision-comparison-v1.json`: 80 meshes / 2,762 active objects,
  about 17 MiB. SHA-256
  `d277ebecb4608f73521c6ebbb65a257a694aeac05a81c84d508c4a3f1daf1f09`.
- `build/forest-collision-geometry-oracle-v1.json`: independent surface oracle.
  SHA-256 `f33fec8d45725569628bc9ba8d3ba20be37dcb1bb4949817035b2347862519dc`.
- `forest-collision-avian-comparison-v1/v2.json`: retained strict failures.
- `forest-collision-avian-geometry-v1.json`: successful native source-geometry
  comparison, including exact body/query counts and timings.
- `forest-collision-geometry-faults-v1`: four actual semantic rejection cases.
- `forest-collision-reference-clippy-v1.log`: denied-warning workspace/all-target
  Clippy. Fresh format/diff checks and **215 Python tests** pass.

Execution is Linux, Bevy 0.20.0, Avian revision
`1070759b2145dc078abedace91dc7fe78013a45c`; no renderer or GPU is involved in this
gate. Scene preparation measured about 2.86 seconds and the targeted ray batch
about 211 ms under uncontrolled desktop load. These are initialization/test-query
measurements, not simulation-step costs or a tick-rate recommendation. Native
Windows remains unqualified.

Next is measured runtime representation/package capacity and shared authority/
camera qualification. Preserve full terrain samples and source object coverage.
The downward water plane requires an explicit gameplay policy before admission;
two-sided geometry agreement does not authorize a walkable water surface.
The existing town default and forest ground-only option remain preserved.
All changes stay local.
