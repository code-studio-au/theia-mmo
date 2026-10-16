# Bounded authoritative terrain walking

An opt-in terrain-only patch now connects the imported map to real multiplayer
walking. The server samples the source triangles for ground height and movement;
the client displays authoritative X, Y and Z. Two independent Bevy clients have
rendered this slice on RX 7900 XTX / RADV with active Vulkan validation.

This is a small walking fixture, separate from the full map inspector and the
human/wolf/farmhouse scene. Players remain nonblocking. The patch defaults to
capsules; optional human player visuals now use the same authoritative roots.
Buildings, props, water and NPCs are absent from this patch. Their visual previews
do not establish reviewed gameplay collision.

## Prepare and run

Use the existing isolated Python asset-tool environment from the
[terrain conversion workflow](MAP_CHECK_2026_10_16.md). Outputs must be new:

```sh
build/asset-tools/bin/python tools/prepare_terrain_world.py /home/steven/wow.export/maps/2999/adt_31_27.obj build/terrain-play --center 180,34,2400
cargo build --locked --workspace
target/debug/theia-server 127.0.0.1:5000 build/terrain-server-cert.der --terrain-world build/terrain-play
```

In two other terminals, launch the same command:

```sh
target/debug/theia-client --terrain-world build/terrain-play --connect 127.0.0.1:5000 build/terrain-server-cert.der
```

WASD/arrows move, Shift runs, RMB orbits, wheel zooms, Home resets the camera,
Escape exits. Movement waits for terrain loading. The server remains loopback-only
and R teleport/reset remains disabled when connected. A server/client without
`--terrain-world` still uses the existing procedural world. The reference-art
scene also retains its existing server collision and launch commands.

The prepared local qualification fixture is
`build/qualification/terrain-walking/world/`. It contains `terrain.glb`,
`terrain.json` and `preparation.json`; all are ignored, external reference content.
No proprietary map data is added to Git.

## Animated human players

Add `--player-assets <directory>` after the terrain directory and before the
connection arguments. Only `human.glb` is needed; the server command is unchanged:

```sh
target/debug/theia-client --terrain-world build/qualification/terrain-walking/world --player-assets build/reference-scene --connect 127.0.0.1:5000 build/terrain-server-cert.der
```

The directory can be the existing prepared reference assets or a separate folder
containing only the prepared human. Bevy's standard named asset source loads
`players://human.glb` independently of the terrain directory. No house or wolf is
loaded/spawned by this option. The full `--reference-scene` flag retains its own
world and must not be combined with terrain mode. Missing assets/dependencies or
animation players fail explicitly, with the existing bounded loading timeout.

Human visuals remain children of local/remote authoritative roots. Source bounds
are uniformly fitted to 1.8 m, the measured undeformed foot offset meets the
server's ground height, and source +X is corrected to game -Z. Server XZ/Y,
interpolation, visibility and despawning control the existing roots. Horizontal
movement selects stand/walk/run through the existing Bevy animation graphs.
There is no foot IK, slope-aligned skeleton, stride matching or animation blending.
This changes presentation only; the server does not trust the human asset as a
collider and wire version 2/world identity are unchanged.

The prepared human remains the same compatibility asset, with the documented
[skin validation errors](REFERENCE_SCENE.md). This slice does not repair weights,
joints or unused source buffers. Source and prepared assets remain unchanged and
outside Git; successful selected-clip rendering is not full glTF conformance.

Reproduce the actual two-client locomotion check on a display:

```sh
python3 tools/verify_multiplayer.py build/terrain-human-check --terrain-world build/qualification/terrain-walking/world --player-assets build/reference-scene
```

The scripted route walks outward for 1.6 s, sprints back for 0.5 s, then stops.
Both actors must produce child-local sampled motion in stand, walk and run;
changing terrain/root height cannot satisfy this animation check. Reports include
per-actor `human_animation.pose_changes` in stand/walk/run order. Sampling is enabled
only for network smoke diagnostics. The checker also verifies server/renderer XYZ,
source-triangle height, matching worlds, pipeline readiness and no wolf/house in
this terrain mode. Saved human/terrain PNGs are inspected separately.

## Geometry, authority and protocol

The tool retains complete source triangles intersecting a 48 m square around
source origin `(180,34,2400)`, subtracting that origin without rescaling or smoothing.
It preserves selected UVs, authored normals and textures. Six textured mesh parts
contain 576 triangles; the GLB is 15,299,184 bytes. The larger visual border allows
camera movement beyond the legal pawn area. No neighboring terrain is invented.

