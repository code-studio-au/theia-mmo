# Bounded triplanar material catalog

P1-10/P2-08/P2-10 continuation, 2026-10-21, after the qualified
[mesh-frame slice](FOREST_MESH_FRAMES.md). The bounded inspector catalog below is
implemented and qualified; full forest/runtime adoption remains open.

The combined forest needs ordinary Standard materials and several triplanar
profiles in the same GLB. Retain schema-1 single-profile inspector behavior and
add an opt-in schema-2 catalog bound to the complete geometry digest. Each of
1–16 unique positive profile IDs supplies the existing pinned shader profile,
tiling/falloff and four hash-bound PNG roles. glTF material extras select that ID;
unmarked Standard materials remain ordinary Bevy materials. Every declared profile
must be used, and every material mark must resolve. Malformed, missing, unknown
or unused profile mappings reject before capture; no silent fallback.

Schema-2 images use contained content-addressed filenames with distinct colour/
normal variants. Share exact asset handles for identical compatible paths, and
share one extended material per profile. Bound metadata, profile count, distinct
images and decoded texture size. Retain source sampler/sRGB policies and report
actual handles/formats/mips/applied counts per profile. Diagnostic modes remain
explicit; they do not turn unmarked Standard geometry into triplanar geometry.

Acceptance: metadata/mapping/budget/error-path tests; actual two-profile plus
Standard native rendering with exact per-profile bindings and shared image
handles; independent source/pixel checks; missing/duplicate/unknown mapping
rejection. Rerun legacy single-profile cases, corrected full forest and encrypted
town/farmhouse regressions with the final client. Fresh fmt, denied-warning
Clippy and appropriate Rust/Python tests. Preserve all earlier qualified binaries
and assets. Origin-zero inspection remains explicit; ordinary chunk/rebase,
multi-profile resource qualification and full forest adoption remain separate.

## Implemented contract

Schema 1 keeps the original single marked material, role filenames, scalar fields
and capture fields. Schema 2 uses root profile `synty-triplanar-catalog-v1`, binds
the complete GLB SHA-256, and holds 1–16 unique IDs in 1..16. Each entry retains the
existing `synty-triplanar-world-v1` shader contract. Every rendered mark must resolve,
every declared profile must be used, and capture waits for every marked primitive
to receive its material. Invalid marks also reject when metadata is missing.

New filenames are `triplanar-color-<sha256>.png` or
`triplanar-normal-<sha256>.png`. The variant prevents colour/linear roles sharing
an incompatible image handle. Metadata stays within 64 KiB, geometry within
256 MiB, each PNG within 64 MiB/4096², and the catalog within sixteen distinct
images / **256 MiB decoded RGBA8**. Schema-2 PNGs must declare eight-bit channels.
Actual loaded image formats and one-mip state are also checked before application.
Shared paths count once toward budgets. These bounds cover extension dependencies;
ordinary embedded glTF images retain their separate loader/resource costs.

The existing standard PBR extension gets one material per profile. Snapshots report
actual applied counts, scalar values, image formats/samplers/mips/handles, unique
image handles and material count per catalog. Inspector traversal separately counts
remaining Standard-material primitives. No shader code, engine/physics pins or
ordinary gameplay transport/movement changes are introduced.

`assemble_triplanar_catalog.py` combines qualified placed profile groups and
ordinary static GLBs, adds identity ownership parents and remaps only profile IDs.
It deduplicates typed external PNG paths. `verify_triplanar_catalog.py` imports no
assembler and checks complete per-input mesh ownership, actual world matrices,
every vertex stream, winding, materials/fallback images, profile scalars and raw
PNG role mappings against immutable predecessors. Embedded fallback images remain
present; this slice does not claim minimum GPU residency.

## Executed actual data and native evidence

The second actual moss prototype, using `Moss_Enchanted_Triplanar_01`, is converted
and reframed from its actual Editor reference in
`build/forest-triplanar-moss01-frame-v1/`. Its 248-triangle geometry matches actual
Editor point sets within **2.94 micrometres**. All **42 exact-binding placements**
pass source-matrix/normal/winding checks in
`build/forest-triplanar-moss01-frame-group-v1/` (10,416 triangles).

