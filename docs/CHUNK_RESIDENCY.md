# Bounded chunk ownership and residency proof

Continuation, 2026-10-20: [authoritative activation](CHUNK_ACTIVATION.md) now
uses this loader on a bounded worker and publishes against current live actors
in the actual Renet loop. Earlier precursor descriptions below are historical;
normal map/client residency remains a separate adoption gate.

Continuation: the [real Bevy publication diagnostic](CHUNK_PRESENTATION.md) now
consumes these descriptors with actual GLB hierarchies, a bounded background
candidate, live client Avian bodies and screenshot-qualified rollback/unload.
The headless proof below retains its own scope; the normal tile remains monolithic.

P1-03/P1-10/P2-06/P2-07 precursor, 2026-10-19. This is an isolated headless proof
using original geometry. It does not change the normal imported tile into chunks,
load client GLBs asynchronously or implement a general movement controller.

## Contract and scope

A logical chunk has exactly one half-open 128 m XZ owner, using the existing
checked global grid: `floor(global_x / 128)`, `floor(global_z / 128)`, including
negative coordinates. `ChunkId::origin` returns its checked f64 lower XZ corner.
The query frame is explicit and independent of that owner. Projection into the
bounded f32 Avian scene subtracts the f64 origin first. Logical ownership is
separate from collision overlap: the original fixture floors overlap by one metre
at seams so a standing capsule can query neighbours without requiring duplicated
logical ownership. General imported geometry/halo authoring remains open.

`world.chunks.json` schema 1 contains `chunk_metres:128`, a checked f64
`query_origin:[x,y,z]`, and `chunks:[{id:[x,z],package:[32 digest bytes]}]`.
It accepts no arbitrary paths: an entry resolves to `chunk_<x>_<z>/` with the
[versioned package](CONTENT_PACKAGES.md) files. Unknown fields, duplicates,
unsupported sizes/schema and out-of-frame IDs are rejected. The index is bounded
to 32 KiB/64 entries. Active budget is explicit, 1–4 chunks; the proof uses two.
Chunk origins must project within ±3,000 m of the query origin, retaining room
for the existing bounded collision payload. No unbounded spatial cache is added.

`Residency::replace` validates requested IDs and budget, checks that protected
actor positions retain their owner, loads/verifies missing packages, constructs
one combined Avian static scene, then publishes the new active set. All I/O and
scene construction are synchronous and **must run outside a simulation step**;
the production authority does not call this loader. A failed candidate leaves
active ownership/collision and committed load/unload counters unchanged. Missing
chunk manifests cannot downgrade to the legacy fixture loader.

Active chunk state and visual-root descriptors change together. One descriptor
names the ordinary GLB and immutable global root for each loaded chunk. These are
not counted as spawned/rendered assets: there is no client consumer yet. Actual
GLB loading/spawning/despawning, asynchronous work queues, frame-safe presentation
publication and replication/session integration are the next acceptance gate.
There is also no hot reload: admitted packages are treated as immutable.

Resident collision has an aggregate ceiling of 8,192 collider entries. Replacing
a set stages candidates before publication and can retain old plus new sets
(at most twice the active chunk-count budget), along with the combined query
scene and per-chunk query worlds. Shape backing storage can be shared. Active
count is not peak memory evidence; no production memory/performance capacity is
claimed. `loads`/`unloads` count committed additions/removals, not failed read
attempts. General streaming budgets and backpressure remain open.

## Original fixture and executable proof

The source generator creates three adjacent original chunks, with matched GLB
and collision floors and a thin wall in the middle chunk. It imports no source-
game content. Run from the repository root with new output directories:

```bash
build/asset-tools/bin/python tools/create_chunk_fixture.py build/chunk-residency-fixture
cargo run --locked -p theia-network --example chunk_residency -- build/chunk-residency-fixture

build/asset-tools/bin/python tools/create_chunk_fixture.py build/chunk-residency-positive \
  --base-x 7812500 --base-z 7812500
cargo run --locked -p theia-network --example chunk_residency -- build/chunk-residency-positive

build/asset-tools/bin/python tools/create_chunk_fixture.py build/chunk-residency-negative \
  --base-x -7812500 --base-z -7812500
cargo run --locked -p theia-network --example chunk_residency -- build/chunk-residency-negative
```

All three actual file/package/Avian runs pass. Logs are ignored
`build/chunk-proof.log`, `build/chunk-positive-proof.log`,
`build/chunk-negative-proof.log`. Each commits three loads/three unloads with two
maximum active chunks, continuous flat ground across signed seams, and a capsule
cast stopping at the thin wall (fraction approximately 0.27474624). After unloading
the left chunk, its distant ground no longer exists in the active Avian scene;
after unloading all chunks both collision scene and visual descriptors are empty.
The far runs use query origins exactly ±1 billion metres in X/Z.

Rust integration tests independently exercise budget/duplicate/index rejection,
protected-owner removal, corrupt/missing candidate packages, rollback of active
collision/counters, and centimetre offsets/owner selection near both far origins.
Python checks bind the real generated GLB triangles and collision profile to the
index's package digests. Synthetic Rust test payloads do not claim glTF rendering.
See [validation](VALIDATION.md) for exact suite/platform evidence.

Next: a bounded standard-Bevy visual consumer and explicit load/publication state
before adopting chunk residency in the normal authority/client path. Then qualify
shared kinematic movement against actual walls/headroom, slopes/steps, stacked
floors and falling/jumping. Existing tile/building routes remain regressions.
This precursor does not close P2-06, general collision, navigation or streaming.
