Update: The moving grip passes again in the new pursuit checkpoint. [Latest playable proof and command](CREATURE_PURSUIT_NATIVE.md).
The dated checkpoint below remains preserved.

# Equipped sword stance and moving grip — 2026-10-31

P2-09 / P3-08 / initial P3-14. Prerequisites are the qualified 52-bone POLYGON
conversion with authored prop sockets, public equipment appearance, standard
Bevy locomotion and owner-only player-life presentation. This slice selects
SwordIdle from committed public appearance and preserves its closed grip during
locomotion. The matched checkpoint is `build/grip-qualified-v1-20261031`; the prior
`build/sword-qualified-v1-20261031` remains preserved.

## Selection and grip contract

A connected actor with `theia.dev.simple-sword` in its exact public appearance
row uses SwordIdle while grounded and standing, if the asset provides that clip.
Both actor ID and world epoch must match. Wand, no item, missing row, stale actor
or epoch and disconnected state do not select sword stance. Local bag selections
and pending commands are not equipment authority. Remote actors use the same
public appearance selection without access to their private bags or health.

The supplied Sword Combat package has no dedicated sword walk/run clips. Base
Locomotion animates an open hand. Therefore moving sword users keep the original
walk/run/jump and layer the authored closed grip on the eleven right-hand digit
joints only. Standard Bevy AnimationGraph masks exclude those joints from the
moving base clip, and exclude all other targets from a paused SwordIdle grip at
time zero. The original gait retains the wrist, prop socket, arm, torso, left
hand, root and legs. Shared graphs have fixed masked/unmasked variants; equipping
does not mutate a graph used by another actor. Replacing or unequipping the sword
removes the grip layer. This is a finger-only grip layer, not upper-body attack
blending or dedicated equipped locomotion.

Death takes precedence over jump, stance and grip. It plays once, holds the last
pose and resets on a fresh full life key. Dead owners leave sword stance/grip
even if the rig has no Death clip. Remote player health remains private; public
remote-player death is not introduced. Standing stance loops. Attack selection,
upper-body combat layers, crossfades and draw/sheathing remain separate work.

Death and SwordIdle use independent optional graph slots. Assets may contain
both, either or neither; semantic pose/sample slots cannot alias graph indices.
The mask builder requires the exact eleven right-hand digit targets beneath
Hand_R, with source channels in each involved clip. Malformed supported-rig
contracts fail explicitly. The old four-clip rig keeps its existing behavior.
Conservative body animation bounds retain ordinary frustum culling. Animation
supplies no movement, collision or damage authority.

## Acceptance and evidence

Five new unit tests cover exact public actor/epoch/item matching, optional clip
slot permutations, stance/movement/death priority, body-bone pose measurement that
ignores owner movement, and disjoint digit/body masks which preserve wrist and
prop socket ownership. Final unchanged-source checks pass 537 workspace tests /
40 explicit fixture skips in 111 groups, formatting and denied-warning Clippy;
41 Python checks pass. Logs: `build/grip-test-v5.log`, `build/grip-clippy-v5.log`,
`build/grip-checks-exit-v5.json` and `build/grip-python-v1.log`.

The focused native equipment fixture uses real widgets to equip sword, replace
with wand, reject an invalid equip, unequip and equip again. It includes an
explicit equipped walk before the existing run/jump tour. Bounded qualification
samples record local joint transforms and exact base-clip time. The independent
`tools/verify_player_grip.py` compares live local and remote Walk/Run samples
against the original GLB: every digit must match authored SwordIdle at time zero,
and every other sampled target must still match its original locomotion curve,
including wrist, socket, root and legs. At least three digits must differ from
the relaxed gait pose; simply following the socket is not a closed-grip proof.
No such pose samples are captured in ordinary play.

Final native qualification passes:

