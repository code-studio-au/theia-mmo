# Human Q158–Q184 — Rust Quest Authoring Review and Finale Contract

**Status: WORKING — narrative/authoring recommendations, not executable definitions.** [Canonical Human Campaign](Human_1_80_Canonical_Campaign.md) is the narrative authority; [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the sole structured working inventory; [Codex Quest Engine](../../../QUEST_ENGINE.md) defines current runtime capabilities.

**27 quests reviewed; 184 total campaign entries remain intact.** Current Rust catalogue supports kill and semantic NPC interaction with ordered stages and `all/any/at_least` completion. Its multi-quest journal and PostgreSQL Store are independently qualified but not yet in the normal live World/client path. Exploration, inspection, escort/defense, scene, choice, world phasing, item/currency rewards, party sharing, Luau and cross-pack registry/content migrations remain planned. No hard level gate is inferred from narrative bands, except Q174's explicit proposed finale level-80 gate, which requires separate product approval and runtime support.

## Q158–Q174: manifestations and inversion preparations

| Quest | Authoring intent / authoritative evidence | Key blockers and continuity |
|---|---|---|
| **Q158 Essence Without Form** | Sera authorizes survey → reach safe frontier → preserve independent non-Unmade readings → report | Manifestation is gradual after Q156; evidence must distinguish it from ordinary Unmade activity |
| **Q159 The Lands Revealed (optional)** | Aru'ven requests aid → reach exposed Kel'vari settlement → protect evacuees → report shelter | **Availability defect:** no prerequisites. Require post-Veilbreaker world phase (Q156 or Q157) and Kel'vari contact access; cannot occur before Shroud fade |
| **Q160 The Architect's Hand** | Infiltrate facility → authenticate Thes'var's orders → report deliberate stabilization plan | Thes'var knowingly serves Nhal; avoid implying he is merely mistaken |
| **Q161 Fractured Manifestation** | Observe inverted Major Anchor effect → preserve residual essence → Oth'varen interprets | Fracture of coherent pattern, not annihilation of divine essence. Scene and measurement are distinct facts |
| **Q162 What Cannot Be Destroyed** | Compare ancient testimony and residual traces in any order → record identity/essence distinction and uncertainty | Possible fragment directions and memory loss are hypotheses; do not determine future expansion outcomes |
| **Q163 The Architect Speaks** | Authenticate transmission → hear Thes'var's rationale → verify coordinates → plan approach | He knowingly serves Nhal under corruption; invitation is evidence, not trust |
| **Q164 Convergence Principle** | Measure two independent Anchors in any order → run bounded synchronized test → record limits | Hypothesis that coordinated inversion could shatter Nhal; not yet guaranteed |
| **Q165 The Guardian's Decision** | Present findings to council → review risks → obtain conditional authorization | Authorization is for preparation, not irreversible activation |
| **Q166 The Architect's Invitation** | Nessa briefing → scout approach → map defenses and retreat in either order | Distinguish from Q163's transmission; no blind acceptance of invitation |
| **Q167 Stone and Strain** | Brynja task → inspect supports → reinforce → confirm safe load | Supports have tested limits; no blanket guarantee of safety |
| **Q168 Life at the Edges (optional)** | Caelith ecological survey → reach affected area → measure disturbance → apply mitigation → report | **Availability defect:** no prerequisites despite post-Shroud environmental change. Gate by Q156 world phase and regional access, not hard level |
| **Q169 The Route Between (optional)** | Tarek transport survey → verify ship and flight links → confirm Passage limits → report | **Availability defect:** no prerequisites although it references restricted Guardian Passage. Require Q081/Q084 knowledge and consent or equivalent permission; never imply fast-travel waypoints |
| **Q170 Instructions That Survive Us** | Oth'varen trains independent crews and deposits redundant procedures | No prophecy or expectation of death; he does not learn fatal correction until Q177–Q178 |
| **Q171 The Consent of Neighbors** | Request Covenant assistance and Kel'vari consent separately → record terms | Sovereignty remains distinct; no single faction can authorize the whole operation |
| **Q172 A Failure We Can See** | Survey vulnerable junction → model risk → verify emergency access → report | **Foreshadowing only:** must not reveal Thes'var's future sabotage or the fatal correction |
| **Q173 The Last Measure** | Check independent cutoffs → bounded alignment test → record results → readiness authorization | No full Nhal shattering before finale; safety trial is not final activation |
| **Q174 Before the Final Crossing** | Confirm team readiness, consent, evacuation and supplies → authorize final operation | **Level-80 gate:** explicitly present in objective; decide whether this is an intentional eligibility rule. Codex's current catalogue has no level-eligibility field; don't fake it through an objective |

## Q175–Q184: finale and homecoming

| Quest | Authoring intent / authoritative evidence | Key blockers and continuity |
|---|---|---|
| **Q175 Inside the Architect's Design** | Enter facility → identify controls → defeat defenders → preserve convergence mechanism | Preserve apparatus for inversion; generic kill credit alone does not prove machinery preserved |
| **Q176 The Architect's Last Argument** | Confront and kill Thes'var → detect unexpected control irregularity afterward | Thes'var dies physically. Sabotage discovery must occur **after** defeat, not during pre-fight dialogue |
| **Q177 The Broken Alignment** | Inspect tampered junction → attempt safe correction → prove failure → alert Sera | Explicit failed-safe-correction event; no automatic catastrophic loss on player error |
| **Q178 The Scholar's Correction** | Verify fatal correction model → Oth'varen chooses → Sera establishes safeguards | He discovers the lethal cost only now, following sabotage; choice is voluntary |
| **Q179 The Sixth Scholar's Choice** | Farewell dialogue → ready stations → receive final instructions | Player tone can vary; no false branch where Oth'varen survives |
| **Q180 The Seven Hold** | Defend mortal stations → confirm Seven's voluntary participation → maintain six-field support | Seven cooperate freely and remain unbound. Defense and field alignment require trusted outcome facts |
| **Q181 The Shattering of Nhal** | Protect correction → verify alignment holds → Oth'varen physically dies → final inversion → Nhal's coherent identity shatters | **Resolved in the working manifest:** `q181.objective.othvaren_death` records the physical death between alignment hold and inversion. Rust support remains planned; do not add a duplicate stage |
| **Q182 Silence After** | Confirm surviving Seven and teams → record memorial → document fragment uncertainty | Essence survives; no determination of fragment destination or new identity |
| **Q183 A World Unbound** | Report to Kel'vari and Covenant separately → verify Shroud fade began at Q156 → war remains | No instant peace treaty, no renewed Binding, no delayed start of Shroud dissipation |
| **Q184 The Road Home** | Return by ordinary travel → revisit Elara and companions → close personal arc | Final contact and closure should be idempotent; optional regional work may remain available |

## Canonical finale transition requirements

1. **Q156** is the only Veilbreaker world transition: Veil destroyed, Binding dissolves immediately, Seven freed, Shroud begins dissipating. Q157 onward observes consequences.
2. **Q161–Q164** establish a credible but uncertain mechanism for fracturing Nhal's coherent identity without annihilating divine essence.
3. **Q176** ends with Thes'var physically dead; **Q177** proves deliberate sabotage and ordinary correction failure.
4. **Q178–Q179** Oth'varen learns of fatal exposure only after sabotage, chooses the correction, tells Sera/player, and prepares the teams.
5. **Q180–Q181** the Seven participate freely; Oth'varen completes correction, **sees alignment hold, then physically dies**. The final inversion shatters Nhal's unified memory and will, not his divine essence.
6. **Q182–Q184** preserve survivor reports, uncertain divine fragments, continued faction war and Millhaven homecoming.

These are **server-authoritative, per-character story facts** requiring atomic or recoverable transitions; a cinematic played on the client is not sufficient evidence. No duplicate world catastrophe or resurrection can occur on reconnect/retry.

## Remaining authoring decisions for the single manifest

- **OPEN:** approve the access/phase conditions for optional Q159/Q168 and the knowledge/permission conditions for Q169. The manifest already preserves these as `authoring_access_review` proposals; they are not executable prerequisites.
- Clarify whether the Q174 level-80 finale gate is a locked eligibility decision; if approved, request a versioned engine capability from Codex rather than inventing a field.
- Q181's explicit death transition is already present in the working manifest. **OPEN:** implement and validate its future authoritative story-fact capability separately; no gameplay-code change is authorized by this documentation cleanup.
- Keep Q161–Q162 essence interpretation carefully uncertain about fragments.
- Separate Thes'var's defeat, discovered sabotage, correction failure and Oth'varen's voluntary choice into non-skippable authoritative facts.
- Define Q184's final giver/turn-in and one-time closure, while allowing independent optional quests to remain playable.

## OPEN: completion contacts after permanent deaths

The working manifest still lists **Thes'var** as Q176's turn-in contact and **Sera / Oth'varen** as Q181's turn-in contact, each with `requires_verification: true`. Both quests permanently kill the named character before completion. These labels must not become living-NPC turn-in requirements. [Finale dialogue](Human_80_Finale_NPC_Dialogue_Scripts.md) assigns the aftermath speech to Sera in both cases, but a speaker label alone does not approve the final gameplay turn-in mechanism. Resolve the intended completion contact or authoritative auto-completion semantics before runtime conversion. Keep the locked deaths and quest outcomes unchanged; no replacement NPC, resurrection or runtime ID is approved here.

## Engine-independent authoring constraints

- Original H/HS IDs remain provenance candidates; no automatic stable runtime IDs based on title similarity.
- Standard objectives should not require Luau. Complex story cinematics may later use scripting for presentation, but Rust owns evidence, completion, rewards and world state.
- Historical evidence survives inventory loss. Physical items require separate current-possession and consumption rules.
- Character scope is the currently supported credit policy; party/instance sharing and phasing remain planned.
- Rewards, NPC/creature IDs, item semantics, retry and abandonment policies are unresolved and must be validated before executable export.
- Future quest packs must resolve prerequisites through a registry and support explicit saved-definition compatibility/migration. No permanent 32-quest split is implied.

## Next milestone

The **entire 184-quest Human campaign now has a first-pass Rust authoring review**. Consolidate reviewed, approved corrections into the sole working manifest, resolve identity/provenance and runtime capability gaps, and remove redundant review/draft documents only after their unique decisions are captured. This document is not a second quest inventory.
