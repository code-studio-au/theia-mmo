# Human Q104–Q128 — Rust Quest Authoring Review

**Status: Needs Review — proposals, not executable definitions.** Narrative authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Single working quest inventory: [Human 1–80 Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json). Runtime contract: [Quest Engine](../../../QUEST_ENGINE.md).

**25 quests reviewed. No quests deleted, merged, renumbered or assigned runtime IDs.** Current Rust catalogue supports ordered stages, `all/any/at_least` completion and only `kill` and semantic `npc_interaction` objective kinds. Pure journal and PostgreSQL Store are qualified separately, but expanded journal adoption into the normal World/client flow remains pending. Object inspection, exploration, defense, encounter outcomes, world phasing, branching and Luau remain planned. The 32-quest limit is a development safeguard, not a production architecture decision.

## Quest-by-quest evidence and authoring decisions

| Quest | Proposed stage grouping and completion evidence | Blockers / safeguards |
|---|---|---|
| **Q104 The Delegation That Failed** | Mara authorizes case → copy surviving dispatch → Oth'varen authenticates provenance | Separate semantic actions for case intake and provenance; never infer guilt from a single dispatch |
| **Q105 Two Accounts of One Tragedy** | Read independent Covenant and Dominion records in either order → record contradictions | Two-source evidence is historical; Dominion copy is archived evidence, not a friendly Dominion NPC conversation |
| **Q106 The King's Lost Adviser** | Travel to Valecrown archive → preserve Edric's correspondence → Mara verifies chronology | Travel and document evidence planned; don't confuse Edric's death with the Dominion delegate's death |
| **Q107 The Brother Who Came for Peace** | Verify anti-peace position, official delegate status and High Ruler's personal request in flexible order → report | **Title nuance:** chief did not come *in support* of peace. He opposed talks and reluctantly attended as official delegate at his brother's urging |
| **Q108 The Records They Buried** | Enter hostile Dominion archive → copy suppressed security orders → escape → authenticate | **Objective bug:** `encounter_defeated:hostile_archive_escape` describes escaping guards, not necessarily killing them. Use future extraction outcome; no friendly Dominion authority |
| **Q109 Road to the Meeting** | Survey historical approach → reconstruct Covenant/Dominion routes in either order → identify attack window | Separate reconstruction evidence from geographic arrival; historical site changed since attack |
| **Q110 Evidence Planted Twice** | Recover two planted clues in either order → compare common fabrication signatures | Shared provenance supports manipulation; does not independently prove Su'vareth identity yet |
| **Q111 The Witness Who Disappeared** | Obtain protected witness location → reach refuge → defend extraction → record testimony | Witness survival and testimony are separate facts; checkpoint retry must preserve protected identity |
| **Q112 The Delegation's Second Ledger** | Compare payment ledger and restricted-access register in either order → Mara traces organizer | Distinct independent records, not an automatic accusation |
| **Q113 The False Safe House** | Investigate decoy → identify staged clues → locate independently corroborated genuine trail → Sera verifies | Decoy must not be mistaken for actual Su'vareth headquarters; wrong lead should be recoverable |
| **Q114 The Veiled Hand** | Assemble independently sourced case → present to Oth'varen → authenticate founder signature | His memory is corroborating context, not sole proof; identity attribution precedes confrontation |
| **Q115 The Witness Vault** | Enter solo instance → protect witnesses/captives → defeat Su'vareth → recover contingency orders | Su'vareth dies permanently. Boss outcome, captive rescue and orders are distinct durable evidence; retry cannot erase prior protected witnesses |
| **Q116 Orders That Outlive Her** | Authenticate standing orders and delivery schedule in either order → Mara issues warning | Her operation persists through successors, not through a resurrection |
| **Q117 A Truth Neither Side Wants** | Return to Aldren → present evidence → record corrected royal history → Mara confirms ongoing war | Court correction is authoritative and persistent. **No automatic Covenant–Dominion peace**; Dominion was not present as allied partner |
| **Q118 The War That Remains** | Recover encrypted Veilbreaker instructions → Oth'varen translates → derive Anchor destination → transfer to Sera | **Title mismatch:** current title suggests war epilogue but objectives introduce Veilbreaker. Propose rename to *The Veilbreaker Orders* (editorial approval required). Oth'varen learns modern operation here, not before |
| **Q119 The Warden's Orders** | Sera accepts lead → authenticate equipment destination → receive assignment | Distinct receipt and deployment interactions; avoid repeated handoff with Q118 |
| **Q120 Network Awakens** | Reach targeted Regional Anchor → install independent sensors → compare readings → report | Instruments must produce trusted measurement facts; no invented automatic server credit for entering zone |
| **Q121 The First Breach** | Locate Unmade installers → defeat hostile technicians → preserve altered components | Kill objective only if exact creature-life IDs and counts are intentionally specified; 'defeat installers' could be an encounter success fact |
| **Q122 A Signal Answered** | Intercept relay → authenticate Warden command signature → Sera records chain | Warden title/signature can be established without claiming full Khor'vun motives |
| **Q123 The Warden's Approaches** | Scout convoy route → copy manifest and destination markings in either order → report | Reconnaissance and document evidence planned; no mandatory faction PvP |
| **Q124 Lines of Defense** | Garrick briefing → defend Covenant outpost → secure civilian exit → report | Protect civilians and outpost as separate authoritative facts; no arbitrary kill count replacing defense |
| **Q125 The Anchor Beneath** | Enter buried installation → recover synchronization schematic and original-tear evidence → report | **Critical canon:** Anchors manipulate the original tear; they are not the Seven's Binding or prison |
| **Q126 A Measure of Time** | Compare activation clocks and relay signatures in either order → Sera confirms convergence | Measurement supports urgency; do not invent a precise countdown unless gameplay timing is defined |
| **Q127 The Gathering Storm** | Request royal audience → present corroborated threat → obtain limited expedition mandate | Limited support, not unrestricted authority over other nations or Kel'vari lands |
| **Q128 Before the Veil Breaks** | Nessa briefing → verify extraction routes and instrument teams in either order → record staging readiness | **Duplicate title:** Q154 uses the same name for final assault preparation. Proposed Q128 rename: *The Expedition Musters*; keep Q154's title |

