# Human Q104–Q128 — Legacy Pack and Brief Reconciliation

**Status: Needs Review; editorial reconciliation, not executable quest definitions.** Narrative authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Sole structured inventory: [Human 1–80 Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json). Implementation agreement: [Codex Quest Engine](../../../QUEST_ENGINE.md). Companion: [Q104–Q128 Rust Authoring Review](Human_Q104_Q128_Rust_Authoring_Review.md).

## Sources compared

- `Human_45_55_Playable_Quest_Pack.md` v0.1: **25 Q104–Q128** quest sections, player-purpose and named-location summaries, common encounter/recovery rules.
- `Human_45_55_Implementation_Brief.md` v0.2: the **same 25 Q104–Q128** quest sections, mostly generic investigation/interaction stage templates, QA and build requirements.

Neither source introduces additional Q identifiers. Historical minimum-level 45/50 assertions and `HUMAN_Q###_DONE` flags are **not approved executable eligibility or permanent runtime IDs**. The generic accept/inspect/report stages in the implementation brief do not supersede the manifest's more specific objectives.

## Preserved quest-by-quest context

| Quest | Historical gameplay purpose | Reconciled canonical requirement |
|---|---|---|
| Q104 The Delegation That Failed | Collect surviving Delegation dispatches, Oth'irun archive | Independently authenticate provenance; not a verdict by itself |
| Q105 Two Accounts of One Tragedy | Compare Covenant and Dominion records | Both perspectives are preserved as evidence, not moral equivalence |
| Q106 The King's Lost Adviser | Examine Edric's surviving correspondence | Distinguish Edric's murder from the Dominion delegate's murder |
| Q107 The Brother Who Came for Peace | Investigate Dominion High Ruler's brother | Great Clan chief opposed peace, reluctantly attended as official delegate at his brother's urging, then died |
| Q108 The Records They Buried | Recover suppressed military security orders | Hostile Dominion infiltration; escape does not automatically mean guard killing |
| Q109 Road to the Meeting | Reconstruct both delegations' approach routes | Independent arrivals and attack window |
| Q110 Evidence Planted Twice | Compare two planted trails | Shared manipulation evidence, not premature attribution |
| Q111 The Witness Who Disappeared | Locate and protect surviving witness | Witness survival and testimony are separate durable facts |
| Q112 The Delegation's Second Ledger | Compare contractor payments and restricted access | Preserve independent payment/access provenance |
| Q113 The False Safe House | Expose decoy and locate genuine lead | A false lead is not Su'vareth's actual base |
| Q114 The Veiled Hand | Authenticate founder's signature | Attribution precedes confrontation |
| Q115 The Witness Vault | Rescue captives, defeat Su'vareth, recover orders | Solo-accessible encounter; Su'vareth dies permanently; contingency orders persist |
| Q116 Orders That Outlive Her | Trace standing orders and next active operation | Su'vareth's death does not end her operation |
| Q117 A Truth Neither Side Wants | Present corrected record to Aldren | War continues; no instant peace treaty |
| Q118 The War That Remains | Translate recovered Veilbreaker orders | Current proposed rename *The Veilbreaker Orders* better matches objectives; no prior Oth'varen foreknowledge |
| Q119 The Warden's Orders | Authenticate delivery and receive Sera's assignment | Avoid redundant handoff with Q118 |
| Q120 Network Awakens | Survey Regional Anchor activity | Instruments and measurements need trusted evidence |
| Q121 The First Breach | Stop hostile Unmade installers | Specific encounter result distinct from recovered components |
| Q122 A Signal Answered | Intercept Warden relay and trace source | Signed command evidence |
| Q123 The Warden's Approaches | Reconnoiter convoy and destination | No mandatory PvP |
| Q124 Lines of Defense | Defend outpost and evacuate civilians | Separate defense and evacuation outcomes |
| Q125 The Anchor Beneath | Recover synchronized activation evidence | **Anchors manipulate the original tear; they are not the Seven's Binding** |
| Q126 A Measure of Time | Compare activation clocks and relay signatures | Urgency without invented exact countdown |
| Q127 The Gathering Storm | Seek limited royal expedition mandate | No unilateral authority over other peoples |
| Q128 Before the Veil Breaks | Stage teams, evacuation and instruments | Proposed rename *The Expedition Musters*; distinguish from Q154 final assault |

## Shared encounter and authoring rules preserved

- Classic manual target/facing and weapon-timed autoattack; no required PvP, profession, class or party.
- Story-critical instances are solo-accessible; committed evidence and safe checkpoints persist across death/reconnect.
- No friendly Dominion NPC cooperation in hostile infiltration; no automatic peace after historical correction.
- Travel uses physical roads, scheduled ships and fixed flight paths, with exceptional restricted Passage only by permission.
- Quest IDs and completion aliases in old drafts are editorial references, not verified runtime identifiers.
- Evidence should be historical and durable even when physical documents are lost; inventory possession/consumption requires separate definition.
- Actor IDs, semantic interaction actions, world locations, rewards, encounter tuning, QA and actual engine integration remain unresolved.
- Codex's current Rust catalogue only supports native kill and semantic NPC-interaction objectives; other events are planned. Development catalogue limits are not permanent production limits.

## Superseded assumptions and cleanup

Do not adopt the old hard level-45/50 requirements, repeated generic stages, or chapter-wide lore payoff as knowledge the player already possesses at Q104. Both old files cover exactly the same 25 quest references. Their useful player purposes, named locations and shared QA/recovery rules are recorded here and in the canonical campaign, current manifest and Rust authoring review.

**Original sources are retained** until inbound references and remaining unique details are verified. Once confirmed, remove redundant source files from the active tree; Git history preserves them.

**Next:** reconcile Q129–Q157 (the Warden's distributed operation, Vaer'uun, Keeper/Guardian testimony and Veilbreaker).
