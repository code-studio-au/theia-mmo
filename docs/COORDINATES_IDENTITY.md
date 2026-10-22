# Runtime coordinates and network identity

Chunk-world continuation, 2026-10-20: [normal chunk startup](CHUNK_STARTUP.md)
keeps a bounded common frame when following one actor would exclude another
loaded actor, static anchor or previous presentation endpoint. Distant joins and
recovery cannot force unchecked wire/render projections. Native original-scene
checks include actors 5.1 km apart; this is not region ownership or a capacity
benchmark. Earlier sections record the preceding bounded fixture contracts.


The [client render/static-physics continuation](FLOATING_ORIGIN.md) now adds
automatic local rebasing. Fixed-client-anchor descriptions below preserve the
initial migration; server content/query anchors still remain fixed.

2026-10-19, bounded P1-02/03/04/08 and P2-03/05 slice. Connected authority now
stores checked `GlobalPosition` values in f64. Collision, content and rendering
remain bounded local f32 frames. Renet stays the normal launcher transport; the
Quinn regression path follows the same coordinate/identity protocol.

## Runtime contract

| Boundary | Representation and owner |
|---|---|
| Authority | Checked global f64 metres, Y up, conservative ±1e12 m limit; no client positions are accepted. |
| Fixture collision | Immutable content anchor plus local Avian queries. Each tick subtracts that anchor in f64 before casting to f32, then checks the resulting global pose. |
| Replication | Checked f64 origin plus bounded f32 actor offsets; reconstruct global positions before projecting into another frame. |
| Client presentation | Reliable welcome establishes the content/render anchor. Local and remote interpolation endpoints, camera targets and diagnostic routes are projected into that anchor. |
| Spatial addressing | 128 m `ChunkId` ownership uses floor on global XZ. `RegionId` groups integer chunks using checked nonzero size and floor division; no region owner/process routing is implemented. |

The server's wire frame recentres when its oldest connected actor exceeds the
configured span in X/Z. The default span is 128 m; snapped frame offsets are
relative to the immutable content anchor. Neither global authority poses nor
static collision geometry move when the wire frame changes. Clients count
accepted origin changes and project both sides of interpolation into the same
content frame, avoiding a false movement jump. Frame validation rejects global
out-of-range values and any snapshot that cannot project into the admitted
content frame. The local bound is ±4096 m per axis; the supplied fixture remains
far smaller. Its existing slopes, wall collision and nonblocking actors remain.

For qualification or a bounded fixture at a different global location, the server
accepts `--world-origin x,y,z` and `--replication-span <1..1024>` after its existing
positionals/options. Origin changes are startup configuration, locked once actors
join or ticks advance, with local-frame headroom required inside the global bound.
The normal launcher retains origin zero and span 128. Profile `source_origin`
remains export provenance: it is not silently treated as a global Y-up placement.

This is live global authority/relative replication, **not automatic floating-origin
rendering/physics or chunk streaming**. The content/render/Avian anchor is fixed
for the admitted fixture. Moving beyond its bounded frame requires later content
ownership and coordinated render/physics rebasing work. No global f64 renderer,
region handoff, persistent identity or unrestricted large world is claimed. The
offline movement sandbox remains local f32 and never establishes network truth.

## Identity and wire v6

`theia-core` provides checked nonzero `WorldEpoch`, `EntityId` and `EntityKey`.
Each world incarnation gets a fresh random nonzero u64 epoch. Actor numbers are
monotonic/non-reused within that incarnation, and connection ownership determines
which actor receives an input. A logical network identity is `(epoch, actor ID)`;
netcode ticket/transport IDs are separate. Disconnect removes the actor. A later
join receives a new actor number; a new process may reuse numbers under a fresh
epoch. These are not database account or character identifiers.

Wire/Quinn ALPN advances to **6**; Renet protocol ID incorporates that version.
Rebuild both applications and issue fresh tickets after the change. The shared
76-byte little-endian reliable welcome contains version, actor ID, epoch, world
digest and three f64 anchor coordinates. Both transports validate it identically.
Snapshots carry epoch/origin, validated tick rate and ordered bounded actors.
Inputs carry the admitted epoch, monotonically increasing sequence and intent;
they still contain no actor ID, position, speed or elapsed time.

A wrong epoch is rejected before it can supersede a pending input or update the
connection's sequence gate. Clients reject snapshots from another incarnation,
without their admitted actor, or outside their frame. The public epoch is a
lifecycle namespace, not authentication: existing secure Renet tickets and pinned
Quinn certificate trust still control admission. It adds no provider login or
resume. The eight-peer/1,200-byte JSON budget remains, including worst-case tested
IDs/sequences, long origins and near-limit offsets. The welcome has its own exact
fixed length and rejects truncation/trailing bytes, zero IDs/epochs and bad origins.

## Executed checks

Core/unit tests cover negative chunk/region seams, nonzero identities, global/local
round trips, wrong/out-of-frame coordinates, restart epochs and stale commands,
wire bounds, and authority equivalence at positive/negative billion-metre anchors.
The actual Renet runtime tests cross multiple wire frames at both distant anchors,
verify continuous local speed, global reconstruction and observer convergence.
A raw authenticated client sends an old-epoch `u64::MAX` sequence followed by a
valid sequence 1; both executable transports must accept the valid input. The
controlled adverse-network and old encrypted-packet replay suites also rerun on v6.

Native evidence and exact totals/devices are in [VALIDATION](VALIDATION.md).
Diagnostic reports contain global positions, fixed content origin, changing wire
origin, incarnation and frame-change count. The verifier compares rendered local
poses against independently subtracted global positions, in addition to the
existing collision, camera, animation and two-client authority checks.
