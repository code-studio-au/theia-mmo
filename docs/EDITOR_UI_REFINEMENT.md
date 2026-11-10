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

## Implemented checkpoint: permanent numeric textboxes

Selected object, plant, marker and guide numeric attributes are permanent native
Bevy TextInputs, grouped under Position, Rotation, Dimensions, Scale, Area or
Path headings as appropriate. They remain visible before focus and after commit
or cancellation. Enter or leaving a field commits one validated command and
updates the object; Escape discards the draft. There is no reveal step or Apply
panel. F9/menu actions focus the existing fields; remove the redundant Transform
button. Tab cycles visible numeric fields and commits the previous field.

Keep the exact f64 strings and existing zone validation/history. A valid focused
value commits before Save writes disk; failed validation cannot write the file.
Typing alone does not write disk or create a command per keystroke. Native text
input owns keyboard shortcuts while focused; viewport gestures are withheld
until the field is finished. A document generation plus revision prevents old
text from modifying a replacement document with coincident IDs/revisions.

The reported background overflow came from the generic textbox styler adding a
second border, six-pixel padding and 28 px minimum height to an inner field.
Permanent fields use the shared framed-input marker: the 28 px outer frame owns
background/border, while the transparent inner text line has no padding/border
and stays centred within its content bounds. Actual theme/layout regressions
check this at local/far origins, plus persistent controls and section headings.

Validation: **110 zone-editor tests**, fmt and strict workspace/all-target Clippy
pass. Native local/far runs pass permanent/direct editing, Tab, exact values,
Enter, invalid input, Escape, undo/redo and focused Ctrl S, along with dock,
filtering and source-preservation checks. Same Linux GPU/device as above; zero
GPU diagnostics in both. Final executable SHA-256:
`20ee6430103a5be701dac179e2a8f4a9e559148721c1752b3b8850cead58396f`.

Evidence:

- `build/sprinkles-review/permanent-input-tests-final-v2.log`
- `build/sprinkles-review/permanent-input-clippy-final-v2.log`
- `build/zone-permanent-input-native-20261110-final-local-v3/qualification.json`
- `build/zone-permanent-input-native-20261110-final-far/qualification.json`
- Native folders include unfocused/focused and invalid-field captures. Owned
  editor processes are closed afterwards. No Windows execution claim.

Add `--inline` to the reproduction commands above to exercise direct inputs.
The earlier click-to-reveal prototype was rejected by the user and is superseded
by this implementation; its captures are not acceptance evidence.

## Implemented checkpoint: property categories — 11 November 2026

A fixed icon rail inside the left properties panel selects Attributes, Tools or
View. Reuse the actual Sprinkles cube/settings/bounds icons and theme. Each
category shares the existing scroll pane; switching resets only that pane's
scroll, preserving the explorer position. Inactive categories have both hidden
visibility and no layout so keyboard navigation cannot enter their fields.
Selection changes open Attributes; unselected/placement workflows open Tools.
Manual category choice survives source revisions and ordinary updates. F9 opens
Attributes, unfolds properties and focuses the existing exact textbox. Leaving a
numeric field via a category button applies its valid draft once, without saving
disk or showing a separate editor.

Validation: **111 zone-editor tests**, formatting, diff checks and strict
workspace/all-target Clippy pass. Whole-inspector CPU checks exercise all three
categories at narrow/wide and short/full layouts. Native local/far runs pass tab
switching, pending-field blur, F9, direct editing, explicit Save, source/history
preservation and the retained dock/dropdown checks. Linux RX 7900 XTX / RADV
NAVI31 / Vulkan at 3440 × 1440; zero GPU diagnostics in both. No Windows claim.
Executable SHA-256:
`f4bab9a98030431d76144d12d160651aae8fa973da2e18e69d92fbe36ce8256d`.

Evidence (ignored):

- `build/sprinkles-review/category-tests-v3.log`
- `build/sprinkles-review/category-clippy-v2.log`
- `build/zone-category-native-20261111-final-local/qualification.json`
- `build/zone-category-native-20261111-final-far/qualification.json`

Use `--inline --categories` with the reproduction commands above. Owned native
processes are closed after qualification. The viewport toolbar, inline name/spawn
workflows and popup refinement remain separate follow-on work.

## Implemented checkpoint: selected-content viewport toolbar — 11 November 2026

A compact strip at the viewport's upper left provides Move, Rotate, Scale, Frame
and Ground using the actual Sprinkles artwork and the existing shared commands.
It follows the dock width and appears for selected content; unsupported gizmo
modes remain disabled. Hide it for placement, terrain, modal dialogs, playtest or
windows too small to contain the strip. The world hit-test rectangle matches its
layout. Pointer gestures started in the strip retain ownership outside it and
through release; left/middle/right buttons cannot leak selection, transforms,
panning or orbiting to the viewport. Lost window focus cancels that ownership.

Validation: **112 editor tests**, fmt/diff checks, strict workspace/all-target
Clippy and editor build pass. CPU regressions cover all three mouse buttons at
local/far origins, release/focus cancellation and exact source/redo preservation.
Native local/far checks pass actual tool changes and visible gizmos, camera/source/
selection/history preservation while dragging away with every mouse button,
Frame, category switching, numeric editing and dock/dropdown/explicit Save checks.
Linux RX 7900 XTX / RADV NAVI31 / Vulkan, 3440 × 1440; zero GPU diagnostics.
No Windows execution claim. Executable SHA-256:
`fd41010e9a09865fdbb22e686781a942bafdd06936fd1d27d9b70e850b9f9517`.

