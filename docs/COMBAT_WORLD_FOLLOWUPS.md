Current update, 3 November 2026: the initial [local Classic minimap](MINIMAP.md)
is qualified with resident terrain, manual heading, committed guide/quest icons
and normal native input/layout/rejoin checks. Normal keyboard chunk/origin traversal and a separate accelerated retained-mode
continent diagnostic now also pass. Broader HUD/map layers remain follow-ups. Earlier dates below retain
their historical checkpoint scope.

Broader follow-up register: [Classic world systems](CLASSIC_WORLD_SYSTEMS_REFERENCE.md)
adds combat membership, attack resolution, separate tagging/credit, spawns,
linked encounters, resource/control rules and corpse recovery. It includes the
repo assessment and working policies. The current strict combat milestone stays
first; these notes do not restart deferred dodge/action-camera work.

# Combat, world interaction and HUD follow-ons — 2026-10-31

**Scope amendment, 1 November 2026:**
[THEIA-COMBAT-001](COMBAT_TARGETING_MOVEMENT_SPEC.md) supersedes earlier hybrid
controls and dodge tasks below. Defer active dodge/action camera; prioritize
classic movement, manual facing, persistent selection and separate weapon-timed
autoattack using the [current migration order](MOVEMENT_CAMERA_TARGET.md).
Enemy, quest, minimap and broader HUD goals remain; historical control requests
below do not override the new specification.

The [enemy AI research/reference](ENEMY_AI_REFERENCE.md) supplied on 1 November
records level/disposition detection, bounded threat, configurable leash/evade,
patrol recovery, assistance and later ranged/flee/dungeon work. Its illustrative
values are not new locked balance rules. Current return remains damageable;
full evade/reset and threat are explicit gaps after the player combat foundation.

The user requested these additions while authorizing continued development and
recommended implementation decisions. Continue the existing working slices; this
register refines existing task scopes rather than replacing the adopted roadmap
or assigning guessed meanings to P3-17/P3-18. Those two remain unspecified.

The equipped stance/moving grip and initial one-NPC perception/chase/melee/leash /
return slice are now [qualified in the playable client](CREATURE_PURSUIT_NATIVE.md).
Player sword melee, initial player hit outcomes and travelling magic are now
qualified. Remaining work includes ranged enemy integration and bow/arrow travel. Quest markers, minimap
and HUD presentation remain scheduled with their actual data prerequisites.

## Current behavior versus requested additions

| Request | Existing behavior | Remaining scope / task IDs |
| --- | --- | --- |
| Better UI, optionally using downloaded HUD elements | Standard Bevy development HUD, targeting, inventory, quest and dialogue widgets; native checks at 1280×720, 1920×1200 and 3440×1440 | Coherent game HUD, hierarchy/readability, assets, tooltips and scalable layout; P2-14 / P3-14 / P5-13 / P5-19 |
| Enemies patrol, perceive a player and run in to attack | One authoritative NPC now has configurable perception/LOS, navigation-backed Chase/Attack and private melee feedback; real two-account native checks pass | Broader AI/threat/bosses and public NPC attack timing; P3-07 / P4-03 / P4-08 |
| Player and enemy melee attacks require reach | Player Sword Slash and chasing enemy melee both use checked 3 m reach; committed sword, windup/completion life/LOS, native moving attacks and unequip cancellation now pass | General weapon/stat balance, more melee abilities and ranged enemy expansion; P2-12 / P3-06 / P3-07 |
| Player and enemy magic have finite range | Player Arcane Bolt now has checked 30 m launch range, finite homing travel, world collision and impact-only damage; see [qualified magic](PROJECTILE_TRAVEL.md) | Ranged enemy behavior and consistent launch/completion/impact semantics; P3-06 / P3-07 / P4-08 |
| Chasing ends at a configured distance; enemy returns to patrol | Configured 35 m leash from the captured patrol anchor, lost-sight grace, safe return and saved-route resumption now pass native checks | Production evade/balance rules, generalized/streamed navigation and population scaling; P3-07 / P4-03 |
| Magic and bows have projectiles | An opt-in server-owned homing Arcane Bolt and visible standard Bevy travel now pass real two-client native checks | Bow equipment/arrow visuals and shared travel, ranged NPC casting, general volumes and AoE; P3-06 / P4-08 / P5-15 |
| Chance to hit, miss or deflect | Opt-in player direct damage now has checked server-owned hit/miss/deflect resolution and encrypted outcomes; actual two-client native feedback now passes | NPC outcome expansion, defence/class rules and final balance; P3-06 / P4-08 |
| Floating quest markers above NPCs | One bounded available/active/ready Bevy guide marker is [qualified](QUEST_MARKERS.md) | Generalized quest/NPC associations, icon art and broader presentation; P3-11 / P3-12 / P3-14 |
| Minimap | [Qualified Classic local map](MINIMAP.md): north-up resident-mesh terrain, manual player arrow, committed nearby guide/quest icons, fixed 17-KiB images, no terrain loads, three native UI sizes and normal quest/restart rejoin | World atlas/zoom/fog, other NPC/party/waypoint icons and art/decluttering remain; P2-14 / P3-13. No P4-02 predictive-streaming completion claim. |

