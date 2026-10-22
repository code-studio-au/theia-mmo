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
