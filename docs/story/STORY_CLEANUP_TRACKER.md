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

Assign KEEP, MERGE, RETIRE or BLOCKED for each remaining target. Compare the entire candidate, preserve genuinely unique information with approval status and provenance in its proper existing home, repair incoming references, validate, commit on a dedicated branch and obtain independent review before integration. Preserve the protected Bible, Chronicle, canonical campaign and manifest. Do not promote proposals, infer runtime mappings or alter permanent outcomes. Current work is Markdown-only; code, CI, schemas and executable quest data are outside scope.

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

## Validation checkpoints, not final certification

- At main 37dd8520f7990cbb42bc7c7c52f02de0efd31a76, the complete 362-Markdown snapshot passed 3,182 relative file links and 56 section links repository-wide; story subset 1,463/51. Later PRs received their own bounded link/preservation checks. Run fresh combined validation after the remaining work.
- That checkpoint found 184 unique canonical quest identifiers and 184 matching review entries with no title mismatches or gaps. It did not establish full stage equivalence, runtime readiness, all reveal gates or approved H/HS mappings.
- Dialogue corpus has 180 quest sections: Q014–Q017 are absent. Q183's Covenant excerpt does not supply its required Kel'vari report dialogue.
- Source evidence records 444 distinct question identifiers / 471 occurrences, 418 historical answered prompts and 26 unanswered prompts. Reused question numbers require stage/page context. Original PDF and complete external audit artifacts are not in the repository; exhaustive independent PDF verification is not certified.
- Legacy branch story/launch-main-story-1-60 was previously 0 commits ahead and 534 behind main. This was an ancestry checkpoint, not content-equivalence proof or permission to delete it.

## Remaining work and exact blockers

- [ ] Finish explicit KEEP/MERGE/RETIRE/BLOCKED dispositions for remaining active Markdown duplicates, including surviving root historical records, campaign support/audit references, individual Lore/History/Characters/Atlas notes, Connected Stories and encounter references. Preserve unique approved narrative forms rather than retire merely because facts overlap. Location/lore editorial appendix retirement is in progress, not completed by this tracker.
- [ ] Complete current canon/status reconciliation across remaining supporting records. Validate Bible/Chronicle/campaign consistency, exact revelation timing, permanent outcomes, evidence custody, contacts/locations, optional versus main-story objectives and solo access. Prior navigation sweeps are not semantic clearance.
- [ ] **Missing dialogue:** Q014–Q017 sections and Q183 Kel'vari report remain absent. Authoring them must follow approved outcomes without adding new lore or silently altering the campaign.
- [ ] **Completion semantics:** Q176 and Q181 completion-contact behavior remains OPEN. Do not let a contact requirement imply a dead character remains available.
- [ ] **Planning/runtime boundary:** Non-executable gate metadata and eventual runtime gate reconciliation remain OPEN. Current Markdown-only authority does not permit JSON, runtime IDs, engine/schema/code/CI work. Determine what can be verified from Markdown; request separate authorization if non-Markdown changes are needed.
- [ ] **Historical identifiers:** Preserve the root [H/HS register](THEIA_HUMAN_QUEST_REGISTER.md), existing [exact-title comparison](Theia/Campaigns/Human_Legacy_64_Exact_Title_Objective_Comparison.md) and archive evidence until all 92 H and 24 HS concepts are reconciled or explicitly archived. Sixty-four title matches are not approved equivalences; no runtime mapping is approved.
- [ ] **Mapping evidence still needs disposition:** Preserve original objectives, contacts, locations, choices, dialogue, access and consequences. Historical flags include the eight objective-detail/four contact discrepancies, H4-07/Q097 prerequisites/consequences, teamster survival/damaged-document authentication, protected route/interview/court access/custody, H1-15/Q027 intended-killing evidence and Garrick's innocence-of-involvement distinction, H1-18 office removal/non-absolution, H2-04 versus Q049 prerequisite difference, and H2-05 goods/customs/hold permission. Later comparisons may resolve individual flags; record their disposition without treating an old review request as a fresh canon contradiction or automatically approving mappings.
- [ ] **Optional title proposal:** Distinct Q128/Q154 UI titles were suggested historically, not approved. Keep their distinct prerequisites/preparation phases; no quest rename, deletion or stage change by inference.
- [ ] Finish unique sidequest/zone/character/faction/Dominion/encounter/future-idea preservation and deduplication in the one archive. Archive concepts remain unapproved; rejected old Keld/Voss, Malrec/projection and alternate Nhal finales cannot return as active canon.
- [ ] Revalidate the complete latest Markdown tree: relative links, section fragments, residual Obsidian references, backlinks to each retired file, source-history links and reference scope. Validate quest identifiers/dependencies/contact/location/reveal/outcome/evidence/solo requirements at the authorized scope.
- [ ] Review final PR diffs for Markdown-only scope and unintended changes; ensure every cleanup PR is reviewed, merged and verified on main. Do not infer validation or merge success from a proposal or stale snapshot.
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
