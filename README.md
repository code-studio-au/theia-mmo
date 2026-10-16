# Theia MMO

A Rust MMO project using **Bevy 0.20.0**. The first target is two clients moving
together in a small world controlled by an authoritative Rust server.

The current application is an **offline movement sandbox**: original graybox
geometry, a lit scene, capsule pawn, follow camera and keyboard controls. Networking
and gameplay authority are not implemented yet. See [status](docs/STATUS.md),
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

WASD/arrows move, Shift runs, R resets, Escape exits. Movement follows world XZ,
with a fixed follow camera and Y-up coordinates. Props have no collision yet.

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

The old `theia-engine` repository remains a separate research/reference project.
This repository has no path dependencies on it and imports none of its private
conversation exports or benchmark-specific rendering code.
