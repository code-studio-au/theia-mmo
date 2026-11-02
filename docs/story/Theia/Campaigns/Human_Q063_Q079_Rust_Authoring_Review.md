# Human Q063–Q079 — Rust Quest Authoring Review

**Status: Needs Review — content-authoring proposals, not executable quest data.** Narrative authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Single working inventory: [Human 1–80 Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json). Runtime implementation agreement: [Quest Engine](../../../QUEST_ENGINE.md). **17 quests reviewed; none removed or renumbered.**

## Implementation boundary

The current Rust catalogue accepts ordered stages with `all`, `any`, or `at_least` completion, but only **kill** and **semantic NPC interaction** objectives. Its pure multi-quest journal and PostgreSQL Store have been qualified separately; the expanded journal is not yet adopted into the normal World/client path. Exploration, inspection, object interaction, defense, scenes, branching, inventory and phasing remain planned. The 32-quest/64-KiB limits are development safeguards; eventual regional packs, cross-pack registry and content migrations are pending. No Luau scripts should be authored for ordinary quest logic.

## Quest-by-quest evidence and stage recommendations

| Quest | Recommended stage grouping and authoritative evidence | Blocking capability / review |
|---|---|---|
| **Q063 The City That Endured** | Urodan's waterway assignment → authorized travel and completed repair → archive referral | Stable NPC action IDs; trusted repair outcome, not merely arriving at waterway |
| **Q064 Water and Memory** | Tarek's tunnel permission → reach store and defend crew → preserve warning copy | Crew survival and warning provenance persist through retry; rescue and inspection planned |
| **Q065 The Missing Margin** | Authenticate original warning and missing margin in either order → Dromak grants restricted access | Two independent archive facts; don't infer identity of the person who removed text |
| **Q066 A Name for the Guardians** | Meet Sera → read Guardian charter → accept limited associate cooperation | Distinguish introduction, charter acknowledgement and association; no forced Guardian membership |
| **Q067 The Instruction Without Its Diagram** | Compare Regional Anchor plan and Halden's incomplete order in any order → Grey Surveyor identifies missing safeguard | Technical inspection planned; no premature identity reveal |
| **Q068 Reach Them Before the Charge** | Arrive at freight works → warn Halden → disable premature charge → report to Sera | Warning alone cannot complete the cutoff. Need server-authorized interrupt event; no timed softlock |
| **Q069 The Broken Measure** | Enter solo instance → rescue researchers → isolate Regional supports → shut down core | Defend/interaction/instance objectives planned; encounter checkpoint and committed rescue credit |
| **Q070 Copies for the People at Risk** | Collect complete instructions → distribute to at-risk crew → deposit independent measurement copy | Distribution requires a semantic handover, not greeting a team NPC |
| **Q071 Make the Measure Small** | Inspect isolation and reserve limits in either order → fit cutoff → receive Sera's bounded-trial authorization | **Spoiler bug:** manifest contact names Oth'varen before Q097; player-facing giver must remain Grey Surveyor. Brynja's participation needs verified location/contact |
| **Q072 A Brief Reversal** | Start bounded trial → measure reversal → activate cutoff at threshold → archive result | Trial success and cutoff require authoritative experiment outcomes; no claim that divine essence can yet be shattered |
| **Q073 The Safer Argument** | Hear Halden and Sera in any order → compare risk assessment → record revisable preference | Choice persistence planned. Both scientific positions credible; no permanent main-story fork |
| **Q074 The Pattern Beneath** | Independently compare disturbance datasets → synchronize timing overlay → report coordinated signal | Six signatures remain an observed network pattern, not proof of the full cosmic mechanism |
| **Q075 The Anchor's Defenses** | Scout Chainworks → document patrols and Harbinger orders → report | Scouting and evidence inspection planned; no automatic hostile NPC interaction credit |
| **Q076 The Harbinger's Design** | Authenticate Oma'vira's signature, Passage schematic and destination in flexible order → Sera verifies | Keep Passage destination clue distinct from proof that a separate Guardian gate survives |
| **Q077 Against the Harbinger** | Enter solo facility → disable conditioning relays → evacuate captives → kill Oma'vira → preserve gate records | Founder death permanent in personal canon; captives survive and gate evidence persists on boss retry. Boss kill may be native if exact trusted creature-life credit is appropriate, but other objectives planned |
| **Q078 The First Major Reversal** | Verify Major Anchor safeguards → defend specialists → record bounded inversion → assign maintenance | **Distinguish** Minor Resonator trial Q072 from captured Major Anchor inversion Q078. Defense and measurement outcomes planned |
| **Q079 The Harbinger's Secret** | Copy Passage destination → independently verify destroyed gate → request Sera's explanation | **Continuity check:** the manifest says gate destroyed here, while Q077–Q078 must establish when/how that destruction occurred. Q079 only verifies the state, not a second destruction |

## Cross-chapter continuity and high-priority corrections

1. **Q071–Q073 player-facing names:** The Grey Surveyor is not publicly revealed as Oth'varen until Q097. The manifest's `contact` metadata and Q072 contact currently expose his later name. Preserve stable internal actor identity but use a reveal-gated display name.
2. **Q068–Q069 retry:** Halden's premature charge is interrupted before the Broken Measure instance. Failure inside Q069 must not rewind that already committed action.
3. **Q071–Q072 trial semantics:** This is the first **bounded Minor Resonator inversion**. The result supports a reversal hypothesis, not a proven solution to Nhal.
4. **Q073 choice:** Persist provisional severance-versus-inversion preference for later dialogue, while preserving the mandatory experimental result and canonical storyline.
5. **Q077–Q079 gate chronology:** Record the destruction once, then make Q079 an evidence-verification quest. Do not imply Q078 is the event that necessarily destroys the gate unless the canonical scene establishes it.
6. **Q079–Q081 revelation:** The player knows Oma'vira used a Passage route to the Shroud, not that the Guardians maintain a separate functioning low-capacity gate. Sera's disclosure is later.
7. **No level eligibility inferred:** Q063–Q079's level band is narrative pacing only. Quest prerequisites remain explicit and are not automatically hard level gates.
8. **Giver/turn-in split:** Urodan/Tarek, Halden/Sera, Sera/Nessa and Brynja/Sera require distinct authorized contacts; `contact` strings are not executable actor IDs.

## Evidence and persistence contracts

- Archive documents, safety measurements, gate records and authenticated findings are **historical evidence** that survives dropping a physical object. If physical documents exist, possession/consumption must be separately specified.
- Semantic NPC interactions need stable pairs of NPC ID and action ID. Reporting evidence is not equivalent to opening the dialogue window.
- Encounter-defense objectives must record a trusted outcome (protected NPC survival, authorized completion, event receipt); ordinary kills do not substitute automatically.
- Character credit is the supported initial scope; party/instance credit is planned. Story dungeons remain solo-accessible.
- XP/reward amounts, NPC and target IDs, item semantics and failure/abandon policies remain unapproved. No invented runtime identifiers.
- A cross-pack registry and saved-journal content-version migrations are required before routine live quest updates.

## Next pass

Review Q080–Q103 (Sera's gate disclosure, Kel'vari consent and first contact, archive access, Grey Surveyor identity). Resolve the Q071/Q072 display-name leak before any player-facing export. Do not create a second quest inventory: approved changes belong in the single current manifest.
