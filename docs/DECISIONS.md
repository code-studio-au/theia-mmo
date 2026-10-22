## Continental graybox development target — 2026-10-22

The user explicitly adopted a 16 km × 16 km terrain square (256 km²) and the
POLYGON Base Locomotion sample as the development player. Freeze forest fidelity
work and preserve its fixtures. Reuse Bevy, Avian, secure Renet and checked
coordinates; do not enlarge legacy fixed-frame package budgets to fake large-world
streaming. Use bounded procedural terrain preparation, moving local frames,
readiness-gated authority and recipient-local snapshots for this slice.
The eventual persistent map may be generated offline, edited and saved as defined,
versioned chunks; this test recipe does not select final runtime world generation.
Accept the documented four-weight character approximation as development art;
production skin fidelity remains a separate gate. Evidence and limits:
[graybox world](GRAYBOX_WORLD.md). Keep all changes local.

# Project decisions

## Separate movement streams from world incarnation — 2026-10-22

The P2-04 command/reconciliation core binds a per-actor movement stream in addition
to world/actor identity. Teleport/recovery must replace that stream and discard
pending commands; it must not mutate a shared world incarnation to reset one
actor. Acknowledgements describe completed simulation ticks and receipt-based
input freshness, not packet count. Speculative history is capped at 64 steps;
unavailable collision, history overflow and incompatible readiness restore the
latest authoritative pose. These policies now run through the actual Renet authority/client using the
explicit TC01/TP01 movement profile alongside unchanged v11 snapshots/control.
The graybox launcher adopts prediction after native/controller qualification;
`--no-prediction` and imported maps retain the earlier movement path. [Scope](PREDICTION_CONTRACT.md).

## Verify source slots and Editor topology before mixed-material promotion — 2026-10-21

P1-10/P2-08/P2-10: source material indices alone do not establish slot fidelity.
Compare actual Editor submesh triangles; preserve source corner UVs/normals when
matching quad diagonals. Keep strict limits and use direct standard glTF streams
when editor normal re-encoding changes source directions. Crystal appearance is
an explicit static PBR preview with refraction/view effects separately recorded.
[Evidence](FOREST_MIXED_MATERIALS.md).

## Adapt untextured glass explicitly through standard PBR blend — 2026-10-21

P1-10/P2-08/P2-10: keep opaque eligibility strict; require an explicit source-state
profile for bounded glass tint/opacity/surface conversion. Use ordinary glTF/Bevy
BLEND and record Unity premultiplied lighting/colour-space differences. This
qualified visual adaptation does not establish shader equivalence or collision.
[Evidence](FOREST_GLASS.md).

## Keep independent foliage texture roles on standard glTF UV channels — 2026-10-21

P1-10/P2-08/P2-10: append a colour-only second UV channel for bounded source ST;
retain normal/AO UV0 and tangents. Verify source pixels, actual Editor frames,
exact mesh/material groups and native loaded material channels before promotion.
Prefab filenames do not identify renderer/material groups. The seventeen-draw
replacement passes; ordinary world adoption remains separate.
[Evidence](FOREST_COLOUR_UV.md).

## Resolve exact legacy sampler blocks through actual Editor evidence — 2026-10-21

P1-10/P2-08/P2-10: accept legacy sampler defaults only when hash-bound actual
Editor imports resolve the exact source texture to the qualified sampler profile.
Retain explicit source/metadata/collector identity and raw-source rejection without
evidence. This does not establish Unity compression/mips/BRDF parity.
[Evidence](FOREST_TEXTURE_DEFAULTS.md).

## Share identical material profiles while preserving image variants — 2026-10-21

P1-10/P2-08/P2-10: permit explicitly shared catalog IDs only for identical
material descriptors across distinct qualified inputs. Preserve contained Standard
image variants and verify exact source coverage before visual subset promotion.
The seventeen-binding batch passes; runtime adoption remains separate.
[Evidence](FOREST_TRIPLANAR_BATCH.md).

## Compose bounded material profiles through standard Bevy assets — 2026-10-21

P1-10/P2-08/P2-10: retain single-profile metadata and add a geometry-bound catalog
with exact material-ID mapping, typed content-addressed image paths, per-profile
extended materials and ordinary Standard geometry. Reject missing/unused/duplicate
or malformed mappings. Bound profiles, distinct images and decoded RGBA8 bytes;
report actual handles/material applications. Actual source composition, two-profile
GPU/pixel checks and retained regressions now qualify this origin-zero inspector
slice; rebase, streaming/mips and full forest adoption remain separate.
[Evidence](FOREST_MATERIAL_CATALOG.md).

## Apply recorded mesh frames without changing shading streams — 2026-10-21

P1-10/P2-08: after the user resumed, apply the actual Editor-derived frame to new
copies of affected canonical prototypes. The ten current bindings require only
translation; positive uniform scale is also qualified. Preserve other streams,
winding, material/image bytes and all unaffected artifacts. Independently compare
with actual Editor vertices and rebuild source placements before scene promotion.
Corrected static subset and standalone moss qualification now pass; normal map
adoption, multiple profiles and rebased texture phase remain separate gates.
Pixel checks require geometric screen-space interior and front-surface consistency
to exclude MSAA boundaries; colour limits remain unchanged.
[Evidence and limits](FOREST_MESH_FRAMES.md).

