# Full-sample forest ground packages

P1-10/P2-06/P2-07/P2-08 continuation, 2026-10-20. This delivers authoritative
**ground-only** collision and opt-in ordinary multiplayer loading. The complete
forest, authored object collision/materials and default-map replacement remain
open. Prerequisites are the independently qualified [TerrainData conversion](UNITY_TERRAIN.md),
existing content/shared-image contracts and bounded chunk startup/controller.

## Packages and invariants

The 150 × 150 metre terrain crosses the existing 128-metre X/Z ownership planes
at zero. Its source grid already has vertices at those planes. Four packages
therefore preserve all 263,169 distinct source vertices and all 524,288 triangles
without clipping or decimation. Each contains 66,049 vertices and 131,072 triangles;
shared boundary vertices are duplicated exactly. All source normals/UVs and the
baked colour image remain unchanged. This preserves the qualified fixed-diagonal
representation; it does not establish Unity adaptive tessellation/TerrainCollider
parity or resolve the documented terrain-material approximations.

`tools/package_unity_terrain.py` verifies the source conversion hash/counts and
requires one static identity terrain mesh/image with the qualified attributes.
It rejects triangles crossing a chunk plane, owners over the existing triangle
budget, invalid source attributes/indices and out-of-domain spawn footprints.
Only new output is written; failed publication removes its new output tree.
Existing `prepare_collision_world.py` and `externalize_world_images.py` provide
checked schema-3 collision, immutable revision-2 packages and canonical shared
PNG bindings. Temporary embedded authoring files are removed afterward.

The index is schema 2 with owners (-1,-1), (-1,0), (0,-1), (0,0), actual contributor
bounds and the usual zero query frame. The schema-2 session loads all four roots,
has budget four, explicit trusted spawn guesses, world bounds ±75 metres and a
fall plane below the lowest source ground. Authority validates the actual capsule
support/headroom and resolves recovery feet before admitting players. No wall is
inserted along internal package seams. The normal 30 Hz default, queue/content
limits, wire/ALPN 11, engine pins and runtime code are unchanged.

Measured encoded content is 69,356,229 collision JSON bytes and 14,751,652 GLB
bytes across four packages, plus **one 10,952,311-byte shared PNG**. Those are
content sizes, not resident-memory or latency measurements. Actual native clients
reuse one standard Bevy image handle across all four roots and create four static
Avian colliders. Each ground package currently fills its triangle allowance;
adding authored object meshes requires a reviewed collision representation or
measured budget qualification, not silently dropping ground samples or obstacles.

## Independent and runtime acceptance

`tools/verify_terrain_packages.py` imports neither packager nor slice writer.
Against the immutable original terrain GLB it verifies:

- All original vertices/heights and exact source UV/normal attributes.
- Complete oriented triangle coverage independently for visuals and collision.
- Every signed owner, compact mesh, actual bounds and immutable package binding.
- Exact source image bytes, shared-image variant, material/sampler and finite
  startup/world coverage.

A rebound collision fixture with a triangle removed fails semantic coverage even
when its manifest/index hashes have been updated. Other tests cover omitted
owners, source mutation, out-of-domain spawns, existing outputs and a triangle
that crosses a chunk plane. Four new Python tests pass.

All four actual GLBs pass Khronos validation with zero errors/warnings. Each has
one informational `URI_GLB` message because its image is intentionally external.

The opt-in Rust source test loads the actual four packages and Avian scene. It
checks **81 source height probes**, including edges and both ownership seams,
against the retained raw height samples. Maximum error is **0.000003815 metres**.
**18 motion cases** cross X, Z and diagonal corner seams, with/without jumping,
at 20/30/60 Hz; body clearance, landing and actual displacement pass. Jump input
starts at validated support, before descending terrain can leave the actor
airborne. Two independently admitted authority actors then remain stable with
zero movement recoveries.

