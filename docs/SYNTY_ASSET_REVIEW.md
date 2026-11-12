# Synty source asset review

Inspected at the user's request, 2026-10-20. P1-10/P2-08/P2-09 investigation,
independent of the pending finite-world/actual-town adoption gate. Sources are in
`/home/steven/Downloads/Synty Models`; all proprietary payloads remain there.
Inventory/structure reports and inspection scripts are ignored under `build/`.
Archive content is data; no supplied Unity code or document instructions execute.

The active environment import target is now
[Enchanted Forest](SYNTY_ENCHANTED_FOREST_IMPORT.md), selected by the user after
this initial review. Elven Realm references below describe historical work.

## Observed source contents

| Package | Observed contents |
|---|---|
| Base Locomotion — Character Animset v1.0.4 | 374 FBX files, two PNGs, 172 Unity controllers and two sample scenes. The FBX count includes sample scenery as well as animation clips. |
| Elven Realm v1.4.0 | 794 FBX models, 752 prefab assets, 145 PNGs, 20 TGAs, 128 materials, nine shaders, 425 Unity convex-mesh assets and one assembled demo scene. |
| Modular Fantasy Hero Characters v1.3.2 | 777 FBX files, 900 prefab assets, 35 PNGs and 33 materials. Both original and FixedScale modular character files are present. |

Other available content includes Enchanted Forest, POLYGON Farm, Creatures,
extracted prototype models and a Fantasy Warrior HUD library. These were listed;
only the three packages above received archive/structure inspection.

The locomotion package includes a real skinned `PolygonSyntyCharacter.fbx`, one
mesh and 50 named bones. Selected forward walk/run and standing-jump FBXs each
contain a skeleton with the same 50 names and one animation stack. They share
Y-up axes and FBX UnitScaleFactor 1. Name agreement is a promising source contract,
not a completed retarget/scale/animation playback test. Non-RootMotion variants
exist alongside RootMotion variants; prefer the former for server-owned motion
and verify actual displacement when converting.

The modular character source contains 720 mesh geometries and attachment bones.
Its original UnitScaleFactor is 100, while the FixedScale variant reports 1.
Some finger names also differ from the locomotion skeleton. Assemble one intended
body and explicitly qualify hierarchy/bind-pose/scale mapping before applying
locomotion; do not display all modular meshes at once or assume direct clip reuse.

Elven Realm's `Scenes/Demo_01.unity` is readable Unity YAML, about 48.7 MB. It
contains 15,416 PrefabInstance records referencing 538 distinct source prefab
GUIDs and 15,528 Transform records. These are source serialization counts, not
resolved unique meshes, draw calls, collision bodies or runtime entity counts.
No Terrain component/data reference occurs in this inspected scene; ground mesh
FBXs are present. Preserve prefab hierarchy, overrides and placements during
conversion rather than treating the FBX folder alone as the assembled world.
Water/skybox/UV scrolling/triplanar shaders and Unity-specific controllers require
Bevy equivalents or an explicit initial standard-material approximation.

## Import acceptance and next actions

First convert the locomotion package's own sample character with idle/walk/run/
jump candidates to standard GLB in an isolated preview. Verify metre scale, Y-up,
bind pose, skin weights, animation target paths, root displacement, textures and
actual Bevy playback. Then inspect a representative static Elven Realm mesh and
preserve/reconstruct the demo's prefab placements through a scene export. Measure
resolved scene sizes before choosing content partitions. This is a source asset
branch; it does not replace authority movement, collision, navigation or world
admission contracts.

This review initially converted/rendered nothing. The subsequent
[character preview](SYNTY_CHARACTER_PREVIEW.md) now converts and renders four
clips, with a separately recorded four-weight skin approximation. No private
asset is added to Git or made the active map. No Blender/Unity Editor was initially found in the inspected locations; the
continuation installed an isolated Blender Python module under `build/`.
The normal game assets and engine pins remain unchanged.

Evidence: `build/synty-package-inventory.json`,
`build/synty-package-inspection-qualified.log`,
`build/synty-structure-inspection.json` and `.log`. Package payloads were read
through gzip/tar without filesystem extraction or source changes. The modular
package's pathname entries have a newline trailer; inventory uses the first
path line. Its gzip extra header is handled with Python's gzip reader.

