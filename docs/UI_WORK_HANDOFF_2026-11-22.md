# UI work handoff — 22 November 2026

## Resumed 23 November

The user explicitly resumed work and authorised autonomous local commits and
continued backlog work, including mouse/display access. This historical checkpoint
has now been completed, including the subsequent full-window, active-text,
section-padding and character drag/zoom requests. Final results and exact scope
are in `UI_SPRITE_INTEGRATION.md`; consult `STATUS.md` for current work. The stop
instructions below describe the 22 November checkpoint, not an active restriction.

## Historical stopping point

The user requested a stop. Work is saved in the working tree on
`dev/quest-inventory-integration`; **the integration/refinement changes are not
committed or pushed**. This note records the checkpoint only; it does not resume
implementation. The last release client build completed successfully. The final
test compilation was cancelled at the user's request, before its results were
available. No owned test or build should be restarted until the user resumes work.

Read the repository contributor instructions and current architecture/alignment
records when resuming. Do not reset the dirty working tree. It includes earlier
unfinished reference-UI work that this integration depends on.

## User decisions to preserve

- Integrate the approved regenerated v3 sprites into native game screens and HUD.
- Never use `Theia_UI_Kit_v1.zip` or the rejected Downloads `contact_sheet.png`.
  Approved provenance is the final sprite package and its v3 regeneration.
- Login stays inside the game; a future launcher handles updates, not login.
- Unsupported settings/features remain visible, greyed out and disabled.
- Icons inside controls should be standalone symbols, without button backings.
- Keep settings panels a fixed size with bottom-anchored actions, fitting the
  display. Do not shrink the window when switching to a shorter tab.
- The newest character-customisation reference is a guide for **refining the
  current compact layout only**. The user explicitly declined rebuilding it with
  the reference's additional sidebars and option grids.
- Local commits were previously authorised; pushing was not. The stop request
  took precedence, so no integration commit was made.

## Implemented in the working tree

### Shared artwork integration

`apps/client/src/ui_theme/sprites.rs` embeds the nine v3 atlas pages and manifest,
using standard Bevy ImageNode/TextureSlicer, explicit rectangles, sRGB/linear
sampling and the package's four texture pixels per UI unit. Shared theme controls
select disabled/pressed/selected/focused/hover/default sprites, preserving real
Bevy input ownership and disabled semantics.

Panels, title plates, buttons, fields, roster rows, tabs, checkboxes, sliders,
inventory/action slots, item/ability icons, stat fills and minimap trim use v3.
Earlier reference layouts for settings, party, inventory, journal, chat/combat
log and menus are also part of this dirty working tree. Unsupported controls
remain disabled; this does not implement new gameplay or production authentication.

### Latest requested corrections

- Fixed invisible typed username and character-name text: field artwork is an
  ImageNode on the TextInput entity, beneath its own glyphs. A child image had
  painted above the parent text despite its negative ZIndex.
- Username/password now share identical outer sizing. Username, password,
  character-name and chat input use shared defaults: 48 logical pixels high,
  form-relative width, consistent padding with space reserved for embedded icons.
- Generated twelve standalone gold symbols matching the existing account bust:
  lock, eye, settings, support, power, graphics, controls, keyboard, interface,
  accessibility, language and rotation. The interface icon's unwanted pale blur
  was removed and its bounds corrected. Account and lock symbols stay static;
  eye and Settings/Support/Exit brighten on hover.
- Settings sidebar symbols use bare icons rather than framed button sprites.
- Title artwork uses BorderBox, and section text has 64px horizontal clearance.
- Disabled checkboxes share Vertical Sync's 22px frame, 32px row and 17px label.
  Disabled dropdowns share the live View Distance dropdown's 32px height, frame
  family, arrow location and text insets. Disabled presentation/input remains.
- Both settings surfaces share a growing content body and non-shrinking bottom
  footer, keeping Restore Defaults / Apply / OK / Cancel fixed across tabs.
- Removed square hover/focus outlines around checkbox/slider rows. Slider focus
  brightens its diamond instead; keyboard control remains functional.
- Name-format feedback is a small `- invalid name!` beside Character name, using
  the existing CharacterName parser. It clears for empty or valid input. Local
  format errors no longer display the ASCII policy explanation in the global
  message area; unrelated server/account errors remain available.
- Replaced Turn left / Turn right with one rotation icon. Primary mouse drag
  rotates preview yaw, including outside the icon while held, and stops on
  release. Left/Right keys work when the control is focused. Empty accounts do
  not show the preview control. Gameplay facing/camera is untouched.

### Final small polish after the successful native run

These edits are in the successful final release build but **have not had their
final native visual check or completed final test rebuild**:

1. The compact creation panel uses the shared decorative section title bar for
   “Create Character” instead of its previous plain heading/divider.
2. Added `Style::Icon` for the eye and rotation controls so keyboard focus cannot
   draw a square outline around them. Focus instead highlights the bare symbol.

## Artwork and important files

- Approved main package: `content/ui/sprites-v3/` (previously committed in
  `abcbd1cb`, before integration).
- Supplemental icons: `content/ui/theme/standalone-v3/`; twelve 128×128 RGBA PNGs.
  `generation.json` contains exact built-in image-generation prompts, style
  reference and source/final hashes. Deterministic cleanup preserves gold/bright
  highlights, removes stray shadow pixels, and fits art into a centred 112px area.
