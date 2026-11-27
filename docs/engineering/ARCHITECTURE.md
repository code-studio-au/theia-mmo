# Theia MMO — Approved Architecture

**Initial controls/combat amendment, 1 November 2026:**
[THEIA-COMBAT-001 v1.0](../gameplay/COMBAT_TARGETING_MOVEMENT_SPEC.md) supersedes the
GW2/hybrid control and facing direction in this reconstructed baseline. Use
strict WoW Classic-style movement/camera, persistent targets, manual facing and
weapon-timed autoattack; movement restrictions are ability-specific. Dodge and
action camera are deferred. Engine/network/world architecture remains adopted.

Reconstructed from the October 19, 2026 planning session. **Approved in conversation does not mean implemented in GitHub.** The original handoff named `theia-engine`; this architecture is adopted for the independent `theia-mmo` repository. Rust + Bevy client, headless authoritative Rust server, modular monolith initially. WoW Classic-style MMO with manual facing and character-relative movement. Production assets must be original or appropriately licensed; converting WoW assets to glTF does not grant distribution rights.

## NET-001

Renet initial UDP networking; secure authenticated transport; Quinn fallback only if real validation fails. Bounded queues, latency/bandwidth/loss benchmarks.

## SIM-001

Authoritative configurable 30 Hz fixed-timestep server; benchmark 20/30/60 Hz; client 60+ FPS independent; simulation-time cooldowns.

## WORLD-001

Global f64 coordinates server/client; local f32 rendering/physics frames; Y-up, 1 unit=1m; integer region/chunk IDs; centralized floating origin; seam tests.

## PHYS-001

Server-authoritative kinematic characters, predominantly static collision, scripted elevators/moving platforms, client prediction and reconciliation.

## PHYS-002

Avian initial Bevy physics choice; Rapier fallback; no mandatory upfront comparison; keep movement logic portable.

## ASSET-001

Source formats including Blender/glTF and authorized WoW converters → offline conversion/validation → runtime content. GLB for characters initially; dedicated terrain/collision formats; stable IDs, versioned manifests. Original/licensed production assets.

## TERRAIN-001

Heightmap terrain plus meshes for caves/cliffs; configurable build-time chunks initially 128m square; hierarchical screen-error LOD, seams, async predictive streaming, long vistas and flying.

## ENV-001

Hybrid deterministic procedural and handcrafted vegetation, offline generation with persistent overrides, instancing/culling/LOD. Client GPU wind and player/creature/mount-driven bending; server collision only where gameplay-relevant.

## RENDER-001

Bevy/wgpu, modernized stylized WoW aesthetic, stylized PBR, lighting/shadows, atmosphere, water, postprocessing, scalable quality. Extend Bevy, do not restart Vulkan renderer.

## ANIM-001

Expressive character animation with manual-facing combat: skeletal graphs, blending/layers, cast and attack while moving where allowed, sockets, mounts, facial foundation, animation LOD.

## VFX-001

Data-driven GPU-capable particles, trails, decals, audio; authoritative event timing; priority budgets. Critical boss telegraphs never dropped by cosmetic throttling, dedicated indicator rendering and accessibility tests.

## AUDIO-001

Spatial audio, memorable fantasy music, adaptive ambience, event-driven cues, voice limits; boss-critical audio priority, subtitles and visual alternatives.

## UI-001

Modular Bevy UI, action bars, quests, inventory, maps, chat, raid frames, scalable customization; sandboxed addons later, not initial release.

## GAME-001

Server-authoritative data-driven tab-target combat, persistent explicit targets, manual facing and weapon-timed autoattacks separate from abilities. Casting movement and interrupt restrictions belong to ability data. Follow THEIA-COMBAT-001 and the Classic formula evidence policy; historical hybrid timings are not universal gameplay defaults.

## AI-001

Server-only data-driven behaviour trees/state machines, simple world mobs, advanced dungeon/boss AI, threat, activity-based scheduling and boss-priority guarantees.

## PERSIST-001

PostgreSQL, async DB, never block 30Hz tick; durable atomic critical item/currency/trade/reward commits before acknowledgement; ordinary periodic checkpoints, idempotency and recovery.

## NET-002

Stable network IDs, spatial interest with relevance overrides, adaptive replication/deltas/quantization, reliable events and unreliable movement, prediction/reconciliation, bounded bandwidth. Crowded encounters first-class.

## WORLD-002

Logical region-based authoritative modular monolith; one owner per entity, seamless crossings, idle-region scheduling, future process handoffs; no distributed single-battle simulation initially.

## WORLD-003

Precomputed tiled navmeshes, hierarchical async bounded paths, elevator links, swim/fly modes; detailed local creature paths vs scheduled long-distance NPC routes.

## WORLD-004

Server world clock with configurable global day length and instance overrides; regional biome weather, smooth transitions, client visuals/audio, optional gameplay hooks; six-hour day only proposed playtest value.

## SEC-001

Established external OIDC identity provider (Keycloak for local development, Cognito for hosted testers); internal session/character authorization; short-lived single-use tickets, encrypted Renet transport, one character session, rate limits, replay defense, audits.

## Still open

Production operational hardening, MMO-scale budgets and capacity, broader platform qualification and completion of the planned gameplay systems. P3-17/18 remain unspecified. Adopted hosting requirements are in [hosting and operations](HOSTING_AND_OPERATIONS_REQUIREMENTS.md); implementation evidence is in [status](../STATUS.md).

Current dependency pins live in [Cargo.toml](../../Cargo.toml) and [Cargo.lock](../../Cargo.lock); the saved terrain format is documented in [saved terrain](../world/SAVED_TERRAIN.md). The [historical baseline](../archive/engineering/ARCHITECTURE_BASELINE.md) retains the original handoff and superseded wording.
