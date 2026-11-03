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

## Second imported-source duplicate audit — 2 November 2026

- [x] Compared full root/imported pairs for Interwoven Chronicle (826 lines each; five normalized differing lines), Detailed Lore Archive (600 each; 36 differing lines), Canon Reconciliation Report (135 each; 21 differing lines), and DECISIONS (normalized text identical).
- [x] Updated four Chronicle creation references to named Kel'vari (commit `c97df5ca`). Updated Chronicle's stale claim that the ending remains open, pointing instead to the approved level-80 canonical campaign (commit `445d5451`).
- [ ] Retire imported Chronicle only after checking remaining live incoming links and normalized equality (editorial ending intentionally newer than historical imported source).
- [ ] Review and reconcile 36 Detailed Lore Archive differences and 21 Canon Reconciliation Report differences before deleting either imported copy.
- [ ] Retire duplicate imported DECISIONS only after validating all incoming links; content matches root after wikilink normalization.

## Additional canon reconciliation — 2 November 2026

- [x] Detailed Lore Archive: replaced outdated 'species name remains open' claim with confirmed Kel'vari name (commit `e6f1dbcf`).
- [x] Canon Reconciliation Report: corrected stale open-name inventory with approved Kel'vari and named founders (commit `62d6ed4c`).
- [ ] Other differences remain; older historical passages may describe superseded decisions, and later level-80 ending locks override old 'Nhal fate open' claims. Do not bulk-copy historical statements or delete source files until verified.

## Imported lore text alignment — 2 November 2026

- [x] Root Detailed Lore Archive and imported source now match after Obsidian wikilink normalization (35 additional line corrections, commit `2f2262cc`).
- [x] Root Canon Reconciliation Report and imported source now match after wikilink normalization (20 additional line corrections, commit `e61734f9`).
- [ ] Incoming links to imported archive/report copies must be migrated and verified before deletion; historical reports must not override the approved level-80 ending.

## Imported lore link migration — 2 November 2026

- [x] Redirected 13 divine-character/historical-event files from imported `Sources/THEIA_DETAILED_LORE_ARCHIVE.md` to canonical root archive.
- [x] Updated three editorial reading maps to link authoritative root documents rather than imported Detailed Lore Archive, Canon Reconciliation Report and Interwoven Chronicle mirrors.
- [x] Redirected three History/Events files from imported Chronicle to root Chronicle.
- [ ] Re-fetch live files and check relative path resolution; recheck additional references beyond the indexed set before deleting source mirrors.

## Four imported source duplicates retired — 2 November 2026

- [x] Live-fetched and verified 19 migrated incoming-link files; no old mirror references in those files and canonical relative links checked.
- [x] Normalized content identical for Detailed Lore Archive, Canon Reconciliation Report and DECISIONS; Chronicle differs only at editorial-status line 784, where canonical root correctly supersedes old 'ending open' wording.
- [x] Deleted imported duplicates from `Theia/Sources/`: Detailed Lore Archive (`4550b1e4`), Canon Reconciliation Report (`e33bf37d`), Interwoven Chronicle (`63a58cb7`), DECISIONS (`54e81ff8`). Root documents retained.
- [ ] Continue remaining source duplicates and vault cleanup; perform full repository-wide link validation before declaring all cleanup finished. **Delete this tracker itself only at final completion.**

## Source Decision Recovery audit — 2 November 2026

- [x] Compared root and imported `THEIA_SOURCE_DECISION_RECOVERY.md`: 232 lines each, 27 normalized differing line positions before correction. Mostly historical 'Seventh' versus confirmed 'Kel'vari' language.
- [x] Updated root final naming statement to confirmed Kel'vari wording (commit `dcff6f81`).
- [ ] Review remaining differences in source-decision context; preserve historical labels where they document the original question/answer rather than replacing them indiscriminately. Check incoming links before deleting mirror.
- [ ] Review remaining `Sources/IMPORT_MANIFEST.json` and `Sources/README.md` as provenance rather than current canon; do not delete provenance until import record is preserved elsewhere or Git history is deemed sufficient.

## Source Decision Recovery — historical distinctions preserved

- [x] Reconciled 23 later locked-decision lines from imported mirror into root (commit `1e6dedca`).
- [x] Remaining three normalized differences are early historical lines 19, 37 and 60: 'seventh race vanished', 'seventh god later renamed Ascendant', and 'seventh god THE ASCENDANT'. These are intentionally preserved as historical language, **not** unresolved current canon. The mirror's substitutions do not add unique decisions.
- [ ] Validate all incoming references to `Sources/THEIA_SOURCE_DECISION_RECOVERY.md` before retiring the imported copy.

## Source Decision Recovery mirror retired — 2 November 2026

- [x] Replaced obsolete Obsidian reading-map mirror link with the authoritative root document (commit `1638594a`).
- [x] Deleted reconciled `Theia/Sources/THEIA_SOURCE_DECISION_RECOVERY.md` (commit `c8f77fba`); three early historical phrases remain preserved in root.
- [x] Appended current retirement status to legacy divergence register (commit `b7fb610b`). Earlier 'mirrors remain' statements there are dated historical snapshots.
- [ ] Review whether `Theia/Sources/README.md` and duplicate `IMPORT_MANIFEST.json` are still needed, and repair import-era references; continue broader story and Obsidian cleanup.

## Import provenance duplicates retired — 2 November 2026

- [x] Root `docs/story/IMPORT_MANIFEST.json` and `Theia/Sources/IMPORT_MANIFEST.json` were byte-identical; deleted duplicate source manifest (commit `5bde970f`). Root import provenance remains.
- [x] Retired obsolete `Theia/Sources/README.md` after its source mirrors were removed (commit `ad728521`). Root story README and import manifest remain.
- [ ] Verify no remaining live source mirrors or stale references, then continue Obsidian/campaign cleanup. This temporary tracker must be deleted at the very end.

## Obsidian-era reading-map corrections — 2 November 2026

- [x] Corrected stale 'both versions remain' statements in Chronicle, Detailed Lore Archive and Canon Reconciliation Report reading maps (commits `449894fd`, `cd66f46f`, `2f9964e1`). Removed duplicate root links while retaining authoritative root references.
- [ ] Audit remaining Obsidian reading maps and legacy quest navigation for stale 1–60 claims and broken links; preserve original H/HS ID provenance until crosswalk is approved.

## Obsidian source reading-map index retired — 2 November 2026

- [x] Replaced `Theia/Editorial/Source Reading Maps.md` obsolete Obsidian wiki-link index with a historical redirect to current 1–80 campaign, canonical story, story index and editorial status (commit `396457b3`).
- [ ] Individual historical reading maps still require unique-annotation and incoming-link review before removal. Do not use their old 1–60 quest claims as active canon.

