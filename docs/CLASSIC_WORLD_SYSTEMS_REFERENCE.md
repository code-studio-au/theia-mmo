Current implementation evidence, 2 November 2026:
[Ability damage and the melee engagement correction](CLASSIC_ABILITY_DAMAGE.md)
are native-qualified. [Combat membership](COMBAT_MEMBERSHIP.md) now has an opt-in
server-only World slice with 250 core / 597 network (ten skips) / 123 client
regression, but normal/client observations and permission consumers remain.
The dated assessment below records the earlier adoption snapshot; consult
[STATUS](STATUS.md) for current completed work and pending slices.

# Classic world systems — direction, repo assessment and delivery order

1 November 2026. Reconciled from the user-supplied “Theia — Lessons from WoW
Classic's Underlying Game Systems” planning text. This extends the
[enemy AI reference](ENEMY_AI_REFERENCE.md) and
[strict initial combat specification](COMBAT_TARGETING_MOVEMENT_SPEC.md).
It refines existing task scopes; P3-17/P3-18 remain unspecified.

The supplied notes cite publication names without exact links/builds. Their
historical claims are research leads, not independently verified formulas.
Use [the existing evidence policy](CLASSIC_FORMULAS.md) for any Classic rule or
number adopted into code. Preserve meaningful positioning, encounter danger,
resource management and shared-world consistency. Keep 1 world unit = 1 metre,
m/km, m/s, checked f64 global coordinates and bounded f32 local frames.

## Three working decisions

These recommendations are selected under the user's standing authorization to
use recommended decisions. They define direction; the matrix below records what
actually works. Content coefficients, complete class rules and numerical death
penalties remain open rather than being invented here.

### 1. Combat and progression mathematics

Use a versioned Classic-oriented ruleset, extending Theia's level 1–80 direction
explicitly. Prefer evidenced original-style Classic rules; label the 61–80
extension and any later-expansion adaptations. Establish attack categories,
level/skill inputs, positional eligibility, mitigation, resources and progression
contracts before tuning all coefficients. Exact weapon-skill tables and uncommon
outcomes can follow balance/evidence work without blocking those interfaces.
Prototype 100 HP, flat mana regeneration and weapon values remain test fixtures.

The authoritative pipeline is: validate the action/target/state and current
position/range/facing/LOS; derive eligible outcomes from attacker and defender
facts; resolve the attack-category table; apply mitigation/damage; then publish
threat, combat state, effects and credit facts. Critical resolution belongs to the
appropriate attack table: do not assume every attack uses an independent second
critical roll. Attacker-facing requirements and defender-facing parry/block
eligibility are separate rules. RNG and results stay server-owned.

Keep white swings, physical abilities, spells and periodic effects distinct.
Record rounding and exact threshold fixtures at 20/30/60 Hz; use shared data for
client descriptions without allowing the client to decide outcomes. A complete
level curve, armour/resistance model, crit/block/parry/dodge, glancing/crushing,
healing and class-specific regeneration are further implementation work.

### 2. Creature tagging and shared credit

Ordinary creatures use first qualifying effective damaging contribution as the
initial tag policy, with owner/eligible-party credit rather than the final hitter
implicitly receiving everything. Non-damaging hostility/CC needs an explicit tag
rule before such effects participate. Selecting, approaching or sending a request
does not claim ownership. This is a Theia working policy; exact Classic tagging
exceptions still require source and patch evidence.

Separate tag owner, contributions, XP, quest credit, corpse loot rights and party
loot distribution. Explicitly marked shared quest/story targets may grant quest
credit to eligible participants without requiring a party; that does not imply
shared ordinary loot or XP. Party membership/range/level eligibility and party
snapshot timing need an explicit rule when party support is added.

Keep bounded server-owned ledgers, checked creature-life identity and an admitted
owner context. Resolve persistent character/party ownership through the trusted
admission/economy adapter. Define disconnect/rejoin, assist/healing, evade/reset,
owner death, abandonment and full-ledger cases. At a kill, convert rights into
idempotent reward/XP/quest/corpse facts before retiring the tag. Preserve the
victim-life deduplication identity and durable-before-success economy contract.

