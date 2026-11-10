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

No upstream Blender/Remix icons, example effects or particle runtime are included
in this first adaptation. Theia-specific controls follow the same visual tokens.

`apps/zone-editor/src/sections.rs` adapts `ui/widgets/panel_section.rs`
into native Bevy 0.20 bundles and retained Theia commands.

`panels.rs` follows the resizable-panel interaction from `ui/widgets/panel.rs`,
with Theia-specific logical bounds, world-input shielding and cancellation.

The standard `ViewportNode` arrangement follows `ui/components/viewport.rs`;
Theia adds an explicit window UI camera, standard image target and checked
window/image coordinate conversion for its retained authoring tools.
