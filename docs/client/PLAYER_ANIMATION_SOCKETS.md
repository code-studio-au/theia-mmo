Latest: [Classic directional locomotion](CLASSIC_DIRECTIONAL_LOCOMOTION.md)
qualifies the optional 22-clip player, local/remote closed grip and directional
lower-body strikes. Earlier dated asset/animation checkpoints below remain
preserved; the existing four-weight preview limitation is unchanged.

Update: [standing sword stance and moving closed grip](EQUIPPED_PLAYER_STANCE.md)
are now qualified on the same source art. The socket/death checkpoint below remains preserved.

# Player animation and authored prop sockets — 2026-10-31

P1-10 / P2-09 / P3-08 / initial P3-14 follow-on. Prerequisites are the qualified
POLYGON body conversion, equipment appearance and authoritative player-life
checkpoint. This slice preserves the existing body skin, adds authored prop
sockets and four Sword Combat clips, and presents the local player's death and
fresh-life restart. The qualified replacement is `build/sword-qualified-v1-20261031`; the wolf
checkpoint remains preserved.

## Attachment correction

The previous development equipment uses `Hand_R` with a hand-tuned 3 cm offset
and rotation. That is a wrist-relative fixture, not proof of the package's
intended grip. In sampled Sword Idle and Light Combo frames, `Prop_R` is a leaf
child of `Hand_R`, about 9.8 cm from its origin, with a different orientation.
These sockets do not deform the body but are needed to preserve authored weapon
placement. The package also contains a prop-bone binding tool and sample-scene
weapon attachments; these were inspected as source, never executed.

The combat conversion retains both `Prop_L` and `Prop_R` as nondeforming leaf
bones, bringing the rig from 50 to 52 bones. All existing body weights are
retained. Sword clips bake the source socket poses; the original locomotion
clips, which contain no prop bones, retain the reference hand-relative socket
transform. The independent verifier checks socket parentage, zero socket body
weights and full source/export socket matrices at every sampled frame.

Equipment prefers the single `Prop_R` directly below `Hand_R` on this rig. Its
original primitive weapon geometry is placed at the socket origin and axes,
with only the centimetre-to-metre scale correction. A malformed socket contract
fails explicitly. The existing 50-bone character keeps its qualified legacy
hand attachment. The primitive sword/wand are still original development
fixtures; the package's sword mesh has not been adopted as a runtime asset.
A rigid prop socket following a hand is separate from finger gripping, weapon
stance selection and layered locomotion/combat animation.

## Animation behavior

The retained Idle / Walk / Run / Jump clips are joined by SwordIdle,
SwordAttack, SwordHit and Death in the converted asset. Standard Bevy GLB assets,
AnimationGraph, hierarchy and skinning remain in use. Sword combat clips are
available for inspection; they are not assigned to the existing spell ability.

On an opted-in POLYGON rig with a Death clip, the local player's authoritative
zero-health observation selects Death ahead of locomotion and airborne state.
It plays once and holds its final pose. A changed full life key (actor, world
epoch or generation) resets selection and pose samples, so a fresh life resumes
locomotion from a fresh phase. Remote player health remains private; this slice
does not introduce a public remote-player death protocol. The original four-clip
rig keeps its existing behavior. Conservative animated body bounds retain normal
frustum culling. Animation supplies no movement, collision or damage authority.

## Evidence and acceptance

Conversion: `build/synty-character-combat-stage-v2`, with independently verified
source poses for all eight clips, 52 bones and 324 sampled frames. Standard glTF
validation has zero errors and warnings. Both full-weight source skin and the
explicit four-weight development preview are retained. The worst sampled preview
error remains about 8.8 mm; the existing 5 mm production fidelity gate still
fails. This slice does not promote that approximation to production quality.

All eight clips also pass native standard-Bevy inspection with the same primitive
sword attached to `Prop_R` in an ignored derivative asset, on Linux RX 7900 XTX /
RADV Vulkan with active Khronos validation and zero GPU diagnostics:
`build/sword-socket-native-clips-v1`. Close-up SwordIdle and SwordAttack captures
were visually inspected in `build/sword-socket-closeup-v1`. These inspection runs
use the existing qualified inspector executable, not the new gameplay executable.

Final qualification passes:

- `build/sword-checks-exit-v1.json`: 532 workspace passes / 40 explicit fixture
  skips in 111 groups, formatting, denied-warning workspace/all-target Clippy
  and optimized Linux client build. `build/sword-python-v2.log`: 41 passes in
  the Blender/asset environment with the exact qualified Rust validator.
- `build/sword-life-1920-native-v2-20261031`: actual two-account OIDC,
  PostgreSQL and encrypted native gameplay. Each local player starts Death once,
  reaches its completed pose, records 52 changing transforms and restarts once
  on a fresh life. Each holds the final pose for 60 observed frames. Equipment
  on local and remote actors uses only `Prop_R`, with zero positional socket
  error and maximum scale error below 0.000014. Rejoin, server/database restart,
  distinct rewarded training and backend-loss handling pass. All seven client
  starts have active Vulkan validation and zero GPU diagnostics.
- `build/sword-legacy-1280-native-v2-20261031`: the original 50-bone, four-clip
  player and wrist-relative fixture still pass full rewarded gameplay, rejoin,
  restart and backend-loss checks; seven starts, zero GPU diagnostics. The
  independent legacy four-clip skin verifier also passes.
- `build/sword-launcher-fullscreen-v1-20261031`: normal launcher with the new
  player asset and thirteen-profile manifest passes interrupt, child-failure
  and pending-login-interrupt cleanup. Both native captures are exactly
  3440×1440, with zero GPU diagnostics; database/service credentials stay out of the client environment; private tickets
  and temporary gameplay snapshots are cleaned.

The death test uses a fixture-only 2500 ms respawn delay to observe the whole
1.433 s clip and satisfy the capture helper's 30-frame stability requirement.
Normal gameplay retains its existing 3000 ms respawn delay. The earlier 1800 ms
fixture completed Death but left only 22 held-pose frames and failed its screenshot
requirement; that diagnostic run is retained and not promoted. The initial generic
Python discovery also used the wrong environment and a stale default validator;
final discovery passes in the proper existing tool environment with the exact
qualified validator.

Core, network, server, persistence and gameplay profiles are unchanged from the
wolf checkpoint. Its 30 PostgreSQL contract results are retained; a new standalone
SQL contract suite was not run. The native flows above use actual PostgreSQL.
No native Windows, crowd capacity or new FPS qualification is claimed. All art and
proof stay in ignored build folders; repository changes remain local.

With the existing local OIDC/PostgreSQL setup, launch this checkpoint using:

```sh
python3 tools/play.py \
  --binaries build/sword-qualified-v1-20261031 \
  --gameplay-validator build/sword-qualified-v1-20261031/validate_gameplay \
  --gameplay-manifest content/gameplay/development-gameplay-player-life.json \
  --players build/sword-qualified-v1-20261031/art/player \
  --creature-asset build/sword-qualified-v1-20261031/art/wolf/wolf.glb \
  --window-size 1920x1200
```

For the current ultrawide monitor use `--window-size 3440x1440 --fullscreen`.
Omit `--players` for the original four-clip character and legacy hand fixture;
omit `--creature-asset` for the primitive enemy. Public remote-player death,
weapon stance/blending, actual package sword mesh, additional combat abilities,
production skin fidelity and broader animation/VFX/audio remain future slices.
