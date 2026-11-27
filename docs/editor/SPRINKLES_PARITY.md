# Sprinkles comparison baseline

Theia imports `doceazedo/sprinkles` at commit
`0e978f5a9afc6b456b3693c8761e73a6c7e155bd`. The comparison below is against
that revision, not an assertion about later upstream releases. The original
uses Bevy 0.19; Theia's local port uses Bevy 0.20.

## Fixed-step preview comparison — 11 November 2026

All seven bundled RON examples were captured in the pinned upstream editor and
Theia's ported standalone host. Both used a 1/60-second manual simulation step,
four seconds of wall-clock warmup, and the same per-emitter seeds assigned in
memory (including for subsequent one-shot loops). The source RON files were
unchanged. Each host captured five points, at 30, 60, 120, 180 and 300 updates
after a seeded restart; a second Theia run provides a repeatability baseline.
GPU extraction and screenshot scheduling are not guaranteed to be synchronized.

Reviewed representative frames of Rain, Tube Trails, 3D Explosion, Magic Puff,
Acid Pool, Loot and Windy Snow look close in shape, color and arrangement. The
selected-emitter outline differs between hosts. On the 750×688 viewport crop,
whole-image mean absolute RGB differences ranged from 0.048 to 0.248 on a
0–255 scale across the 35 upstream/local frames; the largest local repeat mean
was 0.021. Much of each crop is background, so these figures are not an
effect-only accuracy score. Before fixing the loop seeds in the capture setup,
late one-shot frames also differed substantially between two local runs.

A follow-up disabled the selected-emitter bounds in both hosts and repeated
Tube Trails and Magic Puff (`build/sprinkles-no-bounds-20261111/`). Their
representative frames still look close. The current harness uses this cleaner
setting; it does not change the earlier seven-example measurements above.

Theia's captured runs reported zero Vulkan validation errors. The pinned
upstream debug host produced repeated Vulkan validation diagnostics, so its
captures are not a validation-clean reference. Both ran on Linux, AMD Radeon
RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6. Comparison panels, logs, per-frame
metrics and effect/binary hashes are in `build/sprinkles-fixed-seeds-loops-20261111/`
and `build/sprinkles-fixed-seeds-rest-20261111/`. This is evidence for these
preview examples, not proof of identical behavior for all effects, hardware,
editor interactions or the in-world camera and lighting. In-world world-space
particles now translate their GPU positions on ordinary floating-origin shifts;
content-frame changes and out-of-bounds offsets still clear them.

To repeat, apply `tools/sprinkles_deterministic_upstream.patch` to a clean
checkout at the pinned commit and build its `bevy_sprinkles_editor` binary.
Build Theia's `bevy_sprinkles_editor` example `deterministic_capture`, then run
`tools/compare_sprinkles_fixed_step.py NEW_OUTPUT` with the two binary paths and
the desired `--effect` options. The harness modifies in-memory seed values only;
it does not save changes to the examples.

## What is shared and what differs

| Area | Current evidence and differences |
| --- | --- |
| Effect data | Original Sprinkles RON effect data and editor bindings are reused. Theia adds local library/zone references and protects project switches and explicit saves. Migration/reload tests exist; full effect/preset interchange is not yet qualified. |
| Particle simulation and rendering | The Sprinkles runtime and shaders are ported, including emission, lifetime, curves, gradients and materials. Bevy 0.20 shader integration and buffer encoding required changes. Runtime fixes include culling bounds, resource cleanup, checked allocations and degenerate shape geometry. Behavior and visual equivalence across all effects is **not established**. |
| Particle controls | Original inspector, curve, gradient, color and playback components are hosted in Theia. They are adapted for Bevy 0.20 and embedding, so input, focus, popup layering and dock interactions can differ. Native testing has revealed problems and further retesting is pending. |
| Preview and zone | Sprinkles has its own preview scene. Theia can also select a zone placement and show its original inspector while previewing in the zone. Lighting, camera, scale, frame timing and environment can change appearance independently of the effect data. |
| Custom materials | Theia rejects unsupported custom-shader effects instead of reaching the original unimplemented panic path. Custom shader rendering remains unimplemented. |
| Saves and recovery | Theia adds guarded project switches, focused-field save handling, safe writes and draft recovery. These are local behavior changes, not upstream parity. Zone files and effects are saved explicitly. |
| Game integration | The zone editor exports bounded particle packages and can add or explicitly replace particles in a new chunk world. The client plays verified world placements, and a bounded cosmetic projectile cue is driven by the server. These are small validated slices, not MMO-scale capacity or complete gameplay integration. |

