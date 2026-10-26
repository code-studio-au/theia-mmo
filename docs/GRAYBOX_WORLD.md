# Continental graybox development world

Saved terrain continuation, 2026-10-23: the loader now has a checked saved-height
source. Saved terrain is the qualified default; [saved terrain](SAVED_TERRAIN.md)
records current acceptance/evidence. The original runtime-generated source is
retained through `--graybox-world`. The representation below describes that
original source; the same residency/controller budgets apply to saved tiles.

User-directed replacement development target, 2026-10-22. Scope:
P1-03/09/10/12 and P2-06/07/08/09; this slice does not complete those entire tasks.
Prerequisites: checked coordinates, authoritative Renet sessions, the shared Avian
controller, standard Bevy meshes/animation and the locally converted POLYGON
Base Locomotion sample. Forest reproduction is frozen; its fixtures remain available.

## Run

Build optimized client/server, then launch:

```sh
cargo run --release --locked -p theia-network --example bake_terrain -- bake build/graybox-terrain-v1
cargo build --release --locked --workspace
python3 tools/play.py
# Server-configured accelerated development travel:
python3 tools/play.py --dev-speed 32
# Start near a different part of the continent:
python3 tools/play.py --dev-spawn 15000,15000
# Automated server-owned teleport circuit; ordinary input still works:
python3 tools/play.py --dev-tour
# Two actual connected characters:
python3 tools/play.py --clients 2
# Retained movement baseline without client prediction:
python3 tools/play.py --no-prediction
```

Ordinary play retains VSync and normal focus pacing. For an explicit uncapped
benchmark use `THEIA_PERFORMANCE_UNCAPPED=1 python3 tools/play.py`; a paced
60 FPS desktop window is not the uncapped renderer ceiling.

WASD/arrows move, Shift runs, Space jumps, RMB orbits, wheel zooms, Home resets
camera and Escape exits. Normal walking/running remains 5/8 metres per second;
`--dev-speed 8|32` multiplies travel on the authority. This is a local development
setting, not a client-selected gameplay speed. All launches retain the existing
loopback-only admission and eight-peer prototype limit.

The graybox launcher now selects the qualified client prediction/reconciliation
profile by default. Local input is simulated through the same controller and
corrected from completed server ticks; the server remains authoritative. Predicted
local poses and authoritative remote poses are interpolated for rendering. Exact
accepted/live terrain overlap constrains prediction. Teleport, readiness, missing
collision and history/clock resets have explicit behavior; holding Space through a
teleport does not manufacture a jump release. Imported map launches retain held
input and server-pose interpolation. [Protocol and evidence](PREDICTION_CONTRACT.md).

The default player directory is `build/synty-character-canonical-stage`, containing
`character.glb` from the existing licensed local conversion. No private asset is
added to Git. An alternate directory may be passed with `--players`. See
[character provenance](SYNTY_CHARACTER_PREVIEW.md) for extraction and conversion.
The standard four-weight skin is explicitly accepted here as development art:
measured Jump deformation differs from full source weights by up to 6.975 mm.
The old 5 mm production-fidelity gate remains failed; adoption does not rewrite
that evidence. Idle/Walk/Run/Jump are standard Bevy animations; world movement
comes from authority poses, not root motion.

Legacy maps require explicit selection:

```sh
python3 tools/play.py --world build/development-world-town-v3
python3 tools/play.py --chunk-world build/forest-land-runtime-dry-probe-v1
```

## Representation and lifecycle

The finite world is exactly `[0,16000) × [0,16000)` metres: 256 square kilometres,
125 × 125 logical 128-metre chunks. It is a graybox square, not an assertion about
WoW's actual map measurements or future coastline area. The fixed versioned
procedural recipe evaluates broad hills and ridges from global coordinates.
Shared boundary samples are identical. Changing the recipe requires a new world
identity. The intended finished map is persistent and authored: generate a base
offline, edit it, and save/version its chunks. Runtime residency will still load
nearby chunks. This test recipe does not select runtime generation as the final
content workflow. No 15,625-entry package index, whole-world mesh or texture set is loaded.
Existing bounded imported-package formats and limits are unchanged.

