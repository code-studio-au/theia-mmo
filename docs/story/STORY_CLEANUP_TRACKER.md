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
