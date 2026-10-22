# Forest triplanar material prerequisite

The [2026-10-21 mesh-frame continuation](FOREST_MESH_FRAMES.md) now corrects the
prototype/all-85 placements using actual Editor frames and qualifies source/world,
six final native and 512 independent pixel samples. The original stages/evidence
below remain preserved. Multi-profile/rebase/resource integration and full map
adoption remain open; the 85 draws are not yet in the main combined scene.

P1-10/P2-08/P2-10 continuation, 2026-10-20. The remaining cliff/rock family needs
world-projected textures. Ordinary mesh UVs would change source mapping across
instances; a per-instance bake would duplicate resources and lose continuous
world projection. This slice uses Bevy's existing `ExtendedMaterial<StandardMaterial,
Triplanar>` API and normal PBR/asset/pipeline systems. No custom renderer is added.
This qualifies a material prototype; it does not add these meshes to the adopted
world or close remaining material/collision/importer gates.

## Source profile

`unity_triplanar.py` pins shader GUID `66f8401a4c881134f8803eceae302347` and SHA-256
`d478ba3c6b3476570529dee5f65dc725b07a5bc463ea05bec10757e1209fe118`.
It preserves signed X/Y/Z world projections, side texture on downward Y, positive-Y
top texture, `_Tiling`, `_FallOff` and the literal `+0.00001` weight denominator.
It does not replace that denominator with an ordinary normalized blend. Texture
ST and stale Standard surface fields are unused by the pinned source shader and
are recorded without applying them. The explicit PBR adaptation uses metallic 0,
roughness 1 and opaque backface culling. Unknown variants/pass changes, missing
texture references, invalid/underflowed positive scalars and unqualified importers
reject. Source falloff zero remains outside the qualified profile.

The reference PNG workflow flips V relative to source Unity texture coordinates.
Colour textures sample sRGB; typed normal textures sample linear. Normal RGBA is
blended across projections before chosen desktop RG×alpha/G reconstruction. Bevy's
standard MikkTSpace basis maps that vector into world space. This is an explicit
raw-image/desktop reconstruction policy, not Unity platform compression, tangent,
BRDF, lightmap or authored-lighting equivalence.

## Opt-in native path

`preview_unity_static.py --triplanar-shader-source ...` selects an exact canonical
source draw, preserves geometry/UV/normal/tangent streams and emits a marked GLB,
four original PNG dependencies and hash-bound `triplanar.json`. It requires the
explicit rendering-adapter path; the legacy Standard and simple-prefab paths
retain their source-property contracts.

The inspector accepts `--triplanar <metadata>` for lit output,
`--triplanar-diagnostic <metadata>` for unlit colour, or
`--triplanar-normals <metadata>` for decoded normals. These diagnostic modes disable
tonemapping, dithering and reference geometry. Only one marked material/profile
is supported per GLB; missing metadata/marks and mixed material sets reject.
Metadata is capped at 64 KiB, geometry at 256 MiB, each PNG at 64 MiB and 4096².
Role filenames, containment, SHA-256 and positive f32 scalar conversion are checked.
Textures load through ordinary Bevy `ImageLoaderSettings` with repeat/bilinear
sampling, sRGB/linear roles and nearest-mip selection. The native PNG assets have
**one mip level**; source mip-chain/filter/compression equivalence is unproven.
Legacy `-1` importer sampler settings stay unresolved rather than guessed.

The native path is a **bounded ±4096-metre origin-zero inspection frame**. It
reflects render X back into source Unity coordinates for projection. Ordinary
chunk loading, multiple material profiles, centralized f64 origin/rebase texture
phase, GPU resource budgets and runtime publication remain integration work.
The shader is embedded with the client; imported source assets cannot supply
executable shader paths. Capture waits for actual material application, all four
image dependencies and stable compiled pipelines. Snapshot metadata includes
actual image formats/samplers/mip counts, uniforms and camera matrices.

## Independent checks

`verify_unity_triplanar.py` reads raw source draw/material/shader bindings, verifies
original PNG bytes and sampler/import metadata, and checks exact cyclic geometry,
bounded UV/normal export rounding and valid tangent vectors. Independent ufbx checks
canonical positions separately. `verify_triplanar_pixels.py` imports neither the
material authoring math nor shader. It ray-intersects actual captured camera rays
with exported triangles, interpolates source normal directions and independently
calculates signed projection, repeat bilinear sampling, linear-light colour and
pre-TBN normal decoding. Interior source pixels exclude silhouettes/UI/triangle
edges. Its four-sRGB8-level bound covers float interpolation and output encoding;
this is not a Unity Editor pixel oracle or a complete lit-normal/BRDF comparison.

CPU tests cover six signed axes, bottom-versus-top selection, epsilon weights,
negative-coordinate repeat, PNG V direction, linear-light filtering, blending
before normal unpack and source-profile rejection. Two new Rust tests cover
metadata geometry/profile/scalar binding and role path/hash/PNG-dimension rejection,
including refreshed hashes for oversized images. Private fault injections also
reject changed tiling, swapped normals, altered vertices, missing GLB marks and
corrupted GPU pixels after hashes are refreshed.

## Import/root-pivot evidence remains open

The [actual Unity reference](FOREST_UNITY_REFERENCE.md) now covers all 112
scene FBXs / 214 meshes. Independent geometry expressed in the recorded Unity
frames matches every mesh, with byte-identical metadata. This establishes the
missing frame evidence; converter correction and replacement geometry/native
qualification remain pending. The user requested a safe stop before those changes.

The actual `SM_Env_Moss_Lumps_03` FBX mesh has X coordinates approximately
[-43.2695,-40.4277] metres, despite its small 2.84-metre width. ufbx reports root
translation approximately [53.8943,-0.0079,-8.0006] and uniform scale 1.28784.
The serialized prefab root has position zero and scale 1.28784. This mesh has no
serialized LOD-bound calibration in the existing 54-group proof. Raw mesh-local
positions and selected source matrices can be verified independently, but those
checks do not establish Unity's imported mesh/root-pivot treatment for this asset.

An earlier 85-placement diagnostic assembly under `build/forest-triplanar-moss-group-v1/`
spans Y approximately [-105.84,96.13] under the current literal mesh-local policy.
It is preserved as an **unqualified experiment**, not promoted world geometry.
No centering, translation or duplicate scale compensation was invented. The
subsequent actual Editor reference confirms root translation is folded into this
mesh while scale remains on its transform. Apply and independently qualify that
recorded frame before claiming its full-map placement.
The package contains only a `LightMotion.cs` script; no packaged importer script
explains the discrepancy. Unity's documented single-root reuse policy is relevant
but does not resolve these asset-specific vertex offsets. See the
[Unity 2020.3 root policy](https://docs.unity3d.com/2020.3/Documentation/ScriptReference/ModelImporter-preserveHierarchy.html).

The prior two LOD-centre mismatches and all earlier shader/terrain limitations
remain explicit. The main combined subset stays at **18,079 draws**, with **580
unresolved draws**. Material math/prototype success does not qualify all triplanar
placements, complete the forest, or advance the queued Elven Realms import.

## Reproduce

```sh
build/blender-tools/bin/python tools/preview_unity_static.py \
  build/synty-enchanted-source \
  --prefab Assets/PolygonNatureBiomes/PNB_Enchanted_Forest/Prefabs/SM_Env_Moss_Lumps_03.prefab \
  --standard-material --draw-plan build/synty-forest-static-variants-plan.json \
  --instance 7848724 --renderer 6936252194287251932 \
  --triplanar-shader-source build/synty-enchanted-shader-source \
  --output build/new-triplanar-moss
