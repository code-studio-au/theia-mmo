# Ability damage — starting values, authority and client adoption

P2-11 / P2-12 / P3-06 / P3-07 / P3-09. The latest qualified playable checkpoint
is `build/classic-ability-client-qualified-v1-20261102`. The normal schema-8 bundle
uses the versioned special/Arcane authority and presentation described below.
Earlier sections retain dated numerical/World/codec prerequisite evidence; older
manifests and preserved builds retain their fixed effects.

## Explicit numerical contract

`damage::ability_starting::WeaponSpecial` keeps the weapon's base range and clock.
Only the AP contribution can select an explicit normalized scaling speed. The
candidate fixture-v1 order is:

`(weapon roll + AP / 14 × selected seconds) × weapon percentage + flat bonus`.

Fractions remain exact until the caller chooses a named rounding boundary. The
weapon/AP amount, scaled amount and final sum each obey the existing engineering
limit of 1,000,000. Roll, nonnegative AP, scale and flat bonus are bounded. This
helper supplies no hit outcome, armour, RNG, equipment ownership or HP mutation.
It does not infer normalization from animation, current attack speed or cast time.

The pinned CMaNGOS reference distinguishes actual and normalized AP speeds and
weapon categories. It is emulator provenance, not proof of every Blizzard special.
[AP speed reference](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L9705).
Theia independently expresses the arithmetic; it imports no emulator code or data.
This slice accepts an explicit normalized duration without inventing an item class
system. Individual special formulas and whether their bonus is scaled require
separate data and evidence before adopting real abilities. Negative stat/bonus
modifiers, offhand, ranged weapons, aura modifiers and spell-specific exceptions
are outside this first contract. Rational intermediate values are a Theia policy,
not a claim of bit-identical Classic float or integer boundaries.

Spells use the existing checked base range plus explicit bonus coefficient.
There is no automatic cast-time/3.5 derivation: ranks, low-level penalties,
direct/periodic splits and spell-specific exceptions are not established here.
Neither kind uses the ordinary-melee attack table merely because that table exists.

## Bounded candidate recipes

`damage::ability_starting::catalog` checks at most 32 recipes / 16 KiB, strict
schema 1 and `ability_starting_fixture_v1`. It requires the exact logical weapon
catalogue ID and known schema-7 direct-damage abilities, with exactly one recipe
per direct-damage ability. Main-hand specials reference a declared weapon attack,
require its exact main-hand item and direct delivery. A known physical weapon
special cannot silently select the spell recipe. Unknown/missing/null fields,
duplicate/missing abilities, invalid references and unsupported scales fail.
Combined maximal damage can be checked against the supplied server-template stats
before any future authority is enabled. Logical references do not replace exact
byte identity or actual committed equipment validation.

`content/gameplay/development-ability-starting.json` is deliberately inactive and
absent from gameplay manifests. Its Sword Slash uses 100% weapon damage plus 5;
Arcane Bolt uses base 18–22 and coefficient 60%. These are Theia development
fixtures, not canonical Classic spells/ranks. At existing AP 28, sword roll 11
starts at exactly 20.8 before armour/outcomes; at existing spell bonus 0, Arcane
Bolt starts at its base draw. The catalogue does not change existing fixed effects.

## Acceptance and next slice

All 231 core / 556 network (ten skips) / 118 client checks, formatting and
workspace/all-target denied-warning Clippy pass. Exact candidate source/proof:
`build/classic-ability-starting-qualified-v1-20261101`. Seven focused tests check AP-only normalization, unchanged ordinary swing data,
percentage/flat order, exact fractional intermediates, all three overflow stages,
weapon/spell stat isolation, schemas and strict nested fields, exact references,
coverage/duplicates and numerical/byte/count limits. This inactive numerical/content prerequisite requires no native GPU
check; normal application behavior and the playable checkpoint are unchanged.

Next bind recipes and static spell bonus in an explicit opt-in authority profile;
preview/commit RNG only at an eligible effect boundary, preserve source/target life
and projectile semantics, emit separately typed facts, bind exact identity and
version the codec before normal/native adoption. Physical specials need their own
eligible outcome policy; spells need an explicit school, outcome and resistance
policy. Preserve current fixed and ordinary profiles until replacement acceptance
passes. Derive real gear/class/level stats later. P3-17/18 stay unspecified and
all spatial quantities remain metric.

