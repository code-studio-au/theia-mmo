# Theia MMO — Codex Dependency and Acceptance Register

**Status:** proposed engineering task decomposition. All entries default **NOT STARTED / NOT VERIFIED** until repository audit. The voice session approved the phase/wave structure and decisions but did not verify 112 individual implementation tasks. Dependency chains below are conservative initial proposals; Codex must refine per repository and identify safe parallelism. Missing P3-17/18 are intentionally blocked.

## How to execute
For each task: inspect existing code first; write a short design and tests; implement smallest vertical slice; run format/lint/test/build and relevant headless/client integration; capture benchmark evidence; update status only after verification. Preserve ADRs and avoid revisiting locked decisions without explicit approval.

## Phase 1

### P1-01 — Repository audit and reusable/obsolete classification
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** repository audit and architecture baseline.
- **Codex task:** Implement and document repository audit and reusable/obsolete classification using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for repository audit and reusable/obsolete classification; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-02 — Rust workspace, Bevy client, headless server and shared crates
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-01.
- **Codex task:** Implement and document rust workspace, bevy client, headless server and shared crates using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for rust workspace, bevy client, headless server and shared crates; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-03 — World coordinates and spatial conventions
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-02.
- **Codex task:** Implement and document world coordinates and spatial conventions using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for world coordinates and spatial conventions; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-04 — Entity identity and lifecycle
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-03.
- **Codex task:** Implement and document entity identity and lifecycle using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for entity identity and lifecycle; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-05 — 30Hz fixed timestep, bounded input queues
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-04.
- **Codex task:** Implement and document 30hz fixed timestep, bounded input queues using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for 30hz fixed timestep, bounded input queues; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-06 — Command schemas, sequencing, acknowledgements and loss tests
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-05.
- **Codex task:** Implement and document command schemas, sequencing, acknowledgements and loss tests using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for command schemas, sequencing, acknowledgements and loss tests; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-07 — Renet integration and validation (replaces comparative transport study)
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-06.
- **Codex task:** Implement and document renet integration and validation (replaces comparative transport study) using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for renet integration and validation (replaces comparative transport study); automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-08 — Replication protocol contracts
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-07.
- **Codex task:** Implement and document replication protocol contracts using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for replication protocol contracts; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-09 — Kinematic movement and collision contracts
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-08.
- **Codex task:** Implement and document kinematic movement and collision contracts using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for kinematic movement and collision contracts; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-10 — Asset/terrain/animation content contracts
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-09.
- **Codex task:** Implement and document asset/terrain/animation content contracts using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for asset/terrain/animation content contracts; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-11 — Data-driven ability/combat schemas
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-10.
- **Codex task:** Implement and document data-driven ability/combat schemas using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for data-driven ability/combat schemas; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-12 — Tests, instrumentation and benchmark baselines
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-11.
- **Codex task:** Implement and document tests, instrumentation and benchmark baselines using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for tests, instrumentation and benchmark baselines; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 2

### P2-01 — Bevy client bootstrap
- **Primary wave:** W2; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-05, P1-08, P1-09.
- **Codex task:** Implement and document bevy client bootstrap using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for bevy client bootstrap; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-02 — Connection/session management
- **Primary wave:** W2; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-01, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document connection/session management using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for connection/session management; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-03 — Entity replication and lifecycles
- **Primary wave:** W2; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-02, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document entity replication and lifecycles using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for entity replication and lifecycles; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-04 — Prediction and reconciliation
- **Primary wave:** W2; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-03, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document prediction and reconciliation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for prediction and reconciliation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-05 — Third-person camera/controls
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-04, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document third-person camera/controls using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for third-person camera/controls; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-06 — Terrain chunk loading
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-05, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document terrain chunk loading using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for terrain chunk loading; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-07 — Collision integration
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-06, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document collision integration using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for collision integration; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-08 — GLB and asset loading
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-07, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document glb and asset loading using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for glb and asset loading; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-09 — Character skeletal animation
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-08, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document character skeletal animation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for character skeletal animation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-10 — Lighting/materials/atmosphere
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-09, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document lighting/materials/atmosphere using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for lighting/materials/atmosphere; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-11 — Targeting and interactions
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-10, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document targeting and interactions using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for targeting and interactions; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-12 — One authoritative ability, damage, death and respawn
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-11, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document one authoritative ability, damage, death and respawn using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for one authoritative ability, damage, death and respawn; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-13 — Spell VFX and audio hooks
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-12, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document spell vfx and audio hooks using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for spell vfx and audio hooks; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-14 — Health/target/action-bar UI and integrated demo
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-13, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document health/target/action-bar ui and integrated demo using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for health/target/action-bar ui and integrated demo; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 3

