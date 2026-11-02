# Human 70–80 — Detailed Quest Implementation Brief v0.2

**Status: production-authoring draft, NOT implemented or playtested.** Story authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Quest classification and flags: [Quest Dependencies](Human_1_80_Quest_Dependencies.json). This document adds per-quest stage plans, player-facing narrative beats and QA criteria. Where a scene is not yet scripted, dialogue below is an **intent**, not finalized spoken dialogue.

## Shared rules

- Classic-style manual target/facing/range and autoattack. No forced PvP, mandatory group, profession, or class-specific ritual.
- Evidence credits are server-authoritative, idempotent and stored in the case journal; failing or abandoning a quest cannot permanently destroy critical evidence.
- Solo story instances checkpoint completed stages; defeat/disconnect resets hostile encounter state but preserves prior evidence and valid choices.
- Region travel is by ship, fixed flight paths and road/foot/mount; only consented Guardian/Kel'vari Passage access is exceptional. No ordinary travel into Shroud before Veilbreaker.
- No coordinates, aggro radii, combat stats, XP values, item IDs or runtime APIs are claimed as implemented.

## Quest-by-quest stage plans

### Q158 — Essence Without Form

**Track:** Main story · **Level:** 70+ · **Location:** Receding Shroud · **Quest giver / lead:** Sera

**Player objective:** Observe manifestations and collect readings.

**Stages (interaction):**
1. Accept the assignment from Sera.
2. Travel to or inspect the relevant site in Receding Shroud.
3. Complete the objective: Observe manifestations and collect readings.
4. Verify the result and report to Sera.

**Player-facing dialogue intent:** The contact explains why observe manifestations and collect readings matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q158_DONE`; prerequisites `HUMAN_Q157_DONE`; minimum level 70. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q159 — The Lands Revealed

**Track:** Regional / optional · **Level:** 70+ · **Location:** Exposed Kel'vari approaches · **Quest giver / lead:** Aru'ven

**Player objective:** Escort displaced residents to shelter.

**Stages (defense):**
1. Identify who or what needs protection with Aru'ven.
2. Prepare the safe approach and evacuation route.
3. Complete the protection objective: Escort displaced residents to shelter.
4. Confirm survivors and record completion with Aru'ven.

**Player-facing dialogue intent:** The contact explains why escort displaced residents to shelter matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q159_DONE`; prerequisites none; minimum level 70. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q160 — The Architect's Hand

**Track:** Main story · **Level:** 70+ · **Location:** Unmade installation · **Quest giver / lead:** Mara

**Player objective:** Steal Thes'var stabilization orders.

**Stages (interaction):**
1. Accept the assignment from Mara.
2. Travel to or inspect the relevant site in Unmade installation.
3. Complete the objective: Steal Thes'var stabilization orders.
4. Verify the result and report to Mara.

**Player-facing dialogue intent:** The contact explains why steal Thes'var stabilization orders matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q160_DONE`; prerequisites `HUMAN_Q158_DONE`; minimum level 70. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q161 — Fractured Manifestation

**Track:** Main story · **Level:** 70+ · **Location:** Inverted Major Anchor · **Quest giver / lead:** Sera / Oth'varen

**Player objective:** Measure manifestation fracturing under inversion.

**Stages (narrative-critical):**
1. Inversion fractures coherent manifestation, never annihilates divine essence.
2. Sera initially calls it destruction; Oth'varen identifies broken continuity.
3. Evidence motivates distributed inversion tests, not instant certainty.

**Player-facing dialogue intent:** Inversion fractures coherent manifestation, never annihilates divine essence

**Completion / dependencies:** `HUMAN_Q161_DONE`; prerequisites `HUMAN_Q160_DONE`; minimum level 70. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q162 — What Cannot Be Destroyed

**Track:** Main story · **Level:** 70+ · **Location:** Guardian laboratory · **Quest giver / lead:** Oth'varen

**Player objective:** Compare fragments with ancient essence evidence.

**Stages (narrative-critical):**
1. Divine essence cannot be annihilated; identity and memory can be shattered.
2. Compare independent readings and ancient records before approving a hypothesis.
3. Do not reveal what happens to Nhal's future fragments.

**Player-facing dialogue intent:** Divine essence cannot be annihilated; identity and memory can be shattered

