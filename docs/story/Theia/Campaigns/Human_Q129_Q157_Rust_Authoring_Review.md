# Human Q129–Q157 — Rust Quest Authoring Review

**Status: Needs Review — authoring proposals, not executable quest definitions.** [Canonical Human Campaign](Human_1_80_Canonical_Campaign.md) controls story; [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the sole working structured inventory; [Codex Quest Engine Agreement](../../../QUEST_ENGINE.md) controls implemented capabilities.

**29 quests reviewed, none deleted, merged or renumbered.** The Rust catalogue currently supports ordered stages with `all/any/at_least` completion and only `kill` and semantic `npc_interaction` objectives. Pure journal and PostgreSQL Store are independently qualified, but expanded World/client adoption remains pending. Inspection, exploration, object use, defense, cinematics, world phasing, branching and Luau are planned. Development catalogue limits are not permanent production limits.

## Quest-by-quest authoring matrix

| Quest | Proposed evidence and stage grouping | Runtime and narrative safeguards |
|---|---|---|
| **Q129 The Warden Unmasked** | Reach relay → recover signed orders → Sera independently verifies Khor'vun identity | Travel and document inspection planned; Warden identity must be authenticated, not inferred from enemy appearance |
| **Q130 Anchors in Concert** | Survey three relay sites in any order → Tarek compares synchronized signatures | Three independent measurements; no one-site shortcut or invented auto-credit from entering region |
| **Q131 Siege of Lantern Redoubt** | Aru'ven briefing → defend civilians → maintain limited Passage evacuation → confirm survivors | Restricted gate capacity remains limited. Rescue and gate throughput require authoritative encounter/world facts |
| **Q132 The Pattern Breaks** | Inspect topology and isolated branch in either order → Sera approves bounded disruption | Must not imply full network can be stopped by one cutoff |
| **Q133 Into the Warden's Forge** | Enter solo relay dungeon → defeat guards → reach protected controls | Boss/guard outcome and access separate; solo checkpoint, retry and authenticated completion |
| **Q134 The Severed Signal** | Activate branch cutoff → preserve remaining readings → Sera confirms partial victory | **Partial success:** one relay branch interrupted; no global shutdown |
| **Q135 Aftermath of the Break** | Compare post-cutoff network and unaffected branches in either order → report ongoing convergence | **Potential pacing duplication** with Q134. Keep for now because it establishes strategic failure; review whether an independent quest is necessary |
| **Q136 What the Warden Left Behind** | Recover First Voice orders and verify provenance → Mara requests archive comparison | Evidence survives if a physical copy is lost |
| **Q137 Records of the First Voice** | Authorized Oth'irun archive access → early Vaer'uun research → Sel'ora authenticates chronology | Vaer'uun began as respected scholar; no invented early monster narrative |
| **Q138 The Forgotten Investigation** | Enter ancient lab → replay historical debate → preserve independent annotations | **Spoiler check:** do not use Nhal's name in player dialogue before Q143; Oth'varen's account is limited |
| **Q139 The Missing Records** | Identify omitted index → reconstruct damaged pages → Sel'ora confirms omission | Do not assert who removed the pages before independent proof |
| **Q140 The Voice That Answers** | Restore approved communicator → hear Vaer'uun's calculated invitation → verify destination independently | Vaer'uun offers real historical evidence as bait; do not reveal Nhal's name here |
| **Q141 Beneath the Binding** | Reach invited facility → preserve authentic evidence → stop Unmade exploitation → report | Evidence can be true while invitation is manipulative. 'Stop' must be authored as encounter outcome, not arbitrary kill |
| **Q142 Truth Withheld** | Confront Oth'varen → compare contradiction → record partial admission | Player can distrust him; he accepts responsibility for omissions without endorsing Vaer'uun |
| **Q143 Name in the Records** | Authenticate Nhal name from two independent sources → Sel'ora records discovery | **First authenticated Nhal name**; no earlier journal, speaker label or tooltip spoilers |
| **Q144 The First Voice's Design** | Locate apparatus → defeat defenders → disable exploitation cutoff | Defender kill and machine shutdown distinct; authorized object interaction required |
| **Q145 The First Voice** | Enter solo dungeon → disable apparatus → kill Vaer'uun → preserve Keeper clue | Vaer'uun knowingly serves Nhal; permanent death, solo-safe checkpoint, evidence after boss |
| **Q146 The Silence Beneath** | Compare Vaer'uun claims and independent testimony → Oth'varen acknowledges unresolved Keeper role | Do not equate valid records with Vaer'uun's moral justification |
| **Q147 The Weight of Silence** | Present evidence to Sera → confront Oth'varen → receive restricted testimony access | Separate semantic NPC actions; access grant persists |
| **Q148 The Sealed Testimony** | Enter authorized archive → read two independent Divine War accounts in either order → authenticate | Two testimonies independently sourced; avoid pretending either saw the entire war |
| **Q149 The Missing Account** | Locate Keeper omission → restore passage → verify he knowingly concealed consequences | Establish deception through independent authenticated text |
| **Q150 Guardian's Doubt** | Examine Guardian's later testimony → verify discovery occurred *after* Binding → Sera confirms | Guardian was deceived too; never imply Guardian knew before the Binding |
| **Q151 The Seven Bound** | Assemble Keeper and Guardian evidence → Oth'varen explains Binding, Shroud and separate Anchors | **Critical canon:** the Anchors do not imprison the Seven. Binding is prison; Veil is what it contains |
| **Q152 The Cost of Containment** | Compare bounded inversion results with Binding history → Sera records unproven countermeasure | No proven ability to destroy divine essence, defeat Nhal or restore Veil |
| **Q153 The Warden's Final Pattern** | Survey synchronized signals → identify final Veilbreaker sequence → Tarek confirms urgency | Measurements may show irreversible threshold approaching, not guarantee exact timing |
| **Q154 Before the Veil Breaks** | Final assault briefing → verify evacuation and instrument teams → authorize approach | Distinct from Q128 early expedition muster. Avoid repeated generic checklist without unique gameplay |
| **Q155 The Last Approach** | Reach complex → defeat defenses → inspect irreversible activation threshold | **Potential sequencing risk:** threshold is already crossed when player arrives; combat success cannot reopen cutoff window |
| **Q156 Veilbreaker** | Enter central chamber → kill Khor'vun → attempt cutoff → record Veil destruction, immediate Binding dissolution, Seven freed | **Irreversible canonical event** must be one atomic personal-story/world transition, not separate player-triggered scene credits that can partially complete |
| **Q157 The Shroud Begins to Fall** | Protect Kel'vari evacuees → record Shroud's ongoing dissipation and gradual Nhal manifestation | **Timing nuance:** Shroud began fading immediately in Q156; Q157 witnesses continuing fade, not its start. Nhal manifestation is slow |

## Priority consistency corrections

1. **Q155–Q156 atomicity:** Once Veilbreaker destroys the Veil, Binding dissolution and release of the Seven are immediate consequences. The player defeats Khor'vun but cannot reverse the already-completed activation. Never award a successful shutdown flag.
2. **Q157 timing:** The stage description `Witness Shroud immediately begin dissipating` can imply the fade starts after the Q157 rescue. Rewrite as observing **ongoing** dissipation that started at Q156.
3. **Q151 causal explanation:** The Veil and Binding are distinct. Anchors converge energies to strengthen the original tear and enable Nhal's release; they do not sustain the Seven's prison.
4. **Q143 reveal gate:** Nhal's name is first authenticated here. Audit earlier dialogue, journal text, object names, map markers and UI metadata for premature disclosure.
5. **Q138–Q145 Vaer'uun:** The First Voice's manipulation uses genuine evidence and withheld history. He knowingly serves Nhal, though corrupted; his death is permanent.
6. **Q134–Q135 pacing:** One branch severed, remaining network active. Consider whether the second after-action quest delivers sufficient new understanding, without silently removing it.
7. **Q149–Q151 independent proof:** Keeper's deception and Guardian's later discovery must be established before Oth'varen's complete account.
8. **Q155–Q157 retry and persistence:** Combat retries cannot replay Veil destruction as a new global event. Completed story facts are per-character and idempotent until a realm-level phasing policy exists.
9. **Q129–Q154 engine types:** Inspect/observe/defend/travel/object use are planned capabilities. Native kill objectives cannot substitute for encounter outcomes, safe evacuation or world transitions.
10. **No inferred hard level gates:** narrative level bands remain pacing guidance; no mandatory PvP, profession, party or raid requirement.

## Evidence, rewards and implementation constraints

- Signed orders, archive testimony, measurements and research conclusions are **historical facts**, not automatically items that must remain in inventory.
- Current possession and item consumption must be separately specified for any later physical quest items.
- NPC interactions require stable actor/action pairs; report, approve, disclose and authenticate are different semantic actions.
- Per-character credit is the initial supported scope; party/instance sharing is planned.
- XP, currency, item rewards, exact stable NPC/creature IDs and failure/abandonment policy are not yet approved.
- Solo boss checkpoints preserve verified evidence and prevent repeat rewards; no Luau required for ordinary quest objectives.
- The future cross-pack registry and content-version migration rules must be qualified before production quest content is deployed.

## Consolidated legacy context — 3 November 2026

**ARCHIVED source evidence / WORKING authoring detail.** The [retired migration note](https://github.com/code-studio-au/theia-mmo/blob/90365d28de75c7596d5119c6a9595995ec15e495/docs/story/Theia/Campaigns/Human_Q129_Q157_Legacy_Pack_Migration.md) compared `Human_55_70_Playable_Quest_Pack.md` v0.1 and `Human_55_70_Implementation_Brief.md` v0.2. Each covered the same 29 Q129–Q157 references, with no additional quests. The matrix above preserves their distinct narrative and encounter constraints; historical locations remain in the [existing location/lore reference](Human_1_80_Location_and_Lore_Editorial_Reference.md).

- Preserve Kel'vari sovereignty as well as restricted Passage permission and capacity. Physical roads, ships and fixed flight paths remain ordinary travel; no ordinary teleport waypoints are inferred.
- Classic explicit targeting, manual facing, range and weapon-timed autoattack remain the combat baseline. Critical story encounters stay solo-accessible, with no mandatory PvP, group, class or profession requirement.
- Persist authenticated evidence and personal story facts across death/reconnect; reset only active hostile encounter state. Boss outcomes, device cutoffs, research copies, evacuation and irreversible world transitions are distinct evidence, not generic kill or conversation credit.
- Historical level-55/60/65 assumptions, draft completion aliases and repetitive accept/inspect/report templates are not approved eligibility, runtime IDs or additional canonical objectives.
- The old chapter-wide Veilbreaker/Binding/Shroud payoff is not knowledge available before its canonical revelation sequence. Q157 observes Shroud dissipation already begun at Q156; it does not start the event again.
- Spawn geometry, world positions, localization, combat tuning, QA and actual client/World integration remain unresolved alongside the authoring inputs above. These historical plans do not establish implemented capabilities.

## Next pass

Review Q158–Q184, including manifestations, Thes'var's facility, Oth'varen's sacrifice, Nhal's shattering and the Human homecoming. Apply accepted refinements to the **single working manifest**, not a competing quest inventory.
