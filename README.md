> Development target: the 16 km × 16 km procedural graybox world with the POLYGON
> Base Locomotion player. Run `python3 tools/play.py` after a release build.
> See [graybox world](docs/GRAYBOX_WORLD.md) for controls, asset preparation,
> developer travel and explicit legacy-map commands. Forest work is frozen.

# Theia MMO

A Rust WoW-style MMO project using **Bevy 0.20.0**.
The development world is a 16 km × 16 km procedural graybox with the POLYGON
Base Locomotion player. Original or appropriately licensed final world art remains
planned; imported map experiments are retained as regressions.

The current application is a **local multiplayer walking prototype**, with animated
players, bounded continental terrain streaming, server-owned capsule movement,
static collision and an orbit camera. Combat, quests and
persistence are still future work. See [status](docs/STATUS.md),
[decisions](docs/DECISIONS.md) and the [engineering plan](docs/engineering/README.md).
The [repository assessment](docs/engineering/REPOSITORY_ASSESSMENT.md) compares the
current prototype with the October 19 handoff and identifies what to retain or
replace. The [earlier playable roadmap](docs/ROADMAP.md) preserves milestone history.

The normal client and authoritative collision queries now use **Avian 0.8.0-dev**,
pinned to revision `1070759b2145dc078abedace91dc7fe78013a45c`, with Bevy `=0.20.0`.
See [physics migration and current limits](docs/PHYSICS_AVIAN.md).

## Development map

The continental graybox is the world for ongoing development:

```sh
cargo build --release --locked --workspace
python3 tools/play.py
```

The launcher uses optimized `target/release` binaries. For deliberate debug runs,
pass `--binaries target/debug`; unoptimized full-scene runs are unsuitable for
judging interactive performance.

The default world uses the locally prepared `character.glb` under
`build/synty-character-canonical-stage`. Add `--clients 2` for two connected players;
see [graybox controls, travel and evidence](docs/GRAYBOX_WORLD.md). Client movement
prediction is enabled by default; `--no-prediction` retains server-pose interpolation.
Server authority remains unchanged.

The retained town is explicit:
`python3 tools/play.py --world build/development-world-town-v3`. Its [farmhouse](docs/MAP_FARMHOUSE.md) and
[blacksmith](docs/MAP_BLACKSMITH.md) remain collision/camera regressions. Prepared
private legacy assets and manifests are required; see
[preparation and limits](docs/DEVELOPMENT_WORLD.md). Forest reproduction is frozen;
its existing packages can still be selected with `--chunk-world`.

## Run

Install Rust with rustup; `rust-toolchain.toml` pins Rust 1.99.0, formatting and
Clippy. Linux also needs native windowing development packages. On Ubuntu/Mint:

```sh
sudo apt install build-essential pkg-config libwayland-dev libxkbcommon-dev \
  libx11-dev libxi-dev libxrandr-dev libxcursor-dev
cargo run --locked -p theia-client
```

Use a working graphics driver and desktop session. On Windows, use the Rust MSVC
toolchain with Visual Studio C++ build tools/Windows SDK and a working GPU driver;
the same Cargo command applies. Native Windows execution is not yet qualified.
Audio and gamepad dependencies are intentionally absent from this bootstrap.

The no-argument client is the offline sandbox. For default Renet multiplayer, use
the development launcher above. The retained Quinn graybox diagnostic can also be
run manually: build both apps, start the server and launch two clients:

```sh
cargo build --locked --workspace
cargo run --locked -p theia-server
cargo run --locked -p theia-client -- --connect 127.0.0.1:5000 build/server-cert.der
```

Run the final command twice. Each client shows its own blue pawn, other players in
red, its server-assigned ID and the connected-player count. The server creates an
ephemeral localhost identity and writes only its public certificate. Restart
clients after restarting the server. These executables currently accept loopback
addresses only; there is no account authentication or session resume.

WASD/arrows move, Shift runs, Escape exits. R resets only in offline mode.
Hold right mouse to orbit, wheel to zoom and Home to reset the camera. Movement
follows camera yaw in world XZ with Y up. The building, fixed door and rocks block
movement; the camera pulls in around static scenery. The server owns connected
movement; local prediction is not yet implemented.
See [world/camera scope](docs/WORLD_AND_CAMERA.md), [network behaviour/provenance](docs/NETWORKING.md) and
[game direction](docs/GAME_DIRECTION.md).

The current scene is static, including doors. The adopted architecture allows
future scripted platforms/elevators and vegetation effects. Players pass through
other players and NPCs; environment collision and combat hit detection are separate.

