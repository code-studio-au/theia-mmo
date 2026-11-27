# First authoritative ability — qualified development slice, 2026-10-28

P1-11/P2-12, following the qualified opt-in targeting slice. Prerequisites:
validated bounded ability data, checked entity identities/global positions,
fixed-rate World authority, terrain collision/readiness, and nonblocking targets.
The bounded NPC ability slice now passes full regression, actual encrypted UDP,
native Linux Bevy/Vulkan and launcher checks. The original contract and dated
verification history below distinguish that result from broader combat work.
Preserve normal movement and targeting profiles until replacement checks pass.


## Current result and run

```sh
python3 tools/play.py --combat
python3 tools/play.py --combat --clients 2
# Optional alternate validated catalog:
python3 tools/play.py --combat --abilities content/gameplay/development-abilities.json
```

Tab selects/cycles; Digit 1 casts the first catalog ability; Backspace clears.
WASD/Shift/Space retain movement, and Escape exits. Arcane bolt deals 20 damage
with a one-second mobile cast, two-second cooldown, 1.5-second GCD and 300 ms queue.
The target has 100 health and respawns after five simulation seconds with a new
entity ID. The normal launcher retains its existing saved-terrain/prediction
default; `--targeting` remains a separate selection-only regression mode.

**272 workspace tests pass**, ten explicit private-fixture skips:
`build/combat-workspace-v2.log`. Denied-warning workspace/all-target Clippy,
format/diff checks pass (`build/combat-final-clippy-v2.log`,
`build/combat-fmt-final-v1.log`). Nine actual-World combat tests cover all three
rates, including same-step cancellation when a later actor kills the target.
Four codec and four real encrypted transport tests pass within that full run.

`build/combat-native-wifi-v2-20261028/verification.json` passes on Linux Ryzen 7
7800X3D / AMD Radeon RX 7900 XTX (RADV NAVI31), Mesa 26.2.5 kisak, Vulkan,
1280×720, optimized binaries, Khronos validation enabled. Two actual clients run
for 35 seconds over 40–100 ms per-direction delay, 10% configured loss, duplicate
and reordered loopback UDP. The first native attempt failed its origin gate and
exposed the now-fixed actor-order cleanup issue; its history is retained below.

- Both sampled client reports observe 15 damage events, three defeats and two
  fresh-ID respawns, with no event gaps, input pressure or rejected requests.
  Moving-cast telemetry records 157 / 165 actual authority-position changes.
  Inspected game-window screenshots show damage/health, same-step cast cleanup
  on defeat, retirement and a selected fresh target at 100/100 health.
- Both clients observe origin revisions 1 and 2, zero target-transform conversion
  error and zero target collider samples. Maximum 81 terrain roots, 385 loaded /
  304 retired per client; shared live ground-query disagreement is at most
  1.55 micrometres. All four existing locomotion clips continue to animate.
- Server: 18 accepted casts, zero rejected/ignored casts, 74 combat states and
  84 event deliveries queued across peers. Reliable queue peak 521 bytes; no
  backpressure disconnect, skipped interval, movement recovery, simulation or I/O
  deadline miss. Fixed-step maximum work is 0.348 ms at 30 Hz. Server totals span
  final process cleanup; client counters are the periodic observed prefix.
- 77.09 / 78.44 FPS with two active clients, validation and instrumentation is
  diagnostic timing, not an ordinary-play or populated-world ceiling.

The same binaries pass the default no-combat native baseline:
`build/combat-baseline-native-v1-20261028/verification.json`. One client, validation
off, 35 seconds: 829.64 FPS uncapped, 100 terrain meshes loaded / 19 retired,
maximum 81 roots. This is empty-world single-machine evidence.

Actual two-client launcher admission/command traffic and both interruption /
injected child-failure cleanup pass for all three modes:

- `build/combat-launcher-baseline-v1-20261028/verification.json`
- `build/combat-launcher-targeting-v1-20261028/verification.json`
- `build/combat-launcher-optin-v1-20261028/verification.json`

No owned child or temporary ticket remains after those checks. Preserved binaries
are in `build/combat-qualified-v2-20261028` (artifact iteration 2, protocol profile 1):

- Client SHA-256: `0fcaa6105d68e3fd1ccf64a9fc663fe4379b6a07bdfc67ae6d91d304579301c2`.
- Server SHA-256: `15aae36469a43d3ab1a6a8a43b05a8d394aaf6bade5b449a64ed50ba353e8f1b`.
- Catalog SHA-256: `c5c53ea53fc01df78e9be45a733731257bccc76f31853e12c5b09952b54d377c`.
- Existing private character SHA-256:
  `e0711a9603665cf25ec19e5b12539cc9a6f9db70a4658bdc87a74064fc0a0dfe`.

