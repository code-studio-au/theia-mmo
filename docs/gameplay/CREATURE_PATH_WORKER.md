Update: The first real creature consumer is now qualified. [Latest playable proof and command](CREATURE_PURSUIT_NATIVE.md).
The dated checkpoint below remains preserved.

# Creature path worker — P3-07 / P4-03 prerequisite

Status: implemented adapter and worker contracts qualified, 2026-10-31. No creature consumes dynamic paths yet. The preserved playable
checkpoint remains `build/grip-qualified-v1-20261031`.

The requested perception/chase/return behavior needs paths to changing
destinations. Existing patrols consume routes prepared offline. This slice shares
the previously reviewed Landmass query adapter between the offline baker and a
dedicated server-side worker; it does not add a navigation algorithm, crowd
simulation, actor collision or client movement authority.

## Implemented bounds and publication

`navigation::query::Query` retains the checked 1–9 tile package, per-tile limits
of 8,192 vertices / 4,096 polygons, detail-mesh limits, matching source/agent
settings and bounded local frame. Coordinates remain checked global f64 with
centralized local conversion, Y-up metres at the repository boundary. The same
strict height sampling avoids snapping an unreachable floor onto a roof. Route
results contain 2–256 points; missing/disconnected destinations are errors.

`navigation::worker::Worker::load` reads and verifies the prepared package outside
the simulation. One worker owns one static query. A worker permits exactly one
outstanding request, including queued, computing and completed-but-unpolled work;
both channel capacities are one. Submit and poll are nonblocking. The owner uses
exclusive access to the receiver; it never waits on a worker-held mutex. Dropping
the owner closes the channels without joining a path search in a fixed tick.

Requests carry the creature's full actor key, optional target life key, nonzero
destination/engagement revision, checked endpoints and simulation-time validity
window of 1–120 ticks. Target and creature epochs must agree. Replies retain the
exact request and navigation package identity. `Reply::current` exposes points
only for matching navigation, actor, target life, revision and inclusive time
window. Return-to-patrol requests can have no target. A failed query cannot expose
a direct-line fallback. Collision/grounding remains authoritative when a future
consumer follows the route.

Landmass 0.9.3 is unchanged and remains engine-independent. Its dependency moves
from the offline baker to the shared network crate; no package version or engine
pin changes. The baker reexports the shared Query to preserve its existing API.
The query has no public node-expansion limit or hard CPU timeout. Search stays
outside `World::step`; the package bounds work size, and late results expire.
Expiry does not cancel a running search. This is one bounded local-package
prerequisite, not continent-scale navigation or a qualified latency guarantee.

## Acceptance and evidence

The targeted checks pass: two worker transport/publication tests and three actual
baker/backend tests. The obstacle test now verifies that a real worker returns
the same detour as the direct offline query. Other backend checks retain 128 m
tile seams at 15 km, missing tiles, forbidden slope and low-headroom rejection.
Worker checks cover saturation, slot reuse, invalid time/revision/epoch requests,
unavailable destinations, return requests, stale navigation/actor/life/revision
and expired/future observations. Initial log:
`build/navigation-worker-test-v1.log`. Final unchanged-source workspace qualification passes **539 tests / 40 explicit
fixture skips in 112 groups**, denied-warning workspace/all-target Clippy and
formatting. The normal server dependency tree contains Bevy 0.20.0 and Landmass
0.9.3, with no Bevy renderer/window/PBR or Rerecast dependency. Cargo.lock package
versions/sources/checksums are unchanged. Exact source and check logs are preserved
in `build/navigation-worker-qualified-v1-20261031`; final logs are
`build/navigation-worker-*-v2.log`. No native gameplay, FPS, population or Windows
qualification is claimed for this prerequisite. The prior 41 Python checks are
retained for unchanged tools, not rerun as part of this Rust-only change.

## Next consumer slice

Add opt-in perception and pursuit to the existing single development creature.
Capture its current supported patrol position and route progress on engagement;
bind each chase/return job to current keys and destination revision. Poll outside
the simulation, then consume only still-valid immutable results in fixed ticks.
Use existing kinematic collision/grounding, hold on missing paths, and abandon
blocked/stale routes. Bound perception scans, replan frequency and lost-target
time; never search synchronously in a tick or fall back to straight-line motion.

Recommended initial development settings are a 12 m perception radius with static
LOS, a 35 m leash from the captured patrol anchor, and 5 m/s chase/return speed.
These are proposed test settings, not implemented behavior or final balance.
Preserve the exact target life, terminate attacks on evade/death/disconnect,
return to the captured anchor and resume its saved route progress. Qualify
20/30/60 Hz transitions, blocked/unavailable routes, repeated aggro, stale jobs,
range boundaries and actual two-client authority/presentation before promotion.
See [the user-requested feature register](COMBAT_WORLD_FOLLOWUPS.md).
