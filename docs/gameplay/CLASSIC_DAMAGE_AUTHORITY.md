# Shared physical damage — normal client and native checkpoint

P2-11 / P2-12 / P3-06 / P3-08 / P3-09. World, Renet, normal application and native
presentation adoption now pass the checks below. Earlier stage records are historical. Prerequisites are the strict prototype and checked starting
arithmetic, provisional prevention and candidate definitions described in
[damage resolution](CLASSIC_DAMAGE_RESOLUTION.md). Preserve their checkpoints.
P3-17/18 remain unspecified; all spatial values remain metric.

## Implemented numerical prerequisites

`damage::physical::normal_reference_v1` now checks the armour proportion,
penetration, cap and positive minimum with explicit integer boundaries. Starting
damage is floored before armour, then the post-armour value is floored before
health clipping. The pinned header's integer damage storage supports examining
these boundaries; exact rational calculation is not a float-bit-compatibility
claim. The earlier exact prevention fixture remains unchanged.
[Reference storage](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.h#L593).

`damage::stats` schema 1 requires a named `physical_reference_v1` policy, checked
physical player/creature templates and nonnegative player AP/spell bonus. It
validates exact creature-definition coverage and rejects combined stat/attack
magnitudes that exceed numerical limits. The candidate level-60 templates in
`development-damage-stats.json` reproduce the numerical examples; they do not
replace the current level-1 playable character or participate in its manifest.

Six physical-policy and seven stat/profile tests pass, alongside **189 core
tests**, formatting and denied-warning workspace/all-target Clippy. Logs are
`build/classic-damage-stats-core-v1.log` and
`build/classic-damage-stats-clippy-v1.log`; exact candidate source/data/proof is
`build/classic-damage-physical-stats-qualified-v1-20261101`. The explicit World adoption below now passes; new result codec and
normal-application adoption remain pending. The playable
checkpoint remains `build/classic-ability-qualified-v1-20261101`.

## Candidate backend configuration checked

`simulation::combat::physical::Profile` now validates the attack/stat pair against
existing item, ability, base-weapon and creature definitions. It requires schema-7
ability rules, matching declared base weapon speeds, complete weapon coverage,
one known creature attack and valid health bounds. It rejects spell definitions
and nonzero spell bonus for this first physical-only adoption. Its identity
function binds exact attack/stat bytes and health bounds after the existing
ability/life/melee identity. Explicit World opt-in now invokes this binding; normal launcher adoption remains pending.

The separate `development-physical-damage-definitions.json` and
`development-physical-damage-stats.json` candidates use level-1 templates and
modest sword/wolf ranges. They are also outside active manifests. Nine focused
profile checks, **163 network-library tests**, **189 core tests**, formatting and
workspace Clippy pass. Exact component source/data/proof is
`build/classic-damage-profile-qualified-v1-20261101`. Network-library regressions
include existing transport checks; there is no new-profile World, wire, UDP or
normal-client adoption evidence yet. Those remain the next required work.

## Explicit World authority qualified

`World::enable_physical_damage` now binds matching item/weapon, stat, ability and
patrol/life roles before admission. Ordinary swings and creature retaliation use
the same normal-contact physical reference calculation, and emit separate
`PhysicalDamage` facts. Facts distinguish both actor and victim incarnations,
player life generations, attack and sequence, base roll, starting/armour/resolved
amounts, health lost, overkill and remaining HP. Legacy profiles retain fixed
amounts and old events; their codecs explicitly reject the new fact type even
when its amount would fit the old bound.

The authority previews base rolls on a copied server-owned ChaCha stream and
commits the stream only after a successful health/life operation. Rejected
eligibility, failed death deadlines, stale lives and event pressure cannot consume
a new committed roll. Outgoing capacity includes death and pending cast/queue
cancellations; incoming capacity includes the result and both interrupt events.
Checked NPC sequences and base-roll counters fail without changing health.
Static template stats remain explicit; no gear/class/level stat derivation is
claimed. Ordinary attack tables and spell coefficient adoption remain later work.

Twelve World checks cover real fixed steps/Avian at 20/30/60 Hz plus explicit
capacity/deadline fixtures. They prove both damage directions, committed weapon
swaps, facing/readiness/equipment rejection, replay, overkill, player/NPC respawn
incarnations and stale-life handling. One additional RNG exhaustion test passes.
The complete regression passes **189 core tests, 501 network passes / ten skips /
69 groups, 114 client tests**, formatting and denied-warning workspace Clippy.
See `build/classic-damage-world-results-v1.json` and matching regression logs.

Current component source/proof: `build/classic-damage-world-qualified-v1-20261101`.
That checkpoint is headless authority evidence. The following codec/transport
checkpoints now pass; launcher and native feedback remain pending. The normal
playable checkpoint remains `build/classic-ability-qualified-v1-20261101`; no
Windows or MMO-capacity claim is made. The remaining adoption contract follows.

## New result codec checked

`combat_wire::physical` now supplies CE06 with a full configuration hash. It
reconstructs each normal physical result from the bound attack/stat data and roll,
then checks all transmitted starting/armour/resolved/lost/overkill quantities.
Victims must have positive pre-hit health within their declared maximum. Source/
victim life-generation roles and duplicate source sequences are checked. Legacy
CE02–CE05 codecs reject CE06, and CE06 rejects legacy fixed-melee results while
still carrying the existing ability/projectile/death events.

Seven new codec checks and **183 network-library tests** pass, together with
formatting and denied-warning workspace/all-target Clippy. Every truncation and
trailing/oversized payload is rejected; wrong profiles, ordinals, coordinates,
life namespaces and contradictory facts fail. Twelve maximum-size result facts
plus the profile header occupy **1189 bytes**, inside the existing 1200-byte
bound. This is a codec budget check, not a World player-capacity measurement.
Exact component source/proof: `build/classic-damage-codec-qualified-v1-20261101`.
Renet selection and recipient privacy now pass below; normal application and
native feedback remain pending. Keep incoming player damage private to its owner, as existing life
observations are; filtering must preserve each recipient's event sequence.

## Actual Renet and kill-fact adoption qualified

The production Renet runtime selects CE06 from the explicit World profile; the
client requires the matching physical context, classic prediction, life and
inventory modes. Incoming player facts are filtered to their owner before
per-recipient batching, and receivers reject a foreign private victim. Public
outgoing creature damage remains observable to other players. Reliable event
sequence continuity is preserved when another player's private events are absent.
Presentation backpressure disconnects instead of dropping gameplay facts.

New physical fatal hits now feed the existing bounded XP/reward opportunity
boundaries, with duplicate/victim-incarnation checks. This preserves the explicit
development last-hit policy; Classic first-tag/shared-credit policy remains later
work. Opportunity publication is not a durable economy acknowledgement.

Actual encrypted UDP tests run two admitted peers through loss, jitter,
duplication and reordering at 20/30/60 Hz. The attacker receives both damage
directions and matching HP; an observer receives public outgoing hits, no private
incoming facts, and stays at full HP. Cancelling and reengaging proves each
recipient's reliable sequence remains continuous across filtered events. A
separate blocked-presentation test disconnects at the bounded queue. The fixture
uses slow patrol to isolate combat and synthetic committed inventory ownership;
it does not establish OIDC/PostgreSQL or native adoption for the new profile.

The full regression passes **189 core tests, 511 network passes / ten skips /
69 groups, 114 client tests**, formatting and denied-warning workspace Clippy.
Logs: `build/classic-damage-transport-results-v2.json` and matching v2 regression
files. New UDP proof: `build/classic-damage-udp-focused-v2.log`; kill-boundary proof
also passes in the full run. Exact source/proof:
`build/classic-damage-transport-qualified-v1-20261101`.

Next: explicit manifest/launcher/server/client profile loading and presentation,
then normal OIDC/PostgreSQL native sword/wolf acceptance. Keep the qualified
playable checkpoint until that replacement passes. Full attack tables, derived
stats, spell coefficient adoption, Windows execution and MMO capacity remain
outside this component acceptance.

## First live scope

Adopt ordinary player main-hand physical damage and creature melee retaliation
through the same checked calculation. Bind declared base ranges, scaling speed,
nonnegative effective AP, attacker level, defender armour and armour penetration.
The first stats are explicit static development templates; committed equipment
selects the weapon definition. This is not derived gear/class/level scaling.
Creature starting damage uses its own definition without an inventory weapon.

Use a separately named physical reference policy, preserving the existing exact
`resolve_fixture_v1` experiment. The pinned CMaNGOS Classic reference uses an
armour proportion equivalent to `A / (A + 400 + 85 * attacker_level)`, after
nonnegative effective armour, with a 75% cap and a minimum positive post-armour
amount. Inspect integer storage/rounding boundaries before adopting its stage
order. This is emulator evidence, not a declaration that every Blizzard server
rounding detail is proven. Do not silently apply later-expansion level modifiers
or extrapolate a Classic-domain policy to Theia's planned level-80 extension.
[Pinned implementation reference](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L2155).

Keep the first ordinary swings' existing normal-contact behavior while adopting
magnitude/armour. Full ordinary attack tables, critical/glancing/crushing outcomes,
positional defender eligibility and skill/level probabilities are the following
slice. Ability/spell damage retains its qualified behavior until separately
versioned cast/projectile adoption passes. Reject unsupported live definition
kinds rather than silently loading unused spell coefficients. Block/absorption
pools and school resistance need their own verified eligibility, ordering and
consumption contracts; a per-attack fixture value is not a persistent shield.

## Binding and facts

Use explicit new data/profile identity and event codec versions. Bind exact attack
and stat definition bytes before admission on both ends. Do not relax the old
`MeleeDamage.amount <= weapon.damage` validator or reinterpret CE05. Legacy
profiles and result layouts remain intact. A missing definition or incompatible
item/creature reference must fail configuration before gameplay, not fall back
to a fixed amount after a new profile was selected.

Emit bounded resolved facts identifying source attack, attacker/victim incarnation,
sequence, starting amount, armour prevention, resolved damage, health lost,
overkill and remaining health. Incoming player facts must carry the current
player life generation so late reliable delivery cannot affect a replacement
life. Keep player swings, NPC strikes and ability sequences distinct in client
animation/feedback ledgers. Client feedback displays authoritative facts only.

Reserve event/death capacity and checked counters before consuming a roll or
mutating health. Adapt reliable packet batch counts to the maximum new event
size while preserving the existing 1200-byte packet bound and bounded queues.
Use the existing server-owned seeded randomness for bounded base rolls. Invalid
requests, repeated ticks and replayed intents must not draw again. Apply health,
death/reset, retaliation, tag/contribution, XP and loot facts once at their
existing authority boundaries. No awaited I/O enters the simulation step.

## Acceptance

- Numerical reference-policy checks for zero armour, attacker-level dependence,
  penetration, cap, positive minimum and stage rounding; keep illustrative 62.6
  exact-fixture evidence separate from integer reference-policy results.
- Required/non-null/unknown fields and bounded stat/definition loading, reference
  failures, exact identity changes and legacy/new profile refusal.
- Real World/Avian checks at 20/30/60 Hz for both damage directions, committed
  weapon changes, readiness/range/facing rejection, replay/roll counts, armour,
  low health/overkill, death/respawn and exhausted event/deadline/counter capacity.
- Strict new result round trips and malformed facts, legacy decoder rejection,
  actual encrypted impaired UDP and presentation backpressure.
- Normal OIDC/PostgreSQL native sword/wolf fights with authoritative damage
  feedback and correct HP/death. Record actual window dimensions, GPU and display
  conditions. Run formatting, denied-warning workspace Clippy and appropriate
  regression tests before replacing the playable checkpoint.

## Normal application adoption in progress

The explicit `development-gameplay-classic-physical.json` candidate uses manifest
schema 6, sealing sixteen roles including physical definitions and stat templates.
The validator checks the complete pair and cross-profile bindings; the launcher
snapshots exact bytes for both children. Server and client opt in with
`--physical-definitions` and `--physical-stats`, with matching authority identities.
The previous schema-5 playable profile is preserved.

Client presentation now consumes the physical fact: weapon-bound sword cues,
creature hit reactions, white outgoing and red incoming resolved-damage labels.
Health continues to come from authoritative life/combat observations. Facts from
an obsolete local life generation suppress their cosmetics without affecting
reliable batch sequencing. Telemetry distinguishes starting damage, armour
prevention, resolved damage, health lost and overkill. Label values show resolved
damage, which can exceed health lost on a killing hit.

Twenty-three manifest checks pass (three new checks). Full regression, release
and native OIDC/PostgreSQL validation are still running. This candidate does not
yet replace the qualified playable checkpoint. First live scope remains normal
physical contact with static templates; abilities retain their previous fixed
values. Full Classic outcome probabilities, derived stats and spell coefficients
remain subsequent slices.

## Normal client and native acceptance qualified

All **189 core / 514 network passes (ten skips) / 115 client / 39 Python** checks,
workspace/all-target denied-warning Clippy, formatting and release pass. Proof:
`build/classic-damage-client-*-v3.log`, result JSON alongside them. Three new
manifest checks cover schema/pair/alias rules, exact-byte identity, known weapon
references and bounded input. The client regression verifies that a killing hit
labels resolved damage separately from clipped health loss/overkill, and incoming
labels use a separate presentation kind.

`build/classic-damage-native-v1` tests melee controls and wand casting at actual
1920x1200, 3440x1440 and 1280x720. `build/classic-damage-native-v2` tests the
unmodified wolf profile at the same sizes. All six normal OIDC/PostgreSQL launches
pass with zero GPU diagnostics. Per-size production server summaries verify
perception/chase/attack, physical hits in both directions, base-roll accounting,
exactly one durable XP commit and one reward offer. The 1280x720 case includes
more basic swings when the existing sword ability does not supply its normal hit;
no fixed hit count is assumed. Actual loot claiming is outside these checks.

Exact source, art, stripped Linux binaries and proof are preserved at
`build/classic-damage-client-qualified-v1-20261101`. This is now the latest
playable checkpoint; the previous ability checkpoint remains intact. Device:
AMD RX 7900 XTX, RADV NAVI31, Vulkan, Mesa 26.2.6 kisak. No physical output was
connected, so the existing explicit uncapped diagnostic and windowed ultrawide
were used. This does not establish connected-display VSync/fullscreen performance,
native Windows execution or MMO capacity.

Remaining work: category-specific attack/outcome eligibility and probabilities,
critical/glancing/crushing scales, block/resistance/absorption policy, derived
gear/class/level stats and ability/spell starting-damage adoption. The current
physical profile still uses explicit level-1 stat templates, independently of
progression level. The generic existing ability deflect remains a fixture rather
than a renamed Classic parry or block. Changes remain local and unpushed.