## Historical reading-map link check — 2 November 2026

- [x] Fixed `Reading Map - THEIA_HUMAN_QUEST_REGISTER.md` link to root historical register (was `../../../../`, now `../../../`; commit `015daa7b`).
- [x] Verified canonical root links in Chronicle, Detailed Lore Archive and Canon Reconciliation Report reading-map redirects use `../../../` from `Theia/Editorial/Reading Maps/`.
- [ ] Broader remaining reading-map link audit still needed; do not treat individual checks as full repository-wide validation.

## Reading-map link target verification — 2 November 2026

- [x] Resolved 11 Markdown file links across `Source Reading Maps.md`, Source Decision Recovery reading map and Human Quest Register reading map to existing `main` files.
- [x] Replaced one unverified directory-target link with plain folder text (commit `ee49fad6`); this was a directory rather than a verified file target, not proof the directory was absent.
- [ ] Continue auditing other reading maps and campaign files; these 12 checks are not a full-tree validation.

## Additional reading-map link repairs — 2 November 2026

- [x] Live-verified 12 links across Chronicle, Detailed Lore Archive, Canon Reconciliation Report and Source Decision Recovery reading maps; all resolve to existing files.
- [x] Fixed four broken root links in historical Human Story Dungeons, Human Campaign Review, Human Cast and Places, and Human Connected Racial Stories reading maps (commits `3d7be1fa`, `24242a3e`, `31f5d2e9`, `94501e76`). Their old `../../../../` paths escaped `docs/story/`; corrected to `../../../`.
- [ ] Continue remaining historical reading-map link validation and evaluate redundant redirect files for retirement.

## Six more reading maps repaired and verified — 2 November 2026

- [x] Fixed source links in historical reading maps for Human Heartlands Settlement Register (`24e3a1b0`), Zone Population Framework (`15fb2791`), Modern Story Revelation Timeline (`4dff2c9a`), Line-by-Line Verification Status (`aace11dd`), Memory Scenes (`fadb8678`), and Documentation Guide (`0b36df3c`).
- [x] Live-resolved all 13 Markdown file links in those six reading maps against current `main`; all targets exist.
- [ ] Continue remaining reading maps and redundant redirect retirement; final full-tree validation still pending.

## Human historical reading-map verification — 2 November 2026

- [x] Live-verified 14 links across six Human historical reading maps (Quest Register, Story Dungeons, Campaign Review, Cast and Places, Connected Racial Stories, Heartlands Settlement Register); all resolve.
- [x] Confirmed older `Reading Map - THEIA_CHARACTER_ARCS.md` and `Reading Map - THEIA_HUMAN_ACT_I_STORY.md` paths are absent; no replacement created without evidence of required incoming links.
- [ ] Continue checking other old redirects and repository-wide links; historical H/HS ID crosswalk remains protected.

## Historical redirect and mirror-audit consolidation — 2 November 2026

- [x] Live-verified eight links in Human Act 2 historical redirect, World Directory historical redirect and legacy source mirror divergence register; all resolve.
- [x] Replaced outdated divergence register with a short CLOSED historical audit summary, retaining reconciliation outcomes and Git provenance (commit `35c3f918`).
- [ ] Remaining old Obsidian quest notes and campaign registers require preservation/retirement audit; final repository-wide validation outstanding. Delete this temporary tracker only after completion.

## Obsidian legacy quest audit status correction — 2 November 2026

- [x] Updated `Human_Obsidian_Legacy_Quest_Audit.md` with current status: five lore source mirrors previously listed as blockers have been reconciled and retired (commit `4e3e46e2`).
- [ ] Historical H/HS editorial crosswalk remains unresolved; do not delete original objective evidence or claim 184-quest runtime mapping complete.
- [ ] Continue legacy quest note inventory, preservation and link review before retiring the remaining Obsidian pages.

## Quest crosswalk status correction — 2 November 2026

- [x] Verified `Theia/Sources/THEIA_HUMAN_QUEST_REGISTER.md` is absent and root `THEIA_HUMAN_QUEST_REGISTER.md` remains.
- [x] Corrected outdated mirror and retired-pack statements in `Human_Quest_Editorial_ID_Crosswalk_Review.md` (commit `d0a0791a`).
- [ ] Mapping approval for 92 H and 24 HS IDs remains outstanding; preserve original objectives and crosswalk. No stable runtime IDs allocated.

## Exact-title audit historical status correction — 2 November 2026

- [x] Corrected `Human_Legacy_Exact_Title_Mapping_Audit.md` to acknowledge retired Obsidian source mirror while protecting the root historical H/HS register (commit `00e8e8ee`).
- [ ] The 64 exact-title H candidates are title matches only; eight identified objective-specific details and four contact discrepancies still require semantic review. Do not approve runtime mappings from title matches.

## H-series objective-level reconciliation started — 2 November 2026

- [x] Compared crosswalk source objectives to current manifest stages for H1-01/Q001, H4-07/Q097, H5-09/Q071, H5-10/Q072 and H5-20/Q184. Results and remaining details recorded in `Human_Legacy_Exact_Title_Mapping_Audit.md` (commit `393e739f`).
- [ ] Review corresponding dialogue and outcome evidence before marking any mapping approved. Current Grey Surveyor spoiler-safe aliases remain unchanged; no runtime IDs assigned.
- [ ] Continue remaining H/HS objective-level comparisons and preserve unique side hooks in non-canonical archive.

## H4-07 / Q097 dialogue corroboration — 2 November 2026

- [x] Verified canonical branching scene for Q097 includes independently verified signature evidence, Oth'varen's firsthand account, accusation/curiosity/trust responses, and Sel'ora's explicit scrutiny requirement. Dialogue Continuity Audit confirms Grey Surveyor spoiler gate in Q071–Q072 (commit `6f9d6c58`).
- [ ] H4-07 editorial ID mapping still needs prerequisite/consequence review and approval; no runtime ID assignment.

## Quest comparison consolidation — 2 November 2026

- [x] Linked focused exact-title audit to existing full 64-entry objective comparison, avoiding duplicate comparison registers (commit `190d3e65`).
- [ ] Continue reviewing unique dialogue and outcomes against the full comparison and draft crosswalk. No mappings or runtime IDs are approved by this documentation change.

## Additional early H-series prerequisite review — 2 November 2026

