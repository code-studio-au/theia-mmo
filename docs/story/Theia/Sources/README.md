# Theia story, lore and campaign

This folder is the home for Theia's worldbuilding, characters, acts, quests and
story decisions in the active Theia-MMO project, adopted by the user on
2026-10-26. These are narrative design documents; they do not establish working
quests, NPCs, combat, persistence or approved engineering requirements.

## Start here

| Subject | Documents |
|---|---|
| Reading order and canon rules | [Imported documentation guide](THEIA_DOCUMENTATION_GUIDE.md), [local decisions and maintenance](DECISIONS.md) |
| World and lore | [World Bible](THEIA_WORLD_BIBLE.md), [Detailed Lore Archive](THEIA_DETAILED_LORE_ARCHIVE.md), [Interwoven Chronicle](THEIA_INTERWOVEN_CHRONICLE.md) |
| [[Characters]] | [Character Arcs](THEIA_CHARACTER_ARCS.md), [Human Cast and Places](THEIA_HUMAN_CAST_AND_PLACES.md) |
| Campaign acts and revelation | [Modern Story Revelation Timeline](THEIA_MODERN_STORY_REVELATION_TIMELINE.md), [Human Act I](THEIA_HUMAN_ACT_I_STORY.md), [Complete Human Campaign](THEIA_HUMAN_CAMPAIGN.md) |
| Quests and encounters | [Human Quest Register](THEIA_HUMAN_QUEST_REGISTER.md), [Human Story Dungeons](THEIA_HUMAN_STORY_DUNGEONS.md), [Connected Racial Stories](THEIA_HUMAN_CONNECTED_RACIAL_STORIES.md) |
| Settlements and world population | [Human Heartlands Settlement Register](THEIA_HUMAN_HEARTLANDS_SETTLEMENT_REGISTER.md), [Zone Population Framework](THEIA_ZONE_POPULATION_FRAMEWORK.md) |
| Story scenes | [Memory Scenes](THEIA_MEMORY_SCENES.md) |
| Decisions and source recovery | [Source Decision Recovery](THEIA_SOURCE_DECISION_RECOVERY.md), [Canon Reconciliation Report](THEIA_CANON_RECONCILIATION_REPORT.md), [Line-by-Line Verification Status](THEIA_LINE_BY_LINE_VERIFICATION_STATUS.md) |
| Campaign review | [Human Campaign Review](THEIA_HUMAN_CAMPAIGN_REVIEW.md) |

## Approval and precedence

Preserve each document's CONFIRMED, USER LOCKED, WORKING, OPEN, SUPERSEDED and
Needs Review labels. A later explicit creator decision takes precedence over an
older proposal or summary. The imported guide explains the historical source
hierarchy; consult the dated decisions in the relevant documents together.

Human Act I is explicitly creator-approved. The subsequent complete human
campaign packet remains **Needs Review**, including its proposed 92 principal
quests, 24 optional local quests and 12 native-side companion quests. Importing
these files does not approve that packet. The Chronicle also contains proposed
connective history; its narrative form does not confer canon status.

## Import provenance and limits

The 19 documents were recovered from the locally available Engine `origin/main`
commit `94cc79228bc90db13d83116f76f6cdba51b72630`, which merged the campaign branch
at `d89257a9ecb270a18542eed2f9f31acef0b65a3c`. They were absent from the Engine's
checked-out rendering benchmark branch. This is a transfer from those local Git
objects; remote freshness was not checked.

[IMPORT_MANIFEST.json](IMPORT_MANIFEST.json) records every original path and both
source and destination SHA-256 hashes. Eighteen documents are byte-identical;
the reconciliation report's extractor link now points to the source commit.
The Engine checkout and history were left unchanged.

The original 403-page story PDF and full generated question/answer audit registers
are not in this source snapshot. Historical claims about their examination have
not been independently reverified during this import. The extraction command in
the imported guide belongs to the Engine's tools; that tool was not migrated.
Private conversation/account exports were not copied.

Historical headings referring to a custom MMO engine are retained as provenance.
The active game uses the [MMO engineering architecture](../engineering/README.md).
Narrative decisions do not silently override that architecture or create gameplay
implementation commitments.
