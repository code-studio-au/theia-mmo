# wow.export model checks — 2026-10-16

Both models render with their embedded PNG textures through standard Bevy glTF/PBR.
The character also plays the selected stand, walk and run clips. These are local
compatibility checks, not integration into the multiplayer world or a capacity test.

Sources were read from `/home/steven/wow.export/`, after the user re-exported the
character. Immutable copies under ignored `build/qualification/textured-models/inputs/`
were used for final checks. Originals remain untouched in the export folders; neither
large model nor its textures were added to source control.

| Final input | Building: `moonbrook_human_farm.glb` | Character: `humanmale_hd.glb` |
|---|---|---|
| Folder below `wow.export/` | `world/wmo/azeroth/buildings/moonbrook_human_farm/` | `character/human/male/` |
| Bytes | 11,736,020 | 105,004,472 |
| Triangles | 1,555 | 6,766 |
| Mesh primitives | 23 | 13 |
| Materials / embedded PNGs | 12 / 12 | 6 / 6 |
| UVs and authored normals | Present | Present |
| Skeleton / animation clips | None | 215 joints / 416 clips |
| glTF validation | Zero errors or warnings | Still nonconformant; details below |

Hashes of the checked snapshots:

- Building: `7d60b5740f636f536b03695dcf75a6e18ab27e437b15da3c343afd655ee0d798`
- Character: `6253e05cedf604832d441d0525ddcee1164103b8392dddddefa200efde5b0e53`

## Building

The farmhouse displays its thatch, stone and wood textures. All 12 PNGs are packed
inside the GLB, so BLP conversion or the separate texture folder is not needed for
this file. The mesh is static, as required. Source bounds span approximately
19.276 x 17.228 x 18.431 source units; no metre scale was assumed or rescaling applied.

Khronos Validator `2.0.0-dev.3.10` reports no issues. Actual Bevy instantiation and
GPU rendering succeed on RX 7900 XTX / RADV. Collider authoring, traversable interior,
doorway clearance and placement in the authoritative world remain separate tasks.

## Character

The re-export includes textured skin/clothing data and a skinned mesh. Standard
Bevy animation graphs play these exact named clips in inspection mode:

| Clip | Duration | Instantiated players | Sampled transforms changed |
|---|---|---|---|
| `Stand (ID 0 variation 0)` | 2.667 s | 1 | 156 |
| `Walk (ID 4 variation 0)` | 1.000 s | 1 | 129 |
| `Run (ID 5 variation 0)` | 0.667 s | 1 | 130 |

Transforms are sampled after animation evaluation; the native screenshots were
also inspected and show different walking/running poses. This establishes selected
clip playback and skinned rendering, not correctness of all 416 clips, transition
blending, equipment/customization, networking-driven locomotion or WoW shader fidelity.
Reported inspection bounds are undeformed mesh bounds, not animated skin bounds.

The new export fixes the alignment problems observed in the earlier file, but
Khronos validation still reports three empty animation entries (indices 391, 392,
404), incorrect accessor maximum metadata and other warnings. For example,
`/accessors/13/max/0` declares 2.666 while actual data has maximum 0. The saved
100-issue-limited report contains 71 errors and 13 warnings and is **truncated**;
these are not asserted to be the total issue counts. Bevy accepts the file and
the selected clips work, but this is not a clean, fully portable glTF export.
The original export was not repaired or silently rewritten.

The inspector also finds about 2.94 million stored vertex entries across primitives,
while their indices reference only 4,164 distinct entries counted per primitive.
That is buffer/storage redundancy, not 2.94 million visible triangles. The 100 MiB
file and complete animation library are useful compatibility inputs but should be
trimmed/optimized before production asset qualification. No scale/performance claim
is based on loading this one character.

## Reproduce

From the repository root, on a desktop or a prepared native headless display:

