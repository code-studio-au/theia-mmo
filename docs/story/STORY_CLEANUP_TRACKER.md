# Story cleanup tracker — TEMPORARY, DELETE WHEN COMPLETE

> **Lifecycle: DELETE THIS FILE after all checklist items are complete and verified on `main`.** This is a disposable working checklist, **not** a permanent canon document, archive or second source of truth. Do not preserve this tracker after closure; Git history retains its audit trail.

Status: **IN PROGRESS** (2 November 2026). No unchecked item is represented as finished.

## Authoritative destinations

- [Current Human Campaign](Theia/Campaigns/CURRENT_HUMAN_CAMPAIGN.md): current hierarchy and campaign status.
- [Canonical 1–80 Campaign](Theia/Campaigns/Human_1_80_Canonical_Campaign.md): narrative authority.
- [Consolidated Quest Manifest](Theia/Campaigns/Human_1_80_Current_Quest_Authoring_Manifest.json): working 184-quest inventory, not executable Rust.
- [Legacy idea bank](Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md): unique, non-canonical side quests, zones, characters and hooks.
- [Story decisions](DECISIONS.md): decisions and precedence.

## Removal rule

For every candidate: compare the **entire** old file to its authoritative replacement; preserve unique confirmed decisions in the correct canonical file and unique unused ideas in the idea bank; preserve original H/HS ID evidence where mappings remain unresolved; repair all incoming Markdown/Obsidian links and indexes; then delete the redundant file. Do not silently promote WORKING / Needs Review material into canon. Deleted content remains recoverable from Git history.

## Remaining work

- [ ] Inventory all remaining `docs/story/Theia/Sources/` copies against root `docs/story/` and current `Theia/` files. The Sources README is now explicitly historical/non-authoritative (commit `da198252`); **file-by-file deletion review remains outstanding**.
- [ ] Audit and retire remaining Obsidian-only H/HS quest notes, vault navigation, and duplicate source-library copies; verify links before deleting.
- [ ] Reconcile the old H-series 92 and HS-series 24 quest IDs and objectives against Q001–Q184; retain the crosswalk and original evidence until resolved.
- [ ] Compare historical `THEIA_HUMAN_CAMPAIGN.md`, `THEIA_HUMAN_QUEST_REGISTER.md`, `THEIA_HUMAN_ACT_I_STORY.md`, and other older 1–60 campaign material with the current 1–80 campaign; remove only fully superseded copies.
- [ ] Audit duplicate world/lore narrative in `THEIA_WORLD_BIBLE.md`, `THEIA_INTERWOVEN_CHRONICLE.md`, `THEIA_DETAILED_LORE_ARCHIVE.md` and individual `Theia/Lore/`, `History/`, `Characters/` pages. Assign authority by subject; keep genuine historical evidence and distinct narrative forms where useful.
- [ ] Review older editorial reading maps, reconciliation reports and import-era indexes for stale links and duplicate claims; consolidate useful unique evidence before retirement.
- [ ] Validate remaining story links and quest references after removals; check no unique characters, zones, side quests or canon decisions were lost.
- [ ] Verify consolidated manifest, canonical ending, approval labels and Rust-vs-planning boundaries still agree.
- [ ] Final repository review: no live superseded duplicate story files, no broken references, one authority per subject, and historical provenance recoverable in Git history.
- [ ] **Final action:** delete `docs/story/STORY_CLEANUP_TRACKER.md` from `main` once all preceding items have been completed and verified. Remove any links to it in the same cleanup.

## Completed / evidence

- [x] Current Human Campaign already identifies the authoritative 1–80 narrative, consolidated 184-quest / 684-stage working manifest, and non-canonical idea bank.
- [x] Historical `Theia/Sources/README.md` rewritten to explicitly distinguish imported copies from active authority (commit `da198252`).

Do not tick work merely because it is planned or described in older reports. Record exact commits and evidence for completed removals before deleting this temporary tracker.

## World Bible duplicate comparison — 2 November 2026

