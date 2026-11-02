# Human 45–55 — Detailed Quest Implementation Brief v0.2

**Status: production-authoring draft, NOT implemented or playtested.** Story authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Quest classification and flags: [Quest Dependencies](Human_1_80_Quest_Dependencies.json). This document adds per-quest stage plans, player-facing narrative beats and QA criteria. Where a scene is not yet scripted, dialogue below is an **intent**, not finalized spoken dialogue.

## Shared rules

- Classic-style manual target/facing/range and autoattack. No forced PvP, mandatory group, profession, or class-specific ritual.
- Evidence credits are server-authoritative, idempotent and stored in the case journal; failing or abandoning a quest cannot permanently destroy critical evidence.
- Solo story instances checkpoint completed stages; defeat/disconnect resets hostile encounter state but preserves prior evidence and valid choices.
- Region travel is by ship, fixed flight paths and road/foot/mount; only consented Guardian/Kel'vari Passage access is exceptional. No ordinary travel into Shroud before Veilbreaker.
- No coordinates, aggro radii, combat stats, XP values, item IDs or runtime APIs are claimed as implemented.

## Quest-by-quest stage plans

### Q104 — The Delegation That Failed

**Track:** Main story · **Level:** 45+ · **Location:** Oth'irun archive · **Quest giver / lead:** Mara

**Player objective:** Collect surviving Delegation dispatches.

**Stages (interaction):**
1. Accept the assignment from Mara.
2. Travel to or inspect the relevant site in Oth'irun archive.
3. Complete the objective: Collect surviving Delegation dispatches.
4. Verify the result and report to Mara.

**Player-facing dialogue intent:** The contact explains why collect surviving Delegation dispatches matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q104_DONE`; prerequisites `HUMAN_Q103_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q105 — Two Accounts of One Tragedy

**Track:** Main story · **Level:** 45+ · **Location:** Oth'irun archive · **Quest giver / lead:** Sel'ora

**Player objective:** Compare separately sourced faction records.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare separately sourced faction records.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why compare separately sourced faction records matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q105_DONE`; prerequisites `HUMAN_Q104_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q106 — The King's Lost Adviser

**Track:** Main story · **Level:** 45+ · **Location:** Valecrown archive · **Quest giver / lead:** Mara

**Player objective:** Recover Edric's correspondence and witness records.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Recover Edric's correspondence and witness records.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why recover Edric's correspondence and witness records matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q106_DONE`; prerequisites `HUMAN_Q105_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q107 — The Brother Who Came for Peace

**Track:** Main story · **Level:** 45+ · **Location:** Covenant intelligence office · **Quest giver / lead:** Mara

**Player objective:** Reconstruct Great Clan chief's reluctant official attendance.

**Stages (interaction):**
1. Accept the assignment from Mara.
2. Travel to or inspect the relevant site in Covenant intelligence office.
3. Complete the objective: Reconstruct Great Clan chief's reluctant official attendance.
4. Verify the result and report to Mara.

**Player-facing dialogue intent:** The contact explains why reconstruct Great Clan chief's reluctant official attendance matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q107_DONE`; prerequisites `HUMAN_Q106_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q108 — The Records They Buried

**Track:** Main story · **Level:** 45+ · **Location:** Hostile Dominion military archive · **Quest giver / lead:** Mara's field agent

**Player objective:** Infiltrate, copy suppressed security orders and escape.

**Stages (narrative-critical):**
1. Dominion military archive infiltration is hostile, not diplomacy.
2. Stealth or combat through military guards; copy records, withdraw without speaking to friendly Dominion officers.
3. Proof of the High Ruler's brother's reluctant attendance and manipulated orders.

**Player-facing dialogue intent:** Dominion military archive infiltration is hostile, not diplomacy

