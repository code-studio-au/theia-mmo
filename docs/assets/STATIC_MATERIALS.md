# Standard material extensions in shared-image packages

P1-10 / P2-08 prerequisite, 2026-10-21. Immutable shared-image packages now admit
the same two standard glTF material extensions already qualified in the inspector:
`KHR_materials_unlit` and `KHR_materials_emissive_strength`. Normal Bevy loaders
implement both; no custom renderer or dependency-version change is introduced.

The named glTF declarations and complete GLB bytes remain bound by the existing
manifest identity. Changed content requires a new immutable revision. Package
manifest schemas and image byte/count/decoded-size budgets remain unchanged.
This extends the narrowly admitted material payloads; it does not admit arbitrary
extensions or custom triplanar metadata into ordinary chunk rendering.

Both Rust admission and Python packaging require unique supported declarations,
required declarations present in `extensionsUsed`, and declaration-bound payloads
only at material level. Unlit must be empty. Emissive strength must contain only
one finite numeric `emissiveStrength` in [0,16]. Booleans, extra indices/fields,
unsupported names, texture/node extensions and malformed payloads reject. The
explicit Python `allow_material_extensions=False` legacy restriction remains
available. Texture roles, canonical image paths and actual PNG bindings retain
their existing checks.

Static graph pruning now preserves each retained material's exact extension
payload and relevant used/required declarations. Previously the payload survived
while its declarations were omitted, invalidating sky/emissive shards. Removed
materials do not leave unused extension requirements behind. A new pruning test
checks this without changing source documents or geometry.

## Evidence

An original three-chunk schema-4 heightfield/box fixture passes the normal secure
Renet authority plus two native clients with actual contributor readiness,
source-surface capsule clearance, actor pixels, physics probes, independent
render/replication rebasing and compiled pipelines. Three unlit floors and a
standard emissive wall share **one actual image handle**. Reported material values
match the bound source: unlit floors and emissive RGB approximately
`[0.13,1.56,1.56]`, from strength 2.6. Both client captures contain over 24,000
visible cyan emissive-wall pixels. This is rendered/component evidence, not a
photometric match to Unity.

Native execution: Linux, AMD RX 7900 XTX / RADV NAVI31, Vulkan with Khronos
validation active; no validation errors, panics or pending/failed pipelines.
Windows execution/cross-build is not claimed. Existing town and farmhouse
entry/wall/exit/re-entry regressions pass on the rebuilt binaries.

Ten rehashed semantic faults reject through real package dependency admission.
The complete default network suite, 41 client Rust tests, fresh format and
denied-warning workspace/all-target Clippy pass. There are 218 passing asset
Python tests plus nine retained Blender/reference cases. Six original embedded/
external fixture GLBs have zero Khronos errors/warnings; the visible revision
also passes native loading and material-value checks.

Local evidence remains ignored:

- `build/static-materials-native-v2-20261021/materials`, normal material/image/
  pixel/protocol evidence; v1 retains the earlier offscreen-wall component check.
- `build/static-materials-map-regressions-v1-20261021`, town/farmhouse.
- `build/static-materials-all-network-tests-v1.log`, complete network suite.
- `build/static-materials-client-tests-v1.log`, 41 client tests.
- `build/static-materials-clippy-v1.log`, denied-warning Clippy.
- `build/static-materials-python-tests-v1.log`, 218 asset tests.
- `build/static-materials-gltf-validation-v1.json`, six original files.

Full forest visuals still need their custom material/render-flag/lighting and
resource/rebase gates, followed by water policy and actual movement/camera
qualification. This contract does not change the current development map.
