# Implementation Plan checkpoint history

> Archived progress notes. Use the [active document](../../engineering/IMPLEMENTATION_PLAN.md)
> for the plan or acceptance register and [status](../../STATUS.md) for evidence.
> Historical next steps and permissions are not current instructions.

P5-17 permanent numeric attributes, 10 November 2026: selected objects/plants/
markers/guides use always-visible textboxes with one aligned frame/background.
No click-to-reveal or Apply panel; Enter/blur applies one validated exact edit,
Escape cancels, Tab advances and explicit Save commits valid focused text first.
Keep f64/history and reject stale/replaced-document drafts. All 110 zone-editor
tests, fmt and strict workspace/all-target Clippy pass; local/far native checks
pass on RX 7900 XTX / RADV NAVI31 / Vulkan with zero diagnostics. Further P5-17
refinement remains active. See [record](../../editor/EDITOR_UI_REFINEMENT.md).

P5-17 dock/status checkpoint, 10 November 2026: implement the stacked left
explorer/properties dock, independent folding and width/split resizing, type
filter dropdown and compact full-width status bar. Remove repeated explorer/help
text and the companion launcher. All 107 zone-editor tests, fmt and strict
workspace/all-target Clippy pass; native local/far checks pass with zero GPU
diagnostics on RX 7900 XTX / RADV NAVI31 / Vulkan. Inline edits, category tabs,
viewport toolbar and popup refinement remain active work, not completed behavior.
See [refinement record](../../editor/EDITOR_UI_REFINEMENT.md).

P5-17 current execution, 10 November 2026: refine the integrated editor again
under the user's direct feedback. Target inline auto-applied values (explicit
Save writes disk), explorer/attributes stacked with a horizontal divider and
independent collapsible/resizable panels, dropdown filtering, meaningful property
sections, Sprinkles-style popovers and correctly aligned input text. Remove the
separate companion launch option. These are current requirements, not completed
behavior; earlier close-enough acceptance remains historical. Preserve exact
f64/validation/history. See [adoption](../../DECISIONS.md).

P5-17 upstream Sprinkles audit, 10 November 2026: reviewed all 28 issues
against the imported source. Fix obsolete curve/gradient/mesh cache retention,
apply authored bounds to every emitter and move the preview culling toggle onto
meshes. Guard unsupported custom-shader project loads/menu choices to prevent
shared-host panics. Regressions cover reclamation, sharing, bounds, rejected-load
preservation and 0.2 save/reload stamping. Validation and remaining priorities (draft recovery,
long idle memory investigation, per-instance
controls and gameplay integration) are recorded in the
[issue review](../../editor/SPRINKLES_UPSTREAM_REVIEW.md). No full idle-leak/Windows/MMO qualification claim.

P5-17 same-window Sprinkles authoring accepted on Linux, 10 November 2026:
File → Particles workspace opens the actual upstream UI inside Theia; Zone in
its top bar returns. Original editing/playback/save, project/pause retention and
exclusive input/camera/render-layer ownership pass native local/far checks with
exact zone/source/revision/history preservation and zero Vulkan diagnostics on
RX 7900 XTX / RADV NAVI31. All 177 focused tests, three doctests, fmt and strict
workspace/all-target Clippy pass. The original standalone host remains qualified.
The direct editor-integration slice is complete at this scope. Asset placement/
world export/client VFX, full P2-13, Windows and MMO budgets remain separate.
See [SPRINKLES_INTEGRATION](../../editor/SPRINKLES_INTEGRATION.md) for use and exact evidence.

P5-17 original Sprinkles widget commits, 10 November 2026: curve range, gradient
stop and colour edits pass real native input and exact asset/source comparisons
with zero Vulkan diagnostics. Correct the curve range root/child lookup and
reject invalid ranges without commits or panics. All 16 runtime/editor tests,
three doctests, fmt and strict workspace Clippy pass. Same-window hosting is
next. See [SPRINKLES_INTEGRATION](../../editor/SPRINKLES_INTEGRATION.md) for evidence.

P5-17 direct Sprinkles source integration, 10 November 2026: the original
runtime and editor are imported at the reviewed pin and ported to exact Bevy
0.20.0. File → Particle editor opens the retained UI/preview/RON authoring tool
in one companion window. Native editing/save and host launch/return checks pass
on RX 7900 XTX / RADV NAVI31 / Vulkan with zero diagnostics and exact zone/source/
history preservation. All 172 focused tests plus three runtime doctests, fmt and
denied-warning workspace/all-target Clippy pass. Source and asset licenses/hashes
are retained. This completes the first direct-reuse slice; same-window hosting,
asset placement/client integration, Windows and full P2-13 remain pending.
See [SPRINKLES_INTEGRATION.md](../../editor/SPRINKLES_INTEGRATION.md) for exact evidence.

