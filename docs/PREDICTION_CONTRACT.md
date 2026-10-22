# Prediction and reconciliation

Current checkpoint, 2026-10-22: actual Renet completed-step commands and bounded
client reconciliation are the default graybox profile. Linux native traversal,
impaired UDP, teleport/held-jump reset and launcher checks pass. Broader P2-04
qualification remains open. Final executed evidence and limits appear below.

## Original prerequisite analysis (historical)

P2-04 foundation analysis, 2026-10-22. The graybox now has authoritative
continental traversal and render interpolation. Interpolation adds presentation
delay; it is not local prediction. Do not call this feature implemented until
actual client input, server acknowledgement, replay and native latency checks pass.

The existing wire sends held intent at 30 Hz. The authority may run at 20/30/60 Hz,
uses the newest accepted intent each step, and reports its highest input sequence.
A sequence therefore does not identify exactly one simulated step. Server positions
alone also omit vertical velocity, grounded state and held-jump state. Replaying
one movement step per unacknowledged packet would be incorrect, especially at
60 Hz, after a render/transport stall, while jumping or during readiness changes.

Required boundaries before client adoption:

1. Exact bounded kinematic state capture/restore, including held-jump edge state,
   with deterministic replay through the existing controller. Diagnostic counters
   must not become gameplay input or accrue again just because replay runs.
2. Define command-to-simulation-tick semantics and acknowledge completed simulation,
   rather than inferring a replay duration from packet count. Keep authority-owned
   timestep/speed, bounded queues and input expiry; stale epochs/sequences cannot
   poison progress.
3. Bind correction state to world/actor identity, authoritative tick and available
   collision/readiness. Replaying into unloaded or unacknowledged terrain is not
   a valid prediction. Preserve falling while horizontal movement waits.
4. Use bounded history, with explicit reset on overflow, teleport, world change or
   unavailable collision. Reconciliation may correct prediction but never submits
   positions as trusted gameplay truth.
5. Demonstrate immediate local response and convergence over real delayed/lost/
   reordered UDP, actual terrain seams, jump/landing and moving render origins.
   Record input latency, correction distances, history and resource bounds; run
   native client checks and retained map regressions before normal adoption.

This document is a local implementation contract, not a change to the supplied
source roadmap. The source P3-17/18 tasks remain unspecified. The imported-map
experiments remain available; forest fidelity work stays frozen. Saved authored
map packages, navigation, combat and population scaling are independent future work.

## Qualified first prerequisite: kinematic correction state

`kinematic::MotionState` captures vertical velocity, grounded state and held-jump
state. `Motion::restore` rejects nonfinite/out-of-range velocity and contradictory
grounded velocity, normalizes zero, and starts solver diagnostic counters separately.
The current controller's correction bound is −1024..7 m/s; future externally driven
vertical motion needs an explicit contract revision. No position, timestep, speed,
world identity or diagnostic counter can be supplied through this state object.

`crates/network/tests/motion_replay.rs` passes two meaningful checks. At 20/30/60 Hz,
JSON-round-tripped checkpoints in flight and while a jump remains held after landing
replay to exactly the same subsequent positions and gameplay states through the
existing Avian controller. Invalid states/unknown diagnostic fields reject. Evidence:
`build/prediction-motion-state-v1.log`. This qualifies state capture/restore on the
same Linux controller/query frame; it does not establish cross-platform bit equality.

At this prerequisite checkpoint the type was not yet sent or consumed. The later
explicit binary correction profile below carries it; legacy v11 layouts remain unchanged.

## Completed command timing prerequisite

`commands::Timeline` executes exactly one step when the fixed-step owner calls
`advance`. Receiving packets only fills a future window, at most eight commands
(and no further ahead than input expiry). Receipt never acknowledges application
or advances simulation time. Commands bind a monotonic sequence to a target tick;
reordered packets can fill gaps, while conflicting mappings, old sequences,
foreign streams and out-of-window commands leave progress unchanged. Missing
commands hold intent until the existing one-third-second simulation-time expiry.
Freshness is the original receipt tick, so prefetched commands and duplicates
cannot prolong movement after input stops.

`commands::State` records the completed tick, applied sequence, receipt freshness
and held intent. Its `stream_epoch` is **per actor movement stream**, separate from
the world's incarnation. Teleports/recoveries must increment it and discard old
pending commands; outer admission must also validate world and actor identity.
Sequence numbers can restart only in a new stream. The final representable ticks
advance without integer wrap; subsequent advance fails without mutating state.

Six tests in `command_timing.rs` pass, including exact held/gap controller replay
at 20/30/60 Hz, bounded reversed delivery, duplicate flooding, receipt-based expiry
and integer exhaustion. Evidence: `build/prediction-history-v2.log`.

## Bounded reconciliation core

