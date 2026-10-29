# Active work — 2026-10-29: authenticated native admission

The account HTTP/OIDC/session/owned-character boundary is qualified and frozen in
`build/account-http-qualified-v1-20261029` (source copies/hashes and exact tested
executable). Actual two-user Keycloak/PostgreSQL HTTP checks pass authorization,
proof-bound collection/cancellation, revocation/supersession, strict inputs,
measured bounds/deadlines, fail-closed backend loss, graceful shutdown and database/
API restart preserving character IDs while invalidating old process credentials.
[Exact evidence/limits](../ACCOUNT_HTTP_FOUNDATION.md).

**294 workspace tests pass**, 13 explicit skips (ten private fixtures plus three
actual-PostgreSQL contracts executed separately); formatting and workspace/all-
target denied-warning Clippy pass. The standalone server remains headless. This
has two synthetic accounts/nine database characters and no authenticated native
game admission yet. Earlier OIDC/character/legacy native checkpoints are retained.

Next is the [P3-01/P3-03 native admission slice](../AUTHENTICATED_ADMISSION.md):
memory-only encrypted ticket signing, selected character/session binding,
one-use nonce/generation, prepared saved-pose collision, bounded active/retiring
ownership and final checkpoint before slot reuse. System-browser launcher entry,
authenticated native reconnect/revocation, production HTTPS/provider/renewal and
native Windows remain open. Ordinary development launch remains available.
The memory-only loopback ticket issuer is now implemented: two real-UDP issuer
checks plus twelve secure-runtime and two saved-pose regressions pass; network
all-target denied-warning Clippy and formatting pass. This is ticket signing,
not character/session admission. The 294-test workspace result above precedes
this refactor; the sixteen focused tests cover its delta.

The user authorized committing and pushing this engineering checkpoint on
2026-10-29. Story/private export edits remain outside this checkpoint. Earlier
local/uncommitted statements below describe historical work sessions.

# Current development checkpoint — 2026-10-29: persistence qualified

P3-02's initial ordinary-position persistence slice is qualified against actual
PostgreSQL 16.15. Two native encrypted clients save authoritative checked f64
positions, restart both database and server, reconnect in reversed order and
restore their own positions around 15 km despite a changed default spawn. Stable
character/owner IDs survive; runtime actor/world identity is fresh. Wrong-world
startup rejects before listen, and actual database backend loss fails the session
closed. [Exact behavior and evidence](../PERSISTENCE_FOUNDATION.md).

**284 workspace tests pass**, with 12 explicit skips (ten private fixtures and two
actual-PostgreSQL tests separately executed successfully); formatting and denied-
warning all-target Clippy pass. Native integrated persistence/combat/UI/feedback
passes under impaired UDP with 9,728 cosmetic requests per client and all critical
cues delivered. Launcher interruption/child-failure cleanup closes database
ownership too. The persistence-disabled one-client empty-world baseline passes at
821.84 FPS uncapped, validation off; this does not establish MMO capacity.
Qualified artifacts: `build/persistence-qualified-v1-20261029`.

`python3 tools/play.py --persistence` opts into the development database boundary
using private `THEIA_DATABASE_URL` (numeric loopback only); add `--spell-feedback`
for combat/UI/effects. Ordinary launch still works without database configuration.
This uses trusted development issuer slots, not external accounts. Character
creation/selection, inventory/economy durability, power-loss recovery, process
handoff, native Windows and production database transport remain open. Owned test
clusters/processes stopped and plaintext password files were removed.

Next is P3-01 provider-neutral OIDC, validated against an owned local Keycloak
fixture, as selected by the user. Its initial scope and acceptance checks are in
[AUTHENTICATION_FOUNDATION.md](../AUTHENTICATION_FOUNDATION.md). The provider
fixture has passed discovery and a real authorization-page/PKCE code exchange;
application verification, internal sessions and game admission remain unimplemented.
Production identity-provider selection stays open. Engineering changes remain
local/uncommitted; nothing was pushed and story edits remain untouched.
Earlier checkpoints below are historical.

