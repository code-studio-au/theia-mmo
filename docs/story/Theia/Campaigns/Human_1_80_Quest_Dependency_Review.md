# Human 1–80 Quest Dependency Graph — Review Draft

This document accompanies the canonical Human 1–80 campaign and its production quest specification. The JSON graph is a **provisional implementation proposal**, not a new story authority.

It contains 184 entries, 169 main-story entries and 15 optional local entries. Every main-story entry provisionally depends on the preceding main-story entry. Optional quests have no main-story dependencies. This is intentionally conservative: revise for parallel missions and local quest chains before implementation.

Minimum chapter levels are 1, 5, 10, and every five levels through 75. The finale requires level 80. Each completion flag is unique and every prerequisite points backward. Null map, NPC, encounter and reward IDs are **production blockers**, not assigned values.

## Required review

- Validate the local-versus-main classification against the full original optional register; the current optional set covers early regional additions only. Late-region independent quests still need to be authored and classified.
- Confirm Brynja's preliminary technical note precedes the player's first personal visit to Kar Durn.
- Keep Guardian Passage access restricted and Kel'vari-consented; never make optional XP depend on story-only Shroud access.
- Maintain hostile-only Dominion archive infiltration, no friendly Dominion quest contacts.
- Ensure Khor'vun dies at Veilbreaker but activation is irreversible: Veil destroyed, Binding dissolves immediately, Seven freed, Shroud starts dissipating.
- Ensure the level-80 finale is not required to earn XP for level 80; Oth'varen discovers the fatal correction only after Thes'var's sabotage.
- Before release: assign exact map coordinates, NPC IDs, enemy archetypes, interaction objectives, dialogue, XP, loot, checkpoint logic, world-stage presentation, and automated quest tests.

## Automated validation expectations

184 unique quest IDs and completion flags; no missing prerequisite references or cycles; main story never depends on an optional quest; level gates do not decrease; the final operation starts at level 80.
