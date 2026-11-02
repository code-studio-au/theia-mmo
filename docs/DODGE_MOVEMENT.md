# Deferred experiment — superseded initial scope, 1 November 2026

The user adopted [THEIA-COMBAT-001](COMBAT_TARGETING_MOVEMENT_SPEC.md): strict
WoW Classic-style controls and manual-facing combat. All active dodge work is
deferred, including the movement-only implementation and local animation
candidate below. Preserve these experiments and proof; do not resume their
implementation/qualification or include dodge in initial-milestone acceptance
without a later explicit decision. Existing code remains; this is a scope change.

# Authored local dodge animation — paused at user request, 2026-11-01

P2-09 / P3-14 local candidate adds four directional, one-shot dodge clips,
retimed to the existing 250 ms authority policy, with closed sword fingers and
preserved body/wrist/prop animation. Six new presentation tests cover direction,
correction replay, refused requests and changed input streams. The release build,
106 client tests, formatting and workspace/all-target denied-warning Clippy pass.
43 Python checks passed before the final helper-only direction/diagnostic edits.
Network/core sources are unchanged from the 436-pass qualified dodge checkpoint.

Native work is **incomplete**. The v2 desktop run completed animation/control
checks at 1920×1200 and 3440×1440, then its owned-window pointer guard stopped
at 1280×720 before that size's dodge checks. The first size's independent live
bone comparison passes all four clips with exactly closed finger samples; the
other size's independent grip verification and new two-client combat regressions
remain pending. An explicit extra backward case fixed a missing direction in
the first test attempt. The v3 retry was interrupted safely during private
identity startup at the user's stop request; no game window was opened by it.
All owned test processes are stopped. No new fully qualified playable checkpoint
is declared; remote-player dodge animation also remains unimplemented.

Candidate source/binaries/private art: `build/dodge-animation-candidate-v3-20261101`.
Build: `build/dodge-animation-build-results-v1.json`; final Rust/tool checks:
`build/dodge-animation-final-results-v1.json`; partial native logs/telemetry:
`build/dodge-animation-os-native-v2`; completed first-size grip proof:
`build/dodge-animation-os-native-v2/1920x1200/grip-dodge-verification.json`.
The retained qualified playable checkpoint remains
`build/dodge-qualified-v1-20261101`. Changes stay local, uncommitted and unpushed.

Authored dodge/roll conversion and native-inspector checks now pass as a
[separate presentation prerequisite](CHARACTER_ASSET_INTAKE.md). The playable
checkpoint below retains its existing locomotion; no authored gameplay dodge
animation is installed yet.

# Normal client adoption — qualified, 2026-11-01

P2-04 / P2-05 / P2-07 / P3-06 / initial P3-14 builds on the qualified driver
below. Both executables accept `--dodge-profile`; the launcher seals its exact
bytes as the fourteenth role in schema 4. The new opt-in bundle is
`content/gameplay/development-gameplay-dodge.json`; earlier bundles retain their
behavior. Normal V input uses current world travel, or camera-view backward when
stationary. Its shared 200 ms edge is gated by focus, panels, fresh authority,
current life and the intersection of actual prepared/loaded/acknowledged terrain.
Both camera modes use the same authority checks. The existing HUD shows ready,
active and remaining cooldown state. Current locomotion clips remain; no authored
roll animation, invulnerability or endurance system is claimed.

The application candidate passes 100 client tests; the gameplay validator passes
18 tests including schema/required-role, exact-byte, path, alias and size checks.
Full regressions pass **436 network tests / 10 explicit skips / 66 groups**,
**43 Python checks**, formatting and workspace/all-target denied-warning Clippy.
All **seven final native starts** have zero GPU diagnostics: three actual
OIDC/PostgreSQL launcher/OS-input starts at 1280×720, 1920×1200 and 3440×1440
fullscreen; two-player native melee and projectile regressions (two starts each).
Each OS pass measures 3 m stationary dodge in classic and action camera modes,
proves panel blocking, a held key beyond cooldown without another action, and
one moving dodge (3.667 m including ordinary walking around it). The copied
native fixture disables wolf perception to isolate controls; it does not inject
gameplay state or requests. Existing public damage checks separately retain the
server's actual melee/projectile behavior with the default non-dodge profile.

