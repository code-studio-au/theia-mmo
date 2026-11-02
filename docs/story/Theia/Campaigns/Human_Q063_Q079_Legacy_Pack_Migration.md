# Human Q063–Q079 — Legacy Pack and Brief Reconciliation

**Status: Needs Review; editorial reconciliation, not executable quest data.** Narrative authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Sole structured quest inventory: [Human 1–80 Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json). Runtime agreement: [Quest Engine](../../../QUEST_ENGINE.md). Companion: [Q063–Q079 Rust Authoring Review](Human_Q063_Q079_Rust_Authoring_Review.md).

## Sources compared

- `Human_30_35_Playable_Quest_Pack.md` v0.1: **17 quests Q063–Q079**, player purposes, location bands, generic encounter/recovery clauses.
- `Human_30_35_Implementation_Brief.md` v0.2: **same 17 quests**, mostly generalized investigation and interaction stage templates plus QA requirements.

Both files contain a proposed level-30 chapter gate and `HUMAN_Q###_DONE` aliases. Neither constitutes approved executable eligibility or permanent runtime IDs. The brief's generic “accept → inspect → report” pattern must not override the manifest's explicit objectives.

## Preserved quest-specific design

| Quest | Source purpose and context | Current interpretation |
|---|---|---|
| Q063 The City That Endured | Arrive in Drom Kar, accept local custodial limits, help restore water route before archive access | Civic repair is meaningful, not merely a token to open archives |
| Q064 Water and Memory | Enter maintenance tunnels, protect workers, recover older warning | Safety and historical evidence are distinct |
| Q065 The Missing Margin | Identify omitted text in warning; seek Dromak's permission | Missing section is evidence, not proof of who removed it |
| Q066 A Name for the Guardians | Meet Sera and establish limited cooperation | No automatic Guardian membership |
| Q067 The Instruction Without Its Diagram | Compare incomplete Regional Anchor order against safeguards | Grey Surveyor's true identity remains concealed |
| Q068 Reach Them Before the Charge | Warn Halden and interrupt premature activation | Warning and actual cutoff must be separate verified events |
| Q069 The Broken Measure | Rescue researchers and safely isolate unstable Anchor apparatus | Solo encounter checkpoint; rescue persists through retry |
| Q070 Copies for the People at Risk | Distribute complete safety instructions and independent measurements | Durable evidence; a conversation alone does not prove delivery |
| Q071 Make the Measure Small | Prepare bounded Minor Resonator inversion with cutoff and limits | **Legacy name leak:** any player-facing “Oth'varen” contact is superseded by Grey Surveyor until Q097 |
| Q072 A Brief Reversal | Execute limited inversion and record measured result | No proven Nhal defeat or divine-essence annihilation |
| Q073 The Safer Argument | Hear Halden's severance view and Sera's inversion view | Persist provisional preference, no irreversible main-story branch |
| Q074 The Pattern Beneath | Compare six disturbance signatures for coordinated behavior | Observations do not establish full cosmic mechanism |
| Q075 The Anchor's Defenses | Scout Chainworks patrols and defenses | No mandatory PvP |
| Q076 The Harbinger's Design | Authenticate Oma'vira's design and Passage route | Destination clue distinct from later Guardian gate revelation |
| Q077 Against the Harbinger | Rescue captives, disable control relays, kill Oma'vira, preserve gate records | Permanent death; independent captive/evidence credit |
| Q078 The First Major Reversal | Conduct bounded Major Anchor inversion with specialists | Distinct from earlier Minor Resonator trial |
| Q079 The Harbinger's Secret | Verify destroyed Passage gate and Shroud destination | No second destruction; surviving Guardian gate not yet revealed |

## Shared gameplay and QA preserved

Classic manual targeting/facing, weapon-timed autoattack, safe solo encounters, independent case evidence, safe respawn and reconnect, no mandatory PvP/group/class/profession gate, and physical ship/flight/road travel. No ordinary access to the Shroud before Veilbreaker; any restricted Guardian Passage requires separate consent and remains a later revelation.

Author exact NPC/creature/object IDs, semantic interaction actions, encounter phases, spawn/leash rules, locations, rewards, localization and runtime integration only when verified. Death/disconnect should reset active encounter state without deleting previously authenticated research, consent or choices. Validate stage order, duplicate rewards, dialogue knowledge gates, solo completion and accessibility.

## Superseded or unapproved assumptions

- Chapter level-30 minimum is pacing guidance, not automatically a hard runtime requirement.
- Generic stages in the implementation brief are scaffolding, not final gameplay.
- The earlier pack's broad “Oma'vira killed / Major inversion / Shroud gate” lore payoff is a chapter summary, not a reveal available to the player at Q063.
- NPC contact names, scene observations and inspection objectives are **not** evidence that Codex's Rust engine currently supports these objective types.
- The current 32-quest catalogue budget is a development safeguard, not a permanent production cap.

## Cleanup disposition

Both old sources have exactly the same 17 Q references and no additional quest IDs. Their relevant purposes, region context, recovery and QA expectations are recorded here and in the canonical narrative, single manifest and Rust authoring review. **Retain the old files until inbound references and remaining unique details are verified**; then remove redundant sources from the active tree while retaining Git history.

**Next:** reconcile Q080–Q103 (Guardian gate and Kel'vari first contact) and preserve spoiler-safe labels, consent and revelation order.
