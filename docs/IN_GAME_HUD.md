# In-game HUD reference pass

P2-14 uses the supplied Stoneford Outskirts concept and the approved v3 sprites.
Its prerequisites are the native fantasy theme, the admitted character identity,
combat and progression observations, slot bindings, and verified cartography.

The player frame uses the available body-and-face thumbnail from the private
modular hero assets. Without the thumbnail atlas it displays a character glyph.
A level medallion appears when committed progression is available. Health and
mana show numeric values inside green and blue tracks. The selected hostile
frame shows red health and the illustrated wolf portrait for the wolf profile.
A friendly selection keeps its existing party or contact information.

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
