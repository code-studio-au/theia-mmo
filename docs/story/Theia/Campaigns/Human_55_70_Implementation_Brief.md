# Human 55–70 — Detailed Quest Implementation Brief v0.2

**Status: production-authoring draft, NOT implemented or playtested.** Story authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Quest classification and flags: [Quest Dependencies](Human_1_80_Quest_Dependencies.json). This document adds per-quest stage plans, player-facing narrative beats and QA criteria. Where a scene is not yet scripted, dialogue below is an **intent**, not finalized spoken dialogue.

## Shared rules

- Classic-style manual target/facing/range and autoattack. No forced PvP, mandatory group, profession, or class-specific ritual.
- Evidence credits are server-authoritative, idempotent and stored in the case journal; failing or abandoning a quest cannot permanently destroy critical evidence.
- Solo story instances checkpoint completed stages; defeat/disconnect resets hostile encounter state but preserves prior evidence and valid choices.
- Region travel is by ship, fixed flight paths and road/foot/mount; only consented Guardian/Kel'vari Passage access is exceptional. No ordinary travel into Shroud before Veilbreaker.
- No coordinates, aggro radii, combat stats, XP values, item IDs or runtime APIs are claimed as implemented.

## Quest-by-quest stage plans

### Q129 — The Warden Unmasked

**Track:** Main story · **Level:** 55+ · **Location:** Fortified relay · **Quest giver / lead:** Nessa

**Player objective:** Recover signed Khor'vun orders.

**Stages (interaction):**
1. Accept the assignment from Nessa.
2. Travel to or inspect the relevant site in Fortified relay.
3. Complete the objective: Recover signed Khor'vun orders.
4. Verify the result and report to Nessa.

**Player-facing dialogue intent:** The contact explains why recover signed Khor'vun orders matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q129_DONE`; prerequisites `HUMAN_Q128_DONE`; minimum level 55. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q130 — Anchors in Concert

**Track:** Main story · **Level:** 55+ · **Location:** Linked relay sites · **Quest giver / lead:** Tarek

**Player objective:** Measure synchronized signatures at three locations.

**Stages (interaction):**
1. Accept the assignment from Tarek.
2. Travel to or inspect the relevant site in Linked relay sites.
3. Complete the objective: Measure synchronized signatures at three locations.
4. Verify the result and report to Tarek.

**Player-facing dialogue intent:** The contact explains why measure synchronized signatures at three locations matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q130_DONE`; prerequisites `HUMAN_Q129_DONE`; minimum level 55. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q131 — Siege of Lantern Redoubt

**Track:** Main story · **Level:** 55+ · **Location:** Lantern Redoubt · **Quest giver / lead:** Aru'ven

**Player objective:** Defend civilians and extraction gate.

**Stages (travel):**
1. Receive route authorization from Aru'ven.
2. Travel physically to Lantern Redoubt using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Defend civilians and extraction gate.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why defend civilians and extraction gate matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q131_DONE`; prerequisites `HUMAN_Q130_DONE`; minimum level 55. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q132 — The Pattern Breaks

**Track:** Main story · **Level:** 55+ · **Location:** Guardian station · **Quest giver / lead:** Sera

**Player objective:** Locate vulnerable control relay.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Locate vulnerable control relay.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why locate vulnerable control relay matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q132_DONE`; prerequisites `HUMAN_Q131_DONE`; minimum level 55. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q133 — Into the Warden's Forge

**Track:** Main story · **Level:** 55+ · **Location:** Relay solo dungeon · **Quest giver / lead:** Nessa

**Player objective:** Fight defenders and reach relay controls.

**Stages (combat):**
1. Receive encounter authorization and review evacuation/checkpoint conditions.
2. Reach the combat site in Relay solo dungeon and secure the entry.
3. Complete the encounter objective: Fight defenders and reach relay controls.
4. Recover evidence and return to Nessa; preserve solo retry.

**Player-facing dialogue intent:** The contact explains why fight defenders and reach relay controls matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q133_DONE`; prerequisites `HUMAN_Q132_DONE`; minimum level 55. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q134 — The Severed Signal

**Track:** Main story · **Level:** 55+ · **Location:** Relay control chamber · **Quest giver / lead:** Sera

**Player objective:** Disable synchronization branch and preserve data.

**Stages (interaction):**
1. Accept the assignment from Sera.
2. Travel to or inspect the relevant site in Relay control chamber.
3. Complete the objective: Disable synchronization branch and preserve data.
4. Verify the result and report to Sera.

