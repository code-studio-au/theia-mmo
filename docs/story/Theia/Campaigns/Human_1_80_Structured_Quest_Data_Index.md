# Human 1–80 — Structured Quest Data Index and Validation

**Status: authored implementation-data draft, NOT production-ready or engine-integrated.** The [canonical campaign](Human_1_80_Canonical_Campaign.md) is the single source of truth for story outcomes; [production quest specification](Human_1_80_Production_Quest_Specification.md) is the authoring reference.

## Quest data files

- [Human_1_10_Quest_Data_Draft.json](Human_1_10_Quest_Data_Draft.json)
- [Human_10_20_Quest_Data_Draft.json](Human_10_20_Quest_Data_Draft.json)
- [Human_20_30_Quest_Data_Draft.json](Human_20_30_Quest_Data_Draft.json)
- [Human_30_35_Quest_Data_Draft.json](Human_30_35_Quest_Data_Draft.json)
- [Human_35_45_Quest_Data_Draft.json](Human_35_45_Quest_Data_Draft.json)
- [Human_45_55_Quest_Data_Draft.json](Human_45_55_Quest_Data_Draft.json)
- [Human_55_70_Quest_Data_Draft.json](Human_55_70_Quest_Data_Draft.json)
- [Human_70_80_Quest_Data_Draft.json](Human_70_80_Quest_Data_Draft.json)
- [Human_80_Finale_Quest_Data_Draft.json](Human_80_Finale_Quest_Data_Draft.json)

## Cross-file validation — 2 November 2026

- **184 quest definitions**, **184 distinct IDs**, **184 unique completion flags** across nine JSON files.
- **683 authored stage entries**. **0 quests retain generic stage placeholders**.
- **0 structural errors** found in this check.
- Story-locked events have explicit stages: first Kel'vari reveal, Vaer'uun invitation, Veilbreaker's irreversible success, immediate Binding dissolution, and Oth'varen's sacrifice after Thes'var's sabotage.

## Important limitations

- The stages are individually named and authored, but target keys are **design aliases**, not validated runtime NPC/object IDs. They are not executable quest definitions.
- All map IDs, NPC spawn IDs, encounter IDs, XP rewards and item rewards remain unresolved; these are blockers to importing the data into the engine.
- A passing structural check does **not** establish narrative continuity, gameplay balance, combat feasibility, or that the quests have been tested.
- Main/optional classification is provisional and still needs full regional-content balancing.

## Next implementation gates

1. Resolve world geometry and zone/transport data; assign actual map, NPC, object and encounter IDs.
2. Define complete NPC dialogue scripts and player knowledge gates.
3. Specify combat encounters, boss AI, leash/reset behavior and solo checkpoints.
4. Tune XP and rewards against non-story regional progression and travel time.
5. Implement server-authoritative quest persistence and automated tests; playtest full chapter transitions.

**Do not label this data production-ready until the runtime bindings and playtests are complete.**
