# Explicit portable glass adaptation

P1-10/P2-08/P2-10 continuation, 2026-10-21. The lantern's glass now joins the
qualified visual subset: **18,625 source draws / 34 unresolved**. It uses ordinary
glTF/Bevy PBR blend rendering, with explicit differences from Unity lighting.
This is visual content qualification; collision and normal map adoption remain open.

Actual prerequisites were the pinned source plan, canonical mesh/frame path and
qualified combined scene. Acceptance requires an explicit source profile, exact
source render binding, geometry/UV/normal checks, actual Editor point sets and
placement, complete combined scene preservation, native transparency rendering
and source/output rejection tests.

## Implemented adaptation and limits

`glass_material()` accepts only built-in Standard shader 46/type 0, the exact
premultiplied keyword/transparent queue and expected blend/depth/reflection state.
It requires an untextured, nonemissive source, bounded RGBA/metallic/smoothness and
strictly partial opacity. The ordinary opaque profile still rejects this material.
`preview_unity_static.py --standard-material --glass-blend` is explicit and requires
a canonical scene draw. It preserves canonical geometry/UV/normal streams and
exports source tint/opacity as linear base colour, metallic and `1-smoothness`
roughness, with `alphaMode=BLEND` and back-face culling. No image dependency is
invented. `verify_unity_static.py` independently reads raw source eligibility and
checks the material, empty dependency graph and geometry; it imports no author.
The existing exact-binding/frame group path accepts this verified Standard profile.

Source tint is `[0.6226415, 0.0734247, 0.57949954, 0.50980395]`, metallic 0 and
roughness 0.187. These are a portable PBR adaptation. glTF BLEND does not preserve
Unity Standard's premultiplied specular lighting; Unity project colour space,
BRDF and refraction parity are not claimed. Bevy 0.20's pinned glTF loader maps
this standard alpha mode to `AlphaMode::Blend`. No runtime shader/loader, binary,
transport, movement or engine/plugin pin changes are introduced by this slice.

## Executed evidence

The 136-triangle source prototype passes exact cyclic geometry/winding, UVs and
bounded exported normal direction checks. Independent ufbx point sets differ by
at most **0.020 micrometres**; the actual Unity Editor mesh points match with zero
observed error after frame qualification. Its one exact source placement passes
source matrix/geometry/normal/winding checks with arithmetic error below 2e-15 m.
`build/forest-glass-coverage-v1.json` independently checks all new tree/glass groups
against the source plan without overlapping any previous converted binding.

`build/forest-glass-scene-v1/` preserves all earlier inputs and adds only this
placement: **18,865 rendered primitives / 6,092,165 triangles / 165 materials /
55 glTF images / 143 mesh definitions**. The four triplanar profiles and ten
extension images remain unchanged; the new material adds no texture. Actual
Standard primitives total 18,337. Source draw count differs from primitive count
because ground/ruins and foliage have multiple primitives.

Four native Linux Bevy 0.20 cases pass on **RX 7900 XTX / RADV NAVI31 Vulkan**:
prototype, source placement, full overview and focused lantern view. Active
Khronos validation, completed pipelines, exact input counts/UV channels, no renderer
errors/panics/tangent failures and native bounds within **7.50 micrometres** pass.
The prototype and focused lantern images were inspected. The source's unused UV1
stream stays preserved; material sampling uses UV0. The combined scene therefore
carries 42 UV1 streams while seventeen leaf primitives select UV1 for colour.
No translucent-pixel/Unity lighting equivalence claim is made.

Khronos: three files, zero errors; prototype/placement have zero warnings and the
combined scene retains seven runtime tangent warnings, with native generation
passing. Fresh fmt, denied-warning workspace/all-target Clippy and **181 Python
tests** (174 asset + four GLB + three Blender reference) pass. Tests reject wrong
source families/keywords/depth, textures, HDR/invalid opacity, duplicate/nonfinite
properties and altered exported blend/culling/tint/opacity/surface/dependencies.
The same client already passed 35 Rust tests, three legacy triplanar modes and
encrypted town/farmhouse two-client routes in the preceding slice; these remain
retained evidence, with no repeated multiplayer or Windows claim.

Combined scene SHA-256:
`1d76c5f7f414b1b9681b00b9c17fc551f751174b57cd900280dc56c7ac348e56`.
Client stays `3526f43349e7bb02cb93c3469be23040012a9ffd1c077674869781e9475a99b0`.
Evidence: `build/forest-glass-source-v1/`, `build/forest-glass-frame-v1/`,
`build/forest-glass-coverage-v1.json`, `build/forest-glass-scene-v1/`,
`build/forest-glass-native-v1-20261021/`, `build/forest-glass-gltf-v1.json` and
matching test/check logs. All generated/licensed artifacts remain ignored.

Next: preserve actual submesh slot assignments for the 25 mixed crystal/geode
draws before integrating their material adaptation. Seven water draws, one sky
and one emissive willow also remain. Lighting, rebase/resources/mips, object
collision and runtime adoption stay separate; the normal town/opt-in ground paths
remain available. Inspect the new Elven folder after forest completion. Work is local.