## Special outcomes and impact prerequisite — 2 November 2026

`damage::melee_special` now supplies separate ordered miss/dodge/parry selection
and conditional critical draws for unblocked, standing creature defenders. It
requires caller-supplied effective rates and validated player/main-hand/physical
categories; it does not silently apply the ordinary skill/rate policy. Current
reaction, nonmelee cast, rear and disarm facts gate avoidance. A creature still
can dodge from behind; parry needs its front. Blocking defenders and ability-specific
bypasses are explicitly unsupported. Eligibility for sitting, PvP, pets, ranged,
magical melee and control effects remains separate. Active dodge stays deferred.

`damage::ability_impact` floors starting damage, applies the selected physical
special critical multiplier, then armour; magic has an explicitly named
zero-resistance path and its own multiplier. The finite numerical policies retain
health lost and overkill separately, bound critical damage before mitigation,
and preserve zero/positive-hit minimum behavior. These are independently expressed
pinned-emulator reference boundaries, not proof of identical Blizzard floats.
[Special selection and positional eligibility](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L2604),
[ability critical before armour](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L1654),
[reference multiplier and conversion](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L3268).
No code or emulator data is imported.

A useful order fixture is exact starting 20.8: floor to 20, physical special
critical to 40, level-1 armour 100 gives 33. The ordinary reference gives 32
because its armour boundary precedes critical. Magic starting 21 gives critical
31 under the no-resistance path. Block, absorption, nonzero school resistance,
crit-modifying auras and ignore-armour specials are not implemented by this helper.
All dimensions remain those of existing metric world rules; these arithmetic
ratios and probabilities are dimensionless.

Six new tests cover the full 10,000-value avoidance domain, interval boundaries,
clipped/disabled buckets, missing/extra conditional draws, defender capability,
critical order, fractions, health clipping, invalid context and maximal values.
All **241 core tests**, formatting and workspace/all-target denied-warning Clippy
pass. Source/proof: `build/classic-special-impact-qualified-v1-20261102`.
Network/client baselines remain the preceding 556 (ten skips) / 118 checks and
were not rerun for inactive pure numerical helpers. Rendering and live abilities
are unchanged; no new native acceptance is claimed. Next bind exact profile
bytes and checked ability/weapon/creature references, then authority/codec/UDP/native.

## Exact profile binding prerequisite — 2 November 2026

`simulation::combat::ability_damage::Profile` now binds starting recipes, policy
and exact ability bytes together with the complete ordinary/physical context.
It checks coverage, known recipes/categories, maximal combined damage and every
reference before evaluating supplied draws. Loading it does not activate World,
transport, launcher or presentation behavior. Source/proof:
`build/classic-ability-profile-qualified-v1-20261102`.

The explicit `ability_damage_reference_fixture_v1` policy separates the stateless
reference arithmetic from Theia balance data. Initial Sword Slash uses supplied
effective miss/dodge/crit 5% each, no parry, 100% weapon plus 5. These rates are
fixtures, not a newly evidenced universal Classic special skill formula. It
requires an instant, explicitly faced, direct main-hand ability within the initial
3 m development limit. It cannot invent an NPC dodge/parry capability absent in
the bound ordinary context. Blocking defenders are refused until a distinct
full/partial special-block policy is qualified. Defender-relative facing uses
supplied current authoritative facts; attack-facing and gear authority remain
separate execution prerequisites.

Arcane Bolt declares zero-resistance Arcane, base 18–22, 60% coefficient, 0 bonus
spell damage, 0 bonus hit and 5% crit. Spell bonus is an explicit separate temporary
server template; it does not alter ordinary AP or grant derived equipment stats.
The first zero-resistance policy requires equal-level actors, avoiding an implicit
claim that higher-level NPC resistance is supported. Nonzero resistance, binary
spells, immunity and aura exceptions remain unsupported. Damage schools and
coefficients are author declarations for Theia development abilities, not canonical
Classic spell/rank data. No cast-time-derived coefficient is inferred.

Seven focused profile checks and all **231 network library tests**, formatting
and workspace/all-target denied-warning Clippy pass. The 241-core numerical
baseline and preceding full-network/client baselines are retained; no full app,
new wire or native acceptance is claimed by this inactive candidate. Next add
explicit pre-admission World activation, private previewed RNG, actual source life,
projectile impact timing, bounded typed events and durable kill opportunities.
Then qualify a new codec, encrypted UDP and normal native presentation before
switching development play to the replacement ability profile.

