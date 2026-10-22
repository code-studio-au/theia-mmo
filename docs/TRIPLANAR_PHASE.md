# Bounded world-projected texture phase

P1-10 / P2-08 prerequisite, 2026-10-21. The standard Bevy triplanar material
extension now accepts a bounded source texture-origin offset. This qualifies the
shader calculation under translated inspection geometry. Normal-world automatic
frame updates and material/dependency admission remain separate work.

The inspector's optional `--texture-origin x,y,z` requires triplanar metadata and
finite coordinates within ±1023 m. The default remains zero. The offset describes
the source-space translation removed from the inspected scene. For each material
profile the CPU applies the source X mirror, multiplies by the actual f32 tiling
in f64, and reduces the result modulo one texture period. Only this bounded phase
is uploaded. The shader adds it to scaled local positions before signed planar
projection and PNG orientation. Colour and normal textures use the same phase.

This preserves the existing Bevy extended StandardMaterial, MikkTSpace and source
normal unpacking. It does not send large global coordinates to the shader or
introduce a new renderer. It also does not yet connect the offset to the normal
client's floating-origin resource: that integration needs bound runtime profiles
and actual chunk readiness/resource checks.

## Qualification

The actual Unity-qualified moss prototype retains every original stream and child
pose beneath a translation wrapper. Native views compare the prior saved renderer
against the new zero-origin renderer and offsets `[13.25,-7.5,9.75]` and
`[-193.125,101.5,67.75]`. Each has lit, unlit-colour and pre-TBN-normal views.

- **1,056 independent source-pixel checks** pass using actual camera matrices,
  source geometry and repeat/bilinear PNG sampling. The oracle receives the
  expected source translation independently of the shader/phase calculation.
- Nine paired mode/offset cases compare 132 geometric interior pixels each.
  Zero-offset views are exact; translated views differ by at most one RGB8 level.
- A valid render with the offset deliberately omitted fails 74 of 132 source
  colour samples, maximum error 25 levels, against the unchanged four-level limit.
- Two full-forest lighting/catalog regression views retain all 18,925 primitives,
  four custom material profiles, ten custom image handles and 37 core image
  handles. Both are pixel-identical to the preceding qualified views outside HUD.

All 15 native views ran on Linux, AMD RX 7900 XTX / RADV NAVI31, Vulkan with
Khronos validation. Pipelines compile and clear; no validation errors, panics or
unexpected shader/tangent failures occur. No Windows execution or cross-build is
claimed. Fresh format/diff checking, denied-warning workspace/all-target Clippy,
42 client tests and 218 asset Python tests pass. The preceding network/package
and Blender/reference results remain retained.
All three translated fixture GLBs have zero Khronos errors or warnings.

Local ignored evidence:

- `build/triplanar-phase-native-v1-20261021/verification.json`, paired/source
  pixels, previous/new client hashes and deliberately wrong origin.
- `build/triplanar-phase-full-scene-v1-20261021/paired-pixels.json`, exact forest
  regressions with the existing explicit lighting approximation.
- `build/triplanar-phase-client-tests-v1.log`, 42 client tests.
- `build/triplanar-phase-clippy-v1.log`, denied-warning Clippy.
- `build/triplanar-phase-python-tests-v2.log`, asset tests.
- `build/triplanar-phase-gltf-validation-v1.json`, three original/translated files.

Qualified native client SHA-256:
`97a10757abb33e5e6164f285614f2b45cd107043c8b58673ff898ab7aa0362a8`.
The CLI help text was subsequently expanded; renderer/shader behaviour is unchanged.

Next: bind self-contained triplanar parameters/images to immutable runtime content,
reuse compatible image/material handles across package roots, and connect phase
updates to actual render rebasing before qualifying normal forest startup. Mips,
render flags/lighting, water, movement/camera and adoption remain explicit gates.