```sh
cargo run --locked -p theia-client -- --inspect-asset /home/steven/wow.export/world/wmo/azeroth/buildings/moonbrook_human_farm/moonbrook_human_farm.glb
cargo run --locked -p theia-client -- --inspect-asset /home/steven/wow.export/character/human/male/humanmale_hd.glb --animation "Walk (ID 4 variation 0)"
```

Use the exact stand/run names above to select those clips. Add `--capture <directory>`
to save `asset.json` and `scene.png`. Playback is optional and inspector-only;
the default game still uses capsule players and its static graybox environment.
The capture waits for sampled pose changes when a clip is requested. Unknown or
empty clips and absence of a player fail explicitly rather than claim playback.

Twenty workspace tests, formatting, denied-warning Clippy and build pass. Native
checks have Khronos Vulkan validation active, with no VUIDs, validation errors or
panics. Ignored evidence: `build/qualification/textured-models/` contains source
snapshot hashes, initial and fresh validator reports, test/build/lint logs and
`native/{building,stand,walk,run}/` reports, loader logs and screenshots. The initial
pre-re-export character had different mesh/material counts; its results are not
substituted for the checked snapshot above. Native Windows remains unqualified.

## RustyWolfPup

The additional `/home/steven/wow.export/creatures/RustyWolfPup.glb` renders with
embedded textures and selected animations in the existing inspector, without code
changes. Its original bytes remain external and untouched. Checked snapshot:
4,363,660 bytes; SHA-256
`e172562f41abf30fc2f6a9412ccea0f17717ab62ed95978177db9c40b7cb1453`.

The GLB contains two mesh primitives, 608 triangles, two materials, two embedded
PNGs, authored normals/UVs, one 66-joint skin and 46 animation clips. The inspector
reports 1,114 stored vertex entries and 557 referenced entries summed per primitive.
Undeformed bounds span approximately 3.765 x 1.869 x 1.024 source units. Neither
gameplay scale nor animated skin bounds were qualified.

| Exact clip name | Duration | Sampled transforms changed |
|---|---|---|
| `Stand (ID 0 variation 0)` | 1.333 s | 26 |
| `Walk (ID 4 variation 0)` | 1.000 s | 31 |
| `Run (ID 5 variation 0)` | 0.666 s | 30 |
| `AttackUnarmed (ID 16 variation 0)` | 1.500 s | 28 |

Each run instantiated one animation player, exited successfully and saved a real
1280x720 screenshot plus report. Screenshots were inspected: textures and distinct
skinned poses are visible. All four runs used RX 7900 XTX / RADV on a private
headless Wayland display, with actual Khronos Vulkan validation activation and no
VUIDs, validation errors or panics. The other 42 clips, transitions, gameplay/NPC
integration, material fidelity against the source game and Windows remain untested.

Strict glTF validation **fails**. Khronos Validator `2.0.0-dev.3.10`, with its issue
limit disabled, reports 3,084 errors, two warnings and two informational messages;
the report is not truncated. All errors are `ACCESSOR_TOTAL_OFFSET_ALIGNMENT`:
for example, accessor 5 starts at byte offset 2,432,926, which is not a multiple
of four for its component type. The other messages are `NODE_SKINNED_MESH_NON_ROOT`
and `UNUSED_OBJECT`. This is an exporter/buffer layout issue, not missing BLP
textures. Re-exporting or repairing a separate derived copy to align accessors is
needed before calling the asset portable/clean; successful Bevy playback does not
remove that requirement. No repair was applied to this source.

Reproduce selected playback:

```sh
cargo run --locked -p theia-client -- --inspect-asset /home/steven/wow.export/creatures/RustyWolfPup.glb --animation "Run (ID 5 variation 0)" --capture build/wolf-preview
```

Ignored evidence is under `build/qualification/rusty-wolf-pup/`: `input.json`, an
immutable input snapshot, `validator-full.json`, the native host harness and
`native/{stand,walk,run,attack}/` logs/reports/screenshots. `native/verification.json`
records source, executable, Cargo.lock and screenshot hashes. This was asset-only
qualification using the previously built client; no runtime code changed and the
workspace tests were not rerun for this documentation update.
