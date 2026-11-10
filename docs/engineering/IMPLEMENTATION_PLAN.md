Sprinkles theme foundation, 10 November 2026 (P5-17): shared upstream
zinc/blue tokens, adapted button states and embedded licensed Inter now style the
palette, command bar, workspaces, content list, properties and dialogs. Focused
inputs use 1 px blue borders; inactive inputs use zinc borders. Rename uses a
compact matching dialog. The retained playtest context/settings restoration is
included. All 127 focused Rust tests, formatting and denied-warning workspace/
all-target Clippy pass. Linux RX 7900 XTX / Vulkan at 3440 x 1440 passes local/far
placement, focused keys, preview hiding, undo/redo, exact document command oracles,
Save and playtest/settings restoration, with zero GPU diagnostics. Native captures
were inspected. Native local/far invalid/corrected naming and numeric-form input checks also pass;
those dialog captures have not been visually inspected. This is a styling
foundation; collapsible sections and panel resizing follow. No particle runtime
or Bevy version change. Evidence: `build/sprinkles-review/` and
`build/zone-sprinkles-theme-native-20261110/`; provenance:
`third_party/sprinkles/PROVENANCE.md`. Keep working locally; do not push.

## 10 November 2026 — adopted Sprinkles editor UI direction (P5-17)

The user adopts Sprinkles as the concrete look, feel and interaction reference
for the full Theia editor. Adapt its native Bevy UI components where practical;
create matching controls for Theia-specific functions. Keep the existing validated
command/history, exact f64 fields, source schemas, agent interface and offline
playtest underneath the presentation. Continue local implementation and commits
until stopped; mouse, keyboard and display use are authorized. Do not push.

Reviewed upstream: `doceazedo/sprinkles` commit
`0e978f5a9afc6b456b3693c8761e73a6c7e155bd` (Bevy 0.19). Adapt to Theia's
existing Bevy 0.20 pin; no engine downgrade. MIT-selected UI provenance is in
`third_party/sprinkles/`. Particle runtime remains a candidate requiring its own
P2-13 compatibility, rebasing, budget and native GPU qualification.

Initial UI prerequisites are the retained editor command/history, standard Bevy
widgets and existing 3440 x 1440 layout. First slice: shared Sprinkles tokens,
button states and Inter typography across workspaces, palette, outliner,
properties and dialogs. Follow with resizable panels and section/inspector
interaction. Acceptance: formatting, denied-warning Clippy, retained focused
Rust tests, CPU layout and actual Linux 3440 x 1440 input/GPU checks. Planned
migration is not a claim of completed UI or particle adoption.

