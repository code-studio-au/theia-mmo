# Bounded static shapes and full-sample heightfields

P1-10 / P2-06 / P2-07 prerequisite, 2026-10-21. Collision schema 4 extends
the existing package, terrain reader, authoritative movement, client physics and
contributor index. Existing schemas 1–3 and the town default remain supported.
This is a representation and admission slice; navigation and gameplay hit
detection remain separate.

## Contract

Schema 4 requires `static_shapes`. Earlier collision schemas reject that field.
Package manifest schemas and identity domains are unchanged; `collision_schema`
binds the new format. Legacy general triangles/boxes may coexist, but legacy
walk-surface/step-height shortcuts are prohibited, as in schema 3. Explicit
spawns undergo the existing actual capsule checks.

`static_shapes` has three optional arrays, defaulting to empty:

- `meshes`: shared `{vertices, indices}` triangle inputs. Every pool entry must
  be referenced. Indices are bounded; at least one face must be nondegenerate.
- `objects`: tagged `kind`. A `mesh` has pool index `mesh`, a 16-number row-major
  `affine` and Boolean `convex`. A `box` has full `size`, `center` and normalized
  XYZW quaternion `rotation`. A `capsule` has `radius`, `center` and a vector
  `half_segment`, whose endpoints are relative to its centre.
- `heightfields`: `rows` along Z, `columns` along X, `heights_column_major`,
  `center` and full XYZ `scale`. Flat sample index is `row + column * rows`.
  Default standard Parry cell diagonals are retained. This rectangular layout is
  verified independently; a square grid alone cannot establish axis order.

Objects and fields accept optional `queries`, default 7: SOLID=1, GROUND=2,
CAMERA=4. A mask must be nonzero and contain only these bits. There are no actor,
dynamic-body, navigation or combat flags. Standard Avian shapes implement the
representation: triangle meshes/convex hulls, one-child rotated box compounds,
capsules and heightfields. Mesh linear transforms are baked into vertices while
the authored translation remains the body's origin. Recentring thin meshes on
their AABB produced two grazing misses in the first actual-source run; preserving
the authored origin fixes both without changing tolerances.

All shape fields deny unknown keys. Invalid references, nonfinite or singular/
projective transforms, invalid rotations/radii/scales, unbound pools and excess
work reject. Positions and cooked bounds remain within the bounded local frame;
there is a 1 mm reserve below ±1023 m for conservative contributor envelopes.
The publisher pads shape bounds by 1 mm and uses the established outward index
rounding. Runtime admission checks actual cooked Avian bounds against the index.

| Per-profile resource | Limit |
|---|---:|
| Legacy triangles plus pooled input triangles | 131,072 |
| Pooled vertices / mesh entries | 262,144 / 128 |
| Instantiated mesh triangles / vertices | 262,144 / 1,048,576 |
| Height samples / fields | 263,169 / 8 |
| New object plus field bodies | 4,096 |
| Legacy plus new bodies, reserving four world walls | 8,188 |

Fields have 2–1025 rows and columns and exact sample counts. Existing package
byte limits, 32-provider ceiling and 8,192 resident-body ceiling remain unchanged.
Input sharing does not imply that affine-baked native colliders share allocations
or that these limits establish MMO capacity. Cooking occurs during preparation/
admission, outside authoritative simulation steps.

## Qualification

Actual forest source geometry passes through **ten real `Terrain::decode`
profiles**, rather than a separate prototype cooker: 2,762 object colliders plus
four full-sample terrain fields. All 16,572 targeted object rays and 787,457 terrain
vertex/centroid rays pass. Maximum object distance error is 0.266 mm; maximum
terrain height error is 7.16 micrometres. Source comparison retains every active
object, including the invisible downward water plane, for geometry qualification.
This does not admit that plane as a walkable runtime water surface.

A separate complete source-binding verifier compares **1,731,286 instantiated
mesh vertices**, all original mesh indices/coordinates, complete oriented-box
corner sets, capsule endpoints/radii and all **263,169 unique height samples**.
Maximum reconstructed source-shape difference is 7.63 micrometres. It imports
neither the profile author nor its quaternion/bounds implementation. Four
rehashed affine/radius/box/sample faults reject in real Avian; the complete
verifier also rejects the initial width-only box fault that the six centre rays
did not detect. That passing sampled-ray diagnostic is retained rather than
claimed as complete geometry coverage.

The original visible mixed-shape package passes actual encrypted Renet authority
and two native Bevy clients with capsule/source-surface clearance, actor pixels,
physics probes, compiled pipelines and automatic render/replication rebasing.
A separate original three-provider heightfield/box package passes the same normal
chunk startup path at global X/Z ±128 billion metres, using actual conservative
schema-2 contributor envelopes and small f32 frames. This is initial publication,
not a new streaming-capacity claim. Existing town and farmhouse entry/wall/exit/
re-entry regressions pass on the rebuilt binaries.

Native rendering ran on Linux, AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan, with
the Khronos validation layer active. Each case has zero pending/failed pipelines,
compiled pipelines and at least three clear frames. No Vulkan validation errors,
panics or unexpected renderer errors were observed. No Windows execution or
Windows cross-build is claimed. Four original fixture GLBs have zero Khronos
validator errors or warnings.

The complete default network test suite, 41 client tests, mixed-shape movement at 20/30/60 Hz,
package bindings and malformed/amplified-input tests pass. Fresh workspace
format checking, denied-warning all-target Clippy, 217 asset Python tests and
nine Blender/reference Python tests pass. Independent float64 capsule checks
cover analytic capsules, closed convex surfaces, oriented boxes, terrain cells,
camera-only exclusion and a retained zero-area source face.

Ignored local evidence:

- `build/forest-schema4-reference-v1/reference.json`, actual source profile map.
- `build/forest-schema4-native-v2.json`, complete actual-source reader result.
- `build/forest-schema4-complete-geometry-v2.json`, complete source binding and
  five independent fault rejections.
- `build/static-shapes-source-faults-v2/verification.json`, four actual Avian
  semantic fault rejections after rebinding profile hashes.
- `build/forest-schema4-native-v1.json`, retained grazing-miss diagnostic.
- `build/static-shapes-native-v1-20261021`, mixed shapes/town/farmhouse runs.
- `build/static-shapes-chunks-native-v1-20261021`, large-coordinate chunk run.
- `build/static-shapes-gltf-validation-v1.json`, four original GLB results.
- `build/static-shapes-network-tests-v1.log`, complete default network suite.
- `build/static-shapes-clippy-v1.log`, workspace/all-target denied warnings.
- `build/static-shapes-python-tests-v4.log`, 217 asset tests.
- `build/static-shapes-blender-tests-v1.log`, nine reference tests.

Native client SHA-256 is
`151cf6368915d47ae0707e005dcc3a6a19cba52fba1bd8400e6b8a7b9202c9e6`;
server is `804b626bde7ca7b679fcbf511b959a5abadfb18d5fb50ab29c849522160bb1de`.
Engine/plugin pins and dependency versions are unchanged.

## Remaining forest gates

The source profiles are reference-only, without published visuals or spawns.
Complete forest visual partitioning/material integration, shared-image resource
behaviour under rebasing, explicit water policy, actual map movement/camera,
bounded lifecycle/resource measurements and normal development-map adoption remain
open. Strict Unity cooked/backface/shear equality remains a separate diagnostic
failure documented in [source geometry](FOREST_COLLISION_GEOMETRY.md). This slice
does not establish source engine parity, navigation, swimming or whole-map readiness.