Evidence (ignored):

- `build/sprinkles-review/viewport-tools-tests.log`
- `build/sprinkles-review/viewport-tools-clippy.log`
- `build/zone-toolbar-native-20261111-local-v1/qualification.json`
- `build/zone-toolbar-native-20261111-far-v1/qualification.json`

Add `--toolbar` alongside `--inline --categories` to reproduce. Both native
runs close the owned editor process. Inline name/spawn workflows and popup
refinement remain to follow.

## Implemented checkpoint: permanent content names — 11 November 2026

Objects, plants, markers and guides have a permanent Name textbox above their
numeric sections. It reuses the exact same single-frame input and guarded inline
transaction as numbers. F8 and menu Rename focus that field, open Attributes and
unfold properties; no separate name dialog is used for selected content. Keep
zone naming and search dialogs separate. Remove the redundant inspector Rename
button and duplicated name in metadata; the explorer/status context updates on
commit. Enter or blur applies one shared Rename command; Tab applies the name
and advances to the first numeric attribute. Invalid focused names remain for
correction; Escape restores source text. Explicit Save commits a valid focused
name before writing disk. Names never convert or otherwise change f64 attributes.

Validation: **113 editor tests** pass, including local/far Unicode names across
all content kinds, validation, cancellation, blur and shared undo; the added
Name→numeric Tab regression also passes. Formatting, diff checks and strict
workspace/all-target Clippy pass. Native local/far checks exercise the permanent
name, F8, Tab, explorer update, undo/redo, empty-name rejection, Escape and focused
Ctrl S, plus numeric/category/dock regressions and source/history preservation.
Linux RX 7900 XTX / RADV NAVI31 / Vulkan, 3440 × 1440, zero GPU diagnostics.
No Windows execution claim. Executable SHA-256:
`8eeba636287a9c2c892017c672d8f1ef3b1122af5e8859435fff5712571fde24`.

Evidence (ignored):

- `build/sprinkles-review/inline-name-tests-v4.log`
- `build/sprinkles-review/inline-name-tab-test.log`
- `build/sprinkles-review/inline-name-clippy-final.log`
- `build/zone-inline-name-native-20261111-local-v1/qualification.json`
- `build/zone-inline-name-native-20261111-far-v1/qualification.json`

Use `--names --inline --categories` to reproduce. Both native runs close their
owned editor. Spawn attributes and action/popover cleanup remain to follow.

## Implemented checkpoint: inline player spawn — 11 November 2026

Player spawn is selectable in the explorer and retains its existing single-click
camera framing. Its exact world X/Y/Z coordinates appear as permanent textboxes
under Position in the left Attributes tab. Zone → Edit spawn selects it, unfolds
the inspector and focuses the first field. The Name field and selected-object
viewport toolbar are hidden for spawn; selecting an object clears spawn selection.
Enter/blur/Tab apply validated edits through the existing SetSpawn command,
Escape cancels, undo/redo is shared and explicit Save commits a valid focused
field first. World positions retain f64 precision. The retained exact form is
still available to internal regression fixtures; the integrated spawn workflow no
longer opens it.

Validation: **114 editor tests**, fmt/diff checks and strict workspace/all-target
Clippy pass. Headless Bevy input/focus tests cover local/far exact values, the
menu request, explorer selection transitions, three visible fields, validation,
undo/redo, unchanged disk until Save and focused Save. CPU layout/scroll checks
also include spawn at narrow/wide and short/full sizes.

With the user's mouse and keyboard reserved, isolated Xvfb startup captures were
rendered without OS input on **llvmpipe (LLVM 21.1.8, 256 bits), Vulkan** at local
and far origins, both zero GPU diagnostics and unchanged source. These are
software-rendered startup smoke checks; they do not show the selected spawn or
qualify native input. The RX 7900 XTX could not present to Xvfb because the virtual
server lacks DRI3; hardware and native spawn interaction checks remain pending.
Executable SHA-256:
`579517e6b2eaafbadc50fcec4b68618f4dba4f57d88688f534f2a861c845c868`.

Evidence and reproduction (ignored output; Xvfb is extracted locally):

- `build/sprinkles-review/inline-spawn-tests-final-v2.log`
- `build/sprinkles-review/inline-spawn-clippy-final.log`
- `build/zone-inline-spawn-virtual-gpu-20261111-local-lvp-v1/qualification.json`
- `build/zone-inline-spawn-virtual-gpu-20261111-far-lvp-v1/qualification.json`
- `build/zone-inline-spawn-virtual-gpu-20261111-local-v1/editor.log` records the
  failed hardware presentation attempt. Virtual-server and editor processes close
  on success and failure.

`tools/qualify_zone_virtual_gpu.py --xvfb PATH --output NEW_DIR --case local|far`
provides the input-free startup smoke check. Set `VK_ICD_FILENAMES` to the local
software Vulkan ICD when the virtual server lacks DRI3. A future native check is
prepared with `tools/qualify_zone_dock.py --spawn --inline --categories --names`;
its spawn path has only been syntax/help checked, not executed.

## Remaining requested refinement

This does **not** complete the reopened UI refinement. Finish three-dot accessors and popup cleanup.
The retained exact form serves internal regression fixtures; it is not the normal
selected-object or spawn numeric workflow.
Reuse actual Sprinkles components where compatible; retain exact f64 adapters
instead of its f32-formatting controls. Do not expose colors/curves for zone
objects whose schema has no such property; the actual particle workspace already
retains the original color/curve editors.
