# Current scope — 1 November 2026

The [strict WoW Classic decision](COMBAT_TARGETING_MOVEMENT_SPEC.md) defers dodge.
The converted clips and intake proof below are preserved experimental assets;
further dodge gameplay integration is outside the initial milestone.

# Authored dodge intake — qualified offline/inspector prerequisite, 2026-11-01

P1-10 / P2-09 / P3-14; prerequisites are the existing canonical body/socket
conversion and the qualified predicted-dodge gameplay checkpoint. Eight explicit
non-root-motion Sword Combat FBXs share the 52-bone names/parentage, have finite
sampled poses and zero Root translation. Bind poses differ: conversion bakes
explicit desired parent/world matrices rather than copying raw curves.
No Unity scripts/controllers are extracted or executed.

The four directional dodges last 17/30 seconds; the rolls last 35/30 seconds.
The full 16-clip conversion passes independent comparison of 540 sampled frames;
the proposed first 12-clip asset (existing eight plus four shorter dodges) passes
396 frames. Its body vertices, weights and inverse bind matrices exactly match
the existing eight-clip preview. The shorter dodges fit the existing conservative
animated culling bounds. No movement policy or collider is driven by these clips.

Both 16-clip GLBs pass glTF validation with zero errors/warnings; the 12-clip
preview also passes. All eight new clips play in the standard Bevy inspector,
with 20–41 changing transforms per clip, active Khronos validation and zero GPU
diagnostics on Linux RX 7900 XTX / RADV NAVI31 Vulkan (Mesa 26.2.6 kisak).
The captured forward dodge and right roll were visually inspected. These are
asset-viewer checks, not gameplay animation adoption or new performance evidence.

The production 5 mm four-weight gate remains **failed**. The 16-clip preview
reaches 15.64 mm on a roll; the 12-clip selection retains the previous 8.775 mm
maximum. Full source skins are preserved. Use the four shorter dodges for the
first development integration; keep rolls as separately opted-in inspection
assets. Source animation lengths must not silently change the 250 ms authority
policy: playback timing/facing, closed weapon grip, critical cast/swing priority,
life/stream cancellation and correction replay still need gameplay qualification.
Remote-player dodge presentation is also a separate, unimplemented requirement.

`tools/convert_synty_locomotion.py --dodge-source` adds four directional dodges;
`--include-dodge-rolls` adds the other four. Matching verifier flags are
`--combat --dodge` and optionally `--include-dodge-rolls`. Legacy four/eight-clip
verification and all 43 Python checks pass. Formatting and workspace/all-target
denied-warning Clippy pass; no Rust gameplay behavior changed in this intake.
An initial converter source-selection typo stopped before export and was fixed;
its failed log is retained. No source asset or existing qualified output changed.

Exact tooling/source/converted art/proof:
`build/dodge-animation-intake-qualified-v1-20261101`.
Extraction: `build/sword-dodge-source-v1/extraction.json`; comparison:
`build/sword-dodge-inspection-v1.json`, `build/dodge-step-body-comparison-v1.json`;
verification: `build/dodge-native-clips-v1/verification.json`,
`build/dodge-animation-results-v2.json`, conversion/verification logs and GLBs.
The playable build remains `build/dodge-qualified-v1-20261101` with its existing
locomotion presentation. Native Windows, authored gameplay dodge and production
skin fidelity remain open. Art stays ignored; changes remain local and unpushed.

Update: the selected Sword Combat sources now have a
[qualified authored-socket and local-player-death integration](PLAYER_ANIMATION_SOCKETS.md).
The intake findings below remain historical source/export evidence.

Update: the wolf now has a [qualified development integration](WOLF_CREATURE.md).
The intake findings below remain historical source/export evidence; sword
conversion/adoption and Fantasy Rivals retargeting are separate follow-on work.

# Character asset intake — 2026-10-31

P1-10 / P2-08 / P2-09 / P3-14 precursor. This record covers source inspection
and offline exports, not client adoption or completed animation expansion.
Existing player and training-enemy behavior remain unchanged.

## Sword Combat v1.0.1

The Unity package contains 119 FBX assets, including sample rigs and locomotion,
attack, hit, block and death clips. Explicit extraction of the sample character
and Idle Base, Light Combo 01A, Hit Front React and Death Front 01 sources is in
`build/sword-combat-source-v1`; no package scripts or controllers were executed.

Blender imports show all four selected clips and the sample mesh use 52 bones.
All 50 existing Base Locomotion bone names and parent relationships are retained;
the additions are `Prop_L` and `Prop_R`. The existing converter deliberately
requires an exact 50-bone match, so it cannot consume these sources unchanged.
This structural compatibility is not yet proof of pose fidelity, root-motion
safety, blending, equipment alignment or Bevy playback. Comparison evidence:
`build/sword-rig-comparison-v1.json`. Keep the qualified four-clip player until
replacement conversion and native playback checks pass.

## Fantasy Rivals

The supplied folder contains Unreal assets, with no FBX/GLB sources. Native Linux
Unreal Editor 5.8.4 successfully exported 23 skeletal meshes, 23 PNG textures and
three animation FBXs into `build/fantasy-rivals-unreal-export-v1/exports`.
All 49 export tasks report success with no task errors and nonempty output files.
The project uses copies of source assets; originals were not changed or saved.

The first NullRHI commandlet and subsequent offscreen commandlet both crashed at
Unreal's skeletal material mesh extraction assertion. The successful run enables
`-AllowCommandletRendering -RenderOffscreen -vulkan` and disables material baking;
textures are exported independently. Failed logs remain alongside the successful
`editor.log`, `exit.json` and `inventory.json`. These are offline export results,
not native Bevy rendering evidence or Unreal material parity.

Unreal's registry identifies `SK_Character_Skeleton` and
`SK_BR_Character_Skeleton` as shared Skeleton resources, not character meshes.
The 23 skeletal meshes contain no skeleton enemy model. Retargeting compatibility
of the exported humanoids has not been checked.

## Creatures Pack v1.0

This Unity package is a Polygonal Creatures pack with MeshTint source attribution,
not evidence that its rigs match the Synty humanoid. It contains ten creature
folders: wolf, dragon, giant bee, golem, king cobra, magma, one-eyed bat, spiderling,
treant and treasure chest. Inventory: `build/creatures-pack-inventory-v1.json`.

Recommended first development enemy: the wolf, using its own supplied clips.
Explicitly extracted model, brown palette/material and idle, walk/run without root,
bite, take-damage and death FBXs are in `build/creature-wolf-source-v1`.
Blender successfully imports the model and all six clips; each has the same
41-bone names and parent relationships. The mesh has 745 source vertices,
1,400 triangles and at most four positive skin influences per vertex.
Evidence: `build/creature-wolf-inspection-v1.json`.

Before replacing the training enemy, verify source bind poses, metres/Y-up/facing,
texture binding, exported skin/clip fidelity and absence of gameplay root motion.
Then qualify standard Bevy playback and animation selection against authoritative
patrol, retaliation, death and respawn observations. Animation must not change
server positions, hit reach, collision or damage timing. Preserve the existing
training enemy until those checks pass. No creature conversion, Bevy playback or
networked creature-animation adoption is claimed by this intake record.

All source payloads, copied Unreal project, exported art and inspection artifacts
stay in ignored local build directories; this record contains no proprietary art.
