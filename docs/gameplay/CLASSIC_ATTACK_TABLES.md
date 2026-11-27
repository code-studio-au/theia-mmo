# Ordinary melee outcomes — normal client and native adoption qualified

P2-11 / P2-12 / P3-06 / P3-08 / P3-09. The ordinary-outcome World, CE07,
Renet and normal application now pass 224 core / 556 network (ten skips) /
118 client / 41 Python checks, formatting, denied-warning workspace/all-target
Clippy and four release binaries. Latest playable source/art/binaries/proof:
`build/classic-white-client-qualified-v1-20261101`. Earlier physical and transport
checkpoints remain preserved. Static level-1 templates are development data;
this does not establish every Classic formula.

## Normal application and native acceptance

`development-gameplay-classic-outcomes.json` is a schema-7, 17-role exact-byte
manifest that explicitly selects the bounded ordinary profile. Individual CLI
selection requires its physical context. Both applications bind the same profile
identity before connecting; malformed references and unsupported timed weapon
specials fail before play. Generic ability outcomes remain their separate policy.

The client consumes resolved ordinary facts without authorizing HP. Avoided swings
still animate once; creature flinches require actual health lost. Outgoing contact
numbers are white, incoming red, avoidance gold; critical text is larger. Partial
block, glancing and crushing have explicit labels. Current-life filtering suppresses
stale own-life cosmetics while preserving reliable batch continuity. Unit checks
cover all outcome labels and resolved-versus-health-lost overkill values.

Six native runs use normal OIDC login, PostgreSQL admission/persistence and owned
X11 mouse/keyboard input: controls/casting and unmodified wolf fights at actual
1920x1200, 3440x1440 and 1280x720 (A/D strafe preference). All pass with zero GPU
diagnostics, combat event gaps or input backpressure. Device: AMD Radeon RX 7900
XTX / RADV NAVI31, Vulkan, Mesa 26.2.6 kisak. A physical display was connected;
3440x1440 fullscreen passed, the other sizes used windows. This is Linux execution,
not native Windows, a performance capacity benchmark or a full combat milestone.

Controls data isolates input with 1 m perception and 0.1 m/s patrol/chase; ordinary
damage/outcome definitions stay unchanged. Production runs use unmodified gameplay
profiles and demonstrate passive perception, chase, incoming attacks and wolf death.
Each graceful server summary confirms both ordinary directions, one base/outcome
draw per attempt, no legacy physical RNG draws, one durable XP commit and one loot
offer. Loot claiming is not tested. Native samples include hit, miss, dodge and
critical glyphs; current wolves cannot parry/block and equal level-1 combat has
no glancing/crushing. Those outcomes retain numerical, World and codec evidence,
not a claimed native encounter. Proof roots: `build/classic-white-native-v1` and
`build/classic-white-native-v2`, copied into the checkpoint without private runtime
data, database files or tokens.

Actual gear/class/level stat derivation, special/spell starting damage and roll
categories, absorption, resistance, control states and continuous remote creature
heading remain follow-on work. P3-17/18 remain unspecified. Historical prerequisite
records follow and describe their acceptance at that time.

## Contract and evidence

`damage::white_melee` distinguishes miss, dodge, parry, block, glancing,
critical, crushing and normal hit. One supplied checked roll selects exactly one
outcome in a finite 10,000-value domain. Individual nominal rates are bounded
at 10,000 basis points. Eligible buckets consume the domain in that order;
normal hit receives the remainder. Totals above 100% are intentional: later
outcomes are clipped by earlier intervals. Disabled buckets consume no interval;
they do not trigger a reroll or preserve a hole in the table. Effective weights
remain observable separately from nominal rates.

