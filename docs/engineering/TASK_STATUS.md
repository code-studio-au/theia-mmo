Current bounded slice, 2026-10-22 — P1-03/09/10/12, P2-05/06/07/08/09:
[continental graybox adoption](../GRAYBOX_WORLD.md) passes headless and native
Linux checks. Default launcher selects the 256 km² procedural world and POLYGON
player; bounded visual/collision preparation, GPU readiness, culling/retirement,
full native crossing, distant teleports/revisit and local/remote presentation
smoothing are implemented. Release predicted single-client 720p crossing: 821.70 FPS;
this is an empty-world baseline, not MMO population capacity. Whole Pn tasks and
broader P2-04 qualification remain open. Forest work is frozen. Impaired-UDP
traversal/readiness and the input-tick/kinematic-state prerequisites now pass.
The actual Renet authority/client now execute completed-step commands and bounded
reconciliation. Native impaired-UDP, held-jump/teleport and full-continent checks
pass; the graybox launcher selects prediction with a retained `--no-prediction`
baseline. This closes the bounded Linux slice; broader P2-04 qualification remains. [Contract and limits](../PREDICTION_CONTRACT.md).

# Live engineering task status

Updated 2026-10-22. This is the active status overlay for all 112 imported task
entries. The supplied register stays unchanged for provenance. “Partial” and
“Precursor only” do not satisfy the original task's acceptance criteria. No phase
or wave is complete. Two source tasks remain unspecified.

Use [ALIGNMENT](ALIGNMENT.md) for implementation evidence and remaining gates,
[the index](README.md) for the immediate queue, and the imported
[plan](IMPLEMENTATION_PLAN.md) for phase/wave scope. Refine actual dependencies and
specific behavioural/failure/performance acceptance before implementing each task;
list order is not a dependency graph. Follow [execution order](EXECUTION_ORDER.md). Scheduler work is P1-05, inventory P3-05,
and loot P3-09. Do not invent scope for P3-17/18.

## Phase 1

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P1-01 | Repository audit and reusable/obsolete classification | Audit delivered | Pre-alignment inventory is recorded; generic source acceptance is narrowed to documentation audit. |
| P1-02 | Rust workspace, Bevy client, headless server and shared crates | Partial | Independent workspace and core crate exist; remaining domain boundaries and later gameplay integration are open. |
| P1-03 | World coordinates and spatial conventions | Partial | Live checked global authority and local projection/wire rebasing; chunk/region ID contracts tested. Normal chunk startup supports independent client query frames and guarded common wire/render frames for distant loaded actors. General query-scene rebasing and region ownership remain. [Scope](../CHUNK_STARTUP.md). |
| P1-04 | Entity identity and lifecycle | Partial | Checked incarnation/entity keys and monotonic actor lifecycle active; restart/stale-epoch rejection tested. General entities and persistent identity remain. |
| P1-05 | 30Hz fixed timestep, bounded input queues | Partial | Bounded eight-peer graybox scheduler qualified at 20/30/60 Hz, with burst coalescing, expiry, controlled stall/recovery and budget removal; broader scene/soak qualification remains. See [evidence](../SCHEDULER_QUALIFICATION.md). |
| P1-06 | Command schemas, sequencing, acknowledgements and loss tests | Partial | Bounded intents, monotonic acknowledgements, expiry and controlled impaired UDP tested; incarnation guards now active; eight-peer burst/replay/budget qualification now passes; action schemas and broader traffic qualification remain. |
| P1-07 | Renet integration and validation (replaces comparative transport study) | Partial | Renet is the launcher default after real UDP loss/delay/reorder/outage/RTT/bandwidth/queue qualification; Internet security/load and production sessions remain. |
| P1-08 | Replication protocol contracts | Partial | Wire/ALPN 10 adds bounded authority offers and tagged readiness acknowledgements. Confirmation requires the exact latest report and a post-application snapshot. General entity/delta/interest/session contracts remain. [Handover](../CHUNK_HANDOVER.md). [Readiness](../CHUNK_READINESS.md). |
| P1-09 | Kinematic movement and collision contracts | Partial | Shared Avian capsule controller drives normal maps, graybox and offline motion. Coverage sweeps constrain input/gravity and missing-corner shortcuts; support validation/fallback handles imprecise convex contacts. Retained actor/recovery validation now gates opt-in paired replacement. Head-on feature-manifold refinement addresses reproduced sloped-wall drift; see current qualification evidence. Schema-2 complete-cell coverage now requires all geometry contributors for input/gravity/readiness. Platforms/prediction and full movement scope remain. [Contributors](../CHUNK_CONTRIBUTORS.md). [Boundaries](../CHUNK_BOUNDARIES.md). [Handover](../CHUNK_HANDOVER.md). |
| P1-10 | Asset/terrain/animation content contracts | Partial | Stable package IDs/revisions, schema-3 general collision/explicit spawns and schema-2 geometry contributors exist. Manifest schema 2 now binds shared PNG variants (bytes/hash/colour space/sampler) with actual URI/path/header verification and standard Bevy loading. The offline extractor preserves source geometry/material/image bytes and stages the actual town separately. Real graph/terrain/collision partitioning, source collision fidelity, skeleton contracts and measured budgets remain. [Shared images](../SHARED_IMAGES.md). [Contributors](../CHUNK_CONTRIBUTORS.md). |
| P1-11 | Data-driven ability/combat schemas | Not started | Define data-driven abilities and server validation before gameplay loop. |
| P1-12 | Tests, instrumentation and benchmark baselines | Partial | Fixed-memory timing quantiles, separate simulation/pump metrics and eight-peer real UDP scheduler qualification now exist; full-map/render/crowd/soak baselines remain. |

