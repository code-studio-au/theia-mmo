# Shared kinematic capsule controller

## Full-source endpoint safety continuation — 2026-10-21

P1-09 / P2-07. The complete forest survey exposed 31 unstable grounded endpoints
and ten penetrating solver endpoints on sampled routes. The controller now
retains the already admitted position when a step/slide endpoint penetrates, and
counts that conservative rollback in `Motion::solver_rollbacks()`. Invalid input,
invalid initial placement and leaving the bounded coordinate frame still fail;
authority's recovery policy remains in place for those errors.

A returned grounded pose must retain support under the same unchanged scene.
If a snap's resulting pose has no stable support, motion continues airborne.
The existing 0.1 mm penetration and 2 cm support-publication checks are unchanged.
The additional check only confirms grounded motion: it does not ground a rising
airborne actor merely because a surface lies nearby. A targeted actual-Avian
near-surface test first rejected that regression, then passed after correction.
An original uphill sprint/jump test passes at 20/30/60 Hz. The original movement
fixtures now validate every returned pose as well as their existing assertions.

The same 225-point full-scene survey now has zero rejected endpoints/support
checks and 34 clear two-direction starts (previously 27). All 41 earlier rejected
route seeds also pass walking/jump replays at 20/30/60 Hz: 246 cases, with every
returned pose checked. Across those cases, 44 encounter conservative rollbacks
(1,416 rejected steps total). Repeated stalls are deliberately visible; this is
safe endpoint handling, not proof that every obstacle can be traversed. It does
not provide navigation, swimming or complete-map movement coverage.

Evidence: `build/forest-land-route-survey-v1.json` (original failures),
`build/forest-land-route-survey-v2.json` (updated survey),
`build/forest-land-rejected-replay-v2.json` (final fixed-rate/jump replay),
`build/forest-movement-near-surface-v1.log` (retained failing diagnostic), and
`build/forest-movement-network-tests-v2.log` (complete default network suite).
The newer [forest runtime record](FOREST_RUNTIME.md) tracks connected GPU checks
separately. Existing completed terrain experiments remain preserved below.

P1-09/P2-07 continuation, 2026-10-19. `theia-network::kinematic` now provides
shared fixed-step motion against the pinned Avian static query scene. The graybox
authority and offline sandbox use it; connected clients present authoritative poses.
Imported profiles now use the same controller through normal authority, with
validated cached spawns and an explicit finite-world/recovery policy described
below. General collision mesh authoring, navigation, prediction, swimming and
moving-platform support remain open.

## State and geometry

The controller consumes checked local capsule-bottom coordinates, bounded movement
intent, explicit motion state and a qualified `TickRate` (20/30/60 Hz). Simulation
owns the timestep, walking/running speed, gravity and jump. Authority converts its
canonical f64 global centre pose into the query frame, advances local motion, then
reconstructs checked global truth. No client coordinates, velocity or dt are trusted.

The capsule is 0.35 m radius / 1.8 m total height. Initial tuning is 5/8 m/s walk/run,
20 m/s² gravity, 7 m/s jump launch, 0.35 m maximum step and 40° maximum grounded
slope. Speed is normalized for diagonal input. Ground snap is bounded to 0.10 m.
Gravity uses ballistic displacement over each fixed step; terminal downward speed
is bounded at 50 m/s. Geometry queries use at most five slide iterations per motion.
These are initial controller values, not final class/race/feel balance.

Movement sweeps the full capsule against both support and solid world geometry.
It resolves obstacle normals, projects remaining motion for sliding, refuses
uphill projection on unwalkable slopes and permits gravity-driven downhill sliding.
A step requires an unobstructed capsule lift, forward travel, walkable landing and
bounded height. Low ceilings and tall obstacles reject it. Support is selected
near the current body, not by a ray from the highest point in the whole map.
Falling and ceiling impacts update vertical motion; stacked lower/upper floors
remain distinct.

Shape sweeps use a 5 mm target separation and 3 mm support padding. On flat ground
the capsule bottom sits about 8 mm above its support; on a slope its curved base
also contributes geometric clearance. Renderer centre is bottom + 0.9 m. Binary
intersection queries proved unstable at tiny gaps over broad boxes, so pose
rejection uses signed contact depth on BVH candidates, with 0.1 mm penetration
tolerance. Exact-touch CCD normals are refined by a bounded query against the
same obstacle from an offset pose; the character is never moved by that query.
Separating/tangential hits are filtered before selecting the closest blocker.
Residual slide vectors and contact travel below 10 micrometres are discarded to prevent persistent
wall input from accumulating solver-normal noise; minimum valid 60 Hz input
movement remains larger than this cutoff.
Unsupported contact pairs fail closed. Tests independently check
clearance and penetration cases; raw intersection booleans are not treated as
precise signed depth. Downward walkable hits also use signed contact to refine
conservative mesh-CCD distance, with at most 2 cm correction inside the original
sweep interval; unsupported or unwalkable contacts are not extended.

Safe spawn is separate from movement: bounded nearby lifts/support checks resolve
low road surfaces with capsule headroom. It cannot accept an arbitrary client
position or depenetrate a tall blocking wall. Invalid intent, penetrating starts
or out-of-frame results fail without committing motion state. General recovery/
unstuck and invalid-world activation policy remain later work.

Characters are query shapes only; none is inserted into world collision. Two
co-located actors can advance independently and identically through the same scene.
World collision remains separate from gameplay targeting/hit detection.

## Runtime and input contract

