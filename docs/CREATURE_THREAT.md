# Creature damage threat — 2 November 2026

P3-07 / P4-08. Prerequisites: checked resolved health loss, exact player/NPC
lifetimes, pursuit/retaliation, membership, credit and return/reset boundaries.
The latest normal playable remains
`build/creature-reset-client-qualified-v1-20261102`. No live threat-based victim
switching is adopted by the arithmetic prerequisite below.

## Evidence and scope

[Original Classic research, pinned revision 23f84bd](https://github.com/magey/classic-warrior/wiki/Threat-Mechanics/23f84bd22bfe4b88e899c84f2b1c35f4f0b9bfe8)
reports damage/threat at 1:1 and melee/ranged transfer thresholds of 110%/130%.
Its threshold tests cite Classic stress-test build 1.13.2.31407; later individual
ability measurements cite 1.13.6.37497. The reported measurement intervals are
broad. This is original empirical research, not Blizzard server source or proof
of every boundary/equality rule. Checked 2 November 2026. No external code is
imported. Healing, stance/ability modifiers, taunts and social threat are outside
this initial damage-only subset.

## Pure authority-only prerequisite

`theia_core::threat` provides a bounded table with at most 32 exact player lives.
The strict 4 KiB schema-1 policy explicitly selects the empirical damage ruleset
and its 11000/13000 basis-point thresholds. Unsupported thresholds/rulesets,
unknown fields, invalid IDs and participant bounds refuse. Maximum participants
is explicit development data, not a Classic population claim.

Authority supplies actual health lost, excluding prevented damage and overkill,
exactly once. Zero damage allocates nothing; explicit body-pull registration can
add a zero-threat entry. Dead/removed lives must be removed by the owner; an
unremoved earlier life refuses a different incarnation of that actor. Exact-life
removal cannot erase its replacement. This module neither accepts network damage
requests nor proves the future caller's committed-fact/encounter guards.

Selection takes a bounded, unique list of known current eligible lives and
authoritative melee-reach classification. It checks the entire list before
changing the victim. Current threat is sticky until a challenger strictly exceeds
the applicable threshold; equality retains the victim. Among qualifying
challengers choose highest threat, then lowest stable life key. An unavailable
victim permits selection from the remaining eligible entries without hysteresis.
Initial all-zero ties use the same stable ordering. Equality, tie ordering,
actual-HP-loss input and eligibility policy are explicit Theia adaptations where
the evidence does not settle exact behavior.

Arithmetic uses checked u64 addition and widened u128 ratio comparisons, without
floating-point rounding or partial mutation on overflow. Reach is supplied by
the authoritative gameplay check, independent of attack category: entering
melee can qualify existing threat without new damage. A higher ranged entry
below its threshold does not hide a lower qualifying melee challenger. No new
distance/speed or imperial conversion is introduced; the world stays Y-up metres.
The bounded table performs no pathfinding, network, database or awaited I/O.

Ten focused checks cover strict policy limits, zero/effective damage, capacity,
exact/above melee and ranged boundaries, movement-dependent reach, competing
qualifiers, deterministic ties, exact lifetimes, atomic eligibility refusal,
foreign/self actors, explicit clear and overflow. All 269 core tests, formatting
and workspace/all-target denied-warning Clippy pass. Exact source/proof:
`build/creature-threat-core-qualified-v1-20261102`. Network/client/native behavior
is inherited unchanged from the preserved normal reset checkpoint; no new live
victim switching or MMO-capacity claim is made.

## Next acceptance before normal adoption

Add an opt-in single-creature World adapter. Before it changes live targeting,
check prepared threat updates atomically with actual health/RNG/credit commit;
reject duplicate, stale-life and stale-encounter resolutions; use authoritative
living connected eligibility and current reach; retain credit independently of
victim transfer. A switch must cancel the previous victim's pending retaliation,
update pursuit through existing bounded paths and maintain membership correctly.
Handle target death/disconnect, reset, fresh respawn and participant saturation
explicitly. Do not let old casts or a reused life revive stale threat.

Then qualify competing actual encrypted clients at 20/30/60 Hz under impairment,
followed by versioned normal adoption and native checks. Preserve previous
playables until replacements pass. Party/social/healing threat, assistance,
multi-NPC spawns and load qualification remain later slices.


## Explicit transfer prerequisites qualified

The pure pursuit and retaliation policies now expose authority-only `transfer`
hooks; normal World code does not call them. Pursuit allows transfer only from
Chase/Attack before the current tick has been evaluated. It retains the captured
patrol anchor and existing soft deadline, and refuses inactive/dead/returning/
blocked encounters, invalid/foreign/dead/hidden targets and positions outside the
original leash. Requiring visible transfer targets is an initial Theia policy;
existing LOS grace for an unchanged victim remains unchanged. The next normal
pursuit evaluation still owns movement, contact refresh and reset.

Retaliation transfer cancels the previous victim's pending windup and starts no
replacement strike. It preserves an existing recovery; cancellation additionally
pays a fresh recovery before the new victim receives a full windup. Same-target
transfer is a no-op. Inactive/expired engagement, foreign epochs, stale/already-
evaluated ticks and clock overflow refuse atomically. These telegraph/recovery
rules are explicit Theia adaptations, not evidence of Classic NPC swing timing.
The existing ordinary `record_hit` behavior still prevents arbitrary attackers
from stealing an unexpired engagement.

Six new checks include 20/30/60 Hz anchor/deadline and cancellation timing cases,
current-boundary and lifetime refusals, recovery preservation and overflow. All
275 core tests, formatting and workspace/all-target denied-warning Clippy pass.
Preserve `build/creature-transfer-core-qualified-v1-20261102` (748 source files).
There is no new normal World/network/native behavior. The adapter must prepare
pursuit, retaliation, path revision invalidation and threat state together before
commit; a failed preparation must not leave a partial victim change. Existing
asynchronous route replies for the old victim must never become movement for the
new one. The following World slice remains the next acceptance boundary.
