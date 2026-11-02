# Theia — Rust Quest Authority / Luau Script Boundary (Proposed ADR)

**Status: Proposed for approval, not implemented.** The project is a Rust/Bevy MMO. Existing Rust objective progress is documented in [Quest Objectives Foundation](../../../QUEST_OBJECTIVES_FOUNDATION.md). Story authority is [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). This document reconciles narrative authoring with the actual implementation; it does not claim Luau is already integrated.

## Responsibilities

**Rust server:** validates quest definitions and content hashes; checks level, prerequisites, NPC proximity and interaction authority; accepts/abandons quests; consumes trusted combat/interaction facts; atomically persists objective credit, receipts and turn-in rewards; manages authoritative encounter state, timers, personal world flags and replay fencing; publishes bounded, ordered events to scripts.

**Luau sandbox:** responds to Rust-approved events, selects authored dialogue nodes and scene beats, requests spawning or encounter transitions through whitelisted capabilities, and proposes quest-specific reactions. Luau cannot mutate authoritative state, invent earned kill credit, grant XP/items/currency, bypass prerequisites or call arbitrary IO/network/OS functions.

**Bevy client:** presents dialogue, subtitles, objective UI, cameras and visuals; sends interaction intents to server. Client callbacks cannot assert authoritative objective completion.

## Event flow

1. Client sends an intent (interact, dialogue selection, ability use) with the existing authenticated session context.
2. Rust checks proximity, character ownership, quest eligibility, NPC/encounter validity, rate limits and event ordering.
3. Rust commits any earned fact and its receipt using the existing objective foundation, or rejects the request.
4. Rust emits a bounded event to the Luau quest script with read-only context.
5. Luau returns a constrained action proposal (e.g. `show_dialogue`, `request_scene`, `request_spawn`, `request_objective_transition`).
6. Rust validates the proposed action against the quest definition, allowed capabilities, personal story state and deduplication receipts before committing or sending presentation instructions.

**Example event names (proposals, not existing APIs):** `quest.accepted`, `npc.interacted`, `dialogue.choice_selected`, `objective.progressed`, `encounter.boss_defeated`, `quest.ready_to_turn_in`, `quest.completed`, `story.phase_entered`.

## Luau authoring sketch (non-executable pseudocode)

```lua
-- Q156: illustrative contract only; actual host API not yet defined.
function on_event(event, ctx)
  if event.type == "encounter.boss_defeated" and event.boss == "khorvun" then
    return { action = "request_scene", scene = "veilbreaker_after_khorvun" }
  end
end
```

Rust owns the atomic transition: Khor'vun defeated → Veil destroyed → Binding immediately dissolved → Seven released → Shroud dissipation started. Scripted presentation must not independently grant or reverse these world facts.

## Integration design gates

- Decide whether to embed a Luau VM using a Rust binding or run an isolated script worker. Document sandbox, resource/fuel budget, memory ceiling, panic isolation, script versions and rollback.
- Keep Rust objective/receipt semantics from P3-11; do not replace them with script-side counters.
- Define versioned event payloads and action schemas, capabilities per quest, deterministic event ordering and safe replay after disconnect.
- Bind existing JSON quest IDs and dialogue IDs to scripts only after validation; unknown targets fail closed.
- Add Rust integration tests for kill-credit provenance, double turn-in, restart/reconnect, malicious script requests, boss phase recovery and Q156/Q181 irreversible transitions.
- Prototype one existing Human starting-zone quest first, then one boss encounter; avoid porting all 184 quests until the boundary is proven.

## Disposition of earlier JavaScript prototype

The obsolete JavaScript prototype and tests were removed from the current tree. Their historical versions remain recoverable through Git history. Implement the relevant behavior and tests against the existing Rust foundation instead.
