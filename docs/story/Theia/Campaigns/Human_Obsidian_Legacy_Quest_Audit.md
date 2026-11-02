# Human Quest Source-of-Truth Audit — Obsidian Legacy Layer

**Status: cleanup blocker identified; do not delete legacy source graph yet.** The Human 1–80 [canonical campaign](Human_1_80_Canonical_Campaign.md) and [current quest authoring manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) remain the approved narrative and sole working structured inventory. The [H/HS crosswalk](Human_Quest_Editorial_ID_Crosswalk_Draft.json) retains 92 H-series and 24 HS-series editorial IDs but mappings are **not approved**.

## Additional active legacy surfaces found

1. `docs/story/THEIA_HUMAN_QUEST_REGISTER.md` — 92 principal and 24 optional old quest descriptions, old progression conventions and old player decisions.
2. `docs/story/Theia/Sources/THEIA_HUMAN_QUEST_REGISTER.md` — Obsidian source-library copy of the old register, referenced through wiki-links.
3. `docs/story/Theia/Campaigns/Quests/H*.md` and `HS-*.md` — individual legacy quest notes with `type: quest`, old `quest_id`, planning band, original objective, and `source: [[Sources/THEIA_HUMAN_QUEST_REGISTER]]`. Examples verified: `H1-01 - Before the Next Caravan.md` and `H5-16 - The Severed Chain.md`.
4. `docs/story/Theia/Campaigns/Quest Index.md`, `Human Campaign.md`, act pages and Obsidian Story Browser references — may still surface the old 1–60 story as current content.
5. `docs/story/Theia/Source Library.md`, source reading maps, `README.md`, documentation guides and import manifests — track the original source library, and their links need migration or archival decisions.

## Concrete contradiction

`Quests/H5-16 - The Severed Chain.md` still describes selecting safe severance or bounded inversion to complete the original level-60 campaign. This **does not represent** the locked Q176–Q184 level-80 ending (Thes'var killed, sabotage discovered, Oth'varen physically dies correcting alignment, Nhal's coherent identity shatters while essence persists). The old page is provenance only and must not be treated as a current quest.

## Safe consolidation plan

1. **Do not automatically map** 116 old H/HS notes to 184 new quests based on title alone. Record approved `reused/modified/split/merged/retired/new` relations and preserve historical IDs.
2. Preserve unique source facts, original objectives, optional stories, progression conventions and old decision history in an explicit archival provenance record or Git history. **Do not migrate contradicted old finale outcomes as canon.**
3. Choose a single active Obsidian quest browsing strategy: generate or curate Q001–Q184 pages from the current manifest, or make the manifest the sole quest browser source. Do not keep legacy H/HS pages as active quest records alongside it.
4. Update `Quest Index`, Human Campaign, act pages and source links before removing or archiving the old 116 quest notes and mirrored source register. Ensure existing Obsidian wiki-links do not silently break.
5. Check `Source Library` and `IMPORT_MANIFEST.json` references. Distinguish historical source archives from active campaign canon; deleting source documents may affect broader lore provenance beyond the Human campaign.
6. Run repository-wide reference checks and confirm the current quest count is **184** with **684 authored stages**, no accidental hard level gates and no duplicate active runtime quest inventory.
7. Do not modify Codex's Rust quest engine or `docs/QUEST_ENGINE.md` while doing editorial cleanup.

## Immediate disposition

**No deletion of H/HS source register, mirrored source library, or Obsidian quest pages is authorized by this audit.** Their linked graph must be reconciled first. The earlier 17 obsolete playable packs/briefs and duplicate 184-quest production specification were already retired. Git history preserves their old versions.

This audit is an inventory and cleanup plan, not a second set of executable quest definitions.

## Preservation update — 2 November 2026

The [legacy ideas archive](Human_Legacy_Sidequest_Zone_Character_Archive.md) now records all 24 HS optional hooks and all 28 non-exact H dispositions, including five canon conflicts. The [draft crosswalk](Human_Quest_Editorial_ID_Crosswalk_Draft.json) retains source objectives and all 116 historical IDs. This is preservation of candidates, **not approval of mappings or proof of unique content**. The 64 exact-title H candidates also remain unapproved. The original register and mirrored source documents must not be removed until unique dialogue, location, NPC and quest detail comparisons and wiki-link migration are verified. Obsidian application assets can be removed independently. The old story branch has zero commits ahead of main but should be deleted only after this final verification and through an authorized branch deletion mechanism.

## Complete historical objective comparison coverage

The [64 exact-title objective comparison](Human_Legacy_64_Exact_Title_Objective_Comparison.md) now records original objective and current stage descriptions for every exact-title H-series candidate. The [ideas archive](Human_Legacy_Sidequest_Zone_Character_Archive.md) records stage-level triage for the 28 non-exact H-series candidates and all 24 HS-series optional quests. Thus **116/116 historical quest entries have documented comparison material**. This is **not 116/116 approved mappings**: NPC, dialogue, zone, choice, source-link and unique-detail reconciliation remain outstanding. Preserve historical source and mirrored copies until those checks are finished; do not alter the current manifest or Rust runtime to match obsolete drafts.

## World Directory navigation retirement

The former Obsidian-generated `Lore/World Directory.md` (281 wiki-link entries) is now a Markdown redirect to [World Directory Legacy Reference](../Lore/World_Directory_Legacy_Reference.md). The archived list is historical and may contain targets already removed in prior cleanup. **Do not treat conversion of link syntax as validation that all targets resolve.** Keep the redirect until remaining inbound links are migrated; the source register and lore material remain protected pending unique-detail reconciliation.

## Archive link correction

Corrected 25 relative Lore-directory paths in `Lore/World_Directory_Legacy_Reference.md` after the initial wiki-link conversion. Other converted links remain unverified for target existence, including links to historically deleted quest notes. The legacy index is a provenance aid, not a guarantee of valid live navigation.

## Legacy per-quest link retirement

The historical World Directory no longer links directly to the 116 old H/HS per-quest Obsidian pages; those entries are plain historical labels linked to the preserved crosswalk. A sample of eight Lore link targets was verified to exist after correcting their relative paths. This does **not** establish repository-wide link validity or full content equivalence. Preserve surviving source notes until remaining unique details are reconciled.

## Cleanup execution — repository-tree verified

The repository tree was checked after commits `30c0cfe`, `3f93c16` and `6746658`. **38 remaining Obsidian application/plugin files, 43 surviving H/HS per-quest cards, and 12 Obsidian Copilot bundle files** were retired on `main`. The tree contains **zero** `docs/story/Theia/.obsidian/` paths, **zero** `docs/story/Theia/Campaigns/Quests/` paths, and **zero** story `.base` or `.canvas` files. The original 116 IDs/objectives are retained in the crosswalk and historical source register, with 116 comparison records and a noncanonical future-content idea archive. Git history preserves deleted cards. The source library, character/zone notes and historical register remain pending detailed unique-content and inbound-link reconciliation. Do not interpret removal of duplicate cards as approval of historical-to-current runtime mappings.

## Source mirror normalization comparison — 2 November 2026

All 19 `THEIA_*.md` source mirrors were compared to the same-named `docs/story/` originals using a conservative normalization that removes Obsidian wiki-link presentation and collapses whitespace. Nine pairs matched under that normalization and the **nine mirrored copies only** were deleted on `main` (commit `9179951`): CHARACTER_ARCS, HUMAN_ACT_I_STORY, HUMAN_CAMPAIGN_REVIEW, HUMAN_CAST_AND_PLACES, HUMAN_CONNECTED_RACIAL_STORIES, HUMAN_HEARTLANDS_SETTLEMENT_REGISTER, HUMAN_STORY_DUNGEONS, MEMORY_SCENES, ZONE_POPULATION_FRAMEWORK. Original root documents remain. **Ten other mirrors did not match**, so they remain untouched. This normalization is a useful duplication check, not a semantic proof; some archived reading maps still point to the removed mirror paths and require link migration.

## Additional mirror reconciliation

Two further Obsidian source mirrors, `THEIA_HUMAN_QUEST_REGISTER.md` and `THEIA_HUMAN_CAMPAIGN.md`, matched their root historical originals after removing only the root's newer **SUPERSEDED** notice and normalizing wiki-link/whitespace formatting. Their mirror copies were removed on `main` (commit `5af9651`); the original root documents and editorial crosswalk remain. Other mirrors, including World Bible and Interwoven Chronicle, show substantive wording differences and remain protected. References to the deleted mirrors in source reading maps still need migration.

## Source mirror divergence — World Bible

The root `THEIA_WORLD_BIBLE.md` and Obsidian mirror differ in actual wording, not just wiki-link syntax: the root refers to the historical 'seventh creation' wording while the mirror says 'Kel'vari creation' in the Q40 decision summary. This is a historical editorial wording change, **not** approval to alter the locked sequence (uncorrupted Kel'vari creation, accidental First Resonance, later Nhal influence). Keep both documents until remaining differences are classified; never select a canonical variant merely because one mirror is newer or has cleaner links.