## Phase 2

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P2-01 | Bevy client bootstrap | Partial | Existing Bevy bootstrap qualified; complete target foundation integration before closing. |
| P2-02 | Connection/session management | Partial | Default executable Renet lifecycle and impaired-link/fresh-ticket recovery qualified; automatic resume, provider/character authorization and production sessions remain. |
| P2-03 | Entity replication and lifecycles | Partial | Quinn and executable Renet player snapshots with world actor IDs/acks; incarnation/frame contracts now active; general entity replication remains. |
| P2-04 | Prediction and reconciliation | Partial | Actual Renet command/correction profile, 64-step replay, bounded client clock/queues, readiness overlap and stream resets run in the normal procedural client/server. Actual impaired UDP at 20/30/60 Hz and native Linux 30 Hz traversal/camera/animation/teleport/held-jump checks pass. Graybox default adopts it; legacy interpolation remains. Wider latency polish, authored-map prediction, dense load and native Windows remain. [Evidence and limits](../PREDICTION_CONTRACT.md). |
| P2-05 | Third-person camera/controls | Partial | Camera/controls qualified across client origin changes; general movement/prediction integration remains. |
| P2-06 | Terrain chunk loading | Partial | Normal opt-in publication protects readiness, actors/recovery and complete geometry contributors. Compatible PNG variants now share actual Bevy image handles; invalid dependencies retain old scenes. Lookahead retains spare packages until new providers require the budget, avoiding unload-only offer churn. Actual graph/terrain/collision partitioning, wider lifecycle checks and measured production budgets remain; town launcher stays monolithic. [Shared images](../SHARED_IMAGES.md). [Handover](../CHUNK_HANDOVER.md). |
| P2-07 | Collision integration | Partial | Shared controller drives normal maps and graybox/offline motion. Paired publication replaces actual Avian bodies, with readiness withheld until live generation/column checks pass. Schema-2 collider envelopes and contributor-complete coverage protect nonlocal walls, actors, recovery and presentation. Source-mesh fidelity and measured production collision/streaming budgets remain. [Contributors](../CHUNK_CONTRIBUTORS.md). [Boundaries](../CHUNK_BOUNDARIES.md). |
| P2-08 | GLB and asset loading | Partial | Standard Bevy GLB/PBR/image loading follows immutable manifest and dependency verification. Actual nested extents gate initial/replacement readiness; shared PNG variants preserve compatible handle reuse and distinct colour/sampler settings. New packages extract original images and geometry without changing the active map. Imported graph pruning/partitioning, skeleton/character contracts and measured asset budgets remain. [Shared images](../SHARED_IMAGES.md). [Contributors](../CHUNK_CONTRIBUTORS.md). |
| P2-09 | Character skeletal animation | Partial | Stand/walk/run exist; blending, layered mobile casting and skeleton contracts remain. |
| P2-10 | Lighting/materials/atmosphere | Partial | Existing PBR light/shadows and water preview; atmosphere/quality requirements remain. |
| P2-11 | Targeting and interactions | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P2-12 | One authoritative ability, damage, death and respawn | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P2-13 | Spell VFX and audio hooks | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P2-14 | Health/target/action-bar UI and integrated demo | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |

## Phase 3

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P3-01 | Authentication and sessions | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-02 | PostgreSQL persistence/migrations | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-03 | Character creation/selection | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-04 | Stats and resources | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-05 | Inventory and items | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-06 | Abilities and status effects | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-07 | Creature AI | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-08 | Equipment and appearance | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-09 | Loot and rewards | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-10 | Experience and leveling | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-11 | Quest objectives | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-12 | NPC dialogue and vendors | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-13 | Terrain streaming and initial LOD | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-14 | Animation, VFX and audio expansion | Precursor only | Existing locomotion is reusable; animation/VFX/audio expansion is unimplemented. |
| P3-15 | Content authoring and validation | Precursor only | Offline conversion tests are reusable; gameplay authoring/validation is unimplemented. |
| P3-16 | One complete persistent zone and login/logout proof | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P3-17 | UNSPECIFIED in original transcript: needs reconciliation | Blocked: scope unknown | Obtain missing scope from the owner; do not block unrelated defined tasks. |
| P3-18 | UNSPECIFIED in original transcript: needs reconciliation | Blocked: scope unknown | Obtain missing scope from the owner; do not block unrelated defined tasks. |

## Phase 4

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P4-01 | World region ownership | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-02 | Predictive terrain/object streaming | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-03 | Streamed navigation and spatial queries | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-04 | Advanced interest management | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-05 | Adaptive replication | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-06 | Cross-region handoff | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-07 | Population and overload management | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-08 | Advanced combat interrupts/CC/AoE/healing/threat | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-09 | Classes and specializations | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-10 | Group encounters | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-11 | Parties and group rules | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-12 | Chat/moderation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-13 | Friends and presence | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-14 | Guild persistence | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-15 | Secure trading | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-16 | Animation LOD and crowd policies | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-17 | VFX density budgets | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-18 | Time/weather integration | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-19 | Multi-zone scale profiling | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-20 | World and multiplayer integration gate | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |

