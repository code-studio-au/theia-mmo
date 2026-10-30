# Creature AI — P3-07 initial scope and navigation prerequisite

Status: the initial nonblocking patrol slice is qualified, 2026-10-30. Headless
authority, encrypted moving-target combat, two native clients, ordinary/mixed
account launchers and the final full-suite checks pass. Exact qualification and
limits are recorded below. This is not all of P3-07 or streamed runtime navigation.
Prior offline-navigation and status-effect checkpoints remain preserved.
The dated implementation/stop notes below retain their historical context.

The next playable slice is one authoritative, nonblocking patrolling creature on
the saved graybox terrain. It must follow paths around static world obstacles,
pause while dead, respawn at its configured home with a fresh runtime identity,
and remain targetable/damageable through existing mobile combat/status effects.
Idle, patrol, dead and blocked are explicit server states; the first route is
prepared offline rather than requested by a runtime path worker. Behaviour,
route endpoints, speed and wait ticks are checked development content; this is not
final story/class data. Aggro, attacks against players, threat, loot and bosses are
follow-on work, not implied by a patrol.

Actual prerequisites are terrain collision/source identity, checked global/local
coordinates, authoritative entity lifecycle, bounded tick scheduling, asynchronous
work publication, target/combat replication and a navigation route contract.
P4-03's minimum navigation prerequisite therefore overlaps P3-07 rather than
waiting for list order. Do not replace it with bespoke house fixtures or routes
that walk through geometry. Collision, navigation and hit detection remain
separate. Players/NPCs gain no physical collision with each other.