# Current development checkpoint — 2026-10-29

P2-14's initial action-slot/health/cast/cooldown UI is qualified. Standard Bevy
widgets and the Digit 1 shortcut share the existing bounded cast-request path;
all values come from authoritative health/tick deadlines. Two actual clients pass
impaired encrypted UDP, five pointer presses/five widget requests plus five
keyboard requests each, <0.47 px actual fill-width error and integrated
movement/targeting/damage/death/respawn/critical cues. [Exact evidence](../COMBAT_UI.md).

**280 workspace tests pass**, ten private-fixture skips, denied-warning
workspace/all-target Clippy and formatting. A first full-build Rust linker
segmentation fault is retained; the serial retry completed successfully.
Native critical-only quality, isolated PCM, same-binary ordinary baseline
(832.17 FPS uncapped, one empty-world client, validation off) and real two-client
UI-only/feedback launcher interruption/child-failure cleanup pass. Binaries and
original data are preserved in `build/combat-ui-qualified-v1-20261029`.

Run `python3 tools/play.py --combat` for the clickable slot and bars, or
`--spell-feedback` for the original spatial effects/audio too. Tab selects,
Backspace clears; press the slot or Digit 1 to cast. This is one development slot,
not a production multi-bar editor, full accessibility/responsive UI, player HP,
physical OS input, Windows execution or population-capacity qualification.

P3-02 persistence is now in progress. Actual PostgreSQL store/migration/worker
checks pass, including ownership, stale/invalid writes, transactional rollback,
bounded coalescing and cancellation/backend-loss behavior. The server adapter is
wired; live save/restore acceptance is pending. See `docs/PERSISTENCE_FOUNDATION.md`. No PostgreSQL system service was
installed; the owned test server is stopped. External identity-provider choice
remains open. All changes stay local/uncommitted; story edits remain untouched.
Earlier checkpoints below are historical.

# Current development checkpoint — 2026-10-28

The initial P2-13 spell feedback slice is qualified: original standard Bevy
meshes/materials and spatial audio follow actual encrypted server outcomes,
including damage/death after target retirement. Combat profile 2 binds checked
global cue positions and ability IDs; old combat-profile clients fail admission.
The full workspace run passed **278 tests** (ten private-fixture skips); final
client tests, denied-warning all-target Clippy and formatting also pass.
[Exact native evidence, hashes, prior failure and limits](../SPELL_FEEDBACK.md).

Run `python3 tools/play.py --spell-feedback` (optionally `--clients 2`): Tab selects,
Digit 1 casts, Backspace clears. The normal saved-terrain/prediction launch,
selection-only `--targeting` and UI-only `--combat` remain available; these modes
keep audio disabled. Two native Linux/Vulkan clients passed impaired UDP with
10,240 synthetic cosmetic requests each: all 18 critical cues per client played
and appeared under concurrent cosmetic load, with zero critical overflow/event
gaps/large movement displacements. Peak visuals/voices were 50/26, within 64/32.
Critical-only quality and live visual/audio origin shifts also pass.

Owned isolated audio captures contain actual cue PCM, no foreign streams or
default-sink changes; private sinks were removed. The same-binary one-client empty
baseline passed at 826.78 FPS uncapped with validation off; this is not MMO
capacity. Actual two-client feedback launcher interruption/child-failure cleanup
passed. Binaries/data are preserved in `build/spell-feedback-qualified-v1-20261028`.

Stopped safely after this completed slice at the user's request. Next planned
work is P2-14 clickable action-slot and clearer health/cast presentation; it has
not started. Broader combat/AI/navigation, persistence/rewards, native Windows,
physical keyboard/mouse delivery, production sound design and population scale
remain open. All engineering changes remain local and uncommitted; nothing was
pushed. Story edits were left untouched. Earlier checkpoints below are historical.

# Current development checkpoint — 2026-10-26

