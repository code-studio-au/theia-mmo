# Theia story, lore and campaign

This folder is the home for Theia's worldbuilding, characters, acts, quests and
story decisions in the active Theia-MMO project, adopted by the user on
2026-10-26. These are narrative design documents; they do not establish working
quests, NPCs, combat, persistence or approved engineering requirements.

## Current authority — read this first

The historical import table below records source provenance and is **not** the current campaign reading order. For active Human story work, use [Current Human Campaign](Theia/Campaigns/CURRENT_HUMAN_CAMPAIGN.md), then [Canonical 1–80 Campaign](Theia/Campaigns/Human_1_80_Canonical_Campaign.md) and the [184-quest authoring manifest](Theia/Campaigns/Human_1_80_Current_Quest_Authoring_Manifest.json). The manifest is editorial planning, not executable Rust quest data. Unique unused historical hooks belong in the [legacy ideas archive](Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md). [Story decisions](DECISIONS.md) govern precedence. The [temporary cleanup tracker](STORY_CLEANUP_TRACKER.md) is noncanonical and must be removed after final verification.

The older 1–60 campaign, quest register and imported review materials below are historical references only. Their old quest counts, progression and finale do not override the approved 1–80 campaign. Historical source documents may still contain unreconciled details; do not delete them without comparison and link verification.

## Historical import index (not current campaign authority)

| Subject | Documents |
|---|---|
| Reading order and canon rules | [Imported documentation guide](THEIA_DOCUMENTATION_GUIDE.md), [local decisions and maintenance](DECISIONS.md) |
| World and lore | [World Bible](THEIA_WORLD_BIBLE.md), [Interwoven Chronicle](THEIA_INTERWOVEN_CHRONICLE.md) |
| Characters | [Character Arcs](THEIA_CHARACTER_ARCS.md), [Campaign character cards](Theia/Characters/Characters.md) |
| Campaign acts and revelation | [Modern Story Revelation Timeline](THEIA_MODERN_STORY_REVELATION_TIMELINE.md), [Human Act I](THEIA_HUMAN_ACT_I_STORY.md), [Canonical Human 1–80 Campaign](Theia/Campaigns/Human_1_80_Canonical_Campaign.md) |
| Quests and encounters | [Human Quest Register](THEIA_HUMAN_QUEST_REGISTER.md), [Current Encounter Design](Theia/Campaigns/Human_1_80_Encounter_Barks_and_Cinematic_Timing.md), [Connected Racial Stories](Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md) |
| Settlements and world population | [Human Heartlands Settlement Register](THEIA_HUMAN_HEARTLANDS_SETTLEMENT_REGISTER.md), [Zone Population Framework](THEIA_ZONE_POPULATION_FRAMEWORK.md) |
| Story scenes | [Memory Scenes](THEIA_MEMORY_SCENES.md) |
| Decisions and source recovery | [Source Decision Recovery and audit evidence](THEIA_SOURCE_DECISION_RECOVERY.md) |
| Campaign review | [Current Human Campaign](Theia/Campaigns/CURRENT_HUMAN_CAMPAIGN.md) (historical review retired; see Git history) |

## Approval and precedence

This is the repository-wide story authority hierarchy, confirmed by the creator
on 3 November 2026. Authority is scoped by subject; narrative authority does not
approve executable quest data or engineering changes.

1. **Creator-confirmed decisions** have highest authority. Later explicit decisions
   override earlier ones; [Story decisions](DECISIONS.md) records the current locks,
   supersession and evidence. An assistant proposal or older summary is not approval.
2. **[World Bible](THEIA_WORLD_BIBLE.md)** is the reference for established world
   facts, divine history, races, magic and institutions.
3. **[Interwoven Chronicle](THEIA_INTERWOVEN_CHRONICLE.md)** is the historical
   narrative, subject to creator decisions. Proposed connective material is not
   automatically locked canon.
4. **[Canonical Human 1–80 Campaign](Theia/Campaigns/Human_1_80_Canonical_Campaign.md)**
   governs approved Human launch progression, revelations, deaths and ending.
5. **[Quest Authoring Manifest](Theia/Campaigns/Human_1_80_Current_Quest_Authoring_Manifest.json)**
   is the single working inventory of 184 Human quest concepts. It must conform to
   the canonical narrative and is not executable Rust content.
6. **Supporting character, location, dialogue and encounter documents** are
   authoritative only within their explicitly approved scope.
7. **[Legacy Ideas Archive](Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md)**
   preserves unapproved historical and future possibilities, not active canon.

### Canon classifications

- **LOCKED:** explicitly confirmed by the creator.
- **CANONICAL:** established by the current approved narrative, subject to later
  creator decisions. This does not imply every supporting implementation detail is locked.
- **WORKING:** a proposal being developed, not approved canon.
- **OPEN:** requires a decision; preserve the exact unresolved question.
- **SUPERSEDED:** replaced by a later decision; retain provenance, not authority.
- **ARCHIVED:** preserved for possible future use, not active canon.

Historical `CONFIRMED` and `USER LOCKED` labels mean LOCKED only where explicit
creator approval supports their stated scope. `Needs Review` denotes WORKING
material, not a conflicting approval class. Do not relabel an entire mixed-status
document as locked merely because one section contains a creator decision.

Human Act I's approved narrative foundation remains preserved. The historical
1–60 packet's 92 H-series, 24 HS-series and 12 native-side proposals were not
approved by import; they do not downgrade or replace the approved 1–80 campaign.
H/HS identifiers remain provenance, not approved runtime mappings. Keep world
truth separate from player knowledge and revelation timing. Record a fact once
in its authoritative home and link supporting documents to it.

Open questions belong with their relevant authoritative source, including the
[World Bible](THEIA_WORLD_BIBLE.md), not a second canon ledger. Interpret historical
open-question lists under later creator locks; a resolved question does not become
OPEN again merely because an older record still lists it.

## Import provenance and limits

The 19 documents were recovered from the locally available Engine `origin/main`
commit `94cc79228bc90db13d83116f76f6cdba51b72630`, which merged the campaign branch
at `d89257a9ecb270a18542eed2f9f31acef0b65a3c`. They were absent from the Engine's
checked-out rendering benchmark branch. This is a transfer from those local Git
objects; remote freshness was not checked.

[IMPORT_MANIFEST.json](IMPORT_MANIFEST.json) records every original path and both
source and destination SHA-256 hashes. At import, eighteen documents were byte-identical;
the then-imported reconciliation report's extractor link was adjusted to the source commit. Later consolidation and retirement do not rewrite that transfer baseline.
The Engine checkout and history were left unchanged.

The original 403-page story PDF and full generated question/answer audit registers
are not in this source snapshot. Historical claims about their examination have
not been independently reverified during this import. The historical extraction workflow referenced by
the guide belongs to the Engine's tools; that tool was not migrated.
Private conversation/account exports were not copied.

Historical headings referring to a custom MMO engine are retained as provenance.
The active game uses the [MMO engineering architecture](../engineering/README.md).
Narrative decisions do not silently override that architecture or create gameplay
implementation commitments.

