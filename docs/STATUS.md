# Current status

Bevy 0.20.0 client, local multiplayer, static graybox collision, orbit camera
and a separate GLB inspector,
2026-10-16.

Implemented: a standard Bevy PBR scene made from original primitive geometry,
directional light/shadows, a local capsule pawn, fixed-step keyboard movement,
orbit/zoom camera, camera-relative controls, reset/quit and screenshot smoke
mode. There is no dependency on the old custom engine or its benchmark crate.

Linux build, denied-warning Clippy, formatting, 25 Rust tests and 14 offline
Python asset-tool tests pass.
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

The user-provided `duskwoodhouse_medium.glb` passes Khronos validation and renders
through Bevy's standard loader in an independent inspection mode. It was moved
from Downloads into `apps/client/assets/reference/` after rendering succeeded,
with original bytes preserved. The file has 425 source positions, 672 triangles
and no UVs, authored normals, materials or textures. Bevy generates flat normals
and a default material. This qualifies geometry-only import, not a textured house.
The inspector frames actual scene bounds, shows a scale reference and can save
a real screenshot/report. It does not create gameplay collision or alter the
authoritative world. [Import workflow and export guidance](ASSET_IMPORT.md).

The newer farmhouse and re-exported human character also render with embedded
textures. The inspector now supports explicit named-clip playback with Bevy's
standard animation graphs. Stand, walk and run have actual sampled joint motion
and native rendered-pose evidence. The farmhouse passes glTF validation cleanly;
the character still has empty clips and incorrect accessor metadata, despite the
selected animations working. Large redundant vertex buffers also need cleanup.
Both source exports remain outside Git and unchanged. [Detailed model checks](ASSET_CHECK_2026_10_16.md).

The additional RustyWolfPup GLB also renders its embedded textures and plays stand,
walk, run and attack in the inspector on RX 7900 XTX / RADV with active Vulkan
validation. Its 66-joint skin and selected clips work, but the source fails strict
glTF validation with 3,084 accessor alignment errors. It remains an external local
compatibility reference, with no NPC/gameplay integration. The same detailed report
records the exact source hash and limitations.

An opt-in [reference game scene](REFERENCE_SCENE.md) now uses the human for local
and remote players, an idle nonblocking wolf enemy placeholder and a static
farmhouse. Player stand/walk/run follows actual movement. Both clients and server
use an enlarged shared solid building envelope; native two-client, locomotion,
wolf pass-through and house/camera checks pass. Derived models stay under ignored
`build/reference-scene/`; the default scene remains procedural. The prepared wolf
has zero glTF errors, while deeper human validation still finds skin-data errors.

The supplied Ruins of Lordaeron map tile also converts from textured OBJ/MTL into
a separately inspected GLB: 65,504 triangles and 256 textured patches, with zero
glTF errors/warnings. A further [static map assembly](MAP_OBJECTS.md) now includes
846 top-level placements and 624 nested building props using shared model parts.
The combined GLB renders through Bevy with no glTF errors/warnings. One empty
smoke emitter has no visible effect. Liquids and playable terrain collision remain
unimplemented; the assembled map is still separate from the authoritative world.
[Original map inventory](MAP_CHECK_2026_10_16.md).

No local prediction, general terrain/mesh collision, animation blending,
targeting, combat, quests, inventory, accounts or persistence is implemented.
The wolf currently supplies presentation only, without server-owned NPC state or AI;
the solid house has no usable interior, ownership or housing persistence.
The network executables currently allow loopback only; eight peers is a prototype
cap, not a qualified capacity. Manual multiplayer keyboard interaction, adverse
network behaviour and the user-visible walkthrough remain outstanding.

Confirmed gameplay constraints: environment items are static (including doors),
and players never physically collide with other players or NPCs. NPC pass-through
is a requirement for future NPCs; this slice contains players only.

Next: qualify general terrain/asset collision and representative original/licensed
art, resolve the remaining character skin errors, then add a shared
target/ability/health loop. The direct GLBs already contain PNGs and preserve UVs;
the earlier missing-texture problem is resolved for these new exports.
Current world collision is
flat-ground boxes only; slopes, steps and jumping remain unimplemented.
The [WoW-style game direction](GAME_DIRECTION.md), [network slice](NETWORKING.md)
and [roadmap](ROADMAP.md) distinguish intended gameplay from current capability.

Native Windows graphics and other GPUs remain unqualified.