`prediction::History` now implements fixed-step speculation and correction replay,
independently of transport/rendering. A validated anchor binds content/world,
world-incarnation/actor identity, simulation rate, movement stream, completed input
state, global f64 capsule feet, kinematic state and accepted readiness sequence.
It retains at most **64 speculative steps**; reconciliation restores completed
authority truth, discards completed ticks and replays the remaining contiguous
suffix through a supplied collision query. Client prediction optimistically assumes
commands arrive immediately before their target step; the next correction replaces
that receipt assumption with authoritative freshness. Prediction never supplies
trusted coordinates to the server.

Old/duplicate corrections cannot rewind the anchor. Foreign world/actor/rate,
invalid motion, and regressing input/readiness sequences reject before mutation.
A new movement stream or accepted readiness report clears incompatible history.
Missing collision, solver failure, integer exhaustion or a full history restores
the latest authoritative pose and suspends speculation until newer valid truth
arrives. Resetting history in the same stream never reuses command sequences.

`prediction::terrain_step` queries an already-prepared procedural Avian patch using
the existing controller and complete-cell swept readiness boundary. It never builds
or loads geometry. Empty permission holds XZ while jumping, gravity and landing
continue. Physics positions are capsule feet; the runtime's centre-height model
offset must be applied by its eventual adapter, not baked into correction state.

The original seven checks in `prediction_history.rs` pass. The main test starts local movement
before any server step, crosses a real terrain seam and jumps/lands at 20/30/60 Hz.
Its **simulated** delayed/lost/duplicated/reversed command and correction schedules
exercise actual divergence and convergence: each rate replays 737 steps, ignores
75 stale corrections and finishes with four retained steps. Maximum correction is
0.402111/0.268083/0.134038 m respectively; final stopped poses agree within 3 mm.
These correction magnitudes are fault-schedule diagnostics, not a smoothness or
latency target. Other cases qualify overflow/resume, teleports/foreign identity,
replay failure, airborne readiness withdrawal and invalid/exhausted state.
Raw evidence: `build/prediction-history-v2.log`.

## Adoption scope

The graybox launcher now selects the explicit `--prediction` Renet-only profile
by default. `python3 tools/play.py --no-prediction` retains held input and smoothed
server poses. The normal application binaries still require an explicit matching
profile flag; imported maps keep the retained path. The shared driver and authority execute actual
commands/corrections; it is no longer only a headless core. The bounded Linux slice is implemented; broader P2-04
qualification remains open as described below.

The explicit profile integrates command/correction traffic with the actual authority
and Renet client. Remaining qualification covers clock synchronization/input lead;
the eight-tick acceptance window is distinct from the 64-step replay history and
does not establish support for arbitrary RTT (eight ticks is only 133 ms at 60 Hz).
Bind corrections to the exact readiness report and increment the movement stream
on teleport/recovery. Ordinary late/duplicate input must not be treated as a fatal
malformed-packet error. Qualify actual encrypted impaired UDP, stalled clocks,
terrain replacement/query origins, then native camera/animation and retained-map
regressions before default adoption. Rendering still needs presentation smoothing
between fixed predicted steps; the fixed-step core alone is not a frame-rate pose.

## Regression checkpoint

Linux checkpoint before live profile integration:

- `cargo test --locked --workspace -j2`: **214 passed**, zero failed. This includes
  49 client tests and the existing real encrypted UDP/runtime/scheduler regressions.
  Log: `build/prediction-workspace-tests-v1.log`.
- `cargo clippy --locked --workspace --all-targets -j2 -- -D warnings`: pass.
  Log: `build/prediction-clippy-v1.log`.
- `cargo fmt --check` and `git diff --check`: pass.

Build commands use the existing `build/native-sdk` pkg-config/library paths.
The full suite runs with permission for its loopback UDP tests. No renderer,
runtime selection, engine/plugin pins or release binaries changed in this slice;
the previous graybox native GPU evidence still describes its previous release
hashes, not newly rendered prediction. Native Windows and live prediction remain
unqualified. Asset Python tests were not rerun because this slice changes no asset
tools or content.


## Live profile continuation

The normal launcher defaults to prediction for the procedural world;
`--prediction` explicitly selects it and `--no-prediction` retains the old path. Both applications
require the same opt-in mode; a missing server profile times out explicitly instead
of silently running an idle client. Quinn and imported map fixtures keep held-input
movement. Existing v11 snapshot/control layouts remain unchanged; explicit `TC01`
and `TP01` binary tags version the opt-in Renet movement profile separately.

