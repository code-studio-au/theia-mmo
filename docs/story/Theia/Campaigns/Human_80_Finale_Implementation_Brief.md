# Human 80–Finale — Detailed Quest Implementation Brief v0.2

**Status: production-authoring draft, NOT implemented or playtested.** Story authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Quest classification and flags: [Quest Dependencies](Human_1_80_Quest_Dependencies.json). This document adds per-quest stage plans, player-facing narrative beats and QA criteria. Where a scene is not yet scripted, dialogue below is an **intent**, not finalized spoken dialogue.

## Shared rules

- Classic-style manual target/facing/range and autoattack. No forced PvP, mandatory group, profession, or class-specific ritual.
- Evidence credits are server-authoritative, idempotent and stored in the case journal; failing or abandoning a quest cannot permanently destroy critical evidence.
- Solo story instances checkpoint completed stages; defeat/disconnect resets hostile encounter state but preserves prior evidence and valid choices.
- Region travel is by ship, fixed flight paths and road/foot/mount; only consented Guardian/Kel'vari Passage access is exceptional. No ordinary travel into Shroud before Veilbreaker.
- No coordinates, aggro radii, combat stats, XP values, item IDs or runtime APIs are claimed as implemented.

## Quest-by-quest stage plans

### Q175 — Inside the Architect's Design

**Track:** Main story · **Level:** 80+ · **Location:** Architect central facility · **Quest giver / lead:** Nessa

**Player objective:** Infiltrate and disable defenses while preserving alignment mechanism.

**Stages (interaction):**
1. Accept the assignment from Nessa.
2. Travel to or inspect the relevant site in Architect central facility.
3. Complete the objective: Infiltrate and disable defenses while preserving alignment mechanism.
4. Verify the result and report to Nessa.

**Player-facing dialogue intent:** The contact explains why infiltrate and disable defenses while preserving alignment mechanism matters locally, without revealing information the player has not earned.

**Completion / dependencies:** `HUMAN_Q175_DONE`; prerequisites `HUMAN_Q174_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Detailed counters and interaction objects require map/encounter authoring.

### Q176 — The Architect's Last Argument

**Track:** Main story · **Level:** 80+ · **Location:** Solo boss chamber · **Quest giver / lead:** Thes'var

**Player objective:** Defeat and kill Thes'var.

**Stages (narrative-critical):**
1. Thes'var knowingly serves Nhal; not a misunderstood world-repairer.
2. Kill Thes'var in solo-accessible encounter without destroying convergence machinery.
3. Only afterward does his misalignment become evident.

**Player-facing dialogue intent:** Thes'var knowingly serves Nhal; not a misunderstood world-repairer

**Completion / dependencies:** `HUMAN_Q176_DONE`; prerequisites `HUMAN_Q175_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q177 — The Broken Alignment

**Track:** Main story · **Level:** 80+ · **Location:** Control junction · **Quest giver / lead:** Sera

**Player objective:** Assess Thes'var's sabotage and misalignment.

**Stages (narrative-critical):**
1. Thes'var's final sabotage damages alignment after his death.
2. Sera and Oth'varen verify that conventional correction will fail.
3. Oth'varen has not yet committed to death.

**Player-facing dialogue intent:** Thes'var's final sabotage damages alignment after his death

**Completion / dependencies:** `HUMAN_Q177_DONE`; prerequisites `HUMAN_Q176_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q178 — The Scholar's Correction

**Track:** Main story · **Level:** 80+ · **Location:** Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven · **Quest giver / lead:** Oth'varen / Sera

**Player objective:** Maintain evacuation and instrument stations as Oth'varen stabilises the alignment and dies after seeing it hold.

**Stages (narrative-critical):**
1. Oth'varen learns only now that the correction is fatal.
2. He verifies it with Sera, rather than magically knowing in advance.
3. His informed choice is voluntary and physically lethal.

**Player-facing dialogue intent:** Oth'varen learns only now that the correction is fatal

**Completion / dependencies:** `HUMAN_Q178_DONE`; prerequisites `HUMAN_Q177_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q179 — The Sixth Scholar's Choice

**Track:** Main story · **Level:** 80+ · **Location:** Convergence entrance · **Quest giver / lead:** Oth'varen

**Player objective:** Witness Oth'varen tell Sera and player his choice.

**Stages (narrative-critical):**
1. Oth'varen tells Sera and the player what he intends.
2. Give each a distinct response; no false save-him choice.
3. The player's task is protecting the correction, not deciding his fate.

