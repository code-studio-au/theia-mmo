# Human 1–80 — Quest Dependency Review (Current)

**Status: authoring review, not engine implementation.** The sole structured working inventory is [Human 1–80 Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json). The [Canonical Campaign](Human_1_80_Canonical_Campaign.md) remains narrative authority.

## Verified working inventory

- **184** campaign entries, with **683** ordered stage definitions.
- Provisional classification: **156** main-story, **28** optional regional. This classification requires design approval.
- All prerequisite references in the current manifest resolve to existing Q references.
- Prior `minLevel` fields were migrated as **planning level annotations**; no runtime level gate is inferred.
- Stage target aliases and NPC names are not stable engine IDs. The manifest is not executable Rust content.

## Review gates before runtime conversion

1. Validate every main-story dependency against the canonical story and ensure no optional quest is required.
2. Check Q083's optional Passage survey does not gate Q084–Q086, including mandatory safe-return validation and the first living Kel'vari revelation.
3. Check Q093's optional archive service does not gate Q094's mandatory evidence.
4. Preserve Q029's independent civilian/evidence checkpoint before Q030's Cassian encounter.
5. Differentiate Q128's early expedition staging from Q154's final assault preparation; proposed rename is not yet applied.
6. Preserve Q156's irreversible Veil destruction and immediate Binding dissolution; Q181's fatal Oth'varen correction occurs only after Thes'var's death and sabotage.
7. Author historical versus current-inventory objectives, consumption, reward transaction, failure/retry, and character/party/instance credit semantics under the [Rust Quest Authoring Contract](Human_Rust_Quest_Authoring_Contract.md).

## Implementation coordination

The Rust/Bevy quest implementation and its existing verified objective foundation are authoritative for actual capabilities. Do not assume general acceptance, branching, party sharing, phasing, exploration objectives or Luau are implemented. Codex's local `docs/QUEST_ENGINE.md` agreement must be reviewed before adopting or changing executable schemas; it was not available in the GitHub branch at the time of this update.

**Do not recreate the deleted chapter JSON files or the older dependency JSON.** Work from the single manifest and the canonical campaign.
