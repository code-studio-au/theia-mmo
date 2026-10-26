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

# Current status — 2026-10-23

Saved continental terrain is the verified development default. The existing
16 km × 16 km world is baked into 15,625 versioned, hashed 128 m height files.
Visuals and collision load those samples on bounded workers; fixed simulation
performs no file I/O. Missing/corrupt data fails explicitly, with no recipe
fallback. `python3 tools/play.py` uses `build/graybox-terrain-v1` after its offline
bake; `--graybox-world` retains generated terrain. Prediction remains enabled.

P1-10/P2-06/07 and the initial P3-13 slice pass: full saved native crossing,
125 visited columns, maximum 81 terrain roots, 2,345 loads / 2,300 retirements,
zero movement recoveries or missed server deadlines. One-client Linux RX 7900
XTX / Ryzen 7800X3D, 1280×720, validation off: **818.59 FPS**, mean **1.222 ms**.
This is warm-cache empty-world evidence. Two clients under impaired UDP, held-jump
teleports, explicit corrupt-file exit, default launcher cleanup, retained town and
generated-source native regressions all pass. Wire/Quinn ALPN is **12**, with a
77-byte welcome carrying explicit terrain frame policy.

The initial saved-source checkpoint passed **228 Rust tests**, with ten private-fixture
tests explicitly ignored; the strengthened
saved non-spawn revisit tests also pass. Denied-warning Clippy, formatting,
221 asset-tool and four GLB alignment tests pass. [Saved terrain](SAVED_TERRAIN.md)
records exact binaries/content identities, commands, reports, failed attempts
and remaining gates. New work remains local.
P2-04 diagnostics now distinguish matched simulation ticks, pending readiness,
input/loading transitions and steady rendered movement. The qualified client
clock uses a continuous bounded rate and acquires its initial buffer after real
RTT/terrain readiness. Steady held frames on the impaired native route fall from
about 9% to **0% / 0.152%**; ordinary VSync records **0.463%** at 60 FPS. A scoped
1% gate rejects the measured old baseline and accepts these final runs. Rare
corrections under loss remain; broader latency/Windows/load limits are explicit.
[Measurement/clock evidence](PREDICTION_CONTRACT.md). That checkpoint passes
**234 Rust tests**, ten private-fixture skips and Clippy/fmt.

The subsequent actual eight-peer, 180-second saved-terrain headless soak passes:
zero server deadline misses/recoveries, maximum nine authority collision patches
and two per client query runtime, with actual retirement and relevance pruning.
Sampled combined RSS peaks at 166.33 MiB; long-term memory plateau remains open.
[Soak evidence](SAVED_TERRAIN_SOAK.md) separates the completed v2 run from the
latest example-only return-identity checks, which compile/Clippy but remain
runtime unqualified. The full 234-test checkpoint predates the new soak/telemetry.

**Stopped safely at the user’s request.** The build completed; no longer soak was
started. Changes remain local and uncommitted. Larger roadmap tasks remain partial.

## Earlier checkpoints

# Current status — 2026-10-22

The completed Linux development slice is the 16 km × 16 km graybox world with
POLYGON Base Locomotion and actual Renet client prediction/reconciliation enabled
by default. Run `python3 tools/play.py`; `--no-prediction` retains the qualified
server-pose interpolation baseline. Forest reproduction remains frozen.

P1-06/08 and bounded P2-04 acceptance now includes completed-step authority,
64-step replay, explicit movement streams, collision/readiness gating, real
impaired UDP at 20/30/60 Hz, native two-client animation/camera/teleport checks
and held-jump resets. The predicted full-continent crossing visits all 125 chunk
columns with at most 81 terrain roots, actual mesh retirement and zero missed
server deadlines or movement recoveries. Linux RX 7900 XTX / Ryzen 7800X3D,
1280×720, one client, validation off: **821.70 FPS**, mean **1.217 ms**;
this empty-world measurement does not qualify MMO population capacity.

The final client additionally bounds interpolation when a distant teleport's
snapshot precedes its correction. Its fresh native check passes. The default
launcher's two-client startup, real command traffic and owned process/ticket
cleanup pass after both interruption and injected client failure. POSIX shutdown
now allows the server to emit final transport statistics; Windows retains its
existing termination path and remains unqualified natively.

See [the prediction contract](PREDICTION_CONTRACT.md) for exact executable hashes,
checks and limitations, [the terrain baseline](GRAYBOX_WORLD.md), and the
[live task status](engineering/TASK_STATUS.md). Broader P2-04 latency/load/Windows,
authored saved-world streaming, navigation, combat and persistence remain open.
Raw correction diagnostics include readiness/clock catch-up and do not establish
a universal visual smoothness budget. The current slice is complete; further
feature work is stopped at the user's request. Commit/push is explicitly authorized.

Final repository verification, 2026-10-22: **224 Rust tests pass**, zero failures;
10 explicit private-fixture tests remain ignored. All **221 asset-tool Python tests**
and **4 GLB alignment tests** pass. Workspace/all-target denied-warning Clippy,
`cargo fmt --check` and diff whitespace checks pass. Logs:
`build/prediction-commit-workspace-tests.log`, `build/prediction-commit-clippy.log`,
`build/prediction-commit-fmt.log`, `build/prediction-commit-python-tests.log`,
`build/prediction-commit-align-tests.log`. Actual encrypted UDP reports are in
`build/prediction-commit-network`; these remain ignored local evidence, along with
private models, Unity exports, tickets and binaries. Linux execution is qualified;
no native Windows execution is claimed. No engine/plugin pins changed in this
prediction continuation.

## Earlier checkpoints (historical, superseded where noted)

Prediction foundation continuation, 2026-10-22 — P1-06/08, P2-04: completed-step
command timing and a bounded 64-step reconciliation core now pass headless Avian
tests at 20/30/60 Hz. Scheduled delivery faults exercise immediate local response,
terrain seam/jump correction and stopped convergence; teleport, readiness,
collision failure, overflow and stale/foreign-state guards pass. Movement streams
are separate from world incarnation. Full workspace **214 tests pass**, including
the retained real encrypted UDP regressions; denied-warning workspace/all-target
Clippy and fmt/diff pass. The scheduled prediction tests are not real-UDP or native
rendering evidence. Wire v11 and the live client remain on held input and smoothed
authority poses; actual command/correction integration is next. No renderer or
default walkthrough behavior changed. [Scope and evidence](PREDICTION_CONTRACT.md).
All changes remain local.

Continental graybox adoption, 2026-10-22: `python3 tools/play.py` now selects
16 km × 16 km procedural terrain and the POLYGON Base Locomotion development
player. Forest fidelity work is frozen. Bounded workers prepare terrain/Avian
patches; full-detail GPU-ready terrain gates authoritative travel. Standard Bevy
culling, distance LOD, actual mesh retirement, recipient-local snapshots and moving
render/query frames pass native tests. A real release client crosses all 125
horizontal chunk columns: maximum 81 terrain roots, 2,345 mesh publications /
2,300 retirements, zero movement recoveries or missed server deadlines.
At 1280×720 on RX 7900 XTX, one-client uncapped crossing averages 824.14 FPS
(1.213 ms; worst one-second p95 1.376 ms), without validation. Two-client validated
teleport/return/animation and retained town cases pass. Local/remote presentation
smoothing fixes 30 Hz stepping; readiness withdrawal now preserves airborne
gravity/landing. Delayed/lost/reordered two-peer terrain readiness and the actual
default launcher's startup/cleanup pass. Kinematic state replay is qualified as a
P2-04 prerequisite; client prediction remains unimplemented. Full workspace
201 tests (including 49 client), 221 Python tests,
denied-warning Clippy and fmt/diff pass. This is graybox terrain traversal, not
saved authored-map streaming, all-route/Windows/MMO capacity qualification.
[Behavior, evidence and limits](GRAYBOX_WORLD.md). All changes remain local.

# Earlier development record

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

Forest water continuation, 2026-10-21: four bindings / seven source water draws
join the qualified visual subset: **18,658 source draws / one unresolved**. Actual
Linux Unity proves built-in Plane identity/geometry; exact Editor FBX triangles,
raw source poses and explicit static PBR factors pass independent checks. Full
composition, eleven native Vulkan cases and four rehashed faults pass. Ten Khronos
files have zero errors/seven retained warnings. Fresh fmt, denied-warning Clippy
and 197 Python tests pass; runtime binaries remain unchanged. Emissive willow,
source water effects, lighting, rebase/resources/mips, collision and adoption remain
open. All work stays local. [Evidence](FOREST_WATER.md).

