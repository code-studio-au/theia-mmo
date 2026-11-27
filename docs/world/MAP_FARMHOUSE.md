# Entering the farmhouse on the full development map

The complete map now has one authored, enterable farmhouse ground floor, at its
original exported placement. `python3 tools/play.py --world build/development-world-access`
selects this version and starts near its fixed entrance ramp. The default launcher
now selects the [town version](MAP_BLACKSMITH.md), retaining this interior and
adding blacksmith access. In the farmhouse-only profile, other buildings retain their conservative
solid envelopes; the earlier isolated interior fixture is still available.

## Prepare and launch

First prepare the [full outdoor tile](DEVELOPMENT_WORLD.md). Then derive a new
world without overwriting it:

```sh
build/asset-tools/bin/python tools/open_map_farmhouse.py \
  build/development-world \
  /home/steven/wow.export/maps/2999/adt_31_27_ModelPlacementInformation.csv \
  build/development-world-access
cargo build --locked --workspace
python3 tools/play.py --world build/development-world-access
```

`--clients 2` opens a second animated player. WASD/arrows move, Shift runs, RMB
orbits, wheel zooms, Home resets the camera and Escape exits. Head up the brown
ramp into the farmhouse. Everything in the environment stays static, including
the doorway. Players never physically block each other or NPCs.

The complete original outdoor world remains available through
`python3 tools/play.py --world build/development-world`. Server and client must
use the same world profile. This adds no protocol revision or client movement
authority; the existing world hash distinguishes the derived scene.

## Measured layout and provenance

The authored building is ADT placement `adt_65749577`,
`duskwood_human_farm_set5.obj`, with source SHA-256
`090e535370366cc6016c234c3ce39014c5ad6d685c4531d36483717b46d82c52`.
The preparer rejects other exports and verifies that placement/source hashes
match the original prepared visual world. Source files and prepared inputs stay
unchanged and outside Git. Existing outputs are rejected.

The original map building retains its scale, rotation, materials and placement.
Its 25-degree rotation and furnishing placements remain intact. Only the world's
coordinate origin moves to the outdoor approach, approximately source
`(335.803867,34,2278.543409)`, so normal server-assigned spawns start nearby.

Two named source groups, `interior2` and `interior9`, provide 133 approved upward
ground-floor triangles. Four additional triangles represent a fixed, visible
wooden ramp and landing. Doorway width is 2.67 source units and conservative
headroom above the landing is 3.33, sufficient for the existing 1.8 m player.
The ramp follows the sampled source ground and stays within the 40-degree slope
limit. A bounded 0.2 m allowance handles the landing/floor seam.

The original solid building envelope is replaced by measured side/back/front
walls, fireplace exclusion and a blocked loft/stair area. Subdividing rotated
wall rectangles into short conservative world AABBs preserves the open doorway
without allowing gaps through walls. This adds 419 wall boxes; retained props and
water exclusions give 2,934 movement boxes and two camera-only ceiling/lintel
boxes. No furnishing colliders are removed to make the route pass.

The original complete map's embedded BIN remains an unchanged prefix of the new
GLB; only the small ramp payload is appended. All terrain, building, prop and
water visuals are retained. `access.json` records source hashes, house transform,
the authored route, floor/wall counts and the new world identity.

This qualifies one ground floor. Other interiors, loft access, general stairs,
stacked floors, jumping/falling, swimming and general mesh physics remain absent.
The ground support solver still chooses the highest approved surface.

## Camera and visibility

Terrain scenes now use Bevy's standard ambient light at brightness 350 rather
than the default 80, retaining the directional sun and shadows. This improves
visibility in shade and inside buildings; it does not reproduce source-game
lighting or add a day/night cycle.

After the existing shared box/terrain camera clearance, Bevy's standard
`MeshRayCast` checks the actual static imported meshes. Five nearby parallel
probes conservatively protect the camera near plane; backfaces are included.
Only descendants of the static map scene are eligible. Characters never enter
this query. Translucent effects are excluded; existing water exclusion boxes
still apply. Close obstruction hides the local body, and the normal orbit is
recomputed on the next frame so the camera can recover when the view clears.

