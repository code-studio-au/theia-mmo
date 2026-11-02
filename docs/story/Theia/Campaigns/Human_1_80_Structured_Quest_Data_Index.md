# Human 1–80 — Structured Quest Data Index and Validation

**Status: implementation draft, not engine-integrated.** Canonical narrative: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Quest design: [Production Quest Specification](Human_1_80_Production_Quest_Specification.md).

## Data files

- [Human_1_10_Quest_Data_Draft.json](Human_1_10_Quest_Data_Draft.json)
- [Human_10_20_Quest_Data_Draft.json](Human_10_20_Quest_Data_Draft.json)
- [Human_20_30_Quest_Data_Draft.json](Human_20_30_Quest_Data_Draft.json)
- [Human_30_35_Quest_Data_Draft.json](Human_30_35_Quest_Data_Draft.json)
- [Human_35_45_Quest_Data_Draft.json](Human_35_45_Quest_Data_Draft.json)
- [Human_45_55_Quest_Data_Draft.json](Human_45_55_Quest_Data_Draft.json)
- [Human_55_70_Quest_Data_Draft.json](Human_55_70_Quest_Data_Draft.json)
- [Human_70_80_Quest_Data_Draft.json](Human_70_80_Quest_Data_Draft.json)
- [Human_80_Finale_Quest_Data_Draft.json](Human_80_Finale_Quest_Data_Draft.json)

## Validation results (2 November 2026)

- 184 quest definitions across nine files; 184 distinct IDs and 184 distinct completion flags.
- 577 stage definitions in total.
- 28 quests have explicitly authored narrative-critical stages; 139 use generic placeholder stages that **must be replaced** with concrete interactions and encounter scripting before implementation.
- 0 structural errors detected in this authoring pass: none.
- All map IDs, NPC spawn IDs, combat encounter IDs, XP and loot remain unresolved; no file is ready for direct engine import.

## Remaining work to reach implementation quality

1. Replace all generic stage placeholders with real object/NPC targets, unique objective counters, triggers and story-specific dialogue.
2. Resolve the 184-entry narrative inventory into a tuned main/regional quest economy; assign XP budgets and level requirements based on playtesting.
3. Populate actual zone/map coordinates and transport route IDs from the approved game map, not invented values.
4. Specify enemy AI, patrols, social aggro, facing checks, boss state machines, solo difficulty, checkpoints and reset behavior.
5. Implement quest runtime schema validation, persistent credit, versioned migrations and automated quest completion tests.
6. Playtest the level-80 finale and all irreversible canon events; keep the Binding dissolution and Oth'varen's death in their locked sequence.

**Note:** A syntactically complete JSON authoring draft is not the same as a production-ready game system.
