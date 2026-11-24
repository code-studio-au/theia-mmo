# Classic directional locomotion — qualified local-player slice, 3 November 2026

P2-09 / P3-08 / P2-05. Prerequisites are qualified
[ground movement v3](CLASSIC_GROUND_MOVEMENT.md), independent manual facing,
the POLYGON conversion, authored prop sockets, closed sword grip and standard
Bevy animation. Matched source/client/server/art/proof:
`build/classic-directional-client-qualified-v1-20261103`.
The ground-v3 playable and separate directional-art prerequisite remain preserved.

## Implemented presentation

Complete optional art adds backward, left/right and diagonal clips to the
original forward Walk and Run, producing eight directions for each gait.
Direction comes from checked global interpolation endpoints relative to the
actual displayed owner quaternion. Global differences normalize before f32
conversion. Camera and target state do not participate, and selection never
turns or moves the actor. Authority, physics, world identity, movement wire 12,
Renet transport revision 1 and library pins remain unchanged.

Only the explicit Classic profile with a complete directional set enables the
new selection. Zero optional clips retain the qualified four/eight-clip fallback;
a partial set fails explicitly. Semantic slots 10–23 leave the existing 0–5
base/life/stance and deferred 6–9 dodge slots intact. No dodge work is adopted.

The presentation classifier separates walking from running at the midpoint of
the shared 2.5 m/s walk and 4.5 m/s backward-run rates. Thus backward running
uses RunBack. This is an explicit Theia presentation rule, not a new Classic
formula; generalized speed modifiers will need their own gait-mode contract.
A 6° sector margin prevents boundary chatter. Compatible moving clips preserve
normalized cycle phase. Life, critical-pose, mask and attack-sequence changes
retain their existing restart barriers. This slice adds no crossfades or
foot-contact/stride-rate calibration.

Standard Bevy masks preserve the existing partitions across all new bases:
body/wrist/socket versus authored closed fingers, or lower gait versus upper
sword strike. Death, jump, stance, accepted strikes and deferred dodge retain
their priority. The sampled bounds of all 22 clips fit the existing conservative
body bounds; ordinary frustum culling stays enabled. Pose diagnostics aggregate
directional variants into the existing walk/run buckets.

Bounded metrics and the capture-only
`THEIA_DIRECTIONAL_LOCOMOTION_QUALIFY` flag observe native poses without
activating the earlier scripted equipment/movement helpers. No production
bone-pose capture is enabled by default.

## Qualified art prerequisite

The supplied Base Locomotion 1.0.4 package provides fourteen explicit
non-root-motion masculine POLYGON direction FBXs. Optional
`--directional-source` extends the existing standard Blender FBX→GLB conversion,
retaining Idle/Walk/Run/Jump, SwordIdle/SwordAttack/SwordHit/Death, skin, palette
and authored Prop_L/Prop_R sockets. Unity scripts/controllers are not executed.

Independent `--combat --directional` verification checks 22 clips / 52 joints /
702 source frames. Maximum baked world-matrix error is 9.522e-6; maximum source
root horizontal span is 0.690 mm. Full and reduced skins remain separate.
The approximately 8.8 mm worst four-weight preview error is unchanged; the
5 mm production-fidelity gate remains failed.

`tools/verify_directional_extension.py` compares both artifacts against the
ground-v3 art. Geometry, normals/UVs, topology, joints/weights, inverse binds,
hierarchy, material and embedded palette bytes are identical. All eight retained
clips match at keys and midpoints: 640 samples per artifact, **zero** maximum
retained world-matrix error. Seven synthetic checks cover relocated buffers and
refusal of changed geometry/palette/hierarchy, missing directions, altered curves
and changed interpolation with identical key poses. The existing two skeletal
oracle and four Unity-package checks also pass; no proprietary art is needed.

Both GLBs have zero standard glTF validation errors/warnings; the existing
unused UV1 information remains. Fourteen native standard-Bevy inspections pass
with 20–41 changing transforms, one animation player, 22 clips, 52 joints,
4,896 triangles and clear pipelines. All have active Khronos validation (loader
insertion recorded), WGPU validation and zero GPU diagnostics on Linux RX 7900
XTX / RADV NAVI31 Vulkan, Mesa 26.2.6 kisak. WalkBack, RunLeft and RunBackRight
close-up captures were visually inspected. This is source-pose/playback evidence,
not a production skin-fidelity or foot-slide claim.

