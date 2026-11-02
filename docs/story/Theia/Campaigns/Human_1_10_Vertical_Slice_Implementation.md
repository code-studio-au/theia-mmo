# Human 1–10 — First Playable Vertical Slice Implementation Brief

**Status: implementation proposal / Needs Review.** This is a build handoff for Q001–Q004 and Q009–Q013, not a claim that code, NPCs, maps or rewards are implemented. It is subordinate to [the canonical campaign](Human_1_80_Canonical_Campaign.md). [Quest pack](Human_Levels_1_10_Playable_Quest_Pack.md) remains the wider level-1–10 design reference.

## Definition of the slice

**Player promise:** start as an unknown Human in Millhaven, earn Elara's trust through ordinary service, discover targeted caravan theft, confront Iven Rusk, and carry credible evidence toward Stoneford. The first session should feel like living in a village, not being briefed on the Unmade or Nhal.

**Playable boundaries:** Millhaven village; militia ground; healing house; merchant yard; connected roads; safe roadside store; abandoned wagon; bandit camp; Stoneford crossing and warehouse. No unapproved map coordinates or fast-travel links.

**Primary critical path:** Q001 → Q002 → Q003 → Q004 → level 5 → Q009 → Q010 → Q011 → Q012 → Q013. Optional Q005–Q008 and Q014–Q017 can be offered in parallel without blocking the critical path. Quest availability must not require optional quest XP if alternative exploration/combat XP exists.

## NPC dialogue — authored first-pass text (subject to narrative review)

### Q001: Elara — Before the Next Caravan
**Offer:** “You want to be useful? Good. First learn who keeps this place running. Darian has the road watch, Anwen the healing house, and Brenna the wagons. Speak to each, then come back. I won't send you somewhere you don't understand.”

**Darian visit:** “The road's usually quiet until it isn't. I can show you where the patrol turns back. I'd rather have another pair of eyes than another boastful sword.”

**Anwen visit:** “A healer needs clean cloth, steady hands and supplies arriving when promised. Lately the last of those has been difficult.”

**Brenna visit:** “My caravan should have been here. People think merchants only worry about coin. They forget what's in the wagons.”

**Turn-in:** “Now you know why a missing caravan matters. Start with Darian's patrol. Bring me what you actually see, not what someone thinks happened.”

**Knowledge guard:** no one mentions royal seals, Cassian, the Unmade, the Anchors or Nhal.

### Q002: Darian — The Road Patrol
**Offer:** “Two places along the road had fresh animal signs yesterday. We should check them before someone gets hurt. Keep your eyes up; I don't need you chasing every noise into the trees.”

**Worker found:** “I stayed off the road when I heard them. Can you get me past the bend?”

**Turn-in:** “You kept your head. That's worth more than charging first. I'll tell Elara the route needs watching.”

**Knowledge guard:** predators are ordinary animals, not corruption.

### Q003: Anwen — What the Healing House Needs
**Offer:** “The medical parcel was left at the roadside store. I need it before evening. Please bring it here, sealed if possible.”

**Patient scene:** “That's healing faster than I expected,” Anwen says quietly. “I don't know why. We should watch carefully before making promises.”

**Turn-in:** “Thank you. One less worry for today. Brenna still hasn't heard from her caravan.”

**Knowledge guard:** no explanation of six fields, Anchors or the Veil.

### Q004: Brenna — The Missing Wheels
**Offer:** “The teamster was meant to report at the western marker. Follow the route and look for the wagon. If you find anyone alive, bring back their account.”

**Teamster:** “They didn't take everything. They knew which crates they wanted. I heard one of them say the buyer wouldn't pay for substitutions.”

**Turn-in:** “That's not an ordinary robbery. I'll speak to Elara. We have a name to follow, but we need more proof before accusing anyone higher up.”

**Level-5 gate:** Q009 becomes available only after Q004 and character level 5; the player is free to explore or complete optional work in the meantime.

### Q009: Elara — An Unusual Buyer
**Offer:** “Iven Rusk has been buying stolen goods in the hills. Find the cargo and his instructions. I want evidence of who ordered it. Don't punish people just because they camp nearby.”

**Evidence discovery:** A written order lists specific cargo types and bears a damaged official-looking seal. It does not authenticate itself as a royal order.

**Turn-in:** “This seal is too important to guess at. Heddan can compare the metalwork, but we may need regional authority before we know whose office used it.”

### Q010: Heddan — A Seal Without a Name
**Offer:** “I can tell you how this was made. I can't tell you who signed it. Look at these fittings and you'll see why.”

**Turn-in:** “The workmanship resembles official stock, but that proves less than people think. Take it to Stoneford with the shipping records.”

### Q011: Brenna — At the River Crossing
**Offer:** “There's a caravan heading for Stoneford. Travel with it. The warehouse keeper can tell us whether this missing cargo ever reached the books.”

**Arrival:** “You've seen the road. Now let's see what the warehouse says.”

### Q012: Stoneford correspondent — Three Books
**Offer:** “One shipment, three records. They should agree. They don't. Find the entry that can't be true in all three.”