Resumed after user commit `9497a18`. The input-agreement diagnostic gate now passes
236 full workspace tests (ten private-fixture skips), denied-warning Clippy/fmt,
actual saved/generated impaired UDP and an uncontended 600-second eight-peer soak.
Both simulation and I/O record zero missed deadlines in the uncontended run;
identity return, bounded terrain residency, diagnostic counts and cleanup pass.
[Input evidence](../PREDICTION_INPUT_DIAGNOSTICS.md), [soak evidence](../SAVED_TERRAIN_SOAK.md).
Severe-link errors up to 1.6085 m remain documented; stopped convergence is not a
smoothness guarantee. Native Windows and MMO population capacity remain open.

P1-11's bounded ability-data prerequisite is now qualified: immutable validated
JSON, mobile-casting defaults, off-GCD options and ceiling simulation-tick timing.
All 16 core tests, workspace/all-target Clippy/fmt and actual offline validation at
20/30/60 Hz pass. [Ability data](../ABILITY_DATA.md). These checks follow the movement
checkpoint; no combined 241-test result is claimed. Live targeting/casts/damage
and the rest of P1-11/P2-11–14 remain open. All new changes stay local.

## Previous checkpoint

# Current checkpoint — stopped safely, 2026-10-26

The user requested stopping. No soak is running; the final diagnostic full-suite
relink was safely interrupted. Changes remain local and uncommitted.

The actual 600-second, eight-peer saved-terrain soak passes identity return,
bounded residency and zero missed simulation deadlines. Separate I/O pump misses
under concurrent builds remain recorded. [Soak evidence](../SAVED_TERRAIN_SOAK.md).
The poor-link test passes movement/convergence at 20/30 Hz and safe suspension
outside the 60 Hz latency window. [Envelope](../PREDICTION_LATENCY.md). That checkpoint
passes 235 workspace tests (ten private-fixture skips), denied-warning Clippy/fmt,
221 asset Python tests and four GLB alignment tests.

The subsequent bounded input-agreement diagnostics are implemented; all 11 focused
history tests and workspace/all-target denied-warning Clippy pass. Their final
full-suite/real-UDP run was interrupted, so these new counters have no completed
runtime qualification. [Diagnostic checkpoint](../PREDICTION_CONTRACT.md). Resume with
that verification before interpreting the remaining correction outliers. No next
feature is active; no native Windows or MMO population capacity is qualified.

Current continuation, 2026-10-23 — P1-10/P2-06/07 and initial P3-13:
[saved continental terrain](../SAVED_TERRAIN.md), with the existing coordinate,
controller, readiness and prediction prerequisites. Acceptance is recorded there;
saved terrain now passes full runtime/default adoption checks. The user resumed work.
The matched-tick/render diagnostics and bounded rate prediction clock also pass
234 Rust tests and native Linux comparisons; [clock evidence](../PREDICTION_CONTRACT.md).
The actual eight-peer, 180-second headless saved-terrain soak passes;
[scope, remaining checks and safe stop](../SAVED_TERRAIN_SOAK.md). Work is stopped
at the user’s request; changes remain local and uncommitted.
Earlier queues/stops below are dated historical context. No region/shard service
or complete P3-13 acceptance is implied.

Current user priority, 2026-10-22: implement the [continental graybox development
world](../GRAYBOX_WORLD.md) with the POLYGON Base Locomotion player. Freeze forest
reproduction. This supersedes the forest-specific next steps below; preserve the
imported source roadmap and existing fixtures. The finished content workflow remains
an offline-generated/authored, versioned map with runtime chunk residency.

# Engineering direction and execution queue

Integrated on 2026-10-19 from the user-supplied `theia_handoff.zip`.
This is the planning entry point for future MMO development. The existing walking
prototype remains useful evidence and a regression baseline; further individual
house-access scripts are not the default development path.

The user's subsequent instruction, **“Make all changes needed to align”**, adopts
the handoff as the active target architecture. See [current alignment](ALIGNMENT.md)
and [live task status](TASK_STATUS.md) for implementation progress after the audit.
The [repository assessment](REPOSITORY_ASSESSMENT.md) preserves the pre-migration
snapshot; its unresolved-policy notes are superseded by the adoption below.