Graybox authority stores per-actor grounding/vertical speed/jump-held state and
advances it once per configured server step. Input expiry still applies to the
intent, while already airborne physics continues under gravity. The offline
sandbox uses the same controller and its own temporary, untrusted local state.
Graybox bounds remain enforced by the authority. R resets offline pose/state.

Wire/Quinn ALPN becomes **7** (`theia-mmo-walk-7`); secure Renet's protocol ID also
incorporates version 7. Intent now includes a boolean `jump`, defaulting false
when omitted; unknown fields and invalid types/axes/version still fail. Rebuild
both executables together. Space sends the held jump level in graybox play.
A new press while grounded starts one jump; holding through landing cannot repeat
it until a release is observed. This is bounded superseding movement input, not
a reliable action-event queue: a very short press/release can be coalesced or lost.
Input expiry supplies a false level and therefore acts as an implicit release;
a subsequently resumed held value can start another jump once grounded. Future
action-command reliability is separate work.

Normal imported-map authority now uses the shared controller and jump input.
The old grounded helper functions remain historical fixture regressions; ongoing
normal motion does not call them or select the globally highest floor each step.
Existing schema-1/2 profile geometry stays unchanged: conservative boxes and
upward terrain/authored-floor triangles. This does not turn those proxies into
complete source-mesh interiors or support arbitrary vertical mesh authoring.

Client Avian/render-origin probes now compare live ray results to the independent
content-frame query cache, including airborne poses. They no longer assume every
root must be exactly 0.9 m above ground. Body clearance/grounding is qualified by
controller tests; the probe measures frame/query consistency.

## Acceptance

Real Avian fixture tests cover fixed-rate speed/jump/fall, held/released jumps,
thin-wall slide/retreat, valid/too-high steps and blocked headroom, stacked floors,
low ceiling jumps, walkable versus steep slopes, downhill sliding, combined
floor/wall triangle meshes, ledge falls, nonblocking actors, safe low-road spawns,
wall-adjacent grounding and transactional invalid-state rejection.

Real secure Renet tests exercise jump replication, landing without held retrigger
and a fresh release/press at all three rates. The native multiplayer qualifier has
an optional `--jump` case with measured actual local/remote rendered height peaks,
plus existing snapshot/render/physics/rebase checks. Exact results/devices and
remaining migration limits are recorded in [validation](VALIDATION.md).

## Imported-world envelope and recovery

Loading resolves the four historical development spawn XZ locations (0/2/4/6 m,
Z=0) to trusted support once and validates the full capsule. [Schema-3 explicit
spawn metadata](GENERAL_COLLISION.md) now passes native/default
qualification for town revision 2. Each actor stores
its own validated capsule-bottom spawn. Ongoing grounding remains near the current
body and does not use the highest-surface helper.

Normal authority and client presentation call `Terrain::into_world` to add four
static solid envelope walls at the finite map bounds. They extend from below the
fall plane to 16 m above the highest known ground/solid. They are movement-only,
not camera occluders or visual walls. Repeated adoption is idempotent. Individual
residency chunks do not adopt this envelope, so ownership seams remain open.
Preserved tall water-exclusion columns still block characters; there is no swimming.

An unsupported fall below minimum terrain vertex Y minus 18 m, or a rejected
controller result, returns that actor to its trusted spawn and resets vertical
state while preserving the current held jump level. Authority retains identity,
input sequence and session; other actors continue. `movement_recovery` diagnostics
and a monotonic world counter make recovery observable. Native route qualifiers
reject unexpected recoveries rather than allowing reset-assisted success. This is
a temporary development recovery policy, not gameplay death/respawn, last-safe
checkpoints or arbitrary depenetration. Camera/animation continue to follow server
poses; a jump animation and teleport smoothing are not implemented.

## Next integration gate

Qualify general versioned collision meshes and explicit spawn metadata, including
combined floor/wall/ceiling meshes and chunks without the four legacy spawn
locations. Validate full 3D triangle degeneracy/orientation, nearby support and
headroom independently of height-only terrain. Then qualify chunk replacement around moving, airborne and supported actors before adopting
residency in normal authority. Preserve full-map/authored routes as regressions;
keep source-mesh fidelity and navigation separate from conservative proxies.

A combined floor/ceiling mesh now has a qualified CPU support-probe fallback at
20/10/5/0 mm query-only lift. See the [paused schema-3 checkpoint](GENERAL_COLLISION.md);
native mesh/map/chunk/default checks now pass; moving-actor chunk activation remains open.

## Loading-boundary continuation — 2026-10-20

Chunk sessions now pass bounded [coverage sweep constraints](CHUNK_BOUNDARIES.md)
to the same controller. These also constrain sideways projections of gravity;
ordinary non-chunk callers retain the unconstrained entry point. Ground support
keeps the capsule query and validates both full-body and sole clearance, using a
standard Avian lower-sphere fallback only for invalid support results. Existing
exact persistent-contact assertions remain unchanged. This precedes actual
paired rendered retirement/publication; see current validation for executed scope.

## Prediction prerequisite — 2026-10-22

The controller now exposes validated gameplay-only `MotionState` capture/restore:
vertical velocity, grounded and held-jump state. Replay checkpoints are exact on
identical query frames at 20/30/60 Hz, including held jump after landing. Positions,
identity, command scheduling and solver diagnostic counters remain separate. This
is not client prediction or a new network correction message. See the
[prediction contract](PREDICTION_CONTRACT.md) and `build/prediction-motion-state-v1.log`.

Procedural terrain now reuses the existing swept coverage boundary. Empty or
partial presentation readiness holds XZ, including gravity slides, while vertical
jumping/falling/landing continues; imported contributor checks are unchanged.
