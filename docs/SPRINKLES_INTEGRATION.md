# Sprinkles authoring integration (P5-17)

The accepted editor styling migration is recorded in `ZONE_EDITOR.md`. This
slice reuses the actual Sprinkles runtime and editor, pinned at
`0e978f5a9afc6b456b3693c8761e73a6c7e155bd`, instead of recreating its particle
authoring controls. Source, asset licenses and file hashes are under
`third_party/sprinkles/`.

The prerequisites are the accepted Linux authoring shell and the adopted exact
Bevy 0.20.0 engine. Upstream targets Bevy 0.19.0. Both imported crates use our
workspace engine; introducing a second Bevy engine or downgrading Theia is not
part of this slice. Avian remains pinned independently and unchanged.

Upstream issue audit and local cache/culling fixes: see
[SPRINKLES_UPSTREAM_REVIEW.md](SPRINKLES_UPSTREAM_REVIEW.md), including remaining
autosave, idle-memory and gameplay integration work.

## Current use: original UI in Theia's window

Build the zone editor and optional companion:

```sh
cargo build --locked -p theia-zone-editor -p bevy_sprinkles_editor
```

Choose **File → Particles workspace…**. The existing window displays the original
Sprinkles project selector, sidebar, outliner, inspector, property/attribute
editors, settings, playback controls and particle preview. **Zone** in its top
bar returns to the retained zone workspace. The particle project, unsaved data
and manual pause state remain available when re-entering. Ctrl S belongs to the
active workspace. Zone commands, source data, revision and undo/redo stay intact
while editing a particle project. Effects save as original Sprinkles RON assets. The zone editor can author particle references and explicitly preview effects
saved in its local library. World-package export and client playback remain pending.

Product work now focuses on this integrated workspace. The retained standalone
host shares the same runtime/editor library and can remain an internal diagnostic
tool; it is not a separate product/acceptance target. The separate companion
launch option is removed from the normal File menu.

The host uses `ParticleAuthoringPlugin` and `AuthoringHost::embedded(1)`, not a
second application/window. Original authoring updates and zone updates have
mutually exclusive activity gates. Root visibility, native focus/cursor ownership,
window title, camera activation and preview pause are managed on transitions.
Return allows one active frame for original blur/commit handlers. The zone camera
has an explicit marker; source cameras no longer break single-camera queries.
Theia's broad native-widget styling excludes Sprinkles-owned buttons and inputs.

Preview floor, light, particle meshes and bounds use render layer 1; the retained
zone uses layer 0. Particle mesh creation honors an owner's explicit render layers
while retaining default-layer behavior when absent. No global origin conversion,
world collision, navigation or authority semantics change. The embedded plugin
ignores Theia's CLI arguments, shares its default window UI camera and preserves
the host clear color/title. Editor asset loading permits user-selected external
paths, as the standalone source host does; server/client settings are unchanged.

### Linux acceptance

Both `build/zone-sprinkles-embedded-20261110-v1/qualification.json` and
`build/zone-sprinkles-embedded-far-20261110-v1/qualification.json` pass on
AMD Radeon RX 7900 XTX / RADV NAVI31 / Vulkan, 3440×1440, with zero diagnostics.
The exact zone binary SHA-256 is
`79204a1770fca52b7221cec4f0ac6233736edb6961e379cb7515a2968a841a8d`.

Native checks cover the retained zone/menu/property flows, entering the original
particle workspace, six emitters isolated to layer 1, exclusive scene cameras,
blocked zone redo while in particle mode, exact particle-count editing, particle
save without changing zone/source/revision/history, return and re-entry with
project and manual pause retained. The source files remain untouched until their
own explicit Save. These are functional checks, not a pixel-perfect comparison.

`build/sprinkles-embedded-standalone-20261110-v1/qualification.json` additionally
passes preview, playback, scrolling and original curve/gradient/colour value
editing/save in the standalone host at 1366×768, also with zero diagnostics.
All 177 focused Rust tests and three runtime doctests pass; formatting and
workspace/all-target Clippy with denied warnings pass. CPU regressions cover
real zone redo suppression/restoration at local/far origins, root/focus/camera/
title/pause restoration, original widget-style ownership and runtime render-layer
inheritance/default behavior. Dependency inspection still finds no Sprinkles,
render or window dependency in the headless server.

