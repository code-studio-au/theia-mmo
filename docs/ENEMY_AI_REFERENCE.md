Related planning: [broader Classic world systems](CLASSIC_WORLD_SYSTEMS_REFERENCE.md)
connect enemy AI with combat membership, tags/credit, population management,
encounter composition, resources and recovery. Its task mapping and repo matrix
separate implemented behavior from planned systems.

# Enemy AI, aggro, pursuit, patrol and reset reference

Formula/units amendment, 1 November 2026: follow
[Classic formulas with documented evidence](CLASSIC_FORMULAS.md) wherever possible;
keep Theia metric, with 1 world unit = 1 metre. Earlier illustrative prototype
values remain provisional until replaced and verified.

**User-supplied research/design reference · 1 November 2026.** This records the
attached “WoW Classic — Enemy AI, Aggro, Chasing, Patrols & Reset Behaviour” notes.
It informs future slices alongside [THEIA-COMBAT-001](COMBAT_TARGETING_MOVEMENT_SPEC.md).
The attachment's recommendations and external-game claims are reference material,
not evidence that Theia already implements them. No new exact balance constants
are locked by this import. The controls/targeting/autoattack milestone stays first.

## Intended ordinary outdoor loop

Idle/wander/patrol → acquire eligible hostile → pursue with navigation → face
target and attack → pursue again as necessary → evade/return → resume patrol.
Death/respawn are explicit lifecycle transitions. Detection, threat selection,
navigation, facing, attack eligibility and damage resolution remain separate,
server-owned concerns. NPCs remain physically nonblocking.

Player facing is manual. NPC facing may turn rapidly toward its own combat
target; it never follows a player's camera. Status effects can prevent movement,
turning or attacks according to their rules. Prefer extending the existing small
finite-state controller; a behavior tree is unnecessary for this first encounter.

## Reference requirements and current gaps

| Area | Reference direction | Current Theia behavior / next gap |
| --- | --- | --- |
| Disposition | Hostile detects automatically; neutral requires provocation; friendly does not initiate hostility; scripted activation is explicit | One configured hostile wolf; general disposition/faction rules remain |
| Detection | Level-dependent radius, separate LOS, eventual stealth/proximity/orientation modifiers; ordinary detection is not just a narrow cone | Fixed 12 m perception with static LOS; level/disposition modifiers remain |
| Threat | Per-NPC bounded threat table for damage, healing, special threat, taunts and reductions; hysteresis avoids constant switching | Current single acquired player/life is retained; multi-player threat is absent |
| Melee pursuit | Navigate around static obstacles, close to reach, face target, attack and resume chase when needed | Navigation-backed chase, checked 3 m melee, loss-of-sight grace and return exist; combat-facing/weapon timing need the new shared rules |
| Ranged pursuit | Preferred distance band; reposition to restore LOS, chase only as necessary | Player travelling magic exists; ranged NPC behavior remains |
| Outdoor leash | Configurable soft disengagement timer plus hard pursuit boundary; successful combat may refresh only the soft allowance | 35 m captured-anchor hard leash; 30 s soft allowance now refreshes on authoritative visible melee reach; ranged damage/threat refresh remains |
| Evade/reset | Stop attacks, clear target/threat, reject new aggro and damage, clear combat debuffs, restore health, navigate home and resume patrol | Return prevents attacks/reacquisition and restores route progress. Ordinary/typed combat already rejects return damage; legacy generic modes remain damageable. Schema-10 solo credit additionally clears claims/membership on Return/Blocked. Health/debuff reset and full evade are missing |
| Home/patrol | Store spawn and active patrol anchor separately; stationary, wander, waypoint, loop, back-and-forth, group and scripted routes | Saved waypoint progress and captured patrol anchor exist; broader route traits/group/scripted behavior remain |
| Social assistance | Separate assist radius, affiliation/group, helper budget and explicit exclusions | Not implemented; use bounded eligible alerts, not unbounded propagation |
| Fleeing | Optional archetype trait; low-health flee toward eligible assistance, interruptible by applicable crowd control/death | Not implemented; not universal to all enemies |
| Supporting rules | Distinct detection/attack LOS; status-driven stun/root/fear/slow; combat regeneration; death/loot/respawn | Pieces of LOS, damage, periodic status and death/respawn exist; generalized AI/status/regeneration/loot rules remain |
| Dungeons/bosses | Encounter-specific boundaries and reset policy | Do not reuse outdoor leash settings indiscriminately; later encounter scope |

