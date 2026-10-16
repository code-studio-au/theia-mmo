# Static graybox world and third-person camera

Environment items, including doors, stay static. Players pass through players and
NPCs; targeting/hit detection is separate. An optional reference scene has an idle
wolf presentation placeholder; authoritative NPC simulation and combat do not exist yet.

## Shared authored layout

`crates/network/src/world.rs` contains immutable box positions, sizes, surface
categories and pawn dimensions. Bevy renders those same boxes; the server uses
them for legal movement. Offline movement uses the same helper but remains
non-authoritative. No player/NPC list enters environment collision.

This is a flat procedural scene, not a general terrain/asset format. The building
is a solid box, the fixed door a protruding solid box, and rocks are solid boxes.
Thin roads are walkable. The capsule centre stays at Y=0.9. There is no gravity,
jump, step climbing, slope handling, mesh collision or navigation. Those need a
separate slice when authored assets require them; this is not a physics engine.

## Authoritative movement

The server retains its fixed 30 Hz tick and normalized 5/8-unit walk/run speeds.
Each tick sweeps X then Z against boxes expanded by the pawn's 0.35-unit radius.
Sweeps prevent stepping across thin obstacles and allow wall sliding. Clearance
is conservative and square at corners, not an exact round capsule sweep. Axis
order is deterministic and can bias corner behaviour. Geometry above standing
height does not block the pawn; the roof still obstructs the camera.

Bounds remain XZ +/-19. Spawn points are authored clear of solids; the helper
assumes a legal start and does not depenetrate arbitrary teleports or imported
geometry. No client coordinates, timestep, collision result or camera angle is
accepted as authority. Players still pass through each other; NPC pass-through
remains a requirement for future NPC implementation.

## Presentation controls

- Hold right mouse and move it to orbit the local pawn.
- Wheel zooms between 2.5 and 22 units; Home resets the camera.
- WASD/arrows move relative to camera yaw; Shift runs. World-space axes use the
  existing bounded intent protocol. Camera angles stay local.
- The cursor locks/hides only while right-dragging a focused window and releases
  on mouse release or focus loss. R reset remains offline-only.

Pitch is limited to stay above flat ground and away from the poles. A 0.15-unit
expanded-box segment test shortens the target-to-eye distance at static geometry.
The local body hides below a one-unit camera distance and reappears when space
clears. Characters do not obstruct the camera. This is a simple orbit controller,
not a complete WoW input scheme, smoothing system or imported-scene qualification.
Local movement still follows server poses directly; prediction is absent.

The [reference art scene](REFERENCE_SCENE.md) enlarges the shared building to an
8 x 6.4 x 8 body plus roof/door, to fit a uniformly scaled farmhouse. The same
envelope is used in the plain scene and on the server. Door contact is now
Z=-2.54; historical smoke evidence used the earlier smaller house. Collision
stays conservative and solid, with no traversable imported interior.

## Verification scope

CPU tests cover solid door/building/rock sweeps, wall sliding/retreat, clear roads,
camera obstruction/clearance, orbit/zoom limits, camera-relative axes, cursor focus
state and close-body hiding/restoration. An actual QUIC client keeps submitting
movement after reaching the door: server poses must remain outside it while input
sequence numbers continue advancing.

`--world-smoke <directory>` scripts an offline walk to the door, records the blocked
pose, points the camera into the building to record retraction, then orbits/zooms
back to a clear screenshot. `tools/verify_smoke.py` checks this and the original
movement smoke. Scripted GPU runs exercise camera transforms/rendering; physical
mouse/keyboard, OS cursor capture, resize/recovery and native Windows need separate
manual/runtime qualification.

A separate [bounded terrain walking mode](TERRAIN_WALKING.md) now uses source
triangles, server-owned height/slope checks and sampled terrain camera clearance.
The flat graybox and reference-scene limits above still apply to those modes;
the full map inspector does not establish general world collision.