### P3-01 — Authentication and sessions
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-12, P1-11.
- **Codex task:** Implement and document authentication and sessions using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for authentication and sessions; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-02 — PostgreSQL persistence/migrations
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-01, P2-12, P1-11.
- **Codex task:** Implement and document postgresql persistence/migrations using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for postgresql persistence/migrations; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-03 — Character creation/selection
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-02, P2-12, P1-11.
- **Codex task:** Implement and document character creation/selection using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for character creation/selection; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-04 — Stats and resources
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-03, P2-12, P1-11.
- **Codex task:** Implement and document stats and resources using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for stats and resources; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-05 — Inventory and items
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-04, P2-12, P1-11.
- **Codex task:** Implement and document inventory and items using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for inventory and items; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-06 — Abilities and status effects
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-05, P2-12, P1-11.
- **Codex task:** Implement and document abilities and status effects using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for abilities and status effects; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-07 — Creature AI
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-06, P2-12, P1-11.
- **Codex task:** Implement and document creature ai using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for creature ai; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-08 — Equipment and appearance
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-07, P2-12, P1-11.
- **Codex task:** Implement and document equipment and appearance using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for equipment and appearance; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-09 — Loot and rewards
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-08, P2-12, P1-11.
- **Codex task:** Implement and document loot and rewards using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for loot and rewards; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-10 — Experience and leveling
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-09, P2-12, P1-11.
- **Codex task:** Implement and document experience and leveling using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for experience and leveling; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-11 — Quest objectives
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-10, P2-12, P1-11.
- **Codex task:** Implement and document quest objectives using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for quest objectives; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-12 — NPC dialogue and vendors
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-11, P2-12, P1-11.
- **Codex task:** Implement and document npc dialogue and vendors using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for npc dialogue and vendors; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-13 — Terrain streaming and initial LOD
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-12, P2-12, P1-11.
- **Codex task:** Implement and document terrain streaming and initial lod using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for terrain streaming and initial lod; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-14 — Animation, VFX and audio expansion
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-13, P2-12, P1-11.
- **Codex task:** Implement and document animation, vfx and audio expansion using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for animation, vfx and audio expansion; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-15 — Content authoring and validation
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-14, P2-12, P1-11.
- **Codex task:** Implement and document content authoring and validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for content authoring and validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-16 — One complete persistent zone and login/logout proof
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-15, P2-12, P1-11.
- **Codex task:** Implement and document one complete persistent zone and login/logout proof using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for one complete persistent zone and login/logout proof; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-17 — UNSPECIFIED in original transcript: needs reconciliation
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-16, P2-12, P1-11.
- **Codex task:** Implement and document unspecified in original transcript: needs reconciliation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** BLOCKED: obtain missing original approved scope; do not implement guessed task.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-18 — UNSPECIFIED in original transcript: needs reconciliation
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-17, P2-12, P1-11.
- **Codex task:** Implement and document unspecified in original transcript: needs reconciliation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** BLOCKED: obtain missing original approved scope; do not implement guessed task.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 4

