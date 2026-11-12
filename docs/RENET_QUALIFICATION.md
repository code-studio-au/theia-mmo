## 12 November 2026 — managed multi-client launcher (P5-17 / P1-06)

`tools/network_playtest.py` launches one authoritative server and one actual native
client per enabled scenario player (one/two total), each with its own encrypted
local ticket and UDP impairment relay. It validates the matching client content
identity, delegates actual spawn collision validation to the server, and keeps
manual Classic input. A schema-1 scenario extends the server profile's player
entries with optional `enabled` (default true) and `network` settings. Disabled
entries do not launch a client; missing network settings add no artificial faults.
Each network object accepts the Profile fields documented below; outage windows
are JSON arrays `[start_ms, duration_ms]`. Script controllers are not supported yet.

Example using this session's checked local exports and Synty art:

```sh
python3 tools/network_playtest.py \
  --scenario build/starter-valley-network-scenario-v1.json \
  --world build/starter-valley-contact-world-v3 \
  --players build/classic-directional-client-qualified-v1-20261103/art/player \
  --binaries target/debug --output build/network-playtest-new
```

The output directory must be fresh. Ctrl-C, SIGTERM to the CLI launcher, writing
`<output>/stop`, an optional `--duration <seconds>`, or closing a client ends the
whole owned session. Client/server exits and partial startup failures clean up
remaining processes/relays; temporary tickets are removed after child shutdown.
Logs are separate per client; `session.json` records settings, relay counters,
stop reason and cleanup errors. `added_base_rtt_ms` is configured delay, not a
measurement of Renet RTT. The first launcher opens separate ordinary windows;
automatic tiling/embedded camera views and the editor Start/Stop UI remain pending.

Twenty-four focused Python checks pass (configuration, lifecycle, impairment and
existing launcher helpers). Actual RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan
runs two manual native clients for 25 seconds: the first has no added impairment,
the second 80 ms each-way delay + 0..20 ms jitter. Both receive distinct authored
spawns and publish their nine-chunk scene readiness. Two test admissions, zero
rejected inputs, zero recoveries, no relay overflow and zero GPU diagnostics.
Automatic shutdown removes all tickets and reports no cleanup errors. Evidence:
`build/network-playtest-tests-final.log` and `build/network-playtest-native-v1/`.
This native run verifies stationary manual clients, not scripted movement/combat,
embedded editor views, persistence, Windows execution or large-scale capacity.

## 12 November 2026 — checked per-client test spawns (P5-17 / P1-06)

The headless server accepts `--test-players <profile.json>` with an authored world,
Classic prediction and loopback local tickets. This is an explicit development
mode, separate from account login/persistence and ordinary default spawn behavior.
Its bounded schema-1 profile contains `world_identity` (the authored Source's 32
hash bytes) and one or two `players`, each with `client_id` 1 or 2 and global XYZ
`feet`. IDs may appear in either order. Missing/unconfigured IDs are refused;
positions never depend on actor allocation or connection order. The local issuer
still writes its normal eight bearer tickets, but only configured IDs can be
admitted. Tokens are single-use under the existing server-lifetime replay guard.

Every spawn must match the source identity and pass prepared static-capsule
collision/support checks before the server binds its socket or issues tickets.
This retains the selected floor beneath roofs. Preparation uses the existing
bounded worker outside the simulation step, with a 15-second startup timeout.
Unsupported heights, overlaps, duplicate IDs, stale sources, excess bytes/counts,
nonlocal listen addresses and mixed account/persistence/travel modes refuse.
Game commands do not carry or choose test spawn coordinates. Logs and counters
use `test_player_admit` / `test_player_admissions`, separate from persistent entry.

The actual encrypted two-peer test starts client 2 first and verifies the distinct
configured spawns, readiness-gated Classic movement, prediction/reconciliation and
refusal of a third correctly authenticated but unconfigured peer. No rejected
movement/readiness inputs or recovery resets. All 482 authority unit tests, three
actual server CLI checks, formatting, strict workspace/all-target Clippy and
client/server/editor builds pass. Evidence: `build/test-player-spawn-*.log`.
Native rendered two-window playtest, controller scripts and editor pawn controls
remain separate follow-ons; this slice does not claim account or quest persistence.

To reproduce with the current local exported valley, and write its matching
profile for the command-line checks:

```sh
THEIA_AUTHORED_WORLD="$PWD/build/starter-valley-contact-world-v3" \
THEIA_TEST_PLAYERS_PROFILE="$PWD/build/starter-valley-test-players-v1.json" \
cargo test -p theia-network --test authored_renet authored_test_players -- --ignored --nocapture
THEIA_AUTHORED_WORLD="$PWD/build/starter-valley-contact-world-v3" \
THEIA_TEST_PLAYERS_PROFILE="$PWD/build/starter-valley-test-players-v1.json" \
PYTHONPATH=tools python3 -m unittest test_test_player_cli -v
```

## 12 November 2026 — configurable local test connections (P1-06 / P5-17)

`tools/network_profile.py` supplies an immutable, checked `Profile` for the existing
loopback UDP relay. `Relay(Profile())` has no added impairment; `Relay(Profile(
delay_ms=80))` adds 80 ms in each direction (160 ms base added RTT). Jitter adds
0..N ms per packet; measured RTT also includes actual transport/scheduling latency.
Loss choices are repeatable by seed, direction and packet ordinal. This is repeatable
packet impairment, not deterministic game replay. Optional outage windows are
relative to relay start and discard pending packets rather than replaying them
when the link returns. No packet contents or simulation timestep are changed.

Delay/jitter are bounded to 5,000 ms each, loss to 0..100%, seed to u32, and at most
16 ordered nonoverlapping outages of up to 60 seconds each within a one-hour test.
The existing 512-packet queue/32-packet receive batch remain bounded. Excess queued
traffic increments overflow; close cancels pending work. Each relay owns one client
connection. No-argument `Relay()` retains the older native qualification fixture's
exact ordinal loss, jitter and duplication rules for compatibility.

Nine profile/real-loopback tests cover independent clients, round-trip delay,
payload preservation, seeded choices, total loss, outage/recovery, queue overflow,
shutdown, invalid configuration and the old fixture. Together with existing launcher
and qualification-helper regressions, 18 tests pass:

```sh
PYTHONPATH=tools python3 -m unittest test_network_profile test_qualify_graybox test_authored_play -v
```

Evidence: `build/network-profile-tests-final.log`; formatting and denied-warning
workspace/all-target Clippy pass. These new checks use actual UDP echo peers,
not authenticated Renet gameplay or rendered split clients. Editor Test Player
properties, per-pawn spawn/controller binding, integration with secure client
launch, and Windows execution remain follow-ons. Existing Renet evidence below
is unchanged and does not qualify the new configurable workflow.

# Renet local replacement qualification

The timings below record the wire-v5 replacement gate. The newer
[coordinate/identity slice](COORDINATES_IDENTITY.md) reruns the same adverse tests
on v6; current raw reports are under `build/coordinates-qualification/`.

2026-10-19, P1-06/07/12 and P2-02/03. The executable runtime now passes the
bounded local replacement gate. `python3 tools/play.py` and the multiplayer
qualifier select Renet by default; `--transport quinn` retains the prior
regression implementation. The no-option standalone server and old `--connect`
client CLI retain their Quinn meaning. The normal launcher explicitly configures
secure Renet tickets for both processes. No transport fallback failure is declared.

## Repeatable real UDP tests

```sh
THEIA_NETWORK_REPORT_DIR="$PWD/build/renet-qualification" \
  cargo test --locked -p theia-network --test renet_adverse -j 1 -- --nocapture
```

The tests run the executable authority loop and two executable client workers,
using actual encrypted UDP. A third endpoint is reserved for a replacement
session. A bounded test relay changes packet delivery without parsing payloads
or supplying game state. Tokens and captured ciphertext remain in memory; reports
contain counters and measurements only. The test uses the existing graybox at
30 Hz, not rendered clients, loaded terrain or a simulated virtual clock.

Each direction uses the following deterministic delivery rules, including during
handshake. Jitter cycles through four offsets from zero to the stated maximum.
Every Nth received packet is dropped; duplicates are added to surviving packets.
Scheduled delivery produces actual reorder. Counters verify all configured faults
were exercised, no relay overflow occurred and presentation ticks/acknowledgements
never went backwards. The moving actor progresses, stop intent settles, and both
clients agree on the stopped position.

