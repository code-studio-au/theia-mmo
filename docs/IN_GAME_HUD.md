# In-game HUD reference pass

P2-14 uses the supplied Stoneford Outskirts concept and the approved v3 sprites.
Its prerequisites are the native fantasy theme, the admitted character identity,
combat and progression observations, slot bindings, and verified cartography.

The player frame uses the available body-and-face thumbnail from the private
modular hero assets. Without the thumbnail atlas it displays a character glyph.
A level medallion appears when committed progression is available. The selected
hostile uses the same badge with the level from the admitted physical stats
profile. Millhaven's wolves are configured at level 1.

Player name and target name sit on black plates with beveled gold trim. The
name, health and mana tracks overlap vertically and extend behind the portrait.
Player health is green and hostile health is red. An enemy with a configured
mana pool shows its current and maximum mana in blue. The bar is dimmed and
empty when the enemy has no pool. A friendly selection keeps its existing
party or contact information.

`physical_stats.json` accepts an optional `maximum_mana` on a creature row
in the range 1–1,000,000. Setting it gives that creature a server-owned pool
that starts full and refills on a combat reset or respawn. Combat observations
with target mana use CS03 and fit the existing datagram budget. The existing
combat snapshot formats still apply when all visible targets lack mana.
This slice supplies the resource and displays it. Enemy spell selection,
mana spending by enemy abilities and regeneration rules require a casting
behavior. The existing wolves remain melee-only.

Active effects use small icons with a countdown and a hover description. Slots
only display effects that are current for the admitted owner or selected
hostile. The configured development effect is Scorch. Positive buffs require
their own corresponding authoritative observations.

Twelve skill positions keep their established keys, click actions, drag
bindings and cooldown feedback. Actual casts carry the ability icon and
server-tick progress. The violet XP strip shows committed experience
under the skill bar and gives the exact values on hover.

The circular minimap has a region title, fixed north badge and sapphire player
arrow. The compass button opens the framed world map when matching cartography
is available. The M and Escape shortcuts, map projection, landmarks and
boundary hover still use the same admitted map. Map painting remains a bounded
presentation operation.

## Verification

- Strict release Clippy, format and diff checks, release build and 363 client
  tests passed, with five existing skips. 29 Python playtest and window tests
  passed.
- Standard Bevy offscreen captures at 1920×1080, 1720×700 and 1280×720 used
  fixture values for partial vitals, cast and XP. Vulkan validation reported
  zero diagnostics. See `build/reference-hud-v3-preview-*/report.json`.
- The native Millhaven account run displayed the actual saved portrait choices,
  level, vitals and selected wolf. The match verified M, Escape, the compass
  button, seven region highlights and clearing over water. See
  `build/reference-hud-native-v1/hud-target.png` and the adjacent `map/` reports.
  The native compact resize step timed out while the fixture window was
  maximized. The offscreen 1280×720 recording covers geometry at that size.
  The tested rendering path was Linux Vulkan llvmpipe, LLVM 21.1.8 and Mesa
  26.2.7, with zero Vulkan diagnostics. The native account run did not apply a
  live status or earn XP.
- The user’s fullscreen native release opened on Linux RX 7900 XTX / RADV
  Vulkan at 3440×1440. This is a separate manual playtest.

The embedded wolf medallion art comes from built-in image generation. The
prompt, input dimensions and file hashes are in
`content/ui/hud/wolf-portrait.json`. No other part of the HUD uses baked
screenshot text or values.

The follow-up uses a square 176×176 map surface with a two-pixel gold rim
and three small compass accents. Out-of-combat state has an empty life label.
Combat and urgent life messages still draw when applicable. HUD unit frames
draw over world names, and the minimap tooltip starts below the unit frames.
The optional full-colour portrait atlas uses the same body and face choices
as the saved character, with the default outfit and palette for those tiles.

The final Linux native check starts directly at 1280×720 and passes account
entry, target selection, the map controls and boundary hover. It observes
the square map surface and no peaceful-status glyphs. Vulkan validation on
llvmpipe reports zero diagnostics. Offscreen checks pass again at 1920×1080,
1720×700 and 1280×720. See `build/reference-hud-native-round/qualification.json`
and `build/reference-hud-round-preview-*/report.json`. All 363 client tests,
five existing skips, strict Clippy, formatting and release build checks pass.

The joined unit frames were captured in the 1720×1000 native account fixture
on Linux Vulkan llvmpipe (LLVM 21.1.8, Mesa 26.2.7). The normal wolf showed
level 1 and a disabled mana track. A separate profile with a 140-point pool
showed 140 / 140. Both runs passed Vulkan diagnostics checks. See
`build/character-name-refocus-fixed-{no-mana,mana}/wolf-profile.png`.
The resource regression checks initialization, a server-side spend and
restoration. Fourteen wire cases pass, including packing sixteen mana-enabled
targets with sixty-four cooldowns into 1,194 bytes. The 363 client tests,
strict Clippy, format and release checks pass.

The minimap caption below the circle uses the mapped zone name and the
player's current world X and Z coordinates, rounded to metres. The gold rim
is four pixels wide at default UI scale, with sapphire and gold accents
from the existing sprite kit. The raster itself remains 176×176.
The 1720×1000 native Millhaven run verified the caption, the map key and
button, Escape, seven boundary highlights and clearing the highlight over
water. See `build/reference-hud-native-ornate/hud-minimap.png` and
`qualification.json` in the same directory. The three offscreen sizes
also passed. Linux Vulkan llvmpipe reported zero diagnostics. Strict Clippy,
formatting, the release build and 363 client tests passed, with five skips.

## Matte slots and compact inventory

P2-14 and P3-05 share a charcoal slot well and one narrow bronze edge for
ability, bag, equipment, reward and status tiles. Empty slots have no icon.
Occupied tiles sample the interiors of the approved v3 sprites and fit the
artwork inside the frame. Stack counts, cooldowns, hover and selection stay
visible and use live widget state.

The skinned inventory is a four-by-four grid of the sixteen available bag
slots with a 244-pixel outer width at default UI scale. Right-click a stack
for its available actions. Splitting one, moving and unequipping use the
existing destination selection. Destruction requires a second confirmation
bound to the same slot and bag revision. Reading a document uses the existing
authority and reader. Escape dismisses the context menu before the bag.

The Character shortcut opens the existing main-hand equipment state in a
separate panel. Trading accompanies the NPC interaction in its own panel.
The bag has no categories, rewards controls, account reward claim, equipment
row, currency footer or loading-status row. Item operation feedback appears
outside the grid. The text-only development backpack retains its controls
for the original qualification fixtures.

The 1720×1000 native Millhaven test exercised a secondary click on an item
while another item was selected, plus equip, unequip, split, destroy
confirmation and Escape. See `build/compact-inventory-native-final/slot-context.png`
and the adjacent qualification report. Captures at 1920×1080, 1720×700
and 1280×720 use the same bag and equipment widgets with fixture items.
Linux Vulkan llvmpipe (LLVM 21.1.8, Mesa 26.2.7) reported zero diagnostics.
Strict Clippy, formatting, release build and 364 client tests pass, with five
existing skips. A cached library link failure was resolved by rebuilding the
affected dependency; the final checks use the standard release profile.