Forest sky continuation, 2026-10-21: the source skydome now joins the qualified
subset: **18,651 source draws / eight unresolved**. Actual Editor geometry, source
world-height gradient and placement pass independent checks. Standard unlit and
explicit Bevy shadow markers pass four native Vulkan cases; three legacy modes,
two native failure paths and encrypted town/farmhouse regressions pass. Fresh fmt,
denied-warning Clippy, 38 client Rust and 193 Python tests pass. Three Khronos
files have zero errors/seven retained warnings. Seven water draws and one emissive
willow remain, followed by lighting, rebase/resources/mips, collision and adoption.
All work stays local. [Evidence](FOREST_SKY.md).

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

Forest material-catalog continuation, 2026-10-21: the inspector supports bounded
schema-2 profiles alongside Standard materials while retaining schema 1. An actual
85+42-placement fixture applies two profiles to 127 primitives, keeps the building
Standard, and shares six images across eight slots. Native mapping/failure paths,
1,906 independent pixel samples, legacy/full-forest and encrypted town/farmhouse
regressions pass. Fresh fmt, denied-warning Clippy, 35 client Rust and 171 Python
tests pass. The main subset remains 18,079 draws / 580 unresolved. Next: batch
eligible triplanar bindings into the full scene, then rebase/resource/mip and
remaining material/collision/runtime gates. All work stays local.
[Contract, evidence and limits](FOREST_MATERIAL_CATALOG.md).

Forest mesh-frame continuation, 2026-10-21: the user resumed work. Ten bindings /
thirteen placements now use actual Unity mesh frames; the other 106 prototype/
placement pairs stay byte-identical. All 116 prototypes match Editor vertices
within 7.69 micrometres. Full scene coverage, three native Bevy/Vulkan views,
Khronos and retained encrypted town/farmhouse regressions pass. A corrected
85-placement moss group also passes source/world, six final native and 512
independent pixel checks, maximum one sRGB8 level. Fresh fmt, denied-warning
Clippy, 169 Python tests and three private faults pass. The corrected main subset
remains 18,079 draws / 580 unresolved; moss is still an opt-in single-profile
group. Multi-profile/rebase/resource integration, remaining materials, object
collision and adoption remain. All work stays local.
[Contract, evidence and next gates](FOREST_MESH_FRAMES.md).

Forest Unity-reference checkpoint, 2026-10-20: native Linux Unity 6000.6.6f1
compiled/exported 112 actual FBXs / 214 meshes, covering all 118 FBX-backed scene
bindings. Independent actual-frame point sets all match within 11.02 micrometres;
all metadata remains byte-identical. The old unadjusted mesh-local policy matches
195/214 meshes; ten current batch bindings / thirteen draws need frame correction.
Fresh fmt, denied-warning Clippy and 162 Python tests pass. All jobs completed.
The user requested a safe stop; no converter correction, runtime change or forest
promotion began. The 18,079-draw subset / 580 unresolved draws remain preserved,
with importer-frame corrections now explicit. All work stays local.
[Findings and resumption acceptance](FOREST_UNITY_REFERENCE.md).

Forest triplanar material prerequisite, 2026-10-20: a pinned world-projection
profile now renders one actual moss mesh and a transformed instance through
Bevy's standard PBR extension. Six native Vulkan cases and 588 independent
colour/decoded-normal pixel checks pass (maximum one sRGB8 level); missing options/
marks reject. Fresh fmt, denied-warning Clippy, 157 Python and 31 client Rust tests
pass. The 18,079-draw forest subset still has 580 unresolved draws. A newly exposed
FBX root/pivot-offset calibration gap, multiple-profile/streaming/rebase integration,
mips and object collision remain before promotion. No guessed centering is applied.
All work stays local. [Contract and evidence](FOREST_TRIPLANAR.md).

Forest branch-variant geometry continuation, 2026-10-20: all four inherited-ID
branch models and their **179 actual source placements** now join the qualified
static scene (18,079 source draws). Independent raw bindings, complete category/
scene coverage, ufbx and byte-preserved reuse checks pass. Two native Bevy/RADV
Vulkan views pass active validation; 117 glTF files have zero errors and the 14
existing tangent warnings. Fresh fmt, denied-warning Clippy and 152 Python tests
pass. 580 draws, authored object collision, resource review and adoption remain
open. Elven source-folder inspection is queued after forest completion, per user
instruction. All work stays local. [Evidence](FOREST_VARIANT_GEOMETRY.md).

Forest Standard-material continuation, 2026-10-20: **17,900 source draws** now
render in the combined static subset, adding 66 Standard bindings / 2,744 draws.
Canonical export avoids double-applying FBX node scale; independent source normal
direction checks qualify two nonunit-normal meshes without changing geometry.
All 66 Standard references and complete final-source coverage pass. Two native
Bevy/RADV Vulkan views pass active validation; 113 glTF files have zero errors
and 14 documented tangent warnings. Fresh fmt, denied-warning Clippy and 150
Python tests pass; legacy ruins bytes are unchanged. 759 draws, authored object
collision, resource review and full map adoption remain open. All work is local.
[Contract and evidence](FOREST_STATIC_BATCH.md).

Forest foliage batch continuation, 2026-10-20: **15,156 source draws across 46
mesh/material bindings** now join ground/ruins with 17 shared image variants.
Independent full-scene source/pose/material/tangent coverage passes; 3,503 draws
remain explicitly unresolved. Two final native Bevy/RADV Vulkan views pass active
validation; Khronos reports zero errors and 14 documented runtime-tangent warnings.
Fresh fmt, denied-warning Clippy and 149 Python tests pass; basic-card bytes remain
unchanged. Full object collision, remaining shader families/resources and default
map adoption remain open. All work is local. [Evidence](FOREST_FOLIAGE_BATCH.md).

Highest-detail forest foliage continuation, 2026-10-20: a pinned full-shader
static PBR profile now converts the actual LOD0 small tree and fern, preserving
source cutouts and applying explicit normal-map reconstruction. Independent
geometry/pixel/normal/FBX checks pass. Three exact-binding tree and 612 fern
placements join ground/ruins in a source-preserving visual subset. Three native
Bevy/RADV Vulkan cases pass with active validation. The legacy basic card remains
byte-identical. Fresh fmt, denied-warning Clippy and 137 Python tests pass.
Eight of twelve used full-shader property profiles are eligible; four and other
shader families remain unresolved. Wind/highlights, complete scene/object collision
and adoption remain open. All work is local. [Contract](FULL_FOREST_FOLIAGE.md).

Forest ground collision continuation, 2026-10-20: four bounded packages retain all
263,169 distinct source vertices and 524,288 ground triangles with one shared PNG.
Independent visual/collision/UV/normal coverage passes. Actual Avian source probes
and 18 seam/jump cases pass at 20/30/60 Hz. Two encrypted two-client native cases
cross X/Z seams with matching live collision and shared handles; opt-in launcher
and unchanged default-town admission/cleanup pass. Fresh fmt, denied-warning
Clippy, 160 ordinary plus two private Rust tests and 131 Python tests pass;
retained town/farmhouse routes pass all three tick rates. Full forest vegetation,
materials, object collision and adoption remain open. All work is local.
[Contract, commands and evidence](FOREST_TERRAIN_COLLISION.md).

Forest terrain continuation, 2026-10-20: binary TerrainData now decodes with all
263,169 source heights, five layer/control bindings and 1,365 independently checked
patch bounds. A full-sample 150 × 150 metre ground GLB and explicit baked-colour
preview pass independent geometry/UV/pixel checks and Khronos validation. Ground
and ground-plus-five-ruins pass native Bevy/RADV Vulkan checks with active
validation. Fresh fmt, denied-warning Clippy and 127 Python tests pass. Authored
terrain normal maps, full vegetation/scene, terrain collision and multiplayer
adoption remain open; the current town remains the runtime map. All work is local.
[Contract, limitations and evidence](UNITY_TERRAIN.md).

