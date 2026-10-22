# Measured chunk budgets

Offline measurements qualified, 2026-10-20. P1-03/P1-10/P2-06/P2-07/P2-08,
execution-order row 5. Prerequisites are qualified static partition output,
contributor envelopes, shared image bindings and the shared controller. The
[runtime capacity continuation](CHUNK_CAPACITY.md) now qualifies a 32-package
hard ceiling with explicit session budgets; the default town stays monolithic.
The measurements and proposed wider acceptance below preserve their original
scope; implemented subsets are recorded separately in that continuation.

`tools/measure_chunk_budgets.py DIRECTORY --source-world SOURCE --output NEW_JSON`
measures the candidate index when present, otherwise the active index. Explicit
`--global-origin X Z` reconstructs source spawn positions for displaced exports;
this placement is independent of the index's query-frame choice. Source spawn
collision bytes are checked against their manifest, and their complete footprint
providers must match the partition report. Without a source argument, recovery
providers are explicitly marked as report declarations rather than reconstructed.

Each package's identity, GLB/collision digests, canonical image references, actual
PNG bytes/headers and actual contributor envelope are verified. Shared image
variants are counted once per URI key, not once per package. Geometry bounds are
recomputed from payloads. Existing outputs and input packages are not modified.

## Demand analysis

The analysis uses the current authority's f32 capsule footprint radius
0.35499998927116394 metres, four-metre intentional lookahead and eight surrounding
falling lookahead endpoints. For each axis it enumerates ownership breakpoints
and representatives of all intervening intervals. Every current footprint must
have all nine indexed samples. Each required cell expands to its full actual
geometry-provider closure. Recovery providers are retained in every union.

The falling stencil bounds intentional endpoints within four metres because the
cells are 128 metres wide. One-actor and two-actor maxima are exhaustive over the
resulting provider sets. Up to eight greedy actor placements provide a lower
witness; it is an exact maximum only if that witness reaches the full finite
index upper bound. Otherwise the tool reports that the maximum is unproven.

This is a conservative XZ ownership analysis. It ignores height, obstacles,
actual route reachability and proposed finite world boundaries. Some witnesses
are outside the source terrain rectangle. They are not gameplay routes or live
multiplayer capacity evidence. Any authority footprint/lookahead policy change
requires remeasurement. Presentation and staging can retain additional old
positions, but their package/image union is still within this same finite index.

## Actual town results

Evidence: `build/world-partition-town-budgets-qualified.json` and matching `.log`,
against `build/partitioned-town-qualified-stage` and the unchanged shared-image
source town. There are 59 distinct provider sets in each analyzed actor mode.
All four original source spawns reconstruct the same ten-package recovery set.

| Required package union, including recovery | Packages |
|---|---:|
| Recovery spawns | 10 |
| Recovery plus one arbitrary cell | 16 |
| Recovery plus one full capsule footprint | 18 |
| Recovery plus two full capsule footprints | 22 |
| Up to eight conservative XZ actor placements | 26 |
| All finite indexed packages | 26 |

Five additional footprint witnesses already cover all 26 packages with recovery,
so the last actor maximum equals the finite index upper bound. Falling lookahead
has the same count maxima, at different representative positions. Counting only
one cell per actor would miss the 18-package corner requirement.

| Cost | Initial recovery packages | All indexed packages |
|---|---:|---:|
| Packages | 10 | 26 |
| GLB bytes | 6,348,412 | 8,470,060 |
| Collision JSON bytes | 6,962,516 | 12,805,430 |
| Collision triangles | 38,597 | 72,047 |
| Static colliders | 2,783 | 4,001 |
| Unique shared image variants | 377 | 488 |
| Encoded shared image bytes | 474,111,320 | 698,655,860 |
| Base RGBA8 byte estimate | 1,092,964,352 | 1,514,491,904 |

Static collider counts include one triangle collider per nonempty triangle
package plus movement and camera boxes. They fit the current 8,192-collider
per-residency limit, but this arithmetic does not qualify actual combined BVH
construction, startup support or live replacement. Two collision scenes can
coexist during preparation. Actual client staging creates roots only for added
packages, retaining matching old roots. It can add multiple roots in one
candidate; the earlier original-fixture maximum of one staging root is not a
general runtime cap. Asset caches and deferred cleanup can extend lifetimes.

The image estimate is base-level RGBA8, not measured CPU/GPU residency. It excludes
mipmaps, allocations, copies, staging and driver costs. Verification timings are
warm-cache offline Python work and are not Bevy asset-loading or authority
preparation latency. Do not multiply or extrapolate them into load benchmarks.

## Proposed runtime continuation and acceptance

These are design choices to implement and qualify, not active behavior:

1. Raise the bounded hard package ceiling to 32, keeping existing small fixture
   session budgets unchanged. Use a 26-package budget for this finite town, so
   conservative actor/recovery/lookahead unions cannot exceed it. Do not load
   everything initially or make one monolithic permanent provider. A future
   dataset larger than this proof needs its own measured budget/interest policy.
2. Bind session residency budget into world admission identity and enforce
   checked global chunk addresses in readiness/offer messages. Advance the
   protocol version for the wider contract. A conservative JSON size calculation
   for 32 checked-range IDs and maximum-width headers is 926 bytes for readiness,
   979 for wrapped acknowledgement and 955 for wrapped offer, within the current
   1,200-byte datagram budget. This is a sizing estimate; prove actual Rust
   encoding/decoding, invalid addresses/count rejection and full-set encrypted
   transport before adoption. Keep input/snapshot and queue budgets bounded.
3. Bind explicit finite-world bounds into shared startup/admission metadata,
   separate from package contributor bounds. Preserve the source town's playable
   rectangle and whole capsule margins without introducing walls at internal
   seams. Qualify movement/jump/gravity/recovery at all four outer edges and
   corners, including signed distant/query frames and matching client publication.
4. Qualify actual initial ten-package preparation and combined collision support,
   then delayed/successful multi-root publication, reversal/supersession,
   asymmetric failure/retention and retirement. Measure real load latency, peak
   resident/staging resources and bounded live BVH/render readiness. Preserve
   existing original fixtures and monolithic house/map regressions.
5. Run required Rust checks and native Linux Vulkan checks with exact device and
   protocol evidence before promoting the town or changing the launcher. Keep
   native Windows and MMO-scale capacity claims open.

Tests: `tools/asset_tests/test_chunk_budgets.py` independently checks isolated
actor unions, a four-cell capsule corner with a nonlocal provider, real graph
image deduplication/collider totals, and rejection of changed asset bytes,
incorrect actual contributor bounds or falsified recovery providers. These
measurements close a budget-analysis prerequisite, not runtime adoption or a
whole roadmap task. [Executed evidence](VALIDATION.md).

## Subsequent actual-client startup measurements

The [actual town client resource checkpoint](TOWN_CLIENT_RESOURCES.md) now records
native ten/all-26-package startup and sampled Linux RSS/PSS/DRM counters. It also
corrects offline visual envelopes to transformed local AABBs in a separate index;
source package/image bytes remain unchanged and the maximum cell/initial counts
remain nine/ten. The content arithmetic above is historical, not a substitute for
those measurements. Live candidate/staging/retirement and adoption remain open.