## Opt-in World execution — qualified, 2 November 2026

`World::enable_ability_damage` now permits explicit activation only at tick zero,
before any player is admitted and after ordinary combat. Supplied ability bytes
must exactly match the active World catalogue. Failed or duplicate activation
preserves World identity and the existing profile. The new identity wrapper
follows ordinary combat and precedes later gameplay roles.

The candidate owns a separate pre-admission-seeded private random stream. At an
eligible effect it previews base and landing draws, consumes a conditional critical
draw only when needed, evaluates the bound recipe and checks source life, target,
event capacity, retaliation/death deadlines before committing HP, typed fact,
provocation and RNG together. Valid avoided hostile attempts can provoke. Damage
facts retain source life generation, exact recipe/draws, resolved damage, health
lost, overkill and remaining health. No database or transport I/O enters the step.
The fixed effect amount is ignored only in this explicit replacement profile.

Instant specials resolve at their existing next-step effect boundary. Arcane
keeps the existing cast and finite projectile; it draws damage only at impact.
A committed equipment swap after launch does not erase a valid projectile.
Interrupted casts and departed sources produce no ability damage draws. Existing
range, LOS, manual facing, equipment, mana, queue, GCD and cooldown authority
remain in force. The separate ordinary swing stream remains unchanged.

Nine World checks plus seven profile checks pass locally, including 20/30/60 Hz,
source departure, invalid facing/equipment, interruption, preview replay, bounded
capacity, checked counters/deadlines and once-only XP/loot opportunities across
creature respawn. These are headless opportunities, not a new durable database
qualification. Full regression passes 241 core / 572 network (ten skips) /
118 client tests, formatting and workspace/all-target denied-warning Clippy.
Source/proof: `build/classic-ability-world-qualified-v1-20261102`.
All old event codecs explicitly refuse
`AbilityDamage` facts; the client has no accepted new-profile decoder or
presentation yet. Normal application launch remains on the preserved ordinary
checkpoint. Next qualify CE08, actual encrypted UDP, manifest/launcher and native
presentation before enabling the replacement through normal play.

## CE08 codec qualified — 2 November 2026

The explicit `Batch::encode_abilities` / `decode_abilities` entry points use the
immutable catalogue retained by the bound profile. CE08 carries its exact context
hash and allows mixed typed ability, ordinary melee, cast/projectile and lifecycle
events. It refuses legacy fixed ability damage, generic avoidance and older
physical/basic facts. Earlier CE02–CE07 layouts and entry points stay intact and
refuse the new tag/fact. Ability and ordinary-swing sequence namespaces remain
separate, so the same source/life sequence value is valid once in each.

A 75-byte ability fact contains source/target IDs, ability ordinal, source life,
base/landing/conditional-critical draws, checked defender bits, positive pre-hit
health, remaining health and global position. 0xffff is the explicit absent-crit
sentinel. Decoding reconstructs the complete outcome and damage from the bound
profile and verifies remaining health; encoding rejects contradictory typed facts.
This establishes mathematical/profile consistency of observations, not private RNG
ownership or permission for a client to change HP. No health mutation occurs.

The 61-byte header plus twelve ability facts is 961 bytes; twelve ordinary facts
still bound a mixed-mode batch at 1045 bytes, below the existing 1200-byte budget.
Eight codec tests cover exact round trips, hit/miss/dodge/crit/overkill, conditional
draw absence, mixed sequence namespaces/private life, maximal counts, wrong
profile/hash/tag, truncation/trailing bytes and invalid life/ordinal/draw/state/HP/
position. All 241 core / 580 network (ten skips) / 118 client tests, formatting
and workspace/all-target denied-warning Clippy pass. Source/proof:
`build/classic-ability-codec-qualified-v1-20261102`. New-profile encrypted Renet,
normal application, durable database and native presentation checks remain next.
The ordinary playable checkpoint stays preserved.

## Encrypted Renet qualification — 2 November 2026