**Completion / dependencies:** `HUMAN_Q108_DONE`; prerequisites `HUMAN_Q107_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q109 — Road to the Meeting

**Track:** Main story · **Level:** 45+ · **Location:** Historic delegation site · **Quest giver / lead:** Mara

**Player objective:** Survey arrival routes and guard positions.

**Stages (travel):**
1. Receive route authorization from Mara.
2. Travel physically to Historic delegation site using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Survey arrival routes and guard positions.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why survey arrival routes and guard positions matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q109_DONE`; prerequisites `HUMAN_Q108_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q110 — Evidence Planted Twice

**Track:** Main story · **Level:** 45+ · **Location:** Ruined conference compound · **Quest giver / lead:** Mara

**Player objective:** Recover planted clues and compare provenance.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Recover planted clues and compare provenance.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why recover planted clues and compare provenance matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q110_DONE`; prerequisites `HUMAN_Q109_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q111 — The Witness Who Disappeared

**Track:** Main story · **Level:** 45+ · **Location:** Neutral witness refuge · **Quest giver / lead:** Darian

**Player objective:** Find survivor and protect extraction.

**Stages (defense):**
1. Identify who or what needs protection with Darian.
2. Prepare the safe approach and evacuation route.
3. Complete the protection objective: Find survivor and protect extraction.
4. Confirm survivors and record completion with Darian.

**Player-facing dialogue intent:** The contact explains why find survivor and protect extraction matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q111_DONE`; prerequisites `HUMAN_Q110_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q112 — The Delegation's Second Ledger

**Track:** Main story · **Level:** 45+ · **Location:** Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud · **Quest giver / lead:** Mara

**Player objective:** Compare human and hostile-coast contract records without relying on friendly Dominion contacts.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare human and hostile-coast contract records without relying on friendly Dominion contacts.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why compare human and hostile-coast contract records without relying on friendly Dominion contacts matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q112_DONE`; prerequisites `HUMAN_Q111_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q113 — The False Safe House

**Track:** Main story · **Level:** 45+ · **Location:** Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud · **Quest giver / lead:** Sera

**Player objective:** Locate Su'vareth's headquarters using three independent trails and expose its decoy.

**Stages (interaction):**
1. Accept the assignment from Sera.
2. Travel to or inspect the relevant site in Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud.
3. Complete the objective: Locate Su'vareth's headquarters using three independent trails and expose its decoy.
4. Verify the result and report to Sera.

**Player-facing dialogue intent:** The contact explains why locate Su'vareth's headquarters using three independent trails and expose its decoy matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q113_DONE`; prerequisites `HUMAN_Q112_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q114 — The Veiled Hand

**Track:** Main story · **Level:** 45+ · **Location:** Guardian sanctuary · **Quest giver / lead:** Oth'varen

**Player objective:** Present corroborated pattern identifying Su'vareth.

**Stages (investigation):**
1. Receive the case brief from Oth'varen and register the inquiry.
2. Inspect at least two independent sources relevant to: Present corroborated pattern identifying Su'vareth.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Oth'varen for the next lead.

**Player-facing dialogue intent:** The contact explains why present corroborated pattern identifying Su'vareth matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q114_DONE`; prerequisites `HUMAN_Q113_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q115 — The Witness Vault

**Track:** Main story · **Level:** 45+ · **Location:** Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud · **Quest giver / lead:** Oth'varen

**Player objective:** Complete the solo dungeon; defeat Malrec, interrupt the prepared projection and secure the authenticated transaction archive.

**Stages (narrative-critical):**
1. Su'vareth's deception must be disproved by independent sources.
2. Solo dungeon: decoy evidence, captive routes, a real founder confrontation.
3. Su'vareth dies permanently; standing contingency orders survive.

**Player-facing dialogue intent:** Su'vareth's deception must be disproved by independent sources

**Completion / dependencies:** `HUMAN_Q115_DONE`; prerequisites `HUMAN_Q114_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q116 — Orders That Outlive Her

**Track:** Main story · **Level:** 45+ · **Location:** Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud · **Quest giver / lead:** Mara

**Player objective:** Authenticate standing contingency orders and alert affected sites; do not imply Su'vareth survived.

**Stages (interaction):**
1. Accept the assignment from Mara.
2. Travel to or inspect the relevant site in Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud.
3. Complete the objective: Authenticate standing contingency orders and alert affected sites; do not imply Su'vareth survived.
4. Verify the result and report to Mara.

