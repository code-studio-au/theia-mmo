# Current checkpoint — stopped safely, 2026-10-26

The user requested stopping. No soak is running; the final diagnostic full-suite
relink was safely interrupted. Changes remain local and uncommitted.

The actual 600-second, eight-peer saved-terrain soak passes identity return,
bounded residency and zero missed simulation deadlines. Separate I/O pump misses
under concurrent builds remain recorded. [Soak evidence](SAVED_TERRAIN_SOAK.md).
The poor-link test passes movement/convergence at 20/30 Hz and safe suspension
outside the 60 Hz latency window. [Envelope](PREDICTION_LATENCY.md). That checkpoint
passes 235 workspace tests (ten private-fixture skips), denied-warning Clippy/fmt,
221 asset Python tests and four GLB alignment tests.

The subsequent bounded input-agreement diagnostics are implemented; all 11 focused
history tests and workspace/all-target denied-warning Clippy pass. Their final
full-suite/real-UDP run was interrupted, so these new counters have no completed
runtime qualification. [Diagnostic checkpoint](PREDICTION_CONTRACT.md). Resume with
that verification before interpreting the remaining correction outliers. No next
feature is active; no native Windows or MMO population capacity is qualified.

Current continuation, 2026-10-23: [saved height terrain](SAVED_TERRAIN.md) passes
its bounded Linux adoption checks. Same-tick diagnostics and the smoother prediction
clock pass. The actual eight-peer, 180-second headless saved-terrain soak also
passes; [scope and remaining checks](SAVED_TERRAIN_SOAK.md). Work is stopped at the
user’s request, with changes local and uncommitted. The saved slice
extends the existing continental loader (P1-10/P2-06/07, initial P3-13). Preserve
procedural and imported-map regressions. Zone ownership, sharding, navigation,
terrain editing/materials and population capacity remain separate work.

Current priority, 2026-10-22: [256 km² graybox terrain](GRAYBOX_WORLD.md) and the
POLYGON Base Locomotion development player. Forest reproduction is frozen. Complete
and qualify bounded visual/collision generation, culling/unloading, long-distance
frames and authoritative traversal before returning to later gameplay milestones.
This is one terrain foundation slice, not completion of P1/P2 or MMO-scale capacity.
That traversal slice now passes. P1-06/08 and the bounded P2-04 continuation now
include actual Renet prediction/reconciliation, impaired-UDP and native Linux
traversal/reset checks. The graybox launcher adopts prediction and retains
`--no-prediction`; broader latency/load/Windows qualification remains open.
See the [prediction contract](PREDICTION_CONTRACT.md).

# Roadmap

Forest performance correction, 2026-10-22: the reported ~10 FPS walkthrough
used unoptimized binaries and redundant render-mesh camera scans. The launcher
now uses release binaries; chunk cameras use existing accelerated collision
geometry while legacy maps retain visual refinement. The unchanged full scene
measures 99.65 FPS uncapped at 1280×720 on the Linux RX 7900 XTX, versus 64.90 FPS
with optimized visual refinement. The reopened desktop window averages 57.89 FPS
with normal VSync, with frame-time spikes retained in the evidence. All 45 client
tests, denied-warning workspace/all-target Clippy, fmt/diff and 221 asset Python
tests pass. This is one machine/view, not MMO capacity or every-route performance.
Broader quality/performance gates remain. [Evidence](FOREST_PERFORMANCE.md).

Forest normal-runtime continuation, 2026-10-21: ten bound material packages
preserve complete source geometry/collision; independent comparisons, four
rehashed faults, all 804,029 source rays and glTF validation pass. A separate
candidate excludes exactly one invisible water collider. Two real encrypted
Linux/Vulkan clients load the full scene, share four projection materials and
47 actual images, and pass visibility/readiness/rebase checks. Per-client base
RGBA CPU image data is 1,589,649,408 bytes (not total memory). The actual-map
survey records 27 clear starts and 41 rejected routes; movement, lighting,
mips/resources, water and broader camera/adoption remain open. Town stays the
default. All work stays local. [Evidence and limits](FOREST_RUNTIME.md).

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
open. All work stays local. [Contract and evidence](RUNTIME_MATERIALS.md).

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
[Evidence and scope](TRIPLANAR_PHASE.md).

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
[Packages](FOREST_SHAPE_PACKAGES.md), [standard materials](STATIC_MATERIALS.md).

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
All work stays local. [Contract and evidence](STATIC_SHAPES.md).

