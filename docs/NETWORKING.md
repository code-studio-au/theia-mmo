Current graybox update, 2026-10-22: the default launcher selects the
[continental graybox](GRAYBOX_WORLD.md), with wire v11, authoritative procedural
terrain and recipient-centred snapshots. The launcher now enables the explicit
Renet prediction profile (`TC01` commands / `TP01` owner corrections) for this world.
The authority executes bounded tick-addressed inputs; the client reconciles a
64-step history and interpolates predicted local poses. Remote actors still use
server-pose interpolation. `--no-prediction` retains the previous graybox path;
imported maps and Quinn stay on held input. Two-client impaired UDP/native checks,
teleport/held-jump resets and the full native continent crossing pass. See the
[prediction contract](PREDICTION_CONTRACT.md) for exact evidence and limits.
Earlier protocol/default/direct-placement descriptions below are history.

# Local authoritative walking slice

The [Renet executable bridge](NETWORKING_RENET.md) is now the default through
`python3 tools/play.py --clients 2` after [local qualification](RENET_QUALIFICATION.md).
The Quinn implementation below remains available through `--transport quinn`.
Follow the [foundation execution order](engineering/EXECUTION_ORDER.md).

Collision now uses the shared pinned Avian 0.8-dev backend in the server and normal
client. See [physics migration](PHYSICS_AVIAN.md). Wire/ALPN 10 adds [authority offers](CHUNK_HANDOVER.md) and acknowledged
[per-client chunk readiness](CHUNK_READINESS.md) to the [global/frame/identity contracts](COORDINATES_IDENTITY.md)
and [held jump/shared movement](KINEMATIC_CONTROLLER.md). Older protocol revisions
cannot connect; rebuild and restart both applications together.
Dynamic paired publication remains open. See [current status](STATUS.md) for
qualification. Earlier versions below describe historical
checkpoints.

Architecture alignment, 2026-10-19: Renet/netcode 2.0.0 are now pinned with secure
transport helpers, explicit bounded channels and real two-client UDP authority
tests. The launcher now defaults to Renet after the controlled local network gate.
See [alignment scope](engineering/ALIGNMENT.md); the Renet proof does not establish
production authentication, ticket replay protection or adverse-network capacity.

This is a small loopback prototype: one Rust server, up to eight admitted
connections, and ordinary Bevy presentation clients. Two simultaneous rendered
clients are the qualification target. Eight is a hard prototype cap, not tested
MMO capacity. There are no accounts, character selection or persistence yet.

## Authority and wire

The server advances the shared XZ world at a configured 20/30/60 Hz (default 30)
using its own clock. Snapshot tick rate drives presentation interpolation. Each player
has a server-assigned, increasing ID, a position and a latest movement intent.
Players have no blocking collision with each other and must also pass through
NPCs when those are added. NPC targeting/hit checks remain separate. There are no
NPC entities in this slice yet. The default/reference ground is flat at Y=0.9
for the capsule centre; world bounds are XZ +/-19. Solid building/door/rock boxes
now block movement through the shared [static layout](WORLD_AND_CAMERA.md).
Players and NPCs are excluded from environment collision.

Clients send integer axes in [-1000,1000], a sprint flag and a sequence number.
They cannot send a player identity, position, speed or elapsed time. The server
normalizes diagonal movement, owns walk/run speeds (5/8 units per second), clamps
bounds and drops stale sequences. Extra packets do not advance simulation. After
approximately one third of a second without a fresh intent (ten ticks at the
default 30 Hz) the server stops that player's movement.
Camera-relative controls produce world-space intent axes on the client; the server
still owns movement and collision. Camera angles are not sent.

Quinn QUIC DATAGRAM carries superseding inputs and full snapshots; one reliable
server-to-client unidirectional stream sends version, assigned ID and a 32-byte
world identity. Snapshots now also include authoritative Y. The ALPN is
`theia-mmo-walk-5`; JSON datagrams have version 5, a 1200-byte maximum and reject
unknown fields. Full snapshots have bounded, sorted unique IDs and finite bounded
positions (XZ +/-1023, Y +/-1024). Each server world enforces its own movement
bounds; the graybox remains +/-19. This is a new prototype wire format, not compatibility with old Theia
Engine protocols. No cross-repository dependencies exist.

Each connection has one latest-input mailbox; the Bevy bridge also uses bounded
latest-value mailboxes. The server allows eight sessions, bounded handshake time,
bounded transport buffers and 180 received input datagrams per second per session.
Malformed input or excessive packet rate closes that connection. A skipped tick
does not trigger an unbounded simulation catch-up burst. There is no distributed
world, interest management, delta compression or packet-loss qualification yet.

## Trust and lifecycle

The server creates an ephemeral self-signed localhost identity in memory and
writes only its public certificate. Clients trust the supplied certificate through
Rustls verification; they do not bypass verification. Stateful 0-RTT is disabled.
The identity changes on restart, so restart clients with the new public file.
The executable currently rejects non-loopback binds/connects. This provides local
transport trust, not player authentication or Internet deployment readiness.

Disconnect removes the player from future snapshots. Reconnecting by launching a
new client assigns a new ID; there is no session resume. The client displays a
disconnect reason, hides pawns and stops at the last camera pose. QUIC runs outside
the Bevy thread and shutdown joins the worker. The local pawn uses server positions
directly, with no local prediction yet. Remotes blend toward each received pose
over one snapshot interval; this is basic interpolation, not a jitter buffer or
latency-compensated combat. R reset is available only in the offline sandbox.

## Provenance

Reviewed `theia-engine` at `a65a187e27b4b69ed6abdaf94b704fda5315bbab`, specifically
`crates/theia-network-fixture/src/{lib,tls,wire}.rs` and fixture manifests. The small
TLS/transport configuration pattern in `crates/network/src/tls.rs` is adapted from
that project's TLS helper: exact certificate trust, Rustls/Quinn configuration,
explicit ALPN, disabled 0-RTT and bounded transport. The world, JSON walking wire,
server sessions and Bevy bridge are new. The old fixture's fixed two-actor combat,
replication schema and benchmark machinery were not imported. The old repo remains
independent and is not needed to build/run this workspace.

Run commands and actual qualification evidence belong in README and VALIDATION.

An optional [terrain walking fixture](TERRAIN_WALKING.md) now uses the same authority
and transport with source-triangle height and slope checks. The welcome identifies
the exact terrain profile/visual pair, rejecting mismatched clients. Graybox and
reference-art clients explicitly request the default world identity. Version-1/2
executables must be rebuilt; their ALPN/coordinate limits are incompatible. The
[complete development tile](DEVELOPMENT_WORLD.md) uses this same authority, shared
world identity and presentation path, with profile-specific bounds and spatial
lookup for terrain/prop clearance. Buildings use solid envelopes and water cells
are excluded until further gameplay is implemented.