Reproduce the shared-window checks with `tools/qualify_zone_sprinkles.py
--workspace --case local` and `--case far`, each with a new `--output` directory.
All native data directories and fixtures are isolated under those owned outputs.
Windows, the complete upstream preset/widget matrix, world asset placement/client
integration, gameplay VFX/audio and MMO cosmetic-load budgets remain unqualified.

## First host integration (retained history)

Build the companion binary alongside the zone editor:

```sh
cargo build --locked -p theia-zone-editor -p bevy_sprinkles_editor
```

The zone editor's **File → Particle editor…** command opens the actual retained
Sprinkles UI in a separate authoring window. Repeated requests retain one child
process. Normal workspace input/modal/preview guards apply. Starting the tool
does not edit or save the zone. The particle window owns its RON projects and
must save them explicitly. Closing the zone editor leaves that window alive,
so it can finish saving its work.

Projects, examples and preferences default to `~/.theia/particles`; the
`THEIA_PARTICLE_EDITOR_DATA_DIR` override supports isolated native checks.
Existing `~/.sprinkles` preferences/projects are not migrated or overwritten.
The tool can also open an existing effect directly:

```sh
target/debug/theia-particle-editor /absolute/path/to/effect.ron
```

The library extraction is intentional: `bevy_sprinkles_editor::standalone_app`
hosts the original plugins, widgets, inspector bindings, preview and RON
load/save behavior. The companion executable is a thin entry point. This is
the first integration boundary, not a same-window particle workspace.

## Compatibility adaptations

- Flat pointer events, native button/focus components, explicit native text
  inputs and Bevy 0.20 scene composition preserve the original UI behavior.
  A bounded main-row layout keeps panels and the preview within the window.
- Shader buffers retain encase/WGSL storage encoding; replacement writes have
  a round-trip test. Extracted fallback textures target the render app explicitly.