Forest geometry continuation, 2026-10-20: source-specific X-reflection calibration
matches 52 of 54 serialized LOD bounds; two centre mismatches remain explicit.
Independent ufbx positions and an affine-safe shared-mesh assembler now qualify
five actual ruins placements and an asymmetric shear/mirror fixture. Both pass
native Bevy/RADV Vulkan inspection with active validation and vertex bounds within
0.003 mm of independent source calculations. Fresh fmt, denied-warning Clippy
and 121 Python tests pass. TerrainData decoding, full vegetation, full scene and
collision remain open; the current town stays the runtime map. All work is local.
[Contract and evidence](UNITY_GEOMETRY.md).

Qualified model-variant placement continuation, 2026-10-20: inherited aliases now
bind all four branch variants and all 2,864 scene modifications. Requiring complete
authored root TRS resolves their 179 placements without assuming imported root
poses. All 179 pass independent category-completeness/binding/matrix checks.
The source plan now resolves 18,725 prefabs and emits 18,659 static records; the
old 18,480 records and 423 unused overrides are exactly unchanged. Thirty particle
override errors remain, alongside separately recorded unimplemented components.
Fresh fmt, denied-warning Clippy and 115 distinct Python tests pass. Geometry
axes, full materials, assembled scene and collision remain open; no Rust/binary/
renderer/default-map change or new GPU claim.
[Contract and evidence](UNITY_MODEL_VARIANTS.md).

Qualified static foliage continuation, 2026-10-20: actual background-tree cards
and a vine LOD1 card now convert to standard Bevy masked PBR with source-derived
colours, exact alpha, checked branch geometry/UVs and measured mesh-normal export
rounding. Two native inspector cases and Khronos validation pass on RX 7900 XTX
with active Vulkan validation. Custom normal-map output remains an explicit
approximation. Highest-detail vegetation/full scene adoption remain open.
Fresh fmt, denied-warning Clippy and 109 Python tests pass; the original default
scene plan is exactly unchanged. No Rust/binary/default-map change.
[Contract, evidence and limits](FOREST_FOLIAGE.md).

Qualified forest static-placement continuation, 2026-10-20: source hierarchy,
instance transforms/active/material overrides and highest-detail LOD planning now
resolve 18,546 of 18,755 prefab placements. All 18,480 emitted static records pass
independent mathutils/source binding checks. The report retains 423 documented
unused flat-target overrides and 209 unresolved model-variant/particle placements.
Fresh fmt, denied-warning Clippy and 103 distinct Python tests pass. This is an
offline source-coordinate plan; geometry axes, custom materials, assembled scene
and collision remain open. No Rust/runtime/default-map change or new GPU claim.
[Contract and evidence](UNITY_STATIC_PLACEMENTS.md).

Qualified Enchanted Forest material continuation, 2026-10-20: the first actual
ruins prefab converts through a reusable opaque Standard-to-glTF PBR adaptation,
preserving source albedo/emission pixels, surface scalars, culling and sampling.
Independent geometry/UV/material checks and Khronos validation pass; one native
Bevy capture passes on RX 7900 XTX/RADV Vulkan with active validation.
Fresh fmt, denied-warning Clippy and 95 Python tests pass; legacy pillar source
conversion still passes. No Rust runtime/binary/default-map change.
Next: custom forest materials and source hierarchy/LOD/override instantiation
before assembled scene and collision qualification.
[Contract, evidence and scope](SYNTY_ENCHANTED_FOREST_IMPORT.md).

Enchanted Forest replaces Elven Realm as the user-selected environment import
target, 2026-10-20. Its 225 FBXs and selected images contain actual data; no LFS
pointers occur in the 647 staged data payloads. Source/demo audit is complete;
prefab/material conversion and native scene qualification precede map adoption.
The current town remains available during replacement. Elven missing-source
recovery is no longer in the active queue.
[Source evidence and next acceptance](SYNTY_ENCHANTED_FOREST_IMPORT.md).

Qualified actual-town live-publication continuation, 2026-10-20: ordinary encrypted
two-client sessions now publish ten-to-eleven town packages, retaining existing
roots/image handles and matching live Avian. Normal loading and a five-second
asymmetric delay with independent client query frame pass. Shared startup exposes
resolved recovery feet; exact round-trip decimal JSON parsing fixes a reproduced
one-bit admission-identity mismatch without relaxing the independent oracle.

Fresh fmt, denied-warning Clippy/build, **160 ordinary Rust tests**, **three
private-source tests** and **91 Python tests** pass. **Eleven actual native
cases, 22 reports**, pass on RX 7900 XTX/RADV Vulkan with active validation,
including staged-candidate asset cleanup, all-26 startup, town/farmhouse and Quinn.
These bounded debug samples do not establish production streaming or latency.

Next: remaining retirement/reversal/supersession/asymmetric-failure lifecycle
checks before adoption. The normal launcher remains monolithic revision 2 at
30 Hz. All changes stay local. The user confirms no alternate Elven Realm source
is available; missing Git LFS payloads leave full equivalent import pending.
[Live-publication contract](TOWN_LIVE_PUBLICATION.md).

Qualified actual-town client startup/resource continuation, 2026-10-20: ten
packages and all 26 now load in ordinary encrypted two-client Bevy sessions.
The first actual run exposed an offline bounds mismatch; authoring now binds
transformed mesh-local AABB corners, matching the unchanged strict client guard.
Only a new stage's index changes; package/image bytes and the launcher remain.

Two actual native cases, four reports, pass on RX 7900 XTX/RADV Vulkan with active
validation, exact readiness/identity, shared-image handles, live Avian, camera,
pose/mesh-clearance checks and zero unexpected recovery. Sampled per-client peak
RSS is 1.82–1.83 GiB initially and 2.30 GiB for all packages; driver VRAM residency
is 1.54–1.60 / 1.84–1.85 GiB. These are warm-cache debug qualification samples,
not release/latency percentiles or hardware minimums. Fresh fmt, denied-warning
Clippy and 88 Python tests pass; Rust binaries retain the preceding checkpoint.

Next: actual live candidate/staging/resource lifetimes and remaining handover
lifecycle cases, then map regressions before adoption. The normal launcher stays
monolithic revision 2 at 30 Hz. All work stays local.
[Contract, measurements and limitations](TOWN_CLIENT_RESOURCES.md).

Qualified finite-world continuation, 2026-10-20: chunk-session schema 2 binds an
explicit checked global rectangle into admission identity v4. Conservative local
planes constrain shared authority movement independently of loaded/ready chunk
coverage; internal seams and vertical motion stay open, and lookahead clamps to
the finite world. Schema-1 identities and wire/ALPN 11 remain unchanged.

Fresh fmt, denied-warning Clippy/build, **158 distinct Rust tests** and **85 Python
tests** pass. The workspace run passes 157 tests, followed by a four-case targeted
run adding the new finite-world recovery case. Three private tests ignored by
default separately pass, including source-town rectangle preservation and
ten-to-26-package publication. **Six actual two-client native cases**, twelve
reports, pass on RX 7900 XTX/RADV Vulkan with active validation: finite X/Z wall
pressure, delayed negative independent-frame publication, town, farmhouse and
Quinn. No unexpected recovery occurs; all changes remain local.

Next: actual-town client loading/staging memory and latency, then remaining
handover reversal/supersession/asymmetric-failure checks before adoption. The
normal launcher remains monolithic revision 2 at 30 Hz. No whole task/phase,
full-town perimeter, native Windows or MMO capacity claim follows.
[Contract and evidence](WORLD_EDGES.md).

Elven model/source continuation, 2026-10-20: checked modern model-ID candidates
and an older explicit metadata table match 1,109 distinct asset/fileID references.
The source package contains 30 FBX and two PNG Git LFS pointers; the demo closure
reaches 26 missing FBXs and one PNG. Expected blob hashes/sizes are recorded;
18 additional override-target IDs remain unmapped. Full equivalent scene export
therefore remains open. Fresh fmt, denied-warning Clippy and 85 Python tests pass.
No Rust runtime/render changes or default-map adoption occurred. Core row-5 work
can continue independently while complete sources are located.
[Evidence and source-completeness gate](SYNTY_ELVEN_IMPORT.md).