- `build/grip-gear-1920-native-v3-20261031`: actual two-account widget equip,
  sword/wand replacement, invalid equip refusal, unequip and final equip. Both
  local and remote stance transitions and visible moving equipment pass. The
  session waits for equipped Walk/Run observations before logout. Independent
  `grip-verification.json` checks 53 local targets per sample: all eleven digit
  poses match authored closed grip exactly, and maximum other-target matrix
  error is below 0.0000054. Nine digit joints differ from the relaxed gait pose.
  Both local and remote Walk/Run are observed; native Jump grip is not claimed by
  these captures. Rejoin/restart/backend-loss handling passes. Seven starts have
  active Vulkan validation and zero GPU diagnostics.
- `build/grip-life-1920-native-v3-20261031`: actual two-account OIDC/PostgreSQL /
  encrypted native rewarded gameplay at 1920×1200, completed owner Death,
  fresh-life restart, local/remote standing sword stance, distinct private bags,
  rejoin, server/database restart and backend-loss handling pass. Seven starts,
  active Vulkan validation, zero GPU diagnostics.
- `build/grip-legacy-1280-native-v3-20261031`: the original four-clip 50-bone
  rig keeps its fallback behavior and passes full rewarded gameplay, durable
  rejoin/restart and backend-loss checks. Seven starts, zero GPU diagnostics.
- `build/grip-launcher-fullscreen-v3-20261031`: normal launcher with the new
  player asset passes interrupt, child-failure and pending-login-interrupt cleanup;
  both captures are exactly 3440×1440, with zero GPU diagnostics and cleaned
  owned services, private tickets and sealed gameplay snapshots.
- `build/grip-closeup-v1`: native WalkGrip/RunGrip inspection close-ups of an
  ignored derivative using the same original primitive sword and authored closed
  fingers with original body gait. These provide visual inspection, using the
  prior qualified standard asset inspector. The live gameplay pose comparison
  above independently verifies the runtime masked result. No new source art is
  adopted by this derivative.

Diagnostic runs remain preserved. The standing-only continuation passed gameplay,
legacy and equipment flows but its launcher saw a PNG before its header finished
writing. The helper now waits for all 24 header bytes before checking PNG magic
and exact dimensions; bad complete headers/dimensions still fail. The first grip
run passed equipment but failed the independent pose gate because the old test
logged out the sword owner before movement. The fixture now waits for actual
local and remote equipped Walk/Run samples. The next run passed the gameplay/pose gates but exposed that this focused-test
barrier also held the normal rewarded launcher fixture. The barrier now applies
only to the focused non-reward equipment tour. Earlier runs were not promoted.

Core, network, server, persistence, gameplay profiles and source art remain
unchanged from the socket checkpoint. Its 30 PostgreSQL contract results and
independent 324-frame/eight-clip conversion remain applicable; no new standalone
SQL suite or art conversion is claimed. The full-weight source skin is preserved;
the explicit four-weight preview still fails the existing 5 mm production gate.
No new FPS, crowd-capacity or native Windows qualification is claimed. All changes
stay local; proprietary art and build/proof outputs remain ignored by Git.

With the existing local OIDC/PostgreSQL setup, launch this checkpoint using:

```sh
python3 tools/play.py \
  --binaries build/grip-qualified-v1-20261031 \
  --gameplay-validator build/grip-qualified-v1-20261031/validate_gameplay \
  --gameplay-manifest content/gameplay/development-gameplay-player-life.json \
  --players build/grip-qualified-v1-20261031/art/player \
  --creature-asset build/grip-qualified-v1-20261031/art/wolf/wolf.glb \
  --window-size 1920x1200
```

For ultrawide use `--window-size 3440x1440 --fullscreen`. The explicit `--players`
selects the new rig/grip; omitting it retains the original four-clip player and
its legacy behavior. The rendered sword/wand remain original development props;
the package sword mesh and full equipped locomotion/attack are separate slices.

Next: [requested combat, world and HUD follow-ons](COMBAT_WORLD_FOLLOWUPS.md),
starting with bounded authoritative perception and navigation-backed chase/leash.