Forest mixed-material continuation, 2026-10-21: nine crystal/geode bindings /
25 draws join the qualified subset: **18,650 source draws / nine unresolved**.
Actual Editor submesh triangles prove source-slot fidelity; four differing quad
diagonals use Editor topology with original corner normals/UVs. Crystal appearance
is an explicit static standard-PBR preview, with refraction/view effects omitted.
Full composition, nine native Vulkan cases, five rebound faults and 19 Khronos
files pass (zero errors/seven retained warnings). Fresh fmt, denied-warning Clippy
and 189 Python tests pass. Water/sky/emissive willow, lighting, rebase/resources/mips,
collision and adoption remain open. All work stays local.
[Evidence](FOREST_MIXED_MATERIALS.md).

Forest glass continuation, 2026-10-21: an explicit untextured Standard-to-glTF
BLEND adaptation adds the lantern glass: **18,625 source draws / 34 unresolved**.
Source geometry/material, actual Editor frame, exact placement/full composition
and four native Vulkan cases pass. The adaptation records Unity premultiplied
lighting/colour-space differences; no runtime shader/binary change is introduced.
Fresh fmt, denied-warning Clippy and 181 Python tests pass. Next: actual mixed
submesh-slot coverage for 25 crystal/geode draws. Remaining materials, lighting,
rebase/resources/mips, collision and adoption remain open. All work stays local.
[Evidence](FOREST_GLASS.md).

Forest colour-UV continuation, 2026-10-21: two exact giant-tree bindings /
seventeen placements now join the qualified visual subset: **18,624 source draws /
35 unresolved**. A second standard glTF channel preserves leaf colour tiling while
normal/AO retain UV0. Source/Editor/exact-placement/full composition checks and six
native Vulkan cases pass; actual loaded materials select the expected channels.
Legacy rendering and encrypted town/farmhouse regressions pass on the new client.
Fresh fmt, denied-warning Clippy, 35 client Rust and 179 Python tests pass. Remaining
materials, rebase/resources/mips/lighting, collision and adoption stay open. All work
is local. [Evidence](FOREST_COLOUR_UV.md).

Forest texture-default continuation, 2026-10-21: actual Linux Unity 6000.6.6f1
resolves four Grass/Mud textures to bilinear/repeat with original PNGs/metadata
unchanged. The ten grass draws now join the qualified subset: **18,607 source
draws / 52 unresolved**. Fresh eighteen-binding source/Editor/placement checks,
four combined native Vulkan views, three grass modes and 274 pixel checks pass.
Fresh fmt, denied-warning Clippy and 175 Python tests pass. Runtime binaries remain
unchanged. Next: independent leaf-colour UV tiling for seventeen giant-tree draws;
remaining materials, rebase/mips/resources, collision and adoption remain open.
All work stays local. [Evidence](FOREST_TEXTURE_DEFAULTS.md).

Forest triplanar batch continuation, 2026-10-21: seventeen new bindings / 518
placements now join the corrected visual subset: **18,597 source draws / 62
unresolved**. Fresh independent source/Editor/placement and full composition checks,
four combined native Vulkan views, three dirt-profile modes and 304 pixel checks
pass. Three profiles share eight extension images; 31 Standard image variants
remain distinct. Khronos: 35 files, zero errors, seven runtime-tangent warnings.
Fresh fmt, denied-warning Clippy and 173 Python tests pass. Runtime binaries stay
unchanged; collision/rebase/mips/resource and adoption gates remain. Next: actual
Editor sampler defaults for ten grass draws. All work stays local.
[Evidence and limits](FOREST_TRIPLANAR_BATCH.md).

