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
