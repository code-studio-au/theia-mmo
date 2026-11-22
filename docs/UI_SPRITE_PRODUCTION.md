# P2-14 — production sprite preparation, 22 November 2026

The user requested a final production pass on `Theia_UI_Final_Sprite_Package.zip`
and explicitly approved deterministic cleanup preserving its artwork. The earlier
UI ZIP and contact sheet were rejected and are not inputs. This slice prepares
assets and import metadata; it does not integrate them into account/game screens
or certify the separate unfinished UI changes in the working tree.

Prerequisites: supplied final sprite archive, its original manifest and the
existing standard Bevy 0.20 renderer. No new engine infrastructure, network
protocol, account data or simulation behaviour is introduced.

Implemented: 206 named canonical assets, 42 aligned six-state families, 416 PNGs,
one 2048-square atlas. Exterior white matte/speck removal, conservative highlight
repair, explicit split crops, independent slider/bar/Boolean parts, transparent
padding and RGB bleed, deterministic packing, exact source provenance and source
crop disposition. Nine-slice metadata is measured in final output pixels;
non-sliceable artwork preserves aspect ratio. Names/values remain live text.
Every one of 408 original PNG crops has an audit disposition; captions and merged
sample-data compositions are not silently treated as runtime controls.

Decision: reuse a canonical control silhouette for all interaction states with
RGB-only variants. Original source states were unverified and had inconsistent
geometry. Retain painted border variation; do not invent detail by upscaling.
The manifest distinguishes default source cleanup from derived RGB states.
Acceptance requires valid RGBA/alpha safety, identical state geometry, atlas pixel
identity/nonoverlap, usable slice centres, full source accounting, and native
rendered evidence using the actual atlas.

Validation:

- Seven pipeline regression tests pass: outside matte versus enclosed highlight,
  keyed-highlight repair versus intentional holes, alpha-preserving states,
  nine-slice corner preservation, atlas round trips (including invisible RGB),
  rejection of the wrong archive before output mutation, and final export contract.
- Export validator passes all 416 sprites/42 families/408 audit records. Final
  images and state sheets inspected on light/dark backgrounds; large slice samples
  inspected. Repeat builds are checked by full manifest/file SHA-256 identity.
- `cargo fmt --all --check` and denied-warning Clippy for the new example pass.
  Matching release example build passes. Existing game logic is unchanged by this
  slice; earlier uncommitted UI work has its own validation status.
- Native standard-Bevy preview renders all 416 atlas entries across nine pages,
  plus one page of enlarged nine-slice controls, at 1440 × 1000. Linux Vulkan:
  AMD Radeon RX 7900 XTX / RADV NAVI31, Mesa 26.2.7 kisak. No GPU validation errors.
  Two harmless asset-watcher warnings concern the example's unused default asset
  directory; textures are decoded from the package directly.

Evidence: `build/ui-sprites-production/{tests.log,build.log,clippy.log,qa/,native/}`.
Delivery: `build/ui-sprites-production/Theia_UI_Production_Sprites_v1.zip` plus
SHA-256. Runtime assets live in `content/ui/sprites-v1/`; source-preserving build,
validator, tests and packager are in `tools/ui_sprites/`. The source archive remains
unchanged in Downloads. `apps/client/examples/ui_sprite_preview.rs` is the native
atlas/slice qualification harness and never starts account/server services.

Limits: source raster resolution, no Windows execution or target-DPI release
qualification, no replacement fonts or screen backdrops. A sprite's disabled state
does not disable input; consuming widget logic must enforce it. Composite menus,
chat/logs, lists, toggles, dropdowns and game settings still need their existing
widget behaviour and final integration checks. See the package README for the
full renderer/import/composition contract.
