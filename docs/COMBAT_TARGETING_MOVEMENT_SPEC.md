# THEIA — Combat, Targeting & Movement Specification

Formula/units amendment, 1 November 2026: follow
[Classic formulas with documented evidence](CLASSIC_FORMULAS.md) wherever possible;
keep Theia metric, with 1 world unit = 1 metre. Earlier illustrative prototype
values remain provisional until replaced and verified.

**THEIA-COMBAT-001 · Version 1.0 · 1 November 2026 · Adopted**

The initial combat foundation follows a strict World of Warcraft Classic-style
control and targeting model. This supersedes the earlier hybrid-facing and
GW2-oriented movement/camera proposal for the initial playable version.
GW2-inspired mechanics are deferred and require a later explicit decision.

Source: the design specification supplied by the user on 1 November 2026.
This Markdown transcription preserves its requirements and acceptance checklist;
it omits the conversational statement that no repository files had been changed.
**Adopted design is not implemented behavior.** Existing experiments and their
evidence remain available; [migration and gaps](MOVEMENT_CAMERA_TARGET.md) track
the work needed to satisfy this contract.

## 1. Core design requirements

- Traditional third-person MMO movement and camera controls.
- Persistent, explicit enemy and friendly NPC targeting.
- Manual character facing for attacks and abilities.
- Basic autoattacks that continue without repeated input.
- Hotkey-driven abilities with range, resource and facing validation.
- Right-click contextual interactions for enemies, NPCs and world objects.
- No automatic camera lock-on or character-facing assistance.
- Consistent combat rules across PvE, dungeons and PvP.

The player controls where their character moves and faces. Combat determines
whether attacks are valid; it does not correct the player's positioning.

## 2. Mouse and keyboard controls

| Input | Required behavior |
| --- | --- |
| W | Move forward |
| S | Move backward |
| A / D | Turn character left / right |
| Q / E | Strafe left / right |
| Space | Jump |
| Hold left mouse | Rotate camera independently |
| Hold right mouse | Rotate camera and steer character |
| Hold both mouse buttons | Move forward and steer |
| Mouse wheel | Zoom camera |
| Left-click enemy | Select target without attacking |
| Right-click enemy | Select enemy and engage autoattack |
| Tab | Cycle eligible hostile targets |
| Shift + Tab | Reverse target cycle |
| Left-click NPC | Select NPC |
| Right-click NPC | Talk, trade or open available interaction |
| Right-click object | Interact, gather or loot when applicable |
| 1–= | Default primary action-bar keybinds |
| Esc | Close active interface or clear target according to context |

Keybindings must be configurable, including A/D strafe preferences. Dragging
and clicking must remain distinguishable, and UI input ownership must be
preserved. No active dodge binding is part of this initial specification.

## 3. Camera, movement and target independence

| State | Controlled by | Changes selected target? |
| --- | --- | --- |
| Camera orientation | Mouse movement while looking | No |
| Character orientation | Turn keys / right mouse steering | No |
| Character position | Movement keys | No |
| Target selection | Mouse click / targeting keybind | Yes |
| Combat engagement | Right-click / autoattack command | Only when selecting a new enemy |

Left-mouse camera rotation preserves character facing and selected target.
Right-mouse steering rotates the character with the camera while preserving
the target. The camera never rotates automatically to centre a selected target.

## 4. Target selection

Selection is persistent. A valid target remains selected when it moves behind
the player, leaves attack range, breaks line of sight or the player turns away.
These conditions affect attack eligibility rather than selection.

Clear or change selection only on explicit request, entity removal or another
defined invalidation. Tab uses a predictable, configurable search that favours
enemies in front of the character. It must not depend solely on camera direction
or silently change targets during combat. Explicit single-target selection takes
precedence over intelligent auto-targeting in the prototype.

## 5. Autoattack

Autoattack is persistent combat intent, separate from selection and hotkey
ability execution.

| State | Meaning |
| --- | --- |
| IDLE | No basic attack requested |
| ENGAGED | Autoattack enabled against a target |
| ATTACKING | Range, facing and other requirements met |
| PAUSED | Temporarily cannot attack; resumes when requirements are met |

Right-clicking an eligible hostile starts attack intent. Attacks follow equipped
weapon timing and combat modifiers. Validate range, facing, visibility and
eligibility at the appropriate attack event. Turning away or leaving range pauses
attacks without clearing selection or engagement; returning resumes attacks.
Explicit cancellation, target death or relevant combat invalidation ends intent.

Manage the weapon swing timer independently of ability execution. Each ability
needs explicit rules for continuing, suspending, delaying or resetting it.
Melee autoattack is the reference prototype. Ranged autoattack, ammunition and
class-specific basic attacks are later extensions. Repeating a weapon ability
on its cooldown is not the initial autoattack model.

## 6. Manual facing and attack validation

Standard attacks and abilities never turn the character toward a target.

