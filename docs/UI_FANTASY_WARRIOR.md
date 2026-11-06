# General UI: Fantasy Warrior assets and Classic layout

6 November 2026. P2-14 / P5-13 / P5-19, under UI-001.

The user moved development beyond quest mechanics to a general WoW-like UI.
This first slice is an **opt-in presentation candidate**. The existing playable
configuration remains available while desktop/mouse testing is paused. The
unfinished optional full quest-log draft is parked locally; it is outside this UI slice.

## Asset assessment

The supplied folder is `/home/steven/Downloads/Synty Models/HUDs`, rather than
`SYNTY/UI`. Its `User_Guide.pdf` identifies **Interface Fantasy Warrior HUD 1.0.0**,
created for Unity 2022.3.0f1 and a 3840×2160 design reference. Manual pages 2–9
explain PNG sprites, height-based scaling, anchors, nine-slicing, modular decorative
pieces and Unity prefab animations. These are source references; Unity instructions
are not commands to run or dependencies for Theia.

The local audit finds **1,866 PNGs / 211,955,425 encoded bytes**, two TrueType
fonts with accompanying SIL OFL 1.1 notices, and **zero Unity `.meta` files**.
PNG frames/icons and TrueType fonts work with Bevy's standard loaders. Prefabs,
UGUI/TextMeshPro components, Unity animation controllers and additive/multiply
materials are reference designs, not directly executable Bevy widgets. The pack's
radial menus, reticles and controller prompts do not change the adopted Classic
controls. No Unity Editor is required for this slice.

Selected PNGs total **143,415 encoded bytes**, with **2,097,152 bytes of RGBA8
base-level pixel payload**. This is a source/pixel accounting result, not measured
GPU allocation, frame-rate or batching performance. The four images use original
aspect ratios; square frames remain square. Missing Unity border metadata is not
reconstructed as guessed nine-slice settings. No atlas is needed for this small
subset; the manual's atlas suggestion can be reconsidered against measured demand.

| Role | Source path relative to HUDs | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| slot_frame | `Sprites/FantasyWarrior/SPR_FantasyWarrior_Frame_Box04.png` | 512 × 512 | `0b5364d52614b58b5a7394362f7363b1f398fa00992d273049f51add64e6a3ed` |
| sword_slash | `Sprites/Icons_Weapons/ICON_SM_Wep_Sword_07_Clean.png` | 512 × 256 | `88282f72dce6c3602baea8ffd2cb518c49c899580913512baaac5774fc3c046c` |
| arcane_bolt | `Sprites/Icons_Elements/ICON_FantasyWarrior_Element_Air02_Clean.png` | 256 × 256 | `58814b493756cb134599003e3759eafb21cfa3498f5a6dafc8a2743e435443b7` |
| fallback | `Sprites/Icons_Inventory/ICON_FantasyWarrior_Inventory_Spellbooks01_Clean.png` | 256 × 256 | `59a771b91ee76e9ac4c817bb4dc2c21e2d9a35bcba8c9dd7618071321b12ff8b` |

`Fonts/Grenze/Grenze-SemiBold.ttf` supplies headings at 18 px and above;
`Fonts/Texturina/Texturina_18pt-SemiBold.ttf` supplies smaller interface text.
Both stay in the user's local source directory with their OFL notices. Assets,
fonts and the manual have not been copied into Git. The source PNGs are stylized
white silhouettes; arcane bolt's air/swirl icon is an explicit provisional Theia
mapping, sword slash uses the sword icon, and unknown abilities use a spellbook.

## Implemented candidate

Append `--fantasy-ui HUD_DIRECTORY` to an existing valid predicted Classic combat
client command. The flag is presentation-only and is not a new gameplay manifest
role, content identity, protocol, authority or equipment/ability rule. The normal `tools/play.py` launcher accepts the same optional flag, passes it
only to the client and keeps it separate from sealed gameplay profiles. It is
not enabled by default. A missing directory or incompatible mode refuses before
starting children; the Rust client performs complete selected-art preflight. Refuse missing/repeated
flags, incomplete source packs, LFS pointers, wrong PNG dimensions, oversized
files and source paths escaping the selected directory before creating the App.
The preflight checks headers and font signatures; actual loaders/rendering are
separate evidence. It does not cryptographically seal mutable local art.