**Player-facing dialogue intent:** The contact explains why authenticate standing contingency orders and alert affected sites; do not imply Su'vareth survived matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q116_DONE`; prerequisites `HUMAN_Q115_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q117 — A Truth Neither Side Wants

**Track:** Main story · **Level:** 45+ · **Location:** Valecrown court · **Quest giver / lead:** Aldren / Mara

**Player objective:** Submit verified case and secure correction.

**Stages (interaction):**
1. Accept the assignment from Aldren / Mara.
2. Travel to or inspect the relevant site in Valecrown court.
3. Complete the objective: Submit verified case and secure correction.
4. Verify the result and report to Aldren / Mara.

**Player-facing dialogue intent:** The contact explains why submit verified case and secure correction matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q117_DONE`; prerequisites `HUMAN_Q116_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q118 — The War That Remains

**Track:** Main story · **Level:** 45+ · **Location:** Oth'varen sanctuary · **Quest giver / lead:** Oth'varen

**Player objective:** Translate Su'vareth orders and identify exact Anchor lead.

**Stages (narrative-critical):**
1. Oth'varen translates a modern recovered document, not pre-known secret orders.
2. Identify a specific Veilbreaker equipment destination that Sera can act on immediately.
3. Do not explain the full Binding at this point.

**Player-facing dialogue intent:** Oth'varen translates a modern recovered document, not pre-known secret orders

**Completion / dependencies:** `HUMAN_Q118_DONE`; prerequisites `HUMAN_Q117_DONE`; minimum level 45. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q119 — The Warden's Orders

**Track:** Main story · **Level:** 50+ · **Location:** Drom Kar · **Quest giver / lead:** Sera

**Player objective:** Authenticate translated delivery orders.

**Stages (interaction):**
1. Accept the assignment from Sera.
2. Travel to or inspect the relevant site in Drom Kar.
3. Complete the objective: Authenticate translated delivery orders.
4. Verify the result and report to Sera.

**Player-facing dialogue intent:** The contact explains why authenticate translated delivery orders matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q119_DONE`; prerequisites `HUMAN_Q118_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q120 — Network Awakens

**Track:** Main story · **Level:** 50+ · **Location:** Covenant Regional Anchor · **Quest giver / lead:** Sera

**Player objective:** Install sensors and compare abnormal readings.

**Stages (interaction):**
1. Accept the assignment from Sera.
2. Travel to or inspect the relevant site in Covenant Regional Anchor.
3. Complete the objective: Install sensors and compare abnormal readings.
4. Verify the result and report to Sera.

**Player-facing dialogue intent:** The contact explains why install sensors and compare abnormal readings matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q120_DONE`; prerequisites `HUMAN_Q119_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q121 — The First Breach

**Track:** Main story · **Level:** 50+ · **Location:** Anchor perimeter · **Quest giver / lead:** Nessa

**Player objective:** Defeat Unmade installers and preserve equipment.

**Stages (interaction):**
1. Accept the assignment from Nessa.
2. Travel to or inspect the relevant site in Anchor perimeter.
3. Complete the objective: Defeat Unmade installers and preserve equipment.
4. Verify the result and report to Nessa.

**Player-facing dialogue intent:** The contact explains why defeat Unmade installers and preserve equipment matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q121_DONE`; prerequisites `HUMAN_Q120_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q122 — A Signal Answered

**Track:** Main story · **Level:** 50+ · **Location:** Signal station · **Quest giver / lead:** Sera

**Player objective:** Intercept Warden's relay traffic.

**Stages (interaction):**
1. Accept the assignment from Sera.
2. Travel to or inspect the relevant site in Signal station.
3. Complete the objective: Intercept Warden's relay traffic.
4. Verify the result and report to Sera.

**Player-facing dialogue intent:** The contact explains why intercept Warden's relay traffic matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q122_DONE`; prerequisites `HUMAN_Q121_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q123 — The Warden's Approaches

**Track:** Main story · **Level:** 50+ · **Location:** Contested mountain route · **Quest giver / lead:** Nessa

