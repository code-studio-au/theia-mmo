# Sprinkles editor UI adaptation

Source: https://github.com/doceazedo/sprinkles
Revision: `0e978f5a9afc6b456b3693c8761e73a6c7e155bd`
Selected code license: MIT; the unmodified license is alongside this record.

`apps/zone-editor/src/theme.rs` adapts `ui/tokens.rs` and the color, opacity,
border and foreground behavior of `ui/widgets/button.rs`. Theia retains native
Bevy 0.20 widgets and its own validated commands, input guards and history.
The initial UI adaptation used source targeting Bevy 0.19 without adding its
editor as a dependency. The direct-source integration below supersedes that
initial scope.

The unmodified `InterVariable.ttf` comes from the source editor's
`src/assets/InterVariable.ttf` at this revision, SHA-256
`4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf`.
It is embedded into the editor binary. Its SIL Open Font License and copyright
are in `apps/zone-editor/assets/fonts/OFL.txt`, obtained from
https://github.com/rsms/inter/blob/master/LICENSE.txt
(Git blob `9b2ca37b3ffc77391d8b2ebef4a974ef32bf46ea`).

The initial adaptation included no upstream icons. The sidebar slice below adds
six Remix icons. At that stage no Blender icons, example effects or particle
runtime were included. The later direct-source import below expands that scope.
Theia-specific controls follow the same visual tokens.

`apps/zone-editor/src/sections.rs` adapts `ui/widgets/panel_section.rs`
into native Bevy 0.20 bundles and retained Theia commands.

`panels.rs` follows the resizable-panel interaction from `ui/widgets/panel.rs`,
with Theia-specific logical bounds, world-input shielding and cancellation.

The standard `ViewportNode` arrangement follows `ui/components/viewport.rs`;
Theia adds an explicit window UI camera, standard image target and checked
window/image coordinate conversion for its retained authoring tools.

`properties_scrub.rs` adapts numeric dragging from `ui/widgets/text_edit.rs`.
It retains the upstream normal/Alt/Super increments but preserves exact original
f64 text and uses symmetric truncation rather than absolute decimal rounding.
Theia retains whole-candidate validation and one-entry Apply/Cancel history.

`sidebar.rs` adapts `ui/components/sidebar.rs`: 72 px rail, 12 px padding/gap,
28 px icon plates, 16 px icons, 10 px labels and active/hover feedback. The six
workspaces retain Theia's guarded commands. PNGs are copied unmodified from
`crates/bevy_sprinkles_editor/src/assets/icons/` at the pinned revision and
embedded using Bevy's asset system. Exact filenames, byte counts and SHA-256
hashes are recorded in `sidebar-icons.json`.

These icons originate from Remix Icon, copyright Remix Design. The full
Remix Icon License v1.0 is retained as `LICENSE-REMIX-ICON`, obtained from
https://github.com/Remix-Design/RemixIcon/blob/master/License
(Git blob `996917ee688557176a34e369fa1a37a0a4ef77a2`). They are functional
editor controls, not branding or a redistributed icon-library product.

`scrolling.rs` adapts `ui/widgets/scroll.rs` presentation and wheel units:
24 logical pixels per line, pixel deltas for trackpads, a 3 px Zinc-600 thumb,
3 px edge margin and 24 px minimum thumb length. It uses Bevy 0.20's native
ScrollPosition/Scrollbar/ScrollbarThumb. Theia adds retained authoring/focus
guards, finite-input checks and drag/hover visibility. No custom renderer or
engine/plugin version change is introduced.

The shared scrolling presentation is also used for exact attribute drafts.
`scrolling_focus.rs` adds Theia-specific minimal focus reveal and draft/workspace
ownership while retaining byte-exact f64 strings and Apply/Cancel transactions.