## Phase 5

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P5-01 | Rendering benchmarks | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-02 | Lighting and shadows | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-03 | Atmosphere and fog | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-04 | Stylized materials | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-05 | Water | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-06 | Terrain LOD and seams | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-07 | Streaming memory budgets | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-08 | Vegetation instancing and wind | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-09 | Visibility/culling | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-10 | Animation blending and layers | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-11 | Equipment visuals | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-12 | Animation LOD | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-13 | Character readability | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-14 | Particle expansion | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-15 | Projectiles/AoE effects | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-16 | VFX priority and density limits | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-17 | World editor | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-18 | Creature/item/ability/quest editors | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-19 | Asset conversion and license validation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-20 | Developer inspection tools | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-21 | Client performance tuning | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-22 | Server profiling | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-23 | Deployment/monitoring/recovery | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-24 | Integrated visual and load validation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |

## Phase 6

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P6-01 | Release scope and quality gates | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-02 | Minimum playable content | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-03 | Internal playtests/defect tracking | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-04 | Class/combat balance | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-05 | Progression and economy validation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-06 | Quest/encounter chains | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-07 | Onboarding/accessibility | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-08 | Closed alpha accounts/environment | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-09 | Installer/patcher | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-10 | Closed alpha stabilization | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-11 | Concurrent gameplay load test | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-12 | Crowded encounter test | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-13 | Recovery drills | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-14 | Security/exploit testing | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-15 | Closed beta population | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-16 | Long soak testing | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-17 | Production deployment/support | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-18 | Moderation/support workflows | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-19 | Release candidate validation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-20 | Operational readiness review | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-21 | Go/no-go | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-22 | Final load test | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-23 | Launch | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-24 | Launch stabilization | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |



## Offline actual-world partition continuation — 2026-10-20

P1-10/P2-06/P2-07/P2-08, row 5: actual terrain/collision clipping, whole model
ownership and per-package graph pruning now have independent source comparisons.
The town produces 26 packages; all 2,532 model mesh instances and original
transforms/buffers/materials remain, with terrain UV/height/winding and collision
surface/box checks. Each actual package passes the Rust verifier. Generated
original partition output also passes two actual two-client Bevy/Avian cases,
including signed distant/independent frames and delayed publication.
[Contract and measured budgets](../WORLD_PARTITIONING.md).

The town needs nine providers per cell and ten initially, so only a candidate
index is emitted under the four-package proof budget. This closes the offline
partition slice, not actual-town runtime adoption or a whole task. Next: measure
combined actor/lookahead/recovery/staging demand and resource costs, qualify an
appropriate bounded runtime budget and world-edge policy, remaining handover
lifecycle cases, then real town publication/map regressions before default
adoption. Keep prediction behind the relevant row-5 gate. Current checks and
limitations remain separate in [validation](../VALIDATION.md).


## Verified content-budget analysis — 2026-10-20

P1-03/P1-10/P2-06/P2-07/P2-08: source-verified recovery, actual package/image
costs and conservative actor footprint/lookahead unions now have executable
offline analysis. Town union maxima are 18/22/26 packages for one/two/up-to-eight
actors including recovery. All-indexed content has 4,001 static colliders and
1,514,491,904 estimated base RGBA8 bytes; these are not measured live allocations.
A 32-package hard ceiling / 26-package town budget is proposed with concrete
protocol/admission/world-edge/staging/native acceptance in
[the budget contract](../CHUNK_BUDGETS.md). Runtime remains limited to four;
actual town adoption, full task/phase completion and Windows/MMO capacity remain
open. Continue the runtime gate in execution order, retaining map regressions.


## Qualified bounded runtime capacity — 2026-10-20

P1-03/P1-06/P1-08/P1-10/P2-06/P2-07/P2-08 now qualify a 32-package hard
ceiling, unchanged smaller session budgets, checked control addresses,
budget-bound admission identity v3 and wire/ALPN 11. Actual secure UDP carries
full sets; disk/Avian tests protect the 33rd unloaded cell. The opt-in private
town authority loads ten packages then publishes all 26 against a live actor.
Eight actual two-client native cases qualify 32 initial roots, six-provider
delayed publication, independent negative frames, failed-image retention and
normal map/Quinn regressions. [Contract and evidence](../CHUNK_CAPACITY.md).

Continue row 5 with explicit finite-world bounds and edge/corner qualification,
then actual-town client loading/memory/staging measurements and remaining
reversal/supersession/asymmetric-failure checks before promotion. Preserve the
monolithic default at 30 Hz. Native Windows, full task/phase completion and MMO
capacity remain open. New Unity/Synty asset inspection can proceed independently
without changing the adopted runtime architecture or promoting an unqualified map.


## User-supplied Synty source inspection — 2026-10-20

P1-10/P2-08/P2-09: reviewed Base Locomotion, Elven Realm and Modular Fantasy Hero
archives from the user-specified Downloads folder. The locomotion sample has a
50-bone skinned model and matching selected animation bone names. The Elven demo
contains 15,416 serialized prefab instances; preserving scene structure matters
in addition to converting meshes. Modular source scale/rig differences need
qualification. [Review, evidence and concrete import acceptance](../SYNTY_ASSET_REVIEW.md).
No converted/native Synty result or default replacement is claimed. The core
row-5 finite-world and actual-town client/resource gates remain open.


## Isolated Synty locomotion preview — 2026-10-20

