# Classic melee client adoption — qualified

P2-12 / P3-06 / P3-08 / P2-14 follows the qualified
[melee transport](CLASSIC_MELEE_TRANSPORT.md), committed equipment, persistent
targeting and classic controls. Changes are local and unpushed. The preceding
playable targeting checkpoint remains available as a historical checkpoint. The current source/art/binaries/proof are frozen in
`build/classic-melee-client-qualified-v1-20261101`.

## Implemented paths

The new `development-gameplay-classic-melee.json` bundle uses manifest schema 5
and fourteen exact-byte roles, including `melee_weapons`. It requires life,
equipment and combat roles, excludes dodge, and selects classic controls in the
launcher. Earlier schemas and explicit legacy launches remain available.
The Rust validator cross-checks weapon/item definitions and strict role/version,
path, alias and byte budgets. Python snapshots the validated role bytes for both
children, preventing later authoring edits from splitting their configurations.

Server/client `--melee-weapons` loads bounded immutable weapon data. The server
requires classic account-backed combat; the client requires classic predicted
combat and equipment. Exact melee and item bytes participate in gameplay identity
after player life and before rewards/progression/quest/dialogue/vendor and the
outer classic profile. Authentication, persistence and simulation authority stay
on their existing paths; the client does not decide equipment ownership or hits.

In the explicit profile, a hostile right-click selects and sends persistent melee
intent rather than requesting Sword Slash. Left-click and Tab only select.
Camera drags and UI-owned clicks retain their input gates. Changing/clearing
selection ends the old engagement through authority. Turning, range and LOS
failures pause it without clearing selection. Existing clear controls also stop
engagement. A stop-intent API exists independently of selection.

Authoritative basic damage drives sword upper-body animation, wolf hit reaction,
floating white applied-damage numbers and combat counters. Animation uses the
existing standard Bevy graph/masks, locomotion and authored closed hand grip.
The training sword has an explicit development presentation mapping; other weapon
animations need their own reviewed mappings. The first basic animation currently
starts on receipt of its authoritative damage cue; predictive windup/impact
alignment is future presentation work.

Ability and basic-swing sequence namespaces are independent. Presentation keeps
separate source counters and a bounded local playback revision, so equal request
and swing numbers cannot suppress one another. An old ability cancellation cannot
cancel a newer basic cue. The HUD displays engagement or an explicit reason such
as facing, range, LOS, casting, readiness or equipment. Cosmetic labels retain the
existing active/pending bounds; reliable gameplay events are consumed first.

`development-classic-abilities.json` preserves the current projectile fixture and
makes Sword Slash instant, separate from basic swings. Values remain Theia
fixtures. Abilities still require their own facing/minimum-range and swing-timer
interaction data before the complete strict specification can pass. The current
World pauses basic eligibility during active casts while its clock continues;
GCD alone does not pause it. No dodge or automatic facing is introduced.

## Acceptance

Formatting, denied-warning workspace Clippy, release builds, **153 core tests,
466 network passes / ten explicit skips / 69 groups, 114 client tests and 37
Python checks** pass. The network total includes twenty validator tests. New
checks cover schema/weapon identity and reference failures, exact-byte launcher
adoption and independent animation sequences.

Six final normal OIDC/PostgreSQL native starts pass: three isolated-control
fixtures in `build/classic-melee-client-native-v4` and three unmodified
development-profile fights in `build/classic-melee-client-native-v7`. Both cover
1920×1200, 3440×1440 fullscreen and 1280×720 with the A/D-strafe preference file.
Every run has zero GPU diagnostics on Linux, AMD Radeon RX 7900 XTX / RADV NAVI31,
Vulkan, Mesa 26.2.6 kisak. Supported-size captures were visually inspected.

The control fixture disables wolf perception, slows patrol and reduces
retaliation damage to isolate facing/range pause/resume and explicit clear.
The unmodified profile separately proves passive acquisition, chase and attacks
before player damage, committed sword use, timed melee feedback, separate instant
Sword Slash and wolf death without player death. Graceful server summaries show
one acquisition, 21 chase ticks and over 500 attack-state ticks in each unmodified
run. These are development-profile tests, not production balance evidence.
Read-only telemetry observes results; all actions use synthetic X11 input to
owned PID-matched windows. Physical hardware and Windows execution are untested.

Earlier failed harness attempts clicked a wolf region obscured by the player,
sampled chase origin after arrival, or read the launcher log instead of the
separate server log. Final runs click visible model regions and use owned
server pursuit summaries. No camera workaround or gameplay telemetry input was
added. Source and final proof are retained in the qualified checkpoint.

The strict milestone now has **eleven of thirteen** demonstrated checks.
Ability-specific facing/minimum range and timer interaction, plus normal
cast-time ability acceptance, remain open. The complete strict specification is
not declared compliant. Next: [ability rules](CLASSIC_ABILITY_RULES.md), then
[damage resolution](CLASSIC_DAMAGE_RESOLUTION.md). No MMO capacity or Windows
claim is made; all changes remain local and unpushed.
