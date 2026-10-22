# Forest source geometry and placement contract

P1-10/P2-08 continuation, 2026-10-20. Prerequisites are the hash-bound forest
source, checked FBX mesh IDs, explicit material prototypes and independently
verified source instance matrices. This delivers a small rendered assembly;
complete forest, terrain, collision and default-map adoption remain open.

## Mesh-local coordinate evidence

The offline ufbx 0.23.1 probe independently reads FBX positions in metres and
canonical right-handed Y-up axes. It applies `geometry_to_node`, rather than
including the imported object's world transform in a mesh prototype. The pinned
source commit is `26a482ae66871d7de36eb722aa060bce95bce274`; the builder
hash-checks both downloaded files and keeps dependencies/output under ignored
`build/`. The repository contains our small adapter, not vendored engine code.
Our C adapter compiles with `-Wall -Wextra -Werror`; vendor code compiles separately.

Serialized Unity LOD bounds provide an independent coordinate calibration for
54 actual prefab groups/models, including the group's source child transforms.
At 0.1 mm tolerance, **52 groups match an X reflection** of canonical mesh-local
geometry. Forty-two distinguish that convention from unchanged coordinates;
45 distinguish it from a Z reflection. Two retain explicit centre mismatches:

- Portal tree 01: derived Y centre differs by 6 metres.
- Tree trunk 02: derived Z centre differs by about 0.754 metres, matching its
  authored child Z offset.

Their sizes and X centres agree. The cause of those two source-bound differences
is unproven; no compensating geometry offsets are applied. This is empirical
source-specific calibration, not Unity Editor importer equivalence. Original
FBX units are centimetres; canonical geometry is in metres.

## Ordinary glTF instances

`tools/unity_geometry.py` accepts an embedded static single-mesh identity GLB and
explicit source `id`/4×4 `matrix` records. With H = diag(-1,1,1,1), canonical
prototype poses become H M H. Source Unity positions become H p. A singular-value
factorization expresses affine shear as two ordinary glTF TRS nodes; no custom
renderer or Bevy transform decomposition is required. Singular/ill-conditioned,
nonfinite, duplicate-ID and out-of-frame input rejects.

Positive-parity instances share one mesh. Negative parity uses one shared mirrored
mesh variant with reflected positions/normals/tangents and reversed triangle
indices, retaining positive node determinants for ordinary renderer culling.
Material/image/source BIN bytes remain shared and unchanged. The bounded profile
allows at most 32,768 instances/65,536 nodes and a ±4,096-metre local envelope.
The assembler's reported envelope transforms local AABB corners; the independent
reference and native inspector separately measure actual vertex bounds.

## Executed acceptance

The actual `SM_Prop_Ruins_03` prototype passes the existing independent Blender
triangle/winding/UV/material/pixel oracle. Independent ufbx point comparison
checks 1,154 source points against 4,804 exported split vertices: maximum error
0.000000379 metres. Five scene placements use their actual IDs, matrices and
matching source mesh/material bindings. The independent assembly oracle checks
all five, 11,540 triangles, normals, winding and shared immutable assets.
An asymmetric original tetrahedron fixture separately checks translation,
nonuniform scale, shear and negative parity. Two fault tests reject omitted
instances/changed source poses and missing point-set coverage in either direction.

Both final outputs pass Khronos validation with zero errors/warnings. The fixture
has three buffer-view-target hints inherited from its original prototype; the
forest output has no informational messages/hints. Both render through ordinary
Bevy on **AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan**, Linux, with active
`VK_LAYER_KHRONOS_validation`, all pipelines compiled, and no VUID/errors/panics.
Native actual `GlobalTransform` vertex-bound errors are 0.000000954 metres for
the fixture and 0.000002674 metres for the five ruins. Captures were visually
inspected; this is visual inspection without gameplay collision.

Evidence under ignored `build/`:

- `ufbx-0.23.1/build.json`: pinned dependency/probe/compiler hashes.
- `synty-forest-fbx-basis-qualified.json`: all 54 groups and two retained mismatches.
- `synty-forest-ruins03-canonical-qualified.json`: independent mesh-local positions.
- `synty-forest-ruins03-placements-reference.json`: independent five-instance check.
- `unity-geometry-asymmetric-fixture/reference-v2.json`: independent affine fixture.
- `unity-geometry-{fixture,ruins}-v2-validation.json`: Khronos reports.
- `forest-geometry-native-v2-20261020/{fixture,ruins}/`: native reports/logs/captures.

Final asset SHA-256s:

- Ruins: `9427c6b3d8466c0eb4dfdd844e5cadf82d574ab222b4160d6766461f528ca22b`.
- Fixture: `a2b58e58bdfb6122ddc4d9ecf349687dd0904789bf07521c33d245481a66b94a`.

Fresh fmt, denied-warning workspace/all-target Clippy and **121 Python tests**
(114 asset, four GLB, one prefab reference, two variant reference) pass.
Rust application code/binary and the default town remain unchanged. The prior
Rust tests are historical evidence, not rerun for these offline changes.
No Windows or full-scene fidelity/collision/capacity claim follows.

## Reproduction and next gate

```sh
build/asset-tools/bin/python tools/build_ufbx_probe.py
build/asset-tools/bin/python tools/verify_fbx_basis.py build/synty-enchanted-source \
  --dependencies build/synty-enchanted-dependencies.json \
  --probe build/ufbx-0.23.1/ufbx_geometry --report build/new-basis-report.json
build/asset-tools/bin/python tools/unity_geometry.py \
  build/synty-forest-ruins03-prototype/preview.glb \
  build/synty-forest-ruins03-instances.json \
  --output build/new-ruins-assembly.glb --report build/new-ruins-assembly.json
build/asset-tools/bin/python tools/verify_unity_geometry.py \
  --report build/new-ruins-reference.json assembly \
  build/synty-forest-ruins03-prototype/preview.glb \
  build/synty-forest-ruins03-instances.json build/new-ruins-assembly.glb
```

Use fresh output paths. Private staged sources are required. Full forest ground
uses binary Unity TerrainData, not a ground FBX: decode its heightmap, layer
weights and holes with checked source binding before terrain/collision assembly.
Then qualify remaining full vegetation materials and actual scene contributor
coverage, collision/spawns and native multiplayer before map adoption. Retain
particle errors and unimplemented component families explicitly; the static draw
plan does not establish a complete forest.