The first [architecture alignment](docs/engineering/ALIGNMENT.md) adds checked
global/local coordinate contracts, configurable 20/30/60 Hz server simulation and
secure Renet networking, now the launcher default after controlled local network
qualification. Graybox/imported-map authority and the offline sandbox use the
[shared capsule controller](docs/KINEMATIC_CONTROLLER.md), including gravity and
Space to jump. Imported maps use validated cached development spawns and an
explicit finite-world/fall-recovery policy.
Chunk sessions use [coverage sweep boundaries](docs/CHUNK_BOUNDARIES.md) for
unloaded edges, including gravity-driven sliding.
[Geometry contributor bounds](docs/CHUNK_CONTRIBUTORS.md) now distinguish loaded
packages from complete movement cells and validate actual static geometry before
readiness/publication. Imported tile partitioning remains in development.
Wire/ALPN version 10 adds authority offers and tagged readiness acknowledgements to per-client chunk readiness,
held jump input, server tick rate, incarnation and frame origin; rebuild client
and server together. [Opt-in paired handover](docs/CHUNK_HANDOVER.md) uses normal
Bevy/Avian publication. [Shared image packages](docs/SHARED_IMAGES.md) bind
compatible image paths and loader variants for standard Bevy reuse; real imported
partitioning is still ahead. The launcher retains the monolithic development map.
See [current status](docs/STATUS.md) for qualification. Server arguments
accept `--tick-hz 20`, `30` (default) or `60` after the address/certificate.

## Inspect a GLB

```sh
cargo run --locked -p theia-client -- --inspect-asset apps/client/assets/reference/duskwoodhouse_medium.glb
```

The user's reference house loads in a separate visual inspector with orbit/zoom,
actual bounds and a 1.8-unit scale capsule. It has **no UVs, materials or textures**;
this export renders plain shaded. It does not replace the server's graybox world
or establish collision. See [asset import/export guidance](docs/ASSET_IMPORT.md).

Add `--capture build/asset-check` to save `asset.json` and `scene.png`, then run
`python3 tools/verify_asset.py build/asset-check apps/client/assets/reference/duskwoodhouse_medium.glb`.

Newer textured building/character exports were also checked. Add
`--animation "Exact clip name"` to inspect a selected animation; stand, walk and
run are tested for the user's human model. See [model results and commands](docs/ASSET_CHECK_2026_10_16.md)
for the successful rendering and remaining character export errors.

To use the local human players, idle wolf and static farmhouse together:

```sh
cargo run --locked -p theia-client -- --reference-scene build/reference-scene
```

This opt-in mode needs prepared local assets; see [preparation and scope](docs/REFERENCE_SCENE.md).
Add the usual `--connect` arguments after the asset flag for multiplayer.

A bounded imported terrain patch is also playable through the authoritative
server, with source-triangle ground height and slope checks. See
[terrain preparation, launch commands and limits](docs/TERRAIN_WALKING.md).
Add `--player-assets build/reference-scene` after the terrain directory for animated
human players. A [static farmhouse fixture](docs/TERRAIN_STATIC_OBJECTS.md) also
adds shared world/camera collision. An [enterable ground-floor variant](docs/ENTERABLE_HOUSE.md)
also provides a fixed ramp, open doorway and floor support. The [full development
map](docs/DEVELOPMENT_WORLD.md) now connects the complete visual assembly to outdoor
server-owned movement and conservative object/water boundaries.

## Check

```sh
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --release --locked -p theia-client -- --smoke-test build/smoke
```

The smoke mode scripts two movements, records checkpoints/device information and
saves a real rendered `scene.png` before exiting. It requires a graphical display;
a private headless compositor also works. Set `VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation`
with the Vulkan validation layer installed for a separate Vulkan diagnostic run.
This mode tests a local client, not a server or two-client connection.

With Python 3 and Pillow installed, check its output using
`python3 tools/verify_smoke.py build/smoke`. See the [local qualification record](docs/VALIDATION.md).
After building, `target/debug/theia-client` (or `theia-client.exe` on Windows)
also launches the sandbox directly.

The static-world smoke walks into the fixed door, records camera obstruction and
restoration, then saves a clear orbit view:

```sh
cargo run --locked -p theia-client -- --world-smoke build/world-smoke
python3 tools/verify_smoke.py build/world-smoke
```

After building, run the actual two-client/server smoke test on a graphical display
(or an independently prepared private headless compositor):

```sh
python3 tools/verify_multiplayer.py build/multiplayer
```

It launches a server on an ephemeral loopback port and two Bevy clients, scripts
distinct movement intents, checks matching authoritative/rendered poses, saves
both screenshots and cleans up the processes. For Vulkan qualification, also set
`WGPU_BACKEND=vulkan WGPU_VALIDATION=1 WGPU_DEBUG=1`,
`VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation` and `VK_LOADER_DEBUG=layer`.
The checker proves layer activation from loader output when that layer is requested.

The old `theia-engine` repository remains a separate research/reference project.
This repository has no path dependencies on it and imports none of its private
conversation exports or benchmark-specific rendering code.

The next engineering gates follow the [foundation execution order](docs/engineering/EXECUTION_ORDER.md).
The normal launcher now uses Renet: `python3 tools/play.py --clients 2`.
Use `--transport quinn` for the retained regression baseline. See
[network qualification](docs/RENET_QUALIFICATION.md) and [Renet scope](docs/NETWORKING_RENET.md).

Connected authority now stores checked f64 global poses and uses explicit
incarnation/frame contracts across replication and local presentation. See
[runtime coordinates and identity](docs/COORDINATES_IDENTITY.md) for the tested
large-origin slice and fixed-anchor limitations.

Connected clients now rebase their local render/static-physics frame automatically
(default 128 m span), keeping global authority and authored child poses intact.
See [client rebasing](docs/FLOATING_ORIGIN.md) for verification and remaining scope.
