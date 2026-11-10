# Sprinkles upstream issue review — 10 November 2026

Scope: P5-17 editor integration, with findings for eventual P2-13 gameplay VFX.
Reviewed all 28 issues returned by the upstream repository's issue search,
including closed reports, issue bodies, states and relevant maintainer comments.
Compared against our imported revision
`0e978f5a9afc6b456b3693c8761e73a6c7e155bd` and Bevy 0.20 port.
Upstream issue status is evidence of upstream disposition, not proof of our
platform/runtime behavior. No wholesale upstream update or engine change.

## Save integrity prerequisite for recovery

Explicit particle saves now commit a valid focused field first, serialize writes,
and retain dirty state until a successful replacement. Failed writes, concurrent
edits, project switches, existing create-new targets, and migrated projects are
covered by headless regression tests. Issue #7 now has periodic separate drafts and guarded restoration; shutdown and explicit draft disposal still require work.

## Confirmed local fixes

- [#26 memory growth](https://github.com/doceazedo/sprinkles/issues/26): found
  three caches retaining strong handles indefinitely: baked gradients, baked
  curves and expanded particle meshes. Each changed curve/gradient/mesh/count
  could pin another asset until process exit. Preparation now releases baked
  entries absent from live particle systems; the mesh cache releases entries
  absent from live mesh entities. Shared live assets remain shared. Regression
  tests exercise 128 texture edits and 127 mesh counts, actual Bevy asset
  reclamation, sharing and removal of the last owner. This fixes a demonstrated
  editing/project-change retention path. It does **not** establish that the
  upstream hour-long *idle* RAM report has the same cause or is fully resolved.
- [#30 premature culling](https://github.com/doceazedo/sprinkles/issues/30): the
  imported editor wrote authored bounds only onto the inspected emitter; the
  runtime never applied them. Runtime now applies and updates authored AABBs on
  every emitter before visibility checks, independently of the inspector.
  Additionally, the editor's culling option put `NoFrustumCulling` on the camera,
  where Bevy does not use it for mesh culling. It now marks every preview emitter
  mesh, preserving other scenes. CPU regressions exercise multiple emitters,
  live edits, selection changes and both option states. Bounds remain authored
  envelopes: moving world-space trails still require suitable bounds or disabled
  preview culling; this is not automatic GPU bounds tracking.

The source review also found and guarded an independent crash path: runtime
`create_particle_material_from_config` still has `todo!()` for `CustomShader`.
The imported material menu offered that option. Our editor now removes that
unsupported choice and rejects custom-shader RON before replacing/loading the
project, using an ordinary error toast. A regression checks both clean and dirty
current projects, handle/asset/selection/path preservation, unchanged source files
and no cache-history changes. Startup/cache loading uses the same guard. This
protects the editor; custom shader rendering itself remains unimplemented and
programmatic runtime users must not construct that unsupported material.

Review of the final probes also exposed a startup-readiness risk: the original
viewport binder queried only newly added cameras. A retained UI scene arriving
on a later frame could miss that window and leave the preview unbound. Binding
now retries against the existing editor camera until its container is available.
A regression delays the container past the camera-creation frame. Failed probes
are retained separately and are not acceptance evidence. An initial black
capture during shader warmup is also not evidence of a persistent blank preview;
later frames were visually checked and render the floor and effect.

Prerequisites were the completed same-window host, existing runtime caches and
Bevy's normal asset/visibility lifecycle. Acceptance requires the regressions,
strict workspace checks and native original-widget/shared-window flows. Exact
validation evidence is recorded below after execution.

## Remaining work worth adopting

| Issue | Local assessment and priority |
| --- | --- |
| [#7 autosave](https://github.com/doceazedo/sprinkles/issues/7) | Partially addressed: periodic separate drafts and a guarded recovery list exist for committed changes. Orderly shutdown and visible draft disposal remain unfinished; switching to Zone retains the in-memory project but is not itself disk recovery. |
| [#26 idle memory](https://github.com/doceazedo/sprinkles/issues/26) | Keep open locally beyond the cache fix. Needs an instrumented long idle soak and repeated project/open/close runs, separating CPU assets, GPU allocations and process RSS. Maintainer also observed growth with compute and sorting disabled. |
| [#27 per-instance controls](https://github.com/doceazedo/sprinkles/issues/27) | Relevant before gameplay VFX. Mutating the shared asset changes all users; that is not an independent intensity/count override. Design bounded per-instance controls once an actual spell/ambient-effect slice requires them. Existing stop-emitting control helps but is not a full override API. |
| [#40 texture animation](https://github.com/doceazedo/sprinkles/issues/40) | Missing feature, useful for spell flipbooks. Add atlas/frame animation when an actual effect requires it; avoid per-frame asset-path changes. |
| [#41 sub-emitter scale](https://github.com/doceazedo/sprinkles/issues/41) | Missing scale inheritance; current config supports velocity inheritance. Defer until a chosen effect needs it. |
| [#37 UUID textures](https://github.com/doceazedo/sprinkles/issues/37) | Texture refs use preset/path. Revisit with Theia's stable asset catalog and packaging, rather than introducing a second identity scheme in the editor review. |
| [#36 `.sprinkles.ron`](https://github.com/doceazedo/sprinkles/issues/36) | Naming improvement, not a current loading defect. Preserve existing `.ron` compatibility. Decide canonical catalog naming with asset placement/export. |



## Existing fixes and historical reports

| Issue | Evidence in our version / remaining qualification |
| --- | --- |
| [#42 version stamping](https://github.com/doceazedo/sprinkles/issues/42) | Already present: the 0.2 migration stamps current 0.3. Added a real migrate → serialize → reload regression proving no second migration. |
| [#39 Bevy 0.19](https://github.com/doceazedo/sprinkles/issues/39) | Superseded locally by our qualified exact Bevy 0.20 port. No downgrade needed. |
| [#32 stop emission](https://github.com/doceazedo/sprinkles/issues/32) | `EmitterRuntime::set_emitting(false)` exists; simulation continues until particles finish, then emitter becomes inactive. Distinct from pause/clear. |
| [#29 emitter timing](https://github.com/doceazedo/sprinkles/issues/29) | Maintainer traced reproduction to randomized initial scale 10–40. Set equal range endpoints for matching scale. Current emitters use the same Bevy frame delta; do not change timing based on this report. |
| [#28 WASM](https://github.com/doceazedo/sprinkles/issues/28) | Runtime uses `web_time` under wasm and does not enable file watching. Our desktop editor itself requests file watching. WebGPU/browser execution is outside current Linux/Windows scope and remains unqualified. |
| [#25 Y/Z rotation](https://github.com/doceazedo/sprinkles/issues/25) | Compute samples XYZ curves; material shader applies X/Y/Z rotations. Historical fix is present. No new axis-specific native parity claim. |
| [#24 API naming](https://github.com/doceazedo/sprinkles/issues/24) | Already uses tuple `Particles3d(handle)`. |
| [#21 short-lived entity panic](https://github.com/doceazedo/sprinkles/issues/21) | Setup queues parent runtime/child attachment with `queue_silenced`, tolerating parent removal. Existing mitigation retained. |
| [#19 audio dependency](https://github.com/doceazedo/sprinkles/issues/19) | Runtime uses Bevy with default features disabled; no new audio feature from Sprinkles. Server remains independent. |
| [#18 rotation/scale](https://github.com/doceazedo/sprinkles/issues/18) | Extraction uses emitter GlobalTransform for spawning and appropriate local/world render transforms. Present in imported runtime; actual attachment/rebase gameplay qualification still required. |
| [#17 simulation space](https://github.com/doceazedo/sprinkles/issues/17) | `draw_pass.use_local_coords` and distinct local/world spawn/render paths already exist. Moving world-space trails across Theia origin rebases remain future integration work. |
| [#16 completion cleanup](https://github.com/doceazedo/sprinkles/issues/16), [#13 finished cost](https://github.com/doceazedo/sprinkles/issues/13) | `Finished`, `despawn_on_finish`, inactive emitter gating and extraction skip exist. Preview deliberately retains/restarts its effect. Gameplay must select cleanup policy; source presence is not an MMO performance test. |
| [#14 Vulkan flicker](https://github.com/doceazedo/sprinkles/issues/14) | Current source has GPU sorting; our Bevy 0.20 preview already passes Vulkan validation. Do not equate validation-clean with proof that every fading/transparency artifact is gone. Keep visual fade/sort checks in native qualification. |
| [#12 Windows escaping](https://github.com/doceazedo/sprinkles/issues/12) | Generated `include_str!` already uses raw literals. Native Windows build/execution still unverified. |
| [#10 file paths](https://github.com/doceazedo/sprinkles/issues/10) | Paths are not unconditionally prefixed with the editor data directory. Existing native checks open/save owned absolute external files. |
| [#9 position ignored](https://github.com/doceazedo/sprinkles/issues/9) | Setup uses `insert_if_new` for the owner's transform; extraction uses GlobalTransform. Existing implementation retained. |
| [#8 Windows startup](https://github.com/doceazedo/sprinkles/issues/8) | Shaders/textures embedded; ported WESL compiles/runs on Linux Vulkan. Windows/GPU-specific startup still needs native qualification. |
| [#5 coordinate editing](https://github.com/doceazedo/sprinkles/issues/5) | Existing vector/native input UI retained. Treat Blender-style multi-component manipulation as an enhancement, not a crash fix. |
| [#4 Mask crash](https://github.com/doceazedo/sprinkles/issues/4) | Reflection variant construction already supplies primitive fields, including Mask's f32 cutoff; dedicated cutoff editing exists. Historical missing-field mitigation is present. |

## Recommended order

Finish the confirmed fixes and their checks; add particle draft recovery; measure long-session memory; then qualify placement,
packaged assets, rebasing and per-instance controls with the first gameplay VFX
slice. Flipbooks and inherited scale follow actual content needs. Do not treat
all upstream enhancement requests as prerequisites for using this editor.


## Validation and product scope

- 185 focused Rust tests plus three runtime doctests pass: all 184 tests in
  `build/sprinkles-review/upstream-issues-tests-v4.log`, and the added delayed
  viewport-binding regression in `upstream-issues-readiness-tests-v6.log`.
  The 184 comprise five runtime, thirteen loader, seven editor, 55 zone and
  104 zone-editor tests. The additional test brings editor tests to eight.
- `cargo fmt --check`, `git diff --check` and denied-warning workspace/all-target
  Clippy pass. Final Clippy: `upstream-issues-clippy-v5.log` in that directory.
- Full integrated-host functional acceptance passes at
  `build/sprinkles-upstream-issues-workspace-20261110-v2/qualification.json`:
  source widget editing/save, exclusive cameras/layers, pause/project retention,
  blocked zone commands and exact document/source/revision/history preservation.
  This build precedes the small viewport-readiness correction. Its editor hash is
  `6c1f321707cd7bf335ef866c23f92bb224561066d38b8b0d401c3df3876c3bef`.
- Final integrated-host smoke after that correction passes at
  `build/sprinkles-upstream-issues-integrated-20261110-v1/qualification.json`:
  actual rendered scene, six emitters on layer 1, exclusive cameras, pause and
  return/re-entry retention, untouched source/effect/document/history/revision.
  Exact editor SHA-256:
  `72a22cf1bf535221fb011aef55f0cbd0f7030cce9c0359a0db776e6cf6638acf`.
- Both native integrated runs use Linux AMD Radeon RX 7900 XTX / RADV NAVI31,
  Vulkan with the Khronos validation layer, 3440×1440 and zero GPU diagnostics.
  This is not hour-long memory, Windows, gameplay VFX or MMO-budget acceptance.

The user now prioritizes the integrated workspace exclusively. Runtime and UI
source are shared with the retained standalone host, but standalone-specific
polish/qualification is no longer an acceptance gate. Earlier standalone widget
probes remain historical/supplemental; the v4 standalone colour-field timeout
is not a pass and was not pursued after this scope correction. The separate
launch option is to be removed as part of the next requested UI refinement.
