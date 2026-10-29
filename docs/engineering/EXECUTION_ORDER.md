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

User-directed current slice, 2026-10-22: [continental graybox](../GRAYBOX_WORLD.md),
P1-03/09/10/12 and P2-06/07/08/09. Actual prerequisites are checked global/local
coordinates, secure authority, the Avian controller and the existing POLYGON
conversion. Acceptance: 16 km extent, seams, bounded asynchronous visual/collision
residency, moving frames, full crossing/return, teleport/revisit, actual animation,
culling/unloading and measured release GPU behavior. Forest fidelity work is frozen.
This slice does not complete the source tasks or qualify MMO population capacity.

The bounded Linux terrain and prediction slices now pass; see
[TASK_STATUS](TASK_STATUS.md) and the [prediction evidence](../PREDICTION_CONTRACT.md).
The user requested stopping after completion. Do not start another slice as part
of the repository alignment/commit. Broader task acceptance remains open.

# Foundation execution order

Adopted working order, 2026-10-19. The user currently prioritizes engineering
foundations over playability. Use this document to select work; use
[TASK_STATUS](TASK_STATUS.md) to record delivery. The 112 imported entries and
historical experiments remain intact. No entire phase is complete.

The source register chains many tasks to the preceding number. Those chains are
preliminary: for example, instrumentation does not require combat schemas, and
network session validation does not require terrain streaming. The table below
states actual prerequisites for bounded implementation slices. A delivered slice
closes its own acceptance checks, not every task ID it touches.

| Order | Slice / source IDs | Actual prerequisites | Acceptance before proceeding |
|---|---|---|---|
| 1 | Real application Renet bridge: P1-06/07/08, P2-02/03 | Existing fixed-step authority, bounded wire/intents, secure UDP proof and working Bevy bridge | Real server and two Bevy clients on Renet; world matching, authoritative poses/acks, distinct IDs, departure/fresh-ticket reconnect; invalid intent/ticket/world rejection. Keep Quinn default until row 2 passes. |
| 2 | Renet qualification/default switch: P1-06/07/12, P2-02/03 | Row 1 | Repeatable controlled loss, delay, duplication/reorder, outages and reconnect; measured bandwidth/RTT/queue bounds; stale commands cannot move a replacement session; rerun map regressions and launcher cleanup. Record exact tested profiles and limits before default switch. |
| 3 | Live coordinate and identity contracts: P1-02/03/04/08 | Existing checked coordinate types and admitted-session contract | Checked global truth through authority/wire/client, bounded local frames, explicit entity/session lifetime and region/chunk identity. Positive/negative large coordinates, local rebases, chunk seams, actor removal/reconnect and stale-session traffic tests; no renderer dependencies in server. |
| 4 | Scheduler and command qualification: P1-05/06/12 | Existing configurable scheduler and rows 1–3 contracts | Loaded 20/30/60 Hz runs with explicit actor/traffic counts, tick percentiles/lateness/deadlines, bounded queues, input expiry and overload recovery. Separate CPU simulation, wall-clock scheduling and rendered network evidence. |
| 5 | Content and general controller: P1-09/10, P2-06/07/08 | Coordinate/chunk ownership from row 3; pinned Avian already active | Versioned stable content/collision identity; bounded chunk load/unload proof; shared kinematic movement for walls, headroom, steps, slopes, stacked floors, falling/jumping and nonblocking actors. Retain authored house routes as regression fixtures. |
| 6 | Prediction and presentation: P2-04/05/09/10 | Qualified input/ack contract and shared movement from row 5 | Bounded history/replay, corrections under row 2 network profiles, camera and animation follow corrected motion; native Linux evidence and separately qualified Windows support. |
| 7 | Ability schemas and first authoritative gameplay: P1-11, P2-11–14 | Identity/lifecycle, commands, controller and corrected presentation | Data validation and server target/range/cooldown/health authority; mobile casting rules; death/respawn and UI/VFX/audio derived from server events. |
| 8 | Persistent zone and later phases | Relevant foundation gates above | Refine actual dependencies and concrete checks from Phase 3 onward before implementing. External identity provider and durable economy require their own design/validation; no premature distributed-service scaffold. |

