# Project decisions

Approved direction from the project discussion, 2026-10-16. These decisions apply
to this MMO repository; historical engine decisions remain in `theia-engine`.

| Decision | Consequence |
|---|---|
| Build the MMO using Bevy | Standard rendering, asset, animation and UI systems first; specialized plugins require a game need |
| Fresh repository, preserve the old engine | No wholesale copy or runtime dependency on the experimental renderer |
| Keep Theia as the project name | Bevy is the engine foundation; this is a game project |
| Rust client and authoritative server | Client input/presentation never establishes trusted gameplay truth |
| QUIC via Quinn remains the intended transport | Implement the bounded local walking slice first; review and record any adapted old code |
| Linux primary, Windows first-class | Preserve portability and qualify platforms separately |
| Static authored world; players do not collide with other players or NPCs | Environment collision, targeting/hit detection and navigation are separate requirements |
| All environment items remain static, including doors | No opening doors, moving props or destructible environment; character animation is separate |
| Go services when a concrete requirement needs them | No distributed-service scaffold in the first playable slice |
| Start with two clients in one authoritative world | Do not substitute synthetic proxy counts for working multiplayer |
| Use the supplied human for players, RustyWolfPup for enemy NPCs and the farmhouse for building/housing tests | Opt-in local reference scene first; NPC authority/combat and housing gameplay are separate from visual roles |

The previous custom ash/Vulkan renderer, mandatory task/mesh capability, Slang-first
shaders, custom GPU ECS, visibility-buffer pipeline and bespoke vector UI are not
requirements of this application. Use Bevy's renderer/device capabilities and
normal conventions. In particular, this scene uses Bevy's Y-up coordinates with
movement in XZ; the old engine's Z-up convention is not copied into game data.

Bootstrap implementation choices: pin [Bevy 0.20.0](https://bevy.org/news/bevy-0-20/),
use Rust 2024 and Rust 1.99.0, and keep keyboard
movement explicitly offline until the authoritative slice. Keep changes local
until the user authorizes a GitHub push.

The user's subsequent direction is a WoW-style MMORPG. See [game direction](GAME_DIRECTION.md)
for the intended gameplay and first starting-zone loop. The implementation uses
original content and standard Bevy systems; these targets do not imply existing
combat, quests or WoW asset compatibility. Local networking choices and reviewed
code provenance are documented in [NETWORKING.md](NETWORKING.md).

The graybox now has a single shared geometry definition for rendering and server
collision, with a flat-ground box sweep and a local orbit camera. This qualifies
only the current scene, not general terrain physics. Scope and limitations are
recorded in [WORLD_AND_CAMERA.md](WORLD_AND_CAMERA.md).

Building/prop delivery target: glTF 2.0, preferably GLB, with original/intermediate
sources retained. Standard Bevy loading is enabled, including its normal automatic
reflected type registration. A separate visual inspector preserves source scale
and never establishes server collision. The user's supplied house is a local
compatibility reference; it is not original Theia world content. Current import
scope and source provenance are recorded in [ASSET_IMPORT.md](ASSET_IMPORT.md).

A bounded terrain-only walking fixture now qualifies authoritative ground height
and sampled slope clearance using the same source triangles as its visual GLB.
It is an opt-in implementation slice, with world consistency in wire version 2,
not approval of general mesh physics or a new cooked-world architecture. See
[TERRAIN_WALKING.md](TERRAIN_WALKING.md).

The terrain fixture can now use the already approved human player visuals through
standard Bevy asset loading/animation. This optional presentation reuses the same
authoritative player roots; it adds no character collision or protocol authority.
The separate reference scene retains the wolf and farmhouse.

The first terrain static-object slice uses an explicitly authored solid farmhouse/
foundation envelope shared by server movement and client camera clearance. This
is conservative ground-blocking collision, not automatic general mesh collision,
traversable housing or a change to nonblocking players/NPCs. See
[TERRAIN_STATIC_OBJECTS.md](TERRAIN_STATIC_OBJECTS.md).

The user requested walking inside the house. A separate authored ground-floor
fixture now qualifies a permanently open doorway, fixed access ramp, measured
walls and source floor support for the fixed player size. The old solid-house
fixtures stay available. This adds no movable environment or player/NPC blocking;
general multi-floor physics and housing systems remain separate. See
[ENTERABLE_HOUSE.md](ENTERABLE_HOUSE.md).
