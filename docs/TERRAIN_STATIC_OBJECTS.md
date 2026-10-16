# Static farmhouse on terrain

The bounded terrain world can now include authored static solid boundaries. The
first fixture places the supplied farmhouse on a fixed foundation, with animated
human players and server-owned terrain movement. Players stop at its conservative
boundary; the camera retracts against the same box. Players/NPCs remain excluded
from world collision.

A newer [enterable ground-floor fixture](ENTERABLE_HOUSE.md) now provides a
separate open-door/ramp/floor layout. The solid fixture described here is preserved.

This is a single authored placement, not automatic collision for the full map.
The farmhouse/foundation is solid throughout: no usable interior, opening door,
housing ownership, step climbing or walkable roof is implemented.

## Prepare and run

Choose new output directories. Use the existing isolated asset-tool environment
and prepared [reference house/human](REFERENCE_SCENE.md):

```sh
build/asset-tools/bin/python tools/prepare_terrain_world.py /home/steven/wow.export/maps/2999/adt_31_27.obj build/house-terrain --center 350,37,2400
build/asset-tools/bin/python tools/add_terrain_house.py build/house-terrain build/reference-scene/house.glb build/house-world
cargo build --locked --workspace
target/debug/theia-server 127.0.0.1:5000 build/house-server-cert.der --terrain-world build/house-world
```

Launch one or two clients in other terminals:

```sh
target/debug/theia-client --terrain-world build/house-world --player-assets build/reference-scene --connect 127.0.0.1:5000 build/house-server-cert.der
```

The qualified local fixture is `build/qualification/terrain-house/world/`.
Controls are unchanged: WASD/arrows, Shift run, RMB orbit, wheel zoom, Home camera
reset and Escape quit. Server and client must use the same derived world profile.
Existing terrain-only and procedural/reference scenes remain available.

## Authored placement and authority

The new terrain crop uses source origin `(350,37,2400)` on the same supplied map
tile. This gives the house a flatter footprint without modifying any terrain
triangles. The farmhouse is placed at local XZ `(-8,-8)`, uniformly fitted into an
8 x 6.8 x 8 m envelope using its actual transformed scene bounds. Source UVs,
textures and all terrain/house binary buffer bytes are retained unchanged.

The fixed foundation spans an 8.6 m square. Its top comes from exact linear
terrain-triangle intersections with that footprint; its bottom extends 0.1 m below
the lowest intersection. In the accepted fixture, local foundation Y is
approximately `0.992449..1.219195`, so its height is about 0.227 m. The model's
lowest point meets the foundation top. This is an explicit authored placement,
not a claim to reproduce an original map building transform. The first steep-hill
candidate is retained as rejected placement evidence.

The combined GLB has 2,143 triangles: 576 terrain, 1,555 farmhouse and 12 foundation.
It contains 30 meshes, 19 materials and 18 embedded images, totaling 24,194,636 bytes.
Strict Khronos validation reports zero errors/warnings/infos and 23 optional
buffer-target hints. Terrain and house buffers compare byte-for-byte with their
inputs. Original/prepared source files remain unchanged and outside Git.

The experimental schema-1 terrain profile now accepts an optional `solids` list
with at most 64 finite positive-size boxes. This fixture has one:

```text
centre = (-8, 4.505822, -8)
size   = (8.6, 7.026746, 8.6)
```

Movement treats each envelope's XZ footprint as a solid ground-blocking column,
expanded by the existing 0.35 m pawn radius. X/Z sweeps clip exactly to the nearest
boundary before terrain height/slope checks; tangential movement and retreat remain
possible. The front contact is local Z=-3.35. The profile's exact 3D box is also
used for the 0.15 m camera clearance sweep, combined with existing terrain-camera
sampling. Overhead/underpass traversal is not supported by these ground columns.

The primitive sweep/slab helpers are shared with the existing graybox. Its height
filter and original layout remain unchanged. No player or NPC geometry enters
these helpers. Source triangles still determine ground height; the building does
not become a walkable height surface. Invalid sizes/bounds/counts and obstructed
spawn points fail profile loading. A profile without `solids` retains its existing
identity and behavior. New profiles include their solid data and updated visual
hash in the existing SHA-256 world identity; wire version 2 is unchanged.

## Executed checks

36 Rust tests and 26 Python asset-tool tests pass, with formatting, denied-warning
all-target Clippy and build. New tests cover sloped-ground solid sweeps, sliding,
retreat, camera obstruction/recovery, invalid solids/blocked spawn, exact clipped
foundation heights, fitted model bounds, binary preservation and rejected inputs.
An actual QUIC test keeps submitting input after reaching the wall and verifies
that poses stay fixed while applied sequence numbers continue advancing.

Reproduce the connected GPU smoke on a display:

```sh
python3 tools/verify_multiplayer.py build/house-check --terrain-world build/qualification/terrain-house/world --player-assets build/reference-scene --static-collision
```

Client A walks left, turns toward the house and continues pressing against its
boundary. It becomes idle from actual blocked displacement. The test briefly
points its real camera into the house, verifies retraction/body hiding, then
restores a clear camera/body before capture. Client B walks/sprints separately.
The checker accepts either server-assigned initial spawn slot; its route assertions
are relative to the actual initial player position. Ground height must match a
separate triangle interpolator even when the chosen route is nearly flat.

Three independent server/two-client native checks pass on RX 7900 XTX / RADV:
static house, existing human terrain-only, and procedural graybox. Real 1280x720
PNGs and server/client reports are saved. The house run records front contact,
actual camera retraction/restoration and local/remote XYZ matching authority.
Khronos Vulkan validation is proven active, compiled pipelines are ready, and no
validation errors/panics occur in accepted runs. House images are inspected.
These are correctness checks, not performance or MMO capacity measurements.

Ignored evidence: `build/qualification/terrain-house/`, including placement/source
hashes, GLBs, strict validator output, tests/build/lint, intermediate candidates,
and accepted `native-proof/{house,terrain-only,graybox}/` logs/reports/PNGs.
The known human skin-data errors remain unresolved. General mesh collision,
interiors, full-map gameplay, NPC authority, combat and native Windows remain open.
