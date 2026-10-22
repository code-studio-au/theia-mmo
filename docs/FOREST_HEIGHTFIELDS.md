# Full-resolution forest heightfields

P1-10/P2-06/P2-07/P2-08 continuation, 2026-10-21. Standard Avian heightfields
preserve every qualified terrain sample and its complete source triangle surface.
One full field and four adjoining fields now pass the native source/kinematic
gate. Existing runtime packages remain unchanged while representation admission
is prepared.

## Exact source and layout

`prepare_heightfield_reference.py` reads the immutable qualified terrain GLB. It
checks the source hash, complete regular X/Z lattice, unique sample coordinates
and every oriented triangle. After accounting for the source X mirror, all
524,288 triangles use Avian's default diagonal. Holes, changed diagonals, reversed
faces, duplicate X/Z samples and irregular spacing reject; no decimation occurs.

The input retains **263,169 vertices**, their exact float32 heights and original
triangles. Triangle-centroid expectations come directly from the original GLB,
independently of the heightfield sampling implementation. Heights are explicitly
column-major. A separate original rectangular-grid native test checks axis order
and diagonal behaviour; a square forest alone would hide a transpose mistake.

The full representation is one 513 × 513 field covering 150 × 150 metres. The
adjoining representation is four 257 × 257 fields covering 75 × 75 metres each,
with exact shared boundary samples and the original zero query frame. Sample
storage is 1,052,676 float-data bytes for the full field or 1,056,784 across the
four fields. These are sample-array payloads, not full resident-memory figures.

## Actual native acceptance

The test uses real headless Bevy/Avian scenes and retains the existing four
full-triangle packages as its baseline. Each representation checks every source
vertex and every triangle centroid, **787,457 ray queries each**, including
outer boundaries and shared ownership seams:

| Representation | Misses | Maximum height error |
|---|---:|---:|
| Retained four triangle meshes | 0 | 0.000007630 m |
| Single full heightfield | 0 | 0.000007153 m |
| Four adjoining heightfields | 0 | 0.000007153 m |

All 2,362,371 queries meet the unchanged 0.1 mm height tolerance. Each scene also
passes 18 real capsule walking/jumping routes across X, Z and corner seams at
20/30/60 Hz, including body clearance, grounded landing and jump height: **54
route runs** total. This is physics representation evidence, not a new networked
authority or navigation result.

Two rehashed/rebound native faults fail. Changing one off-route height sample
causes seven source-surface errors in each heightfield representation, up to one
metre. Changing X extent from 150 to 149 metres causes 6,148 missing queries and
420,863 excessive errors per heightfield representation. The retained triangle
baseline stays correct. Unit guards independently reject malformed source grids
and incomplete/wrong oriented topology.

Fresh format checks, denied-warning workspace/all-target Clippy, the native
rectangular case, the actual private-source case and **217 Python tests** pass.
Execution is Linux, Bevy 0.20.0 / Avian revision
`1070759b2145dc078abedace91dc7fe78013a45c`. No GPU/rendering change or native
Windows execution is claimed.

## Evidence and runtime boundary

Ignored local evidence includes:

- `build/forest-heightfield-reference-v1.json` and receipt, derived from terrain
  SHA-256 `efd983710e32dd7c9fd88441fe4048744bb228e1f57e47effd0fc2b04ae9eb68`.
- Fixture SHA-256:
  `0b80f1cf265232033781943c7ca3ca869758f678f890317bbb491c8c0256d200`.
- `forest-heightfield-comparison-v1.json`: all query counts, errors, routes and
  diagnostic timings; full/four-field initialization measured approximately
  35/39 ms under uncontrolled desktop load. This is not a tick-cost benchmark.
- `forest-heightfield-faults-v1`: both actual semantic rejections.
- `forest-heightfield-clippy-v1.log`, `forest-heightfield-python-tests-v1.log`.

`crates/network/tests/heightfield_reference.rs` contains the default original
rectangular test and explicit opt-in private-source comparison. The normal town
default and opt-in [forest ground packages](FOREST_TERRAIN_COLLISION.md) remain
preserved. Unity TerrainCollider query equality is not claimed; this verifies the
already adopted source-triangle geometry.

Offline `analyze_collision_layout.py` now projects the verified object shapes and
four fields into existing 128-metre owners. The measured maximum per collision
provider is **47,245 shared input triangles, 259,623 instantiated mesh triangles
and 66,049 height samples**; a finite terrain cell needs at most eight collision
providers in this projection. These are collision-only footprints and counts,
not visual contributors, runtime publication or a capacity claim. Evidence:
`build/forest-collision-layout-analysis-v1.json`.

Next: admit bounded shared mesh instances, analytic primitives and heightfields
through a versioned collision profile with explicit expansion/sample budgets.
Preserve schema-1/2/3 behaviour and existing limits. Then qualify actual runtime
authority/camera loading and full source composition before forest adoption.
All changes remain local.
