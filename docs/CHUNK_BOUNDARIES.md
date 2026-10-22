# Movement at chunk loading boundaries

2026-10-20, P1-09/P2-06/P2-07, execution-order row 5. This follows the
[acknowledged readiness contract](CHUNK_READINESS.md) and precedes paired
rendered replacement.

Post-step input gating was insufficient: a stationary capsule could slide down
a steep surface into client-unready collision. The original reproduction crosses
x=127.646 at 20 Hz (`build/chunk-gravity-boundary-reproduction.log`). Loading
boundaries now participate in the shared capsule solver's sweeps, including
horizontal projections of vertical collision motion and step-up alternatives.

Each coverage set contains at most four indexed 128 m cells. Checked global
positions determine full capsule-footprint ownership, including skin. At most
sixteen outer edges become infinite-height movement constraints in the bounded
local frame. Shared edges stay open; missing corners and disconnected gaps cannot
be shortcut by a sweep. These are movement permissions, not physical scene bodies,
navigation tiles or gameplay hit shapes. No filesystem access or per-actor ECS
world construction occurs during a step.

Authority and per-client coverage are separate constraints; the nearer hit wins.
Empty or withdrawn client coverage holds XZ without suppressing free vertical
sweeps. On a steep surface the terrain and boundary may together prevent further
descent; gravity state still advances. After loading/readiness releases the edge,
ordinary gravity sliding resumes. Actor state commits once per fixed step, and
boundary contact is observable through existing blocked/unready counters.
Unknown-region contact does not require a respawn. The normal monolithic map
controller uses the same solver without chunk constraints.

The concave-corner test also exposed a support cast on a large convex floor
snapping a valid capsule about 17 mm downward into collision. Support candidates
retain the ordinary capsule query with full-body and sole penetration validation
before becoming a ground snap. An invalid candidate falls back to Avian's
standard lower-sphere query and the existing smaller query lifts. Keeping the
ordinary capsule contact also preserves exact persistent wall contacts. This preserves the existing
steep-surface rule and does not relax penetration tolerances.

Acceptance includes 20/30/60 Hz steep sliding, empty withdrawal, missing server
collision and release at origin and signed billion-metre coordinates; continuous
sweeps over disconnected gaps; actual Avian movement around a missing corner;
large-convex-floor support; unchanged jump/diagonal speed and real UDP readiness
regressions. Executed full/native evidence belongs in [validation](VALIDATION.md)
and [current status](STATUS.md); targeted cases do not establish completed
rendered replacement or streaming.

Next: use the same footprint coverage for retained local poses, remote
interpolation endpoints and trusted recovery spawns before retiring a candidate.
Pair that presentation check with exact readiness acknowledgement, then publish
matching standard Bevy roots and actual Avian bodies. Normal tile ownership,
contributor geometry and partitioning remain separate adoption gates.

## Sloped-wall contact continuation

The paired-handover qualification exposed intermittent sub-millimetre drift in
the unchanged real-network persistent-contact regression. A deterministic fixture
now varies approach positions at 20/30/60 Hz and requires exact XZ retention for
two seconds after contact, grounding, smallest intentional tangential input and
retreat. Near-head-on wall sweeps use a matching Avian feature manifold for
bounded signed-distance/normal refinement; glancing contacts retain conservative
CCD behavior. This supplements capsule-first support validation/fallback rather
than replacing it. See [current executed evidence](VALIDATION.md).
