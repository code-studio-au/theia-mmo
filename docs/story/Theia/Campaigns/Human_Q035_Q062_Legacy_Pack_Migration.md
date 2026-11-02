# Human Q035–Q062 — Legacy Pack and Brief Reconciliation

**Status: Needs Review — source comparison, not executable quest data.** The [canonical Human campaign](Human_1_80_Canonical_Campaign.md) is narrative authority, the [single quest manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the working inventory, and [Codex Quest Engine](../../../QUEST_ENGINE.md) governs runtime support. Read with [Q035–Q062 Rust Authoring Review](Human_Q035_Q062_Rust_Authoring_Review.md).

## Sources compared

- `Human_20_30_Playable_Quest_Pack.md` v0.1: 28 Q035–Q062 entries with locations, player-purpose summaries and generic completion/retry clauses.
- `Human_20_30_Implementation_Brief.md` v0.2: the same 28 entries, broad investigation/interaction stage patterns and build acceptance criteria.

Both sources repeat a 20/25 minimum-level assumption and `HUMAN_Q###_DONE` completion aliases. **Neither constitutes approved hard level gates or permanent runtime identifiers.** The implementation brief's “accept, inspect two sources, report” structure is often generic scaffolding; it must not overwrite the more precise stage targets in the current manifest.

## Retained chapter-specific gameplay and continuity

| Q range | Source-derived player purpose / setting | Current disposition |
|---|---|---|
| Q035–Q037 | Follow Cassian's shipping lead to Tidehold; register inquiry with alliance customs; inspect a falsified requisition | Preserve independent port jurisdiction and record provenance |
| Q038–Q041 | Scheduled travel to Northwatch, rescue supply party, observe contested Dominion coast, infiltrate hostile military depot | Physical travel, civilian safety, no friendly Dominion authority |
| Q042–Q045 | Meet Sera and the Grey Surveyor at Signal House, make a safe sample choice, use Brynja's preliminary note, disable Minor Resonator | Grey Surveyor unnamed; Brynja not personally met until Q049; safe evidence retained whichever Q043 choice |
| Q046–Q048 | Optional ordinary wartime defense and healing observation; main scientific analysis proceeds from authenticated Signal House readings | Neither optional quest is a mandatory prerequisite |
| Q049–Q052 | Kar Durn contract, Brynja's signed drawing versus substituted plan, optional maintenance rescue, grain/iron dispute | Verify whether Q052 trade settlement belongs on main path; do not infer collective Dwarven culpability |
| Q053–Q058 | High King's introduction to Sylvaren, independent Aelari permission, nursery assistance, ward alteration, researcher testimony, Queen's limited disclosure | Distinct sovereign permissions; optional nursery quest must not gate mandatory evidence |
| Q059–Q062 | Return to hostile Dominion coast, recover rejected component and inspection record, intercept courier, compare three independent sources for Korrath lead | Preserve hostile infiltration, safe extraction and evidence copies; don't assume universal infiltration |

## Common implementation and QA details retained

- Classic manual target/facing/range and weapon-timed autoattack; no mandatory PvP, class, profession or group requirement.
- Story dungeons solo-accessible; retries reset active encounters but preserve already verified evidence, persistent choices and authorized permissions.
- Road, ship and fixed flight path travel; no ordinary waypoint teleport. Exceptional Guardian Passage access requires permission and must not be prematurely revealed.
- Quest contacts, locations and objectives are authoring references, not installed NPC, item, spawn or object IDs.
- Validate journal stage order, interaction semantics, duplicate reward prevention, logout/reconnect, travel availability, spawn/leash behavior, dialogue knowledge gates and class-neutral completion.
- The brief's “sample-preservation choice changes follow-up reactions only” is correctly associated with Q043; no mandatory story fork.
- Actual map positions, enemy tuning, item consumption, XP, loot, localization and client/World integration remain unresolved.

## What was not migrated as authoritative

- Repeated generic accept/inspect/report stages where they add no distinct evidence or gameplay.
- Automatic level-20/25 eligibility based solely on chapter bands.
- Assumptions that inspection, travel, defense, object use, choices or escort events are already supported by Codex's initial Rust catalogue.
- Assertions that the 28-quest chapter must be one executable catalogue or fit the current development catalogue limit.

## Cleanup disposition

Both source files contain the same 28 Q references and no additional quest IDs. Their shared rules, regional player purposes and QA requirements are represented in this reconciliation note, the canonical campaign, current manifest and Q035–Q062 authoring review. **Do not delete the old files until inbound references and any remaining unique text are checked.** Git history will retain the originals after deletion.

**Next:** inspect Q063–Q079 legacy pack and implementation brief using the same approach; then reconcile inbound links and remove redundant sources safely.