Linux RX 7900 XTX / RADV NAVI31, Vulkan, Mesa 26.2.6 kisak. HUD screenshots at
all three sizes were inspected. Native input is X11/XTest synthetic OS input;
no physical hardware or Windows execution claim. No new sustained FPS or
population/streaming capacity measurement is claimed. Full-network v3 printed
all passing results but its parent was interrupted before saving the exit result;
the detached v4 run supplies completed exit evidence and the Python run.
OS capture v1 created an empty ultrawide final PNG because shutdown raced image
writing. The helper now waits for full PNG verification; the final v3 desktop
pass and every copied screenshot are checked for complete image data. Rust
binary hashes remain identical across the helper-only correction.
The v2 desktop pass was interrupted by concurrent user mouse input while opening
the backpack; it is retained as an incomplete attempt.

Proof: `build/dodge-native-build-*-v2.log`,
`build/dodge-app-{fmt,clippy}-v3.log`, `build/dodge-app-{network,python}-v4.log`,
`build/dodge-app-results-v4.json`, `build/dodge-os-native-v3/verification.json`,
`build/dodge-{melee,projectile}-native-v1/verification.json` and matching
screenshots/telemetry. Frozen checkpoint: `build/dodge-qualified-v1-20261101`.

Launch with the normal documented account environment and asset prerequisites:

```sh
build/blender-tools/bin/python tools/play.py --account-login \
  --binaries build/dodge-qualified-v1-20261101 \
  --gameplay-manifest content/gameplay/development-gameplay-dodge.json \
  --players build/dodge-qualified-v1-20261101/art/player \
  --creature-asset build/dodge-qualified-v1-20261101/art/wolf/wolf.glb
```

V is a movement-only dodge with a 3 m / 250 ms / 3 s development policy.
It grants no invulnerability or stamina/endurance system. Authoritative casting,
collision and range rules still apply. Locomotion clips currently show the motion;
authored directional dodge/roll clips and independently replicated facing remain
next presentation work. The earlier playable checkpoints remain available.

# Current dodge slice — shared driver and expiring input, 2026-11-01

`Driver::with_dodge` checks source-world/profile identity, normal speed and timed
state before prediction, then uses completed-tick callbacks for normal steps,
bootstrap and correction replay. Legacy drivers refuse dodge metadata; a bound
driver refuses metadata removal. Existing life holds and prepared collision
suspension retain their command-debt behavior.

`dodge_input::Input` owns one pending edge for at most 200 ms, stops on authority
nonce acknowledgement, and never queues an airborne press for landing. Focus,
panel/readiness suspension, disconnect and actor/stream changes discard pending
input. A simultaneous edge on changed context is discarded. Nonces are checked,
monotonic and seeded from authority; exhaustion fails without wrapping. Cooldown
and acceptance remain server-owned. The caller must supply the focus/life/
prepared-permission gate; normal Bevy input has not adopted this helper yet.

Four input tests and three new driver tests pass, including actual authority
convergence from one tap at 120 simulated render Hz and 20/30/60 Hz authority.
A new real encrypted impaired-UDP check combines the driver, expiring tap,
TC02/TP02 and World authority at every rate. Formatting and workspace/all-target
Clippy pass. **41 targeted passes / six groups** include legacy driver, shared
replay, codec and actual adverse-UDP regression checks; logs:
`build/dodge-driver-{network,clippy,fmt}-v2.log`, results JSON of the same prefix.
Exact source/proof: `build/dodge-driver-qualified-v1-20261101`.
The prior full-network qualification is the separate 427-pass authority revision.
No normal-client, native dodge, new FPS/capacity or Windows claim is made.

Cinnamon's current `mouse-button-modifier` is `<Alt>`. The next normal-input
slice will use V for dodge to avoid conflicting with Alt mouse window controls;
the earlier Left Alt plan below is historical. Rebinding remains a later UI task.

# Current dodge slice — authority and encrypted transport, 2026-11-01

