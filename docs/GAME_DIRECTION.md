# Game direction — 2026-10-16

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

All environment items are static, including doors: no opening/closing doors,
moving props or destructible scenery. Players and NPCs can still move and animate.
Quest/loot state and UI do not require moving environment geometry. Future asset
imports and collision should preserve these rules rather than adding interactive
doors or physics-driven scenery.

Build a small starting-zone experience before expanding the world:

1. Two actual players can connect and walk together under server authority.
2. A representative character and small environment load through Bevy's normal
   asset pipeline; camera controls and world collision make exploration usable.
3. Select one NPC target, use an ability, show health/cooldown, defeat it and see
   the same result on both clients. The server checks target, range and cooldown.
4. Accept one quest, complete its objective and return for a server-owned reward.
5. Add the minimal inventory, progression and persistence needed to retain that
   experience across sessions. Then expand classes, groups and encounters.

The first combat archetype, visual style, progression rules and precise camera
behaviour remain design choices. Do not build every WoW subsystem at once or
present the existing capsule scene as representative game content. The current
milestones and execution evidence are tracked separately in ROADMAP and STATUS.
