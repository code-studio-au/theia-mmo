# Prediction and reconciliation

Current continuation, 2026-10-23: saved terrain and same-tick diagnostics pass.
A smoother prediction clock, with first-readiness RTT acquisition, passes the
full 234-test workspace checkpoint; final native comparison passes.
Historical analysis/checkpoints below retain their original scope and binaries.

Measurement continuation implemented, 2026-10-23 — P2-04/P1-12. The saved-terrain
Linux slice now passes; see [saved terrain](SAVED_TERRAIN.md) for current wire v12
and source qualification. The next bounded slice retains post-step poses inside
the existing 64-step history, compares corrections only at matching completed
ticks, and refreshes those poses during replay. Report position/body differences
separately from raw catch-up displacement and incompatible-context resets.
Acceptance: analytical clock-ahead/aligned/replay tests, actual impaired UDP at
20/30/60 Hz, bounded diagnostic counts and native normal-speed reports. The measurement-only sub-slice preserved gameplay,
clock, collision permissions and server authority. The later clock change is
qualified separately below.

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


## Measurement continuation — 2026-10-23

Each retained speculative step now also stores its post-step pose, within the
same 64-step cap. Before restoring a correction, compare its completed authority
tick with that retained pose. Report three-dimensional/horizontal/vertical
position error, vertical-speed difference, grounded and held-jump mismatches.
Replay refreshes retained post-step poses, so later samples describe the updated
prediction. An authority-ahead tick with no predicted counterpart is **unmatched**,
not a fabricated error sample. Stale/foreign corrections and incompatible stream/
readiness resets produce no aligned sample. The original `max_correction_metres`
remains a raw before/after replay displacement for backwards diagnostic continuity.

The driver separately labels pending readiness: an outstanding local report or
accepted cells no longer present in the prepared query patch. Each prediction
stores that label; a comparison is tagged if either its stored prediction or the
current measurement context is pending. Replay refreshes the label as well.
This diagnostic flag never changes input, collision permissions, solver calls,
clock decisions, history resets, authority or the network protocol.

Counts/maxima distinguish all aligned samples from those with stable readiness.
Two fixed eight-counter histograms use inclusive upper bounds in metres:
`[0.0001, 0.001, 0.01, 0.05, 0.1, 0.5, 1.0]`, then `>1.0`. Histogram counts must
sum to their sample counts; stable + pending samples must equal total aligned
samples; aligned + unmatched must equal applied corrections. There is no
unbounded per-correction log or new wire traffic. These are simulation diagnostics,
not rendered frame discontinuities or input-to-photon latency.

Analytical tests distinguish a seven-tick clock advance from an actual same-tick
position/body error, verify replay refreshes measured poses, and retain the pending
label even when the report resolves before the correction arrives. Actual UDP and
native normal-speed measurements below now qualify the bounded diagnostic slice. Intermediate v1 UDP results deliberately include terrain
permission withdrawal and are not a pure continuous-movement error distribution.


### Executed measurement evidence

- `build/prediction-measurement-unit-v2.log`: ten history tests and four driver
  tests pass, including analytical aligned/clock-ahead/replay/context cases.
- `build/prediction-measurement-integration-v2.log`: **34** relevant network,
  command/controller/replay/saved-terrain tests pass, including actual encrypted
  impaired UDP at 20/30/60 Hz. `build/prediction-measurement-client-tests-v2.log`:
  **50** client tests pass. Workspace/all-target denied-warning Clippy passes in
  `build/prediction-measurement-clippy-final.log`. The subsequent full workspace checkpoint passes **230 tests**, with ten explicit
  private-fixture skips (`build/prediction-measurement-workspace-final.log`).
  All 221 asset-tool tests and four GLB alignment tests pass again.
- Saved UDP profile (`build/prediction-measurement-network-v2/saved-prediction-wifi.json`):
  stable readiness yields 172 / 258 / 385 matched samples at 20/30/60 Hz, maximum
  position errors **0.0191 / 0.0155 / 0.0236 mm**. Every stable sample is in the
  <=0.1 mm bucket. Deliberate pending-readiness transitions peak at **0.4020 /
  0.2680 / 0.6701 m**. They remain included in the total-error counters.
