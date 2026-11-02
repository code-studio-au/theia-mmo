# Human 20–30 — Detailed Quest Implementation Brief v0.2

**Status: production-authoring draft, NOT implemented or playtested.** Story authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Quest classification and flags: [Quest Dependencies](Human_1_80_Quest_Dependencies.json). This document adds per-quest stage plans, player-facing narrative beats and QA criteria. Where a scene is not yet scripted, dialogue below is an **intent**, not finalized spoken dialogue.

## Shared rules

- Classic-style manual target/facing/range and autoattack. No forced PvP, mandatory group, profession, or class-specific ritual.
- Evidence credits are server-authoritative, idempotent and stored in the case journal; failing or abandoning a quest cannot permanently destroy critical evidence.
- Solo story instances checkpoint completed stages; defeat/disconnect resets hostile encounter state but preserves prior evidence and valid choices.
- Region travel is by ship, fixed flight paths and road/foot/mount; only consented Guardian/Kel'vari Passage access is exceptional. No ordinary travel into Shroud before Veilbreaker.
- No coordinates, aggro radii, combat stats, XP values, item IDs or runtime APIs are claimed as implemented.

## Quest-by-quest stage plans

### Q035 — A Ledger Reaches the Sea

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Mara

**Player objective:** Trace the passage ledger's shipping reference to Tidehold; register the case with alliance customs through the human warrant.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Trace the passage ledger's shipping reference to Tidehold; register the case with alliance customs through the human warrant.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why trace the passage ledger's shipping reference to Tidehold; register the case with alliance customs through the human warrant matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q035_DONE`; prerequisites `HUMAN_Q031_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q036 — Four Peoples, One Harbour

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Corven

**Player objective:** Attend the alliance briefing and distinguish the human commission from alliance authority; obtain a bounded inspection permit.

**Stages (travel):**
1. Receive route authorization from Corven.
2. Travel physically to Tidehold, Northwatch, hostile Dominion military sites, Heartlands using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Attend the alliance briefing and distinguish the human commission from alliance authority; obtain a bounded inspection permit.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why attend the alliance briefing and distinguish the human commission from alliance authority; obtain a bounded inspection permit matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q036_DONE`; prerequisites `HUMAN_Q035_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q037 — Short of the Manifest

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Ysra

**Player objective:** Audit the provision depot and secure exposed goods; separate ordinary loss from the false requisition chain.

**Stages (interaction):**
1. Accept the assignment from Ysra.
2. Travel to or inspect the relevant site in Tidehold, Northwatch, hostile Dominion military sites, Heartlands.
3. Complete the objective: Audit the provision depot and secure exposed goods; separate ordinary loss from the false requisition chain.
4. Verify the result and report to Ysra.

**Player-facing dialogue intent:** The contact explains why audit the provision depot and secure exposed goods; separate ordinary loss from the false requisition chain matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q037_DONE`; prerequisites `HUMAN_Q036_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q038 — Northwatch Bound

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Nessa

**Player objective:** Sail to Northwatch Landing, meet the garrison and Darian's support unit, and learn arrival/extraction procedures.

**Stages (travel):**
1. Receive route authorization from Nessa.
2. Travel physically to Tidehold, Northwatch, hostile Dominion military sites, Heartlands using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Sail to Northwatch Landing, meet the garrison and Darian's support unit, and learn arrival/extraction procedures.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why sail to Northwatch Landing, meet the garrison and Darian's support unit, and learn arrival/extraction procedures matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q038_DONE`; prerequisites `HUMAN_Q037_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q039 — The Stranded Supply Party

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Darian

**Player objective:** Rescue a Covenant supply party by a PvE coastal route; return its military route report. No hostile-player interaction required.

**Stages (investigation):**
1. Receive the case brief from Darian and register the inquiry.
2. Inspect at least two independent sources relevant to: Rescue a Covenant supply party by a PvE coastal route; return its military route report. No hostile-player interaction required.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Darian for the next lead.

