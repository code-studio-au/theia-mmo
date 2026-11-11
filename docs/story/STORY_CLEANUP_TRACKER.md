# Story cleanup tracker — TEMPORARY, DELETE WHEN COMPLETE

> **Lifecycle: DELETE THIS FILE only after all acceptance criteria below are satisfied and verified on main.** This is a disposable progress tracker, not a canon source or permanent report. Git history preserves its audit trail.

Status: **IN PROGRESS — 3 November 2026.** This compaction removes repeated intermediate reviews and stale remaining-item snapshots, not unfinished work. The [full pre-compaction tracker](https://github.com/code-studio-au/theia-mmo/blob/6e9fddfec838efff5f0968def98e506c15ce57cc/docs/story/STORY_CLEANUP_TRACKER.md) preserves every prior checkpoint, filename, commit and source-specific observation. Historical validations are not a current completeness claim.

## Authoritative destinations

- [Story README](README.md#approval-and-precedence): hierarchy, classifications and source interpretation.
- [Story decisions](DECISIONS.md): creator decisions and supersession.
- [World Bible](THEIA_WORLD_BIBLE.md): established world facts.
- [Interwoven Chronicle](THEIA_INTERWOVEN_CHRONICLE.md): historical narrative; proposed connective material is not automatically locked.
- [Canonical Human 1–80 campaign](Theia/Campaigns/Human_1_80_Canonical_Campaign.md): approved progression, revelations and outcomes.
- [Current Human Campaign](Theia/Campaigns/CURRENT_HUMAN_CAMPAIGN.md): reading order and status.
- [Consolidated quest manifest](Theia/Campaigns/Human_1_80_Current_Quest_Authoring_Manifest.json): sole working 184-quest inventory, not executable Rust.
- [Legacy Ideas Archive](Theia/Campaigns/Human_Legacy_Sidequest_Zone_Character_Archive.md): unique noncanonical possibilities and historical H/HS evidence.
- [Source Decision Recovery](THEIA_SOURCE_DECISION_RECOVERY.md): historical question/page evidence and audit limits.

## Removal rule

Assign KEEP, MERGE, RETIRE or BLOCKED for each remaining target. Compare the entire candidate, preserve genuinely unique information with approval status and provenance in its proper existing home, repair incoming references, validate, commit on a dedicated branch and obtain independent review before integration. Preserve the protected Bible, Chronicle, canonical campaign and manifest. Do not promote proposals, infer runtime mappings or alter permanent outcomes. The creator expanded scope on 4 November to reconcile approved quest changes in the JSON manifest and perform dependency/canon checks. Gameplay code and CI remain outside scope; authoring JSON is not executable quest content.

## Completed / evidence

These are integrated outcomes, not a claim that all remaining documents are consistent. Linked PRs carry diffs, review context and merge evidence; older individual commits remain in the full historical tracker.

| Area | Completed result | Evidence |
|---|---|---|
| Imported duplicates and navigation | Reconciled and retired the imported Bible, Chronicle, Detailed Lore Archive, reconciliation report, decisions and source-recovery mirrors; duplicate import manifest and Sources README retired. Root provenance retained. Historical reading maps, H/HS cards and obsolete vault navigation were retired or repaired. The current pinned tree contains no Sources or Editorial files. | Historical commits 4ed71c05, 4550b1e4, e33bf37d, 63a58cb7, 54e81ff8, c8f77fba, 5bde970f, ad728521; [full history](https://github.com/code-studio-au/theia-mmo/blob/6e9fddfec838efff5f0968def98e506c15ce57cc/docs/story/STORY_CLEANUP_TRACKER.md) |
| Authority and finale | Established hierarchy/status definitions; reconciled later ending locks and old October 29/30 wording; corrected premature Oth'varen death at Q178 without changing Q179 farewell or Q181 death. | [PR3](https://github.com/code-studio-au/theia-mmo/pull/3), [PR4](https://github.com/code-studio-au/theia-mmo/pull/4), [PR6](https://github.com/code-studio-au/theia-mmo/pull/6) |
| Quest supporting duplicates | Dependency review consolidated into authoring contract; all eight range migration notes consolidated into existing reviews, retaining source rows and unique QA/continuity context. Historical H/HS mappings remain unapproved. | PRs [5](https://github.com/code-studio-au/theia-mmo/pull/5), [7](https://github.com/code-studio-au/theia-mmo/pull/7), [8](https://github.com/code-studio-au/theia-mmo/pull/8), [10](https://github.com/code-studio-au/theia-mmo/pull/10), [11](https://github.com/code-studio-au/theia-mmo/pull/11), [13](https://github.com/code-studio-au/theia-mmo/pull/13), [14](https://github.com/code-studio-au/theia-mmo/pull/14), [15](https://github.com/code-studio-au/theia-mmo/pull/15), [18](https://github.com/code-studio-au/theia-mmo/pull/18) |
| Cast and history | Root cast/place packet retired; 73 cards retain provenance and unique constraints. Six scholar histories retained; seven approved court foundations preserved; outcome duplicates point to authority. Delegation/network sequence and founder-order supersession corrected. | PRs [9](https://github.com/code-studio-au/theia-mmo/pull/9), [12](https://github.com/code-studio-au/theia-mmo/pull/12), [16](https://github.com/code-studio-au/theia-mmo/pull/16), [19](https://github.com/code-studio-au/theia-mmo/pull/19), [20](https://github.com/code-studio-au/theia-mmo/pull/20), [25](https://github.com/code-studio-au/theia-mmo/pull/25) |
| Protected world references | Bible retained and scoped to current campaign; public First Anchor authorization distinguished from secret alteration. Chronicle narrative retained, future appendix ideas archived. Eighteen duplicate lock copies consolidated into Bible links. | PRs [19](https://github.com/code-studio-au/theia-mmo/pull/19), [23](https://github.com/code-studio-au/theia-mmo/pull/23), [29](https://github.com/code-studio-au/theia-mmo/pull/29), [31](https://github.com/code-studio-au/theia-mmo/pull/31) |
| Legacy possibilities | Future Saga and level 35–40 prototype retired with unique ideas/numerical assumptions preserved. Repeated HS and H tables consolidated; detailed lore sections 7–8 catalogue moved to the existing archive. No future outcome approved. | PRs [21](https://github.com/code-studio-au/theia-mmo/pull/21), [22](https://github.com/code-studio-au/theia-mmo/pull/22), [30](https://github.com/code-studio-au/theia-mmo/pull/30), [32](https://github.com/code-studio-au/theia-mmo/pull/32), [35](https://github.com/code-studio-au/theia-mmo/pull/35) |
| Source evidence | Reconciliation and line-verification reports retired into source index with unique evidence/limits. Sixty Q376–435 rows and nine unanswered Q436–444 prompts consolidated. Stale open-canon lists corrected; no new PDF audit claimed. | PRs [28](https://github.com/code-studio-au/theia-mmo/pull/28), [37](https://github.com/code-studio-au/theia-mmo/pull/37), [39](https://github.com/code-studio-au/theia-mmo/pull/39), merge 6e9fddfec838efff5f0968def98e506c15ce57cc |
| Narrative and location consistency | Canonical summaries differentiated; Q115 Su'vareth outcome verified, old Malrec/projection conflict resolved. Dialogue reveals and stage boundaries corrected; H4-05/Q092 record-help claim corrected to copyist rescue. Act I foundation KEEP, Heartlands references reconciled, four location cards corrected; settlement service proposals aligned with locked transport. | [Historical summary/Q115 evidence](https://github.com/code-studio-au/theia-mmo/blob/6e9fddfec838efff5f0968def98e506c15ce57cc/docs/story/STORY_CLEANUP_TRACKER.md), PRs [17](https://github.com/code-studio-au/theia-mmo/pull/17), [24](https://github.com/code-studio-au/theia-mmo/pull/24), [26](https://github.com/code-studio-au/theia-mmo/pull/26), [27](https://github.com/code-studio-au/theia-mmo/pull/27), [33](https://github.com/code-studio-au/theia-mmo/pull/33), [34](https://github.com/code-studio-au/theia-mmo/pull/34), [36](https://github.com/code-studio-au/theia-mmo/pull/36), [38](https://github.com/code-studio-au/theia-mmo/pull/38) |

- [x] [PR41](https://github.com/code-studio-au/theia-mmo/pull/41) compacted the temporary tracker while preserving full historical evidence, genuine blockers and acceptance criteria.
- [x] [PR43](https://github.com/code-studio-au/theia-mmo/pull/43) retired the Detailed Lore Archive after unique-source comparison and preservation in Bible/source index/Legacy Ideas Archive; 19 backlink documents repaired and source-specific cards retain immutable provenance. Merge efc76763a72163b6aaefe169812ecf0827c2b60d verified.
- [x] [PR44](https://github.com/code-studio-au/theia-mmo/pull/44) retired the historical World Directory after label preservation and a complete zero-backlink scan; useful navigation now lives in subject indexes. Merge ebbaa3c44cbf01db4aab0c8aa5708d5550db94b8 verified.
- [x] [PR45](https://github.com/code-studio-au/theia-mmo/pull/45) clarified retained document roles and source limits; merge 22ee99a76609fef2d7405f0b9c084d061266aa9d verified. This is not blanket semantic certification.

## Validation checkpoints, not final certification

- **Fresh checkpoint at main 22ee99a76609fef2d7405f0b9c084d061266aa9d:** all 359 Markdown files fetched at that commit; main rechecked unchanged. All 3,069 relative file links and 87 local anchors passed (story subset 1,333 file links / 82 anchors), plus 168 local source/up metadata references. No residual wiki links found.
- Canonical campaign has 184 unique quest IDs; eight authoring reviews contain 184 unique rows with no gaps. All 156 available review titles match; the other 28 rows are ID-only. These Markdown checks do not establish manifest schema, complete dependency/stage equivalence or runtime readiness.
- **Historical dialogue checkpoint at 22ee99a:** 180 unique quest sections, then missing Q014–Q017 and completion speech for Q013 Elara, Q031 Elara, Q053 Caelith and Q183 Kel'vari report. No quoted Nhal reveal was found before Q143 in that checked corpus; this was a targeted reveal check, not certification of every presentation path.
- **Dialogue coverage update at ad2f17093b806413420802313da64d48617ac542:** 184 unique Q001–Q184 section headings now exist. Q014–Q017 and the four missing completion/report passages above have WORKING drafts. This heading count includes Q169's explicitly SUPERSEDED transport-survey text, not current journal-quest dialogue; replacement Q169 speech is still unwritten. Draft existence is not creator approval or full semantic validation.
- All 92 H and 24 HS source rows are explicitly ARCHIVED. No open PRs existed at the checkpoint. Prior retirements received independent preservation/link review. The checkpoint was subsequently reviewed and integrated in [PR46](https://github.com/code-studio-au/theia-mmo/pull/46); that former pending review is complete.
- Source evidence records 444 distinct question identifiers / 471 numbered prompt occurrences, 418 identifiers with a source-located answer or confirmed later variant, and 26 open/deferred identifiers. Reused question numbers require stage/page context. Original PDF and complete external audit artifacts are not in the repository; exhaustive independent PDF verification is not certified.
- Legacy branch story/launch-main-story-1-60 was previously 0 commits ahead and 534 behind main. This was an ancestry checkpoint, not content-equivalence proof or permission to delete it.

### 4 November authoring-JSON reconciliation

At base ad2f17093b806413420802313da64d48617ac542, the approved title, access, contact and canonical objective corrections were reconciled in the sole manifest and its derived dependency reference. Validation found 184 quests, 685 stages, 687 objectives, 158 prerequisite edges, no duplicate or unresolved IDs, acyclic progression and no main-story dependency on the 28 provisionally optional quests. Every prior stage/objective ID remains; added objectives represent already-canonical ward safety, hold permission and Corven proposal delivery. All 184 titles agree with the canonical campaign. Q174's level-80 requirement applies at finale authorization, not an inferred acceptance gate. These are authoring-shape and targeted canon checks, not a published JSON Schema or executable Rust certification. Q169's death-event cutoff is now approved; its post-conversation unfinished custody edge case and detailed presentation remain unresolved; all runtime IDs and stable actor IDs remain unassigned.

## Remaining work and exact blockers

- [x] Remaining distinct document roles have KEEP-by-purpose dispositions in the [existing guide](THEIA_DOCUMENTATION_GUIDE.md#retained-document-roles-and-disposition); known redundant compilations and navigation were retired. No additional concrete duplicate target was identified at this checkpoint. KEEP is not blanket approval or proof that every statement has been semantically rechecked. The [PR40](https://github.com/code-studio-au/theia-mmo/pull/40) location appendix retirement preserved 80 differing associations as 65 site labels and repaired nine backlinks.
- [ ] **Final semantic acceptance:** recorded contradiction repairs and targeted reveal checks are complete, but mechanical link/row coverage alone does not prove full Bible/Chronicle/campaign equivalence, every stage/dependency, evidence condition or solo path. Resolve a specific discovered discrepancy through its existing authority rather than repeat already completed reviews or invent a new canon decision.
- [ ] **Future dialogue authoring and review:** Q014–Q017, Q013 Elara, Q031 Elara, Q053 Caelith and Q183's separate Kel'vari/Covenant reports now have WORKING drafts. Q183's report recipients and separate-report requirement are creator-approved; exact speech remains WORKING. Q169 still needs replacement dialogue for the approved journal-discovery story; its old transport-survey lines remain explicitly SUPERSEDED. Review supporting speech against existing canon without treating draft additions as new locked objectives, staging or lore.
- [x] **Completion contact decision resolved:** the creator approved Sera as the turn-in contact for Q176 and Q181 after their final objectives; see the [decision ledger](DECISIONS.md#2026-11-03--locked-q176-and-q181-turn-in-contact) and [Q158–Q184 review](Theia/Campaigns/Human_Q158_Q184_Rust_Authoring_Review.md#locked-completion-contacts-after-permanent-deaths). Deaths and finale sequence are unchanged. JSON authoring contact labels now reflect this decision; runtime actor binding and implementation remain uncertified.
- [ ] **Authoring manifest acceptance boundary:** creator-authorized JSON reconciliation records approved titles, Q156 optional access, Q174's canonical cap, finale turn-ins and report recipients without assigning runtime IDs. Structural and targeted semantic checks are recorded below; executable Rust/schema integration, stable actor binding, rewards and quest-specific item/choice semantics are not certified. Q169 becomes unavailable at Oth'varen's Q181 death, expiring already accepted quests without his completed conversation and preserving completed quests. Exact message wording, runtime expiry support and the completed-conversation/unfinished-custody edge case remain unresolved. No resurrection or replacement contact is approved.
- [x] **Historical identifiers archived:** [PR42](https://github.com/code-studio-au/theia-mmo/pull/42) establishes ARCHIVED preservation for all 92 H and 24 HS source concepts. The root [H/HS register](THEIA_HUMAN_QUEST_REGISTER.md) and [exact-title comparison](Theia/Campaigns/Human_Legacy_64_Exact_Title_Objective_Comparison.md) are KEEP as historical evidence, not active inventories. No runtime mapping is approved.
- [ ] **Future optional reuse, not missing archival coverage:** Specific contact/access/consequence differences still require resolution before reuse, including H2-04 progression edge, H2-05 measuring-assembly/hold permission, four historical contact discrepancies and eight objective details. Canonical Q004/Q009–Q011/Q013/Q018–Q019/Q021/Q027/Q031 already establish teamster survival, damaged document, authentication, protected travel, interview/court authority, chronology, Garrick's innocence-of-involvement distinction, intended-killing link and office-removal/non-absolution. Historical flags do not reopen these facts or block archival preservation.
- [x] **Distinct titles approved:** Q128 is **The Warden's Preparations** and Q154 remains **Before the Veil Breaks**, as [approved by the creator](DECISIONS.md#2026-11-04--locked-distinct-q128-and-q154-titles). Canonical and supporting Markdown titles agree; objectives, quest IDs and preparation phases are unchanged. JSON authoring/dependency titles now agree; runtime export remains unimplemented.
- [x] Known unique sidequest/zone/character/faction/Dominion/encounter/future ideas are preserved in the one Legacy Ideas Archive; duplicate source compilations and repeated H/HS tables were consolidated. Archive concepts remain unapproved. Further consolidation requires a specific remaining duplication, not another blanket review; old Keld/Voss, Malrec/projection and alternate Nhal finales remain superseded.
- [x] Complete current-tree Markdown link, anchor, source/up metadata, residual-wiki and quest-row checks passed at the pinned checkpoint above. The PR46 tracker-only delta was verified after integration. Later changes require their own scoped checks; this checkpoint does not certify non-Markdown structures or every narrative condition.
- [x] **Subsequent integration verified:** PR46 merged as `dd437996c0e26775cc1abc503c73c01933c249e1`; [PR47](https://github.com/code-studio-au/theia-mmo/pull/47) corrected Q063's assignment-before-access boundary (`64bcaec97815007fa9cb7b3a4024dff4a3f40452`); [PR48](https://github.com/code-studio-au/theia-mmo/pull/48) recorded the creator's Sera turn-ins (`95f06d9abea258f4b38d42337e7caea7addf5130`). These verified merges close the old pending items, not the remaining acceptance criteria. Use the final review/merge criterion below for closure rather than opening a new task to record each tracker edit.
- [ ] Retain historical branch/provenance pending verified disposition; any branch deletion needs its own authorized action. Old requests to inspect imported .agents/.copilot configuration are outside the current story-Markdown scope, not permission to inspect or remove them.
- [ ] Produce final repository integrity summary naming authoritative homes, intentionally OPEN creative decisions and future development work; then delete this tracker and repair its incoming links only when all acceptance criteria are satisfied.

## Definition of done

All boxes remain open until final current-main verification, even where substantial work is already complete.

- [ ] Authority hierarchy clear and consistently applied.
- [ ] Creator-locked facts distinguishable from canonical narrative and WORKING/OPEN proposals.
- [ ] No unresolved known contradictions among Bible, Chronicle and canonical Human campaign.
- [ ] Sole 184-quest manifest structurally consistent with approved narrative; no unapproved quest/outcome changes.
- [ ] H/HS material reconciled or explicitly archived without unapproved runtime mappings.
- [ ] Duplicate and superseded active documents retired after unique-content preservation.
- [ ] Distinct future possibilities preserved without unnecessary duplicate descriptions.
- [ ] Internal links and references pass latest validation.
- [ ] No unintended Rust/gameplay, schema, runtime-ID or unrelated-project changes.
- [ ] All cleanup PRs reviewed, merged and verified.
- [ ] Final integrity summary complete, including intentionally OPEN decisions and future work.
- [ ] Temporary tracker deleted with incoming references repaired.

After completion, stop cleanup cycles and maintain the established sources rather than create competing ones.


## 2026-11-12 — Full conversation-to-repository reconciliation audit (IN PROGRESS)

**Authority:** Creator-approved decisions recorded in [DECISIONS.md](DECISIONS.md#2026-11-12--covenant-racial-openings-tidehold-handoff-and-cast-review--creator-locked-documentation) and the current canonical campaign. **Do not treat this audit as approval of unmade racial 1–20 plots or as permission to invent missing biographies.** Review only the scope of approved details; preserve stable quest IDs and prior outcomes.

### Initial verified findings on `main`

- [x] Decision ledger includes the new Covenant racial handoff, human cast, Bromund, Oth'varen/Father, and race-history direction.
- [x] Canonical campaign Q035 is `The Covenant Calls`, Q036 is the shared Tidehold briefing, Q184 has a race-specific homecoming, and Q053 names the High Thane.
- [x] Individual character notes updated for Darian, Brenna, Anwen, Aldren, Cassian, Corven, Veylan, Bromund and Oth'varen; Dwarves, Aelari, Vaelari and Korrath people notes updated.
- [ ] **BLOCKING — Structured quest inventory:** `Theia/Campaigns/Human_1_80_Current_Quest_Authoring_Manifest.json` still contains old Q035 title and legacy `Grey Surveyor` / `High King` strings. Reconcile Q035 race-aware prerequisites, Q036 briefing and Q184 branches carefully, preserving all stages/evidence semantics.
- [ ] **BLOCKING — Quest dependency inventory:** `Theia/Campaigns/Human_1_80_Quest_Dependencies.json` still contains old Q035 title and legacy titles. Verify Q035 is not gated by human Q031 or Cassian's ledger and has a correct per-race completion gate.
- [ ] **Dialogue:** `Theia/Campaigns/Human_20_30_NPC_Dialogue_Scripts.md` still has old Q035 Cassian-ledger introduction and old `Grey Surveyor` / `High King` references. Review later dialogue packs and spoiler timing before replacements.
- [ ] **Canon consistency:** `Theia/Campaigns/Human_1_80_Canonical_Campaign.md` still contains older `Grey Surveyor` and `High King` references beyond the Q053 change. Determine each reference's context (historical vs current player-facing) before editing.
- [ ] **Race/culture review:** Reconcile reciprocal Aelari/Vaelari and Korrath/Vorrath ancestral directions with the World Bible, Chronicle, race profiles and historic schism timing. Neither modern alliance existed at the ancient schisms. Review Vorrath note specifically.
- [ ] **Character coverage:** Check remaining cast files and scene scripts for approved motivations, dialogue and relationships from the conversation. Avoid inventing details not actually recoverable.
- [ ] **Narrative and reveal timing:** Confirm Guardians' secrecy, Father alias, Oth'varen's hidden transformation, Harbinger name timing, incremental resonator discovery, forgotten Kel'vari, divine absence, Shattered Delegation accounts and race-specific dialogue across all relevant files.
- [ ] **Final validation:** Cross-check quest IDs, prerequisites, dialogue, old/new names and later epilogues; report anything not fully committed or implemented. Only mark complete after concrete verification.

**Audit scope note:** The current project conversation includes many spoken approvals. A full assertion of 100% coverage requires comparing all available approval records and their detailed source conversations, not just this initial targeted file scan. This tracker records confirmed gaps rather than asserting completeness.

### Reconciliation progress — 12 November 2026

- [x] Q035/Q036 canonical, dependency reference, structured authoring manifest and 20–30 dialogue now use the universal Tidehold summons and shared Corven briefing. Human Q031 no longer gates Q035 in the derived dependency reference or authoring manifest. **Per-race completion flags remain unassigned** because the three other racial campaigns are not yet authored; this is a design/implementation dependency, not a claim of executable readiness.
- [x] Q032–Q034 optional human aftermath now require Q031 in the structured reference and manifest without gating Q035.
- [x] Q053 uses High Thane terminology in the canonical register, derived reference, authoring manifest and 20–30 dialogue.
- [x] Q097 and pre-reveal Guardian references use Father instead of Grey Surveyor in the current canonical register and structured inventories. Historical archives may retain the superseded label as provenance.
- [x] Q184 has race-specific homeland/contacts in the canonical register and structured inventories, with the shared quest ID preserved. Exact non-human NPC bindings remain OPEN.
- [x] Vorrath ancestry note updated to complement Korrath and Aelari/Vaelari reciprocal origin directions without placing modern factions in the ancient past.
- [x] Re-fetched and JSON-parsed both updated inventories; both retain 184 quest records and Q035's new title.
- [ ] **Unresolved: all older narrative/dialogue references.** Other dialogue packs, Chronicle, World Bible, character cross-links, legacy registers and archival sources require context-aware checks. Do not globally replace historical quotations or turn unapproved interpretations into canon.
- [ ] **Unresolved: exact spoken-approval coverage.** Earlier character-by-character discussion contains approvals beyond those summarized in the ledger; verify source details before asserting all individual backgrounds, motivations and dialogues are fully reflected.
- [ ] **Unresolved: engine and race integration.** Implement real race-specific Q035 eligibility, Q184 actors/locations, quest catalogue migration, dialogue branching and end-to-end tests when the other racial campaigns and engine bindings exist.

### Further source and narrative audit — 12 November 2026

- [x] World Bible now explicitly carries the approved racial campaign boundary, reciprocal schisms, Guardian secrecy, Father alias, player reveal timing, political leadership and race-specific epilogue.
- [x] Current 10–20 and 70–80 NPC dialogue packs were fetched and checked for the exact superseded labels `Grey Surveyor` and `High King`; neither label appears in those two packs.
- [ ] **Chronicle intentionally not modified:** `THEIA_INTERWOVEN_CHRONICLE.md` still has 28 `Grey Surveyor` and six `High King` occurrences. The creator previously asked not to edit the Chronicle yet; a separate prose review is required before touching it. This is a documented exception, not a completed reconciliation.
- [ ] **Source approval fidelity:** Specific character-by-character spoken approvals earlier in this conversation must still be checked against the complete available conversation, not inferred from the limited recap. Do not invent detailed character motivations or mark these verified.
- [ ] **Additional dialogue locations:** The expected filenames `Human_30_40_NPC_Dialogue_Scripts.md`, `Human_40_50_NPC_Dialogue_Scripts.md`, `Human_50_60_NPC_Dialogue_Scripts.md` and `Human_60_70_NPC_Dialogue_Scripts.md` were not present at those exact paths; discover their actual locations before claiming comprehensive dialogue coverage.
- [ ] **Runtime readiness:** Per-race Q035 completion flags and Q184 contact/location bindings cannot be finalized until dwarf, Aelari and Korrath starting campaigns are approved. Structured manifests are editorial, not executable.

### Continued audit — 12 November 2026 (Korrath cast and actual dialogue packs)

- [x] Located the actual dialogue packs `Human_30_35_NPC_Dialogue_Scripts.md`, `Human_35_45_NPC_Dialogue_Scripts.md`, `Human_45_55_NPC_Dialogue_Scripts.md`, and `Human_55_70_NPC_Dialogue_Scripts.md`. Earlier 'missing dialogue' finding resulted from assuming incorrect level-band filenames; these packs do exist.
- [x] Verified the 30–35, 45–55 and 55–70 packs have no exact `Grey Surveyor` / `High King` labels; updated the Q097 heading in 35–45 to `Father` and updated the engine-independent story invariant likewise.
- [x] Updated individual Urodan and Tarek Oss profiles with their approved wise-traditional versus questioning-safety-conscious tension, while leaving exact personal biographies and dialogue open.
- [ ] Remaining Chronicle editing is explicitly deferred per prior creator instruction; the old narrative labels are known discrepancies, not yet reconciled.
- [ ] Broader full-conversation acceptance still requires checking the entire spoken approval source and all related cast files. Do not equate a file scan with complete semantic verification.

### Cross-race cast pass — 12 November 2026

- [x] Updated Elara, Commander Nessa Ward, Caelith Rowan, Irielle Mossward, Queen Seralyth and First Elder Dromak character notes to respect distinct 1–20 racial histories and independent sovereign authority in the shared campaign.
- [x] Reconfirmed that the missing mid-level dialogue packs exist under actual bands 30–35, 35–45, 45–55 and 55–70; earlier missing-path assumption is closed.
- [ ] Character-by-character details beyond the approved summaries remain subject to full original voice-transcript comparison. These cross-race notes do not establish new personal backstories.
- [ ] Chronicle remains deferred by explicit creator instruction; runtime gate/actor binding remains dependent on future racial campaigns.

### Historical migration and dialogue guardrail pass — 12 November 2026

- [x] Historical `Human_Quest_Editorial_ID_Crosswalk_Draft.json` retains original H2-01 `A Ledger Reaches the Sea` source wording, but now explicitly marks it superseded as a live Q035 mapping; universal summons and race-aware Q184 migration constraints are recorded without rewriting historical evidence.
- [x] `Human_1_80_Key_Scenes_Branching_Dialogue.md` now explicitly guards Q035–Q036 shared authority, Q184 race-specific history, cross-race knowledge, Guardian secrecy and Q097/Q143 reveal timing.
- [ ] Verify other historical crosswalks and legacy dialogue references as **provenance**, not as active canonical quest definitions; avoid indiscriminate replacements.
- [ ] Audit remains OPEN pending full spoken-approval verification, deliberately deferred Chronicle revision, and unapproved racial campaign/runtime bindings.

### Late-campaign quest continuity audit — 12 November 2026

- [x] Compared canonical Q156, Q157, Q159, Q168, Q169, Q181 and Q183 against both 184-quest JSON inventories.
- [x] Q159 and Q168 both use Q156 as their sole quest prerequisite, with approved access notes; neither wrongly requires Q157 or Q181 or imposes an extra level gate.
- [x] Q169 is correctly titled `What the Shroud Concealed`, has a post-Q156 journal investigation and Oth'varen conversation, and its authoring metadata explicitly records the Q181 death-event availability cutoff. The derived dependency reference records expiry for accepted quests that have not completed the conversation; exact runtime handling remains pending.
- [x] Q181's authoring objectives preserve Oth'varen's physical death **after** verifying alignment and **before** final inversion. Nhal's unified will and memory are shattered; divine essence is not annihilated.
- [x] Q183 has both the Sel'ora/Aru'ven Kel'vari response and separate Mara/Corven Covenant report in canonical and structured authoring content.
- [ ] Q169 post-conversation but pre-custody completion remains an explicitly open edge case; do not silently decide it.
- [ ] The runtime still needs to implement Q169 expiration, Q159/Q168 regional permissions and Q183 dual-report evidence. JSON authoring notes are not proof of engine support.
