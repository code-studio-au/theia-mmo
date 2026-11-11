# Sprinkles comparison baseline

Theia imports `doceazedo/sprinkles` at commit
`0e978f5a9afc6b456b3693c8761e73a6c7e155bd`. The comparison below is against
that revision, not an assertion about later upstream releases. The original
uses Bevy 0.19; Theia's local port uses Bevy 0.20.

## What is shared and what differs

| Area | Current evidence and differences |
| --- | --- |
| Effect data | Original Sprinkles RON effect data and editor bindings are reused. Theia adds local library/zone references and protects project switches and explicit saves. Migration/reload tests exist; full effect/preset interchange is not yet qualified. |
| Particle simulation and rendering | The Sprinkles runtime and shaders are ported, including emission, lifetime, curves, gradients and materials. Bevy 0.20 shader integration and buffer encoding required changes. Runtime fixes include culling bounds, resource cleanup, checked allocations and degenerate shape geometry. Behavior and visual equivalence across all effects is **not established**. |
| Particle controls | Original inspector, curve, gradient, color and playback components are hosted in Theia. They are adapted for Bevy 0.20 and embedding, so input, focus, popup layering and dock interactions can differ. Native testing has revealed problems and further retesting is pending. |
| Preview and zone | Sprinkles has its own preview scene. Theia can also select a zone placement and show its original inspector while previewing in the zone. Lighting, camera, scale, frame timing and environment can change appearance independently of the effect data. |
| Custom materials | Theia rejects unsupported custom-shader effects instead of reaching the original unimplemented panic path. Custom shader rendering remains unimplemented. |
| Saves and recovery | Theia adds guarded project switches, focused-field save handling, safe writes and draft recovery. These are local behavior changes, not upstream parity. Zone files and effects are saved explicitly. |
| Game integration | Zone preview exists; world-package export, game-client playback and gameplay integration are pending. Neither preview demonstrates MMO-scale performance. |

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
