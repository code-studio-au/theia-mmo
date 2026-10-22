# Bound projection materials in the normal client

P1-10 / P2-08 prerequisite, 2026-10-21. A self-contained projection profile now
connects immutable package dependencies to standard Bevy extended materials and
actual floating-origin updates. The current original-fixture qualification does
not adopt the forest as the development map.

## Versioned data contract

Package manifest **schema 3** retains the existing shared-image fields and budgets
and uses identity domain `theia-world-package-v3\0`. It requires at least two image
declarations and at least one bound projection profile during visual verification.
Schemas 1/2 retain their byte layouts and golden identities. A saved previously
qualified schema-2 server rejects the new package version before startup; unknown
render requirements cannot silently fall back on an older reader.

A marked material carries `extras.theia_triplanar` (source-local ID 1–16) and
`extras.theia_triplanar_profile`:

```json
{
  "schema": 1,
  "profile": "synty-triplanar-world-v1",
  "id": 1,
  "tiling": 0.7,
  "falloff": 4.1,
  "source_anchor": [0, 0, 0],
  "textures": {
    "top": "../shared/<sha256>-s-10497-10497-9729-9729.png",
    "side": "../shared/<sha256>-s-10497-10497-9729-9729.png",
    "top_normal": "../shared/<sha256>-l-10497-10497-9729-9729.png",
    "side_normal": "../shared/<sha256>-l-10497-10497-9729-9729.png"
  }
}
```

The source anchor is the checked global f64 position corresponding to authored
world zero. It is separate from an individual chunk's owner or local origin.
Tiling/falloff are finite positive f32 values bounded by 1000/100. Profiles deny
unknown fields, shaders, arbitrary paths/URLs, mismatched marks/roles/samplers and
invalid anchors. PNGs must be manifest-bound, at most 4096 per dimension, 8-bit
RGB/RGBA and noninterlaced; decoded images must have the qualified RGBA8 colour
space and one mip. Existing encoded/decoded image budgets remain unchanged.

GLB bytes bind parameters and image references; the manifest binds their PNG
bytes, colour roles and sampling. No separate inspector sidecar is needed.
Content-based material keys ignore source-local numeric IDs, allowing compatible
profiles in different packages to reuse a material. Different parameters, anchors
or image paths retain separate keys. Both admission and the live active/staging
registry allow at most 16 distinct profiles. Unreferenced registry entries drop
their handles through standard Bevy asset ownership.

## Presentation and readiness

Only imported static scenery receives these materials. The normal client loads
the four bound texture slots through Bevy's AssetServer, generates required
MikkTSpace tangents and uses the existing triplanar extension of StandardMaterial.
Root readiness waits for image decoding, format checks and material application.
Malformed bare inspector marks reject instead of displaying a placeholder.
Candidate failures are scoped to their root; the retained scene remains available.
Camera clearance accepts both standard and extended opaque/masked materials.

The registry subtracts global render/source origins in f64 before uploading a
bounded periodic phase. Material uniforms update when that render frame changes;
unchanged frames do not repeatedly mutate material assets. Geometry, actor
coordinates and texture phase use the same frame. This does not change movement
authority, collisions, navigation or server tick scheduling.

For qualification only, `THEIA_RUNTIME_TRIPLANAR_DIAGNOSTIC=1`/`2` enables colour/
pre-TBN-normal views during a network smoke. Other normal play modes reject it.
The inspector remains a separate sidecar workflow. The older `--chunk-smoke`
presentation qualifier explicitly rejects schema-3 material packages; use the
normal Renet path for this contract.

## Executed evidence

Three original providers at global X/Z ±128 billion metres run a real encrypted
authority and two native clients in lit/colour/normal modes. All six views pass
actual root/readiness, body clearance, compiled-pipeline, actor and independent
render/replication-rebase checks. Three distinct source IDs share **one actual
material handle** and **three actual projection image handles**; standard floor
images remain separate. Reported slots come from the actual material assets,
including path, colour format, sampler, dimensions, mip count and uniform phase.
Every client records phase updates after rebasing.

Independent captured-camera/package-geometry/PNG reconstruction passes **828
source pixels**: colour maximum error one RGB8 level, normal error zero. No
proxy render is used. A temporary oracle adapter renumbers equivalent source IDs
and adds the exact checked root translations; original package geometry is retained.

Five native failure cases reject malformed fields, manifest downgrade, missing
PNG, rehashed truncated PNG data and an unbound inspector mark. The last two reach
the actual material-loading/readiness path and exit without a fallback capture.
The original failing wall-clock farmhouse gait diagnostic is retained. Its smoke
controller now walks/reverses to observed authoritative displacement while keeping
the existing >3 m movement and animation checks; fresh town/farmhouse cases pass.

Device evidence is Linux, AMD RX 7900 XTX / RADV NAVI31, Vulkan with Khronos
validation active. No Windows run/cross-build is claimed. The complete default
network suite, cross-language schema-3 identity golden, capability/parameter/
PNG-binding/profile-budget tests, 44 client tests and 221 asset Python tests pass.
Three original package GLBs have zero Khronos errors or warnings. Denied-warning
workspace/all-target Clippy passes. Three existing paired handover cases also
pass: delayed publication, deliberate asset failure retaining the active scene,
and differing large negative query frames.

Ignored local evidence:

- `build/runtime-materials-native-v2-20261021`, six normal connected views,
  actual asset/phase checks and `pixels-v2-*` independent source comparisons.
- `build/runtime-material-failures-v1/verification.json`, five failure cases.
- `build/runtime-materials-map-regressions-v2-20261021`, fresh town/farmhouse.
- `build/bound-material-old-reader-v1/verification.json`, explicit old-reader rejection.
- `build/bound-material-package-native-v1.log`, real manifest/collision/PNG loader.
- `build/bound-material-all-network-tests-v1.log`, full default network suite.
- `build/runtime-materials-client-tests-v4.log`, 44 client tests.
- `build/runtime-materials-clippy-v6.log`, denied-warning Clippy.
- `build/runtime-materials-python-tests-v2.log`, 221 asset tests.
- `build/runtime-materials-paired-regressions-v1-20261021`, three retained handover cases.
- `build/bound-material-gltf-validation-v1.json`, three original GLBs.

Remaining: source-preserving embedding into the ten complete forest candidates,
actual forest runtime resources/rebase checks, render flags/lighting/mips, explicit
water behaviour, movement/camera and development-map adoption. Original-fixture
results do not establish full-forest streaming/resource capacity or source-engine parity.
