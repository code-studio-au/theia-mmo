# General UI: Fantasy Warrior assets and Classic layout

6 November 2026. P2-14 / P5-13 / P5-19, under UI-001.

The user moved development beyond quest mechanics to a general WoW-like UI.
This first slice is an **opt-in presentation candidate**. The existing playable
configuration remains available pending complete-theme qualification. The user
restored desktop/mouse permission after supplying the visual reference; earlier
paused-test statements below describe historical checkpoints. The
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
player or NPC appearance. The immediately following presentation work is the
existing quest/dialogue window, after the nameplate/damage-label check finishes.

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

The current three selected PNGs total **24,728 encoded bytes**, with
**1,048,576 bytes of RGBA8 base-level pixel payload**. The initial four-image
candidate included an ornamental square frame; the reference-aligned skin no
longer loads or requires it. This is a source/pixel accounting result, not measured
GPU allocation, frame-rate or batching performance. The three images retain their original
aspect ratios. Thin frames and the circular map use standard Bevy UI styling. Missing Unity border metadata is not
reconstructed as guessed nine-slice settings. No atlas is needed for this small
subset; the manual's atlas suggestion can be reconsidered against measured demand.

| Role | Source path relative to HUDs | Dimensions | SHA-256 |
| --- | --- | --- | --- |
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

The candidate loads three images and two fonts through a separate `fantasy-ui://`
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