- [x] Compared H1-02 through H1-06 against Q002, Q003, Q004, Q009 and Q010 in the existing 64-entry objective comparison (commit `7dc61235`). All five candidate prerequisite chains align; stage/objective structure strongly corresponds.
- [ ] Verify historical location, surviving teamster, original damaged document and authentication-dialogue details before any mapping approval. No runtime IDs or canon stages changed.

## H1-07 through H1-11 comparison — 2 November 2026

- [x] Compared five additional historical quests to Q011/Q012/Q013/Q018/Q019 in existing 64-entry comparison; objectives, contacts and prerequisite chains align (commit `baebd2d9`).
- [ ] Verify historical protected travel route, interview authority, court-access framing and evidence custody before mapping approval. Do not change canonical quest stages or assign runtime IDs based on this review.

## H1-12 through H1-16 review — 2 November 2026

- [x] Compared H1-12 through H1-16 against Q020/Q021/Q022/Q027/Q028 in the full 64-entry objective comparison; contact, stage structure and prerequisite chains align (commit `4db494eb`).
- [ ] Verify original intended-killing evidence in H1-15/Q027 dialogue and custody record before approving the mapping. Preserve Garrick's innocence-of-involvement distinction and lawful court handling.

## Act I conclusion reconciliation — 2 November 2026

- [x] Compared H1-17/Q030 and H1-18/Q031; H1-17 is a **modified prerequisite** because current Q029 introduces separate evacuation/evidence preparation before Q030 (commit `535a6d09`). Preserve Q029.
- [x] Confirmed draft crosswalk has no H1-19 or H1-20 rows; do not invent IDs.
- [ ] Verify H1-18 removal-from-office and Dominion non-absolution consequences in current dialogue before mapping approval.

## Act II opening reconciliation — 2 November 2026

- [x] Compared H2-01 through H2-05 against Q035/Q036/Q037/Q049/Q050 (commit `e9e8a3a2`).
- [ ] H2-04's old direct prerequisite H2-03 differs from current Q049 prerequisite Q047; preserve intervening modern investigations.
- [ ] Review exposed goods, customs registration and H2-05's historical hold permission before approving mappings. No runtime IDs assigned.

## H2-series triage complete — 2 November 2026

- [x] All 18 H2-series candidates now have objective/prerequisite triage recorded in `Human_Legacy_64_Exact_Title_Objective_Comparison.md` (commit `22f7bf12`).
- [x] Flagged six changed prerequisite edges: H2-06, H2-07, H2-08, H2-16, H2-17, H2-18. Preserve the modern campaign ordering.
- [ ] Dialogue, consequences, original optional access and evidence details still need review; no historical mappings approved.
- [ ] Continue H3/H4/H5 and 24 HS optional quest provenance; do not delete original evidence prematurely.

## H3-series triage complete — 2 November 2026

- [x] Compared all 18 H3-series historical objectives, contacts and prerequisite edges with current manifest (commit `9415b095`).
- [x] Flagged four changed dependency edges: H3-01, H3-02, H3-03 and H3-18; optional content must not be made mandatory by importing old prerequisites.
- [ ] Dialogue/consequence and unique detail checks remain before mapping approval; continue H4/H5 and HS optional hooks.

## Root story entry-point authority correction — 3 November 2026

- [x] Updated `docs/story/README.md` to direct readers first to `CURRENT_HUMAN_CAMPAIGN.md`, the canonical 1–80 campaign, the single 184-quest authoring manifest, the legacy ideas archive and story decisions (commits `43dce554`, `2b627ced`). Its former 1–60 import table is now explicitly a historical index, not the active quest reading order.
- [x] Verified the active `CURRENT_HUMAN_CAMPAIGN.md` identifies the 184-quest / 684-stage manifest as working editorial data, not executable Rust, and retains the original H/HS crosswalk as unapproved provenance.
- [ ] Audit other entry points and backlinks to the historical 1–60 import index. The README change does not establish that all remaining vault links resolve or that old quest mappings are approved.
- [ ] Continue the H4/H5 and HS dialogue, prerequisites and consequence comparisons; preserve unique evidence before retiring the historical quest register.

## H4-series triage — 3 November 2026

- [x] Compared all 18 H4 historical objectives and proposed mappings against live current-manifest stages and prerequisites; recorded the complete per-entry comparison in `Theia/Campaigns/Human_Legacy_64_Exact_Title_Objective_Comparison.md` (commit `0d3660c7`).
- [x] Flagged major non-equivalences: Kel'vari consent, optional WvW exercise, winter levy versus restitution, earlier Dominion arc relocation, Malrec versus Su'vareth, and Cassian custody follow-up. No canonical quest data changed.
- [ ] Verify dialogue, location, historical choices and consequences for H4 before approving any editorial mappings; retain unique candidate side-story material and source provenance.
- [ ] Continue H5-series and HS-series detailed review; final link audit, branch retirement and deletion of this tracker remain outstanding.

## H5-series triage — 3 November 2026

- [x] Compared all 20 historical H5 source objectives and prerequisites to proposed live-manifest quest stages, recording per-entry evidence in `Theia/Campaigns/Human_Legacy_64_Exact_Title_Objective_Comparison.md` (commit `1f7dd05f`).
- [x] Identified non-equivalent historical finale claims: premature Nhal naming, early Thes'var intervention, old Keld/Voss outcomes, severance branch, and unconfirmed Cassian sentencing. The locked level-80 finale remains unchanged.
- [ ] Review source dialogue, contacts, evidence consequences and regional optional ideas before approving historical mappings or deleting original register. H4 and H5 objective/prerequisite triage is complete, **not** semantic approval.
- [ ] Continue 24 HS optional entries, other historical lore/navigation duplicates and full-tree link validation. Keep this tracker until verified closure.

## HS optional quest stage comparison — 3 November 2026

- [x] Compared all 24 HS historical optional hooks to live proposed Q-stage descriptions and recorded historical access conditions in `Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md` (commit `425a08b4`).
- [x] Preserved distinct thematic ideas where multiple HS hooks point to one current Q (HS-02/HS-09, HS-04/HS-10/HS-11, HS-13/HS-15, HS-17/HS-18), and flagged HS-19, HS-22 and HS-24 access/outcome differences. No active quest count, runtime IDs or canonical outcomes changed.
- [ ] Review source-specific NPC dialogue, geography, decisions, access and consequences before any mapping approval. Historical 92 H + 24 HS source entries now have documented objective/stage comparison coverage, **not** 116 approved equivalences.
- [ ] Finish remaining legacy file/backlink reconciliation, full-tree link audit, branch retirement and final tracker deletion only after verification.

## Historical story branch ancestry check — 3 November 2026

