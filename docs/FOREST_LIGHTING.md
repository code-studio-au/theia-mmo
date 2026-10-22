# Forest source lighting preview

This is an opt-in inspector continuation of P1-10/P2-08/P2-10 and the row-5
content gate, dated 2026-10-21. The complete static scene remains 18,659 source
draws. Lighting inspection does not establish authored collision, streaming,
Unity rendered equivalence or normal development-map adoption.

The prerequisite is the qualified [shared-resource scene](FOREST_RESOURCES.md).
Acceptance requires independent source identities/layers/poses, actual Editor
cookie imports, native pixel checks, complete scene loading, rejected unsupported
receiver masks and retained rendering/network regressions.

## Source data and supported boundary

The source contains 22 point lights, one spotlight and three directional lights.
The point lights and spotlight affect all source layers. The directional lights
select layers 7, 6 and 2 respectively. Renderer layers belong to each renderer's
GameObject; they are not inherited from its parent. The static plan contains
13,697 layer-0, 4,635 layer-2 and 327 layer-6 draws; terrain is also layer 2.

Actual Linux Unity 6000.6.6f1 imports four model variants and the two cookie
textures in isolated reference projects. Source scripts/scenes are not executed.
The cookie textures import as Alpha8 with alpha generated from grayscale.
The installed Unity AutoLight.cginc samples cookie alpha; reading the PNG's red
channel is insufficient: one texture differs by one byte in approximately 3.95%
of texels. A separate Editor collector temporarily enables readability, exports
actual base-level alpha, then restores the import settings. Importer/platform
settings match before/after; original package images and metadata stay untouched.

The offline source stage binds all 26 original light records, full source masks,
renderer layers, static poses and actual cookie alpha. Independent mathutils pose
checks differ by at most 3.36 micrometres. Geometry, materials, shared images and
custom triplanar images remain unchanged beyond explicit node layer extras and
their geometry-hash binding.

**The native test rejects direct selective-mask application.** Bevy 0.20
RenderLayers filters lights per camera and shadow casters; it does not link PBR
receivers to lights. A two-layer fixture illuminated both quads despite the
original component counts being correct. The client now rejects every light
mask other than all 32 bits, including an empty mask. Component counts alone are
not lighting qualification.

`prepare_standard_lighting_preview.py` creates a separately identified inspector
copy that explicitly omits the three selective directional lights. It preserves
the 23 supported source lights and all other settings; it adds no replacement
sun. The independent verifier checks the source again, every copied file, exact
omitted records and exact remaining metadata. Source masks remain preserved in
the original offline stage. A qualified selective-light solution remains open.

## Explicit rendering adaptations

Standard Bevy components provide ambient light, point/spot lights, DistanceFog,
shadow enablement and cookie textures. Bevy's `pbr_light_textures` feature is
explicitly enabled: without it, texture components load but shaders ignore them.
The native fixture caught that failure before promotion.

- Legacy intensity maps to 10,000 lux per directional unit or 100,000 lumens per
  punctual unit. Serialized source colours are used directly as linear values;
  this is a preview policy, not proof of Unity project colour-space equivalence.
- Trilight ambient becomes its spherical mean `(sky + 2*equator + ground)/4`,
  with brightness 3,000 times source intensity. Directional ambient/SH parity is
  not implemented.
- Source exponential-squared fog uses density 0.009. It obscures the distant
  overview; an interior forest view is the useful inspection view.
- Actual imported cookie alpha becomes grayscale PNG sampled through Bevy's
  linear R channel. Native loading expands it to Rgba8Unorm with one preview mip;
  the source has 12 mips. Source repeat/clamp is requested explicitly.
- Directional cookie projection follows Bevy's actual shader convention
  `local.xy * (-0.5, 0.5) + 0.5`, with source cookie size as XY transform scale.
  This entails a different span/orientation convention; no Unity projection
  equivalence is claimed. The source directional lights remain excluded.
- Static rest poses and source shadow enablement are retained. Bevy shadow
  defaults replace source filtering/bias/strength; animated source light scripts,
  baking, motion and Unity BRDF/effects are not reproduced.

The original native cookie fixtures independently reconstruct surface positions
from the captured camera matrix and sample the expected texture projection.
Directional/spot checks cover 6,451/1,416 interior samples. Dark-cookie 95th
percentiles are zero; bright medians exceed 200/100 RGB8 respectively. Correlation
is 0.588/0.923, reflecting directional saturation. These checks demonstrate actual
projected GPU patterns rather than component presence or Unity rendered parity.

## Local evidence

Licensed images, Editor projects, generated scenes, captures and logs stay ignored
under `build/`:

- Source reference projects: `forest-lighting-editor-v2`,
  `forest-cookie-editor-v1`.
- Complete independently verified source stage: `forest-lighting-source-v2`.
- Explicit supported subset: `forest-lighting-standard-v1`; independent receipt
  `forest-lighting-standard-verification-v1.json`.
- Native cookie application and pixel receipts:
  `lighting-cookie-native-v3-20261021`.
- Full-scene overview and interior: `forest-lighting-native-v2-20261021`.
  Both retain 18,925 primitives / 6,137,378 triangles, 190 materials, 37 shared
  core image handles and 1,421,877,248 loaded core image bytes. Actual primitive
  layer counts are 13,697 / 4,661 / 567 for layers 0 / 2 / 6; primitive counts
  differ from source draw counts because meshes can have multiple submeshes.

Native checks use Linux Vulkan on AMD Radeon RX 7900 XTX (RADV NAVI31), active
Khronos validation, completed pipelines and unchanged bounds. Windows execution
has not been qualified. The initial fixture's component-only success is retained
as a diagnostic, not promoted as receiver-mask or cookie pixel equivalence.

The client adds bounded strict lighting metadata, source-node layer ownership,
hash-bound grayscale cookie loading and actual component/image inventories.
Direct serde is the only lockfile dependency-list addition; dependency versions,
Bevy and Avian pins remain unchanged. Normal map/launcher selection stays on the
previous qualified town; forest lighting remains an inspector option.

Fresh formatting, denied-warning workspace/all-target Clippy, 41 client Rust and
204 Python tests pass. Khronos reports zero errors and seven retained runtime-
tangent warnings on the new scene. Four rehashed source faults (pose, mask, layer and actual
cookie pixels) reject. Three actual CLI cases reject selective masks, missing
lighting metadata and incomplete layer coverage without captures or panics. Three
legacy lit/colour/normal modes and encrypted two-client town/farmhouse regressions
pass on the same client. Local evidence includes `forest-lighting-faults-v1/v2`,
`forest-lighting-failures-native-v2-20261021`,
`forest-lighting-legacy-native-v1-20261021` and
`forest-lighting-map-regressions-v1-20261021`.

Qualified client SHA-256:
`e530dd287bc8341fd9fdd3c4d3844c1867b193f79df34a6afcd9001905572cbf`,
preserved under `build/forest-lighting-qualified-binaries-v1`. Lockfile SHA-256:
`6077de1de211135fa61f1b0c9ef4655d9c10f1455a00af9b8a689e79b8607732`.
All changes remain local.
Next is general authored source collision, beginning with enabled components,
hierarchy/override resolution and finer terrain providers before object budgets.
