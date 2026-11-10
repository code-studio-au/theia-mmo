# Sprinkles editor UI adaptation

Source: https://github.com/doceazedo/sprinkles
Revision: `0e978f5a9afc6b456b3693c8761e73a6c7e155bd`
Selected code license: MIT; the unmodified license is alongside this record.

`apps/zone-editor/src/theme.rs` adapts `ui/tokens.rs` and the color, opacity,
border and foreground behavior of `ui/widgets/button.rs`. Theia retains native
Bevy 0.20 widgets and its own validated commands, input guards and history.
The source editor targets Bevy 0.19 and is not added as a dependency.

The unmodified `InterVariable.ttf` comes from the source editor's
`src/assets/InterVariable.ttf` at this revision, SHA-256
`4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf`.
It is embedded into the editor binary. Its SIL Open Font License and copyright
are in `apps/zone-editor/assets/fonts/OFL.txt`, obtained from
https://github.com/rsms/inter/blob/master/LICENSE.txt
(Git blob `9b2ca37b3ffc77391d8b2ebef4a974ef32bf46ea`).

The initial adaptation included no upstream icons. The sidebar slice below adds
six Remix icons; no Blender icons, example effects or particle runtime are included.
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