## Continuity and implementation corrections to approve

1. **Q107:** The Dominion High Ruler's brother was a Great Clan chief, fiercely opposed peace, reluctantly attended as an official delegate at his brother's urging, and died in the attack. Preserve this complexity even if title is later revised.
2. **Q108:** Hostile archive infiltration and escape must not be converted into a mandatory guard-kill objective. Evidence survives failed escape/retry.
3. **Q114–Q115:** Independent Su'vareth attribution precedes the solo dungeon; her death is permanent, and orders remain active afterward.
4. **Q117:** Aldren corrects the record, but the war persists. Do not introduce friendly Dominion witnesses or a peace treaty.
5. **Q118:** Oth'varen does not have foreknowledge of the modern Veilbreaker. Rename proposal avoids conflating war aftermath with recovered operational intelligence.
6. **Q120–Q126:** Anchor network manipulation is directed toward widening the original tear, not reinforcing the Seven's Binding. Stage data must use this distinction consistently.
7. **Q128:** Distinguish early expedition logistics from Q154's final Veilbreaker assault; the player has not yet reached the irreversible event.
8. **Cross-pack progression:** preserve stable prerequisites when packs are eventually introduced; current exact-catalogue-byte journal binding requires a version migration strategy first.
9. **No inferred hard level gates:** the narrative band is pacing guidance, not eligibility. No mandatory PvP, profession, party or raid gate.

## Evidence, rewards and retries

- Historical evidence: Edric letters, dual-faction records, false attribution clues, witness testimony, Su'vareth orders, Anchor measurements and royal corrections must persist independently of physical document possession.
- Physical quest items, if any, need separate current-possession and consumption-at-turn-in decisions. No automatic inventory consumption is inferred from 'recover' or 'copy'.
- Stable NPC templates and semantic action IDs are unassigned. A generic conversation is not enough for an authenticated report.
- XP, currency, item rewards and credit scope beyond initial per-character are not invented.
- Encounter defeat, rescue, escape and defense require future authoritative event types where native kill/interaction cannot express them.
- Failed solo encounters reset only active encounter state, not previously committed evidence or one-time story outcomes.
- Standard quests should require no Luau. Optional scripted presentation must not own authoritative quest credit.

## Consolidated legacy context — 3 November 2026

**ARCHIVED source evidence / WORKING authoring detail.** The [retired migration note](https://github.com/code-studio-au/theia-mmo/blob/399561a1f7645b481961f56ef115c6572bdec86b/docs/story/Theia/Campaigns/Human_Q104_Q128_Legacy_Pack_Migration.md) compared `Human_45_55_Playable_Quest_Pack.md` v0.1 and `Human_45_55_Implementation_Brief.md` v0.2. Both contained the same 25 Q104–Q128 references with no additional quests. Their distinct case, political and Anchor-network constraints are retained above; historical named locations remain in the [retained historical site proposals](Human_Legacy_Sidequest_Zone_Character_Archive.md#retired-production-location-and-lore-proposals).

- Covenant and Dominion accounts are independent evidence, not a declaration of moral equivalence. Preserve the Great Clan chief's anti-peace position, reluctant official attendance and personal relationship to the High Ruler, and distinguish his murder from Edric's.
- The old chapter-wide lore payoff is not knowledge already available to the player at Q104. Su'vareth attribution, her permanent death and the recovered Veilbreaker intelligence retain their canonical sequence; no prior Oth'varen foreknowledge is implied.
- Retain Classic manual target/facing and weapon-timed autoattack, solo-accessible critical instances, safe recovery and no mandatory PvP, profession, class or party requirement. Travel remains physical roads, scheduled ships and fixed flight paths; restricted Passage is exceptional and permission-based.
- Historical level-45/50 assumptions, draft completion aliases and generic accept/inspect/report templates are not approved eligibility, runtime IDs or replacements for specific objective evidence.
- Positions, actor identities, semantic actions, rewards, encounter tuning, QA and integration remain unresolved authoring inputs. Evidence durability and separate possession/consumption decisions remain governed by the requirements above, not by obsolete generic retry clauses.

## Next pass

Review Q129–Q157 (distributed Warden network, Vaer'uun's selective truths, Keeper/Guardian testimony, Veilbreaker's irreversible activation). Approved edits belong in the single current manifest, with narrative/dialogue references updated together; this review is not a competing quest-definition set.