**Completion / dependencies:** `HUMAN_Q162_DONE`; prerequisites `HUMAN_Q161_DONE`; minimum level 70. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q163 — The Architect Speaks

**Track:** Main story · **Level:** 70+ · **Location:** Ancient communicator · **Quest giver / lead:** Thes'var transmission

**Player objective:** Receive Architect's invitation and challenge claims.

**Stages (interaction):**
1. Accept the assignment from Thes'var transmission.
2. Travel to or inspect the relevant site in Ancient communicator.
3. Complete the objective: Receive Architect's invitation and challenge claims.
4. Verify the result and report to Thes'var transmission.

**Player-facing dialogue intent:** The contact explains why receive Architect's invitation and challenge claims matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q163_DONE`; prerequisites `HUMAN_Q162_DONE`; minimum level 70. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q164 — Convergence Principle

**Track:** Main story · **Level:** 70+ · **Location:** Distributed Anchor sites · **Quest giver / lead:** Brynja / Sera

**Player objective:** Test synchronized bounded readings.

**Stages (narrative-critical):**
1. Small experiment does not prove full-Nhal shattering by itself.
2. Run multiple synchronized bounded tests with independent cutoffs.
3. A risky but evidence-based convergence theory.

**Player-facing dialogue intent:** Small experiment does not prove full-Nhal shattering by itself

**Completion / dependencies:** `HUMAN_Q164_DONE`; prerequisites `HUMAN_Q163_DONE`; minimum level 70. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q165 — The Guardian's Decision

**Track:** Main story · **Level:** 70+ · **Location:** Guardian council · **Quest giver / lead:** Sera

**Player objective:** Present risks and obtain conditional approval.

**Stages (interaction):**
1. Accept the assignment from Sera.
2. Travel to or inspect the relevant site in Guardian council.
3. Complete the objective: Present risks and obtain conditional approval.
4. Verify the result and report to Sera.

**Player-facing dialogue intent:** The contact explains why present risks and obtain conditional approval matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q165_DONE`; prerequisites `HUMAN_Q164_DONE`; minimum level 70. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q166 — The Architect's Invitation

**Track:** Main story · **Level:** 70+ · **Location:** Architect facility approaches · **Quest giver / lead:** Nessa

**Player objective:** Map defenses and extraction route.

**Stages (travel):**
1. Receive route authorization from Nessa.
2. Travel physically to Architect facility approaches using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Map defenses and extraction route.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why map defenses and extraction route matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q166_DONE`; prerequisites `HUMAN_Q165_DONE`; minimum level 70. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q167 — Stone and Strain

**Track:** Main story · **Level:** 75+ · **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp · **Quest giver / lead:** Brynja

**Player objective:** Inspect one inverted installation's structural supports and repair a documented failure.

**Stages (interaction):**
1. Accept the assignment from Brynja.
2. Travel to or inspect the relevant site in Distributed Anchors, Covenant and Kel'vari regions, Guardian camp.
3. Complete the objective: Inspect one inverted installation's structural supports and repair a documented failure.
4. Verify the result and report to Brynja.

**Player-facing dialogue intent:** The contact explains why inspect one inverted installation's structural supports and repair a documented failure matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q167_DONE`; prerequisites `HUMAN_Q166_DONE`; minimum level 75. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q168 — Life at the Edges

**Track:** Regional / optional · **Level:** 75+ · **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp · **Quest giver / lead:** Caelith

**Player objective:** Measure ecological effects near another installation and mitigate harm.

**Stages (travel):**
1. Receive route authorization from Caelith.
2. Travel physically to Distributed Anchors, Covenant and Kel'vari regions, Guardian camp using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Measure ecological effects near another installation and mitigate harm.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why measure ecological effects near another installation and mitigate harm matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q168_DONE`; prerequisites none; minimum level 75. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q169 — The Route Between

**Track:** Regional / optional · **Level:** 75+ · **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp · **Quest giver / lead:** Tarek

**Player objective:** Travel by ship, flight path and overland to check network connections; no unlimited teleportation.