- The corresponding generated-terrain profile has one stable-readiness **0.134 m**
  outlier at 60 Hz; all its other stable samples are <=0.1 mm. This is retained
  openly. Packet scheduling/input changes mean these runs do not establish a
  universal maximum error; stopped convergence and bounded history still pass.
- `build/prediction-measurement-native-v2/verification.json`: two actual Linux
  RX 7900 XTX clients, 1280×720, Vulkan validation, 65 seconds, normal speed,
  saved terrain, 40 ms + 0–60 ms jitter, 10% loss and duplication/reorder. No
  scripted teleports/permission withdrawal. Both perform walk/run/jump/stops,
  actual tile replacement/retirement and all four animation clips. **3,237**
  matched-tick samples, all <=0.1 mm; **3,212** have stable readiness, largest
  error **0.005312 mm**. Grounded/latch mismatch counts and producer drops are
  zero. Two authority-ahead samples remain explicitly unmatched. History peaks
  at ten steps each. Server records zero recoveries and missed deadlines.

Exact measured release client:
`fc822e5b937daf8465f51e97f701c617353824dd4c5ac0933e22d4c9b326a5e2`;
server:
`5febc808ec78f24d83b9db4498d73935422e61aa07252b544ca505e2bfccf976`.
Copies are preserved under `build/prediction-measurement-qualified-v2`.
These diagnostics change no command/solver/clock/presentation behavior. The native
run covers a straight route with gait changes and jumps, not every turn, network
or authored collision condition. Simulation agreement is separate from visual
frame pacing: the existing moving-tagged counter still records 290/4,003 and
321/4,011 held samples. Input transitions and presentation timing need separate
classification before attributing those to a particular cause or changing smoothing.


### Render-motion classification continuation (in progress)

P2-05/P2-04/P1-12: new bounded client observations distinguish unchanged desired
input from starts/stops/turns, pending readiness, query-patch/stream/clock resets,
half-second settling windows, >100 ms frame gaps and >=64 m horizontal or >32 m
vertical displacements. Steady observations record held positions and a fixed
speed/expected-speed histogram (inclusive bounds 0.1, 0.5, 0.9, 1.1, 1.5, 2, 4,
then >4). Desired speed follows the existing axes, walk/run and server development
multiplier. Fixed memory; no frame trace allocation or feedback into gameplay.

Two controlled trace tests cover constant global speed and intentional idle/turn
settling, actual held input, loading, large displacement and long frames. The
client suite now passes **52 tests**; denied-warning Clippy passes. These are
classification tests, not evidence of actual rendering. Native normal-speed
observations are being collected in `build/render-motion-native-v1` before any
smoothing/clock behavior changes. The samples observe CPU-side rendered transforms,
not screen scanout. Constant desired input can still be legitimately blocked by
world collision or a network pause; these counters alone do not establish jitter.


### Native render-motion baseline and clock candidate

`build/render-motion-native-v1/verification.json` passes the unchanged gameplay
baseline plus the new classification checks. Two Linux/Vulkan validated clients,
normal speed and the established impaired UDP profile, 65 seconds without scripted
teleports: **285/3,184** and **281/3,162** steady-input frames hold position
(**8.95% / 8.89%**). No >100 ms gaps or large displacements. Maximum observed
horizontal speed/desired speed is **2.58 / 2.80**. Only 1,598/3,184 and 1,645/3,162
samples fall in the 0.9–1.1 speed-ratio bucket. This confirms remaining render
cadence variation on this route despite the closely matching simulation states.
Client SHA `1bb4cb162f459b3c19b3ad72d2e39a678bf7e20d85e5b727d67b6add7ebdeb15`;
server remains the measurement-v2 hash above. Copies:
`build/render-motion-qualified-v1`. Existing qualified saved/procedural behavior
remains recoverable with the launcher's explicit `--binaries` selection.

