# Authoritative chunk activation

P1-09/P1-10/P2-06/P2-07 continuation, 2026-10-20. This extends the existing
[bounded residency](CHUNK_RESIDENCY.md) into the actual fixed-step authority and
secure Renet runtime. The normal development tile remains monolithic. The
[Bevy chunk diagnostic](CHUNK_PRESENTATION.md) remains a separate presentation
proof; paired live client/authority residency adoption is the next gate.

## Prerequisites and acceptance

This slice depends on verified immutable packages, checked global/chunk frames,
staged Avian scenes, the shared capsule controller and real Renet input/ack
sessions. It does not depend on prediction or combat. Acceptance requires current
actor checks at publication, safe missing-neighbor movement, failed/stale candidate
rollback, off-tick file loading and continued actual secure UDP actor replication.
These checks close this bounded slice, not all source task scopes.

## Authority contract

`World::with_residency` prepares a bounded chunk world before the runtime starts.
It requires 1–8 trusted global capsule-bottom spawn guesses, resident collision,
a finite global fall plane and initial body support/clearance. Resolved spawns
remain resident for fresh joins and per-actor recovery. Players never become
scene colliders. The content query frame remains fixed for this slice; changing
it through `World::set_origin` is rejected for a chunk world.

The dataset identity hashes sorted chunk IDs/package digests and chunk size.
The version-2 session fingerprint also binds trusted global spawn configuration,
fall plane and initial chunk IDs, so initial-only clients cannot admit with a
different loaded set. It is independent of paths, entry ordering, subsequent
residency changes and the implementation query origin. Different dataset
locations/package revisions or spawn configuration yield different identities.
An activation must additionally match the exact query frame and resident budget.
This is a bounded content identity, not a production region ownership protocol.

`World::plan_chunks` captures the world epoch and active residency generation.
`LoadPlan::load` loads/verifies missing collision packages and prepares the combined
Avian scene on a blocking worker. It does not mutate the live world. A failed load
or abandoned candidate leaves committed residency/counters untouched.

`World::activate_chunks` derives capsule-bottom positions and grounding from the
**current** authoritative actors. Publication does no file loading, does not
advance simulation or change poses, identities, input sequences or motion state.
It rejects:

- An old residency generation, another world epoch/dataset/frame or budget.
- Removal of any trusted spawn or actor's full XZ capsule footprint, including
  skin. Protecting only the centre's owner is insufficient at a seam.
- Capsule penetration into candidate geometry or loss/change of nearby grounded
  support beyond 2 cm.
- Removal/change beyond 2 cm of an airborne actor's existing landing surface,
  checked with a bounded 64 m capsule probe. This covers the supported walk/jump
  controller, not arbitrary flight or high-altitude landing guarantees.

Only a validated candidate replaces the active residency. Failed publications
increment a rejection counter but preserve geometry, generation and committed
load/unload metrics. Publication also protects spawns when no actors are present.
A same-set candidate is a no-op: no generation increase or artificial load count.

During movement, all nine points of the capsule's conservative XZ bounding square
must have indexed resident owners. An unavailable neighbor blocks crossing before
the body straddles it. The controller tries the requested axes, individual axes
using the original normalized speed, then zero horizontal axes. Each attempt
starts from the original motion state; only one fixed step commits, preserving
jump/gravity and avoiding diagonal speed gain. This is a conservative movement
boundary, not a loaded proxy collider or a navigation path. Invalid body state or
a fall below the configured plane uses existing per-actor recovery.

## Runtime and bounds

`renet_runtime::chunk_control` creates a local owner-side management channel;
`run_server_with_chunks` uses the same executable Renet loop as normal sessions.
`ChunkControl::replace` requests a plan, runs preparation with `spawn_blocking`,
and submits it for serialized authority publication. No client-to-server chunk
command or client-controlled residency is introduced. The controller permits one
transaction and one queued management command. Concurrent replacements fail busy;
there is no unbounded backlog. Cancellation retains the permit while a blocking
load is still running. The management loop never awaits file I/O in `World::step`.

Existing limits remain: 64 indexed chunks, 1–4 resident chunks, 8 actors, bounded
local query extents and 8,192 combined collider entries. Staging retains old plus
candidate resources; active counts do not measure peak memory. Returned server
stats include generation, publication rejections, blocked movement steps and
movement recoveries. Load failures return to the owner and do not count as
publication rejections. Broader load latency/memory/soak qualification is open.

## Executed evidence

Nine `chunk_activation` tests use original schema-3 headless package fixtures and
actual Avian scenes. They cover actor movement after preparation, a centre in the
retained chunk with a capsule in the removed neighbor, accepted later publication,
body intrusion, grounded/airborne support loss, stale generation/foreign epoch,
spawn retention, corrupt load rollback, abandoned candidates and canonical
identity/query-frame separation, bounded shared startup configuration, global
frame margin and distant-actor common wire frames. Boundary/jump
cases also check diagonal fallback speed and cover 20/30/60 Hz and origins at 0 and signed billion metres, with no movement
recoveries. Visual binding bytes in these headless fixtures are not rendered GLBs.

The `renet_runtime` test
`real_chunk_authority_loads_off_tick_and_publishes_against_two_live_peers` uses the
actual secure UDP runtime and two actual clients at a distant positive origin.
It observes boundary waiting, jumping, worker-loaded publication, continued
movement/acks, concurrent request backpressure and rejected spawn unload/corrupt
loads without disconnecting the players. Its returned stats require generation 2,
one publication rejection, positive blocked steps and zero recoveries.

No renderer code changes are part of this slice. Earlier native schema-3
qualification remains the client baseline; this headless extension does not prove
paired Bevy streaming. Windows execution is still unqualified.

## Next gate

[Shared startup metadata and opt-in normal initial loading](CHUNK_STARTUP.md)
now connect the descriptor, ordinary authority sessions and actual Bevy consumer.
Next add paired dynamic residency/readiness for real moving actors. Retain old client roots until verified replacement assets
are ready, and hold movement where either required presentation or collision is
not ready. Qualify load/failure/unload at seams and distant render rebases with
real visible clients before making the normal development map chunked. Define
geometry ownership/overlap from the actual tile; do not just replicate the entire
monolithic mesh into every chunk. Predictive streaming, general query-frame
rebasing, large-world budgets, navigation and prediction remain separate gates.
