# Human Production Specification — Retirement Audit

**Status: historical retirement audit — production specification already retired from active `main`.** The former 175 KB `Human_1_80_Production_Quest_Specification.md` contained **184 distinct Q001–Q184 sections** and is a competing editorial register alongside the [single current authoring manifest](Human_1_80_Current_Quest_Authoring_Manifest.json).

## Findings

- All 184 Q references are represented in the sole manifest; no extra Q identifiers appear in the production specification.
- Each old entry contains a location band, primary contact, player objective, broad lore purpose, a largely generic quest flow, old completion alias and generic failure rules.
- **Conflict:** the old specification treats chapter minimum levels and previous main-story completion as generic prerequisites. These are not approved hard eligibility rules. Its repeated accept/travel/inspect/report template is not a validated objective sequence.
- At the time of this audit, the old file's opening links had been redirected. The production specification was subsequently retired, and its location/lore editorial context was preserved in [Location and Lore Reference](Human_1_80_Location_and_Lore_Editorial_Reference.md).
- The current manifest contains more specific stage/objective target aliases and explicit blockers; the canonical campaign controls story outcomes.

## Historical retirement requirements (completed for the old production register; runtime validation remains separate)

1. Verify the 184 old per-quest locations, contacts, player-objective summaries and lore purposes against the manifest and canonical campaign. Preserve any unique, still-valid detail as reviewed metadata or in the canonical narrative.
2. Confirm no active documentation links to the old specification or relies on its old level-gate assertions.
3. **Completed:** the redundant production specification was removed from the active tree; Git history preserves it. This does not approve its former level gates or generic stages.
4. Reconcile the original H/HS register separately. The current crosswalk preserves 92 H-series and 24 HS-series entries, but proposed mappings are **not approved**.
5. **Still outstanding:** retire or fold temporary migration/review notes into a concise approved authoring guide only once their unique decisions are reflected in the manifest.

**Current active authorities:** [Current Human Campaign](CURRENT_HUMAN_CAMPAIGN.md), [Canonical Campaign](Human_1_80_Canonical_Campaign.md), [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json), [Rust Authoring Contract](Human_Rust_Quest_Authoring_Contract.md), and [Codex Quest Engine](../../../QUEST_ENGINE.md).

**No Rust engine changes are proposed.**