The candidate loads four images and two fonts through a separate `fantasy-ui://`
Bevy asset source. Existing terrain/player/creature asset roots are preserved.

- Player health/mana/life at upper left; the selected-target heading and health
  share one frame alongside it. The heading still comes from the existing
  authoritative-selection presenter; the skin does not resolve a new target.
- Twelve familiar bottom action positions with compact 56 logical-pixel square framed icons,
  existing hotkey labels, bounded cooldown strips and hover descriptions/costs/
  authoritative rejection messages. Tooltip geometry stays clear of cast/rejection
  feedback; the windowless exporter checks their separation. Only the existing first five supported action
  bindings can activate configured abilities; unused positions are noninteractive
  decoration inside a pointer-blocking bar. This does **not** implement 1–= binding
  configuration, drag/drop actions or ten new abilities.
- Cast/autoattack feedback above the action bar. No attack range, facing, cast,
  cooldown, manual autoattack or keyboard behavior is changed.
- Shared charcoal/bronze pane styling for backpack, dialogue, loot, note reader and
  minimap. Inset outlines preserve existing content dimensions and scroll state.
- Quest tracker below the existing upper-right minimap, progression below player
  vitals, selected statuses below target information and a compact lower-left
  connection indicator. Loading/disconnection details remain available.
- Standard Bevy widgets, picking, image/font loaders and rendering; no custom UI
  renderer, Unity runtime or new server component.

## Windowless review and evidence

A developer CLI renders **actual combat HUD widgets** with explicitly synthetic
health/mana/target/cooldown values to a GPU image. It installs no Winit or audio
plugin and creates no OS window. It uses no account, Network resource, game command
collection, server, desktop pointer or keyboard. This is a visual preview, not a
live multiplayer screenshot or an interaction test.

```bash
env -u DISPLAY -u WAYLAND_DISPLAY target/debug/theia-client \
  --preview-fantasy-ui '/home/steven/Downloads/Synty Models/HUDs' \
  --output build/fantasy-ui-review --size 1920x1200
```

Use the repository native-SDK `LD_LIBRARY_PATH` when needed. A fresh output path is
required. Exports contain `hud.png` and `report.json`, including the real adapter,
zero window count, actual action bounds and laid-out text glyph count. The exporter
waits for all selected art/fonts and refuses zero-sized/offscreen actions or absent
text. A marked hover fixture displays the real tooltip without OS input. The
90-second deadline prevents an indefinite screenshot wait.

Local evidence lives under `build/ui-audit-20261106/`: manual extraction, asset
hash/dimension audit, source contact sheet, headless test/build/Clippy logs and
windowless captures. The first export was blank because an image target needs an
explicit default UI camera; it is retained as failed diagnostic evidence, not a
qualification. Later exports require real UI geometry and glyphs before success.

The final v3 windowless exports pass at **1280×720, 1920×1200,
3440×1440 and 3840×2160**. Each records two real 56×56 configured action buttons,
208 laid-out glyphs and zero windows. Actual GPU: **AMD Radeon RX 7900 XTX /
RADV NAVI31 / Vulkan / Mesa 26.2.6 (kisak PPA)**. The four logs have no WARN/ERROR
diagnostics. The fixtures show partial health/mana/cooldown and the real tooltip.

`cargo fmt --all --check`, denied-warning workspace/all-target Clippy,
**159 client tests including the explicitly enabled local licensed-pack PNG
loader case**, and the development client build pass. Default tests can omit
that one local-source test. The added behavior checks cover compact widget
activation/feedback, cosmetic geometry/scroll/visibility preservation, argument
refusal and invalid source headers. This is Linux execution, not Windows evidence.
No server/persistence changes were made and no services or desktop tests ran.
The normal live game/pane checks below remain pending.

## Follow-on UI work

Before default adoption, qualify the complete normal connected HUD and panels at
1280×720, 1920×1200 and 3440×1440, including actual hover/click/camera ownership,
panel/Escape priority, long authored text, item capacity, reader and disconnect
states. **Desktop/mouse checks remain prohibited by the user**; windowless GPU
exports do not replace that gate. Native Windows, OS DPI/accessibility certification, performance and full-theme
integration are not established by this slice. Controlled UI scale checks below
cover GPU fixture layout, not live pointer behavior.

