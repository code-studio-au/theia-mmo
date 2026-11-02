# Travelling magic — qualified, 2026-10-31

P3-06 / initial P5-15 adds finite server-owned homing magic and visible standard
Bevy travel. Checked cast/range/LOS, committed equipment, exact life identities,
static world collision and the qualified hit-outcome path are prerequisites.
This completes the initial magic slice; bows, ranged NPC attacks and AoE remain open.

Matched source, binaries, art and proof: `build/projectile-qualified-v1-20261031`.
**608 workspace tests pass / 40 explicit fixture skips in 115 groups**, along with
formatting, denied-warning workspace/all-target Clippy and **41 Python tests**.
Actual encrypted delivery, OIDC/PostgreSQL/two-client travel/outcomes, old melee/
miss-feedback compatibility and 3440×1440 fullscreen launcher fixture cleanup pass.
All **12 final native starts record zero GPU diagnostics**. Work stopped at the
user's request after this slice; no later feature was started or commit/push made.

## Development behavior

The opt-in `development-gameplay-projectiles.json` keeps Sword Slash and adds a
wand-gated Arcane Bolt: **30 m launch range, 20 m/s homing speed, 45 m maximum
accumulated travel, 3 s lifetime and 0.2 m target radius**. It costs 25 mana, has a
500 ms mobile cast and a 2 s cooldown. Its prototype table is certain hit; other
checked hit/miss/deflect weights are supported. These are development values.

The server validates equipment, range, LOS, readiness and current target at cast
start and completion. It reserves bounded projectile capacity before spending
mana. Launch retains the started costs/GCD/cooldown; flight first advances on the
following fixed step. A valid current impact, rather than cast completion, resolves
the hit table exactly once and applies damage only for Hit. Cosmetic travel never
reports an impact or changes health. Post-launch equipment changes leave the bolt
in flight. Source departure/death/life replacement, target death/respawn,
obstruction, unavailable collision, accumulated distance or lifetime retire it
without a damage roll or refund. A target cannot extend travel indefinitely.

Each complete swept point segment checks static Avian solid and ground collision
before accepting contact. Characters remain physically nonblocking. This is an
initial point projectile with explicit target radius, not a general swept volume,
ballistic arrow or area-effect system. Launch is 0.6 m above the existing actor/
target centers; muzzle/socket presentation and a casting animation remain later art
work. Terminal cues use the last clear position. No I/O or terrain loading occurs
inside the fixed step; unavailable prepared collision fails closed.

At most **16 active/reserved projectiles globally and four per actor** are allowed.
Schema-6 fixed-step combat has a 128-event budget, covering the prior 64-event budget
plus bounded projectile bursts. Older profiles retain their 64-event limit.
Schema 6 permits checked travel only for direct damage; declared launch range
must fit both distance and speed/lifetime. Older catalogues retain instantaneous
semantics and their CE02/CE03 bytes. CE04 carries launch and terminal events in
batches of at most 12; 12 largest launches use **917 bytes** within the 1,200-byte
budget. The 29-byte attack request still has no client trajectory, impact or roll.

## Standard Bevy presentation

A reusable unlit blue sphere follows the known presented target through checked
f64 globals and the central rendering frame. Limits are 32 cosmetic tracks/entities,
64 recent cue identifiers and eight catch-up motion steps per track per frame.
Interpolation smooths fixed travel steps. Authoritative damage, avoidance or
terminal events remove its matching visual; cosmetic expiry/drop never affects
combat. Existing critical feedback queues and reservations remain independent.
This is placeholder spell art, without a trail, muzzle attachment or bow animation.

## Evidence and limits

The pure policy remains separately saved at
`build/projectile-motion-qualified-v1-20261031`: 121 core tests in 19 groups,
including six new motion tests, core Clippy and formatting. It covers supported
20/30/60 Hz rates, exact-budget contact, lifetime clipping between ticks, moving
targets, terminal idempotence, malformed data, deterministic replay and checked
far globals. Fixed time derives from tick count rather than accumulated rounded
periods. Contact comparisons account for coordinate precision without increasing
travel budgets. Later schema-6 catalogue checks bring core tests to 122.