Forest P1-10/P2-08/P2-10 catalog continuation, 2026-10-21: actual mixed Standard/
two-profile rendering, shared images, negative mapping cases and independent pixel
checks qualify schema 2 while retaining schema 1. Batch remaining eligible
triplanar bindings and compose the full visual scene next; rebase texture phase,
resources/mips, remaining materials, collision and adoption remain.
[Contract and evidence](FOREST_MATERIAL_CATALOG.md).

Forest P1-10/P2-08 mesh-frame continuation, 2026-10-21: thirteen placements are
corrected in a new source-preserving subset; all 116 prototypes, full scene,
native Vulkan and retained multiplayer regressions pass. The corrected moss
prototype/all-85 group also qualifies with independent pixel checks. Continue
multiple triplanar profiles, rebased texture phase and resource/mip integration,
remaining shaders/slots, then object collision and runtime adoption. Main subset:
18,079 draws / 580 unresolved. [Scope and evidence](FOREST_MESH_FRAMES.md).

Forest P1-10/P2-08 checkpoint: actual Linux Editor references cover all 112 scene
FBXs / 214 meshes; independent recorded-frame point sets match with byte-identical
metadata. Thirteen current batch draws need converter frame correction. The user
requested a safe stop before that work. Resume with source-bound corrections,
independent replacement geometry/placement checks and native regressions before
promotion. Existing town/forest-ground remain available.
[Acceptance and evidence](FOREST_UNITY_REFERENCE.md).

As of 2026-10-19, the [engineering plan](engineering/README.md) is the entry point
for future work. It incorporates the supplied architecture, six-phase plan and
112-entry task register. Follow the refined [foundation execution order](engineering/EXECUTION_ORDER.md),
which prioritizes dependencies and engineering evidence over playability. It includes a [repository assessment](engineering/REPOSITORY_ASSESSMENT.md)
and immediate foundation queue. The user has adopted the handoff architecture;
the [alignment record](engineering/ALIGNMENT.md) and [live task status](engineering/TASK_STATUS.md)
separate delivered foundations from remaining migrations and game features.

- [ ] Import the user-selected [Enchanted Forest](SYNTY_ENCHANTED_FOREST_IMPORT.md)
  in place of Elven Realm: source audit delivered; material/prefab conversion,
  native scene and authoritative collision qualification precede map adoption.
- [x] Shared bounded chunk startup metadata and opt-in normal executable loading:
  ordinary two-client sessions, GLB roots, live Avian/camera and explicit client
  query frames. Initial selection is bound into admission. [Scope](CHUNK_STARTUP.md).
- [x] Initial per-client chunk readiness on secure Renet, with bounded control
  messages, independent movement gates and actual Bevy/live-Avian startup checks.
  [Contract and remaining publication gate](CHUNK_READINESS.md).
- [x] Exact post-simulation readiness acknowledgements and a client confirmation
  barrier tied to the latest report and a post-application snapshot. Rendered
  retention and paired replacement remain open. [Contract](CHUNK_READINESS.md).
- [x] Shared coverage sweep constraints for input and gravity-driven motion,
  missing-corner/gap protection and validated support fallback. Native loading-edge
  pressure and retained map regressions pass. [Scope](CHUNK_BOUNDARIES.md).
- [x] Bounded paired normal-client handover on original fixtures: authority
  offers, retained presentation/recovery coverage, exact retirement barriers,
  standard Bevy roots and live Avian generations. Delayed preparation, failed
  replacement retention and distant frames qualify. [Scope](CHUNK_HANDOVER.md).
- [x] Bounded contributor-complete geometry: schema-2 bound package extents,
  real collider/Bevy validation and shared coverage/demand/readiness. Original
  straight/diagonal nonlocal geometry and extent failures qualify.
  [Scope](CHUNK_CONTRIBUTORS.md).
- [x] Bound shared PNG variants and standard Bevy handle reuse: immutable
  dependencies, actual image format/sampler checks, failed-dependency retention
  and source-preserving extraction of the actual town. [Scope](SHARED_IMAGES.md).