Elven Realm source/import checkpoint, 2026-10-20: selected source payloads now
retain hash-bound Unity metadata. The recursive dependency audit identifies
model-prefab ID mapping and unresolved serialized references before full-scene
assembly. One actual pillar converts to standard GLB with its source palette;
all 404 triangle positions/winding/UVs and palette pixels pass independent export
comparison. Khronos validation is clean. A native Bevy capture passes on RX 7900
XTX/RADV Vulkan with active validation. The material is explicitly base-colour
only; the full scene, collision and game adoption remain open.
Fresh fmt, denied-warning Clippy and 80 Python tests pass; Rust runtime/binaries
are unchanged. All assets/output remain ignored and changes stay local.
[Scope, evidence and remaining acceptance](SYNTY_ELVEN_IMPORT.md).

Qualified isolated Synty preview, 2026-10-20: selected Base Locomotion FBXs
now convert to standard GLB with their own palette, 50-joint skin and four
baked Idle/Walk/Run/Jump clips. Source bind poses differ, so source world poses
are baked onto the sample skin. An identity mesh root fixes ordinary bounds/
skinning agreement at 1.793552 metres. Both GLBs validate without errors/warnings.
Independent source-pose comparison passes all 175 sampled frames, and four actual
native Bevy animation cases pass on RX 7900 XTX/RADV Vulkan with active validation.
The walk preview was opened on the user's desktop.

The full source skin is retained separately. The standard-renderer four-weight
preview changes 42 source vertices; sampled Jump deformation reaches 6.974 mm,
so the initial 5 mm skin gate stays failed. An explicit preview flag permits
rendering without claiming exact source skin or production character adoption.
Fresh fmt, denied-warning Clippy and **75 Python tests** pass. Rust binaries are
unchanged from the preceding 154-test/eight-native-case checkpoint; that suite
was not rerun for these offline tools/assets. All private assets/build outputs
remain ignored and all work stays local.

Next asset step: Elven Realm representative mesh/material and assembled prefab
scene conversion. The core finite-world edge, actual-town client/resource and
remaining handover gates remain open. The normal game character/map and 30 Hz
default are unchanged. [Preview contract and commands](SYNTY_CHARACTER_PREVIEW.md).
[Evidence](VALIDATION.md).

Qualified runtime capacity continuation, 2026-10-20: the hard ceiling is now
32 with explicit per-session budgets, checked global control addresses, world
identity v3 binding the budget, and wire/ALPN 11. Existing fixture budgets and
bounded datagram/input/snapshot/queue limits remain unchanged. Actual native
startup supports 32 roots; delayed publication adds six required roots while
retaining the old root, shared images and matching live Avian.

Fresh fmt, denied-warning Clippy/build, **154 Rust tests** and **70 Python tests**
pass. Three private tests are ignored by default and separately pass: actual town
ten-to-26-package live authority publication, both house routes at 20/30/60 Hz
without recovery, and all staged shared-image dependencies. A final 27-test
library run verifies corrected current-version input-negative fixtures.
**Eight actual two-client native cases**, 16 reports, pass on RX 7900 XTX/RADV
Vulkan with active validation, including negative independent frames, failed
image retention, both map regressions and Quinn. Imported sources retain their
hashes. All work remains local.

The default remains monolithic town revision 2 at 30 Hz. The 26-package town
runtime session is opt-in/headless-qualified only. Next: explicit finite-world
bounds and edge/corner qualification, then actual-town client loading/resource
measurements and remaining handover lifecycle cases before adoption. No complete
task/phase, Windows or MMO capacity claim follows.
[Contract and scope](CHUNK_CAPACITY.md). [Executed evidence](VALIDATION.md).

Qualified budget-analysis continuation, 2026-10-20: the new offline tool
verifies actual package/image bytes and geometry envelopes, reconstructs source
spawn footprints and measures conservative actor/recovery/lookahead unions.
The town needs 18 providers for one capsule near a corner, 22 for two actors,
and at most all 26 packages for this finite index. Five conservative XZ actor
placements with recovery already reach that upper bound; these ignore terrain
height, obstacles, reachability and future finite-world bounds.

All 26 packages together contain 4,001 static colliders, 698,655,860 shared PNG
bytes and an estimated 1,514,491,904 base RGBA8 bytes. These are verified content
counts/estimates, not live preparation latency or measured CPU/GPU residency.
Fresh fmt, denied-warning Clippy and **68 Python tests** pass. No Rust application,
renderer or asset files changed; the previous native/Rust checkpoints remain
historical evidence. A 32-package hard ceiling / 26-package town session is now
a concrete proposal with explicit protocol, admission, world-edge, staging and
native acceptance checks. Runtime capacity remains four and the launcher stays
monolithic revision 2 at 30 Hz. Continue with that runtime qualification; no task
or phase is complete. All changes remain local. [Measurements and next gate](CHUNK_BUDGETS.md).

Qualified offline partition continuation, 2026-10-20: the actual town now has
26 new source-preserving packages. Explicit terrain and matching collision clip
at 128-metre planes; each static model mesh instance has one owner and retains
its ancestor transforms and original buffers. Per-package asset graphs are
pruned, while canonical shared PNG dependencies remain shared.

Independent source comparison verifies all 2,532 model mesh instances, 256
terrain groups, terrain height/UVs/surface winding, collision surfaces/winding
and 3,976 movement/camera boxes. All 26 actual packages separately pass the
headless Rust manifest/collision/image verifier. Fresh fmt, denied-warning
Clippy and **65 Python tests** pass. Two new actual two-client native cases pass
on RX 7900 XTX/RADV Vulkan with active validation: generated original geometry,
then signed billion-metre global positions with independent query frames. These
are four reports with delayed publication, shared image handles, matching live
Avian, retained roots and no unexpected recoveries. Rust application sources and
binaries are unchanged from the preceding 151-test checkpoint.

The town output totals 8,470,060 GLB bytes and 12,805,430 collision JSON bytes.
Each package uses at most 139 textures, compared with 491 in the source. Actual
content requires up to nine providers per cell and ten at the current spawns;
the four-package proof budget therefore prevents active metadata publication.
Only a candidate town index is emitted. The launcher remains the qualified
monolithic revision 2 at 30 Hz. Next: measured residency/staging budgets,
world-edge policy, remaining handover lifecycle checks and actual-town runtime
qualification before adoption. No entire task/phase or Windows/MMO capacity
claim follows. All changes remain local. [Contract](WORLD_PARTITIONING.md).
[Executed evidence](VALIDATION.md).

Qualified shared image continuation, 2026-10-20: manifest schema 2 binds canonical
PNG dependencies, colour space and sampler settings. Normal chunk clients verify
URI/path/hash/header budgets before standard Bevy loading; prepared roots ignore
unbound editor metadata. Native capture proves compatible handles are shared and
incompatible colour/sampler variants remain distinct. Failed dependencies retain
the active scene. An arrival-order regression also fixed unload-only offer churn:
spare packages remain until a new load needs the same bounded resident budget.

Fresh fmt, denied-warning Clippy/build, **151 Rust tests** and **56 Python tests**
pass. Two private-data tests are ignored by default and separately pass: both
house routes at 20/30/60 Hz with zero recoveries, and actual staged town package/
image validation. **Eighteen actual two-client native cases**, 36 reports, plus
both retained startup rejection checks pass on RX 7900 XTX/RADV Vulkan with active
validation and the same final binaries. All four imported source documents retain
their original hashes. All changes remain local.

The unchanged town was staged as a 5,476,980-byte GLB plus 488 shared resources
containing the original 698,655,860 PNG bytes. Independent comparison confirms
original geometry, transforms, material and collision semantics, and PNG bytes.
This changes placement, not total image compression or terrain partitioning.
Next: prune real node/mesh/material/texture graphs, partition actual terrain and
matching collision, and measure residency/load budgets. The launcher remains the
revision-2 monolithic town at 30 Hz. Windows/crowds/production streaming and whole
phase/task completion remain unqualified. [Contract](SHARED_IMAGES.md).
[Executed evidence](VALIDATION.md).

Qualified geometry contributor continuation, 2026-10-20: schema-2 chunk bounds
now bind dataset identity and determine complete-cell coverage, demand, readiness,
protected actors/recovery and retained presentation. A loaded owner floor alone
does not permit movement without a crossing geometry provider. Off-thread
preparation checks actual collider extents; normal clients check actual nested
Bevy mesh bounds before initial readiness or replacement publication. The new
offline tool derives combined static GLB/collision bounds without changing
packages. Schema-1 experiments retain their original identity and limited scope.

