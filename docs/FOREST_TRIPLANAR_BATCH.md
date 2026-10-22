# Forest triplanar batch

P1-10/P2-08/P2-10 continuation, 2026-10-21. The origin-zero visual
subset now contains **18,597 source draws / 62 unresolved**, adding seventeen
exact mesh/material bindings and 518 placements to the corrected 18,079-draw
predecessor. This qualifies offline conversion and inspector composition;
object collision, normal map adoption and chunk/rebase integration remain open.

Actual prerequisites were the [Editor mesh-frame corrections](FOREST_MESH_FRAMES.md),
[bounded catalog](FOREST_MATERIAL_CATALOG.md), pinned source shader and immutable
source plan. Acceptance requires fresh independent source material/Editor frame
checks for every new binding, complete exact source placements, preservation of
the existing scene and shared image variants, actual native mapping/render checks,
and a new dirt-profile pixel check. Unsupported bindings stay explicit.

## Implemented behavior

`batch_unity_triplanar.py` selects unresolved exact bindings of the pinned shader,
converts new prototypes, applies actual Editor frames and builds exact placements.
`verify_triplanar_batch.py` imports no batch converter and freshly checks all
seventeen source material/geometry/frame bindings, complete IDs/matrices and
assembly winding/normals. It independently classifies converted and remaining
source-plan groups. Maximum actual Editor point-set error is **2.94 micrometres**.

`prepare_triplanar_batch.py` checks bound prototype proofs and independently
rechecks source-plan placements before making placed catalog inputs. Each placed
proof binds the qualified prototype predecessor; it does not claim that a
prototype geometry hash also qualifies an unchecked placement file.

The catalog assembler now explicitly permits multiple distinct inputs to share
one profile ID only when complete material descriptors match. Default duplicate-ID
rejection remains. Bound inputs to 64, distinct profiles to sixteen, and preserve
the existing extension image/geometry budgets. Ordinary external images are copied
byte-for-byte into contained `shared/catalog-<input>-<variant>.png` paths;
identical bytes with different source colour/sampler variants keep distinct paths.
Generic GLB merging retains its default embedded-only behavior; external merging
requires the explicit option after catalog path checks.

The independent catalog checker verifies complete input ownership, world poses,
all vertex streams, winding and material/image signatures. It additionally checks
the exact external image inventory and variant paths against source inputs.
Rebound same-byte variant aliasing and altered PNG bytes reject in tests.

## Executed evidence

`build/forest-triplanar-batch-v1/independent-coverage.json` passes fresh checks for
all seventeen bindings / 518 placements. Three profiles contribute respectively
**176 Moss 01, 257 Dirt Leaves and 85 Moss 02** source draws. One further selected
binding / ten Grass draws remains rejected because legacy sampler fields need
actual Editor verification; no guessed defaults count as converted content.

`build/forest-triplanar-scene-v1/` composes these placements with the preserved
corrected forest. Independent full input composition passes: **18,820 rendered
primitives, 5,449,323 triangles, 159 authored materials, 48 glTF images and 139
mesh definitions**. Primitive count includes multi-primitive ground/ruins;
18,597 is the source draw count. Thirty-one existing external image variants
remain separate, and the extension uses **eight distinct 2048² RGBA8 images /
128 MiB decoded**. Embedded placeholder images remain; this is not a minimum
residency or streaming budget claim.

Four native Linux Bevy 0.20 views pass on **RX 7900 XTX / RADV NAVI31 Vulkan**:
overview, forest view, unlit colour and decoded-normal diagnostics. Actual
snapshots show all 518 material applications in the exact 176/257/85 split,
18,302 retained Standard primitives and eight extension image handles. Active
Khronos validation, completed pipelines, no renderer errors/panics/tangent failures
and native bounds within **15.02 micrometres** pass. The forest view was visually
inspected. Existing dark lighting and shadow speckling remain unqualified.

The newly introduced Dirt Leaves profile additionally passes three native
prototype modes and **152 colour + 152 decoded-normal pixel samples**, maximum
one/two sRGB8 levels against the unchanged four-level limit. Existing moss-profile
pixel evidence remains in the catalog predecessor; no full-scene CPU pixel oracle
or Unity BRDF/compression parity is claimed.

Khronos checks all seventeen prototypes, seventeen placement files and the combined
scene: **35 files, zero errors, seven runtime tangent-generation warnings**, plus
189 informational unused-accessor/tangent/external-URI messages. Native tangent
generation passes. Fresh fmt, denied-warning workspace/all-target Clippy and
**173 Python tests** (166 asset + four GLB + three Blender references) pass.
The client/server binaries, Cargo.lock and runtime Rust/WESL are unchanged from
the qualified catalog checkpoint, whose native multiplayer regressions remain
retained. No fresh multiplayer, full workspace Rust or Windows execution claim.

Combined GLB SHA-256:
`35ce360c6a4fe9b2ba39a8c3bd2ecf1ee2703f245fd1c9e7f57b34b5283c2315`.
Batch receipt SHA-256:
`1d8ea0781235a277a0cacd986436d7aabacf3aec08bf32bf55a86e5a9b677cdc`.

Evidence stays ignored under `build/forest-triplanar-{batch-v1,placed-inputs-v1,scene-v1}/`,
`build/forest-triplanar-scene-native-v1-20261021/`,
`build/forest-triplanar-dirt-native-v1-20261021/`,
`build/forest-triplanar-dirt-*-pixels-v1.json`,
`build/forest-triplanar-batch-gltf-v1.json` and the matching check logs.
All work remains local; previous qualified assets remain intact.

## Next gates

Inspect actual Editor texture defaults for Grass before accepting its ten draws.
Remaining shader families and mixed slots account for the other 52 unresolved
draws. Rebased world-projection phase, mip/resource policy, authored object collision
and runtime map adoption still require separate bounded slices. Keep the town
default and opt-in ground collision path available. Inspect the new Elven Realms
source folder after forest completion, as requested by the user.
