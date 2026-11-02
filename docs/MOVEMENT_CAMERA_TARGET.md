# Current target — strict WoW Classic controls, 1 November 2026

The governing design is [THEIA-COMBAT-001 v1.0](COMBAT_TARGETING_MOVEMENT_SPEC.md).
It supersedes the hybrid proposal retained below. Dodge, action camera, reticle
selection and facing assistance are deferred. No further dodge implementation or
qualification is scheduled for the initial playable milestone.

[The independent-facing headless prerequisite](CLASSIC_MOVEMENT.md) is now
qualified through actual World/Avian and impaired secure UDP. Normal classic input/local presentation now pass native checks; persistent
selection now passes [native qualification](CLASSIC_TARGETING.md); weapon-timed
combat and combat-facing adoption remain open.
The table below describes the remaining normal-player experience.

## Repository alignment and gaps

| Area | Existing implementation / evidence | Required migration |
| --- | --- | --- |
| Movement | Qualified character-relative W/S, A/D turn and Q/E strafe in `--classic`; configurable A/D strafe | Directional locomotion animation and movement-speed/feel calibration |
| Camera | Qualified independent LMB orbit, stationary RMB steering, both-button forward, static collision/zoom and UI ownership in `--classic` | Broader feel tuning; action camera remains disabled in classic |
| Facing | Independent authoritative heading, correction replay, render-rate local input and interpolated remote yaw | Native independent remote-turn evidence and ability/autoattack-facing validation |
| Selection | Qualified persistent hostile/friendly selection, character-front-biased Tab and Shift+Tab; explicit clear/removal | Immediate local pending-selection feedback and general dynamic friendly-removal observations |
| Right-click hostile | Qualified explicit-profile persistent weapon-timed melee intent; native range/facing pause/resume and feedback | Ability-specific facing/minimum-range and swing interaction; older profiles retain the ability experiment |
| Abilities | Authoritative equipment/range/LOS/cost/cast/cooldown checks, sword melee and travelling magic | Ability-data facing arcs, minimum range and swing interaction; audit movement/interrupt rules; prove one instant and one cast-time ability |
| NPC/world interaction | Guide dialogue, quest/vendor UI and server range checks | Preserve contextual F/RMB with E released for strafe; general objects/loot need their own applicable interactions |
| Input/UI | Embedded basic-attack selection, five initial hotkey slots, panel/click ownership | Complete binding settings and primary 1–= slots; contextual Escape is qualified |
| Enemy/server | Authoritative wolf pursuit, attack/death, server damage and persistent equipment | Reuse these foundations; prove the new one-player/one-hostile loop with manual facing and weapon timing |
| Deferred experiments | Opt-in dodge profile and local animation candidate; optional action-camera implementation | Preserve code/assets/evidence as experiments; do not present their controls as the new initial contract |

This is a documentation/design alignment, not a runtime migration. Current `--classic` source implements qualified manual controls; earlier
profiles retain their controls for comparisons. Earlier native checks qualify their exact
historical configurations, not the strict milestone. The retained
`build/dodge-qualified-v1-20261101` build is an experimental checkpoint; the local
authored animation candidate remains incomplete. The classic controls checkpoint is qualified for its stated slice; no build is
yet declared compliant with the complete strict combat specification. Engine, transport, terrain, persistence and coordinate
choices are unchanged.

## Next slices and concrete acceptance

1. **Combat-loop contract — P2-11 / P2-12 / P3-06.** Define initial weapon swing
   speed, attack event validation timing, a calibrated frontal arc, instant/cast
   ability timing, GCD/swing interactions, resource generation/regeneration and
   minimal threat behavior. These are explicit data/rules to decide before class
   expansion; do not infer numeric balance from the old sword ability cooldown.
2. **Classic input and independent facing — P2-04 / P2-05 / P2-07 / P2-09.**
   Prerequisites: existing movement authority/prediction and input/UI ownership.
   Implement configurable bindings and checked, replicated character orientation.
   Prove W/S/Q/E preserve facing, A/D turn in place, LMB orbit preserves facing,
   RMB steers in place/motion, both buttons move forward, and camera movement
   preserves target selection. Preserve collision and origin conversion. Test
   packet loss/replay and 20/30/60 Hz without trusting cosmetic transforms.
3. **Persistent explicit selection — P2-11 / P3-06.** Prerequisite: character
   facing for predictable Tab ordering. Prove click/Tab/Shift+Tab behavior and
   retained selection behind the character, outside attack range and behind an
   obstacle; explicit clear and entity invalidation still work. Resolve E's old
   interaction alias and Escape priority without replaying UI clicks.
4. **Authoritative melee autoattack — P2-12 / P3-06.** Prerequisites: weapon
   timing rules, persistent selection and authoritative facing. Separate
   engagement from ability requests. Prove one RMB starts repeated timed swings;
   range/facing/LOS failures pause without target loss; returning resumes;
   explicit cancel, target death and relevant life/equipment invalidation stop
   engagement. Prove an off-camera target in front can be attacked and a target
   behind cannot. Validate at the defined attack event, including adverse network
   conditions; clients cannot accelerate swing timing or invent valid facing.
5. **One-hostile playable milestone — P2-12 / P3-06 / P3-07 / P3-14.** Reuse
   current wolf, equipment, guide, HUD and ability infrastructure. Prove one
   instant plus one cast-time ability, contextual NPC interaction, all thirteen
   specification checks, and readable selected-target/engagement feedback. Run
   native checks at 1280×720, 1920×1200 and 3440×1440, record Linux GPU evidence
   separately from Windows qualification, and preserve earlier experiments until
   the replacement passes.

