# Source-derived forest sky

P1-10/P2-08/P2-10 continuation, 2026-10-21. The actual skydome adds one source
placement: **18,651 source draws / eight unresolved**. Seven water draws and one
emissive willow remain. This is qualified inspector content; full-map collision,
lighting, resource/rebase/mip integration and normal runtime adoption remain open.

Prerequisites were the immutable source plan, actual Editor vertex/index report,
source renderer/material overrides and the qualified combined catalog. Acceptance
requires exact source geometry/winding/placement, independent gradient/image checks,
actual unlit/shadow state, native composition and clean failure/regression checks.

## Implemented behavior

`unity_sky.py` pins shader GUID `b79e425cce5c90642a8dd66b6852353e` and SHA-256
`24694dc81dc4d3f6e60bb9974b13397e1a6920d25d56ec11784a8b3bb2ac165c`.
Its active world-height branch is `lerp(bottom, top, pow(clamp((offset + y) /
distance, 0, 1), falloff))`. Source offset -52.2, distance 55.38 and falloff 2
remain explicit; UV-based and textured source variants reject.

`convert_unity_sky.py` uses actual Editor positions/indices, calibrated X reflection
and reversed winding: 114 source vertices and 160 triangles. It exports a static
source-world-height lookup through ordinary glTF `KHR_materials_unlit`, opaque
backface-culling material and a clamped bilinear 2 × 1,024 RGBA8 PNG. Linear source
colours are sRGB encoded; endpoint-centred UV0 coordinates remain unclamped before
interpolation, so texture clamping happens at the correct source heights. This is
a quantized static gradient; Unity colour-space/tonemapping parity and changing
runtime sky poses are unqualified. No custom runtime sky shader is introduced.

`verify_unity_sky.py` derives material/renderer properties from raw source, verifies
the pinned shader, exact Editor positions/indices, all height UVs, every decoded
LUT row, exact material/sampler state and independent source placement. It imports
no sky author. Composition separately checks every predecessor input and dependency.

Offline static readers/composition now admit only declared, material-level, empty
`KHR_materials_unlit` payloads. Unknown extensions, wrong locations, undeclared use
and nonempty payloads reject. This does not enable all extensions or all package/
streaming tools. `preview_render_flags.rs` applies standard Bevy `NotShadowCaster`
and `NotShadowReceiver` markers from strict boolean material extras in the inspector.
Unmarked materials retain normal defaults; malformed flags exit with an error.
Normal gameplay adoption of those source flags remains open.

Capture readiness distinguishes temporary missing/importing shaders from terminal
processing/module failures. Terminal failures and capture timeout now exit with
an error instead of waiting for an assertion panic. The initial UV1-only unlit
fixture exposed a Bevy 0.20 shader declaration failure; that rejected diagnostic is
retained. The qualified sky uses UV0. No engine version or plugin pin changed.

## Executed qualification

`build/forest-sky-source-v4/` is the final source stage; earlier v1–v3 stages are
superseded diagnostics. Prototype SHA-256 is
`330f9c61cda4257d613b3bbc3684c8c88a64bf09f1258db0fd348d0fde20f5d3`;
placements SHA-256 is
`ea907abcb012f1d4bdfba82e07cea2935d91649a1dbc9c07c8abaa42f580771c`.
Source geometry and complete placement checks pass. Four rehashed source faults
reject shadow changes, height UV changes, triangle winding and LUT colour changes.

`build/forest-sky-scene-v1/preview.glb` SHA-256 is
`969ba1deb06e23411a6a3759a5a148c1f1368fef8ee2086677842dc4f06e7115`:
18,916 primitives / 6,110,380 triangles / 184 materials / 74 glTF images /
153 mesh definitions. Four triplanar profiles still apply to 528 primitives using
ten extension images. Standard materials total 18,388 instances, including
17 using colour UV1. Exactly one material is unlit and one mesh has each disabled
shadow marker. Source coverage is bound in `build/forest-sky-coverage-v1.json`;
all previous inputs remain independently checked.

Native Linux checks pass on **RX 7900 XTX / RADV NAVI31 Vulkan**: prototype,
source placement, combined overview and forest view, plus three legacy triplanar
modes. Active Khronos validation, expected actual material/shadow state, completed
pipelines, screenshots and bounds all pass; maximum bound error is below
0.00001183 m. Two native negative cases exit nonzero without panic or capture:
UV1 shader failure in 1.47 s and malformed shadow flags in 0.52 s. Combined forest
lighting remains visibly dark; these captures do not qualify atmosphere tuning.

Encrypted two-client town and farmhouse regressions pass on the final client.
Fresh fmt, workspace/all-target denied-warning Clippy, 38 client Rust tests and
193 Python tests pass (184 asset, four GLB and five Blender checks). Khronos checks
three GLBs with zero errors: both sky files have zero warnings; the combined file
retains seven runtime-tangent warnings. Final client SHA-256 is
`75d46a365d6357c872223875647a8c27363331404aeaf5f5ba8356a473e9ff45`;
server and Cargo.lock remain unchanged during this slice. A qualified binary copy
is preserved under `build/forest-sky-qualified-binaries-v1/`.

Evidence roots: `forest-sky-prototype-native-v3-20261021`,
`forest-sky-scene-native-v1-20261021`, `forest-sky-failures-native-v1-20261021`,
`forest-sky-legacy-native-v1-20261021`, `forest-sky-map-regressions-v1-20261021`,
and `forest-sky-gltf-v1.json`, under ignored `build/`. No licensed assets or build
outputs enter Git; all work stays local. Windows execution is not claimed.

Next: remaining water/emissive material coverage, then lighting, bounded resources/
rebase/mips and authored object collision before normal forest map adoption.
The new Elven Realms folder remains queued after forest completion.
