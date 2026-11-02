# Human Q080–Q103 — Rust Quest Authoring Review

**Status: Needs Review — proposed content corrections, not executable Rust quest definitions.** [Canonical Human Campaign](Human_1_80_Canonical_Campaign.md) governs narrative. [Current Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) remains the sole working quest inventory. [Codex Quest Engine Agreement](../../../QUEST_ENGINE.md) governs current capabilities.

**24 quests reviewed. No quest removed, renumbered, assigned a runtime ID, or changed in the manifest.** The Rust catalogue currently supports ordered stages, `all/any/at_least` completion, kill and semantic NPC-interaction objectives. Other kinds, cross-pack registry, content migration, party sharing, phasing and Luau remain planned. The qualified multi-quest journal/Store is not yet the normal live quest path.

## Quest-specific authoring matrix

| Quest | Objective/evidence design | Runtime and continuity requirements |
|---|---|---|
| **Q080 Aftermath of Inversion** | Present destroyed-gate records, verify Shroud destination, request Sera's explanation | Three separate semantic actions/evidence facts. The player does not yet know the surviving Guardian gate |
| **Q081 The Gate They Kept** | Confront Sera using authenticated proof; record her admission of a *separate* surviving restricted Passage | Persistent disclosure fact, not a passive cutscene-only flag; Sera acknowledges concealment without automatic absolution |
| **Q082 The Price of Secrecy** | Inspect historic crossing and custodial-consent logs; discuss the decision | **Reveal leak:** `kelvari_consent` target alias may expose Kel'vari before Q086. Player-visible text should say 'custodial consent' without identifying the people |
| **Q083 The Route That Never Closed (optional)** | Survey gate throughput and return safety under existing permission | **Availability defect:** no prerequisites despite secret-gate location. Require Q081/Q082 disclosure and permitted access, or rework as post-arrival optional research. Do not gate Q084–Q086 |
| **Q084 Permission to Enter** | Submit Guardian petition, obtain independent consent from custodian beyond gate, record bounded terms | **Reveal leak:** player-visible `Kel'vari custodian` label before Q086. Use 'Custodian' until reveal. Permission must be issued by the people beyond the gate, not solely Guardians |
| **Q085 The Hidden Crossing** | Verify approved consent, traverse low-power Passage, register arrival | **First-contact sequencing:** reaching Lantern Redoubt must not display Kel'vari identity before mandatory Q086; arrival NPC nameplate should remain 'Outpost Custodian' |
| **Q086 Lantern Redoubt** | First living Kel'vari scene and Aru'ven introduction; review guest rules | Mandatory first reveal. Do not repeat Q085 crossing. Persist `living_kelvari_revealed` as an authoritative personal-story fact |
| **Q087 Those Who Remained (optional)** | Rescue Kel'vari work party at authorized site | **Availability defect:** empty prerequisites. Gate behind Q086 first-contact and Q088 access consent (or independent equivalent), not hard level. Must not become mandatory for Q089 |
| **Q088 Permission to Remain** | Hear Aru'ven's terms, register boundaries, receive supervised research access | Consent is an independent Kel'vari decision. Permission must persist; no Covenant ownership or military access |
| **Q089 The Broken Path** | Reach ruined Passage, document Oma'vira intrusion and damage, report without reopening | Ruin exploration and two independent evidence observations planned; no second gate destruction |
| **Q090 A Warning Preserved (optional)** | Stabilize damaged warning, copy fragment, deposit record | **Availability defect:** no prerequisites. Require authorized archive access; the main Q091 warning lead must not depend on this optional copy |
| **Q091 Questions for the Sixth** | Review independently authenticated Sixth Scholar reference, petition for audience, secure entry authorization | Avoid implying the player already knows the sixth's identity. Archive permission requires custodial approval |
| **Q092 The Custodian's Conditions** | Accept archive terms, help endangered copyist, confirm supervised research | Copyist rescue must be mandatory only if narratively essential; do not duplicate Q093's optional store protection |
| **Q093 An Audience Earned (optional)** | Protect archive store and rehouse records | **Availability defect:** empty prerequisites. Require archive-site access; Q094 cannot depend on Q093 |
| **Q094 Six Names in the Record** | Compare original scholar list and verifier mark in either order, then authenticate | Two independent historical facts; do not reveal the Grey Surveyor's identity here |
| **Q095 The Scholar Who Remained** | Present measurements to Surveyor, inspect unusual annotations, seek independent verification | NPC label remains Grey Surveyor. Precision is evidence, not yet identity proof |
| **Q096 The Verifier's Mark** | Compare ancient and modern marks in either order, then authenticate through Leth'ari | Verification is independent of Surveyor's claims; no premature nameplate change |
| **Q097 The Last Witness** | Present verified proof, confront Grey Surveyor, record identity disclosure | **Reveal gate:** first player-visible Oth'varen name occurs in shared reveal. Persist identity flag; change nameplate/dialogue only afterward |
| **Q098 Echoes Before the Sundering** | Review original project and deliberate alterations; obtain Oth'varen's limited account | His knowledge is firsthand but not omniscient. No full Keeper/Binding truth |
| **Q099 The Six Reunited** | Activate authorized historical projection, observe six scholars' disagreement, preserve source copy | Scene and record provenance separate. Projection is evidence, not necessarily exhaustive history |
| **Q100 The Cost of Keeping Quiet (optional)** | Receive protection request, reach archive frontier, defend custodians | **Availability defect:** empty prerequisites. Gate by Q086/Q088 access or equivalent; don't reveal hidden land beforehand |
| **Q101 Two Histories (optional)** | Read two conflicting historical accounts in any order, register differences | **Availability defect:** empty prerequisites. Require archive access; neither source automatically resolves the ancient dispute |
| **Q102 The Delegation's Shadow** | Compare conflicting dates and dispatches, obtain Oth'varen's referral | Cross-check whether Mara or Oth'varen is giver; preserve independent evidence before Su'vareth attribution |
| **Q103 The Scholar's Request** | Receive bounded investigation proposal, obtain permissions, arrange Mara handoff | Oth'varen's request is not a claim of authority over Dominion records; no friendly Dominion access assumed |

