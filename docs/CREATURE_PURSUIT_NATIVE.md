Current-source amendment, 2 November 2026: the unconditional engagement cutoff
reported below is historical. The [melee contact correction](ENEMY_AI_REFERENCE.md)
passes pure-policy and actual fixed-step World checks at 20/30/60 Hz, plus a
35-second native passive fight with zero returns. Current source/art/binaries
and proof: `build/classic-ability-client-qualified-v1-20261102`.
The dated frozen checkpoint below keeps its original behavior.

# Playable creature pursuit — 2026-10-31

P3-07 / P4-03 / initial P3-08 and P3-14. The opt-in development wolf now
perceives a live player, runs along checked navigation, attacks within reach,
gives up at its leash, returns to its captured patrol position and resumes its
saved route progress. Server authority drives the existing Bevy presentation.
The exact runnable checkpoint is `build/pursuit-qualified-v1-20261031`.
Earlier grip, socket, wolf, life and navigation checkpoints remain preserved.

## Implemented behavior

The new thirteen-profile `development-gameplay-creature-pursuit.json` bundle
selects strict optional settings in `development-creature-pursuit.json`:
12 m perception with static LOS, a 35 m leash from the captured patrol anchor,
5 m/s chase/return, 2 s lost-sight grace, 30 s maximum engagement and 2 s before
reacquisition after return. Existing profile bytes keep their earlier behavior.
The initial enemy remains melee-only, with 3 m reach checked at windup and
completion. Player Arcane Bolt retains its finite 30 m catalogue range.

The current life and exact first target are retained; departure, death or a
replacement life ends engagement. Lost sight follows the last seen position
briefly. Return clears attack permission and restores the saved waypoint and
remaining wait at arrival. Return remains damageable and does not reset health
or grant immunity. Production evade/balance rules remain separate work.

One bounded background worker consumes the verified local navigation package.
Fixed ticks never search or read navigation files. Queued, expired, mismatched
or unavailable paths cannot become direct-line movement. The existing collision
and grounding controller follows accepted waypoints. Waypoint arrival tolerates
the declared 25 cm baked-height approximation while retaining a tight XZ bound;
a different floor cannot be silently skipped. Client/server startup and the
content validator reject pursuit without its life/retaliation prerequisites.
[Policy, controller, worker and identity details](CREATURE_PURSUIT.md).

## Final qualification

The unchanged final source passes **557 workspace/all-target tests / 40 explicit
fixture skips in 113 groups**, workspace formatting, denied-warning
workspace/all-target Clippy and **41 Python tests**. The full test run uses
`CARGO_PROFILE_TEST_DEBUG=0` to bound local build storage; runtime debug assertions
remain enabled. Native executables use the normal optimized release profile.
Logs are `build/pursuit-final-*-v1.log` and `build/pursuit-python-v2.log`.

- `build/pursuit-oidc-native-v2-20261031`: two actual OIDC accounts, owned
  PostgreSQL and encrypted native clients at 1920×1200. A receives melee damage,
  retreats 45 m and stops taking damage beyond the leash. B observes more than
  15 m of NPC movement, retains private full health, and sees the wolf return.
  Authority records one acquisition, one return/arrival, no failed/blocked paths
  and subsequent patrol movement. Sampled movement rejects teleport-sized jumps;
  Run and owner Attack animation frames are observed. The deterministic fixture
  changes initial patrol wait to 15 s and damage to 1; chase/leash/reach rules are
  unchanged. Both captures have zero GPU diagnostics and owned services/tickets
  are cleaned. This uses ordinary Bevy movement input, not physical OS keys.
- `build/pursuit-gear-1920-native-v1-20261031`: real equipment widgets, committed
  replacement/unequip, local/remote standing stance and moving equipment pass.
  Independent live Walk/Run joint comparison preserves the authored closed
  fingers exactly; all other sampled joints retain the original gait within
  0.0000054 matrix-element error. Seven native starts, zero GPU diagnostics.
- `build/pursuit-life-1920-native-v1-20261031`: prior rewarded life/quest/equipment
  integration, completed owner Death, fresh-life animation, rejoin, server/database
  restart and backend-loss handling pass with the preserved non-perception
  fixture. Seven native starts, zero GPU diagnostics.
- `build/pursuit-legacy-1280-native-v1-20261031`: the original four-clip player
  retains its fallback and passes rewarded gameplay/rejoin/restart/backend-loss.
  Seven native starts, zero GPU diagnostics.
- `build/pursuit-launcher-fullscreen-v1-20261031`: normal executables with the new
  pursuit manifest pass interrupt, child-failure and pending-login-interrupt
  cleanup. Both native captures are exactly 3440×1440 fullscreen, with zero GPU
  diagnostics and cleaned owned children, private tickets and sealed snapshots.

The actual final client and server also refuse an incomplete pursuit configuration
before launching gameplay. The final release validator accepts both new pursuit
and preserved player-life bundles. Linux hardware is RX 7900 XTX / RADV NAVI31,
Mesa 26.2.6 Vulkan, Ryzen 7800X3D. Native Windows is unqualified.

## Running the checkpoint

With the existing local OIDC/PostgreSQL configuration:

```sh
python3 tools/play.py \
  --account-login \
  --binaries build/pursuit-qualified-v1-20261031 \
  --gameplay-validator build/pursuit-qualified-v1-20261031/validate_gameplay \
  --gameplay-manifest content/gameplay/development-gameplay-creature-pursuit.json \
  --players build/pursuit-qualified-v1-20261031/art/player \
  --creature-asset build/pursuit-qualified-v1-20261031/art/wolf/wolf.glb \
  --window-size 1920x1200
```

For ultrawide use `--window-size 3440x1440 --fullscreen`. Explicit `--players`
selects the qualified 52-bone combat rig and moving grip. Its source art and
full-weight master are unchanged; the four-weight development preview still
fails the existing 5 mm production skin-fidelity gate.

## Limits and next slices

This is one development creature in an eight-player prototype world, using a
bounded terrain navigation package. Streamed navigation, population capacity,
general threat/boss behavior, public NPC attack timing and remote-player death
remain open. The query library has no hard CPU deadline; its late results expire
outside the simulation. Earlier 30 PostgreSQL contract results apply to unchanged
persistence code; no new standalone SQL contract suite is claimed.

A subsequent unchanged-source [single-client frame baseline](CREATURE_PURSUIT_PERFORMANCE.md)
records 713/712 FPS near spawn; it is separate from these validation runs. Player sword attacks, ranged enemy
abilities, hit/miss/deflect outcomes, travelling magic/arrows, quest markers,
minimap and HUD redesign remain in the
[requested follow-on register](COMBAT_WORLD_FOLLOWUPS.md). Changes remain local;
proprietary art, private fixture data and build/proof outputs stay out of Git.