### P4-01 — World region ownership
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-16, P1-03.
- **Codex task:** Implement and document world region ownership using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for world region ownership; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-02 — Predictive terrain/object streaming
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-01, P3-16, P1-03.
- **Codex task:** Implement and document predictive terrain/object streaming using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for predictive terrain/object streaming; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-03 — Streamed navigation and spatial queries
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-02, P3-16, P1-03.
- **Codex task:** Implement and document streamed navigation and spatial queries using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for streamed navigation and spatial queries; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-04 — Advanced interest management
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-03, P3-16, P1-03.
- **Codex task:** Implement and document advanced interest management using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for advanced interest management; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-05 — Adaptive replication
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-04, P3-16, P1-03.
- **Codex task:** Implement and document adaptive replication using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for adaptive replication; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-06 — Cross-region handoff
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-05, P3-16, P1-03.
- **Codex task:** Implement and document cross-region handoff using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for cross-region handoff; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-07 — Population and overload management
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-06, P3-16, P1-03.
- **Codex task:** Implement and document population and overload management using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for population and overload management; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-08 — Advanced combat interrupts/CC/AoE/healing/threat
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-07, P3-16, P1-03.
- **Codex task:** Implement and document advanced combat interrupts/cc/aoe/healing/threat using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for advanced combat interrupts/cc/aoe/healing/threat; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-09 — Classes and specializations
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-08, P3-16, P1-03.
- **Codex task:** Implement and document classes and specializations using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for classes and specializations; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-10 — Group encounters
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-09, P3-16, P1-03.
- **Codex task:** Implement and document group encounters using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for group encounters; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-11 — Parties and group rules
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-10, P3-16, P1-03.
- **Codex task:** Implement and document parties and group rules using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for parties and group rules; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-12 — Chat/moderation
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-11, P3-16, P1-03.
- **Codex task:** Implement and document chat/moderation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for chat/moderation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-13 — Friends and presence
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-12, P3-16, P1-03.
- **Codex task:** Implement and document friends and presence using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for friends and presence; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-14 — Guild persistence
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-13, P3-16, P1-03.
- **Codex task:** Implement and document guild persistence using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for guild persistence; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-15 — Secure trading
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-14, P3-16, P1-03.
- **Codex task:** Implement and document secure trading using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for secure trading; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-16 — Animation LOD and crowd policies
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-15, P3-16, P1-03.
- **Codex task:** Implement and document animation lod and crowd policies using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for animation lod and crowd policies; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-17 — VFX density budgets
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-16, P3-16, P1-03.
- **Codex task:** Implement and document vfx density budgets using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for vfx density budgets; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-18 — Time/weather integration
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-17, P3-16, P1-03.
- **Codex task:** Implement and document time/weather integration using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for time/weather integration; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-19 — Multi-zone scale profiling
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-18, P3-16, P1-03.
- **Codex task:** Implement and document multi-zone scale profiling using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for multi-zone scale profiling; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-20 — World and multiplayer integration gate
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-19, P3-16, P1-03.
- **Codex task:** Implement and document world and multiplayer integration gate using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for world and multiplayer integration gate; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 5

