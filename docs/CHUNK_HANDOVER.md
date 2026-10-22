# Bounded paired chunk handover

2026-10-20, P1-06/P1-08/P1-09/P2-06/P2-07/P2-08, execution-order row 5.
Prerequisites are verified startup/residency, authoritative activation,
[readiness acknowledgement](CHUNK_READINESS.md) and
[movement boundaries](CHUNK_BOUNDARIES.md).

## Implemented path

Wire/ALPN 10 adds reliable tagged acknowledgement/offer events. Offers bind epoch,
monotonic authority generation and at most four canonical chunk IDs. Client state
coalesces the latest offer; snapshots retain it. Normal initial chunk startup still
binds the initial selection into admission. Both executables need rebuilding.

The opt-in server `--chunk-stream` policy uses a bounded four-metre lookahead,
protecting current actor footprints and recovery spawns within the resident budget.
Falling actors can request neighbors regardless of input direction, including
while input points uphill. Spare packages stay resident until a new load needs
the budget; movement within existing coverage does not issue unload-only offers
that would supersede a delayed prepare. Explicit replacement/unload APIs retain
their existing behavior. One owner task prepares
collision off-thread through the existing bounded controller; publication
revalidates current authority state. No package I/O occurs in `World::step`.

The actual normal Bevy client keeps its active roots/handles/physics while one
candidate prepares on the I/O task pool and standard GLB dependencies load. New
roots stage hidden. Asset/hash/dependency failures retain the active pair and
suppress retries for the same failed offer. Superseded worker results are
discarded before another worker starts; obsolete staged hierarchies are removed.

Before retirement the client reports the intersection of old and candidate
coverage, then waits for the exact acknowledgement and a post-application
snapshot. Shared validation protects full footprints for current snapshot roots,
the presented local root, both remote interpolation endpoints and trusted
recovery spawns, including penetration/landing support. Retained visual roots
keep their actual Bevy entity/hierarchy. Publication replaces actual static Avian
bodies with the matching scene and advances the client physics generation.
Zero-delta frames refresh the standard query tree on generation changes.

Full readiness is withheld until Last-stage queries agree with the new generation
and standard render pipelines are ready. Live queries check loaded and removed
chunk columns, not just collider counts. A delayed client withholds remote roots
outside its active coverage while keeping their authoritative state; a ready peer
continues independently. Diagnostic surface probes project known world points
into screenshots to verify visible replacement floor or its absence on rollback.

## Acceptance and scope

Original three-chunk scenes qualify delayed preparation, exact retirement barriers,
retained roots, real scene/BVH replacement, failed replacement rollback and signed
distant/query-frame changes. `THEIA_CHUNK_PREPARE_DELAY_MS` is a bounded local
fault-injection setting (0 normally, at most 10 seconds), not a gameplay control.
See [current status](STATUS.md) and [executed evidence](VALIDATION.md) for checks
that have actually passed.

The launcher remains the monolithic town. The opt-in policy is an original-fixture
proof. The schema-1 index still does not guarantee crossing geometry; the
[schema-2 contributor continuation](CHUNK_CONTRIBUTORS.md) adds bound extents,
complete-cell coverage and actual collider/Bevy validation. Actual imported tile partitioning, texture/
material sharing and measured memory/load budgets remain required before normal
map adoption. Full reversal/superseding-offer native scenarios, asymmetric asset
failure, larger crowds, native Windows and production streaming remain separate
qualification gates. No whole task or phase closes with these bounded fixtures.
