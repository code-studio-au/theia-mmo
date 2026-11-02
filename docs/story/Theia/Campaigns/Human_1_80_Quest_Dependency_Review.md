# Human 1–80 Quest Dependency Review — v0.2

**Status: Needs Review.** This is an implementation-planning document subordinate to [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). The machine-readable graph is [Human 1–80 Quest Dependencies](Human_1_80_Quest_Dependencies.json).

## Revised classification

- **156 provisional main-story entries**, **28 optional regional entries**, **184 total**. The classifications are design proposals, not locked narrative canon or tested quest database.
- Main-story entries depend on the preceding main-story completion flag. Optional entries are independently available by level in this initial graph. This is a conservative chain, not a final mission-by-mission prerequisite graph.
- There are no references from main-story prerequisites to optional entries, and no backward cycles were found in the reviewed data.

## Specific narrative fixes needed when converting optional quests

- **Q046 A Landing Still at War:** ordinary military defense optional. The evidence extraction required for the main investigation must be completed by Q045 or Q047.
- **Q048 A Different Kind of Healing:** optional Anwen follow-up; essential scientific comparison must be available from Sera's main investigation.
- **Q051 The Feeder Gallery:** optional maintenance story. The main Kar Durn investigation needs Brynja's authenticated plans from Q050, independent of tunnel rescue.
- **Q055 The Nursery Route:** optional Aelari woodland rescue. Research permission and ward alteration evidence cannot depend on it.
- **Q083 The Price of Secrecy:** optional historical context. Sera's disclosure of the Guardian gate is mandatory in Q081–Q082.
- **Q087 Those Who Remained:** optional rescue activity only. The **first living Kel'vari reveal must happen in mandatory Q086 The Hidden Crossing**, with Q089 Permission to Remain preserving sovereign consent.
- **Q090 A Warning Preserved:** optional restoration. The main Sixth Scholar lead must be available in Q091.
- **Q093 An Audience Earned:** optional archive service. Q092 custodial conditions must still authorize mandatory investigation.
- **Q100 The Cost of Keeping Quiet** and **Q101 Two Histories:** optional service and background, while mandatory Delegation evidence remains in Q102 and later.
- **Q159 The Lands Revealed:** optional regional refugee mission. The Shroud's immediate dissipation and the Seven's release remain mandatory in Q157.
- **Q168 Life at the Edges** and **Q169 The Route Between:** optional follow-up activities; Q174 final readiness must include essential ecological and transport safeguards independent of those missions.

## Production-stage requirements

1. Assign explicit zone/map IDs and precise NPC/object coordinates from the approved world map. No fabricated coordinates.
2. Define each quest's objective stages and triggers, required items or interactions, faction rules, combat encounters, AI, retry checkpoints, rewards and localization.
3. Break the provisional main-story chain into chapter-specific mission bundles; identify optional branches that can start in parallel and group travel logically.
4. Build regional alternatives sufficient to reach chapter minimum levels without completing the main plot. The original XP envelope is a planning hypothesis and requires balancing.
5. Confirm the Q156 Veilbreaker cutscene: player defeats Khor'vun but is too late; the Veil is destroyed, the Binding immediately dissolves, Seven freed and Shroud dissipation begins. Nhal's manifestation remains gradual.
6. Confirm Q177–Q181 final sequence: Thes'var dies, sabotage is discovered, Oth'varen only then realizes the correction is fatal, tells Sera and player, dies physically after alignment holds, Nhal's coherent identity shatters while essence persists.

**Validation:** 184 unique editorial quests, 156 provisional main and 28 optional; no main-story dependency on optional quests. This does not establish production readiness.