The next candidate keeps a continuous local prediction clock that includes input
lead. Fresh latency estimates change its rate within **0.95–1.05** of nominal;
they no longer jump the clock's phase or add a separately rounded lead to each
frame's target. Phase errors larger than eight ticks retain explicit resync.
Server simulation remains fixed at its configured rate. No queue/window/expiry,
authority, collision or wire contract changes. Tests oscillate RTT around a lead
threshold at 800 FPS for 20/30/60 Hz and require single-step, regular, non-rewinding
prediction cadence. Native comparison and broader qualification are in progress;
this candidate is not yet claimed to fix the observed render holds.


Clock candidate v1 native comparison (`build/prediction-clock-native-v1/verification.json`):
steady holds fall from the baseline's 8.95% / 8.89% to **8/3,282 (0.244%)** and
**2/3,369 (0.059%)** on the same two-client normal-speed impaired route. Maximum
speed ratios fall from 2.58 / 2.80 to **1.60 / 1.58**. No >100 ms gaps, large
jumps, clock backsteps or grounded/latch mismatches. One stable-readiness sample
has a **0.1673 m** error on the second client; it is retained in the counters.

`build/prediction-clock-paced-v1/verification.json`: actual ordinary VSync/focus
pacing, one normal-speed client, validation off, impaired UDP, 40 seconds,
**60.00 FPS**, 12/1,302 steady holds (0.922%), 1,253/1,302 speed ratios in 0.9–1.1.
A 0.1673 m correction is also retained in this run. This is separate from the
uncapped two-client comparison and is not a same-pacing baseline comparison.

The full v1 saved crossing also passes (`build/prediction-clock-continent-v1/verification.json`):
125 columns, maximum 81 roots, 2,345 loads / 2,300 retirements and the same eastern
endpoint. Uncapped one-client RX 7900 XTX, validation off: 827.08 FPS, mean
1.20907 ms, worst one-second p95 1.31693 ms; warm-cache empty-world evidence.
Copies of these exact executables are in `build/prediction-clock-qualified-v1`:
client `33ec54b76df5a986701afa21f1d58db1bdedea8e1f4fa65d7b72ac4d8a3ac0f4`,
server `fd899f07707315677df60812b1f71d25fd4e49491c884f9fe822d873c2073449`.

Startup follow-up: first terrain may become playable while the link's RTT is
still the unknown initial zero. Acquire the measured prediction buffer once when
both authorized live terrain and a positive measured RTT exist. Bootstrap that
clock with held input rather than inventing past commands. Later latency changes
still use the bounded rate adjustment. A controlled regression demonstrates that
new commands remain future-dated on arrival after RTT discovery; ready geometry
alone must not lock in the unknown RTT. Final v2 full/native checks are in progress.
The earlier partial full-suite attempt was deliberately interrupted after this
startup rule changed; `build/prediction-clock-workspace-final.log` is not a pass.


Final startup/clock source checkpoint: **234 workspace tests pass**, ten explicit
private-fixture tests skipped (`build/prediction-clock-workspace-v2.log`). The six
focused driver tests pass, including ready terrain before RTT discovery and the
20/30/60 Hz cadence regression (`build/prediction-clock-startup-unit-v2.log`).
Denied-warning workspace/all-target Clippy and formatting pass in the final-v2
logs. Final native runs use `build/prediction-clock-native-v2`,
`build/prediction-clock-paced-v2` and `build/prediction-clock-continent-v2`;
all three now pass with exactly one initial RTT acquisition and zero ordinary
clock backsteps per client.


### Final clock adoption — 2026-10-23

Final v2 native comparison passes. Two validated clients on the same normal-speed
impaired route record **0/3,280 (0%)** and **5/3,280 (0.152%)** steady held frames,
versus the original 8.95% / 8.89%. Max speed ratios are 1.61 / 1.48. Body/latch
mismatch counts, producer drops, >100 ms frame gaps and large displacements are
zero. One first-client same-tick sample has a **0.2680 m** stable-readiness error;
all its other 1,610 aligned samples are <=0.1 mm. All 1,615 aligned samples on the
second client are <=0.1 mm. Packet-loss corrections remain possible; this is not
a universal maximum-error or all-input latency guarantee.