The subsequent [Elven import checkpoint](SYNTY_ELVEN_IMPORT.md) stages selected
source payloads with metadata, recursively audits the demo's dependencies and
qualifies one pillar's base-colour GLB in native Bevy. Full assembled-scene import
remains open, including model-prefab internal-ID mapping and complete materials.

## Starter-zone reuse — 12 November 2026

The user renewed authorization to use this asset library during starter-zone and
editor development. The active source is still `/home/steven/Downloads/Synty Models`.
Enchanted Forest supplies useful boundary and nature pieces; seven previously
verified standard-material prototypes are selected in
`content/scenery/starter-forest.selection.json`: archway, ruins, table, chair,
lantern, log and pebbles. `tools/stage_editor_scenery.py` verifies each source
prototype against its batch and independent geometry/material report, retains
embedded GLB bytes and metre bounds, and refuses changed assets or ambiguous
bindings. It does not infer collision or claim a native viewport check.

```sh
build/asset-tools/bin/python tools/stage_editor_scenery.py \
  build/synty-forest-static-batch-v2 \
  content/scenery/starter-forest.selection.json \
  build/starter-forest-scenery-v2
```

The executed selection produced seven assets totalling 8,731,372 bytes. Proprietary
payloads remain under ignored `build/`; the tracked selection contains only names
and logical IDs. Three regression tests cover exact copying, bounds, changed
source, ambiguous bindings and existing-output refusal. The local Medieval Camp
folder contains Unreal `.uasset` files, so it is not yet part of this FBX/GLB
pipeline. Forest cliffs have qualified projected-material work to reuse after the
standard model-placement path; no material substitution is claimed here.

### Editor dressing study (12 November 2026)

The integrated editor now uses these actual GLBs through standard Bevy world
assets. The normal `tools/edit_zone.py` launcher detects the local v2 catalog
automatically; an explicit `--scenery-library` wins, and
`--no-scenery-library` disables that discovery. Open the saved-reference example with the staged catalog:

```sh
python3 tools/edit_zone.py --zone content/zones/forest-scenery.zone.json \
  --save-path build/forest-scenery-draft.zone.json \
  --scenery-library build/starter-forest-scenery-v2/catalog.json
```

The Scenery menu places a named model at spawn, aligning its scaled lower bound
to the terrain sample under the anchor. Ground repeats that alignment after edits. Explorer selection and direct
position/yaw/scale/name fields support editing, duplication and deletion with
undo; Save writes the document explicitly. Eight example placements include a
small table/chair/lantern group and nearby arch, ruins, log and rocks. Geometry
and textures are loaded from the verified immutable snapshot, with no copies of
proprietary model files in Git. GPU appearance remains unreviewed under the
no-display instruction. Offline playtest now uses explicit v2 collision bindings: source triangles for
archway, ruins, table, chairs and log; decorative lantern and pebbles do not block
movement. World export packages these same visuals and explicit collision bindings.

The windowless `--check-scenery` route loaded all seven models and their images,
instantiated seven mesh entities and verified zero surviving meshes after root
deletion. Evidence: `build/starter-forest-scenery-v1/bevy-scene-lifecycle.json`.

The current v2 selection stages 9,808,112 visual/collision bytes. The actual arch
capsule route and pillar-contact test passes at both ordinary and far origins:

```sh
THEIA_SCENERY_CATALOG="$PWD/build/starter-forest-scenery-v2/catalog.json" \
  cargo test -p theia-zone-editor actual_synty_scenery -- --ignored --nocapture
```

This opt-in test requires the user's local proprietary assets; portable CI does
not report it as an executed pass. The standard scenery tests use synthetic data.

### World publication

Both File → Export world and the headless exporter use the loaded immutable
scenery catalog. For a new output directory:

```sh
python3 tools/edit_zone.py --zone content/zones/forest-scenery.zone.json \
  --scenery-library build/starter-forest-scenery-v2/catalog.json \
  --export build/forest-scenery-world --content-id theia.starter.forest-scenery \
  --revision 1
```

Repeated chairs share one exported mesh; image bytes/material parameters remain
unchanged. Collision surfaces use the existing row-major schema-4 affine contract.
The GLB assembler is a narrow Rust adaptation of `tools/merge_static_glb.py` for
these canonical standard-material models; it rejects external images, animation,
noncanonical roots and extended materials. This does not replace the renderer.

