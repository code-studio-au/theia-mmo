# Bounded authority scheduler qualification

2026-10-19, P1-05/P1-06/P1-12. This qualifies the existing executable Renet
scheduler with eight real authenticated UDP clients and the graybox Avian query
scene. It does not establish MMO population capacity or full-map performance.
The client/server protocol, normal 30 Hz default and presentation are unchanged.

## Runtime contract

The server advances one fixed simulation step at each selected 20/30/60 Hz timer
wake. Tokio intervals use `MissedTickBehavior::Skip`. An overdue wake can execute
one step immediately; missed wall-clock periods are skipped rather than replayed
in an unbounded catch-up loop. Simulation time therefore slows during overload.
Both the 5 ms transport pump and simulation execute on the same authority loop;
a blocked executor delays both. Neither the simulation step nor its replication
encoding awaits network or database operations.

Input expiry uses **simulation ticks**, not a wall-clock deadline. Its nominal
10/30-second duration rounds upward to 7/10/20 ticks at 20/30/60 Hz. A stalled
server cannot enforce a wall-clock movement timeout while it is not executing.
Admission/session transport timeouts remain separate from gameplay expiry.

Each admitted peer has one superseding pending intent. The pump drains at most
32 movement messages per peer per poll and disconnects a sender exceeding 180
received messages in its one-second receipt window. That window measures server
receipt time: delayed traffic can arrive together. Each Renet channel is bounded
to 8,192 bytes per peer; capacity remains eight peers. Strict schemas and world
incarnation checks precede coalescing; authority rejects already applied sequences.
This is bounded application traffic handling, not Internet flood protection:
netcode packet reception and decoding are additional pump work.

Shutdown's existing `transport_summary` JSON now includes player/input high-water
marks, accepted input count, configured rate, wall duration, skipped simulation
intervals and separate simulation/pump timings. `simulation` includes pending-input
application, Avian movement, snapshot encoding and Renet enqueue; `io` includes
transport update, admissions/departures, decode/coalescing and packet sends.
Missed deadlines mean work plus wake lateness reached that loop's budget; they
are distinct from the number of skipped scheduled simulation intervals.

Timing storage is fixed: 64 logarithmic nanosecond buckets per histogram, with
p50/p95 reported as **inclusive upper bounds**, normally powers of two minus
one. The last overflow bucket reports `Duration::MAX`. Exact maxima are reported
separately. Samples cover the entire session, including startup, expiry, the
controlled stall, recovery and the input-budget violation; the separately timed
three-second moving window measures delivered wall-clock cadence.

## Cold physics initialization regression

Workspace verification exposed a startup deadlock in the existing shared graybox
query scene. Captured Linux thread stacks showed a `OnceLock` initializer running
nested Bevy startup while other shared-pool workers waited for that same scene.
Nested schedules could also exhaust the shared pool independently of the lock.
The fix builds the tiny immutable scene outside the publication lock and uses
Bevy's standard single-thread executor for its startup schedules. Concurrent cold
callers may build duplicate tiny scenes; only one completed scene is published.
The live client physics schedule and collision geometry are unchanged.

`physics_initialization.rs` starts eight simultaneous Bevy callers with only two
compute workers, checks real ground queries and imposes a ten-second completion
timeout. It failed before the combined fix and passes afterward. Network test
dev-dependencies explicitly enable Bevy's shared task pool, matching client/workspace
feature unification; normal headless production dependencies do not gain rendering.

## Repeatable acceptance

`crates/network/tests/scheduler_qualification.rs` calls the same
`run_server_measured` loop as the executable. A dedicated authority OS thread
uses a single-thread Tokio executor. Eight raw secure Renet clients run on a
separate executor and continue sending while the authority is deliberately
blocked for 250 ms. The blocking operation exists only in test code; there is no
production stall switch or synthetic work in gameplay.

At each rate the test:

1. Admits all eight peers and checks actual eight-player snapshots/rate.
2. Moves them for three wall-clock seconds, with four messages per nominal
   1/30-second send: two new intents, one replay, one wrong-epoch `u64::MAX`
   sequence. Actual send counts are recorded; timer rounding means the offered
   rate is below the nominal 120 messages/second/peer. Checks monotonic acks,
   real movement and speed bounded by elapsed simulation ticks.
3. Stops sending while keeping sockets alive, waits 650 ms, then checks unchanged
   positions for another 200 ms. Resumes movement in the opposite direction.
4. Stalls only the authority for 250 ms while clients keep sending. Checks skipped
   intervals, reduced simulated time, fresh acknowledgements and movement recovery.
5. Floods one authenticated peer with 64-message bursts and checks its removal,
   seven surviving peers, bounded pending inputs/batches/channel memory, and
   coalescing many received messages into fewer applied intents.

Run from the repository root (absolute report paths keep artifacts in ignored
root `build/`, since Cargo executes tests from the crate directory):

```bash
THEIA_SCHEDULER_REPORT_DIR="$PWD/build/scheduler-release" \
  cargo test --locked --release -j 1 -p theia-network \
  --test scheduler_qualification -- --nocapture
```

The regular workspace test suite also executes the qualification in debug mode.
This is a finite timing acceptance test, not a soak or statistical capacity study.

## Evidence and limits

