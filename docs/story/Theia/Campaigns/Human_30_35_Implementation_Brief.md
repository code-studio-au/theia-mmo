# Human 30–35 — Detailed Quest Implementation Brief v0.2

**Status: production-authoring draft, NOT implemented or playtested.** Story authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Quest classification and flags: [Quest Dependencies](Human_1_80_Quest_Dependencies.json). This document adds per-quest stage plans, player-facing narrative beats and QA criteria. Where a scene is not yet scripted, dialogue below is an **intent**, not finalized spoken dialogue.

## Shared rules

- Classic-style manual target/facing/range and autoattack. No forced PvP, mandatory group, profession, or class-specific ritual.
- Evidence credits are server-authoritative, idempotent and stored in the case journal; failing or abandoning a quest cannot permanently destroy critical evidence.
- Solo story instances checkpoint completed stages; defeat/disconnect resets hostile encounter state but preserves prior evidence and valid choices.
- Region travel is by ship, fixed flight paths and road/foot/mount; only consented Guardian/Kel'vari Passage access is exceptional. No ordinary travel into Shroud before Veilbreaker.
- No coordinates, aggro radii, combat stats, XP values, item IDs or runtime APIs are claimed as implemented.

## Quest-by-quest stage plans

### Q063 — The City That Endured

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Urodan

**Player objective:** Arrive at Drom Kar, hear its custodial limits and secure a water-route work assignment before archive access.

**Stages (investigation):**
1. Receive the case brief from Urodan and register the inquiry.
2. Inspect at least two independent sources relevant to: Arrive at Drom Kar, hear its custodial limits and secure a water-route work assignment before archive access.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Urodan for the next lead.

**Player-facing dialogue intent:** The contact explains why arrive at Drom Kar, hear its custodial limits and secure a water-route work assignment before archive access matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q063_DONE`; prerequisites `HUMAN_Q062_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q064 — Water and Memory

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Urodan / Tarek

**Player objective:** Secure the existing maintenance route and retrieve records from the endangered store; obtain the omitted warning.

**Stages (investigation):**
1. Receive the case brief from Urodan / Tarek and register the inquiry.
2. Inspect at least two independent sources relevant to: Secure the existing maintenance route and retrieve records from the endangered store; obtain the omitted warning.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Urodan / Tarek for the next lead.

**Player-facing dialogue intent:** The contact explains why secure the existing maintenance route and retrieve records from the endangered store; obtain the omitted warning matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q064_DONE`; prerequisites `HUMAN_Q063_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q065 — The Missing Margin

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Dromak

**Player objective:** Authenticate the warning and obtain council permission for restricted comparison; no definitive judgment on the Vorrath exile.

**Stages (investigation):**
1. Receive the case brief from Dromak and register the inquiry.
2. Inspect at least two independent sources relevant to: Authenticate the warning and obtain council permission for restricted comparison; no definitive judgment on the Vorrath exile.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Dromak for the next lead.

**Player-facing dialogue intent:** The contact explains why authenticate the warning and obtain council permission for restricted comparison; no definitive judgment on the Vorrath exile matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q065_DONE`; prerequisites `HUMAN_Q064_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q066 — A Name for the Guardians

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Sera

**Player objective:** Receive a private account of the order and its multiracial origins; accept associate status without compulsory membership.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Receive a private account of the order and its multiracial origins; accept associate status without compulsory membership.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why receive a private account of the order and its multiracial origins; accept associate status without compulsory membership matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q066_DONE`; prerequisites `HUMAN_Q065_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q067 — The Instruction Without Its Diagram

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Grey Surveyor

**Player objective:** Compare the Regional Anchor plan with Halden's incomplete shutdown order; obtain the missing stability material.

**Stages (investigation):**
1. Receive the case brief from Grey Surveyor and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare the Regional Anchor plan with Halden's incomplete shutdown order; obtain the missing stability material.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Grey Surveyor for the next lead.

