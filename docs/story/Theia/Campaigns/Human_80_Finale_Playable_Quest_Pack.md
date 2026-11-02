# Human 80–Finale — The Shattering of Nhal: Playable Quest Pack v0.1

**Status: Needs Review / gameplay authoring.** Companion to [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md) and [Quest Dependencies](Human_1_80_Quest_Dependencies.json). This pack adds quest-specific gameplay stages, player-facing narrative intent, and retry criteria. Canonical outcomes are unchanged. Locations are named regions, **not approved coordinates**. Enemy stats, rewards, dialogue scripts and exact interactions remain production blockers.

## Common encounter contract

Classic-style manual target and facing, hold-to-steer mouse controls, range/facing checks and persistent autoattack; solo critical story; instance checkpoints for boss encounters; persistent evidence credit independent of inventory; no required PvP, class, profession, or group. Regional missions can be completed independently. Hostile Dominion infiltration never involves friendly Dominion authorities. All quest stages must be safely retryable after death or disconnect.

### Q175 — Inside the Architect's Design

**Track:** Main story · **Minimum level:** 80 · **Contact:** Nessa · **Location:** Architect central facility

**Player purpose:** Infiltrate and disable defenses while preserving alignment mechanism. **Lore payoff:** Final operation begins.

**Proposed staged objective; verify against encounter design.**

1. Receive the assignment from Nessa and establish the objective at Architect central facility.
2. Infiltrate and disable defenses while preserving alignment mechanism.
3. Return the verified result to Nessa and record completion.

**Completion:** Award `HUMAN_Q175_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

### Q176 — The Architect's Last Argument

**Track:** Main story · **Minimum level:** 80 · **Contact:** Thes'var · **Location:** Solo boss chamber

**Player purpose:** Defeat and kill Thes'var. **Lore payoff:** Last founder dead.

**Narrative-critical staged design; preserve exact event order.**

1. Confront Thes'var inside the central control facility.
2. Defeat and physically kill him in a solo-accessible boss encounter.
3. Discover signs of his final sabotage after his death.

**Completion:** Award `HUMAN_Q176_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

### Q177 — The Broken Alignment

**Track:** Main story · **Minimum level:** 80 · **Contact:** Sera · **Location:** Control junction

**Player purpose:** Assess Thes'var's sabotage and misalignment. **Lore payoff:** New lethal correction required.

**Narrative-critical staged design; preserve exact event order.**

1. Inspect the control junction for Thes'var's deliberate misalignment.
2. Attempt safe corrections and prove they cannot hold.
3. Notify Sera and Oth'varen of the emergency.

**Completion:** Award `HUMAN_Q177_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

### Q178 — The Scholar's Correction

**Track:** Main story · **Minimum level:** 80 · **Contact:** Oth'varen / Sera · **Location:** Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven

**Player purpose:** Maintain evacuation and instrument stations as Oth'varen stabilises the alignment and dies after seeing it hold. **Lore payoff:** Thes'var killed; sabotage misaligns convergence; Oth'varen realizes fatal correction only then, tells Sera and player, physically dies after alignment holds; Nhal's coherent identity shattered, divine essence persists for later expansion; Seven survive, war continues.

**Narrative-critical staged design; preserve exact event order.**

1. Reconstruct the fatal correction with Oth'varen after the sabotage is known.
2. Confirm he alone can perform it at the convergence junction.
3. Let him make an informed decision without preordained sacrifice.

**Completion:** Award `HUMAN_Q178_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

### Q179 — The Sixth Scholar's Choice

**Track:** Main story · **Minimum level:** 80 · **Contact:** Oth'varen · **Location:** Convergence entrance

**Player purpose:** Witness Oth'varen tell Sera and player his choice. **Lore payoff:** Death not foreknown.

**Narrative-critical staged design; preserve exact event order.**

1. Witness Oth'varen tell Sera and the player that he expects to die.
2. Allow a meaningful farewell without offering a false rescue branch.
3. Confirm independent evacuation and team responsibilities.

