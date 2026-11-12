# Human Q158–Q184 — Rust Quest Authoring Review and Finale Contract

**Status: WORKING — narrative/authoring recommendations, not executable definitions.** [Canonical Human Campaign](Human_1_80_Canonical_Campaign.md) is the narrative authority; [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the sole structured working inventory; [Codex Quest Engine](../../../QUEST_ENGINE.md) defines current runtime capabilities.

**27 quests reviewed; 184 total campaign entries remain intact.** Current Rust catalogue supports kill and semantic NPC interaction with ordered stages and `all/any/at_least` completion. Its multi-quest journal and PostgreSQL Store are independently qualified but not yet in the normal live World/client path. Exploration, inspection, escort/defense, scene, choice, world phasing, item/currency rewards, party sharing, Luau and cross-pack registry/content migrations remain planned. Do not infer hard level gates from narrative bands. Q174 explicitly requires reaching the level-80 cap in the canonical campaign; reconciled authoring metadata places that requirement at finale authorization. Runtime realization remains OPEN, not permission to bypass that narrative requirement.

## Q158–Q174: manifestations and inversion preparations

| Quest | Authoring intent / authoritative evidence | Key blockers and continuity |
|---|---|---|
| **Q158 Essence Without Form** | Sera authorizes survey → reach safe frontier → preserve independent non-Unmade readings → report | Manifestation is gradual after Q156; evidence must distinguish it from ordinary Unmade activity |
| **Q159 The Lands Revealed (optional)** | Aru'ven requests aid → reach exposed Kel'vari settlement → protect evacuees → report shelter | **LOCKED creator decision (2026-11-04):** available after Q156; earlier legitimate Kel'vari contact required; Aru'ven's evacuation request grants limited affected-settlement access. No Q157 or Q181 prerequisite and no additional level gate. Approved JSON authoring eligibility reconciled; runtime implementation pending |
| **Q160 The Architect's Hand** | Infiltrate facility → authenticate Thes'var's orders → report deliberate stabilization plan | Thes'var knowingly serves Nhal; avoid implying he is merely mistaken |
| **Q161 Fractured Manifestation** | Observe inverted Major Anchor effect → preserve residual essence → Oth'varen interprets | Fracture of coherent pattern, not annihilation of divine essence. Scene and measurement are distinct facts |
| **Q162 What Cannot Be Destroyed** | Compare ancient testimony and residual traces in any order → record identity/essence distinction and uncertainty | Possible fragment directions and memory loss are hypotheses; do not determine future expansion outcomes |
| **Q163 The Architect Speaks** | Authenticate transmission → hear Thes'var's rationale → verify coordinates → plan approach | He knowingly serves Nhal under corruption; invitation is evidence, not trust |
| **Q164 Convergence Principle** | Measure two independent Anchors in any order → run bounded synchronized test → record limits | Hypothesis that coordinated inversion could shatter Nhal; not yet guaranteed |
| **Q165 The Guardian's Decision** | Present findings to council → review risks → obtain conditional authorization | Authorization is for preparation, not irreversible activation |
| **Q166 The Architect's Invitation** | Nessa briefing → scout approach → map defenses and retreat in either order | Distinguish from Q163's transmission; no blind acceptance of invitation |
| **Q167 Stone and Strain** | Brynja task → inspect supports → reinforce → confirm safe load | Supports have tested limits; no blanket guarantee of safety |
| **Q168 Life at the Edges (optional)** | Caelith ecological survey → reach affected area → measure disturbance → apply mitigation → report | **LOCKED creator decision (2026-11-04):** available after Q156 in post-Veilbreaker environmental phase; Caelith assigns survey; use established regional access and obtain local permission where restricted. No Q157, Q181 or additional level gate. Approved JSON authoring eligibility reconciled; runtime implementation pending |
| **Q169 What the Shroud Concealed (optional)** | Tarek's exposed-site report → investigate ruins → recover Thes'var's pre-Sundering personal journal and independent authentication → consult Oth'varen → preserve journal with custodians | **LOCKED story direction (2026-11-04):** post-Q156, no Q157/Q181 or extra level gate; local permission where restricted. No pre-Q156 Shroud exposure, early First Anchor mechanism disclosure, absolution of Thes'var, or Q176–Q181 foreknowledge. Site/dialogue WORKING; JSON authoring stages reconciled; creator-approved death-event cutoff recorded in JSON authoring metadata; runtime implementation pending |
| **Q170 Instructions That Survive Us** | Oth'varen trains independent crews and deposits redundant procedures | No prophecy or expectation of death; he does not learn fatal correction until Q177–Q178 |
| **Q171 The Consent of Neighbors** | Request Covenant assistance and Kel'vari consent separately → record terms | Sovereignty remains distinct; no single faction can authorize the whole operation |
| **Q172 A Failure We Can See** | Survey vulnerable junction → model risk → verify emergency access → report | **Foreshadowing only:** must not reveal Thes'var's future sabotage or the fatal correction |
| **Q173 The Last Measure** | Check independent cutoffs → bounded alignment test → record results → readiness authorization | No full Nhal shattering before finale; safety trial is not final activation |
| **Q174 Before the Final Crossing** | Confirm team readiness, consent, evacuation and supplies → authorize final operation | **CANONICAL:** Q174 explicitly requires reaching the level-80 cap. Approved authoring metadata records level 80 at final authorization, not an inferred Q174 acceptance gate. **OPEN:** future runtime support; no executable content is claimed |

## Q175–Q184: finale and homecoming

| Quest | Authoring intent / authoritative evidence | Key blockers and continuity |
|---|---|---|
| **Q175 Inside the Architect's Design** | Enter facility → identify controls → defeat defenders → preserve convergence mechanism | Preserve apparatus for inversion; generic kill credit alone does not prove machinery preserved |
| **Q176 The Architect's Last Argument** | Confront and kill Thes'var → detect unexpected control irregularity afterward → turn in to Sera | Thes'var dies physically. Sabotage discovery must occur **after** defeat, not during pre-fight dialogue |
| **Q177 The Broken Alignment** | Inspect tampered junction → attempt safe correction → prove failure → alert Sera | Explicit failed-safe-correction event; no automatic catastrophic loss on player error |
| **Q178 The Scholar's Correction** | Verify fatal correction model → Oth'varen chooses → Sera establishes safeguards | He discovers the lethal cost only now, following sabotage; choice is voluntary |
| **Q179 The Sixth Scholar's Choice** | Farewell dialogue → ready stations → receive final instructions | Player tone can vary; no false branch where Oth'varen survives |
| **Q180 The Seven Hold** | Defend mortal stations → confirm Seven's voluntary participation → maintain six-field support | Seven cooperate freely and remain unbound. Defense and field alignment require trusted outcome facts |
| **Q181 The Shattering of Nhal** | Protect correction → verify alignment holds → Oth'varen physically dies → final inversion → Nhal's coherent identity shatters → turn in to Sera | **Resolved in the working manifest:** `q181.objective.othvaren_death` records the physical death between alignment hold and inversion. Rust support remains planned; do not add a duplicate stage |
| **Q182 Silence After** | Confirm surviving Seven and teams → record memorial → document fragment uncertainty | Essence survives; no determination of fragment destination or new identity |
| **Q183 A World Unbound** | Report to Kel'vari and Covenant separately → verify Shroud fade began at Q156 → war remains | No instant peace treaty, no renewed Binding, no delayed start of Shroud dissipation |
| **Q184 The Road Home** | Return to own racial homeland by ordinary travel → revisit race-specific familiar NPCs (Elara and companions for humans) → close personal arc | Final contact and closure should be idempotent; optional regional work may remain available |

## Canonical finale transition requirements

1. **Q156** is the only Veilbreaker world transition: Veil destroyed, Binding dissolves immediately, Seven freed, Shroud begins dissipating. Q157 onward observes consequences.
2. **Q161–Q164** establish a credible but uncertain mechanism for fracturing Nhal's coherent identity without annihilating divine essence.
3. **Q176** ends with Thes'var physically dead; **Q177** proves deliberate sabotage and ordinary correction failure.
4. **Q178–Q179** Oth'varen learns of fatal exposure only after sabotage, chooses the correction, tells Sera/player, and prepares the teams.
5. **Q180–Q181** the Seven participate freely; Oth'varen completes correction, **sees alignment hold, then physically dies**. The final inversion shatters Nhal's unified memory and will, not his divine essence.
6. **Q182–Q184** preserve survivor reports, uncertain divine fragments, continued faction war and race-specific homeland homecoming (Millhaven for humans).

These are **server-authoritative, per-character story facts** requiring atomic or recoverable transitions; a cinematic played on the client is not sufficient evidence. No duplicate world catastrophe or resurrection can occur on reconnect/retry.

## Remaining authoring decisions for the single manifest

- **Q159 LOCKED:** post-Q156, earlier Kel'vari contact and Aru'ven's limited evacuation permission; no Q157 or extra level gate. **Q168 LOCKED:** post-Q156, Caelith's assignment, established regional access and restricted-site permission as needed, without Q157/Q181 or extra level gate. **Q169 LOCKED story direction:** journal recovery after Q156 and Oth'varen interpretation; specific site and dialogue WORKING. JSON authoring objectives and approved access metadata are reconciled. The manifest remains non-executable; Q169's death-event cutoff is creator-approved; post-conversation unfinished custody handling and runtime realization remain OPEN.
- **CANONICAL / OPEN implementation:** Q174's level-80 cap is now recorded as approved final-authorization metadata; quest-acceptance eligibility is not inferred. Future runtime realization remains OPEN; no other narrative band becomes a hard gate.
- Q181's explicit death transition is already present in the working manifest. **OPEN:** implement and validate its future authoritative story-fact capability separately; no gameplay-code change is authorized by this documentation cleanup.
- Keep Q161–Q162 essence interpretation carefully uncertain about fragments.
- Separate Thes'var's defeat, discovered sabotage, correction failure and Oth'varen's voluntary choice into non-skippable authoritative facts.
- Define Q184's race-specific final giver/turn-in and one-time shared closure, while allowing independent optional quests to remain playable.

## LOCKED: completion contacts after permanent deaths

**Creator decision, 3 November 2026:** turn Q176 and Q181 in to Sera after their final objectives, as recorded in the [canonical campaign](Human_1_80_Canonical_Campaign.md) and [decision ledger](../../DECISIONS.md#2026-11-03--locked-q176-and-q181-turn-in-contact). The earlier choice between Sera and automatic completion is resolved. Thes'var's physical death in Q176 and Oth'varen's death after alignment holds but before final inversion in Q181 remain unchanged.

**JSON authoring reconciliation complete; runtime binding OPEN:** Q176 and Q181 now name Sera alone as turn-in contact after all objectives. The earlier Thes'var and Sera / Oth'varen labels are superseded, not permission for dead-NPC turn-in. Stable actor IDs remain null. Existing [finale dialogue](Human_80_Finale_NPC_Dialogue_Scripts.md) assigns both aftermath speeches to Sera; wording remains WORKING.

## Engine-independent authoring constraints

- Original H/HS IDs remain provenance candidates; no automatic stable runtime IDs based on title similarity.
- Standard objectives should not require Luau. Complex story cinematics may later use scripting for presentation, but Rust owns evidence, completion, rewards and world state.
- Historical evidence survives inventory loss. Physical items require separate current-possession and consumption rules.
- Character scope is the currently supported credit policy; party/instance sharing and phasing remain planned.
- Rewards, NPC/creature IDs, item semantics, retry and abandonment policies are unresolved and must be validated before executable export.
- Future quest packs must resolve prerequisites through a registry and support explicit saved-definition compatibility/migration. No permanent 32-quest split is implied.

## Consolidated legacy context — 3 November 2026

**ARCHIVED source evidence / WORKING authoring detail.** The [retired final migration note](https://github.com/code-studio-au/theia-mmo/blob/a7ca2aa17e4c2f1fcb40b7392fe68164af579b1a/docs/story/Theia/Campaigns/Human_Q158_Q184_Legacy_Pack_Migration.md) compared `Human_70_80_Playable_Quest_Pack.md` v0.1 with `Human_70_80_Implementation_Brief.md` v0.2 (17 Q158–Q174 references each), and `Human_80_Finale_Playable_Quest_Pack.md` v0.1 with `Human_80_Finale_Implementation_Brief.md` v0.2 (10 Q175–Q184 references each). No extra quests were introduced. Their distinctive purposes and finale constraints are retained above; location bands follow the [canonical campaign](Human_1_80_Canonical_Campaign.md), while distinct unapproved site/design details remain in the [retained historical site proposals](Human_Legacy_Sidequest_Zone_Character_Archive.md#retired-production-location-and-lore-proposals).

- Preserve Classic explicit targeting, manual facing and weapon-timed autoattack, solo-accessible critical story, safe checkpoints and no mandatory PvP, group, class or profession requirement.
- Travel remains physical roads, ships and fixed flight paths. Restricted Passage requires separate consent, and the Millhaven homecoming does not require optional regional cleanup.
- Encounter failure resets active combat, not irreversible story facts, durable choices or verified evidence. Boss death, rescue, correction, alignment and final inversion need distinct trusted events; watching a cinematic alone is not authoritative credit.
- Historical level-70/75/80 chapter minima, draft completion aliases and repeated accept/inspect/report scaffolding are not approved eligibility, runtime IDs or extra objectives. The old source treated Q174's level-80 rule as a proposal; the canonical campaign now states the cap requirement explicitly. Authoring metadata is reconciled; runtime realization remains OPEN as described above.
- The older Q181 description omitted the explicit physical-death stage now recorded in the working manifest. Preserve the canonical order above and do not add a duplicate death stage.
- Spawn geometry, encounter timing, localization and actual engine integration remain unresolved alongside the actor, reward, item, phasing and saved-definition inputs above. Historical QA plans do not establish implementation.

## Next milestone

The **entire 184-quest Human campaign now has a first-pass Rust authoring review**. Consolidate reviewed, approved corrections into the sole working manifest, resolve identity/provenance and runtime capability gaps, and remove redundant review/draft documents only after their unique decisions are captured. This document is not a second quest inventory.