Art checkpoint: `build/classic-directional-art-qualified-v1-20261103`.
Proof: `build/classic-directional-character-stage-v1-20261103/{character-verification,retained-player-verification,gltf-validation}.json`
and `build/classic-directional-native-v2-20261103/verification.json`.

## Runtime acceptance

All **1,145 workspace tests / 50 explicit skips**, **144 client tests**,
formatting and workspace/all-target denied-warning Clippy pass. Six new client
checks cover every direction at multiple headings and large globals, sector
hysteresis, complete/partial/fallback sets, backward-run classification,
normalized phase and invalid/discontinuous input. Existing grip/attack mask
tests now check all fourteen additional bases.

| Final native proof | Scope |
| --- | --- |
| `classic-directional-runtime-native-v1-20261103` | Two actual OIDC accounts / PostgreSQL / encrypted clients at 1920×1200; copied **1 m passive perception only** isolates animation, with original damage/speeds/outcomes/life timings |
| `classic-directional-controls-native-v2-20261103` | New art at 3440×1440 fullscreen; existing isolated-controls fixture, camera/target/UI ownership and all nine authority-tick speed checks |
| `classic-directional-journal-native-v1-20261103` | New art with **unmodified normal schema-15 wolf/quest data**, two quests / three kills / four actions; database restart/fresh rejoin and normal W chunk/origin traversal at 1920×1200 |
| `classic-directional-fallback-native-v1-20261103` | Old eight-clip art with unmodified normal data at 1280×720; same quest/restart/rejoin flow, with directional availability verified false |

The two-account test selects all sixteen walk/run poses on both clients.
Displayed forward stays [0,0,-1] throughout those cases; no travel-derived facing
is introduced. Continuous direction changes observe phase preservation on both
roles. Jump overrides movement. A real accepted sword attack retains RunBack
below the upper-body strike. The observer sees a completed corpse (52 changing
Death transforms), then a fresh generation; owner dead movement/casting remains
blocked, observer HP stays 100 and private incoming windups remain absent.

Independent live source comparisons verify **17 grip poses per role** (all
sixteen directions plus Jump): closed-digit matrix error zero; maximum other
body/wrist/socket matrix-element error 7.140e-6. Both roles' RunBack strike
samples keep authored upper-body and gait lower-body curves, with maximum
matrix-element errors 3.100e-6 / 4.365e-6. These compare transforms, not skin
deformation or physical/pixel accuracy. The observer's WalkLeft capture has
overlapping nonblocking bodies; RunBackRight and corpse captures show the remote
actor clearly. Pose comparisons do not depend on screen overlap.

Both unmodified-data runs preserve journal head 13 / 125 XP after restart/rejoin,
without duplicated progress. With the replacement art, normal W travels
207.889 m through three observed chunk owners and an origin revision (1→2).
Loaded/retired terrain reaches 116/52, four collision patches and 153 saved-tile
reads. Distant guide filtering and quest rewards stay correct. Maximum numerical
map-to-rendered-position error is 9.981e-6 m; the 4 m map texels do not establish
that physical or visual precision.

All **seven final native client starts** have **zero GPU diagnostics** on Linux
RX 7900 XTX / RADV Vulkan / Mesa 26.2.6 kisak. Owned services stop and private
test files are cleaned. No new sustained FPS, native Windows, crowd/MMO capacity
or exhaustive continent-coverage claim. Crossfades, stride/foot-contact tuning,
broader equipped locomotion and speed-modifier gait semantics remain follow-ups.

The first ultrawide checker encountered an empty telemetry file during a write;
the reader now uses the same bounded 2-second retry as the other native helpers.
Final v2 retains all original control/speed assertions and the same binaries.
Earlier parse/Clippy/checker attempts remain under ignored build logs and are
not promoted to passes.

Additional proof: `build/classic-directional-workspace-v2-20261103.log`,
`build/classic-directional-clippy-v2-20261103.log`,
`build/classic-directional-runtime-native-v1-20261103/{grip-directional-verification,attack-directional-verification}.json`.
All changes remain local, uncommitted and unpushed; concurrent story files are
untouched.

With the existing local account/provider/database configuration, launch the
matched replacement using:

```sh
python3 tools/play.py --account-login --classic \
  --binaries build/classic-directional-client-qualified-v1-20261103 \
  --gameplay-manifest content/development-gameplay-classic-public-life.json \
  --players build/classic-directional-client-qualified-v1-20261103/art/player \
  --creature-asset build/classic-directional-client-qualified-v1-20261103/art/wolf/wolf.glb \
  --window-size 1920x1200
```

Use `--window-size 3440x1440 --fullscreen` for the ultrawide monitor.
