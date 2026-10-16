# Current status

Initial Bevy 0.20.0 client bootstrap, locally qualified 2026-10-16.

Implemented: a standard Bevy PBR scene made from original primitive geometry,
directional light/shadows, a local capsule pawn, fixed-step keyboard movement,
follow camera, control instructions, reset/quit and an explicit screenshot smoke
mode. There is no dependency on the old custom engine or its benchmark crate.

Linux build, denied-warning Clippy, formatting and three movement tests pass.
A private native Wayland run on RX 7900 XTX / RADV passes scripted movement and a
real 1280x720 screenshot with Vulkan validation active and no validation errors.
The initial Bevy 0.19.1 probe exposed startup presentation diagnostics; the final
0.20.0 run passes without an application renderer workaround.
[Executed evidence and reproduction](VALIDATION.md).

No server, connection, replication, prediction, collision, skeletal animation,
external model import or persistent gameplay is implemented. This is an offline
movement sandbox, not a multiplayer game. The client owns its position only for
this sandbox; the upcoming server will own legal movement/gameplay state.

Next: the authoritative two-client walking slice in `ROADMAP.md`. Replace the
sandbox movement path with input intents and server state rather than treating
client coordinates as authoritative. Asset qualification can proceed with a small
original or appropriately licensed glTF scene when the standard loader is added.

Native Windows graphics and other GPUs remain unqualified.
