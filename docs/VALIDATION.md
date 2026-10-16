# Bootstrap qualification — 2026-10-16

This qualifies an offline Bevy client, not a multiplayer game.

## Executed checks

- Bevy 0.20.0, locked dependencies, Rust 1.99.0, Rust 2024, Linux debug build.
- `cargo fmt --check`, denied-warning workspace/all-target Clippy and build pass.
- Three CPU movement tests pass: normalized diagonal input, equivalent distance
  at 30/60 fixed ticks per second, and ground/boundary/idle invariants.
- Native Wayland in an owned private Weston headless/GL compositor on
  AMD Radeon RX 7900 XTX, RADV NAVI31, Vulkan device ID `0x744c`.
- `VK_LAYER_KHRONOS_validation` activation is recorded by Vulkan loader trace;
  `WGPU_VALIDATION=1`, `WGPU_DEBUG=1`. No Vulkan VUID/validation errors, renderer
  errors or panics occur in the successful final run. A benign window-decoration
  warning about an unknown button type remains.
- Scripted smoke input records `(0, 0.9, 0)` at frame 59, `(0, 0.9, -5)` at 119,
  and `(5, 0.9, -5)` at 179. The fixed-step movement checkpoints match within
  0.001 world units. The real rendered screenshot is requested at frame 239.
- The screenshot is 1280x720, contains 2561 distinct RGB colours and 477614 green
  pixels, and was visually inspected. The scene, blue capsule, road, shadow and
  control instructions are present. The image check establishes rendered content,
  not a pixel-exact geometry/lighting oracle or GPU transform readback.

Ignored local evidence lives in `build/qualification/bootstrap/`: `tests-020.log`,
`clippy-final.log`, `build-final.log` and `native-final/{run.log,report.json,scene.png,
verification.json,qualification.json}`. The final qualification records executable
and Cargo.lock hashes, device and compositor information. No generated binaries,
screenshots, dependency SDKs or private conversation files are committed.

## Dependency/platform observations

The initial 0.19.1 probe rendered and passed the movement/image checks, but emitted
undefined-present-layout and already-signaled-acquire-semaphore Vulkan diagnostics.
That run remains rejected under `native/`, with its original lock retained locally.
The fresh 0.20.0 runs pass the same target/device checks with ordinary Bevy rendering;
no custom Vulkan initialization patch or copied benchmark renderer was added.

This machine's Wayland/xkbcommon development packages were available as an
extracted third-party SDK. It was copied into ignored `build/native-sdk/` and used
only through `PKG_CONFIG_PATH`/`LIBRARY_PATH` during compilation. Normal installations
use the OS packages listed in the README. No old engine source or path dependency
is needed to compile or run the application. The local private-display host harness
was reused for qualification; it is not a game dependency.

Linux runtime is qualified on this GPU/compositor only. Linux/Windows CI checks
are configured but have not run on GitHub because no push was authorized. Native
Windows graphics, resize/minimize/recovery, manual keyboard interaction and other
GPUs remain unqualified. This is not a performance benchmark or capacity claim.

## Reproduce

After installing the README's native development packages:

```sh
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation \
WGPU_BACKEND=vulkan WGPU_VALIDATION=1 WGPU_DEBUG=1 \
cargo run --locked -p theia-client -- --smoke-test build/smoke
python3 tools/verify_smoke.py build/smoke
```

The run requires a graphical display or a separately prepared native Wayland
headless compositor. The image verifier needs Python 3 and Pillow. Runtime logs
must also be checked; an image/movement verification alone does not establish a
clean Vulkan validation run. The smoke mode scripts local intents and waits for
a real screenshot callback before success. It never starts or simulates a server.