- [x] Compared `main` with `story/launch-main-story-1-60` through GitHub commit ancestry: old branch 0 commits ahead, 534 commits behind at the time of the check. No branch-only commits require cherry-picking.
- [x] Recorded the result and its limits in `Theia/Campaigns/Human_Quest_Editorial_ID_Crosswalk_Review.md` (commit `6f708688`). Branch ancestry does not prove all historical content was semantically reconciled.
- [ ] Keep the old branch until remaining protected historical source, semantic checks and backlinks are resolved; retire it with an authorized branch deletion action only after verification.
- [ ] Do not delete this tracker until the full cleanup is complete.

## Obsidian navigation retirement — 3 November 2026

- [x] Replaced obsolete `Story Browser.base` embeds and `Sources/THEIA_*` wiki-links in `Theia/Atlas/Atlas.md`, `Theia/History/Timelines.md` and `Theia/Characters/Characters.md` with Markdown links to surviving root sources and current campaign. Commits `0d6b6adf`, `4c7c22cc`, `2080effa`.
- [x] Removed two unverified Aethyr/Nhal file links from the character index after direct fetches returned 404 (commit `8eea1037`). Divine context remains accessible through the Seven Gods and current campaign links.
- [ ] Continue replacing remaining obsolete `.base` embeds and source-mirror wiki-links in other index pages. Verify each target against the live repository before claiming full link health.
- [ ] Do not retire historical source evidence, story branch or this tracker until final validation is complete.

## Additional index retirement — 3 November 2026

- [x] Replaced stale Obsidian source links and the obsolete five-climax active roster in `Theia/Campaigns/Story Encounters.md` with a historical note and current 1–80 narrative/encounter links (commit `94eb392d`). The original titles remain in the historical dungeon source and Git history.
- [x] Replaced obsolete `.base` embed and source-mirror references in `Theia/Lore/Lore and World.md` with root story sources and subject navigation (commit `703c9bb8`).
- [x] Replaced retired source-mirror wiki-link navigation in `Theia/History/Chronicles and History.md` with Markdown links to surviving history/lore sources (commit `74948ecb`).
- [ ] Continue auditing other index pages, full link targets, original H/HS dialogue and consequences. The old branch and this tracker remain until complete.

## Recent index target spot-check — 3 November 2026

- [x] Directly fetched all six Conclave/Sixth Scholar character-note targets referenced by `Theia/Characters/Characters.md`: Oth'varen, Vaer'uun, Thes'var, Su'vareth, Oma'vira and Khor'vun all exist on `main`.
- [x] Removed two directory-only Markdown links in `Theia/Atlas/Atlas.md`, leaving the folder names as plain navigation guidance rather than pretending they are verified file targets (commit `b97a6c94`).
- [ ] Continue checking every relative link in remaining indexes and documents; six verified character targets do not establish repository-wide link validity.

## Lore index file-target verification — 3 November 2026

- [x] Directly verified all 13 Markdown file targets in `Theia/Lore/Lore and World.md` against live `main`; **13/13 exist, zero missing**. Includes root World Bible, Detailed Lore Archive, Chronicle, story decisions, eight topic notes and current campaign entry point.
- [ ] This verifies file existence only, not heading fragments, semantic accuracy or repository-wide links. Continue other index and legacy-content audits before closure.

## History and encounter index file-target verification — 3 November 2026

- [x] Live-fetched **9/9** Markdown targets in `Theia/History/Chronicles and History.md`, **5/5** in `Theia/History/Timelines.md`, and **4/4** in `Theia/Campaigns/Story Encounters.md`; **18/18 target references exist, zero missing**. Some target files are shared between indexes, so 18 counts link occurrences, not unique documents.
- [ ] This does not verify heading anchors, all Obsidian wiki-links, narrative equivalence or all repository paths. Continue remaining index checks and protected historical content review.

## Character index link-target verification — 3 November 2026

- [x] Live-fetched all **9 Markdown file targets** in `Theia/Characters/Characters.md`: six Conclave/Sixth Scholar character pages, the historical Human Cast and Places source, Seven Gods lore, and current Human campaign. **9/9 exist, zero missing**.
- [x] Replaced one directory-only Markdown link to `Campaign Cast/` with a plain folder reference (commit `ecef0783`).
- [ ] Continue repository-wide source references and historical-content semantic audit. File-target existence does not verify heading anchors or canon status.

## Historical Timeline obsolete-finale correction — 3 November 2026

- [x] Removed obsolete Obsidian Story Browser embed and superseded five-act level-60 launch / later-expansion Nhal finale sections from `Theia/History/Historical Timeline.md` (commit `cdd2772d`). Kept the thirteen historical milestone entries and present-day context, with links to the current 1–80 campaign and creator decisions.
- [x] Explicitly documented that Oth'varen dies and Nhal's unified identity is shattered during the level-80 launch finale, while divine essence persists. Old wording is recoverable from Git history, not active authority.
- [ ] Audit historical event links and remaining old-finale claims elsewhere. The retained milestone wiki-links and headings have not all been verified; final link and semantic validation remains outstanding.

## Oth'varen current-canon character correction — 3 November 2026

- [x] Replaced the obsolete 'survives launch, dies later' statement in `Theia/Characters/Oth'varen.md` with the locked level-80 finale sequence, including Thes'var's defeat, Oth'varen's physical death, Nhal's identity shattering and surviving divine essence (commit `6f44e8d5`).
- [x] Replaced obsolete historical `Sources/THEIA_CHARACTER_ARCS` embed with a direct root historical source link, explicitly subordinate to current campaign canon.
- [ ] Audit other derived character pages and historical Future Saga for stale launch-versus-expansion wording. The original historical source can remain as provenance but must not be presented as current canon.

## Future Saga authority reconciliation — 3 November 2026

- [x] Reclassified `Theia/Campaigns/Future Saga.md` as a historical proposal and deferred-idea archive with a prominent 3 November 2026 authority notice (commit `0d33f360`). Current Human 1–80 narrative and decisions take precedence over older launch-versus-expansion speculation.
- [x] Corrected historical text that claimed Oth'varen must survive launch and that conflicting launch-inversion endings remained unresolved. The approved launch finale includes Oth'varen's death and Nhal's coherent identity shattering; divine essence persists. No future fragment outcome or expansion storyline is approved.
- [ ] Review the remaining historical proposal sections and their inbound links before considering any deletion. The archived ideas are not active canon and must not be silently promoted.

## Five Conclave founder source-embed migrations — 3 November 2026