P5-17 UI migration, 10 November 2026: accepted at the user's close-enough
style/usability target. Local/far native Linux checks pass full/short layouts,
menus, attributes, resize, scrolling and playtest return with exact source/file
preservation and zero Vulkan diagnostics on RX 7900 XTX. All 156 focused Rust
checks and strict lint/build checks pass for that production code. Windows remains
unverified. Begin direct Sprinkles runtime/editor source compatibility work;
no particle integration is claimed yet. See STATUS/ZONE_EDITOR for exact evidence.

P5-17, 10 November 2026: divider hover/cursor feedback follows the same
availability rules as resizing, including menu/scrollbar ownership and lost
window focus. The cursor matches Sprinkles' native column-resize icon. All 156
focused tests, formatting, denied-warning Clippy and editor build pass; native
acceptance remains pending. See STATUS/ZONE_EDITOR for scope/evidence.

P5-17, 10 November 2026: CPU tests send native Bevy pointer press/release
through all exact-form slider types at local/far origins. Range endpoints,
focus/release and draft/source/redo preservation pass. All 155 focused tests,
formatting and denied-warning Clippy pass. No production change or OS/GPU input
occurred; native acceptance remains pending. See STATUS/ZONE_EDITOR.

P5-17, 10 November 2026: naming/property native checks are prepared for
current inspector geometry and duplicate menu/action telemetry. Syntax/--help
checks pass; no native run or production change occurred while desktop use is
stopped. See STATUS/ZONE_EDITOR; native acceptance remains pending.

P5-17, 10 November 2026: the grid/pan qualifier is prepared for current
canvas/panel geometry and View/Zone menus. Syntax/--help checks pass; it has not
run while desktop use is stopped. No production behavior changed and no native
acceptance is claimed. See STATUS/ZONE_EDITOR.

P5-17, 10 November 2026: the content-browser native qualifier is prepared
for current layout telemetry and Zone-menu access. Syntax/--help checks pass;
it has not run while desktop use is stopped. No production code changed and no
new native acceptance is claimed. See STATUS/ZONE_EDITOR.

P5-17, 10 November 2026: tool settings separate compact muted names from
right-aligned tabular values/units; exact-form slider labels share the field type.
Setting captions, input and source semantics remain. All 154 focused tests,
formatting, denied-warning Clippy and editor build pass. Native acceptance remains
pending while desktop use is stopped. See STATUS/ZONE_EDITOR for scope/evidence.

P5-17, 10 November 2026: settings/forms share a thin-track, compact-thumb
native slider in Sprinkles colors, including hover/focus/disabled feedback.
Native input and horizontal value mapping are retained. All 154 focused tests,
formatting, denied-warning Clippy and editor build pass; native acceptance remains
pending while desktop use is stopped. See STATUS/ZONE_EDITOR for scope/evidence.

P5-17, 10 November 2026: tool rows separate action names from right-aligned
muted shortcut hints, preserving workspace actions and live brush labels. All
154 focused tests, formatting, denied-warning Clippy and editor build pass.
Native acceptance remains pending while desktop use is stopped. See STATUS/ZONE_EDITOR.

P5-17, 10 November 2026: browser actions use shared Sprinkles MD sizing/type;
file rows use original pinned folder/file icons, filenames and Ghost/Active
states. Menu shortcuts retain muted secondary type. All 154 focused tests,
formatting, denied-warning Clippy and editor build pass. Native acceptance remains
pending while desktop use is stopped. See STATUS/ZONE_EDITOR and icon provenance.

P5-17, 10 November 2026: all 20 complete selected-inspector width/height
cases pass native scroll-reveal CPU geometry checks with source/redo unchanged.
All 154 focused tests, formatting and denied-warning Clippy pass. Native GPU/input
acceptance remains pending while desktop use is stopped; Windows tooling is not
installed, so no Windows claim is made. See STATUS and ZONE_EDITOR.

P5-17, 10 November 2026: inspector values now enter the exact edit form
at the selected attribute, with target/revision/point guards and unchanged full
transaction semantics. All 153 focused tests, formatting, denied-warning Clippy
and editor build pass, including focus/cancel/reflow regressions. Native acceptance
remains pending while desktop use is stopped. See STATUS and ZONE_EDITOR.

P5-17, 10 November 2026: default attribute summaries use aligned compact
Sprinkles-style labels/tabular values over the exact editor's shared f64 capture.
Preview/annotation/guide metadata and the existing edit transaction stay explicit.
All 150 focused tests, formatting, denied-warning Clippy and editor build pass;
native acceptance remains pending while desktop use is stopped. See STATUS and
ZONE_EDITOR for scope/evidence.

