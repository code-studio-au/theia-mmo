# P2-14 — regenerated sprite artwork

The source-preserving v1 export could not recover detail absent from the supplied
raster crops. The user requested new artwork and a direct regeneration comparison
after reviewing the panel, button and party-icon samples. This slice regenerates
the approved final package; the rejected early UI ZIP and contact sheet are not
inputs. It does not integrate the new artwork into the game screens.

Prerequisites are the approved final source archive, the v1 canonical naming and
layout metadata, the three accepted regeneration samples, and the existing Bevy
0.20 atlas preview. The built-in image generator supplies individually generated
masters. Deterministic finishing, state derivation and packaging use the user's
earlier explicit authorization. No new gameplay, network or engine systems are
introduced.

The export uses regenerated masters only. Repeated control states, panel sizes,
colour variants, mirrored arrows and composite controls share artwork instead of
being independently redrawn. All six states of a control retain the same alpha
silhouette and dimensions. Coloured glyphs retain their colour in selected states;
disabled glyphs are desaturated. Notification badges sit outside the horizontal
stretch region and retain their logical height.

The runtime contract uses four texture pixels per logical UI unit, eight pixels
of transparent padding, bled exterior RGB, straight alpha, sRGB, linear filtering
and no mipmaps. Master finishing removes faint alpha debris, makes near-opaque
paint solid and preserves deliberate holes. Nine-slice insets and atlas rectangles
remain texture-pixel measurements; the Bevy preview applies the density when
scaling corners. Individual master PNGs remain unchanged in a separate archive.

`tools/ui_sprites/regenerate_export.py` creates the runtime export.
`regeneration_review.py` records exact prompts/hashes, validates the export and
creates equal-display-size comparisons, state sheets and stretch samples.
`package_regenerated.py` requires completed native evidence before producing the
runtime and master ZIPs. All three use the existing canonical source mapping;
the master archive contains that mapping and generation jobs for reproducibility.

Completed on 22 November 2026: 112 generated masters produce 206 canonical
assets / 416 runtime PNG sprites, including 42 six-state families, packed into
nine 2048-square RGBA atlases. All 408 source crops have an audit disposition.
Exact generation prompts, master hashes and the approved archive hash are recorded
in `content/ui/sprites-v3/generation.json`. Raw masters are delivered separately
and are not committed to Git.

Side-by-side review covers all nine canonical comparison sheets, four state
sheets, light/dark backgrounds and resized panels. Follow-up corrections removed
unwanted frames from the list/maximise glyphs, confined item-rarity recolouring to
the border, restored success/error badge colours, and prevented badge stretching.
The new artwork is sharper at enlarged sizes and uses a consistent navy inset;
some original glows, ornamental details and proportions intentionally differ.
The comparison viewer retains both versions at the same logical display size.

Qualification passed:

- Thirteen Python pipeline tests, including alpha holes/padding, aligned states,
  coloured selected glyphs, notice stretch limits and rarity-border recolouring.
- Export validation of PNG hashes/dimensions, transparent padding, atlas bounds,
  non-overlap, exact atlas pixels, six-state silhouettes and nine-slice metadata.
- `cargo fmt --all --check`, denied-warning release Clippy and release build for
  the `ui_sprite_preview` example (client/server/editor/nav-bake/network package
  selection used to unify the existing feature graph).
- Ten native Bevy preview pages, all completed without GPU errors on Linux,
  AMD Radeon RX 7900 XTX / RADV NAVI31 / Mesa 26.2.7 / Vulkan. Captures include
  every atlas entry and resized nine-slice samples. Two existing unused asset
  watcher warnings do not affect rendering.
- Local headless Chrome comparison checks: 416 selectable entries, decoded
  images, equal display sizes, zoom/background changes and previous/next controls.

Evidence is under `build/ui-sprites-regenerated-v3/`: `qa/comparison.html`,
comparison/state/stretch PNGs, `native/report.json`, ten native captures and
`viewer-report.json`. The runtime directory includes the validation and native
reports. Delivery ZIPs and SHA-256 sidecars are in the same build directory:
`Theia_UI_Regenerated_Sprites_v3.zip` includes the runtime and visual evidence;
`Theia_UI_Regenerated_Masters_v3.zip` includes unchanged masters, exact prompts,
generation jobs and derivation mapping. The ZIPs are CRC-checked after writing.

Limits: regeneration changes small ornamental and silhouette details. These are
raster assets with finite resolution. Asset validation does not establish finished
screen layout, keyboard/accessibility behaviour, client performance or Windows
rendering. The consuming controls must enforce disabled interaction and keep text
live; the sprite state alone does neither.
