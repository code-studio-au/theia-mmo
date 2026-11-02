# Offline navigation baker

Initial terrain-only prerequisite for P3-07 patrols/P4-03 navigation. This tool
bakes data for the qualified initial headless patrol consumer. The reviewed query
adapter is now shared with a dedicated bounded path worker; dynamic creature
pursuit has not yet adopted that worker.

```sh
cargo run --release --locked -p theia-nav-bake -- bake build/graybox-terrain-v1 build/navigation-terrain-dev
cargo run --release --locked -p theia-nav-bake -- verify build/graybox-terrain-v1 build/navigation-terrain-dev
```

Choose a fresh output directory. Optional integer centre X/Z follow the bake
arguments (default 62 62). At most nine 128 m tiles and a development patrol route
are produced. Geometry comes from the decoded saved terrain's collision triangles,
with neighbors included before clearance erosion and tile clipping. Imported
buildings/caves are outside this first input adapter; cosmetic meshes are not
implicitly collision or navigation. The procedural recipe is not reevaluated.

The package binds terrain identity, controller clearance, tile coordinates and
file hashes. Coarse polygon and detail-height data have bounded validated shapes;
route feet use checked f64 global positions. Runtime consumers must retain actual
collision/grounding authority and reject unavailable geometry. `verify` reports
sampled height error/misses alongside artifact validation; it does not prove
continuous coverage or all-world navigation. The development route is test data.

The tool pins engine-independent Rerecast 0.5.0; the shared query adapter pins
Landmass 0.9.3 in `theia-network`. The baker reexports that adapter. Rerecast remains
outside the server graph, and neither library adds a Bevy integration plugin.
Coordinates stay Y-up at the repository boundary; the query adapter
rotates `(x,y,z)` to the library's internal `(x,-z,y)` and back. No crowd agents,
player/NPC collision, animation links or dynamic path service are introduced.
Landmass's public query API has no node-expansion limit. Queries run offline or on
the dedicated worker over a bounded package, never from `World::step`. Worker
transport, expiry, identity and remaining limits are recorded in
[the path worker contract](../../docs/CREATURE_PATH_WORKER.md).

API/pipeline provenance: [Rerecast](https://github.com/janhohenheim/rerecast) and
its published 0.5.0 C++ comparison test, plus
[Landmass 0.9.3](https://docs.rs/landmass/0.9.3/landmass/). Both packages are
MIT OR Apache-2.0; exact registry checksums are in Cargo.lock. The libraries are
unmodified. [Scope, measured proof and remaining work](../../docs/CREATURE_AI_FOUNDATION.md).