Rows 3 and 4 may overlap once their specific interfaces are stable. Content
investigation may also overlap; crossing a gate still requires executed evidence.
Early navigation contracts belong with content ownership; full tiled navigation
remains P4-03. Existing camera/art can be reused throughout. Neither gameplay nor
more manually opened buildings is the next acceptance gate.

## Current slice

Row 1 supplies the executable path through `tools/play.py --transport renet`.
It retains existing f32 fixture poses and wire v5, eight-peer capacity and local
snapshots. Local tickets are a development issuer, not external account login.
See [Renet bridge](../NETWORKING_RENET.md) for behavior, evidence and limitations.
Row 2's bounded local replacement gate now passes: deterministic impaired UDP
profiles, measured RTT/traffic/queue bounds, outages and captured old-session
packet replay. Renet is the default for the launcher and multiplayer qualifier;
Quinn remains an explicit regression path. See [qualification](../RENET_QUALIFICATION.md).
This closes those local slice gates, not full production transport/session scope.
Row 3 now has a bounded [runtime coordinate/identity slice](../COORDINATES_IDENTITY.md):
checked global authority, epoch/origin wire v6, live relative-wire rebases and
fixed-anchor presentation at distant origins. The [client rebase continuation](../FLOATING_ORIGIN.md) now recentres render and
static Avian worlds together. Server query scenes remain in the fixed content
frame; general multi-chunk ownership is open. Refine those remaining interfaces
before claiming the entire row complete; row 4 scheduler
qualification can proceed where these existing interfaces suffice.

Row 4's bounded scheduler gate now has a real eight-peer graybox qualification at
20/30/60 Hz, with explicit burst traffic, bounded timing histograms, expiry,
controlled authority stall/recovery and input-budget removal. See
[scheduler qualification](../SCHEDULER_QUALIFICATION.md). Larger populations,
full-map/crowd/render/soak baselines and action schemas remain open. Continue with
row 5's shared content/collision and general controller contracts; preserve the
existing map fixtures and regressions.

Row 5's first prerequisite now has [versioned bounded content packages](../CONTENT_PACKAGES.md):
stable logical IDs/revisions, independent artifact binding, headless collision
verification and normal map adoption. Continue with chunk visual/collision
ownership and bounded residency/load/unload before general movement. This is a
partial row, not completed chunk streaming or controller acceptance.

An isolated [chunk-residency precursor](../CHUNK_RESIDENCY.md) now proves bounded
synchronous package/collision loading, unload effects, protected owners and signed/
distant seams on original geometry. Visual descriptors remain unconsumed; add a
bounded standard-Bevy visual/publication proof before normal tile adoption. Do
not count this as asynchronous streaming or completed P2-06/general controller.

The [actual-client original-fixture diagnostic](../CHUNK_PRESENTATION.md) now passes
standard Bevy GLB readiness/retention and publication with live Avian, including
rendered replacement, failed-asset rollback and unload. This closes the bounded
visual-consumer proof; normal tile/authority adoption and moving-actor activation
are still open. Continue row 5 with shared general kinematic movement so live
chunk activation can be qualified against its real collision/grounding needs.

A [shared capsule controller](../KINEMATIC_CONTROLLER.md) now drives graybox
authority and offline movement, with actual Avian fixture coverage for walls,
headroom, steps/slopes, stacked floors, falling/jumping and nonblocking actors.
Wire v7 carries a held jump level. The imported map still uses its legacy walker;
continue row 5 by migrating its collision/controller contract and qualifying live
actor/chunk activation before prediction. This bounded controller slice does not
close row 5, P1-09 or P2-07.

