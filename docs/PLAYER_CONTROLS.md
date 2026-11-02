Current implementation: the [qualified `--classic` profile](CLASSIC_MOVEMENT.md)
now supplies the manual movement/camera controls below, disables action camera
and dodge, and supports movement-key remapping. [Persistent Tab targeting](CLASSIC_TARGETING.md) is qualified;
weapon-timed autoattack remains next. Earlier experimental controls are
retained for historical profiles; they are not the default classic contract.

# Governing controls — strict WoW Classic, 1 November 2026

[THEIA-COMBAT-001 v1.0](COMBAT_TARGETING_MOVEMENT_SPEC.md) is the adopted
initial control contract: W/S forward/back, A/D turn, Q/E strafe, independent LMB
orbit, RMB steer, both-button forward, persistent targeting, Tab/Shift+Tab and
weapon-timed autoattack. Keybindings must be configurable. E's old dialogue alias,
the five-slot-only bar and current Escape behavior need migration.

Dodge (V), action camera (F8/temporary cursor), reticle selection and automatic
facing are deferred. Legacy profiles still expose earlier experimental controls;
the descriptions below document those implementations, not the newly adopted
contract. [Gaps, prerequisites and acceptance](MOVEMENT_CAMERA_TARGET.md).

# Historical controls update — 2026-11-01

The [dodge checkpoint](DODGE_MOVEMENT.md) adds V and cooldown hints to the
camera/panel controls below. It passes 100 client tests, 436 network passes / ten
explicit skips, 43 Python checks, workspace Clippy and seven final native starts
with zero GPU diagnostics. Use `build/dodge-qualified-v1-20261101` and the
schema-4 gameplay bundle for current development. Earlier evidence below retains
its exact historical revision and limits.

# Mouse targeting and interaction — qualified, 2026-11-01

P2-05 / P2-11 / P2-12 / P3-06 / P3-12 / initial P3-14 implements the
user's requested control contract as a small extension of existing authority
commands. Prerequisites: known entity keys and target/guide observations,
committed inventory, ability catalogues, server dialogue/range/life checks,
standard Bevy picking/UI and the existing nonblocking actor presentation.

The subsequent [adopted WoW/GW2 movement and camera target](MOVEMENT_CAMERA_TARGET.md)
adds optional action-camera scope; it is separate from the qualified controls below.

## Implemented behavior

| Input | Initial behavior |
| --- | --- |
| Left-click enemy | Request hostile selection |
| Right-click enemy | Request selection and one configured basic attack |
| Tab / Backspace | Cycle enemies / clear target |
| 1–5 | First five catalogue ability slots; absent slots submit nothing |
| Left-click guide | Focus the known friendly NPC and clear hostile selection |
| Right-click guide | Focus guide and request its dialogue/quest/vendor interaction |
| F | Open the known nearby guide; E remains a compatibility alias |
| WASD / Shift / Space / RMB drag / wheel | Existing movement, run, jump and camera controls |

The source binding `content/input/development-controls.json` explicitly selects
`theia.dev.sword-slash` as the basic attack. It is embedded at client build time;
this is not a complete runtime key-binding editor. An older catalogue without
that ID remains usable, but right-click cannot invent a replacement ability.
The first right-click behavior is one request, without continuous auto-attack or
automatic movement into range. Existing server equipment, range, LOS, life,
windup, GCD and cooldown rules remain authoritative.

Friendly focus is a client presentation of the known server guide, separate from
hostile target commands. It clears hostile selection and suppresses combat input
while that clear is acknowledged. Dialogue still uses the current server NPC key
and revision and cannot bypass entitlement/range checks. F currently covers the
one guide; generic door/loot/other interaction contexts remain future scope.

Only actor meshes and the 3D camera opt into standard Bevy MeshPickingPlugin.
Terrain is excluded from those mesh tests. A separate Avian SOLID/GROUND ray
rejects selection through static collision. Clicks resolve through at most 32
hierarchy ancestors, recheck exact enemy keys and loaded chunks, and coalesce
into one pending local intent. UI nodes are excluded; pointer drag/raw mouse
travel beyond six pixels suppresses click actions. Animated mesh picking uses
Bevy's CPU mesh geometry, not a newly implemented per-frame skinning hit volume.
World collision, targeting and gameplay damage remain separate.