This qualifies one static, nonblocking NPC ability loop. Player health/death/PvP,
AI/navigation, persistence/rewards, production combat interest, cast animations,
spell VFX/audio, physical keyboard/OS automation, native Windows and MMO population
capacity remain open. No inventory/economy success or durable reward is implied.
The native fixture injects Bevy ButtonInput and captures only its own game window.
The full authority contract follows; dated earlier checkpoints remain historical.

## Authority contract

The initial headless slice binds the actual catalog bytes and creature health /
respawn configuration into a separate optional world profile. World owns casting,
range, terrain line of sight, cooldowns, health, death and respawn. No input may
supply health, damage, a position, a timestep or a caster identity. The admitted
session binds the caster. Requests identify a known catalog ability and target,
and carry checked world/monotonic sequence identity. Stale, foreign and excessive
sequence jumps must not mutate cast state or poison later valid requests.

Casts start at the current completed simulation tick; positive cast durations
use the existing ceiling conversion. Instant casts resolve on the next fixed
step, so incoming packet bursts cannot apply unlimited damage between steps.
Cooldown and GCD start with an accepted cast and are not refunded on interruption.
Moving while casting is the default. Restricted casts reject movement intent and
interrupt on actual movement; trusted teleports interrupt independently of input.
Both start and completion revalidate target life, range, actor readiness and
terrain line of sight. This is a graybox terrain policy, not production building
LOS, PvP, threat, latency compensation or a navigation solution.

One bounded queued spell may replace another within the catalog's queue window;
replacement must be explicit in returned outcomes. The queue cannot create an
unlimited backlog. Queued casts revalidate at execution. Off-GCD abilities bypass
GCD only; this initial slice still allows one active cast per actor.

A defeated target retires from selection/presentation, clears all selections and
respawns after a fixed simulation-time delay at its original validated position.
Respawn allocates a new monotonically increasing entity ID. Old commands cannot
hit the replacement. Trusted owner removal must cancel future respawn. NPCs and
players remain absent from world collision.

Acceptance: actual World tests at 20/30/60 Hz for mobile casts, exact deadlines,
queue/GCD/off-GCD behavior, stale/forged/unknown requests, range/LOS/readiness,
competing lethal hits, disconnect and fresh-ID respawn. Then add a bounded reliable
combat transport profile and verify two actual encrypted clients under impairment,
followed by native Bevy health/cast/cooldown UI and death/respawn. A headless pass
alone does not deliver P2-12. Full regression, denied-warning Clippy, formatting
and native GPU evidence remain required before launcher adoption.

## Focused checkpoint

- `build/combat-authority-tests-v4.log`: 30 library tests and eight actual-World
  combat tests pass, including 20/30/60 Hz deadlines, mobile/restricted movement,
  a real static wall at cast start/completion, stale/foreign requests, single-slot
  queue replacement/expiry, off-GCD behavior, competing lethal hits, fresh-ID
  respawn, disconnect and owner deletion of a pending respawn.
- `build/combat-wire-tests-v2.log`: four codec tests pass. Maximum state is
  **1,066 bytes**, measured by encoding all 64 cooldowns, 16 health records, cast,
  queue and reply. A 16-event batch is 701 bytes; requests are 29 bytes. Tests
  reject all truncations, trailing fields, unknown flags/ordinals, duplicate or
  unordered health/cooldowns, invalid life/deadlines and foreign cast identities.
  The initial size test corrected a draft 1,166-byte arithmetic estimate.
- `build/combat-live-tests-v8.log`: four real encrypted production-runtime tests
  pass; the saved/impaired report is `build/combat-live-v8/combat-wifi.json`.
  Two clients agree on mobile casting, a single lethal hit/death and fresh-ID
  respawn; stale-life commands reject and disconnect cancels an unfinished cast.
  Fresh sessions inherit no cast/cooldown, and targeting-only profile mismatch
  rejects at the welcome. Actual forged/stale/foreign/flooded controls fail closed.
- Actual bounded client presentation-queue exhaustion closes that client while a
  healthy observer receives the damage. A separate one-way downstream outage
  reaches the server's real reliable-channel budget and retires only that peer.
  The first outage fixture hit the existing five-second netcode timeout before
  filling the queue; its replacement exercises 64 real catalog cooldown entries
  and reaches the send-budget gate. Production timeouts were not changed.