The order is a reference choice independently expressed from the pinned
CMaNGOS ordinary-melee die definition and resolver. This emulator source is not
proof of Blizzard's exact probability or random-number implementation. No source
code, helper, data table or dependency is imported. A half-open zero-based roll
is Theia's explicit numerical contract, not an assertion of identical PRNG draws.
[Reference outcome order](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.h#L551),
[ordinary-melee resolver](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L2491).

The authority must establish eligibility from actual actor/defender categories,
facing, capabilities, current state, usable equipment and level/skill rules.
The primitive accepts explicit facts and does not invent those rules. In
particular, block requires separately established eligibility; a wolf does not
receive a shield merely because the numerical enum contains block. Passive dodge
outcomes are separate from the deferred active dodge movement mechanic.

Original Classic measurements distinguish ordinary white single/dual-weapon
attacks, skill differences, measured values and estimated extensions. They do
not establish one universal table for all specials, spells, PvP or low levels.
[Original attack-table research](https://github.com/magey/classic-warrior/wiki/Attack-table).
Blizzard's reference comparison tested specific levels and conditions; numerical
probabilities outside those conditions need an explicitly evidenced policy.
[Archived original Blizzard QA](https://www.bluetracker.gg/wow/topic/us-en/185675-bug-hit-tables/).

The existing generic ability `hit_outcomes::Table` remains unchanged. Its deflect
fixture is not renamed into parry/block and its ineligible-bucket contract is not
silently changed. The new numerical table supplies neither damage multipliers
nor block amounts, shield pools, skill formulas or randomness. Sitting/stunned
special cases and mixed-school attacks need separate explicit policy before use.

## Acceptance and next integration

Six numerical tests exhaust the whole roll domain for exact frequencies, check
every interval boundary, disable eligibility, exercise critical push-off and
reclaimed capacity, bound maximal rates and refuse malformed rates/rolls.
All 195 core tests, formatting and denied-warning workspace/all-target Clippy
pass. Exact component source/proof is preserved at
`build/classic-white-melee-qualified-v1-20261101`. No native check is required
for this inactive pure numerical module.

Next establish a bounded ordinary-physical category/eligibility and rate profile,
including actual player/NPC types and defender-facing facts. Preserve old data
and codecs until outcome-aware World facts, failure/RNG invariants, encrypted
UDP and normal native presentation pass. Do not select spell/ability roll rules
from this white-melee table. Keep P3-17/18 unspecified and all spatial rules metric.

## Standing physical eligibility prerequisite

`damage::white_melee::eligibility` independently expresses a bounded reference
policy for players and uncontrolled creatures at levels 1–63. It checks actual
capability facts separately from supplied nominal chances: player rear attacks
exclude dodge; parry/block need the defender's front; player parry needs a usable
melee weapon and block a usable shield. Disarmed defenders cannot parry. Lost
reaction or a nonmelee cast suppress defensive avoidance. Critical capability
remains separate. Glancing is restricted to player-to-creature with defender
level above 10; crushing to uncontrolled-creature-to-player capability. Rate and
level/skill probability calculations must still decide whether their chance is
positive. This is an emulator reference policy, not a complete Blizzard rule set.
[Reference eligibility](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L2803).

Nonstanding defenders fail explicitly instead of guessing the special sitting
critical branch. Pets, charms, mixed schools, spells and specials require their
own policies. No live authority, NPC facing, profile or wire adoption is implied.

Six further checks cover player/NPC rear attacks, equipment and disarm,
reaction/casting state, distinct attacker categories, the level-10/11 glancing
gate and unsupported levels/standing states. All **201 core tests**, formatting
and workspace/all-target denied-warning Clippy pass. Exact candidate source/proof:
`build/classic-white-eligibility-qualified-v1-20261101`. The physical playable
checkpoint remains unchanged.

## Authoritative creature-facing prerequisite qualified

`Facing::towards` computes checked horizontal global bearings in the existing
Y-up, yaw-zero-towards-minus-Z convention; coincident XZ returns no bearing.
The single prepared patrol now retains an authoritative facing. Its initial and
respawn facing follows the first checked route segment. Successful movement
updates facing from actual displacement, and waits preserve it. Pursuit attack
holds face the exact selected current-life target while remaining stationary.
A failed movement/publication does not commit a new facing. `patrol_facing`
requires the exact current creature incarnation and an existing target; old or
dead keys do not expose the replacement's orientation.

This is a prerequisite for defender-relative eligibility, not new avoidance
behavior. It does not change player facing, wire data, cosmetic heading or the
existing physical damage rules. Legacy retaliation without pursuit retains
its last patrol heading; it must not be treated as complete attack-facing policy.
Coincident actors retain the previous heading; eligibility still uses independent
range/LOS/life checks. Future live table adoption must version the gameplay
profile and establish complete category/rate rules.

Five focused network checks pass, including the two new fixed-step orientation
checks at 20/30/60 Hz; existing death/respawn and actual static-collision tests now
also assert orientation retirement/reset/hold. Full regression passes **202 core / 516 network (ten skips) / 115 client**
tests, formatting and workspace/all-target denied-warning Clippy. Exact source/proof:
`build/classic-creature-facing-qualified-v1-20261101`. The previous playable checkpoint
remains frozen.

## Rate selection evidence before live adoption

The pinned emulator is a reference for structure, not an automatic authority for
all rates. Its miss branch needs separate comparison against Blizzard's 2019
reference result before adoption. Prefer the original QA's 8% base miss against
+3 creatures under its stated max-skill conditions, and keep the first-hit-bonus
suppression separate from base miss. The original research further distinguishes
skill breakpoints, dual-weapon penalties and a low-level creature adjustment;
these are not established by merely choosing a seven-bucket table. The current
level-1 fixture must not silently receive endgame chances or glancing eligibility.
Use independently tested, explicitly bounded policy inputs and record any
estimated extension. A complete rate formula and skill/class data are still
pending; neither this numerical table nor creature facing enables them.

## Candidate ordinary player/creature miss calculation

`white_melee::miss::player_creature_research_v1` now separately reports creature
defense (five times level), signed skill deficit, raw and nonnegative normal base,
dual-weapon penalty, hit suppression, effective hit bonus and final miss chance.
It accepts explicit authoritative weapon skill and bonus hit rather than deriving
them from character level or owned equipment. Bounds are creature levels 1–63,
weapon skill 0–1000 and bonus hit 0–10000 basis points. These are engineering
bounds, not a claim that every corner has been measured in Classic.

The policy follows the original research's skill breakpoint, target-below-level-10
adjustment and dual-weapon model, separately applying suppression to bonus hit.
The confirmed +3 example retains 8% base miss at zero bonus, ignores the first
1% bonus and reaches zero at 9% bonus. A five-skill bonus reaches the separate
6% example without that suppression. Research interpolation, low-level behavior
and extreme skill gaps remain estimates rather than Blizzard-confirmed coverage.
[Original measured research and estimated formulas](https://github.com/magey/classic-warrior/wiki/Attack-table#miss),
[Blizzard QA clarification](https://www.bluetracker.gg/wow/topic/us-en/185675-bug-hit-tables/).

Theia's numerical contract keeps the signed base contribution through the final
calculation so early flooring cannot erase reductions before the dual penalty.
The low-level adjustment precedes that penalty, and the final probability is
bounded to [0,10000]. These explicit engineering/rounding decisions need further
empirical comparison at extreme gaps before a full-fidelity claim. The adjustment
is exact in whole basis points for the chosen integer rates. No special/spell,
PvP/pet, mixed-school or aura rule is inferred.

Six checks cover the confirmed +3 and skill-bonus examples, the 10/11 breakpoint,
dual-wield 27%/28% hit cases, low-level adjustment, large signed skill/hit values
and malformed inputs. All 208 core tests, formatting and workspace/all-target
denied-warning Clippy pass. Exact source/proof:
`build/classic-white-miss-qualified-v1-20261101`. The live
profile still has no skill/rate adoption; the playable checkpoint is unchanged.

## Candidate remaining chance arithmetic

`white_melee::chances` adds independently bounded normal-physical reference
helpers for player/creature defensive rates, melee-class glancing, creature
crushing and isolated critical adjustments. Explicit zero base defenses remain
zero. Category/capability gates remain separate; these helpers do not grant a
wolf parry/block or derive equipment and class stats. The critical level helper
covers same/higher creature targets only and excludes the separate aura modifier;
lower targets fail rather than receiving guessed bonuses.

Original QA's equal-level dodge and +3 dodge/parry/critical examples are checked
separately from the emulator reference's other skill factors and crushing gates.
Research-based glancing chance caps attacker skill by level, while damage
reduction must later use its own uncapped skill input. These are candidate
policies with the evidence limits above, not a universal complete attack table.
[Original QA evidence](https://www.bluetracker.gg/wow/topic/us-en/185675-bug-hit-tables/),
[reference defensive/chance calculations](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L2971),
[original glancing research](https://github.com/magey/classic-warrior/wiki/Attack-table#glancing-blows).

Eight new checks cover example probabilities, defensive skill factors and the
parry breakpoint, disabled bases, capped glancing skill, crushing deficit/capped
defense, isolated critical suppression and malformed/extreme bounds. All **216
core tests**, formatting and denied-warning workspace/all-target Clippy pass.
Exact component source/proof: `build/classic-white-chances-qualified-v1-20261101`.
No live rate selection or damage multiplier adoption is implied. The playable
checkpoint remains `build/classic-damage-client-qualified-v1-20261101`.

## Candidate selected-outcome physical amounts

`white_melee::impact` applies the existing normal physical armour reference,
then the selected ordinary outcome and health clipping. Miss/dodge/parry produce
zero damage with no applied armour stage. Contact retains the explicit armour
breakdown; ordinary critical doubles post-armour damage, crushing uses a floored
3/2, glancing uses a supplied checked whole-percent draw, and block removes a
bounded flat amount capped by the remaining hit. A full block or small glancing
hit can reach zero: the normal-contact minimum is not reapplied afterward.
Critical modifiers, absorption pools, resistance and mixed schools remain outside
this first policy. Eligibility and outcome selection still precede this API.
[Reference mitigation/outcome ordering](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L1786).

The melee-class glancing range uses uncapped skill for damage, unlike chance.
A supplied draw must lie within the inclusive checked interval. The chosen
integer percent/floor boundaries are explicit reference engineering policy,
not a claim of identical Blizzard floating-point or random sampling behavior.
Caster-class reductions require a separate policy. Damage beyond the numerical
bound fails before health clipping, even when the victim has one HP.

Seven new checks cover every selected outcome, mitigation-before-scaling rounding,
health loss/overkill, zero/full-block cases, glancing skill bonuses and bounded
ranges, and malformed levels/defenses/amounts. All **223 core tests**, formatting
and denied-warning workspace/all-target Clippy pass. Exact source/proof:
`build/classic-white-impact-qualified-v1-20261101`. World and presentation still
use the previous normal-contact physical profile; the latest playable checkpoint
remains `build/classic-damage-client-qualified-v1-20261101`.

## Candidate profile binding qualified

`simulation::combat::white_melee::Profile` schema 1 now binds an explicit ordinary
outcome profile to the complete physical context. It requires matching definition,
stat and creature IDs, a named reference ruleset, complete unique weapon-skill/hit
bindings and a named melee glancing policy. Current scope is single main-hand,
standing players and one uncontrolled creature, with creature skill/defense fixed
to five times its declared level. Lower-creature critical adjustment is explicitly
unsupported; it needs another policy rather than a guessed bonus.

`development-white-melee.json` is outside gameplay manifests. Its level-1 skill
and base chance templates are development fixtures, not derived race/class/gear
stats. The wolf has no parry/block, and its level excludes glancing; player
parry/block are also disabled. Incoming ordinary miss uses its separate reference
skill factor. Explicit actor state supplies front/reaction/casting and usable
weapon/shield facts; profile capabilities and percentages stay separate.

Checked supplied draws evaluate either direction through the same selected-outcome
physical arithmetic. Missing/out-of-range or unused glancing draws fail. Current
victim health must be positive and within the bound profile. Startup validates
maximum possible critical/crushing damage before a future attack can be admitted.
Identity binds exact profile bytes and the full physical context, separately from
existing World identity until explicit activation.

Nine new profile checks plus the incoming-miss numerical check pass: **224 core
and 197 network-library tests**, formatting and denied-warning workspace/all-target
Clippy. This includes existing library UDP regressions, but no new-profile World,
wire, UDP or native adoption is established. Exact component source/proof:
`build/classic-white-profile-qualified-v1-20261101`. Continue explicit World
activation, transactional rolls/facts, versioned codec and actual encrypted UDP,
then normal application/native acceptance. Preserve the previous playable
checkpoint throughout.

## Explicit World activation qualified

`World::enable_white_melee` now requires inactive classic physical authority,
one valid pursuit configuration and supported ability timing before admission.
It wraps physical identity and retains the old normal-physical profile separately.
The first scope uses standing actors, no control/disarm status system and no
usable offhand shield. Defender-facing checks use current server global positions
and the authoritative patrol/player facing. Actual cast timing suppresses player
defensive avoidance during a nonzero cast; instant sword abilities do not create
that casting restriction. Cast-time melee specials are refused until their
casting policy is explicit. Reset/blocked phases pause swings before rolling;
an explicit Evade result/display remains separate work.

An independent server-owned ChaCha stream previews base/outcome and conditional
glancing draws on a clone. Only a successful health/life/event operation commits
the stream and checked NPC sequence. Death and provocation deadlines are checked
before mutation. Event capacity includes pending death/cast/queue cancellations.
Valid avoided swings may provoke without damage, through a previewed engagement
policy; this is the existing single-target policy, not a complete threat table.
`WhiteMelee` facts carry current life namespaces, attack/sequence, defender state,
draws, selected outcome amounts and remaining health. Existing codecs reject them.

Eleven World checks plus three counter/stream checks now pass, alongside nine
profile checks: fixed-step outgoing/incoming resolution at 20/30/60 Hz, avoided
provocation outside passive perception, facing/readiness/gear/replay gates,
event pressure, failed death/provocation deadlines, stale lives, preview replay,
NPC sequence exhaustion, XP/reward handoff and retirement, legacy codec refusal,
conditional glancing/crushing and actual casting/committed-gear state. Seed
selection for rare reference outcomes exists only in tests, never commands or
normal profile flags. Full regression passes **224 core / 539 network (ten
skips) / 115 client tests**, formatting and denied-warning workspace/all-target
Clippy. Exact source/proof: `build/classic-white-world-qualified-v1-20261101`.
New-profile wire, actual UDP, normal application and native adoption remain pending.

## Explicit CE07 result codec qualified

`Batch::encode_white/decode_white` now require the complete ordinary-melee profile
and existing schema-7 ability/weapon bindings. CE07 binds its exact context hash.
Each fixed 82-byte fact carries attack ordinal, actor/target namespaces and life
generations, sequence, base/outcome/glancing draws, seven defender-state bits,
positive pre-hit/remaining health and checked global position. Full selected
outcome and damage breakdown are reconstructed through the bound profile; the
encoder independently checks the supplied fact before serializing it. This is
profile/math consistency for authenticated server observations, not authorization
for a client to change health or proof of an unknown server RNG seed.

Reserved state bits, unsupported state/draw combinations, bad roll domains,
unknown ordinals, invalid life roles/identity/sequence, duplicate source-life
sequences, inconsistent remaining health, nonfinite positions and malformed byte
budgets fail. CE07 can share existing ability/death events but rejects old fixed
melee and normal-physical facts. CE02–CE06 reject its layout. Twelve full facts
use **1045 bytes**, preserving the existing 1200-byte and twelve-event limits.

Nine new checks pass, alongside **224 core / 220 network-library tests**,
formatting and denied-warning workspace/all-target Clippy. Exact source/proof:
`build/classic-white-codec-qualified-v1-20261101`. This includes existing library
transport regression, but CE07 Renet profile selection, private victim routing,
actual impaired UDP and normal/native adoption are still next. Keep the qualified
normal-physical playable checkpoint unchanged.

## Renet selection and owner routing qualified

The publisher now selects CE07 only for explicit ordinary authority. Client
`MeleeActions.white` requires the matching physical context and complete weapon
bindings before network I/O; existing classic/life/inventory/schema-7 gates remain.
The shared private-victim helper routes incoming physical/ordinary facts only to
the exact victim. Visibility filtering precedes per-recipient batch sequencing,
so other peers retain contiguous streams. Full or closed presentation queues
continue to disconnect rather than silently dropping facts.

Three focused checks pass: actual encrypted two-peer World/driver traffic through
loss/jitter/duplicate/reordering at 20/30/60 Hz, blocked presentation queue, and
missing/mismatched transport bindings before I/O. Each received fact reconstructs
through its bound profile and actual pre-hit health; avoided hits are allowed to
leave health unchanged. The second peer sees public swings and no private incoming
fact or health loss; subsequent public swings preserve its sequence. Private base,
outcome and conditional glancing counters remain separate. Admission/inventory
ownership here is synthetic; no new OIDC, SQL durability, native rendering or
MMO-capacity claim follows. Full regression passes **224 core / 551 network
(ten skips) / 115 client tests**, formatting and workspace/all-target denied-warning
Clippy. Exact source/proof: `build/classic-white-transport-qualified-v1-20261101`.

The current wolf template has no parry/block, and the normal client already turns
its wolf toward the local victim during a windup. Continuous authoritative
creature-heading replication/presentation remains a prerequisite before enabling
position-dependent defenses on additional enemy types. The internal server-facing
getter alone does not establish that visual agreement for other peers.