- [x] Offline actual terrain/collision partitioning, whole model ownership and
  per-package graph pruning: independent source comparison and all 26 Rust
  package checks pass; two generated original-fixture native cases qualify the
  format. [Scope and measured provider requirements](WORLD_PARTITIONING.md).
- [x] Offline provider/resource budget analysis: source-verified recovery and
  actual package/image costs, complete capsule corners and bounded actor/
  lookahead unions. [Measured 18/22/26-provider requirements](CHUNK_BUDGETS.md).
- [x] Bounded 32-package runtime capacity, budget-bound admission and wire/ALPN
  11: actual secure UDP, 32 native roots, six-provider delayed publication and
  opt-in ten-to-26-package town authority checks. [Scope](CHUNK_CAPACITY.md).
- [ ] Qualify actual-town runtime residency/staging resources and world-edge policy,
  remaining handover lifecycle checks and real town adoption. The current
  town authority fits the new budget; client resources and adoption remain open.
  The normal launcher stays monolithic until the runtime adoption gate passes.
- [x] Explicit finite chunk-world bounds: admission identity v4, conservative
  local boundary sweeps, bounded lookahead, edge/corner motion and fall recovery.
  Native original fixtures and opt-in town authority qualify the bounded slice;
  actual-town client resources and adoption remain open. [Scope](WORLD_EDGES.md).
- [x] Bounded live-publication diagnostics, resolved recovery feet and exact decimal
  identity parsing; staged-candidate cleanup and retained map regressions pass.
  Broader lifecycle cases and adoption remain open. [Scope](TOWN_LIVE_PUBLICATION.md).
- [x] Actual town ten/all-26-package two-client startup and sampled process/DRM
  resources, with corrected conservative offline visual envelopes. Live staging/
  retirement and adoption remain open. [Measurements](TOWN_CLIENT_RESOURCES.md).

- [x] Bounded moving-actor authority chunk activation: current capsule/support
  checks, stale candidate rollback, conservative unloaded-boundary movement and
  background loading in the actual two-client Renet loop. Normal paired client/
  authority residency and tile partitioning remain. [Scope](CHUNK_ACTIVATION.md).

- [x] Native qualification/adoption of schema-3 general collision and explicit
  spawns: original mesh and imported map cases, distant frames, spawn-free chunk
  publication/rollback, startup rejection and actual default launcher package/
  cleanup pass. [Scope](GENERAL_COLLISION.md). Moving-actor activation remains.

- [x] Normal imported maps now use shared capsule movement/jumping, validated
  cached development spawns, finite-world barriers and observable per-actor fall/
  invalid-state recovery. [Scope](KINEMATIC_CONTROLLER.md). Live chunk/actor
  activation and normal residency adoption remain open.

- [x] Shared general capsule controller in graybox authority/offline sandbox:
  actual Avian walls, headroom, steps, slopes, stacked floors, gravity/jumping and
  nonblocking actors; wire v7 jump input. [Scope](KINEMATIC_CONTROLLER.md).
  Live chunk/actor activation remains open.

- [x] Real Bevy original-chunk publication diagnostic: one background candidate,
  standard GLB hierarchies, live Avian body/BVH agreement, retained roots and
  screenshot-proven replacement, failed-asset rollback and unload. [Scope](CHUNK_PRESENTATION.md).
  Normal tile partition/adoption, actor activation and production streaming remain.

- [x] Isolated bounded chunk-residency precursor on original geometry: signed
  ownership seams, real collision load/unload, protected-owner and corrupt-package
  rejection, and distant query frames. [Scope](CHUNK_RESIDENCY.md). Normal tile
  partitioning, client visual consumption and asynchronous streaming remain open.

- [x] Bounded content-package identity slice: versioned manifest, stable logical
  IDs/revisions, independent collision/visual verification and stale-revision
  admission rejection. [Contract](CONTENT_PACKAGES.md). Chunk ownership/residency,
  general collision and character/skeleton contracts remain open.

- [x] Bounded eight-peer Renet scheduler qualification at 20/30/60 Hz, input
  coalescing/expiry/budget enforcement, controlled stall/recovery and bounded
  simulation/pump timing diagnostics. [Evidence](SCHEDULER_QUALIFICATION.md).
  Full-map, crowd, action-command and sustained-overload capacity remain open.

