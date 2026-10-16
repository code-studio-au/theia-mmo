# Bootstrap qualification — 2026-10-16

This first section records the original offline Bevy bootstrap. The subsequent
multiplayer qualification is recorded below; neither is an MMO capacity claim.

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

## Local multiplayer qualification — 2026-10-16

Implemented and executed one authoritative 30 Hz Rust server and two independently
launched Bevy clients using Quinn QUIC, verified localhost certificate trust and
real input/snapshot datagrams. No mocked peers or synthetic proxy counts are used.

- Formatting, denied-warning workspace/all-target Clippy, workspace build and
  eleven tests pass: the three original offline movement tests, five server/wire
  unit tests and three actual loopback transport tests.
- Tests cover normalized movement, fixed server timestep, stale input rejection,
  movement expiry, bounds, player overlap/pass-through, capacity/ID lifecycle,
  bounded/versioned messages, two clients seeing movement, disconnect/reconnect,
  rejection of an untrusted certificate and closure on malformed input. A separate
  transport test reorders/replays movement and checks that it neither reverses
  movement nor refreshes the stale intent's timeout.
- Two independent 1280x720 Bevy windows run in the private native Wayland/Weston
  headless GL display on RX 7900 XTX / RADV NAVI31, Vulkan. Both receive distinct
  server identities and report matching final server poses. Local rendered poses
  match their authoritative snapshots, and interpolated remotes settle to the
  same positions. Actual PNGs contain both local blue and remote red players;
  screenshots were visually inspected as well as checked for coloured pixels.
- `VK_LAYER_KHRONOS_validation` activation is proved in **both** client logs by
  loader insertion messages. No VUIDs, validation errors, renderer errors or panics
  appear. The existing benign window-decoration warning remains.
- The original offline smoke mode is rerun with the connected implementation:
  all three movement checkpoints and the real screenshot still pass, with active
  Vulkan validation and no validation errors.

Ignored evidence: `build/qualification/multiplayer/{tests.log,clippy.log,
build-final.log,native-final.log}`, and `native-final/` containing server log,
per-client `a/` and `b/` reports/logs/PNGs, `verification.json` with executable,
lockfile and image hashes, and `offline/` regression evidence. The host compositor
harness is the same independent qualification tool described above. It is not a
runtime/build dependency of the game. Public ephemeral certificates are kept only
in ignored evidence; the private signing identity never leaves server memory.

This qualifies local two-player walking only. Native Windows, manual multiplayer
keyboard/camera interaction, adverse network loss/jitter, Internet exposure,
automatic session resume, local prediction/reconciliation and MMO scale remain
unqualified. NPCs do not exist yet: the player/NPC nonblocking rule is recorded
for future implementation, while the player/player case has an executed test.
World collision, imported character animation, targeting, combat and quests remain
future work; no interactive or moving environment is planned.

To reproduce the two-client GPU check after building on a prepared display:

```sh
VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation VK_LOADER_DEBUG=layer \
WGPU_BACKEND=vulkan WGPU_VALIDATION=1 WGPU_DEBUG=1 \
python3 tools/verify_multiplayer.py build/multiplayer
```

The tool requires Python 3/Pillow, launches/cleans up its own server and clients,
and checks diagnostic logs as well as received state and real screenshots. The
reusable smoke is not a performance or latency benchmark.

## Static-world and camera qualification — 2026-10-16

The renderer and server now use one immutable graybox box layout. The server owns
collision against the solid building, fixed door and rocks; players remain
nonblocking. This is conservative flat-ground clearance with X/Z wall sliding,
not general terrain physics. The local camera supports right-drag orbit, wheel
zoom, Home reset, camera-relative movement and static-box obstruction.

Eighteen workspace tests pass (six client, eight shared/core, four real QUIC).
New checks cover obstacle sweeps in both directions, sliding/retreat, clear roads,
camera obstruction/recovery, angle/zoom limits, rotated movement including
diagonals, cursor release on focus loss and close-body hiding/restoration. The new
transport test keeps submitting fresh movement into the door; snapshots remain
at contact while acknowledged input sequences advance.

Formatting, denied-warning all-target Clippy and workspace build pass. Native
Wayland/Weston headless GL on the same RX 7900 XTX / RADV device passes:

- The real server/two-client rendered smoke, with matching authoritative poses.
- The original offline movement smoke with unchanged expected checkpoints.
- The new `--world-smoke`: pawn reaches approximately `(-7,0.9,-4.54)` at the
  fixed door, camera retracts to approximately 0.232 units when pointed into the
  building, then restores to 10 units after orbiting away. A real 1280x720 PNG is
  saved and visually inspected; it shows the pawn outside the fixed door.

All four client processes prove Vulkan validation activation through loader
insertion logs, with no validation errors, VUIDs, renderer errors or panics. The
existing benign window-decoration warning remains. Ignored local evidence lives
under `build/qualification/static-world/`: test/lint/build logs and native reports,
PNGs, loader logs and artifact hashes. No dependency or hardware changes were
needed. The private host compositor harness remains a qualification tool only.

The camera's native test scripts angles/zoom and records CPU transforms. OS cursor
capture and physical mouse/keyboard operation are covered only by resource/system
tests, not manual native interaction. Native Windows, imported terrain/collision,
slopes/steps/jumps, jitter/loss and local prediction remain unqualified. No NPCs
exist yet; NPC nonblocking remains a requirement. Static environment items have no
opening, movement or destruction.