**Player objective:** Scout convoys and identify supply terminus.

**Stages (travel):**
1. Receive route authorization from Nessa.
2. Travel physically to Contested mountain route using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Scout convoys and identify supply terminus.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why scout convoys and identify supply terminus matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q123_DONE`; prerequisites `HUMAN_Q122_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q124 — Lines of Defense

**Track:** Main story · **Level:** 50+ · **Location:** Covenant frontier · **Quest giver / lead:** Garrick

**Player objective:** Defend outpost and evacuate civilians.

**Stages (defense):**
1. Identify who or what needs protection with Garrick.
2. Prepare the safe approach and evacuation route.
3. Complete the protection objective: Defend outpost and evacuate civilians.
4. Confirm survivors and record completion with Garrick.

**Player-facing dialogue intent:** The contact explains why defend outpost and evacuate civilians matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q124_DONE`; prerequisites `HUMAN_Q123_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q125 — The Anchor Beneath

**Track:** Main story · **Level:** 50+ · **Location:** Buried installation · **Quest giver / lead:** Tarek

**Player objective:** Recover synchronization schematics.

**Stages (interaction):**
1. Accept the assignment from Tarek.
2. Travel to or inspect the relevant site in Buried installation.
3. Complete the objective: Recover synchronization schematics.
4. Verify the result and report to Tarek.

**Player-facing dialogue intent:** The contact explains why recover synchronization schematics matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q125_DONE`; prerequisites `HUMAN_Q124_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q126 — A Measure of Time

**Track:** Main story · **Level:** 50+ · **Location:** Drom Kar research chamber · **Quest giver / lead:** Sera / Oth'varen

**Player objective:** Compare activation timing across sites.

**Stages (travel):**
1. Receive route authorization from Sera / Oth'varen.
2. Travel physically to Drom Kar research chamber using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Compare activation timing across sites.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why compare activation timing across sites matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q126_DONE`; prerequisites `HUMAN_Q125_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q127 — The Gathering Storm

**Track:** Main story · **Level:** 50+ · **Location:** Valecrown court · **Quest giver / lead:** Aldren

**Player objective:** Present proof and obtain limited mandate.

**Stages (investigation):**
1. Receive the case brief from Aldren and register the inquiry.
2. Inspect at least two independent sources relevant to: Present proof and obtain limited mandate.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Aldren for the next lead.

**Player-facing dialogue intent:** The contact explains why present proof and obtain limited mandate matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q127_DONE`; prerequisites `HUMAN_Q126_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q128 — Before the Veil Breaks

**Track:** Main story · **Level:** 50+ · **Location:** Guardian staging camp · **Quest giver / lead:** Nessa

**Player objective:** Verify extraction, supply and instrument teams.

**Stages (investigation):**
1. Receive the case brief from Nessa and register the inquiry.
2. Inspect at least two independent sources relevant to: Verify extraction, supply and instrument teams.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Nessa for the next lead.

**Player-facing dialogue intent:** The contact explains why verify extraction, supply and instrument teams matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q128_DONE`; prerequisites `HUMAN_Q127_DONE`; minimum level 50. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

## Build acceptance checklist

- Every quest has an NPC interaction, a visible player objective, stage-by-stage credit, a safe failure/retry route, and a verified turn-in.
- Travel cannot be bypassed by invented waypoints; hostile Dominion zones remain hostile and PvE story objectives cannot be blocked by opposing players.
- Dialogue must respect knowledge timing and previously established choices. A player's sample-preservation choice changes follow-up reactions only, not main outcome.
- Combat encounters require actual spawn and leash data, aggro/threat behavior, target facing checks, checkpoint tests and boss scripting before implementation.
- Replace generic stages with precise actor, object, counter and trigger IDs after approved maps and encounter blueprints are available.
- Review the chapter in-game for pacing, XP, difficulty, travel time, accessibility, and solo completion.

**Release blockers:** exact world geometry, quest NPC/creature/item IDs, complete dialogue scripts, cinematic assets, XP and loot tuning, encounter scripting, server quest integration and playtest results. None are asserted complete here.