**Player-facing dialogue intent:** The contact explains why rescue a Covenant supply party by a PvE coastal route; return its military route report. No hostile-player interaction required matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q039_DONE`; prerequisites `HUMAN_Q038_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q040 — The Defender's Coast

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Nessa

**Player objective:** Scout Rukhan's military posts and civilian boundaries; select a patrol-avoidance or guarded approach without entering civilian homes.

**Stages (travel):**
1. Receive route authorization from Nessa.
2. Travel physically to Tidehold, Northwatch, hostile Dominion military sites, Heartlands using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Scout Rukhan's military posts and civilian boundaries; select a patrol-avoidance or guarded approach without entering civilian homes.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why scout Rukhan's military posts and civilian boundaries; select a patrol-avoidance or guarded approach without entering civilian homes matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q040_DONE`; prerequisites `HUMAN_Q039_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q041 — A Loss Before the Raid

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Mara's expedition clerk

**Player objective:** Recover depot accounting from a military store; compare Nemi's serial records with a private consignment removed before the reported attack.

**Stages (investigation):**
1. Receive the case brief from Mara's expedition clerk and register the inquiry.
2. Inspect at least two independent sources relevant to: Recover depot accounting from a military store; compare Nemi's serial records with a private consignment removed before the reported attack.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara's expedition clerk for the next lead.

**Player-facing dialogue intent:** The contact explains why recover depot accounting from a military store; compare Nemi's serial records with a private consignment removed before the reported attack matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q041_DONE`; prerequisites `HUMAN_Q040_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q042 — The Surveyor at the Signal House

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Sera

**Player objective:** Find Sera, secure an unstable observation position and compare the small device's housing with Brynja's report.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Find Sera, secure an unstable observation position and compare the small device's housing with Brynja's report.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why find Sera, secure an unstable observation position and compare the small device's housing with Brynja's report matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q042_DONE`; prerequisites `HUMAN_Q041_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q043 — Keep the Evidence Safe

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Sera / Grey Surveyor

**Player objective:** Choose sealed sample preservation or safe sample destruction after copying measurements; both retain the critical technical record.

**Stages (investigation):**
1. Receive the case brief from Sera / Grey Surveyor and register the inquiry.
2. Inspect at least two independent sources relevant to: Choose sealed sample preservation or safe sample destruction after copying measurements; both retain the critical technical record.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera / Grey Surveyor for the next lead.

**Player-facing dialogue intent:** The contact explains why choose sealed sample preservation or safe sample destruction after copying measurements; both retain the critical technical record matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q043_DONE`; prerequisites `HUMAN_Q042_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q044 — Instructions With Limits

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Brynja

**Player objective:** Prepare separate mechanical isolators and a documented shutdown procedure; the Surveyor states its limits and emergency cutoff.

**Stages (investigation):**
1. Receive the case brief from Brynja and register the inquiry.
2. Inspect at least two independent sources relevant to: Prepare separate mechanical isolators and a documented shutdown procedure; the Surveyor states its limits and emergency cutoff.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Brynja for the next lead.

**Player-facing dialogue intent:** The contact explains why prepare separate mechanical isolators and a documented shutdown procedure; the Surveyor states its limits and emergency cutoff matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q044_DONE`; prerequisites `HUMAN_Q043_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q045 — The Silent Signal

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Sera

**Player objective:** Complete the solo dungeon; defeat Halvek and disable the Minor Resonator while protecting essential observations and available captives.

**Stages (narrative-critical):**
1. The Minor Resonator is disabled, not converted into the Binding.
2. Solo instance: Halvek guards an unstable signal mechanism; specialists provide nonmagical shutdown controls.
3. Save the technical measurements and make safe extraction possible.

**Player-facing dialogue intent:** The Minor Resonator is disabled, not converted into the Binding

