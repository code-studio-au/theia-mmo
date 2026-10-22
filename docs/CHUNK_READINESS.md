# Per-client chunk readiness

2026-10-20 continuation of P1-06/P1-08/P2-06/P2-07, execution-order row 5.
Prerequisites are shared initial chunk startup, bounded authoritative activation,
and the shared capsule controller. [Paired handover](CHUNK_HANDOVER.md) now consumes
this barrier in the opt-in normal client path; broader publication gates remain.

Wire/ALPN version 10 carries a reliable client readiness report with world epoch,
monotonic sequence and a canonical set of at most four indexed chunk IDs. The
transport supplies actor identity. Reports cannot specify another actor, geometry,
position or velocity. Both executables must be rebuilt together.

Renet chunk sessions begin with an empty per-actor readiness set. Horizontal
movement requires both authoritative collision residency and the
client's reported coverage of the candidate capsule footprint, including skin.
Transient [sweep boundaries](CHUNK_BOUNDARIES.md) also constrain gravity-driven
sideways sliding. Blocked motion still advances input acknowledgement and keeps
free vertical gravity/jumping. One
unready client does not hold other clients. Reports cannot load authoritative
collision or bypass the controller. Empty reports withdraw movement permission.
Trusted headless controller callers retain their explicit non-network policy.
Chunk sessions on the Quinn prototype are rejected before admission; ordinary
Quinn graybox and imported-map sessions remain supported.

Wrong epochs, unknown IDs and stale sequences are rejected before changing the
actor's readiness sequence. Structural violations disconnect the sender. Control
and movement share the existing per-peer rate budget; the control pump processes
at most eight messages per peer per pass. Client publication coalesces the latest
set and waits for bounded reliable-channel capacity.

Authority coalesces one pending acknowledgement per peer and emits it only after
a fixed simulation step. The acknowledgement binds the exact epoch, report
sequence, chunk set and application tick. Channel-capacity checks retain pending
state when the reliable queue is full. The client ignores acknowledgements of
superseded reports and rejects foreign, unsent or mismatched acknowledgements.
`State::chunk_permission_confirmed` requires the latest report's exact
acknowledgement and a snapshot at or after the application tick. Cycling from A
to another set and back to A cannot reuse A's old acknowledgement. This is the
authority barrier; actual unloading must separately preserve current local and
remote presentation/interpolation footprints.

The normal Bevy client reports its initial set only after standard GLB dependency
loading, hierarchy/pipeline readiness and a live Avian probe. Failed initial assets
never become ready. The default town launcher remains monolithic.

Acceptance covers controller boundaries at 20/30/60 Hz, independent client
release after authority replacement, fresh-session reset, sequence poisoning and
malformed addressing over real secure UDP, and actual two-client Bevy readiness
at ordinary and distant coordinates. See [validation](VALIDATION.md) and
[current status](STATUS.md) for executed checks; implementation is not evidence
that a pending check passed.

## Remaining publication contract

Normal client rendering can now consume offers through [bounded paired handover](CHUNK_HANDOVER.md).
The earlier real UDP replacement test
publishes readiness from a headless peer; it does not prove replacement GLBs were
rendered by that peer. The separate Bevy publication diagnostic remains useful
is retained as an independent publication regression. Current normal paired
evidence and adoption limits are recorded in the handover contract and validation.

Dynamic unloading must retain removed roots and collision until authority has
acknowledged withdrawal and old presentation endpoints no longer need them.
Otherwise a delayed readiness report could leave authority permitting movement
into geometry already unloaded by the client. The present initial-only path does
not perform such unloading. Future publication must also associate live physics
probes with the exact candidate generation, retain the active pair on failure,
bound staging, and specify geometry contributors crossing ownership boundaries.
Tile partitioning, asset-memory budgets and production streaming remain open.

The next bounded paired-publication slice must demonstrate this sequence with
actual secure clients and standard Bevy scenes:

1. Stage and verify one offered collision/visual candidate off the simulation
   thread, retaining the active roots, handles and physics. Coalesce superseding
   offers; bound both active and staging ownership.
2. Report the intersection of old and candidate coverage to withdraw permission
   for removed chunks. Keep the old pair until the exact report is acknowledged
   and the client has a post-application snapshot.
3. Revalidate the candidate against current authoritative poses, the local
   presented pose and both endpoints of every remote interpolation. Wait while
   retained presentation still depends on removed chunks. Preserve trusted
   recovery-spawn coverage. Reuse sweep constraints for gravity-driven sideways
   motion and qualify recovery during actual asset staging before treating this
   as a complete missing-presentation boundary.
4. Publish matching scene roots and actual Avian bodies, then prove the new live
   BVH generation before reporting the candidate's full coverage. Retained roots
   must preserve identity; asset failures preserve the previous active pair.
5. Qualify crossing, delayed peer readiness, reversal during withdrawal, invalid
   assets and superseding offers with live frame/physics and rendered pixel
   evidence. A headless readiness report does not satisfy this gate.

An asset-delayed client can receive a ready peer's new authoritative pose before
its own environment is loaded. The paired slice must gate that remote actor's
presentation on actual coverage, while retaining authoritative state and allowing
the ready peer to progress. Test delayed and failed asset preparation explicitly;
independent local movement gates alone do not prove correct remote presentation.

This sequence is a plan, not implemented rendered replacement. It does not solve
geometry crossing chunk ownership boundaries; explicit contributor envelopes or
partitioned payloads must constrain normal map demand before general adoption.

## Contributor-complete movement cells

Wire/ALPN 10 reports loaded package IDs. With [index schema 2](CHUNK_CONTRIBUTORS.md),
those IDs authorize a movement cell only when they include every package whose
bound geometry contributes to it. Authority residency uses the same closure.
A loaded owner floor does not authorize movement without a nonlocal wall provider.
Full capsule footprints and continuous coverage boundaries still apply; schema-1
experiments retain their limited owner-only semantics. Extent metadata changes
alter dataset identity, so different contributor contracts cannot admit together.