P2-04 / P2-07 / P3-06; prerequisites are the checked policy, Avian controller,
restorable snapshots, versioned codec and tick-aware shared replay qualified below.
The current library revision installs opt-in normal-speed predicted World authority
and explicitly selects TC02/TP02 over encrypted Renet when enabled. Normal gameplay
bundles/client controls do not enable it yet. The playable camera checkpoint is
`build/camera-panels-qualified-v1-20261101`.

Authority consumes the one-shot nonce, replaces walking with one swept proposal,
and ticks policy even without a prepared patch. Missing readiness, death,
teleport and solver recovery cancel active motion without refunding cooldown or
nonce; life/teleport stream resets discard queued input. Stationary casts count
active dodge as movement even on its zero-displacement accepted start tick.
No invulnerability is granted. Configuration is rejected after admission, at
accelerated development speed, or without predicted terrain. World identity wraps
the exact dodge profile after all other gameplay identities.

The transport binds the client's source-world/profile identity before connecting,
retains existing actor/world/stream/rate/readiness checks and bounded queues, and
refuses legacy correction decoding for a profile-2 session (and conversely).
Six new authority tests cover every-tick real-Avian agreement with shared
prediction at 20/30/60 Hz, readiness loss, missing collision, teleport and legacy
refusal. Two life/cast tests prove death during active motion, no immunity,
respawn nonce/cooldown retention and stationary-cast cancellation before movement.
Three transport tests include actual encrypted impaired UDP at all rates,
repeated consumed edges beyond cooldown, invalid policy binding and legacy-client
refusal. Full network regression passes **427 tests / 10 explicit skips / 65 groups**,
formatting and workspace/all-target denied-warning Clippy. Proof:
`build/dodge-authority-{network,clippy,fmt}-v2.log` and
`build/dodge-authority-results-v2.json`. The initial stationary-cast test used a
mobile ability fixture; setting that test-only long cast to stationary made the
check exercise the intended rule. No production casting rule was relaxed.
Frozen headless source/proof: `build/dodge-authority-qualified-v1-20261101`.

Remaining gates: select this binding in the live prediction driver; bounded
client edge lifetime and reset handling; gameplay-bundle/launcher and normal
client/server adoption; visible standard Bevy feedback and native input checks.
No playable dodge, new native/FPS/capacity or Windows qualification is claimed.
Changes remain local. Earlier dated prerequisite records follow.

# Dodge movement — policy prerequisite, 2026-11-01

P2-04 / P2-05 / P2-07 / P3-06 / P3-14 refines the user's requested active
evasive movement. The adopted default binding is Left Alt. **Dodge is not yet
installed in the server, prediction protocol or client.** The current playable
build remains `build/world-input-qualified-v1-20261101`.

## Checked policy

`theia_core::dodge` and the unused opt-in `development-dodge.json` define an
initial **3 m / 250 ms / 3 s cooldown** movement proposal. These are development
values. A strict 4 KiB schema-1 profile checks its ID, finite 0.5–6 m distance,
100–750 ms duration, duration-to-10 s cooldown and maximum 20 m/s speed.
Direction retains checked nonzero signed input axes within ±1000 and normalizes
before producing displacement. Diagonal requests cannot increase distance.

An authority-supplied alive/grounded/prepared gate controls initial acceptance.
The caller still must validate the exact current actor/life and deduplicate
commands. Acceptance consumes the fixed-tick cooldown and captures profile/rate;
it returns no movement on that tick. Following consecutive ticks propose at most
one XZ displacement each, clipped to exactly 3 m even at 30 Hz, where 250 ms ends
between ticks. Duplicate/old advance calls cannot move again. A skipped tick
retires motion rather than creating a catch-up leap. Invalid starts and tick
overflow leave state unchanged. Cancellation is idempotent and keeps cooldown.

This is a bounded copyable policy state, not a physics body, transport, authority
admission check or invulnerability system. No health/resource setter or additional
movement is hidden in it. The normal capsule controller must sweep each proposal;
collision may shorten it, without saving blocked distance for a later burst.
No invulnerability frames or combat immunity are adopted.