- Original shaders use WESL imports, paths and conditional attributes. PBR alpha
  discard follows the Bevy 0.20 signature. This follows the
  [official shader migration](https://bevy.org/learn/migration-guides/0-19-to-0-20/).
- Embedded asset paths use the extracted library's namespace. The selected code
  license is MIT; separate upstream asset licenses are retained.
- Immediate and retained Bevy gizmos both produced Vulkan synchronization
  hazards for preview bounds on RX 7900 XTX. Standard Bevy line meshes/materials
  display bounds and collider outlines instead. Shared assets remain bounded;
  selection, transforms, disabled colors and cleanup have a CPU regression.
  The UI camera excludes scene overlays. No custom renderer is introduced.
- Imported legacy BSN syntax/Interaction remains scoped compatibility debt.
  Deprecated allowances and the scene macro's generated unit-binding lint are
  confined to the imported editor, without weakening Theia's workspace checks.

## Acceptance and limits

Final direct-source validation passes at
`build/sprinkles-direct-native-20261110-final-v1/qualification.json`:
six-emitter Magic Puff preview, pause/resume, loop changes preserving the asset,
real inspector scrolling, particle-count entry through native text input,
exact expected in-memory modification and explicit Ctrl S RON save. Vulkan
validation reports zero diagnostics on AMD Radeon RX 7900 XTX (RADV NAVI31),
Linux, 1366×768. Validation-host SHA-256:
`5205585100a6c8fb43381eb8c1c652b087abbf6f78f9710e395cd9d906ef11e8`.

`build/zone-particle-host-20261110-v2/qualification.json` passes the actual File
command, single-child behavior, clean close/return and exact zone/revision/history
and source-file preservation, alongside the retained native menu/property flows.
The zone window is 3440×1440, on the same GPU/driver/backend with zero diagnostics
across both applications. Zone binary SHA-256:
`e578c063d4d9b5295cb5af5fadda31fa0c7cc40be020eebbbbae52af20ed5a96`.
Companion binary SHA-256:
`8d25fb3ca4d49968d7350add1e968f45d01bd5a9ed952a0e1dfd6ec11f8ebb02`.

All 172 focused Rust unit/integration tests and three runtime doctests pass;
formatting and denied-warning workspace/all-target Clippy pass. Dependency
inspection confirms no Bevy 0.19/0.20.1 modules, no removed package versions,
and no Sprinkles/render/window dependency added to the server. First probe
failures are retained as failures: shader/layout/native gizmo issues were fixed;
the host v1 failure was a harness assumption about worker-thread child ownership.
The harness now discovers children of all owned process threads and cleans up
its known child. Failed probes do not establish acceptance.

Reproduce after building the companion and validation host:

```sh
cargo build --locked -p theia-zone-editor -p bevy_sprinkles_editor --bins --example theia_validate
python3 tools/qualify_sprinkles_port.py --output build/particle-port-check
python3 tools/qualify_zone_sprinkles.py --output build/particle-host-check --case local --particles
```

Remaining work is the same-window workspace, particle asset discovery/reference
and placement, and then client presentation integration. A particle preview is
cosmetic authoring evidence: it does not establish authoritative spell effects,
world collision/navigation behavior, protected critical telegraphs, audio,
network replication or MMO population/cosmetic-load capacity. Full P2-13 remains
separate. Windows build/native rendering and the complete upstream preset/widget
matrix still require qualification.

## Original widget commits — 10 November 2026

`build/sprinkles-original-widgets-20261110-v6/qualification.json` extends native
validation to the original curve, gradient and nested colour popovers. It changes
the scale curve maximum through native input, removes its preset name as
upstream custom-curve behavior requires, edits/restores a gradient stop and
changes/restores a colour through the RGB/Hex selector. Full expected asset
comparisons pass after each operation; the source file stays untouched until
Ctrl S. Preview, playback, scrolling and clean close still pass, with zero Vulkan
diagnostics on the same RX 7900 XTX / RADV NAVI31 at 1366×768.
Validation binary SHA-256: `1a45e0414cd7c9e3dfc496f95926c91ea898971db95027485d9069235da47390`.

The native test exposed an existing range-handler hierarchy assumption: the
range marker is on the vector root, with fields as direct children, whereas the
handler expected another vector layer. Synchronization and commit now follow
the actual hierarchy. Valid changes commit once, clamp existing points to the
new range and mark the curve custom. Invalid/inverted/non-finite values restore
the canonical input without committing or panicking. A CPU regression exercises
this tree, one-time commits, clamping and rejection. This is a targeted source
functionality fix, not a change to the effect format.

The earlier widget probe failures are not acceptance: first two probes used
incorrect harness assumptions about nesting/visible margins; v4 exposed the
range bug, and v5 assumed Hex rather than the actual default RGB mode.
Run `tools/qualify_sprinkles_port.py --widgets` to include these checks.

All 16 runtime/editor tests and three runtime doctests pass for the widget fix,
along with formatting and denied-warning workspace/all-target Clippy. Retained
zone/host behavior is unchanged; earlier host proof applies to its recorded binary.

## Same-window preparation: activity scheduling

`hosting::AuthoringHost` defaults to active for the retained standalone tool.
All original editor Update registrations share `AuthoringUpdate`, whose run
condition reads that resource. A native-keyboard regression observes the actual
SaveProjectEvent: Ctrl S belongs to Sprinkles while active, produces no particle
save while inactive, and works again after reactivation. No test writes projects
or accesses a user data directory. All four editor tests, formatting and strict
workspace/all-target Clippy pass.

The original native preview/playback/count/range/gradient/colour/save flow also
passes after this scheduling refactor, with zero Vulkan diagnostics:
`build/sprinkles-authoring-gates-20261110-v1/qualification.json`, validation hash
`e08d79a3cb121bde1a2eda56611321f53ab7253f0b319106b5858c76f13a1fea`. The companion remains usable.

This is preparation, not yet an embedded workspace. The host must still switch
UI roots, clear focus/cursor ownership, suspend its own authoring input, manage
preview/camera lifetime and isolate render layers before offering a same-window
mode. Startup title/UI/scene setup and harmless text presentation post-update
work are not governed by the Update set.

## Particle project save integrity

Particle Save remains explicit: it first commits the active valid text field,
then writes a same-directory temporary file and replaces the destination only
after the write succeeds. The project stays dirty on failure or if it changes
while the write is in progress. Save As assigns its path after success; creating
a project cannot overwrite an existing file. Migrated 0.2 projects remain
marked dirty until saved. This protects explicit saves; recoverable drafts and
recovery after an interruption are still pending. Native Windows execution and
filesystem-specific behavior have not been qualified.

The separate draft store keeps independently identified session snapshots,
uses create-only and atomic replacement writes, bounds each file, preserves
corrupt drafts and refuses additional drafts at its configured count limit.
The project menu offers a recovery list, loaded in the background. Drafts are
validated with the usual migration and material checks and restore as dirty
projects without assigned paths. Restoration rejects unsaved or actively edited
projects. While the particle workspace is active, committed changes are captured
in the background at five-second intervals. This is best-effort recovery: edits
still in text fields and abrupt termination are not guaranteed to reach a draft.
When Bevy delivers an exit message before the final schedule, the editor queues a
final committed snapshot and waits a bounded time for writes. Leaving the
workspace or switching projects queues a final committed
snapshot; a full queue or write error is reported, and queued work is not yet
durable until the writer completes. Drafts remain after an explicit Save;
the recovery list provides a two-step discard control for each draft. The
storage API removes only the selected draft without following a symbolic link. Its count check assumes a
single editor process; concurrent processes are not yet coordinated.

A bounded background writer serializes snapshots for each editing session and
reports write errors. Accepting a queued snapshot does not mean that it has
reached storage. Exit flushing is bounded and is not a guarantee against forced
termination, power loss or storage errors.

The exit handler also captures the outgoing and incoming committed projects if a
project switch occurs after the periodic update in the same frame. It drains
completed errors first, allowing an outgoing snapshot to retry when storage has
recovered. Headless schedule and failure tests exercise both cases. The bounded
flush and uncommitted-field limitations above still apply.

## Runtime material admission prerequisite — P2-13 / P5-17

Custom shader materials are not implemented. The Bevy asset loader rejects
them before a particle system asset is admitted, while programmatically supplied
assets cannot start until all emitter materials are supported. A live emitter
that switches to an unsupported material stops rendering and can resume after a
supported material is restored; its resized buffers are synchronized first.
Other emitters remain independent. No substitute material is silently rendered.

The real Bevy loader and multi-emitter mutation/recovery are covered headlessly.
A windowless 256×256 render-target smoke test with no DISPLAY or WAYLAND_DISPLAY
ran on AMD Radeon RX 7900 XTX, RADV NAVI31, Mesa 26.2.6, Vulkan with the Khronos
validation layer explicitly enabled: no unexpected GPU diagnostics. Its one
unsupported-material error is intentional. This does not qualify Windows,
visual appearance, zone placement, gameplay VFX or performance budgets.

## Checked particle/trail arithmetic — P2-13

Before allocating, the runtime checks the particle-by-trail slot count, history
entry count and platform-sized sub-emitter buffer calculation. Non-finite,
negative or unrepresentable active trail-history timing is refused. Invalid
programmatic systems do not start; invalid live emitters skip resizing, mesh
replacement, uniform writes and render extraction, and their draw material is
suppressed until repaired. Headless regressions cover overflow and recovery.

The windowless 256×256 RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan test
exercised unsupported shader and overflowing trail transitions with explicit
Khronos validation and zero unexpected GPU diagnostics. Both rejection logs
are intentional. These are arithmetic checks, not memory limits, mesh-shape
validation, untrusted-content admission, gameplay budgets or a performance claim.

## Degenerate particle mesh normals — P2-13

Flat and extremely thin cylinders now calculate side normals without division
by zero or f32 slope overflow. Degenerate prisms use finite zero normals where
a direction cannot be determined. Normal-sized cylinder slopes retain their
expected direction. Headless tests check finite positions/normals, valid prism
triangle indices and the existing mesh-cache lifecycle. A windowless RX 7900 XTX
/ RADV / Mesa 26.2.6 Vulkan smoke run with Khronos validation switches through
flat-cylinder and degenerate-prism emitters with no unexpected GPU diagnostics.
This does not validate arbitrary mesh parameters or establish VFX memory budgets.


## Degenerate particle spheres — P2-13

Sphere meshes use unit-sphere normals and scale their positions by the authored
radius, avoiding non-finite normals at radius zero. Segment and ring counts are
clamped to the minimum geometry accepted by Bevy (three and two). Tests cover
zero and tiny radii, zero resolution, finite attributes and valid indices; the
windowless Vulkan smoke test also transitions to a zero-radius, zero-resolution
sphere. This does not validate arbitrary dimensions or limit memory usage.


## Quad mesh arithmetic — P2-13

Before construction, quad subdivisions, base vertex and index counts, and
replicated vertex and index counts are checked for the integer overflow relevant
to their construction and indexing. Invalid live edits suppress the affected
emitter until corrected. Runtime regressions and the windowless Vulkan smoke test
cover rejection and recovery. This is not a memory cap; representable inputs may
still exhaust memory. Other mesh types are not covered by this quad check.


## Sphere mesh arithmetic — P2-13

The Bevy UV-sphere builder's intermediate count, vertex and index counts and
replicated mesh counts are checked for integer overflow before construction.
Regression tests cover extreme subdivision counts, particle replication and
minimum resolution; windowless Vulkan exercises invalid live edits and recovery.
Representable inputs can still exhaust memory; cylinder, prism and trail mesh
arithmetic remain outside these quad and sphere checks.


## Cylinder mesh arithmetic — P2-13

Cylinder side, optional cap and replicated mesh counts are checked before
construction. Tests cover extreme radial and ring counts, cap-dependent vertex
counts and minimum subdivisions; windowless Vulkan covers invalid live edits and
recovery. Representable inputs can still exhaust memory; prism and trail mesh
arithmetic remain outside the checks.


## Trail mesh arithmetic — P2-13

Tube and flat/crossed ribbon section, vertex, index and replicated mesh counts
are checked before construction. Tests cover extreme counts and minimum
subdivisions; windowless Vulkan covers invalid live edits and recovery.
Representable inputs can still exhaust memory; prism and cuboid mesh arithmetic
remain outside these checks.


## Prism and cuboid mesh arithmetic — P2-13

Runtime admission checks prism subdivision, vertex, index and replicated mesh
counts and cuboid replication counts. Tests compare expected counts to actual
generated geometry and exercise overflowing inputs without allocating them;
windowless Vulkan exercises invalid live edits and recovery. These checks cover
the current mesh variants but are not memory budgets, general validation of
dimensions, or protection for direct external use of the public mesh cache.


## Protect unsaved projects when switching — P5-17

Opening or creating another particle project commits a valid focused text field,
then asks before discarding dirty work. Cancel leaves the current asset and file
untouched; a changed asset or path invalidates a pending confirmation, and a save
in progress blocks the switch. Creating a project does not create its destination
file before confirmation. The delete-item confirmation is tied to its own dialog
so another dialog action cannot trigger it. Changes still require the user to use
Save; the existing new-project initial write behavior is retained. Migrated files already marked dirty by the editor also prompt. Bundled examples
require Save As, but do not prompt on switching unless their content has changed.

Headless tests cover focused text, stale changes, cancel, dialog identity and New
without a write. Forty-one particle-editor and 122 zone-editor tests, formatting
and denied-warning workspace/all-target Clippy pass. A windowless 512×384 UI
layout test and the runtime transition example ran on RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 Vulkan with explicit Khronos validation and no unexpected diagnostics.
These checks did not visually inspect pixels or qualify Windows. The examples now
return their Bevy exit status; earlier runtime logs were checked for their explicit
success message after discovering that the old harness discarded failure status.


The New project action is also scoped to its own dialog. If another modal is
open, the New button cannot start a hidden form; a different dialog's action
cannot create a project from stale form state. Headless regressions cover both.


The original serialized example content is tracked by asset identity at both
startup and later Open. Comparing current content avoids a discard prompt for
an untouched example without weakening protection for an edited example. A
missing or unserializable baseline takes the conservative confirmation path.


## Zone particle placement data — P2-13

Prerequisites: the existing validated zone document/transaction history and the
integrated Sprinkles runtime/authoring workspace. Zone schema 8 adds up to 256
particle placements, each with a shared authoring ID, name, stable `ContentId`,
checked global f64 anchor, yaw and uniform scale. The ID is an unresolved
catalogue key; no filesystem location is embedded. Only the anchor is bounded,
because effect-specific visual extents are not yet available. Put/delete edits
participate in validation, revision checks, atomic rollback and undo/redo. Older
zone documents load with no placements; older binaries do not read schema 8.

The existing world exporter rejects any zone with particle placements before
creating an output directory. This is intentional until effect and texture
dependencies can be packaged and verified. The editor now provides reference creation, selection and inline attribute controls;
clients do not load or render them. No MMO particle-count or GPU memory budget is implied by
the authoring document bound.

Acceptance for this slice: far-origin coordinate roundtrip, invalid keys and
coordinates, duplicate authoring IDs, schema compatibility, undo/redo, atomic
rollback and explicit export rejection. Fifty-eight zone tests and 123
zone-editor tests pass, with the final shared-ID validation also exercised by a
repeat of the export test. Formatting and denied-warning workspace/all-target
Clippy pass. No rendering changed.


Particle records participate in the same name lookup, rename, f64 translation,
duplicate and explicit terrain-grounding commands as other placements. A
far-origin regression checks the complete transaction, effect-ID preservation,
conflicting IDs, rollback and undo/redo. These commands do not yet provide editor
controls or resolve an effect asset. Fifty-nine zone tests and denied-warning
workspace/all-target Clippy pass; no rendering changed.


## Particle reference controls — P2-13 / P5-17

The Add menu creates an unresolved particle reference at the player spawn,
selects it in the left explorer and exposes its name, stable effect ID, position,
yaw and uniform scale in the normal inline inspector. The Particles filter,
rename, duplicate, ground, frame, delete and undo use the shared document
commands. The initial `vfx.unassigned` ID must be replaced with the intended
content ID; asset catalogue resolution remains pending. The currently open project
can be previewed temporarily at the selected reference. The viewport gizmo remains
hidden for these references; an editor-only anchor
marks their position. World export still rejects them.

CPU tests exercise creation, selection, filtering, exact far-origin edits, invalid
effect IDs and undo. All 126 zone-editor tests, formatting and denied-warning
workspace/all-target Clippy pass. A native 3440×1440 capture of the selected
particle inspector on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan with
Khronos validation reported zero GPU diagnostics. The capture does not validate
particle rendering, packaging, client playback or Windows.


## Particle reference anchors — P2-13

Unresolved references have a small purple editor badge at their checked local
position, highlighted orange when selected. Clicking the badge routes selection
to the normal particle inspector. It has no collision, casts no shadow, is hidden
in offline playtest, and does not represent a running effect. The shared badge
mesh and materials are reused; document edits, selection, deletion and undo
update the entity without accumulating old anchors. Particle workspace rendering
uses a separate layer and camera. Transform gizmos, asset resolution, packaging
and runtime playback remain pending.

Headless tests check a far-origin anchor, selection material, playtest visibility,
movement, deletion and undo. All 127 editor tests, formatting and denied-warning
workspace/all-target Clippy pass. A native 3440×1440 selected-anchor capture on
RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan with explicit Khronos validation
reported zero GPU diagnostics. It verifies rendered placement and inspector, not
physical mouse picking, particle playback or Windows.


## Temporary open-project preview at an anchor — P2-13

For a selected particle reference, the inspector's three-dot menu offers
**Toggle open-project preview**. It renders the currently open Sprinkles asset
at the selected anchor, including its initial transform, and uses the retained
Sprinkles one-shot restart behavior. This is an explicit temporary preview: the
open project is not associated with the authored effect ID, the zone is not
changed by toggling, and no file is saved. Position/yaw/scale edits update the
preview. It is removed while the Sprinkles workspace or offline playtest is
active and returns when authoring resumes; changing the open project or zone
document, deleting the reference, or toggling it off stops the preview. Only one
such preview is active.

Tests cover far-origin transforms, shared asset identity, unchanged zone state,
toggle, suspension, project/document changes and one-shot restart. Forty-three
particle-editor and 130 zone-editor tests, fmt and denied-warning workspace/
all-target Clippy pass; the preview-specific tests were rerun after the final
status-text change. Native 3440×1440 captures on RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 Vulkan with Khronos validation visibly show the retained rain and
Magic Puff burst effects at the anchor, with zero unexpected diagnostics. This
does not bind content IDs, package textures, enable client playback, establish
MMO budgets or qualify Windows.


On first run, the bundled explosion project loads from the configured examples
directory and its real path is cached. Previously the lookup used a path
relative to the process working directory and could silently fall back to a blank
project. A fresh isolated data-directory native run rendered the bundled demo at
a zone anchor on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan with Khronos
validation and zero unexpected diagnostics. Forty-four particle-editor tests,
formatting and denied-warning workspace/all-target Clippy pass.


## Local effect library save foundation — P2-13

An explicit editor save event maps a validated stable content ID to
`<particle data directory>/library/<ID>.ron`. It rejects the unassigned
placeholder and symbolic-link targets, commits a valid focused field through the
existing save flow, and uses the existing asynchronous atomic writer. A new
binding is create-only; a project already saved to that same library path can
update it. Failed writes retain dirty state and do not overwrite another project.
This is a local authoring library, not a world package or a client feature. The selected reference's three-dot menu now connects the save action. Assets can still contain
machine-local texture paths. Forty-six particle-editor tests, formatting and
denied-warning workspace/all-target Clippy pass.


## Save the open project to a local effect ID — P2-13

Select a particle reference, set its Effect ID, and choose **Save project to
local Effect ID** from its inspector menu. The currently open Sprinkles project
is explicitly saved to the local authoring library using the existing atomic
Save flow; the project then uses that path for later Save operations. Save
completion or failure is shown in the zone status bar. The placeholder ID is
rejected, and a new binding cannot overwrite an existing different project.
The separate local preview action can load that ID for the reference; this save
operation does not package dependencies or make them available to clients.

A fresh isolated-data native test saved the bundled explosion to
`vfx.campfire.ron`, verified the source and saved bytes matched, and reported
zero unexpected Vulkan diagnostics on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6
with Khronos validation. All 131 zone-editor tests, formatting and denied-warning
workspace/all-target Clippy pass. Native mouse activation and Windows are not
qualified by this automated capture.


## Preview a local effect ID — P2-13 / P5-17

Select a particle reference and choose **Toggle local Effect ID preview** from
its inspector menu. The editor resolves the validated ID to the local library
and loads its saved Sprinkles asset through Bevy, independently of the open
particle project. The single explicit preview follows the placement transform,
uses the shared one-shot loop behavior, and suspends in the particle workspace
and playtest. Changing the reference ID or document stops it. Toggling it off
and on requests a reload; an already cached asset can remain visible briefly
while that reload completes. Missing and malformed local assets report failure.

A real-file loader test covers loading, reload, changed IDs, missing files and
malformed RON. An automated 3440×1440 native run on RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 Vulkan with Khronos validation loaded the saved effect while a
different project was open, created runtime emitter entities, and had zero GPU
diagnostics. The captured frame was between bursts, so visual burst appearance
was not verified by that capture. Physical menu activation and Windows remain
unqualified. Library files and their texture paths are local to the authoring
machine: package export still refuses particle references and clients cannot yet
load them. Preview state is not persisted.

## Local export dependency preflight — P2-13

Before rejecting a zone with particle placements, the exporter checks each unique
Effect ID against the local library. It bounds each effect file to 4 MiB, uses
Sprinkles' version migration, rejects custom shader materials, and checks that
all referenced external textures in the five standard material slots exist.
Built-in preset textures need no external file. The diagnostic reports a missing
or invalid dependency, or counts distinct resolved external texture paths. This
is an authoring aid and a prerequisite for packaging: existence is not image
validation, dependency hashing, portability, or a rendering budget. Export still
refuses every zone with particle placements, including when preflight succeeds.

## Particle catalog decoder foundation — P2-13

The headless network crate has a preliminary schema-1 particle catalog decoder.
It accepts 1–256 placements with bounded local Y-up positions, yaw and scale, and
1–256 uniquely named effect descriptors. Every effect must be referenced; each
has a byte count up to 4 MiB and SHA-256, with a 32 MiB total and a 256 KiB
catalog limit. The fixed effect filename is `effect-<content-id>.ron`; the
verification helper checks regular files, lengths and hashes. IDs use the
existing content-ID rules. It rejects unknown fields, duplicate identities,
unassigned references and out-of-range transforms.

This decoder is deliberately not part of a world manifest or client loading yet.
It does not parse effects, validate textures, protect against files being changed
after checking, or establish GPU resource budgets. Export remains blocked until
those admission and rendering steps are implemented and qualified. Local
positions must be produced through the existing checked global-to-local frame
conversion; this contract does not assign authority or collision to effects.

Export preflight constructs an in-memory draft catalog from deterministic RON
snapshots with editor asset-folder metadata removed, sorted unique effect IDs and
placement IDs, then passes it to the headless decoder. External PNG references
are copied into bounded in-memory variants and rewritten to hash-derived paths
relative to the package. Positions use the zone's checked global-to-local frame
before conversion to f32. The draft is discarded: no catalog or effects are
published, and external texture dependencies still need a package format.

## Runtime allocation counts — P2-13

The local Sprinkles port exposes checked per-emitter counts for replicated mesh
vertices and indices, particle-buffer slots and trail-history entries. A zero
particle amount still counts one base mesh, because the runtime constructs that
mesh before replication. Counts share the runtime's existing overflow checks;
they do not include every CPU/GPU allocation, texture, shader cost, emitter
instance or cache behavior. They are a prerequisite for a package admission
policy, not a memory cap or a measured capacity. Generated-mesh tests compare
counts for all seven mesh types; runtime tests cover trails, zero amount and
overflow.

Export preflight totals those counts across all placements, including repeated
instances of a shared effect, and rejects invalid allocation arithmetic. Mesh
sharing can lower actual allocation; texture, material, renderer and other costs
are not included. The count is diagnostic only and is not an admission budget.

The preliminary catalog can now be bound by world manifest schema 4, whose
fingerprint includes its byte length and SHA-256. The headless loader can use
that identity without particle I/O; visual verification rejects schema 4 until
the client implements particle admission and playback. No particle-bearing
package is published by the zone exporter yet. The byte-level fingerprint
contract is in [CONTENT_PACKAGES.md](CONTENT_PACKAGES.md).

Particle export preflight and preset snapshots also reject non-finite mesh
dimensions for each supported mesh shape. This narrow check does not validate
all material, transform or simulation parameters, and it does not bound finite
but excessively large dimensions or allocations.

The catalog can additionally describe up to 64 PNG texture variants by SHA-256,
length and sRGB/linear role, with fixed hash-derived filenames. The preliminary
limits are 16 MiB per file, 128 MiB total encoded bytes and 256 MiB total
base-level RGBA; common PNG header checks are shared with world images. The
helper verifies regular files, hashes and headers, but does not decode PNGs or
prove that every effect reference matches the catalog. The Sprinkles material
loader still needs package-context path resolution, role-aware image loading and
independent reference validation before these descriptors can be used for client
admission or export.

The client-side `theia-vfx` helper can pre-request those PNG variants through
Bevy's standard `AssetServer`, using sRGB for color and linear for data, with
validated package-relative prefixes. A real-file test confirms that later normal
Bevy loads reuse the same handles with the requested formats. The caller must
verify catalog and image files, request the variants before any other load of
the same paths, and keep the handles alive. This helper is not yet connected to
world loading or Sprinkles material creation; the world client still rejects
schema 4.

## Reusable inspector schedule — P5-17

The original Sprinkles inspector, binding and widget systems now have a separate
host-controlled update set. The full particle workspace enables both this set
and its own workspace systems. An embedded host can enable the reusable UI set
without starting Sprinkles' workspace camera and project shortcut systems; the
zone editor still keeps this mode off. A headless test exercises a real original
checkbox under the separate set. This is a scheduling prerequisite, not yet an
inline particle inspector or a qualification of simultaneous keyboard, popup,
scroll and focus ownership. Native interactive qualification was interrupted at
the user's request to stop using the display and mouse; it has not passed for
this change.

## Scoped live edit preparation — P5-17

Sprinkles inspector edits now restart only emitters belonging to the currently
edited asset, using each emitter's own seed; unrelated particle systems are not
reset. The zone preview listens for Sprinkles' structural emitter/collider
respawn events and rebuilds only when its preview uses that edited asset. The
existing shared-preview transform and explicit save behavior are retained. This
prepares direct inspector embedding; the zone inspector is not yet wired to the
original Sprinkles controls. Automated headless tests cover edit scoping, per-
emitter seeds and zone-preview rebuild. No native display or mouse was used for
this change, and its visual behavior remains unqualified.

An open project is identified as the selected local effect only when the actual
selected zone particle's validated library path and the open project path resolve
to the same existing file. This avoids using names or unrelated open handles as
identity. The open-project preview reports when that match exists. This check is
a prerequisite for the inline inspector; it does not yet switch projects or
embed the inspector.

A headless integration test also invokes the original Sprinkles field binding
while the full workspace is inactive: changing an emitter count updates the
live asset, marks it dirty and emits the existing respawn event. This validates
the binding seam, not the layout, focus handling or save routing of an embedded
inspector.