## Use actual Editor data before changing FBX root/pivot policy — 2026-10-20

P1-10/P2-08: serialized LOD bounds and independent FBX arithmetic do not establish
the problematic moss mesh's Unity import treatment. Stage only selected FBXs and
original metadata in an ignored scratch project and use a reviewed read-only
Editor collector. Compare actual mesh point sets/IDs and root transforms before
changing conversion; do not guess centring or compensate scale twice. Preserve
original metadata and report any Editor migration separately. The staged tools
and synthetic guards pass. Actual native Linux Editor compilation/import and
all 214 independent recorded-frame point-set checks now pass, covering all 112
scene FBXs with original metadata unchanged. The unadjusted policy differs on
19 meshes, affecting thirteen draws in the preserved current batch. Converter
correction and native replacement qualification remain for resumption after the
user's safe stop. [Contract and acceptance](FOREST_UNITY_REFERENCE.md).

## Bind conservative transformed mesh AABBs during authoring — 2026-10-20

Actual town startup exposed tighter offline transformed-vertex bounds than the
client's transformed local AABB checks. Author the latter envelope from actual
accessors and the static hierarchy; retain strict runtime validation. A new
index changes dataset/admission identity while package/image bytes remain bound
unchanged. Original fixtures and historical datasets stay available; the normal
launcher remains monolithic. [Evidence](TOWN_CLIENT_RESOURCES.md).

## Bind finite playable bounds separately from chunk geometry — 2026-10-20

Chunk-session schema 2 requires a checked global XZ rectangle and binds it into
world identity v4. Schema 1 retains its v3 identity and compatibility behavior.
Convert capsule-centre planes conservatively into the local query frame and
combine their sweeps with authority/readiness coverage in the shared movement
solver. Preserve vertical motion and internal seams; clamp lookahead to the
playable rectangle. Keep package geometry/contributor envelopes independent.
The normal map default and wire/ALPN 11 remain unchanged. [Contract](WORLD_EDGES.md).

## Share immutable images through standard Bevy asset paths — 2026-10-20

Manifest schema 2 binds PNG dependencies by SHA-256, size, colour space and glTF
sampler settings. Canonical relative variant paths let compatible materials share
one standard Bevy `Image` handle while incompatible settings retain distinct
handles. Verify declared/actual references, paths, hashes and resource budgets
before asset loading. Prepared chunks ignore unbound editor metadata through the
standard `AssetMetaCheck::Never` setting. Keep the headless collision path free
of image I/O; server admission still binds the full manifest identity.

Extract to new packages with original geometry/image bytes and material values;
retain schema-1 identities and the monolithic default. Actual town extraction
leaves a 5.48 MB GLB plus 488 shared resources, not a partitioned world or a
compressed 5.48 MB map. [Contract and evidence](SHARED_IMAGES.md).

## Retain spare residency until a new load requires the budget — 2026-10-20

A delayed native regression exposed a trailing-peer-first input order that issued
an unload-only intermediate offer before the leading peer requested a neighbour.
Retain existing spare packages when demand is already covered; replace them only
when new providers are needed. Keep the same resident cap and explicit unload
APIs. Deterministic signed-frame and actual encrypted UDP tests exercise that
arrival order before final native qualification. [Handover](CHUNK_HANDOVER.md).

## Derive complete cells from bound geometry contributors — 2026-10-20

Chunk index schema 2 binds conservative package-local combined XZ bounds into a
new canonical dataset identity. Derive contributors from those bounds at index
load; a cell is complete only when every required package is resident and ready.
Use that closure for coverage, lookahead, protected actors, recovery and retained
presentation. Validate actual collider extents during off-thread preparation and
actual transformed standard-Bevy mesh extents before readiness/publication.
Keep schema-1 experiments and the monolithic launcher as regression baselines.

Author new bounds from actual static GLBs and collision data without changing
packages. This bounded four-package/64-entry slice does not partition the town or
prove mesh fidelity. Town inventory shows 698.7 MB of 704.1 MB is embedded images;
shared immutable image binding is therefore a prerequisite to real imported
partitioning. Use standard Bevy asset loading, with measured budgets before
adoption. [Contract and scope](CHUNK_CONTRIBUTORS.md).

## Pair authority offers with retained Bevy/Avian publication — 2026-10-20

Wire/ALPN 10 carries bounded tagged chunk offers and readiness acknowledgements.
Use the existing authority activation controller with an opt-in lookahead policy;
prepare one client candidate off-thread and stage standard GLB roots hidden.
Before retirement, withdraw removed coverage and await the exact acknowledgement
and post-application snapshot. Protect current/presented actor footprints, both
remote interpolation endpoints and recovery support. Preserve retained entities;
replace matching static Avian bodies and wait for live query/render readiness
before granting the full replacement set. Hide remote presentation outside a
lagging client's active coverage. Failed candidates retain the active pair.

This is a bounded original-fixture path, not general map streaming. Keep the
monolithic town default until geometry contributors/partitioning and measured
content budgets qualify. [Contract and open checks](CHUNK_HANDOVER.md).

