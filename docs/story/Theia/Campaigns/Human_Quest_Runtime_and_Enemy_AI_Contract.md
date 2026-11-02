# Theia — Engine-independent Quest Runtime and Enemy AI Contract v0.1

> **Architecture correction — 2 November 2026:** Theia's implementation is Rust/Bevy. Rust owns authoritative quest lifecycle, objective validation, persistence, rewards and event delivery; Luau is the intended sandboxed authoring layer for quest-specific dialogue, scripted reactions and encounters, pending integration design and approval. The JavaScript companion prototype is a disposable illustrative simulator **not** the implementation target. Preserve and extend the existing Rust P3-11 objective foundation; do not build a competing quest runtime. See [Rust–Luau Quest Scripting Boundary](Human_Rust_Luau_Quest_Scripting_Boundary.md).

**Status:** technical prototype proposal, not approved combat tuning or MMO engine integration. Story authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Structured quest data: [Human 1–80 Current Quest Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json).

## Quest runtime

- **Definition identity:** stable quest ID, unique completion flag, minimum level, prerequisite completion flags, ordered stages with unique stage IDs, target keys, event kinds and reset policies.
- **Character state:** level, accepted quests, stage-credit sets, completed quest flags, reward-claim ledger, personal world-stage flags. Store on server with transactional writes; simulator uses an in-memory state object only.
- **Lifecycle:** LOCKED (level or prerequisite missing) → AVAILABLE → ACTIVE → READY_TO_TURN_IN → COMPLETED. Abandon ACTIVE to AVAILABLE without deleting globally persistent evidence; repeated completion and reward claims must be idempotent.
- **Event contract:** `{kind,targetKey,questId?,encounterId?,actorId?,eventId?}`. The server validates interaction proximity, actor authorization, encounter outcome and instance membership **before** crediting a stage. An event sent by a client is never sufficient proof of a kill, defense or world transition.
- **Stage order:** default sequential for the initial slice. Future definitions may declare objective groups that can progress in parallel. Stage credits must be idempotent. `encounter_retry` stages reset on failed encounter; completed evidence stages persist.
- **Turn-in:** require all mandatory stage credits, valid authorized turn-in contact, and one atomic transaction that records completion and grants rewards exactly once. Never use a client-provided XP amount.
- **Availability:** regional optional quests must never be prerequisites for main-story quests. Story access and level gates are distinct: XP may come from regional activity even if story is paused.
- **Personal story phasing:** Veilbreaker Q156 and Nhal finale Q181 advance per-character world-state flags. Shared persistent world behavior needs explicit realm-level policy; a cutscene cannot repeat as a second global catastrophe.
- **Versioning:** definitions carry schema version and content revision; migrations must map stage credits when content changes. Never silently discard progress for characters mid-quest.
- **Security:** authoritative validation, replay protection and audit logging are required in the future server. The prototype does not supply these.

## Enemy AI state machine

`IDLE → PATROL → ALERT → CHASE → ATTACK → CHASE`; any engaged state can enter `EVADE → RETURN_HOME → IDLE/PATROL`; death enters `DEAD` and server-controlled respawn restores `IDLE`.

- **Detection:** aggro checks use perception, distance, line-of-sight, faction hostility and stealth rules. No fixed aggro radius until balancing.
- **Threat:** each hostile actor maintains a threat table keyed by valid combatant ID; update on authorized damage/healing/taunt events. Select highest eligible threat, with hysteresis to avoid rapid target thrashing.
- **Movement:** chase toward target until attack range; use pathfinding and avoid impossible terrain. Attacks require server-checked range, line-of-sight, facing and cooldown. Enemy turning uses explicit rotation; player manual facing remains unchanged.
- **Leash:** home region, maximum chase path/time and stuck checks. On leash breach, clear threat, become untargetable to new combat until returned, reset health and encounter-specific temporary effects.
- **Patrol:** waypoint traversal with idle pauses; social aggro only within authored faction groups and alert range. No endless chain pulls.
- **Bosses:** scripted phase transition flags, checkpoint-safe reset, deterministic death and loot credit. Story boss outcomes are separate from ordinary enemy respawn.
- **Test matrix:** target lost; player behind enemy; player leaves leash; player dies/disconnects; multi-player threat switch; heal threat; ranged LoS blocked; simultaneous death and reset; no auto-facing assistance.

## Story invariants for integration tests

- Q086 first living Kel'vari reveal occurs after permitted gate crossing.
- Q097 reveals Grey Surveyor identity, not before.
- Q143 first authenticated Nhal name.
- Q156 Khor'vun killed, Veil destroyed, Binding immediately dissolved, Seven freed, Shroud fade started; manifestation remains gradual.
- Q177 sabotage discovered after Thes'var dies at Q176; Q178 Oth'varen learns correction is fatal; Q181 he physically dies only after alignment holds, Nhal's identity shatters while essence survives.
- No optional quest required to unlock a main quest; no mandatory PvP, group or profession gate.

## Prototype scope

The obsolete JavaScript illustration and tests have been deleted. Implement and test the lifecycle in the existing Rust quest foundation; this document is a design contract, not an implementation proof.