P3-17/P3-18 remain unspecified. General PvP, ground effects and world-object
content follow the shared contracts without expanding this minimum milestone.

---

# Superseded WoW/GW2 proposal — historical record, 2026-11-01

P2-04 / P2-05 / P2-07 / P2-09 / P2-11 / P3-06 / P3-14.
The user requested GW2-style movement/camera with optional action camera, then
clarified that playing should feel like WoW/GW2. This refines the approved
tab-target/mobile-combat architecture. It does not replace the authoritative
movement, networking, terrain or Bevy foundations.

## Desired experience

Movement should start and stop promptly, without artificial input smoothing.
Camera motion should respond immediately to the mouse, independently of server
ticks. Ordinary reconciliation should remain visually smooth without hiding
collision or delaying player intent. Presentation should match actual speed,
with readable idle/run/strafe/backpedal/jump/dodge transitions. Animations and
camera behavior need playtesting; passing policy tests alone cannot prove feel.

Use GW2 as the primary movement/camera reference and retain familiar WoW-style
target selection, ability bars, NPC interaction and readable combat feedback.
Use normalized WASD camera-relative movement as the initial GW2-oriented base;
do not silently introduce WoW's keyboard-turn default into that control scheme.
Exact speed, backward/sideways modifiers, acceleration, zoom and sensitivity are
tuning choices to validate rather than claimed copies of either game's internals.

Classic cursor mode remains the default. Left-drag should orbit independently of
travel; right-drag should steer movement/facing while preserving the existing
click threshold and UI ownership. Wheel zoom must respect static obstacles and
UI consumption. Left/right world clicks retain the adopted target/basic-attack/
friendly-interaction contract. Both-button forward movement and autorun belong
to the subsequent classic-control pass; neither is implemented by this decision.

Optional action camera should provide continuous mouse-look, a centre reticle,
camera-relative movement, reticle-based enemy selection/basic attacks, and F
interaction. Tab and numbered abilities remain available. Use a dedicated
toggle, with temporary cursor release and automatic release for dialogue,
inventory, menus and loss of focus. Restore the requested mode after a panel
closes; never fire an attack from a UI click or replay input accumulated while
unfocused. Changing modes must not rotate/teleport the pawn or reset zoom.
Bindings should be explicit and eventually rebindable. The first source candidate
uses F8 to toggle and Left Ctrl for temporary cursor release. Native qualification
passes at the three sizes below. The qualified normal dodge uses V because the Linux desktop reserves Alt as its
window mouse modifier; native checks pass at all three sizes.

ArenaNet describes continuous mouse-look, crosshair targeting, a toggle and
automatic suspension for dialogue/menus/panels in its
[official action-camera introduction](https://www.guildwars2.com/en-gb/news/introducing-the-action-camera/).
This reference informs our proposed behavior; the implementation and validation
status below describe Theia itself.

## Slice order and acceptance

1. Finish the bounded dodge command/correction prerequisite already underway,
   preserve legacy movement envelopes and verify deduplication and exact replay
   state at 20/30/60 Hz. Keep the existing playable checkpoint available.
2. Add the client camera modes and classic orbit/steering separation using
   standard Bevy input, UI and picking. Prove toggle/cursor/focus/menu ownership,
   reticle targeting, no accidental clicks, zoom/collision and unchanged server
   range/LOS checks. Run native OS-input checks at 1280×720, 1920×1200 and
   3440×1440 fullscreen, with exact Linux GPU evidence.
3. Install server-owned dodge and shared prediction with collision/cooldown,
   death/stream/chunk-edge and adverse-network acceptance. Initial 3 m / 250 ms /
   3 s policy remains a tuning placeholder and grants no invulnerability. A
   faithful GW2 endurance/evade system would require its own explicit gameplay
   rules, authoritative checks, presentation and tests.
4. Playtest and tune the combined loop: movement, facing, camera, animation,
   targeting, mobile casts, melee range and latency correction. Record measured
   jitter and responsiveness separately from subjective feel. Preserve keyboard
   accessibility and normal UI use in either mode.

## Current implementation boundary

`build/dodge-qualified-v1-20261101` additionally qualifies server-owned predicted
V dodge, expiring one-shot input, sealed gameplay profile and cooldown feedback:
100 client / 436 network / 43 Python passes and seven successful native starts
with zero GPU diagnostics. [Dodge behavior and limits](DODGE_MOVEMENT.md).


`build/camera-mode-qualified-v1-20261101` qualifies the first optional action
camera and independent classic left-orbit, with 97 client tests, 407 network
passes / 10 skips, denied-warning workspace Clippy and seven successful native
starts (zero GPU diagnostics). [Concrete controls, evidence and limits](PLAYER_CONTROLS.md).
`build/world-input-qualified-v1-20261101` and earlier checkpoints remain available.

The subsequent `build/camera-panels-qualified-v1-20261101` qualifies Escape panel
priority, world dragging around open UI and explicit control hints, with 99 client
tests, 416 network passes / 10 skips and three final native OS-input starts with
zero GPU diagnostics. Full WoW/GW2 feel remains ongoing work: movement/facing
animation and tuning,
autorun, independently replicated facing and authored directional dodge/roll
presentation. Server-owned predicted dodge is now qualified: [DODGE_MOVEMENT.md](DODGE_MOVEMENT.md). No Windows, physical hardware input,
new FPS or capacity claim is made. Changes stay local.