Subsequent general UI slices can improve backpack item icons and tooltips,
add a consistent window header/
close treatment, and replace debug status rows with readable contextual feedback.
Retain the qualified gameplay behavior and expose no placeholder chat, party,
character, currency or equipment controls as working systems.


## Unified frame and launcher refinement

The next local refinement combines the previously separate target heading and
health into one standard Bevy frame. The normal targeting presenter writes the
same heading marker under the combat frame; the legacy skin retains its original
standalone heading. The exporter now previews that actual shared frame rather
than a separate illustrative heading box.

`tools/play.py --help` advertises `--fantasy-ui`. Append the following to an
existing valid Classic gameplay launch command, using rebuilt binaries:

```text
--fantasy-ui '/home/steven/Downloads/Synty Models/HUDs'
```

It stays out of server arguments and gameplay manifest roles/hashes. Fifty
headless launcher/bundle/environment tests pass, including actual Rust manifest
validation, option handoff through the ordinary launcher parser, paths with
spaces and refusals before child startup. The accepted launcher call is observed
with a substituted launch function; no real server, browser, client window or
mouse is started by these tests. A live launcher/game interaction gate remains.

The v4 visual review exposed a tooltip overlapping cast/rejection text. The
refinement moves the tooltip above that feedback and makes the image export
refuse overlap using the actual laid-out rectangles. All four **v5** exports (1280×720, 1920×1200, 3440×1440, 3840×2160)
pass the actual feedback-separation, button-bound, glyph and zero-window checks
on the same RX 7900 XTX Vulkan device, with no WARN/ERROR diagnostics. Formatting,
strict workspace/all-target Clippy, 159 client cases and build pass again after
the refinement. The real launcher CLI also refuses a missing source directory
before any child startup. Earlier v3 evidence belongs to the first local skin
checkpoint. No desktop, account or live gameplay qualification is claimed.


## Preferred UI scale and resize fitting

The client and normal launcher accept optional `--ui-scale FACTOR` beside
`--fantasy-ui`, with a default preference of 1.0 and accepted preferences from
0.75 to 1.5. Nonfinite, out-of-range, missing/repeated client values and a scale
without the skin refuse before client App creation; launcher validation rejects
invalid factors before child startup. The same option works at the end of the
windowless preview command.

The implementation uses Bevy's `UiScale`, with a preferred value kept separate
from the applied value. The applied scale is the lesser of the preference,
window width / 1280 and window height / 720. Resizing/fullscreen can therefore
reduce the interface to retain the established logical layout; it restores the
preference when room returns. It never resizes a window, moves the camera or
changes world/gameplay units. A minimized/zero-sized window does not set a zero
UI scale. Very small windows can reduce below the requested 75% minimum; normal
1280×720 and larger review sizes remain the supported readability targets.
This is a client CLI preference, not a saved in-game settings screen.

| Windowless extent | Preferred scale | Applied scale | Actual action size |
| --- | --- | --- | --- |
| 1280×720 | Default 1.0 | 1.0 | 56×56 px |
| 1280×720 | 0.75 | 0.75 | 42×42 px |
| 1280×720 | 1.5 | 1.0 (fit) | 56×56 px |
| 1920×1200 | 1.25 | 1.25 | 70×70 px |
| 3440×1440 | 1.25 | 1.25 | 70×70 px |
| 3840×2160 | 1.5 | 1.5 | 84×84 px |

All six final `scaled-v2-*` exports pass on the recorded RX 7900 XTX Vulkan
device, with actual physical-pixel bounds, visible glyphs, separated tooltip/
feedback rectangles, zero OS windows and no WARN/ERROR diagnostics. Reports now
record preferred/applied scale and declare geometry units as physical pixels.
The preliminary `scaled-v1-*` captures predate resize fitting and are diagnostic
iterations. The 1280×720 / 150% case motivated retaining the logical layout for
other panes as well as the combat preview.

Formatting, strict workspace/all-target Clippy, **161 client tests including the
local licensed-art case**, **51 launcher/bundle/environment tests** and the client
build pass. A headless ECS Window-component test proves shrink/restore behavior
without installing Winit or creating an OS window. It is not a native window/
monitor/DPI test. Full connected pane layout and scaled pointer interactions
remain pending. Existing OS-test traces assumed UI scale 1.0; their UI-to-window
coordinate conversion must be updated before nondefault-scale pointer checks.


