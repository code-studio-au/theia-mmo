# Renet executable bridge

Foundation slice, 2026-10-19: P1-06/07/08 and P2-02/03. This extends the earlier
secure UDP proof to the real server, Bevy client and development launcher. The
[local replacement gate](RENET_QUALIFICATION.md) now passes and Renet is the
normal launcher default.
See [execution order](engineering/EXECUTION_ORDER.md) for prerequisites and gates.

```sh
python3 tools/play.py --transport renet --clients 2
# Explicit retained regression baseline:
python3 tools/play.py --transport quinn --clients 2
```

Renet and renet_netcode remain pinned to 2.0.0. Secure netcode authenticates issued
tickets and encrypts transport traffic. No unsecure authentication path is used.
The application now uses wire v10 with [paired chunk offers](CHUNK_HANDOVER.md) and [acknowledged per-client readiness](CHUNK_READINESS.md),
[global/frame/identity contracts](COORDINATES_IDENTITY.md)
and [held jump input/shared graybox movement](KINEMATIC_CONTROLLER.md),
current terrain identity, input-only movement, Avian authority, 20/30/60 Hz
simulation, applied-sequence acknowledgements, direct local poses and remote
interpolation. Automatic [render/physics floating origin](FLOATING_ORIGIN.md) is
active; prediction/reconciliation remains open.

## Session and transport contract

The server optionally accepts `--renet-tickets <fresh-directory>` after its
existing bind/certificate positional arguments. The certificate argument is unused
in Renet mode. It binds a loopback UDP socket and generates eight client tickets
with unique transport IDs, 120-second admission expiry and five-second transport
timeout. The random signing key stays in server memory. Tickets are written with
exclusive creation and Unix mode 0600; an existing ticket file makes startup fail.
The launcher uses a private temporary directory under ignored `build/` and deletes
it after terminating its child processes. Direct CLI users must remove their own
expired tickets. Windows uses inherited filesystem permissions; native Windows
execution and ACL qualification remain open.

Each admitted transport ID maps to a newly assigned world actor ID. The reliable
welcome contains protocol version, actor ID, incarnation epoch, world digest and
f64 content anchor. The client checks
these before publishing any poses or sending movement. Pre-welcome snapshots are
discarded; later snapshots must contain the local actor and have increasing ticks.
Departure removes the actor. A spent local ticket cannot create another application
session in that server lifetime; reconnect requires a fresh ticket. Server restart
generates a new issuer key. Actor IDs are process-local, not persistent character IDs.

Reliable ordered channel 1 carries welcome, client chunk-readiness reports and
tagged authority offers/acknowledgements after admission. Acknowledgements follow
a fixed simulation step. At most one pending
acknowledgement per peer coalesces newer accepted reports while capacity is full.
Channel 0 carries unreliable
inputs/full snapshots. Both directions have an 8 KiB budget per channel with
2,400 bytes per Renet update; the application JSON budget remains 1,200 bytes.
Transport polls every 5 ms independently of simulation cadence; these configured
budgets are not measured network bandwidth. Each peer has one pending movement
intent and at most 32 movement plus eight control messages drained per peer per
poll, with disconnect above 180 received messages per second across both channels.
Unsupported reliable messages
and malformed input disconnect the sender. Simulation rejects stale sequences
and expires old intent. Transport I/O uses nonblocking sockets outside the world
step; Bevy uses the existing superseding watch mailboxes on its background worker.
Upstream transport socket draining is not an Internet flood-defense guarantee.

## Qualification scope

Five real-time runtime integration tests cover two actual UDP clients, distinct
actors and applied acknowledgements, movement, departure, fresh-ticket reconnect,
spent-ticket rejection, world mismatch, invalid intent, bounded/exact ticket
parsing, endpoint mismatch, exclusive ticket creation and Unix permissions.
The earlier secure UDP proof additionally rejects an untrusted issuer and checks
channel memory limits. Added runtime cases qualify distant origins/live wire
rebases and wrong-epoch sequence rejection. Existing Quinn tests remain regression coverage.

Native integration evidence and full-workspace checks are recorded in
[VALIDATION](VALIDATION.md). Controlled local loss/delay/reordering, outages,
RTT/UDP bandwidth and queue observations now pass the
[replacement qualification](RENET_QUALIFICATION.md). Renet is the default for
`tools/play.py` and the multiplayer qualifier. Exhaustive rate-limit attacks,
provider-backed character authorization, production ticket delivery, Internet
hosting and MMO load qualification remain open. Quinn remains an explicit
regression path; no Renet failure/fallback decision is declared.

Scheduler diagnostics and the eight-peer 20/30/60 Hz burst/expiry/stall/input-budget
qualification are documented in [scheduler qualification](SCHEDULER_QUALIFICATION.md).
This adds bounded shutdown diagnostics without changing wire v6 or movement authority.