Two actual encrypted Renet sessions, **four native Bevy reports**, pass on **AMD
Radeon RX 7900 XTX / RADV NAVI31, Vulkan**, Linux, with active Khronos validation,
compiled pipelines, matching live Avian generation, exact readiness/identity,
visible local/remote players and visually inspected captures. Both clients cross
the X seam; a second source stage uses explicit source-triangle-derived spawns
and crosses the Z seam. The Z route descends about 0.826/0.858 metres. All four
reports show zero client-vs-authority ground-query error and one shared image
handle per client; no unexpected movement recovery, VUID/error or panic occurs.

`tools/play.py --chunk-world` launches the actual client/server with the existing
human assets and authenticated Renet. Actual launcher checks verify both the
forest option and the unchanged default town: two clients admitted, correct
content IDs/revisions and owned-child/ticket cleanup after SIGINT and child failure.
These four launcher scenarios prove admission/cleanup, not additional rendered
movement cases or a forest-default promotion.

Fresh fmt and denied-warning workspace/all-target Clippy pass. All **160 ordinary
Rust tests** pass; four private tests are ignored by default. The new forest test
and the retained town/farmhouse route test separately pass: **162 distinct executed
Rust tests** in this checkpoint. Both old map routes finish all six stages at
20/30/60 Hz with zero recovery. **131 Python tests** pass (124 asset, four GLB,
one prefab reference, two variant reference). The test-only Rust file does not
change application binaries. No Windows execution or streaming/capacity/whole-task
claim follows. All work remains local.

## Commands and evidence

```sh
build/asset-tools/bin/python tools/package_unity_terrain.py \
  build/synty-forest-terrain-v1 build/new-forest-ground-packages \
  --content-id theia.forest.terrain --spawn -2 -0.97 -2 --spawn -2 -0.97 -4
build/asset-tools/bin/python tools/verify_terrain_packages.py \
  build/synty-forest-terrain-v1 build/new-forest-ground-packages \
  --report build/new-forest-ground-reference.json
python3 tools/play.py --chunk-world build/new-forest-ground-packages --clients 2
```

Use fresh output paths and a working native display. These are ground-only
packages; the capsule/human characters do not imply full forest import. Do not
pass `--transport quinn`: chunk readiness requires Renet. The no-argument launcher
still selects the qualified town.

The ignored source test is reproducible with absolute fixture paths (Cargo tests
run from the package directory):

```sh
THEIA_FOREST_TERRAIN_WORLD="$PWD/build/synty-forest-terrain-packages-v1" \
THEIA_FOREST_TERRAIN_SOURCE="$PWD/build/synty-forest-terrain-v1" \
cargo test --locked -p theia-network --test forest_terrain -j1 -- --ignored --nocapture
```

Use the documented native SDK environment if system development libraries are
missing. Private evidence under ignored `build/`:

- `synty-forest-terrain-packages-v1/`: final index/session, four packages, one image,
  per-package Khronos reports and content counts.
- `synty-forest-terrain-packages-reference-v1.json`: full independent source oracle.
- `forest-terrain-collision-rust-final.log`: actual private source/Avian/motion test.
- `forest-terrain-packages-map-regression-final.log`: retained map routes.
- `forest-terrain-packages-rust-final.log`: complete ordinary workspace suite.
- `forest-terrain-packages-clippy-final.log`: denied-warning check.
- `forest-terrain-packages-python-final-v2.log`: asset suite; seven additional GLB/
  independent reference tests passed separately.
- `forest-terrain-collision-native-v1-20261020/x-seam/` and
  `forest-terrain-collision-z-native-v1-20261020/z-seam/`: reports/logs/screenshots.
- `forest-terrain-launcher-native-v1-20261020/{forest,default-town}/`: actual
  admission/cleanup verification.

Next: full vegetation/material conversion, complete authored scene and object
collision. Measure the collision representation/budget needed alongside this
full-sample ground; keep terrain normals/detail and source effects unresolved
until qualified. Actual ground dynamic residency, complete forest collision,
region/navigation scope and final map adoption remain open.