**Player-facing dialogue intent:** The contact explains why disable synchronization branch and preserve data matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q134_DONE`; prerequisites `HUMAN_Q133_DONE`; minimum level 55. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q135 — Aftermath of the Break

**Track:** Main story · **Level:** 55+ · **Location:** Guardian staging camp · **Quest giver / lead:** Sera

**Player objective:** Compare surviving signals after branch shutdown.

**Stages (interaction):**
1. Accept the assignment from Sera.
2. Travel to or inspect the relevant site in Guardian staging camp.
3. Complete the objective: Compare surviving signals after branch shutdown.
4. Verify the result and report to Sera.

**Player-facing dialogue intent:** The contact explains why compare surviving signals after branch shutdown matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q135_DONE`; prerequisites `HUMAN_Q134_DONE`; minimum level 55. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q136 — What the Warden Left Behind

**Track:** Main story · **Level:** 55+ · **Location:** Relay archive · **Quest giver / lead:** Mara

**Player objective:** Recover First Voice instructions.

**Stages (investigation):**
1. Receive the case brief from Mara and register the inquiry.
2. Inspect at least two independent sources relevant to: Recover First Voice instructions.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Mara for the next lead.

**Player-facing dialogue intent:** The contact explains why recover First Voice instructions matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q136_DONE`; prerequisites `HUMAN_Q135_DONE`; minimum level 55. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q137 — Records of the First Voice

**Track:** Main story · **Level:** 60+ · **Location:** Kel'vari archive · **Quest giver / lead:** Sel'ora

**Player objective:** Retrieve Vaer'uun's early research papers.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Retrieve Vaer'uun's early research papers.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why retrieve Vaer'uun's early research papers matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q137_DONE`; prerequisites `HUMAN_Q136_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q138 — The Forgotten Investigation

**Track:** Main story · **Level:** 60+ · **Location:** Abandoned research facility · **Quest giver / lead:** Oth'varen

**Player objective:** Recover recorded scientific debate.

**Stages (investigation):**
1. Receive the case brief from Oth'varen and register the inquiry.
2. Inspect at least two independent sources relevant to: Recover recorded scientific debate.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Oth'varen for the next lead.

**Player-facing dialogue intent:** The contact explains why recover recorded scientific debate matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q138_DONE`; prerequisites `HUMAN_Q137_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q139 — The Missing Records

**Track:** Main story · **Level:** 60+ · **Location:** Restricted archive · **Quest giver / lead:** Sel'ora

**Player objective:** Restore omitted research pages.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Restore omitted research pages.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why restore omitted research pages matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q139_DONE`; prerequisites `HUMAN_Q138_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q140 — The Voice That Answers

**Track:** Main story · **Level:** 60+ · **Location:** Ancient communicator · **Quest giver / lead:** Vaer'uun transmission

**Player objective:** Receive invitation and verify destination.

**Stages (narrative-critical):**
1. Vaer'uun uses genuine suppressed facts to manipulate the player's trust.
2. Invitation names a research site and asks why Oth'varen concealed the Shroud's origins.
3. The player obtains evidence but is not forced to believe Vaer'uun.

**Player-facing dialogue intent:** Vaer'uun uses genuine suppressed facts to manipulate the player's trust

**Completion / dependencies:** `HUMAN_Q140_DONE`; prerequisites `HUMAN_Q139_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q141 — Beneath the Binding

**Track:** Main story · **Level:** 60+ · **Location:** Ancient research site · **Quest giver / lead:** Sera

**Player objective:** Inspect Binding evidence and interrupt Unmade exploit.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Inspect Binding evidence and interrupt Unmade exploit.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why inspect Binding evidence and interrupt Unmade exploit matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q141_DONE`; prerequisites `HUMAN_Q140_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q142 — Truth Withheld

**Track:** Main story · **Level:** 60+ · **Location:** Oth'varen sanctuary · **Quest giver / lead:** Oth'varen

**Player objective:** Confront Oth'varen with independently copied evidence.

**Stages (investigation):**
1. Receive the case brief from Oth'varen and register the inquiry.
2. Inspect at least two independent sources relevant to: Confront Oth'varen with independently copied evidence.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Oth'varen for the next lead.

**Player-facing dialogue intent:** The contact explains why confront Oth'varen with independently copied evidence matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q142_DONE`; prerequisites `HUMAN_Q141_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q143 — Name in the Records

**Track:** Main story · **Level:** 60+ · **Location:** Kel'vari archive · **Quest giver / lead:** Sel'ora

**Player objective:** Authenticate original reference to Nhal.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Authenticate original reference to Nhal.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why authenticate original reference to Nhal matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q143_DONE`; prerequisites `HUMAN_Q142_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q144 — The First Voice's Design

**Track:** Main story · **Level:** 60+ · **Location:** Unmade research site · **Quest giver / lead:** Sera

