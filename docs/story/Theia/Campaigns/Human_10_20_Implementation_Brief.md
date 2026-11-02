# Human 10–20 — Detailed Quest Implementation Brief v0.2

**Status: production-authoring draft, NOT implemented or playtested.** Story authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Quest classification and flags: [Quest Dependencies](Human_1_80_Quest_Dependencies.json). This document adds per-quest stage plans, player-facing narrative beats and QA criteria. Where a scene is not yet scripted, dialogue below is an **intent**, not finalized spoken dialogue.

## Shared rules

- Classic-style manual target/facing/range and autoattack. No forced PvP, mandatory group, profession, or class-specific ritual.
- Evidence credits are server-authoritative, idempotent and stored in the case journal; failing or abandoning a quest cannot permanently destroy critical evidence.
- Solo story instances checkpoint completed stages; defeat/disconnect resets hostile encounter state but preserves prior evidence and valid choices.
- Region travel is by ship, fixed flight paths and road/foot/mount; only consented Guardian/Kel'vari Passage access is exceptional. No ordinary travel into Shroud before Veilbreaker.
- No coordinates, aggro radii, combat stats, XP values, item IDs or runtime APIs are claimed as implemented.

## Quest-by-quest stage plans

### Q018 — An Investigator's Standard

**Track:** Main story · **Level:** 10+ · **Location:** Valecrown and approaches · **Quest giver / lead:** Mara

**Player objective:** Present the sealed case in Valecrown; verify courier and warehouse testimony independently before receiving court access.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Present the sealed case in Valecrown; verify courier and warehouse testimony independently before receiving court access.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why present the sealed case in Valecrown; verify courier and warehouse testimony independently before receiving court access matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q018_DONE`; prerequisites `HUMAN_Q013_DONE`; minimum level 10. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q019 — Dates in the Register

**Track:** Main story · **Level:** 10+ · **Location:** Valecrown and approaches · **Quest giver / lead:** Lysa

**Player objective:** Check authorised office records; make an authenticated copy of the post-delegation reorganisation dates.

**Stages (investigation):**
1. Receive the case brief from Lysa and register the inquiry.
2. Inspect at least two independent sources relevant to: Check authorised office records; make an authenticated copy of the post-delegation reorganisation dates.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Lysa for the next lead.

**Player-facing dialogue intent:** The contact explains why check authorised office records; make an authenticated copy of the post-delegation reorganisation dates matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q019_DONE`; prerequisites `HUMAN_Q018_DONE`; minimum level 10. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q020 — The King's Confidence

**Track:** Main story · **Level:** 10+ · **Location:** Valecrown and approaches · **Quest giver / lead:** Aldren / Cassian

**Player objective:** Attend the audience, hear Aldren's account and accept a continuing warrant; Cassian supplies a plausible military lead.

**Stages (investigation):**
1. Receive the case brief from Aldren / Cassian and register the inquiry.
2. Inspect at least two independent sources relevant to: Attend the audience, hear Aldren's account and accept a continuing warrant; Cassian supplies a plausible military lead.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Aldren / Cassian for the next lead.

**Player-facing dialogue intent:** The contact explains why attend the audience, hear Aldren's account and accept a continuing warrant; Cassian supplies a plausible military lead matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q020_DONE`; prerequisites `HUMAN_Q019_DONE`; minimum level 10. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q021 — A Convenient Suspect

**Track:** Main story · **Level:** 10+ · **Location:** Valecrown and approaches · **Quest giver / lead:** Garrick

**Player objective:** Check the alleged military source against Garrick's preserved logs; establish his opposition to peace is not proof of involvement.

**Stages (investigation):**
1. Receive the case brief from Garrick and register the inquiry.
2. Inspect at least two independent sources relevant to: Check the alleged military source against Garrick's preserved logs; establish his opposition to peace is not proof of involvement.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Garrick for the next lead.

**Player-facing dialogue intent:** The contact explains why check the alleged military source against Garrick's preserved logs; establish his opposition to peace is not proof of involvement matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q021_DONE`; prerequisites `HUMAN_Q020_DONE`; minimum level 10. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q022 — Before the Request

**Track:** Main story · **Level:** 10+ · **Location:** Valecrown and approaches · **Quest giver / lead:** Mara

