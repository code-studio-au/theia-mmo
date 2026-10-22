# Emissive willow and complete static draw coverage

P1-10/P2-08/P2-10 continuation, 2026-10-21. The willow adds the final planned
binding: **18,659 source static draws / zero unresolved static draws in the plan**.
This completes the selected highest-detail static geometry/material-preview
coverage. It does not complete source shader effects, particles, lighting, resource/
rebase/mip handling, authored object collision or normal forest map adoption.
The town remains the default; the forest ground remains separately opt-in.

Prerequisites were the exact source plan, calibrated canonical/Editor-frame path,
branch/alpha/normal/image verifiers and the bounded material catalog. Acceptance
requires explicit source effect limits, independent HDR mask pixels/roles, exact
Editor geometry and source poses, full composition, native material state/rendering,
rehashed faults and legacy/multiplayer regressions.

## Explicit source adaptation

`unity_foliage_emission.py` adds an opt-in `synty-full-emissive-preview-v1` profile;
ordinary full-foliage conversion still rejects nonzero emission and varying noise.
The source shader remains pinned to GUID `6697e1f185bc8b840a2f02567abf15b5` /
SHA-256 `87a2b6f6206262674de9b2ff05d3743ee2a62e0d1bc7eff5b06575ab4c56abe4`.
Original source properties and omitted spatial noise colours remain recorded.
The base-floor/surface constructor uses an explicit surrogate with noise colours
collapsed and emission disabled; independent verification uses original source.
This retains branch colour floors, alpha, surface factors and rest geometry while
omitting spatial albedo noise, wind/jitter/gust effects and Unity BRDF parity.

Two actual leaf masks use identity UV0, source sRGB-red decoding and bilinear/repeat
sampling: `Leaves_Willow_01.tga` and `Gradient 1.png`, both 1,024². Their two source
emission colours sum and multiply by source amount **1.3**. The source pulse texture
is unassigned; the pinned shader defaults it to white, forcing its clamped pulse
multiplier to one. This binding therefore has static emission without inventing
a time sample. Trunk emission is zero. Other pulse/trunk profiles reject.

The derived RGB8 emission PNG encodes half the two linear mask-weighted colours;
ordinary glTF `KHR_materials_emissive_strength` **2.6** restores source HDR energy.
The base-floor images remain 1,024² leaf / 4,096² trunk with exact source alpha.
Emission is exact at quantized source texels under the chosen linear workflow;
filtered/mip samples, Unity project colour space and compression parity remain
unqualified. No custom vegetation runtime shader is introduced.

The explicit preview accepts no active source normal maps. Its initial optional
Blender tangents were ambiguous at shared vertices and rejected. The final profile
omits those unnecessary streams; it preserves positions/UVs/normals/indices and
does not relax the tangent or normal limits. A subsequent export-report variable
error also rejected before publication; both failed logs remain diagnostic.
`build/forest-willow-source-v3/` is the qualified source stage.

Offline static readers now admit only declared material-level unlit/strength
extensions. Strength payloads have exactly one finite numeric scalar in 0..16,
without index references; booleans, extra fields, unknown/location-mismatched
extensions reject. Frame conversion retains these validated material payloads.
This does not enable arbitrary extensions in package/streaming paths.

`verify_unity_foliage_emission.py` imports no author. It checks original raw source
properties/defaults and every decoded emission pixel, material strength, UV channel,
sampler and texture role. The existing foliage verifier still checks complete
branch geometry/winding/UVs/normals, cutouts and base-floor pixels. The inspector
also records actual loaded StandardMaterial emission RGB coefficients, after
Bevy's standard loader applies glTF strength; these are coefficients, not measured
screen pixels or emitted light on surrounding surfaces.

## Executed qualification

Actual source branches total **24,398 triangles**: 18,546 leaf / 5,852 trunk.
Independent complete base/emission pixels and geometry checks pass. Leaf cutouts
retain 202,900 texels and discard 845,676 at source cutoff 0.5. Maximum source mesh
normal error is **8.40e-5**, below the unchanged 0.00018 bound. Actual Editor mesh
point-set/frame error is zero observed; source placement arithmetic error is below
7.11e-15 m. The source plan and all predecessor assets remain unchanged.

SHA-256 values:

- Source prototype: `e747436fac05583aead05306256e2de498ef1e02838bc1f1ec171bdfe57d0753`.
- Editor-frame prototype: `669776e175634b3a7c7f195d2752e022ad5c775e50f59cc52977822e0a65f75a`.
- Source placement: `4f09bd65dd079d7c1ada063eccadad8272172145dda967585a0b05a78abf1145`.
- Combined forest: `6384df9f1e91398811fff25450f6ea5cdac639bfe7abecefcf16a98fea808030`.

`build/forest-willow-scene-v1/` preserves all earlier input geometry/material/image
roles: **18,925 primitives / 6,137,378 triangles / 190 materials / 77 glTF images /
158 mesh definitions**. Four triplanar profiles still apply to 528 primitives using
ten extension images. Actual Standard instances total 18,397, including seventeen
colour-UV1 instances; one mesh is unlit, seven disable casting and one disables
receiving. The catalog retains its sixteen-Standard-input cap. Bound coverage in
`build/forest-willow-coverage-v1.json` reaches all 18,659 planned static draws.
Unsupported source particles/effects are outside that static-plan claim.

Native Linux **RX 7900 XTX / RADV NAVI31 Vulkan** checks pass: willow prototype and
actual placement, combined overview/forest view and all three legacy material modes.
Active Khronos validation, expected bounds/counts/channels, completed pipelines and
actual loaded HDR coefficients 2.6 pass; maximum bound error is below 0.00001183 m.
Screenshots were inspected. Full canopy views remain dark and do not qualify scene
lighting. Encrypted town/farmhouse two-client regressions pass on the final client.
Four rehashed faults reject changed strength, swapped emission/base texture role,
a changed emission pixel and changed cutoff. Four Khronos GLBs have zero errors;
three have zero warnings and the combined scene retains seven tangent warnings.

Fresh fmt, workspace/all-target denied-warning Clippy, **38 client Rust tests** and
**201 Python tests** pass (192 asset, four GLB and five Blender). Final client:
`a0a741a006a908a093f62dc68e380c72ab3fa785bf830010ac4aead1f95088c2`.
Server and Cargo.lock remain unchanged during this slice. Evidence roots under
ignored `build/`: `forest-willow-frame-v1`, `forest-willow-prototype-native-v1-20261021`,
`forest-willow-scene-native-v1-20261021`, `forest-willow-legacy-native-v1-20261021`,
`forest-willow-map-regressions-v1-20261021`, `forest-willow-faults-v1.json` and
`forest-willow-gltf-v1.json`. All work stays local; Windows execution is not claimed.

## Next measured gate

`build/forest-image-inventory-v1.json` estimates 77 declared image entries at
3,124,764,672 RGBA8 base bytes versus 37 distinct texture variants at 1,421,877,248
bytes. Ten custom-profile images add a separate 167,772,160-byte base estimate.
These are offline estimates, not measured CPU/GPU residency, mips or capacity.
Qualify immutable sharing and actual loaded handles before further promotion,
then lighting, rebase/mips and authored collision/runtime acceptance. Elven folder
inspection remains queued after forest completion.
