# Highest-detail forest foliage and first planted subset

P1-10/P2-08 continuation, 2026-10-20. Prerequisites are hash-bound source/shaders,
checked model fileIDs, independent source placement matrices and the
[mesh-local coordinate contract](UNITY_GEOMETRY.md). This qualifies two actual
highest-detail prototypes and a static subset, not the full forest or its object
collision. The separately qualified [multiplayer ground](FOREST_TERRAIN_COLLISION.md)
and default town remain unchanged.

## Full shader profile

`tools/unity_full_foliage.py` binds the reviewed full vegetation shader GUID
`6697e1f185bc8b840a2f02567abf15b5`, SHA-256
`87a2b6f6206262674de9b2ff05d3743ee2a62e0d1bc7eff5b06575ab4c56abe4`.
Source shader bytes are read, never executed. This is a separate explicit
`--full-static` profile; the [basic foliage profile](FOREST_FOLIAGE.md) remains.

Where leaf base/small-noise/large-noise RGB colours all agree and trunk base/noise
RGB colours agree, the shader's spatial colour blends collapse to those constant
colours. This avoids inventing a world-space noise field. Unequal colours,
active frosting/emission, unsupported variants/passes, nonfinite values and
nonidentity texture transforms reject. The RGB colour-floor/flat-colour blend
becomes a derived sRGB PNG while source alpha remains byte-exact. Binary constant
per-face vertex blue chooses leaf/trunk materials; metallic, roughness, source
cutoff and double-sided MASK settings follow the selected branch.

