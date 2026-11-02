# Human 35–45 — Detailed Quest Implementation Brief v0.2

**Status: production-authoring draft, NOT implemented or playtested.** Story authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Quest classification and flags: [Quest Dependencies](Human_1_80_Quest_Dependencies.json). This document adds per-quest stage plans, player-facing narrative beats and QA criteria. Where a scene is not yet scripted, dialogue below is an **intent**, not finalized spoken dialogue.

## Shared rules

- Classic-style manual target/facing/range and autoattack. No forced PvP, mandatory group, profession, or class-specific ritual.
- Evidence credits are server-authoritative, idempotent and stored in the case journal; failing or abandoning a quest cannot permanently destroy critical evidence.
- Solo story instances checkpoint completed stages; defeat/disconnect resets hostile encounter state but preserves prior evidence and valid choices.
- Region travel is by ship, fixed flight paths and road/foot/mount; only consented Guardian/Kel'vari Passage access is exceptional. No ordinary travel into Shroud before Veilbreaker.
- No coordinates, aggro radii, combat stats, XP values, item IDs or runtime APIs are claimed as implemented.

## Quest-by-quest stage plans

### Q080 — Aftermath of Inversion

**Track:** Main story · **Level:** 35+ · **Location:** Drom Kar laboratory · **Quest giver / lead:** Sera

**Player objective:** Show the captured Passage records and request explanation.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Show the captured Passage records and request explanation.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why show the captured Passage records and request explanation matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q080_DONE`; prerequisites `HUMAN_Q079_DONE`; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q081 — The Gate They Kept

**Track:** Main story · **Level:** 35+ · **Location:** Guardian sanctuary · **Quest giver / lead:** Sera

**Player objective:** Present proof of the destroyed gate and demand disclosure.

**Stages (narrative-critical):**
1. Sera initially withholds the truth because she fears exposing Kel'vari.
2. Present evidence of Oma'vira's destroyed gate; Sera acknowledges the Guardians' own restricted gate.
3. Trust reaction can change, but access still requires Kel'vari permission.

**Player-facing dialogue intent:** Sera initially withholds the truth because she fears exposing Kel'vari

**Completion / dependencies:** `HUMAN_Q081_DONE`; prerequisites `HUMAN_Q080_DONE`; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q082 — The Price of Secrecy

**Track:** Main story · **Level:** 35+ · **Location:** Restricted archive · **Quest giver / lead:** Guardian archivist

**Player objective:** Inspect gate maintenance and consent logs.

**Stages (travel):**
1. Receive route authorization from Guardian archivist.
2. Travel physically to Restricted archive using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Inspect gate maintenance and consent logs.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why inspect gate maintenance and consent logs matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q082_DONE`; prerequisites `HUMAN_Q081_DONE`; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q083 — The Route That Never Closed

**Track:** Regional / optional · **Level:** 35+ · **Location:** Drom Kar, hidden Guardian gate, Lantern Redoubt, Kel'vari homeland · **Quest giver / lead:** Tarek / Dromak

**Player objective:** Survey the separate low-capacity Passage, test a small-group return path and deliver an expedition proposal to Corven.

**Stages (travel):**
1. Receive route authorization from Tarek / Dromak.
2. Travel physically to Drom Kar, hidden Guardian gate, Lantern Redoubt, Kel'vari homeland using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Survey the separate low-capacity Passage, test a small-group return path and deliver an expedition proposal to Corven.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why survey the separate low-capacity Passage, test a small-group return path and deliver an expedition proposal to Corven matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q083_DONE`; prerequisites none; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q084 — Permission to Enter

**Track:** Main story · **Level:** 35+ · **Location:** Guardian sanctuary · **Quest giver / lead:** Sera / Kel'vari custodian

**Player objective:** Obtain permission for a small supervised delegation.

**Stages (interaction):**
1. Accept the assignment from Sera / Kel'vari custodian.
2. Travel to or inspect the relevant site in Guardian sanctuary.
3. Complete the objective: Obtain permission for a small supervised delegation.
4. Verify the result and report to Sera / Kel'vari custodian.