**Player objective:** Compare the authorisation time, office format and independently interviewed witness; record the contradictions without an accusation.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare the authorisation time, office format and independently interviewed witness; record the contradictions without an accusation.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why compare the authorisation time, office format and independently interviewed witness; record the contradictions without an accusation matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q022_DONE`; prerequisites `HUMAN_Q021_DONE`; minimum level 10. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q023 — The Clerk's Queue

**Track:** Regional / optional · **Level:** 10+ · **Location:** Valecrown and approaches · **Quest giver / lead:** Lysa

**Player objective:** Help restore a damaged public index to learn how Valecrown's records are accessed.

**Stages (investigation):**
1. Receive the case brief from Lysa and register the inquiry.
2. Inspect at least two independent sources relevant to: Help restore a damaged public index to learn how Valecrown's records are accessed.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Lysa for the next lead.

**Player-facing dialogue intent:** The contact explains why help restore a damaged public index to learn how Valecrown's records are accessed matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q023_DONE`; prerequisites none; minimum level 10. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q024 — Bread and Barracks

**Track:** Regional / optional · **Level:** 10+ · **Location:** Valecrown and approaches · **Quest giver / lead:** City quartermaster

**Player objective:** Investigate a legitimate ration shortage and secure a non-corrupt supply solution.

**Stages (travel):**
1. Receive route authorization from City quartermaster.
2. Travel physically to Valecrown and approaches using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Investigate a legitimate ration shortage and secure a non-corrupt supply solution.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why investigate a legitimate ration shortage and secure a non-corrupt supply solution matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q024_DONE`; prerequisites none; minimum level 10. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q025 — A Witness's Own Words

**Track:** Regional / optional · **Level:** 10+ · **Location:** Valecrown and approaches · **Quest giver / lead:** Mara

**Player objective:** Interview a witness without leading questions; record and compare the independent account.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Interview a witness without leading questions; record and compare the independent account.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why interview a witness without leading questions; record and compare the independent account matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q025_DONE`; prerequisites none; minimum level 10. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q026 — The Walls Outside Court

**Track:** Regional / optional · **Level:** 10+ · **Location:** Valecrown and approaches · **Quest giver / lead:** Garrick

**Player objective:** Inspect a neglected approach road and repel opportunistic raiders, separating wartime danger from political suspicion.

**Stages (travel):**
1. Receive route authorization from Garrick.
2. Travel physically to Valecrown and approaches using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Inspect a neglected approach road and repel opportunistic raiders, separating wartime danger from political suspicion.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why inspect a neglected approach road and repel opportunistic raiders, separating wartime danger from political suspicion matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q026_DONE`; prerequisites none; minimum level 10. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q027 — The Price of an Office

**Track:** Main story · **Level:** 15+ · **Location:** Valecrown and Millhaven · **Quest giver / lead:** Mara

**Player objective:** Secure the surviving intermediary and independently stored payments linking Cassian to the security compromise and intended killing.

**Stages (interaction):**
1. Accept the assignment from Mara.
2. Travel to or inspect the relevant site in Valecrown and Millhaven.
3. Complete the objective: Secure the surviving intermediary and independently stored payments linking Cassian to the security compromise and intended killing.
4. Verify the result and report to Mara.

**Player-facing dialogue intent:** The contact explains why secure the surviving intermediary and independently stored payments linking Cassian to the security compromise and intended killing matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q027_DONE`; prerequisites `HUMAN_Q022_DONE`; minimum level 15. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q028 — A Warrant, Not a Challenge

**Track:** Main story · **Level:** 15+ · **Location:** Valecrown and Millhaven · **Quest giver / lead:** Aldren

**Player objective:** Present the corroborated case in private; witness lawful detention ordered and Cassian's flight. No attack on a neutral court NPC.

**Stages (interaction):**
1. Accept the assignment from Aldren.
2. Travel to or inspect the relevant site in Valecrown and Millhaven.
3. Complete the objective: Present the corroborated case in private; witness lawful detention ordered and Cassian's flight. No attack on a neutral court NPC.
4. Verify the result and report to Aldren.

**Player-facing dialogue intent:** The contact explains why present the corroborated case in private; witness lawful detention ordered and Cassian's flight. No attack on a neutral court NPC matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q028_DONE`; prerequisites `HUMAN_Q027_DONE`; minimum level 15. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q029 — Before the Hidden Passage

**Track:** Main story · **Level:** 15+ · **Location:** Valecrown and Millhaven · **Quest giver / lead:** Mara

**Player objective:** Secure civilian exits and establish an independent evidence-copy checkpoint before Cassian's capture.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Secure civilian exits and establish an independent evidence-copy checkpoint before Cassian's capture.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why secure civilian exits and establish an independent evidence-copy checkpoint before Cassian's capture matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q029_DONE`; prerequisites `HUMAN_Q028_DONE`; minimum level 15. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q030 — The Hidden Passage

