> **LATEST CREATOR OVERRIDE (12 October 2026):** [[Nhal]] could observe the Physical Realm from behind the Veil but could not influence it until the **[[First Resonance]]** accidentally produced by later Kel'vari researchers. **Ascendant created the Kel'vari entirely independently**, without Nhal altering the design. This explicitly supersedes the original Q40 interpretation of corruption during creation. The original answer remains in the source archive as historical provenance; the Interwoven Chronicle and World Bible contain the governing story.

> **LATEST LORE RECONCILIATION (12 October 2026):** The World Bible, Detailed Lore Archive, Interwoven Chronicle, Source Decision Recovery and Canon Reconciliation Report have been updated against the explicit decisions in the present conversation. Ascendant's **sole** creation of the Kel'vari, the First Anchor-triggered [[Second Sundering]], [[Divine War]], Keeper's knowing deception in **[[The Binding]]**, the Guardians' multiracial founding, the surviving Kel'vari enclaves, continuous low-capacity gates and the modern Anchor rules are now aligned. The [Canon Reconciliation Report](THEIA_CANON_RECONCILIATION_REPORT.md) records the review and remaining open decisions. This is **not** a new independent word-by-word re-audit of the original 403-page PDF.

> **SOURCE REVIEW COMPLETE (12 October 2026):** Read [THEIA_CANON_RECONCILIATION_REPORT.md](THEIA_CANON_RECONCILIATION_REPORT.md) for the 403-page source-index methodology, 471 separately indexed question prompt occurrences, 418 source-located numbered identifiers, 26 deliberately open/deferred identifiers, and corrections propagated into the current World Bible and chronicle. The complete original-question evidence register was generated in an earlier ChatGPT workspace but is **not committed to GitHub**; regenerate or upload it before treating the repository as a complete source archive.

# Theia documentation — source of truth and reading order

## Current documents (after the original 403-page conversation export was supplied)

1. **[THEIA_WORLD_BIBLE.md](THEIA_WORLD_BIBLE.md)** — short canonical reference. Where a user-confirmed decision conflicts with an assistant proposal, the decision wins.
2. **[THEIA_INTERWOVEN_CHRONICLE.md](THEIA_INTERWOVEN_CHRONICLE.md)** — current creator-facing chronological narrative, expanded with divine relationships, the vanished Kel'vari, Keeper/Ascendant conflict, Nhal, ancient institutions, Second Sundering, racial migrations, the [[The Guardians|Guardians]] and [[The Unmade|Unmade]], [[Shattered Delegation]], modern leaders and overseas incursions. It contains proposed connective history that is NOT automatically ratified.
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

- **CONFIRMED** — explicitly selected by the user, including later corrections and names.
- **WORKING** — coherent narrative proposal; may be adopted, revised or rejected.
- **OPEN** — no final decision.
- **SUPERSEDED** — old name, earlier draft or previous selection replaced by later user instruction.

The question numbers were reused around Q297–317, so identify questions by stage and source page as well as number. Stage 62 Q436–444 received no submitted answers. The creator narrative's detailed Second Sundering machinery, original surviving great order identity and fate of individual gods are still working/open.

## Remaining archival work

The recovered decision file is a *summary*, not a complete verified answer register. The remaining high-effort task is to compare every original Q1–435 prompt with the **following user response and all later corrections**, record exact source pages and status, then reconcile each confirmed consequence into the World Bible and narrative. This must not be described as finished until the audit is actually performed.

The story's long-form draft is deliberately separate from the authoritative decision ledger so it can become increasingly novel-like without erasing provenance.

## Human launch campaign — Needs Review (15 October 2026)

The creator authorised completing the human story and necessary connected racial material autonomously, with new work marked **Needs Review**. The following proposed packet is not a new canon lock:

- [Complete Human Campaign](THEIA_HUMAN_CAMPAIGN.md) — five acts, levels 1–60, concluding the human launch narrative.
- [Human Quest Register](THEIA_HUMAN_QUEST_REGISTER.md) — 92 principal quests, 24 optional local quests, dependencies and choices.
- [Solo Story Climaxes](THEIA_HUMAN_STORY_DUNGEONS.md) — all five essential story encounters without required raids or PvP.
- [Cast, Places and Services](THEIA_HUMAN_CAST_AND_PLACES.md) — recurring humans, new contacts, working geography and stable services.
- [Connected Racial Stories](THEIA_HUMAN_CONNECTED_RACIAL_STORIES.md) — allied visits, [[Dominion]] counterpart motivations and 12 proposed native-side companion quests.
- [Campaign Review Record](THEIA_HUMAN_CAMPAIGN_REVIEW.md) — source precedence, continuity checks, proposed decisions and implementation limits.

Read the approved [Human Act I](THEIA_HUMAN_ACT_I_STORY.md) and [Heartlands framework](THEIA_HUMAN_HEARTLANDS_SETTLEMENT_REGISTER.md) alongside the packet. Latest dated creator locks in the [Modern Timeline](THEIA_MODERN_STORY_REVELATION_TIMELINE.md) establish the five-act bands and first Shroud expedition around level 40; these supersede older six-band and later-expansion proposals. New dialogue, locations, quest staging, Cassian's final sentence and inversion outcomes remain **Needs Review**, even where the draft provides a complete proposed outcome.
