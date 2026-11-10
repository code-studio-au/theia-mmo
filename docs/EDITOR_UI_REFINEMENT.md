# P5-17 integrated editor usability refinement

Adopted 10 November 2026. Keep the dock on the **left**. Sprinkles supplies the
visual and interaction reference; Blender supplies the stacked explorer/properties
pattern, category tabs, viewport tool strip and compact status bar. Pixel-perfect
matching is not an acceptance requirement. The integrated editor is the product
focus. Keep retained standalone source as an internal host without a separate
Theia launcher or standalone polish/qualification gate.

## Implemented checkpoint: dock, filtering and status bar

- Explorer above properties, with a horizontal draggable divider and a shared
  draggable outer width. Each header independently folds its content. Hidden
  bodies also lose inherited visibility so keyboard navigation cannot enter them.
- Content type dropdown replaces the filter-button grid. Retain native Bevy
  menus, focus, popup placement and existing filtering/history behavior.
- A full-width, single-line 28 px status bar shows contextual tool hints and
  feedback, selected content and actual object count, plus pinned Saved/Unsaved
  state. Feedback is bounded and sanitized; invalid placement feedback takes
  priority. Keep explicit Save for disk writes.
- Remove the explorer's repeated selection/instruction footer and lengthy
  navigation help from properties. Keep authoring metadata and attributes there.
- Viewport size, pointer conversion, camera panning, dock geometry and native
  reports share the status-bar height. Footer clicks cannot start world/dock
  gestures. Resize gestures retain input ownership through release and restore
  their prior dimensions on cancellation.
- Text input uses a centered intrinsic-height line inside its existing frame.
  Theme base colors/font colors reuse the imported Sprinkles tokens directly.
- Remove the companion launcher; File → Particles workspace remains the shared
  window entry to the actual Sprinkles editor.

## Validation

`cargo fmt --check`, `git diff --check`, workspace/all-target Clippy with
`-D warnings`, and **107 zone-editor tests** pass. CPU checks include short/full
and narrow/wide inspectors, exact far-coordinate attributes, source/history/redo
preservation, input boundaries, split/width cancellation and release ownership,
status-bar bounds, centered field lines, scrolling to inspector controls, menu/
scrollbar ownership and viewport projection/picking round trips at 1×/1.5×/2× DPI.

Native Linux runs use AMD Radeon RX 7900 XTX / RADV NAVI31 / Vulkan, 3440 × 1440.
Both local and ±128-billion-metre origins pass width/split resizing, independent
folding, opening and selecting dropdown filters, footer input isolation,
duplicate/undo/explicit Save and exact source/history comparisons. Each reports
zero GPU diagnostics. Final executable SHA-256:
`4c2ef12d61fdb08601f2db65cc4f1bb6128593676469c5c1a21a0ce757930a65`.
Owned editor processes are closed after qualification. No Windows execution claim.

Evidence (ignored build output):

- `build/sprinkles-review/refinement-status-tests-final-v2.log`
- `build/sprinkles-review/refinement-status-clippy-final-v2.log`
- `build/zone-stacked-status-native-20261110-final-local/qualification.json`
- `build/zone-stacked-status-native-20261110-final-far/qualification.json`
- Each native folder includes selected/unsaved/final screenshots and dropdown
  capture, source JSON, reports and device log.

Reproduce from the repository root after building `theia-zone-editor`:

```sh
python3 tools/qualify_zone_dock.py --output build/new-dock-local --case local
python3 tools/qualify_zone_dock.py --output build/new-dock-far --case far
```

These checks use synthetic OS input guarded to the process-owned X11 window,
owned fixture documents and an isolated particle-editor data directory. They do
not open or polish the retained standalone host.

## Remaining requested refinement

This checkpoint does **not** complete the reopened UI refinement. Normal attribute
clicks still enter the existing exact form. Replace that workflow with inline
validated edits, immediate object application and explicit disk Save, retaining
f64 precision and undo/redo. Add relevant property section headings/category icon
tabs, the selected-object viewport tool strip with input isolation, and finish
three-dot accessors and popup-frame cleanup. Reuse actual Sprinkles components
where compatible; adapt exact f64 fields instead of its f32-formatting controls.
Do not expose colors/curves for zone objects whose schema has no such property;
the actual particle workspace already retains the original color/curve editors.