A command datagram is at most 113 bytes: world epoch, movement stream, observed
readiness sequence and up to four redundant monotonic tick/sequence/intent commands.
It contains no position, actor choice, timestep or developer speed. An owner
correction is exactly 128 bytes, including content identity, world/actor identity,
rate, completed input state, global feet, motion, accepted readiness and configured
speed. Strict lengths, tags, flags, finite state and identity validation apply.
Packets cannot advance simulation time; the fixed owner consumes the bounded
command timeline. Old/late/duplicate commands are ordinary rejections, not malformed
peer disconnects. The authority always queries its latest accepted readiness;
observing an older readiness report in an otherwise timely command cannot grant
old geometry permission. Future/unsent readiness rejects.

The live client driver estimates server time from correction arrival and measured
RTT, advances an independent rate clock and gently adjusts phase by at most 0.1
simulation tick per correction. Arrival jitter does not directly set its cadence.
Input lead is bounded by the existing future window; stale corrections (>0.5 s)
or unsupported RTT suspend command production. Bootstrap/catch-up uses held input
without inventing commands for past sampling times. Clock resynchronization retains
sequence allocation. At most four recent uncompleted commands are redundantly sent;
completed ticks are removed, and the producer/worker queue holds eight packets.
The command worker drains a bounded batch and sends its latest redundant packet.

A bounded eight-report readiness history supplies the exact report referenced by
a correction. Prediction uses its intersection with the latest actual loaded-ready
set and prepared collision patch. Preparing a replacement does not erase still-live,
already-authorized overlap; withdrawn geometry cannot be predicted into. The
server's newly accepted report still resets incompatible replay history. Independent
correction arrival can precede a snapshot: terrain/query preparation and render
origin selection follow the latest authoritative owner focus, with safe handling
of old remote poses outside the new local frame. Teleports/recoveries replace the
movement stream and discard queued authority commands.

The rendered client interpolates predicted global poses between fixed steps while
remote players retain server-pose interpolation. The server remains sole movement
authority. Renderer, physics/plugin pins and source assets are unchanged.

Completed intermediate evidence (not the final source checkpoint):

- `build/prediction-udp-v3/prediction-wifi.json`: actual encrypted UDP with 40 ms
  delay + 0–60 ms jitter, 10% scheduled loss, duplicates/reorder at 20/30/60 Hz.
  First local driver response: 29.12 / 2.25 / 8.37 ms; stopped error below 0.01 mm;
  maximum retained history 7 / 9 / 13, zero queue drops or movement recoveries.
  Three expected startup/readiness clock resets per rate. These are headless
  driver timings, not rendered input-to-photon measurements.
- `build/prediction-native-v1/verification.json`: two actual Linux RX 7900 XTX
  clients, Vulkan validation, all four clips, distant teleport/return, streaming/
  retirement, local/remote presentation. About 84 FPS each with validation at
  1280×720. Development speed 32 exaggerates readiness correction distances;
  this is not a performance ceiling or normal-speed latency target.
- The first normal-speed impaired native run captured both clients successfully,
  but its verifier rejected the intentional netcode duplicate-packet log. Its
  artifacts remain in `build/prediction-native-wifi-v1`; it is not marked passed.
  The verifier now whitelists only that exact replay rejection during impairment,
  counts it explicitly and continues to reject every other error/validation fault.
  Normal-speed retirement requires actual eviction and the inventory invariants;
  accelerated traversal retains the larger churn threshold. A final fresh run
  follows the readiness-overlap correction.

These logs remain ignored intermediate build evidence; later checkpoint results
follow below. They do not qualify arbitrary Internet conditions or MMO population.


Held-input reset follow-up: a new movement stream does not imply the client
released Space. While its applied sequence is still zero, the server and replay
controller preserve the body's held-jump latch through neutral bootstrap. The
first actual release command clears it. Authority/replay tests wait longer than
input expiry and confirm holding Space cannot retrigger after teleport; an actual
release/repress still jumps. This is the eighth history test. Evidence:
`build/prediction-reset-latch-v1.log` and
`build/prediction-native-held-v1/verification.json`. The latter checks 1,224 actual
rendered post-reset samples under impaired UDP, with zero latch failures, all four
animation clips, bounded streaming and Vulkan validation. It is the full impaired native
binary checkpoint; its own recorded hashes identify the exact executables.


## Completed native/controller checkpoint

P1-06/08 and P2-04, Linux, 2026-10-22:

| Check | Executed result / raw evidence |
|---|---|
| Full native predicted continent crossing | `build/prediction-continent-v1/verification.json`: actual 16 km traversal, 125 visited chunk columns, 2,345 mesh publications / 2,300 retirements, maximum 81 roots. Final X 15,999.6448 m. CPU/GPU mesh inventories remain bounded; no movement recoveries, skipped server intervals or missed simulation deadlines. |
| Uncapped empty-world performance | RX 7900 XTX / RADV Vulkan, Ryzen 7800X3D, Linux, 1280×720, one client, validation off: **821.70 FPS**, mean 1.21699 ms, worst one-second p95 1.33161 ms over 59,246 post-warmup frames. Same workload's earlier interpolation-only result was 824.14 FPS; this small difference is not evidence of a meaningful performance change. Includes the stationary finite-edge finish; not populated MMO performance. |
| Two native clients under impaired UDP | `build/prediction-native-held-v1/verification.json`: 40 ms + 0–60 ms jitter, 10% loss, duplicates/reorder, Vulkan validation, all four animation clips, normal speed and five teleports/return. 96.77 / 97.47 FPS with validation. History peaks 10 each, zero producer-queue drops. The tour actor records 1,224 held-jump reset samples with zero failures. |
| Continuous native seam correction check | `build/prediction-native-wifi-v3/verification.json`: two clients, same impairment profile, all clips and teleport/return. Largest recorded raw replay pose deltas 0.0357 / 0.0945 mm. These are this run's measurements, not a worst-case guarantee. |
| Actual UDP driver at 20/30/60 Hz | `build/prediction-final-network/prediction-wifi.json`: first local driver response 25.55 / 3.13 / 8.68 ms, stopped errors below 0.01 mm, history peaks 7 / 9 / 13, no movement recoveries or producer drops. These are headless timing measurements, not photon latency. |
| Retained town presentation | `build/prediction-town-regression-v1/verification.json`: two encrypted clients, source interiors/exteriors, camera/animation, 2 m wire/render rebases and Vulkan validation pass with the held-input legacy path. |

Exact executable/character digests are recorded independently by each native
verification file. The full crossing and held-jump runs use release-v5 binaries
(client `aa4222327d795199f5289a5859f056f95bbafcd3389aa925bb75da4a731dc867`,
server `ad465fe18db0325b4d9c8f8aa5f0207ebec2b146c9e9352feefd3e0bb13baef7`).
The earlier interpolation-only binaries are preserved under
`build/graybox-qualified-v4`; the runtime baseline remains selectable without
prediction. Final client tests additionally guard the reverse teleport-arrival
order: an old interpolation track falls back to a bounded new authoritative focus
instead of projecting an obsolete pose into a distant render frame.

Limits: raw pose-delta diagnostics also include readiness/clock catch-up; the
held-jump run's maximum is 1.3361 m on the tour actor (0.1675 m on the continuous
actor), and the accelerated crossing records 1.1330 m. These are retained openly;
not every logged delta is an equal-time prediction error, and these runs do not
establish a universal correction/smoothness budget. Local response, stopped
convergence, body/latch validity and bounded traversal are qualified here. Further
latency/presentation polish, dense populations, authored-map prediction and native
Windows remain open. The applications remain loopback-only prototypes. Combat,
persistence, navigation and saved authored-world packages are unchanged.


### Final client guard and launcher checkpoint

The final client-only guard also passes 50 client tests and a fresh 20-second
native predicted traversal (`build/prediction-final-client-native/verification.json`).
Client SHA-256: `f086fff57b15e3e13ee65d1537cb944644af5bb58dcc020d1b3dc2e0d9779ba6`;
server remains `ad465fe18db0325b4d9c8f8aa5f0207ebec2b146c9e9352feefd3e0bb13baef7`.
This check visits 19 columns, publishes/retires 423/342 terrain meshes, observes all
four clips and has zero producer drops. It uses the same Linux GPU at 1280×720,
validation off; it supplements the full-crossing/impaired v5 evidence above.

`build/prediction-default-launcher-v2/verification.json` passes both interruption
and injected client failure: three owned children removed, temporary tickets
removed, prediction flags on both clients/server and nonzero queued commands.
POSIX cleanup now sends the owned server SIGINT before its existing timeout/kill
fallback, allowing final transport statistics. The first launcher attempt expected
a summary after SIGTERM and failed that assertion; its v1 artifacts are preserved.
Windows cleanup is unchanged; this /proc-based check is Linux only.

Final repository verification, 2026-10-22: **224 Rust tests pass**, zero failures;
10 explicit private-fixture tests remain ignored. All **221 asset-tool Python tests**
and **4 GLB alignment tests** pass. Workspace/all-target denied-warning Clippy,
`cargo fmt --check` and diff whitespace checks pass. Logs:
`build/prediction-commit-workspace-tests.log`, `build/prediction-commit-clippy.log`,
`build/prediction-commit-fmt.log`, `build/prediction-commit-python-tests.log`,
`build/prediction-commit-align-tests.log`. Actual encrypted UDP reports are in
`build/prediction-commit-network`; these remain ignored local evidence, along with
private models, Unity exports, tickets and binaries. Linux execution is qualified;
no native Windows execution is claimed. No engine/plugin pins changed in this
prediction continuation.
