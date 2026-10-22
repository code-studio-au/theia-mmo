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