**Player-facing dialogue intent:** Oth'varen tells Sera and the player what he intends

**Completion / dependencies:** `HUMAN_Q179_DONE`; prerequisites `HUMAN_Q178_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q180 — The Seven Hold

**Track:** Main story · **Level:** 80+ · **Location:** Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven · **Quest giver / lead:** Guardian teams

**Player objective:** Coordinate distributed support of six fields with the Seven; solo-accessible objectives, no combined player ritual.

**Stages (narrative-critical):**
1. The Seven freely contribute with identities intact.
2. Player defends stations and specialists, no combined player-mage ritual.
3. Independent gods remain independent.

**Player-facing dialogue intent:** The Seven freely contribute with identities intact

**Completion / dependencies:** `HUMAN_Q180_DONE`; prerequisites `HUMAN_Q179_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q181 — The Shattering of Nhal

**Track:** Main story · **Level:** 80+ · **Location:** Final convergence chamber · **Quest giver / lead:** Sera / Oth'varen

**Player objective:** Defend correction, complete inversion and confirm Nhal's unified identity shattered.

**Stages (narrative-critical):**
1. Oth'varen physically dies after seeing alignment hold.
2. The player completes the defensive encounter; synchronized inversion shatters Nhal's unified identity.
3. Divine essence survives fragmented; expansion fate intentionally unresolved.

**Player-facing dialogue intent:** Oth'varen physically dies after seeing alignment hold

**Completion / dependencies:** `HUMAN_Q181_DONE`; prerequisites `HUMAN_Q180_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q182 — Silence After

**Track:** Main story · **Level:** 80+ · **Location:** Guardian sanctuary · **Quest giver / lead:** Sera

**Player objective:** Secure survivors and verify Seven's survival.

**Stages (narrative-critical):**
1. The Seven survive; Oth'varen does not return.
2. Memorial and verified reports, not a resurrection twist.
3. Avoid speculative fate of Nhal's essence.

**Player-facing dialogue intent:** The Seven survive; Oth'varen does not return

**Completion / dependencies:** `HUMAN_Q182_DONE`; prerequisites `HUMAN_Q181_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q183 — A World Unbound

**Track:** Main story · **Level:** 80+ · **Location:** Oth'irun, Tidehold, Valecrown · **Quest giver / lead:** Mara / Corven

**Player objective:** Deliver restricted final reports.

**Stages (narrative-critical):**
1. Shroud began fading at 70; it is not a new level-80 consequence.
2. Covenant and Dominion remain at war despite cosmic victory.
3. Kel'vari sovereignty continues.

**Player-facing dialogue intent:** Shroud began fading at 70; it is not a new level-80 consequence

**Completion / dependencies:** `HUMAN_Q183_DONE`; prerequisites `HUMAN_Q182_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

### Q184 — The Road Home

**Track:** Main story · **Level:** 80+ · **Location:** Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven · **Quest giver / lead:** Elara

**Player objective:** Post-finale epilogue: revisit Millhaven, close local relationships and record the cost and consequence of victory.

**Stages (narrative-critical):**
1. Return to Millhaven and the original cast.
2. Elara recognizes service and loss rather than declaring the player ruler of the world.
3. Close Human emotional arc.

**Player-facing dialogue intent:** Return to Millhaven and the original cast

**Completion / dependencies:** `HUMAN_Q184_DONE`; prerequisites `HUMAN_Q183_DONE`; minimum level 80. **Failure / QA:** Each stage is retryable after logout or defeat; no duplicate rewards; solo accessibility for story objectives; Use the exact outcome order in the canonical campaign.

## Build acceptance checklist

- Every quest has an NPC interaction, a visible player objective, stage-by-stage credit, a safe failure/retry route, and a verified turn-in.
- Travel cannot be bypassed by invented waypoints; hostile Dominion zones remain hostile and PvE story objectives cannot be blocked by opposing players.
- Dialogue must respect knowledge timing and previously established choices. A player's sample-preservation choice changes follow-up reactions only, not main outcome.
- Combat encounters require actual spawn and leash data, aggro/threat behavior, target facing checks, checkpoint tests and boss scripting before implementation.
- Replace generic stages with precise actor, object, counter and trigger IDs after approved maps and encounter blueprints are available.
- Review the chapter in-game for pacing, XP, difficulty, travel time, accessibility, and solo completion.

**Release blockers:** exact world geometry, quest NPC/creature/item IDs, complete dialogue scripts, cinematic assets, XP and loot tuning, encounter scripting, server quest integration and playtest results. None are asserted complete here.