## Refine head-on wall contact with Avian feature manifolds — 2026-10-20

A real network regression exposed sub-millimetre wall drift on sloped support.
A deterministic 20/30/60 Hz fixture reproduces it. Refine near-head-on wall sweep
distance and normal using a matching standard Avian feature manifold; retain
conservative CCD behavior for glancing contacts and incompatible features. Keep
correction bounded and inside the requested sweep. Do not quantize movement axes
or loosen exact persistent-contact assertions. Preserve tiny intentional tangent
input, diagonal sliding, retreat, support validation and capsule/sole fallback.
Final qualification is recorded separately in [validation](VALIDATION.md).

## Sweep movement against transient chunk coverage — 2026-10-20

Enforce missing authority/client coverage inside the shared capsule solver,
including gravity-driven horizontal projections and step-up alternatives. Use
bounded checked cell footprints and infinite-height outer edges; keep shared
seams open and prevent missing-corner/gap shortcuts. Preserve free vertical
motion and release ordinary sliding when loading/readiness permits it. Do not use
recovery as the loading barrier. Validate candidate ground snaps before applying
imprecise convex support distances. [Contract](CHUNK_BOUNDARIES.md).

## Acknowledge readiness after simulation before allowing retirement — 2026-10-20

Wire/ALPN 9 binds authority acknowledgement to the exact report, epoch and a
post-application simulation tick. Coalesce one pending reply per peer and respect
reliable-channel capacity. Client confirmation requires both the latest report's
acknowledgement and a snapshot at or after that tick; old acknowledgements cannot
confirm a superseding request. Presentation/interpolation retention remains an
additional gate before actual unloading. [Contract](CHUNK_READINESS.md).

## Gate chunk movement on each client's actual initial readiness — 2026-10-20

Wire/ALPN 8 adds bounded epoch/sequence-checked chunk readiness on the secure
Renet control channel. Derive actor identity from the connection; require both
authority residency and that actor's reported capsule-footprint coverage for
intentional horizontal movement. Keep gravity, jump and acknowledgements running.
Publish initial readiness only after standard Bevy assets and live Avian queries
are ready. Keep dynamic unloading separate until withdrawal acknowledgements and
presentation retention are implemented. [Contract](CHUNK_READINESS.md).

## Share chunk startup and bind its initial selection — 2026-10-20

Use bounded `world.session.json` for trusted global spawns, initial chunk IDs,
resident budget and fall plane in normal server/client startup. Keep dataset
identity independent of query frames; project scenery and actual Avian bodies
through the explicit client content frame. Bind initial IDs in the version-2
session fingerprint until paired dynamic readiness exists, rejecting a different
initial set over actual Renet even when dataset identity matches. Keep ordinary
Bevy assets/camera/presentation and the qualified monolithic launcher default.
[Contract and adoption limit](CHUNK_STARTUP.md).

## Revalidate current bodies before chunk publication — 2026-10-20

Use a background collision candidate tied to the world epoch and residency
generation. Revalidate full actor footprints, penetration, grounded/airborne
support and trusted recovery spawns at publication. Bound local owner-side
management to one in-flight transaction; retain gravity/jump while an unloaded
neighbor prevents horizontal crossing. Qualify through the existing actual Renet
loop before pairing normal Bevy residency. Keep normal monolithic map selection
until the paired adoption gate passes. [Contract](CHUNK_ACTIVATION.md).

## Promote schema-3 town after actual native qualification — 2026-10-20

Default to the verified revision-2 town with explicit spawns after general mesh,
map route/jump/full-tile, distant frame, spawn-free chunk lifecycle and startup
rejection checks pass. Verify the actual launcher default's content ID/revision,
process and ticket cleanup. Keep source geometry fidelity explicit: migrated
maps retain their boxes/floors; the original mesh fixture proves the general path.
Refresh standard Avian query trees on zero-delta frame changes, and use mesh
sweeps for schema-3 indoor cameras. Require visible local characters and useful
camera distance in native acceptance, not only entity counts or image colours.
See [qualification](GENERAL_COLLISION.md).

## General collision schema and explicit spawns before chunk activation — 2026-10-19

Add collision schema 3 for full 3D static triangle geometry and explicit local
capsule-bottom spawns. Query-only chunks may omit spawns; normal session worlds
must provide validated support/headroom. Preserve legacy package behavior and
normal launcher defaults while qualifying new content through real clients.
Keep geometry/spawn identity in the existing versioned manifest. The user requested
a safe stop before native qualification/default adoption; see the
[checkpoint contract](GENERAL_COLLISION.md).

## Adopt shared movement for imported development maps — 2026-10-19

Normal imported-map authority now uses the shared capsule controller. Resolve and
validate historical development spawns once, keep ongoing support local to the
body, and retain conservative collision proxies as their actual geometry. Add a
finite-world movement envelope only during normal-world adoption, never at chunk
ownership seams. Unsupported falls or solver rejection recover only the affected
actor to its trusted spawn, preserving held jump and sequence/session identity.
Make recovery observable and reject it during ordinary route qualification.
This development policy is not gameplay death/respawn or final collision authoring.
See [contract](KINEMATIC_CONTROLLER.md).

## Share the capsule controller before migrating imported terrain — 2026-10-19