P1-10/P2-08/P2-09: explicit Unity payload extraction, source-pose baking into a
50-joint sample skin, standard GLB export, independent frame comparison and four
native Bevy playback cases now qualify the bounded preview. Full weights are
retained; the four-weight Jump preview exceeds the initial 5 mm gate at 6.974 mm
and is explicitly not production-character adoption. [Contract](../SYNTY_CHARACTER_PREVIEW.md).
Default gameplay/character/map stay unchanged. Elven scene conversion can proceed
as the user's asset branch while core world-edge/resource gates remain open.

## Elven Realm source and representative static preview — 2026-10-20

P1-10/P2-08: hash-bound metadata extraction and bounded recursive Unity dependency
inspection now identify model-prefab ID gaps and unresolved serialized references.
A single pillar's standard GLB/base-colour preview passes independent geometry,
winding, UV and palette comparison, clean Khronos validation and one actual native
Bevy capture. Full scene instantiation, complete materials, collision/navigation
and map adoption remain open. [Acceptance and scope](../SYNTY_ELVEN_IMPORT.md).
This is an isolated asset branch; no task or core foundation gate is complete.

The follow-on model-ID audit matches 1,109 distinct serialized references, with
62 remaining explicit gaps. Source completeness is now a concrete Elven gate:
32 selected payloads are LFS pointers; 26 FBXs and one PNG are reachable from the
demo closure. Missing blob hashes/sizes are recorded privately. This blocks a
complete equivalent scene export, not unrelated core row-5 work. [Details](../SYNTY_ELVEN_IMPORT.md).

## Explicit finite chunk-world bounds — 2026-10-20

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08: schema-2 session rectangles participate
in admission, conservative local swept planes constrain shared authority motion,
lookahead clamps to the rectangle and recovery spawns retain full clearance.
Legacy identities/internal seams remain. Edge/corner, distant-frame, jump/retreat
and unsupported-fall/held-jump recovery checks pass at 20/30/60 Hz. A separate
opt-in town session preserves its source rectangle. [Scope](../WORLD_EDGES.md).
Actual-town client preparation/staging resources and remaining handover lifecycle
checks still precede promotion; no complete task/phase claim follows.

## Actual-town client startup/resource checkpoint — 2026-10-20

P1-10/P1-12/P2-06/P2-07/P2-08: actual ten/all-26-package ordinary two-client
startup now passes with measured Linux process/DRM memory and approximate ready
times. The new index binds transformed mesh-local AABBs after a strict runtime
rejection exposed the tighter vertex-only authoring envelope. Package/image
bytes remain unchanged. [Contract and measurements](../TOWN_CLIENT_RESOURCES.md).
Live client candidate/staging/retirement memory and remaining lifecycle checks
still precede default promotion. No whole task/phase or capacity claim follows.

Actual-town live-publication continuation (row 5, P1-10/P1-12/P2-06/07/08):
shared startup preparation now exposes authority-resolved recovery feet, and exact
round-trip JSON parsing protects admission identity for decimal source heights.
Bounded Bevy lifecycle traces and synchronized two-client captures support source
primitive, retained-root/image and rejected-candidate cleanup checks.
See [the contract and qualification scope](../TOWN_LIVE_PUBLICATION.md).
Broader handover lifecycle checks and default adoption remain open.

Environment-source steering, 2026-10-20: the user selected
[Enchanted Forest](../SYNTY_ENCHANTED_FOREST_IMPORT.md) instead of Elven Realm.
Forest conversion/qualification is the active asset branch; do not wait for Elven
LFS recovery. Preserve town regressions until forest adoption passes.

Forest P1-10/P2-08 continuation: reusable opaque Standard material adaptation
and one actual ruins prefab now pass independent geometry/pixel/material checks
and native Bevy Vulkan inspection. Full scene/foliage/hierarchy/collision gates
remain; see [qualification](../SYNTY_ENCHANTED_FOREST_IMPORT.md).

Forest P1-10/P2-08 source-placement continuation now resolves ordinary prefab
hierarchy/overrides/LOD selection with independently checked emitted source
matrices. Four FBX-backed variants, effects, geometry axes, custom materials and
assembled-scene/collision acceptance remain open.
[Scope and evidence](../UNITY_STATIC_PLACEMENTS.md).

Forest P1-10/P2-08 basic-foliage continuation qualifies real tree/vine cards
through source-derived masked standard PBR, independent pixels/geometry and two
native captures. Explicit preview LOD selection preserves the entire default
highest-detail scene plan. Full vegetation, custom normals, geometry coordinates
and assembled scene/collision adoption remain open. [Scope](../FOREST_FOLIAGE.md).

Forest P1-10/P2-08 model-variant continuation now resolves the four branch
variants through checked inherited aliases and complete authored root TRS. All
179 placements pass independent completeness/binding/matrix checks; old records
are unchanged. Geometry/importer coordinates, full materials and assembled
scene/collision adoption remain open. [Evidence](../UNITY_MODEL_VARIANTS.md).


Forest P1-10/P2-08 geometry continuation: independent mesh-local calibration
matches 52/54 source LOD bounds with two explicit centre mismatches. A shared
standard-glTF assembler preserves shear and mirrored winding; five authored ruins
placements and an asymmetric fixture pass native Bevy/Vulkan checks. Binary
TerrainData, full vegetation, assembled-scene/collision/multiplayer adoption
remain open; preserve town regressions. [Contract](../UNITY_GEOMETRY.md).