**Player objective:** Disable mechanism exploiting player's access.

**Stages (combat):**
1. Receive encounter authorization and review evacuation/checkpoint conditions.
2. Reach the combat site in Unmade research site and secure the entry.
3. Complete the encounter objective: Disable mechanism exploiting player's access.
4. Recover evidence and return to Sera; preserve solo retry.

**Player-facing dialogue intent:** The contact explains why disable mechanism exploiting player's access matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q144_DONE`; prerequisites `HUMAN_Q143_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q145 — The First Voice

**Track:** Main story · **Level:** 60+ · **Location:** First Voice solo dungeon · **Quest giver / lead:** Sera

**Player objective:** Defeat Vaer'uun and isolate apparatus.

**Stages (narrative-critical):**
1. Vaer'uun's arguments contain truth but his purpose serves Nhal.
2. Solo dungeon: disable machinery exploiting research access; confront and kill Vaer'uun.
3. Keeper's deception is still not fully disclosed.

**Player-facing dialogue intent:** Vaer'uun's arguments contain truth but his purpose serves Nhal

**Completion / dependencies:** `HUMAN_Q145_DONE`; prerequisites `HUMAN_Q144_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q146 — The Silence Beneath

**Track:** Main story · **Level:** 60+ · **Location:** Sanctuary · **Quest giver / lead:** Oth'varen

**Player objective:** Compare Vaer'uun claims with historical testimony.

**Stages (investigation):**
1. Receive the case brief from Oth'varen and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare Vaer'uun claims with historical testimony.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Oth'varen for the next lead.

**Player-facing dialogue intent:** The contact explains why compare Vaer'uun claims with historical testimony matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q146_DONE`; prerequisites `HUMAN_Q145_DONE`; minimum level 60. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q147 — The Weight of Silence

**Track:** Main story · **Level:** 65+ · **Location:** Oth'varen sanctuary · **Quest giver / lead:** Sera

**Player objective:** Demand full account of Binding and Keeper.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Demand full account of Binding and Keeper.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why demand full account of Binding and Keeper matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q147_DONE`; prerequisites `HUMAN_Q146_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q148 — The Sealed Testimony

**Track:** Main story · **Level:** 65+ · **Location:** Kel'vari restricted archive · **Quest giver / lead:** Sel'ora

**Player objective:** Recover independent Divine War testimonies.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Recover independent Divine War testimonies.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why recover independent Divine War testimonies matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q148_DONE`; prerequisites `HUMAN_Q147_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q149 — The Missing Account

**Track:** Main story · **Level:** 65+ · **Location:** Ancient testimony vault · **Quest giver / lead:** Sel'ora

**Player objective:** Reconstruct omitted Keeper account.

**Stages (investigation):**
1. Receive the case brief from Sel'ora and register the inquiry.
2. Inspect at least two independent sources relevant to: Reconstruct omitted Keeper account.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sel'ora for the next lead.

**Player-facing dialogue intent:** The contact explains why reconstruct omitted Keeper account matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q149_DONE`; prerequisites `HUMAN_Q148_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q150 — Guardian's Doubt

**Track:** Main story · **Level:** 65+ · **Location:** Historical chamber · **Quest giver / lead:** Guardian testimony archive

**Player objective:** Examine Guardian's post-Binding discovery.

**Stages (interaction):**
1. Accept the assignment from Guardian testimony archive.
2. Travel to or inspect the relevant site in Historical chamber.
3. Complete the objective: Examine Guardian's post-Binding discovery.
4. Verify the result and report to Guardian testimony archive.

**Player-facing dialogue intent:** The contact explains why examine Guardian's post-Binding discovery matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q150_DONE`; prerequisites `HUMAN_Q149_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q151 — The Seven Bound

**Track:** Main story · **Level:** 65+ · **Location:** Sanctuary · **Quest giver / lead:** Oth'varen

**Player objective:** Assemble evidence and hear full Binding account.

**Stages (narrative-critical):**
1. Keeper deceived the Seven; Guardian was deceived and learned afterward.
2. Confront Oth'varen with separately authenticated testimonies.
3. Anchors amplify the original tear; the Binding contains the Veil; Shroud is visible effect.

**Player-facing dialogue intent:** Keeper deceived the Seven; Guardian was deceived and learned afterward