This uses Bevy's `mesh_picking` feature, with no custom renderer or physics
dependency. These presentation queries do not establish server collision or
gameplay line of sight. Masked foliage triangles are conservatively treated as
solid geometry, including transparent texture pixels; exact alpha-aware camera
clearance remains a limitation. The probes are sampled clearance, not an exact
swept camera volume.

## Verify

```sh
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
build/asset-tools/bin/python -m unittest discover -s tools/asset_tests
python3 tools/test_align_glb.py
python3 tools/verify_multiplayer.py build/map-access-check --terrain-world build/development-world-access --player-assets build/reference-scene --map-access
```

The native route enters the actual map house, presses against its right wall,
returns outside and re-enters. Reports retain server-owned checkpoints, input
sequences, local/remote poses, sampled animation and camera distances. The checker
independently verifies source ground/support heights and wall clearance. Only
the diagnostic harness uses a shared start marker, allowing both clients to
finish scene/pipeline loading before either begins its timed route. Normal
interactive clients have no start gate.

## Executed evidence — 2026-10-16

Formatting, denied-warning all-target Clippy, workspace build, 42 Rust tests and
33 Python tests pass. New tests exercise Bevy mesh ray casts against real primitive
geometry, actor/translucency exclusion, close-body hiding, clearing an obstruction,
late-arriving static-scene descendants, the rotated doorway's clearance, continuous
wall coverage and source-hash/output protection.

The source/prepared hashes remain unchanged and agree with the original outdoor
world's source audit. Khronos validation of the 704,122,020-byte derived GLB reports
zero errors/warnings and the existing empty-emitter info. Its original map BIN is
preserved byte-for-byte as a prefix. An independent terrain comparison measures
maximum render/profile coordinate difference about 0.0000611 units after rebasing.
The source triangle topology is unchanged. The 12,000,872-byte profile adds exactly
133 source floor and four ramp/landing triangles.

A direct CPU route through the actual authoritative walker passes all six stages.
Three subsequent independent Rust-server/two-Bevy-client sessions pass on **AMD
Radeon RX 7900 XTX / RADV NAVI31**, in an owned private headless Wayland display:
actual map entry/exit, the original outdoor route and the older interior fixture.
All six clients have active Khronos Vulkan validation, ready render pipelines,
real inspected screenshots and no validation errors or panics.

The map-access client records all six server-owned checkpoints with advancing
input sequences. Final re-entry is near local `(7.18,-15.41)`, with capsule-centre
Y about 2.920; the exterior checkpoint is about 1.022. Source-space checks confirm
the floor near Y=4.11 and contact against the right wall. Local/remote rendering
matches authority. The final interior camera distance is about 4.68 rather than
the unobstructed 13.89. The interior screenshot shows the player on the original
wooden floor among the retained furnishings; the exterior view shows the fixed
ramp in the original doorway.

Static mesh camera queries execute in the native map scenes. The qualified
routes are already limited by the conservative terrain/box checks, so no additional
mesh retractions occur on these routes; the direct Bevy ray-cast tests exercise
that additional branch. No alpha-aware or exact swept-volume claim follows.

The first paired run exposed a large difference in client loading times. The
diagnostic start gate fixes premature scripted movement/capture/exit by waiting
for both scenes. The first camera attempt also caught an invalid attempt to
disable a plugin absent from the selected default plugin group; that explicit
disable was removed before accepted runs. These rejected attempts are separate
from final accepted evidence.

Ignored `build/qualification/map-access/` records source audit, CPU route, strict
validation, geometry comparison, tests/build/lint and native reports/logs/PNGs
under `native-final/{map-access,outdoor-camera,interior-fixture}/`. Source assets
stay outside Git; the display harness remains host tooling. These are Linux
correctness checks, not Windows, latency, throughput or MMO-scale qualification.

The normal default launcher also reaches map readiness with one animated client
and exits cleanly on Ctrl-C, leaving neither its server nor client running.
The launcher lifecycle proof is recorded under `launcher/`.