- [x] Replaced obsolete `Sources/THEIA_CHARACTER_ARCS` embeds and `Sources/DECISIONS` wiki-links in the Thes'var, Vaer'uun, Su'vareth, Khor'vun and Oma'vira character pages with direct links to root historical arcs, story decisions and the authoritative 1–80 campaign. Commits `d42f2716`, `b9bfa991`, `95fe1fc0`, `50db606a`, `5ea19475`.
- [x] Preserved their existing creator-locked outcomes without rewriting quest implementation or inventing encounter details. The canonical 1–80 campaign now explicitly governs launch timing in each page.
- [ ] Continue stale Obsidian embeds and source references in other derived pages; verify full link graph before deleting the temporary tracker or retiring the old story branch.

## Lore topic World Bible reference cleanup — 3 November 2026

- [x] Replaced obsolete `Sources/THEIA_WORLD_BIBLE` wiki-links with direct surviving root World Bible and story decisions links in `Lore/Magic.md`, `Lore/The Seven Gods.md`, and `Lore/Primordial Cosmology.md` (commits `2a492fad`, `fc7f9008`, `aa85611d`).
- [x] Removed a duplicated identical historical-source link in Magic. Existing lore claims and individual deity references were not changed.
- [ ] Continue auditing surviving wiki-links and references across other lore/character/atlas notes; source-link cleanup is not a semantic canon approval.

## Additional World Bible topic source-link cleanup — 3 November 2026

- [x] Replaced retired `Sources/THEIA_WORLD_BIBLE` wiki-links in `Lore/Divine and Mortal History.md`, `Lore/Anchors and the Shroud.md` and `Lore/Peoples and Allegiances.md` with root World Bible and dated story decision links (commits `7e9b603f`, `e084b055`, `122bd35e`).
- [x] Collapsed three identical historical-source links in Anchors and the Shroud into one. No narrative content or quest definitions changed.
- [ ] Continue link and semantic verification across the remaining derived notes; do not infer whole-vault completion from these targeted changes.

## Orders and population reference cleanup — 3 November 2026

- [x] Removed stale `Sources/THEIA_WORLD_BIBLE` wiki-link and duplicate root source link from `Lore/Orders and Hidden Conflict.md`, linking current story decisions (commit `34dda4e2`).
- [x] Removed obsolete `Sources/THEIA_ZONE_POPULATION_FRAMEWORK` wiki-link and duplicate historical source references from `Lore/World Population.md`, retaining the root framework and its source-specific approval boundary (commit `58db7202`).
- [ ] Continue remaining derived source references, incoming links and final semantic verification. No quest definitions changed.

## Editorial and racial source-link migration — 3 November 2026

- [x] Replaced obsolete source-mirror links and World Bible embed in `Theia/Editorial/Canon and Open Questions.md` with verified root story sources and current campaign precedence (commit `072232de`).
- [x] Migrated `Theia/History/Memory Scenes.md` to the surviving root Memory Scenes source (commit `a539c09e`).
- [x] Migrated nine source-section wiki-links in `Theia/Campaigns/Connected Racial Stories.md` to the surviving root connected racial stories source, retaining **Needs Review** status (commit `cb48c20c`).
- [ ] Continue residual wiki-link and content checks; these migrations do not establish all individual quest-note links resolve or that racial stories are approved.

## Connected racial index source-link deduplication — 3 November 2026

- [x] Replaced nine repeated links to the same root `THEIA_HUMAN_CONNECTED_RACIAL_STORIES.md` document with one explicit historical source link and a list of covered cultural routes (commit `d6fe4a11`). The detailed racial proposals remain in the source, and the index retains **Needs Review**.
- [ ] Individual CR/DR quest-note wiki-links still require target verification. Do not remove the index or assert those links resolve without checking live files.

## CR/DR historical quest navigation retirement — 3 November 2026

- [x] Converted 21 CR/DR Obsidian wiki-link quest labels in `Theia/Campaigns/Connected Racial Stories.md` into plain historical editorial labels, preserving their titles and the root cultural-stories source (commit `53003595`). Sampled candidate per-quest target paths were absent; **this is not a full 21-target path audit**.
- [ ] Verify any remaining inbound links to retired individual CR/DR notes and compare unique source details before deleting other historical records. These labels are not current executable quest definitions.

## Navigation repairs recorded — 3 November 2026

- [x] Updated Connected Racial Stories navigation to current campaign and peoples references (commit `e01b24d0`).
- [x] Replaced missing Vault Import Record navigation with surviving import provenance and story index (commit `8fed9439`).
- [x] Updated Memory Scenes navigation to Markdown links, including the existing Six Reunited scene (commit `e26529e4`).
- [ ] Continue checking remaining historical links and unique material. These changes do not complete the repository-wide audit.

## Recent navigation target validation — 3 November 2026

- [x] Directly fetched eight unique destinations referenced by the recent Memory Scenes, editorial and racial index repairs: character index, history index, timelines, Six Reunited, import manifest, story README, current Human campaign and Peoples and Allegiances. **8/8 file targets exist on main**.
- [ ] File existence does not verify every wiki-link, heading anchor or story claim. Continue the broader audit before retiring the story branch and deleting this tracker.

## Memory-scene character navigation and racial index clarification — 3 November 2026

- [x] Converted all six remaining character wiki-links in `Theia/History/Memory Scenes.md` to direct Markdown paths for previously live-verified Conclave/Sixth Scholar character notes (commit `4e2164d8`).
- [x] Clarified that the 21 CR/DR labels in `Theia/Campaigns/Connected Racial Stories.md` are noncanonical historical references rather than unverified active quest links; removed duplicate current campaign link (commit `9fad5f39`).
- [ ] Continue other surviving historical wiki-links and source-content reconciliation; no claim of whole-vault validation.

## Approved memory-scene navigation migration — 3 November 2026

- [x] Converted `Theia/History/The Six Reunited.md` Obsidian frontmatter, character, institution, historical-event and index navigation to Markdown paths (commit `aab2b494`). The scene's approved status and disclosure caveats remain unchanged.
- [x] Directly checked the Corrupted Conclave institution and Binding history target files exist. The six scholar character targets had already been verified in prior cleanup.
- [ ] Continue other derived historical scenes and source links; this does not approve new dialogue or complete whole-vault link validation.

## Binding aftermath historical source-link repair — 3 November 2026

- [x] Replaced three obsolete imported-Chronicle section wiki-links in `Theia/History/The Binding and its Aftermath.md` with one root Chronicle reference, and added direct editorial decisions, current campaign and history navigation (commit `b3df7922`).
- [x] Kept First Years of Silence, Age of Reconstruction and Modern Age / Shattered Delegation as historical source-section labels without asserting unverified heading anchors.
- [ ] Continue checking remaining historical pages and backlinks; canonical modern Binding dissolution remains governed by the 1–80 campaign.

