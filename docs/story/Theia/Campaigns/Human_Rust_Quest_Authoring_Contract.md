# Theia — Rust Quest Content Authoring Contract

**Status: proposed content-authoring standard / Needs Review.** Source: Codex instruction supplied by the creator on 2 November 2026. The [canonical Human 1–80 campaign](Human_1_80_Canonical_Campaign.md) remains the sole narrative authority. This contract governs translation into future executable Rust content; it does not claim the engine supports the planned capabilities yet.

## Separate identities and sources

- **Editorial ID:** preserve historical quest identifiers such as `H1-14` unchanged when a quest is reused. An original quest can map to multiple revised quests; a revised quest can consolidate multiple originals. Record relation type and review status. Never infer mapping solely from a similar title.
- **Runtime ID:** stable namespaced identifier, e.g. `theia.quest.human.h1-14` for a confirmed original; for newly created quests use an independent allocated identifier (proposal: `theia.quest.human.launch.0001`), never silently recycle an original ID.
- **Current campaign inventory ID:** `Q001`–`Q184` is an editorial ordering/reference index, not a runtime identity. `HUMAN_Q001_DONE` etc. are draft completion aliases to migrate explicitly.
- **Narrative Markdown:** story, characterization, dialogue, locations and intended emotional effects.
- **Executable definition:** validated data with runtime ID, schema version, content digest, giver/turn-in actor IDs, eligibility conditions, stage graph, typed objectives, evidence, credit rules, persistent choice keys, rewards and failure policy.

## Per-quest required fields

| Field | Meaning |
|---|---|
| `runtime_id` / `editorial_ids` | Permanent runtime identity and original editorial provenance |
| `content_version` / `schema_version` | Immutable definition revision and validation schema |
| `narrative_ref` | Relative path to human-readable quest documentation |
| `giver_npc_id` / `turn_in_npc_id` | Stable actor template identifiers; distinct from runtime spawn entity IDs |
| `prerequisites` | Explicit completed-quest/flag/condition predicates; no inferred level gates |
| `stages` | Stable stage IDs, objective IDs, typed target references, required counts, completion rules, transitions |
| `completion_evidence` | Trusted event receipt, validated interaction, inventory snapshot or approved world-state fact |
| `credit_scope` | Character, or planned party/instance scope with authoritative eligibility |
| `persistent_choices` | Stable choice key, legal values and whether it affects later reactions |
| `rewards` | XP, currency and item grants; exact values or explicit unresolved blockers; atomic once-only turn-in |
| `retry_policy` | What resets on failure, logout, abandon and instance restart |
| `capabilities_required` | Current native engine versus planned multi-stage, branch, explore, inventory, party sharing, phasing or optional Luau |
| `quest_item_consumption` | Whether relevant quest items are consumed at turn-in; independent of objective history |

## Objective semantics

- **Historical progress:** count eligible validated past facts (e.g. three unique creature-life kills, three conversations or regions discovered). Later loss of inventory does not undo a historical fact.
- **Current possession:** evaluate the authoritative current inventory snapshot at the required stage/turn-in; obtaining then dropping an item does **not** satisfy possession.
- **Consumption:** separate flag per item requirement; inventory removal and reward grant must be atomic at turn-in, or the transaction fails with no partial result.
- **Evidence:** combat comes from trusted Rust victim-life facts, not client-provided `CreatureKilled`; interactions from authorized NPC/object proximity and state; inventory from server-owned balances.
- **Credit scope:** default character; party sharing and instance credit are future capabilities and must be labeled planned, not implemented.
- **Standard quests:** use Rust native objective evaluators without scripting. Luau is an optional sandboxed extension only when standard content cannot express an authored event.

## Level and progression policy

Narrative bands (1–5, 5–10, etc.) describe pacing. **Do not** translate them to `minimum_level` prerequisites unless explicitly approved as a real gate. The previously generated `minLevel` in `Human_1_80_Current_Quest_Authoring_Manifest.json` and all quest JSON drafts is a **provisional planning annotation** and must be reviewed or omitted from executable eligibility. Any explicit level-80 finale gate must be separately documented as intentional; the level-5 narrative transition is likewise not automatically approved as a runtime requirement.

## Migration and validation

1. Build a reviewed crosswalk from original H-series IDs to Q001–Q184; mark `reused`, `modified`, `split`, `merged`, `retired` or `new`.
2. Allocate runtime IDs only after confirming provenance. Never delete old IDs from the migration record.
3. Transform each quest into typed stages/objectives with stable IDs and evidence semantics, without changing its canonical narrative.
4. Validate all references, objective target kinds, completion rules, choice values, consumption behavior, reward definitions and prerequisite graph.
5. Mark unsupported features as **planned**, not implemented. Existing P3-11 Rust objective foundation supports only its qualified initial slice; do not claim it handles the full campaign.
6. Prototype a standard kill quest with no Luau; then NPC interaction/turn-in; then staged quests; then scripted boss/cinematic events.

## Human campaign dependency review