Forest P1-10/P2-06/P2-07/P2-08 terrain prerequisite now preserves all 263,169
binary source height samples, five control/layer bindings and 1,365 patch extrema.
Full-grid ground and ground-plus-five-ruins render in native Bevy; the finite
colour bake/geometric normals remain explicit approximations. Partition matching
collision under current budgets, complete full vegetation/source assembly and
qualify multiplayer before adoption. [Contract](../UNITY_TERRAIN.md).


Forest P1-10/P2-06/P2-07/P2-08 ground collision continuation now qualifies four
full-sample packages, independent complete source coverage, real Avian probes/
seam/jump checks and two encrypted native two-client cases. An opt-in launcher
loads the ground; the default town and old map routes remain qualified. Complete
vegetation/materials, authored object collision and representation/budget review
still precede forest adoption. [Contract](../FOREST_TERRAIN_COLLISION.md).


Forest P1-10/P2-08 highest-detail foliage continuation qualifies a static full-
shader PBR profile, actual tree/fern geometry/normal/cutout checks and three native
cases. Three trees and 612 exact-binding ferns now join the visual subset; eight
of twelve used full-shader property profiles are eligible, four remain explicit.
Batch remaining mesh/material overrides and shader families, then complete scene/
collision/resource/multiplayer acceptance before adoption. [Scope](../FULL_FOREST_FOLIAGE.md).


Forest P1-10/P2-08 bulk foliage continuation now qualifies 15,156 source draws in
46 exact mesh/material bindings, actual scene overrides and a shared-image visual
assembly. Independent complete source/category/material/tangent checks and two
native Bevy/Vulkan views pass; 3,503 draws retain explicit unresolved reasons.
Continue remaining Standard/other shader families and mixed slots, then review
object collision and resource budgets before full multiplayer/default adoption.
Town and opt-in forest-ground remain available. [Evidence](../FOREST_FOLIAGE_BATCH.md).


Forest P1-10/P2-08 Standard-material continuation qualifies another 66 bindings /
2,744 source draws, bringing the static subset to 17,900 draws with 31 shared
image variants. Canonical mesh-local export excludes imported FBX node placement;
source normal directions, exact palettes, full scene poses and two native Bevy/
Vulkan views qualify. 759 draws remain explicit. Continue four model variants,
other shader families/mixed slots, then object collision and resource/multiplayer
acceptance before adoption. [Evidence](../FOREST_STATIC_BATCH.md).


Forest P1-10/P2-08 canonical model-variant continuation now qualifies all four
branch models / 179 poses, with inherited IDs, raw nested/outer material bindings,
independent ufbx/source geometry and byte-preserved prior content. The combined
18,079-draw scene passes complete source and native Bevy/Vulkan checks. 580 draws
remain explicit; continue triplanar cliffs/rocks and other shader/mixed-slot work,
then object collision and resource/multiplayer acceptance before adoption.
[Contract](../FOREST_VARIANT_GEOMETRY.md). The user queues the new Elven Realms
folder under Downloads / Synty / Biomes only after forest completion; its contents
have not yet been verified.


Forest P1-10/P2-08/P2-10 triplanar prerequisite now has a source-hash-bound world
projection profile and opt-in Bevy StandardMaterial extension. Actual canonical/
transformed mesh cases pass native lit/diagnostic rendering and independent GPU
pixel checks. The main subset remains 18,079 draws with 580 unresolved. Before
batch adoption, calibrate the exposed FBX mesh/root-pivot offset (including existing
LOD-bound evidence), then integrate multiple profiles, f64 rebase texture phases,
resources/mips and complete scene/object collision acceptance. No automatic mesh
centering or default-map change. [Contract](../FOREST_TRIPLANAR.md). Elven folder
inspection remains queued after forest completion.

Forest P1-10/P2-08 importer checkpoint: actual Linux Unity 6000.6.6f1 compilation/
export and independent recorded-frame point sets pass for all 112 scene FBXs /
214 meshes, with original metadata unchanged. Thirteen existing batch draws need
frame correction. All jobs are terminal; the user requested a safe stop before
converter changes. Resume with source-bound frame adaptation and independent/
native replacement checks, then remaining material/collision/resource gates.
[Findings and acceptance](../FOREST_UNITY_REFERENCE.md).

Resumed 2026-10-21: P1-10/P2-08 now applies recorded Unity mesh frames to ten
bindings / thirteen placements while retaining 106 prototype/placement pairs.
All 116 prototypes and complete rebuilt scene pass source/native checks; town/
farmhouse regressions remain. Corrected moss/all-85 source poses pass as a
standalone single-profile material group, not main-scene adoption. Continue
multi-profile/rebased texture phase/resources/mips, remaining materials, then
object collision/runtime gates. [Evidence](../FOREST_MESH_FRAMES.md).

P1-10/P2-08/P2-10 catalog continuation now qualifies actual mixed Standard and
two-profile rendering, shared typed image handles, complete source composition,
negative mappings and per-profile pixel samples. Legacy and map regressions pass
on the final client. Continue eligible binding batches/full-scene composition,
then rebased texture phase/resources/mips and remaining material/collision/runtime
gates. [Evidence](../FOREST_MATERIAL_CATALOG.md). Main subset counts are unchanged.

## Forest triplanar batch continuation — 2026-10-21