**Player-facing dialogue intent:** The contact explains why compare the Regional Anchor plan with Halden's incomplete shutdown order; obtain the missing stability material matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q067_DONE`; prerequisites `HUMAN_Q066_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q068 — Reach Them Before the Charge

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Sera

**Player objective:** Reach the freight works and interrupt Halden's premature destruction attempt in a retryable story scene.

**Stages (interaction):**
1. Accept the assignment from Sera.
2. Travel to or inspect the relevant site in Drom Kar, freight works, mainland Chainworks.
3. Complete the objective: Reach the freight works and interrupt Halden's premature destruction attempt in a retryable story scene.
4. Verify the result and report to Sera.

**Player-facing dialogue intent:** The contact explains why reach the freight works and interrupt Halden's premature destruction attempt in a retryable story scene matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q068_DONE`; prerequisites `HUMAN_Q067_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q069 — The Broken Measure

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Halden / Sera

**Player objective:** Complete the solo dungeon, rescue the cell and isolate the Regional Anchor's supports before core shutdown.

**Stages (defense):**
1. Identify who or what needs protection with Halden / Sera.
2. Prepare the safe approach and evacuation route.
3. Complete the protection objective: Complete the solo dungeon, rescue the cell and isolate the Regional Anchor's supports before core shutdown.
4. Confirm survivors and record completion with Halden / Sera.

**Player-facing dialogue intent:** The contact explains why complete the solo dungeon, rescue the cell and isolate the Regional Anchor's supports before core shutdown matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q069_DONE`; prerequisites `HUMAN_Q068_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q070 — Copies for the People at Risk

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Grey Surveyor

**Player objective:** Receive and distribute full safety instructions to the cleared team; lodge the measured evidence independently.

**Stages (investigation):**
1. Receive the case brief from Grey Surveyor and register the inquiry.
2. Inspect at least two independent sources relevant to: Receive and distribute full safety instructions to the cleared team; lodge the measured evidence independently.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Grey Surveyor for the next lead.

**Player-facing dialogue intent:** The contact explains why receive and distribute full safety instructions to the cleared team; lodge the measured evidence independently matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q070_DONE`; prerequisites `HUMAN_Q069_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q071 — Make the Measure Small

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Oth'varen / Brynja

**Player objective:** Set up a fully isolated Minor Resonator trial with instruments, physical cutoff and reserve limits.

**Stages (investigation):**
1. Receive the case brief from Oth'varen / Brynja and register the inquiry.
2. Inspect at least two independent sources relevant to: Set up a fully isolated Minor Resonator trial with instruments, physical cutoff and reserve limits.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Oth'varen / Brynja for the next lead.

**Player-facing dialogue intent:** The contact explains why set up a fully isolated Minor Resonator trial with instruments, physical cutoff and reserve limits matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q071_DONE`; prerequisites `HUMAN_Q070_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q072 — A Brief Reversal

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Oth'varen

**Player objective:** Conduct the controlled inversion trial; stop at its measured duration and record both result and uncertainty.

**Stages (investigation):**
1. Receive the case brief from Oth'varen and register the inquiry.
2. Inspect at least two independent sources relevant to: Conduct the controlled inversion trial; stop at its measured duration and record both result and uncertainty.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Oth'varen for the next lead.

**Player-facing dialogue intent:** The contact explains why conduct the controlled inversion trial; stop at its measured duration and record both result and uncertainty matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q072_DONE`; prerequisites `HUMAN_Q071_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q073 — The Safer Argument

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Halden / Sera

**Player objective:** Review severance and bounded-inversion plans; select a provisional preference that may be changed at the final cutoff.

**Stages (travel):**
1. Receive route authorization from Halden / Sera.
2. Travel physically to Drom Kar, freight works, mainland Chainworks using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Review severance and bounded-inversion plans; select a provisional preference that may be changed at the final cutoff.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why review severance and bounded-inversion plans; select a provisional preference that may be changed at the final cutoff matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q073_DONE`; prerequisites `HUMAN_Q072_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q074 — The Pattern Beneath

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar research chamber · **Quest giver / lead:** Sera