## High-priority corrections to approve

1. **Secret gate access:** Q083 cannot be available before the player knows the gate exists. Q087, Q090, Q093, Q100 and Q101 also require explicit region/permission conditions; empty prerequisite lists do not grant teleportation or NPC access.
2. **Spoiler-safe NPC display:** Q082, Q084, Q085 and Q097 must separate stable internal actor identity from reveal-gated public labels. First Kel'vari identity at Q086; Oth'varen at Q097.
3. **Mandatory versus optional:** Q084–Q086 and Q088 contain required consent/reveal/access. Optional Q083, Q087, Q090, Q093, Q100 and Q101 cannot block Q089, Q091, Q094 or Q102.
4. **No duplicate reveal:** Q085 is crossing and registration; Q086 is first living Kel'vari introduction. Q094–Q096 build proof; Q097 is the first explicit identity reveal.
5. **Stage grouping:** Unordered independent inspections can be grouped in `all` stages, but later interpretation or report interactions belong in a subsequent stage to prevent pre-credit.
6. **Historical progress:** gate consent, disclosure, evidence copies, access permissions and identity reveal are durable facts; do not depend on carrying a losable item. If physical tokens are introduced, inventory possession and consumption require separate decisions.
7. **Recovery:** failed escort or archive defense resets only the active encounter. Previously verified evidence and permissions persist.
8. **Runtime boundaries:** NPC semantic interactions may be native; scene, region, object, defense and phasing facts require future engine support. Do not counterfeit them as generic dialogue events.
9. **No inferred level gates:** the 35–45 narrative band does not itself impose hard eligibility requirements.

## Proposed review decisions

- **Keep all 24 quests provisionally.** They represent different permissions, evidence sources, rescues and revelations; the optional access conditions need correction before export.
- **Rewrite Q083, Q087, Q090, Q093, Q100 and Q101 availability** using explicit access prerequisites, not hard level restrictions.
- **Clarify Q092 versus Q093** so mandatory copyist rescue and optional archive-store defense are distinct.
- **Correct player-visible labels** in Q082, Q084, Q085 and Q097 without changing the underlying stable actor IDs.
- **Preserve Q086 and Q097 as mandatory reveal gates**, with per-character, idempotent story flags.

## Next review

Proceed to Q104–Q128: Shattered Delegation evidence, Su'vareth's death, Veilbreaker discovery and the first distributed Anchor operations. Apply approved corrections to the **single manifest**, not to a parallel quest pack. No engine modifications are proposed.
