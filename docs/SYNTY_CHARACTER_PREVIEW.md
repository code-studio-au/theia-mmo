Development adoption update, 2026-10-22: the user's [graybox-world slice](GRAYBOX_WORLD.md)
now selects the converted POLYGON character as development art and uses the four
standard clips. This supersedes the earlier preview-only selection below. The
measured 6.975 mm Jump skin approximation remains explicit; the historical 5 mm
production-fidelity gate is still failed.

# Synty character conversion and preview

Bounded preview qualified, 2026-10-20. P1-10/P2-08/P2-09, user-directed asset
branch alongside execution-order row 5. Prerequisites are the reviewed Base
Locomotion sources and existing standard Bevy glTF/animation inspector. The
normal map/character, authoritative movement, renderer and engine pins are unchanged.
This does not close game character adoption or the remaining world-edge gate.

## Source and conversion

`tools/unity_package.py` inventories and extracts explicit Unity payloads by
pathname/GUID. It handles gzip extra headers and the observed pathname trailer,
never calls `extractall`, rejects traversal/aliases/duplicate destinations and
bounds member/selection sizes. Source archives and existing outputs cannot be
overwritten. Unity scripts and controllers are not executed. The sample uses
seven extracted files: character mesh, idle/walk/run/jump FBXs, the dummy palette
and its Unity material. These remain private/ignored under
`build/synty-locomotion-source`.

The separate Python 3.13 environment in `build/blender-tools` pins Blender
`bpy==5.2.2` via `tools/synty-conversion-requirements.txt`. It changes no system
installation or Cargo dependency. Use Blender's standard FBX import and glTF
export operators. [Python module documentation](https://docs.blender.org/api/current/info_advanced_blender_as_bpy.html),
[glTF export API](https://docs.blender.org/api/current/bpy.ops.export_scene.html).

`tools/convert_synty_locomotion.py` validates extracted input hashes, imports the
sample skin and the four explicit clips, checks all 50 bone names/parents, and
bakes source world poses onto the mesh's own skeleton. Matching names alone was
insufficient: the source bind matrices differ by arm/hip pose. Source mesh Hips
is also connected to Root in Blender, while the clips translate it independently.
Clearing that editing constraint preserves bind matrices/parentage and permits
correct pose baking; it introduces no custom game animation code.

Every baked frame compares all target/source bone world matrices. Export slides
each clip to time zero, retains Y-up metres and omits unused vertex colour layers
that the selected Unity material does not consume. The dummy palette GUID agrees
with the material's BaseMap/MainTex, and both exported embedded PNGs retain exact
RGBA source pixels. Standard material uses metallic 0 and roughness 0.5; this is
an explicit material approximation, not Unity shader/lighting parity.

The skinned mesh is an identity scene root with its geometry transform applied.
This makes standard geometry bounds and skin evaluation agree in metres. The
initial nonroot export produced correct skin pixels but centimetre-sized,
misoriented ordinary bounds; that export is not the qualified artifact.

## Full source skin and bounded preview

The source has 2,477 vertices before flat-normal/UV splitting. Forty-two use five
or six influences. The pinned Bevy standard skinning shader takes four weights.
The converter therefore retains two standard GLBs rather than silently losing
the original weights:

- `build/synty-character-canonical-stage/character-source-skin.glb`: retains all
  source influences and the retargeted/baked clips. This is the source-skin
  reference, not the asset passed to the current four-weight Bevy renderer.
- `.../character.glb`: explicitly requested four-weight preview, keeping the four
  largest influences and renormalizing. Both files contain one skin, 50 joints,
  four animations, one palette image/material and 4,896 triangles. The rendered
  vertex count is 10,287 after export splitting. Bind-pose height is 1.793552 m.

| Clip | Duration (s) | Maximum sampled four-weight skin difference |
|---|---:|---:|
| Idle | 1.766667 | 1.309 mm |
| Walk | 1.033333 | 2.164 mm |
| Run | 0.700000 | 3.588 mm |
| Jump | 2.200000 | 6.974 mm |

These compare full and reduced skinning over every source frame for these four
clips. They are not bounds over all continuous times or the whole animation pack.
The initial 5 mm skin gate remains failed for Jump. Normal conversion rejects
that result; `--four-weight-preview` explicitly permits the measured preview and
records `four_weight_skin_within_5mm_gate=false`. The full source-weight GLB is
preserved. This preview is not approval of that approximation for production
characters; do not replace the current skin or broaden the quality claim.
Root horizontal motion is below 0.7 mm in the inspected clips. Source skeletal
animation is presentation data, never movement authority.

## Verification and use

`tools/verify_synty_character.py` independently samples exported glTF interpolation,
node hierarchies, inverse binds and skin matrices using NumPy. Joint positions
and world deformation deltas compare against the original Blender source-rig
samples, independent of changed local bone bases. All 175 sampled frames pass;
maximum joint position error is about 0.01154 mm. It separately measures the
full-to-four-weight deformation difference above rather than calling it exact.
Both GLBs pass Khronos validation with zero errors/warnings; one informational
unused UV1 attribute remains.

Four native Bevy inspector cases pass Idle/Walk/Run/Jump on RX 7900 XTX/RADV Vulkan
with active validation, one instantiated animation player, changing transforms,
matching skin/joint/image/material counts, metre bounds, ready render pipelines
and substantial source-palette body pixels. A walk screenshot was visually
inspected. This is standard Bevy playback, not gameplay locomotion integration,
networked animation selection or native Windows qualification.

```sh
python3.13 -m venv build/blender-tools
build/blender-tools/bin/python -m pip install -r tools/synty-conversion-requirements.txt
build/blender-tools/bin/python tools/convert_synty_locomotion.py \
  build/synty-locomotion-source NEW_OUTPUT --four-weight-preview
build/asset-tools/bin/python tools/verify_synty_character.py NEW_OUTPUT
target/debug/theia-client --inspect-asset \
  build/synty-character-canonical-stage/character.glb --animation Walk \
  --focus 0,1.1,0,1.5,0.3,0.12
```

The desktop preview uses that last command. RMB orbits, wheel zooms, Escape closes.
Idle/Run/Jump are also valid `--animation` names. Generated/proprietary files remain
ignored. [Executed evidence](VALIDATION.md). Next asset step is representative
Elven Realm mesh/material conversion and resolved demo prefab placements; retain
the core finite-world/resource and authority gates before normal adoption.

Before gameplay adoption, also qualify Unity importer/controller semantics and
root/ground contact. The sampled preview contains below-floor vertices: Jump
reaches about -0.145 metres, Run -0.055 and Walk -0.033. These are measured animated
bounds, not authority collision errors; current evidence proves exported poses,
not foot planting, humanoid retarget equivalence or animation-driven movement.