V requests the [qualified authoritative predicted dodge](DODGE_MOVEMENT.md) in
the schema-4 dodge bundle: 3 m / 250 ms / 3 s cooldown. Movement follows current
travel, or camera-relative backward when stationary. Panels/focus/readiness and
life gate the bounded tap; holding V does not repeat it. There is no invulnerability
or endurance rule. The earlier Left Alt plan is superseded because Cinnamon uses
Alt as its window mouse modifier.

## First optional camera mode — qualified 2026-11-01

The current source adds F8 to toggle optional action camera; hold Left Ctrl to
use the cursor temporarily. Active mode locks/hides the cursor and shows a
centre reticle above the player. Its raised aiming pivot blends smoothly and
remains bounded by static ceiling clearance; menus keep the requested framing.
Left-click requests the configured basic attack at the enemy
under that reticle; right-click selects. Friendly reticle clicks select only;
F remains interaction. Both modes retain Tab and numbered ability keys.

Dialogue/pending dialogue, inventory and focus loss suspend action camera;
closing a panel restores the requested mode. Escape first disables active action
camera, preserving the normal dialogue-close and classic quit paths. Pointer
messages are filtered before standard picking while locked, with cancellation
at transitions, so stale cursor coordinates cannot activate UI. No position,
range or LOS authority changes. Reticle ray hits reuse existing checked mouse
command consumption and static occlusion. Requests remain single presses, with
no new held auto-attack behavior.

Classic left-drag now orbits without redirecting held travel; right-drag steers.
A fresh movement press adopts the current view. Pitch permits looking toward/
above the horizon; existing world camera collision still retracts the eye.
Mode changes preserve view angles and zoom; transition-frame cursor motion is
ignored. Movement input clears when the window loses focus.

**97 client tests**, formatting, denied-warning workspace/all-target Clippy and
**41 Python checks** pass. The full network suite also passes **407 tests / 10
explicit fixture skips / 63 groups** after the inactive dodge DTO/codec additions.
This is not a new full-workspace test run.

Actual X11 synthetic OS input passes at 1280×720, 1920×1200 and 3440×1440
fullscreen, including crosshair guide/enemy selection/basic requests, F dialogue,
classic dragging, UI ownership, temporary cursor, inventory suspension/restoration
and Escape releasing action mode. Two-client authenticated melee and projectile
regressions pass. All **seven final native starts have zero GPU diagnostics**
on Linux Vulkan, RX 7900 XTX / RADV NAVI31, Mesa 26.2.6 kisak. These are synthetic
OS-input and native rendering checks, not physical hardware or Windows execution.
No new FPS, population or streaming-capacity claim follows.

Matched source/binaries/art/proof: `build/camera-mode-qualified-v1-20261101`.
Final evidence: `build/camera-mode-os-native-v8`,
`build/camera-mode-enemy-native-v3`, `build/camera-mode-projectile-native-v3`;
exit summary `build/camera-mode-native-exit-v3.json` is all zero. Formatting,
client tests and release: `build/camera-mode-build-*-v3.log`; Clippy:
`build/camera-mode-clippy-v3.log`; Python: `build/camera-mode-python-v2.log`.
The earlier quest/rejoin/restart/marker lifecycle evidence below is retained;
that entire lifecycle was not rerun for this camera slice.

Retained diagnostics: earlier desktop attempts exposed the fixture's vendor
backpack remaining open, excessive centre-point precision for discrete mouse
motion, and moving/CPU-mesh aim misses. The fixture now closes all relevant
panels and retries bounded points on the moving mesh. OS v6/v7 were deliberately
interrupted during the aiming-pivot refinement. Screenshot inspection caught the
initial reticle being covered by the player; the final raised, collision-bounded
pivot fixes that presentation. Failed/interrupted attempts are not counted as
qualified starts. Picking still uses CPU mesh geometry rather than deformed
skinning; reticle attack requests do not by themselves prove damage. The
separate actual melee/projectile regressions verify the existing damage paths.

This first camera slice has no live dodge, independently replicated facing,
authored strafe/roll animations, autorun, settings persistence or general
rebinding UI. The prior qualified playable checkpoint stays available. Backpack currently
closes with I; proper Escape panel priority and classic world dragging while
panels are open are the next control refinement. Movement-facing animation,
classic autorun/both-button travel and subjective tuning remain open.

## Qualified evidence and limits

Matched source, executables, art and safe proof:
`build/world-input-qualified-v1-20261101` (607 source/config/tool files).
All **91 client tests**, formatting, denied-warning workspace/all-target Clippy
and **41 Python tests** pass. The preceding whole-workspace result remains
608 passes / 40 explicit fixture skips; a new complete workspace test is not
claimed for this client/tool-only slice. Server/core/network code is unchanged.