## Future Saga remaining imported-source links — 3 November 2026

- [x] Migrated three old `Sources/THEIA_*` section wiki-links in `Theia/Campaigns/Future Saga.md` to surviving root World Bible, Modern Story Revelation Timeline and Interwoven Chronicle documents (commit `f9364ae2`). Section descriptions remain historical provenance, not independently verified heading anchors or approved future expansion canon.
- [ ] Continue the repository-wide audit of other source-mirror references and unverified wiki-links before deleting this tracker or retiring the historical branch.

## Historical Timeline event-link retirement — 3 November 2026

- [x] Converted 13 historical milestone wiki-links in `Theia/History/Historical Timeline.md` into plain event labels, preserving the event order and descriptive claims (commit `f2793917`). Removed additional obsolete wiki-links in the present-day marker. Sampled First Sundering, Second Sundering and Shattered Delegation paths under `History/` were absent, but **not all 13 possible note locations were exhaustively searched**.
- [ ] Verify individual event-page locations and incoming links separately if needed; the timeline labels are now historical navigation without asserting live per-event file targets. Keep current 1–80 campaign as narrative authority.

## World Directory historical redirect status correction — 3 November 2026

- [x] Updated `Theia/Lore/World Directory.md` to reflect the earlier recorded 281/281 historical Markdown file-target check instead of incorrectly saying the converted archive had never been verified (commit `936d2ba7`). This is a correction to the redirect's status statement, **not a fresh 281-link scan**.
- [ ] Revalidate targets after any further deletions and separately check heading anchors and semantic currency. The World Directory archive remains historical provenance, not current quest authority.

## Three Atlas place source-link migrations — 3 November 2026

- [x] Migrated source, Atlas parent and current campaign navigation in `Atlas/Places/Lantern Redoubt.md`, `Outer Passage site.md` and `Northwatch Landing.md` to direct Markdown paths (commits `55b9b790`, `1faf73a2`, `0cbe7c60`). The original source remains `THEIA_HUMAN_CAST_AND_PLACES.md`.
- [x] Preserved each location's **Needs Review** status and working geography without asserting new travel routes or approved map coordinates.
- [ ] Continue reviewing remaining related-note wiki-links and verifying their actual targets; these three files are not a complete Atlas semantic audit.

## Atlas place related-note link verification — 3 November 2026

- [x] Live-verified seven unique related-note destinations across Lantern Redoubt, Outer Passage site and Northwatch Landing: Covenant, Dominion, Kel'vari, Thuma'rei, Drom Kar, Korrath and Tidehold; **7/7 file targets exist**.
- [x] Replaced those seven old Obsidian-style related-note links with explicit Markdown paths in the three place notes (commits `718f8ede`, `02dc99b8`, `47dc4adc`).
- [ ] Continue remaining Atlas and character references. Link existence does not approve working geography or settle map distances.

## Coastal and quarry Atlas reference migration — 3 November 2026

- [x] Migrated `Atlas/Places/Greyfall Quarry.md`, `Storm-coast depot.md` and `Relay coast.md` from obsolete Obsidian source/related-note links to Markdown links for the root cast-and-places source, Atlas, current campaign and adjacent places/peoples/institutions (commits `6c9e731d`, `1bf742c9`, `b833e03d`).
- [x] Live-verified five unique related destinations: Stoneford, Tidehold, Vorrath, Dominion and Vaelari. All five exist on `main`.
- [ ] Continue Atlas and historical semantic review. Their geography remains **Needs Review** and is not a newly approved travel plan.

## Chainworks and Heartlands place navigation — 3 November 2026

- [x] Migrated `Atlas/Places/Chainworks.md`, `Millhaven.md` and `Valecrown.md` from obsolete Obsidian source/parent/related links to Markdown links for the root cast-and-places source, Atlas, current campaign and related notes (commits `79fc4a7f`, `377ea42b`, `11e5d214`).
- [x] Live-verified four unique related destinations: The Unmade, Lantern Redoubt, Stoneford and Tidehold. All exist on `main`.
- [ ] Retain **Needs Review** geography and continue checking the remaining place notes and source-reference graph; no map placement was approved by these edits.

## Stoneford and allied capital Atlas navigation — 3 November 2026

- [x] Migrated `Atlas/Places/Stoneford.md`, `Tidehold.md` and `Kar Durn.md` to direct root cast-and-places source, Atlas parent and current Human campaign Markdown links (commits `2f3e577d`, `22f8d1b9`, `dee87ae4`).
- [x] Converted the Stoneford→Valecrown and Kar Durn→Tidehold related-note links; both targets were live-fetched during earlier cleanup.
- [ ] Continue remaining Atlas notes and full source-link audit. These files retain **Needs Review** geography and do not authorize map or travel changes.

## Woodland and highland Atlas link migration — 3 November 2026

- [x] Updated `Atlas/Places/Sylvaren.md`, `Drom Kar.md` and `Elderwood.md` to use root Human Cast and Places, Atlas and current Human campaign Markdown links instead of retired Obsidian source/parent references (commits `9c46346f`, `a33ba706`, `0999e2fc`).
- [x] Converted Sylvaren's Tidehold/Aelari and Drom Kar's Outer Passage site/Korrath related links; all four unique destinations were directly verified to exist on `main`.
- [ ] Keep their **Needs Review** geography status; finish other historical references, full-tree link checks and content reconciliation before branch/tracker retirement.

## Five Kel'vari enclave navigation migrations — 3 November 2026

- [x] Migrated Atlas parent, story-index, Kel'vari people and related campaign-cast wiki-links in the five enclave pages Veyr'athun, Oth'irun, Oru'sava, Thuma'rei and Soru'vaeth to direct Markdown links (commits `83de7b5a`, `778fcfcf`, `d8d4ea1c`, `a82dd23b`, `a1b8e349`).
- [x] Live-verified the Kel'vari people note and Sel'ora, Leth'ari and Aru'ven campaign-cast targets. Kept enclave identities and source-specific approval status unchanged.
- [ ] Continue wider Atlas/lore link validation and historical semantic reconciliation before final tracker deletion and branch retirement.

## Institution World Bible path correction — 3 November 2026

- [x] Corrected root World Bible paths in three institution notes under `Theia/Lore/Institutions/`: Guardians, Unmade and Corrupted Conclave. Their former `../../../../THEIA_WORLD_BIBLE.md` links escaped the story folder; correct relative path is `../../../THEIA_WORLD_BIBLE.md`. Commits `265d02e5`, `6acd9425`, `603415c6`.
- [x] Replaced retired imported-source references and parent navigation in all three. Guardians related character/institution links were also migrated.
- [ ] Continue the remaining Unmade/Conclave related-note wiki-links, then revalidate the three documents and broader repository paths. No semantic canon approval follows from path correction.

