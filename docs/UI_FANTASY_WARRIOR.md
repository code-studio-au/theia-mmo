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
role, content identity, protocol, authority or equipment/ability rule. It is not
yet offered by the normal launcher or enabled by default. Refuse missing/repeated
flags, incomplete source packs, LFS pointers, wrong PNG dimensions, oversized
files and source paths escaping the selected directory before creating the App.
The preflight checks headers and font signatures; actual loaders/rendering are
separate evidence. It does not cryptographically seal mutable local art.

The candidate loads four images and two fonts through a separate `fantasy-ui://`
Bevy asset source. Existing terrain/player/creature asset roots are preserved.

- Player health/mana/life at upper left; selected-target information alongside it.
  The current target heading and health remain separate existing widgets.
- Twelve familiar bottom action positions with compact 56 px square framed icons,
  existing hotkey labels, bounded cooldown strips and hover descriptions/costs/
  authoritative rejection messages. Only the existing first five supported action
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
exports do not replace that gate. Native Windows, arbitrary DPI/accessibility
scaling, performance and full-theme integration are not established by this slice.

Subsequent general UI slices can combine the target heading/health into one unit
frame, improve backpack item icons and tooltips, add a consistent window header/
close treatment, and replace debug status rows with readable contextual feedback.
Retain the qualified gameplay behavior and expose no placeholder chat, party,
character, currency or equipment controls as working systems.
