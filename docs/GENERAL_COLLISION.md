# General collision content — schema 3

Continuation, 2026-10-20: [bounded authority activation](CHUNK_ACTIVATION.md) now
checks current moving bodies against staged collision in the actual Renet loop.
Normal paired client/authority residency remains the next adoption gate.


Native-qualified continuation, 2026-10-20. General static triangle geometry,
explicit spawns and query-only chunks now pass real Bevy/client/authority and
chunk lifecycle checks. The normal launcher selects
`build/development-world-town-v3/`, logical `theia.development.lordaeron-town`
revision 2. Staged revision-2 outdoor/farmhouse packages also pass their native
regressions. Migrated maps preserve their old boxes/floors and visual bytes; this
format does not infer source-mesh interiors from those proxies. Original legacy
packages remain available through explicit `--world`. [Evidence](VALIDATION.md).

## Contract

`terrain.json` schema 3 retains checked content-frame Y-up metre coordinates,
visual SHA-256 binding, explicit finite XZ bounds, and optional static/camera-only
boxes. `triangles` accepts nondegenerate full 3D triangles, including walls and
ceilings; vertices stay within ±1023 m. At most 131,072 triangles, 4,096 boxes and
32 MiB profile bytes are accepted. Spatial coverage remains bounded. General
triangles participate in world movement, ground queries and camera collision.
Use consistently authored outward surface winding; collision is queried on both
sides. Arbitrary source meshes are not inferred from existing conservative boxes.

`spawns` holds zero to eight local capsule-bottom guesses. Loading validates
finite bounds, nearby support and full-body clearance through the shared
controller. Explicit lower/upper spawns choose their intended floor without a
highest-surface ray. Zero-spawn packages are valid query-only chunks; normal client
and server startup reject them before session/GPU initialization. World admission
also fails closed if no spawn exists. Schema 1/2 retain historical trusted spawn
selection for compatibility; schema 3 does not require origin or 0/2/4/6 m support.
Legacy `walk_surfaces`/nonzero `step_height` are rejected in schema 3; merge authored
floors into the general triangle list. Ordinary world-envelope/fall policy remains
as described in [the controller contract](KINEMATIC_CONTROLLER.md).

Manifest schema 1 now accepts collision schema 3. Its canonical identity layout
is unchanged; geometry/spawn bytes, collision schema and package revision bind the
new identity. Headless loading still needs no visual-file I/O. Wire remains v7;
new geometry/spawn data changes package identity, not the input/snapshot layout.

## Authoring and migration

`tools/prepare_collision_world.py VISUAL.glb COLLISION.glb OUTPUT --content-id ID
--revision N --spawn X Y Z` imports separate unextended self-contained static GLBs.
It bakes each collision node instance's world transform into bounded local triangle
data, preserves source bytes, rebases visual roots when requested, and publishes
an exclusive new package directory. Animation, skinning, morphs and non-triangle
primitives are rejected. `--origin` and `--bounds` are optional; spawn coordinates
are local after origin subtraction. Omit spawns for query-only chunk content.
The importer checks structural/geometry budgets; authoritative loading checks
spawn support/headroom. Publication alone is not runtime qualification.

`tools/upgrade_collision_world.py SOURCE OUTPUT --revision N --spawn X Y Z`
verifies an existing schema-2 package's byte bindings, preserves its visual and
collision geometry, merges legacy authored floors, adds supplied explicit spawns
and publishes a strictly newer revision under the same logical ID. Route fixtures
are retained with their updated profile digest. Original directories remain intact.
Three local development packages were migrated to revision 2 using their existing
trusted spawn locations. The town is now the normal launcher default after native
qualification; the other packages are explicit regression worlds.

`tools/create_collision_fixture.py OUTPUT` generates an original multi-room mesh
with floor, walls, ceiling, step and overpass through the actual importer. Its
`OUTPUT/world` package has two explicit spawns. `tools/create_chunk_fixture.py`
now generates schema-3 packages with no spawns for the residency diagnostic.

## Controller finding and remaining gate

A combined floor/ceiling mesh exposed hidden support when the usual raised probe
started against a separating ceiling in the same collider. Ground acquisition now
tries bounded query-only lifts of 20/10/5/0 mm, stopping on a blocking nonwalkable
hit. It never moves the character through headroom. Tests cover a 1.815 m ceiling
with a 1.8 m character, as well as ceiling-limited jumps and walls in one mesh,
explicit stacked floors, chunk-only admission and invalid geometry/spawns.

The bounded native gate now passes: original mesh walk/jump, town jump/route,
farmhouse route, explicit Quinn full-tile travel, signed billion-metre frames,
spawn-free chunk publication/replacement/rollback/unload, startup rejection and
actual launcher defaults/cleanup. See [validation](VALIDATION.md). Moving-actor
chunk activation, normal residency adoption, streaming and prediction remain open.

## Native qualification findings — 2026-10-20

The resumed native gate found and fixed first-frame zero-delta BVH refresh
([contract](FLOATING_ORIGIN.md#zero-delta-startuprebase-continuation--2026-10-20))
and a camera error. Schema-3 camera clearance now uses the existing world-mesh
sweep directly. The legacy highest-terrain sample remains only for schemas 1/2;
applying it inside a general mesh room mistook its roof for ground and collapsed
the camera into the character. A combined-room regression independently checks
camera clearance below the ceiling. Native qualification now also requires a
visible local character and camera distance above one metre at capture; animation
entity counts or image colour variety alone cannot satisfy those checks.
