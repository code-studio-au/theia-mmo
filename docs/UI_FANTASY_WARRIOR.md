# General UI: Fantasy Warrior assets and Classic layout

6 November 2026. P2-14 / P5-13 / P5-19, under UI-001.

The user moved development beyond quest mechanics to a general WoW-like UI.
This first slice is an **opt-in presentation candidate**. The existing playable
configuration remains available pending complete-theme qualification. The user
authorized desktop/mouse checks for the completed native evidence below, then
paused desktop/mouse tests again while using the machine. Continue headless only;
prior permission statements below describe their historical checkpoints. The
unfinished optional full quest-log draft is parked locally; it is outside this UI slice.

## Adopted visual reference — 6 November 2026

The user supplied [this AntonilzUI WoW Classic screenshot](https://www.warcrafttavern.com/wp-content/uploads/2019/08/AntonilzUI-WoW-Classic-UI-Addon-Enemy-Nameplates-10.jpg)
as the interface direction. This is a particular addon layout, rather than a
requirement to reproduce the stock Classic interface. The screenshot is a visual
reference, not supplied artwork to redistribute or evidence of implementation.

Target composition:

- Keep the centre and most of the world view unobstructed. Place compact player
  and selected-target frames to the lower left and lower right of the character,
  above the bottom action area. Anchor these around the screen centre so an
  ultrawide display does not push combat information to distant edges.
- Put a narrow primary action bar at bottom centre, with clear ability images,
  key labels and cooldowns. Place the cast bar immediately above it. Show XP as
  a slim strip near the bar rather than a large permanent information panel.
- Use a circular minimap at top right and compact quest objectives below it;
  keep tracker backgrounds unobtrusive. Reserve bottom left for eventual chat
  and bottom right for eventual bag/menu controls as those systems are built.
- Show readable enemy names and health above creatures, with an obvious selected
  target treatment. Keep world nameplates distinct from the persistent target
  frame and from floating quest markers. They do not imply camera lock-on or
  automatic facing.
- Prefer thin dark frames, restrained gold accents, readable light text and
  clear health/resource colours. Use Synty icons and decorative pieces only where
  they support this composition. The existing ornate square bezel is a candidate,
  not a requirement to theme every element around it.

**Gap at reference adoption:** the earlier opt-in draft used upper-left unit
frames, a square minimap, a separate progression panel and heavier framed action
slots. The lower-HUD refinement below replaces the unit/action/XP arrangement;
the circular-map refinement below also replaces the square minimap. Its windowless previews prove their existing layout only; they do not
show the newly adopted composition. General enemy health nameplates, circular
map masking, portraits, chat and a bag/menu strip are not established by those
previews. Do not add illustrative controls that appear functional before their
underlying systems exist.

Next presentation slice (P2-14 / P5-13 / P5-19): move and compact the existing
server-derived unit frames, reduce bar decoration and integrate the existing
cast/XP information into the lower HUD. Preserve target selection, manual facing,
ability requests, pending transactions and UI/world input ownership. Then handle
nameplates and the circular minimap as separate bounded slices. Compare actual
widgets at 1280×720, 1920×1200, 3440×1440 and 3840×2160 with the existing scale
preferences; check clipping, unit-frame/feedback separation and a clear central
view. Native connected/input qualification remains pending; desktop testing is
authorized again. This section records design direction, not completion of those changes.

## Additional panel and character-screen references — 6 November 2026

The user supplied four further inline images during implementation. They refine
visual targets; they do not supply distributable artwork or change the adopted
combat controls. Use the earlier AntonilzUI/lower-HUD composition together with
these screen-specific details:

| Reference | Adopted direction | Implementation scope |
| --- | --- | --- |
| Quest reward and backpack | Parchment quest body, dark portrait/title header, clear reward icons and compact dark bag grid | Restyle existing dialogue/reward and inventory controls; retain authoritative offers, item counts, capacity checks and pending states |
| NPC service/equipment window | Dark framed equipment slots, consistent NPC portrait header, legible service actions and money | Guide presentation styling; repair, durability and currencies require their own implemented gameplay before controls appear |
| Character/faction selection | Prominent character preview, framed portrait choices and coherent faction theming | Future character-screen presentation; no extra races, factions, classes or creation features are adopted from the picture |
| In-world combat HUD | Compact player/target frames, thin action slots, circular minimap, readable objective tracker and clear world view | Refine the current lower HUD; chat, extra bars and other systems in the image remain separate features |

Use licensed Synty parchment, frames and icons where appropriate, and Bevy's
standard UI for layout and interaction. Preserve contrast on both pale parchment
and dark panels. Portrait sourcing and the portrait/character rendering method
still need assessment; no placeholder portrait should claim to show the actual
player or NPC appearance. At that checkpoint the next presentation slice was the existing quest/dialogue
window; later sections record its implementation.

## User ultrawide captures — 6 November 2026, 18:30–18:32

Three further local reference captures are 3440×1440:
`Screenshot from 2026-11-06 18-32-19.png`, `18-31-01.png` and `18-30-32.png`
under the user's Pictures directory. They remain local visual references; do not
copy their game artwork, player names or chat into distributed assets or fixtures.

These refine the earlier AntonilzUI composition:

- Keep a broad unobstructed central view. Compact lower player/target frames sit
  farther apart on ultrawide, with clear health/resources and eventual real
  portraits/level badges. The current fixed centre-relative frame gap is a
  candidate to refine, not a claimed match to this reference.
- Use a small legible cast bar immediately above the bottom action strip, thin
  XP treatment, compact square ability/utility/bag icons and restrained gold/bronze
  borders. Decorative ends are optional; avoid making the entire HUD ornate.
- Keep the circular map and compact objectives at top right, eventual chat at
  bottom left, and a compact combined backpack above the bottom-right controls.
  Tooltip backgrounds should be dark, readable and separate from core combat data.
- Retain clear floating damage/XP feedback, distinct selected-target treatment and
  readable world names. Friendly/hostile colour and level data require actual
  authority metadata; do not infer new gameplay facts from screenshot colours.

The pictures include chat, party frames, search, locked bag slots, currency and
other addon controls. They guide future presentation after the corresponding
systems exist; no placeholder control should appear functional in the meantime.
Keep Theia range labels metric. Do not change Classic controls, combat formulas,
item capacity or add unimplemented
systems to fill the reference strip. Continue the current reward-icon work, then
refine density/spacing and confirmed-data displays in bounded UI slices.
The latest instruction still prohibits desktop/mouse testing while the user
uses the machine; these supplied image files can be reviewed without desktop input.

## Reader, gossip and reward-detail captures — 6 November 2026, 18:34–18:37

The user supplied five more 3440×1440 Pictures-directory references, at 18-37-35,
18-36-54, 18-34-37, 18-34-28 and 18-34-20. They show a compact left parchment
reader with page controls under its title, a concise NPC greeting/quest list,
and a dedicated completion detail with item cards and an explicit completion
button. Use their proportions and hierarchy together with the preceding HUD
references. Keep a bounded left panel rather than stretching prose over the world.

The reward-choice highlight is a future choice-reward requirement to assess,
not implemented by the current fixed reward icons. Those grants remain a whole
fixed bundle. Portraits, NPC speech captions, voice playback/Stop controls and
addon-specific behaviour in these pictures likewise need separate real support;
do not add inert controls or infer gameplay from them. The current reader's
page navigation stays local and grants no extra read credit. These references
and their chat/player artwork stay outside Git; no story text is imported.

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

The current nine selected PNGs total **954,838 encoded bytes**, with
**18,350,080 bytes of RGBA8 base-level pixel payload**. Four item illustrations
are 1024-square source images; this accounts for most of that payload. This is
source/pixel accounting, not measured GPU allocation, frame rate or batching
performance. The initial ornamental square frame is no longer loaded or required.
Thin frames and the circular map use standard Bevy UI styling. Missing Unity
border metadata is not reconstructed as guessed nine-slice settings. Asset
resampling/atlasing can follow measured need; the rest of the pack is not loaded.

| Role | Source path relative to HUDs | Dimensions | SHA-256 |
| --- | --- | --- | --- |
| sword_slash | `Sprites/Icons_Weapons/ICON_SM_Wep_Sword_07_Clean.png` | 512 × 256 | `88282f72dce6c3602baea8ffd2cb518c49c899580913512baaac5774fc3c046c` |
| arcane_bolt | `Sprites/Icons_Elements/ICON_FantasyWarrior_Element_Air02_Clean.png` | 256 × 256 | `58814b493756cb134599003e3759eafb21cfa3498f5a6dafc8a2743e435443b7` |
| fallback | `Sprites/Icons_Inventory/ICON_FantasyWarrior_Inventory_Spellbooks01_Clean.png` | 256 × 256 | `59a771b91ee76e9ac4c817bb4dc2c21e2d9a35bcba8c9dd7618071321b12ff8b` |
| training-stone | `Sprites/Icons_Resources/ICON_SM_Item_Rock_01.png` | 1024 × 1024 | `1413b9a56fa96fb910e04c1b261e26a1d9118e219b5799dd3476ac93a5473c57` |
| training-token | `Sprites/Icons_Inventory/ICON_FantasyWarrior_Inventory_Currency01_Clean.png` | 256 × 256 | `903bf21c0bfdee50c2534d6acdfc0ac35e7a13770e91b89dfd1a9bc1f8bd5483` |
| training-sword | `Sprites/Icons_Weapons/ICON_SM_Wep_Sword_07.png` | 1024 × 1024 | `d5802f4c836ba38d74d56e9ba72589f3c445c99eabae0091310e3dadd7ebee30` |
| training-wand (provisional sceptre) | `Sprites/Icons_Weapons/ICON_SM_Wep_Sceptre_01.png` | 1024 × 1024 | `85e1f856bc7166a604e09cdd0b1c1f73c4670c98bf3e4bd6815225d91c426ab1` |
| wolf-pelt (provisional material) | `Sprites/Icons_Inventory/ICON_FantasyWarrior_Inventory_Crafting01_Clean.png` | 256 × 256 | `de97ec98a3da49b430b704cc3eb7042108fbbd946d43b5d400f37596803b85ab` |
| roadside-orders | `Sprites/Icons_Resources/ICON_SM_Item_Parchment_01.png` | 1024 × 1024 | `a056c4c7e4aab6c4037b12d765a6ba2b4426151883085e05097c52e67ed99c20` |

`Fonts/Grenze/Grenze-SemiBold.ttf` supplies headings at 18 px and above;
`Fonts/Texturina/Texturina_18pt-SemiBold.ttf` supplies smaller interface text.
Both stay in the user's local source directory with their OFL notices. Assets,
fonts and the manual have not been copied into Git. The ability PNGs are stylized
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

The candidate loads nine images and two fonts through a separate `fantasy-ui://`
Bevy asset source. Existing terrain/player/creature asset roots are preserved.

- Player health/mana/life and selected-target heading/health occupy compact
  frames either side of the lower central world view. The heading still comes from the existing
  authoritative-selection presenter; the skin does not resolve a new target.
- Twelve familiar bottom action positions with compact 44 logical-pixel square outlined icons,
  existing hotkey labels, bounded cooldown strips and hover descriptions/costs/
  authoritative rejection messages. Tooltip geometry stays clear of cast/rejection
  feedback; the windowless exporter checks their separation. The compact skin
  supports the default twelve primary keys `1–0`, `-`, `=`; only configured
  abilities activate, and unused positions retain key labels inside the
  pointer-blocking bar. The retained legacy UI keeps its five-key limit.
  Rebinding, drag/drop actions and additional gameplay abilities remain separate.
- Cast/autoattack feedback above the action bar. No attack range, facing, cast,
  cooldown, manual autoattack or keyboard behavior is changed.
- Shared charcoal/bronze pane styling for backpack, dialogue, loot, note reader and
  minimap. Inset outlines preserve existing content dimensions and scroll state.
- Quest tracker below the existing upper-right minimap, progression above the
  action bar, selected statuses in the retained upper-left status pane and a compact lower-left
  connection indicator. Loading/disconnection details remain available.
- Passive enemy name/health plates use existing server health and rendered creature
  anchors, with residency/epoch/death/projection and static-geometry checks.
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
states. Desktop/mouse checks are authorized again; windowless GPU
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


## Twelve default primary keys and readable labels

The compact skin now collects all twelve default primary keys (`1` through `0`,
then `-` and `=`), and the normal combat presenter applies the same twelve-slot
limit when dispatching configured abilities and deriving their server cooldowns.
Empty slots do not invent or send an ability. Widget activation still coalesces
with keyboard input without buffering repeated presses. The retained legacy
presentation continues to collect/display at most five actions.

A detached twelve-entry catalogue fixture checks construction, `6`/`0`/`-`/`=`
slot mapping, no repeats while held, the correct late-slot tooltip labels and
legacy exclusion of the extra keys. This is headless `ButtonInput` state, not
physical keyboard input or an encrypted game command test. The server's existing
stable-ID cast authority/protocol is unchanged. The canonical gameplay catalogue
still contains its two original abilities; no content catalogue was expanded.

Display names now humanize stable IDs as title-case words (for example Arcane
Bolt), while authoritative IDs remain exact. Authored/localized display-name
metadata and configurable action bindings remain future work. The compact normal
connection indicator shows Online/Loading-style player information rather than
tick/player-count diagnostics; detailed disconnection errors and the legacy
full diagnostic HUD are retained. Unchanged connection text is no longer assigned
every frame.

All six `primary-bar-v1-*` windowless exports pass with the canonical **two**
configured buttons, labelled empty slots, bounded minimap/actions, valid glyphs,
separate hover/feedback rectangles and a stable projection anchor. No windows or
WARN/ERROR diagnostics appear on the recorded Vulkan adapter. Formatting, strict
workspace/all-target Clippy, **165 client cases including local art**, and build
pass. Physical keyboard, mouse and the complete connected theme remain pending.


## Shared panel headers and existing close actions

In the opt-in skin, backpack, guide dialogue, loot and the item reader now share
a standard Bevy title row and 28×28 logical-pixel `X` control. Title text can
shrink/wrap beside the fixed close control. The helper replaces Node/font style
after spawning existing bundles, avoiding duplicate-component bundles. Original
footer controls and the legacy layout remain available.

Each header uses the pane's **existing** action: backpack Toggle, dialogue Close,
loot Close or document Close. No alternate save, cancellation, quest, purchase
or reading path is introduced. Headless activation checks prove the backpack
header queues only its ordinary toggle while keeping a pending save, and the
dialogue header keeps its displayed revision and existing pending-request guard.
These are event/queue checks, not a complete connected transaction test.

The exporter adds a clearly labelled header-only fixture using the same helper;
it is not a live backpack or dialogue. All six `headers-v1-*` GPU exports retain
that fixture within the target alongside action, map, anchor, scale and feedback
checks. They create zero windows and have no WARN/ERROR diagnostics on the same
Vulkan adapter. Formatting, strict workspace/all-target Clippy, **167 client
cases including local art**, and build pass. Native connected close/Escape,
long-title reflow, scrolling and pending-transaction panel checks remain for the
complete-theme gate after desktop use is permitted.


## Reference-aligned lower HUD — 6 November 2026

P2-14 / P5-13 / P5-19. The opt-in skin now anchors two 220-logical-pixel unit
frames relative to the horizontal screen centre, with left offsets −340 and
+120 and a shared bottom offset of 190. This keeps a 240-pixel gap between the
frames as ultrawide space grows. Smaller label/bar spacing reduces their height;
the existing server-derived vitals, life and target presenters remain in use.
These are text/bar frames; portraits are not implemented by this change.

The twelve primary slots are now 44×44 with thin ordinary Bevy outlines, rather
than ornamental PNG frame overlays. Original aspect ratios and the existing two
ability icons remain. A 590-pixel-wide action container sits 12 pixels above the
bottom edge; existing progression widgets are centred above it at bottom 78.
Cast/rejection feedback remains above XP at bottom 112. The same bounded scale
fitting applies to the entire composition. Hover descriptions share a position
beside the right edge of the action bar so they do not cover unit frames or the
cast bar. Empty slots still block world clicks without issuing abilities.

The windowless exporter now includes the actual progression widgets with a
clearly labelled synthetic XP value, plus the actual unit frames. All six
`build/ui-reference-20261106/lower-v1-*` cases pass through 3840×2160, including
75%, 125%, 150% preferences and small-window fitting. Reports verify physical
unit-frame bounds, centre-relative anchors, clear central separation, no
unit/tooltip/cast overlap and XP above the configured action buttons. Rendering
uses RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6 with zero OS windows and no
WARN/ERROR diagnostics. These are synthetic-widget exports, not live game captures.

All 167 client cases, formatting, strict workspace/all-target Clippy and the
development build pass. The initial test run found an obsolete assertion for
twelve decorative image overlays; it now expects only configured ability images.
The six-case aggregate is `lower-verification-v1.json` in the same ignored folder.
No server, protocol, asset-source file or gameplay catalogue changed. Desktop
use is authorized again, but this slice has not yet run native connected/pane/
input qualification. Square minimap, general enemy nameplates, portrait art and
chat/menu systems remain separate work. The skin remains opt-in.


## Circular minimap using standard Bevy UI — 6 November 2026

P2-14 / P5-13 / P5-19. The skin now draws the existing 176×176 minimap image
with `Node.border_radius = BorderRadius::MAX` and a thin rounded outline. Its
heading and metric extent remain readable without an opaque rectangular pane.
The legacy square presentation remains available without the skin. No shader,
Unity material, custom renderer, additional map texture or terrain request was
introduced. The existing 64×64 raster, north-up global projection, 256 m diameter,
player heading and update cadence are unchanged.

Guide markers whose complete 18×22 bounds could extend outside the circular
surface are hidden for this skin using a conservative 15-pixel inset. This is
presentation filtering after the existing checked global projection; it grants
no quest visibility, interaction range or new authority. A boundary test covers
centre, rim, excluded corners, invalid insets and nonfinite coordinates. Marker
edge arrows are not implemented. The minimap remains a read-only display.

Removing the final square-frame overlay also removes its image handle, loader
requirement and unused overlay helper. Three existing icon PNGs and two fonts
remain; no source asset files were changed or committed. The current asset
accounting above reflects that smaller subset.

All six `round-v1-*` windowless GPU cases pass on the same Linux Vulkan adapter,
with actual map-surface bounds and circular corner radii in each report. Pixel
checks confirm all four map corners match the unobstructed background and an
inner raster sample remains visible. The aggregate is
`build/ui-reference-20261106/round-verification-v1.json`; each case also retains
its lower HUD, XP, feedback, header and projection-anchor checks. Zero windows
and no WARN/ERROR diagnostics. Formatting, strict workspace/all-target Clippy,
168 client tests including local art, and the development build pass. These are
fixture exports; connected/native evidence is recorded separately when complete.


### First connected reference-layout check

`build/ui-reference-20261106/native-round-v1/verification.json` passes the
existing native journal/minimap walkthrough at 1280×720 and after a genuine
PostgreSQL process restart/fresh login. It uses the new client, retained matching
server/validator/art checkpoint, a private OIDC provider and the normal launcher
with the optional skin appended. Input is synthetic OS mouse/keyboard to the
PID-verified owned Linux window. The fixture's combat/quest data is explicitly
controlled; this is not ordinary random-loot or full-campaign qualification.
Both passes end at journal revision 11 / XP 125, with saved completion restored.
All owned services stop and private plaintext password files are removed.

This first live scene exposes remaining integration defects: the old wide bag
toggle overlaps the lower action area, the terrain diagnostic HUD clutters the
same corner, and passive world/status labels can paint over dialogue. Fix and
recheck those before complete-theme adoption. The native pass proves its stated
flow, not that the whole UI is visually or interactively qualified at every size.


## Native integration cleanup — 6 November 2026

The first connected scene identified three presentation defects not covered by
the original widget-only fixture. The optional skin now uses a 140-pixel backpack
toggle rather than the legacy 340-pixel control, and positions action hover
information above that dock. The actual shared toggle widget is included in the
windowless fixture; all six `integration-v1-*` cases check it against actions,
tooltips and viewport bounds alongside the existing lower HUD/map assertions.

Standard panes without an explicit layer receive `GlobalZIndex(20)`, placing
interactive windows above passive status/quest/world labels. Existing loot (25)
and document-reader (60) priorities are preserved; the headless preservation test
covers the latter. This uses Bevy's normal stacking/picking. It adds no alternate
close/transaction/Escape path. The themed graybox view hides its terrain debug
text; existing telemetry and the legacy diagnostic display remain available.

Formatting, strict workspace/all-target Clippy, 168 client cases including local
art, the development build and six windowless GPU cases pass. Connected Linux
rechecks now pass at **1280×720, 1920×1200 and 3440×1440**, plus a fresh login
after PostgreSQL process restart. Evidence:
`build/ui-reference-20261106/native-integration-v1/verification.json`. Every
initial flow ends at journal revision 11 / XP 125; rejoin restores the same saved
completion. Each recorded native GPU diagnostic count is zero on RX 7900 XTX /
RADV NAVI31 / Vulkan / Mesa 26.2.6. The owned X11 windows receive synthetic OS
input; all owned services stop and private plaintext password files are removed.

These are the existing controlled two-quest/minimap walkthroughs using the new
client and retained matching server/validator/art, not a new persistence or
content-capacity result. Screenshots confirm the compact dock, removed terrain
text and passive labels behind dialogue. They do not qualify every reader/loot/
pending-capacity state, header-X click, remapped or nondefault-scale OS input,
Windows or subjective performance. The skin remains opt-in while those broader
checks and the remaining reference features are outstanding.


## Passive enemy nameplates and separated combat text — 6 November 2026

P2-14 / P5-13 / P5-19. The skin now installs a standard Bevy UI pool of sixteen
name/health labels, matching the existing targeting limit. It consumes existing
combat health and target observations without a new packet, authority rule,
health prediction or gameplay command. Missing/disconnected observations,
foreign epochs, unknown targets, unloaded chunks, dead/respawning creatures and
invalid health suppress the label. Unused slots are hidden every frame.

Labels project the current rendered target-proxy position, keeping interpolation
and floating-origin conversion in the existing presenter. The anchor is 0.9 m
above the current body-centre root: an explicit Theia art-presentation offset,
not a hitbox or claimed Classic formula. Bevy UI scale conversion, viewport
bounds and the existing Avian solid/ground occlusion helper apply. Occlusion
shares the already tested actual-wall/terrain/nonblocking-character query.
No new maximum-distance formula is adopted; eligibility is bounded by existing
replicated/resident targets. A hidden plate never clears the persistent target.

White text with a black shadow and a red health bar remain readable against the
greybox. The selected health outline is gold. Names retain the current prototype
presenter's Wolf/Training target mapping; general authored NPC names, levels and
reaction colours require their own presentation data. Plates are passive and
ignore picking; existing model clicks and Tab remain the targeting paths.
Clickable plates, friendly/player plates, crowd stacking and toggle settings
are not implemented by this slice. Sixteen constructed widgets are not evidence
of MMO-scale NPC throughput.

The first connected capture showed an outgoing damage number over the name.
The themed combat-feedback presenter now starts its 160-pixel text column beside
the nameplate, choosing the left side near the viewport's right edge. This fixes
the measured stationary-target collision; moving/crowded label stacking remains
future work. The legacy floating-text layout and all damage/lifetime facts remain
unchanged. A boundary test checks the two lanes stay inside the viewport and
clear of the initial name column.

Formatting, strict workspace/all-target Clippy, **171 client cases including
local art**, and the development build pass. Health guards and bounded hide/
reuse have dedicated headless checks. All six `nameplates-v2-*` windowless cases
render the actual plate widget with clearly synthetic health and retain all
existing HUD/map/dock bounds. Aggregate:
`build/ui-reference-20261106/nameplates-verification-v2.json`. These exports do
not run the world-space combat-feedback presenter.

The separate native check passes the controlled 1280×720 quest/minimap flow and
PostgreSQL restart/fresh login, ending at journal revision 11 / XP 125:
`build/ui-reference-20261106/native-nameplates-v2/verification.json`. A selected
live wolf is captured at **87/100**, with the matching authority/UI health in
`1280x720/nameplate-selected-state.json`; the image shows the damage number
beside the name. Read-only observation confirms a living visible plate and a
hidden plate during death. Actual UI layout/selected-frame counters are
positive. These counters supplement the captured game/authority observations;
they do not prove pixel visibility through every overlapping window.

Both native stages have zero GPU diagnostics on the recorded RX 7900 XTX /
RADV Vulkan / Mesa 26.2.6 adapter. Owned services stop and private plaintext
password files are removed. The earlier `native-nameplates-v1` diagnostic
captures the resolved text collision. The prior three-size native integration
pass predates this addition; nameplates additionally have six scaled GPU cases
and this 1280×720 connected check, not native Windows/DPI/crowd qualification.


## Parchment dialogue and readable reward rows — 6 November 2026

P2-14 / P5-13 / P5-19, following the additional inline references. The skin's
existing dialogue pane now sits at the left: 12 logical pixels inset, 64 down,
420 wide and 70% of viewport height, with a bounded width. Its body uses a warm
opaque paper colour and dark ink. The shared header has a padded charcoal strip;
headers and button labels retain their light text. Later-added labels receive
the same body contrast. Dialogue and quest action buttons use dark rust/brown
normal, hover, pressed and pending colours. Legacy no-skin layout/palette remain
available.

This is a flat parchment-style colour treatment using ordinary Bevy UI, not a
new paper texture. Inspected local HUD parchment images are mostly tint masks;
no additional PNG or material was needed. The same three icons/two fonts remain.
The header shows the existing NPC name; portrait sourcing/rendering remains
future work. This does not implement equipment repair, money or a new quest
reward type from the reference pictures.

Native review exposed squeezed multi-line reward labels: a 34-pixel row left
only 24 pixels for a 34-pixel text layout. Offer rows and their text now retain
intrinsic height instead of shrinking inside the scroll list. The same two-line
case measures **44-pixel row / 34-pixel content / 34-pixel text**. Read-only
telemetry exports real content/text bounds; the existing native reward check
asserts the rendered text fits both its label and clickable row. No quest,
receipt, pending, capacity, authority or item-count behavior changed.

A virtual-wheel startup issue in the first owned test left the offset unchanged.
The qualifier now allows one additional actual wheel gesture only when the
virtual device was just created and the first gesture produced no observed
scroll. Later gestures retain their normal failure check. This changes test
readiness, not the game's scroll ownership or offset. The sibling dialogue and
backpack scroll positions must still remain unchanged while scrolling offers.

Formatting, strict workspace/all-target Clippy, **172 client cases including
local art**, Python syntax and the development build pass. A new headless style
check covers dark body/late text while preserving header/button contrast. Six
`paper-v1-*` windowless cases retain the shared paper/header sample and preceding
HUD/map/nameplate/dock gates through 3840×2160 with scale preferences; these are
not full quest-window exports.

The actual paper window, 32 offers, last-offer acceptance, note pages/close,
fixed item rewards, item-loss reversal, turn-in and database restart/fresh login
pass at **1280×720, 1920×1200 and 3440×1440** in the controlled schema-21 fixture.
Final evidence: `build/ui-reference-20261106/native-paper-v4/verification.json`,
including the repository's real first-wheel helper and reward containment checks.
All four stages finish at journal revision 9 / XP 75; saved rewards/completion
survive the PostgreSQL process restart. Each GPU diagnostic count is zero on
RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6. Input is synthetic owned-X11
mouse/keyboard/wheel; all owned services stop and plaintext password files are
removed. This adds UI evidence, not new quest mechanics or catalogue capacity.

Earlier v1 failed the first virtual-wheel gesture; v2/v3 passed scrolling but
precede the visual reward-row fix. v4 is the current acceptance. Normal random
loot, every pending/full-capacity state, actual header-X clicks, native Windows,
DPI/nondefault-scale OS tracing and human feel remain outside this fixture.
The skin stays opt-in. Portraits, item icons, a dedicated quest-detail/reward
composition and the other reference screens remain follow-on work.


## Contextual HUD cleanup — 6 November 2026

P2-14 / P5-13 / P5-19. In the skin, the target frame hides while no explicit
friendly/hostile target is selected. Friendly NPCs show their existing name
without hostile health or a combat-selection prompt. Hostiles retain the short
name and server-derived Health line; repeating Tab/clear/interact hints are
removed from that header. Selection, target retention, attacks and facing remain
unchanged. Header text no longer writes the same string every frame.

The selected-status pane hides when there are no observed effects to show.
Existing active-effect text/timers and metrics remain available. World quest
markers retain the same eligibility, phases, tints, scale and occlusion checks,
but use their existing leading glyph in a 40×36 logical box rather than a large
two-line caption. The current active-phase glyph remains `...`; this does not
claim exact Classic marker semantics. The legacy diagnostic presentation keeps
its headings, prompts, status descriptions and full quest captions.

Formatting, strict workspace/all-target Clippy, 172 client cases including local
art, and build pass. `build/ui-reference-20261106/native-context-v1/verification.json`
passes the controlled 1280×720 quest/minimap flow plus genuine PostgreSQL
restart/fresh login at revision 11 / XP 125. Owned screenshots show the compact
hostile frame, glyph-only quest marker and absent empty status block. The live
nameplate/death checks still pass; both native stages have zero GPU diagnostics
on RX 7900 XTX / RADV Vulkan / Mesa 26.2.6. All owned services stop and plaintext
password files are removed. This follow-on has 1280×720 native evidence; it does
not expand prior Windows, scale/DPI, crowd or full-theme qualification claims.


## Backpack icons and confirmed-count tooltips — 6 November 2026

P2-14 / P3-05 / P5-13 / P5-19. Prerequisites are the existing confirmed bag,
slot actions, input ownership and optional skin. The themed backpack retains its
16 slots and 340-pixel panel, with dark thin-gold frames, 32-pixel item art and
stack counts above one. Real mouse hover shows the catalogue name, confirmed
count and maximum stack in a separate read-only tooltip beside the bag. Accessible
slot labels retain the full item description. Empty/loading slots have no stale
art, count or tooltip. Unknown item IDs use a generic icon. Pelt/crafting and
wand/sceptre art are provisional mappings, not new item properties or appearance
rules. No invented rarity, durability, price or quest classification is displayed.

Split, destroy confirmation, equip, purchase, reward and document operations use
the existing widgets and command/receipt guards. No inventory mutation or
optimistic holding is introduced. The unskinned text-grid presentation remains.
The windowless fixture uses those same slot widgets and visual projector with a
clearly synthetic six-item bag and explicit fixture-only hover resource. It
installs no inventory observer, network or command producer; normal clients use
Bevy's actual hover component. The first GPU check exposed the invalid assumption
that a manually set hover flag would survive Bevy picking without a pointer;
that failed preview is retained as `bag-v1-*`, not accepted evidence.

Acceptance: counts/art/descriptions follow a newer confirmed bag and clear on
projection loss; all 16 slots and the hovered tooltip fit the viewport; the tooltip
does not cover bag slots; actual pointer hover and existing operations still work.
The 173-client-case run including local art, strict workspace/all-target Clippy,
formatting and development build pass. Six final `bag-v4-*` windowless
GPU cases through 3840×2160 pass the previous HUD gates plus 16-slot containment,
six occupied icons and tooltip bounds/separation. Source hashes/budget are in
`build/ui-reference-20261106/bag-source-sha256.json`; layout reports are in
`bag-verification-v4.json`. Screenshot review found that the first placement
covered XP; the final tooltip clears XP, cast feedback and action buttons, with
explicit geometry assertions. Earlier `bag-v2-*` exports precede that fix;
`bag-v3-*` preserves the intermediate failed separation assertion.

`native-bag-v1/verification.json` passes real owned-X11 mouse hover, empty-slot
hiding, equipment, 32 offers, note use, item-loss reversal, fixed rewards and
PostgreSQL restart/rejoin at 1280×720, 1920×1200 and 3440×1440. The final tooltip
placement passes `native-bag-v2/verification.json` at 1280×720 plus restart/rejoin.
All stages end at journal revision 9 / XP 75, with zero GPU diagnostics on Linux
RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6. All owned services stop and
plaintext password files are removed. The native hover screenshots and confirmed
stack/tooltip JSON are retained per stage. These are controlled UI fixtures,
not native Windows, nondefault-scale/DPI input, human feel, random-loot balance
or performance evidence. The skin remains opt-in.


## Window-coordinate diagnostic correction — 6 November 2026

P2-14 / P5-19. Read-only native test exports for backpack/equipment, reader,
loot, dialogue and quest controls previously multiplied physical UI geometry by
`ComputedNode.inverse_scale_factor()` alone. In the pinned Bevy 0.20 source,
`propagate_ui_target_cameras` (`bevy_ui/src/update.rs`) combines render-target DPI
with `UiScale`; the inverse removes both. That produced UI logical coordinates,
while the native pointer helper expects window logical coordinates and applies
window DPI itself. At UI scale 1.25 the exported click targets would be too small
and too near the origin. Earlier native runs used UI scale 1 and are unaffected.

A shared factor now removes window DPI while retaining UI enlargement for those
read-only positions, sizes and glyph bounds. ScrollPosition values and runtime
scroll/clipping clamps remain in their existing UI logical units. The OS trace
records active UI scale and explicit geometry/scroll unit labels. This changes
diagnostics, not Bevy picking, gameplay commands, world coordinates or rendering.
Other older specialized qualifiers are still qualified only at their original
scale; this does not promote them to general scale/DPI evidence.

Headless fixtures check scale 1/1.25/1.5 with distinct window DPI, and check the
actual dialogue exporter gives window geometry while retaining a scroll offset
of 17 UI pixels. The 175-client-case run includes local PNG decoding. Formatting
and strict workspace/all-target Clippy pass. No new desktop/input test
has been launched: the user paused desktop/mouse tests while using the machine.
Real enlarged-UI interaction, native DPI and Windows checks remain pending.


## Backpack action palette and shared preview — 6 November 2026

P2-14 / P3-05 / P5-13 / P5-19. The optional backpack's existing action buttons,
header close and dock now use the shared dark red palette, with selected actions,
pending dimming and the original red destruction confirmation retained. Slots
keep their dark/gold treatment. Unskinned colours remain unchanged.

The original button painter is now a separate presentation system after the
existing network/label preparation. Footer construction is shared between the
normal backpack and the labelled windowless fixture; action IDs, widget types,
observers, sizes, order, labels and command handlers are preserved. The fixture
renders the real equip, unequip, split, cancel and destruction widgets but
installs no action observer, Pending resource or network. Its fixed status text
explicitly says it cannot issue item commands. The fixture deliberately forces
hover descriptions alongside other HUD samples; it is a layout/palette test,
not a complete live-screen composition or simultaneous-panel visibility claim.

All six `bag-actions-v1-*` windowless GPU cases pass the previous layout gates
plus five real action rectangles contained in the viewport. The selected local
PNGs/fonts render on Linux RX 7900 XTX / RADV Vulkan / Mesa 26.2.6 with no OS
window, Winit, mouse, keyboard or desktop input. Evidence is
`build/ui-reference-20261106/bag-actions-verification-v1.json`. Development build,
175 client cases including local art, formatting and strict Clippy pass. No new desktop check has run after the user's pause; native input
qualification of this presentation-only extraction remains pending. Earlier
backpack native evidence belongs to its exact pre-palette sources. No push.


## Fixed reward icon cells — 6 November 2026

P2-14 / P3-05 / P3-11 / P5-13 / P5-19. The themed quest offer/turn-in buttons now
include decorative item cells below their unchanged full text names, counts and
XP description. Only the validated grants in the bound quest reward profile
supply item IDs/counts. Quests with no fixed grants and the unskinned UI gain no
cells. The same item art mapping is used as the backpack; no new assets are
loaded, and the wand/pelt/unknown mappings retain their stated limitations.

Each cell uses standard Bevy image/text/outline primitives and shows a 28-pixel
icon plus count in an 80×36 logical area. The row wraps, and the offer retains
intrinsic height. All decorative nodes ignore pointer picking; the existing
offer button, pending guard, revision, receipt and server transaction remain the
only action. This adds no item selection, claim, currency or reward mechanism.
The contrast marker is generalized from headers to dark surfaces so count text
stays light on dark item cells inside parchment.

The existing fixed-reward headless case now also constructs the actual themed
and unthemed offer trees, checking exact bound items/counts, absent art for the
legacy/other quest and ignored picking. All **175 client cases**, formatting,
strict workspace/all-target Clippy and development build pass. Six
`reward-icons-v1-*` windowless GPU cases through 3840×2160 render a validated,
explicitly labelled two-item metadata fixture in a deliberately narrow parent.
Both cells wrap into separate rows, remain inside the paper sample and render
counts whose glyph bounds fit their text nodes. These checks retain the other
HUD/backpack/action/tooltip gates. Evidence:
`build/ui-reference-20261106/reward-icons-verification-v1.json`, on Linux RX 7900
XTX / RADV Vulkan / Mesa 26.2.6, without any OS window or desktop input.

The paper sample exercises the shared decorative renderer, not a connected quest
offer list. Actual taller-row scrolling, clipping, click ownership, pending and
capacity behavior with these new cells still need the normal native replay.
Desktop/mouse testing remains paused at the user's request. Prior native proof
belongs to the exact earlier text-only offer rows; the skin remains opt-in.


## Compact parchment reader — 6 November 2026

P2-14 / P3-11 / P5-13 / P5-19. The optional reader now sits at the left with a
420-pixel logical width, dark title/navigation rows, light parchment and dark
body text. Prev/Page/Next sit directly below the title; the existing close and
Escape actions remain. It uses 45% of viewport height, capped at 600 logical
pixels. The earlier 70% candidate obscured part of the player frame at 1280×720;
the shorter reader keeps both lower combat frames visible in the qualified
fixtures and still fits the short first page. Longer prose retains its existing
scroll area. The themed dialogue also gains the 600-pixel height cap; its
current multi-offer composition and scroll behaviour remain separate from a
future dedicated quest detail screen.

Reader buttons use the shared dark red palette and explicit accessible labels.
Unskinned geometry/navigation is retained. Opening, exact document identity,
server-confirmed read credit, local page changes, pending guards and Escape
priority are unchanged. No voice/Stop, portrait, speech-caption or choice-reward
control was added from the new references.

Append `--reader` to the existing exclusive windowless preview command to render
the actual reader setup and painter with the validated local two-page document.
The fixture installs no Network, read-command Pending resource or observer, and
produces zero approved reads, submissions or read clicks. It is not normal
server-open evidence. A headless case checks that isolation. All **176 client
cases** including local art, formatting, strict workspace/all-target Clippy and
development build pass.

Six `reader-v2-*` windowless GPU cases pass through 3840×2160 with the existing
scale settings. They check reader bounds/cap, dark ink and actual nonempty text,
page/footer controls inside the pane and outside the scroll body, the short page
fitting without scrolling, and no overlap with either lower combat frame. The
ordinary HUD preview branch also passes once; three malformed reader CLI cases
refuse before GPU/window creation and leave no output directory. Evidence:
`build/ui-reference-20261106/reader-verification-v2.json`,
`reader-default-regression-v1/report.json` and `reader-cli-refusals-v1.json`.
Device: Linux RX 7900 XTX / RADV Vulkan / Mesa 26.2.6; OS windows/input are absent.
Earlier `reader-v1-*` results precede the compact-height fix. Native interaction,
long-text wheel ownership, nondefault-scale/DPI input and Windows remain pending.
Desktop/mouse testing remains paused at the user's request. The skin stays opt-in.


## Minimap objective geography follow-up — 6 November 2026

The requested search-area overlay now starts with existing bound reach volumes:
only active, unfinished goals select a region. Six windowless GPU layouts retain
the preceding UI gates and show a labelled blue fixture region. The reader branch
passes once. This adds no terrain requests or live actor disclosure. 180 client
and 15 focused network/profile cases, formatting, strict Clippy and build pass.
[Data, cost evidence and remaining kill/collection mappings](MINIMAP.md#initial-active-quest-area-overlay--6-november-2026).
Desktop/input checks remain paused. Existing single-guide `!`/`?` support remains;
no general NPC registry or spawn/collection authoring mapping is claimed yet.


Explicit quest search mappings — 6 November 2026: optional area metadata now
links executable objectives to reviewed search regions, including kill/collection
and document/interaction goals. Confirmed item loss restores collection regions;
occupancy cannot grant kill credit. Existing content/profile hashes are unchanged.
181 client, 385 core/persistence and 27 focused network cases pass; 73 SQL-dependent
cases remain ignored. [Format and exact scope](MINIMAP.md#explicit-objective-search-regions--6-november-2026).
General multi-NPC `!`/`?` locations and native overlay interaction remain pending;
existing support is for the single known guide. No desktop input tests were run.


Minimap hover descriptions — 6 November 2026: actual guide `!`/`?` pins now list
available/ready quest titles, while authored region hovers show active objective
counts. 185 client cases and six windowless layouts pass. A 1280×720 connected
OIDC/Renet/PostgreSQL hover/completion/rejoin trace passes, ending at revision 11 /
XP 125 with zero GPU diagnostics; services and temporary input devices stop.
[Implementation, evidence and limits](MINIMAP.md#read-only-hover-descriptions--6-november-2026).
Desktop testing is authorized again by the user's latest instruction. Enlarged UI
and native search-region hovering remain separate checks. Local commits, no push.


## Enlarged native panel replay — 6 November 2026

P2-14 / P3-11 / P3-12 / P3-15 / P5-13. With desktop permission restored, the
current client passes the actual normal launcher/OIDC/Renet/PostgreSQL fixed-
reward/document/32-offer flow at **1920×1200 with `--ui-scale 1.25`**, then after
an actual database process restart and fresh login. Window telemetry reports
DPI 1; that value is not UI scale. The explicit launcher flag and screenshots
record 125% UI enlargement (the 420-logical-pixel reader is 525 window pixels).

The compact left reader opens with one click, both pages work, Escape retains
the expected sibling panel, the last of 32 offers is reached using the bounded
wheel area, and complete reward labels fit their clickable rows. Decorative
bound reward icons/counts render on the real offers. The bag's dark red actions,
confirmed item destruction, possession reversal, replacement pickup, consumption
and fixed grants retain their original transaction semantics. Final journal
revision 9, XP 75 and 103 tokens survive restart without another grant. This is a
guaranteed-loot, low-damage development fixture, not an ordinary drop-rate or
human feel/performance claim.

Both native passes report zero GPU diagnostics on RX 7900 XTX / RADV Vulkan /
Mesa 26.2.6. Owned services stop, the temporary wheel device is removed and
plaintext password files are removed. Evidence:
`build/ui-reference-20261106/native-scaled-paper-v1/verification.json`, the exact
launcher wrapper `native-scaled-paper-v1.py`, and page/reward/offer screenshots in
its `1920x1200` folder. This qualifies the current compact reader/reward-row
geometry and 125% pointer exports for this flow. Native 75%/150%, DPI scaling,
resize/restoration, broader resolutions, search-region hover and Windows remain
separate. Earlier pending entries above describe prior checkpoints.


Native reach-area guidance — 6 November 2026: the actual 1280×720 Linux area
quest walk/hover/turn-in/restart trace passes. Hovering outside the authored volume
shows `0/1` without credit; entry clears shading and completion survives rejoin
at revision 14 / XP 150. Zero GPU diagnostics; owned services/input device stop.
[Exact evidence and limits](MINIMAP.md#native-objective-area-walk-and-hover--6-november-2026).


## Responsive wide-screen combat frames — 6 November 2026

P2-14 / P5-13 / P5-19. The local 3440×1440 reference captures put the player and
target frames farther apart than the earlier fixed centre offsets. The opt-in
skin now computes available width in logical UI pixels: viewport width divided
by current UI scale. Above 1920 logical pixels each frame moves outward by one
quarter of the excess, capped at 240 logical pixels. Smaller layouts keep their
previous anchors. Widths, vertical placement, unit data and interaction remain
unchanged. Standard Bevy Window/Camera viewport data and Node margins are used;
this does not change camera projection, gameplay coordinates or targeting.

The headless resize/scale case checks 3440 / 125%, shrink to 1920, re-enlargement
at 150%, the maximum shift and invalid-width fallback. All **186 client cases**
including local art, formatting, strict workspace/all-target Clippy and build
pass. Six `wide-frames-v1-*` windowless Linux GPU layouts retain the HUD, tooltip,
reader-free, map and bag gates. At 3440×1440 / 125%, actual rasterized frame centres
are x=1172.5 (player) and x=2267.5 (target), close to the user's supplied composition.
No additional texture, camera, actor or image asset is introduced. Device: RX 7900
XTX / RADV Vulkan / Mesa 26.2.6. Exact evidence:
`build/ui-reference-20261106/wide-frames-verification-v1.json` and the corresponding
client/Clippy/build logs.

Native Linux 3440×1440 / 125% UI scale passes contact hovering, the two-quest
combat/completion flow and actual PostgreSQL restart/fresh login: revision 11 /
XP 125, zero GPU diagnostics. The trace also passes minimap pointer/wheel
ownership, unchanged idle streaming and the existing image budget. Native
screenshots show the wider lower frames; the numeric centre measurements above
come from the windowless geometry report. Evidence:
`build/ui-reference-20261106/native-wide-frames-v1/verification.json`.
Owned services stop, temporary wheel device and plaintext password files are
removed. Windows, native DPI and interactive resize/restoration remain separate.


## In-game session UI scale — 6 November 2026

P2-14 / P5-13 / P5-19. The opt-in skin now has an **Interface [F10]** dock button
beside the backpack. F10 toggles a standard Bevy panel with 75%, 100%, 125% and
150% presets. The panel reports both preferred and currently fitted size. Changes
apply immediately to the existing UI-scale resource and responsive anchors;
small windows fit down and later restore the preferred value. Preferences last
for this client session; no saved settings file or account setting is claimed.
The world continues simulating while the panel is open.

Escape closes Interface first and retains its ownership for the rest of that
frame. It preserves an underlying reader, loot panel, dialogue, backpack and
Classic target; subsequent presses use the existing priority. Unfocused windows
do not toggle the interface. Hidden presets cannot edit the preference, and
out-of-range preset values refuse. Widgets send no network/gameplay commands.
Read-only native qualification exports identify the actual toggle/preset/close
positions and preferred/displayed scales. No engine, protocol or asset changes.

Three new headless cases exercise preset bounds/fit restoration, keyboard focus
and the actual multi-panel Escape chain including Classic target-clear ownership.
All **189 client cases** including local art pass. Formatting, strict workspace/
all-target Clippy and client build pass. Six `interface-v1-*` windowless Linux GPU
layouts through 3840×2160 render the shared settings panel, check viewport fit and
clear combat feedback, and retain existing HUD/map/bag gates. Device: RX 7900 XTX /
RADV Vulkan / Mesa 26.2.6. The `--interface` developer preview uses synthetic
resources and no action observer/network; it is mutually exclusive with `--reader`.
Evidence: `build/ui-reference-20261106/interface-verification-v1.json` and the
corresponding client/Clippy/build logs. Native pointer interaction with these
new controls and interactive resize/DPI/Windows remain pending. The user's running
playtest keeps its frozen earlier binary and has not been interrupted or replaced.


## Floating committed XP feedback — 6 November 2026

P2-14 / P5-13 / P5-19. The client now reuses one noninteractive standard Bevy
Text/Node label above the authoritative player position. Positive committed XP
deltas appear in purple, coalesce while visible, rise and fade over 1.5 seconds.
Checked global/local projection and UI-scale conversion keep the label inside
the viewport. The observer validates owner, epoch, World, profile and monotonic
progression snapshots. Saved login/rejoin progress establishes a baseline;
duplicates produce no extra cue. Invalid regressions clear the cue while retaining
the valid high-water mark. This presentation never awards XP and does not promise
one popup per kill: multiple server awards can arrive in one observed delta.

Four new cases cover gain coalescing/expiry, reconnect/context resets, replay
rejection and viewport-edge placement. All 193 client cases including local art
pass, as do formatting, denied-warning workspace/all-target Clippy and client
build. Six windowless Linux GPU layouts (1280×720 through 3840×2160, preferred
scales 75–150%) render the shared static `XP +25 (fixture)` widget and check its
bounds and separation from cast/XP-bar feedback. Device: AMD RX 7900 XTX,
RADV NAVI31 / Vulkan / Mesa 26.2.6. The fixture can overlap its diagnostic preview
title; it does not simulate a live award. Live connected XP presentation, native
DPI/resize and Windows remain unverified. No desktop input was used.

Evidence (ignored): `build/ui-reference-20261106/xp-popup-client-v3.log`,
`xp-popup-clippy-v2.log`, `xp-popup-build-v1.log`, `xp-popup-fmt-v1.log` and
`xp-popup-verification-v1.json`. Earlier parser-failure logs are not passing
evidence. Work stopped here at the user's request; no further play session was
launched. The separately frozen release playtest pack predates this XP change.


## Saved local interface size — 7 November 2026

P2-14, prerequisite: existing Interface presets and preferred/fitted-scale policy.
Optional client/`tools/play.py` argument `--ui-preferences FILE` enables JSON
schema 1: `{"schema":1,"ui_scale":1.25}`. A missing file uses the usual launch
size. Existing files must be regular, at most 4 KiB, have no unknown fields and
contain a finite scale in 0.75–1.5. Invalid files refuse startup without rewriting.
`--ui-scale` takes precedence for this launch; neither loading nor that override
writes a file. Only a later in-game preferred-size change requests a save.

One Bevy I/O task writes at a time; subsequent changes coalesce into the latest
preferred size. A complete, synced temporary candidate in the same directory is
renamed over the prior file. Normal failures clean the candidate and preserve
the previous file. Directory durability across power loss is not claimed. Fitted
viewport reductions never change the preference file. The existing panel reports
saving/saved/failure text. Failures keep the session size and do not retry every
frame; a later size change retries. Wait for the saved message before exiting;
abrupt shutdown during a pending task is not a save guarantee. Simultaneous
clients sharing a file have last-completed-write semantics, not synchronization.
No network/account setting or gameplay command is added.

`tools/play_local.py --ui-preferences /absolute/local/file.json` can share a size
across otherwise disposable test accounts. Without an explicit scale it lets the
saved value apply; without a preference file it retains its old 125% default.
This option needs the current client; older frozen packs do not support it.
Use an ignored path such as `build/local-interface.json` for development.

Five new headless cases cover strict loading/CLI precedence, complete replacement
and failed-candidate cleanup, actual background-write coalescing/new-client reload,
and failure/session retention/recovery. All 198 client cases with local art and
53 launcher checks pass. Formatting, strict workspace/all-target Clippy and client
build pass. Evidence: ignored `build/ui-preferences-20261107/{client-v3.log,
clippy-v3.log,build-v1.log,launcher-v3.log,fmt-v1.log}`. Python byte compilation
passes. Earlier v1 failures were corrected and are not accepted evidence.
No window, display, mouse or keyboard was used. Native text/layout/input, Windows
execution and abrupt-exit behavior remain unverified. This extends the earlier
session-only decision; it does not promote the opt-in skin to fully qualified.