Use an existing navigation library after reviewing and testing it. Initial review
is the engine-independent `rerecast` generator and a bounded path-query adapter;
the Bevy integrations must not introduce a second/incompatible Bevy version.
Candidate versions are not adopted workspace pins until the isolated probe passes.
Sources: [Rerecast](https://github.com/janhohenheim/rerecast), its
[published API](https://docs.rs/rerecast/0.5.0/rerecast/), and
[Landmass query API](https://docs.rs/landmass/0.9.3/landmass/).
The first fetched dependency graph for the raw candidates contains no Bevy crate;
optional dependencies fetched into the registry are not runtime dependencies.
Landmass's query loop has no public expansion-limit parameter, so it cannot be
called unrestricted in `World::step`. Its suitability remains under review.

Navigation is precomputed from the same static collision triangles as the server,
not from cosmetic render LOD. Use 128 m terrain tile IDs, checked f64 global origins
and bounded Y-up f32 bake/query frames through the shared origin conversion.
Bind navigation to terrain revision/hash and checked agent radius, height, slope,
step and voxel parameters. Include a neighboring geometry border before erosion
and clip tile output consistently so seams connect. Bake outside the simulation;
validate finite geometry, file/count bounds, indices and content hashes before
publication. Imported buildings/caves require their actual collision geometry;
a terrain-only bake is explicitly terrain-only.

Start with a bounded local patch, not an entire-continent runtime path service.
Keep worker count, pending jobs, resident tiles/polygons, route length, query
input/output and stale-result lifetime bounded. Define these concrete budgets
before runtime integration. Missing/unloaded/mismatched navigation means wait or
reject, never direct-line fallback. Path work performs no SQL/transport I/O inside
the fixed step; results carry world/entity/navigation generations and publish only
if still current. Runtime actor movement uses existing static collision/grounding
and stops/replans on blocked or stale paths. No client route is authoritative.

Acceptance starts with an isolated bake/query proof: obstacle detour, forbidden
slope/headroom, disconnected destinations, two-tile seams, coordinate translation
at distant world origins, malformed/bounded data and missing tiles. Then exact
20/30/60 Hz patrol/wait/death/respawn and stale worker-publication tests; actual
encrypted two-client moving-target replication; native Linux target/combat/status
integration; ordinary resource/backpack regressions; headless dependency, format,
denied-warning Clippy and appropriate tests. Record source/device and distinguish
Windows cross-builds from native execution. A small successful patrol does not
prove continent-wide navigation, population capacity, dynamic doors/elevators,
swimming/flying, hierarchical long-distance routing or completed P3-07/P4-03.


Backend feasibility checkpoint: `build/navigation-backend-review-v1-20261029`
records exact registry versions, lockfile/source hashes, normal dependency graph
and passing probe/denied-warning Clippy logs. Raw Rerecast 0.5.0 plus Landmass 0.9.3
bakes and routes around a solid obstacle, connects two independently generated
20 m fixture tiles, rejects a missing tile and a forbidden slope, and excludes a
low-headroom floor while retaining its supported roof. Existing checked coordinate
frames round-trip routes at 0, 15 km and near the global bound. No Bevy crate is in
the probe's normal dependency graph. These are fixtures, not a 128 m saved-terrain
bake, runtime streaming, MMO population capacity or Windows execution.

Recommended first runtime scope: consume offline-computed, checked patrol routes
from the baked navmesh. Keep the general path-query loop outside the fixed step.
This meets the first static patrol requirement without adding an unrestricted
runtime A-star call, a crowd simulation or a distributed navigation service.
Dynamic chasing/replanning will require a separately bounded worker query policy;
the current library's public query API does not expose a node-expansion limit.
The offline bake and route artifacts must still carry checked source/settings
identity, tile IDs, bounded local geometry and bounded route points. Runtime
collision remains authoritative even with a precomputed route.


Implemented offline prerequisite (in progress, 2026-10-29): `theia-nav-bake` is a
separate headless workspace tool. It pins raw Rerecast `=0.5.0` and Landmass
`=0.9.3`, with no Bevy integration plugin; all previously locked package versions,
including Bevy/Avian, are preserved. Their dependencies are absent from the normal
server graph. The network crate contains only bounded checked file contracts and
loading methods, not a path-query call in the simulation.

`bake SOURCE OUTPUT [CENTRE_X CENTRE_Z]` prepares at most nine 128 m terrain tiles
from decoded saved full-resolution collision triangles and neighboring geometry.
It does not reevaluate the procedural height recipe or read cosmetic render LOD.
Output is terrain-only; the development patrol endpoints are test data. Existing
output directories reject. Tile files are written before the final manifest is
published. `verify SOURCE OUTPUT` checks source/profile/hash/shape and reports a
sampled height comparison, not a proof of continuous coverage.

Clearance comes from the shared controller's radius/height/step/ground-normal
constants. Tile vertices stay in checked tile-local Y-up metres, clockwise convex
voxel-grid polygons have 3..6 distinct indices, and routes use checked f64 global
feet positions. Initial limits: nine files, 8,192 coarse vertices/4,096 polygons
per tile, 32,768 detail vertices/65,536 detail triangles, 2 MiB tile/256 KiB manifest,
8 routes/256 points each/1,024 points total and 1,024 m per local patrol route.
Unknown fields, missing tiles, foreign terrain/profile/coordinates, malformed
indices/winding/convexity/detail ownership and escaping tile symlinks reject.
Opening/loading is outside `World::step`; the new headless patrol consumer is
implemented below, with native qualification still pending.

The first coarse-only saved-terrain query rejected endpoints (`OutOfRange`).
Recast detail-height triangles are therefore retained and validated, permitting
strict 0.2 m vertical sampling and rejecting a low-ceiling endpoint instead of
snapping it onto a roof. The route exporter projects waypoint feet back onto the
saved terrain's actual triangle planes; runtime collision will remain the final
movement authority. The first actual nine-tile bake/reload passes, with zero
misses among 576 interior height samples and 0.1273 m maximum sampled difference.
That is navigation approximation, not movement/grounding error. Initial debug bake
log: `build/navigation-saved-bake-v2.log`; current file validation/height probe:
`build/navigation-saved-height-check-v1.log`. Exact release source qualification now passes; see below. Three backend contracts pass for obstacle detours, actual 128 m seams at
15 km, missing tiles, slope disconnection and headroom/layer rejection. Five file
contracts and fifteen movement regression tests pass; workspace/all-target denied-warning Clippy passes.


Qualified offline checkpoint: `build/navigation-offline-qualified-v1-20261029`
freezes exact source, tool executable, dependency evidence, both package hashes and
proofs. The final optimized tool bakes the central nine-tile patch in 5.492 seconds
and the far patch (centre 117,117, near 15 km) in 5.096 seconds on this Linux
7800X3D. These are bounded offline bake observations, not server-tick/streaming or
population-capacity results. The central regeneration has identical hashes for
all ten files compared with the earlier debug output. Both packages independently
reload, validate and query 576 interior height samples with zero misses; maximum
sampled height differences are 0.1273 m and 0.1200 m respectively. No continuous
coverage or entire-continent navigation claim follows from these samples.

Final focused checks: five file/detail contracts, three actual generator/query
contracts, and fifteen shared movement regression tests pass (23 total, no skips).
Formatting and workspace/all-target denied-warning Clippy pass. Every preexisting
locked package version is preserved; the new offline packages introduce no Bevy
integration/version change. The standalone server graph still excludes the
navigation backends and renderer/audio/UI/wgpu. The 336-test full workspace result
belongs to the preceding status checkpoint, not a newly claimed full-suite run on
this navigation source. No native GPU/Windows/navigation gameplay qualification
is claimed; no rendering behavior changed in this offline prerequisite.

Next: consume an already loaded checked route for one nonblocking authoritative
training target; keep motion inside the existing collision controller, retain its
world collision while players travel, synchronize combat positions, and return
its fresh respawn to home. Prove timing/lifecycle before adding native presentation
and testing moving-target combat over encrypted impaired UDP. General dynamic
chasing, runtime path workers and streamed navmesh residency remain open.


Headless patrol implementation (not yet native-qualified): a checked 4 KiB profile
binds stable creature/route IDs, 0.1–5 m/s walking speed and 100–60,000 ms waits to
the navigation package identity. Startup validates all files and prepares one
static collision patch. Route waypoints must be supported, fit that patch and
close explicitly; there is no implicit straight-line return leg. The fixed step
only follows the immutable route with the existing swept Avian capsule controller.
The first profile walks at 2 m/s and waits one second at each waypoint. Arrival
uses a 1 cm tolerance; it does not warp onto waypoints or cut corners by 10 cm.

One patrol is bounded initially. Its home collision patch stays in server demand
while players travel elsewhere, and is shared through the existing terrain cache.
Players/NPCs are never inserted as physical blockers. A blocked/stale route stops;
removal cancels future movement/respawn. Death pauses motion; fresh respawn uses a
new runtime ID at home, with its initial wait restored. Combat positions update
before cast/status resolution. The admitted profile identity stays fixed even
when the live creature is removed. Rate changes reject after patrol configuration.
No path search, filesystem, SQL, wall-clock timing or awaited transport work runs
in the fixed step.

Four actual World/Avian contracts pass at 20/30/60 Hz: exact wait/start speed,
arrivals/loops, movement/combat pose coherence, death/home respawn, blocking world
geometry, actor pass-through/forged NPC movement rejection, configuration/source
failure, removal and collision residency. Existing ten combat, five resource and
five status authority regressions also pass. Logs:
`build/creature-authority-tests-v3.log` and
`build/creature-combat-regression-tests-v1.log`.

An actual two-client encrypted route-consumer test passes delayed/lossy/duplicating
UDP (`build/creature-encrypted-tests-v2.log`): twenty matching authoritative target
pose ticks, moving hit locations, death, fresh home respawn, old-life rejection
and continued patrol after one client disconnects. Its in-memory authored route/file
fixture tests the consumer; it is not an offline-baker composition claim. The native
qualification will use the actual saved-terrain baked package.

`tools/play.py --creatures` now selects the opt-in profile/navigation and implies
resource combat. Add `--status-effects --spell-feedback` for Scorch/protected cues.
The client binds the same profile identity and uses existing remote-pawn
interpolation math for moving target visuals, checking geometry permission for
both the latest pose and the presented pose. New lives start fresh presentation
tracks. Target motion/lag metrics distinguish interpolated display from server
truth. Native render/movement/feedback/backpack checks remain pending.


## Safe stop requested by user — 2026-10-29

Development stopped after the successful actual native integration run; no new
feature work should proceed until the user resumes. Changes remain local and
uncommitted. The complete P3-07 checkpoint is not yet qualified: ordinary launcher,
mixed OIDC/backpack launcher and baseline checks on the final creature binaries
remain to be run, and the final source has not had a new full-workspace run after
presentation tuning.

The full-workspace run before that tuning passed 349 tests/15 explicit skips in
75 groups (`build/creature-workspace-tests-v3.log`). The final presentation source
passes all 61 client tests (`build/creature-presentation-tests-v3.log`), including
two added catch-up/fresh-life contracts; all 38 headless network library tests pass
(`build/creature-authority-regression-tests-v1.log`). Workspace/all-target denied-
warning Clippy passes (`build/creature-presentation-clippy-v4.log`). The five real
PostgreSQL contracts/restart qualification and five login-helper tests pass on
this development run. These counts are separate runs, not a claimed 351-test final
full-workspace result.

`build/creature-native-wifi-v3-20261029/verification.json` passes with two actual
native encrypted clients, impaired UDP, Vulkan validation and the actual baked
saved-terrain package. Both load 119 terrain tiles and retire 38 during a real
continuous run out/back. The server records a full patrol loop, three waypoint
arrivals, three fresh-home respawns and no blocked route. Each client sees 74
periodic hits and three deaths/respawns, mana rejection/recovery, and all 77
critical visual/audio cues under 28,160 cosmetic requests, with no critical
overflow. Each has thousands of actual visible moving-target samples. The captured
damage screenshot was inspected.

Measured presentation catch-up spikes of 11–13 m/s in the initial attempt led to
using the existing interpolator with a longer distance-based catch-up interval.
The final declared impaired-link run peaks near 4 m/s with 0.501/0.470 m maximum
observed pose lag. This is presentation lag, not an authoritative-position error
or a guarantee for arbitrary long outages. The fixture now allows a non-damaging
full patrol loop, retains a steady-input segment, and crosses a real streamed
boundary before combat. Earlier attempts v1/v2 failed fixture acceptance and are
preserved as unqualified evidence; their failures are not counted as passes.

Native device remains Linux Ryzen 7800X3D / RX 7900 XTX / RADV NAVI31 / Mesa 26.2.5
kisak Vulkan, 1280×720. Overall two-client validation/mixed-phase rates are
73.20/74.18 FPS, not an ordinary baseline or population-capacity result. No native
Windows or runtime pathfinding/streamed-navigation completion is claimed.

Final native binaries at this stop: client
`e131fc3f4708058436212b2cecd472c069a3e0f6e0a8f0363963bfdaad532125`, server
`fb7c91e26fc8931bff2c7edbc26f31db9ad644261d74b5fb65139322b8bf18a1`.
The status and offline-navigation qualified checkpoints remain preserved.


## Initial patrol slice qualified — 2026-10-30

The user resumed development. Final source passes **351 full-workspace/all-target
tests, zero failures and fifteen explicit skips**, across 75 groups
(`build/creature-workspace-tests-v4.log`). This includes the final presentation
catch-up/fresh-life tests. Formatting and denied-warning workspace/all-target
Clippy pass (`build/creature-resumed-fmt-v1.log`,
`build/creature-resumed-clippy-v1.log`); five login-helper tests pass. Five skipped
PostgreSQL contracts have separate actual database/restart qualification in
`build/creature-postgres-v1-20261029/verification.json`; ten private-asset skips
remain skips. The normal server graph excludes renderer/audio/UI/wgpu and the
navigation backends (`build/creature-server-dependencies-v2.txt`).

The passed native integration remains
`build/creature-native-wifi-v3-20261029/verification.json`, on the exact binaries
listed in the preceding stop record: actual saved terrain and baked navmesh,
two encrypted/impaired clients, full patrol loop, moving-target combat/statuses,
three fresh-home respawns, tile load/unload and protected visual/audio cues.
Earlier v1/v2 fixture failures remain unqualified evidence.

Ordinary creature/status/feedback launcher interruption and child-failure cleanup
now pass (`build/creature-launcher-v1-20261030/verification.json`). The actual
OIDC/PostgreSQL/backpack/creature/status/feedback launcher passes interruption,
child failure and pending-browser cancellation
(`build/creature-backpack-launcher-v1-20261030/verification.json`): real durable
backpack widget mutations, patrol movement, no blocked route, final pose commits,
closed database ownership, removed tickets and stopped owned services. This mixed
launcher loads Scorch but does not cast; the separate impaired native test above
exercises moving-target damage and statuses. Resource-only and zero-cost combat
launcher regressions also pass, each in both cleanup modes.

Same-binary ordinary baseline passes at **809.20 FPS uncapped**, one native
saved-terrain client, validation off, 1280×720
(`build/creature-legacy-baseline-v1-20261030/verification.json`): 834 tiles loaded,
753 retired, maximum residency 81. This is an empty-world traversal baseline,
not populated MMO capacity. Hardware remains Linux Ryzen 7800X3D / RX 7900 XTX /
RADV NAVI31 / Mesa 26.2.5 kisak Vulkan. No native Windows result is claimed.

Exact source/binaries/docs/evidence are frozen locally in
`build/creature-qualified-v1-20261030`. The navigation is terrain-only and prepared
offline; no general runtime chasing/replanning, streamed navigation, player damage,
threat, loot, bosses or continent-wide routing is implied. No commit or push was
performed. Concurrent story files remain untouched. Next is the separately scoped
[P3-08 equipment and appearance slice](EQUIPMENT_FOUNDATION.md).