Native Linux x86_64, AMD Ryzen 7 7800X3D (8 cores / 16 threads), Linux
7.0.0-38-generic, Rust 1.99.0. Release profile retains debug information. Eight
actual peers, in-memory graybox, local loopback UDP; no rendering, database,
NPC/ability load, asset streaming or external network. Raw client pumps model
traffic and acknowledgements; earlier native Bevy integration evidence remains
in [validation](VALIDATION.md), rather than being counted as this load run.

Final release artifact: ignored `build/scheduler-release-final/scheduler.json`
with `build/scheduler-release-final.log` (all three profiles pass).

| Hz | Baseline ticks / wall time | Session simulation samples | Work p95 upper (µs) | Exact max work (µs) | Max wake lateness (ms) | Skipped intervals |
|---|---|---|---|---|---|---|
| 20 | 60 / 3.002 s | 126 | 32.767 | 4613.365 | 226.321 | 4 |
| 30 | 90 / 3.002 s | 187 | 32.767 | 30.717 | 232.295 | 6 |
| 60 | 180 / 3.002 s | 372 | 32.767 | 35.857 | 247.268 | 14 |

Each profile offered 4,628 input messages across all phases; the authority decoded
4,305. Applied intents were 758/1,026/1,033 at 20/30/60 Hz, demonstrating coalescing;
1,078 rejected messages include the intentional wrong-epoch inputs and the budget
violation. Unreliable traffic and the removed sender mean offered and decoded
counts need not match. All profiles reached eight pending slots, with at most
23 messages drained from one peer in a pump (configured ceiling 32). Movement
queue maxima were 565/603/605 bytes, and reliable-event maxima 76 bytes, below
the 8,192-byte channel limits.

All profiles recorded one simulation deadline miss from the injected stall. Pump
work p95 upper bounds were 131.071 µs; exact maxima 621.599/711.647/647.498 µs.
Pump deadline misses were 2/1/1. These include timer/OS scheduling, startup and the
intentional stall; they are not a zero-jitter or hard real-time guarantee. The
20 Hz run includes the first cold shared graybox query-scene initialization
(4.613 ms maximum step); subsequent rate runs reuse that cache in this process.
The recovery windows delivered 14/21/40 ticks in about 0.904 seconds, confirming
that missed wall time was skipped while fresh inputs and movement resumed.

No native Windows or Windows cross-build evidence is claimed.

The next foundation gate is the shared content/collision and general controller
contract (P1-09/P1-10, P2-06/07/08), preserving current building/map regressions.
General action commands, crowd/render baselines, sustained overload/soak,
production authentication and large-region capacity remain open. No entire
engineering phase or wave is closed by this bounded scheduler gate.

## Shared-controller rerun — 2026-10-19

After graybox authority adopted the [shared capsule controller](KINEMATIC_CONTROLLER.md),
the same release qualification passed again at all three rates with eight actual
secure UDP peers. Artifact: ignored
`build/kinematic-scheduler-release/scheduler.json` and matching log. This run
included background compilation and is correctness/load evidence, not an isolated
CPU benchmark or a direct performance comparison with the earlier walker.

| Hz | Baseline ticks / wall time | Session simulation samples | Work p95 upper (µs) | Exact max work (µs) | Skipped intervals after controlled stall |
|---|---|---|---|---|---|
| 20 | 60 / 3.006 s | 125 | 131.071 | 251.149 | 4 |
| 30 | 90 / 3.004 s | 186 | 262.143 | 240.840 | 7 |
| 60 | 180 / 3.001 s | 372 | 262.143 | 337.223 | 14 |

Counts include the full admission/expiry/stall/flood sequence, not just baseline
movement. Histogram bucket bounds can exceed the exact maximum. At most eight
pending inputs, 21-message batches, 661 movement-channel bytes and 76 event-channel
bytes were recorded. The intentional 250 ms stall still skips intervals rather
than running a catch-up burst. Budget removal affects the offending peer only.
30 Hz remains the default; these small local graybox runs do not establish
full-map, crowd or production capacity.

## Imported-controller adoption rerun — 2026-10-19

The final shared controller, including downward mesh-contact refinement and
observable recovery, passes the same release graybox qualification again.
Artifact: ignored `build/map-kcc-scheduler-release/scheduler.json` with matching
log. Eight actual secure loopback UDP peers at each rate; native map rendering
ran concurrently, so these numbers are load/correctness evidence rather than an
isolated benchmark or a full-map authority timing claim.

| Hz | Baseline ticks / wall time | Session simulation samples | Work p95 upper (µs) | Exact max work (µs) | Skipped intervals |
|---|---|---|---|---|---|
| 20 | 60 / 3.003 s | 125 | 262.143 | 280.343 | 4 |
| 30 | 90 / 3.003 s | 186 | 262.143 | 267.299 | 7 |
| 60 | 180 / 3.003 s | 372 | 262.143 | 1149.276 | 14 |

Each profile still expires stale input, skips missed intervals during the
intentional 250 ms stall, recovers under fresh traffic and removes only the
budget violator. At most eight pending inputs, 20-message batches, 657 movement
bytes and 76 event bytes were recorded. 30 Hz remains default; imported-map
crowds, production traffic and long-duration capacity remain unqualified.