Fresh fmt, denied-warning Clippy/build, **144 Rust tests** and **51 Python tests**
pass. One private-map test is ignored by default and separately passes both house
routes at 20/30/60 Hz with zero recoveries. **Fifteen actual two-client native
cases**, 30 reports, plus invalid initial GLB and initial extent rejection pass
on RX 7900 XTX/RADV Vulkan with active validation and the same final binaries.
New cases cover nonlocal geometry, delayed independent frames and valid GLB extent
rejection/retention. All four imported engineering sources retain their hashes.

Next: shared immutable image bindings, then actual imported terrain/collision
partitioning and measured residency/load budgets. The town's 704.1 MB GLB contains
698.7 MB of embedded images; copying that image set into each chunk is not an
adoption path. The launcher remains monolithic, 30 Hz remains default, native
Windows/crowds/production streaming remain unqualified, and no whole task or phase
is complete. All changes remain local. [Contract](CHUNK_CONTRIBUTORS.md).
[Executed evidence](VALIDATION.md).

Qualified paired handover continuation, 2026-10-20: wire/ALPN 10 adds authority
chunk offers and an opt-in `--chunk-stream` policy. Normal Bevy clients stage one
replacement, protect local/remote/recovery coverage, await exact withdrawal
acknowledgement and publish matching roots/actual Avian generations. Full
readiness waits for live BVH and render readiness. Delayed preparation, failed
replacement retention and distant independent frames pass on original fixtures.

The wider suite exposed sloped-wall contact drift. A deterministic multi-rate
regression and head-on Avian feature-manifold refinement fix it while preserving
exact contacts, tiny tangential input, diagonal sliding and retreat. Fresh fmt,
denied-warning Clippy/build and **140 Rust tests** pass (one private-map test
ignored by default and passed separately for both house routes at 20/30/60 Hz,
zero recoveries). The unchanged Python suites pass 46 tests. **Thirteen actual
two-client native cases**, 26 reports, plus invalid initial GLB rejection pass on
RX 7900 XTX/RADV Vulkan with active validation and the same final binaries.

The launcher remains the monolithic town; 30 Hz stays default. General geometry
contributors, imported tile partitioning, measured content budgets and wider
handover/streaming qualification remain open. Native Windows and MMO capacity
remain unqualified. No whole task or phase closes here. All changes remain local.
[Handover contract](CHUNK_HANDOVER.md). [Executed evidence](VALIDATION.md).

Qualified movement continuation, 2026-10-20: capsule sweeps now enforce authoritative/client chunk
coverage for gravity-driven horizontal motion as well as input. A reproduced
steep-slide boundary failure is fixed; targeted signed/distant 20/30/60 Hz cases
pass, including missing-corner coverage and release after loading. New support
validation/fallback avoids the large-convex-floor regression while preserving
exact persistent contacts. Fresh fmt, denied-warning Clippy/build, 134 Rust tests
and 46 Python tests pass (one private-map test ignored by default and passed
separately across both houses at 20/30/60 Hz with zero recoveries). Ten actual
two-client native cases and invalid-GLB rejection pass on RX 7900 XTX/RADV Vulkan
with active validation. Next: retain local/remote presentation and recovery
coverage, then pair live standard-Bevy/Avian publication with authority offers.
Normal chunk rendering remains initial-only; the launcher stays monolithic and
30 Hz remains default. Native Windows and MMO capacity remain unqualified.
[Contract](CHUNK_BOUNDARIES.md). [Evidence](VALIDATION.md). All changes remain local.

Qualified continuation, 2026-10-20: wire/ALPN 9 adds exact readiness acknowledgements emitted
after a fixed simulation step. Client confirmation requires the latest report and
a snapshot at or after its application tick, preventing old acknowledgements from
confirming a new request. Fresh fmt, denied-warning Clippy/build, 129 Rust tests
and 46 Python tests pass (one private-map test ignored). Nine actual two-client
native cases and invalid-GLB rejection pass on RX 7900 XTX/RADV Vulkan with active
validation. A diagnostic wall-steering fix passes two actual town runs against
unchanged contact limits; the earlier failed run is retained as rejected evidence.
The version-8 checkpoint below remains historical. Next: retained presentation,
recovery/gravity boundary coverage and paired dynamic publication, then explicit
geometry contributors/tile partitioning. Normal rendering remains initial-only;
the launcher stays monolithic and 30 Hz remains default. Windows remains
unqualified. [Contract](CHUNK_READINESS.md). [Evidence](VALIDATION.md).

Resumed at the user's request, 2026-10-20. Wire/ALPN version 8 now gates each
networked chunk actor's intentional horizontal movement on its own client's
reported coverage, in addition to authority residency. The normal Bevy client
reports only after initial assets and live Avian queries are ready. Fresh fmt,
denied-warning workspace Clippy, 126 Rust tests and 46 Python tests pass (one
private-map test explicitly ignored); the final workspace build also passed.
Eight actual two-client native cases and invalid-GLB rejection pass on RX 7900
XTX/RADV Vulkan with active validation. Windows remains unqualified. Normal chunk
rendering is initial-only; next are acknowledged withdrawal/retained presentation,
paired dynamic publication and geometry contributors/tile partitioning. The
launcher remains monolithic and 30 Hz stays default. [Contract](CHUNK_READINESS.md).
[Evidence](VALIDATION.md). All changes remain local.

Opt-in normal chunk startup, 2026-10-20: `--chunk-world` now uses shared bounded
startup metadata in the actual server and Bevy client. Initial standard GLB roots,
trusted spawns, matching live Avian collision, camera and ordinary encrypted player
presentation work through signed distant frames and render/wire rebases. The
version-2 session fingerprint also binds initial chunk IDs so an initial-only
client cannot admit with a different loaded set. Stable dataset identity and
client/server query frames remain separate. That admission guard and the initial
native path pass. A continuation now
protects wire/render frames for distant loaded actors and previous presentation
poses: the original 5.1 km two-client case passes with bounded common frames.
Fresh fmt, denied-warning Clippy/build, 123 Rust and 46 Python tests pass
(one private-map test explicitly ignored). Seven actual two-client native cases,
including the normal town route, and real GLB startup failure pass on RX 7900 XTX/
RADV Vulkan with active validation. Windows remains unqualified. 30 Hz stays
default. [Contract](CHUNK_STARTUP.md).
Normal executable loading is initial-only. Dynamic paired readiness/publication,
actual imported tile partitioning and broader streaming budgets remain open;
the launcher retains the monolithic town. All changes remain local.

Resumed at the user's request, 2026-10-20. The chunk-activation scaffold is now
integrated into fixed-step authority and a bounded owner-side management path in
the actual Renet runtime. Current capsule footprints/support/clearance and
world/generation guards protect publication; missing-neighbor movement preserves
jump/gravity. Targeted original-geometry and real two-client UDP cases pass.
Workspace checks pass: 116 tests plus the final expanded six-test chunk target
(one new case; 117 distinct passing tests), with the existing private-map test
explicitly ignored. Fresh fmt and denied-warning Clippy pass. Normal map/client
residency adoption is the next gate; the qualified town remains monolithic.
[Contract and scope](CHUNK_ACTIVATION.md). All changes remain local.

Schema-3 native qualification/adoption, 2026-10-20: general mesh collision and
explicit spawns now pass real two-client mesh/map checks, signed distant rebases,
spawn-free chunk publication/replacement/rollback/unload and normal-world startup
rejection. The launcher now selects `build/development-world-town-v3`, logical
`theia.development.lordaeron-town` revision 2; both actual default launch cleanup
cases verify that package. The tile still uses its existing conservative boxes
and authored floors; the new format does not infer complete model collision.
Native inspection fixed first-frame zero-delta Avian BVH freshness and a
highest-surface camera error inside general mesh rooms. Regressions and native
visibility/camera-distance checks now cover both. [Contract](GENERAL_COLLISION.md).
Fresh fmt, denied-warning Clippy/build, 110 Rust and 46 Python tests pass. Six
native two-client cases, four chunk cases, startup rejection and explicit/default
launcher cleanup pass on RX 7900 XTX/RADV Vulkan with active validation. Windows
remains unqualified. 30 Hz stays default. Next: live moving-actor chunk activation
and normal residency adoption; prediction remains open. [Evidence](VALIDATION.md).