**Player objective:** Compare six measured disturbances and mark common timing.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare six measured disturbances and mark common timing.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why compare six measured disturbances and mark common timing matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q074_DONE`; prerequisites `HUMAN_Q073_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q075 — The Anchor's Defenses

**Track:** Main story · **Level:** 30+ · **Location:** Chainworks exterior · **Quest giver / lead:** Nessa

**Player objective:** Scout guard routes and recover Harbinger orders.

**Stages (travel):**
1. Receive route authorization from Nessa.
2. Travel physically to Chainworks exterior using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Scout guard routes and recover Harbinger orders.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why scout guard routes and recover Harbinger orders matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q075_DONE`; prerequisites `HUMAN_Q074_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q076 — The Harbinger's Design

**Track:** Main story · **Level:** 30+ · **Location:** Chainworks records vault · **Quest giver / lead:** Sera

**Player objective:** Secure signed instructions and destroyed-gate schematic.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Secure signed instructions and destroyed-gate schematic.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why secure signed instructions and destroyed-gate schematic matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q076_DONE`; prerequisites `HUMAN_Q075_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q077 — Against the Harbinger

**Track:** Main story · **Level:** 30+ · **Location:** Chainworks solo dungeon · **Quest giver / lead:** Sera / Nessa

**Player objective:** Free captives, sever conditioning controls and kill Oma'vira.

**Stages (narrative-critical):**
1. Oma'vira is first founder encountered; her title is learned before her name.
2. Rescue captives and disable conditioning controls before the boss; avoid instant reversal of all conditioning.
3. Defeat Oma'vira permanently; the damaged gate record is recoverable.

**Player-facing dialogue intent:** Oma'vira is first founder encountered; her title is learned before her name

**Completion / dependencies:** `HUMAN_Q077_DONE`; prerequisites `HUMAN_Q076_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q078 — The First Major Reversal

**Track:** Main story · **Level:** 30+ · **Location:** Drom Kar, freight works, mainland Chainworks · **Quest giver / lead:** Brynja / Sera

**Player objective:** Defend the cutoff while qualified specialists invert the captured Major Anchor; record bounded success and maintenance duties.

**Stages (narrative-critical):**
1. The Major Anchor inversion is a bounded experiment, not the final Nhal solution.
2. Qualified specialists handle resonance; player defends cutoff and stabilizers.
3. Inversion recorded; later maintenance remains required.

**Player-facing dialogue intent:** The Major Anchor inversion is a bounded experiment, not the final Nhal solution

**Completion / dependencies:** `HUMAN_Q078_DONE`; prerequisites `HUMAN_Q077_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q079 — The Harbinger's Secret

**Track:** Main story · **Level:** 30+ · **Location:** Chainworks archive · **Quest giver / lead:** Sera

**Player objective:** Recover gate destination record and verify gate is unusable.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Recover gate destination record and verify gate is unusable.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why recover gate destination record and verify gate is unusable matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q079_DONE`; prerequisites `HUMAN_Q078_DONE`; minimum level 30. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

## Build acceptance checklist

- Every quest has an NPC interaction, a visible player objective, stage-by-stage credit, a safe failure/retry route, and a verified turn-in.
- Travel cannot be bypassed by invented waypoints; hostile Dominion zones remain hostile and PvE story objectives cannot be blocked by opposing players.
- Dialogue must respect knowledge timing and previously established choices. A player's sample-preservation choice changes follow-up reactions only, not main outcome.
- Combat encounters require actual spawn and leash data, aggro/threat behavior, target facing checks, checkpoint tests and boss scripting before implementation.
- Replace generic stages with precise actor, object, counter and trigger IDs after approved maps and encounter blueprints are available.
- Review the chapter in-game for pacing, XP, difficulty, travel time, accessibility, and solo completion.

**Release blockers:** exact world geometry, quest NPC/creature/item IDs, complete dialogue scripts, cinematic assets, XP and loot tuning, encounter scripting, server quest integration and playtest results. None are asserted complete here.
