# Repository assessment against the October 19 handoff

This is the **pre-alignment audit snapshot**. The subsequent user instruction to
align adopts the target architecture, resolves the environment-motion policy, and
starts implementation. See [current alignment](ALIGNMENT.md) and
[live task status](TASK_STATUS.md) for changes and fresh results after this audit.

Assessment date: 2026-10-19, Australia/Sydney. Scope: the working tree of
`/home/steven/dev/theia-mmo`, including pre-existing uncommitted prototype work.
This is a local source/documentation audit, not verification of the GitHub repo
named in the attachment. No implementation migration, commit or push was made.

## Overall finding

The repository is a useful Rust/Bevy local multiplayer walking prototype. It is
not the full Phase 1 architecture, a completed Phase 2 game, or an MMO backend.
Keep its tested presentation, import tooling and authority regressions. Replace
the narrow foundations through bounded migrations. Continuing bespoke house
fixtures would deepen the wrong abstraction for the supplied plan.

Recommendation: retain this repository and perform an architectural reset within
it. The handoff still targets Rust, Bevy and a headless authoritative server, so
there is no engine/product boundary that demands a new repository. A fresh repo
would require carrying over or rebuilding the same useful tools and tests, and
could obscure their provenance. Cleanliness comes from explicit module contracts,
removing superseded paths after acceptance, and an accurate task/status record.
Do not delete or relocate uncommitted experiments to obtain a cosmetically clean
tree. Any later fresh-repository decision needs a concrete scope and preservation
plan; this review does not create one or alter Git history.

The strongest existing evidence is two-client server-owned movement, ordinary
Bevy rendering/GLB animation, and repeatable local terrain/building/camera checks.
Major gaps are Renet, f64 global/local frames, Avian/general collision, prediction,
data-driven gameplay, streamed terrain/navigation, persistence and production
security/operations. The handoff is a useful target structure, but its task
register needs more specific acceptance criteria and a real dependency graph.

## Retain, migrate and retire

| Source | Classification | Action |
|---|---|---|
| `Cargo.toml`, `apps/client`, `apps/server` | Retain workspace/application foundation | Rust 2024, Rust 1.99 and Bevy 0.20 are currently pinned; verify target plugin compatibility before migration. Server currently has no Bevy renderer dependency. |
| `apps/client/src/{asset_preview,capture_ready,preview_animation}.rs` | Retain tooling | Standard Bevy GLB/animation inspection and render-readiness captures are useful independently of gameplay/network changes. |
| `apps/client/src/{camera,movement,scene,reference_scene,terrain_scene}.rs` | Retain selectively; adapt integration | Keep controls, static-camera queries and movement-driven animation; separate fixture-specific setup and reconcile with new coordinate/controller contracts. |
| `crates/network/src/simulation.rs` | Retain authority behaviour; refactor domain boundary | Server owns intents, speed, collision, lifecycle and tick count. Simulation currently lives in the transport crate; move shared contracts only when the slice needs them. |
| `crates/network/src/server.rs` | Retain scheduler/queue pattern; qualify | Fixed 30 Hz, latest-input slots, bounded joins and admission. Needs configurable rates, load metrics and an explicit overload/time policy. |
| `crates/network/src/{client,server,tls,wire}.rs` | Migrate transport/protocol | Existing Quinn local baseline is useful. Renet, authentication tickets, broader replication, precision and prediction require new contracts and validation. |
| `crates/network/src/{terrain,world}.rs` | Retain fixture regression; replace general movement path | Conservative boxes and highest-surface sampling cannot provide arbitrary interiors, stacked floors, jumping or full kinematic movement. |
| `crates/network/tests/loopback.rs` | Retain behavioural tests | Port tests for authority, stale input, expiry, malformed messages, lifecycle and terrain consistency to the target transport; add adverse-network cases. |
| `tools/{convert_terrain,assemble_map,merge_static_glb,map_placement,align_glb}.py` and related tools | Retain offline experiments | Useful conversion/placement/validation logic; add target content IDs, manifests and terrain/collision contracts before treating them as production pipeline. |
| `tools/{open_map_farmhouse,open_map_blacksmith,prepare_enterable_house}.py` | Freeze as fixtures | Preserve measured source layout and regression routes; stop expanding this into the general building-access solution. Retire runtime reliance after general collision passes equivalent checks. |
| `tools/{play,verify_multiplayer,verify_asset,verify_smoke}.py` | Retain developer entry points | Adapt to replacement slices and preserve deterministic qualification/cleanup. |
| `build/development-world*`, supplied art | Local compatibility fixtures | Ignored outputs and temporary art do not establish reproducible production content or distribution rights. Do not add them to this documentation import. |
| Existing `docs/*` | Retain evidence/history; update planning entry points | Historical implementation claims stay dated. New target architecture is linked through the engineering index. |