Existing ranges above describe current development profiles, not finalized game
balance. Existing experiments and checkpoints stay available until replacements
pass their concrete checks.

## Small slices and acceptance

1. **P3-07 perception and engagement.** Prerequisites: current patrol, entity/life
   identity, player health and static collision/visibility queries. Add a bounded,
   server-owned candidate scan and configurable perception radius/LOS policy.
   Acquire only a live, current player; a cosmetic proximity test in the client
   must never create aggro. Test inside/outside radius, occlusion, multiple
   candidates, disconnect and old actor/life observations. Demonstrate actual
   two-client patrol-to-engagement behavior.
2. **P3-07 / P4-03 chase, melee and leash.** Prerequisites: perception plus a
   bounded supported navigation/query path for moving destinations. Capture the
   NPC's last valid patrol position and route progress on engagement. Measure the
   leash from that anchor so repeated small player moves cannot pull it across
   the world. Stop attacks on evade, return to the saved patrol anchor and resume
   the route. Configure chase/return speed, leash distance and appropriate timeout
   or LOS grace. Navigation must respect terrain/buildings, unavailable chunks
   and queue limits; do not substitute straight-line movement through obstacles.
   Test reach boundaries, target death/disconnect/new life, leash crossings,
   blocked/unavailable routes, repeated aggro and return without teleporting or
   duplicate damage. No general multi-target threat service is needed yet.
3. **P2-12 / P3-06 melee and finite ranged attacks.** Add the initial player sword
   attack and ranged enemy ability through catalogues. Server validates equipment
   requirements, current target/life, finite reach/range, LOS and timing at each
   required boundary. Test stepping into/out of reach during windup, moving casts,
   stale lives and interrupted/dead attackers. Client animation and weapon mesh
   placement do not establish whether damage lands. Keep player/NPC physical
   collision disabled; gameplay hit detection is a separate requirement.
4. **P3-06 / P4-08 hit outcomes.** Add explicitly configured hit/miss/deflect
   probabilities and eligibility. Resolve on the server with controlled/replayable
   test randomness, never client-selected rolls. Bound probabilities and queues;
   test certain-hit/certain-miss edges, repeated command/event delivery, correct
   resource/cooldown behavior and no damage/rewards for a miss or deflect. Present
   the same result to the attacker and relevant observers. Final balance and
   additional outcomes can follow this functional slice.
5. **P3-06 / P5-15 travelling magic and arrows.** Specify launch and impact rules,
   speed, maximum distance/lifetime, homing/trajectory and obstruction behavior in
   data. Server owns travel/impact timing and applies damage at most once to a
   valid life. A visible trail after instant damage is not completion. Test travel
   time, expiry/range, obstacles, target movement/death/new life, attacker departure
   and delayed/duplicated observations. Use standard Bevy presentation and bounded
   projectile/cue budgets; preserve critical telegraphs/audio under cosmetic load.
