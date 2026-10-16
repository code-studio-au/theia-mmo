# Project decisions

Approved direction from the project discussion, 2026-10-16. These decisions apply
to this MMO repository; historical engine decisions remain in `theia-engine`.

| Decision | Consequence |
|---|---|
| Build the MMO using Bevy | Standard rendering, asset, animation and UI systems first; specialized plugins require a game need |
| Fresh repository, preserve the old engine | No wholesale copy or runtime dependency on the experimental renderer |
| Keep Theia as the project name | Bevy is the engine foundation; this is a game project |
| Rust client and authoritative server | Client input/presentation never establishes trusted gameplay truth |
| QUIC via Quinn remains the intended transport | No networking code is implemented yet; adapt prior work only after review |
| Linux primary, Windows first-class | Preserve portability and qualify platforms separately |
| Static authored world; ordinary characters do not block each other | World collision, hit detection and navigation still need implementation |
| Go services when a concrete requirement needs them | No distributed-service scaffold in the first playable slice |
| Start with two clients in one authoritative world | Do not substitute synthetic proxy counts for working multiplayer |

The previous custom ash/Vulkan renderer, mandatory task/mesh capability, Slang-first
shaders, custom GPU ECS, visibility-buffer pipeline and bespoke vector UI are not
requirements of this application. Use Bevy's renderer/device capabilities and
normal conventions. In particular, this scene uses Bevy's Y-up coordinates with
movement in XZ; the old engine's Z-up convention is not copied into game data.

Bootstrap implementation choices: pin [Bevy 0.20.0](https://bevy.org/news/bevy-0-20/),
use Rust 2024 and Rust 1.99.0, and keep keyboard
movement explicitly offline until the authoritative slice. Keep changes local
until the user authorizes a GitHub push.