## Architectural alignment

“Partial” means relevant code exists; it does not mean the target contract or its
acceptance suite is complete. “Missing” means no implementation was found in the
reviewed workspace. These are source-based findings, with fresh execution limited
to the verification section below.

| Handoff decision | Finding | Evidence and gap |
|---|---|---|
| NET-001 | Divergent | Quinn QUIC is pinned in `Cargo.toml`; no Renet dependency. TLS trust is local certificate pinning, not player authentication. Renet migration is planned. |
| SIM-001 | Partial | `server.rs` drives `World::step()` at constant 30 Hz; `simulation.rs` owns movement and input expiry. No configurable 20/30/60 comparison, cooldown system or loaded tick benchmark. |
| WORLD-001 | Missing global/local model | `wire::Player`, terrain profiles and movement use f32; source origin is also f32. Y-up/XZ movement exists. No centralized floating origin, region/chunk position type or seam precision tests. |
| PHYS-001 | Partial/divergent | Server-owned grounded movement exists, but no general kinematic controller, jump/fall, prediction/reconciliation or moving supports. Platforms conflict with current static-world constraints. |
| PHYS-002 | Missing | No Avian or Rapier dependency. Existing custom box/surface movement is a fixture implementation. Compatible versions are unverified. |
| ASSET-001 | Partial | Standard GLB loading, offline conversion and profile/visual hashes exist. Stable content IDs, versioned manifests and dedicated terrain/collision formats remain unspecified. Character source validation errors remain documented. |
| TERRAIN-001 | Partial fixture only | Full supplied tile renders and supports bounded outdoor walking. One assembled GLB/profile is not chunk streaming. No 128 m chunk contract, heightmap runtime, cave collision, hierarchical LOD or flight. |
| ENV-001 | Partial presentation only | Imported static props exist. No deterministic vegetation generation/overrides, wind/bending or measured vegetation policies. Environment animation conflicts need scope resolution. |
| RENDER-001 | Aligned foundation | Standard Bevy PBR, lights, shadows and static liquid preview exist. Scalable atmosphere/water/postprocessing and measured quality budgets are incomplete. |
| ANIM-001 | Partial | Stand/walk/run and inspector clip playback exist in `reference_scene.rs` and preview tools. No run-and-cast layers, equipment sockets, mounts, facial system or animation LOD. |
| VFX-001 | Missing | No gameplay event-driven effects system, boss telegraphs or priority budgets. |
| AUDIO-001 | Missing | Current Bevy features omit audio. No spatial audio, music system or critical-cue priority. |
| UI-001 | Partial bootstrap only | Standard Bevy connection/control HUD exists. No target/action bars, inventory, quests, raid frames or customization. |
| GAME-001 | Missing | No server ability schema, targeting, damage, health, GCD, ability queue or movement/casting rules. |
| AI-001 | Missing | The rendered wolf is a visual placeholder; no server-owned NPC lifecycle, AI, threat or scheduling. |
| PERSIST-001 | Missing | No PostgreSQL dependency, migrations, durable characters or atomic economic operations. |
| NET-002 | Partial | Increasing session-local player IDs, full snapshots, applied input sequence and remote interpolation exist. No general entity lifecycle, spatial interest, deltas, reliable gameplay events or prediction. |
| WORLD-002 | Partial authority only | One `World` owner exists in one process. No logical regions, cross-region interactions, idle-region scheduling or handoff. |
| WORLD-003 | Missing | No navmesh bake/runtime, path service, swim/fly modes or navigation links. |
| WORLD-004 | Missing | No authoritative world clock/day cycle, biome weather or instance overrides. |
| SEC-001 | Partial transport protection only | Verified TLS connection, bounded admission and input-rate limits exist. No external identity provider, user authorization, one-character-session rules or single-use tickets. Packet sequence rejection is not ticket replay protection. |
| OPS-001 / open choices | Unresolved | Provider, target dependency pins, quantified budgets, terrain layout and operational architecture still need decisions. Current Linux evidence does not qualify Windows. |

## Task coverage: Phase 1

No phase completion percentage is claimed. The imported register's requirements
are broader than existing experiments, and some are not yet measurable.