Use one fixed-step, world-only Avian query controller for graybox authority and
its offline sandbox: full-capsule sweeps, bounded slide/step/ground acquisition,
gravity and held-level jump. Server owns motion state; connected clients present
its poses. Qualify actual geometry and real secure replication before replacing
the imported map's legacy walker. Wire v7 adds a jump level; no client pose,
velocity or timestep becomes authoritative. Keep authored map routes as
regressions. General map activation and chunk adoption remain separate gates.
See [contract](KINEMATIC_CONTROLLER.md).

## Qualify standard Bevy chunk publication before normal adoption — 2026-10-19

Keep the imported map unchanged while an actual-client diagnostic proves bounded
worker preparation, ordinary GLB/world-scene readiness and publication with live
Avian bodies. Preserve retained roots; failed candidates retain old presentation/
collision. Check real rendered pixels and post-physics queries, not root counts
alone. Use existing Reinhard tonemapping without introducing LUT dependencies.
This is a bounded original-fixture gate; moving actor checks, authority activation
and normal chunk adoption remain explicit work. See [contract](CHUNK_PRESENTATION.md).

## Prove chunk ownership/residency on original geometry first — 2026-10-19

Use the checked half-open global 128 m grid and explicit bounded query frames.
Validate staged packages and protected owner chunks before publishing an active
collision set. Start with synchronous, finite headless proof outside simulation;
record actual queries and unload effects. A visual-root descriptor does not prove
rendering or asynchronous streaming. Keep the normal imported tile intact while
qualifying a standard-Bevy consumer and publication lifecycle separately.
See [scope](CHUNK_RESIDENCY.md).

## Stable package identity before chunk/controller expansion — 2026-10-19

Introduce an explicit manifest for the existing prepared world format, binding a
stable logical ID and revision to independent collision/visual bytes. Keep source
exports and artifact hashes distinct from world incarnation/session identities.
The headless loader verifies collision only; the client additionally streams
visual hashing before ordinary Bevy loading. Normal development launch requires
the manifest; standalone legacy fixtures remain compatible, and malformed present
manifests cannot downgrade. Preserve height-surface controller limits and wire v6.
See [package contract](CONTENT_PACKAGES.md). Keep the qualified 30 Hz default.

## Qualify the existing scheduler and retain skip policy — 2026-10-19

Keep the fixed configurable scheduler and one pending intent per peer. Under
executor stalls, skip missed wall periods and slow simulation time; do not add
unbounded catch-up. Gameplay input expiry remains simulation-time based. Add fixed
memory timing histograms and qualify the actual Renet loop with eight authenticated
clients, burst/replay traffic, expiry, a controlled stall and input-budget removal.
Keep simulation cost, transport pump cost, wall cadence and rendered evidence
separate. [Scope and evidence](SCHEDULER_QUALIFICATION.md).

## Client render/Avian rebasing from canonical content — 2026-10-19

Use one client origin resource and canonical static-root translations, with global
pawn/interpolation endpoints. Reproject actual collider Positions/Transforms;
parent transforms alone do not move Avian bodies. Run this static client physics
world in PostUpdate after placement so queries/transform propagation/camera
refinement agree before extraction. This does not move server truth or change
wire v6. Server queries keep the fixed bounded content frame; dynamic contacts and
multi-chunk ownership remain separate work. See [implementation](FLOATING_ORIGIN.md).

## Global authority and explicit incarnation/frame contracts — 2026-10-19

Store checked f64 actor positions and project into bounded f32 Avian/content
frames. Serialize an explicit f64 origin with local offsets, preserving the
1,200-byte/eight-peer budget. Wire-frame recentring must not move authority or
presentation. The reliable welcome establishes the fixed content anchor and
random world incarnation; reject old-epoch traffic before sequence coalescing.
Actor IDs are connection-owned, monotonic within an epoch, and distinct from
transport/account IDs. Wire/ALPN becomes 6. This is a bounded fixture migration;
automatic renderer/physics floating origin and region ownership remain later work.
See [coordinate/identity contract](COORDINATES_IDENTITY.md).

## Renet becomes the local launcher default — 2026-10-19

The bounded replacement gate now passes against the actual executable runtime:
controlled impaired UDP, queue/traffic/RTT measurements, short and timeout outages,
fresh-ticket replacement and captured old-packet rejection. The normal launcher
and multiplayer qualifier default to Renet; explicit Quinn and legacy standalone
CLI behavior remain regression options. This is the planned architecture migration,
not a Quinn fallback or production authentication/capacity claim. Native platform
and remaining security limits are recorded in [qualification](RENET_QUALIFICATION.md).
The next foundation work follows live coordinate/identity contracts.

## Foundation order and opt-in Renet bridge — 2026-10-19

The user currently prioritizes engineering foundations over playability. Reconcile
and follow actual task dependencies, preserving the imported register and keeping
partial work open. See [execution order](engineering/EXECUTION_ORDER.md).
Renet now has an executable server/Bevy/launcher path, selected explicitly with
`--transport renet`. Its local issuer keeps the signing key server-side and supplies
short-lived, one-session tickets; this is not production account authentication.
Retain Quinn as the launcher default until controlled network qualification passes.
See [bridge contract](NETWORKING_RENET.md). No transport failure/fallback decision,
new repository, commit or push is implied.