The final exact client has **14 native starts, zero GPU diagnostics**:

- Three normal-launcher X11 synthetic OS input sessions at **1920×1200,
  3440×1440 fullscreen and 1280×720** prove left-click guide selection,
  right-click dialogue, Escape/F, world camera dragging and UI pointer ownership.
  This is operating-system delivery, not physical mouse/keyboard hardware.
  `build/world-input-os-native-v2/verification.json` and per-size screenshots.
- Seven actual OIDC/PostgreSQL/native starts prove two characters' quest flow,
  guide mesh picking, F return, marker phases/removal, rejoin, server/database
  restart and backend-loss refusal. `build/world-input-friendly-native-v2`.
- Two actual secure-Renet melee clients prove wolf mesh left-click selection,
  right-click sword damage and existing windup/unequip/wrong-weapon/range checks.
  `build/world-input-enemy-native-v2`.
- Two actual travelling-magic clients preserve finite launch/travel/impact,
  visible pre-damage flight and the existing authority/gear/range checks.
  `build/world-input-projectile-native-v2`.

Device: Linux, AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan, Mesa 26.2.6
(kisak-mesa PPA). The ordinary OS sessions use the normal presentation cadence;
no new sustained FPS, crowd/streaming capacity or native Windows claim is made.
The first revision separately passed 30 native starts, including full quest/OS
widget layouts at all three sizes; those precede the final camera ownership fix
and have a different exact client hash. Final OS screenshots were inspected.

The camera waits for more than six pixels of drag before locking the cursor.
A UI-origin press retains ownership throughout its drag. Unit tests and actual
X11 delivery verify both; ordinary non-targeting legacy camera mode remains.
Input telemetry and scripted drivers install only under explicit qualification
flags. Normal play contains no automatic login, input or test output writer.

The preserved projectile checkpoint remains available. Dodge, continuous basic
attack, generalized friendly targeting/healing, runtime rebinding, additional
context interactions and final HUD/skin/animation quality remain separate scope.
No engine/plugin versions or movement wire schemas changed. Changes remain local.

## Launch

With the existing local account/database settings (see
[authenticated admission](AUTHENTICATED_ADMISSION.md)):

```sh
python3 tools/play.py --account-login \
  --binaries build/world-input-qualified-v1-20261101 \
  --gameplay-manifest content/gameplay/development-gameplay-projectiles.json \
  --players build/world-input-qualified-v1-20261101/art/player \
  --creature-asset build/world-input-qualified-v1-20261101/art/wolf/wolf.glb \
  --window-size 1920x1200
```

Add `--window-size 3440x1440 --fullscreen` for the ultrawide mode. Equip a sword
in the backpack before basic melee; a wand enables the separate Arcane Bolt slot.

## Panel/control-hint refinement — qualified, 2026-11-01

The current local revision closes dialogue before backpack and permits quit
only after both close. Classic world drags/zoom work around open panels; actual
UI hover consumes wheel input and UI-origin gestures retain their ownership.
Mode-specific HUD hints expose F8, Left Ctrl, mouse actions and Escape context.
The real input/inventory close systems have a headless panel-priority test;
actual camera tests cover UI-wheel versus world zoom and world/UI dragging with
the backpack open. All 99 client tests, formatting and workspace/all-target
Clippy pass (`build/camera-panel-tests-v2.log`, `build/camera-panel-clippy-v1.log`).
All three final OS-input native starts pass at 1280×720, 1920×1200 and
3440×1440 fullscreen, with zero GPU diagnostics. Checks cover Escape priority,
world dragging with the vendor backpack open, UI gesture ownership, action
camera restoration and visible mode-specific hints. Evidence:
`build/camera-panels-os-native-v1/verification.json` and its screenshots/telemetry.
Linux RX 7900 XTX / RADV NAVI31, Vulkan, Mesa 26.2.6 kisak. This revision also
passes 416 network tests (10 explicit skips / 64 groups) and 41 Python checks.
Frozen source/binaries/art/proof: `build/camera-panels-qualified-v1-20261101`.
The earlier camera checkpoint above retains its separate actual melee/projectile
proof; those combat starts were not rerun on this panel revision. No live dodge,
new sustained FPS/capacity, Windows or physical hardware input qualification.