## Historical World Directory — full file-target verification

Verified all **281 Markdown link targets** in `Lore/World_Directory_Legacy_Reference.md` against the current recursive `main` repository tree. One remaining missing target (`Campaigns/Quest Index.md`) was redirected to `Campaigns/CURRENT_HUMAN_CAMPAIGN.md`; the subsequent check returned **281/281 existing file targets, zero missing**. This validates file paths only, not heading anchors or semantic correctness. The broader character/reading-map backlink audit and five protected source-mirror differences remain open.

## Reading-map navigation verification

All 15 surviving `Editorial/Reading Maps/` Markdown files were inspected after replacing four divergent-source maps with explicit links to both preserved versions. **Zero of the 15 files now contain Obsidian `[[...]]` wiki-link syntax.** This is a syntax audit of the reading-map directory only; other character, lore, atlas and source notes may still contain wiki-links and require separate migration. The five protected lore mirrors remain untouched.

## Campaign-cast source migration — 47 files processed

All 47 `Characters/Campaign Cast/` cards were processed to replace their obsolete `Sources/THEIA_HUMAN_CAST_AND_PLACES` references with links to the preserved original `docs/story/THEIA_HUMAN_CAST_AND_PLACES.md`. Character descriptions and canon statuses were not intentionally changed. A subsequent search surfaced three recently updated cards; direct fresh-file checks confirmed those three no longer contain the old source reference. **A complete fresh scan of all 47 files has not yet been completed**, so do not assert repository-wide absence of every obsolete reference until that check passes.

## Campaign-cast migration — final verification PASS

A fresh complete fetch of all **47** `Characters/Campaign Cast/` Markdown files was completed in four batches (12+12+12+11). **Zero** contained the obsolete `Sources/THEIA_HUMAN_CAST_AND_PLACES` reference. The source-link migration for this directory is verified complete. This does not certify other Obsidian wiki-links elsewhere in the vault or the five protected lore-mirror differences.
