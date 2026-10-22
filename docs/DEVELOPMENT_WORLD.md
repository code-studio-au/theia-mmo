> Superseded as the everyday development map on 2026-10-22. The default launcher
> now targets the [continental graybox](GRAYBOX_WORLD.md). This document preserves
> the imported-map experiments; select them explicitly with `--world` or
> `--chunk-world`. Forest reproduction is frozen.

# Full-tile development world

Interactive runs use optimized binaries: `cargo build --release --locked --workspace`
then `python3 tools/play.py`. The launcher defaults to `target/release`; use
`--binaries target/debug` explicitly when debugging. The full forest's reported
low frame rate is under [measured investigation](FOREST_PERFORMANCE.md); earlier
functional qualification did not establish an interactive performance pass.

The supplied Ruins of Lordaeron tile is the chosen development world for upcoming
targeting, abilities and quests. Prepared assets live locally in
`build/development-world/`; proprietary source exports and derived assets stay
outside Git. This is the complete **supplied tile**, about 533 × 533 metres, not
the surrounding source-game continent.

## Schema-3 default adoption — 2026-10-20

The normal launcher now selects `build/development-world-town-v3/`, the verified
revision-2 town package with explicit capsule-bottom spawns. Its visual bytes,
conservative boxes and authored floors are retained. [General collision format](GENERAL_COLLISION.md)
also supports dedicated static collision meshes; that capability does not make
all imported building models exact mesh collision. Old packages remain explicit
regressions through `--world`.

After preparing/publishing the legacy town, reproduce the migration into a new
directory with its four qualified local spawn guesses:

```sh
python3 tools/upgrade_collision_world.py \
  build/development-world-town build/development-world-town-v3 --revision 2 \
  --spawn 0 1.164313264334784 0 --spawn 2 1.1891326904296875 0 \
  --spawn 4 1.1680269850182443 0 --spawn 6 1.1891326904296875 0
```

The tool verifies the old package, preserves sources/visuals and publishes the new
identity. Runtime validates full-body spawn support/headroom. Never overwrite an
existing revision directory; qualify newly prepared content before using it.

## Shared movement continuation — 2026-10-19

Normal authority now uses the [shared capsule controller](KINEMATIC_CONTROLLER.md)
for imported maps, including gravity and Space to jump. Existing map packages
retain their collision bytes/identity: conservative static boxes, authored floor
triangles and tall water-exclusion columns. Runtime adoption adds only the finite
world envelope; individual residency chunk seams remain open. Trusted cached
spawns and observable per-actor fall/rejection recovery are development policies.
This does not add complete source-mesh interiors, swimming, navigation, prediction
or normal chunk streaming. Earlier grounded-walker evidence below is historical.

## Versioned content packages

The normal launcher now requires a [world manifest](CONTENT_PACKAGES.md). After
all preparation/access edits, publish the final directory with its stable logical
ID and revision. Existing local fixtures have been packaged; a new checkout or
new preparation needs this explicit step:

```sh
python3 tools/package_world.py build/development-world \
  --content-id theia.development.lordaeron-outdoors --revision 1
```

Use `theia.development.lordaeron-town` for the completed town fixture and
`theia.development.lordaeron-farmhouse` for the farmhouse fixture. A publisher
refuses overwriting an existing manifest; changed content belongs in a new
prepared directory/revision. The earlier preparation commands and historical
wire/grid descriptions below retain their original context. Current runtime is
wire v7, Avian queries and default Renet; current package matching includes ID,
revision and independent collision/visual digests. No new interiors are inferred.

## Launch

After building both applications, launch the server and an animated player together:

```sh
cargo build --locked --workspace
python3 tools/play.py
```