Authority tests exercise delayed hit/miss, post-launch unequip, source death/
departure, actual target respawn identity, moving-target range exhaustion, static
obstruction/unavailable collision and capacity refusal without extra mana cost.
The real encrypted case observes health unchanged after launch, then one impact
at both peers, identical launch/impact ticks separated by at least 20 ticks at
30 Hz and one impact-only draw. Its synthetic admission/item owner is not SQL/OIDC
proof. Protocol tests cover byte bounds, old-version rejection, invalid positions,
foreign identity, impossible catalogue references, extra bytes and terminal kinds.
Client tests cover visual bounds, duplicate/reordered terminals and late launches.

Final receipts are `build/projectile-full-exit-v1.json`,
`build/projectile-native-exit-v1.json` and `build/projectile-compat-exit-v2.json`.
The final source has 602 source/config/tool files. Tests omit debug symbols but
retain runtime debug assertions. Forty persistence/asset fixture skips are explicit;
no fresh standalone PostgreSQL contract-suite run is claimed by this slice.

Each hit/miss/deflect native case uses actual owned Keycloak accounts, PostgreSQL,
secure Renet and two Linux Vulkan clients at 1920×1200. Both clients capture a
visible bolt before the first result while observed target health is 100. Hit
then changes health 100→80→60→40; miss/deflect leave it at 100. Each case records
exactly three launches, impacts, resolutions and random draws, no projectile
failures or cosmetic drops, and checks windup unequip cancellation, wrong weapon
and out-of-range refusal. Global/local conversion error stays below 0.001 m.

The native fixture deliberately uses a stationary target, 3 m/s travel and a
10-second lifetime for readable captures. Those values do not replace ordinary
content. Moving-target behavior is tested headlessly; prior pursuit remains a
separately qualified checkpoint. These cases do not demonstrate a combined native
chasing-enemy/projectile encounter or an origin change during flight. The rebase
counter includes initial projection and is not proof of a crossed origin boundary.

Old schema-4 melee and schema-5 miss feedback pass with the same new binaries,
including original source-matched local/remote standing/walking/running sword
poses. Normal launcher cleanup uses a projectile fixture whose primary spell requires the
final sword from the existing inventory tour; the ordinary wand requirement is
qualified separately by the two-client cases. All three cleanup modes pass, with captured
3440×1440 fullscreen layout. Owned services stop and private tickets/snapshots are
removed. The first cleanup attempt retained the inventory tour’s sword and correctly
failed its wand-gated cast tour. That failed diagnostic run is preserved. The
corrected fixture changes only the test’s required item and bundle ID; ordinary
content and candidate binaries are unchanged. The ultrawide launcher checks are
not an in-flight ultrawide capture.

Device: Linux Ryzen 7800X3D, RX 7900 XTX, RADV NAVI31 / Mesa 26.2.6 Vulkan.
No current standalone FPS, native Windows, crowd or MMO-capacity result is claimed.
One enemy and at most eight development players remain the prototype scope. Prior
melee/outcome checkpoints, original assets and full-weight character master remain
preserved; the four-weight character preview still fails its separate production
5 mm skin-fidelity gate.

## Launch

With the existing local OIDC/PostgreSQL setup and prepared development terrain:

```sh
python3 tools/play.py --account-login \
  --binaries build/projectile-qualified-v1-20261031 \
  --gameplay-validator build/projectile-qualified-v1-20261031/validate_gameplay \
  --gameplay-manifest content/gameplay/development-gameplay-projectiles.json \
  --players build/projectile-qualified-v1-20261031/art/player \
  --creature-asset build/projectile-qualified-v1-20261031/art/wolf/wolf.glb \
  --window-size 1920x1200
```

Use `--window-size 3440x1440 --fullscreen` for the checked ultrawide layout.
Equip the wand for action 1; equip the sword for action 2. Remaining bow equipment/
arrow presentation, ranged enemy abilities, quest markers, minimap and HUD work are
tracked in [COMBAT_WORLD_FOLLOWUPS.md](COMBAT_WORLD_FOLLOWUPS.md).
