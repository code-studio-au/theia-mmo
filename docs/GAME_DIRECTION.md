# Game direction

Theia's narrative design lives in [theia-story](../../theia-story/README.md): worldbuilding,
characters, campaign acts, quests and their approval/decision records. Narrative
plans remain separate from implemented gameplay.

Updated 2026-11-14 to reflect the adopted [architecture](engineering/README.md)
and [THEIA-COMBAT-001 v1.0](COMBAT_TARGETING_MOVEMENT_SPEC.md), adopted 2026-11-01.
The initial playable version uses Classic-style third-person controls, persistent
explicit targets, manual authoritative character facing and character-relative
movement. Weapon-timed autoattack is separate from hotkey abilities. Casting
movement and interrupt restrictions belong to ability data. Action camera, active
dodge, reticle targeting and automatic facing/tracking remain deferred; the earlier
GW2/hybrid proposal is historical. See [migration and evidence](MOVEMENT_CAMERA_TARGET.md)
for the contract and qualified slices, and [Classic formulas](CLASSIC_FORMULAS.md)
for source/version uncertainty and metric mappings.

The user wants a WoW-style MMORPG. Theia is now a game built with Bevy, rather
than a replacement general-purpose engine. Use WoW as the gameplay reference and
create original world content, names, characters, artwork and audio.

The intended experience is third-person exploration of a shared fantasy world,
tab-target combat, distinct character classes/roles, abilities on an action bar,
NPC quests, progression, equipment/loot and cooperative encounters. These are
targets, not implemented features. No particular expansion, complete rule set,
content catalogue or MMO-scale capacity is currently selected or promised.

Players pass through both other players and NPCs, as explicitly confirmed by the
user. Physical world collision applies to terrain, buildings and solid props.
NPC targeting, ability range and hit detection remain separate gameplay checks;
an NPC being nonblocking does not make it untargetable or invulnerable.

The world is predominantly static. Scripted server-authoritative elevators and
moving platforms are allowed; client vegetation wind/bending is a presentation
effect. Buildings, doors and ordinary props remain static unless explicitly
specified. No general destructible or freely simulated scenery system is planned.
Players and NPCs can move and animate; quest/loot state does not itself require
moving environment geometry. None of the platform/vegetation systems exists yet.

Build a small starting-zone experience before expanding the world:

1. Two actual players can connect and walk together under server authority.
2. A representative character and small environment load through Bevy's normal
   asset pipeline; camera controls and world collision make exploration usable.
3. Select one NPC target, use an ability, show health/cooldown, defeat it and see
   the same result on both clients. The server checks target, range and cooldown.
4. Accept one quest, complete its objective and return for a server-owned reward.
5. Add the minimal inventory, progression and persistence needed to retain that
   experience across sessions. Then expand classes, groups and encounters.

The initial control and targeting contract is adopted. Prototype ability values,
progression fixtures and imported presentation do not establish final class balance
or visual direction. Do not build every WoW subsystem at once or
present the existing capsule scene as representative game content. The current
milestones and execution evidence are tracked separately in ROADMAP and STATUS.