| Task | Audit status | Present evidence and remaining acceptance |
|---|---|---|
| P1-01 | Scoped documentation audit delivered | This assessment inventories local code, classifies reuse and records checks. The source entry's generic automated integration/benchmark acceptance is not certified by a prose audit. |
| P1-02 | Partial | Three-member Rust workspace, Bevy client, headless server and shared network crate exist. Target domain/module contracts and compatible Renet/Avian pins remain. |
| P1-03 | Requires migration | Y-up and existing bounded f32 movement exist; f64 global/local types, floating origin and seam tests are missing. |
| P1-04 | Partial | Server assigns non-reused player IDs within a process, handles disconnect/reconnect; no persistent/general entity identities or session-epoch contract. |
| P1-05 | Partial, next qualification | Constant 30 Hz and bounded watch/mpsc channels exist. Add explicit scheduling contract, configurability, overload tests and measured timings. |
| P1-06 | Partial | Versioned bounded intents, monotonic sequences, applied-sequence acknowledgements, expiry and stale-input tests exist. No action-command schema, resend/history policy or controlled loss/delay qualification. |
| P1-07 | Missing target implementation | Quinn is not Renet acceptance. Validate target encrypted/authenticated connection and benchmark it; retain Quinn as baseline. |
| P1-08 | Partial | Wire v3, 1,200-byte budget, bounded sorted full snapshots and world identity exist. Document target channels, reliable events, precision, authority and lifecycle semantics. |
| P1-09 | Partial fixture only | Authoritative box/surface movement, slopes, wall sliding and character pass-through tests exist. General collision/controller contract and Avian proof are absent. |
| P1-10 | Partial tooling | GLB qualification and hash-bound terrain profiles exist. Need stable IDs, manifests, skeleton/animation contracts and runtime terrain/collision definitions. |
| P1-11 | Missing | No data-driven ability/combat schemas. |
| P1-12 | Partial | Unit/loopback/asset tests and native smoke harnesses exist. No agreed simulation/replication/render budgets or reproducible load/latency baseline. |

## Task coverage: Phase 2

| Task | Audit status | Present evidence and remaining acceptance |
|---|---|---|
| P2-01 | Existing bootstrap, target integration pending | Runnable standard Bevy scene and inspector. Native GPU evidence is historical; current full workspace check is environment-blocked. |
| P2-02 | Partial | Loopback connect, disconnect, certificate trust and relaunch work. No Renet sessions, player authentication or session resume. |
| P2-03 | Partial | Two-client player snapshots/spawn/despawn and remote interpolation; no general entities, interest or target replication contract. |
| P2-04 | Missing | Local pawn follows server poses directly; no prediction, saved input history or reconciliation. |
| P2-05 | Existing prototype, target integration pending | Orbit/zoom, camera-relative controls, body hiding and static mesh obstruction. Requalify with origin shifts/general movement and adverse networks. |
| P2-06 | Missing chunk system | Full tile is a single prepared world. No chunk lifetime, async streaming or memory-pressure policy. |
| P2-07 | Partial fixture only | Terrain samples and conservative solids work on qualified routes; no target general physics integration. |
| P2-08 | Existing asset slice, contracts incomplete | Bevy GLB imports/inspector and textures work; asset IDs/manifests and clean target character validation remain. |
| P2-09 | Partial | Basic human locomotion and sampled clip evidence; no combat layers/blending or target skeleton contract. |
| P2-10 | Partial | PBR lighting/shadows, ambient fill and static translucent water; no complete atmosphere/quality implementation. |
| P2-11 | Missing | No server target/interaction loop. |
| P2-12 | Missing | No ability, damage, death or respawn loop. |
| P2-13 | Missing | No spell VFX/audio hooks or authoritative effect events. |
| P2-14 | Missing gameplay demo | Bootstrap HUD is not health/target/action UI or a two-player enemy encounter. |

## Task coverage: Phases 3–6

These groups account for every remaining task ID. Existing tools/presentation are
precursors only; none of these later delivery gates is verified complete.

| Tasks | Finding |
|---|---|
| P3-01..P3-12 | Missing gameplay/persistence systems: accounts, PostgreSQL, characters, stats, inventory, effects, AI, equipment, loot, XP, quests and NPC dialogue/vendors. |
| P3-13 | Missing runtime terrain streaming and initial LOD. |
| P3-14 | Basic skeletal playback is a precursor; expanded animation/VFX/audio is unimplemented. |
| P3-15 | Python conversion/validation tools are reusable precursors; target content authoring/validation acceptance is unverified. |
| P3-16 | No persistent zone or login/logout persistence proof. |
| P3-17..P3-18 | Blocked by unspecified source scope; retain placeholders. |
| P4-01..P4-20 | No region ownership/streaming/navmesh, advanced replication, handoff, overload policies, advanced combat/classes/groups/social/economy, animation/VFX budgets, weather or multi-zone integration gate. Current single-world authority/interpolation does not satisfy these. |
| P5-01..P5-24 | Rendering, asset tools, inspector and captures are useful precursors. No verified completion of rendering benchmarks, atmosphere/water/LOD/vegetation policies, animation/equipment/VFX systems, editors, performance budgets or deployment/integrated load gates. |
| P6-01..P6-24 | Release gates are unstarted/unverified: no persistent content, alpha/beta, installer/patcher, economic balance, crowded gameplay load/soak, recovery/security qualification or launch operations. |