`build/forest-triplanar-catalog-fixture-v1/` combines the existing 85-instance moss
group, these 42 placements and the corrected Standard building placement. Complete
composition checks pass: **128 primitives / 33,823 triangles / three materials /
four glTF images / four source mesh definitions**. Two profiles use eight image
slots backed by **six distinct 2048² GPU images** (96 MiB decoded extension data).
The two shared top-role handles are identical; side-role handles remain distinct.
The building retains an actual Standard-material component. Tiling is respectively
0.2 and 0.5, with source falloff 4.1, verified in actual material snapshots.

Two views, each lit/unlit-colour/decoded-normal, pass native Bevy 0.20 rendering on
**RX 7900 XTX / RADV NAVI31 Vulkan**, Linux, in owned private Wayland compositors.
Active Khronos validation, complete pipelines and actual bounds within **3.26
micrometres** pass without VUIDs, renderer errors, panics or tangent failures.
Both lit views were inspected. Lit moss shows self-shadow speckling; lighting,
shadow and Unity normal/compression parity remain unqualified.

Independent final pixel checks cover **953 colour + 953 decoded-normal samples**.
Each channel checks **872 samples from profile 1 and 81 from profile 2**, with at
least thirty required per declared profile. Maximum errors are one/two sRGB8 levels
against the unchanged four-level limit. Earlier coarse sampling gave only 28 valid
pixels for the smaller profile and remained rejected. A denser deterministic grid
checks more pixels without relaxing edge/visibility/error gates. Unmarked Standard
triangles participate in ray visibility but are not compared with triplanar math.

Six native negative cases reject unknown/invalid marks, missing/unused/duplicate
profile mappings and missing metadata. Rebound geometry/metadata cases exit nonzero
without asset reports/screenshots or panics. Four CPU composition faults also reject
changed source scalars, role mappings, material assignments and poses after hashes
are rebound.

Final-client regressions pass: three legacy single-profile modes, three corrected
full-forest views, and actual encrypted town/farmhouse two-client routes. Khronos
checks the new moss prototype/group and catalog with **zero errors/warnings**.
No source map is switched and the main subset remains **18,079 draws / 580 unresolved**.
The standalone two-profile fixture does not add 127 draws to that counter.

Catalog GLB SHA-256:
`ee603296fa7fb70a3c2ca2be106462f247a4d89d0fb1902d1b2d4f134546b3b7`.
Final client SHA-256:
`061d27f8541ea2fbaa00d5573973f4064edf93bedb8c21aabe9e8b669018a51b`.
The previous qualified client is retained under
`build/forest-frame-qualified-binaries-v1/` with its original SHA receipt.

Evidence: `build/forest-catalog-native-v{1,2}-20261021/`,
`build/forest-catalog-all-*-pixels-v3.json`,
`build/forest-catalog-faults-v1-20261021/`,
`build/forest-catalog-legacy-native-v1-20261021/`,
`build/forest-catalog-forest-regression-v1-20261021/`,
`build/forest-catalog-map-regressions-v1-20261021/`, and
`build/forest-catalog-gltf-validation-v1.json`.

## Checks and continuation

Fresh **35 Rust client tests** cover backward compatibility, exact applied mapping,
image sharing and independent unique-image/decoded-byte budgets; **171 Python tests**
(164 asset, four GLB, three Blender prefab/variant) cover source composition and
retained tools. Fresh fmt and denied-warning workspace/all-target Clippy pass.
The server binary and Cargo.lock stay unchanged; no full workspace Rust test rerun
or native Windows execution is claimed.

Continue eligible source-binding batching and full-scene catalog composition,
then rebased texture phase, resources/mips and remaining shaders/material slots.
Object collision and runtime/chunk adoption remain. Do not use this origin-zero
inspector as evidence of production streaming or complete forest multiplayer.
All work remains local; Elven folder inspection still follows forest completion.
