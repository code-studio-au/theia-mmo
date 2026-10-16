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

## Geometry-only GLB inspection — 2026-10-16

The user's 9884-byte `duskwoodhouse_medium.glb` was inspected read-only in Downloads,
then moved into the client's `assets/reference/` after actual rendering succeeded.
Original bytes/hash are recorded in that folder's README. The ZIP left in Downloads
contains the same GLB only; no loose BLP files were available for conversion.

Khronos glTF Validator `2.0.0-dev.3.10` reports zero errors/warnings: one mesh,
425 vertices, 672 triangles, no UVs/materials/images/animation/skin. Bevy's standard
loader instantiates it and generates flat normals, yielding 2016 render vertices.
Instantiated bounds match the source bounds; size is about 18.433 x 16.776 x 25.009
source units. No geometry, material assignments or unit scale were altered.
The inspector adjusts its camera/floor and supplies a separate 1.8-unit capsule.

The first native attempt was rejected: the minimal Bevy feature configuration did
not register reflected scene types, causing a scene-instantiation panic and abnormal
shutdown. Enabling Bevy's standard `reflect_auto_register` feature fixes that
configuration; no custom scene clone or manual per-type workaround was introduced.
Rejected logs remain under `build/qualification/gltf/native-rejected-reflection/`.
The subsequent actual GPU run loads/renders successfully with Vulkan validation
active and no validation errors. The plain-shaded house screenshot was inspected.
Reinhard tone mapping in the inspector keeps the untextured white material readable.

Twenty workspace tests pass, including GLB header preflight and framing/zooming a
large asset without snapping to the game's camera limit. Workspace build, formatting
and denied-warning all-target Clippy pass. Successful native evidence uses the same
RX 7900 XTX / RADV and owned private Wayland/Weston headless GL compositor. The
existing two-client, offline movement and world/camera smokes are also rerun after
enabling glTF/reflection support. A malformed GLB must exit unsuccessfully without
producing a successful report/screenshot.

Ignored evidence is under `build/qualification/gltf/`: validator report, build/test/
Clippy logs, rejected and passing native runs, per-client logs/reports/PNGs and the
final asset screenshot. The validator's pinned npm installation is a local ignored
qualification dependency, not part of the game runtime. The game remains independent
of the old engine; the existing external compositor harness is only a test host.

Only geometry import of this file is qualified. PNG/JPEG decoding is enabled but
textured import, BLP conversion, UV/material preservation, character animation,
custom WoW shaders, imported-world collision and native Windows remain unqualified.
Physical camera input is still not manually qualified; the asset run uses auto-fit.

## Textured building and character — 2026-10-16

The [two-model report](ASSET_CHECK_2026_10_16.md) records the exact fresh exports,
hashes, validation issues, image/mesh counts and stand/walk/run playback evidence.
Bevy's `gltf_animation` feature and standard animation graphs are now enabled in
the inspector. A requested clip must have a player and sampled transform changes
before a successful capture is written. Building playback is not automatic and
the authoritative game remains separate.

Twenty workspace tests, formatting, denied-warning all-target Clippy and build
pass. Four native checks render the textured building and the three character
clips on RX 7900 XTX / RADV, with active Vulkan validation and no validation errors.
Textures and changing skinned poses are visible in the PNGs; all source files are
preserved unchanged. These results supersede the previous geometry-only limit for
these particular new exports, but do not validate all character animations or
complete WoW asset compatibility. The character's strict glTF errors remain real;
successful Bevy rendering is not a substitute for a valid portable source export.

## RustyWolfPup — 2026-10-16

Four further native inspector runs qualify stand, walk, run and attack playback
for the user's external `creatures/RustyWolfPup.glb`. Each saves an actual textured
skinned screenshot and reports sampled transform changes. RX 7900 XTX / RADV,
private headless Wayland and active Vulkan validation were used; no validation
errors or panics occurred. Strict glTF validation separately fails with 3,084
alignment errors. [Exact input, counts, reproduction and limitations](ASSET_CHECK_2026_10_16.md#rustywolfpup).
Evidence: ignored `build/qualification/rusty-wolf-pup/`. No runtime changes or
additional workspace test run were needed for this asset/documentation check.

## Integrated reference scene — 2026-10-16

[Reference scene qualification](REFERENCE_SCENE.md) records prepared-file hashes,
remaining human skin errors, animated local/remote players, an idle nonblocking
wolf and the static farmhouse. Twenty-three Rust tests, four Python repair tests,
formatting, denied-warning all-target Clippy and build pass. Seven native client
runs pass on RX 7900 XTX / RADV with active Vulkan validation: reference locomotion,
imported-house world/camera smoke, two connected reference clients, two connected
graybox clients and the original offline movement smoke. All save actual rendered
screenshots. No VUIDs, Vulkan validation errors or panics occur in successful runs.

The shared building envelope is enlarged for sensible reference-model scale.
Door contact moves to Z=-2.54; both the actual QUIC door test and native camera
checks use the new geometry. The wolf has no collider. Native Windows, NPC AI/
authority, combat, traversable house interiors and terrain collision remain open.

## Map terrain visual import — 2026-10-16

The [map report](MAP_CHECK_2026_10_16.md) records the supplied Ruins of Lordaeron
tile, source/derived hashes, exact texture preservation and separate native Bevy
terrain capture. All 65,504 triangles and 256 textured patches survive conversion;
two identical PNGs share one image. Khronos validation has zero errors/warnings and
516 optional-buffer-target hints. RX 7900 XTX / RADV runs with active Vulkan
validation pass for both terrain rendering and a wolf animation regression.
The large-asset inspector now scales camera depth/zoom; loading/capture remain bounded.
Twenty-four Rust tests, seven Python asset-tool tests, format, denied-warning Clippy
and build pass. Map object placements, liquids and terrain-aware authoritative
movement remain unimplemented; this establishes visual import only.

## Static map objects — 2026-10-16

The [assembly report](MAP_OBJECTS.md) records primary-source placement conventions,
846 top-level and 624 nested placements, 183 distinct model inputs, source hashes
and an assembled GLB with 253,552 triangles. One smoke emitter has no source
geometry. Khronos validation reports no errors/warnings, one empty-node info and
2,322 optional buffer-target hints. Default/focused map captures and a wolf
animation regression use RX 7900 XTX / RADV with actual Vulkan validation active
and no validation errors or panics. Shared mesh references survive assembly.

Twenty-five Rust tests, 14 Python asset-tool tests, format, denied-warning Clippy
and build pass. The focus option records camera framing separately from whole-scene
statistics. Native Windows, liquids, map collision/movement, lights/particles and
exact WoW shader reproduction remain unqualified or unimplemented as detailed in
the report. The multiplayer simulation and protocol are unchanged by this slice.