`properties_fields.rs` adapts `ui/widgets/text_edit.rs` frame/prefix/typography
and `ui/widgets/vector_edit.rs` equal-width vector layout/12 px gap. Theia uses
these for exact property fields, retains full-width world-f64 coordinates and
its validated transaction, and exposes units separately from the editable text.
Original source formatting/rounding, f32 narrowing and character filters are
not adopted for Theia's global coordinates.

Shared plain inputs in `theme.rs`, `naming.rs` and `file_picker.rs` also adapt
the pinned text_edit transparent frame, font and cursor/selection presentation.
Theia retains its native validated input handlers and field length limits.

The shared scroll.rs presentation also wraps the bounded zone content browser
in `outliner.rs`. Theia retains bounded row paging and adds Bevy ScrollIntoView
selection reveal with checked source/history invariants.

`theme_checkbox.rs` adapts `ui/widgets/checkbox.rs` at the same pinned revision:
16 px outlined square, 1 px border, 2 px radius, 6 px label gap, 12 px label and
hover-lightened border. Theia draws the check using standard UI borders rather
than upstream's icon image, adds visible keyboard focus/disabled treatment, and
retains Bevy 0.20 Checkbox/ValueChange behavior and guarded Session/View state.

`theme_dialog.rs`, `naming.rs` and `file_picker.rs` adapt static framing from
`ui/widgets/dialog.rs` at the pinned revision: 6 px radius, 1 px border, black
backdrop at 0.8 alpha, default max width 448 px, separated 24 px padded header,
content and footer, 6 px action spacing and right-aligned Cancel/action buttons.
Exact title/description/body sizes follow `ui/tokens.rs` (18/14/12 px). The file
browser keeps a wider 900 px maximum for paths. Bevy scroll widgets and Theia's
existing validation/cancellation/overwrite controllers remain; no easing crate,
animated lifecycle or upstream file controller is imported. The shared dialog
buttons also use upstream `button.rs` Primary and Destructive colors/foreground.

`sections.rs` further matches the pinned panel_section.rs MD presentation:
12 px semibold display title, 12 px vertical spacing/padding, compact header and
bottom divider. The shared pane supplies the 12 px horizontal inset. Selection
actions use the same section as Tools/Tool settings; session collapse state and
native guarded activation/focus behavior remain Theia-owned.

The shared action-button Node in `theme.rs` follows pinned button.rs MD sizing:
28 px high, 12 px horizontal padding, 4 px radius and 12 px text. Default borders
are 1 px; Primary, Destructive, Active, Ghost and Disabled are borderless with
upstream state colors. Wrapped full-width tool actions keep automatic height;
file rows keep left alignment. Native widget activation/guards are unchanged.

The shell in `workspace.rs` now adapts `ui/components/topbar.rs` at the pinned
revision: 52 px height, 12 px inset, ZINC_800 panel background, 1 px bottom border
and right-aligned primary Save. Theia retains its native menus and offline
Playtest action instead of importing particle timeline/playback semantics. Its
camera/grid/snapping/handles/gizmo controls use the shared panel-section styling
in a named Viewport section. All picking/render-image conversions retain the
central shared header boundary rather than duplicating screen-space offsets.

`menus_context.rs` adapts the interaction structure of pinned
`ui/components/data_panel.rs`: a small per-item ellipsis action menu, opened
by its trigger or secondary click, with Duplicate/Rename/Delete plus Theia's
retained attributes/Ground/Frame actions. Shared native MenuItem presentation
comes from `menus.rs`; native Bevy 0.20 Popover and OverrideClip handle placement
and scrolling-panel clipping. Theia keeps a bounded row pool and validated
selection/command/history APIs, with target/revision/row snapshots to refuse
stale contexts. This does not import Sprinkles' particle respawn/state controller.

`menus_select.rs` adapts the default selected-value trigger in pinned
`ui/widgets/combobox.rs`: full-width MD button, selected label, right arrow and
active-open feedback. Shared labels follow the compact field presentation.
Theia uses native Bevy MenuButton/MenuPopup/MenuItem, Popover and OverrideClip
with existing camera/gizmo preference handlers. Current values and availability
come from retained Session/View state; no particle settings or custom popover
lifecycle are imported. The chevron uses standard UI borders.

