> **LATEST CREATOR OVERRIDE (12 October 2026):** Nhal could observe the Physical Realm from behind the Veil but could not influence it until the **First Resonance** accidentally produced by later Kel'vari researchers. **Ascendant created the Kel'vari entirely independently**, without Nhal altering the design. This explicitly supersedes the original Q40 interpretation of corruption during creation. The original answer remains in the source archive as historical provenance; the Interwoven Chronicle and World Bible contain the governing story.

> **LATEST LORE RECONCILIATION (12 October 2026):** The World Bible, Detailed Lore Archive, Interwoven Chronicle, Source Decision Recovery and Canon Reconciliation Report have been updated against the explicit decisions in the present conversation. Ascendant's **sole** creation of the Kel'vari, the First Anchor-triggered Second Sundering, Divine War, Keeper's knowing deception in **The Binding**, the Guardians' multiracial founding, the surviving Kel'vari enclaves, continuous low-capacity gates and the modern Anchor rules are now aligned. The [Canon Reconciliation Report](THEIA_CANON_RECONCILIATION_REPORT.md) records the review and remaining open decisions. This is **not** a new independent word-by-word re-audit of the original 403-page PDF.

> **SOURCE REVIEW COMPLETE (12 October 2026):** Read [THEIA_CANON_RECONCILIATION_REPORT.md](THEIA_CANON_RECONCILIATION_REPORT.md) for the 403-page source-index methodology, 471 separately indexed question prompt occurrences, 418 source-located numbered identifiers, 26 deliberately open/deferred identifiers, and corrections propagated into the current World Bible and chronicle. The complete original-question evidence register was generated in an earlier ChatGPT workspace but is **not committed to GitHub**; regenerate or upload it before treating the repository as a complete source archive.

# Theia documentation — historical source guide and current reading order

> **Current campaign authority (3 November 2026):** For the Human launch story use [Current Human Campaign](Theia/Campaigns/CURRENT_HUMAN_CAMPAIGN.md), [Canonical Human 1–80 Campaign](Theia/Campaigns/Human_1_80_Canonical_Campaign.md), and the [single working quest manifest](Theia/Campaigns/Human_1_80_Current_Quest_Authoring_Manifest.json). This document retains older import-era reading guidance as historical context, not as a competing 1–60 campaign authority. The original H/HS register is provenance only; its 92 H and 24 HS editorial IDs are not approved runtime mappings.


## Historical source documents (after the original 403-page conversation export was supplied)

1. **[THEIA_WORLD_BIBLE.md](THEIA_WORLD_BIBLE.md)** — short canonical reference. Where a user-confirmed decision conflicts with an assistant proposal, the decision wins.
2. **[THEIA_INTERWOVEN_CHRONICLE.md](THEIA_INTERWOVEN_CHRONICLE.md)** — current creator-facing chronological narrative, expanded with divine relationships, the vanished Kel'vari, Keeper/Ascendant conflict, Nhal, ancient institutions, Second Sundering, racial migrations, the Guardians and Unmade, Shattered Delegation, modern leaders and overseas incursions. It contains proposed connective history that is NOT automatically ratified.
3. **[THEIA_DETAILED_LORE_ARCHIVE.md](THEIA_DETAILED_LORE_ARCHIVE.md)** — supporting worldbuilding archive, cultural systems, world regions, institutions and late-stage question answers.
4. **[THEIA_SOURCE_DECISION_RECOVERY.md](THEIA_SOURCE_DECISION_RECOVERY.md)** — source-audit summary for original Q1–435, including corrections and later renames.

The earlier duplicate **THEIA_TRUE_CHRONICLE.md** was removed from the current branch to avoid two competing primary narratives. Its content remains recoverable from Git history.

## Original primary source

**Create MMO Story.pdf** — 403-page export of the original conversation, provided in ChatGPT on 12 October 2026. This PDF is the ultimate evidence for user decisions. It is **not yet stored in this repository**. The source contains rejected alternate story proposals before the collaborative Stage 1 questions begin around page 41; these rejected concepts must never be silently imported as canon.

To generate the full question-prompt index locally after placing the PDF in your checkout:

```bash
python tools/extract_theia_questions.py "Create MMO Story.pdf"
```

This produces `docs/THEIA_ORIGINAL_QUESTIONS_Q001-Q444.md`. The generated file records original question wording and options but **does not automatically infer which answer the user selected**. The extraction script is committed; the full generated 444-question file and original PDF are not yet committed.

## Editorial guardrails

Use the [repository authority hierarchy and six canon classifications](README.md#approval-and-precedence).
Historical CONFIRMED / USER LOCKED labels require creator evidence; Needs Review
means WORKING. Do not treat import-era review status as a downgrade of the later
approved Human 1–80 campaign, or as approval of new connective history.

The question numbers were reused around Q297–317, so identify questions by stage
and source page as well as number. Stage 62 Q436–444 received no submitted answers.
Unanswered historical questions stay OPEN unless a later explicit decision resolves
them. In particular, the Seven's survival and Oth'varen's physical death at the
level-80 finale are settled by the [current creator lock](DECISIONS.md); old
statements that their launch fates remain open are SUPERSEDED.

## Remaining archival work

The recovered decision file is a *summary*, not a complete verified answer register. The remaining high-effort task is to compare every original Q1–435 prompt with the **following user response and all later corrections**, record exact source pages and status, then reconcile each confirmed consequence into the World Bible and narrative. This must not be described as finished until the audit is actually performed.

The story's long-form draft is deliberately separate from the authoritative decision ledger so it can become increasingly novel-like without erasing provenance.

## Human launch campaign — current and historical scope

Start with [Current Human Campaign](Theia/Campaigns/CURRENT_HUMAN_CAMPAIGN.md),
then the [canonical 1–80 narrative](Theia/Campaigns/Human_1_80_Canonical_Campaign.md)
and [single working quest manifest](Theia/Campaigns/Human_1_80_Current_Quest_Authoring_Manifest.json).
The narrative is approved; the manifest is planning material, not executable
Rust content. Detailed dialogue and encounter design retain their stated review
boundaries. The [current encounter design](Theia/Campaigns/Human_1_80_Encounter_Barks_and_Cinematic_Timing.md)
does not acquire blanket approval from the narrative.

The creator's 15 October instruction to complete a packet marked Needs Review
applies to the historical 1–60 proposals, not to the later approved 1–80 baseline:

- [Historical Human Quest Register](THEIA_HUMAN_QUEST_REGISTER.md): 92 H and
  24 HS editorial source IDs, not current order, runtime IDs or approved mappings.
- [Cast, Places and Services](THEIA_HUMAN_CAST_AND_PLACES.md): historical contacts,
  working geography and services, with approval boundaries retained.
- [Legacy Ideas Archive](Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md):
  preserved connected-racial, sidequest and encounter proposals, not added Human quests.
- The retired Human 1–60 Campaign Review Record remains in Git history at
  `docs/story/THEIA_HUMAN_CAMPAIGN_REVIEW.md`.

The approved [Human Act I](THEIA_HUMAN_ACT_I_STORY.md) and
[Heartlands framework](THEIA_HUMAN_HEARTLANDS_SETTLEMENT_REGISTER.md) retain their
approved scope. The [Modern Timeline](THEIA_MODERN_STORY_REVELATION_TIMELINE.md)
contains dated historical locks; superseded five-act bands and launch outcomes
must defer to the current canonical campaign. New dialogue, locations and
encounter mechanics remain WORKING unless explicitly approved. Cassian's final
sentence and other unresolved details must not be inferred from historical drafts.