- Local raw icon masters and finishing script:
  `build/ui-sprites-integration-v3/standalone-masters/` and `finish-icons.py`.
- Shared theme: `apps/client/src/ui_theme.rs`, `ui_theme/sprites.rs`.
- Shared reference widgets: `apps/client/src/fantasy_ui/kit.rs` and `kit/menu.rs`.
- Frontend: `apps/client/src/frontend.rs`, `frontend/ui.rs`, `frontend/login.rs`.
- Settings: `apps/client/src/fantasy_ui/settings.rs` and shared kit helpers.
- Required embedded fonts and licence: `content/ui/fonts/` (currently untracked).
- Integration scope/evidence draft: `docs/UI_SPRITE_INTEGRATION.md`.
- Native qualifier: `tools/qualify_game_frontend.py`.

Untracked `kit-icons-v2.png`, `panel-v2.png`, `secondary-v2.png` and
`selected-v2.png` under `content/ui/theme/` are earlier local experiments, **not
runtime dependencies**. Do not accidentally stage them with the final integration.
Preserve them unless their deletion is separately appropriate. Historical
provenance is labelled accordingly.

## Verification already completed

- Client suite: **350 passed, 5 existing ignores**, before the final two small
  polish edits above. Log: `build/ui-sprites-integration-v3/tests-native.log`.
- Denied-warning Clippy passed before those final edits:
  `build/ui-sprites-integration-v3/clippy-final.log`.
- Formatting and diff whitespace checks passed during the work; rerun at finish.
- Final release build, including the last two edits, succeeded:
  `build/ui-sprites-integration-v3/build-final.log`.
- Full native fresh-account **and** restarted-account flow passed at 1920×1080
  and 1720×700, with **zero GPU validation diagnostics**:
  `build/ui-sprites-integration-v3/native-final-controls/qualification.json`.
  Device: Linux AMD RX 7900 XTX / RADV NAVI31 / Mesa 26.2.7 / Vulkan.
- Native checks cover invalid/valid login, visible typed username/name pixels,
  equal login field dimensions, settings mouse/keyboard controls and Apply/Cancel,
  unchanged footer geometry between tabs, empty/full roster, eight saved heroes,
  appearance persistence, drag rotation both directions and stopping on release,
  inline-name error clearing on deletion/valid edit, same-window world entry,
  inventory, quests, abilities, party, menu and chat/combat-log screens.
- Twelve supplemental icons were visually reviewed. The first eleven had
  automated RGBA, transparent margin, size and hash checks; rotation was then
  generated with the same finishing pipeline and inspected separately.
- No Windows execution or performance qualification is claimed by these checks.

Earlier `native-polished/` stopped because the test helper selected an invalid
name but did not actually delete it when asked to type an empty string. The helper
now sends BackSpace after Ctrl+A; the subsequent full native run passed. This was
a harness defect, not a remaining name-validation failure. Earlier evidence
folders are retained for diagnosis; use `native-final-controls/` as the successful
full-flow reference.

## Resume checklist

1. Confirm the user has resumed. Inspect status/diffs; preserve all saved work.
2. Complete the final test rebuild, run its binary with local socket permission,
   and rerun formatting/denied-warning Clippy for the final source.
3. Run a final native fresh-account check of the creation title at both compact
   and normal sizing as needed, and verify the eye/rotation controls never gain
   square focus frames. Existing full-flow evidence need not be repeated beyond
   what the final edits or any new failures justify.
4. Inspect final screenshots for text clipping and control overlap. In particular,
   the effects column's reduced row gap keeps its last checkbox above the footer.
5. Finish `docs/UI_SPRITE_INTEGRATION.md` (it still ends with a pending final-evidence
   sentence), prepend the implemented/qualified result to `docs/STATUS.md`, and
   reconcile other affected docs with actual evidence rather than plans.
6. Review and locally commit the related integration/refinement files, including
   shared kit modules, atlas loader, fonts/licence and new standalone icons.
   Exclude build outputs, fixture credentials, raw private state and unused v2
   experiments. Do not push.

## Useful commands when resumed

Use the following environment for Cargo commands:

```sh
export PKG_CONFIG_PATH=/home/steven/dev/theia-mmo/build/native-sdk/usr/lib/x86_64-linux-gnu/pkgconfig
export LIBRARY_PATH=/home/steven/dev/theia-mmo/build/native-sdk/usr/lib/x86_64-linux-gnu
cargo fmt --all --check
cargo clippy --release --locked -p theia-client -p theia-server -p theia-zone-editor -p theia-nav-bake -p theia-network --bin theia-client -j2 -- -D warnings
cargo test --release --locked -p theia-client -p theia-server -p theia-zone-editor -p theia-nav-bake -p theia-network --bin theia-client --no-run -j2
```

Run the test executable printed by Cargo with the required sandbox permission;
some tests bind local UDP sockets. The last binary path was
`target/release/deps/theia_client-d07005d7a2a3ebc2`, but use the rebuilt path.

The native qualifier owns its temporary provider, database and game process.
Use a new output directory; it refuses to overwrite existing evidence. GPU,
display and local-network access require escalation. Example fresh-only check:

```sh
env DISPLAY=:0 XDG_RUNTIME_DIR=/run/user/1000 python3 tools/qualify_game_frontend.py --once --output build/ui-sprites-integration-v3/native-last-polish
```

Do not print fixture credentials or diagnostic files containing secrets. Do not
restart old cancelled exec sessions or stop unrelated user applications.