P5-17, 10 November 2026: panel headings use shared compact Sprinkles
typography; inspector summaries focus on selected content/tools while document
state stays in the selector/footer. All 148 focused tests, formatting, denied-
warning Clippy and editor build pass. Native acceptance remains pending while
desktop use is stopped. See STATUS and ZONE_EDITOR for scope/evidence.

P5-17, 10 November 2026: top-left document selection now shows the zone
name/unsaved marker and shared file actions in Sprinkles project-selector style.
Explicit zone rename is visually available with selection preserved. All 148
focused tests, formatting, denied-warning Clippy and editor build pass; native
acceptance remains pending while desktop use is stopped. See STATUS/ZONE_EDITOR.

P5-17, 10 November 2026: Camera/Gizmo choices use current-value Sprinkles
combobox-style fields over native menus and the existing guarded preferences.
All 146 focused tests, formatting, denied-warning Clippy and editor build pass,
including actual selector/menu geometry across modes/widths. Native acceptance
remains pending while desktop use is stopped. See STATUS and ZONE_EDITOR.

P5-17, 10 November 2026: content rows gain Sprinkles ellipsis/secondary-
click menus over native Bevy menus, popup placement and clipping. Shared command
routing binds actions to the target/revision/row snapshot and rejects stale views.
All 143 focused tests, formatting, denied-warning Clippy and editor build pass.
Native acceptance
remains pending while desktop use is stopped. See STATUS and ZONE_EDITOR.

P5-17, 10 November 2026: the shell adopts Sprinkles' single 52 px top bar.
Menus/Playtest/primary Save remain at top; camera/grid/snap/handles/gizmo controls
have a named collapsible Viewport inspector section. Shared viewport/panel/input
bounds recover 40 px of height. All 140 focused tests, formatting, denied-warning
Clippy and editor build pass; native acceptance is pending while desktop use is stopped.
See STATUS and ZONE_EDITOR for scope/evidence.

P5-17, 10 November 2026: shared action buttons match Sprinkles MD size,
padding, text and live variant borders. Actual theme/checkbox systems now run in
the CPU panel-layout regression across modes/widths. All 140 focused tests,
formatting, denied-warning Clippy and editor build pass; native acceptance remains
pending while desktop use is stopped. See STATUS and ZONE_EDITOR for evidence.

P5-17, 10 November 2026: Selection actions join Tools/Tool settings as an
independently collapsible Sprinkles-style inspector section. Shared headers use
upstream typography/spacing/dividers. All 140 focused tests and denied-warning
Clippy pass; native acceptance remains pending while desktop use is stopped.
See STATUS and ZONE_EDITOR for exact scope/evidence.

P5-17, 10 November 2026: naming/search and zone-file forms adopt shared
Sprinkles dialog framing, separated titles/status, fixed right-aligned actions
and bounded scrolling bodies. Native focus reveal responds to changed pane size
without fighting manual scrolling. Headless checks include 140 focused tests;
formatting, denied-warning Clippy and editor build pass. Native acceptance is
pending while
desktop use is stopped. See STATUS and ZONE_EDITOR for exact scope/evidence.

P5-17, 10 November 2026: Snap/Handles/Grid use Sprinkles-style outlined
checkboxes and plain labels over native Bevy Checkbox and existing guarded state.
All 139 focused tests and denied-warning Clippy pass; native acceptance remains
pending while desktop use is stopped. See STATUS and ZONE_EDITOR for evidence.

P5-17, 10 November 2026: the content browser now shares Sprinkles-style
scrolling and native ScrollIntoView selection reveal. Short-window wheel routing
preserves bounded paging without double actions or changing keyboard focus.
All 139 focused tests and denied-warning Clippy pass; native acceptance remains
pending while desktop use is stopped. See STATUS and ZONE_EDITOR for evidence.

P5-17, 10 November 2026: numeric attributes adopt Sprinkles' compact framed
inputs, tabular figures, drag prefixes/units and grouped X/Y/Z size. World-f64
position fields stay full width. All 137 focused tests and denied-warning Clippy
pass; native acceptance remains pending while desktop use is stopped. See STATUS
and ZONE_EDITOR for evidence and the retained exact transaction boundary.

P5-17, 10 November 2026: exact attributes now share inspector scrolling,
neutral framing and keyboard focus reveal. Headless checks cover six forms,
local/far precision and short layouts; native checks are prepared but pending
because desktop use is stopped. Continue headlessly across local commits. See
STATUS and ZONE_EDITOR for evidence and the distinction from qualified binaries.

P5-17, 10 November 2026: Sprinkles inspector scrolling and reflow-safe button
activation pass 136 focused Rust tests and local/far native UI/placement/playtest
checks (RX 7900 XTX Vulkan, zero GPU diagnostics). Desktop use is authorized and
work continues across local commits. See `docs/STATUS.md` and the inspector
scroll contract in `docs/editor/ZONE_EDITOR.md` for exact scope, binary hashes and
pending attribute-editor work.

