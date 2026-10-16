# Theia MMO

A Rust WoW-style MMO project using **Bevy 0.20.0**, with original content.
Two clients can now move together in a small world controlled by a Rust server.

The current application is a **local multiplayer walking prototype**: original
graybox geometry, a lit scene, capsule players, follow camera and keyboard controls.
Static graybox collision and an orbit camera are implemented; combat, quests and
persistence are still future work. See [status](docs/STATUS.md),
[decisions](docs/DECISIONS.md) and the [playable roadmap](docs/ROADMAP.md).

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

The no-argument client is the offline sandbox. For multiplayer, build both apps,
start the server and launch a client in each of two other terminals:

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

Environment items stay static, including doors. Players pass through other players
and NPCs; environment collision and combat hit detection are separate systems.

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