**WORKING / OPEN, not approval of executable content.** This section preserves the unique dependency checks and remaining gates from the [retired dependency review](https://github.com/code-studio-au/theia-mmo/blob/c6ac6c69f3af21b79b228c1e03f5859556153bd8/docs/story/Theia/Campaigns/Human_1_80_Quest_Dependency_Review.md). Use the [single working manifest](Human_1_80_Current_Quest_Authoring_Manifest.json); do not recreate the deleted chapter JSON files or older dependency JSON.

### Structural checkpoint — 3 November 2026

The Markdown review recorded 184 campaign entries, 684 unique stage IDs and 684 unique objective IDs; all prerequisite and next-stage references resolved, and the prerequisite graph was acyclic. The provisional split was 156 main-story / 28 optional quests, with no main-story dependency on optional quests. All candidate/proposed Q references in 92 H-series and 24 HS-series source rows resolved. This is a dated structural checkpoint, not approval of the split, H/HS equivalence, runtime IDs, semantic order or executable readiness. [Current campaign status](CURRENT_HUMAN_CAMPAIGN.md) remains the entry point.

### Remaining semantic gates

- **OPEN:** validate revelation, permission and world-phase order against the canonical campaign. Structural reference checks cannot establish these.
- **Approved authoring / OPEN runtime:** Q159/Q168 access and Q169's replacement journal quest follow the creator decision ledger and are reconciled in the non-executable manifest. Q169's death-event cutoff is approved; post-conversation unfinished custody handling remains unresolved. Do not infer runtime support or an unapproved late-completion solution. Q084 depends on Q082 rather than optional Q083; Q094 depends on Q092 rather than optional Q093. Preserve those independent mandatory paths.
- Preserve Q029's civilian/evidence checkpoint before the Q030 Cassian encounter.
- Keep Q128's early expedition staging distinct from Q154's final assault preparation. Any further rename remains a proposal.
- Preserve the canonical Q156 transition and Q176–Q181 fatal-correction order. Exact events and revelation timing belong in the [canonical campaign](Human_1_80_Canonical_Campaign.md), not a second account here.
- Historical evidence, current possession, consumption, atomic rewards, retries and credit scope require quest-specific authoring under this contract.
- Consult the existing [Quest Engine agreement](../../../QUEST_ENGINE.md) for actual supported capabilities before conversion. Do not infer acceptance, branching, party sharing, phasing, exploration or Luau support from these authoring plans; development limits are not permanent campaign limits.

## Example schema (illustrative, not a deployed runtime definition)

```json
{
  "runtime_id": "theia.quest.human.launch.0001",
  "editorial_ids": [],
  "campaign_reference": "Q001",
  "schema_version": 1,
  "content_version": 1,
  "narrative_ref": "Human_1_10_NPC_Dialogue_Scripts.md",
  "giver_npc_id": null,
  "turn_in_npc_id": null,
  "prerequisites": [],
  "planning_level_band": "1–5",
  "eligibility_min_level": null,
  "stages": [
    {
      "stage_id": "intro",
      "completion_rule": "all",
      "objectives": [
        {"objective_id": "talk_darian", "type": "npc_interacted", "target_npc_id": null, "evidence": "server_validated_interaction", "progress_semantics": "historical", "required_count": 1},
        {"objective_id": "talk_anwen", "type": "npc_interacted", "target_npc_id": null, "evidence": "server_validated_interaction", "progress_semantics": "historical", "required_count": 1},
        {"objective_id": "talk_brenna", "type": "npc_interacted", "target_npc_id": null, "evidence": "server_validated_interaction", "progress_semantics": "historical", "required_count": 1}
      ],
      "next_stage": "report"
    }
  ],
  "credit_scope": "character",
  "persistent_choices": [],
  "rewards": {"xp": null, "currency": null, "items": [], "grant_once": true},
  "quest_item_consumption": [],
  "retry_policy": {"on_death": "preserve_verified_credit", "on_logout": "persist", "on_abandon": "needs_review"},
  "capabilities_required": ["planned_npc_interaction", "planned_multistage_quest"]
}
```

**The null actor IDs, XP and currency are intentional unresolved inputs.** Do not import this illustration into production.

## Creator-locked cross-race migration safety — 12 November 2026

**Important migration boundary:** The legacy `HUMAN_Q001_DONE`–`HUMAN_Q184_DONE` strings in the derived dependency reference are editorial completion aliases, **not approved runtime IDs or proof that shared Q035–Q184 require a human character**. Do not mechanically compile or enforce them as human-only gates.

**Racial entry:** The player's race-specific level 1–20 storyline must be complete before the common Q035 `The Covenant Calls` summons. Stable completion facts for dwarf, Aelari and Korrath have not been approved because their campaigns are not yet authored. An empty legacy `prerequisites` array for Q035 must not mean unconditional availability.

**Shared progression:** After Q035, common Covenant quests must not require the human-only Q031 flag or the Cassian ledger. Shared quest progress and completion must be race-neutral. Q032–Q034 remain optional human aftermath after Q031 and do not block Q035.

**Q184:** One common quest identity/completion and race-specific homeland, contact and dialogue bindings. Never grant the human Millhaven/Elara history to other races.

**Implementation acceptance checks:** Validate four distinct racial completions unlocking the same Q035; reject premature Q035 access; verify Q036–Q037 without Q031; verify Q032–Q034 remain optional; validate Q184's race-specific actors and a single shared completion; preserve saved journal/catalogue migration semantics. Until stable IDs and all race variants exist, the JSON inventories remain planning material, not importable executable quests.