The Document kind in `menus_select.rs` further adapts pinned
`ui/components/project_selector.rs`: a ghost trigger showing the current asset
name, unsaved '* ' prefix and right chevron. Theia uses its existing zone file
commands rather than importing Sprinkles' project storage/recent-project manager.
The 320 px bound/clipped label protects the shared top bar; explicit zone rename
uses the retained validated naming controller and transaction/history boundary.

Browser/property headings also follow pinned panel_section.rs typography:
12 px semibold display-colored titles. Theia's selection/tool summaries keep
its own metadata and safety feedback, with document state in the shared header
and footer rather than repeated before each inspector description.

`properties_summary.rs` creates the Theia-specific read-only counterpart to the
pinned inspector-field/text-edit presentation: 10 px medium muted labels and
12 px tabular body values in aligned columns. It reuses Theia's exact f64 target
capture rather than importing f32 particle bindings. Actual values, candidate
pose/annotation/guide metadata and transaction/validation semantics remain
Theia-owned; these rows have no editable-input affordance or source write path.

The aligned summary values further use shared native Ghost button presentation
as Theia-specific entry points into its exact edit form. Native ActivateOnPress
and the existing reflow release cleanup handle pointer ownership; target/revision/
guide-point checks protect reused UI data. Full f64 validation, Apply/Cancel and
history semantics stay in the retained Theia property controller.

The file picker embeds two additional unchanged Remix icons from pinned
`crates/bevy_sprinkles_editor/src/assets/icons/`: `ri-file-line.png` and
`ri-folder-open-line.png`. They are stored under
`apps/zone-editor/src/browser_icons/`; `browser-icons.json` records SHA-256,
byte counts and upstream Git blob identities. The retained
`LICENSE-REMIX-ICON` applies. Native ImageNode rows pair these 16 px icons with
filenames and shared Ghost/Active button states; bounded browsing and validated
file operations remain Theia-owned. Browser actions also use shared MD button
sizing/type. Menu keyboard hints retain muted secondary text through live states.

Theia Tools rows reuse the menu secondary-typography treatment: action names
left, muted 10 px keyboard hints right. Existing per-workspace action captions
and live brush-value descriptions remain the source for this presentation.

`theme_slider.rs` is a Theia-specific matching component: the reviewed upstream
widgets contain no slider. Thin Zinc track, rounded blue thumb and shared
hover/focus/disabled colors follow the adopted tokens. Standard Bevy slider
input, original horizontal geometry and Theia validation/controllers are retained.

Shared field-label typography also covers settings and exact-form sliders:
10 px medium muted labels with 12 px tabular setting values/units, following
the pinned inspector-field tokens while retaining Theia setting captions.

Panel dividers also use the pinned panel.rs native ColResize cursor. Theia adds
shared interaction-availability guards for feedback and resize acquisition so
menus, scrollbars, modal/preview/playtest ownership and window focus agree.


## Direct runtime/editor source reuse — 10 November 2026

`crates/bevy_sprinkles` and `crates/bevy_sprinkles_editor` import the runtime,
original authoring UI, assets and asset-loader tests at the same pinned revision.
Both are local, non-published workspace crates using exact Bevy 0.20.0.
`ATTRIBUTIONS.md` records all code and separate asset licenses.
`source-files.json` records the upstream and adapted file hashes, including
renamed shaders and source additions. The upstream README is retained.

Theia extracts the original editor entry point into a reusable library and a
thin companion executable. Engine adaptations cover pointer/text/button APIs,
scene composition, embedded asset paths, WESL shaders and encoded shader buffers.
Preview bounds/collider outlines use standard Bevy line meshes after native
gizmo synchronization failures; this is a measured compatibility adaptation,
not replacement of the particle editor UI or simulation. User data is namespaced
under `.theia/particles`. See `docs/SPRINKLES_INTEGRATION.md` for acceptance and
remaining host/runtime integration boundaries.

