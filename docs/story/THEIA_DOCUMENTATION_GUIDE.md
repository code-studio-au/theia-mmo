> **Historical creator override (12 October 2026):** Nhal could observe the Physical Realm from behind the Veil but could not influence it until the **First Resonance** accidentally produced by later Kel'vari researchers. **Ascendant created the Kel'vari entirely independently**, without Nhal altering the design. This explicitly supersedes the original Q40 interpretation of corruption during creation. The original answer remains in the source archive as historical provenance; the Interwoven Chronicle and World Bible contain the governing story.

> **Source evidence and limits:** The [consolidated recovery index](THEIA_SOURCE_DECISION_RECOVERY.md#historical-audit-evidence-and-limits) preserves the 12 October historical audit methodology, 444 identifiers / 471 prompt occurrences, 418 source-located identifiers, 26 open/deferred identifiers, page locators and later correction context. It distinguishes source indexing from exhaustive manual adjudication. The original PDF and full evidence artifacts were not committed; this consolidation does not claim a fresh PDF audit or current access to those earlier workspace files. Use the World Bible and later creator decisions for current canon.

# Theia documentation — historical source guide and current reading order

> **Current campaign authority (3 November 2026):** For the Human launch story use [Current Human Campaign](Theia/Campaigns/CURRENT_HUMAN_CAMPAIGN.md), [Canonical Human 1–80 Campaign](Theia/Campaigns/Human_1_80_Canonical_Campaign.md), and the [single working quest manifest](Theia/Campaigns/Human_1_80_Current_Quest_Authoring_Manifest.json). This document retains older import-era reading guidance as historical context, not as a competing 1–60 campaign authority. The original H/HS register is provenance only; its 92 H and 24 HS editorial IDs are not approved runtime mappings.


## Historical source documents (after the original 403-page conversation export was supplied)

1. **[THEIA_WORLD_BIBLE.md](THEIA_WORLD_BIBLE.md)** — short canonical reference. Where a user-confirmed decision conflicts with an assistant proposal, the decision wins.
2. **[THEIA_INTERWOVEN_CHRONICLE.md](THEIA_INTERWOVEN_CHRONICLE.md)** — current creator-facing chronological narrative, expanded with divine relationships, the vanished Kel'vari, Keeper/Ascendant conflict, Nhal, ancient institutions, Second Sundering, racial migrations, the Guardians and Unmade, Shattered Delegation, modern leaders and overseas incursions. It contains proposed connective history that is NOT automatically ratified.
3. **[THEIA_SOURCE_DECISION_RECOVERY.md](THEIA_SOURCE_DECISION_RECOVERY.md)** — consolidated historical question/page evidence, including Q376–435 selection detail, unanswered Q436–444 prompts, corrections and later renames.
4. **[Legacy Ideas Archive](Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md)** — distinct unapproved world, institution, character and campaign possibilities preserved from retired secondary compilations.

The earlier duplicate **THEIA_TRUE_CHRONICLE.md** was removed from the current branch to avoid two competing primary narratives. Its content remains recoverable from Git history.

## Original primary source

**Create MMO Story.pdf** — 403-page export of the original conversation, provided in ChatGPT on 12 October 2026. This PDF is primary evidence for the historical decisions it contains; later explicit creator decisions still take precedence. It is **not yet stored in this repository**. The source contains rejected alternate story proposals before the collaborative Stage 1 questions begin around page 41; these rejected concepts must never be silently imported as canon.

The historical extraction workflow belongs to the source Engine repository, not this story checkout. The [source index's extraction provenance](THEIA_SOURCE_DECISION_RECOVERY.md#historical-audit-evidence-and-limits) links its immutable source; the [import provenance](README.md#import-provenance-and-limits) records that the tool was not migrated. The old command produced `docs/THEIA_ORIGINAL_QUESTIONS_Q001-Q444.md`, a prompt/options index rather than inferred user answers. Neither that full generated artifact nor the original PDF is committed here. No extractor or code inspection is required or claimed by this Markdown consolidation.

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

**Evidence limit, not a direction to repeat completed reviews:** The recovered source index is not a certified verbatim answer register. A future exhaustive audit would require the original PDF, each prompt's following user response and all later corrections, with exact page/status evidence. Those artifacts are currently unavailable in this repository; no such new audit is claimed.

Use the recorded evidence and later explicit decisions to complete safe consolidation now. Raise a specific missing-source blocker only when it prevents resolving a material canon question. Preserve uncertain material as WORKING/OPEN rather than inventing approval. The long-form Chronicle remains deliberately separate from the evidence index so its approved narrative can survive without erasing provenance.

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
- [Retained cast, geography and service context](Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md#retired-cast-geography-and-service-packet): historical proposal context, with individual character/place cards preserving their approval boundaries.
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


## Retained document roles and disposition

**KEEP by purpose, with approval scope preserved.** These roles explain why the remaining documents are useful rather than accidental duplicate canon homes. KEEP is not blanket content approval or a final integrity certification; unfinished checks remain in the temporary tracker. A later specific duplication finding can still justify a bounded consolidation.

| Existing home | Distinct role and boundary |
|---|---|
| [Story README](README.md), [decision ledger](DECISIONS.md), this guide | Hierarchy/classification, dated decisions and navigation/evidence limits respectively; do not create another canon ledger. |
| [World Bible](THEIA_WORLD_BIBLE.md) | Established world facts and their intentionally OPEN limits. |
| [Interwoven Chronicle](THEIA_INTERWOVEN_CHRONICLE.md) | Historical narrative, with proposed connective passages subordinate to creator decisions. |
| [Source Decision Recovery](THEIA_SOURCE_DECISION_RECOVERY.md) | Question/page evidence, later corrections and source verification limits, not a second current world narrative. |
| [Character Arcs](THEIA_CHARACTER_ARCS.md), [Modern Revelation Timeline](THEIA_MODERN_STORY_REVELATION_TIMELINE.md) | Approved character histories and scoped disclosure/court foundations; current campaign owns exact quest placement and outcomes. |
| [Memory Scenes](THEIA_MEMORY_SCENES.md) | Separately approved full dramatic scenes, not replaceable by the Chronicle's shorter account; delivery/discovery mechanisms retain their own status. |
| [Human Act I foundation](THEIA_HUMAN_ACT_I_STORY.md), [Heartlands settlement reference](THEIA_HUMAN_HEARTLANDS_SETTLEMENT_REGISTER.md) | Unique approved early narrative, settlement/service context and explicit unresolved details; historical act bands do not override current progression. |
| [Zone Population Framework](THEIA_ZONE_POPULATION_FRAMEWORK.md) | Working authoring checks, encounter/population examples and independently playable regional life; specific creatures, distributions and levels are not locked. |
| [Canonical Human campaign](Theia/Campaigns/Human_1_80_Canonical_Campaign.md), [Current Human Campaign](Theia/Campaigns/CURRENT_HUMAN_CAMPAIGN.md) | Approved narrative and its reading/status entry point. The linked manifest is the sole working quest inventory, not executable Rust. |
| [Quest authoring contract](Theia/Campaigns/Human_Rust_Quest_Authoring_Contract.md) and eight range authoring reviews | Documentation of narrative-to-authoring constraints, source rows, QA and unresolved semantics; neither review tables nor the contract create a second approved campaign or runtime mapping. |
| [Key scenes](Theia/Campaigns/Human_1_80_Key_Scenes_Branching_Dialogue.md), range dialogue and [encounter design](Theia/Campaigns/Human_1_80_Encounter_Barks_and_Cinematic_Timing.md) | Supporting speech, choices, barks and staging within their own approvals; canonical quest outcomes do not automatically approve every presentation detail. |
| [Character index](Theia/Characters/Characters.md), [Atlas](Theia/Atlas/Atlas.md), [Lore index](Theia/Lore/Lore%20and%20World.md), [Historical Timeline](Theia/History/Historical%20Timeline.md) and their individual notes | Subject navigation and scoped references, not competing histories. Source-specific cards retain original provenance and must defer to current authority for later events. |
| [Historical H/HS register](THEIA_HUMAN_QUEST_REGISTER.md), [exact-title comparison](Theia/Campaigns/Human_Legacy_64_Exact_Title_Objective_Comparison.md) | ARCHIVED source evidence and optional future-reuse comparison. All 92 H and 24 HS concepts are archived; no runtime mapping is approved. |
| [Legacy Ideas Archive](Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md) | One home for unique unapproved regional, racial, character, encounter and future possibilities. Nothing becomes canon through preservation. |

Other implementation-oriented Markdown records keep their existing planning/evidence purpose; this guide does not validate code, architecture, schemas or runtime behavior. The original import manifest remains immutable provenance, not a current file inventory.

For individual notes, `source-specific` is a provenance description, not a seventh canon classification or blanket approval. Read each claim under its stated source scope and the six classifications in the README; historical `Needs Review` means WORKING. Preserve legitimate OPEN creative decisions without reopening later-confirmed facts.