The client has one bounded geometry worker and one bounded collision worker;
each has one in-flight job and a one-result mailbox. Nearby visual chunks use a
32 × 32 grid (4-metre samples), distant chunks an 8 × 8 grid (16-metre samples).
A Chebyshev radius of four admits at most 81 roots, about 512 metres in each
cardinal direction with the outer chunk extending farther depending on position.
Standard Bevy frustum culling chooses draw visibility. Nonresident roots are
despawned and their actual Mesh assets removed. LOD replacement removes the old
mesh; downward visual skirts cover coarse/fine edge gaps and never enter collision.
This is simple distance LOD, not a production terrain renderer or occlusion system.

Full-resolution 3 × 3 collision patches use the same near-visual triangles and
standard Avian triangle meshes. Each patch has a checked f64 global origin and
bounded f32 coordinates, independent of the renderer's moving frame. Authority
patches are generated outside fixed simulation by a single bounded worker. Old
support stays resident while a replacement prepares; retired patches leave the
cache. Cache demand is bounded by the eight actors, one admission location and
one pending trusted teleport, rather than continent size. Actors remain absent
from the collision set.

The client also publishes these colliders into its live Avian world. Ground probes
compare that BVH with the independently framed query patch. Camera sweeps use the
patch's accelerated geometry, avoiding CPU render-mesh scans. Ground/collision
frames and visual roots are reprojected from global truth when the render origin
moves; imported fixed-content-frame assumptions are not stretched to 16 km.

Readiness reports identify full-detail visual chunks with corresponding client
collision. The server independently generates collision and checks coverage of the
capsule footprint and attempted motion. Unready boundaries stop intentional travel.
A teleport first prepares destination authority collision, then moves the actor and
withdraws old readiness; client meshes/physics prepare before movement resumes.
The optional tour visits four distant corners and returns to the centre. No network
input supplies coordinates or authorizes a teleport.

Procedural sessions send recipient-centred snapshots, including actors within
1024 metres on both X/Z axes. Admission checks world identity, epoch, actor identity
and finite terrain ownership; snapshot local coordinates retain their existing
bounded validation. This is a development interest filter, not region ownership
or MMO population capacity. Legacy session/frame checks remain in force.

## Acceptance and evidence

Required checks: exact world extent and matching chunk edges; actual Avian surface
queries against visual triangle planes; readiness/stale-report failures; ordinary
movement/jumping at 20/30/60 Hz; continent crossing and return; distant teleport
and revisit; bounded caches and actual render mesh assets; standard animated
character, culling and live physics on a native GPU; release frame-time evidence.

The original traversal-slice results follow below. Its later prediction
continuation is qualified separately in [the prediction contract](PREDICTION_CONTRACT.md). Combat,
persistence, authored terrain, navigation, water and population scaling are outside
this slice. Windows runtime evidence must be recorded separately from Linux.

### Historical interpolation-only Linux qualification — 2026-10-22

The slice passes on Ryzen 7 7800X3D / RX 7900 XTX (RADV NAVI31), Mesa 26.2.5,
Vulkan, native desktop, optimized builds, 1280 × 720. Exact binary/model hashes
are retained in the reports below. Engine/physics pins remain unchanged.

| Check | Executed result |
|---|---|
| Headless terrain/controller | Six terrain/controller tests pass: shared sample edges, independent triangle-plane/Avian checks, readiness/stale reports and airborne withdrawal, 20/30/60 Hz jump/travel, full continent crossing/return and four teleports. 251 patches generated, 250 retired, one retained; zero recoveries. The simulated test permits worker preparation time; native travel below supplies wall-clock evidence. |
| Two native clients | 65 seconds, encrypted Renet, GPU validation active, five server-owned distant teleports and centre revisit; both clients show actual Idle/Walk/Run/Jump joint changes. At most 81 terrain roots and a bounded asset inventory; retired roots/LOD meshes are removed. Source clips remain development approximations. |
| Complete native eastward crossing | 80 seconds, one secure client, developer speed 32, no validation overhead. X=10 → 15999.645 m, all 125 chunk columns visited, 125 collision patches; 2,345 visual mesh publications and 2,300 retirements leave 45 at the finite eastern edge. Interior maximum stays 81. Zero movement recoveries and zero skipped server intervals/missed deadlines. |
| Release frame-time baseline | Crossing reports after 8-second warm-up: 59,416 frames, weighted mean **1.213 ms / 824.14 FPS**; worst one-second p95 **1.376 ms**. This includes traversal and the stationary finish. It is a simple graybox, one hardware/view configuration, not populated-MMO capacity or a universal FPS guarantee. |
| Actual resources | Interior terrain CPU mesh buffers 2,490,024 bytes; whole client RSS sampled peak 311.8 MiB, server 61.0 MiB during the single-client crossing. Final CPU/GPU mesh assets 52 = 45 terrain + seven separately inventoried Bevy/player assets. Counts remain bounded while thousands of tiles are replaced. RSS is not total VRAM; deduplicated driver counters are retained separately. |
| Live physics and culling | 65,907 rendered-player ground probes during crossing, maximum frame/query disagreement 1.252 micrometres. Standard ViewVisibility reports fewer visible than resident roots. Validation runs have no Vulkan validation errors; the timing run deliberately disables validation. |
| Presentation jitter correction | Local and remote players interpolate global positions between authoritative samples using explicit snapshot intervals. Teleports snap; vertical jumps preserve facing. In the two-client run, only 24/3,554 and 22/2,871 moving render samples held position (0.68% / 0.77%). Crossing: 1,256/54,924 (2.29%). The camera follows the smoothed local pose. This adds up to a snapshot interval of presentation delay; prediction was not yet implemented at this historical checkpoint. Current default behavior is documented above. |
| Retained map | Actual town interior/exterior, animation and camera regression passes with two encrypted clients, 2 m render/wire rebases and active Vulkan validation. |