The original-widget acceptance slice also corrects the curve range handler's
vector-root lookup and rejects inverted/non-finite input without a commit. This
upstream functionality correction retains the original curve/gradient/colour UI
and effect representation. Native source-preserving value edits are recorded in
`docs/SPRINKLES_INTEGRATION.md`.

The shared-window host adds an original-plugin wrapper, embedded-host settings,
activity/root markers and a Zone return action built with the original ghost
button. Embedded startup shares the host UI camera and ignores host CLI arguments.
Preview scene layers are isolated, and runtime emitter meshes honor their owner's
explicit render layers on creation. Theia's existing theme excludes retained
source widgets. These are host adapters; the original project/inspector/curve/
gradient/colour/playback functionality remains directly reused. Native local/far
shared-window and standalone acceptance is in `docs/SPRINKLES_INTEGRATION.md`.


## Upstream issue audit — 10 November 2026

Review all 28 upstream issues against the retained revision. Local corrections
release obsolete strong curve/gradient/expanded-mesh cache handles while preserving
live sharing, apply authored AABBs to every emitter after mesh-bound calculation
and before visibility checking, and put the preview NoFrustumCulling marker on
emitter meshes instead of its camera. Add actual Bevy asset-lifecycle, multi-emitter
bounds/selection/culling-toggle and migration-save-reload regressions. These are
local adaptations; the upstream revision stays unchanged. Source-file hashes are
refreshed in source-files.json. See docs/SPRINKLES_UPSTREAM_REVIEW.md in Theia for
issue links, validation evidence and explicit remaining scope. This does not
claim the original hour-long idle-memory report is fully resolved. The audit
also guards the imported custom-shader panic path: remove the unimplemented menu
choice and reject unsupported projects before changing the current document;
regressions prove clean/dirty project and source preservation. The RON enum and
runtime format remain compatible; custom shader rendering is not implemented.

Source review during qualification additionally exposed a viewport readiness race.
Retain the editor camera query until the delayed UI scene container can bind;
a delayed-container regression covers that path. No source UI look or runtime
shader algorithm changes are involved.

The local project-save adaptation commits focused valid text before taking its
snapshot, serializes writes and preserves dirty state until success. Writes use
same-directory temporary files; create-new refuses existing destinations. It
also preserves migration dirtiness. These changes do not implement draft
recovery. Original imported file hashes are updated in source-files.json; the
local project_save_tests.rs is additional Theia regression coverage.

Theia adds drafts.rs as a separate draft-storage foundation with temporary-file
replacement and bounded reads. It is not connected to the imported editor UI or
lifecycle yet and does not provide user-facing recovery.

The local recovery dialog is additional Theia UI composed from Sprinkles buttons,
dialogs and scroll controls. It scans draft files off the UI thread and restores
through the guarded local recovery event. Automatic draft capture is pending.

The runtime custom-shader guard now rejects unsupported files through the Bevy
asset loader and prevents programmatic assets from panicking. A live unsupported
emitter temporarily loses its draw-material component and resumes only with a
supported material after buffer synchronization; no fallback shader is supplied.
The format remains readable through version migration, and custom shader rendering
remains unimplemented. Local tests and a windowless GPU example exercise this.

Checked emitter slot, history and sub-emitter-size arithmetic is now applied
before runtime allocation and extraction. Invalid live edits suppress rendering
until repaired. This does not establish a memory cap or validate mesh complexity.

Cylinder side-normal math now handles zero and extremely small heights, and
degenerate prism normals avoid non-finite values. Local headless geometry and
windowless Vulkan regressions cover these cases; no broader mesh-parameter or
performance qualification is claimed.

Sphere generation now preserves finite normals at zero radius and clamps zero
resolution to a minimally valid sphere; this is a local runtime modification.