### P5-01 — Rendering benchmarks
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-14, P4-07.
- **Codex task:** Implement and document rendering benchmarks using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for rendering benchmarks; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-02 — Lighting and shadows
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-01, P2-14, P4-07.
- **Codex task:** Implement and document lighting and shadows using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for lighting and shadows; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-03 — Atmosphere and fog
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-02, P2-14, P4-07.
- **Codex task:** Implement and document atmosphere and fog using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for atmosphere and fog; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-04 — Stylized materials
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-03, P2-14, P4-07.
- **Codex task:** Implement and document stylized materials using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for stylized materials; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-05 — Water
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-04, P2-14, P4-07.
- **Codex task:** Implement and document water using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for water; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-06 — Terrain LOD and seams
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-05, P2-14, P4-07.
- **Codex task:** Implement and document terrain lod and seams using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for terrain lod and seams; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-07 — Streaming memory budgets
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-06, P2-14, P4-07.
- **Codex task:** Implement and document streaming memory budgets using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for streaming memory budgets; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-08 — Vegetation instancing and wind
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-07, P2-14, P4-07.
- **Codex task:** Implement and document vegetation instancing and wind using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for vegetation instancing and wind; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-09 — Visibility/culling
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-08, P2-14, P4-07.
- **Codex task:** Implement and document visibility/culling using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for visibility/culling; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-10 — Animation blending and layers
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-09, P2-14, P4-07.
- **Codex task:** Implement and document animation blending and layers using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for animation blending and layers; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-11 — Equipment visuals
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-10, P2-14, P4-07.
- **Codex task:** Implement and document equipment visuals using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for equipment visuals; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-12 — Animation LOD
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-11, P2-14, P4-07.
- **Codex task:** Implement and document animation lod using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for animation lod; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-13 — Character readability
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-12, P2-14, P4-07.
- **Codex task:** Implement and document character readability using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for character readability; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-14 — Particle expansion
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-13, P2-14, P4-07.
- **Codex task:** Implement and document particle expansion using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for particle expansion; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-15 — Projectiles/AoE effects
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-14, P2-14, P4-07.
- **Codex task:** Implement and document projectiles/aoe effects using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for projectiles/aoe effects; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-16 — VFX priority and density limits
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-15, P2-14, P4-07.
- **Codex task:** Implement and document vfx priority and density limits using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for vfx priority and density limits; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-17 — World editor
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-16, P2-14, P4-07.
- **Codex task:** Implement and document world editor using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for world editor; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-18 — Creature/item/ability/quest editors
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-17, P2-14, P4-07.
- **Codex task:** Implement and document creature/item/ability/quest editors using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for creature/item/ability/quest editors; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-19 — Asset conversion and license validation
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-18, P2-14, P4-07.
- **Codex task:** Implement and document asset conversion and license validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for asset conversion and license validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-20 — Developer inspection tools
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-19, P2-14, P4-07.
- **Codex task:** Implement and document developer inspection tools using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for developer inspection tools; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-21 — Client performance tuning
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-20, P2-14, P4-07.
- **Codex task:** Implement and document client performance tuning using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for client performance tuning; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-22 — Server profiling
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-21, P2-14, P4-07.
- **Codex task:** Implement and document server profiling using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for server profiling; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-23 — Deployment/monitoring/recovery
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-22, P2-14, P4-07.
- **Codex task:** Implement and document deployment/monitoring/recovery using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for deployment/monitoring/recovery; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-24 — Integrated visual and load validation
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-23, P2-14, P4-07.
- **Codex task:** Implement and document integrated visual and load validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for integrated visual and load validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 6