Normal imported-map authority now also uses the shared controller. Trusted cached
spawns, finite-world-only collision barriers and observable per-actor fall/invalid-
state recovery replace the old highest-surface walker in ongoing simulation.
Individual chunk scenes remain open at their seams. Continue row 5 with explicit
spawn/general collision authoring contracts and live moving-actor chunk activation;
normal streaming and prediction remain open. See [movement contract](../KINEMATIC_CONTROLLER.md).

Safe-stop checkpoint: [schema-3 collision and explicit spawns](../GENERAL_COLLISION.md)
are implemented with authoring/migration tooling and staged revision-2 maps.
The user requested stopping before native qualification/default adoption. Resume
with actual Bevy mesh/map/chunk regression checks; retain the existing schema-2
launcher default until those pass. Then continue moving-actor chunk activation.

The schema-3 native/default gate now passes (2026-10-20): original mesh and map
motion/camera, signed distant frames, spawn-free chunk lifecycle/rollback and
actual launcher default package/cleanup. Default town is revision 2 with explicit
spawns. Continue row 5 with moving-actor chunk activation, then normal residency
adoption; prediction follows its acceptance gate. [Scope](../GENERAL_COLLISION.md).

## Moving-actor authority activation continuation — 2026-10-20

[Bounded authority activation](../CHUNK_ACTIVATION.md) now connects residency to
actual fixed-step actors and the real secure Renet loop. Publication revalidates
current footprints/support/clearance and epoch/generation, retaining active state
on failure. One owner-side transaction prepares collision off-tick; unknown
neighbors hold horizontal motion while preserving jump/gravity. Original Avian
fixtures and actual two-client secure UDP checks pass. Normal development map
selection remains monolithic. Continue row 5 with paired Bevy/authority residency
and actual tile ownership/partitioning; prediction follows that acceptance gate.

## Shared normal initial chunk startup — 2026-10-20

[The shared startup descriptor](../CHUNK_STARTUP.md) now connects normal executable
server/client startup, trusted global spawns, initial standard GLB scenes, live
Avian/camera and actual encrypted sessions. Client and server query frames may
differ without changing physical content; the initial selection is bound in the
version-2 session fingerprint to reject mismatched initial-only clients. Normal
loading remains initial-only and the launcher remains the monolithic town. Next:
paired dynamic readiness/publication, then actual tile geometry ownership and
partitioning with measured budgets. Do not infer complete streaming from these
bounded original fixtures or move to prediction before the row-5 adoption gate.

## Per-client chunk readiness — 2026-10-20

Row 5 now has [initial per-client readiness](../CHUNK_READINESS.md) on wire/ALPN 8.
The actual normal Bevy client waits for assets and live Avian before reporting;
authority gates each actor's horizontal movement independently. Real UDP tests
cover server replacement, delayed client release and replay/malformed reports;
the peer in that replacement test is headless. Native qualification covers normal
initial scenes, distant/query frames, invalid assets and monolithic regressions.
Continue with acknowledged withdrawal and retained presentation, then paired
dynamic publication and explicit geometry contributors/tile partitioning. Normal
rendered chunk loading remains initial-only; no row or whole task closes here.

## Acknowledged readiness continuation — 2026-10-20

P1-06/P1-08/P2-06/P2-07 now have exact epoch/sequence/set acknowledgements emitted
after a fixed simulation step. Authority coalesces one pending reply per peer;
the client confirms only the latest report with a post-application snapshot.
Real UDP withdrawal/regrant tests preserve independent actor progress and input
acknowledgements. The normal Bevy capture/verifier consumes the same barrier.
This supplies the authority half of retirement, not actual rendered retirement.
Continue the [paired publication sequence](../CHUNK_READINESS.md), protecting
local/remote interpolation, recovery spawns and gravity-driven boundary motion
before unloading. Native scope and checks are in [validation](../VALIDATION.md).

## Continuous loading boundaries — 2026-10-20

