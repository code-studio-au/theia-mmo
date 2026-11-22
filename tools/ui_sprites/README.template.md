# Theia production sprite package — v1.0.0

416 transparent PNG sprites, 42 complete interaction-state families, one 2048 ×
2048 atlas, and an audit of all 408 input crops. 206 canonical assets are exported;
42 have six states. Sample text, sheet labels and merged demonstration layouts are
not runtime sprites. See `source-audit.csv` for every inclusion/exclusion.

## Source and processing

Only the user-provided `Theia_UI_Final_Sprite_Package.zip` was used. Its SHA-256 is
`7de9cfe741783121df90c37434597fd2f88abb9ce9d621c4bccc207c0206f90b`.
The original archive is unchanged. Neither the rejected UI ZIP nor its contact
sheet is an input. Source provenance/rights remain those of the supplied artwork;
this processing pass does not introduce a new artwork licence.

The user approved deterministic processing preserving the artwork. Processing
removes exterior white matte and isolated extraction specks, repairs small enclosed
white-keyed highlights on solid controls, separates combined crops, opens portrait/
minimap/bar frames, trims empty margins, and adds transparent safety padding with
RGB bleed. Convex gold controls are clipped to their measured gold silhouette.
Interior painted detail and natural border variation remain. No AI regeneration,
vector tracing, new font, artificial upscale or sharpening was applied.

`recipes.json` pins the source and records semantic mappings. `manifest.json`
records source ID/crop/size, trim, output dimensions, pivot, padding, resizing,
state derivation, cleanup counts, atlas rectangle and file hashes.

## Import contract

- PNG RGBA8, **straight alpha**, **sRGB**. Use ordinary alpha blending, linear
  sampling and clamp-to-edge. Do not premultiply again. No mipmaps or lossy texture
  compression for this atlas; aggressive downsampling needs a separate export.
- Import either individual files in `sprites/` or `atlases/ui-00.png`. Loading both
  duplicates texture memory. The uncompressed atlas is 16 MiB (without mipmaps).
- Atlas `rect` is `[left, top, right, bottom]` in pixels, right/bottom exclusive,
  top-left origin. Nothing is rotated. Divide by 2048 only if normalized UVs are
  required. The rectangle includes each sprite's four-pixel transparent padding.
- Every sprite has four transparent border pixels with nearby RGB extended into
  them. Atlas packing adds a four-pixel gutter and two-pixel padded-texel extrusion.
  Do not trim that padding a second time.
- `nine_slice` is `[left, top, right, bottom]` in **output texture pixels**, including
  padding. Preserve corners at 1:1 and stretch edges/centre. Respect `min_size`;
  at UI scale S multiply both borders and minimum dimensions by S. Bevy 0.20 uses
  `NodeImageMode::Sliced(TextureSlicer)` with `BorderRect.min_inset=(L,T)` and
  `max_inset=(R,B)`. Use `ImageNode.rect` for the atlas region.
- Assets with `nine_slice: null` must retain aspect ratio. Their `min_size` is the
  nominal source-detail size, not a prohibition on uniform downscaling. Tooltip
  tails, ornaments, icons, crests and portraits must not be stretched independently
  on X/Y. Compose a scalable panel plus a separate ornament where needed.
- Artwork is small raster source material. Resizing panels is supported; enlarging
  tiny icons cannot recover new detail. Inspect target DPI/UI scaling before release.

## States, layers and live content

Families provide `default`, `hover`, `pressed`, `selected`, `focused`, `disabled`.
All six share identical dimensions, alpha and pivot, so state changes do not jump.
Non-default states are deterministic RGB variants, not claimed to be artist-labeled
original crops. Focus follows the existing silhouette without a square outside box.
Disabled art does **not** disable behaviour: the consuming widget must reject mouse,
keyboard and controller activation. Use an appropriate live label colour too.

Checkbox: centred `checkbox.tick` over a base checkbox when checked. Radio:
centred `radio.dot` when selected. Keep the Boolean value independent of interaction
state; tint the overlay when disabled. `selected` is an interaction appearance,
not a baked checkmark. Sliders: track underneath, clipped fill, then diamond thumb;
clamp the thumb centre to the visible rail. Bars: clip a fill to the inner opening,
then draw `bar.frame`; do not fill the transparent padding or decorative endcaps.
Portrait/minimap frames have transparent centres: mask their live content to a
circle underneath. Hotbar endcaps are separate from slots. Fixed item portraits,
ability art and source crests are examples of artwork, not real account/game data.

Render names, labels, chat, combat values and descriptions as live text. This pack
contains no font or full screen background. Toggles, dropdown popups, lists, menus,
settings screens and tooltips are composed widgets, not implemented behaviours in
this package. Links use live text and the game's focus/hover styles. Unsupported
features still need disabled widget logic when integrated.

## Validation and reproduction

`validation.json` records automated export checks. `native-report.json` records
actual GPU/device/scope. Previews are supplied in the delivery archive. Rebuild
from the repository with Python 3, Pillow 10.2.0 and NumPy 1.26.4:

```sh
python3 tools/ui_sprites/build.py --source /path/to/Theia_UI_Final_Sprite_Package.zip
python3 tools/ui_sprites/validate.py content/ui/sprites-v1 --previews build/ui-sprites-production/qa
python3 -m unittest discover -s tools/ui_sprites -p 'test_*.py' -v
cargo run --release -p theia-client --example ui_sprite_preview
```

The Bevy preview loads the actual atlas and manifest, renders every sprite and
large nine-slice samples, captures ten pages, and exits. It needs a native display
and GPU, and never connects to an account/server. Qualification is Linux only;
Windows, target-DPI acceptance and final game-screen integration remain separate.
This pass prepares the asset package; it does not replace the live game UI.
