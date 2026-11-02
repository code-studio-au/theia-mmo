# Human 1–80 — Detailed Quest Necessity Review, Pass 2

**Status: proposed dispositions, not approved changes.** This pass examines actual quest objectives and dependencies rather than titles alone. [Machine-readable review](Human_1_80_Quest_Necessity_Decisions_Draft.json). **184 entries remain intact.**

## hidden passage

**Recommendation:** keep distinct. Q029 records evacuation and independent evidence before the solo instance; Q030 owns combat, arrest and dungeon retry. A pre-instance checkpoint is a meaningful gameplay boundary.

**Risk / correction:** Do not repeat dispatch collection in Q030 if Q029 already secures the same copy; use different evidence targets.

- **Q029 — Before the Hidden Passage:** Secure civilian exits and establish an independent evidence-copy checkpoint before Cassian's capture. (main_story)
- **Q030 — The Hidden Passage:** Finish the solo story dungeon; stop Sorn, secure a dispatch copy and capture Cassian alive. Group play is optional. (main_story)

## guardian gate

**Recommendation:** keep with boundary rewrite. Survey optional, sovereign permission mandatory, crossing mandatory, first living Kel'vari reveal mandatory, ongoing access consent mandatory.

**Risk / correction:** Q083's return-path verification is optional but Q085 must independently validate safe return. Q086 repeats crossing language from Q085; rewrite Q086 as arrival and first contact, not another crossing.

- **Q083 — The Route That Never Closed:** Survey the separate low-capacity Passage, test a small-group return path and deliver an expedition proposal to Corven. (regional_optional)
- **Q084 — Permission to Enter:** Obtain permission for a small supervised delegation. (main_story)
- **Q085 — The Hidden Crossing:** Cross with approved expedition and verify return capacity. (main_story)
- **Q086 — Lantern Redoubt:** Cross in a small surveyed group and agree access/maintenance terms for this outer site; no continent-wide gate restoration. (main_story)
- **Q088 — Permission to Remain:** Agree boundaries, supplies and site access. (main_story)

## archive access

**Recommendation:** keep with optional separation. Request, custodial terms, optional archive service, then authenticated research are distinct; Q093 cannot gate Q094.

**Risk / correction:** Q092 already includes copyist rescue while Q093 includes archive service. Separate them by unique location and independent objectives.

- **Q091 — Questions for the Sixth:** Request supervised archive audience. (main_story)
- **Q092 — The Custodian's Conditions:** Meet the archive party at a protected approach, accept access limits and recover its missing copyist. (main_story)
- **Q093 — An Audience Earned:** Secure exposed records and access approval. (regional_optional)
- **Q094 — Six Names in the Record:** Compare the project list, verifier image and distinctive stability sequence; request the Surveyor's explanation. (main_story)

## delegation investigation

**Recommendation:** keep for now review pacing. Route reconstruction, planted evidence, living witness, payments, decoy discovery and independent founder attribution are different evidence classes.

**Risk / correction:** Avoid six mandatory trips to the same quest giver. Group steps into field sequences and validate travel time; combining later may be justified.

- **Q109 — Road to the Meeting:** Survey arrival routes and guard positions. (main_story)
- **Q110 — Evidence Planted Twice:** Recover planted clues and compare provenance. (main_story)
- **Q111 — The Witness Who Disappeared:** Find survivor and protect extraction. (main_story)
- **Q112 — The Delegation's Second Ledger:** Compare human and hostile-coast contract records without relying on friendly Dominion contacts. (main_story)
- **Q113 — The False Safe House:** Locate Su'vareth's headquarters using three independent trails and expose its decoy. (main_story)
- **Q114 — The Veiled Hand:** Present corroborated pattern identifying Su'vareth. (main_story)

## warden preparation

**Recommendation:** keep rename q128. Q128 stages early expedition logistics while Q154 stages final Veilbreaker assault after the Keeper revelation.

**Risk / correction:** Identical titles confuse players and content lookup. Proposed Q128 title: The Expedition Musters. Q154 remains Before the Veil Breaks.

- **Q128 — Before the Veil Breaks:** Verify extraction, supply and instrument teams. (main_story)
- **Q154 — Before the Veil Breaks:** Prepare evacuation and assault support. (main_story)

## finale preparation

**Recommendation:** keep distinct with tighter scopes. Q170 transfers knowledge, Q172 identifies structural risk, Q173 performs a bounded test, Q174 checks final operation readiness.

**Risk / correction:** No Oth'varen foreknowledge of death in Q170; Q172 cannot reveal Thes'var's later sabotage or fatal correction. Avoid repetitive generic readiness interactions.

- **Q170 — Instructions That Survive Us:** Train independent teams and deposit redundant safety records for operation without the ancient scholar. (main_story)
- **Q172 — A Failure We Can See:** Detect the weak junction and establish an emergency correction route before final activation. (main_story)
- **Q173 — The Last Measure:** Run bounded distributed alignment test. (main_story)
- **Q174 — Before the Final Crossing:** Verify teams and complete readiness. (main_story)

## Validation before applying edits

1. Compare every proposed rewrite with canonical narrative, stage JSON and dialogue scripts.
2. Where Q128 is renamed, update the canonical register, structured JSON, quest pack, dialogue references and index together; preserve Q128 identity.
3. Verify Q083 optionality cannot block Q085 safe return or Q086 Kel'vari reveal; Q093 cannot block Q094 archive investigation.
4. Check no main-story prerequisite points to an optional quest, no quest reference disappears and no unintended level gate is introduced.
5. Keep 184 as the count until a specific merge/retirement is approved and its narrative effect recorded.

**Next:** author precise, Rust-compatible quest definitions with original editorial IDs, objective evidence and capability requirements, then remove redundant drafts only after content has been migrated.