Ordinary pacing v2 passes at **60.00 FPS**: **6/1,297 (0.463%)** steady held
frames, 1,248/1,297 speed ratios in 0.9–1.1, largest stable-readiness prediction
error **0.00410 mm**. All 975 aligned samples are <=0.1 mm. Full saved-continent
v2 passes again: all 125 columns, maximum 81 roots, 2,345 loads / 2,300 retirements,
same finite-edge endpoint, **823.40 FPS**, mean **1.21447 ms**, worst one-second
p95 **1.34346 ms** over 59,360 post-warmup frames (one client, validation off,
warm cache). Input held at the finite world edge is legitimate blocking; the
normal-speed cadence budget does not apply to this accelerated endpoint finish.

Exact final client:
`6fe925859aaaa8d62618c3feb0e11575b8be0f4313a2b87c3370070eece94d11`;
server:
`9ef5d4131e82ba2c7d65e47a76ed32406025ddf546fa428c04d25a1aa7100fe4`.
Copies: `build/prediction-clock-qualified-v2`. Source checks: **234 Rust tests**,
ten private-fixture skips, denied-warning Clippy/fmt; Python checks remain 221
asset tests plus four alignment tests. Saved data/character identities are unchanged.

The qualification tool supports `--paced` and an explicit normal-speed,
no-tour `--steady-hold-budget 0.01`. The actual gate rejects both measured old
baseline clients and accepts all three final native/VSync clients; evidence:
`build/prediction-clock-cadence-budget-v2.json`. This is a scoped regression gate
on the documented unobstructed route, not a claim that blocked movement, loading,
Windows, crowds or arbitrary Internet links must meet the same count. The server
still runs fixed 30 Hz by default; only the client prediction clock is slewed.


### Completed-input diagnostic checkpoint — 2026-10-26

P2-04/P1-12 adds the completed speculative input state to each of the existing
64 bounded history entries. Matched-tick measurements compare held intent,
applied sequence and receipt freshness with authoritative input state, and refresh
that recorded state when replay refreshes a pose. Fixed counters/maxima partition
held-intent agreement/disagreement; independent sequence/freshness mismatch counts
remain separate. No gameplay branch, solver, wire, input-window or smoothing rule
changes. Matching intent at one tick does not prove identical prior histories or
establish that a residual error is a solver defect.

All **11 prediction-history tests** pass, including analytical lost-command,
same-intent/different-sequence/freshness and replay-refresh cases
(`build/prediction-input-history-v1.log`). Denied-warning workspace/all-target
Clippy passes (`build/prediction-input-clippy-v1.log`). The earlier complete
workspace checkpoint passed **235 tests**, with ten explicit private-fixture skips,
and includes the [poor-link regression](PREDICTION_LATENCY.md).

The user requested a safe stop during the final diagnostic full-suite relink.
The owned Cargo invocation was interrupted; `build/prediction-input-workspace-v1.log`
is **not a full-suite pass**. Real encrypted UDP reports with these additional
input counters remain unqualified, as does their native presentation observation.
Resume with those checks before attributing the recorded 0.2–0.4 m corrections
to input disagreement. No further soak was started. Work remains local/uncommitted.


Resumed after commit `9497a18`: the new input diagnostics now pass the full
**236-test workspace suite** (ten private-fixture skips), denied-warning Clippy,
formatting and actual saved/generated impaired UDP. The interrupted v1 remains
historical. [Completed input observations and limits](PREDICTION_INPUT_DIAGNOSTICS.md)
record both matching/different-intent outliers, including a **1.6085 m** error on
the severe 20 Hz profile. Stopped convergence passes; high-latency smoothness is
not claimed. An uncontended eight-peer soak is in progress.