- Denied-warning workspace/all-target Clippy passes in
  `build/combat-transport-clippy-v3.log`. The qualified 253-test targeting full
  regression predates combat; no post-combat combined result or native ability
  GPU pass is claimed yet. Earlier fixture/compiler failures remain in local logs.

CA01 requests use catalog ordinals bound by the profile digest, never client
health/damage/caster coordinates. CS01 carries checked bounded authoritative state.
CE01 is an ordered per-peer event sequence; events are broadcast to this bounded
development population, not a production combat-interest policy. Each peer allows
ten cast requests per one-second control period, within the existing eight
controls per pump and 180 total messages per second. This is a prototype action
budget, not a recommended final class/input rate.

Client action queues allow at most eight requests and presentation queues at most
16 event batches. Fixed-step events are consumed before the next step; immediate
request outcomes flush in the same I/O pump. Event batches contain at most 16
events, with at most 64 pending events per bounded server pump/step. Changed
combat state is sent reliably; if state/events cannot fit, the peer is retired
instead of coalescing through a death or silently dropping gameplay events. The
server remains headless and uses no file/database/awaited transport I/O in its
fixed step. The baseline wire remains version 12; profile identity changes opt-in
application admission independently of immutable terrain data.

## Client presentation in verification

The draft client/server application opt-in is `--combat-catalog <path>`, which
implies targeting and binds identical validated bytes before welcome admission.
Both applications cap the startup read at 64 KiB plus one rejection byte. The
normal launcher has not adopted combat yet. The development target has 100 health
and a five-second simulation-time respawn. The existing catalog provides 20 damage,
a one-second mobile cast, two-second cooldown, 1.5-second GCD and 300 ms queue.

A standard Bevy panel shows authoritative target health, cast/queue status and
cooldown. Digit 1 sends a bounded cast request for the selected target; Tab and
Backspace retain targeting controls. The UI computes countdown presentation from
received simulation ticks, never damage, health or cast success. Existing
locomotion/animation continues; there is no new cast animation, spell VFX or audio.
The bridge drains at most 16 ordered event batches per frame and counts any gaps.

Denied-warning workspace/all-target Clippy passes with the draft presentation
(`build/combat-client-clippy-v2.log`); full workspace verification is running in
`build/combat-workspace-v1.log`. Native ability visuals remain unqualified.
The opt-in native fixture uses a balanced eight-second walk/run/jump/idle circuit
near the NPC, starting at 8058,8000 (before per-actor spawn offsets) to exercise a real chunk/origin crossing.
It injects production Bevy Digit-1/Tab controls, observes authoritative damage and
fresh-ID respawn, and captures only the game window. Moving-cast telemetry counts
actual horizontal changes between authoritative snapshots during an active cast,
not intent or local predicted motion alone. Qualification is still pending.

## Native review follow-up

The intermediate full regression passes **271 tests** (ten private-fixture skips),
with all 53 client tests repeated after final connection-report metadata, plus
workspace/all-target denied-warning Clippy (`build/combat-workspace-v1.log`,
`build/combat-final-client-tests-v1.log`, `build/combat-final-clippy-v1.log`).

The first native run (`build/combat-native-wifi-v1-20261028`) completes 18 accepted
casts, 15 damage events, three defeats and two fresh-ID respawns on both clients,
with no rejection, event gap, input pressure, movement recovery or missed
simulation/I/O deadline. Its overall gate **fails** because the second player's
spawn offset starts beyond the tested origin boundary; only one rendered origin
revision is observed there. The corrected fixture starts at 8058,8000, placing
both initial player spawns before the boundary. The failed report/log is retained.

Screenshot review also identifies an actual authority ordering issue: an earlier
actor can retain an unfinished cast for one tick after a later actor kills its
target. The completed step now cancels every cast/queue on the retired target life
before publication. A new staggered-cast World test covers both actor orders at
20/30/60 Hz. Nine focused authority tests and denied-warning Clippy pass
(`build/combat-authority-tests-v5.log`, `build/combat-final-clippy-v2.log`). The
post-fix full regression is running in `build/combat-workspace-v2.log`; native
qualification must be repeated with newly built binaries before adoption. No
native combat pass is claimed from the failed first run.

The corrected authority now passes **272 full workspace tests**, ten explicit
private-fixture skips (`build/combat-workspace-v2.log`), and denied-warning
workspace/all-target Clippy (`build/combat-final-clippy-v2.log`). Optimized
binaries are rebuilding in `build/combat-release-v2.log` before the corrected
native fixture. The native check remains pending at this checkpoint.
