# Current status

Bevy 0.20.0 client, local multiplayer, static graybox collision and orbit camera,
2026-10-16.

Implemented: a standard Bevy PBR scene made from original primitive geometry,
directional light/shadows, a local capsule pawn, fixed-step keyboard movement,
orbit/zoom camera, camera-relative controls, reset/quit and screenshot smoke
mode. There is no dependency on the old custom engine or its benchmark crate.

Linux build, denied-warning Clippy, formatting and eighteen tests pass.
Private native Wayland runs on RX 7900 XTX / RADV pass scripted movement,
static-world/camera checks and real 1280x720 screenshots with active Vulkan validation
and no validation errors.
The initial Bevy 0.19.1 probe exposed startup presentation diagnostics; the final
0.20.0 run passes without an application renderer workaround.
[Executed evidence and reproduction](VALIDATION.md).

The connected mode now uses one Rust authoritative server at 30 Hz, Quinn QUIC,
server-assigned identities, bounded input intents and full shared snapshots.
Clients cannot submit coordinates. Server speed, diagonal normalization, bounds,
stale-input rejection and input expiry control movement. The local pawn follows
server positions directly; remote pawns interpolate over a snapshot interval.
Disconnect removes the player; relaunching reconnects with a new ID. The offline
sandbox remains separate and R reset is disabled when connected.

Tests include player overlap/pass-through, actual encrypted loopback two-client movement,
disconnect/reconnect, untrusted-certificate rejection, malformed input and reordered
input/expiry. Two independent Bevy clients ran on RX 7900 XTX / RADV in a private
headless Wayland display. Both rendered both players and reported matching server
poses; Vulkan validation was active with no validation errors. This establishes
local multiplayer walking, not MMO scale or Internet deployment readiness.

Rendering and simulation now share immutable box geometry. The server stops
movement at the building, fixed door and rocks, with conservative pawn clearance
and wall sliding. Thin roads remain walkable on the flat ground. The camera
orbits/zooms and retracts against static geometry; the local body hides when the
eye gets very close. Tests cover door collision through an actual QUIC connection,
camera-relative movement, orbit limits, focus/cursor state and obstruction recovery.
The native world smoke reaches the fixed door and records camera retraction and
restoration before saving its screenshot. [Scope/limitations](WORLD_AND_CAMERA.md).

No local prediction, general terrain/mesh collision, skeletal animation, imported models,
targeting, combat, quests, inventory, accounts or persistence is implemented.
The network executables currently allow loopback only; eight peers is a prototype
cap, not a qualified capacity. Manual multiplayer keyboard interaction, adverse
network behaviour and the user-visible walkthrough remain outstanding.

Confirmed gameplay constraints: environment items are static (including doors),
and players never physically collide with other players or NPCs. NPC pass-through
is a requirement for future NPCs; this slice contains players only.

Next: qualify representative original/licensed glTF/GLB assets and character
animation, then a shared target/ability/health loop. Current world collision is
flat-ground boxes only; slopes, steps and jumping remain unimplemented.
The [WoW-style game direction](GAME_DIRECTION.md), [network slice](NETWORKING.md)
and [roadmap](ROADMAP.md) distinguish intended gameplay from current capability.

Native Windows graphics and other GPUs remain unqualified.
