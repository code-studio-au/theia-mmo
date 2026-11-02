# Human Q129–Q157 — Legacy Pack and Brief Reconciliation

**Status: Needs Review; editorial reconciliation, not executable quest definitions.** [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md) is narrative authority, [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the sole structured inventory, and [Codex Quest Engine](../../../QUEST_ENGINE.md) defines runtime capabilities. Companion: [Q129–Q157 Rust Authoring Review](Human_Q129_Q157_Rust_Authoring_Review.md).

## Sources compared

- `Human_55_70_Playable_Quest_Pack.md` v0.1: **29 Q129–Q157** sections, location bands, player purposes, generic recovery and QA.
- `Human_55_70_Implementation_Brief.md` v0.2: the **same 29 Q129–Q157** sections, generalized stage templates and implementation blockers.

The original minimum-level 55/60/65 assertions are **not** automatic hard eligibility. `HUMAN_Q###_DONE` are draft completion aliases, not runtime IDs. Generic brief stages must not overwrite the current manifest's precise objective sequence.

## Chapter-specific narrative and gameplay retained

| Quest range | Original authoring purpose | Canonical interpretation and implementation boundary |
|---|---|---|
| Q129–Q130 | Authenticate Khor'vun's signed Warden orders and compare linked relays | Warden identity and distributed synchronization require independent evidence |
| Q131 | Defend Lantern Redoubt and maintain restricted Passage evacuation | Kel'vari sovereignty and gate capacity limits persist |
| Q132–Q135 | Find vulnerable relay branch, enter Warden Forge, sever branch, measure remaining network | One branch disrupted, wider network remains active; review Q135 pacing but do not silently remove it |
| Q136–Q139 | Trace Vaer'uun's early orders and omitted ancient research | The First Voice was originally a scholar; omissions need independent provenance |
| Q140–Q142 | Vaer'uun's invitation and selective truths; Oth'varen admits withholding evidence | Genuine records can be used manipulatively; player may distrust both without joining Vaer'uun |
| Q143 | First authenticated Nhal name | No earlier player-facing Nhal name in journal, subtitles, map or NPC text |
| Q144–Q145 | Disable First Voice apparatus, defeat Vaer'uun, recover Keeper clue | Vaer'uun knowingly serves Nhal; dies permanently in solo encounter |
| Q146–Q148 | Examine testimony and gain restricted archive access | No premature full Binding revelation; authenticated accounts remain independent |
| Q149–Q151 | Prove Keeper concealed Binding consequences and Guardian learned afterward | Guardian was deceived; Anchors do **not** imprison Seven; Binding and Veil are distinct |
| Q152–Q153 | Compare inversion results and detect final Warden convergence | Countermeasure still hypothetical; urgency not invented exact timer |
| Q154 | Prepare final Veilbreaker assault and evacuations | Distinct from Q128 early muster; no repeated generic checklist |
| Q155 | Reach fortified complex, defeat outer defenses and discover threshold already crossed | Player is too late; combat victory cannot undo irreversible activation |
| Q156 | Defeat Khor'vun, attempt cutoff, witness Veil destruction and Binding dissolution | **One irreversible transition:** Veil destroyed → Binding dissolves immediately → Seven freed → Shroud starts dissipating |
| Q157 | Protect Kel'vari evacuees, observe Shroud and gradual manifestation | Shroud fading **already began in Q156**, not after Q157; Nhal manifestation is slow |

## Shared gameplay, QA and continuity

- Strict Classic targeting, manual facing, range and autoattack; solo critical story, no mandatory PvP/group/class/profession.
- Persist authenticated evidence and personal story flags; reset only active hostile encounter state on failure/reconnect.
- Physical roads, ships and fixed flight paths; restricted Passage only with permission. No invented ordinary teleport waypoints.
- Separate verified boss outcomes, device cutoffs, research copies, evacuation and world transitions; don't reduce them to generic kill or NPC-click credit.
- The current Rust catalogue only supports native kill and semantic NPC interaction; object inspection, travel, defense, scene and world phasing are planned.
- Quest actor IDs, semantic actions, spawn/geometry, XP/loot, combat tuning, localization, client/World integration and saved-definition migration remain unapproved.
- No permanent 32-quest production catalogue limit is implied.

## Superseded source assumptions

- Historical chapter minimum levels are pacing guidance unless explicitly approved as eligibility.
- The old playable pack's broad “Veilbreaker/Binding/Shroud” lore payoff is a chapter summary, **not** player knowledge before Q156.
- The implementation brief's repetitive accept/inspect/report templates are not additional canonical objectives.
- The source phrase “Shroud dissipation begin immediately” at Q157 must be interpreted as observing an event that started at Q156, not starting it again.

## Cleanup disposition

Both older files contain exactly the same 29 Q references and no additional quests. Their useful location, gameplay, encounter/retry and QA details are represented here, in the canonical campaign, manifest and Rust authoring review. **Keep original sources until incoming links and any remaining unique details are verified**; then remove them from the active tree, preserving history in Git.

**Next:** reconcile Q158–Q174 and Q175–Q184, then audit inbound references and begin deleting redundant packs safely.