**Stages (travel):**
1. Receive route authorization from Tarek.
2. Travel physically to Distributed Anchors, Covenant and Kel'vari regions, Guardian camp using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Travel by ship, flight path and overland to check network connections; no unlimited teleportation.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why travel by ship, flight path and overland to check network connections; no unlimited teleportation matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q169_DONE`; prerequisites none; minimum level 75. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q170 — Instructions That Survive Us

**Track:** Main story · **Level:** 75+ · **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp · **Quest giver / lead:** Oth'varen / Halden

**Player objective:** Train independent teams and deposit redundant safety records for operation without the ancient scholar.

**Stages (investigation):**
1. Receive the case brief from Oth'varen / Halden and register the inquiry.
2. Inspect at least two independent sources relevant to: Train independent teams and deposit redundant safety records for operation without the ancient scholar.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Oth'varen / Halden for the next lead.

**Player-facing dialogue intent:** The contact explains why train independent teams and deposit redundant safety records for operation without the ancient scholar matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q170_DONE`; prerequisites `HUMAN_Q167_DONE`; minimum level 75. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q171 — The Consent of Neighbors

**Track:** Main story · **Level:** 75+ · **Location:** Covenant and Kel'vari councils · **Quest giver / lead:** Corven / Aru'ven

**Player objective:** Obtain separate operational permissions.

**Stages (interaction):**
1. Accept the assignment from Corven / Aru'ven.
2. Travel to or inspect the relevant site in Covenant and Kel'vari councils.
3. Complete the objective: Obtain separate operational permissions.
4. Verify the result and report to Corven / Aru'ven.

**Player-facing dialogue intent:** The contact explains why obtain separate operational permissions matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q171_DONE`; prerequisites `HUMAN_Q170_DONE`; minimum level 75. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q172 — A Failure We Can See

**Track:** Main story · **Level:** 75+ · **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp · **Quest giver / lead:** Tarek

**Player objective:** Detect the weak junction and establish an emergency correction route before final activation.

**Stages (travel):**
1. Receive route authorization from Tarek.
2. Travel physically to Distributed Anchors, Covenant and Kel'vari regions, Guardian camp using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Detect the weak junction and establish an emergency correction route before final activation.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why detect the weak junction and establish an emergency correction route before final activation matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q172_DONE`; prerequisites `HUMAN_Q171_DONE`; minimum level 75. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q173 — The Last Measure

**Track:** Main story · **Level:** 75+ · **Location:** Guardian test chamber · **Quest giver / lead:** Sera

**Player objective:** Run bounded distributed alignment test.

**Stages (travel):**
1. Receive route authorization from Sera.
2. Travel physically to Guardian test chamber using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Run bounded distributed alignment test.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why run bounded distributed alignment test matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q173_DONE`; prerequisites `HUMAN_Q172_DONE`; minimum level 75. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q174 — Before the Final Crossing

**Track:** Main story · **Level:** 75+ · **Location:** Staging camp · **Quest giver / lead:** Nessa

**Player objective:** Verify teams and complete readiness.

**Stages (narrative-critical):**
1. Finale begins only after level 80, not to award the XP needed to reach it.
2. Check transport, consent, equipment and independent teams.
3. No required class-specific ritual.

**Player-facing dialogue intent:** Finale begins only after level 80, not to award the XP needed to reach it

**Completion / dependencies:** `HUMAN_Q174_DONE`; prerequisites `HUMAN_Q173_DONE`; minimum level 75. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

## Build acceptance checklist

- Every quest has an NPC interaction, a visible player objective, stage-by-stage credit, a safe failure/retry route, and a verified turn-in.
- Travel cannot be bypassed by invented waypoints; hostile Dominion zones remain hostile and PvE story objectives cannot be blocked by opposing players.
- Dialogue must respect knowledge timing and previously established choices. A player's sample-preservation choice changes follow-up reactions only, not main outcome.
- Combat encounters require actual spawn and leash data, aggro/threat behavior, target facing checks, checkpoint tests and boss scripting before implementation.
- Replace generic stages with precise actor, object, counter and trigger IDs after approved maps and encounter blueprints are available.
- Review the chapter in-game for pacing, XP, difficulty, travel time, accessibility, and solo completion.

**Release blockers:** exact world geometry, quest NPC/creature/item IDs, complete dialogue scripts, cinematic assets, XP and loot tuning, encounter scripting, server quest integration and playtest results. None are asserted complete here.