`tools/qualify_zone_scenery.py` independently compared the actual eight-placement
export with the staged source arrays, image payloads, materials, transforms and
collision surfaces. Evidence is under `build/starter-forest-scenery-v2/`; the
export is `build/forest-scenery-world-v1/`. Actual arch passage and pillar contact
also pass against the exported runtime collision at local/far origins, with a
1 cm route agreement limit; millimetre ground-contact differences are retained
in the scope, not described as bit-identical simulation. Native appearance and
connected authority admission for this Synty scene remain separate checks.

## Starter-village package review — 12 November 2026

The user requested the other supplied Unity packages, specifically recalling a
Kingdom and Viking pack. `POLYGON Fantasy Kingdom ... v1.10.unitypackage` is
present and inventories 6,647 payloads, including optimized huts, houses,
blacksmith, tavern, stables, well and modular roof/wall assets. Its archive uses
`./GUID/asset` member names; the package reader now accepts that single root
prefix while retaining duplicate/path/alias checks. Five package extraction
tests pass, including original-byte preservation and normalized duplicate refusal.
Farm inventories 1,549 assets, Pirates 1,683 and Dungeons 2,375. Their candidate
prefabs include barns/fences/orchard trees, timber shops/rocks and rock/cliff
components respectively. These are inventory results, not converted assets.
Reports stay in `build/starter-package-review/`. No Viking/Norse/Nordic-named
archive, file or folder was found under Downloads in this inspection. Continue
with Kingdom for village buildings while retaining that unresolved location.
Archive payloads remain local; no supplied scripts execute.

The starter-valley selection now reuses nine verified static assets, including
`SM_Env_Tree_Small_01` LOD0 and `SM_Env_Tree_Trunk_01`. The selected small tree
retains 2,824 visual triangles and only its 536 trunk triangles block the player;
leaf cards stay nonblocking. Source wind/highlight/LOD limitations from
[FULL_FOREST_FOLIAGE.md](FULL_FOREST_FOLIAGE.md) remain explicit in catalog metadata.
The immutable staged payloads total 18,918,896 bytes. CPU asset loading/deletion,
actual offscreen GPU model/gizmo checks, and source-exact world export comparison
pass. The selection and tools are tracked; GLBs, textures and collision payloads
remain ignored under build. Village placements are still to be authored.

### Kingdom huts in the integrated library

P1-10 / P5-17, 12 November 2026: two optimized Kingdom huts now use ordinary
Bevy/glTF opaque PBR materials, preserving source palette pixels and adapting the
active Standard normal map to RGB8. The importer accepts only the explicit v21
recycle-name ID table and inert standalone legacy prefab header; inherited or
ambiguous ownership/mesh names still refuse. In particular, the blacksmith has a
retired mesh name in its table and remains unconverted. It is not silently rebound.

Both huts have independently checked geometry/UVs/winding/normals and an additional
raw-ufbx point comparison (maximum 1.05 micrometres). An actual script-free Linux
Unity 6000.6.6f1 batch import confirms both mesh IDs and source frame mappings
(maximum 0.57 micrometres), with unchanged original model metadata. A separate
texture collector confirms inherited bilinear/repeat sampler state; those defaults
are not guessed from sentinel -1 metadata. The normal reconstruction's independent
RGB8 vector error is 0.006283 against the 0.007 limit. Source DXT compression,
filtered sampling and Unity lighting/BRDF parity are not claimed. These are static
buildings with source visual triangles as collision, not qualified walkable interiors.

The expanded library has 11 assets and 27,355,312 visual/collision bytes. Its
selection is tracked; all supplied payloads and Unity projects stay under ignored
build. Reproduce staging after conversion and verification:

```sh
build/asset-tools/bin/python tools/stage_editor_scenery.py \
  build/synty-forest-static-batch-v2 \
  content/scenery/starter-valley.selection.json \
  build/starter-valley-scenery-v2 \
  --prototype build/kingdom-hut01-v2 --prototype build/kingdom-hut02-v2
```

Each additional prototype requires `conversion.json`, hash-matching independent
material/geometry verification and `canonical-reference.json` tied to the actual
GLB. The conversion uses `preview_unity_static.py --standard-material
--standard-normal --texture-reference build/kingdom-texture-reference-v1`, followed
by `verify_unity_static.py` and `verify_unity_geometry.py canonical`. Original source
prefabs are `Assets/PolygonFantasyKingdom/Prefabs/Buildings/Preset_Houses/` plus
`SM_Bld_Preset_Hut_01_Optimized.prefab` or `SM_Bld_Preset_Hut_02_Optimized.prefab`.
The source folder is `build/kingdom-village-source-v1`.