The current last-hit development fact extractor is not first-tag ownership.
Changing it requires a versioned replacement with competing-player tests; old
experiments and their grant records retain their original meaning.

### 3. Death and recovery

Use corpse recovery and authored graveyard/safe recovery points as the intended
overworld direction. Recovery must preserve exploration danger while offering a
bounded route back into mandatory solo story content. Story instances can have
explicit checkpoint recovery rules. Exact durability, repair cost, time penalty
and optional safe-resurrection rules require economy/content balancing.

Represent death, corpse ownership, recovery eligibility and resurrection as
server-owned life states. Use life-generation identity to reject stale combat,
interaction and recovery requests. Validate recovery against authoritative
positions and supported terrain; persist the relevant rights/checkpoint through
the existing asynchronous persistence boundary. NPC populations continue under
normal encounter/spawn rules while players recover.

Normal resurrection is restricted by combat policy; special combat resurrection
is an explicit ability exception. The existing three-second supported home
respawn remains a development fixture while recovery is built. Do not describe
that fixture as corpse-run implementation or replace it before the new slice
passes.

## Repo assessment

Current playable evidence: [classic controls](CLASSIC_MOVEMENT.md) and
[persistent targeting](CLASSIC_TARGETING.md), five of thirteen strict combat
checks. Current headless work: [weapon clock](CLASSIC_MELEE_CLOCK.md) and
[basic-melee World authority](CLASSIC_AUTOATTACK_AUTHORITY.md), with 153 core,
458 network passes / ten skips and 113 client tests. That profile still refuses
normal transport pending its explicit adoption.

| System | Implemented evidence | Required work / existing tasks |
| --- | --- | --- |
| Movement, selection and autoattack | Native classic movement/camera/selection; headless weapon clock and authoritative facing/reach/LOS checks | Secure autoattack transport/client adoption, ability-facing/swing rules, complete strict milestone; P2-04/05/11/12/14, P3-06/08 |
| Hit resolution | Bounded player ability hit/miss/deflect table and private server RNG; basic fixture always hits | Attack categories, defender positioning, crit/dodge/parry/block, mitigation and evidenced skill/level rules; P1-11, P3-04/06, P4-08 |
| Global combat state | Authoritative bounded life-bound membership and PL02 own-life indicator, hostility/encounter reasons, death/reset release, impaired encrypted transport and normal native checks | Mounting/rest/logout/assistance/regen consumers remain ungated; P3-04/06/07, P4-08 |
| Aggro, threat and groups | One NPC perception/chase/attack/leash/return, prepared navigation and qualified opt-in health/effect reset; retaliation reacts to hits | Bounded threat, assistance, encounter-group alerts, linked encounters, split pulls and broader evade policies; P3-07, P4-03/08 |
| Tags, loot, XP and quests | Bounded first-effective-hit ownership and contributions; schema-10 solo tag-owner XP/accepted-quest/training-offer routing, two-peer UDP and separate real SQL restart/replay; older modes retain last-hit | Party/range/shared-target eligibility, independent quest-sharing distributions and per-corpse loot rights; P3-09/10/11, P4-08/11 |
| Spawns and populations | One development creature has a fixed respawn and fresh entity life | Authored spawn slots/regions, min/max budgets, delay ranges, occupancy safety and rare/quest overrides; adaptive tuning later; P3-07/15/16, P4-07 |
| Navigation and collision | Static Avian collision, prepared tiled navigation and bounded asynchronous NPC path worker | More representative doors/caves/stairs/bridges, ranged LOS repositioning, unreachable cases, cross-region and scripted-transport tests; P2-07, P3-07/13, P4-03 |
| NPC roles and encounter composition | One melee wolf and prototype ability data | Archer/caster/healer/lookout/flee/elite roles through the same FSM/ability foundation, authored camp groups and patrol timing; P3-06/07/15/16, P4-08/10 |
| Levels and resources | Level-80 progression direction, bounded XP data and static vitals/resource pool | Combat stat progression, geography/difficulty, in/out-of-combat health and class-specific mana/resource regeneration; P3-04/06/10, P4-09 |
| Interrupts, pushback and CC | Cast interruption gates and bounded periodic/status effects | Explicit pushback, interrupts, action/movement restrictions, immunity/resistance and diminishing-return categories; P3-06, P4-08 |
| Pets, summons and healing threat | No completed pet/summon gameplay slice | Ownership/lifetime, commands, navigation, replicated state, contribution/credit and threat attribution; P3-06/07, P4-08/09/11 |
| Death and recovery | Authoritative health/death, home respawn, changed movement streams and stale-request rejection | Corpse/ghost/graveyard and solo-instance recovery, resurrection permissions, durability/economy policy; P2-12, P3-05/08/16, P4-08 |
| Shared simulation | Configurable authoritative fixed tick, secure Renet, prediction/reconciliation, interest/readiness and async PostgreSQL foundations | Continue measured real multiplayer, queue/recovery/region tests; no inferred MMO capacity from local proxy/entity counts; P1-05/07/08, P4-01/04/07/19 |

