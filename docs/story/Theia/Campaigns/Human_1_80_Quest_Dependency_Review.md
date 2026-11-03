# Human 1–80 — Quest Dependency Review (Current)

**Status: WORKING authoring review, not engine implementation.** The sole structured working inventory is [Human 1–80 Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json). The [Canonical Campaign](Human_1_80_Canonical_Campaign.md) remains narrative authority.

## Verified working inventory

- **184** campaign entries, with **684** ordered stage definitions (manifest checked 3 November 2026).
- Provisional classification: **156** main-story, **28** optional regional. This classification requires design approval.
- All prerequisite references resolve, the prerequisite graph is acyclic, and all 684 stage IDs and 684 objective IDs are unique. Every next-stage reference remains within its quest.
- No provisionally main-story quest requires any of the 28 provisionally optional quests. Q084 depends on Q082, not optional Q083; Q094 depends on Q092, not optional Q093.
- All proposed/candidate Q references in the 92 H-series and 24 HS-series crosswalk rows resolve. This verifies references only, not creative equivalence or mapping approval.
- Prior `minLevel` fields were migrated as **planning level annotations**; no runtime level gate is inferred.
- Stage target aliases and NPC names are not stable engine IDs. The manifest is not executable Rust content.

## Review gates before runtime conversion

1. **OPEN:** validate semantic dependency order against the canonical story. Structural checks above do not prove revelation, permission or world-phase timing.
2. **WORKING:** optional access conditions remain `authoring_access_review` proposals, not executable prerequisites. Preserve Q083's independence from Q084–Q086 and Q093's independence from Q094.
3. **OPEN:** decide the proposed access/phase conditions before runtime conversion; do not silently activate them or treat planning level bands as eligibility.
4. Preserve Q029's independent civilian/evidence checkpoint before Q030's Cassian encounter.
5. Preserve the distinction between Q128's early expedition staging and Q154's final assault preparation. Any further rename remains a proposal.
6. Preserve Q156's irreversible Veil destruction and immediate Binding dissolution; Q181's fatal Oth'varen correction occurs only after Thes'var's death and sabotage.
7. Author historical versus current-inventory objectives, consumption, reward transaction, failure/retry, and character/party/instance credit semantics under the [Rust Quest Authoring Contract](Human_Rust_Quest_Authoring_Contract.md).

## Implementation coordination

The Rust/Bevy quest implementation and its existing verified objective foundation are authoritative for actual capabilities. Do not assume general acceptance, branching, party sharing, phasing, exploration objectives or Luau are implemented. The repository's [Quest Engine](../../../QUEST_ENGINE.md) agreement must be reviewed before adopting or changing executable schemas. Its current qualified development limits are not permanent campaign limits.

**Do not recreate the deleted chapter JSON files or the older dependency JSON.** Work from the single manifest and the canonical campaign.