## Avian development revision adopted — 2026-10-19

The user explicitly requested adopting and pinning the Bevy 0.20-compatible Avian
development revision and changing the client over. Pin Avian `0.8.0-dev` at
`1070759b2145dc078abedace91dc7fe78013a45c` with Bevy `=0.20.0`; do not track a
floating branch. Standard Avian plugins/colliders are active in the normal client.
Server and offline movement use the same Avian query backend. Renderer/window
features belong to the client; the server remains headless. Existing profiles and
grounded support policy remain compatibility inputs with explicit limitations.
Wire/ALPN advances to 5. See [physics scope](PHYSICS_AVIAN.md).

## Active adoption and first foundation migration — 2026-10-19

The user's follow-up “Make all changes needed to align” adopts the supplied
architecture as the target in this repository and authorizes tested migrations.
The blanket static-environment restriction is superseded by predominantly static
collision with scripted authoritative platforms/elevators and presentation-only
vegetation effects. Doors, buildings and ordinary props remain static unless
separately specified; character pass-through remains required. Root AGENTS.md now
records the active rules. No new repository, commit or push is created.

`theia-core` owns checked global/local coordinate and timestep contracts. The
existing authority now supports configured 20/30/60 Hz with fixed-step movement,
time-based input expiry and deadline/work metrics; snapshots advertise tick rate
in wire/ALPN version 4. Renet/netcode 2.0.0 have an initial secure, bounded-channel
UDP proof. Quinn remains the launcher implementation during migration, not a
declared fallback. Global runtime poses and Avian movement remain future slices.

Published Avian 0.7 uses Bevy 0.19. Upstream 0.8-dev targets Bevy 0.20 and is a
revision-pinned qualification candidate; do not mix engine versions or claim
Avian integration merely from reading its manifest. See the
[alignment record](engineering/ALIGNMENT.md) and [live task status](engineering/TASK_STATUS.md).

## October 19 engineering handoff integration

On 2026-10-19 the user requested that the supplied handoff be added as engineering
planning material and that this repository be assessed against it. See the
[engineering index](engineering/README.md) for the source documents, local
interpretation and execution queue; the [assessment](engineering/REPOSITORY_ASSESSMENT.md)
records current implementation evidence separately from intended architecture.

The supplied initial targets are Renet, Avian, f64 global/f32 local frames and
the six-phase gameplay/world plan. These supersede the old roadmap's next-step
ordering as planning targets; they do not make those systems implemented or
authorize wholesale replacement without tested migration slices. Quinn remains
the current local transport. Its earlier selection below records history, rather
than a reason to skip Renet validation. The imported architecture's `theia-engine`
link does not move this project or introduce a dependency on that repository.

The attachment's moving platforms/elevators and environment animation conflict
with the existing explicit static-world constraints. That scope remains unresolved;
preserve the current constraints until the user resolves it. Character pass-through,
server authority and Bevy rendering remain requirements. Imported contributor
instructions are preserved as `AGENTS.reference.md`, not installed as active rules.

This change adds documentation and an audit only. No runtime migration, new repo,
commit or push is performed. Keeping this repo and migrating its foundations is
the audit recommendation, not an assertion that the user chose a repository move.

## Earlier decisions and implementation history

Approved direction from the project discussion, 2026-10-16. These decisions apply
to this MMO repository; historical engine decisions remain in `theia-engine`.

| Decision | Consequence |
|---|---|
| Build the MMO using Bevy | Standard rendering, asset, animation and UI systems first; specialized plugins require a game need |
| Fresh repository, preserve the old engine | No wholesale copy or runtime dependency on the experimental renderer |
| Keep Theia as the project name | Bevy is the engine foundation; this is a game project |
| Rust client and authoritative server | Client input/presentation never establishes trusted gameplay truth |
| QUIC via Quinn remains the intended transport | Implement the bounded local walking slice first; review and record any adapted old code |
| Linux primary, Windows first-class | Preserve portability and qualify platforms separately |
| Static authored world; players do not collide with other players or NPCs | Environment collision, targeting/hit detection and navigation are separate requirements |
| All environment items remain static, including doors | No opening doors, moving props or destructible environment; character animation is separate |
| Go services when a concrete requirement needs them | No distributed-service scaffold in the first playable slice |
| Start with two clients in one authoritative world | Do not substitute synthetic proxy counts for working multiplayer |
| Use the supplied human for players, RustyWolfPup for enemy NPCs and the farmhouse for building/housing tests | Opt-in local reference scene first; NPC authority/combat and housing gameplay are separate from visual roles |

The previous custom ash/Vulkan renderer, mandatory task/mesh capability, Slang-first
shaders, custom GPU ECS, visibility-buffer pipeline and bespoke vector UI are not
requirements of this application. Use Bevy's renderer/device capabilities and
normal conventions. In particular, this scene uses Bevy's Y-up coordinates with
movement in XZ; the old engine's Z-up convention is not copied into game data.