Sprinkles-style workspace sidebar, 10 November 2026 (P5-17): replaces
the top workspace row with a 72 px vertical rail using Sprinkles' icon plates,
labels, spacing and active/hover feedback. Six unmodified Remix icons are embedded
through Bevy's standard asset system with license and exact hashes retained.
Existing guarded workspace commands remain authoritative for mode changes.
Browser, inspector, canvas projection/input, wheel routing, grips and telemetry
include the rail offset; the future native qualifier reads that offset.

All 134 focused Rust tests pass, including actual Bevy CPU layout across all six
workspaces and three panel-width pairs, viewport/ray conversion across DPI scales,
resize cancellation and browser wheel exclusion over the rail. Formatting,
denied-warning workspace/all-target Clippy and build pass. With desktop use
renewed, Linux native 3440 x 1440 checks pass at local/far fractional origins on
AMD Radeon RX 7900 XTX (RADV NAVI31), Vulkan with validation enabled: six workspace
rail buttons, resize/cancel, sections, mouse/keyboard menus, file browser,
stationary exact attributes, numeric drag/Cancel/Apply/undo and New/undo preserve
full documents. Zero GPU diagnostics. Visual review found unsupported font
chevrons; standard Bevy UI border geometry now renders them consistently.
Evidence: `build/sprinkles-review/sidebar-*.log` and
`build/zone-sprinkles-native-20261110-v3/qualification.json`.
Placement/preview/playtest requalification remains incomplete; desktop use is
stopped again. See STATUS for partial local placement evidence. No Windows claim.

Sprinkles-style numeric attribute dragging, 10 November 2026 (P5-17):
visible ↔ labels scrub their exact field values while ordinary text input remains
available. Normal drag steps by 0.1 per 5 logical pixels; Alt/Super uses 1 per 10.
The adapter preserves the starting f64 and exact original string at zero steps,
uses a symmetric dead zone, and refuses invalid/composing input. It deliberately
omits upstream absolute decimal rounding so distant global coordinates cannot
lose precision. Changes remain pending until the retained Apply transaction;
Escape/focus loss cancels without changing source or redo.

All 134 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and build pass. Headless real field-controller checks cover stationary
precision, local/far object/plant/marker/guide-point/spawn/guide-width drags,
command-oracle equality, one-entry Apply, malformed fields and cancellation.
Native mouse/UI/GPU acceptance remains pending while desktop use is stopped.
Evidence: `build/sprinkles-review/scrub-*.log`.

Standard Bevy embedded viewport, 10 November 2026 (P5-17): adopts the same
ViewportNode mechanism used by Sprinkles. A separate window UI camera presents
the panels and the scene camera renders into the widget's standard image target;
Bevy resizes it to the canvas rectangle. Shared input conversion maps logical
window coordinates into image pixels, and report projection maps back to window
coordinates. Panning uses the actual canvas height. No custom renderer or new
engine/plugin version is introduced.

All 132 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and build pass. Real Bevy camera projection/ray round trips cover all four
views, three panel-width pairs and 1x/1.5x/2x scale factors, including panel input
exclusion. The scene/ghost data-only test initializes the real viewport and its
pointer map. Native rendering, GPU/input, resize and playtest acceptance remain
pending while mouse/display use are stopped. Earlier partial native evidence is
for the pre-ViewportNode binary only. Evidence: `build/sprinkles-review/viewport-*.log`.

Sprinkles-style panel resizing, 10 November 2026 (P5-17): browser and
inspector boundaries expose bounded 12 px grips, hover/drag feedback and standard
resize cursors. Browser width is 300..480 px and inspector width 300..600 px.
Settings remain session-local; cancellation/focus loss restores the original
widths, and gesture ownership shields world/camera/keyboard edits through release.
Shared canvas bounds, wheel routing, exact form placement and footer track the
widths. All 131 focused Rust tests, CPU layouts at narrow/wide/intermediate widths,
formatting, denied-warning workspace/all-target Clippy and build pass.

A partial Linux RX 7900 XTX/Vulkan 3440 x 1440 run exercised resizing, Escape
cancellation, source/history preservation and section collapse/expansion with zero
GPU diagnostics. It stopped safely when focus left the owned window before File
menu input. Full native menu/form/local/far acceptance remains pending; its two
captures have not been visually inspected. Mouse/display use is stopped again.
Evidence: `build/sprinkles-review/panels-*.log` and
`build/zone-sprinkles-native-20261110-v1/`. New native harness:
`tools/qualify_zone_sprinkles.py`. Keep working headlessly; do not push.