## Institution related-link completion — 3 November 2026

- [x] Verified 10 unique character, people and institution targets referenced by the Guardians, Unmade and Corrupted Conclave pages; all 10 exist on live `main`.
- [x] Replaced remaining institution/founder/Kel'vari wiki-links in `Theia/Lore/Institutions/The Unmade.md` and `The Corrupted Conclave.md` with direct Markdown paths (commits `150d9964`, `01d15f7c`). Nhal is retained as a plain historical name rather than an unverified file target.
- [ ] Continue full-tree reference and semantic checks before branch and tracker retirement.

## Divine character source-path audit — 3 November 2026

- [x] Found obsolete `../../Sources/THEIA_WORLD_BIBLE.md` source metadata across the seven `Theia/Characters/Divine/` notes; the surviving root story World Bible is reached from that folder via `../../../THEIA_WORLD_BIBLE.md`.
- [x] Corrected The Keeper and The Guardian source metadata, source-account links, parent navigation and historical Binding reference (commits `76cdaba1`, `b0c2f403`). Their confirmed-foundation/details-open statuses remain unchanged.
- [ ] Apply the same verified path correction to Judge, Warrior, Seeker, Shaper and Ascendant, then verify their related-note links. Do not claim all seven complete yet.

## Seven divine source paths completed — 3 November 2026

- [x] Corrected the remaining five divine character source metadata and source-account links to the root World Bible: Judge (`5e2d0dc8`), Warrior (`05dce4eb`), Seeker (`9d5d7e34`), Shaper (`5269cb17`) and Ascendant (`8528b332`). Keeper and Guardian were corrected in the preceding batch. All seven now use `../../../THEIA_WORLD_BIBLE.md` and link to the Seven Gods topic index.
- [x] Preserved confirmed-foundation/details-open status and original creator/patron descriptions; no divine outcome was changed.
- [ ] Continue related-note wiki-link target checks and the remaining full repository audit before retiring the old branch and tracker.

## Seven divine related-note links completed — 3 November 2026

- [x] Verified all nine unique peoples' file targets referenced by the seven divine character notes: Korrath, Halflings, Humans, Aelari, Orcs, Vaelari, Dwarves, Vorrath and Kel'vari. **9/9 exist** on live `main`.
- [x] Replaced all `Lore/Peoples/...` Obsidian wiki-links across Keeper, Guardian, Judge, Warrior, Seeker, Shaper and Ascendant notes with explicit relative Markdown links (commits `22c31123`, `4515aefd`, `670799fb`, `b293ad67`, `89cc1b3b`, `5dbcf195`, `14d9df57`). Ascendant's Keeper and Kel'vari references were also clarified.
- [ ] Continue remaining source/backlink and content audits. No divine lore or runtime quest data changed.

## Nine proposed service NPC source migrations — 3 November 2026

- [x] Replaced obsolete `Sources/THEIA_HUMAN_CAST_AND_PLACES` source metadata/embeds and Obsidian parent/location navigation in all nine `Theia/Characters/Services/` cards: Cal Norr, Oda Flint, Adra Pell, Evin Reed, Marta Venn, Joric Hale, Sella Marr, Torren Vale and Sabren Holt. Commits `44929a9c`, `6e83cf6c`, `2b21b8a9`, `d0c4f73a`, `3af413ba`, `ab3a67dd`, `b131063c`, `ffdd1ce5`, `65f2010c`.
- [x] Live-fetched surviving root Human Cast and Places, Characters index, Stoneford and Valecrown destinations. All four exist. Preserved all nine **Needs Review** labels and proposed service roles; no approved NPC or class roster created.
- [ ] Continue broader character backlinks and full-tree link/content validation before branch retirement or tracker deletion.

## First Archon Selrion source migration — 3 November 2026

- [x] Migrated `Theia/Characters/First Archon Selrion.md` off obsolete `Sources/THEIA_HUMAN_CAST_AND_PLACES` metadata and source wiki-links to the root cast-and-places document, with direct Characters, story index, Vaelari, Covenant and Dominion links (commit `9740c8b3`).
- [x] Live-fetched the Vaelari, Covenant and Dominion related files; all three exist. Preserved Selrion's approved role and the restriction against inventing friendly Covenant meetings.
- [ ] Continue remaining character/campaign backlinks and full-tree validation. GitHub search may retain stale indexed paths; verify apparent obsolete references with fresh file fetches.

## Covenant and Dominion institution path repair — 3 November 2026

- [x] Corrected `Theia/Lore/Institutions/Covenant.md` and `Dominion.md` root World Bible source paths from the obsolete four-parent form to `../../../THEIA_WORLD_BIBLE.md`, and migrated Lore/Story navigation (commits `76f06831`, `6713c9fb`).
- [x] Live-verified ten unique related race/leadership file targets: Humans, Dwarves, Aelari, Korrath, Orcs, Halflings, Vaelari, Vorrath, Corven Vaelmont and First Archon Selrion; all exist. Replaced their wiki-links with direct Markdown paths and linked the two institutions to each other.
- [ ] Continue the remaining institution, character and repository-wide link checks. No alliance or canon outcomes changed.

## Campaign cast navigation follow-up — 3 November 2026

- [x] Corven Vaelmont: migrated parent/current campaign and Tidehold/Kel'vari links (commit `21d1ad5d`).
- [x] Sel'ora, Aru'ven and Leth'ari: migrated parent/current campaign, Kel'vari and associated enclave links (commits `09ac468f`, `16be2e3e`, `6d1ebbac`). Their approval statuses remain unchanged.
- [ ] Continue remaining campaign cast navigation and semantic audit. This tracker remains temporary.

## Core Human cast navigation — 3 November 2026

- [x] Migrated Mara Thorne, Darian Cole and Elara Fenwick campaign cast notes to direct Characters/current campaign navigation and verified Valecrown/Millhaven/Elara related file paths (commits `12b56934`, `3af13d18`, `596f9bf6`).
- [x] Preserved approved starting roles and **Needs Review** development/outcome status; no canonical dialogue or quest data changed.
- [ ] Continue remaining campaign cast cards and broader backlink/content checks.

## Heartlands cast navigation retry completed — 3 November 2026

