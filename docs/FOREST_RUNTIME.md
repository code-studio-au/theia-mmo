Priority update, 2026-10-22: forest reproduction is frozen. Everyday development
moves to the [continental graybox](GRAYBOX_WORLD.md). The following forest results
and open fidelity/performance gates remain regression evidence, not the active queue.

# Complete forest runtime candidate

The interactive walkthrough subsequently exposed poor frame rate. Functional
qualification below is not a performance pass; see the active
[performance investigation](FOREST_PERFORMANCE.md). Later movement endpoint
checks pass 246 fixed-rate/jump replay cases, with conservative stalls recorded;
see [controller evidence](KINEMATIC_CONTROLLER.md). Fresh forest-human, town and
farmhouse native regressions pass in `build/forest-movement-native-v5-20261021/`.

P1-10 / P2-06 / P2-07 / P2-08 / P2-10 prerequisite, 2026-10-21.
The complete static forest now loads through the normal encrypted authority and
two Bevy clients. This is a candidate qualification checkpoint; the development
launcher still uses the retained town map.

## Bound source packages

`tools/bind_world_materials.py` converts the ten qualified source shape/visual
packages into package schema 3. It embeds 72 material definitions with four
content-equivalent source profiles, global-zero anchors and canonical PNG
dependencies. All 18,660 draws, 18,925 primitive instances and 6,137,378 triangles
remain unchanged. There are 47 shared PNG variants: 37 core and ten projection
variants. The immutable reference publishes no session or spawn file.

`tools/verify_bound_world_materials.py` independently compares exact binary
geometry, every scene/fallback field, collision, source sidecar parameters, PNG
bytes and package/index identities. Rehashed parameter, fallback, geometry and
collision faults all reject; the source still verifies afterward. Actual Rust
package admission and all 804,029 source collision rays pass. Ten Khronos files
have zero errors and the same 22 retained generated-tangent warnings.

## Explicit provisional water policy

The complete reference retains the source's invisible downward water collider.
The first startup probe rejected old terrain-only spawn guesses because that
plane obstructs their capsules. A full-scene survey also demonstrated that the
two-sided collider creates an artificial walkable floor around water level.

`tools/prepare_forest_land_candidate.py` creates a separate candidate excluding
only source object `864140990` (`Water_Collision`) from all queries. It checks the
qualified source shape mapping before conversion and compacts unused mesh pools.
An independent comparison expands mesh references and verifies that every other
object/field, all visual bytes and all PNG bytes remain identical. The candidate
retains 2,761 object colliders plus four full-sample fields (2,765 bodies).

Water is presentation-only in this provisional candidate. Swimming, buoyancy,
underwater behavior and an improved water shader remain unimplemented. This is
not approval of underwater land-walking as final gameplay. No launcher default
or persistent source asset changes are made.

## Executed normal-client evidence

A separate explicit probe session loads all ten providers within the existing
budgets, with finite terrain bounds and actual clear capsule spawns. A real Renet
authority and two Linux RX 7900 XTX / RADV NAVI31 Vulkan clients pass ordinary
readiness, actor visibility/pixels, movement clearance, independent replication
and render rebasing, image/material bindings and validation-layer checks. Each
capture has 63 compiled pipelines, zero pending/failed pipelines, and five clear
frames. This is native Linux execution, not Windows execution or a capacity test.

Each actual client reports four shared projection materials across all 528
projected primitive instances. Independent phase checks cover every binding
after two render rebases. Explicit source renderer flags now use ordinary Bevy
shadow markers in normal static imports as well as inspection; actual markers
match all seven noncasting instances, including the one nonreceiving instance.
Standard material unlit/emissive properties also match their source bindings.

The actual global shared-image inventory has exactly 47 distinct paths/handles,
with source PNG dimensions, RGBA8 colour roles and one mip each. Each client holds
1,589,649,408 bytes of base RGBA CPU image data. This excludes mesh data, render
targets, GPU overhead and process memory; it is not a total VRAM measurement or a
production texture budget. Mips/resource quality remains an adoption gate.

The view uses generic development lighting. The separately qualified inspector's
23 supported source lights/fog are not yet integrated into this normal world.
Water and other documented source shader approximations remain visible.

## Movement findings and remaining gates

The opt-in `forest_runtime` survey evaluates 225 points and two-second routes
against actual full-scene capsule queries. The land candidate yields 27 clear
two-direction starts. It also records 41 rejected routes: 31 support stability
failures and ten solver penetration rejections. The survey completing is not a
claim that these routes pass. The earlier complete-water reference yielded 26
clear starts and 37 rejected routes. Both reports and the initial failing survey
are preserved. These findings require solver investigation before adoption.

Remaining work includes robust movement on overlapping/uneven source collision,
broader actual camera/routes, authored lighting integration, mip/resource policy,
water behavior and an explicit default-map qualification. No navigation mesh or
MMO-scale streaming capacity is established by this checkpoint. The subsequent
Elven Realms import remains after the forest gate.

Local evidence stays ignored:

- `build/forest-bound-materials-reference-v1/reference.json` and
  `build/forest-bound-materials-verification-v1.json`.
- `build/forest-bound-materials-native-v1.json`, full source rays.
- `build/forest-bound-materials-faults-v1/verification.json` and
  `build/forest-bound-materials-khronos-v1.json`.
- `build/forest-land-candidate-verification-v1.json`, exact water exclusion.
- `build/forest-bound-materials-route-survey-v2.json` and
  `build/forest-land-route-survey-v1.json`, including rejected cases.
- `build/forest-land-native-v1-20261021/forest/verification.json`, two-client
  captures/logs, and `resource-verification.json`, actual image inventory.

Fresh 44 client tests, 221 asset Python tests, denied-warning workspace/all-target
Clippy, format and diff checks pass for this continuation. The complete default
network suite passed at the preceding material contract checkpoint; the new
private-map survey is opt-in and does not replace that suite. Later movement
changes require fresh regression evidence.