**Track:** Main story · **Level:** 15+ · **Location:** Valecrown and Millhaven · **Quest giver / lead:** Mara

**Player objective:** Finish the solo story dungeon; stop Sorn, secure a dispatch copy and capture Cassian alive. Group play is optional.

**Stages (narrative-critical):**
1. Cassian is arrested alive; Mara refuses extrajudicial execution.
2. Solo instance: Sorn controls a three-room pursuit with readable patrols; final Cassian surrender.
3. The dispatch case is copied before the last confrontation; restart at the last secured room.

**Player-facing dialogue intent:** Cassian is arrested alive; Mara refuses extrajudicial execution

**Completion / dependencies:** `HUMAN_Q030_DONE`; prerequisites `HUMAN_Q029_DONE`; minimum level 15. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q031 — The Commission

**Track:** Main story · **Level:** 15+ · **Location:** Valecrown and Millhaven · **Quest giver / lead:** Aldren / Elara

**Player objective:** Confirm removal from office and lawful custody; receive the follow-up warrant and report home. Close the betrayal arc without absolving Dominion.

**Stages (investigation):**
1. Receive the case brief from Aldren / Elara and register the inquiry.
2. Inspect at least two independent sources relevant to: Confirm removal from office and lawful custody; receive the follow-up warrant and report home. Close the betrayal arc without absolving Dominion.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Aldren / Elara for the next lead.

**Player-facing dialogue intent:** The contact explains why confirm removal from office and lawful custody; receive the follow-up warrant and report home. Close the betrayal arc without absolving Dominion matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q031_DONE`; prerequisites `HUMAN_Q030_DONE`; minimum level 15. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q032 — After the Arrest

**Track:** Regional / optional · **Level:** 15+ · **Location:** Valecrown and Millhaven · **Quest giver / lead:** Elara

**Player objective:** Return recovered goods and inform affected households without disclosing protected witnesses.

**Stages (defense):**
1. Identify who or what needs protection with Elara.
2. Prepare the safe approach and evacuation route.
3. Complete the protection objective: Return recovered goods and inform affected households without disclosing protected witnesses.
4. Confirm survivors and record completion with Elara.

**Player-facing dialogue intent:** The contact explains why return recovered goods and inform affected households without disclosing protected witnesses matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q032_DONE`; prerequisites none; minimum level 15. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q033 — The Watch Restored

**Track:** Regional / optional · **Level:** 15+ · **Location:** Valecrown and Millhaven · **Quest giver / lead:** Darian

**Player objective:** Lead a local patrol and protect a newly reopened road without a mandatory escort timer.

**Stages (travel):**
1. Receive route authorization from Darian.
2. Travel physically to Valecrown and Millhaven using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Lead a local patrol and protect a newly reopened road without a mandatory escort timer.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why lead a local patrol and protect a newly reopened road without a mandatory escort timer matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q033_DONE`; prerequisites none; minimum level 15. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q034 — A Village Still Working

**Track:** Regional / optional · **Level:** 15+ · **Location:** Valecrown and Millhaven · **Quest giver / lead:** Brenna

**Player objective:** Choose a local relief priority and complete the delivery; the royal case ends but village life continues.

**Stages (interaction):**
1. Accept the assignment from Brenna.
2. Travel to or inspect the relevant site in Valecrown and Millhaven.
3. Complete the objective: Choose a local relief priority and complete the delivery; the royal case ends but village life continues.
4. Verify the result and report to Brenna.

**Player-facing dialogue intent:** The contact explains why choose a local relief priority and complete the delivery; the royal case ends but village life continues matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q034_DONE`; prerequisites none; minimum level 15. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

## Build acceptance checklist

- Every quest has an NPC interaction, a visible player objective, stage-by-stage credit, a safe failure/retry route, and a verified turn-in.
- Travel cannot be bypassed by invented waypoints; hostile Dominion zones remain hostile and PvE story objectives cannot be blocked by opposing players.
- Dialogue must respect knowledge timing and previously established choices. A player's sample-preservation choice changes follow-up reactions only, not main outcome.
- Combat encounters require actual spawn and leash data, aggro/threat behavior, target facing checks, checkpoint tests and boss scripting before implementation.
- Replace generic stages with precise actor, object, counter and trigger IDs after approved maps and encounter blueprints are available.
- Review the chapter in-game for pacing, XP, difficulty, travel time, accessibility, and solo completion.

**Release blockers:** exact world geometry, quest NPC/creature/item IDs, complete dialogue scripts, cinematic assets, XP and loot tuning, encounter scripting, server quest integration and playtest results. None are asserted complete here.
