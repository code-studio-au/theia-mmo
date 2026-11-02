Latest playable classic follow-up: [persistent targeting](CLASSIC_TARGETING.md)
adds forward/reverse Tab and selection retention, with its own qualified v2
profile checkpoint. The controls evidence below retains its original scope.

# Classic controls and manual facing — qualified local-player slice, 1 November 2026

P2-04 / P2-05 / P2-07 / P2-09 / P2-11. The normal executables and launcher now
accept `--classic`, binding the headless facing profile below to actual player
input. This is the current classic development configuration; earlier default
profiles remain for comparisons until the full replacement combat milestone
passes. It is not yet the complete [strict combat specification](COMBAT_TARGETING_MOVEMENT_SPEC.md).

W/S move relative to character facing; A/D turn, Q/E strafe, Space jumps.
Independent LMB orbit preserves heading. RMB world dragging steers even while
stationary; both buttons move forward and steer. During RMB steering, A/D strafe.
A short selection click does not turn toward the selected actor. Keyboard turning
follows the camera unless LMB is held. World/UI click ownership remains separate.
Panels close before Escape clears a target; Escape does not quit classic play.
E no longer opens guide dialogue. F interaction and contextual RMB remain.
F8 action camera, its reticle controls and dodge are disabled in this profile.
Classic and dodge profiles cannot be combined; accelerated tours, unpredicted
movement and Quinn cannot select classic controls.

Movement bindings and keyboard turn rate load from a bounded, strict JSON via
client/launcher `--controls-file`; defaults are `content/input/classic-controls.json`.
A/D strafe is supported by binding turn to arrows or leaving turn keys unbound.
Conflicting movement bindings and current F/I interaction/backpack conflicts
are rejected. The first settings file covers movement, not a complete binding
editor/action-bar remap. Keyboard turning defaults to 180 degrees/second and
retains a continuous frame-rate-independent accumulator before quantization.
Large frame deltas cap at 100 ms. Current travel retains development 5 m/s and
Shift-modified 8 m/s speeds; backward/sideways tuning, walk/run conventions and
turn/camera sensitivity still need combat-feel calibration, not a WoW parity claim.

Local heading responds at render cadence; source/actor/stream changes reseed it
from authoritative correction. Local position remains server-owned/predicted.
Remote snapshots supply independent yaw, with shortest-arc interpolation rather
than travel-derived facing. That interpolation passes a wrap-boundary unit check;
rendered remote independent turning is not yet separately demonstrated. The
headless real-UDP proof below carries authoritative yaw through actual snapshots.

## Acceptance and evidence

**111 client tests**, **45 Python checks**, formatting and workspace/all-target
denied-warning Clippy pass. The prior full **142 core / 444 network passes /
10 explicit network skips / 68 groups** apply to byte-identical core/network
sources, including real impaired encrypted UDP at 20/30/60 Hz. No new whole-
workspace test or Windows execution claim is made.

Three normal OIDC/PostgreSQL launcher starts receive X11/XTest synthetic OS input
at 1920×1200, 3440×1440 fullscreen and 1280×720. They prove stationary keyboard
turning, independent LMB orbit, stationary RMB steering, forward/back/strafe with
preserved heading, both-button forward movement, guide selection/dialogue,
UI mouse ownership, panel-before-target Escape and disabled action camera/dodge.
The 720p run loads a real alternate settings file with A/D strafe and arrow turn.
The copied fixture disables wolf perception to isolate controls; telemetry only
observes state and does not submit gameplay commands. All captures verify as PNG.
The first desktop attempt was interrupted by concurrent mouse use; the user
confirmed that interference. The complete v2 retry supplies acceptance evidence.

Two-client melee and projectile regressions add four native starts with actual
server damage in the retained non-classic combat configuration. All **seven
final native starts have zero GPU diagnostics** on Linux RX 7900 XTX / RADV
NAVI31, Vulkan, Mesa 26.2.6 kisak. This preserves prior combat behavior; it does
not prove facing-gated classic autoattack. No physical hardware input, new
sustained FPS, MMO population or streaming-capacity claim is made.