**Player-facing dialogue intent:** The contact explains why obtain permission for a small supervised delegation matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q084_DONE`; prerequisites `HUMAN_Q082_DONE`; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q085 — The Hidden Crossing

**Track:** Main story · **Level:** 35+ · **Location:** Restricted Guardian Passage · **Quest giver / lead:** Tarek

**Player objective:** Cross with approved expedition and verify return capacity.

**Stages (narrative-critical):**
1. The gate is an ancient low-capacity exception, not ordinary fast travel.
2. Supervised crossing, limited group size, checked return route.
3. Kel'vari remain sovereign; never convert Lantern Redoubt into a Covenant military fort.

**Player-facing dialogue intent:** The gate is an ancient low-capacity exception, not ordinary fast travel

**Completion / dependencies:** `HUMAN_Q085_DONE`; prerequisites `HUMAN_Q084_DONE`; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q086 — Lantern Redoubt

**Track:** Main story · **Level:** 35+ · **Location:** Drom Kar, hidden Guardian gate, Lantern Redoubt, Kel'vari homeland · **Quest giver / lead:** Nessa / Aru'ven

**Player objective:** Cross in a small surveyed group and agree access/maintenance terms for this outer site; no continent-wide gate restoration.

**Stages (narrative-critical):**
1. First mandatory encounter with living Kel'vari happens here.
2. Meet custodians in a long-standing joint outpost; show a thriving culture beyond ruins.
3. The player should not already know the Kel'vari by name from public history.

**Player-facing dialogue intent:** First mandatory encounter with living Kel'vari happens here

**Completion / dependencies:** `HUMAN_Q086_DONE`; prerequisites `HUMAN_Q085_DONE`; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q087 — Those Who Remained

**Track:** Regional / optional · **Level:** 35+ · **Location:** Kel'vari settlement · **Quest giver / lead:** Aru'ven

**Player objective:** Rescue work party and meet surviving Kel'vari.

**Stages (defense):**
1. Identify who or what needs protection with Aru'ven.
2. Prepare the safe approach and evacuation route.
3. Complete the protection objective: Rescue work party and meet surviving Kel'vari.
4. Confirm survivors and record completion with Aru'ven.

**Player-facing dialogue intent:** The contact explains why rescue work party and meet surviving Kel'vari matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q087_DONE`; prerequisites none; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q088 — Permission to Remain

**Track:** Main story · **Level:** 35+ · **Location:** Lantern Redoubt · **Quest giver / lead:** Aru'ven

**Player objective:** Agree boundaries, supplies and site access.

**Stages (travel):**
1. Receive route authorization from Aru'ven.
2. Travel physically to Lantern Redoubt using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Agree boundaries, supplies and site access.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why agree boundaries, supplies and site access matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q088_DONE`; prerequisites `HUMAN_Q086_DONE`; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q089 — The Broken Path

**Track:** Main story · **Level:** 35+ · **Location:** Ancient Passage ruins · **Quest giver / lead:** Tarek

**Player objective:** Recover intrusion traces linked to Oma'vira.

**Stages (travel):**
1. Receive route authorization from Tarek.
2. Travel physically to Ancient Passage ruins using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Recover intrusion traces linked to Oma'vira.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why recover intrusion traces linked to Oma'vira matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q089_DONE`; prerequisites `HUMAN_Q088_DONE`; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q090 — A Warning Preserved

**Track:** Regional / optional · **Level:** 35+ · **Location:** Kel'vari record store · **Quest giver / lead:** Sel'ora

**Player objective:** Stabilize damaged warning and preserve copy.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Stabilize damaged warning and preserve copy.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why stabilize damaged warning and preserve copy matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q090_DONE`; prerequisites none; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q091 — Questions for the Sixth

**Track:** Main story · **Level:** 35+ · **Location:** Lantern Redoubt · **Quest giver / lead:** Sel'ora

**Player objective:** Request supervised archive audience.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Request supervised archive audience.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why request supervised archive audience matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q091_DONE`; prerequisites `HUMAN_Q089_DONE`; minimum level 35. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q092 — The Custodian's Conditions

**Track:** Main story · **Level:** 40+ · **Location:** Oth'irun and Kel'vari archive approaches · **Quest giver / lead:** Sel'ora

**Player objective:** Meet the archive party at a protected approach, accept access limits and recover its missing copyist.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Meet the archive party at a protected approach, accept access limits and recover its missing copyist.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why meet the archive party at a protected approach, accept access limits and recover its missing copyist matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q092_DONE`; prerequisites `HUMAN_Q091_DONE`; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q093 — An Audience Earned

**Track:** Regional / optional · **Level:** 40+ · **Location:** Oth'irun archive · **Quest giver / lead:** Sel'ora

**Player objective:** Secure exposed records and access approval.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Secure exposed records and access approval.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why secure exposed records and access approval matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q093_DONE`; prerequisites none; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q094 — Six Names in the Record

**Track:** Main story · **Level:** 40+ · **Location:** Oth'irun and Kel'vari archive approaches · **Quest giver / lead:** Sel'ora / Mara

**Player objective:** Compare the project list, verifier image and distinctive stability sequence; request the Surveyor's explanation.

**Stages (investigation):**
1. Receive the case brief from Sel'ora / Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare the project list, verifier image and distinctive stability sequence; request the Surveyor's explanation.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora / Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why compare the project list, verifier image and distinctive stability sequence; request the Surveyor's explanation matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q094_DONE`; prerequisites `HUMAN_Q092_DONE`; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q095 — The Scholar Who Remained

**Track:** Main story · **Level:** 40+ · **Location:** Oth'irun archive · **Quest giver / lead:** Leth'ari

**Player objective:** Match verifier mark to ancient instruments.

**Stages (investigation):**
1. Receive the case brief from Leth'ari and register the inquiry.
2. Inspect at least two independent sources relevant to: Match verifier mark to ancient instruments.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Leth'ari for the next lead.