**Turn-in:** “Now we have a discrepancy we can show someone else. That is better than a rumor.”

### Q013: Elara's correspondent — The Office After Edric
**Offer:** “Perrin kept the transfer orders. We need the original payment instruction and his statement, under the reeve's authority.”

**Turn-in:** “This goes beyond Stoneford. Elara knows someone in Valecrown who can authenticate it. Her name is Mara.”

## Encounter implementation

### Road predator — Q002
- **Idle:** within home territory, non-hostile until normal aggro radius or player attack.
- **Engage:** target the player or scripted worker defender; chase with ordinary locomotion, respecting melee range and facing.
- **Attack:** telegraphed basic melee; no unavoidable burst or chain-pull.
- **Leash:** if target leaves the designated encounter area, return to spawn, clear threat, restore health; worker returns to safe checkpoint.
- **Death:** grant quest objective credit only for the appropriate patrol encounter, not any random predator anywhere.
- **QA:** loss of target, running behind enemy, character death, disconnect and multiple nearby players cannot permanently break worker rescue.

### Iven Rusk's camp — Q009
- **Outer patrol:** predictable walking loop, social assistance within an authored alert radius; never infinite reinforcements.
- **Camp guards:** ordinary humanoid melee and ranged archetypes, with line-of-sight, facing, target switching and leash behavior.
- **Rusk:** dialogue/surrender when conditions are met, or defeat to a nonlethal story state. His death is not required canon.
- **Evidence:** targeted crates, buyer order and damaged seal have independent objective credits; no loot competition across players.
- **Reset:** enemies respawn and Rusk resets on failure; evidence already recorded in the case journal remains recorded.

### Stoneford records — Q012
- **Interactables:** stock book, courier docket, warehouse tally.
- **Puzzle:** present three readable entries and allow comparison. A wrong answer gives an explanation and retry, not permanent failure.
- **Completion:** only after inspecting all three and identifying the contradictory shipment.
- **Accessibility:** clues conveyed in text as well as visual highlights; no color-only dependency.

## Server-authoritative quest state contract

Use the existing editorial flags `HUMAN_Q001_DONE` through `HUMAN_Q013_DONE` (excluding optional flags unless taken). Stage identifiers are design proposals, not runtime API names.

| Quest | Required stage flags | Completion invariant |
|---|---|---|
| Q001 | DAR_VISITED, ANW_VISITED, BRE_VISITED | All three contact credits recorded; Elara turn-in once |
| Q002 | TRACK_A, TRACK_B, WORKER_SAFE | Both signs and worker safety; Darian turn-in once |
| Q003 | PARCEL_SECURED, PARCEL_DELIVERED, TREATMENT_OBSERVED | No inventory-loss softlock |
| Q004 | MARKER_SEEN, WAGON_INSPECTED, TEAMSTER_FOUND | Teamster evidence saved in case journal |
| Q009 | CAMP_FOUND, CARGO_SECURED, ORDER_COPIED, SEAL_COPIED | Level ≥5, Q004 done, no mandatory Rusk kill |
| Q010 | FITTINGS_COMPARED, REFERRAL_OBTAINED | Seal remains unverified |
| Q011 | CARAVAN_JOINED, STONEFORD_ARRIVED, WAREHOUSE_INSPECTED | Travel checkpoint rejoin |
| Q012 | BOOK_READ, DOCKET_READ, TALLY_READ, CONTRADICTION_IDENTIFIED | Repeatable puzzle |
| Q013 | PAYMENT_COPIED, PERRIN_INTERVIEWED, MARA_INTRODUCTION | Court access not granted yet |

## Acceptance tests for a first playable build

1. New Human player can complete Q001–Q004 without any optional quest.
2. Q009 is unavailable before level 5 even if Q004 is complete, then becomes available immediately on reaching level 5.
3. Optional Q005–Q008 do not appear as dependencies for Q009.
4. NPC interactions and evidence credit survive logout and re-entry; turn-in cannot grant rewards twice.
5. A worker rescue cannot permanently fail if the player dies, leaves the area, or disconnects.
6. Road predators and bandits obey manual facing/range, leash and threat rules; no automatic character rotation.
7. Iven Rusk's encounter can be resolved without a canon-breaking mandatory execution.
8. Q011 requires physical road travel; leaving and returning resumes at a safe caravan checkpoint, not instant teleport.
9. Q012 incorrect answers are retryable, and all three records are readable by keyboard and assistive UI.
10. Q013 unlocks the Valecrown lead but not unrestricted court access; Q018 remains level 10 gated.
11. A character with no magical class abilities can complete every mandatory objective.
12. Two players interacting with the same object cannot steal each other's evidence or break stage credit.

## Unresolved build inputs

Actual map topology, named NPC identities where only roles are given, spawn coordinates, combat stats and aggro radii, exact quest XP, item IDs, equipment rewards, dialogue voiceover/localization, cinematics and regional population density. Do not claim this pack is implemented or production-ready until these are assigned and tested.
