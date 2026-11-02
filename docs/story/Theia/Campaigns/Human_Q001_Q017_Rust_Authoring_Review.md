# Human Q001–Q017 — Rust Quest Authoring Review

**Status: content-authoring review; not an executable catalogue.** Narrative authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Working inventory: [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json). Runtime contract: [Quest Engine Agreement](../../../QUEST_ENGINE.md). Codex owns runtime and persistence integration.

## Hard boundaries

- Keep 184 Q-series entries unchanged while reviewing; editorial H/HS provenance is still candidate-only. Never allocate an H-series runtime ID from title similarity.
- No permanent 32-quest production limit. Exportable packs and cross-pack registry are planned, not implemented.
- Saved journals currently bind exact catalogue bytes. Do not publish executable campaign content before version compatibility and migration rules are qualified.
- Native Rust catalogue currently accepts only `kill` and `npc_interaction`, with stable NPC and semantic action IDs. Other objective kinds remain authored intentions, not silently downgraded to NPC clicks.
- Standard quests require no Luau. XP values, actor IDs, creature IDs, item consumption and true eligibility gates must be reviewed before export.
- Existing Q001–Q017 stage list is a **planning order**, not a mandatory one-stage-per-interaction runtime structure.

## Quest-by-quest disposition

| Quest | Proposed stage grouping / content design | Current-engine feasibility | Open review |
|---|---|---|---|
| Q001 Before the Next Caravan | Stage A: ask Darian, Anwen and Brenna in any order (`all`); Stage B: report to Elara | Native semantic NPC interactions, once verified | Four NPC IDs and four distinct action IDs; preserve H1-01 candidate |
| Q002 The Road Patrol | Inspect two distinct signs; find worker; defend rescue; report to Darian | Partial: NPC report native; inspection and defense planned | Rescue credit must be a trusted encounter fact; retry only failed encounter |
| Q003 What the Healing House Needs | Recover sealed parcel; deliver to Anwen; witness treatment | Planned item/interaction and scene evidence | Decide physical quest item vs persistent evidence token, and consumption at delivery |
| Q004 The Missing Wheels | Inspect marker and wagon; interview teamster; report to Brenna | Partial: semantic conversations native | Inspection and witness access require authorized world facts |
| Q005 The Fence at Dawn (optional) | Inspect fence; defend livestock; collect repair materials; repair | Planned interactions/defense | Preserve independence from main chain |
| Q006 Seed Before Winter (optional) | Count sacks and compare ledger in either order; correct duplicate with Tomas | Partial: Tomas conversation native | Keep ledger correction as historical fact, not possession |
| Q007 A Cart That Can Move (optional) | Inspect axle; gather salvage; repair cart | Planned inventory/object use | Decide whether iron is consumed by repair; no profession gate |
| Q008 The Healer's Ledger (optional) | Interview two patients in either order; deliver herbs; file observations | Partial: interviews native | Herb acquisition vs possession and consumption unresolved; no magical explanation |
| Q009 An Unusual Buyer | Locate camp; defeat defenders; recover cargo and inspect order/seal; report | Partial: kill native, other events planned | Cargo must not be permanently losable story evidence |
| Q010 A Seal Without a Name | Compare fitting and workshop reference; obtain Heddan's interpretation | Partial: Heddan interaction native | Confirm giver and turn-in: Elara / Heddan is ambiguous |
| Q011 At the River Crossing | Join caravan; reach Stoneford; visit warehouse; report arrival | Partial: conversation native; travel planned | Protected PvE travel, not a fast-travel waypoint |
| Q012 Three Books | Read three records in any order; identify contradiction through explicit semantic action | Partial: final action may be NPC interaction if designed as such; inspection/choice planned | Wrong answer retryable; don't count greeting as correct deduction |
| Q013 The Office After Edric | Secure payment copy; interview Perrin; obtain introduction | Partial: conversations native | Evidence copy must survive item loss; confirm authorized official |
| Q014 The Quarry Bell (optional) | Find crew; defeat ordinary hazard; recover tally; report | Partial: kill native | Crew rescue and tally require distinct evidence |
| Q015 One Bridge, Two Claims (optional) | Inspect supports; hear both parties in either order; record repair priority; repair bridge | Partial: interviews native | Choice persists as local acknowledgement; both needs ultimately addressed |
| Q016 The Warehouse Night Watch (optional) | Review losses; patrol; defeat thieves; recover tools | Partial: kill native | Patrol and tools planned; avoid treating ordinary theft as Unmade |
| Q017 The Long Way Home (optional) | Prepare supplies; travel with caravan; clear obstruction; deliver | Partial: conversation native | Escort/region objectives planned; independent of main-story progression |

## Q001 proposed native-only shape

This is **illustrative authoring guidance**, not valid executable JSON because actual stable NPC and semantic interaction IDs have not been verified.

- Quest: Q001; editorial candidate H1-01; runtime ID **unassigned**.
- Giver and turn-in: Elara, stable NPC ID **unassigned**.
- Prerequisites: none; level 1 is a narrative starting band, not a hard eligibility gate.
- Stage 1: three distinct semantic NPC interactions (`ask_about_road`, `ask_about_healing_house`, `ask_about_caravan`), `completion=all`, any order.
- Stage 2: `report_findings` interaction with Elara, `completion=all`.
- Credit: per character, historical trusted interaction receipts; repeat once.
- Reward: XP **TBD**, no invented item/currency grant.
- Failure: committed conversations persist across death/reconnect; no encounter reset. Abandonment behavior **unapproved**.
- Luau: not needed. Semantic interaction IDs are proposals and require Codex/content registry confirmation.

## Explicit content-versus-runtime blockers

1. **NPC/object identity:** editorial aliases such as `anwen` and `medical_parcel` are not installed content IDs.
2. **Objective types:** `object_inspected`, `object_interacted`, `region_discovered`, `region_reached`, `encounter_defended`, `scene_observed`, `dialogue_choice_recorded`, `patrol_completed` and generic `encounter_defeated` cannot be presumed equivalent to native `kill` or `npc_interaction`.
3. **Quest items:** do not assume collecting an object means holding it at turn-in. Each item needs `historical_acquisition` versus `current_possession`, and an explicit consumption policy.
4. **Version migration:** the current exact-byte catalogue binding makes even safe content evolution an explicit compatibility concern; wait for Codex's registry/version migration contract.
5. **Cross-pack prerequisites:** planned registry must resolve IDs across packs; don't manufacture a fixed 32-quest split.
6. **Authorship versus deployment:** no Q001–Q017 definition should be imported until the content validator, actor IDs and runtime authority are ready.

## Next authoring pass

Refine Q018–Q034 using the same evidence-first approach; then reconcile dialogue stage conditions and the first solo dungeon. Update the **single manifest** only after review decisions are accepted; this file records recommendations and does not create a second quest inventory.
