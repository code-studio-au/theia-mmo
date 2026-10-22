# Actual Editor texture defaults

P1-10/P2-08/P2-10 continuation, 2026-10-21. The visual subset now contains
**18,607 source draws / 52 unresolved** after adding the ten grass placements.
This remains an origin-zero inspector scene, with object collision, resources,
mips, rebase and normal map adoption still open.

The [triplanar batch](FOREST_TRIPLANAR_BATCH.md) rejected Grass because its colour
and normal PNG metadata uses exactly the legacy sampler block `filterMode: -1`,
`aniso: -1`, `mipBias: -1`, `wrapMode: -1`, with no separate U/V fields. Resolve
this from actual import state rather than assuming defaults. Acceptance requires
original source/metadata hashes, compiled read-only Editor collection, exact
texture identities, source-sized imports, independently checked effective sampler
settings, actual geometry/placements, native rendering and pixel checks.

## Implemented and executed

`unity_texture_reference.py` stages only selected PNGs and their original metadata,
an empty package manifest and `qualification/TheiaTextureReference.cs`. No source
scripts, materials, scenes or packages are copied. The collector reads actual
TextureImporter/Texture2D settings and persistent IDs without changing the source.
The independent inspection path binds collector/request/report/version/source
hashes, exact coverage, staged/backup metadata and original PNG dimensions, and
rejects invalid sampler/mip/boolean fields. Tests use synthetic guard fixtures;
they are not presented as Editor execution.

Native Linux **Unity 6000.6.6f1** compiled and ran the collector on four actual
Grass/Mud colour/normal PNGs in `build/forest-grass-texture-editor-v1/`.
Every original PNG and metadata file stays byte-identical. The actual textures
resolve to **bilinear/repeat U/repeat V, anisotropy 1, mip bias 0**. All are 2048²
with twelve imported mips; colours import as DXT1/sRGB, normals as DXT5/linear.
This establishes effective sampler state for these exact imports, not Unity
compression/BRDF/rendered parity or the runtime Bevy mip policy.

The triplanar author and separate raw-source checker accept that exact legacy
block only with bound, successfully inspected actual Editor evidence for the
selected texture. Missing evidence retains rejection; arbitrary sampler variants
and unknown defaults remain unsupported. Metadata binds project, request, report,
collector and Editor version. Explicit sampler profiles remain unchanged.

`build/forest-grass-frame-v1/` passes raw source material/UV/normal/tangent/image
checks, actual Editor point sets (zero observed error), and all **ten exact source
placements** (2,360 triangles; assembly error below 1e-13 m). Their authored poses
extend the scene bounds; no clipping, recentering or guessed compensating offset
is applied. New batch v2 preserves the seventeen previously qualified prototype
and placement files byte-for-byte, adds Grass, and freshly rechecks all eighteen
bindings / 528 placements against actual source and Editor references.

The combined `build/forest-triplanar-scene-v2/preview.glb` passes complete input
composition checks: **18,830 rendered primitives / 5,451,683 triangles / 160
materials / 49 glTF images / 140 mesh definitions**. Source draw count is 18,607;
multi-primitive ground/ruins explain the different primitive count. Four profiles
apply to **176 Moss 01, 257 Dirt Leaves, ten Grass and 85 Moss 02** placements.
Ten shared extension images decode to **160 MiB RGBA8**; the 31 existing Standard
image variants and embedded placeholders remain separate resource costs.

Four combined native Linux Bevy 0.20 views and three Grass prototype modes pass
on **RX 7900 XTX / RADV NAVI31 Vulkan** with active Khronos validation, exact
mapping/counts, complete pipelines and no renderer errors, panics or tangent
failures. Combined bounds differ by at most **11.30 micrometres**. The Grass
colour capture was visually inspected. **137 colour + 137 decoded-normal pixel
samples** pass at zero/one maximum sRGB8 error under the unchanged four-level
limit. Missing-reference and rebound report-hash faults both reject in real
raw-source verification. No full-scene pixel or Unity rendered parity claim.

Khronos reports zero errors/warnings for Grass prototype/placements; the combined
scene retains seven runtime tangent-generation warnings, with native generation
passing. Fresh fmt, denied-warning workspace/all-target Clippy and **175 Python
tests** (168 asset, four GLB, three Blender reference) pass. Client/server binaries,
Cargo.lock and WESL remain unchanged from the qualified catalog checkpoint, whose
multiplayer evidence remains retained. No new multiplayer/Windows execution claim.

Combined scene SHA-256:
`735e4dc78753c1685e6f91fbefd3c43985c40b7b0a5eab52af3885a261b62adf`.
Collector SHA-256:
`de54d74dcf6ed24c2e7fad4a71b82197d165dd57c7dcf462df1ed38e257dfbe9`.
Evidence: `build/forest-grass-texture-reference-final-v1.json`,
`build/forest-triplanar-batch-v2/independent-coverage.json`,
`build/forest-triplanar-scene-v2/independent-verification.json`,
`build/forest-triplanar-scene-native-v2-20261021/`,
`build/forest-triplanar-grass-native-v1-20261021/`,
`build/forest-triplanar-grass-*-pixels-v1.json`, `build/forest-grass-faults-v1.json`,
`build/forest-grass-gltf-v1.json` and matching test/qualification logs.

Next: the source shader uses independent leaf colour/normal/AO texture transforms.
Two giant-tree bindings / seventeen draws require leaf colour tiling 2×2 while
normal/AO UVs remain unchanged. Qualify a separate standard glTF UV channel for
that colour role before converting them; do not scale every texture's UVs together.
The other 35 remaining draws include mixed crystal slots, glass, water/sky and
emissive vegetation. Rebase/resource/mips, authored object collision and runtime
adoption remain separate gates. Elven Realms folder inspection remains queued
after forest completion. All assets/evidence stay ignored and work stays local.