Use `python3 tools/play.py --clients 2` for two independent players. Closing either
client or pressing Ctrl-C in the launcher cleans up the other processes. The
launcher uses an ephemeral loopback port and writes its server log/public
certificate under ignored `build/development-session/`. It requires the existing
prepared `build/reference-scene/human.glb`. The default launcher now selects
`build/development-world-town-v3/`, the same tile with an enterable
[farmhouse](MAP_FARMHOUSE.md) and [blacksmith main room](MAP_BLACKSMITH.md).
The commands below reproduce the original outdoor version; select it explicitly
with `tools/play.py --world build/development-world`.
WASD/arrows move, Shift runs, Space jumps, RMB orbits,
wheel zooms, Home restores the camera and Escape quits. This becomes the normal
development launch path; the no-argument binary still opens the old offline sandbox.

Equivalent manual commands, in separate terminals:

```sh
target/debug/theia-server 127.0.0.1:5000 build/development-server-cert.der --terrain-world build/development-world
target/debug/theia-client --terrain-world build/development-world --player-assets build/reference-scene --connect 127.0.0.1:5000 build/development-server-cert.der
```

## Prepare from the existing export

First produce the [assembled map](MAP_OBJECTS.md) and [liquid preview](MAP_LIQUIDS.md).
Then choose a **new** output directory:

```sh
build/asset-tools/bin/python tools/prepare_map_world.py \
  /home/steven/wow.export/maps/2999/adt_31_27.obj \
  /home/steven/wow.export/maps/2999/adt_31_27_ModelPlacementInformation.csv \
  /home/steven/wow.export/maps/2999/liquid_31_27.json \
  build/qualification/map-liquids/map-with-liquids.glb \
  build/development-world
```

The default source origin `(180,34,2400)` reuses the previously checked outdoor
spawn area. All four spawn positions must pass server footprint, ground, slope
and obstruction checks. `--origin X,Y,Z` prepares an alternative world; restart
server and clients together and pass its directory to `tools/play.py --world`.

The tool translates the assembled GLB with one root node, preserving its entire
embedded binary payload, materials, textures and shared model references. The
terrain profile uses all 65,504 source triangles, translated to the same origin
at float32 precision. Source dependencies are hashed before/after preparation;
existing outputs and malformed geometry are rejected. `preparation.json` includes
every placement's collision policy and the indices of generated envelopes.

## Movement and collision scope

The Rust server owns ground height, movement, slopes, map boundaries and static
obstruction. Clients send only input intents and must match the world's exact
profile/visual identity. Players remain nonblocking. Environment geometry stays
static, including doors.

- M2 props use exported `collisionBox` metadata, converted from source Z-up into
  OBJ `(x,z,-y)` coordinates and through the same parent/placement transforms as
  their visuals. All eight bound corners produce a conservative world AABB.
  Bushes and effects with empty collision bounds remain nonblocking. The exact
  exported empty sentinel is recognized; other inverted bounds fail.
- WMO buildings use their complete transformed visual bounds as solid envelopes.
  **Building interiors are blocked in this original outdoor profile.** The
  [new default profile](MAP_FARMHOUSE.md) opens the actual map farmhouse. The earlier enterable farmhouse
  remains a separate fixture; it does not establish access to these buildings.
- Nonempty imported collision envelopes total 1,144, including nested props.
  Full-map movement considers their vertical overlap with the standing 1.8 m body,
  allowing passage below high props. Camera clearance uses the same finite boxes.
- All 1,372 visible water cells are blocked for movement and camera clearance.
  Water remains the static generic translucent material. No underwater walking or
  swimming is enabled. Blocking cells conservatively reduce shoreline access.
- A spatial grid restricts terrain/solid queries to nearby candidates. It is a
  simple lookup for this measured tile size, with a bounded index-entry budget.
  It adds no streaming subsystem or custom renderer.

Profile schema 2 accepts explicit XZ bounds, up to 131,072 support triangles,
4,096 boxes and 32 MiB JSON. Schema 1 retains its earlier patch limits and column
behavior. Missing/steep terrain and the tile edge block movement. Wire version
and ALPN are now **3** (`theia-mmo-walk-3`), allowing bounded local XZ coordinates
up to ±1023 instead of the old ±19 patch limit. Rebuild both executables together.
The server still constrains each world to its own actual bounds. Eight peers,
30 Hz authority, 1,200-byte datagrams and loopback-only connections remain.

