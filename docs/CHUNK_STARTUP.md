# Shared chunk-world startup

P1-10/P2-06/P2-07/P2-08 continuation, 2026-10-20. The normal server and Bevy client
now accept an opt-in `--chunk-world <directory>`. This uses original chunk
packages through the ordinary encrypted session, player presentation, camera,
standard Bevy GLB scenes and live Avian plugin. It is separate from the older
`--chunk-smoke` diagnostic. Normal launcher selection remains the qualified
monolithic town; dynamic paired replacement/readiness is the next gate.

## Shared metadata

`world.session.json` schema 1 is bounded to 8 KiB and rejects unknown fields:

```json
{
  "schema": 1,
  "resident_budget": 2,
  "initial_chunks": [[-1, 0], [0, 0]],
  "spawns": [[126.0, 0.0, 32.0], [124.0, 0.0, 32.0]],
  "fall_plane": -18.0
}
```

Spawns are checked global f64 capsule-bottom guesses, not per-chunk coordinates.
Budget is 1–4, initial IDs are unique/indexed/nonempty and fit that budget, and
there are 1–8 trusted spawns. Initial preparation verifies packages and actual
capsule support/clearance/full residency before server admission or client GPU
startup. Unsupported spawns, invalid IDs/coordinates, empty spawns, unknown
fields and out-of-budget data fail closed. A later disk edit does not mutate an
admitted world. Runtime replacement still uses the bounded worker and current
actor checks in [the activation contract](CHUNK_ACTIVATION.md).

The stable dataset identity binds sorted global IDs/package digests. The version-2
chunk session fingerprint additionally binds trusted global spawn configuration,
fall plane and sorted initial chunk IDs. Initial-only clients must agree on that
selection until the paired readiness protocol is implemented; the actual secure
runtime rejects a different initial set even for the same dataset. Query frame,
paths and capacity budget remain implementation choices. A client can
use a different valid query origin while retaining identical physical content
and admission identity. Actor/render positions remain global truth; static roots
and actual Avian bodies use the client's explicit content frame and subsequent
render frame. Server query scenes remain fixed in their own frame.

The original fixture generator writes both index and startup metadata. The
`chunk_residency` example validates this actual Python-to-Rust startup contract
before its existing load/unload proof. Older residency-only fixtures may omit the
startup file; the normal `--chunk-world` executable never permits that omission.

## Normal client lifecycle

The client prepares the initial collision and streams visual digest verification
before creating the app. It requests one standard GLB handle/root per initial
chunk, preserving handles for the session. All dependency loads must succeed;
empty GLB scene lists and actual loader failures exit without reporting readiness.
Ordinary scene hierarchies render while startup waits for meshes and settled
pipelines. Input remains zero, local presentation remains hidden and smoke
readiness is withheld until the chunk world is ready. The server starts actors
at its trusted spawns with zero input while each client loads.

The ordinary static scenery camera tags and mesh clearance queries include chunk
hierarchies. Live Avian probes compare rendered-frame ground hits with the shared
chunk collision scene every connected frame, including render rebases. The
normal HUD, player identity/acks, remote interpolation and camera remain in use;
players are never added to physical world collision.

This version loads the **initial set only** in the normal executable. Movement
waits at unavailable indexed neighbors using the authority's conservative capsule
footprint rule. The separate owner-side runtime API can activate collision
candidates, but normal clients do not yet subscribe to replacements or exchange
readiness for them. Do not drive that API against these initial-only clients.
Trusted spawns stay pinned, and the four-chunk bound is not an MMO streaming or
memory capacity claim. Original fixture floors remain original fixture geometry;
this does not partition or improve the imported town's model collision.

## Bounded common frames

Following one actor must not exclude another loaded actor or static anchor from
±4,096 m local coordinates. Before producing snapshots the authority restores the
fixed content frame if a newly joined actor is outside the current wire frame;
it follows the oldest actor only when all actors fit the proposed frame. Actor
positions, time, identity and the local coordinate bound are unchanged.