- [x] Automatic client render/static-Avian rebasing, canonical root placement,
  stable global interpolation and same-frame camera/query verification.
  [Scope](FLOATING_ORIGIN.md). Server/query-scene rebasing and multi-chunk content
  ownership remain open.

- [x] Bounded runtime coordinate/identity slice: checked global authority, explicit
  epoch/origin wire v6, relative replication rebases and stable local presentation.
  Multi-chunk ownership and server query-scene rebasing remain open.
  [Contract](COORDINATES_IDENTITY.md).

- [x] Renet local replacement gate: measured real UDP delay/loss/duplicate/reorder,
  outages, bounded queues, captured stale-session packet rejection and default
  launcher switch. [Qualification](RENET_QUALIFICATION.md).

- [x] Executable Renet bridge for server/client/launcher, secure local tickets,
  world matching and authority lifecycle. Production identity, Internet security
  and load qualification remain open. [Scope](NETWORKING_RENET.md).

- [x] Pin Bevy 0.20-compatible Avian development revision and migrate normal
  client plugins/colliders plus shared server/offline movement, ground and camera
  queries. Retain grounded/profile limits. [Scope and evidence](PHYSICS_AVIAN.md).

The milestones below preserve the earlier walking-prototype plan and evidence.
Their former next-step ordering is superseded by the engineering queue. Keep the
map and building fixtures as regressions while replacing the general movement
foundation; adding more individually authored interiors is not the default path.

## Earlier first playable milestones

These are game outcomes, not a continuation of the old engine's M0–M6 schedule.

## Bootstrap — runnable local client

- [x] Independent Rust workspace and pinned Bevy application.
- [x] Original graybox scene, lit geometry, local movement and follow camera.
- [x] Controls, reset/quit, scripted movement and rendered screenshot path.
- [x] Execute native qualification and record limitations.

## First multiplayer slice — two players walking together

- [x] One local Rust authoritative server with a fixed simulation tick.
- [x] Review/reuse suitable protocol and Quinn code from the old project, preserving
  provenance and relevant tests. Do not copy the whole engine or assume its
  experimental protocol is production-ready.
- [x] Two independently launched clients connect, receive distinct player identities
  and see each other's server-owned movement in the same scene.
- [x] Clients submit bounded input intents; server controls speed, bounds and state.
- [x] Basic remote interpolation, direct local server poses, disconnect/reconnect
  and malformed/stale input tests. Rendered poses agree with server snapshots.
- [x] Repeatable local two-client/server verification with actual GPU screenshots.
- [ ] Local prediction/reconciliation when needed for responsiveness; qualify
  jitter/loss/delay and recovery. Direct server poses currently provide correction.
- [ ] Manual two-client walkthrough on request; current GPU checks use an owned
  private headless display. Automatic session resume is not implemented.

## Representative game scene

- [x] Enable ordinary Bevy glTF loading and an independent bounds/scale inspector.
- [x] Qualify the user's geometry-only house GLB and move it into the repo as a
  compatibility reference. No authored UVs/materials/textures exist in that file.
- [ ] Qualify textured static assets and animation loading with original or
  appropriately licensed character, terrain, building and props.
  Local compatibility checks now render the user's textured farmhouse and character;
  stand/walk/run playback works in the inspector. The character export still fails
  strict glTF validation. Representative game content remains a separate step.
- [x] First explicit reference placement with conservative server collision:
  textured farmhouse inside the shared solid envelope, human players and a
  nonblocking idle wolf placeholder. [Scope and checks](REFERENCE_SCENE.md).
- [ ] Qualify final representative asset placement and colliders; viewing imported
  meshes does not automatically create server collision or usable interiors.
- [x] First static-world collision pass for the flat procedural scene: solid
  building, fixed door and rocks. Rendering and authority share authored geometry.
- [x] Orbit/zoom camera, camera-relative movement and static camera obstruction.
- [x] First bounded terrain-only walking patch: shared source triangles, server-owned
  height/slope checks and two rendered multiplayer clients. [Scope](TERRAIN_WALKING.md).
