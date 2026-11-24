# Theia regenerated UI sprites — v3

Regenerated artwork for the approved Theia_UI_Final_Sprite_Package.zip. This is an asset deliverable, not a completed game UI integration. The rejected early UI ZIP/contact sheet were not used.

## Contents

- `sprites/`: individual transparent PNGs, including consistent control states.
- `atlases/`: 2048 px RGBA texture pages; use the manifest's pixel rectangles.
- `manifest.json`: names, states, geometry, slicing, logical sizes and hashes.
- `generation.json`: exact prompts, master hashes and finishing provenance.
- `source-audit.csv`: disposition of the original 408 reference crops.
- `validation.json`, `native-report.json`: automated asset and Linux Bevy evidence.
- `previews/`: equal-size old/new comparisons, states, nine-slice and GPU captures.

Raw generated masters are retained in the separate masters archive. Runtime PNGs contain regenerated pixels only. Repeated colours, states, mirrored arrows, composite controls and panel sizes are deterministic derivatives of the individually generated masters; they are not separate generative calls. Sample labels and text from the source are replaced by live game text.

To rebuild in the Theia repository, install `tools/ui_sprites/requirements.txt`, extract the masters archive and run `python3 tools/ui_sprites/regenerate_export.py --source /path/to/extracted-master-package`. Then run `python3 tools/ui_sprites/regeneration_review.py --source /path/to/extracted-master-package`. These commands use the repository's canonical recipes and v1 metadata/reference images; they never include v1 pixels in the regenerated export. The default output is `content/ui/sprites-v3`. The native preview accepts an evidence output directory followed by the sprite package directory.

## Import contract

Use sRGB, **straight alpha**, linear filtering and no mipmaps. Do not colour-key black, white or navy. Transparent exterior pixels contain bled RGB deliberately to prevent dark fringes under filtering. Solid panel interiors are opaque; intentional frame holes remain transparent.

Each texture contains **4 pixels per logical UI unit**. Use `logical_size` or divide `size` by `pixels_per_ui_unit`; do not display every texture pixel as a UI pixel. This preserves the small intended controls while retaining detail for high DPI. Eight transparent pixels surround each sprite. Atlas gutters are separate from these borders.

`nine_slice` and atlas `rect` are in texture pixels. Rectangles are `[left, top, right_exclusive, bottom_exclusive]`. For Bevy `TextureSlicer`, use the given insets and `max_corner_scale = 1 / pixels_per_ui_unit`. A nine-slice preserves its corners while the centre and rails stretch. `min_size` is also in texture pixels: divide by the density for logical minimum dimensions. Icons, portraits, ability and inventory artwork should keep their aspect ratio. Notices marked `resize_axis: horizontal` must retain their logical height so the embedded status badge remains round; use the separate bar and icon assets when a different height is required.

Control families contain `default`, `hover`, `pressed`, `selected`, `focused` and `disabled`. Their dimensions and alpha silhouettes are identical, so state changes do not move the artwork. The engine must disable interaction as well as select disabled artwork. Render labels, values, focus behaviour, keyboard navigation and accessible semantics in the game UI.

## Limits

Generation repaints the original: small ornamental and silhouette differences are intentional and visible in the comparisons. These are raster assets, not vectors. High-resolution masters do not imply infinite scalability. Textures and preview tests do not establish finished screen layout, accessibility, performance or Windows rendering in the game client.