Paused at the user's request, 2026-10-19, after a safe verification checkpoint.
Collision schema 3, explicit validated spawns, query-only chunks and static GLB
import/migration tooling are implemented. A combined-mesh low-ceiling grounding
bug is fixed. Staged revision-2 development maps preserve their geometry/visuals;
the launcher still defaults to the previously qualified schema-2 town. No new
native schema-3 qualification/default adoption is claimed. Resume with the
[checkpoint and next gate](GENERAL_COLLISION.md), then moving-actor chunk activation.
Fresh locked build, fmt, denied-warning Clippy, 108 Rust and 46 Python tests pass.
Separate headless staged-v3 town/farmhouse routes pass at 20/30/60 Hz with zero
recoveries. No native/GPU/Windows qualification was started for this new slice
before the user requested stopping. Earlier native records below are historical
baselines. Changes remain local. [Evidence](VALIDATION.md).

Imported-map controller adoption, 2026-10-19: normal imported authority now uses
the shared capsule controller, including gravity/jumping and nearby body support.
Trusted legacy spawn locations are resolved/validated once. Normal-world adoption
adds finite collision barriers; individual residency chunks retain open seams.
Unsupported falls or rejected solver states recover only that actor to its trusted
spawn, preserving input sequence/session and held jump. Recovery is observable and
ordinary route qualifiers reject it. [Contract](KINEMATIC_CONTROLLER.md). Content
profiles/packages stay unchanged; conservative boxes are still proxies. Explicit
spawn metadata, arbitrary collision mesh authoring and live actor/chunk activation
remain open. 30 Hz stays default; prediction and jump animation remain open.
Fresh fmt, denied-warning Clippy/build, 103 Rust and 43 Python tests pass. The
separate private-map test passes all six house-route cases at 20/30/60 Hz with zero
recoveries. Final native graybox/distant-rebase, stationary map jump, town/farmhouse
routes, full-tile Quinn and both launcher cleanup checks pass on RX 7900 XTX/RADV
with active Vulkan validation and zero unexpected recoveries. Windows remains
unqualified. [Evidence](VALIDATION.md).

Shared-controller continuation, 2026-10-19: graybox authority and the offline
sandbox now use one general Avian capsule controller for walls, headroom,
steps/slopes, stacked floors, gravity/jumping and nonblocking actors. Wire v7
adds held jump input; authority owns vertical/grounding state. Connected clients
present authoritative motion. [Contract](KINEMATIC_CONTROLLER.md). The imported
map deliberately retains its legacy grounded walker during migration; no normal
map jumping, prediction or live authority chunk adoption is claimed. Next:
qualify imported content/spawn/fall recovery with this controller, then moving
actor/chunk activation. 30 Hz remains default.
Fresh fmt, denied-warning Clippy/build, 98 Rust and 41 Python tests pass. Release
eight-peer scheduler qualification passes at 20/30/60 Hz. Final-binary native
Renet/Quinn jump cases at signed billion-metre origins, packaged town/farmhouse
regressions and both default launcher cleanup cases pass on RX 7900 XTX/RADV with active Vulkan validation. Windows
remains unqualified. [Evidence](VALIDATION.md).

Bevy chunk-publication continuation, 2026-10-19: the actual client now has an
opt-in `--chunk-smoke` diagnostic that stages one verified residency candidate on
Bevy's I/O task pool, loads standard GLB scenes, preserves retained roots and
publishes ready hierarchy visibility with matching actual Avian bodies. Last-stage
queries check the live BVH throughout transitions. Native screenshots independently
prove red unloads/blue loads, retained green remains, failed glTF preserves the old
pair and final unload removes all geometry. Origin and ±1-billion-metre cases pass.
[Contract](CHUNK_PRESENTATION.md). This is original-fixture qualification, not normal
tile partitioning, production streaming or multiplayer chunk adoption. Next: shared
general kinematic movement, then integrate bounded residency with live authority/
presentation after its activation/owner checks are qualified. 30 Hz remains default.
Fresh fmt, denied-warning Clippy/build, 85 Rust and 41 Python tests pass. Four
native chunk cases and the normal Renet two-client graybox pass on RX 7900 XTX/RADV
with active Vulkan validation. Windows remains unqualified. [Evidence](VALIDATION.md).

Chunk-residency precursor, 2026-10-19: an isolated original three-chunk fixture
now proves half-open ownership, explicit bounded residency and real Avian collision
load/unload. Protected actor owners cannot be removed; corrupt candidates preserve
the active scene. Origin and ±1-billion-metre runs pass with at most two active
chunks and a real thin-wall capsule hit. [Contract](CHUNK_RESIDENCY.md). Visual-root
descriptors track active ownership, but are not rendered/streamed assets. The normal
imported tile remains monolithic; synchronous proof loading never runs in its
simulation loop. Next: bounded Bevy visual consumption/publication, then the general
shared controller. The 30 Hz default is unchanged.
Fresh fmt, denied-warning Clippy and all 84 Rust tests pass; 41 Python tests pass
(37 asset plus four GLB). All three headless executable residency proofs pass.
Normal server dependencies remain renderer-free. No new renderer path is claimed
by this precursor; the immediately preceding native map checks remain valid
historical evidence. [Validation](VALIDATION.md).

Content-package continuation, 2026-10-19: the normal map launcher now requires a
versioned manifest with stable logical ID/revision and separate collision/visual
lengths and SHA-256 bindings. Shared Rust loading verifies collision packages;
the client streams visual verification before standard Bevy asset loading. The
headless server does not read visual files. Wire v6 carries the package fingerprint;
real secure admission rejects stale revisions. Existing ignored map fixtures have
revision-1 packages; malformed present manifests fail closed, while standalone
historical unmanifested fixtures retain legacy identity. [Contract](CONTENT_PACKAGES.md).
This is the row-5 content prerequisite. Chunk residency, general mesh collision,
stacked floors and falling/jumping remain open; 30 Hz remains the server default.
Fresh checks pass fmt, denied-warning Clippy/build, 79 Rust and 39 Python tests.
Packaged town/farmhouse native routes, Quinn graybox and both default launcher
cleanup cases pass on RX 7900 XTX/RADV with active Vulkan validation. Windows
remains unqualified. [Evidence](VALIDATION.md).

Scheduler continuation, 2026-10-19: the existing Renet authority now reports
bounded timing histograms, skipped intervals and separate simulation/transport-pump
diagnostics. A real eight-client UDP qualifier covers 20/30/60 Hz, burst/replayed/
wrong-epoch traffic, simulation-time input expiry, a controlled 250 ms authority
stall, recovery and removal of an over-budget sender while seven peers continue.
[Contract, measurements and limits](SCHEDULER_QUALIFICATION.md). This qualifies the
bounded graybox scheduler, not MMO capacity or full-map performance. Verification
also exposed and fixed cold query-scene initialization deadlock under concurrent
Bevy callers, with a two-worker/eight-caller regression. General action
schemas and crowd/render/soak baselines remain open. Next: shared content/collision
and general controller contracts, preserving existing map regressions.
Fresh verification passes fmt, denied-warning workspace Clippy/build, 73 Rust and
35 Python tests, the release eight-client qualifier and a native two-client
Renet graybox check with active Vulkan validation on RX 7900 XTX/RADV. Linux only;
Windows remains unqualified. [Evidence](VALIDATION.md).

Client render/physics rebase continuation, 2026-10-19: connected clients now
recentre local scenery, pawn/interpolation presentation and actual Avian static
collider Positions/Transforms together. Canonical globals and wire v6 are unchanged.
Standard PostUpdate physics refreshes the collider tree before Last-stage ground
probes/capture; camera queries explicitly cross the render/content-frame boundary.
[Contract and scope](FLOATING_ORIGIN.md). Server queries remain in the fixed bounded
content frame; multi-chunk ownership, streaming and dynamic-body rebasing remain
open. Earlier coordinate notes below describe the prior fixed client anchor.
Fresh verification passes fmt, denied-warning Clippy/build, 70 Rust and 35 Python
tests, five native two-client checks and both actual launcher cleanup cases.
The blacksmith/farmhouse routes exercised 22/24 shifts on their entering clients,
with actual rebased Avian queries and intact animation/camera checks. Device:
RX 7900 XTX/RADV with active Vulkan validation. Windows remains unqualified.
[Evidence](VALIDATION.md).