- [x] Animated local/remote human players on the bounded terrain patch, with actual
  stand/walk/run sampling alongside server-owned height.
- [x] One authored farmhouse/foundation on terrain with shared server/camera solid
  boundaries and actual connected contact checks. [Scope](TERRAIN_STATIC_OBJECTS.md).
- [x] One enterable farmhouse ground floor: measured open doorway, fixed ramp,
  source floor support and networked entry/exit with interior wall/camera checks.
  [Scope](ENTERABLE_HOUSE.md).
- [ ] Qualify general authored-world collision alongside imported assets; steps,
  jumping and general mesh colliders remain outstanding.
  The Ruins of Lordaeron terrain and 846 top-level placements plus 624 nested props
  now have a qualified [static visual assembly](MAP_OBJECTS.md). Exported liquid
  geometry also has a [static PBR preview](MAP_LIQUIDS.md). The
  [full development world](DEVELOPMENT_WORLD.md) now connects all source terrain
  and the complete assembly to authoritative outdoor movement, conservative object
  envelopes and blocked water cells. Precise mesh collision, interiors and swimming
  remain outstanding.
- [x] First movement-driven character animation states: human stand/walk/run for
  local and remote players, alongside the existing controls/connection HUD.
- [x] Open the original full-map farmhouse ground floor with source floor support,
  measured rotated walls, retained furnishing collision, a fixed ramp and outdoor
  approach spawns. Improve static-mesh camera clearance and shaded visibility.
  [Scope and checks](MAP_FARMHOUSE.md).
- [x] Retain farmhouse access and open the original blacksmith main room, with
  source floors, a fixed ramp, wall contact and exterior forge/bay exclusions.
  [Scope and checks](MAP_BLACKSMITH.md).
- [ ] Animation blending/combat states and the gameplay target/health/ability HUD.
  Environment geometry stays static, including doors. Players do not physically
  collide with either players or NPCs; targeting/hit detection is separate.
- Define an asset layout and stable game identifiers; avoid custom cooked formats
  until a measured requirement needs them.
- Verify the same small scene and connection flow on native Windows.

- [x] Make the complete supplied tile the outdoor development world, with animated
  multiplayer players, bounded terrain/prop lookup, conservative static collision,
  water exclusions and a single local launch command. Buildings remain solid.

## Gameplay loop, then measured scale

Add a server-owned interaction/combat loop, then persistence and accounts when the
playable experience needs them. Increase entities/clients while measuring actual
simulation, replication, asset and rendering costs. Interest management, larger
world streaming and services follow observed constraints. No MMO-scale capacity
is claimed by the bootstrap or first two-client milestone.

The intended game is WoW-style: begin with a targetable NPC, one server-validated
ability, health/cooldown, then one quest and reward before broadening systems.
See [GAME_DIRECTION.md](GAME_DIRECTION.md) for the small starting-zone outcome.


Forest geometry prerequisite: [calibrated source poses and five-instance native
assembly](UNITY_GEOMETRY.md) now qualify. Full binary terrain import, vegetation,
scene coverage and authoritative collision still precede replacement-map adoption.


Forest terrain prerequisite: [full-sample binary terrain decoding and native
ground preview](UNITY_TERRAIN.md) now qualify. Matching bounded collision,
authored terrain normals/material fidelity and full vegetation/scene/multiplayer
remain before replacement-map adoption.


Forest ground prerequisite: [full-sample authoritative collision and native
multiplayer ground](FOREST_TERRAIN_COLLISION.md) now qualify in four opt-in
packages. Full vegetation/materials and object collision remain before forest
adoption; the default town remains available.


Forest visual continuation: [LOD0 full-shader tree/fern prototypes and actual
planted subset](FULL_FOREST_FOLIAGE.md) now qualify. Complete material variants,
other shader families, authored object collision and scene adoption remain open.