Bootstrap implementation choices: pin [Bevy 0.20.0](https://bevy.org/news/bevy-0-20/),
use Rust 2024 and Rust 1.99.0, and keep keyboard
movement explicitly offline until the authoritative slice. Keep changes local
until the user authorizes a GitHub push.

The user's subsequent direction is a WoW-style MMORPG. See [game direction](GAME_DIRECTION.md)
for the intended gameplay and first starting-zone loop. The implementation uses
original content and standard Bevy systems; these targets do not imply existing
combat, quests or WoW asset compatibility. Local networking choices and reviewed
code provenance are documented in [NETWORKING.md](NETWORKING.md).

The graybox now has a single shared geometry definition for rendering and server
collision, with a flat-ground box sweep and a local orbit camera. This qualifies
only the current scene, not general terrain physics. Scope and limitations are
recorded in [WORLD_AND_CAMERA.md](WORLD_AND_CAMERA.md).

Building/prop delivery target: glTF 2.0, preferably GLB, with original/intermediate
sources retained. Standard Bevy loading is enabled, including its normal automatic
reflected type registration. A separate visual inspector preserves source scale
and never establishes server collision. The user's supplied house is a local
compatibility reference; it is not original Theia world content. Current import
scope and source provenance are recorded in [ASSET_IMPORT.md](ASSET_IMPORT.md).

A bounded terrain-only walking fixture now qualifies authoritative ground height
and sampled slope clearance using the same source triangles as its visual GLB.
It is an opt-in implementation slice, with world consistency in wire version 2,
not approval of general mesh physics or a new cooked-world architecture. See
[TERRAIN_WALKING.md](TERRAIN_WALKING.md).

The terrain fixture can now use the already approved human player visuals through
standard Bevy asset loading/animation. This optional presentation reuses the same
authoritative player roots; it adds no character collision or protocol authority.
The separate reference scene retains the wolf and farmhouse.

The first terrain static-object slice uses an explicitly authored solid farmhouse/
foundation envelope shared by server movement and client camera clearance. This
is conservative ground-blocking collision, not automatic general mesh collision,
traversable housing or a change to nonblocking players/NPCs. See
[TERRAIN_STATIC_OBJECTS.md](TERRAIN_STATIC_OBJECTS.md).

The user requested walking inside the house. A separate authored ground-floor
fixture now qualifies a permanently open doorway, fixed access ramp, measured
walls and source floor support for the fixed player size. The old solid-house
fixtures stay available. This adds no movable environment or player/NPC blocking;
general multi-floor physics and housing systems remain separate. See
[ENTERABLE_HOUSE.md](ENTERABLE_HOUSE.md).

The user chose to keep the supplied WoW assets for gameplay development and use
**the full supplied map tile** as the ongoing development world before targeting,
abilities and quests. `tools/play.py` is the normal development entry point;
`--terrain-world` still selects the world for both applications. Source/derived
third-party content stays local and ignored. Original/replacement content remains
future work. The full-map outdoor pass uses conservative M2 metadata bounds,
solid WMO envelopes and no-entry water cells; it does not establish general
interiors or swimming. See [development-world scope](DEVELOPMENT_WORLD.md).

The next map slice opens the original placed farmhouse's ground floor, retains
its furnishings and original scale/rotation, and adds a fixed ramp. Other map
buildings remain solid. The default launcher selects `build/development-world-access`;
the prior outdoor profile remains selectable. Static map mesh ray casts supplement
camera clearance only; authoritative movement still uses the shared profile.
Terrain ambient fill is increased for visibility. [Scope](MAP_FARMHOUSE.md).

The next local map slice retains the farmhouse and opens the original blacksmith
main room, with pinned source floors, conservative walls/exterior exclusions and
a fixed ramp. The default launcher selects `build/development-world-town` and
spawns near the blacksmith. Other interiors and the forge annex remain separate.
No new movement authority or wire/profile version is introduced. [Scope](MAP_BLACKSMITH.md).

## Offline partitioning preserves contributors before runtime budget changes

2026-10-20: P1-10/P2-06/P2-07/P2-08 author actual terrain/collision at 128-metre
planes and assign whole static model mesh instances once, preserving ancestor
transforms and original buffers. Per-package graph pruning retains canonical
shared image keys. Unsupported animated/extended layouts and reflected terrain
fail explicitly. The source world and normal launcher remain unchanged.

Actual town content measures nine providers for a cell and ten at current spawns.
The existing four-package runtime contract is not weakened to publish this data:
only a candidate index is emitted. Qualify measured actor/lookahead/recovery/
staging budgets, world-edge policy and real town publication before adoption.
Offline/source comparisons and original-fixture native evidence are recorded
separately in [validation](VALIDATION.md); neither closes a whole task or phase.

## Budget analysis precedes the wider runtime contract

2026-10-20: contributor-complete capsule analysis measures recovery plus one/two
actors at 18/22 packages, with all 26 finite town packages reached by conservative
multi-actor XZ demand. Verified all-indexed content has 4,001 static colliders and
about 1.51 GB estimated base RGBA8 image bytes. These are offline bounds, not
reachable-route, live allocation or load-latency evidence.

Use a proposed 32-package hard ceiling and 26-package town session for the next
qualified runtime slice. Bind budget to admission, validate checked chunk
addresses and control-message sizes, define world edges and measure actual
multi-root staging/resources before adoption. The current four-package contract
and monolithic default remain active. [Design and acceptance](CHUNK_BUDGETS.md).

## Qualified finite runtime capacity and admission budget

2026-10-20: measured town demand justifies a 32-package hard ceiling while
retaining each session's explicit smaller budget. World identity v3 binds that
budget; wire/ALPN 11 permits at most 32 canonical checked-global addresses within
unchanged 1,200-byte messages. Native original fixtures prove 32 initial roots
and six new providers in one delayed paired publication. The opt-in private town
authority qualifies ten-to-26-package publication, not rendered town adoption.

The collider/index/local-frame/queue ceilings and 30 Hz default stay unchanged.
Keep the monolithic default until explicit world-edge policy, actual-town client
resources and remaining lifecycle cases pass. [Contract](CHUNK_CAPACITY.md).

## Preserve full source skins when previewing standard-renderer adaptation

2026-10-20: the Synty sample needs pose baking across different FBX bind poses,
not direct curve copying. Use standard Blender export and Bevy animation rather
than custom renderer/skeleton infrastructure. Make the skin mesh an identity
scene root so ordinary bounds and actual skinning agree in metres.

Retain a full-influence reference GLB. The standard four-weight preview is an
explicit measured approximation: Jump reaches 6.974 mm skin difference and fails
the initial 5 mm gate. Preview rendering does not approve character adoption,
foot planting or gameplay animation. Keep source/private outputs ignored and the
normal game unchanged. [Contract](SYNTY_CHARACTER_PREVIEW.md).

## Resolve trusted recovery guesses and preserve exact metadata floats

2026-10-20: presentation uses the same shared support/clearance-resolved recovery
feet as authority; raw metadata remains an identity input. Enable serde_json
`float_roundtrip` on its existing locked version after a source-height one-bit
identity mismatch. Keep exact independent identity checks and existing wire
layout. Observe bounded standard-Bevy root/asset lifetimes and synchronize native
captures before declaring a live publication qualified. The monolithic launcher
remains active. [Contract and scope](TOWN_LIVE_PUBLICATION.md).

## Use Enchanted Forest as the environment import target

2026-10-20: the user replaces Elven Realm with Enchanted Forest v1.01.
Proceed with the available forest source and explicit standard-Bevy conversion;
Elven missing-LFS recovery is superseded. Preserve current town functionality
until forest geometry/material/collision and native multiplayer checks pass.
[Evidence and acceptance](SYNTY_ENCHANTED_FOREST_IMPORT.md).

## Explicit source material profiles for forest conversion

2026-10-20: adapt the identified opaque Unity Standard subset through ordinary
Blender/glTF PBR, preserving source albedo/emission, scalar surface parameters,
backface culling and repeat/bilinear sampling. Reject unsupported active features;
do not treat a custom shader as Standard merely because it exposes similar keys.
Bind independent geometry/pixel/material checks to each qualified asset. This is
PBR adaptation, not Unity shader/lighting equivalence or assembled-map adoption.
[Contract and executed evidence](SYNTY_ENCHANTED_FOREST_IMPORT.md).


## Preserve source hierarchy and classify only proven unused flat overrides

2026-10-20: prepare static forest instance plans in source coordinates with
explicit highest-detail LOD selection and independent matrix/binding checks.
Only absent target IDs in fully materialized flat prefabs with the matching
source GUID are classified as unused serialized overrides. Keep nested/model
inheritance and unknown live properties unresolved; never invent replacement
transforms or material bindings. Geometry-axis and rendered/collision adoption
gates remain separate. [Contract](UNITY_STATIC_PLACEMENTS.md).


## Source-specific static foliage profiles through standard PBR

2026-10-20: bind the basic vegetation adapter to reviewed shader bytes; consume
binary per-face vertex selectors, derive colour-floor PNGs with exact source
alpha, and use ordinary glTF MASK/double-sided/surface settings. Record mesh-normal
and colour-workflow approximations explicitly. Keep preview LOD selection separate
from the default highest-detail scene plan. Full vegetation and normal-map effects
require their own qualification, not automatic material substitution.
[Contract and evidence](FOREST_FOLIAGE.md).


## Compile model variants only from checked aliases and complete authored poses

2026-10-20: the static single-mesh model-variant profile requires explicit source
materials and every final root TRS component in the outer instance. Bind inherited
IDs to actual source references, preserve collider records as unqualified data,
and reject missing pose components instead of inventing imported defaults.
Verify category completeness and source matrices independently; geometry/importer
axes and scene adoption remain separate. [Contract](UNITY_MODEL_VARIANTS.md).


## Calibrated forest mesh poses through standard glTF TRS

2026-10-20: use source-specific mesh-local X reflection and H M H placement
conversion, backed by independent ufbx/Unity LOD-bound evidence. Preserve the two
explicit source-bound mismatches. Express affine shear as two ordinary TRS nodes
and cache a winding-correct mirrored mesh variant for negative parity, keeping
normal Bevy rendering and shared materials/images. Five actual source placements
and an original asymmetric fixture qualify the bounded slice; full terrain,
materials, collision and map adoption remain separate. [Evidence](UNITY_GEOMETRY.md).


## Preserve full TerrainData samples and qualify rendering differences explicitly

2026-10-20: decode the forest's binary Unity 2020 terrain offline using pinned
UnityPy, checked source GUID/fileIDs and all serialized patch extrema. Retain
all height/control samples; represent ground with standard glTF. The initial
five-layer colour bake is an explicit finite-resolution approximation with
geometric normals, not terrain shader or normal-map parity. Preserve authored
normal bindings and keep map adoption open. Partition full-sample collision to
fit existing budgets rather than silently decimating. [Evidence](UNITY_TERRAIN.md).


## Full-sample forest ground through existing bounded packages

2026-10-20: publish four cell-aligned ground packages without clipping/decimation,
using existing schema-3 collision and shared-image contracts. Preserve exact
source attributes and finite world bounds, with ordinary Renet/Bevy/Avian and
one image handle across roots. Add an explicit chunk-world launcher option while
retaining the default town. Ground-only qualification does not adopt forest
objects/materials or prove Unity TerrainCollider parity. Each package fills its
current triangle allowance; review full-scene collision representation/budgets
before adding object meshes. [Evidence](FOREST_TERRAIN_COLLISION.md).


## Explicit static full-vegetation PBR profiles and canonical prototypes

2026-10-20: source-identical noise colours may collapse to constant PBR colour
floors; other spatial colour/effect profiles stay unresolved. Apply explicit
normal-map decoding/import policy and source alpha through standard glTF while
recording unapplied wind/highlights, filtering and Unity parity limits. Keep mesh
prototypes separate from source placement, preserving original instance matrices.
Match exact f32 cyclic positions and bounded UV export rounding in the independent
oracle after the tree exposed a quantized-key boundary issue.
[Evidence and scope](FULL_FOREST_FOLIAGE.md).


## Bind foliage batches to source overrides and qualify the assembled subset

2026-10-20: group exact source mesh/material references after independent whole-
plan validation; require raw source instance overrides and highest-detail LOD.
Retain unresolved categories explicitly. Deduplicate standard shared images and
independently check every final source pose/material/attribute. Repair only uniquely
defined zero derived tangents and bounded quaternion rounding; source geometry
remains unchanged. Standard Bevy MikkTSpace handles absent streams with documented
glTF portability warnings. This qualifies a visual subset, not full scene collision
or multiplayer adoption. [Contract and evidence](FOREST_FOLIAGE_BATCH.md).


## Preserve mesh-local geometry separately from imported FBX node placement

2026-10-20: extend the existing opaque Standard profile through exact scene
bindings. Bake file-axis/units with Blender, then clear object-node placement
already supplied by source scene matrices. Reject unqualified geometric transforms.
Check canonical positions independently with ufbx. Compare normalized nonzero
source normal directions against unit glTF normals, recording actual nonunit
source samples; preserve geometry/UVs. Reuse terrain-only assembly bases when
batched props already include ruins. Remaining variants/shaders and authored object
collision still precede adoption. [Evidence](FOREST_STATIC_BATCH.md).


## Qualify explicit single-model variants without inventing imported root poses

2026-10-20: opt in to the existing reviewed variant compiler only for the canonical
Standard path. Require inherited IDs, explicit nested materials and complete outer
source TRS; independently derive raw renderer/mesh/material references and verify
all source-category poses. Reuse prior content only after source/final-scene/hash
checks, in a new batch with recorded input hashes and unchanged prior bytes.
General nested prefabs remain outside this profile. Full forest collision,
materials and adoption remain open. [Evidence](FOREST_VARIANT_GEOMETRY.md).


## Preserve world-projected material behavior through the standard PBR extension

2026-10-20: the pinned triplanar source requires world-space projection that ordinary
mesh UVs cannot preserve across instances. Use Bevy ExtendedMaterial rather than a
new renderer or per-instance resource-heavy baking. Preserve signed projections,
epsilon blend and explicit texture/normal sampling policy. Qualify native colour/
normal diagnostics against independent camera rays/source pixels. Keep this bounded
origin-zero, one-profile inspector path separate from normal chunk/rebase adoption.
Treat large uncalibrated mesh/root-pivot offsets as an importer-reference gate;
do not invent centering corrections. [Evidence and limits](FOREST_TRIPLANAR.md).

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
All work stays local. [Contract and evidence](STATIC_SHAPES.md).

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
[Packages](FOREST_SHAPE_PACKAGES.md), [standard materials](STATIC_MATERIALS.md).

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
[Evidence and scope](TRIPLANAR_PHASE.md).

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
open. All work stays local. [Contract and evidence](RUNTIME_MATERIALS.md).

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
default. All work stays local. [Evidence and limits](FOREST_RUNTIME.md).

## Interactive performance gate — 2026-10-22

P1-12 / P2-05 / P2-08 / P2-10. Require representative optimized, native frame-time
measurements before scene adoption. Correctness runs with validation, functional
screenshots and asset counts do not establish acceptable performance. Preserve
legacy camera regressions and use the existing accelerated CAMERA query geometry
for normal chunk worlds. Keep explicit debug/visual-refinement modes for diagnosis.
Current measured scope and pending checks are in [the performance record](FOREST_PERFORMANCE.md).