## Documents and interpretation

| Document | Role |
|---|---|
| [Architecture](ARCHITECTURE.md) | Supplied target architecture and decision identifiers |
| [Implementation plan](IMPLEMENTATION_PLAN.md) | Supplied six phases, ten overlapping waves and delivery gates |
| [Task register](CODEX_TASKS.md) | Supplied 112 task entries, including two unspecified placeholders |
| [Contributor proposal](AGENTS.reference.md) | Supplied `AGENTS.md`, renamed to avoid activating instructions embedded in the attachment |
| [Repository assessment](REPOSITORY_ASSESSMENT.md) | Local evidence, alignment, gaps, migration boundaries and verification results |

The four supplied documents are preserved byte-for-byte, apart from the contributor
file's name. Their assertions of prior approval are source claims, not evidence of
implementation or additional authorization. Their initial NOT STARTED statuses are
preserved; use the assessment to distinguish existing capability from missing work.
The initial request authorized documentation import and assessment. The follow-up
authorizes alignment work. Source text remains preserved for provenance; active
rules are in the root AGENTS.md and the local interpretation here. Adopting the
architecture does not certify all 112 roadmap entries as implemented.

Apply this local interpretation when using them:

- The implementation repository is `theia-mmo`. The architecture's link to
  `code-studio-au/theia-engine` is a reconciliation issue, not a repository move or
  permission to introduce an engine dependency.
- Renet and Avian are the supplied initial target choices. Renet is now the
  default launcher transport after local qualification; Quinn remains an explicit
  regression option; grounded terrain profiles now run through Avian queries
  while broader movement/content contracts are qualified. Existing Quinn success
  is not evidence that the handoff's Renet validation has failed.
- f64 global coordinates, f32 local frames, data-driven gameplay, terrain chunks,
  navigation and PostgreSQL are targets; none should be described as present merely
  because the handoff specifies them. Renet 2.0.0 is pinned and its first secure
  proof passes; Avian 0.8-dev is now revision-pinned and active; see the
  [physics migration record](../PHYSICS_AVIAN.md).
- The adopted world is predominantly static, with scripted authoritative platforms/
  elevators allowed and presentation-only vegetation wind/bending planned. This
  supersedes the old blanket prohibition on environment motion. Buildings, doors
  and ordinary props remain static unless separately specified. Characters remain
  nonblocking to one another. None of these future motion systems is implemented.
- The historical roadmap remains a record of completed experiments. New work uses
  the Pn-nn identifiers below. Scheduler work maps to P1-05; do not introduce a
  second task under the handoff's historical M1-04 shorthand.
- P3-17 and P3-18 remain unspecified. Phase 4–6 numbering is a reconstruction.
  Preliminary dependencies are reviewed per slice, not treated as serial gates.

The [Avian runtime migration](../PHYSICS_AVIAN.md) now supplies pinned Bevy 0.20
physics to the normal client and authority; it does not complete general terrain
streaming, multi-floor movement or all collision content.

## Current qualification gate