## Central contracts to add

**Combat membership:** use a bounded component/module inside the existing World,
with explicit reasons such as outgoing/incoming hostility, active threat and
assistance. Selection and an active swing timer alone are not the combat flag.
Release membership on defined encounter resolution/reset/death and tested grace
rules. Gate mounting, eating/drinking, flight initiation, normal resurrection and
safe logout through this common state. Boarding a ship does not clear combat.
Class/resource rules decide regeneration; stealth and special abilities declare
exceptions. This does not introduce a distributed service or awaited tick I/O.

**Spawn management:** stable authored spawn slots and integer region/chunk
addresses are distinct from individual entity lives and render residency.
Client culling/unloading never authorizes a respawn. Start with fixed bounded
spawn groups and safe delays; add regional thresholds and population/kill-rate
adjustment only after ownership, occupancy and load evidence. Validate supported
spawn terrain and avoid spawning directly on a player's authoritative position.
Named/rare/quest-critical rules are explicit data.

**Navigation and encounters:** retain separate collision, navigation and hit/LOS
queries, with nonblocking characters. Test moving targets, attack-distance
approach, blocked/unreachable paths, route return, doors/caves/elevation, zone
edges and scripted ship/elevator context as those systems become playable.
The same NPC framework handles Unmade enemies through abilities, encounter rules
and special states. Encounter composition is authored content alongside roads,
settlements, safe areas, terrain and quests; no separate AI engine is required.

**Control effects:** distinguish cast pushback, cast interruption, CC restrictions
and immunity/resistance. Repeated control uses explicit categories/reset timing,
with PvE/PvP exceptions recorded. Ability choices, healing/pet threat and resource
exceptions are data-driven additions to common authority.

## Execution and acceptance

1. Finish the current one-hostile strict combat slice: secure engage/cancel/state,
   manual ability facing, explicit swing interactions and normal native play.
2. Define combat membership and attack/credit interfaces alongside the mathematics
   baseline. Add a competing two-player creature test: first contribution, final
   hit by another player, owner disconnect/death, reset and fresh entity life.
   Prove one intended set of XP/quest/loot rights and durable grants.
3. Add a small authored encounter/spawn group: a melee guard, a patrol and an
   assisting pair. Prove bounded alert/threat/leash/reset, safe respawn and patrol
   recovery. Expand ranged/caster LOS behavior through the same ability system.
4. Implement one authored graveyard/corpse recovery and a solo-story checkpoint
   policy. Test stale life requests, supported recovery positions, combat gates,
   disconnect/restart and economy idempotency before replacing home respawn.
5. Extend class resource, interrupt/CC, group/healing/pet and encounter roles.
   Validate ordinary enemies before advanced boss/faction scripting. Add dynamic
   population tuning and large events after representative multiplayer load tests.

These slices overlap according to actual prerequisites; the imported list order
is not a rigid gate. Complete the existing combat slice before a whole-zone
content push. Preserve working experiments until replacements pass. No new
engine, renderer, physics library, network transport or sharding design follows
from these notes. Exact Classic numbers and new capacity claims need evidence.