Forest triplanar batch continuation, 2026-10-21: seventeen new bindings / 518
placements now join the corrected visual subset: **18,597 source draws / 62
unresolved**. Fresh independent source/Editor/placement and full composition checks,
four combined native Vulkan views, three dirt-profile modes and 304 pixel checks
pass. Three profiles share eight extension images; 31 Standard image variants
remain distinct. Khronos: 35 files, zero errors, seven runtime-tangent warnings.
Fresh fmt, denied-warning Clippy and 173 Python tests pass. Runtime binaries stay
unchanged; collision/rebase/mips/resource and adoption gates remain. Next: actual
Editor sampler defaults for ten grass draws. All work stays local.
[Evidence and limits](../FOREST_TRIPLANAR_BATCH.md).


## Forest texture-default continuation — 2026-10-21

Forest texture-default continuation, 2026-10-21: actual Linux Unity 6000.6.6f1
resolves four Grass/Mud textures to bilinear/repeat with original PNGs/metadata
unchanged. The ten grass draws now join the qualified subset: **18,607 source
draws / 52 unresolved**. Fresh eighteen-binding source/Editor/placement checks,
four combined native Vulkan views, three grass modes and 274 pixel checks pass.
Fresh fmt, denied-warning Clippy and 175 Python tests pass. Runtime binaries remain
unchanged. Next: independent leaf-colour UV tiling for seventeen giant-tree draws;
remaining materials, rebase/mips/resources, collision and adoption remain open.
All work stays local. [Evidence](../FOREST_TEXTURE_DEFAULTS.md).


## Forest colour-UV continuation — 2026-10-21

Forest colour-UV continuation, 2026-10-21: two exact giant-tree bindings /
seventeen placements now join the qualified visual subset: **18,624 source draws /
35 unresolved**. A second standard glTF channel preserves leaf colour tiling while
normal/AO retain UV0. Source/Editor/exact-placement/full composition checks and six
native Vulkan cases pass; actual loaded materials select the expected channels.
Legacy rendering and encrypted town/farmhouse regressions pass on the new client.
Fresh fmt, denied-warning Clippy, 35 client Rust and 179 Python tests pass. Remaining
materials, rebase/resources/mips/lighting, collision and adoption stay open. All work
is local. [Evidence](../FOREST_COLOUR_UV.md).

## Forest glass continuation — 2026-10-21

Forest glass continuation, 2026-10-21: an explicit untextured Standard-to-glTF
BLEND adaptation adds the lantern glass: **18,625 source draws / 34 unresolved**.
Source geometry/material, actual Editor frame, exact placement/full composition
and four native Vulkan cases pass. The adaptation records Unity premultiplied
lighting/colour-space differences; no runtime shader/binary change is introduced.
Fresh fmt, denied-warning Clippy and 181 Python tests pass. Next: actual mixed
submesh-slot coverage for 25 crystal/geode draws. Remaining materials, lighting,
rebase/resources/mips, collision and adoption remain open. All work stays local.
[Evidence](../FOREST_GLASS.md).

## Forest mixed-material continuation — 2026-10-21

Forest mixed-material continuation, 2026-10-21: nine crystal/geode bindings /
25 draws join the qualified subset: **18,650 source draws / nine unresolved**.
Actual Editor submesh triangles prove source-slot fidelity; four differing quad
diagonals use Editor topology with original corner normals/UVs. Crystal appearance
is an explicit static standard-PBR preview, with refraction/view effects omitted.
Full composition, nine native Vulkan cases, five rebound faults and 19 Khronos
files pass (zero errors/seven retained warnings). Fresh fmt, denied-warning Clippy
and 189 Python tests pass. Water/sky/emissive willow, lighting, rebase/resources/mips,
collision and adoption remain open. All work stays local.
[Evidence](../FOREST_MIXED_MATERIALS.md).

## Forest source sky — 2026-10-21

Forest sky continuation, 2026-10-21: the source skydome now joins the qualified
subset: **18,651 source draws / eight unresolved**. Actual Editor geometry, source
world-height gradient and placement pass independent checks. Standard unlit and
explicit Bevy shadow markers pass four native Vulkan cases; three legacy modes,
two native failure paths and encrypted town/farmhouse regressions pass. Fresh fmt,
denied-warning Clippy, 38 client Rust and 193 Python tests pass. Three Khronos
files have zero errors/seven retained warnings. Seven water draws and one emissive
willow remain, followed by lighting, rebase/resources/mips, collision and adoption.
All work stays local. [Evidence](../FOREST_SKY.md).

## Forest static water — 2026-10-21

Forest water continuation, 2026-10-21: four bindings / seven source water draws
join the qualified visual subset: **18,658 source draws / one unresolved**. Actual
Linux Unity proves built-in Plane identity/geometry; exact Editor FBX triangles,
raw source poses and explicit static PBR factors pass independent checks. Full
composition, eleven native Vulkan cases and four rehashed faults pass. Ten Khronos
files have zero errors/seven retained warnings. Fresh fmt, denied-warning Clippy
and 197 Python tests pass; runtime binaries remain unchanged. Emissive willow,
source water effects, lighting, rebase/resources/mips, collision and adoption remain
open. All work stays local. [Evidence](../FOREST_WATER.md).

## Forest emissive/static coverage — 2026-10-21

Forest emissive continuation, 2026-10-21: the willow completes **18,659 planned
static source draws / zero unresolved static draws**. Source mask HDR emission
uses standard glTF strength 2.6 with an explicit base-floor preview omitting spatial
colour noise. Exact source/Editor geometry, full composition, seven native Vulkan
cases, four rehashed faults and encrypted town/farmhouse regressions pass. Fresh
fmt, denied-warning Clippy, 38 client Rust and 201 Python tests pass; four Khronos
files have zero errors/seven retained warnings. Source effects/particles, lighting,
resources/rebase/mips, object collision and normal map adoption remain open. Next:
qualify shared images against the measured duplicate-image inventory. All work
stays local. [Evidence](../FOREST_EMISSIVE.md).

