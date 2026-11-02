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