Fresh checks: the full workspace suite passes **201 tests**, including all 49 client
tests, six terrain/controller tests, the impaired-network procedural case and two
movement-state replay prerequisites. Workspace/all-target denied-warning Clippy,
formatting, diff checks and 221 Python asset tests pass. The actual default launcher
also passes two-client startup and owned child/ticket cleanup after interruption
and injected client failure (`build/graybox-launcher-v1-20261022/verification.json`). No native Windows run is claimed.
Cross-platform procedural numerical identity still needs qualification before a
mixed Linux/Windows deployment; the future saved-heightmap workflow avoids relying
on cross-platform transcendental-function equality.

Local evidence:

- `build/graybox-workspace-tests-v4.log`,
  `build/graybox-client-tests-v4.log`, `build/graybox-clippy-v6.log`.
- `build/graybox-native-v4-20261022/verification.json`: final two-client GPU run,
  per-client screenshots/world inventories and sampled process/driver resources.
- `build/graybox-continent-v2-20261022/verification.json`: full crossing,
  per-second frame/mesh/position diagnostics, process samples and screenshot.
- `build/graybox-town-regression-v2-20261022/verification.json`: retained map.
- Earlier failed attempts remain: initial spawn guessed support 2 m too high;
  the first traversal drove simulated time faster than its real worker could
  prepare; the first GPU verifier assumed too few non-terrain Bevy meshes;
  the first town verifier lacked required layer-activation logging. These are
  preserved failures, superseded by the named passing checks.

Reproduce native checks on a local display with prepared private assets:

```sh
python3 tools/qualify_graybox.py NEW_OUTPUT --clients 2
python3 tools/qualify_graybox.py NEW_OUTPUT --duration 80 --continent --no-tour --no-validation
```

Use the repository's existing native SDK `LD_LIBRARY_PATH` when required. The
qualifier uses the actual desktop display and captures only its game windows;
validation and performance runs are deliberately separate. Initial startup and
teleports may display loading while distant terrain prepares. Ordinary travel
waits at unready boundaries. This is generated graybox residency, not streamed
saved/authored map packages, hierarchical screen-error terrain, navigation or a
population soak test.

### Readiness/network continuation

The follow-up uses the existing shared swept coverage boundary in the procedural
controller. Readiness withdrawal holds horizontal movement, including gravity
slides into missing cells, while vertical jumps/falls/landing continue. A dedicated
airborne-withdrawal regression verifies the distinction. Initial readiness tests
compare held XZ and valid support, not bit-identical Y while gravity is active.

The actual UDP impairment test now also runs the procedural authority through
40 ms one-way delay plus 0–60 ms deterministic jitter, 10% scheduled packet loss,
duplication and reorder. It crosses a terrain seam, waits for exact withdrawal
acknowledgement, holds, regrants and resumes, then checks stopped convergence on
both clients. Zero movement recoveries and bounded queues are required. This is
headless transport/controller evidence, not a rendered latency test. Raw first
passing profile: `build/graybox-adverse-v3/procedural-wifi.json`; existing
baseline/wifi/poor/outage tests also pass. The follow-up passes all 201 workspace tests and fresh two-client validated GPU /
full-continent uncapped runs; the table above records these final binaries.

The next P2-04 work is governed by the [prediction contract](PREDICTION_CONTRACT.md).