Forest P1-10/P2-08 bulk foliage continuation now qualifies 15,156 source draws in
46 exact mesh/material bindings, actual scene overrides and a shared-image visual
assembly. Independent complete source/category/material/tangent checks and two
native Bevy/Vulkan views pass; 3,503 draws retain explicit unresolved reasons.
Continue remaining Standard/other shader families and mixed slots, then review
object collision and resource budgets before full multiplayer/default adoption.
Town and opt-in forest-ground remain available. [Evidence](FOREST_FOLIAGE_BATCH.md).


Forest P1-10/P2-08 Standard-material continuation qualifies another 66 bindings /
2,744 source draws, bringing the static subset to 17,900 draws with 31 shared
image variants. Canonical mesh-local export excludes imported FBX node placement;
source normal directions, exact palettes, full scene poses and two native Bevy/
Vulkan views qualify. 759 draws remain explicit. Continue four model variants,
other shader families/mixed slots, then object collision and resource/multiplayer
acceptance before adoption. [Evidence](FOREST_STATIC_BATCH.md).


Forest P1-10/P2-08 canonical model-variant continuation now qualifies all four
branch models / 179 poses, with inherited IDs, raw nested/outer material bindings,
independent ufbx/source geometry and byte-preserved prior content. The combined
18,079-draw scene passes complete source and native Bevy/Vulkan checks. 580 draws
remain explicit; continue triplanar cliffs/rocks and other shader/mixed-slot work,
then object collision and resource/multiplayer acceptance before adoption.
[Contract](FOREST_VARIANT_GEOMETRY.md). The user queues the new Elven Realms
folder under Downloads / Synty / Biomes only after forest completion; its contents
have not yet been verified.


Forest P1-10/P2-08/P2-10 triplanar prerequisite now has a source-hash-bound world
projection profile and opt-in Bevy StandardMaterial extension. Actual canonical/
transformed mesh cases pass native lit/diagnostic rendering and independent GPU
pixel checks. The main subset remains 18,079 draws with 580 unresolved. Before
batch adoption, calibrate the exposed FBX mesh/root-pivot offset (including existing
LOD-bound evidence), then integrate multiple profiles, f64 rebase texture phases,
resources/mips and complete scene/object collision acceptance. No automatic mesh
centering or default-map change. [Contract](FOREST_TRIPLANAR.md). Elven folder
inspection remains queued after forest completion.

## Forest source sky — 2026-10-21

Forest sky continuation, 2026-10-21: the source skydome now joins the qualified
subset: **18,651 source draws / eight unresolved**. Actual Editor geometry, source
world-height gradient and placement pass independent checks. Standard unlit and
explicit Bevy shadow markers pass four native Vulkan cases; three legacy modes,
two native failure paths and encrypted town/farmhouse regressions pass. Fresh fmt,
denied-warning Clippy, 38 client Rust and 193 Python tests pass. Three Khronos
files have zero errors/seven retained warnings. Seven water draws and one emissive
willow remain, followed by lighting, rebase/resources/mips, collision and adoption.
All work stays local. [Evidence](FOREST_SKY.md).

## Forest static water — 2026-10-21

Forest water continuation, 2026-10-21: four bindings / seven source water draws
join the qualified visual subset: **18,658 source draws / one unresolved**. Actual
Linux Unity proves built-in Plane identity/geometry; exact Editor FBX triangles,
raw source poses and explicit static PBR factors pass independent checks. Full
composition, eleven native Vulkan cases and four rehashed faults pass. Ten Khronos
files have zero errors/seven retained warnings. Fresh fmt, denied-warning Clippy
and 197 Python tests pass; runtime binaries remain unchanged. Emissive willow,
source water effects, lighting, rebase/resources/mips, collision and adoption remain
open. All work stays local. [Evidence](FOREST_WATER.md).

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
stays local. [Evidence](FOREST_EMISSIVE.md).

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
[Evidence](FOREST_RESOURCES.md).

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
All work stays local. [Evidence](FOREST_LIGHTING.md).

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
remain open. All work stays local. [Evidence](FOREST_COLLIDER_REFERENCE.md).

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
work stays local. [Evidence](FOREST_COLLISION_GEOMETRY.md).

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
defaults remain preserved. All work stays local. [Evidence](FOREST_HEIGHTFIELDS.md).
