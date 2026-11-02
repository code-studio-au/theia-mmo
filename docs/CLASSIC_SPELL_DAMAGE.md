# Spell landing and conditional critical — inactive reference prerequisite

P2-12 / P3-06 / P3-09. `damage::spell` is a pure numerical reference for
player magic against an uncontrolled creature, levels 1–63. It does not activate
spell damage, replace the generic ability table or change the playable checkpoint
`build/classic-white-client-qualified-v1-20261101`.

## Contract and provenance

The ordinary, non-always-hit reference starts with 4% miss, changes by 1 percentage
point per target level through +2 and by 11 points per further NPC level. Supplied
spell-hit bonus is subtracted before clamping miss to 1–100%. At level 60 against
63, base miss is 17%; 16% bonus hit reaches the 99% landing ceiling. Levels,
bonus and roll domains are checked. Negative raw level penalties and penalties
above 100% survive until the final clamp. There is no weapon skill or low-level
ordinary-melee miss multiplier in this spell policy.

This is independently expressed from the pinned emulator's magic miss calculation,
with no auras, always-hit exceptions, pets, PvP or expansion extension. It is a
reference implementation, not new direct Blizzard measurement.
[Reference level curve and clamp](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L3556).

A separate supplied spell-critical chance uses a second checked draw only after
landing, with positive critical chance and explicit ability eligibility. Missing
or extra critical draws fail. A miss cannot become a critical; critical chance
is conditional on landing and is not pushed out by an earlier melee-table bucket.
The helper does not generate or establish independence/ownership of the supplied
random values. Zero chance or ineligible criticals require no second draw.
[Separate reference critical decision](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L3590),
[magic outcome path](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp#L2643).
No external code or data table is copied into Theia.

`Outcome::Miss` retains the base landing cause separately from school resistance.
Classic presentation may call both “resist”; that does not establish a school
mitigation value. This first helper implements neither binary/full/partial
resistance nor immunity, reflection, crit multiplier, damage, projectile timing,
random-number state or health. A future live profile must explicitly bind school,
resistance assumptions and eligible categories. Do not rename the old generic
Deflect fixture into a Classic spell outcome.

## Acceptance and next slice

Four tests cover reference boundaries, +3 bonus cap, large/negative raw penalties,
invalid inputs, every supported level pair, monotonic level/hit effects, conditional
critical draws, eligibility and zero/100% critical boundaries. Exhaustive 10,000-value
landing and critical domains establish exact conditional frequencies: 96% landing
with 25% conditional crit gives 24% critical, 72% normal and 4% miss over independent
domains. No fake network client or HP mutation is used as evidence.

All 235 core tests, formatting and denied-warning workspace/all-target Clippy pass.
Network/client behavioral baselines remain the preceding 556 (ten skips) / 118
checks; they were not rerun for this inactive pure module. Exact source/proof:
`build/classic-spell-numerical-qualified-v1-20261101`. No rendering change requires
native GPU acceptance in this slice.

Next establish main-hand-special outcomes and explicitly scoped mitigation, then
bind starting recipes, private RNG, life generations, authoritative health and
separately versioned facts. Preserve prior profiles/codecs and complete encrypted
and normal/native checks before replacing fixed abilities. Keep all distances
metric and P3-17/18 unspecified.