Sprinkles-style application menus, 10 November 2026 (P5-17): compact File,
Edit, View and Zone menus replace the long document-command button row. Save and
Playtest remain directly accessible. Native Bevy MenuButton/MenuPopup/MenuItem
provide the menu interaction path; command activation shares existing validation,
undo/history and modal/playtest guards. Headless events exercise open/close/focus,
one-entry undo/redo, disabled/unfocused refusal and far-coordinate preservation.
Raw menu focus blocks authoring navigation, camera movement and browser wheel
paging; closed menu buttons retain global shortcuts. All 130 focused Rust tests
pass. Native visual/input/GPU acceptance remains pending while desktop use is
stopped. Evidence: `build/sprinkles-review/menu-*.log`.

Sprinkles browser/inspector layout, 10 November 2026 (P5-17): replaces the
old opposing/stacked side panels with adjacent full-height left browser and
inspector columns, followed by the world canvas. The inspector contains the
contextual palette in a collapsible Tools group and a separate Tool settings
group. Exact transform editing occupies the same inspector column. Wheel routing,
canvas input exclusion and footer bounds use the new regions. Actual Bevy CPU
layout checks cover all six workspaces, reachable tools, long properties and
local/far exact coordinates; all 129 focused Rust tests pass. Native UI/input/GPU
acceptance is pending while mouse and display use remain stopped. Existing native
evidence applies only to the earlier theme binary, retained under
`build/sprinkles-theme-qualified/`. Menus, richer direct attribute editing and
resizing remain subsequent migration work. Evidence: `build/sprinkles-review/`.

