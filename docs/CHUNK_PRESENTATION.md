# Bounded Bevy chunk publication

Continuation, 2026-10-20: [normal initial chunk startup](CHUNK_STARTUP.md) now
uses the ordinary server/Bevy client with standard GLB roots, player sessions,
live Avian and explicit client query frames. The diagnostic below retains its
separate staged replacement/rollback scope. Paired dynamic adoption remains open.

P2-06/P2-07/P2-08 continuation, 2026-10-19. The real client has an opt-in
`--chunk-smoke <original-fixture-directory> <output> [--expect-failure]` diagnostic.
It consumes the [bounded residency model](CHUNK_RESIDENCY.md), ordinary Bevy GLB
assets/world scenes and actual Avian static bodies. Normal multiplayer tile
loading remains unchanged and monolithic. This does not implement navigation,
prediction, a general controller or production streaming.

## Publication lifecycle

The diagnostic keeps one published residency set and at most one pending worker.
A candidate clones shared immutable package/query references, loads/verifies the
required collision packages and streams visual-hash checks on Bevy's I/O task
pool. The main thread polls completion without blocking. The published set stays
usable while this work runs; a failed candidate never replaces it.

The main-world state then requests standard `Handle<Gltf>` assets. Both root asset
and recursive dependency failures reject the candidate. Once all dependencies
load, missing chunk roots instantiate normal `WorldAssetRoot` scene hierarchies
with hidden visibility. Roots use translations projected from canonical f64
chunk origins. Existing retained roots/handles are reused, not respawned.

After the new hierarchies contain mesh instances, one exclusive Update publication
reveals staged roots, removes evicted roots and their linked scene children,
creates/removes matching flat Avian static bodies, and replaces the published
residency resource. Body translations are explicitly projected into the same
frame; moving only a scene-parent transform is insufficient for physics.
PostUpdate Avian updates the live broad phase before Last-stage verification.
Staged hidden scenes do not publish physics bodies early.

The diagnostic checks every frame, including while preparing replacements:

- Visible root ownership equals published collision ownership.
- Live Avian ground-query presence/heights match the published shared query scene
  at points in all three chunks; unloaded distant ground actually disappears.
- The live standing-capsule cast hits the middle thin wall at the expected distance
  whenever its owning chunk is published, and misses after everything unloads.
- At most two active roots and two pending roots; retained middle-root entity
  identity survives the replacement.

This is hierarchy/dependency readiness, not a general guarantee that a new GPU
pipeline draws on its first publication frame. Screenshots wait for observed
pipeline compilation and stable frames, then verify actual colored floor pixels.
The original fixture uses compatible PBR material variants. A production consumer
still needs gameplay activation policy, changing-actor owner checks at commit,
request supersession/cancellation, replication integration and measured memory/
GPU/resource budgets. Staging can retain old and new scenes/assets/query worlds;
a two-root active limit is not a peak-memory claim. Asset/GPU resource release
uses normal Bevy handle lifetimes and asynchronous renderer behavior.

## Native qualification

Create a new original fixture, then run on a working native display:

```bash
build/asset-tools/bin/python tools/create_chunk_fixture.py build/chunk-visual-fixture
cargo run --locked -p theia-client -- \
  --chunk-smoke build/chunk-visual-fixture build/chunk-visual-capture
```

The fixture colors left/middle/right floors red/green/blue. It captures the first
pair, replacement pair and empty scene. The independent `tools/verify_chunks.py`
qualifier creates temporary original fixtures, runs the actual binary, checks the
report and actual 960×640 screenshot pixels, then removes the temporary inputs.
It requires Vulkan validation activation evidence in loader logs, not merely an
environment request. It records the client/index hashes and exact GPU.

```bash
build/asset-tools/bin/python tools/verify_chunks.py build/chunk-visual-check
# Optional distant signed grid placement:
build/asset-tools/bin/python tools/verify_chunks.py build/chunk-visual-far-check \
  --base-x -7812500 --base-z -7812500
# A correctly bound package with deliberately invalid glTF:
build/asset-tools/bin/python tools/verify_chunks.py build/chunk-visual-failure-check \
  --expect-failure
```

The rejection case replaces the third visual with invalid glTF, updates its
profile/manifest/index bindings consistently, then lets Bevy reject asset decoding.
The old red/green pair and collision must remain; blue must never publish. The
positive case must remove red and add blue; final unload must remove all colored
pixels. Reports of loaded entities alone cannot satisfy these image checks.
Native devices, accepted runs and limitations are recorded in [validation](VALIDATION.md).

## Schema-3 continuation — 2026-10-20

The actual-client diagnostic now qualifies schema-3 packages with zero spawn
points at origin and signed billion-metre frames. Replacement, retained-root
pixels, failed-glTF rollback, unload and live BVH queries pass again under active
Vulkan validation. Normal session executables separately reject those query-only
packages. Evidence: ignored `build/general-collision-native-qualified/chunks-*/`.
Moving-actor activation and normal multiplayer residency remain open.