Proof: `build/classic-client-build-*-v2.log`,
`build/classic-client-build-results-v2.json`, `build/classic-client-python-v1.log`,
`build/classic-controls-native-v2/verification.json`,
`build/classic-{melee,projectile}-native-v1/verification.json`.
Exact source/binaries/art/proof: `build/classic-controls-qualified-v1-20261101`.
Binaries and Rust source are identical between staging v1/v2; v2 retains the
final Python test/helper edits. Earlier qualified experiments remain available.

With the normal documented private account/provider/database environment:

```sh
build/blender-tools/bin/python tools/play.py --account-login --classic \
  --binaries build/classic-controls-qualified-v1-20261101 \
  --gameplay-manifest content/gameplay/development-gameplay-projectiles.json \
  --players build/classic-controls-qualified-v1-20261101/art/player \
  --creature-asset build/classic-controls-qualified-v1-20261101/art/wolf/wolf.glb
```

Next: persistent explicit selection and forward/reverse character-facing Tab,
then separate weapon-timed melee autoattack, data-driven combat arcs and the
instant/cast-time ability pair. Full action-bar bindings, directional locomotion
animation and movement tuning remain. The newly supplied
[enemy AI reference](ENEMY_AI_REFERENCE.md) informs later threat/evade slices;
its illustrative balance values are not locked. Dodge stays deferred.
Changes remain local, uncommitted and unpushed.

---

# Headless classic facing prerequisite — qualified, 1 November 2026

P2-04 / P2-05 / P2-07 / P2-11 / P3-06; prerequisites are the existing
bounded fixed-tick movement commands, prepared Avian terrain, prediction driver
and encrypted Renet admission. Character yaw is an independent canonical u16
revolution: zero faces -Z, positive yaw turns left toward -X, Y-up metres.
Checked conversion rejects nonfinite yaw. Character-relative axes transform
without turning the character; diagonals normalize before quantization.
A shared parameterized horizontal arc uses f64 global differences and never
reads camera or target selection state. No numeric attack arc is enabled in
combat by this prerequisite; the 180-degree math fixture is not a parity claim.

Opt-in `World::enable_classic()` requires inactive normal-speed predicted terrain
and excludes dodge/tour/teleport-tour mode. Facing travels in the existing
bounded command timeline, independently of movement. Input expiry stops travel
and keeps the last accepted heading. Refused duplicate/conflicting/stale commands
cannot turn later. Owner corrections and player snapshots carry independent yaw.
Legacy movement profiles refuse facing metadata instead of silently stripping it.
TC03/TP03 envelopes are bounded to 121/133 bytes and validate canonical optional
held input, exact lengths and existing epoch/sequence/tick/readiness contracts.
Classic admission binds the complete source-world identity to its fixed profile.
The shared history/driver replays heading with movement and rejects profile loss.

Four new core checks and eight new network checks pass. Actual World/prepared
Avian comparisons cover stationary turning, strafe/backward travel, expiry,
deduplication, correction replay and the eight-player datagram budget. Facing
and physics motion state match exactly; position comparisons permit 0.1 mm for
the existing f32 centre/feet conversion (an initial strict-equality test exposed
about 0.00000036 m of Y difference). The actual shared driver also passes over
secure Renet UDP with configured loss, jitter and duplication at 20/30/60 Hz.
This is real headless transport, not a rendered multiplayer qualification.

Full regression: **142 core passes**, **444 network passes / 10 explicit skips /
68 groups**, **106 client passes**, formatting and workspace/all-target
denied-warning Clippy. Exact source/proof: `build/classic-facing-qualified-v1-20261101`.
Logs/results: `build/classic-facing-{fmt,clippy,core,network,client}-v2.log` and
`build/classic-facing-results-v2.json`. Earlier failed equality/compile/Clippy
attempts remain available. No GPU, native Windows or new FPS/capacity claim.

## Remaining adoption at the headless checkpoint

The normal executables/launcher do not yet select this profile. Existing client
controls remain camera-relative; they do not yet consume authoritative facing.
Next: classic key bindings and mouse controls, normal profile flags/admission,
local/remote heading presentation, UI/focus ownership and native desktop checks.
Then persistent selection, weapon-timed autoattack and ability-facing validation.
The thirteen [strict specification](COMBAT_TARGETING_MOVEMENT_SPEC.md) milestone
checks remain open. Dodge remains deferred. Changes are local and unpushed.