## Forest shared image resources — 2026-10-21

Forest resource continuation, 2026-10-21: the complete static scene now shares
**37 actual core image handles instead of 77** across 298 slots. Loaded CPU Image
payloads reduce 3,124,764,672 → 1,421,877,248 bytes; both paired native views are
pixel-identical outside the HUD. Complete 944-accessor/material/PNG/role/sampler
checks, four rehashed faults and three legacy modes pass. Fresh fmt, denied-warning
Clippy, 38 client Rust and 202 Python tests pass; Khronos has zero errors/seven
retained warnings. All 18,659 static source draws remain covered. This is inspector
sharing; package admission stays unchanged. Lighting, authored collision, rebase/
mips and runtime adoption remain open. All work stays local.
[Evidence](../FOREST_RESOURCES.md).

## Forest source lighting boundary — 2026-10-21

Forest lighting continuation, 2026-10-21: actual Unity imports qualify source
renderer layers and cookie alpha; all 26 offline source lights are preserved.
Native pixels expose unsupported selective receiver masks and a disabled cookie
feature. The client enables standard Bevy light textures and rejects selective
masks; a separately verified inspector copy explicitly omits three masked suns,
retaining 22 point lights, one spotlight, ambient and fog. Two full-scene and two
cookie GPU cases, three legacy modes, three failure paths and encrypted town/
farmhouse regressions pass. Four rehashed faults reject. Fresh fmt, denied-warning
Clippy, 41 client Rust and 204 Python tests pass. This is an explicit lighting
approximation; selective suns, authored collision, rebase/mips and normal forest
adoption remain open. Next: source collider geometry and measured package capacity.
All work stays local. [Evidence](../FOREST_LIGHTING.md).

## Forest authored collision reference — 2026-10-21

Forest authored-collision reference, 2026-10-21: actual Linux Unity now covers
all 79 raw prefab mesh references, including 39 new collision FBXs and 17 exact
serialized meshes. The actual scene and independent raw hierarchy agree on all
2,776 collider components, with 2,763 active/enabled and maximum matrix difference
0.046 mm. Native targeted rays provide 16,578 reference cases. Four rebound
pose/mesh/state/capsule faults reject; 213 Python tests pass. The invisible downward
water collider, nonuniform primitives, mirrored/sheared meshes and full terrain
budgets remain explicit constraints. Next: native Avian object-query comparison
and measured representation/capacity. Runtime object admission and forest adoption
remain open. All work stays local. [Evidence](../FOREST_COLLIDER_REFERENCE.md).

## Forest Avian source geometry — 2026-10-21

Forest Avian geometry continuation, 2026-10-21: a real headless Bevy/Avian
world matches 2,762 active source object colliders across 16,572 targeted rays,
maximum distance difference 0.27 mm. A float64 source-surface oracle separately
proves all 17 convex inputs are closed hulls. Four rehashed expected-hit/distance/
normal/mesh faults reject in actual Avian. Strict Unity ray equality remains a
recorded diagnostic failure because cooking, backfaces and sheared-mesh behaviour
differ; it is not promoted as parity. Fresh format/diff checks, denied-warning
workspace/all-target Clippy and 215 Python tests pass. Terrain, package budgets,
water policy, movement/camera and forest adoption remain separate gates. Next:
measure the source representation against existing runtime package limits. All
work stays local. [Evidence](../FOREST_COLLISION_GEOMETRY.md).

## Forest full-sample heightfield prerequisite — 2026-10-21

Forest heightfield continuation, 2026-10-21: standard Avian full/adjoining
heightfields retain every source sample and oriented triangle surface. All
2,362,371 source-vertex/centroid queries and 54 capsule seam route runs at 20/30/60
Hz pass against retained triangle packages; maximum heightfield error is 7.16
micrometres. Rectangular axis/diagonal checks pass; two rehashed sample/scale faults
reject. Fresh fmt, denied-warning workspace/all-target Clippy and 217 Python tests
pass. Offline collision-only layout measures at most 47,245 pooled input triangles,
259,623 instantiated triangles, 66,049 height samples per provider and eight
contributors per finite terrain cell. These are not runtime capacity/publication
claims. Next: bounded versioned shape/heightfield admission; existing packages and
defaults remain preserved. All work stays local. [Evidence](../FOREST_HEIGHTFIELDS.md).

## Bounded static-shape admission — 2026-10-21

Static-shape admission continuation, 2026-10-21: collision schema 4 now uses
standard Avian shared mesh inputs, oriented boxes, capsules and full-sample fields
through the normal package/client/server paths. Complete source binding covers
1,731,286 mesh vertices and 263,169 unique heights; all 804,029 source object/
terrain rays pass. Four actual Avian faults and five independent geometry faults
reject. Original encrypted two-client, ±128-billion-metre contributor, town and
farmhouse cases pass on Linux RX 7900 XTX with Vulkan validation. Fresh format,
denied-warning Clippy, complete default network suite, 41 client and 226 Python
tests pass. Existing budgets/defaults remain preserved. Forest visual/material
publication, rebase/resources, water policy and actual-map adoption remain open.
All work stays local. [Contract and evidence](../STATIC_SHAPES.md).