Coordinate/identity slice, 2026-10-19: connected actors now store checked f64
global positions; both transports carry an incarnation epoch and explicit snapshot
origin. The client reconstructs global positions before projecting into its fixed
local content frame. Wire origins rebase without moving authority poses or static
geometry. Protocol/ALPN is now v6 with a shared 76-byte welcome. Wrong-epoch input
is rejected before sequence/pending-input state changes. Renet remains the default.
[Contract, evidence and limits](COORDINATES_IDENTITY.md). Content/render/physics
anchors remain fixed for this fixture: automatic floating origin, multi-chunk
ownership, streaming and persistent character identity remain open.
Fresh checks pass fmt, denied-warning Clippy/build, 66 Rust and 35 Python tests.
Five native two-client checks pass (positive/negative distant Renet graybox,
distant blacksmith, ordinary farmhouse, distant Quinn), plus both default launcher
cleanup cases. Device: RX 7900 XTX/RADV with active Vulkan validation; Windows
execution remains unqualified. [Validation](VALIDATION.md).

Earlier Renet replacement qualification follows.

Renet replacement qualification, 2026-10-19: `python3 tools/play.py` and the
multiplayer qualifier now default to Renet. Deterministic real UDP profiles pass
10/20% loss, delay, duplication/reorder, monotonic acknowledgement, movement/stop
agreement and bounded queue/traffic checks. Two-second outages expire input and
recover the same session; a longer outage removes it after about five seconds.
Replaying 384 old-session packets cannot move a fresh-ticket replacement actor.
[Measured scope](RENET_QUALIFICATION.md). Quinn remains an explicit regression
option; standalone legacy CLI meanings are preserved. Next is live global/local
coordinate and entity/session identity integration, not combat or more houses.
Fresh qualification passes fmt, denied-warning Clippy/build, 59 Rust tests and
35 Python tests. Four native two-client checks pass (default Renet graybox,
blacksmith, farmhouse; explicit Quinn graybox), plus both actual launcher cleanup
cases. Device: RX 7900 XTX/RADV with active Vulkan validation. Windows execution
remains unqualified. [Evidence](VALIDATION.md).

Earlier executable-bridge continuation, 2026-10-19: the active task list now has a
[dependency-based execution order](engineering/EXECUTION_ORDER.md), with
foundations prioritized over gameplay. An opt-in [Renet executable bridge](NETWORKING_RENET.md)
connects the actual server, Bevy client and launcher using secure local tickets,
world checks and existing authority. `tools/play.py --transport renet --clients 2`
selects it. Quinn remains the default pending controlled network qualification.
Fresh continuation checks pass fmt, denied-warning Clippy/build, 56 Rust tests,
35 Python tests and three native server/two-client sessions (Renet graybox and
blacksmith route, Quinn graybox) on RX 7900 XTX/RADV with active Vulkan validation.
See [validation](VALIDATION.md). Windows execution remains unqualified.
No entire roadmap phase is complete; global runtime coordinates, general movement,
external identity and prediction remain outstanding.

Avian migration, 2026-10-19: the normal client now installs standard Avian physics
plugins and static world colliders. Avian `0.8.0-dev` is pinned to Git revision
`1070759b2145dc078abedace91dc7fe78013a45c` alongside exact Bevy `0.20.0`.
Server/offline movement, ground queries and camera volume queries use the shared
Avian backend. Characters remain nonblocking; connected movement remains server
owned. Existing terrain/building profiles are retained, with capsule sweeps
replacing custom box sweeps. Wire/ALPN version 5 prevents mixing old/new backends.
See [physics scope](PHYSICS_AVIAN.md). General multi-floor movement, gravity/jumping,
automatic building collision, prediction and navigation remain unfinished.
Earlier alignment notes below describe the prior implementation state.

Fresh qualification passes formatting, denied-warning workspace Clippy/build,
53 Rust and 35 Python tests, plus three native server/two-client Avian sessions
(graybox, blacksmith and farmhouse) on RX 7900 XTX/RADV with active Vulkan
validation. Windows graphics remain unqualified. [Evidence](VALIDATION.md).

## Earlier foundation migration

Alignment implementation, 2026-10-19: the handoff is now adopted as the active
architecture. Contributor rules and the future queue are aligned; scripted
platforms are allowed but unimplemented. A new headless `theia-core` supplies
checked f64/global and f32/local coordinate/frame contracts, seam tests and
timestep utilities. The existing authority now supports configurable 20/30/60 Hz
(30 default), preserved movement speed/input expiry and shutdown deadline metrics.
Wire/ALPN version 4 includes validated tick rate, used by client interpolation.
Renet/netcode 2.0.0 secure transport helpers and a two-client real UDP proof pass;
the desktop launcher still uses Quinn. The running map remains the local f32
terrain fixture, with no Avian/general collision or prediction yet.

Fresh full-workspace checks now pass using the existing ignored native SDK and
asset-tool environment: formatting, denied-warning Clippy, build, 51 Rust tests
and 35 Python tests. This resolves the environment blockers from the earlier audit
below. See [current alignment](engineering/ALIGNMENT.md),
[live task status](engineering/TASK_STATUS.md) and [validation](VALIDATION.md).

## Earlier planning and audit snapshot

Planning update, 2026-10-19: the user supplied a more defined engineering handoff.
Its four Markdown documents are preserved under [engineering](engineering/README.md),
with a [local repository assessment](engineering/REPOSITORY_ASSESSMENT.md). Future
work follows that reconciled queue. This documentation integration changes no
runtime behaviour: Renet, Avian, f64 global/local coordinates, prediction and the
planned gameplay systems remain unimplemented. It is not a fresh repository or
a rewrite. The assessment recommends preserving this repo and migrating foundations.

Fresh audit checks pass formatting, denied-warning headless Clippy and 23 headless
Rust tests (16 unit, 7 actual QUIC loopback). Full workspace Clippy/tests are
blocked by missing `wayland-client.pc`; the default Python asset suite cannot
import nine modules without its numpy/trimesh dependencies. Historical full-suite
and GPU results below remain dated evidence, not fresh audit results.

## Prototype implementation record — 2026-10-16

Bevy 0.20.0 client, local multiplayer, full-tile outdoor development world, orbit
camera and a separate GLB inspector,
2026-10-16.

Normal development launch: `python3 tools/play.py`, using the complete supplied
tile with enterable farmhouse and blacksmith main room, starting near the latter.

Implemented: a standard Bevy PBR scene made from original primitive geometry,
directional light/shadows, a local capsule pawn, fixed-step keyboard movement,
orbit/zoom camera, camera-relative controls, reset/quit and screenshot smoke
mode. There is no dependency on the old custom engine or its benchmark crate.

Linux build, denied-warning Clippy, formatting, 42 Rust tests and 35 offline
Python asset-tool tests pass.
Private native Wayland runs on RX 7900 XTX / RADV pass scripted movement,
static-world/camera checks and real 1280x720 screenshots with active Vulkan validation
and no validation errors.
The initial Bevy 0.19.1 probe exposed startup presentation diagnostics; the final
0.20.0 run passes without an application renderer workaround.
[Executed evidence and reproduction](VALIDATION.md).

The connected mode now uses one Rust authoritative server at 30 Hz, Quinn QUIC,
server-assigned identities, bounded input intents and full shared snapshots.
Clients cannot submit coordinates. Server speed, diagonal normalization, bounds,
stale-input rejection and input expiry control movement. The local pawn follows
server positions directly; remote pawns interpolate over a snapshot interval.
Disconnect removes the player; relaunching reconnects with a new ID. The offline
sandbox remains separate and R reset is disabled when connected.

Tests include player overlap/pass-through, actual encrypted loopback two-client movement,
disconnect/reconnect, untrusted-certificate rejection, malformed input and reordered
input/expiry. Two independent Bevy clients ran on RX 7900 XTX / RADV in a private
headless Wayland display. Both rendered both players and reported matching server
poses; Vulkan validation was active with no validation errors. This establishes
local multiplayer walking, not MMO scale or Internet deployment readiness.

