# Human Quest Editorial-ID Reconciliation — Current Review

**Status: all legacy entries triaged; mapping approval outstanding.** This is the active migration review companion to [Human Quest Editorial-ID Crosswalk](Human_Quest_Editorial_ID_Crosswalk_Draft.json) and [Current Human Campaign](CURRENT_HUMAN_CAMPAIGN.md). Canonical story remains [Human 1–80 Campaign](Human_1_80_Canonical_Campaign.md).

## Coverage

- **92 original H-series quests:** 64 exact-title candidates, 28 other proposed dispositions. None automatically inherits a stable runtime ID.
- **24 original HS-series optional quests:** thematic candidates identified, but not proven equivalent. Several distinct HS quests map to one new Q entry, so they may represent separate regional content worth retaining.
- **184 current campaign entries:** editorial Q001–Q184 references; runtime identity and provenance approval remain open.
- **No hard level gates approved by this review.** All prior level bands and `minLevel` data remain planning annotations until a specific runtime gate is intentionally adopted.

## Canon conflicts that must not be resurrected

- **H5-03 — A Name Beyond the Veil:** proposed Q143; Original Nhal naming earlier than current Q143; do not migrate old timing.
- **H5-12 — The Architect's Objection:** proposed Q163; Thes'var early intervention differs from current invitation and final death.
- **H5-14 — Cut the Outer Links:** proposed Q155; Old level-60 finale superseded by level-80 convergence.
- **H5-15 — The Ledger and the Warden:** proposed Q156; Keld/Voss outcome differs from Khor'vun and Veilbreaker.
- **H5-16 — The Severed Chain:** proposed Q181; Old local severance choice cannot replace canonical Nhal shattering.

## Approval rules

1. Compare each legacy source objective, contact, prerequisites and consequence with current quest-stage data. Classify `reused`, `modified`, `split`, `merged`, `retired`, or `new`; record rationale and reviewer approval.
2. If several old quests suggest the same Q entry, **do not** merge editorial IDs. Either explicitly record a multi-source merge or preserve separate regional content.
3. If an old optional quest has a unique standalone purpose, keep it as a candidate for a separate regional quest; do not erase it by mapping to a similar story beat.
4. Allocate immutable runtime IDs only after provenance decisions. Existing `HUMAN_Q###_DONE` flags are temporary design aliases.
5. Validate objective evidence, current inventory possession, consumption at turn-in, NPC IDs, credit scope, rewards and retry rules against the Rust authoring contract.
6. Only then remove the old H/HS register from the active tree. Git history will preserve it, and the approved migration ledger will retain its original IDs and objectives.

## Remaining repository cleanup

- Superseded 1–60 drafts and the JavaScript prototype have already been removed.
- The root H-series register remains **temporarily** until mapping approval. The duplicated Obsidian source copy has already been retired; do not recreate it.
- Earlier playable packs and implementation briefs were retired after consolidation; continue checking any surviving notes for unique mechanics and dialogue before deletion.
- Unrelated engine and world lore documentation is outside this Human campaign cleanup.

## Verified branch provenance — 3 November 2026

GitHub comparison of `main` against `story/launch-main-story-1-60` reports the historical story branch **0 commits ahead and 534 behind** `main` at the time checked. This means no branch-only commits require cherry-picking; it does **not** establish semantic equivalence of all old story documents or authorize deletion of the protected H/HS source register. Retire the branch only after final content/backlink verification and an authorized branch deletion action. The 92 H + 24 HS objective/stage comparisons are documented, but editorial ID mappings remain unapproved.

## Five H5 historical conflict dispositions — 3 November 2026

Rechecked the five conflict-flagged source objectives and `proposed_relation` fields in the 116-entry crosswalk against the current campaign and manifest. These are **historical source dispositions, not approved one-to-one quest migrations**:

| Historical source | Proposed modern thematic reference | Safe editorial disposition |
|---|---|---|
| H5-03 | Q143 | Preserve independent network-record/testimony idea, but **retire the early Nhal-name reveal**; Q143 alone authenticates the name. |
| H5-12 | Q163 | Preserve possible observation-post evacuation/diagram-protection side hook; **do not insert an early Thes'var boss encounter** or alter his Q176 permanent death. |
| H5-14 | Q155 | Preserve worker evacuation and recoverable courier evidence as optional encounter ideas; **do not substitute the old level-60 finale** for the level-70 Veilbreaker or level-80 convergence. |
| H5-15 | Q156 | Preserve safe-retry/defence-support ideas only if independently compatible; **do not replace Khor'vun with Keld/Voss** or change the immediate Veil/Binding transition. |
| H5-16 | Q181 | Preserve bounded severance as historical research context only; **do not offer a branching finale** that bypasses Oth'varen's physical death and Nhal's coherent shattering. |

The original objectives and candidate references remain intact in the crosswalk and root historical register. No new runtime IDs, canonical quest changes, or approval of optional quest additions follows from this classification. The archive remains the destination for distinct future sidequest ideas.