- [x] Successfully migrated Brenna Holt, Sister Anwen, Heddan Pike and Perrin Vale campaign cast notes to direct Characters/current campaign and Millhaven/Stoneford Markdown links after an earlier blocked attempt (commits `ba0f7394`, `2ca84611`, `d8402c8e`, `20188814`).
- [x] Preserved existing approved-role versus **Needs Review** distinctions and character development text. No quest data or canonical outcomes changed.
- [ ] Continue remaining Campaign Cast cards and repository-wide historical link/content checks.

## Civic and regional cast navigation completed — 3 November 2026

- [x] Migrated Tomas Greymere, Mera Reed, Nera Holt and Joss Arden character notes to direct Characters/current campaign and related Millhaven/Stoneford/Brenna links (commits `a6834e2f`, `1575d30a`, `15b4cb0b`, `ce4537aa`). Previously blocked edits were retried successfully.
- [x] Preserved existing approval statuses and Nera Holt's working surname caveat; no kinship or new canon established.
- [ ] Continue other Campaign Cast notes and full repository backlink/content validation.

## Aelari campaign cast navigation — 3 November 2026

- [x] Migrated Queen Seralyth, Caelith Rowan and Veylan Istren character cards to direct Characters/current campaign and Aelari people links after earlier blocked update (commits `c7647d35`, `d3bf464c`, `e548150a`).
- [x] Preserved Queen Seralyth's approved role and the two other characters' **Needs Review** status; no new relationship with Selrion or new canon was established.
- [ ] Continue remaining campaign cast and repository-wide historical link/content checks.

## Korrath and dwarf campaign cast navigation — 3 November 2026

- [x] Migrated Urodan and Tarek Oss to direct Characters/current campaign and Korrath links (commits `14422c49`, `2a7ab3f0`).
- [x] Retried and completed Rurik Anvilmark's Characters/current campaign and Dwarves links (commit `d3918065`).
- [ ] Continue remaining Campaign Cast cards and full repository semantic/link checks; existing **Needs Review** statuses remain unchanged.

## Port and expedition cast navigation completed — 3 November 2026

- [x] Migrated Ysra Keel, Iven Rusk and Sera Venn to direct Characters/current Human campaign navigation, including Tidehold and Covenant related-note links (commits `d8a3d019`, `28ec6efd`, `dd7ea3ee`). Previously blocked edits were retried successfully.
- [x] Preserved each **Needs Review** status and established role distinctions; no quest or canon outcomes changed.
- [ ] Continue remaining Campaign Cast notes and final repository-wide historical/backlink verification.

## Military, civic and dwarven cast navigation — 3 November 2026

- [x] Retried and completed Garrick Ashford, Halden Marr, Lysa Marden and Edda Flintwake character navigation to direct Characters/current campaign and related Valecrown, Covenant, Dwarves and Brynja links (commits `022c2bc7`, `030975a0`, `8d0039cd`, `ff20c7c9`).
- [x] Preserved Garrick's noninvolvement in sabotage, Halden's incomplete knowledge, Lysa's limited archive access, and Edda/Brynja's no-assumed-kinship restriction. No canonical outcomes changed.
- [ ] Continue remaining cast and repository-wide link/content verification.

## Dwarf and historical antagonist cast navigation — 3 November 2026

- [x] Retried and completed Brynja Flintwake, Varric Sorn and Halvek Dorn character navigation to direct Characters/current campaign and relevant Dwarves, Edda, Covenant and Unmade references (commits `4cd1ed96`, `c93aaada`, `eda2390f`).
- [x] Preserved all **Needs Review** status and no-assumed-kinship limits; historical antagonist roles do not override the canonical 1–80 campaign.
- [ ] Continue remaining cast and repository-wide historical link/content checks.

## Three legacy antagonist character outcomes clarified — 3 November 2026

- [x] Updated Teren Malrec, Maelin Voss and Sareth Keld to direct Characters/current Human campaign and Unmade/Chainworks navigation (commits `07b7ae60`, `fea09014`, `10ffeb7d`).
- [x] Added explicit historical-candidate caveats: their old Act IV/V custody, capture and finale outcomes are not approved current 1–80 campaign canon. Retained their source provenance and **Needs Review** status for possible optional use.
- [ ] Continue remaining Campaign Cast and source/backlink reconciliation. No active quest definitions changed.

## Dominion cast navigation follow-up — 3 November 2026

- [x] Updated Ilyss Varen's current campaign, Characters, Vaelari, Dominion and Unmade links (commit `befe8ad3`). Her independent research does not establish defection or faction cooperation; **Needs Review** remains.
- [ ] Continue remaining cast and full repository source/link checks.

## Scholar, Aelari and Halfling cast navigation — 3 November 2026

- [x] Migrated Master Oren Bell, Irielle Mossward and Nemi Brindlewick character cards to direct Characters/current campaign and related location, people or institution Markdown paths (commits `d8137c26`, `58bcce0e`, `97adf2d1`).
- [x] Preserved Oren's non-Guardian scholarly status, Irielle's lack of hidden Guardian rank, and Nemi's Dominion independence/no-friendly-trade and no-assumed-kinship boundaries.
- [ ] Continue remaining Campaign Cast and full-tree backlink/content audit. GitHub text search can return stale results; direct file fetches govern completion status.

## Royal court character navigation and legal-status clarification — 3 November 2026

- [x] Retried and completed King Aldren Vaelmont, Lord Cassian Veyr and Lord Edric Halewyn direct Characters/current campaign navigation (commits `38346bae`, `1e1d0a3a`, `eb047fad`). Aldren's Valecrown, Cassian and Covenant related links were also migrated.
- [x] Explicitly marked Cassian's conviction/permanent imprisonment as a **Needs Review historical proposal**, not approved current 1–80 canon. Preserved his approved starting role and Edric's death/legacy boundaries.
- [ ] Continue remaining Campaign Cast and repository-wide source/link/semantic checks.

## Leadership cast navigation — 3 November 2026

- [x] Updated First Elder Dromak, High Thane Bromund Stonevein and Warden Shorvek Tal links (commits `83eb22ea`, `7432bc22`, `1b069fa6`).
- [x] Preserved Dromak's council constraints, Bromund's working-name status and Shorvek's coastal sovereignty.
- [ ] Continue remaining cast and repository-wide validation.

## Northwatch and Dominion officer navigation — 3 November 2026

- [x] Migrated Commander Nessa Ward, Captain Rukhan Graal and Lieutenant Corra Baines to direct Characters/current campaign, Northwatch, related officer, Orcs and Covenant/Dominion/Unmade links (commits `7e510db4`, `25cbd532`, `ab44f700`).
- [x] Preserved their **Needs Review** status, Rukhan's non-Unmade default and Great Clan terminology, and Corra's independent garrison continuity role.
- [ ] Continue remaining character records and final repository audit.