P1-09/P2-06/P2-07 now constrain capsule sweeps to bounded authority/client
coverage, including gravity projections, step-up attempts, missing corners and
disconnected gaps. Capsule-first support uses sole validation/fallback for
imprecise convex contacts while retaining exact persistent wall contacts.
[Contract](../CHUNK_BOUNDARIES.md). Native edge pressure and existing house/map
regressions remain required evidence, not a streaming-capacity claim.

Continue row 5 with actual client retention of local poses, remote interpolation
endpoints and recovery spawns. Pair that coverage with the acknowledgement
barrier, handle newly arriving remote poses while assets lag, then connect
bounded standard-Bevy/Avian publication to authority offers. Actual tile
contributors/partitioning and wider budgets remain before normal adoption.

## Paired normal-client handover — 2026-10-20

P1-06/P1-08/P1-09/P2-06/P2-07/P2-08 now have an opt-in normal executable path:
authority lookahead and reliable generation offers, one hidden standard-Bevy
candidate, exact retirement acknowledgement, retained local/remote/recovery
coverage, matching Avian replacement and live generation readiness. Original
three-chunk fixtures exercise delayed preparation, invalid replacement rollback
and independent distant query frames. [Contract](../CHUNK_HANDOVER.md).
Current executed qualification, including the sloped-wall contact regression
found by the wider suite, is recorded in [status](../STATUS.md) and
[validation](../VALIDATION.md); implemented behavior alone is not acceptance.

Continue row 5 by specifying geometry contributors before importing actual tile
streaming. Schema-1 package geometry may extend beyond its owner cell, so owner
lookahead alone cannot prove collision completeness. Acceptance must cover a
building/mesh spanning cell edges, arrival from either side and a diagonal corner,
matching visible/collision contributors, candidate rejection, delayed peers and
retirement safety. Bind contributor metadata into dataset identity and validate
it against actual payloads; retain the qualified monolithic map until real tile
partitioning and memory/load budgets pass. Native reversal/superseding offers and
asymmetric failed assets remain explicit handover checks, not implied by the
original three cases. Prediction remains the next major foundation after the
relevant row-5 gates; no entire task or phase is closed here.

## Geometry contributor continuation — 2026-10-20

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08 now implement schema-2 bound package extents,
complete-cell coverage/demand/readiness and actual collider/Bevy hierarchy extent
checks. Original nonlocal-wall fixtures cover straight/diagonal entry, independent
frames, protected presentation/recovery and invalid extent retention. Current
executed checks are recorded separately in [status](../STATUS.md).
[Contract](../CHUNK_CONTRIBUTORS.md). The town remains monolithic.

The actual town inventory is 704.1 MB, including 698.7 MB of embedded images.
Before real partitioning, bind shared image resources and validate immutable
relative paths/hashes for standard Bevy loading. Then split actual terrain and
matching collision at cell edges, retain model transforms/contributors, and
measure resident/staging/load budgets. Keep whole task/phase and production
streaming claims open; refine the four-package proof budget only against actual
content requirements rather than duplicating the town's image payload per chunk.

## Shared image continuation — 2026-10-20

P1-10/P2-06/P2-08 implement manifest-v2 shared PNG bindings with canonical
colour-space/sampler variants and normal Bevy loading. The offline extractor
preserves source geometry and PNG bytes, removes image payloads from new GLBs
and publishes a new immutable representation. The actual town stage is a
5.48 MB GLB plus 488 shared images; it remains monolithic geometry and is not
promoted. [Contract](../SHARED_IMAGES.md). Current executed checks remain separate
in [status](../STATUS.md) and [validation](../VALIDATION.md).

Continue row 5 by pruning each package's real node/mesh/material/texture graph
and partitioning actual terrain and matching collision at cell edges. Preserve
model transforms, UVs and contributor completeness. Measure resident/staging
bytes, required provider counts and loading behavior before changing the default
map or proof budgets. Keep reversal/supersession/asymmetric-failure checks open;
prediction remains after the relevant content/controller gates rather than a
claim that this packaging slice completes a phase.


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