All **130 core tests in 20 groups**, including eight new dodge checks, pass.
Formatting and denied-warning workspace/all-target Clippy pass. Exact source and
proof: `build/dodge-policy-qualified-v1-20261101`. Logs:
`build/dodge-policy-test-v2.log`, `build/dodge-policy-clippy-v2.log`.
No new whole-workspace, native dodge or movement-replication result is claimed.
The preserved mouse-control build separately has 91 client tests and 14 final
native starts with zero GPU diagnostics.

## Shared capsule controller prerequisite

`kinematic::advance_dodge` now feeds a checked accepted proposal through the
existing Avian solver, replacing ordinary horizontal movement and applying
gravity once. The accepted start tick holds horizontal motion. Jump remains
latched while new jumping is suppressed. Walking/development travel still use
the same solver through their retained entry points. The helper is not selected
by authority, prediction or client input yet.

Eight new actual-Avian checks pass at 20/30/60 Hz: exact free-space distance,
thin walls, steps/headroom, walkable/steep slopes, actual missing-neighbour chunk
permission, held-jump release/repress, start-tick hold and gravity/invalid-step
atomicity. Every movement endpoint passes the existing penetration/support
validator. A low step contacts the rounded capsule sole near x=0.661 m; the first
approximate x<0.66 assertion was corrected to x<0.67 while retaining complete
per-step Avian validation. No collision or headroom rule was relaxed.

The complete network all-target suite passes **394 tests / 10 explicit fixture
skips in 61 groups**, including existing real encrypted impaired-UDP movement
at all supported tick rates. Formatting and denied-warning workspace/all-target
Clippy pass. Exact source/proof: `build/dodge-controller-qualified-v1-20261101`.
Logs: `build/dodge-controller-{network,clippy,fmt}-v1.log`.
This is headless shared-controller qualification, not live multiplayer dodge or
new rendering/performance evidence. The prior runnable controls build remains.

## Restorable policy state prerequisite

Checked snapshots now carry the completed tick, cooldown deadline and optional
start tick/raw direction axes. Restoration derives elapsed/requested distance
using the same arithmetic as ordinary advancement; it cannot restart a partial
motion or refund cooldown. Finished motions cannot be marked active, impossible
future cooldowns/ticks/directions fail, and unobserved default state differs from
an enabled idle state anchored to a completed tick. State remains fixed-size and
copyable. No position or immunity fields are accepted.

The caller must supply the exact immutable profile/rate and bind the world,
owner/life and outer correction tick. These snapshots are not a network authority
envelope; in particular they do not independently identify the profile. That
binding belongs to the upcoming versioned transport/profile layer.

Eight additional snapshot tests pass: every active/terminal/cooldown tick at all
rates resumes identically, partial 30 Hz distance and cancellation retain their
budgets, malformed fields fail, and near-u64 clock exhaustion remains bounded.
All **138 core tests / 21 groups**, the nine filtered kinematic checks (eight new
controller cases plus the existing upward-motion case), formatting and denied-
warning workspace/all-target Clippy pass. Exact source/proof:
`build/dodge-snapshot-qualified-v1-20261101`. The prior 394-pass network suite is
retained rather than rerun for these unused core snapshot additions. Live dodge,
movement-codec, authority/prediction and client adoption remain open.

## One-shot and wire prerequisites — qualified 2026-11-01

The shared controller now consumes monotonic action nonces, including refused
edges. Holding/retransmitting an edge cannot start another dodge after landing
or cooldown. Duplicate ticks perform no solver step; death/unprepared collision
retires active motion without refund. Immutable profile/rate changes and invalid
requests fail atomically. Snapshots retain the last consumed nonce.

Explicit TC02 commands are at most 165 bytes; TP02 corrections are exactly
170 bytes. They carry bounded nonces/directions and restorable authoritative
policy fields, without client-supplied displacement or cooldown truth. Decode
requires exact lengths, canonical absent/active fields and checked policy
restoration at the outer tick/rate. Profile/world/owner/life admission remains the
transport caller's responsibility. TC01/TP01 preserve their byte representation
and reject metadata they cannot carry. Existing live World/legacy input paths
reject dodge edges until opt-in authority integration is complete. The generic
prediction error sample also distinguishes dodge-state disagreement.