## Forest complete reference packaging — 2026-10-21

Forest package continuation, 2026-10-21: ten source-preserving candidate packages
retain all 18,660 draws, 18,925 primitive instances, 6,137,378 triangles and complete
object/full-sample collision, with at most eight cell contributors. Independent
stream/pose/material/collision checks and four rebound faults pass; actual package
loading plus all 804,029 source rays pass. Ten Khronos files have zero errors and
22 retained generated-tangent warnings. Standard unlit/HDR-emissive shared-image
admission also passes native encrypted material/town/farmhouse cases, ten semantic
faults, the complete default network suite and 41 client tests. Fresh fmt/diff,
denied-warning Clippy and 227 Python tests pass. No forest session/default is
published; custom material rebasing/admission, flags/lighting, resources/mips,
water and actual map movement/camera remain open. All work stays local.
[Packages](../FOREST_SHAPE_PACKAGES.md), [standard materials](../STATIC_MATERIALS.md).

## Forest source texture phase — 2026-10-21

Forest texture-phase continuation, 2026-10-21: the standard Bevy material
extension now accepts a bounded source translation and uploads only its periodic
texture phase. Fifteen native Vulkan views pass: 1,056 independent source-pixel
checks, nine paired colour/normal/lit cases with at most one RGB8 level difference,
a deliberately wrong origin rejected, and both complete forest views pixel-exact
outside HUD. Three translated GLBs have zero Khronos errors/warnings. Fresh fmt/
diff, denied-warning Clippy, 42 client and 218 asset Python tests pass. This is
bounded inspection qualification; normal-world automatic frame updates and bound
runtime material/image admission remain next, followed by flags/lighting,
resources/mips, water and actual-map movement/camera/adoption. All work stays local.
[Evidence and scope](../TRIPLANAR_PHASE.md).

## Bound runtime projection materials — 2026-10-21

Forest runtime-material continuation, 2026-10-21: package schema 3 now binds
self-contained projection parameters, global f64 source anchors and canonical
PNG dependencies; a saved older reader rejects it before startup. The normal
client gates root readiness, shares actual materials/images and updates bounded
phase uniforms with live render rebasing. Six encrypted two-client fixture views
at ±128 billion metres pass, including 828 independent source pixels (colour ≤1
RGB8 level, normals exact). Five native failure paths, fresh town/farmhouse and
three retained handover cases pass. Position-based gait diagnostics preserve the
existing movement/animation acceptance checks. Complete default network tests,
44 client tests, 221 asset Python tests, fresh fmt/diff and denied-warning Clippy
pass. This qualifies the original runtime slice; forest embedding, actual forest
resources/rebase, flags/lighting/mips, water and movement/camera/adoption remain
open. All work stays local. [Contract and evidence](../RUNTIME_MATERIALS.md).

## Complete forest runtime candidate — 2026-10-21

Forest normal-runtime continuation, 2026-10-21: ten bound material packages
preserve complete source geometry/collision; independent comparisons, four
rehashed faults, all 804,029 source rays and glTF validation pass. A separate
candidate excludes exactly one invisible water collider. Two real encrypted
Linux/Vulkan clients load the full scene, share four projection materials and
47 actual images, and pass visibility/readiness/rebase checks. Per-client base
RGBA CPU image data is 1,589,649,408 bytes (not total memory). The actual-map
survey records 27 clear starts and 41 rejected routes; movement, lighting,
mips/resources, water and broader camera/adoption remain open. Town stays the
default. All work stays local. [Evidence and limits](../FOREST_RUNTIME.md).

## Interactive performance correction — 2026-10-22

P1-12 / P2-05 / P2-08 / P2-10: release launch defaults and authored collision-based
chunk camera clearance address the reported slow debug walkthrough. The full
forest measures 99.65 FPS uncapped at 1280×720; a normal desktop relaunch averages
57.89 FPS with VSync. Frame-time spikes and measurement limits are retained.
All 45 client tests, denied-warning Clippy and 221 asset Python tests pass.
This does not complete the task rows or establish MMO capacity. See the
[performance record](../FOREST_PERFORMANCE.md).

### P2-04 prerequisite continuation — 2026-10-22

Gameplay-only kinematic state capture/restore and exact same-frame Avian replay
now pass at 20/30/60 Hz, with invalid-state/diagnostic injection rejection. The
new procedural two-peer impairment check passes readiness, seam crossing,
withdrawal/regrant and stopped convergence under delay/jitter/10% loss/duplication/
reorder. An airborne withdrawal keeps gravity and landing active while holding XZ.
These are prerequisites, not prediction adoption. Wire v11 still carries held input
and position snapshots; completed-tick/command and authoritative motion-state
correction messages remain next. [Contract](../PREDICTION_CONTRACT.md).

### P1-06/08, P2-04 bounded replay continuation — 2026-10-22

Completed-step input timing, a separate movement-stream epoch and 64-step
reconciliation history now pass headless shared-controller checks at 20/30/60 Hz.
Seven replay/reset checks and six timing checks pass, including real Avian terrain
seam/jump queries under scheduled packet faults. Full workspace: 214 tests pass;
denied-warning Clippy and fmt/diff pass. These simulated delivery schedules are not
new real-UDP prediction evidence. Actual authority/Renet/client integration and
native presentation remain next; P2-04 stays precursor-only. Existing wire v11 and
qualified default release behavior are preserved. [Evidence](../PREDICTION_CONTRACT.md).