### P6-01 — Release scope and quality gates
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-16, P5-24.
- **Codex task:** Implement and document release scope and quality gates using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for release scope and quality gates; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-02 — Minimum playable content
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-01, P3-16, P5-24.
- **Codex task:** Implement and document minimum playable content using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for minimum playable content; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-03 — Internal playtests/defect tracking
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-02, P3-16, P5-24.
- **Codex task:** Implement and document internal playtests/defect tracking using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for internal playtests/defect tracking; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-04 — Class/combat balance
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-03, P3-16, P5-24.
- **Codex task:** Implement and document class/combat balance using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for class/combat balance; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-05 — Progression and economy validation
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-04, P3-16, P5-24.
- **Codex task:** Implement and document progression and economy validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for progression and economy validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-06 — Quest/encounter chains
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-05, P3-16, P5-24.
- **Codex task:** Implement and document quest/encounter chains using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for quest/encounter chains; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-07 — Onboarding/accessibility
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-06, P3-16, P5-24.
- **Codex task:** Implement and document onboarding/accessibility using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for onboarding/accessibility; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-08 — Closed alpha accounts/environment
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-07, P3-16, P5-24.
- **Codex task:** Implement and document closed alpha accounts/environment using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for closed alpha accounts/environment; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-09 — Installer/patcher
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-08, P3-16, P5-24.
- **Codex task:** Implement and document installer/patcher using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for installer/patcher; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-10 — Closed alpha stabilization
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-09, P3-16, P5-24.
- **Codex task:** Implement and document closed alpha stabilization using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for closed alpha stabilization; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-11 — Concurrent gameplay load test
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-10, P3-16, P5-24.
- **Codex task:** Implement and document concurrent gameplay load test using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for concurrent gameplay load test; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-12 — Crowded encounter test
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-11, P3-16, P5-24.
- **Codex task:** Implement and document crowded encounter test using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for crowded encounter test; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-13 — Recovery drills
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-12, P3-16, P5-24.
- **Codex task:** Implement and document recovery drills using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for recovery drills; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-14 — Security/exploit testing
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-13, P3-16, P5-24.
- **Codex task:** Implement and document security/exploit testing using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for security/exploit testing; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-15 — Closed beta population
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-14, P3-16, P5-24.
- **Codex task:** Implement and document closed beta population using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for closed beta population; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-16 — Long soak testing
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-15, P3-16, P5-24.
- **Codex task:** Implement and document long soak testing using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for long soak testing; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-17 — Production deployment/support
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-16, P3-16, P5-24.
- **Codex task:** Implement and document production deployment/support using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for production deployment/support; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-18 — Moderation/support workflows
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-17, P3-16, P5-24.
- **Codex task:** Implement and document moderation/support workflows using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for moderation/support workflows; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-19 — Release candidate validation
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-18, P3-16, P5-24.
- **Codex task:** Implement and document release candidate validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for release candidate validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-20 — Operational readiness review
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-19, P3-16, P5-24.
- **Codex task:** Implement and document operational readiness review using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for operational readiness review; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-21 — Go/no-go
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-20, P3-16, P5-24.
- **Codex task:** Implement and document go/no-go using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for go/no-go; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-22 — Final load test
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-21, P3-16, P5-24.
- **Codex task:** Implement and document final load test using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for final load test; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-23 — Launch
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-22, P3-16, P5-24.
- **Codex task:** Implement and document launch using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for launch; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-24 — Launch stabilization
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-23, P3-16, P5-24.
- **Codex task:** Implement and document launch stabilization using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for launch stabilization; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## High-priority cross-cutting acceptance suites
1. Fixed 30Hz scheduling under load; bounded queues, ordering, 20/30/60Hz behaviour.
2. Renet encrypted connection, auth ticket expiry/replay, loss/reorder/reconnect, bandwidth budgets.
3. f64 global/f32 local precision, origin shift, seam collision, long-distance flying.
4. Avian kinematic slopes/jumps/swim/fly and scripted elevator boarding.
5. Chunk streaming and LOD cracks, predictive flight loading, memory pressure, visual/collision separation.
6. Character run-and-cast layering, mounts, equipment, animation LOD.
7. Boss critical telegraphs and audio retained at lowest quality under saturated effects.
8. Durable economic transactions, idempotent retries, crash injection and backup restores.
9. Spatial interest correctness, delta baseline recovery, 200+ crowded-scene benchmark (target workload, not capacity claim).
10. Multi-region ownership uniqueness, cross-boundary interactions and future handoff contract.

## Known reconciliation issues
- Phase 3 was called 18 stages but only 16 stage descriptions were spoken; P3-17/18 are unknown.
- Phase 4–6 descriptions were thematic, so per-stage numbering here is newly proposed.
- Earlier shorthand mapped Phase 3 inventory to stage 9, but detailed enumeration puts inventory at P3-05 and loot at P3-09. Detailed enumeration takes precedence pending review.
- A spoken total of 88 for phases 1–5 conflicts with 12+14+18+20+24=88 (correct), plus 24 =112.
- Original 'M1-04' scheduler task ID differs from this document's P1-05 due to differing milestone schemes. Resolve before creating issue IDs.