## Avoid unchanged combat-widget writes

A headless change-detection probe of `combat::ui::paint` found **six Text,
five Node and two BackgroundColor changes on each steady legacy frame**, and
**six Text, seven Node and two BackgroundColor changes** with the compact skin.
The extra two compact Nodes were unchanged tooltip display states. The before
failure is retained in `idle-widgets-before-v1.log`.

The painter now compares label contents, fill widths, tooltip display and button
colours before assigning. The same probe reports **zero changes in all three
categories** for both skins after settling, while a new health/connection state
still changes the expected widgets and displays the new value. This measures
Bevy change notifications for this painter; other HUD writers, CPU/GPU timings,
frame-rate improvement and crowded gameplay performance are not measured here.

The six `idle-v1-*` offscreen images are **RGBA-pixel-identical** to their
`scaled-v2-*` references, retaining scaled geometry, feedback separation, fonts
and zero windows on the recorded Vulkan device. No WARN/ERROR diagnostics appear.
Formatting, strict workspace/all-target Clippy, **162 client tests including the
local art case**, and build pass. Existing activation/hover/feedback checks also
pass. No desktop or mouse test ran; full connected interaction remains pending.


## Scaled world-projected labels and newly spawned fonts

NPC quest markers and floating combat text now convert camera/window projection
coordinates into logical UI coordinates before assigning absolute Node offsets.
Marker clipping converts the viewport into that same coordinate frame. Without
this division, applying UI scale would move the label away from its world anchor.
Scale 1.0 preserves the original placement; marker eligibility, occlusion,
combat facts, label lifetimes and world-coordinate/origin conversion are unchanged.
Invalid/nonfinite projection inputs refuse presentation rather than publishing
invalid UI positions. This is UI integration, not additional quest mechanics.

Theme fonts now apply to newly added UI text before Bevy's UI content/layout
stage, including text spawned after startup. A headless test checks initial and
later-created labels with distinct heading/body font handles. Existing text is
not restyled every frame.

The six `projection-v1-*` windowless exports add an explicitly marked projected
anchor fixture. Actual rendered anchor centres remain within **0.6 physical
pixels** of their requested window points across the tested scales/resolutions;
the check allows 1.5 px for layout rounding. This exercises the conversion and
real Bevy layout, not a live NPC camera/occlusion scene. The upper HUD and lower
action/feedback bands remain pixel-identical to the corresponding `idle-v1-*`
references; the new middle fixture is intentional. All six retain zero windows,
valid actions/glyphs, feedback separation and no WARN/ERROR diagnostics on the
same Vulkan adapter. Formatting, strict workspace/all-target Clippy,
**164 client cases including local art decoding**, and build pass.

Real moving-NPC markers, floating combat text under gameplay, complete panels
and scaled OS pointer checks remain pending while desktop testing is prohibited.


## Shared minimap bezel and actual widget preview

The opt-in skin now places the existing square slot frame over the live map
surface, sharing the already loaded PNG rather than adding a texture. The
176×176 logical raster, north/heading arrow, guide indicator, resident-terrain
sampling and world extent retain their existing implementation. The decoration
ignores picking and does not resize the raster. This is a square fantasy bezel,
not a circular/masked map implementation.

The developer exporter installs the **same minimap setup** and seeds an explicitly
labelled checker raster, centered arrow and guide marker. This preview-only
function is never installed by normal gameplay; it installs no World, terrain
streaming, map-present logic or networking. Reports label the synthetic raster
and include the actual laid-out minimap bounds.

All six `minimap-v1-*` windowless cases retain the complete map inside the target
through 3840×2160 / 150%, alongside the previously qualified scale/anchor/feedback
checks. Existing player/target and lower action/feedback image regions remain
pixel-identical to their `projection-v1-*` references. No WARN/ERROR diagnostics
appear on the same RX 7900 XTX Vulkan device. Formatting, strict workspace/
all-target Clippy, 164 client cases including local art and the build pass.
Live terrain/heading/input and the complete normal connected theme remain for
the desktop gate. The image preview is not additional terrain-streaming evidence.