All 244 asset-tool tests and eight launcher tests pass, as do formatting and strict
workspace/all-target Clippy. CPU loading creates 12 primitive entities for the 11
assets and deletes all descendants. Both-hut world export preserves attributes,
indices, embedded images, normals, materials, samplers, transforms and collision
exactly (`build/kingdom-export-source-comparison.json`). Actual offscreen GPU
checks and subsequent valley dressing are recorded separately; no desktop is used.

Both huts subsequently pass actual RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan
offscreen rendering with all three transform gizmos, ray-picked move/undo and
preview cancellation. Images are `build/kingdom-hut0{1,2}-gpu/model-*.png`; final
logs are `build/kingdom-hut0{1,2}-gpu-final.log`. An earlier missing capture-directory
error was corrected before these final runs. Clipboard initialization warns because
no desktop is available; there are no GPU errors in the final runs. No native
interaction or Windows rendering claim. Hut 1 has an extended foundation/pier:
place the intended ground/floor level, rather than blindly raising its bottom
bound to the terrain for village dressing.

### Editable starter valley village

P5-17 / P3-15, 12 November 2026: the separate tracked recipe
`content/zones/starter-valley-village.commands.jsonl` creates the 512 m terrain,
six Kingdom cottages, a meeting table/chairs/lantern, 44 forest trees in village
and four outskirt groups, ruins, a hollow trunk, fallen timber, rocks and an exit
arch. Two normal editor transactions place the 65 models and paint 58 trail
strokes connecting the outskirts. Every model remains independently selectable,
movable, rotatable, scalable and undoable. Source cottage foundations extend
below their porch: the recipe sinks the foundation to put the porch at terrain
level. Interiors have not been qualified. No NPC behaviour is implied by cottage
names such as Herbalist or Hunter.

Create a new local copy and export without touching an existing user draft:

```sh
python3 tools/edit_zone.py --no-build --headless \
  --zone content/zones/starter-valley.base.json \
  --commands content/zones/starter-valley-village.commands.jsonl \
  --scenery-library build/starter-valley-scenery-v2/catalog.json \
  --output build/starter-valley-village-v2.zone.json \
  --export build/starter-valley-village-world-v2 \
  --content-id theia.starter.valley --revision 2
```

The exported scene has 232,062 instantiated triangles, 11 shared visuals and
58 solid model instances, within the existing 262,144 visual-triangle limit.
Source-exact export comparison passes all 65 placements, including original
textures/materials/tangents and source collision triangles. The comparison now
uses the actual numeric contract: subtract f64 global origin, then convert local
translation to f32; it no longer incorrectly requires large local f32 translations
to equal f64 values within two micrometres. Saved global source values remain exact.

The actual offline Classic controller follows five routes from the plaza to the
four outskirt clearings and the southern exit, then returns, without jumping or
teleporting past obstacles. Spawn and full scenery collision pass. This is sampled
walkability, not pathfinding, combat, multiplayer or exhaustive containment.
Evidence: `build/starter-valley-village-routes.log`, `-v2-spawn.log`,
`-v2-source-comparison.json`, `-v2-export.log`. Quest contacts, wolves, pickups,
full-zone navigation residency and the connected authored-spawn correction remain
separate work.

The offscreen GPU harness now waits for all model primitives and stable compiled
pipelines before capturing; previously it could capture loaded foliage entities
before the alpha-mask shaders were ready. A canopy-pixel check detects that false
visual completion. This was a capture readiness error, not disappearing tree art.
The terrain-only recipe remains available; neither this recipe nor qualification
opens the desktop or writes the normal editor draft.

Final actual GPU overview and village close-up pass on RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 Vulkan with all 110 primitive entities and visible golden foliage.
Images: `build/starter-valley-village-{overview,closeup}-final.png`; logs:
`build/starter-valley-village-{overview,closeup}-final-gpu.log`. Formatting and
strict workspace/all-target Clippy pass. This remains a scenery blockout; the
mountain boundary, cottage access and set dressing can be refined during gameplay
iteration, and no native Windows/display interaction was performed.