Rendering and simulation now share immutable box geometry. The server stops
movement at the building, fixed door and rocks, with conservative pawn clearance
and wall sliding. Thin roads remain walkable on the flat ground. The camera
orbits/zooms and retracts against static geometry; the local body hides when the
eye gets very close. Tests cover door collision through an actual QUIC connection,
camera-relative movement, orbit limits, focus/cursor state and obstruction recovery.
The native world smoke reaches the fixed door and records camera retraction and
restoration before saving its screenshot. [Scope/limitations](WORLD_AND_CAMERA.md).

The user-provided `duskwoodhouse_medium.glb` passes Khronos validation and renders
through Bevy's standard loader in an independent inspection mode. It was moved
from Downloads into `apps/client/assets/reference/` after rendering succeeded,
with original bytes preserved. The file has 425 source positions, 672 triangles
and no UVs, authored normals, materials or textures. Bevy generates flat normals
and a default material. This qualifies geometry-only import, not a textured house.
The inspector frames actual scene bounds, shows a scale reference and can save
a real screenshot/report. It does not create gameplay collision or alter the
authoritative world. [Import workflow and export guidance](ASSET_IMPORT.md).

The newer farmhouse and re-exported human character also render with embedded
textures. The inspector now supports explicit named-clip playback with Bevy's
standard animation graphs. Stand, walk and run have actual sampled joint motion
and native rendered-pose evidence. The farmhouse passes glTF validation cleanly;
the character still has empty clips and incorrect accessor metadata, despite the
selected animations working. Large redundant vertex buffers also need cleanup.
Both source exports remain outside Git and unchanged. [Detailed model checks](ASSET_CHECK_2026_10_16.md).

The additional RustyWolfPup GLB also renders its embedded textures and plays stand,
walk, run and attack in the inspector on RX 7900 XTX / RADV with active Vulkan
validation. Its 66-joint skin and selected clips work, but the source fails strict
glTF validation with 3,084 accessor alignment errors. It remains an external local
compatibility reference, with no NPC/gameplay integration. The same detailed report
records the exact source hash and limitations.

An opt-in [reference game scene](REFERENCE_SCENE.md) now uses the human for local
and remote players, an idle nonblocking wolf enemy placeholder and a static
farmhouse. Player stand/walk/run follows actual movement. Both clients and server
use an enlarged shared solid building envelope; native two-client, locomotion,
wolf pass-through and house/camera checks pass. Derived models stay under ignored
`build/reference-scene/`; the default scene remains procedural. The prepared wolf
has zero glTF errors, while deeper human validation still finds skin-data errors.

The supplied Ruins of Lordaeron map tile also converts from textured OBJ/MTL into
a separately inspected GLB: 65,504 triangles and 256 textured patches, with zero
glTF errors/warnings. A further [static map assembly](MAP_OBJECTS.md) now includes
846 top-level placements and 624 nested building props using shared model parts.
The combined GLB renders through Bevy with no glTF errors/warnings. One empty
smoke emitter has no visible effect. [Static liquid surfaces](MAP_LIQUIDS.md) now
add 30 masked patches using source heights and a generic translucent PBR material.
The inspector waits for render pipeline readiness before captures; CPU asset loading
alone was insufficient for reliable screenshots. The original assembled GLB remains available in the inspector. A separate bounded
[terrain walking patch](TERRAIN_WALKING.md) now has server-owned height and slope
checks, matching visual/collision triangles, world identity checks and two rendered
multiplayer clients. An optional `--player-assets` mode now attaches the existing
human to both local and remote terrain players, with sampled stand/walk/run
evidence from real connected clients. A separate [authored farmhouse placement](TERRAIN_STATIC_OBJECTS.md)
now adds a fixed foundation and shared solid boundary, with server collision and
real camera obstruction/body visibility checks. A new [enterable ground-floor
fixture](ENTERABLE_HOUSE.md) uses measured wall boundaries, source floor triangles,
a permanently open doorway and a fixed ramp. Native networked entry, interior wall
contact, exit and re-entry pass with camera clearance below the ceiling. The subsequent
[full-tile development world](DEVELOPMENT_WORLD.md) now connects the
complete visual assembly to terrain movement, conservative imported object bounds
and blocked water cells. Precise mesh collision, general interiors and swimming
remain outstanding.
[Original map inventory](MAP_CHECK_2026_10_16.md).

No local prediction, general world mesh collision, animation blending,
targeting, combat, quests, inventory, accounts or persistence is implemented.
The wolf currently supplies presentation only, without server-owned NPC state or AI;
the older solid-house fixtures remain closed. The isolated ground-floor fixture
and the original farmhouse and blacksmith main room on the full map now have
authored access. Housing ownership/persistence and loft access are absent.
The network executables currently allow loopback only; eight peers is a prototype
cap, not a qualified capacity. Manual multiplayer keyboard interaction, adverse
network behaviour and the user-visible walkthrough remain outstanding.

Confirmed gameplay constraints: environment items are static (including doors),
and players never physically collide with other players or NPCs. NPC pass-through
is a requirement for future NPCs; this slice contains players only.

Earlier next-step plan (superseded by the October 19 engineering queue): use the
full supplied tile for the shared target/ability/health loop, with quests following.
The current queue first defines and qualifies the foundation migrations while
retaining this map as a fixture. Remaining character skin errors and replacement
original/licensed art still require qualification. The direct GLBs contain PNGs and preserve UVs;
the earlier missing-texture problem is resolved for these new exports.
The default/reference world still uses flat-ground boxes. The opt-in terrain patch
adds sampled height/slope movement; general stair climbing, jumping and arbitrary mesh collision remain unimplemented;
the new house permits only bounded small support seams and its authored ramp.
The [WoW-style game direction](GAME_DIRECTION.md), [network slice](NETWORKING.md)
and [roadmap](ROADMAP.md) distinguish intended gameplay from current capability.

Native Windows graphics and other GPUs remain unqualified.

The full supplied tile is now prepared under ignored `build/development-world/`,
with all 65,504 terrain triangles, the existing assembled props/buildings/liquids,
1,144 conservative object boxes and 1,372 blocked water cells. Runtime profile
schema 2 adds actual tile bounds and nearby spatial lookup. Schema-1 fixtures keep
their prior collision behavior. Wire/ALPN version 3 supports the larger coordinates;
clients and server must be rebuilt together. `python3 tools/play.py` launches the
world with an authoritative server and animated human; `--clients 2` adds a second
client. Native full-map, interior and graybox multiplayer checks and launcher
cleanup pass on RX 7900 XTX / RADV with active Vulkan validation. Buildings in this
map remain solid envelopes, while the separate authored
house fixture retains its interior. This replaces the full-map inspector-only
limitation for outdoor development, without adding NPCs, combat or swimming.

The default launcher now selects `build/development-world-access`, the complete
map with its original placed farmhouse ground floor made enterable. The building
retains its original scale/rotation and furnishings. Source floor triangles,
419 short rotated-wall envelope pieces and a fixed visible ramp qualify entry,
wall contact, exit and re-entry; other buildings remain solid and the loft stays
blocked. Development spawns move to the outdoor approach. Terrain ambient fill
is brighter, and Bevy mesh ray casts supplement static camera clearance without
including characters. Native two-client map-access, outdoor and prior interior
checks pass on RX 7900 XTX / RADV with active Vulkan validation. The diagnostic
harness now waits for both scenes to load before starting timed routes.
[Measured layout, evidence and limitations](MAP_FARMHOUSE.md).

The default launcher now selects `build/development-world-town`: the same complete
tile with both farmhouse access retained and the original blacksmith main room
opened. The new building contributes 226 source floor/four ramp triangles, 576
short main-room wall/exclusion boxes and 463 exterior forge/bay/porch exclusions.
Original source art and furnishings remain intact. Spawns move to its outdoor
approach. The new map's CPU routes pass for both buildings; native two-client
blacksmith entry/contact/exit/re-entry and normal launcher lifecycle checks pass
on RX 7900 XTX / RADV with active Vulkan validation. Camera mesh retractions are
observed on the actual blacksmith route. The forge annex and other unqualified
buildings remain blocked. [Scope and evidence](MAP_BLACKSMITH.md).