**Completion / dependencies:** `HUMAN_Q045_DONE`; prerequisites `HUMAN_Q044_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q046 — A Landing Still at War

**Track:** Regional / optional · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Nessa / Garrick

**Player objective:** Support an NPC defensive action or equivalent instanced task; report the hidden operation separately from the continuing Dominion attack.

**Stages (investigation):**
1. Receive the case brief from Nessa / Garrick and register the inquiry.
2. Inspect at least two independent sources relevant to: Support an NPC defensive action or equivalent instanced task; report the hidden operation separately from the continuing Dominion attack.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Nessa / Garrick for the next lead.

**Player-facing dialogue intent:** The contact explains why support an NPC defensive action or equivalent instanced task; report the hidden operation separately from the continuing Dominion attack matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q046_DONE`; prerequisites none; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q047 — What the Measurements Say

**Track:** Main story · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Mara

**Player objective:** Authenticate the device report in Tidehold and connect its procurement to a restricted Aelari material reference.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Authenticate the device report in Tidehold and connect its procurement to a restricted Aelari material reference.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why authenticate the device report in Tidehold and connect its procurement to a restricted Aelari material reference matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q047_DONE`; prerequisites `HUMAN_Q045_DONE`; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q048 — A Different Kind of Healing

**Track:** Regional / optional · **Level:** 20+ · **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands · **Quest giver / lead:** Anwen / Oren

**Player objective:** Return home, document a real healing benefit and make a local ward safe; distinguish the beneficial awakening from deliberate interference.

**Stages (interaction):**
1. Accept the assignment from Anwen / Oren.
2. Travel to or inspect the relevant site in Tidehold, Northwatch, hostile Dominion military sites, Heartlands.
3. Complete the objective: Return home, document a real healing benefit and make a local ward safe; distinguish the beneficial awakening from deliberate interference.
4. Verify the result and report to Anwen / Oren.

**Player-facing dialogue intent:** The contact explains why return home, document a real healing benefit and make a local ward safe; distinguish the beneficial awakening from deliberate interference matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q048_DONE`; prerequisites none; minimum level 20. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q049 — The Mountain Contract

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Garrick

**Player objective:** Establish why the altered fitting and the expedition's legitimate repair needs require a Kar Durn visit.

**Stages (travel):**
1. Receive route authorization from Garrick.
2. Travel physically to Kar Durn, Sylvaren, hostile Dominion coast, Tidehold using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Establish why the altered fitting and the expedition's legitimate repair needs require a Kar Durn visit.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why establish why the altered fitting and the expedition's legitimate repair needs require a Kar Durn visit matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q049_DONE`; prerequisites `HUMAN_Q047_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q050 — No Fault in the Drawing

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Brynja

**Player objective:** Inspect her original plans and a broker's substituted specification; secure hold permission to recover a measuring assembly.

**Stages (investigation):**
1. Receive the case brief from Brynja and register the inquiry.
2. Inspect at least two independent sources relevant to: Inspect her original plans and a broker's substituted specification; secure hold permission to recover a measuring assembly.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Brynja for the next lead.

**Player-facing dialogue intent:** The contact explains why inspect her original plans and a broker's substituted specification; secure hold permission to recover a measuring assembly matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q050_DONE`; prerequisites `HUMAN_Q049_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q051 — The Feeder Gallery

**Track:** Regional / optional · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Rurik

**Player objective:** Restore maintenance access by securing its existing route and returning equipment; document a mundane pressure fault and neglected maintenance.