**Completion / dependencies:** `HUMAN_Q151_DONE`; prerequisites `HUMAN_Q150_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q152 — The Cost of Containment

**Track:** Main story · **Level:** 65+ · **Location:** Guardian research chamber · **Quest giver / lead:** Sera

**Player objective:** Compare inversion readings with Binding history.

**Stages (investigation):**
1. Receive the case brief from Sera and register the inquiry.
2. Inspect at least two independent sources relevant to: Compare inversion readings with Binding history.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Sera for the next lead.

**Player-facing dialogue intent:** The contact explains why compare inversion readings with Binding history matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q152_DONE`; prerequisites `HUMAN_Q151_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q153 — The Warden's Final Pattern

**Track:** Main story · **Level:** 65+ · **Location:** Anchor monitoring network · **Quest giver / lead:** Tarek

**Player objective:** Trace Khor'vun final convergence.

**Stages (investigation):**
1. Receive the case brief from Tarek and register the inquiry.
2. Inspect at least two independent sources relevant to: Trace Khor'vun final convergence.
3. Compare and authenticate the findings; add an immutable case-journal copy.
4. Return the verified result to Tarek for the next lead.

**Player-facing dialogue intent:** The contact explains why trace Khor'vun final convergence matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q153_DONE`; prerequisites `HUMAN_Q152_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q154 — Before the Veil Breaks

**Track:** Main story · **Level:** 65+ · **Location:** Expedition staging ground · **Quest giver / lead:** Nessa

**Player objective:** Prepare evacuation and assault support.

**Stages (interaction):**
1. Accept the assignment from Nessa.
2. Travel to or inspect the relevant site in Expedition staging ground.
3. Complete the objective: Prepare evacuation and assault support.
4. Verify the result and report to Nessa.

**Player-facing dialogue intent:** The contact explains why prepare evacuation and assault support matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q154_DONE`; prerequisites `HUMAN_Q153_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q155 — The Last Approach

**Track:** Main story · **Level:** 65+ · **Location:** Veilbreaker complex · **Quest giver / lead:** Nessa

**Player objective:** Disable defenses and reach central apparatus.

**Stages (travel):**
1. Receive route authorization from Nessa.
2. Travel physically to Veilbreaker complex using permitted ships, flight routes, roads or restricted gates.
3. Complete the on-route objective: Disable defenses and reach central apparatus.
4. Reach the destination contact and confirm arrival; record a safe rejoin point.

**Player-facing dialogue intent:** The contact explains why disable defenses and reach central apparatus matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q155_DONE`; prerequisites `HUMAN_Q154_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q156 — Veilbreaker

**Track:** Main story · **Level:** 65+ · **Location:** Veilbreaker solo dungeon · **Quest giver / lead:** Nessa / Sera

**Player objective:** Kill Khor'vun and attempt cutoff after irreversible threshold.

**Stages (narrative-critical):**
1. Khor'vun is defeated but the player arrives too late to stop activation.
2. Boss fight remains winnable; completion triggers irreversible Veil destruction.
3. Binding dissolves immediately, Seven freed, Shroud begins dissipating, Nhal manifests slowly.

**Player-facing dialogue intent:** Khor'vun is defeated but the player arrives too late to stop activation

**Completion / dependencies:** `HUMAN_Q156_DONE`; prerequisites `HUMAN_Q155_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q157 — The Shroud Begins to Fall

**Track:** Main story · **Level:** 65+ · **Location:** Kel'vari frontier · **Quest giver / lead:** Aru'ven

**Player objective:** Evacuate residents as Shroud starts dissipating.

**Stages (narrative-critical):**
1. Dissipation begins immediately after Veilbreaker's success.
2. Protect Kel'vari civilians and Guardian personnel during evacuation.
3. No delayed Shroud dissolution at level 80.

**Player-facing dialogue intent:** Dissipation begins immediately after Veilbreaker's success

**Completion / dependencies:** `HUMAN_Q157_DONE`; prerequisites `HUMAN_Q156_DONE`; minimum level 65. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

## Build acceptance checklist

- Every quest has an NPC interaction, a visible player objective, stage-by-stage credit, a safe failure/retry route, and a verified turn-in.
- Travel cannot be bypassed by invented waypoints; hostile Dominion zones remain hostile and PvE story objectives cannot be blocked by opposing players.
- Dialogue must respect knowledge timing and previously established choices. A player's sample-preservation choice changes follow-up reactions only, not main outcome.
- Combat encounters require actual spawn and leash data, aggro/threat behavior, target facing checks, checkpoint tests and boss scripting before implementation.
- Replace generic stages with precise actor, object, counter and trigger IDs after approved maps and encounter blueprints are available.
- Review the chapter in-game for pacing, XP, difficulty, travel time, accessibility, and solo completion.

**Release blockers:** exact world geometry, quest NPC/creature/item IDs, complete dialogue scripts, cinematic assets, XP and loot tuning, encounter scripting, server quest integration and playtest results. None are asserted complete here.
