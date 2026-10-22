# Offline static world partitioning

Offline slice qualified, 2026-10-20; actual-town runtime adoption remains open. P1-10/P2-06/P2-07/P2-08, execution-order row 5.
Prerequisites: stable content packages, bound shared images, contributor-complete
coverage and shared controller/publication qualification. The default map stays
monolithic until actual imported output and runtime budgets qualify.

The tool must clip explicitly selected terrain meshes and all collision triangles
at checked 128-metre cell planes, preserving winding, terrain height and UVs.
Whole static model mesh instances keep their original transform hierarchy and
receive one owner based on transformed geometry. Crossing geometry remains a
contributor rather than being silently discarded. Static collision boxes remain
whole contributors; individual package seams receive no artificial walls.

Each output prunes its node/mesh/accessor/material/texture/image/sampler graph and
copies only required binary views. Shared PNG URI keys remain unchanged. The
publisher must validate source bindings, preserve source files, reject unsupported
layouts and write new packages. Derived terrain vertices use interpolated
attributes and normalized normals. Source model buffers and transforms are not
re-encoded. All output uses standard GLB/Bevy assets.

Acceptance includes independent clipping/area/UV/seam checks, exact static model
buffers and transforms, removal of unused image dependencies, signed distant
coordinates, actual imported inventories, and native original-fixture loading/
collision/movement checks. Record provider counts, bytes and load requirements;
do not reduce contributor completeness or wrap the town in one permanently
resident provider to fit the existing proof budget. Empty collision packages,
world-edge policy and any measured residency-cap changes need their own runtime
qualification before adoption. No complete task/phase or MMO capacity follows.


## Implemented tools

`tools/partition_world.py SOURCE NEW_OUTPUT --content-id ID --revision N
--terrain-mesh M` accepts repeated explicit terrain mesh indices. Input must
already be a bound shared-image package. Optional `--origin-x`/`--origin-z`
select checked global XZ placement. Model ownership uses transformed geometry
bounds; duplicate node names do not merge nodes. Terrain supports float position,
normal, UV0/UV1 and colour attributes. Normals use inverse-transpose conversion
and normalization. Reflected/singular terrain transforms, tangent attributes,
animations/skins, glTF extensions and unsupported sparse layouts fail explicitly.
Whole static models retain their hierarchy rather than flattening matrices.

Outputs include individual packages, the shared PNG directory,
`world.chunks.candidate.json` and `partition-report.json`. Active index/session
metadata is emitted only when the current four-provider budget and trusted spawn
footprints fit. This flag proves metadata fit, not runtime qualification. New
outputs are required; existing/source directories are never overwritten. Failed
builds remove only their new output directory. The launcher is unchanged.

`tools/verify_world_partition.py SOURCE OUTPUT --terrain-mesh M` independently
reconstructs original model buffers/transforms/material image settings, tests new
terrain vertices against barycentric source height/UVs, compares total area and
checks complete oriented output faces against original surfaces. Collision uses
an independent 3D face oracle, so equal-area translated or reversed faces fail.
Movement and camera boxes must occur exactly once in original global positions.
A failed rerun removes the old verification result before checking. It writes
`partition-verification.json` only after successful comparison. This is offline
source equivalence; it does not infer precise model collision or navigation.

## Actual town measurements

The private source is the unchanged shared-image town representation from
[shared image qualification](SHARED_IMAGES.md). Terrain selection is the reviewed
first 256 terrain meshes, not a name-based guess applied to arbitrary content.
The qualified stage is `build/partitioned-town-qualified-stage`.

| Measurement | Output |
|---|---:|
| Packages | 26 |
| Model mesh instances retained exactly once | 2,532 |
| Terrain instance/primitive groups | 256 |
| Collision triangles after clipping | 72,047 |
| Whole movement / camera boxes | 3,972 / 4 |
| Total GLB bytes | 8,470,060 |
| Total collision JSON bytes | 12,805,430 |
| Maximum textures per package | 139 (source: 491) |
| Maximum providers for one movement cell | 9 |
| Providers required at current spawns | 10 |
| Maximum spawn providers plus one arbitrary cell | 16 |
| Initial unique shared image variants | 377 |
| Initial encoded shared image bytes | 474,111,320 |

The 488 global PNG resources retain the original 698,655,860 encoded bytes. Graph
pruning reduces per-package dependencies; it does not compress images or measure
GPU memory. Terrain-group maximum area difference is 0.00023984098743312643 square
metres; aggregate collision area difference is zero. Source positions/UVs remain
within the documented verifier tolerances (0.5 mm height/plane, 0.00003 UV).
Collision comparison uses 1 micrometre plane tolerance. These are comparison
tolerances, not measured gameplay collision accuracy.

The real town deliberately has no active `world.chunks.json` or session: nine
providers and ten initial packages exceed the existing four-package runtime
contract. Do not drop crossing models or weaken contributor closure to fit it.
Sixteen covers spawn providers plus one arbitrary cell only; it does not prove
all actor/lookahead/interpolation/staging demand fits. Measure those unions,
encoded/decoded image costs, static body limits and preparation latency before
choosing runtime budgets. Define outer-world boundaries and recovery behavior,
qualify reversal/supersession/asymmetric failures, then qualify actual town
publication and map regressions before changing the launcher.

Two generated original-geometry native cases qualify the output format with the
existing runtime budget: ordinary and signed billion-metre global positions,
independent query frames, delayed candidate preparation, exact retirement
acknowledgements, retained roots, actual shared Bevy image handles and matching
live Avian. They are not native rendering of the 26-package town. See
[executed evidence](VALIDATION.md).


The subsequent [budget analysis](CHUNK_BUDGETS.md) measures full capsule
footprints and conservative multi-actor/falling lookahead, rather than only
single-cell unions: maxima with recovery are 18 packages for one actor, 22 for
two and 26 across the bounded eight-actor case. It also verifies resource counts
and source spawn provenance. Runtime capacity and default adoption remain open.


The [capacity continuation](CHUNK_CAPACITY.md) subsequently qualifies a runtime
hard ceiling of 32 and an opt-in 26-package town authority session. The original
publisher still uses a four-provider automatic metadata gate; the candidate town
stage and default launcher remain unchanged. World edges, actual-town native
loading/resources and the remaining lifecycle cases still precede adoption.