- Compared complete `docs/story/THEIA_WORLD_BIBLE.md` (476 lines) and `docs/story/Theia/Sources/THEIA_WORLD_BIBLE.md` (476 lines).
- The files are **not byte-identical**; 43 line positions differ after stripping Obsidian `[[wikilinks]]` syntax. Differences include the root copy's older generic “seventh race” wording versus the source copy's named Kel'vari, plus editorial and cross-reference differences.
- **Decision: retain both pending full semantic merge.** The source copy is not current authority; do not delete until every unique substantive difference is reconciled into authoritative lore and all links are repaired. Avoid promoting either entire copy as canon without checking later creator locks.

## World Bible canonical corrections — 2 November 2026

- [x] Reconciled three stale substantive lines in root `THEIA_WORLD_BIBLE.md` against the newer source copy: approved King Aldren Vaelmont / Corven Vaelmont ruler names, confirmed Kel'vari name instead of OPEN, and official Kel'vari / Oth'vaeri terminology. Commit `28f8ecfb`.
- [ ] Remaining World Bible differences still require review (mostly Kel'vari terminology and Obsidian wikilinks). **Do not delete the source copy yet**; validate references and unique content first.

## World Bible content aligned; reference migration outstanding — 2 November 2026

- [x] Root World Bible reconciled with all 40 remaining differing line positions from the imported Obsidian source copy; Obsidian wikilinks converted to plain labels in the root. Commit `482e7a10`. This resolves the 43 line differences previously recorded, **not** the incoming-link problem.
- [ ] Migrate explicit `Sources/THEIA_WORLD_BIBLE.md` references in approximately 29 indexed lore/atlas pages to the root `docs/story/THEIA_WORLD_BIBLE.md` path. Search results may not be exhaustive; verify all incoming links before removal.
- [ ] Delete `Theia/Sources/THEIA_WORLD_BIBLE.md` only after confirming its normalized content is represented by the root and incoming links are fixed.

## World Bible link migration — 2 November 2026

- [x] Redirected source links to canonical root World Bible in five files: `Lore/Magic.md`, `Lore/Oth'vaeri.md`, `Lore/Peoples/Orcs.md`, `Lore/Passage Gates.md`, `Lore/Peoples/Humans.md`.
- [ ] Migrate remaining indexed references (initial search found 29 files total; five updated in this pass). Do not remove old source until link verification is complete.

## World Bible link migration — additional batches

- [x] Migrated eight more files: `Lore/Peoples/Aelari.md`, `Lore/The Seven Gods.md`, `Lore/Peoples/Dwarves.md`, `Lore/Peoples/Vaelari.md`, `Lore/Peoples/Korrath.md`, `Lore/Peoples/Vorrath.md`, `Lore/Peoples/Kel'vari.md`, `Lore/Peoples/Halflings.md`.
- [x] Migrated seven more files: all five indexed `Atlas/Enclaves/` pages, `Lore/Resonance Inversion.md` and `Lore/Primordial Cosmology.md`.
- [ ] Initial 29-file link list: 20 migrated, 9 remaining. Re-search repository for additional incoming links after these are complete.

## World Bible reference migration — latest pass

- [x] Checked the final nine files in the initial indexed list: four were already migrated and five were updated in this pass (`The Unmade`, `Divine and Mortal History`, `Orders and Hidden Conflict`, `The Guardians`, `The Corrupted Conclave`).
- [ ] **Repository-wide live-link verification still outstanding.** GitHub code search continues to return its previously indexed 29 paths; do not interpret stale search results as proof of current file contents. Re-fetch any apparent matches and validate links before deleting the source duplicate.

## World Bible duplicate retired — 2 November 2026

- [x] Re-fetched all 29 originally indexed incoming-reference files from live `main`; none contains `Sources/THEIA_WORLD_BIBLE.md`.
- [x] Full root and imported World Bible text compared after normalizing Obsidian `[[wikilinks]]`; content matches.
- [x] Deleted `docs/story/Theia/Sources/THEIA_WORLD_BIBLE.md` from `main` (commit `4ed71c05`). Canonical root `THEIA_WORLD_BIBLE.md` remains. Git history preserves the retired file.
- [ ] Continue other imported source duplicates, obsolete Obsidian notes, campaign and lore audits; tracker itself must still be deleted only at final completion.