The server now selects CE08 for an opt-in ability World and reports separate
ability RNG/outcome metrics. Clients require an explicit `MeleeActions.ability`
profile. Preflight rejects a missing/mismatched ordinary context or changed parsed
ability catalogue before network I/O. Catalogue comparison covers timing, arcs,
resources, effects and nested projectile data; the profile still binds exact raw
bytes independently. Older constructors explicitly select no ability replacement.
Reliable owner-private ordinary damage filtering remains before recipient sequence
allocation, and a full/closed presentation queue disconnects rather than dropping
resolved facts.

Three new transport tests pass: two actual encrypted peers at 20/30/60 Hz under
20 ms delay, 30 ms jitter, periodic loss/duplication and observed reordering;
capacity-one presentation refusal; and four pre-I/O context/catalogue mismatches.
The two-peer check mixes special and ordinary swings, receives private incoming
white facts only on the victim, swaps committed fixture equipment for Arcane,
proves impact tick follows launch, then receives another public special without
a reliable-stream gap. Public ability facts match exactly on both peers.
Three committed abilities use three base and landing draws plus only their
required critical draws; legacy physical RNG stays untouched.

All **241 core / 583 network (ten skips) / 118 client tests**, formatting and
workspace/all-target denied-warning Clippy pass. Exact source/proof:
`build/classic-ability-transport-qualified-v1-20261102`. The inventory bridge is
a synthetic committed observation shared by fixture admissions, not independent
OIDC/database character ownership. No new-profile durable SQL, launcher, native
rendering, Windows or MMO-capacity acceptance is implied. Normal application
adoption and six logged-in native checks are next; preserve the ordinary playable
checkpoint until they pass.


## Normal application qualification — 2 November 2026

P2-12 / P3-06 / P3-09: schema-8
`development-gameplay-classic-ability-damage.json` adds paired `ability_starting`
and `ability_damage` roles to the seventeen-role ordinary bundle. Rust validation
checks all nineteen exact-byte inputs and profile/category references. The normal
OIDC/PostgreSQL launcher snapshots and forwards the paired roles to both server
and client. Server opt-in binds the actual loaded ability catalogue; client
preflight and CE08 bind the same immutable profile and ordinary context. Missing,
partial, oversized, wrong-schema or inconsistent configurations fail before play.
Legacy manifests retain their fixed ability effects.

Standard bounded Bevy combat labels distinguish physical-special and Arcane
facts: physical numbers are white, Arcane purple, avoidance gold and criticals
larger. Numbers show resolved damage, with health loss/overkill retained in
metrics. Arcane landing failure displays “Resisted”; this is a miss label, not
implemented resistance mitigation. Current Arcane authority is explicitly
zero-resistance and equal-level only. Matching projectiles retire on hit or miss;
positive surviving hits flinch creatures, and positive Arcane hits trigger the
existing impact effect. Physical specials do not emit the generic magic impact.
Old-own-life facts cannot trigger new-life labels/effects. Snapshots remain the
health authority; labels never alter health.

All 243 core / 588 network (ten skips) / 123 client / 43 Python tests,
formatting, workspace/all-target denied-warning Clippy and release pass.
Seven fresh normal OIDC/PostgreSQL native runs pass: controls/abilities and actual
wolf fights at 1920x1200, 3440x1440 fullscreen and 1280x720, plus a 35-second
passive melee check. All three fights end with a typed special killing hit,
one durable XP commit and one loot offer, with zero pursuit returns. The passive
check has one acquisition, zero returns and ongoing incoming attacks throughout.
All runs have zero GPU diagnostics and combat event gaps. Linux AMD Radeon
RX 7900 XTX / RADV Vulkan / Mesa 26.2.6, physical display connected. Other sizes
are windowed; 120 FPS VSync observations are not MMO-capacity measurements.

Exact source/art/binaries/public proof:
`build/classic-ability-client-qualified-v1-20261102`. The
[melee contact correction](ENEMY_AI_REFERENCE.md) prevents the previous hard
30-second acquisition cutoff from ending a valid stationary fight. Earlier
artifacts remain immutable. All changes remain local and unpushed.

Static AP/spell-bonus templates, development special rates and spell coefficients
remain fixtures. Derived class/gear/level stats, damage/threat-based ranged leash
refresh, resistance/absorption, full evade/reset and general PvP remain separate
work. This slice makes no native Windows, loot-claim or MMO-capacity claim.
