# Complete forest shape/visual reference packages

P1-10 / P2-06 / P2-07 / P2-08 prerequisite, 2026-10-21. The qualified complete
static forest visuals and actual source shape references now form ten immutable
candidate packages with a schema-2 contributor index. This is source-preserving
packaging and headless qualification. It does not replace the development map.

## Representation and completeness

Whole draw nodes use their actual transformed mesh-envelope centroid's 128 m
integer owner. Meshes retain all source vertex/index/normal/UV/tangent/colour
streams, hierarchy, material factors, source shader marks and image/sampler roles.
A parent translation converts each owner to bounded local space; source child
transforms are unchanged. The terrain visual remains whole in its owner package;
it is not decimated or clipped by this slice. Collision retains four adjoining
full-sample fields and all 2,762 active object shapes.

The measured visual and collision layouts have the same ten owners, with no
visual-only package or fabricated supporting geometry. Complete conservative
visual/collision envelopes require at most eight contributors per indexed cell.
All **18,660 draw nodes** (18,659 source objects plus terrain), **18,925 primitive
instances** and **6,137,378 triangles** remain represented. These are asset counts,
not a draw-call, streaming or MMO-capacity claim.

Canonical sibling-shared paths retain the 37 previously qualified core image
variants. Each candidate's retained triplanar sidecar contains only its used
source profiles and exact PNGs, with the new GLB hash and a receipt-bound sidecar
hash. This sidecar is **not** part of normal runtime package admission. A missing
session/spawn file deliberately keeps these complete geometry references out of
normal authoritative startup until remaining presentation/gameplay gates pass.

The invisible downward water plane remains in these complete source references;
it has not been adopted as a two-sided walkable gameplay surface. Original source
files, qualified inspector stages, existing packages and default map are preserved.

## Independent checks and actual loader evidence

The verifier does not import the package author. It checks complete source draw
coverage without duplicates, every primitive/material, **2,131 retained accessor
stream comparisons**, exact original stream bytes, material/image/sampler/extension
semantics, custom PNGs, complete unchanged collision data, manifest bindings and
the restored world transforms. Maximum pose difference is 1.42e-14. Four rehashed
pose/material/UV/collision faults reject after rebuilding the GLB/collision/
manifest/index/sidecar hashes; the retained candidate still verifies afterward.

The actual Rust `Terrain::load` loads all ten published collision packages and
checks their visual/shared-image dependencies. The real Bevy/Avian world then
passes all **16,572 object rays and 787,457 terrain vertex/centroid rays**, with
maximum errors unchanged at 0.266 mm and 7.16 micrometres respectively. Geometry
is cooked outside simulation steps. This is headless Linux qualification, not a
rendered connected forest session or Windows execution.

Ten candidate GLBs have **zero Khronos errors and 22 generated-tangent warnings**.
The warnings are repeated retained source materials across shards; source streams
were not silently rewritten. Fresh format/diff checks, denied-warning workspace/
all-target Clippy and 218 asset Python tests pass. The preceding standard-material
checkpoint also passes the complete default network suite, 41 client tests and
native encrypted material/town/farmhouse regressions.

Local evidence is ignored:

- `build/forest-visual-layout-analysis-v1.json`, whole-draw layout measurement.
- `build/forest-shape-visual-reference-v1/reference.json`, package/source ledger.
- `build/forest-shape-visual-verification-v1.json`, independent complete comparison.
- `build/forest-shape-visual-faults-v1/verification.json`, semantic faults.
- `build/forest-shape-visual-native-v1.{json,log}`, real manifest/dependency/Avian run.
- `build/forest-shape-visual-gltf-validation-v1.json`, ten Khronos files.
- `build/forest-shape-visual-clippy-v1.log`, denied warnings.
- `build/forest-shape-visual-python-tests-v1.log`, 218 asset tests.

## Next gates

Continue custom material texture phase under render rebasing and a bounded
runtime material/dependency contract, then source render flags/lighting and
resource/mip behaviour across package roots. Explicit water behaviour, actual
forest movement/camera, bounded lifecycle/resource measurements and normal map
adoption remain open. No navmesh or gameplay systems are inferred from this work.
