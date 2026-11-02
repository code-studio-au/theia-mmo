# Next classic combat slice — weapon-timed melee

The [weapon/clock prerequisite](CLASSIC_MELEE_CLOCK.md) and
[World authority](CLASSIC_AUTOATTACK_AUTHORITY.md) now pass headless checks.
[Explicit transport](CLASSIC_MELEE_TRANSPORT.md) passes encrypted UDP and
regression qualification. [Normal-client adoption](CLASSIC_MELEE_CLIENT.md) is
qualified. [Ability rules](CLASSIC_ABILITY_RULES.md) are next.
P2-12 / P3-06 / P3-08 / P2-14 build on qualified classic facing and persistent
selection, existing committed equipment observations, authoritative damage,
readiness/LOS queries, player life and secure combat transport.

## Required contract

Selection, engagement and ability requests are separate. Left-click and Tab
select without attacking. Right-click chooses a target and explicitly enables
basic melee attack intent. Clearing selection, death, entity retirement, leaving
the session or an explicit stop ends the applicable engagement. Turning, camera
movement, leaving melee range and breaking LOS preserve selection and engagement;
they pause attacks until current authoritative validation succeeds.

Bind engagement to a checked world/entity incarnation. Repeated engage requests
must not reset a weapon timer or produce extra attacks. A late tick cannot catch
up with an unbounded burst. Re-entry into range/facing must not accumulate stored
swings. Keep bounded sequence, event and per-tick work budgets and checked tick
arithmetic. Turning away never rotates the character back toward the enemy.

## Data and execution

Use a bounded immutable weapon profile keyed by the existing item content ID.
Resolve the equipped item only through the committed inventory observation;
client appearance or an uncommitted equipment proposal cannot authorize damage.
Define weapon period, base damage and metric reach independently of ability
cooldowns. Do not implement basic attacks by repeatedly issuing Sword Slash.
An initial training weapon's tuning must be labelled a Theia fixture; adopting
Classic skill/level/AP tables follows CLASSIC_FORMULAS.md and separate fixtures.

Use a data-defined frontal arc, shared with ability validation, explicitly marked
as provisional until measured Classic calibration. Validate current authority
position, independent facing, life, readiness, range, LOS, equipment and disabling
statuses at the swing event. Abilities declare continue/suspend/delay/reset
interaction with the weapon timer. A cast's ordinary cooldown is not that timer.

Reuse authoritative damage/death, retaliation, loot/quest and presentation
pipelines with an explicit basic-swing event/cue. Preserve event identity and
bounded transport budgets. The headless server never waits on inventory I/O.
Bind changed rules/data into the gameplay identity so old clients cannot silently
join a different combat profile. Keep legacy experiments available.

## Acceptance order

1. Numerical weapon-timer and engagement boundary tests at 20/30/60 Hz: repeated
   enable, pause/resume, no stored burst, ability interactions, weapon change,
   death/removal, sequence overflow and time exhaustion.
2. Actual World checks: independent selected/engaged state, successful swings,
   behind/out-of-range/occluded pauses and resumption, committed equipment gates,
   abilities and current authoritative damage/death events.
3. Actual encrypted UDP with loss/jitter/duplication: explicit engage/cancel,
   authoritative swing state/events, replay and foreign-incarnation rejection.
4. Normal OIDC/PostgreSQL native play: equip sword, right-click wolf, turn away
   and back, move out/in range, use instant and cast-time abilities, kill wolf.
   Verify visible feedback and zero unexpected GPU diagnostics at supported sizes.

[Persistent-targeting native qualification](CLASSIC_TARGETING.md) now passes;
weapon timing and authoritative engagement are the next implementation slice. Dodge, action camera, ranged autoattack and full Classic
balance remain outside this slice. No P3-17/18 scope is assigned.
