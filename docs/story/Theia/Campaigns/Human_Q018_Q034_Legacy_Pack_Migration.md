# Human Q018–Q034 — Legacy Pack and Brief Migration

**Status: legacy content reconciled at editorial level; detailed runtime authoring still Needs Review.** The [canonical Human campaign](Human_1_80_Canonical_Campaign.md) governs narrative, [current quest manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the sole structured inventory, and [Codex Quest Engine](../../../QUEST_ENGINE.md) governs implementation. This document preserves still-useful design context from two older sources without making them competing quest definitions:

- `Human_10_20_Playable_Quest_Pack.md` (v0.1): all Q018–Q034 entries, player-purpose summaries, location bands and general encounter/recovery rules.
- `Human_10_20_Implementation_Brief.md` (v0.2): all Q018–Q034 entries, investigation/interaction stage suggestions, generic QA and build acceptance checklist.

## Shared mechanics retained

Classic manual target/facing, persistent weapon autoattack, safe solo story instances, evidence retained independently of inventory, and no mandatory PvP/group/profession/class gate. Hostile Dominion territory remains hostile. Travel is by road, ship or fixed flight path; exceptional Guardian/Kel'vari Passage requires consent and does not substitute for ordinary world travel. Respawn/reconnect resumes from a safe checkpoint; hostile encounters reset, while verified evidence and meaningful choices persist. Map coordinates, spawn IDs, interaction volumes, combat stats, XP, loot, actor IDs and localization are not yet assigned.

**Superseded claims:** the older pack's `minimum level 10/15` and chapter unlock requirements are narrative pacing proposals, **not automatically executable eligibility**. `HUMAN_Q###_DONE` is a draft alias, not a runtime ID. The implementation brief's repeated “accept → inspect → complete → report” pattern is generic scaffolding, not proof of four real gameplay stages.

## Quest-specific retained context

| Quest | Historical playable purpose and location | Consolidation decision |
|---|---|---|
| Q018 An Investigator's Standard | Present sealed case, verify courier and warehouse testimony in Valecrown | Preserve independent evidence authentication; Mara controls inquiry authorization |
| Q019 Dates in the Register | Check office reassignment chronology and copy records in Valecrown | Two independent sources; Lysa controls authorized access |
| Q020 The King's Confidence | Hear Aldren's account and Cassian's plausible military lead | Cassian initially credible; lawful royal audience |
| Q021 A Convenient Suspect | Question Garrick and test military allegations | His opposition to peace does not establish guilt |
| Q022 Before the Request | Compare administrative dates, formats and witness account | Preserve pre-request chronology, avoid duplicated optional witness |
| Q023 The Clerk's Queue | Public index repair, ordinary civic work | Optional; does not unlock restricted archives |
| Q024 Bread and Barracks | Ration shortfall and legitimate shipment delay | Optional mundane supply problem |
| Q025 A Witness's Own Words | Independent witness interview and evidence handling | Optional; distinguish witness from mandatory Q022 |
| Q026 The Walls Outside Court | Patrol neglected approaches outside Valecrown | Optional ordinary raider defense; Classic leash/facing |
| Q027 The Price of an Office | Trace intermediary and independently authenticated payment | Safe witness extraction and persistent chain of custody |
| Q028 A Warrant, Not a Challenge | Present case privately, Aldren orders detention, Cassian flees | No attack on neutral court NPCs; Cassian alive |
| Q029 Before the Hidden Passage | Secure civilian exits and backup evidence | Distinct pre-dungeon checkpoint |
| Q030 The Hidden Passage | Defeat Sorn and capture Cassian alive | Solo story dungeon, safe restart, evidence before arrest |
| Q031 The Commission | Confirm lawful custody and limited royal authority | Return to Millhaven; authority is bounded |
| Q032 After the Arrest | Return goods and repair household losses | Optional post-arrest content; requires story-appropriate availability |
| Q033 The Watch Restored | Reopen safe local patrol routes | Optional post-arrest content; requires story-appropriate availability |
| Q034 A Village Still Working | Local relief priorities and closure | Optional, persistent acknowledgement without main-story fork |

## Specific findings from both old sources

- Both contain **17 matching Q references**, with no new quest IDs.
- The playable pack supplies named location bands and narrative intent; the newer implementation brief mostly repeats these with a generic stage template.
- The old implementation brief says the player's sample-preservation choice changes later reactions. That choice belongs to Q043, not Q018–Q034, so it should remain a **cross-chapter continuity requirement**, not a new Q018–Q034 objective.
- Old stage counts are not authoritative. The manifest's objective targets and the completed [Q018–Q034 Rust Authoring Review](Human_Q018_Q034_Rust_Authoring_Review.md) are the current basis for future refinement.
- No new dialogue text, coordinates, encounter stats or production-ready quest data is implied by the old files.

## Runtime and QA handoff

Use stable giver/turn-in actor IDs and distinct semantic action IDs; a dialogue opening is not a verified report. Ensure ordered-stage progress does not cascade on one event. Persist historical evidence separately from physical possession, and define any quest item consumption explicitly. Test optional independence, scene replay, nonlethal arrest, boss reset, travel availability, reconnect, duplicate rewards, class-neutral completion and accessibility. XP/reward tuning and actual engine validation remain outstanding.

**Disposition:** the two old documents' still-current shared rules, per-quest purposes and QA expectations are represented here, the single manifest, canonical narrative, encounter dialogue and authoring review. This note is not executable quest data. Remove the old sources only after checking inbound references and confirming no unique source detail was omitted.