Sprinkles collapsible tool settings, 10 November 2026 (P5-17): a native Bevy
section header now collapses/expands the retained tool settings with Sprinkles'
chevron, spacing, ghost hover and separator styling. Activation moves focus to
the header so hidden sliders cannot own keys. Actual observer checks at local/far
fractional origins preserve exact source, redo, preferences and saved state;
unfocused/playtest activation is refused. Section telemetry supports later native
qualification. CPU layout and all 128 focused Rust tests pass; denied-warning
workspace/all-target Clippy passes. Native UI/input/GPU acceptance is pending:
mouse and display use are stopped. The previously qualified theme binary remains
available until the new presentation passes native checks. Evidence:
`build/sprinkles-review/sections-tests.log` and `sections-clippy.log`.

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
3440 x 1440 with zero Vulkan diagnostics. See [the record](../../editor/ZONE_EDITOR.md#placement-refusal-hints-and-renderer-guards--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Read-only primitive placement preview, 9 November 2026 (P5-17 / P3-15 / P1-10):
A standard Bevy ghost shows the prospective stamp: blue for valid, red for a
refused candidate. It shares pose/object construction and command validation
with actual placement while preserving source, history, redo and IDs. All 127
focused Rust tests, formatting, strict Clippy and build pass. Native local/far
3440 x 1440 preview/stamp equality, panel hiding and refused-click isolation
pass with zero Vulkan diagnostics. See [the record](../../editor/ZONE_EDITOR.md#read-only-primitive-placement-preview--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Workspace slider focus cue, 9 November 2026 (P5-17 / P3-15 / P1-10):
A pale thumb identifies the focused slider; the footer explains its keys.
All 124 focused Rust tests, strict Clippy, formatting and build pass. Native
local-origin 3440 x 1440 rendering/keys/placement pass with zero Vulkan diagnostics;
app captures were inspected. See [the record](../../editor/ZONE_EDITOR.md#workspace-slider-focus-cue--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Workspace slider keyboard ownership, 9 November 2026 (P5-17 / P3-15 / P1-10):
Workspace sliders now take focus on click. Left/Right/Home/End adjust that widget
without also editing the selection through raw keyboard polling; ordinary nudges
resume when focus clears. All 124 focused Rust tests, formatting, strict Clippy
and build pass. Native grid/size keys preserve selected source and redo at local/
far origins, 3440 x 1440, with zero Vulkan diagnostics. See [the record](../../editor/ZONE_EDITOR.md#workspace-slider-keyboard-ownership--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Primitive placement dimensions, 9 November 2026 (P5-17 / P3-15 / P1-10):
Properties has quarter-step size sliders for new box X/Y/Z, cylinder diameter/
height and sphere diameter. Preferences preserve source/history; stamps use
checked snapping/surface support and one shared edit. All 123 focused Rust tests,
formatting, strict Clippy and build pass. Native local/far shape controls,
placement, full-document command oracles, undo/redo and Save pass at 3440 x 1440
with zero Vulkan diagnostics. See [the record](../../editor/ZONE_EDITOR.md#primitive-placement-dimensions--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Application shortcuts, 9 November 2026 (P5-17 / P3-15 / P1-10):
Ctrl+N invokes shared New; Ctrl+F opens content search without framing. Modal,
preview, playtest and focus guards remain. All 119 focused Rust tests, strict
Clippy, formatting and build pass; native key delivery and full New/save/play
workflow pass at 3440 x 1440 from local/far origins with zero Vulkan diagnostics.
See [the record](../../editor/ZONE_EDITOR.md#application-document-and-search-shortcuts--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Native application controls accepted, 9 November 2026 (P5-17 / P3-15 / P1-10):
Content paging/wheel, filters, search, double-click framing and yaw/scale/radius/
guide-width sliders pass native 3440 x 1440 checks at local/far origins, including
Reset, Home/End, exact source/oracle equality and undo/redo. Vulkan validation is
active with zero diagnostics. The earlier wheel-delivery gap is closed for these
accepted runs after pre-launch synthetic-device creation. See [the record](../../editor/ZONE_EDITOR.md#native-application-controls-acceptance--9-november-2026).
Continue after local commits; desktop access authorized; no push. Historical
headless records below retain their original execution limitations.

New untitled zones, 9 November 2026 (P5-17 / P3-15 / P1-10):
New zone creates an undoable empty document. Its first Save chooses a filename;
cancel/failed writes preserve untitled state and the prior file is never written.
All 118 focused Rust tests, formatting, strict Clippy and the editor build pass.
Native New/save/playtest checks pass at 3440 x 1440 from local/far origins,
with active Vulkan validation and zero GPU diagnostics.
See [the zone record](../../editor/ZONE_EDITOR.md#new-untitled-zones--9-november-2026).
Desktop access is authorized again for native 3440 x 1440 checks; continue after
local commits and do not push. Historical headless limits below remain historical.

Reset transform fields, 9 November 2026 (P5-17 / P3-15 / P1-10):
A Reset fields button restores the open form’s exact original numbers and
sliders, retaining the form, original revision and pending redo. Apply/Cancel
remain the transaction boundary. Headless behavior/layout checks pass; native
UI/input/GPU acceptance remains pending. See [the zone record](../../editor/ZONE_EDITOR.md#reset-transform-fields--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Property keyboard routing, 9 November 2026 (P5-17 / P3-15 / P1-10):
Headless real input/focus/visibility tests cover Tab/Shift-Tab and slider keys
without OS input. They caught and fixed a yaw +180/-180 thumb jump. All 115
focused Rust tests pass; native key delivery/UI/GPU acceptance remains pending.
See [the zone record](../../editor/ZONE_EDITOR.md#property-keyboard-routing-and-yaw-endpoints--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Bounded property sliders, 9 November 2026 (P5-17 / P3-15 / P1-10):
Plant scale, marker radius and whole/point-guide width now use quarter-step
sliders in the retained exact form. Shared controlled-widget handling preserves
precision, validation and one-entry undo. All 113 focused Rust tests and CPU
layout pass; native UI/input/GPU acceptance remains pending. See [the zone record](../../editor/ZONE_EDITOR.md#bounded-property-sliders--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Transform yaw slider, 9 November 2026 (P5-17 / P3-15 / P1-10):
Object/plant/marker properties now expose a standard Bevy yaw slider alongside
exact fields. It edits the pending form; Apply remains one shared validated undo
entry. Precision, no-op/redo, cancellation and 3440 x 1440 CPU layout pass;
native UI/input/GPU acceptance remains pending. See [the zone record](../../editor/ZONE_EDITOR.md#transform-yaw-slider--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content double-click framing, 9 November 2026 (P5-17 / P3-15 / P1-10):
Double-clicking an entry selects and frames it through the existing checked
Frame helpers. Single-click behavior, view preset, source and redo are preserved.
Actual headless Bevy event checks cover every content kind at local/far origins;
native input/GPU acceptance remains pending. See [the zone record](../../editor/ZONE_EDITOR.md#content-double-click-framing--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content search, 9 November 2026 (P5-17 / P3-15 / P1-10):
Find/Clear search use the retained Bevy text form for bounded name/kind/#ID
matching within the chosen type filter. Search, including a pending form, leaves
source/save state/history unchanged. Headless behavior/layout checks pass;
native UI/input/GPU acceptance remains pending. See [the zone record](../../editor/ZONE_EDITOR.md#content-search--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content-type filters, 9 November 2026 (P5-17 / P3-15 / P1-10):
All/Objects/Guides/Markers/Plants/Terrain buttons narrow the content list while
preserving source, selection and pending redo. Actual Bevy observer/sync and
CPU font/layout checks pass; native UI/input/GPU acceptance remains pending.
See [the zone record](../../editor/ZONE_EDITOR.md#content-type-filter-buttons--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content-list paging, native acceptance pending, 9 November 2026
(P5-17 / P3-15 / P1-10): the row pool reserves space above Properties; wheel
paging and disabled boundary buttons have headless Bevy-system coverage.
Standard Bevy CPU font/layout checks also verify all row/footer bounds at 3440 x 1440.
The earlier native wheel check timed out; do not claim GPU/input acceptance.
See [the zone record](../../editor/ZONE_EDITOR.md#content-list-paging-headless-coverage--9-november-2026).
Continue headlessly after local commits; no mouse, keyboard/display use or push.

Piped structured commands, 9 November 2026 (P5-17 / P3-15 / P1-10):
--commands - reads bounded JSONL from stdin through EOF using the same command
validation/history path as files and the viewport. Headless output is written
only after all commands succeed; read-only playtest preflight accepts the same
stream. See [the zone record](../../editor/ZONE_EDITOR.md#piped-structured-commands--9-november-2026).
Continue headless after local commits; no desktop input/display use; no push.

Structured entry translation, 9 November 2026 (P5-17 / P3-15 / P1-10):
The shared translate edit offsets primitive/plant/marker anchors or every point
of a whole guide, preserving identity and metadata. Mixed Apply batches retain
whole-candidate validation and one-entry undo. CLI/preflight and mailbox paths
are qualified headlessly; see [the zone record](../../editor/ZONE_EDITOR.md#structured-entry-translation--9-november-2026).
Continue headless after local commits; no desktop input/display use; no push.

Headless playtest preflight, 9 November 2026 (P5-17 / P3-15 / P1-10):
--check-playtest builds the actual offline collision scene and resolves the
requested capsule spawn, returning JSON/readiness and a nonzero refusal status.
It opens no window, writes no source, and supports in-memory command validation.
See [the zone record](../../editor/ZONE_EDITOR.md#headless-playtest-preflight--9-november-2026).
Continue headless after local commits; no desktop input/display use; no push.

Placement/move grid presets, 9 November 2026 (P5-17 / P3-15 / P1-10):
Properties now exposes a discrete 0.125/0.25/0.5/1/2/4/8 m step slider. Shared
zone-origin snapping covers placement and primitive move/body gestures; nudges
use the chosen increment. Settings preserve history/redo; rotation and scale
snapping retain their contracts. See [the zone record](../../editor/ZONE_EDITOR.md#placement-and-move-grid-presets--9-november-2026).
Continue after each local commit; no push.

Pointer viewport panning, 9 November 2026 (P5-17 / P3-15 / P1-10):
Middle-mouse drags now pan in the current view plane with projection/zoom-aware
logical cursor tracking. Pans must begin inside the viewport; panels, Escape,
focus loss, editing gestures, modals and playtest cancel/refuse them. Source and
history stay unchanged. See [the zone record](../../editor/ZONE_EDITOR.md#pointer-viewport-panning--9-november-2026).
Continue after each local commit; no push.

Exact spawn properties, 9 November 2026 (P5-17 / P3-15 / P1-10):
Edit spawn opens three exact world-coordinate fields in the existing docked
properties form. Apply uses shared SetSpawn validation and one-entry undo;
unchanged values preserve redo. Existing placement actions remain available.
See [the zone record](../../editor/ZONE_EDITOR.md#exact-spawn-properties--9-november-2026).
Continue after each local commit; no push.

Active document context, 9 November 2026 (P5-17 / P3-15 / P1-10):
The footer now persistently identifies the active save path and saved/unsaved
state; the window title identifies the zone/file and marks dirty content. Older
terrain/vegetation shortcut hints now distinguish Open from Reload. See
[the zone record](../../editor/ZONE_EDITOR.md#active-document-context--9-november-2026).
Continue after each local commit; no push.

View-plane reference grids, 9 November 2026 (P5-17 / P3-15 / P1-10):
Front/Right now expose adaptive vertical reference grids; zones without terrain
also use an adaptive ground grid. The standard Grid checkbox controls references
and retained terrain surface lines, shows metre spacing, and preserves history.
See [the zone record](../../editor/ZONE_EDITOR.md#view-plane-reference-grids--9-november-2026)
for acceptance and limitations. Continue after each local commit; no push.

Zone file browser, 9 November 2026 (P5-17 / P3-15 / P1-10):
The command bar now exposes Open and Save As with a standard Bevy folder/file
browser, filename input, paging and explicit overwrite confirmation. Open uses
validated document replacement with undo; Save As changes the active path only
after success and preserves history. Ctrl+O opens, Ctrl+Shift+S saves as and
Ctrl+Shift+O reloads the active file. See [the zone record](../../editor/ZONE_EDITOR.md#zone-file-browser--9-november-2026)
for acceptance and limitations. Continue after each local commit; no push.

Shared body-drag previews, 9 November 2026 (P5-17 / P3-15 / P1-10):
Selected-body XZ dragging now shares the same validated gesture path as axis,
yaw and scale handles. Relative local deltas preserve original f64 values on a
stationary click, with unchanged Y/yaw/size. Invalid candidates never replace the
last valid preview, live source or undo/redo. See [the zone record](../../editor/ZONE_EDITOR.md#validated-body-plane-drag--9-november-2026) for acceptance.
Continue after each local commit until stopped; no push.

Canonical authoring views, 9 November 2026 (P5-17 / P3-15 / P1-10):
Standard Bevy orthographic Top/Front/Right views and a Perspective control now
support terrain and greybox authoring. Canonical pan/zoom, stable orthographic
camera distance, projection-aware handle sizing and camera locking during drags
are implemented; playtest retains its perspective camera and restores the editor
view. See [the zone record](../../editor/ZONE_EDITOR.md#canonical-authoring-views--9-november-2026) for acceptance.
Continue after each local commit until stopped; no push.

Primitive uniform scale handle, 9 November 2026 (P5-17 / P3-15 / P1-10):
Scale uniformly joins Move/Rotate Y in the application toolbar. A standard Bevy
cube/stem scales all dimensions by one factor, with optional 10% snapping, keeping
position/yaw exact and preserving sphere/cylinder constraints. Changed previews and
release share existing whole-candidate Transform validation and one-entry undo.
See [the zone record](../../editor/ZONE_EDITOR.md#primitive-uniform-scale-handle--9-november-2026) for acceptance.
Continue after local commits until stopped; no push.

Primitive yaw rotation handles, 9 November 2026 (P5-17 / P3-15 / P1-10):
The application toolbar now chooses Move or Rotate Y. A standard Bevy torus
provides a yaw ring, with 15-degree snapping and unsnapped editing; angle unwrapping
supports crossing the ±180-degree seam and full turns. Position and size remain
exact. Both tools share validated previews and one-entry Transform history.
See [the zone record](../../editor/ZONE_EDITOR.md#primitive-yaw-rotation-ring--9-november-2026) for acceptance.
Continue after local commits until stopped; no push.

Primitive move handles, 9 November 2026 (P5-17 / P3-15 / P1-10):
Selected primitives now expose standard Bevy red X / green Y / blue Z move arrows.
Axis drags preview through shared Transform validation and commit once on release;
Escape, focus loss and release over panels cancel. A Move XYZ checkbox controls
visibility. Exact f64 coordinates on the other axes remain untouched. See [the zone record](../../editor/ZONE_EDITOR.md#primitive-world-axis-move-handles--9-november-2026)
for constraints and acceptance. Continue after each local commit; no push.

Guide properties and point controls, 9 November 2026 (P5-17 / P3-15 / P1-10):
The docked properties form now exposes whole-guide width or selected-point XYZ
and width, through existing validated PutPath commands. Mouse Previous/Next point,
Whole guide, Insert and Remove controls expose point selection/topology directly.
Undo/redo and pending/modal/playtest isolation remain shared. See [the zone record](../../editor/ZONE_EDITOR.md#guide-properties-and-point-controls--9-november-2026)
for prerequisites and acceptance. Continue after local commits until stopped; no push.

Plant and marker properties, 9 November 2026 (P5-17 / P3-15 / P1-10):
The docked Transform form now edits plants and NPC/quest authoring markers as
well as primitives. Five relevant fields expose position/yaw and scale or radius;
shared candidate commands preserve names, kinds and marker definition keys.
Apply/Cancel, exact f64 values, modal input isolation and one-entry history remain.
The primitive form still exposes all seven fields. See [the zone record](../../editor/ZONE_EDITOR.md#plant-and-marker-transform-properties--9-november-2026)
for prerequisites, acceptance and evidence. Continue locally; no push.

Vegetation properties, 9 November 2026 (P5-17 / P3-15 / P1-10):
The application properties area now exposes contextual scatter radius (1..32 m)
and integer plants-per-click (1..64) sliders for tree/shrub scatter tools.
Presets remain editor-local; a ground click uses the existing seeded shared
ScatterVegetation edit, with whole-candidate validation and one-entry undo.
Source schema, gameplay and engine/plugin pins are unchanged. Native local/far
3440x1440 acceptance and exact evidence are in [the zone record](../../editor/ZONE_EDITOR.md#vegetation-scatter-properties--9-november-2026).
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
with Vulkan validation active and zero GPU diagnostics; [contract and evidence](../../editor/ZONE_EDITOR.md#wayfarer-hollow-example-and-terrain-framing--8-november-2026).
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
[Exact contracts, acceptance evidence and remaining scope](../../editor/ZONE_EDITOR.md).
Older task/queue entries below retain their historical scope; P3-17/18 remain
unspecified. Live NPC/quest publication, general vegetation assets/behavior,
terrain grading/materials and broader terrain authoring remain separate work.