W1 has bootstrap/authority precursors but missing architecture contracts. W2 has
two-client movement on the legacy transport, without Renet or prediction. W3's
two-player enemy fight is absent. W4–W10 delivery outcomes are not demonstrated.

## Planning issues to resolve

1. **Repository identity:** the supplied architecture names `theia-engine`; this
   audit targets `theia-mmo`. Retain repository independence and current provenance.
2. **World motion:** PHYS-001/WORLD-003 include moving platforms/elevator links;
   ENV-001 includes wind/bending. Current explicit static-environment constraints
   conflict with these features. Record a scope decision before implementing them;
   character animation and static collision/navigation remain independent work.
3. **Transport:** Renet is the handoff target; Quinn is current code. A migration
   must preserve authority/security tests and cannot cite existing Quinn success
   as proof that Renet failed. Establish the exact secure transport configuration
   and compatible dependency versions during the bounded validation slice.
4. **Dependencies:** P1-01 depends on an audit baseline, effectively itself if
   interpreted literally. P1 entries then form a list-order chain; e.g. baseline
   instrumentation need not wait for ability schemas. Camera/GLB loading do not
   intrinsically require completed prediction/collision. Review actual prerequisites
   before scheduling; the supplied plan explicitly allows overlapping workstreams.
5. **Acceptance quality:** almost every task repeats the same implementation,
   test, metric and integration sentence. That cannot define success for audits,
   balance, release decisions or concrete systems. Specify observable behaviour,
   workload, limits and failure handling per slice. Do not add artificial tests
   solely to satisfy boilerplate around a documentation audit.
6. **Missing stages:** P3-17/18 stay blocked, but do not block unrelated known work.
   Phase 4–6 numbers were reconstructed. Inventory is P3-05, loot P3-09; scheduler
   is P1-05, not M1-04. Avoid creating duplicate issue identifiers.
7. **Counts:** 12+14+18+20+24+24 = 112 register entries, including the two unknown
   placeholders. The source's last “spoken total of 88 ... conflicts” bullet is
   misleading: 88 for phases 1–5 is arithmetically consistent. The uncertainty is
   the reconstructed/missing scope, not that sum.
8. **Budgets and external choices:** 128 m chunks, a 200+ crowded-scene workload,
   tick comparisons, security, identity and operations need measurable contracts.
   They are targets, not tested capacity. No dependency compatibility or provider
   recommendation was externally verified in this local audit.

## Verification executed for this assessment

No runtime code, assets or renderer configuration changed. Native GPU checks were
not rerun; previous RX 7900 XTX/RADV evidence remains historical in
[VALIDATION](../VALIDATION.md). Windows execution remains unqualified.

| Command/check | Fresh result |
|---|---|
| `cargo fmt --check` | Passed. |
| `cargo test --locked -p theia-network -p theia-server` | Passed: 16 network unit tests and 7 actual QUIC loopback integration tests; server/doc-test targets contain no tests. |
| `cargo clippy --locked -p theia-network -p theia-server --all-targets -- -D warnings` | Passed. |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | Blocked during dependency build: `wayland-sys` cannot find `wayland-client.pc` through pkg-config. No full workspace lint result claimed. |
| `cargo test --locked --workspace` | Blocked by the same missing Wayland development metadata. No fresh client test result claimed. |
| `python3 -m unittest discover -s tools/asset_tests -v` | Two merge tests passed; nine modules failed import because this Python environment lacks `numpy`/`trimesh`. No full asset-suite pass claimed; requirements are in `tools/requirements-assets.txt`. |
| Archive import integrity, task count and local Markdown links | Checked after integration; supplied contents preserved, 112 unique task headings, local document links resolve. |

Earlier STATUS reports of 42 Rust and 35 asset tests describe previous qualified
runs. They are not rerun results for 2026-10-19. This audit did not install system
packages or change the source/toolchain to conceal an environment limitation.

The next work is the bounded foundation queue in the [engineering index](README.md).
