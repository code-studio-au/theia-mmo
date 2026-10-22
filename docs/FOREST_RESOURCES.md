# Shared forest image resources

P1-10/P2-08/P2-06 continuation, 2026-10-21. The complete static inspector scene
now shares **37 actual core image handles instead of 77** across 298 texture slots.
Loaded CPU Image payloads fall from **3,124,764,672 to 1,421,877,248 bytes**.
Both paired native views are pixel-identical outside the HUD. This is measured
image sharing, not GPU allocation, streaming capacity or normal map adoption.
All 18,659 planned source static draws remain covered.

Prerequisites were complete static source/Editor/material-preview coverage, the
bounded catalog and immutable PNG variant bindings. Acceptance requires complete
geometry/material/image/role/sampler equivalence, actual loaded image inventory,
paired native captures, source-preserving failure cases and retained legacy checks.

## Implemented behavior

`prepare_forest_resources.py` accepts a freshly verified material catalog. It
normalizes bound external PNGs in memory, then uses the existing byte-preserving
extraction helper to share variants by exact PNG SHA-256, colour space and glTF
sampler. It retains all scene nodes, primitives, source attributes, material factors,
UV channels and known unlit/emissive/shadow metadata. Source texture roles must be
complete and unambiguous before sharing. No images are resized or recoloured.

The inspector stage places GLB/metadata at its asset root and PNGs under `shared/`.
Custom profile metadata changes only its geometry hash; ten custom PNGs remain
byte-identical and retain the existing typed filenames. The superseded v1 stage
uses package-style `scene/../shared` layout and remains a CPU-only diagnostic;
`build/forest-resource-stage-v2/` is the native-qualified inspector stage.

`shared_images.py` admits the known material extensions only through an explicit
inspector opt-in. Its default remains unextended, matching Rust package admission.
No content manifest, server, protocol, package admission or normal world path is
changed by this slice. Mips remain one level; custom source projections still
require separate rebase integration. All existing shared resource budgets remain.

`verify_forest_resources.py` imports no resource author. It freshly verifies the
predecessor catalog, independently derives PNG/colour/sampler variants, checks
canonical dependencies, all **944 accessors**, complete hierarchy/primitive/profile
metadata and semantic material/image equality. All 298 texture slots remain; image
references share immutable variants. Custom metadata and ten custom image bytes
are checked separately. Source files stay unchanged.

`preview_images.rs` reports actual loaded Bevy `GltfMaterial` image handles for
base colour, emission, normal, occlusion and metallic/roughness roles, including
triplanar placeholders retained by the source asset. It records roles, dimensions,
formats, samplers, mip counts and CPU data lengths. These measurements exclude
custom-profile images, process overhead and measured GPU allocation/capacity.
A missing image/material delays reporting until its data is available.

## Executed qualification

Source GLB SHA-256:
`6384df9f1e91398811fff25450f6ea5cdac639bfe7abecefcf16a98fea808030`.
Shared inspector GLB SHA-256:
`01a39b00d1e6b95be7e96a5aa4b3bfa047032371318475eac3fd232a55d43e49`.

Complete equivalence checks pass: **18,925 primitives / 6,137,378 triangles /
190 materials / 158 mesh definitions**. The output has 37 glTF images. Actual
core handles reduce 77 → 37, with the exact measured CPU byte totals above. All
37 distinct variants have matching native dimensions, sRGB/linear formats,
samplers, CPU bytes and one mip level across source/candidate. Four custom profiles
still apply to 528 primitives using ten additional images; their 160 MiB RGBA8 base
estimate is separate from the measured core payload totals.

Native Linux **RX 7900 XTX / RADV NAVI31 Vulkan**: source/shared overview and
source/shared forest view all pass active Khronos validation, completed pipelines,
expected counts/bounds, shadow flags, UV channels and loaded HDR coefficients 2.6.
Both paired comparisons, excluding the first 110 HUD rows, have **zero observed
RGB8 pixel differences**. Maximum native bound error stays below 0.00001183 m.
Three fresh legacy triplanar modes also pass on the new client. No render resolution,
geometry or material-quality reduction establishes the sharing result.

Four real faults reject after hashes/metadata are rebound: changed geometry,
changed sampler with renamed canonical resources, linear/sRGB role aliasing and a
changed PNG with rebound filename/descriptor. Khronos reports zero errors and
seven retained tangent warnings for the shared forest. Fresh fmt, workspace/
all-target denied-warning Clippy, **38 client Rust tests** and **202 Python tests**
pass (193 asset, four GLB and five Blender). Retain the preceding encrypted town/
farmhouse regressions on the willow binary; no gameplay code changes here.

Final client SHA-256:
`731d3640be5a999e3699f409ec9e7d0f762edb76f824ca08a4af82b46a9b30f2`.
Cargo.lock and all four imported engineering documents remain byte-identical.
Evidence under ignored `build/`: `forest-resources-native-v1-20261021`,
`forest-resources-legacy-native-v1-20261021`, `forest-resources-faults-v1.json`,
`forest-resources-gltf-v1.json` and `forest-resources-verification-v2.log`.
Licensed inputs/outputs remain out of Git; all work stays local. No Windows native
execution, VRAM total, sustained frame-time or MMO-scale capacity is claimed.

Next: scene lighting/fog and authored object collision, with rebase/mips and runtime
resource/lifecycle gates still required before normal forest adoption. The raw
lighting audit finds 26 enabled Light components, including inactive-hierarchy
questions still to resolve; it is not a qualified runtime lighting import.
Elven folder inspection remains queued after forest completion.