| Profile | One-way base + jitter | Loss | Duplicate every | Observed RTT p50 / p95 |
|---|---|---|---|---|
| Baseline | 0 + 0 ms | None | None | 10.03 / 10.18 ms |
| Wi-Fi fixture | 40 + 0–60 ms | Every 10th packet (~10%) | Packet ordinal divisible by 17 | 127.00 / 134.04 ms |
| Poor-link fixture | 100 + 0–150 ms | Every 5th packet (~20%) | Packet ordinal divisible by 11 | 286.45 / 302.66 ms |

Duplicate selection uses original packet ordinal divisible by 17 or 11, after
loss selection; it does not count only surviving packets. Profile names are test
labels, not claims about every real Wi-Fi or Internet connection. RTT percentiles
are samples of Renet's smoothed RTT over a three-second moving window, not
individual ping percentiles. OS scheduling affects measured values. Raw accepted
reports are `build/renet-qualification/{baseline,wifi,poor,outages}.json`.

Across the two peers, maximum average encrypted UDP payload rates per peer were:

| Profile | Client → server | Server → client | Relay queue high water |
|---|---|---|---|
| Baseline | 8,120 B/s | 10,833 B/s | 4 packets |
| Wi-Fi fixture | 9,223 B/s | 11,894 B/s | 38 packets |
| Poor-link fixture | 11,472 B/s | 13,879 B/s | 78 packets |

Rates count bytes entering the relay before impairment, including netcode,
Renet, handshake/ack traffic and configured-loss packets, excluding IP/UDP headers.
They cover each complete run (roughly 4.3–5.9 seconds), not only the sample window.
The relay has 512 queued packet slots, 128 captured client packet slots and bounded
receive batches. It asserts zero overflow. Measured relay storage is a test bound,
not application network capacity.

The predeclared checks are <3,000 ms smoothed RTT, <128 KiB/s average UDP payload
in each direction per peer, no backwards presentation/acknowledgement, continued
movement and eventual exact stopped-position agreement. Delayed profiles must
also observe RTT at least their one-way base delay. These are permissive local
regression ceilings, not production service levels or enforced bandwidth shaping.

## Queue and session recovery evidence

Nonrendered graybox qualification high-water observations: server movement send queues ≤167 bytes,
reliable welcome queue 44 bytes, client movement send queue 70 bytes, at most two
pending actor inputs and one input drained in an observed peer batch. The runtime
still caps each drain at 32 and keeps one superseding input per peer. Configured
Renet send/receive channel payload memory is 8 KiB per channel; this is not a total
process-memory bound. A separate offline queue-pressure case fills each sender
channel to exactly 8 KiB and verifies it refuses another byte without I/O.

During a two-second bidirectional outage on one client, the other client observes
its movement stop through the existing one-third-second input expiry. The actor
remains stationary for the remainder of the interruption and the session recovers
with the same identity. A second outage lasts through the transport timeout:
removal was observed at 5.035 seconds, the old worker exits with an error, and a
fresh ticket creates a new actor. Replaying 384 captured old-session datagrams
through the replacement endpoint leaves that actor stationary, with only its
own acknowledged input sequence and no resurrection of the old actor.

The client publishes optional transport diagnostics through its bounded state
mailbox: smoothed RTT, Renet packet loss/byte rates, sender queue high waters,
snapshot count and stale-snapshot discard count. The authority returns/logs
queue/input batch diagnostics from the same loop used by the executable. These
are observations, not gameplay authority or new wire fields; wire remains v5.

## Application and platform gate

Native regressions and launcher ownership/cleanup evidence are recorded in
[VALIDATION](VALIDATION.md). These use the default Renet selection for graybox,
blacksmith and farmhouse fixtures, plus an explicit Quinn graybox regression.
`tools/verify_launcher.py` runs the actual default launcher with two clients and
checks both Ctrl+C and child-process failure: owned children and temporary bearer
tickets must be removed. That ownership qualifier uses Linux `/proc`; it does
not establish Windows execution.

This passes the local transport replacement gate, not complete P1-07/P2-02 or
production authentication. No external identity provider, Internet admission,
anti-flood guarantee, automatic session resume, prediction, exhaustive adversarial
security review, native Windows qualification or MMO-scale capacity is claimed.
The next foundation slice is live coordinate and identity contracts; see
[execution order](engineering/EXECUTION_ORDER.md).
