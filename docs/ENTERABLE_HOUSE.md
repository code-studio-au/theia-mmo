# Enterable farmhouse ground floor

A separate farmhouse fixture now supports walking up a fixed ramp, through a
permanently open doorway, around the ground floor and back outside. The Rust
server owns movement, wall collision and floor height. Players/NPCs remain
nonblocking and all environment geometry stays static.

The previous solid-house fixture and original reference scene retain their
existing conservative boundaries. This layout qualifies one checked model and
its ground floor; it is not a general mesh physics/import solution.

## Prepare and run

Use the existing prepared human/house and isolated Python asset-tool environment.
Choose new output directories:

```sh
build/asset-tools/bin/python tools/prepare_terrain_world.py /home/steven/wow.export/maps/2999/adt_31_27.obj build/interior-terrain --center 350,37,2400
build/asset-tools/bin/python tools/prepare_enterable_house.py build/interior-terrain build/reference-scene/house.glb build/interior-world
cargo build --locked --workspace
target/debug/theia-server 127.0.0.1:5000 build/interior-server-cert.der --terrain-world build/interior-world
```

Launch one or two clients in other terminals:

```sh
target/debug/theia-client --terrain-world build/interior-world --player-assets build/reference-scene --connect 127.0.0.1:5000 build/interior-server-cert.der
```

Prepared qualification output is `build/qualification/house-interior/world/`.
The house/ramp is left of the default starting point. The ramp approach is near
local XZ `(-8.91,3)`; head toward negative Z from there. WASD/arrows move, Shift
runs, RMB orbits, wheel zooms, Home resets the camera and Escape quits.
Restart the server and clients together when switching world profiles.

## Measured model and authored access

The previous 0.3947 scale gave the entrance only about 1.3 m of headroom, below
the 1.8 m player height. The source also represents its steep lower entrance as a
textured face, not walkable stair treads. This fixture uses uniform scale **0.6**,
with approximately 1.998 m of conservative entrance clearance above the landing,
and a fixed brown access ramp. The source house/terrain geometry, UVs, textures
and embedded binary payloads are unchanged. The ramp, landing and foundation are
separate authored geometry in the derived GLB. No door opens or closes.

The source house SHA-256 must match
`7d60b5740f636f536b03695dcf75a6e18ab27e437b15da3c343afd655ee0d798`.
The preparer refuses other models/exports instead of silently applying measured
layout assumptions to an unqualified asset. It also checks the expected source
geometry/ground-floor triangle counts and refuses existing output directories.
Original/prepared inputs are hashed before/after assembly and stay outside Git.

Two measured source floor parts supply 80 upward ground-floor triangles. Six
additional support triangles represent the visible ramp, landing and foundation
top. Seven authored ground-blocking boxes cover the side/back walls, the front
walls around the opening, the fireplace and an excluded loft-stair area. Two
camera-only boxes conservatively cover the ceiling/roof and entrance lintel.
These boxes follow the measured layout, including a doorway gap large enough for
the existing square 0.35 m pawn clearance.

The ramp and actual floor have matching support surfaces. Its landing-to-floor
seam is about 0.037 m. This fixture enables at most 0.2 m of sampled height change
per movement substep for small support seams; the existing 40-degree slope limit
still applies. The floor is raised approximately 2.73 m above the outside approach
in the tested route. Camera obstruction respects the ceiling while inside.

## Runtime representation and limits

The bounded schema-1 walking profile now accepts optional `walk_surfaces`,
`camera_solids` and `step_height`. Existing profiles default to empty surfaces/
camera-only boxes and zero step allowance. Terrain plus support triangles remain
bounded to 8,192 total; all static/camera boxes remain bounded to 64 total, with
finite geometry, valid upward triangles, safe spawn and step allowance <=0.25 m.
The 4 MiB profile limit remains. Exact profile/visual hashes still identify the
world under wire version 2. Inputs remain axes/sprint/sequence only.

Support selects the highest approved surface. This is suitable for this single
accessible floor, bounded by authored walls and an explicit entrance ramp. It does
not support general overlapping floors, walking underneath a bridge, jumping,
falling or navigation. The height-seam allowance is not a qualified general stair
climber or a swept capsule step solver. Clearance remains conservative and square.
Headroom is qualified for this fixed 1.8 m player and measured route; there is no
general dynamic character/ceiling solver. The loft/stair area stays blocked.

The known human skin-data validation errors remain. No foot IK, blending,
interactive doors, housing ownership/persistence, full-map gameplay or combat is
added by this slice. The full assembled map remains a separate visual preview.

## Executed evidence — 2026-10-16

37 Rust tests and 28 Python asset-tool tests pass, with formatting, denied-warning
all-target Clippy and build. New checks cover entry/exit via a ramp and raised
floor, wall stopping, ceiling camera clearance, bounded seam allowance, measured
door/roof headroom, ramp slope, the open collision gap and rejection of an
unqualified source model/output overwrite.

The derived GLB passes strict Khronos validation with zero errors/warnings/infos
and 26 optional-buffer-target hints. Original terrain and house BIN payloads are
compared byte-for-byte in the combined file; source hashes remain unchanged.

Reproduce the actual connected entry/exit check on a display:

```sh
python3 tools/verify_multiplayer.py build/interior-check --terrain-world build/qualification/house-interior/world --player-assets build/reference-scene --interior
```

Two independent Bevy clients connect to a real Rust/QUIC server. Client A walks
to the approach, climbs the ramp, enters, presses against the right interior wall,
returns outside and enters again. Client B walks/sprints independently. Snapshots
record inside/outside/wall poses with advancing input sequences; the checker
verifies the elevated floor against a separate triangle interpolator, wall
clearance, matching local/remote XYZ, completed route and a real shortened camera
inside the ceiling boundary. The scripts wait for both clients to settle before
capture. Their longer lifetime applies only to this diagnostic mode.

RX 7900 XTX / RADV checks pass for entry/exit, the previous solid-house fixture and
procedural graybox: six real GPU clients across three two-client/server sessions.
Actual Khronos Vulkan validation is active, compiled pipelines are ready and no
validation errors/panics occur. Real 1280x720 captures are saved and inspected;
the interior image shows the player standing on the room floor below the roof.
This is correctness evidence, not a performance, scale or Windows qualification.

Ignored `build/qualification/house-interior/` holds prepared assets, source hashes,
strict validation, tests/build/lint, native server/client logs, route/camera reports,
screenshots and aggregate qualification JSON. Proprietary assets remain outside Git.