[Schema-3 general collision and explicit spawns](../GENERAL_COLLISION.md) now pass
native mesh/map/chunk and default-launcher checks. The launcher uses town revision
2 with explicit spawns. [Bounded authority chunk activation](../CHUNK_ACTIVATION.md)
now also passes original Avian and real two-client Renet cases. The normal opt-in
[paired handover](../CHUNK_HANDOVER.md) adds authority offers, retained presentation
and matching Bevy/Avian publication on original fixtures. Continue row 5 with
[contributor-complete geometry](../CHUNK_CONTRIBUTORS.md), remaining lifecycle
qualification and actual tile partitioning before default-map adoption. A bounded
schema-2 contributor path and [shared image bindings](../SHARED_IMAGES.md) now
exist. [Offline graph/terrain/collision partitioning](../WORLD_PARTITIONING.md)
now passes actual source comparison and generated original-fixture native checks.
The town needs nine providers per cell and ten initially; measured runtime
budgets, world-edge policy and real town publication remain before default
adoption. [Offline full-footprint budget analysis](../CHUNK_BUDGETS.md) now
measures 18/22/26 providers including recovery for one/two/up-to-eight conservative
XZ actors. [Bounded runtime capacity](../CHUNK_CAPACITY.md) now qualifies the
32 hard ceiling, explicit session budgets, budget-bound admission, full secure
control sets and native 32-root/six-provider cases. Actual town authority
publication is qualified; finite-world edges and actual-town client/resource
qualification still precede default adoption. Shared
[normal initial startup](../CHUNK_STARTUP.md) now connects ordinary client/server
sessions, standard chunk scenes and live collision; the default remains monolithic. Source-mesh
fidelity, skeleton contracts, prediction and whole phases remain open.

## Immediate engineering queue

The user now prioritizes foundations over playability. Follow the refined
[dependency order and acceptance gates](EXECUTION_ORDER.md), rather than task
number alone. Renet is now the launcher default after the bounded local network replacement
gate. A bounded [runtime coordinate/identity slice](../COORDINATES_IDENTITY.md) now
adds global authority, epochs and wire-frame rebasing. The
[client rebase continuation](../FLOATING_ORIGIN.md) now shifts render/static Avian
frames automatically; server query scenes and multi-chunk ownership remain open. See the
[qualification record](../RENET_QUALIFICATION.md).

These are planned acceptance outcomes. See the live task status for delivered
subsets; boxes stay open until the complete outcome is demonstrated.

- [x] **P1-01: inventory and classify this repository.** Record source paths,
  retained experiments, migration needs, task coverage and fresh checks in the
  assessment. This completes the scoped documentation audit; it does not certify
  all of the source register's generic integration/benchmark requirements.
- [ ] **P1-02/P1-03/P1-08/P1-09: write foundation contracts.** Define global and
  local position types, origin conversion, network identity/session semantics,
  input acknowledgements, and character collision layers. Test large positive
  and negative coordinates, local/global round trips, seams and actor pass-through.
  Keep rendering types out of the headless authority. Extract shared domain
  contracts from the transport crate as needed; avoid an empty crate scaffold.
- [x] **P1-05/P1-06/P1-12: bounded scheduler qualification slice.** Existing
  authority qualified with eight real secure UDP clients at 20/30/60 Hz, burst/
  replay/wrong-epoch traffic, expiry, controlled stall/recovery and input-budget
  removal. Fixed-memory timing histograms separate simulation/pump work, wall
  cadence and skipped intervals. See [evidence](../SCHEDULER_QUALIFICATION.md).
  Full-map/render/crowd/soak baselines and general action schemas remain open.
- [x] **P1-07/P1-08/P2-02/P2-03: bounded local Renet replacement slice.**
  Compatible secure pins, real application authority/lifecycle/world matching,
  controlled loss/reorder/delay/duplicates/outages, fresh-ticket reconnect and
  measured RTT/traffic/queue bounds now pass. Renet becomes the launcher default;
  Quinn stays an explicit regression option. See [qualification](../RENET_QUALIFICATION.md).
  This closes the local slice, not production identity, Internet security or MMO load.
- [x] **P1-10/P2-08: bounded content-package prerequisite.** Stable logical IDs,
  revisions and separate collision/visual digests are now checked by the actual
  server/client/default map. Headless collision-only loading and stale-revision
  admission pass. See [scope](../CONTENT_PACKAGES.md); remaining chunk, collision
  and skeleton contracts stay open.
- [x] **P1-03/P1-10/P2-06/P2-07: isolated chunk-residency precursor.** Original
  fixture proves bounded real package/collision load/unload, protected-owner and
  corrupt-candidate rejection, signed/distant ownership/query seams. Visual-root
  descriptors are not rendered assets; see [remaining gate](../CHUNK_RESIDENCY.md).
