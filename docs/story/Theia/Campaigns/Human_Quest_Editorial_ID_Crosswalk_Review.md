# Human quest editorial-ID reconciliation — first pass

**Status: Needs Review; no runtime IDs allocated.** The [crosswalk JSON](Human_Quest_Editorial_ID_Crosswalk_Draft.json) preserves the original H-series IDs and source objectives separately from the canonical [Human 1–80 campaign](Human_1_80_Canonical_Campaign.md). It is a provenance tool, not a quest import file.

## Findings

- **92 original H-series quest rows** inspected from the superseded Human register.
- **64 exact normalized-title candidates** in the current 184-quest campaign. A matching title alone is not proof of a reused quest.
- **28 original quests have no exact title candidate** and require manual split/merge/retirement review.
- **184 canonical campaign entries** retain their Q001–Q184 references. None has yet been assigned a permanent runtime ID.
- The original register also describes optional local quests; this pass parses only rows bearing an H-series ID. Those additional identifiers need separate review.

## Unmatched legacy quests requiring review

- **H4-01 — An Expedition, Not an Army** (original band 40–41)
- **H4-03 — A Living Homeland** (original band 41–43)
- **H4-04 — The Frontier Beyond the Camp** (original band 41–43)
- **H4-09 — What the Village Can Give** (original band 45–47)
- **H4-10 — A Third Shore** (original band 45–47)
- **H4-11 — The Missing Survey Team** (original band 46–48)
- **H4-12 — Stolen Under Another Flag** (original band 46–48)
- **H4-13 — The Inherited Contract** (original band 47–49)
- **H4-14 — A Way Out for the Bound** (original band 48–49)
- **H4-16 — More Than a Confession** (original band 50)
- **H4-17 — The King's Correction** (original band 50)
- **H4-18 — The Answer Cassian Avoided** (original band 50)
- **H5-01 — A Branch Still Growing** (original band 50–51)
- **H5-02 — The Covenant's Terms** (original band 50–52)
- **H5-03 — A Name Beyond the Veil** (original band 51–53)
- **H5-04 — The Crossing Again** (original band 51–53)
- **H5-05 — What Edric Asked** (original band 52–54)
- **H5-06 — The Coast Remembers** (original band 52–54)
- **H5-07 — The Team Behind the Line** (original band 53–55)
- **H5-08 — The Relay's Schedule** (original band 54–56)
- **H5-12 — The Architect's Objection** (original band 56–58)
- **H5-13 — Everyone's Route Home** (original band 57–59)
- **H5-14 — Cut the Outer Links** (original band 58–59)
- **H5-15 — The Ledger and the Warden** (original band 59–60)
- **H5-16 — The Severed Chain** (original band 60)
- **H5-17 — What the Instruments Prove** (original band 60)
- **H5-18 — A Corrected Record** (original band 60)
- **H5-19 — A Different Legacy** (original band 60)

## Required next steps

1. Compare source objective and new gameplay objective for each title candidate; explicitly approve `reused`, `modified`, `split`, `merged`, `retired`, or `new` with supporting rationale.
2. Resolve optional local quest provenance and all unmatched H-series quests before assigning `theia.quest.human.h1-14`-style IDs.
3. Allocate stable runtime IDs for new quests independently; preserve old H-series editorial identifiers and the current Q references as separate fields.
4. Remove implicit hard level gates from executable eligibility. Preserve planning bands as editorial annotations; only add gates where separately approved.
5. Populate giver/turn-in stable NPC IDs, evidence semantics, possession-versus-history, item consumption, credit scope, rewards, retry and required engine capability status.
6. Do not migrate or rewrite the existing JSON quest definitions as executable Rust data until these mappings are approved.