## Actual-content budget analysis — 2026-10-20

P1-03/P1-10/P2-06/P2-07/P2-08 now have verified package/image/geometry cost
analysis and conservative complete-footprint/recovery/lookahead unions. The
finite town reaches 18 providers for one capsule corner, 22 for two actors and
26 for the bounded actor set. All-indexed content has 4,001 colliders and an
estimated 1.51 GB base RGBA8 image payload; actual residency/loading is unmeasured.
[Evidence and concrete runtime acceptance](../CHUNK_BUDGETS.md).

Continue by implementing and qualifying a 32-package hard ceiling with a
26-package town session, checked control-message addresses and size bounds,
budget-bound admission, explicit world-edge policy and real multi-root staging/
retirement. This is a proposal, not a runtime cap change. Preserve the small
fixtures, actual map regressions and 30 Hz default. Then qualify real town
loading/resources and remaining handover lifecycle cases before promotion.


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

## Synty asset branch: model identity/source completeness — 2026-10-20

P1-10/P2-08: checked source-derived IDs now match 1,109 distinct Elven asset/fileID
references, including a legacy explicit metadata table. However, the package has
32 Git LFS pointer payloads, and the demo closure reaches 26 missing FBXs plus
one missing PNG. Eighteen other override-target IDs remain unmapped. Resolve
these and qualify actual hierarchy/material conversion before claiming full scene
import or adopting this map. [Evidence and acceptance](../SYNTY_ELVEN_IMPORT.md).
This asset-specific source gap does not block row 5's core finite-world, actual-
town client/resource and handover-lifecycle work. No task/phase is complete.

## Explicit finite-world bounds — 2026-10-20

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08 add schema-2 chunk-session rectangles,
admission identity v4, conservative local planes in shared movement and clamped
lookahead. Legacy session identities and open internal seams remain. Real Avian
edge/corner/jump/retreat and finite-world fall recovery qualify at 20/30/60 Hz;
source-town bounds are retained in a separate opt-in headless stage.
[Contract and scoped evidence](../WORLD_EDGES.md).

Continue row 5 with actual-town client preparation/staging memory and latency,
then reversal/supersession/asymmetric-failure lifecycle checks and map regressions
before promotion. Finite-world original-fixture tests do not establish the full
town perimeter's traversability, rendered resource budgets or MMO capacity. Keep
the monolithic launcher at 30 Hz. No complete task/phase or Windows claim.

## Actual-town startup/resource checkpoint — 2026-10-20

P1-10/P1-12/P2-06/P2-07/P2-08 now qualify ten-package and all-26-package
normal two-client startup on RX 7900 XTX/RADV Vulkan. A measured failure corrected
offline authoring to use transformed local mesh AABBs, matching unchanged runtime
validation; package/image bytes remain untouched in a new index stage.
Per-client RSS/DRM VRAM and approximate ready-marker times are sampled, with
scope/limitations in [the resource record](../TOWN_CLIENT_RESOURCES.md).

Continue row 5 with actual live candidate preparation/staging/publication,
retained shared-image/resource lifetimes and cleanup, then remaining reversal/
supersession/asymmetric-failure checks and map regressions before promotion.
All-index startup does not establish ten-to-26 live client replacement or
maximum staging memory. Keep the monolithic launcher and 30 Hz default.

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

## Interactive performance gate — 2026-10-22

P1-12 / P2-05 / P2-08 / P2-10. Require representative optimized, native frame-time
measurements before scene adoption. Correctness runs with validation, functional
screenshots and asset counts do not establish acceptable performance. Preserve
legacy camera regressions and use the existing accelerated CAMERA query geometry
for normal chunk worlds. Keep explicit debug/visual-refinement modes for diagnosis.
Current measured scope and pending checks are in [the performance record](../FOREST_PERFORMANCE.md).