**Stages (travel):**
1. Receive route authorization from Rurik.
2. Travel physically to Kar Durn, Sylvaren, hostile Dominion coast, Tidehold using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Restore maintenance access by securing its existing route and returning equipment; document a mundane pressure fault and neglected maintenance.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why restore maintenance access by securing its existing route and returning equipment; document a mundane pressure fault and neglected maintenance matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q051_DONE`; prerequisites none; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q052 — Grain for Iron

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Brynja / hold authority

**Player objective:** Settle the contested shipment with verified records; secure a limited engineering commission and an inspection report.

**Stages (investigation):**
1. Receive the case brief from Brynja / hold authority and register the inquiry.
2. Inspect at least two independent sources relevant to: Settle the contested shipment with verified records; secure a limited engineering commission and an inspection report.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Brynja / hold authority for the next lead.

**Player-facing dialogue intent:** The contact explains why settle the contested shipment with verified records; secure a limited engineering commission and an inspection report matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q052_DONE`; prerequisites `HUMAN_Q050_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q053 — The King's Correspondence

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn royal court → Sylvaren · **Quest giver / lead:** High King / Caelith

**Player objective:** Obtain High King's sealed introduction and deliver Brynja's findings to Caelith.

**Stages (narrative-critical):**
1. High King endorses Brynja's evidence but does not grant authority in Sylvaren.
2. Travel with the sealed introduction; Caelith must independently approve Aelari research.
3. Optional Aelari woodland tasks remain separate.

**Player-facing dialogue intent:** High King endorses Brynja's evidence but does not grant authority in Sylvaren

**Completion / dependencies:** `HUMAN_Q053_DONE`; prerequisites `HUMAN_Q052_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q054 — Permission to Ask

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Caelith

**Player objective:** Enter Sylvaren and request a limited material inquiry; hear the ward-tenders' concerns before visiting restricted sites.

**Stages (interaction):**
1. Accept the assignment from Caelith.
2. Travel to or inspect the relevant site in Kar Durn, Sylvaren, hostile Dominion coast, Tidehold.
3. Complete the objective: Enter Sylvaren and request a limited material inquiry; hear the ward-tenders' concerns before visiting restricted sites.
4. Verify the result and report to Caelith.

**Player-facing dialogue intent:** The contact explains why enter Sylvaren and request a limited material inquiry; hear the ward-tenders' concerns before visiting restricted sites matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q054_DONE`; prerequisites `HUMAN_Q053_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q055 — The Nursery Route

**Track:** Regional / optional · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Caelith

**Player objective:** Rescue workers and secure a woodland maintenance path threatened by local fauna and a neglected branch-ward.

