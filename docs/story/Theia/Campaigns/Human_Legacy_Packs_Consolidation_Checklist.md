# Human 1–80 — Legacy Pack Consolidation Checklist

**Status: In progress — no deletion authorization.** This document inventories old quest packs and implementation briefs to prevent loss of unique design details. [Current Human Campaign](CURRENT_HUMAN_CAMPAIGN.md) is the entry point; [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the sole structured working quest inventory. [Quest Engine](../../../QUEST_ENGINE.md) is Codex's runtime agreement.

## Sources awaiting reconciliation

| Band | Playable quest pack | Detailed implementation brief |
|---|---|---|
| 1–10 | `Human_Levels_1_10_Playable_Quest_Pack.md` | no matching chapter brief in this inventory |
| 10–20 | `Human_10_20_Playable_Quest_Pack.md` | `Human_10_20_Implementation_Brief.md` |
| 20–30 | `Human_20_30_Playable_Quest_Pack.md` | `Human_20_30_Implementation_Brief.md` |
| 30–35 | `Human_30_35_Playable_Quest_Pack.md` | `Human_30_35_Implementation_Brief.md` |
| 35–45 | `Human_35_45_Playable_Quest_Pack.md` | `Human_35_45_Implementation_Brief.md` |
| 45–55 | `Human_45_55_Playable_Quest_Pack.md` | `Human_45_55_Implementation_Brief.md` |
| 55–70 | `Human_55_70_Playable_Quest_Pack.md` | `Human_55_70_Implementation_Brief.md` |
| 70–80 | `Human_70_80_Playable_Quest_Pack.md` | `Human_70_80_Implementation_Brief.md` |
| 80 finale | `Human_80_Finale_Playable_Quest_Pack.md` | `Human_80_Finale_Implementation_Brief.md` |

These **17 files** contain substantial authored prose. They are **not authoritative** over the canonical story or the single manifest. They cannot safely be deleted merely because the JSON stage data has been consolidated.

## Required migration checks per quest

1. Confirm Q reference and historical H/HS provenance; preserve source distinctions for reused, split, merged and retired entries.
2. Compare objective ordering, interaction wording, named NPC roles, exact locations, failure/retry and evidence against the current manifest.
3. Transfer unique, still-canonical gameplay details to the manifest or referenced encounter/dialogue specifications. Do not copy superseded narrative decisions.
4. Resolve planned capabilities explicitly (inventory possession versus historical acquisition, consumption, exploration, branching, party sharing, phasing). Do not misrepresent them as currently executable.
5. Validate dialogue/reveal gates, solo story dungeons and atomic world transitions; preserve Q156/Q181 causal sequencing.
6. Check all incoming links and replace references before removing a source file.
7. Record reviewed source and migration disposition in a final provenance ledger. Only then delete the redundant source from the active tree; Git history retains it.

## Current progress and blocker

- **184 quests / 684 stages** are consolidated in one manifest; first-pass Rust authoring reviews cover the whole campaign.
- The chapter JSON duplicates have been removed after count/stage/target verification.
- These 17 prose sources have been located and inventoried, **not yet line-by-line reconciled**. Their unique details are not guaranteed to be represented in the manifest.
- The older H/HS register remains until editorial mapping is approved.
- Codex's quest catalogue registry, migration and live adoption are separate engineering tasks; do not edit or delete Codex's Rust files during story cleanup.

**Levels 1–10 comparison completed:** the old pack has 17 quest sections, including gameplay and QA notes not yet fully present in the manifest. It also proposes hard level-1/5 eligibility and strict sequential stages that conflict with the current authoring agreement. Preserve the pack until those unique notes are migrated; do not delete it merely because objective targets match. Specific findings: Q001 should allow the three NPC conversations in any order; Q003 adds parcel-seal inspection; Q004 adds wheel-track and cargo-frame clues; Q006 adds a restitution dialogue choice; Q007 adds cart movement testing; Q009 adds Iven Rusk surrender/defeat alternative; Q011 adds caravan checkpoint recovery; Q012 adds retryable non-guesswork deduction; Q015 adds repair-material collection; Q017 adds caravan rejoin logic. These are review candidates, not approved runtime data.

**Next:** migrate those unique mechanics and QA requirements into the sole manifest, then delete the old 1–10 pack only after a complete content comparison. Continue chapter by chapter. Delete only after verified migration, not as a bulk cleanup.

## Levels 1–10 migration update

The 17 quest sections and their distinctive mechanics, NPC/location context, failure behavior and QA notes have been transferred into [Q001–Q017 Gameplay Detail Migration](Human_Q001_Q017_Gameplay_Detail_Migration.md). These details remain editorial proposals rather than executable quest definitions. The original playable pack is retained until the proposed extra objectives and inbound references are reviewed; the manifest is still the sole structured inventory.

## Levels 10–20 reconciliation

Both legacy sources (`Human_10_20_Playable_Quest_Pack.md` and `Human_10_20_Implementation_Brief.md`) have been compared at quest-section and shared-rules level. Their current narrative purposes, location bands, recovery expectations and QA checks are captured in [Q018–Q034 Legacy Pack Migration](Human_Q018_Q034_Legacy_Pack_Migration.md). The older brief's generic stage templates and minimum-level assumptions are not authoritative. **Original files remain until inbound links and unique details are verified; no deletion is claimed.**

## Levels 20–30 reconciliation

The 28 entries in both `Human_20_30_Playable_Quest_Pack.md` and `Human_20_30_Implementation_Brief.md` were compared by ID and purpose. Their chapter-specific gameplay context, region/permission rules, common retry and QA expectations and superseded level assumptions are captured in [Q035–Q062 Legacy Pack Migration](Human_Q035_Q062_Legacy_Pack_Migration.md). The original files are retained until reference and unique-detail verification; this comparison does **not** authorize deleting them yet.

## Levels 30–35 reconciliation

Both `Human_30_35_Playable_Quest_Pack.md` and `Human_30_35_Implementation_Brief.md` cover the same 17 Q063–Q079 quests. Chapter context, encounter/retry rules, player-purpose distinctions and superseded level assumptions are recorded in [Q063–Q079 Legacy Pack Migration](Human_Q063_Q079_Legacy_Pack_Migration.md). Special care preserves Grey Surveyor anonymity, the two separate inversion trials, Oma'vira's permanent death and the one-time destroyed-gate evidence. Original sources remain pending inbound-reference and unique-detail checks.

## Levels 35–45 reconciliation

Both `Human_35_45_Playable_Quest_Pack.md` and `Human_35_45_Implementation_Brief.md` contain the same 24 Q080–Q103 entries. Location/purpose context, shared recovery/QA requirements and reveal/consent constraints are captured in [Q080–Q103 Legacy Pack Migration](Human_Q080_Q103_Legacy_Pack_Migration.md). The older minimum-level and generic-stage assumptions are not adopted. The originals remain pending inbound-reference and unique-detail checks.
