# Millhaven whole-zone overview

17 November 2026 — P2-14 / P3-15 continuation.

The user requested a whole-zone map to locate the player relative to Millhaven's
village, bandit caves and other key locations, with supplied MMO map screenshots
as layout references. Prerequisites are the accepted whole-zone saved-terrain
package, checked global/local positions, Classic manual facing, and standard
Bevy UI and pointer ownership.

Press **M** or **Zone map [M]** to open/close. Escape closes the map first and
preserves underlying panels/targets. Choose a landmark in the list to show its
exact pin, description and straight-line distance. The map follows the player's
live presented position through the existing checked origin conversion, with
authoritative fallback during admission/rebase. Its arrow follows character
facing independently of the camera. Movement keys still work while the map is
open; mouse steering, orbit and zoom are suspended until it closes. Map controls
do not teleport, navigate automatically or send gameplay requests.
The terrain-only client also installs the existing UI gesture tracker, preserving
the origin of a mouse drag even before the map opens and after the pointer leaves
a button. This does not install targeting or combat for the walkthrough.

The complete 4,096 x 3,072 m terrain is represented by a 1024 x 768 offline baked
relief/contour image, road/watercourse lines and seven supplied landmark sites.
It remains complete regardless of nearby 3D residency. No runtime tile reads,
worker, camera or terrain demand is added. Standard Bevy images and nodes provide
the interface. The overview preserves the 4:3 map aspect and fits small and wide
windows. Minor village sites appear when selected to avoid overlapping pins.

The map is explicitly bound to the exact imported Millhaven V3 terrain identity
and bounds. It is not automatically reused for older authored Millhaven worlds,
unrelated terrain or a modified package. The source's +Z-north convention applies
to both this overview and its matching local minimap, with -X east to preserve
right-handed Y-up orientation; other worlds retain -Z north / +X east. World positions, controls, camera and physics conventions are unchanged.

The source labels describe planned sites. Blue lines indicate watercourses;
water surfaces, bridge decks, buildings and cave interiors remain unbuilt.
The feature does not add fog of war, a world/continent hierarchy, quest-log
integration, live NPC locations, teleporting, map zoom/pan or general map editing.
The supplied screenshots' game art/chat remain local reference material.

Acceptance checks: exact package binding, checked projection and map bounds,
source-north consistency, live position/facing, keyboard/focus/Escape ownership,
landmark selection, small/wide layout, existing client regressions, strict Clippy,
fmt, native rendered UI, tile/mesh budgets and Vulkan diagnostics. Native evidence
is recorded in STATUS: 323 client tests, strict Clippy/fmt and optimized build
pass (`build/zone-map-v4-checks.json`). Native 640x480, 1280x720 and 3440x1400
runs pass under `build/zone-map-native-{small,1280,wide}-v4`, with zero Vulkan
validation diagnostics on Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6.
Windows execution remains separate.

Reproduce native checks with `tools/qualify_imported_terrain.py --zone-map`,
`--case village` or `--case bandit-terrace`, `--window-size WIDTHxHEIGHT` and a
fresh `--output` directory. The opt-in harness uses app-owned key/Activate
fixtures, with an app-local focused snapshot restored in PostUpdate before
Winit's Last-schedule window synchronization. It never requests desktop focus
or sends OS input. This qualifies rendered UI and callbacks, not native physical
keyboard/mouse events. Production focus guards retain automated regression tests.

17 November direction correction: the initial +Z-north / +X-east projection
reflected the scene. Arrow-to-world agreement alone did not catch right turns
appearing as left turns. Schema 2 reverses both map axes, rebakes cartography,
remaps farm compass labels, and shares arrow rotation between both maps. New
regressions apply actual D-key turning through all four cardinal directions and
compare projected forward/right vectors to the arrow for both north conventions.
Native map-turns checks capture north/east/south/west/north and eastward movement.
Earlier v4 acceptance did not qualify this handedness behavior.

Direction-fix evidence: 325 client tests pass, plus 265 editor tests for the
separate camera-navigation correction; strict Clippy/fmt and builds pass.
`build/map-direction-native-v1/qualification.json` verifies cardinal turns and
forward movement on the rendered overview and minimap. Compact captured UI-only
evidence is in `build/map-direction-small-native-v1/ui-qualification.json`;
that process ended before its final terrain report. See STATUS for exact scope.