**Completion:** Award `HUMAN_Q179_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

### Q180 — The Seven Hold

**Track:** Main story · **Minimum level:** 80 · **Contact:** Guardian teams · **Location:** Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven

**Player purpose:** Coordinate distributed support of six fields with the Seven; solo-accessible objectives, no combined player ritual. **Lore payoff:** Thes'var killed; sabotage misaligns convergence; Oth'varen realizes fatal correction only then, tells Sera and player, physically dies after alignment holds; Nhal's coherent identity shattered, divine essence persists for later expansion; Seven survive, war continues.

**Narrative-critical staged design; preserve exact event order.**

1. Protect the distributed stations while the Seven cooperate knowingly.
2. Maintain six-field support without requiring player-character ritual casting.
3. Keep mortal crews and extraction routes intact.

**Completion:** Award `HUMAN_Q180_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

### Q181 — The Shattering of Nhal

**Track:** Main story · **Minimum level:** 80 · **Contact:** Sera / Oth'varen · **Location:** Final convergence chamber

**Player purpose:** Defend correction, complete inversion and confirm Nhal's unified identity shattered. **Lore payoff:** Oth'varen physically dies, divine essence remains.

**Narrative-critical staged design; preserve exact event order.**

1. Defend Oth'varen as he corrects the convergence alignment.
2. Witness his physical death only after the alignment holds.
3. Complete coordinated inversion and shatter Nhal's coherent identity, not divine essence.

**Completion:** Award `HUMAN_Q181_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

### Q182 — Silence After

**Track:** Main story · **Minimum level:** 80 · **Contact:** Sera · **Location:** Guardian sanctuary

**Player purpose:** Secure survivors and verify Seven's survival. **Lore payoff:** Mourning and victory.

**Narrative-critical staged design; preserve exact event order.**

1. Recover survivors and confirm the Seven remain alive with intact identities.
2. Record Oth'varen's final act and memorial.
3. Avoid revealing the fate of Nhal's fragments.

**Completion:** Award `HUMAN_Q182_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

### Q183 — A World Unbound

**Track:** Main story · **Minimum level:** 80 · **Contact:** Mara / Corven · **Location:** Oth'irun, Tidehold, Valecrown

**Player purpose:** Deliver restricted final reports. **Lore payoff:** War not resolved.

**Narrative-critical staged design; preserve exact event order.**

1. Deliver restricted final reports to Covenant and Kel'vari authorities.
2. Acknowledge that Shroud dissolution began at Veilbreaker.
3. Confirm Covenant and Dominion remain at war.

**Completion:** Award `HUMAN_Q183_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

### Q184 — The Road Home

**Track:** Main story · **Minimum level:** 80 · **Contact:** Elara · **Location:** Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven

**Player purpose:** Post-finale epilogue: revisit Millhaven, close local relationships and record the cost and consequence of victory. **Lore payoff:** Thes'var killed; sabotage misaligns convergence; Oth'varen realizes fatal correction only then, tells Sera and player, physically dies after alignment holds; Nhal's coherent identity shattered, divine essence persists for later expansion; Seven survive, war continues.

**Proposed staged objective; verify against encounter design.**

1. Receive the assignment from Elara and establish the objective at Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven.
2. Post-finale epilogue: revisit Millhaven, close local relationships and record the cost and consequence of victory.
3. Return the verified result to Elara and record completion.

**Completion:** Award `HUMAN_Q184_DONE` after the final verified stage; evidence copied to case journal where applicable. **Recovery:** respawn or reconnect at the last safe stage; reset hostile encounters, not completed evidence. **Access:** Previous main-story completion and chapter minimum level; no optional quest dependency.

## Release gates

- Assign exact map and spawn IDs, NPC dialogue, interaction volumes, enemy archetypes, combat stats, loot and XP values from the actual world and progression systems.
- Validate chapter entry requirements and regional alternatives so a player can level without completing every main-story step.
- Test stage completion order, evidence persistence, death/disconnect, optional group scaling, and travel-route availability.
- Confirm all named story-critical sequences with the canonical campaign before implementation.