**Stages (travel):**
1. Receive route authorization from Caelith.
2. Travel physically to Kar Durn, Sylvaren, hostile Dominion coast, Tidehold using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Rescue workers and secure a woodland maintenance path threatened by local fauna and a neglected branch-ward.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why rescue workers and secure a woodland maintenance path threatened by local fauna and a neglected branch-ward matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q055_DONE`; prerequisites none; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q056 — The Boundary Altered

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Caelith

**Player objective:** Compare an abandoned research ward with its approved baseline; establish deliberate alteration without blaming every forest hazard.

**Stages (investigation):**
1. Receive the case brief from Caelith and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare an abandoned research ward with its approved baseline; establish deliberate alteration without blaming every forest hazard.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Caelith for the next lead.

**Player-facing dialogue intent:** The contact explains why compare an abandoned research ward with its approved baseline; establish deliberate alteration without blaming every forest hazard matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q056_DONE`; prerequisites `HUMAN_Q054_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q057 — A Price for Observation

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Mara / Veylan

**Player objective:** Recover Veylan's sale correspondence and obtain an accountable statement under Aelari authority; separate his grievance from his concealment.

**Stages (investigation):**
1. Receive the case brief from Mara / Veylan and register the inquiry.
2. Inspect at least two independent sources relevant to: Recover Veylan's sale correspondence and obtain an accountable statement under Aelari authority; separate his grievance from his concealment.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara / Veylan for the next lead.

**Player-facing dialogue intent:** The contact explains why recover Veylan's sale correspondence and obtain an accountable statement under Aelari authority; separate his grievance from his concealment matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q057_DONE`; prerequisites `HUMAN_Q056_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q058 — What the Queen Will Share

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Seralyth

**Player objective:** Present the local work and findings; receive permission for a limited sample, Caelith's assistance and the coastal lead.

**Stages (travel):**
1. Receive route authorization from Seralyth.
2. Travel physically to Kar Durn, Sylvaren, hostile Dominion coast, Tidehold using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Present the local work and findings; receive permission for a limited sample, Caelith's assistance and the coastal lead.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why present the local work and findings; receive permission for a limited sample, Caelith's assistance and the coastal lead matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q058_DONE`; prerequisites `HUMAN_Q057_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q059 — Back Across the Water

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Nessa

**Player objective:** Depart Tidehold via Northwatch for the second Dominion operation; establish the coastal extraction route.

**Stages (travel):**
1. Receive route authorization from Nessa.
2. Travel physically to Kar Durn, Sylvaren, hostile Dominion coast, Tidehold using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Depart Tidehold via Northwatch for the second Dominion operation; establish the coastal extraction route.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why depart Tidehold via Northwatch for the second Dominion operation; establish the coastal extraction route matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q059_DONE`; prerequisites `HUMAN_Q058_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q060 — The Rejected Component

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Sera

**Player objective:** Retrieve Ilyss's inspection copy from a military observation post by guarded PvE assault or a stealth route; no friendly enemy dialogue.

**Stages (travel):**
1. Receive route authorization from Sera.
2. Travel physically to Kar Durn, Sylvaren, hostile Dominion coast, Tidehold using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Retrieve Ilyss's inspection copy from a military observation post by guarded PvE assault or a stealth route; no friendly enemy dialogue.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why retrieve Ilyss's inspection copy from a military observation post by guarded PvE assault or a stealth route; no friendly enemy dialogue matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q060_DONE`; prerequisites `HUMAN_Q059_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q061 — The Courier Who Burned Both Copies

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Mara's field agent

**Player objective:** Intercept an Unmade courier before the cached archive is destroyed; retain redundancy through an already lodged duplicate.

**Stages (investigation):**
1. Receive the case brief from Mara's field agent and register the inquiry.
2. Inspect at least two independent sources relevant to: Intercept an Unmade courier before the cached archive is destroyed; retain redundancy through an already lodged duplicate.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara's field agent for the next lead.

**Player-facing dialogue intent:** The contact explains why intercept an Unmade courier before the cached archive is destroyed; retain redundancy through an already lodged duplicate matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q061_DONE`; prerequisites `HUMAN_Q060_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q062 — A Common Hand

**Track:** Main story · **Level:** 25+ · **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold · **Quest giver / lead:** Mara

**Player objective:** Compare Aelari and Vaelari records and authorise the Korrath inquiry; establish common suppression, not universal infiltration.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare Aelari and Vaelari records and authorise the Korrath inquiry; establish common suppression, not universal infiltration.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why compare Aelari and Vaelari records and authorise the Korrath inquiry; establish common suppression, not universal infiltration matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q062_DONE`; prerequisites `HUMAN_Q061_DONE`; minimum level 25. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

## Build acceptance checklist

- Every quest has an NPC interaction, a visible player objective, stage-by-stage credit, a safe failure/retry route, and a verified turn-in.
- Travel cannot be bypassed by invented waypoints; hostile Dominion zones remain hostile and PvE story objectives cannot be blocked by opposing players.
- Dialogue must respect knowledge timing and previously established choices. A player's sample-preservation choice changes follow-up reactions only, not main outcome.
- Combat encounters require actual spawn and leash data, aggro/threat behavior, target facing checks, checkpoint tests and boss scripting before implementation.
- Replace generic stages with precise actor, object, counter and trigger IDs after approved maps and encounter blueprints are available.
- Review the chapter in-game for pacing, XP, difficulty, travel time, accessibility, and solo completion.

**Release blockers:** exact world geometry, quest NPC/creature/item IDs, complete dialogue scripts, cinematic assets, XP and loot tuning, encounter scripting, server quest integration and playtest results. None are asserted complete here.