Normal references are now applied through ordinary glTF normal textures. The
explicit desktop-RG bake multiplies sampled R by alpha, decodes/scales XY,
reconstructs Z, normalizes and encodes RGB8. Texture type 1 is consumed as linear
normal data, recording its serialized sRGB flag separately. Type 0 follows its
serialized sRGB setting under the chosen linear workflow. The actual tree normal
image is type 0/sRGB; the fern image is type 1 with a stored sRGB flag. Treating
all files named "normal" identically would erase that distinction.
[Unity's linear-texture guidance](https://docs.unity3d.com/2020.2/Documentation/Manual/LinearRendering-LinearTextures.html)
and [normal format helpers](https://github.com/Unity-Technologies/Graphics/blob/master/Packages/com.unity.render-pipelines.core/ShaderLibrary/Packing.hlsl)
explain why an explicit profile is needed; they do not prove Unity 2020 platform
import/helper/compression equivalence for this bake.

The independent verifier compares every base texel's reconstructed normal with
its exported unit vector, using a 0.007 vector-error limit from RGB8 quantization.
Actual maxima are 0.005831 for the tree and 0.004129 for the fern. Filtering/mips
of baked XYZ can differ from reconstructing a normal after source RG sampling;
this pixel bound does not establish filtered runtime shading equivalence.
Source tangent-basis, Unity project colour-space, platform compression and BRDF
parity remain unproven. Optional AO consumes the source sampled R channel as a
linear glTF occlusion texture. Both actual assets use neutral missing AO maps;
nonzero AO has synthetic binding/pixel/fault tests, not an actual-source native
AO qualification.

This is **authored rest geometry**. Source wind, jitter and animated gust colour
highlights are recorded but unapplied. Source LOD crossfade remains absent.
These are explicit presentation limitations, not a claim that the source shader
has no animation or that the forest material family is fully implemented.

## Prototype and coordinate handling

`tools/convert_unity_foliage.py --full-static --canonical-mesh` selects an exact
source renderer/mesh/material and exports a canonical prototype. It records the
source draw matrix separately for later H M H placement. Capsule-collider records
may remain unqualified source metadata in this explicit visual-only mode; this
never authors collision. Other unsupported component families still reject.
Nonidentity source poses are not silently baked into a reusable mesh twice.
A used branch without a bound diffuse image rejects; unused shader branches may
retain default references without forcing irrelevant texture conversion.

Independent ufbx mesh-local checks pass all source points: 3,796 for the tree
(maximum position error 0.969 micrometres) and 242 for the fern (0.135 micrometres).
The independent branch oracle checks all geometry, winding, UVs, mesh normals,
raw source alpha/derived colour pixels and normal texture bindings/reconstruction.

The tree exposed an old quantized-key boundary issue: all positions were exact,
but V inversion/export rounding differed by at most 2.98e-8. The oracle now
matches exact f32 cyclic triangle positions, checks UV differences within
max(1e-7, two f32 ULP), and retains the 0.00018 mesh-normal export-rounding bound.
Changed positions, reversed winding and UV corruption reject. This improves
source correspondence instead of broadening a quantized equality threshold.

## Actual assets and placements

| Asset | Source LOD | Triangles | Materials | Images |
|---|---:|---:|---:|---:|
| `SM_Env_Tree_Small_01` | 0 | 2,824 | 2 | 3 |
| `SM_Env_Fern_01` | 0 | 286 | 1 | 2 |

All **three tree** and **612 fern** draw records having these exact source
mesh-GUID/fileID/material bindings are assembled with their actual source IDs
and world matrices. Other material variants remain separate work. Independent
assembly checks pass all 615 instances, preserving shared prototype assets,
triangle winding/normals and positions (maximum f64 error 1.422e-14 metres).
The source placement plan's bytes/hash remain unchanged.

Adding these to the previously qualified ground/five ruins yields **719,332
triangles, 624 mesh instances, four shared source meshes, five materials and
eight images**. This is 620 authored object placements plus ground; mesh instances
also count separate material primitives. Elevated authored plant placements and
overhangs remain intact while the surrounding/supporting source geometry awaits
conversion. The subset extends to Z -81.045 metres; it does not change the
finite ±75-metre walking-ground boundary or supply plant/tree/ruins collision.

Three actual native Bevy inspector cases pass: tree, fern and combined subset,
on **AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan**, Linux, with active Khronos
validation, compiled pipelines, no VUID/errors/panics and visually inspected
captures. Actual transformed bounds have maximum error 1.701 micrometres in the
assembled scene. These visual-only inspector cases do not demonstrate new
multiplayer object loading or streaming/performance capacity.

Khronos validation reports zero errors/warnings for both prototypes and the
combined subset. The tree/assembly have three informational messages: two unused
UV1 attributes and an unused trunk tangent. The fern is clean. The legacy basic
card was freshly regenerated, passes the updated independent oracle and is
**byte-identical** to its previously native-qualified GLB.

A field-level audit of the full-shader materials actually used by the source
plan accepts **8 of 12**. Four remain explicit: Undergrowth/Leaves variant/pass
settings, Willow emission/spatial colour and Tree 02 texture transforms. This
is property-profile eligibility, not qualification of all eight materials'
images/meshes or complete scene adoption. Other shader families remain separate.

Fresh fmt, denied-warning workspace/all-target Clippy and **137 Python tests**
pass: 130 asset, four GLB, one prefab reference and two variant reference. Six new
cases cover profile/effect rejection, all-byte colour/alpha, normal strength/
alpha/sRGB, geometry/UV faults and synthetic normal/AO source binding corruption.
Rust application code/binaries/lock and the ground/default map remain unchanged;
the preceding 162-test Rust checkpoint was not rerun for these offline tools.
No native Windows, Unity Editor parity or whole-task completion follows.

## Commands and evidence

```sh
build/blender-tools/bin/python tools/convert_unity_foliage.py \
  build/synty-enchanted-source build/synty-enchanted-shader-source \
  --prefab Assets/PolygonNatureBiomes/PNB_Enchanted_Forest/Prefabs/SM_Env_Tree_Small_01.prefab \
  --output build/new-full-tree --full-static --canonical-mesh
build/asset-tools/bin/python tools/verify_unity_foliage.py \
  build/synty-enchanted-source build/synty-enchanted-shader-source build/new-full-tree
```

Use fresh output paths. Private evidence under ignored `build/`:

- `synty-forest-full-{tree,fern}-v1/`: prototypes, images and independent reports.
- `synty-forest-full-{tree,fern}-canonical-v1.json`: independent FBX positions.
- `synty-forest-full-{tree,fern}-instances-v1.json`: actual selected placements
  bound to the unchanged source plan hash.
- `synty-forest-full-{tree,fern}-placements-reference-v1.json`: independent
  source-pose/geometry assembly checks.
- `synty-forest-ground-ruins-trees-ferns-v1.glb` and its `-reference.json` sidecar:
  combined geometry, input hashes and the independently derived union envelope.
- `synty-forest-{full-tree-v1,full-fern-v1,ground-ruins-trees-ferns-v1}-*validation.json`:
  Khronos reports; exact filenames use `-gltf-validation` for prototypes and
  `-validation` for the assembly.
- `forest-full-foliage-native-v1-20261020/{tree,fern,assembly}/`: native evidence.
- `synty-full-foliage-profile-audit.json`: eight eligible/four unresolved profiles.
- `forest-full-foliage-{assets,clippy}-final.log`: current checks.

Final asset SHA-256s:

- Tree: `4dbe141a81082bcb26c0984a369552bb237a8e82d924056e05fab760d59a079a`.
- Fern: `c4bdee2ce48d0f36cc05ba9c943f508f478a7dafc531b6d9dd074af03f68e5f5`.
- Combined: `d00257cc3bf1f51e20642890d0e56ad25fe25ea1a3c74f9fd4c519475b0b9293`.

Next: batch the remaining qualified mesh/material bindings, including explicit
scene material overrides, then resolve the other full-vegetation settings and
shader families. Preserve the source plan and current ground/map regressions.
Complete authored scene/collision and resource/native multiplayer acceptance
still precede replacing the development map.


The [bulk foliage continuation](FOREST_FOLIAGE_BATCH.md) supersedes the earlier
small subset: 15,156 exact source draws, shared images and two native final-scene
views now qualify. Remaining shader/material/collision/adoption work stays open.
