# Explicit finite chunk-world edges

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08, execution-order row 5. The prerequisite
is contributor-complete residency/readiness with the qualified 32-package ceiling.
This slice separates the finite playable rectangle from geometry envelopes and
transient loaded/ready chunk coverage. It does not adopt chunked town rendering
or establish production streaming capacity.

## Contract

`world.session.json` schema 2 requires `world_bounds` in checked global XZ metres:
`[[min_x,min_z],[max_x,max_z]]`. Its dimensions must admit the whole capsule
footprint (radius plus skin), all edges must fit the chosen bounded local query
frame, and trusted initial/recovery spawns must lie inside with full clearance.
Spawns still require real prepared collision, support and loaded contributors.
Reversed, nonfinite, excessively large, collapsed or out-of-frame bounds reject.

Schema 1 remains a compatibility fixture without explicit finite bounds. It
rejects a non-null bounds field and retains its exact v3 world identity. Schema 2
uses domain `theia-authoritative-chunk-world-v4\0`, index digest, LE-u16 resident
budget, then four LE-f64 bounds values in min-X/min-Z/max-X/max-Z order, followed
by the existing spawn/fall-plane/initial-set encoding. These bounds therefore
participate in admission independently of contributor envelopes and query frame.
Wire/ALPN remains 11; no wire message layout changed. An incompatible world
identity still rejects admission.

`WorldBounds` converts the admitted global rectangle to conservative local
capsule-centre planes. Lower bounds round inward/up and upper bounds inward/down
when f64-to-f32 conversion would otherwise extend the allowed rectangle. The
shared kinematic solver selects the earliest world, authority-coverage or
client-readiness hit. Its existing sliding/step/gravity machinery consumes those
hits; no extra physical walls are inserted in every package and internal seams
remain open. The finite boundary is infinite in height and constrains XZ only.
Pure vertical motion remains free. Jumping does not bypass the world rectangle.

Ordinary authority movement and candidate publication revalidate full global
footprints. Existing finite fall-plane recovery uses the already validated world
spawns. Directional/airborne lookahead clamps candidate positions to the admitted
rectangle before deriving contributor demand, so it does not request a cell
solely beyond a finite world edge. Geometry may extend outside the gameplay
rectangle; it is not implicitly clipped or counted as a larger playable area.

The shared client startup consumes and validates the same schema/configuration;
its expected admission identity includes the rectangle. Rendering/query frames
remain independent. The client has no new trusted movement authority. The normal
launcher remains monolithic town revision 2 at 30 Hz.

## Acceptance and evidence boundaries

Headless original fixtures exercise all four edges and four corners, nonintegral
bounds, 20/30/60 Hz real Avian motion, jumps/landing/retreat, zero/signed billion-
metre positions, vertical sweeps, open internal seams, bounded demand and legacy/
changed-bound/query-frame-independent admission identities. These are bounded
controller/contract checks, not source-world perimeter traversability claims.

Native checks must separately establish actual encrypted two-client readiness,
wall pressure, authority positions/rendered agreement and live Avian, including
independent client frames and delayed multi-provider publication, before claiming
this integration gate. Existing town/farmhouse and Quinn regressions remain
required. Executed results are recorded in [validation](VALIDATION.md), not inferred
from the fixtures or this contract.

An opt-in `build/world-edges-town-runtime-stage` preserves the source town's
playable rectangle from the monolithic profile, with unchanged package/image
bytes and a new schema-2 session descriptor. The legacy runtime-capacity stage
and launcher stay unchanged. Headless publication of all 26 packages is distinct
from actual-town GLB loading/memory/staging and full-world edge/corner rendering.
Those resource/adoption checks and remaining handover reversal/supersession/
asymmetric-failure cases still precede default adoption.

## Qualified checkpoint — 2026-10-20

Fresh fmt, denied-warning Clippy and build pass. The workspace passes 157 Rust
tests with three private tests ignored; a final four-case edge suite includes
three existing cases plus the newly added finite-world unsupported-fall case,
for **158 distinct passing ordinary Rust tests**. The three private tests pass
separately. **85 Python tests** pass. Actual application binaries retain the
qualified source after the recovery test addition; that addition changes tests only.

**Six native two-client cases / twelve reports** pass on RX 7900 XTX / RADV NAVI31
Vulkan with active validation: positive X and negative Z finite-edge pressure,
negative billion-metre delayed six-provider publication with independent client
query frame, town/farmhouse routes and Quinn. Both actors stay within their
admitted rectangle and finish at the expected finite plane in the edge cases.
The negative publication keeps retained standard roots/shared images, exact
retirement barriers and matching live Avian; no unexpected recovery occurs.
Its screenshot was visually inspected. Original-fixture rendering does not
establish actual-town client resource usage or full source-map edge accessibility.

The opt-in finite-town authority loads ten providers then publishes all 26 with
a live actor, preserving its pose/support and zero recoveries over 90 ticks.
Single concurrent-qualification samples are 2,068.804 ms initial preparation,
2,104.401 ms candidate preparation and 14.084 ms publication. These are not cold
load measurements, scheduling percentiles or a resource/performance baseline.
Both source map routes pass all six stages at 20/30/60 Hz with zero recoveries,
and the monolithic shared-image dependency verifier still passes all 488 images.

Private evidence: `build/world-edges-qualified-native-v2-20261020/`,
`build/world-edges-qualified-summary.json` and `build/world-edges-*.log`.
The rejected first fixture is retained separately: its wall allowed only 2.395 m
of travel, while the unchanged native diagnostic requires over 3 m. Corrected
fixtures provide over 3 m before the wall; no diagnostic threshold was weakened.
No default map change, native Windows execution or complete streaming claim.