6. **P3-11 / P3-12 floating quest markers.** Use known server quest/dialogue state
   for available, active and ready-to-turn-in markers. A marker does not bypass
   server interaction range, LOS or eligibility. Test accept, objective progress,
   turn-in, reconnect, unknown/despawned NPCs, camera occlusion, distance scaling
   and floating-origin changes. Include shape/text distinctions as well as color.
7. **P2-14 / P3-13 minimap.** Start with an inexpensive map layer and local player
   heading plus known nearby guide/quest icons. Use checked global positions and
   centralized map conversion. Do not load/render the entire 256 km² world or
   force terrain residency for the minimap. Test chunk/origin boundaries, map
   edges, heading, known-entity filtering and bounded texture/update costs. Verify
   the map at the supported UI resolutions before adding larger world-map scope.
8. **P2-14 / P3-14 HUD redesign.** Inventory selected local sprites/fonts and their
   source notices first. Build the layout using standard Bevy UI, preserve actual
   health/resource/action/target/quest/inventory behavior and keep diagnostics
   separate from the player-facing layout. Check pointer bounds, text fit,
   tooltips, readable priorities and scaling at 1280×720, 1920×1200 and actual
   3440×1440. Record native GPU/performance evidence and any limitations; do not
   equate a mockup with working gameplay UI.

## Local HUD source intake

The supplied assets were found at `/home/steven/Downloads/Synty Models/HUDs`
(the spelling/case differs from the informal path in the request). Read-only
inventory finds 1,866 PNG files, 261 Unity prefabs, 151 animation files,
51 controllers, 29 scenes, two TTF fonts with OFL notice files and a user-guide
PDF. These are available source assets, not an imported or qualified Bevy HUD.
Selected PNGs/fonts may be useful directly; Unity prefabs/controllers/animation
bindings need deliberate adaptation. No Unity scripts have been executed or
package assets copied into Git. Review the source notices when selecting assets.

The initial moving grip, pursuit, player melee, hit outcomes and travelling magic
slices are qualified in their separately preserved checkpoints. The user resumed development on 2026-11-01; floating quest markers are the current
small slice. The projectile checkpoint remains preserved.

Read-only follow-up intake also found three quest-icon PNG variants under
`Sprites/Icons_Map` and minimap box/dial/diamond prefabs in the supplied HUD folder.
They remain available source assets, without an import or qualified game HUD.

## Adopted mouse and interaction controls — 2026-11-01

The user supplied this control contract during continued development. Implement
it after the current marker qualification. Task IDs: P2-05 / P2-11 / P2-12,
P3-06 / P3-12 / P3-14. Preserve server validation, bounded commands and the
working keyboard path while adding mouse input.

| Input | Intended behavior |
| --- | --- |
| Left-click enemy | Select enemy |
| Right-click enemy | Select and request a basic attack |
| Tab | Cycle eligible enemies |
| 1–5 | Combat ability slots |
| Left-click friendly NPC | Select NPC |
| Right-click friendly NPC | Request dialogue, trade or quest interaction |
| F | Context-sensitive interaction |
| WASD and mouse | Movement and camera |
| V | Authoritative predicted dodge, in the schema-4 dodge bundle |

The initial mouse/guide/1–5/F controls are now [qualified](PLAYER_CONTROLS.md).
Dodge is now [qualified](DODGE_MOVEMENT.md); directional animation and facing
remain follow-up work. Distinguish a right
click from camera dragging with a bounded movement threshold; UI consumes its
own clicks. Mouse selection must resolve a known current entity, never an
arbitrary mesh or stale life. Friendly selection must not grant damage, and
interaction/basic attacks must still validate range, line of sight, equipment,
life and timing on the server. Basic attack policy should be explicit rather
than guessing which ability slot is melee. Dodge requires a separate checked
movement intent, authoritative timing/distance/cooldown/collision and prediction;
no client-only teleport or implicit invulnerability. Actual native input checks
must cover enemy/friendly selection, drag/UI suppression, invalid/out-of-range
requests and existing keyboard regressions.
