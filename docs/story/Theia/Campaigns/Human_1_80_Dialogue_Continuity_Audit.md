# Human 1–80 — Dialogue Continuity Audit and Revision Plan

**Status: Needs Review — editorial audit.** Story authority: [Canonical Human Campaign](Human_1_80_Canonical_Campaign.md). All nine NPC dialogue packs are first-pass drafts, not production-final or voice-ready.

## Core revelation matrix

| Quest interval | Player may know | Player must not yet know | Mandatory reaction |
|---|---|---|---|
| Q001–Q034 | Caravan theft, administrative corruption, Cassian's betrayal | Kel'vari, Nhal, Binding, Conclave | Elara and Mara treat evidence seriously; Cassian credible before exposure |
| Q035–Q062 | Resonators, suspicious alterations, Guardians' technical role | Guardians' surviving Passage, Oth'varen identity, Nhal | Sera is careful; allied authorities remain independent |
| Q063–Q079 | First inversion, Oma'vira, destroyed Shroud Passage | Living Kel'vari, hidden Guardian gate, Sixth Scholar identity | Grey Surveyor remains unidentified |
| Q080–Q091 | Guardians' secret restricted gate, living Kel'vari at Q086 | Oth'varen's true identity before Q097 | Sera accepts criticism; Kel'vari consent required |
| Q092–Q103 | Six scholars; Oth'varen identity at Q097; ancient project sabotage | Full Keeper/Binding history; Nhal's name | Oth'varen acknowledges limited knowledge |
| Q104–Q118 | Su'vareth sabotaged peace; Veilbreaker recovered in Q118 | Veilbreaker's complete mechanism, Nhal name | No friendly Dominion authority; Oth'varen learns modern operation from recovered orders |
| Q119–Q139 | Distributed Anchor manipulation; First Voice research | Nhal's name before Q143, Keeper's full deception | Local Warden victories are real but incomplete |
| Q140–Q146 | Vaer'uun's selective truth; Nhal name at Q143 | Keeper's complete testimony before Q149–Q151 | Vaer'uun manipulates; Oth'varen admits omissions |
| Q147–Q155 | Keeper deceived Seven; Guardian learned afterward; Veil/Binding distinction | Full fate of Nhal's fragments | Countermeasure only a hypothesis |
| Q156–Q157 | Veil destroyed; Binding dissolves immediately; Seven free; Shroud starts fading | Instant stable Nhal manifestation | Victory over Khor'vun cannot reverse activation |
| Q158–Q174 | Manifestations; divine essence persists; inversion can fracture coherent identity; bounded convergence hypothesis | Oth'varen's fatal sacrifice before Q177–Q178 | Oth'varen expects to survive |
| Q175–Q184 | Thes'var dies; sabotage; fatal correction; Nhal shattered at Q181 | Future fate of divine fragments | Seven survive, Oth'varen physically dies, war remains |

## First-pass audit findings

1. **Early foreshadowing:** Q003 and Q008 mention anomalous healing, but must not attribute it to the same six-field mechanism the Guardians have not yet characterized. Keep observation separate from theory.
2. **Grey Surveyor attribution:** Q071–Q072 dialogue speaker labels can reveal Oth'varen's name to a reader or player before Q097. In-game speaker labels must say **Grey Surveyor**, even if internal metadata uses a stable character ID.
3. **Kel'vari revelation:** Q084 custodial dialogue must not reveal the name Kel'vari before Q086 to the player. Use “custodian beyond the gate” and withhold speaker race until Q086.
4. **Guardian disclosure:** Q081 acknowledges a gate; Q082 provides historical context. Do not present Sera's concealment as a simple misunderstanding or absolve her automatically.
5. **Shattered Delegation:** Q107 must preserve the High Ruler's brother's anti-peace position and reluctant attendance as an official delegate; Q108 must be hostile infiltration only.
6. **Oth'varen's foreknowledge:** Q118 must not imply he knew the modern Veilbreaker before translating Su'vareth's records. Q170 should express the value of sharing knowledge, not a foreseen death.
7. **Anchors versus Binding:** Q125 and Q151 must explicitly separate them. Veilbreaker destroys the Veil; the Binding dissolves immediately because it has no object left to contain.
8. **Vaer'uun's manipulation:** His invitation at Q140 is persuasive because the historical evidence is authentic. He knowingly serves Nhal, even though corruption distorts his justifications.
9. **Nhal's name (verified resolved in current dialogue):** Q138 Oth'varen's answer now avoids the literal name and says the researchers lacked knowledge to name the disturbance. Q143 Sel'ora's authenticated source comparison introduces Nhal by name. Preserve this reveal gate in subsequent edits; earlier reports of a live Q138 spoiler were historical.
10. **The Seven:** Q180 should not imply the Seven were re-bound or compelled. Their participation must be informed and voluntary.
11. **Oth'varen:** The fatal cost is learned only after Thes'var's sabotage in Q177–Q178. Physical death occurs only after he sees alignment hold in Q181.
12. **Fragments:** Q162 may describe possible mana direction and loss of memory, but Q182–Q184 must not establish where fragments go or what they become.

## Dialogue production requirements

- Replace one-size-fits-all “in progress” fallback lines with stage-conditioned text and player response branches.
- Assign actor/speaker IDs, voice tone and scene blocking; verify no premature spoiler in NPC labels, subtitles, journal summaries or tooltips.
- Include player choices with emotional response differences but stable canonical quest outcomes.
- Author combat phase barks and cutscene timing for Cassian, Oma'vira, Su'vareth, Vaer'uun, Khor'vun, Thes'var and Oth'varen.
- Use a content validator to flag forbidden names and concepts in quests before their reveal gates; manual editorial review remains essential.
- Do not assert voice-ready or engine-integrated status before full localization and scene QA.