**Player-facing dialogue intent:** The contact explains why match verifier mark to ancient instruments matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q095_DONE`; prerequisites `HUMAN_Q094_DONE`; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q096 — The Verifier's Mark

**Track:** Main story · **Level:** 40+ · **Location:** Oth'irun and Kel'vari archive approaches · **Quest giver / lead:** Leth'ari

**Player objective:** Match a project signature with independently preserved instrument records.

**Stages (investigation):**
1. Receive the case brief from Leth'ari and register the inquiry.
2. Inspect at least two independent sources relevant to: Match a project signature with independently preserved instrument records.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Leth'ari for the next lead.

**Player-facing dialogue intent:** The contact explains why match a project signature with independently preserved instrument records matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q096_DONE`; prerequisites `HUMAN_Q095_DONE`; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q097 — The Last Witness

**Track:** Main story · **Level:** 40+ · **Location:** Oth'irun and Kel'vari archive approaches · **Quest giver / lead:** Oth'varen

**Player objective:** Hear his identity and firsthand account, then cross-check independent records; trust may be qualified rather than absolute.

**Stages (narrative-critical):**
1. Oth'varen is revealed by independent verifier records, not his unsupported claim.
2. Player questions the Grey Surveyor and hears the sixth scholar's firsthand account.
3. Oth'varen is knowledgeable but not omniscient.

**Player-facing dialogue intent:** Oth'varen is revealed by independent verifier records, not his unsupported claim

**Completion / dependencies:** `HUMAN_Q097_DONE`; prerequisites `HUMAN_Q096_DONE`; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q098 — Echoes Before the Sundering

**Track:** Main story · **Level:** 40+ · **Location:** Ancient archive · **Quest giver / lead:** Sel'ora

**Player objective:** Reconstruct early interference from original records.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Reconstruct early interference from original records.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why reconstruct early interference from original records matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q098_DONE`; prerequisites `HUMAN_Q097_DONE`; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q099 — The Six Reunited

**Track:** Main story · **Level:** 40+ · **Location:** Historical projection chamber · **Quest giver / lead:** Sel'ora

**Player objective:** Witness six scholars' recorded dispute.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Witness six scholars' recorded dispute.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why witness six scholars' recorded dispute matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q099_DONE`; prerequisites `HUMAN_Q098_DONE`; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q100 — The Cost of Keeping Quiet

**Track:** Regional / optional · **Level:** 40+ · **Location:** Oth'irun and Kel'vari archive approaches · **Quest giver / lead:** Sera

**Player objective:** Rescue threatened custodians using an authorised route disclosure; preserve enclave security through restricted case access.

**Stages (travel):**
1. Receive route authorization from Sera.
2. Travel physically to Oth'irun and Kel'vari archive approaches using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Rescue threatened custodians using an authorised route disclosure; preserve enclave security through restricted case access.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why rescue threatened custodians using an authorised route disclosure; preserve enclave security through restricted case access matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q100_DONE`; prerequisites none; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q101 — Two Histories

**Track:** Regional / optional · **Level:** 40+ · **Location:** Oth'irun archive · **Quest giver / lead:** Sel'ora

**Player objective:** Compare conflicting historical copies.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare conflicting historical copies.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why compare conflicting historical copies matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q101_DONE`; prerequisites none; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q102 — The Delegation's Shadow

**Track:** Main story · **Level:** 40+ · **Location:** Oth'irun archive · **Quest giver / lead:** Mara

**Player objective:** Compare Delegation dates with archival entries.

**Stages (interaction):**
1. Accept the assignment from Mara.
2. Travel to or inspect the relevant site in Oth'irun archive.
3. Complete the objective: Compare Delegation dates with archival entries.
4. Verify the result and report to Mara.

**Player-facing dialogue intent:** The contact explains why compare Delegation dates with archival entries matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q102_DONE`; prerequisites `HUMAN_Q099_DONE`; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q103 — The Scholar's Request

**Track:** Main story · **Level:** 40+ · **Location:** Oth'varen's study · **Quest giver / lead:** Oth'varen

**Player objective:** Accept bounded Shattered Delegation investigation.

**Stages (travel):**
1. Receive route authorization from Oth'varen.
2. Travel physically to Oth'varen's study using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Accept bounded Shattered Delegation investigation.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why accept bounded Shattered Delegation investigation matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q103_DONE`; prerequisites `HUMAN_Q102_DONE`; minimum level 40. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

## Build acceptance checklist

- Every quest has an NPC interaction, a visible player objective, stage-by-stage credit, a safe failure/retry route, and a verified turn-in.
- Travel cannot be bypassed by invented waypoints; hostile Dominion zones remain hostile and PvE story objectives cannot be blocked by opposing players.
- Dialogue must respect knowledge timing and previously established choices. A player's sample-preservation choice changes follow-up reactions only, not main outcome.
- Combat encounters require actual spawn and leash data, aggro/threat behavior, target facing checks, checkpoint tests and boss scripting before implementation.
- Replace generic stages with precise actor, object, counter and trigger IDs after approved maps and encounter blueprints are available.
- Review the chapter in-game for pacing, XP, difficulty, travel time, accessibility, and solo completion.

**Release blockers:** exact world geometry, quest NPC/creature/item IDs, complete dialogue scripts, cinematic assets, XP and loot tuning, encounter scripting, server quest integration and playtest results. None are asserted complete here.