For chunk worlds the client similarly checks static anchors, current replicated
poses, previous local presentation and both remote interpolation endpoints before
rebasing. If a candidate excludes one, it retains/restores the common content
frame. Recovery across a previous render frame's range is handled before any
projection. Rejected rebase attempts do not increment the revision counter. This
is a fixed-scene fallback, not interest management, region handoff or unbounded
rendering. Normal chunk startup also requires valid checked global coordinates
for the query frame's ±4,096 m corners, matching the existing world-origin margin.

The original regression uses two loaded floor chunks and real actors at -2,500 m
and +2,600 m. A headless test first reproduced the old snapshot-frame panic, then
qualifies admission, departure, a later distant join and encoded global truth.
A client test covers distant static roots and old interpolation/recovery poses.
The native separated case requires 5.1 km actual actor separation, both local
avatars rendered, remote entities replicated/projected, live ground agreement
and common wire/render frames. It explicitly does not require distant offscreen
actors to contribute pixels. The ordinary native cases retain both-avatar pixel
checks. No scene-size or MMO capacity claim follows from two sparse planes.

## Reproduce

From the repository root, create a new original fixture directory:

```bash
build/asset-tools/bin/python tools/create_chunk_fixture.py build/chunk-session
cargo run --locked -p theia-network --example chunk_residency -- build/chunk-session

target/debug/theia-server 127.0.0.1:5000 build/chunk-session-cert.der \
  --chunk-world build/chunk-session --renet-tickets build/chunk-session-tickets

target/debug/theia-client --chunk-world build/chunk-session \
  --renet-connect 127.0.0.1:5000 build/chunk-session-tickets/client-1.token
# Run a second client with client-2.token.
```

The existing native multiplayer verifier accepts `--chunk-world`. It uses two
ordinary clients, requires their initial visual root/physics ownership, compares
logical identity using an independent Python encoding, independently checks
capsule mesh clearance and checks actual rendered/network frames through rebases:

```bash
build/asset-tools/bin/python tools/verify_multiplayer.py build/chunk-session-proof \
  --chunk-world build/chunk-session --render-span 2 --replication-span 2
```

Use `--offscreen-remote` only for a separated-actor fixture (the verifier requires
actual separation over 4,096 m). Use `--client-chunk-world <copy>` to qualify a different client query origin
for the same dataset and initial selection.

Run native verification on an owned display with validation enabled, following
[the validation record](VALIDATION.md). CPU tests and cross-builds cannot replace
native rendering evidence. Windows execution remains a separate gate.

## Qualification — 2026-10-20

Fresh locked build, fmt, denied-warning workspace/all-target Clippy and 123 Rust
checks pass; the existing private prepared-map test remains explicitly ignored.
The 42 asset and four GLB-alignment Python tests pass. Source handoff documents
remain byte-for-byte unchanged. [Detailed evidence](VALIDATION.md).

Seven actual two-client native cases pass on AMD RX 7900 XTX/RADV NAVI31 Vulkan
with active Khronos validation: original chunk startup at origin and signed
billion-metre roots; independent shifted client query frame; separated loaded
actors; graybox; and the normal town route. Actual Bevy also rejects a consistently
hashed invalid initial GLB without declaring readiness. All current native cases
use the same rebuilt binaries. Live ground agreement error stays below 1 µm.
Screenshots were inspected for original ordinary and separated actors. Windows
execution, dynamic paired replacement, imported tile partitioning and larger
streaming/memory/performance budgets are not qualified by this slice.

Next, pair dynamic client readiness and retained visual/physics publication with
server movement at unavailable boundaries. Define immutable geometry contributor
bounds/overlap so required collision is loaded even when geometry crosses its
logical owner. Qualify actual imported content and memory budgets before making
the normal launcher chunked. The initial four-chunk bound and sparse original
fixtures do not establish those outcomes.