| Condition | Required behavior |
| --- | --- |
| Enemy in range and in front | Attack permitted if other checks pass |
| Enemy outside range | Attack cannot hit |
| Enemy directly behind character | Standard frontal attack blocked |
| Enemy to the side | Ability's attack arc determines eligibility |
| Enemy behind obstacle | Line-of-sight restrictions apply |
| Enemy outside camera view but in front of character | Attack remains valid |
| Character moving or strafing | Allowed if ability rules permit |
| Character jumping | Does not inherently cancel basic combat |
| Character stunned or disabled | Status-effect rules apply |

The prototype uses one consistent frontal arc, calibrated against the intended
WoW Classic-style behavior. The numeric arc remains design work; this decision
does not invent an angle. Define arcs in ability data so future positional skills
do not require rewriting targeting.

## 7. Ability activation

Abilities are distinct from basic attacks. Each definition needs:

| Property | Purpose |
| --- | --- |
| Target type | Enemy, friendly, self, ground or none |
| Minimum / maximum range | Activation distance |
| Facing requirement | Required attack direction |
| Line of sight | Visibility requirement |
| Resource cost | Mana, energy, rage or other class resource |
| Cast time | Delay before execution |
| Cooldown | Reuse restriction |
| Global cooldown | Shared cooldown participation |
| Movement restriction | Whether casting while moving is allowed |
| Interrupt rules | Movement, damage or status-effect interruption |
| Autoattack interaction | Continue, suspend, delay or reset basic attacks |

Support instant abilities, cast-time spells, melee strikes, targeted ranged
abilities and ground-targeted effects in the design. The first playable milestone
requires at least one instant and one cast-time ability. Abilities never
implicitly rotate the player. Mobile casting is an ability rule rather than a
universal requirement.

## 8. NPC and world interaction

Right-click friendly NPCs to open their appropriate quest, merchant, trainer or
conversation interface. Right-click interactable objects to attempt activation,
gathering or looting, subject to range, availability, permissions and other rules.
Right-click hostiles engages combat.

Out-of-range interaction gives feedback in the prototype. It does not move the
character automatically into range. Automatic approach is separate future work.

## 9. PvP and server authority

Selection should feel immediate locally; the server owns combat outcomes and
validates target existence/eligibility, character position and facing, range,
LOS, swing timing, costs, cooldowns, damage, healing, interrupts and statuses.
PvE and PvP share targeting and facing rules. Selecting another player never
changes facing. Account for latency without bypassing authoritative validation.

## 10. Deferred GW2-inspired features

The initial scope excludes automatic facing assistance, action camera/permanent
mouse-look, crosshair targeting, active dodge (including the existing movement-only
experiment and any future invulnerability version), repeating weapon skills as
basic attacks, directional action combat, camera-aim target switching and
skill-specific automatic tracking.

Deferral does not promise eventual inclusion. Each feature needs evidence of
improvement before reconsideration. Retain extensible movement/ability contracts
without unnecessary new prototype abstractions.

## 11. Minimum playable combat milestone

Prove one player can reliably fight one hostile NPC with this control model.
All thirteen prototype checks below are demonstrated by the [classic controls](CLASSIC_MOVEMENT.md),
[persistent targeting](CLASSIC_TARGETING.md), [melee client](CLASSIC_MELEE_CLIENT.md)
and [ability rules](CLASSIC_ABILITY_RULES.md) profiles. This is development-profile
acceptance with fixed damage fixtures, normal OIDC/PostgreSQL and synthetic OS
input. Final ability checks use disconnected-display uncapped/windowed rendering;
previous melee evidence covers connected-display fullscreen. Full Classic damage
formulas, physical hardware and Windows remain outside this acceptance.

- [x] WASD/QE movement and standard mouse-look work.
- [x] Left mouse rotates camera without turning character.
- [x] Right mouse rotates camera and character.
- [x] Click and Tab selection retain the same target.
- [x] Right-click hostile target engages autoattack.
- [x] Basic attacks obey weapon timing, range and facing.
- [x] Turning away pauses attacks without clearing target.
- [x] Returning to valid facing and range resumes attacks.
- [x] At least one instant ability and one cast-time ability work.
- [x] Right-click friendly NPC opens interaction.
- [x] Enemy has health, pursuit, attack and death states.
- [x] Server validates combat events and damage.
- [x] Client displays selected target and combat feedback.

Advanced AI, complete class trees, group encounters and comprehensive PvP balance
are outside this milestone.

## 12. Logical architecture

Keep player input/configurable bindings, movement/camera, target selection,
combat intent/abilities, and authoritative combat simulation separate. The camera
never owns target selection; targeting never owns character orientation.
This permits later camera options without changing combat rules.

## 13. Locked decisions and remaining design

**Locked:** strict WoW Classic-style controls, persistent targeting, manual
facing and weapon-based autoattack for the initial foundation.

**Deferred:** GW2-inspired mechanics. **Open:** attack speeds, resources,
class abilities, exact arcs, GCD durations, threat and combat pacing.
Define the fundamental loop—weapon speed, ability timing, resource generation,
combat regeneration and enemy threat—before expanding individual classes.

The strict development prototype now passes its thirteen checks. Existing MMO foundations, level
1–80 progression direction, race stories, exploration, solo story dungeons,
faction conflict and open-world travel remain compatible with this decision.