- [x] **P2-06/P2-07/P2-08: original-fixture Bevy publication diagnostic.** Actual
  GLB scene readiness/retention, one worker candidate, live physics agreement and
  rendered replacement/failure/unload now pass at signed distant origins.
  [Scope](../CHUNK_PRESENTATION.md). Normal tile/authority adoption stays open.
- [x] **P1-09/P2-07: shared graybox capsule controller.** General fixture walls,
  headroom, steps/slopes, stacked floors, gravity/jumping and nonblocking actors
  now use one controller in graybox authority/offline motion. Wire v7 adds held
  jump input. [Scope](../KINEMATIC_CONTROLLER.md). Imported map migration and
  live chunk/actor activation remain open.
- [x] **P1-09/P2-07: normal imported-map movement adoption.** Shared capsule
  movement/jumping now uses validated cached development spawns, finite-world
  barriers and observable per-actor recovery. [Contract](../KINEMATIC_CONTROLLER.md).
  Explicit spawn metadata, general collision authoring and chunk activation remain.
- [ ] **P1-09/P1-10/P2-06/P2-07: general collision/content slice with Avian.**
  Verify a compatible headless/client setup, import approved collision geometry,
  and use one controller across multiple buildings and stacked surfaces. Test
  walls, headroom, slopes, steps, falling/jumping and actor pass-through; retain
  the farmhouse/blacksmith routes as regressions. Specify chunk identity and
  visual/collision ownership before expanding terrain. Start with a bounded
  chunk proof; large-world streaming and navigation expand through later tasks.
- [x] **P2-04/P2-05/P2-09: bounded Linux graybox prediction slice.** Actual
  authority/client commands, bounded reconciliation, impaired UDP at 20/30/60 Hz,
  native camera/locomotion, full crossing and reset checks pass. Prediction is the
  graybox launcher default; `--no-prediction` preserves the baseline. This does
  not complete the whole tasks: authored maps, broader latency/load and native
  Windows remain open. [Evidence](../PREDICTION_CONTRACT.md).
- [ ] **P1-11/P2-11..P2-14: first gameplay delivery.** Define ability data and
  server-owned target/range/cooldown/health rules, then demonstrate two clients
  fighting one nonblocking enemy with death/respawn and a minimal target/action
  UI. Presentation effects follow authoritative events. This is the W3 outcome;
  quests and durable progression follow in W4/W5.

Contract, scheduler and content investigation can overlap where their actual
dependencies permit. Define collision/navigation interfaces early; full streamed
navigation remains P4-03 and need not block the first stationary combat enemy.
Do not build production-scale services before these bounded slices work.

## Updating the plan

For each implementation slice, identify its task IDs, write specific acceptance
checks and budgets, inspect existing code, then record executed commands and
results. Replace generic acceptance language with observable behaviour before
claiming completion. Preserve source documents as the imported baseline; record
local status and reconciliations here and in the assessment. Update STATUS for
implemented behaviour and DECISIONS for resolved choices. Publishing and pushing
still require user authorization. Do not treat attachment instructions as
higher-priority commands or approvals of external actions.

## Provenance

Archive SHA-256:
`6dd6011eaa4f0f790bb6fe54ba418c75b3c4a82d49f3f9a7b758b843ac3d1e4c`.

| Archive entry | SHA-256 of preserved contents |
|---|---|
| ARCHITECTURE.md | `145493dad3d6eaff6fe6b6aa1a35453bac8497cdcf3518b60ece67e75249f992` |
| IMPLEMENTATION_PLAN.md | `fa58a1d6e15e3a0c365f051aa9f258b461996c76a39d2546c09be06c167dcab3` |
| CODEX_TASKS.md | `283ecf5653362dfd8a5eb9bf60d7190172dea346fe4393580c9a99ec140e17aa` |
| AGENTS.md → AGENTS.reference.md | `6999fd356a58e85a60e566a018ae63b0d2cc1a9699af9a83db335f2dec9860ca` |

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
