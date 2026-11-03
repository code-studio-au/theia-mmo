# Human 1–80 — Legacy Pack Consolidation Checklist

**Status: CLOSED for legacy pack retirement; historical reconciliation log retained pending broader story cleanup.** This document inventories old quest packs and implementation briefs to prevent loss of unique design details. [Current Human Campaign](CURRENT_HUMAN_CAMPAIGN.md) is the entry point; [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the sole structured working quest inventory. [Quest Engine](../../../QUEST_ENGINE.md) is Codex's runtime agreement.

## Historical source inventory — all 17 retired

| Band | Playable quest pack | Detailed implementation brief |
|---|---|---|
| 1–10 | **Retired** (content preserved in `Human_Q001_Q017_Gameplay_Detail_Migration.md`) | no matching chapter brief in this inventory |
| 10–20 | **Retired** (Q018–Q034 migration retained) | **Retired** |
| 20–30 | **Retired** (Q035–Q062 migration retained) | **Retired** |
| 30–35 | **Retired** (Q063–Q079 migration retained) | **Retired** |
| 35–45 | **Retired** (Q080–Q103 migration retained) | **Retired** |
| 45–55 | **Retired** (Q104–Q128 migration retained) | **Retired** |
| 55–70 | **Retired** (Q129–Q157 migration retained) | **Retired** |
| 70–80 | **Retired** (Q158–Q184 migration retained) | **Retired** |
| 80 finale | **Retired** (Q158–Q184 migration retained) | **Retired** |

These **17 historical files** contained substantial authored prose and have all been retired from the active tree after editorial migration. They were never authoritative over the canonical story or sole structured manifest. Git history preserves their original bytes.

## Required migration checks per quest

1. Confirm Q reference and historical H/HS provenance; preserve source distinctions for reused, split, merged and retired entries.
2. Compare objective ordering, interaction wording, named NPC roles, exact locations, failure/retry and evidence against the current manifest.
3. Transfer unique, still-canonical gameplay details to the manifest or referenced encounter/dialogue specifications. Do not copy superseded narrative decisions.
4. Resolve planned capabilities explicitly (inventory possession versus historical acquisition, consumption, exploration, branching, party sharing, phasing). Do not misrepresent them as currently executable.
5. Validate dialogue/reveal gates, solo story dungeons and atomic world transitions; preserve Q156/Q181 causal sequencing.
6. Check all incoming links and replace references before removing a source file.
7. Record reviewed source and migration disposition in a final provenance ledger. Only then delete the redundant source from the active tree; Git history retains it.

## Historical progress snapshot (superseded by final retirement below)

- **184 quests / 684 stages** are consolidated in one manifest; first-pass Rust authoring reviews cover the whole campaign.
- The chapter JSON duplicates have been removed after count/stage/target verification.
- At this earlier stage the 17 prose sources had only been inventoried. Subsequent chapter migration notes and verified deletions supersede this snapshot; migrated mechanics are still editorial, not automatically executable.
- The older H/HS register remains until editorial mapping is approved.
- Codex's quest catalogue registry, migration and live adoption are separate engineering tasks; do not edit or delete Codex's Rust files during story cleanup.

**Levels 1–10 comparison completed:** the old pack has 17 quest sections, including gameplay and QA notes not yet fully present in the manifest. It also proposes hard level-1/5 eligibility and strict sequential stages that conflict with the current authoring agreement. Preserve the pack until those unique notes are migrated; do not delete it merely because objective targets match. Specific findings: Q001 should allow the three NPC conversations in any order; Q003 adds parcel-seal inspection; Q004 adds wheel-track and cargo-frame clues; Q006 adds a restitution dialogue choice; Q007 adds cart movement testing; Q009 adds Iven Rusk surrender/defeat alternative; Q011 adds caravan checkpoint recovery; Q012 adds retryable non-guesswork deduction; Q015 adds repair-material collection; Q017 adds caravan rejoin logic. These are review candidates, not approved runtime data.

**Historical next action, completed for retirement:** capture unique mechanics and QA requirements in the Q001–Q017 editorial migration note, then retire the old pack after reference checks. Migration into executable Rust quest content remains separate.

## Levels 1–10 migration update

The 17 quest sections and their distinctive mechanics, NPC/location context, failure behavior and QA notes have been transferred into [Q001–Q017 Gameplay Detail Migration](Human_Q001_Q017_Gameplay_Detail_Migration.md). These details remain editorial proposals rather than executable quest definitions. The original playable pack was subsequently retired after editorial migration and inbound-reference checks; the manifest remains the sole structured inventory. Proposed extra objectives are not approved executable data.

## Levels 10–20 reconciliation

Both legacy sources (`Human_10_20_Playable_Quest_Pack.md` and `Human_10_20_Implementation_Brief.md`) have been compared at quest-section and shared-rules level. Their current narrative purposes, location bands, recovery expectations and QA checks are captured in [Q018–Q034 Legacy Pack Migration](Human_Q018_Q034_Legacy_Pack_Migration.md). The older brief's generic stage templates and minimum-level assumptions are not authoritative. **Original files remain until inbound links and unique details are verified; no deletion is claimed.**

## Levels 20–30 reconciliation

The 28 entries in both `Human_20_30_Playable_Quest_Pack.md` and `Human_20_30_Implementation_Brief.md` were compared by ID and purpose. Their chapter-specific gameplay context, region/permission rules, common retry and QA expectations and superseded level assumptions are captured in [Q035–Q062 Legacy Pack Migration](Human_Q035_Q062_Legacy_Pack_Migration.md). The original files are retained until reference and unique-detail verification; this comparison does **not** authorize deleting them yet.

## Levels 30–35 reconciliation

Both `Human_30_35_Playable_Quest_Pack.md` and `Human_30_35_Implementation_Brief.md` cover the same 17 Q063–Q079 quests. Chapter context, encounter/retry rules, player-purpose distinctions and superseded level assumptions are recorded in [Q063–Q079 Legacy Pack Migration](Human_Q063_Q079_Legacy_Pack_Migration.md). Special care preserves Grey Surveyor anonymity, the two separate inversion trials, Oma'vira's permanent death and the one-time destroyed-gate evidence. Original sources remain pending inbound-reference and unique-detail checks.

## Levels 35–45 reconciliation

Both `Human_35_45_Playable_Quest_Pack.md` and `Human_35_45_Implementation_Brief.md` contain the same 24 Q080–Q103 entries. Location/purpose context, shared recovery/QA requirements and reveal/consent constraints are captured in [Q080–Q103 Legacy Pack Migration](Human_Q080_Q103_Legacy_Pack_Migration.md). The older minimum-level and generic-stage assumptions are not adopted. The originals remain pending inbound-reference and unique-detail checks.

## Levels 45–55 reconciliation

Both `Human_45_55_Playable_Quest_Pack.md` and `Human_45_55_Implementation_Brief.md` cover the same 25 Q104–Q128 entries. Their still-current player purposes, location context, safe encounter/retry and QA rules are captured in [Q104–Q128 Legacy Pack Migration](Human_Q104_Q128_Legacy_Pack_Migration.md). Critical canon preserved: the reluctant anti-peace Dominion delegate, Su'vareth's permanent death, ongoing war and Anchors' separate function from the Binding. Older hard level gates and generic stages remain superseded. Original files await inbound-reference and unique-detail checks.

## Levels 55–70 reconciliation

Both `Human_55_70_Playable_Quest_Pack.md` and `Human_55_70_Implementation_Brief.md` cover the same 29 Q129–Q157 quests. Their still-current location, player-purpose, recovery and QA information is captured in [Q129–Q157 Legacy Pack Migration](Human_Q129_Q157_Legacy_Pack_Migration.md). Special continuity checks preserve the first Nhal name at Q143, Keeper/Guardian testimony, Khor'vun's death and the immediate Q156 Veil/Binding/Shroud transition. The old files remain pending incoming-reference and unique-detail verification.

## Levels 70–80 and finale reconciliation

All four remaining legacy files (`Human_70_80_Playable_Quest_Pack.md`, `Human_70_80_Implementation_Brief.md`, `Human_80_Finale_Playable_Quest_Pack.md`, `Human_80_Finale_Implementation_Brief.md`) have been compared for their 17 + 10 quest sections. Still-current gameplay, QA and ending chronology are captured in [Q158–Q184 Legacy Pack Migration](Human_Q158_Q184_Legacy_Pack_Migration.md). The manifest's explicit Q181 death stage takes precedence over old finale text. **Historical snapshot:** all 17 files had received editorial reconciliation at this stage. Subsequent verified retirement, recorded below, removed all 17 from the active tree.

## Verified deletion — levels 1–10

The Q001–Q017 legacy playable pack was removed from the active tree after its 17 sections and distinct gameplay/QA details were preserved in [Q001–Q017 Gameplay Detail Migration](Human_Q001_Q017_Gameplay_Detail_Migration.md). Direct links in the production specification, NPC dialogue and vertical-slice handoff were updated to the current migration document. The source remains recoverable through Git history. **The first pack was retired at this stage;** the later final-retirement section records completion for all 17. This does not imply approval of all migrated mechanics as executable quest objectives.

## Verified deletion — levels 10–30

Four legacy sources were removed after both chapter pairs were compared, their 17 and 28 quest references confirmed, and current narrative/QA/recovery details preserved in [Q018–Q034 migration](Human_Q018_Q034_Legacy_Pack_Migration.md) and [Q035–Q062 migration](Human_Q035_Q062_Legacy_Pack_Migration.md). The current production specification links were already redirected to those documents. Git history retains originals. **5 of 17 legacy pack/brief files are now deleted; 12 remain.** This is an editorial cleanup, not a claim that the migrated quest data is executable or production ready.

## Verified deletion — levels 30–45

Four reconciled source files were deleted after confirming Q063–Q079 (17 quests) and Q080–Q103 (24 quests) coverage, redirecting their earlier active links and preserving relevant narrative, encounter and QA constraints in [Q063–Q079 migration](Human_Q063_Q079_Legacy_Pack_Migration.md) and [Q080–Q103 migration](Human_Q080_Q103_Legacy_Pack_Migration.md). Git history retains originals. **Nine of 17** legacy pack/brief files had been deleted at this stage; the remaining eight were removed later as recorded in the final-retirement section. Editorial migration does not certify executable Rust content.

## Final legacy-pack retirement

All **17 of 17** playable quest packs and implementation briefs have now been removed from the active repository. Their 184 editorial quest references, story outcomes, still-relevant gameplay intentions and general QA/recovery requirements are represented by the canonical campaign, sole structured quest manifest, Rust authoring reviews and eight reconciliation notes. Old versions remain in Git history. This completes **legacy pack retirement only**: the large production specification, H/HS provenance register, reconciliation notes and other redundant narrative planning documents still need a separate source-of-truth audit before deletion. No Rust quest engine files were changed.