A byte comparison of the two source trees at this baseline found 11 changed
and 88 unchanged same-path files in the runtime, and 61 changed and 68 unchanged
same-path files in the editor. There are also files present only on one side,
including renamed shaders and locally added host/recovery code. These counts
include non-code assets and do not measure the significance of each change.
The source inventory is recorded in `third_party/sprinkles/source-files.json`;
see `third_party/sprinkles/PROVENANCE.md` for adaptation history.

## What remains to establish visual parity

Run the same upstream examples and representative authored effects in the
original editor and Theia. Use the same effect file and textures, fixed emitter
seeds, matched camera, lighting, background, scale and timing, and compare
multiple points over an effect's life and after looping. Cover shape, spawn and
lifetime, velocity, size and rotation curves, color/alpha gradients, materials,
trails, sub-emitters, turbulence, collisions, sorting and culling. Test saving
and reopening in both directions where supported. Record devices, renderer,
engine versions and effect versions, and distinguish a deliberate local fix
from an unexplained difference. Fixed seeds reduce random differences but do
not by themselves guarantee equal GPU or frame-timing results.

Existing native checks demonstrate selected original-widget edits and preview
operation on Linux; they are not that controlled cross-version comparison.
Windows, the full preset/widget matrix and user-reported popup, gradient and
particle-deletion interactions remain unqualified. See
[SPRINKLES_INTEGRATION.md](SPRINKLES_INTEGRATION.md) and
[SPRINKLES_UPSTREAM_REVIEW.md](SPRINKLES_UPSTREAM_REVIEW.md).

## Exploratory native comparison — 11 November 2026

Captured all seven bundled examples (3D Explosion, Acid Pool, Loot, Magic Puff,
Rain, Tube Trails and Windy Snow) in the original pinned Sprinkles editor and
Theia's ported Sprinkles standalone host, using identical upstream RON files,
default preview settings and 1366×768 windows. Reviewed a representative
capture from each. The effect colors, shapes and general appearance were
similar; individual positions and the selected-emitter bounds overlay differ.
This is a visual observation, not proof of equivalent simulation or rendering.
The standalone host shares Theia's ported editor/runtime but does not exercise
the zone workspace, its camera, lighting, placement or input interactions.
Subsequent connected-client tests use exported upstream effects, including
chunk startup, streaming and rebasing; they do not make the original and Theia
captures frame-synchronized or establish cross-version visual equivalence.

Each run produced three stills around 3, 6 and 9 seconds after emitters appeared.
Separate approximately seven-second recordings cover Magic Puff and Tube Trails.
Evidence and full logs are under `build/particle-parity/`; these generated files
are not committed. The recordings include startup shader warmup; early black
frames do not establish a persistent blank preview. RON files were not edited.
Original and ported captures were separate runs; random seeds, precise phase,
frame timing and shader warmup were not synchronized. Consequently, differing
particles cannot be attributed to the port from these images alone. Windows,
other devices, the embedded zone and deterministic frame-by-frame comparison
remain untested. Video frames were sampled for inspection; full animation
equivalence has not been established.

Linux Vulkan reports AMD Radeon RX 7900 XTX (RADV NAVI31), Mesa 26.2.6, for
both recorded hosts. The original debug build emitted Vulkan validation errors,
including synchronization, presentation and shader diagnostics; local video
logs did not report validation errors. This is not a validation-clean upstream
run or a performance benchmark. The original capture host differs from the
pinned code only by `tools/sprinkles_parity_upstream.patch`, which isolates its
preferences and schedules timed screenshots/exit; it does not change particle
simulation or rendering. The matching Theia capture host is
`third_party/sprinkles/crates/bevy_sprinkles_editor/examples/parity_capture.rs`.
The instrumented upstream binary SHA-256 was
`f32941e7e2bb3e5bbfa1db7c7b2052b6bbe54f6e27309cb573d7516fcca8f115`;
the local host was
`03f91c5195bf79669688c4aa8bf3633192d9c6f0360c433197ca3155e5f20239`.