Placement hints and renderer guards, 9 November 2026 (P5-17 / P3-15 / P1-10):
Properties explains refused candidates. The renderer hides cached ghosts during
forms/playtest or when the tool no longer matches, independently of pointer order.
All 127 focused Rust tests, formatting, strict Clippy and build pass; long-name/
refusal layout fits. Native local/far Find/playtest/preview checks pass at
3440 x 1440 with zero Vulkan diagnostics. See [the record](../ZONE_EDITOR.md#placement-refusal-hints-and-renderer-guards--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Read-only primitive placement preview, 9 November 2026 (P5-17 / P3-15 / P1-10):
A standard Bevy ghost shows the prospective stamp: blue for valid, red for a
refused candidate. It shares pose/object construction and command validation
with actual placement while preserving source, history, redo and IDs. All 127
focused Rust tests, formatting, strict Clippy and build pass. Native local/far
3440 x 1440 preview/stamp equality, panel hiding and refused-click isolation
pass with zero Vulkan diagnostics. See [the record](../ZONE_EDITOR.md#read-only-primitive-placement-preview--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Workspace slider focus cue, 9 November 2026 (P5-17 / P3-15 / P1-10):
A pale thumb identifies the focused slider; the footer explains its keys.
All 124 focused Rust tests, strict Clippy, formatting and build pass. Native
local-origin 3440 x 1440 rendering/keys/placement pass with zero Vulkan diagnostics;
app captures were inspected. See [the record](../ZONE_EDITOR.md#workspace-slider-focus-cue--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Workspace slider keyboard ownership, 9 November 2026 (P5-17 / P3-15 / P1-10):
Workspace sliders now take focus on click. Left/Right/Home/End adjust that widget
without also editing the selection through raw keyboard polling; ordinary nudges
resume when focus clears. All 124 focused Rust tests, formatting, strict Clippy
and build pass. Native grid/size keys preserve selected source and redo at local/
far origins, 3440 x 1440, with zero Vulkan diagnostics. See [the record](../ZONE_EDITOR.md#workspace-slider-keyboard-ownership--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Primitive placement dimensions, 9 November 2026 (P5-17 / P3-15 / P1-10):
Properties has quarter-step size sliders for new box X/Y/Z, cylinder diameter/
height and sphere diameter. Preferences preserve source/history; stamps use
checked snapping/surface support and one shared edit. All 123 focused Rust tests,
formatting, strict Clippy and build pass. Native local/far shape controls,
placement, full-document command oracles, undo/redo and Save pass at 3440 x 1440
with zero Vulkan diagnostics. See [the record](../ZONE_EDITOR.md#primitive-placement-dimensions--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Application shortcuts, 9 November 2026 (P5-17 / P3-15 / P1-10):
Ctrl+N invokes shared New; Ctrl+F opens content search without framing. Modal,
preview, playtest and focus guards remain. All 119 focused Rust tests, strict
Clippy, formatting and build pass; native key delivery and full New/save/play
workflow pass at 3440 x 1440 from local/far origins with zero Vulkan diagnostics.
See [the record](../ZONE_EDITOR.md#application-document-and-search-shortcuts--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Native application controls accepted, 9 November 2026 (P5-17 / P3-15 / P1-10):
Content paging/wheel, filters, search, double-click framing and yaw/scale/radius/
guide-width sliders pass native 3440 x 1440 checks at local/far origins, including
Reset, Home/End, exact source/oracle equality and undo/redo. Vulkan validation is
active with zero diagnostics. The earlier wheel-delivery gap is closed for these
accepted runs after pre-launch synthetic-device creation. See [the record](../ZONE_EDITOR.md#native-application-controls-acceptance--9-november-2026).
Continue after local commits; desktop access authorized; no push. Historical
headless records below retain their original execution limitations.

New untitled zones, 9 November 2026 (P5-17 / P3-15 / P1-10):
New zone creates an undoable empty document. Its first Save chooses a filename;
cancel/failed writes preserve untitled state and the prior file is never written.
All 118 focused Rust tests, formatting, strict Clippy and the editor build pass.
Native New/save/playtest checks pass at 3440 x 1440 from local/far origins,
with active Vulkan validation and zero GPU diagnostics.
See [the zone record](../ZONE_EDITOR.md#new-untitled-zones--9-november-2026).
Desktop access is authorized again for native 3440 x 1440 checks; continue after
local commits and do not push. Historical headless limits below remain historical.

Reset transform fields, 9 November 2026 (P5-17 / P3-15 / P1-10):
A Reset fields button restores the open form’s exact original numbers and
sliders, retaining the form, original revision and pending redo. Apply/Cancel
remain the transaction boundary. Headless behavior/layout checks pass; native
UI/input/GPU acceptance remains pending. See [the zone record](../ZONE_EDITOR.md#reset-transform-fields--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Property keyboard routing, 9 November 2026 (P5-17 / P3-15 / P1-10):
Headless real input/focus/visibility tests cover Tab/Shift-Tab and slider keys
without OS input. They caught and fixed a yaw +180/-180 thumb jump. All 115
focused Rust tests pass; native key delivery/UI/GPU acceptance remains pending.
See [the zone record](../ZONE_EDITOR.md#property-keyboard-routing-and-yaw-endpoints--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Bounded property sliders, 9 November 2026 (P5-17 / P3-15 / P1-10):
Plant scale, marker radius and whole/point-guide width now use quarter-step
sliders in the retained exact form. Shared controlled-widget handling preserves
precision, validation and one-entry undo. All 113 focused Rust tests and CPU
layout pass; native UI/input/GPU acceptance remains pending. See [the zone record](../ZONE_EDITOR.md#bounded-property-sliders--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Transform yaw slider, 9 November 2026 (P5-17 / P3-15 / P1-10):
Object/plant/marker properties now expose a standard Bevy yaw slider alongside
exact fields. It edits the pending form; Apply remains one shared validated undo
entry. Precision, no-op/redo, cancellation and 3440 x 1440 CPU layout pass;
native UI/input/GPU acceptance remains pending. See [the zone record](../ZONE_EDITOR.md#transform-yaw-slider--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content double-click framing, 9 November 2026 (P5-17 / P3-15 / P1-10):
Double-clicking an entry selects and frames it through the existing checked
Frame helpers. Single-click behavior, view preset, source and redo are preserved.
Actual headless Bevy event checks cover every content kind at local/far origins;
native input/GPU acceptance remains pending. See [the zone record](../ZONE_EDITOR.md#content-double-click-framing--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content search, 9 November 2026 (P5-17 / P3-15 / P1-10):
Find/Clear search use the retained Bevy text form for bounded name/kind/#ID
matching within the chosen type filter. Search, including a pending form, leaves
source/save state/history unchanged. Headless behavior/layout checks pass;
native UI/input/GPU acceptance remains pending. See [the zone record](../ZONE_EDITOR.md#content-search--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content-type filters, 9 November 2026 (P5-17 / P3-15 / P1-10):
All/Objects/Guides/Markers/Plants/Terrain buttons narrow the content list while
preserving source, selection and pending redo. Actual Bevy observer/sync and
CPU font/layout checks pass; native UI/input/GPU acceptance remains pending.
See [the zone record](../ZONE_EDITOR.md#content-type-filter-buttons--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content-list paging, native acceptance pending, 9 November 2026
(P5-17 / P3-15 / P1-10): the row pool reserves space above Properties; wheel
paging and disabled boundary buttons have headless Bevy-system coverage.
Standard Bevy CPU font/layout checks also verify all row/footer bounds at 3440 x 1440.
The earlier native wheel check timed out; do not claim GPU/input acceptance.
See [the zone record](../ZONE_EDITOR.md#content-list-paging-headless-coverage--9-november-2026).
Continue headlessly after local commits; no mouse, keyboard/display use or push.

Piped structured commands, 9 November 2026 (P5-17 / P3-15 / P1-10):
--commands - reads bounded JSONL from stdin through EOF using the same command
validation/history path as files and the viewport. Headless output is written
only after all commands succeed; read-only playtest preflight accepts the same
stream. See [the zone record](../ZONE_EDITOR.md#piped-structured-commands--9-november-2026).
Continue headless after local commits; no desktop input/display use; no push.

Structured entry translation, 9 November 2026 (P5-17 / P3-15 / P1-10):
The shared translate edit offsets primitive/plant/marker anchors or every point
of a whole guide, preserving identity and metadata. Mixed Apply batches retain
whole-candidate validation and one-entry undo. CLI/preflight and mailbox paths
are qualified headlessly; see [the zone record](../ZONE_EDITOR.md#structured-entry-translation--9-november-2026).
Continue headless after local commits; no desktop input/display use; no push.

Headless playtest preflight, 9 November 2026 (P5-17 / P3-15 / P1-10):
--check-playtest builds the actual offline collision scene and resolves the
requested capsule spawn, returning JSON/readiness and a nonzero refusal status.
It opens no window, writes no source, and supports in-memory command validation.
See [the zone record](../ZONE_EDITOR.md#headless-playtest-preflight--9-november-2026).
Continue headless after local commits; no desktop input/display use; no push.

Placement/move grid presets, 9 November 2026 (P5-17 / P3-15 / P1-10):
Properties now exposes a discrete 0.125/0.25/0.5/1/2/4/8 m step slider. Shared
zone-origin snapping covers placement and primitive move/body gestures; nudges
use the chosen increment. Settings preserve history/redo; rotation and scale
snapping retain their contracts. See [the zone record](../ZONE_EDITOR.md#placement-and-move-grid-presets--9-november-2026).
Continue after each local commit; no push.

Pointer viewport panning, 9 November 2026 (P5-17 / P3-15 / P1-10):
Middle-mouse drags now pan in the current view plane with projection/zoom-aware
logical cursor tracking. Pans must begin inside the viewport; panels, Escape,
focus loss, editing gestures, modals and playtest cancel/refuse them. Source and
history stay unchanged. See [the zone record](../ZONE_EDITOR.md#pointer-viewport-panning--9-november-2026).
Continue after each local commit; no push.

Exact spawn properties, 9 November 2026 (P5-17 / P3-15 / P1-10):
Edit spawn opens three exact world-coordinate fields in the existing docked
properties form. Apply uses shared SetSpawn validation and one-entry undo;
unchanged values preserve redo. Existing placement actions remain available.
See [the zone record](../ZONE_EDITOR.md#exact-spawn-properties--9-november-2026).
Continue after each local commit; no push.

Active document context, 9 November 2026 (P5-17 / P3-15 / P1-10):
The footer now persistently identifies the active save path and saved/unsaved
state; the window title identifies the zone/file and marks dirty content. Older
terrain/vegetation shortcut hints now distinguish Open from Reload. See
[the zone record](../ZONE_EDITOR.md#active-document-context--9-november-2026).
Continue after each local commit; no push.

View-plane reference grids, 9 November 2026 (P5-17 / P3-15 / P1-10):
Front/Right now expose adaptive vertical reference grids; zones without terrain
also use an adaptive ground grid. The standard Grid checkbox controls references
and retained terrain surface lines, shows metre spacing, and preserves history.
See [the zone record](../ZONE_EDITOR.md#view-plane-reference-grids--9-november-2026)
for acceptance and limitations. Continue after each local commit; no push.

Zone file browser, 9 November 2026 (P5-17 / P3-15 / P1-10):
The command bar now exposes Open and Save As with a standard Bevy folder/file
browser, filename input, paging and explicit overwrite confirmation. Open uses
validated document replacement with undo; Save As changes the active path only
after success and preserves history. Ctrl+O opens, Ctrl+Shift+S saves as and
Ctrl+Shift+O reloads the active file. See [the zone record](../ZONE_EDITOR.md#zone-file-browser--9-november-2026)
for acceptance and limitations. Continue after each local commit; no push.

Shared body-drag previews, 9 November 2026 (P5-17 / P3-15 / P1-10):
Selected-body XZ dragging now shares the same validated gesture path as axis,
yaw and scale handles. Relative local deltas preserve original f64 values on a
stationary click, with unchanged Y/yaw/size. Invalid candidates never replace the
last valid preview, live source or undo/redo. See [the zone record](../ZONE_EDITOR.md#validated-body-plane-drag--9-november-2026) for acceptance.
Continue after each local commit until stopped; no push.

Canonical authoring views, 9 November 2026 (P5-17 / P3-15 / P1-10):
Standard Bevy orthographic Top/Front/Right views and a Perspective control now
support terrain and greybox authoring. Canonical pan/zoom, stable orthographic
camera distance, projection-aware handle sizing and camera locking during drags
are implemented; playtest retains its perspective camera and restores the editor
view. See [the zone record](../ZONE_EDITOR.md#canonical-authoring-views--9-november-2026) for acceptance.
Continue after each local commit until stopped; no push.

Primitive uniform scale handle, 9 November 2026 (P5-17 / P3-15 / P1-10):
Scale uniformly joins Move/Rotate Y in the application toolbar. A standard Bevy
cube/stem scales all dimensions by one factor, with optional 10% snapping, keeping
position/yaw exact and preserving sphere/cylinder constraints. Changed previews and
release share existing whole-candidate Transform validation and one-entry undo.
See [the zone record](../ZONE_EDITOR.md#primitive-uniform-scale-handle--9-november-2026) for acceptance.
Continue after local commits until stopped; no push.

Primitive yaw rotation handles, 9 November 2026 (P5-17 / P3-15 / P1-10):
The application toolbar now chooses Move or Rotate Y. A standard Bevy torus
provides a yaw ring, with 15-degree snapping and unsnapped editing; angle unwrapping
supports crossing the ±180-degree seam and full turns. Position and size remain
exact. Both tools share validated previews and one-entry Transform history.
See [the zone record](../ZONE_EDITOR.md#primitive-yaw-rotation-ring--9-november-2026) for acceptance.
Continue after local commits until stopped; no push.

Primitive move handles, 9 November 2026 (P5-17 / P3-15 / P1-10):
Selected primitives now expose standard Bevy red X / green Y / blue Z move arrows.
Axis drags preview through shared Transform validation and commit once on release;
Escape, focus loss and release over panels cancel. A Move XYZ checkbox controls
visibility. Exact f64 coordinates on the other axes remain untouched. See [the zone record](../ZONE_EDITOR.md#primitive-world-axis-move-handles--9-november-2026)
for constraints and acceptance. Continue after each local commit; no push.

Guide properties and point controls, 9 November 2026 (P5-17 / P3-15 / P1-10):
The docked properties form now exposes whole-guide width or selected-point XYZ
and width, through existing validated PutPath commands. Mouse Previous/Next point,
Whole guide, Insert and Remove controls expose point selection/topology directly.
Undo/redo and pending/modal/playtest isolation remain shared. See [the zone record](../ZONE_EDITOR.md#guide-properties-and-point-controls--9-november-2026)
for prerequisites and acceptance. Continue after local commits until stopped; no push.

Plant and marker properties, 9 November 2026 (P5-17 / P3-15 / P1-10):
The docked Transform form now edits plants and NPC/quest authoring markers as
well as primitives. Five relevant fields expose position/yaw and scale or radius;
shared candidate commands preserve names, kinds and marker definition keys.
Apply/Cancel, exact f64 values, modal input isolation and one-entry history remain.
The primitive form still exposes all seven fields. See [the zone record](../ZONE_EDITOR.md#plant-and-marker-transform-properties--9-november-2026)
for prerequisites, acceptance and evidence. Continue locally; no push.

Vegetation properties, 9 November 2026 (P5-17 / P3-15 / P1-10):
The application properties area now exposes contextual scatter radius (1..32 m)
and integer plants-per-click (1..64) sliders for tree/shrub scatter tools.
Presets remain editor-local; a ground click uses the existing seeded shared
ScatterVegetation edit, with whole-candidate validation and one-entry undo.
Source schema, gameplay and engine/plugin pins are unchanged. Native local/far
3440x1440 acceptance and exact evidence are in [the zone record](../ZONE_EDITOR.md#vegetation-scatter-properties--9-november-2026).
Continue local implementation and commits; no push.

Application-style editor work resumed, 9 November 2026 (P5-17 / P3-15 / P1-10):
The user chooses to continue the in-repository editor and prioritizes Blender-like
application workflows: direct sliders, buttons, toggles and a properties area,
rather than larger controls or additional shortcut-driven features. Standard Bevy
widgets now provide workspace buttons, a command bar, a snap checkbox and
contextual terrain sliders. The right column splits a 24-row outliner from a
properties area; exact transform editing docks there with existing transactional
Apply/Cancel validation. Pending previews and modal/playtest isolation remain
protected. The first shell slice has 77 focused Rust checks and native actual
3440x1440 local/far evidence; see the zone record for exact scope. Keep changes
local and do not push. The earlier safe-stop record below is historical.

Safe stop, 8 November 2026 (P5-17 / P3-15 / P1-10; P2-06/07):
The user requested stopping after the current slice. Terrain-conforming brush
feedback, accepted stroke counts/cap feedback, 2 m spatial drag sampling,
surface-supported cursor spawn placement and a clearer spawn marker are complete.
Unrelated document edits also preserve resident terrain meshes. The current
77 focused Rust cases, formatting, strict Clippy and native 3440x1440 checks pass;
exact contracts/evidence are in the zone record. Work is committed locally; no
push. No further autonomous slices begin until the user resumes work.

Zone authoring workspace, 8 November 2026 (P5-17 / P3-15 / P1-10; P2-06/07):
H and structured `project_to_surface` now ground selected objects, plants,
markers and guide points through shared candidate validation and undo. Primitive
centres account for half-height; foot anchors use the exact terrain support.
Whole guides are one undo entry. The summary also distinguishes saved content
from unsaved visual/agent edits and pending drafts. This grounds anchors rather than prop stacks
or full slope footprints. Tab/Shift-Tab also cycle hidden/distant objects; F
frames objects, plants, markers and the existing guide/point selection without
changing history. Committed guides also insert/remove points through Insert /
Backspace and narrow shared commands; structural changes clear stale point
selection while visual edits explicitly select the new/next point. A standard-Bevy
right-side content browser also selects named entries across modes through a
fixed 40-row pool and paging, without changing document/history. Both side
panels are excluded from world mouse actions; the list hides in playtest. F8 /
Shift-F8 also rename selected content / the zone through standard Bevy TextInput
and narrow shared commands, with modal input isolation and one-entry undo. Names
retain existing UTF-8 byte validation; geometry/definitions are unchanged.
Local/far/reopen name-dialog native checks pass with zero GPU diagnostics.
F9 also edits exact f64 primitive position/yaw/size through standard Bevy fields
and modal tab navigation, applying one existing validated Transform transaction.
All six local/far primitive edits match headless commands exactly; native
save/reopen and zero-GPU-diagnostic checks also pass. Both forms clear stale
error messages on text edits while retaining validation on Apply.
Terrain brush feedback also conforms to the exact surface triangles and temporary
stroke preview, sharing sculpt/paint falloff and locked-border rules. One resident
standard Bevy mesh shows potential sample influence without source/history changes.
Held strokes also show accepted stamp counts and an explicit 128-stamp limit
message, preserving the existing bounded commit/cancel behavior. Fast drags
also interpolate 2 m stamps, breaking continuity across panels/outside terrain.
Both grids pass native sculpt/paint command equivalence, undo/redo and save.
P also places a surface-supported playtest spawn from Sculpt/Paint, respecting
snapping and pending-stroke/modal isolation through the existing SetSpawn edit.
The view-centre button shares checked conversion and surface sampling.
An unlit downward spawn marker also replaces the small sphere and hides during
playtest. Unrelated edits retain the unchanged terrain mesh instead of rebuilding it.
Flatten also samples the surface with I and adjusts its local
level in 1 m / 0.1 m steps, sharing exact visual/headless stamp output.
Ctrl-D and structured `duplicate` commands copy any authoring kind with fresh
shared IDs, bounded names/offsets and one-entry history. Local/far/empty native
copy/framing and positive/negative flatten cases pass. Raise/lower strength also
halves/doubles within 0.05..8 m, with exact visual/headless 0.125 m stamp checks
on both grids. Brackets also adjust independent sculpt/paint blends within
5..100%, retaining 50% / 100% defaults and exact 12.5% command equivalence.
`python3 tools/edit_zone.py --example` now opens/reopens a separate
Wayfarer Hollow draft from an original four-transaction recipe. F also frames
the terrain patch. Its graded walking route and shelter pass actual Avian checks;
local/far 3440x1440 editor runs and two real Renet client pairs pass with zero GPU
diagnostics, including far origin Y=20. Markers remain authoring notes and the
stream remains a visual guide. 77 focused Rust cases, seven launcher cases, formatting
and strict workspace/all-target Clippy pass. The full native 3440x1440 walkthrough also passes
with Vulkan validation active and zero GPU diagnostics; [contract and evidence](../ZONE_EDITOR.md#wayfarer-hollow-example-and-terrain-framing--8-november-2026).
Local commits only; no push. Chat composer/bubbles remain deferred.

Current focus, 8 November 2026 (P5-17 / P3-15 / P1-10; terrain P2-06/07):
The user prioritizes terrain, greyboxing and zone authoring and defers chat
composer/bubbles. The standard Bevy editor now has shared validated visual/agent
history, primitive transforms/snapping, bounded terrain brushes, road/stream
guides, NPC/quest annotations, greybox vegetation/scatter, save/load, offline
playtest and export into the retained Renet world-package path. Actual prerequisites
are the pinned stack, checked coordinates and existing query/package pipeline;
later waves and P5-16 are not blanket gates. Guide-driven terrain grading also shares validated commands and undo.
Individual guide points also share validated moves, visual picking and camera focus.
Terrain colour painting adds shared preview/history and standard vertex-colour export.
A one-command launcher reopens drafts and defers initial writes until explicit Save.
Optional 2 m terrain refinement/coarsening also shares validation/history and export.
Current qualification is 50 focused
Rust cases, strict workspace Clippy, default-fullscreen 3440x1440 OS-input checks
and actual exported-package encrypted client pairs. The editor targets only
3440x1440 under the user's current direction. Local commits only; no push.
[Exact contracts, acceptance evidence and remaining scope](../ZONE_EDITOR.md).
Older task/queue entries below retain their historical scope; P3-17/18 remain
unspecified. Live NPC/quest publication, general vegetation assets/behavior,
terrain grading/materials and broader terrain authoring remain separate work.

# Theia MMO — Integrated Engineering Implementation Plan

**Current initial control/combat scope, 1 November 2026:**
[THEIA-COMBAT-001](../COMBAT_TARGETING_MOVEMENT_SPEC.md) and the
[migration slices](../MOVEMENT_CAMERA_TARGET.md) supersede earlier hybrid/dodge
priorities. Reuse existing task IDs and actual prerequisites. Defer dodge/action
camera; establish classic controls, authoritative manual facing, persistent
selection and weapon-timed autoattack for the one-hostile milestone.

Approved six phases, **112 planned stages**, ten overlapping waves and twelve continuous workstreams. This is a reconstructed actionable plan from the voice discussion, **not a verified repository audit**. Phase 3 originally listed 18 stages but only 16 were individually named; placeholders are explicitly marked. Phase 4–6 numbering below is a proposed normalization of the approved scope, not a verbatim previously approved numbered list.

## Phase 1 — 12 stages
- **P1-01** Repository audit and reusable/obsolete classification
- **P1-02** Rust workspace, Bevy client, headless server and shared crates
- **P1-03** World coordinates and spatial conventions
- **P1-04** Entity identity and lifecycle
- **P1-05** 30Hz fixed timestep, bounded input queues
- **P1-06** Command schemas, sequencing, acknowledgements and loss tests
- **P1-07** Renet integration and validation (replaces comparative transport study)
- **P1-08** Replication protocol contracts
- **P1-09** Kinematic movement and collision contracts
- **P1-10** Asset/terrain/animation content contracts
- **P1-11** Data-driven ability/combat schemas
- **P1-12** Tests, instrumentation and benchmark baselines

## Phase 2 — 14 stages
- **P2-01** Bevy client bootstrap
- **P2-02** Connection/session management
- **P2-03** Entity replication and lifecycles
- **P2-04** Prediction and reconciliation
- **P2-05** Third-person camera/controls
- **P2-06** Terrain chunk loading
- **P2-07** Collision integration
- **P2-08** GLB and asset loading
- **P2-09** Character skeletal animation
- **P2-10** Lighting/materials/atmosphere
- **P2-11** Targeting and interactions
- **P2-12** One authoritative ability, damage, death and respawn
- **P2-13** Spell VFX and audio hooks
- **P2-14** Health/target/action-bar UI and integrated demo

## Phase 3 — 18 stages
- **P3-01** Authentication and sessions
- **P3-02** PostgreSQL persistence/migrations
- **P3-03** Character creation/selection
- **P3-04** Stats and resources
- **P3-05** Inventory and items
- **P3-06** Abilities and status effects
- **P3-07** Creature AI
- **P3-08** Equipment and appearance
- **P3-09** Loot and rewards
- **P3-10** Experience and leveling
- **P3-11** Quest objectives
- **P3-12** NPC dialogue and vendors
- **P3-13** Terrain streaming and initial LOD
- **P3-14** Animation, VFX and audio expansion
- **P3-15** Content authoring and validation
- **P3-16** One complete persistent zone and login/logout proof
- **P3-17** UNSPECIFIED in original transcript: needs reconciliation
- **P3-18** UNSPECIFIED in original transcript: needs reconciliation

## Phase 4 — 20 stages
- **P4-01** World region ownership
- **P4-02** Predictive terrain/object streaming
- **P4-03** Streamed navigation and spatial queries
- **P4-04** Advanced interest management
- **P4-05** Adaptive replication
- **P4-06** Cross-region handoff
- **P4-07** Population and overload management
- **P4-08** Advanced combat interrupts/CC/AoE/healing/threat
- **P4-09** Classes and specializations
- **P4-10** Group encounters
- **P4-11** Parties and group rules
- **P4-12** Chat/moderation
- **P4-13** Friends and presence
- **P4-14** Guild persistence
- **P4-15** Secure trading
- **P4-16** Animation LOD and crowd policies
- **P4-17** VFX density budgets
- **P4-18** Time/weather integration
- **P4-19** Multi-zone scale profiling
- **P4-20** World and multiplayer integration gate

## Phase 5 — 24 stages
- **P5-01** Rendering benchmarks
- **P5-02** Lighting and shadows
- **P5-03** Atmosphere and fog
- **P5-04** Stylized materials
- **P5-05** Water
- **P5-06** Terrain LOD and seams
- **P5-07** Streaming memory budgets
- **P5-08** Vegetation instancing and wind
- **P5-09** Visibility/culling
- **P5-10** Animation blending and layers
- **P5-11** Equipment visuals
- **P5-12** Animation LOD
- **P5-13** Character readability
- **P5-14** Particle expansion
- **P5-15** Projectiles/AoE effects
- **P5-16** VFX priority and density limits
- **P5-17** World editor
- **P5-18** Creature/item/ability/quest editors
- **P5-19** Asset conversion and license validation
- **P5-20** Developer inspection tools
- **P5-21** Client performance tuning
- **P5-22** Server profiling
- **P5-23** Deployment/monitoring/recovery
- **P5-24** Integrated visual and load validation

## Phase 6 — 24 stages
- **P6-01** Release scope and quality gates
- **P6-02** Minimum playable content
- **P6-03** Internal playtests/defect tracking
- **P6-04** Class/combat balance
- **P6-05** Progression and economy validation
- **P6-06** Quest/encounter chains
- **P6-07** Onboarding/accessibility
- **P6-08** Closed alpha accounts/environment
- **P6-09** Installer/patcher
- **P6-10** Closed alpha stabilization
- **P6-11** Concurrent gameplay load test
- **P6-12** Crowded encounter test
- **P6-13** Recovery drills
- **P6-14** Security/exploit testing
- **P6-15** Closed beta population
- **P6-16** Long soak testing
- **P6-17** Production deployment/support
- **P6-18** Moderation/support workflows
- **P6-19** Release candidate validation
- **P6-20** Operational readiness review
- **P6-21** Go/no-go
- **P6-22** Final load test
- **P6-23** Launch
- **P6-24** Launch stabilization

## Ten approved overlapping execution waves
- W1: Phase 1 foundations and early rendering/terrain/animation/persistence spikes
- W2: Phase 2 client/session/replication/prediction foundations
- W3: Phase 2 playable terrain/camera/animation/combat/UI integration
- W4: Phase 3 accounts, persistence, characters, inventory and ability foundation
- W5: Phase 3 full persistent slice, creatures/quests/loot/tooling
- W6: Phase 4 world partition, navigation, interest and scale
- W7: Phase 4 advanced combat, social, guilds, trading and presentation foundations
- W8: Phase 4 completion and Phase 5 visuals, animation, VFX and performance
- W9: Phase 5 tooling/operations and Phase 6 internal alpha preparation
- W10: Phase 6 external tests, hardening, release and stabilization

## Twelve continuous workstreams
Architecture/workspace; simulation/physics; networking/replication; world/terrain/streaming; rendering/vegetation; assets/tooling; animation/VFX/audio; gameplay/combat/AI; persistence/economy; social/UI; security/operations; testing/profiling/release.

## Delivery gates
W1: headless authoritative server, compiling Bevy client, architecture contracts and asset proof. W2: two clients exchange authoritative movement. W3: two players fight one enemy in a rendered world. W4: persistent character and inventory. W5: one complete questing zone survives logout/reconnect. W6: seamless multi-region travel with measured capacity. W7: multiplayer social/group combat. W8: measured visual and simulation performance in crowded scenes. W9: internal alpha and content tooling. W10: closed alpha/beta, security/recovery and launch readiness.

## Dependency rules
Coordinate types before terrain, physics, networking and navigation; scheduler and identities before replication; replication before prediction; ability schema before combat/VFX; character skeleton contracts before equipment/mounts; persistence before transactional inventory/trades; terrain collision before streamed navmesh; region ownership before handoffs; prioritization before crowded raid effects; instrumentation before optimization. Workstreams overlap and no phase is a rigid waterfall.