Eight controller and five codec checks pass. The complete network all-target
suite records **407 passes / 10 explicit fixture skips / 63 groups**, including
legacy real encrypted adverse-network movement. Workspace/all-target denied-
warning Clippy and formatting pass before the subsequent camera work. Logs:
`build/dodge-wire-tests-v1.log`, `build/dodge-wire-network-v1.log`,
`build/dodge-wire-clippy-v2.log`. This is a headless prerequisite; no session
negotiates TC02/TP02 or selects live dodge yet.

## Shared prepared-terrain replay — qualified 2026-11-01

A fixed immutable policy binding now checks the outer world/profile identity,
rate and authoritative initial snapshot before solving. The adapter restores
policy/body state, consumes the same one-shot nonce and applies one existing
Avian walk/dodge/gravity step against prepared collision and footprint readiness.
Absent prepared collision suspends speculation. No store reads or awaited I/O
occur in a step. Prepared patch content, actor/life, movement streams and readiness
epochs still belong to the existing runtime/transport owners.

History now offers tick-aware predict/reconcile callbacks while preserving the
legacy APIs. Solvers receive the original completed tick during speculation and
replay, including held/expired inputs. A solver cannot return a timed snapshot
from another tick or erase it; a correction cannot remove timed state while
keeping the same world identity. Same-tick metrics distinguish dodge-state error.

Nine new tests use actual Avian procedural/saved terrain: a lost first edge and
three-tick delayed corrections at 20/30/60 Hz converge without another start or
cooldown reset; duplicate/invalid ticks and changed bindings fail; death/unready
cancellation retains cooldown; missing neighbours clip the capsule without debt;
two prepared local frames agree within 0.1 mm across a chunk seam; saved-terrain
step counts cause no additional tile reads; missing collision suspends history.
These use simulated delivery, not live transport carrying dodge commands.

The full network all-target suite passes **416 tests / 10 explicit fixture skips
/ 64 groups**. Formatting and denied-warning workspace/all-target Clippy pass;
**99 client tests** pass with the separate panel/HUD source refinement.
Logs: `build/dodge-replay-network-v1.log`, `build/dodge-replay-tests-v2.log`,
`build/dodge-replay-clippy-v2.log`, `build/camera-panel-build-*-v1.log`.
The preserved playable camera checkpoint remains unchanged. World authority,
Renet profile-2 selection, the live prediction driver, input expiry, client dodge
presentation and native qualification are still required. No live dodge is
claimed. The panel/HUD refinement awaits coordinated desktop qualification.

## Next gates

1. Apply proposed displacement through the existing shared Avian capsule solver
   once per tick, replacing ordinary horizontal movement rather than adding to
   walking/sprint. Keep gravity, walls, headroom, slopes, steps and unavailable
   chunk boundaries. A grounded-start dodge suppresses new jumping while keeping
   the held-jump latch, so holding Space cannot manufacture a jump at completion.
2. Define checked restorable cooldown/active state and versioned prediction
   commands/corrections. Bound input edges/retransmission and preserve actor/life/
   movement-stream identity, queue limits, stale-replay refusal and 20/30/60 Hz.
   Never accept client position, elapsed time, cooldown or collision truth.
3. Install authority and shared prediction, counting a dodge as movement for
   existing cast-while-moving rules even when ordinary WASD axes are zero.
   Death, interruption and unavailable movement retire active motion without
   refunding cooldown; fresh admission remains consistent with transient state.
4. Add the normal Left Alt input and standard Bevy presentation. No authored roll
   animation has been adopted yet. Actual encrypted impaired-UDP and native OS
   input must prove displacement/collision/cooldown, life/rejoin, no extra steps,
   origin/chunk edges and melee range behavior before a playable dodge claim.

The gameplay bundle does not reference this profile yet. Earlier movement,
melee, projectile and input experiments remain available. Native Windows and
population/streaming capacity remain open. All changes are local.
