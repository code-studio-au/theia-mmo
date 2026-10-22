# Independent foliage colour UVs

P1-10/P2-08/P2-10 continuation, 2026-10-21, after
[actual texture defaults](FOREST_TEXTURE_DEFAULTS.md). Two remaining giant-tree
bindings require leaf colour tiling 2×2; their normal/AO coordinates stay separate.
Use ordinary glTF UV channels and Bevy StandardMaterial before introducing a
custom shader. This slice concerns static visual content and inspector evidence;
collision, rebase/resource/mips and normal map adoption remain separate.

The pinned full vegetation shader
`87a2b6f6206262674de9b2ff05d3743ee2a62e0d1bc7eff5b06575ab4c56abe4`
samples independent colour/normal/AO ST fields. Acceptance requires positive,
bounded colour ST only; unchanged primary UVs, geometry, normals, tangents and
auxiliary mapping; independent source colour/alpha/geometry checks; actual Editor
mesh frames and exact source-plan groups; native loaded UV-channel assignments,
full scene preservation, and retained regressions with the final client.

## Implemented contract

The full foliage converter has explicit `--colour-uv` adaptation, requiring a
canonical full-static source draw. Default behavior still rejects nonidentity
transforms. Colour scale must be finite in (0,16], offset bounded to ±16;
normal/AO transforms remain identity. Preserve the existing texture colour-floor,
cutout, normal decoding and unapplied wind/highlight limitations.

`foliage_colour_uv.py` appends `TEXCOORD_1` only for a transformed colour branch
and selects it through `baseColorTexture.texCoord=1`. Normal/AO retain UV0, and
the existing tangent basis remains unchanged. Reflect Unity's upward V through
the transform: glTF V becomes `scale_y * V + 1 - scale_y - offset_y`.
Keep the original binary prefix byte-identical; no geometry/UV0/image rebaking.
The opt-in canonical path uses the same baked file-axis/unit convention as the
qualified Standard converter, with imported model-node placement excluded.

The separate foliage verifier derives expected colour UVs from the raw source
properties and compares the actual added stream within bounded f32 rounding.
It verifies the channel selection, auxiliary mappings and existing source geometry,
winding, normals, tangents, branch selectors, derived colour/alpha/normal pixels
and material properties. Reframe verification now admits this independently
qualified full-foliage profile while preserving all shading streams and images.

`qualify_foliage_group.py` selects every exact mesh/material match to the bound
source witness across the plan. A prefab filename alone is insufficient: these
prefabs contain other renderers and material overrides. A focused regression test
checks same-prefab exclusions and matching bindings from another prefab.
`verify_foliage_groups.py` independently checks group coverage, predecessor
bindings and fresh complete source placements, with no overlap with earlier groups.

The inspector reports actual loaded StandardMaterial colour/normal/occlusion UV
channels weighted by primitive count, plus meshes carrying UV1. This is inspection
evidence only; no custom renderer or ordinary gameplay/movement change is added.

## Qualification

Source and actual Editor checks pass for both exact bindings: thirteen Giant 01
placements and four Giant 02 placements. Canonical positions match independent
ufbx within 5.62 micrometres; actual Editor point sets match with zero observed
error. Each prototype has separate leaf/trunk materials, with colour on UV1 only
for leaf. Seventeen placements contribute 640,346 triangles. Independent complete
composition and source-plan classification pass for the larger candidate.

## Executed native replacement

The qualified visual subset now contains **18,624 source draws / 35 unresolved**.
`build/forest-colour-uv-scene-v1/` preserves all predecessor inputs and adds the
seventeen placements: **18,864 rendered primitives / 6,092,029 triangles / 164
materials / 55 glTF images / 142 mesh definitions**. Multi-primitive ground/ruins
and leaf/trunk splits explain the difference from source draw count. Four triplanar
profiles still apply to 528 primitives using ten extension images; ordinary
Standard materials apply to 18,336 primitives. No minimum-residency claim.

Six native Linux Bevy 0.20 cases pass on **RX 7900 XTX / RADV NAVI31 Vulkan**:
both prototypes, both complete placement groups, full overview and forest view.
Actual loaded materials select **colour UV1 / normal UV0 / occlusion UV0 for all
seventeen leaf primitives** and UV0 for the other 18,319 Standard primitives.
The full scene carries 41 UV1 streams, including 24 preserved predecessor streams;
carrying UV1 is distinct from selecting it for colour. Source and loaded counts
match independently. Active Khronos validation, completed pipelines, no renderer
errors/panics/tangent failures and native bounds within **7.50 micrometres** pass.
The Giant 01 and combined forest images were inspected. Dark lighting, canopy
aliasing/filtering, shadow quality and Unity BRDF/compression parity remain open;
the source rest-geometry/wind/highlight limitations remain unchanged.

The final client additionally passes three legacy single-profile modes and both
actual encrypted two-client town/farmhouse routes. Khronos checks two prototypes,
two groups and the combined scene: **five files, zero errors, seven retained
combined runtime-tangent warnings**. Prototypes/groups have zero warnings, and
native tangent generation passes. Fresh fmt, denied-warning workspace/all-target
Clippy, **35 client Rust tests and 179 Python tests** (172 asset + four GLB + three
Blender reference) pass. Full workspace Rust and native Windows are not claimed.

Combined scene SHA-256:
`c890bde58b6ddb7218345712d1c51a17261558c19a33265ca954a6946a640eb9`.
Final client SHA-256:
`3526f43349e7bb02cb93c3469be23040012a9ffd1c077674869781e9475a99b0`.
Server/Cargo.lock/WESL stay unchanged. Evidence lives in
`build/forest-giant-colour-uv-source-v2/`, `build/forest-giant-frame-v2/`,
`build/forest-colour-uv-scene-v1/`, `build/forest-colour-uv-native-v1-20261021/`,
`build/forest-colour-uv-legacy-native-v1-20261021/`,
`build/forest-colour-uv-map-regressions-v1-20261021/`,
`build/forest-colour-uv-gltf-v1.json` and matching qualification/test logs.
The initial prefab-name-based scratch placements are explicitly rejected and
never composed into the scene; exact-binding groups replace that selection.

Next: the remaining sixteen bindings / 35 draws comprise 25 mixed-slot crystal/
geode draws, seven water draws, one glass, one skydome and one emissive willow.
Use explicit bounded adaptations through standard Bevy where practical, preserving
source slots/geometry and recording effect limitations. Rebase/resources/mips,
lighting, authored object collision and runtime map adoption remain separate gates.
Keep town and opt-in ground collision available; inspect the new Elven folder
after forest completion. All generated/licensed inputs and evidence stay ignored,
and changes remain local.
