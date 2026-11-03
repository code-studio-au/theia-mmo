# Human 1–80 — Quest Necessity and Duplication Audit (Pass 1)

**Status: Needs Review; no canonical quests removed by this audit.** This document evaluates the current 184-entry canonical Human campaign without changing its story or quest count. Source: [Canonical Campaign](Human_1_80_Canonical_Campaign.md), [Quest Dependencies](Human_1_80_Quest_Dependencies.json), [Current State](CURRENT_HUMAN_CAMPAIGN.md). The existing classification of 156 main and 28 regional quests is **provisional**, not an approved release design.

## Current inventory

- **184 distinct editorial Q references** (Q001–Q184), still present.
- **156 provisional main-story entries**, **28 provisional optional regional entries**.
- **No quest has been deleted, merged, or renamed** in this audit.
- A title similarity check identified two pairs with identical names or near-duplicates. Identical names do not prove redundant gameplay.

## Priority editorial reviews

| Entries | Why review | Proposed disposition |
|---|---|---|
| Q128 / Q154 — *Before the Veil Breaks* | Exactly the same title, different stages of preparation | **Keep both for now**; rename one after reviewing the different operational purposes |
| Q029 / Q030 — *Before the Hidden Passage* / *The Hidden Passage* | Preparation immediately followed by dungeon | **Keep separate** if civilian exits/evidence backup need an independent checkpoint; otherwise embed preparation as dungeon entry objectives |
| Q137 / Q145 — *Records of the First Voice* / *The First Voice* | Similar title, archive research versus boss confrontation | **Keep both**; clearly distinguish archive and encounter in UI |
| Q083 / Q084 / Q085 / Q086 / Q088 | Gate survey, consent, crossing, Kel'vari reveal and access agreement | **Review chain**: optional Q083 cannot be a hidden prerequisite to mandatory Q084–Q086; ensure permission and safe return are on the mandatory path |
| Q091 / Q092 / Q093 / Q094 | Multiple archive-permission and records tasks | **Review for consolidation**; optional Q093 cannot gate Q094; preserve independent archive consent |
| Q109 / Q110 / Q111 / Q112 / Q113 / Q114 | Delegation investigation with repeated evidence acquisition | **Keep evidence provenance** but check whether some steps should be objectives within fewer quests, rather than repeated return-to-NPC loops |
| Q129 / Q130 / Q132 / Q133 / Q134 / Q135 | Warden network reconnaissance, raid and aftermath | **Review pacing**; a separate after-action quest is justified only if it changes the player's understanding or unlocks new choices |
| Q147 / Q148 / Q149 / Q150 / Q151 | Keeper and Guardian revelation | **Preserve chronology**; consider combining purely administrative steps, not the independent evidence needed to prove deception |
| Q158 / Q161 / Q162 / Q164 | Manifestation and inversion research | **Keep scientific distinctions**: observation, fracture evidence, interpretation, distributed testing are not interchangeable |
| Q170 / Q172 / Q173 / Q174 | Finale preparations and safety reviews | **Review repetitive checklists**; Q170 must not imply Oth'varen foresees his death |
| Q175–Q184 | Level-80 finale and epilogue | **Preserve all canonical outcomes**; only combine entries if no story event, checkpoint or player agency is lost |

## Review questions for each quest

1. Does it deliver unique story evidence, a character beat, a gameplay tutorial, a region experience, a meaningful choice, or a required encounter checkpoint?
2. Would merging it with a neighboring quest improve pacing without hiding a critical revelation or reducing solo accessibility?
3. Does the quest ask the player to repeat an identical activity with no meaningful new context?
4. Can a main quest proceed if an optional regional quest is skipped? Does an optional quest require story-only region access that should be explicit?
5. Are original H-series/HS-series IDs preserved separately from Q references, with no automatic runtime-ID reassignment?
6. Are the listed level bands **planning only**, unless a particular level gate has been explicitly approved?
7. Does the content require a planned engine capability (multi-stage, inventory, exploration, branching, party sharing, phasing), or can it run as a native Rust objective without Luau?

## Suggested decision statuses

- **Keep:** unique narrative or gameplay purpose established.
- **Combine:** two editorial quests become one executable quest with multiple objectives; keep both original Q references in the migration history.
- **Regionalize:** move an independent activity off the critical path without making its rewards or evidence mandatory.
- **Retire:** redundant objective and no unique content; record the decision before removing it.
- **Rewrite:** preserve the quest slot but revise objective pacing or dialogue.
- **Needs Review:** insufficient evidence for any of the above.

## Important limits

This is a first-pass editorial audit, **not** a full objective-by-objective deduplication or playtest. It does not establish that all 184 entries are necessary or that a lower count would be better. No stable runtime IDs have been assigned. Do not delete the source register, optional stories, or quest packs based solely on title similarity.

## Next work

Review the flagged quest clusters against their detailed stage data and dialogue, approve concrete keep/combine/regionalize decisions, and build one Rust-compatible quest-definition set. The 17 legacy playable packs/implementation briefs and duplicate 184-entry production specification have already been retired after separate migration reviews; do not recreate them. Continue checking surviving review notes and the historical H/HS register before any further removals.