[Existing pursuit behavior and evidence](CREATURE_PURSUIT_NATIVE.md) remain the
implemented baseline. A completed return-to-patrol experiment does not imply
threat tables, evade immunity, health reset, social aggro or population capacity.

## Research values versus Theia policy

The notes describe an illustrative aggro approximation of
`20 + (enemy level − player level)`, clamped roughly to 5–45 **WoW yards**.
They also describe 110% melee / 130% ranged threat switching. These are supplied
research claims, not independently verified exact WoW algorithms or adopted
Theia constants. Do not copy yard values into metre-based world data silently.
Stealth, vertical distance and creature-specific modifiers need explicit rules.

The suggested 60 m hard leash and 12 s soft timer are explicitly illustrative
Theia tuning examples. Precise detection radii, threat thresholds, leash values,
refresh events and NPC attack arcs should be configured and validated against
the first outdoor encounter. Existing 12 m / 35 m / 30 s profiles remain historical
development values until a replacement passes.

## Melee engagement correction — 2 November 2026

P3-07 / P3-06: the previous acquisition-relative 30-second cutoff could reset a
wolf during an ordinary stationary fight. The pursuit policy now refreshes that
soft allowance whenever its retained live target has server-verified visible
melee reach. Misses, dodges and slow weapon swings do not interrupt this contact.
After contact ends, the allowance expires 30 seconds after the last contact.
The original captured anchor and 35 m hard boundary never move or refresh.
Blocked navigation, target death/disconnect/life replacement and the two-second
LOS grace still take precedence. Return travel retains its separate bounded
30-second deadline. This is an explicit temporary Theia rule; it does not claim
to reproduce Classic's exact leash algorithm. Ranged damage/threat refresh,
full evade/reset and multiplayer threat remain separate work.

Acceptance covers 90 seconds of continuous contact and exact post-contact
expiry at 20/30/60 Hz, hard-boundary/LOS/blocked-route precedence, and an actual
35-second fixed-step World fight at each rate. Validation results are recorded
in STATUS; this paragraph describes source behavior, not a native runtime claim.

## Small slices after the player combat foundation

1. **P3-07 / P3-06:** retain the working hostile/NPC lifecycle; add explicit
   disposition and level-based detection, fast NPC facing and weapon-timed melee
   using the shared combat contract. Prove hostile acquisition, neutral
   provocation, friendly non-acquisition and no camera dependency.
2. **P3-07 / P4-08:** bounded per-NPC threat with deterministic switching; explicit
   LOS, soft/hard leash and full evade/reset. Prove two-player threat hysteresis,
   no damage/aggro while returning, cleared debuffs/threat, full health and no
   immediate chase/reset loop. Combat refresh must not bypass the hard boundary.
3. **P3-07 / P4-03:** expand patrol traits and bounded social assistance. Prove
   interrupted route recovery, independent spawn/patrol anchors, group exclusions
   and fixed work/queue limits under repeated alerts.
4. **P3-06 / P3-07 / P4-08:** ranged positioning/casting, optional flee/help traits
   and status-driven AI interruption. Prove obstacle-driven LOS repositioning,
   finite spell reach and prevention of assistance by applicable crowd control.
5. **P4-08 and later encounter work:** dungeon/boss boundaries and group-scale
   combat acceptance. Keep these separate from the one-hostile outdoor milestone.

Suggested creature data groups are identity/level/faction/disposition, detection,
movement/navigation, spawn/home/region/patrol anchor, route/waits, weapon/abilities,
facing/arc, threat/taunt, assistance, pursuit, fleeing, reset and respawn. Introduce
fields when the corresponding slice needs them rather than building a complete
unused AI framework. P3-17/P3-18 remain unspecified.