This supplies an outdoor development map with conservative collision. It does
not reproduce source-game collision meshes or material shaders. Rotated AABBs
can block visibly empty corners; tree envelopes can exceed their visible trunks.
Precise entrances, walkable building floors, stairs, stacked surfaces, jumping,
falling, swimming, NPC gameplay and streaming are further work. The sampled walker
is not a general physics solver. Character export validation errors are unchanged.

## Verification

```sh
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
build/asset-tools/bin/python -m unittest discover -s tools/asset_tests
python3 tools/test_align_glb.py
python3 tools/verify_multiplayer.py build/full-map-check --terrain-world build/development-world --player-assets build/reference-scene --full-map
```

The full-map GPU check walks and then runs two real clients, captures settled
views, checks shared authoritative/rendered XYZ and static clearance, independently
interpolates source ground heights and requires travel beyond the old patch. Existing separate
interior and graybox checks remain regression cases. Native execution evidence
and platform limitations follow.

## Executed evidence — 2026-10-16

Formatting, denied-warning all-target Clippy, workspace build, 40 Rust tests and
31 Python tests (27 asset-tool plus four GLB repair tests) pass. New checks cover
full-tile spatial lookup versus brute-force triangle sampling, height-filtered
obstacles, water exclusion, index-budget rejection, movement beyond the old patch
through actual QUIC, source collision-axis conversion, nested placement/scale,
liquid masks, binary preservation and overwrite protection.

The real derived visual passes Khronos glTF validation with zero errors/warnings
and the existing empty smoke-emitter info. Its 704,120,576 bytes preserve the
original assembly's embedded binary payload exactly. All 65,504 rendered terrain
triangles match the authoritative profile exactly at float32 precision, including
winding. The profile is 10,822,701 bytes, with 2,516 total static envelopes.

Three independent native server/two-client sessions pass: full map with animated
humans, the existing enterable-house fixture and procedural graybox. All six Bevy
clients use **AMD Radeon RX 7900 XTX / RADV NAVI31**, Vulkan, an owned private
headless Wayland display and active Khronos validation. There are no validation
errors, VUIDs or panics. Final pipeline readiness, local/remote/server XYZ, source
ground heights, sampled human stand/walk/run and actual PNGs are checked. The
full-map routes end near local `(2,-51.30)` and `(32.59,0)`; the latter contacts
the conservative tree envelope. Both views are inspected; the wooded view is
dark/obscured, and source collision bounds do not provide exact visible-foliage
camera clearance. This remains a presentation limitation.

The normal `python3 tools/play.py` launcher also runs natively with one animated
client, reaches map readiness and exits cleanly on Ctrl-C, with neither its server
nor client left running. Full-map loading took tens of seconds in these debug,
validation-enabled checks; the HUD now reports loading, with a bounded 120-second
terrain timeout. These are correctness checks, not performance measurements.

The first full-map capture was rejected because independently loading clients
had not both settled. The full-map smoke now allows additional settle time. An
interior regression exposed scripted waypoint overshoot from queued movement;
the diagnostic route now eases into goals without changing gameplay controls or
authority. Earlier runs remain separate from final accepted evidence.

Ignored `build/qualification/development-world/` contains strict validation,
geometry comparison, source/preparation audit, native logs/reports/screenshots
under `native-final/{full-map,interior,graybox}/` and launcher lifecycle proof.
The local display harness is host tooling only, with no runtime/build dependency
on the old engine. Native Windows, other GPUs, adverse networks, precise mesh
collision and MMO capacity remain unqualified. All work remains local.

As of 2026-10-19, the launcher defaults to secure Renet transport after
[local network qualification](RENET_QUALIFICATION.md). `--transport quinn` selects
the retained regression path. World/collision content and movement authority are
unchanged by the default switch.