build/asset-tools/bin/python tools/verify_unity_triplanar.py \
  build/synty-enchanted-source build/synty-enchanted-shader-source build/new-triplanar-moss
target/debug/theia-client --inspect-asset build/new-triplanar-moss/preview.glb \
  --triplanar build/new-triplanar-moss/triplanar.json
```

Use fresh output directories. Licensed assets, private source reports, generated
stages and native captures remain in ignored `build/`. All changes stay local.

## Executed prototype evidence

Actual source stages are `build/forest-triplanar-moss-v1/` (canonical mesh) and
`build/forest-triplanar-moss-placed-v1/` (one actual source instance/matrix).
Both contain 196 triangles. Raw source/geometry/texture checks pass; canonical
ufbx positions differ by at most **2.76 micrometres**, and source mesh-normal
export difference is 7.87e-5 within the existing 0.00018 direction bound. The
single-instance geometry/winding/normal reference passes with maximum position
arithmetic difference 4.27e-14 metres. This establishes the stated mesh-local
pipeline, not the unresolved Unity importer/root-pivot equivalence.

Six final native cases in `build/forest-triplanar-native-v3-20261020/` cover lit,
unlit-colour and decoded-normal output for both stages on **AMD Radeon RX 7900 XTX /
RADV NAVI31 Vulkan**, Linux, in an owned private Wayland compositor. Active Khronos
validation is confirmed by loader trace. Pipelines finish with zero pending/failed
entries and at least three clear frames. No VUID, validation/renderer error, panic
or tangent-generation failure occurs. Exactly one primitive receives the extended
material, and all four actual images have the expected sRGB/sRGB/linear/linear
formats, repeat/bilinear samplers and one mip. Native vertex-bound errors are zero
for the canonical stage and at most **0.00318 mm** for the transformed stage.

Independent pixel checks cover **152 colour + 152 normal samples** on the canonical
mesh and **142 colour + 142 normal samples** on the transformed mesh: **588 pixels**
total. Every maximum difference is **one sRGB8 level**, below the declared four-level
bound. This checks world mapping under an actual source matrix as well as colour
and pre-TBN normal reconstruction. Both lit captures were visually inspected;
these are inspector lighting and reference geometry, not a Unity lighting match.
Two additional native failure cases reject a missing material option and an
unmarked GLB with refreshed metadata binding; neither captures fallback output
or panics.

Khronos checks both prototype GLBs with **zero errors/warnings** and one unused-
tangent informational message each. That validator sees the standard fallback
GLB graph; the custom material is qualified separately by the native/pixel checks.
Five additional private fault injections reject rebound tiling, swapped normal
bytes, altered geometry, a missing mark and deliberately corrupted GPU pixels.

Fresh **157 Python tests** pass (150 asset, four GLB and three independent prefab/
variant-reference tests), plus **31 Rust client tests**, including two new metadata
validation tests. Fresh fmt, denied-warning workspace/all-target Clippy, source
payload/metadata rechecks and original archive hash recheck pass. Engine/physics/
transport pins and server code/binary remain unchanged. This slice rebuilds the
client to add the opt-in material inspector; earlier full Rust/ground-network
qualification is separate.

Final client SHA-256:
`448c900042af0f0ecca810110a56e63f3f0460952a2f63e3a57b483bdee4d4f6`.
Embedded WESL SHA-256:
`6c844c7c44f711ca0311f1d24fceb5887c65125d0acba6ada29c1175f865f27b`.
Canonical GLB SHA-256:
`ff5b67362b7979fa6e736fb371488baf875f913071cc3c2aa8a3b626adb6ee36`.
Single-instance GLB SHA-256:
`76a076f87eec4080e7400fb7dd63fc2c3847ece20ce025eb360ed2403947bf4b`.

Ignored evidence includes the stage's `conversion.json`, `triplanar.json`,
`geometry-reference.json`, `independent-verification.json`,
`build/forest-triplanar-moss-v1-canonical.json`,
`build/triplanar-{prototype,placed}-{colour,normals}-v3.json`,
`build/triplanar-gltf-validation.json`, `build/forest-triplanar-faults-v1.json`,
`build/triplanar-{rust-client-tests-final,python-tests-final,reference-tests-final,
glb-tests-final,clippy-final-v2,source-recheck}.log`, the source-script inventory
and the final native capture/report folders.

On the final client, the unchanged combined forest also passes two ordinary native
inspector views in `build/forest-triplanar-forest-regression-native-v2-20261020/`,
with its prior 18,302 primitive / 5,171,337 triangle / 31 image inventory and at most
0.0151 mm bound error. The extension is not activated for those ordinary views.
Town and farmhouse each pass a fresh ordinary encrypted two-client Renet route
through `build/forest-triplanar-map-regressions-v2-20261020/`, retaining existing
map movement/camera assertions and cleanup. These are regression checks, not
networked triplanar-forest or production performance qualification. Their wrapper
logs are `build/forest-triplanar-{forest-regression-native-v2,map-regressions-v2}.log`.
The one authored GLB fallback image and four runtime extension images are separate
inventories; resource sharing/budget qualification remains open.

The legacy ruins converter also regenerates byte-identical GLB output at SHA-256
`b9872ee03251e38980cfa2d929f148861043d40ee8488cf4a18b84ca24b8a617`
in `build/triplanar-legacy-ruins-regression/`. Matching Unity's generated mesh
normal/tangent recalculation remains part of importer-reference work; current
surface checks use the explicitly documented Blender/Bevy normal/tangent policy.