`terrain.json` is a bounded experimental collision fixture, not a general cooked
world format. It records the same local triangle coordinates as the GLB, its
SHA-256 and the source origin. The preparer verifies source OBJ/MTL/PNG hashes
before and after conversion and refuses missing textures or existing outputs.
Rust rejects malformed, oversized, nonfinite, downward/vertical/degenerate triangle
profiles and unsafe authored spawn points. At most 8,192 triangles / 4 MiB JSON
are accepted. The source heightmap PNG's encoding remains unqualified and unused.

The authoritative server owns the existing 30 Hz clock, input expiry, identities
and normalized horizontal walk/run speeds of 5/8 m/s. Ground height comes from
barycentric interpolation of terrain triangles; capsule centre is ground +0.9 m.
There is no client position, height, timestep or collision result in input packets.
Legal centre XZ stays within +/-18.65 m, allowing the existing 0.35 m pawn radius
inside the +/-19 m boundary. Spawn points are local X=0,2,4,6 and Z=0, reused for
additional peers; overlap is allowed.

Movement divides each tick into horizontal steps no longer than 5 cm, resolving X
then Z. Nine square-footprint samples require present ground with slope <=40 degrees.
Centre height changes must also respect that slope, rejecting sampled abrupt
steps/drops. This permits sliding and retreat without actor collision. Horizontal
speed drives animation classification, so walking uphill does not become running.

Wire version and ALPN are now **2** (`theia-mmo-walk-2`): snapshots include bounded
finite Y, and the reliable 44-byte welcome includes the 32-byte world identity.
For terrain this is SHA-256 of the exact profile bytes, including the visual hash;
for the procedural/reference world it is an explicit zero identity. A mismatched
client fails before sending movement, and terrain clients verify the local GLB
hash before connecting. This is asset consistency under the existing trusted
local-server transport, not account authentication or an anti-cheat guarantee.
The 1,200-byte datagram budget and eight-peer prototype cap remain unchanged.
Old version-1 executables must be rebuilt together.

The terrain camera samples its segment at <=5 cm intervals and retracts before
missing ground or terrain within 0.15 m of the eye. It does not use the unrelated
graybox building boxes. Characters never obstruct the camera. Captures also require
stable compiled render pipelines, not just CPU asset loading.

## Executed checks and limits

34 Rust tests and 24 Python asset-tool tests pass, with formatting, denied-warning
all-target Clippy and build. Tests cover triangle interpolation, safe spawn,
steep/missing ground and sampled step rejection, sliding/retreat/bounds, player
pass-through on sloped ground, independent wire validation, matching Y through
actual QUIC and rejection of a mismatched world. Offline tests verify exact
visual/collision triangle identity, normals, UVs, texture pixels and source/overwrite
protection. Additional tests check the human foot/axis fit, child cleanup when a
player despawns and rejection of root movement as animation evidence. The real
prepared terrain GLB has zero Khronos errors/warnings.

`tools/verify_multiplayer.py --terrain-world <directory>` launches a server and two
real clients on the caller's display. It verifies distinct IDs, independent
movement, changing server heights, agreement with a separate Python triangle
interpolator, matching local/remote XYZ, both pawn colours in real PNGs and render
pipeline readiness. Native terrain and procedural regression runs pass on
RX 7900 XTX / RADV. Both 1280x720 terrain views are inspected. A reference-art
regression run separately checks the existing two human actors, wolf and house.
The subsequent human-terrain run also verifies actual sampled stand/walk/run for
both local and remote humans using a directory containing only `human.glb`.
These are correctness runs; no terrain throughput, MMO capacity, latency or
native Windows claim follows.

Ignored evidence under `build/qualification/terrain-walking/` records preparation,
source hashes, strict glTF validation, geometry comparison, tests/build/lint and
native server/client logs, JSON reports and PNGs.

This is a grounded height-surface walker, with sampled square clearance. It does
not qualify arbitrary sub-5-cm gaps, exact swept capsule collision, rounded corners,
falling, jumping, step climbing, bridges/caves/stacked floors, navigation, full-map
streaming, authored building interiors, swimming or water gameplay. The sampler
selects the highest overlapping surface, so overlapping walkable floors require
a future qualified collision representation. Reviewed static object clearance and
broader authored-world collision remain next work.
