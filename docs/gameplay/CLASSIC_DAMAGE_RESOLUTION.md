Current playable adoption: [shared physical authority and native checkpoint](CLASSIC_DAMAGE_AUTHORITY.md)
now passes 189 core / 514 network / 115 client / 39 Python checks and six normal
native runs. `build/classic-damage-client-qualified-v1-20261101` preserves the
result. The stage records below describe the numerical prerequisites historically;
full attack tables, derived stats and spell coefficient adoption remain pending.

# Damage resolution — starting arithmetic implemented, live adoption pending

P2-11 / P2-12 / P3-06 / P3-09: separate starting damage, attack outcome and
what the defender prevents. This incorporates the user's 1 November explanation
under the [Classic evidence and metric policy](CLASSIC_FORMULAS.md). It does not
replace the current fixed-damage fixtures or establish every Classic formula.
[Ability-facing and swing rules](CLASSIC_ABILITY_RULES.md) are qualified. Keep
P3-17/18 unspecified.

## Implemented numerical slice

`theia_core::damage` now checks inclusive base ranges, a server-supplied roll,
nonnegative AP/bonus damage and explicit weapon scaling speed or spell coefficient.
It preserves exact rational starting damage. Floor and nearest-half-up conversion
are explicitly named numerical policies; neither is adopted as Classic health
rounding. The 118 weapon and 260 spell examples, coefficients above 100%, half
boundaries, zero/fixed creature amounts, malformed ranges/rolls/stats and final
magnitude limits pass six new tests. Excess damage fails before rounding rather
than saturating. Negative effective stats and effect-specific coefficient rules
remain outside this first slice.

All **169 core tests**, formatting and denied-warning workspace/all-target Clippy
pass (`build/classic-damage-resolution-core-v2.log` and
`build/classic-damage-resolution-clippy-v2.log`). This is pure arithmetic with no
rendering change; the latest playable/native checkpoint remains
`build/classic-ability-qualified-v1-20261101`. The exact prevention fixture remains numerical-only. Starting damage and the
separate physical reference policy now serve an explicit experimental World mode;
the normal playable profile and old wire bounds remain unchanged. A separate `resolve_fixture_v1` now applies a supplied impact scale, supplied
armour or resistance reduction, flat block, absorption and explicitly chosen
rounding. It reports exact prevention/resolved amounts plus rounded damage,
health lost and overkill. Seven further tests cover the 62.6 example, full/partial
prevention, avoiding damage without consuming defences, intermediate bounds,
small fractions through multiple scales, and overkill. These inputs do not
determine critical/glancing/crushing or avoidance eligibility/probability. No
integer shield-pool consumption rule is implied by exact absorption facts.
The initial numerical checkpoint is preserved separately; final numerical source
and proof are in `build/classic-damage-numerics-qualified-v1-20261101`.

## Candidate definition loading

Damage catalogue schema 1 now loads bounded, stable-ID weapon, direct-spell and
creature base definitions through `theia_core::damage::catalog`. The candidate
`content/gameplay/development-damage-definitions.json` reproduces the numerical
examples; it is deliberately absent from active gameplay manifests. Loading it
does not change live damage, identity or wire validation.

Seven new catalogue tests cover required/non-null and unknown fields, schema,
byte/total/category budgets, independent duplicate IDs/references, numeric limits,
stable lookup order, unknown references, stackable weapon items, incompatible
ability effects and missing required spell equipment. References validate against
existing item/ability catalogues and supplied known creature IDs. Runtime authority
must still validate actual committed equipment and creature incarnation.

All **176 core tests**, formatting and denied-warning workspace/all-target Clippy
pass (`build/classic-damage-catalog-core-v1.log` and
`build/classic-damage-catalog-clippy-v1.log`). Candidate source, data and proof are
in `build/classic-damage-catalog-qualified-v1-20261101`. These are direct starting
components, not a complete rank/coefficient catalogue or category probability
table. Next bind explicit actor stats and checked category eligibility, then
version World identity/result codecs and prove real gameplay adoption. Existing
fixed-damage profiles remain intact until that replacement passes. The
[shared physical authority slice](CLASSIC_DAMAGE_AUTHORITY.md) now has checked
physical reference arithmetic and static stat-template prerequisites (189 core
tests). Its explicit ordinary-physical World and encrypted Renet adoption now pass the
full regression (511 network passes). Normal launcher/client/native adoption
remains pending.

## Shared authoritative stages

1. Validate attack category, incarnation, life, equipment, position/facing,
   range/LOS, readiness and timing. Invalid requests do not roll a hit or spend
   resources unless that specific ability's checked rules require it.
2. Build starting damage from an authoritative attack definition and stats.
   Player main-hand swings, weapon specials, spells and creature attacks carry
   explicit categories. A creature attack need not own a player inventory item.
3. Determine eligible outcomes for that category and defender. Keep hit chance,
   critical chance and positional/skill/level eligibility distinct from damage
   magnitude. Do not give every attack all outcomes or turn the existing generic
   deflect fixture into parry, shield block or magical resistance. Verify roll
   structure and combinable results per attack category rather than assuming
   one universal mutually exclusive table.
4. Resolve outcome scaling, modifiers and defences in a versioned order. Physical
   armour, shield block, school resistance and absorption are distinct stages.
   Verify their ordering, caps and rounding for the chosen ruleset before adoption.
5. Apply health loss once through authority and emit a bounded result breakdown.
   Resolved damage, actual health lost and overkill are separate quantities.
   Reuse death, retaliation, contribution/tag, XP and loot fact boundaries.

Player outgoing damage and creature incoming damage use these same stages with
appropriate attack definitions and defender stats. The client displays results;
it never supplies trusted attack power, armour, resistance, damage rolls or health.
Keep outcome randomness bounded and reproducible from server-owned event identity
and seed state, independent of frame rate and tick rate. Rejected/replayed requests
must not reroll; tests can inject deterministic rolls into the numerical resolver.

## Candidate arithmetic and evidence limits

For an ordinary main-hand swing, the user proposes weapon roll plus AP/14 times
weapon speed in seconds. The supplied example is arithmetically 90 + 140/14 × 2.8
= 118. The pinned extracted Blizzard Era UI declares the AP divisor 14; this
supports the reference constant, not the complete server formula or all special
attack rules. Verify base versus effective/normalized weapon speed explicitly;
changing the swing scheduler through haste must not accidentally multiply AP
scaling twice. [Pinned Blizzard UI extraction](https://github.com/Gethe/wow-ui-source/blob/4192ac60067fc55566182291bc67c85dd9b6794d/Interface/AddOns/Blizzard_CharacterFrame/Vanilla/PaperDollFrame.lua#L6).

The supplied defence example gives 118 × 0.7 − 20 = 62.6 before choosing integer
rounding. Reporting approximately 63 is illustrative; do not infer Classic's
stage ordering or rounding from it. Armour reduction and absorption values in
that example are not tuning defaults. The supplied
[University of Tartu PDF](https://dspace.ut.ee/bitstreams/dce37831-5d58-410a-b1e7-34d154e6c9f5/download)
could not be retrieved during this review and is not treated as verified evidence.

Keep ordinary swings, specials, single/dual wield, player/creature defenders and
position-dependent eligibility explicit. Original Classic testing records build,
samples and uncertainty, and distinguishes measured data from formulas used to
fill gaps. Use its original logs and Blizzard confirmations to select fixtures;
do not adopt its whole table as a universal combat algorithm.
[Original attack-table research](https://github.com/magey/classic-warrior/wiki/Attack-table).

For spells, the user's base-plus-bonus-times-coefficient model is the intended
starting shape. Rank, effect type, direct/periodic/channel/hybrid portions and
ruleset can affect the coefficient. The cited addon describes these components
and marks derived estimates separately; it is useful provenance, not proof of
Blizzard server coefficients. No addon code or coefficients have been imported.
[Addon author's documentation](https://www.curseforge.com/wow/addons/spellcoeffsforever).

The next eligibility/data review also checks the independent CMaNGOS Classic
implementation at commit `5731249195520323cb5fb9aa29761d4f033272ce`. It separates
ordinary melee outcome eligibility from spell/ability checks and has distinct
armour, block and absorption stages. This is emulator reference behavior, not
Blizzard server evidence. No CMaNGOS source has been copied into Theia; the
numerical fixture's ordering is not promoted to exact Classic behavior.
[Pinned reference](https://github.com/cmangos/mangos-classic/blob/5731249195520323cb5fb9aa29761d4f033272ce/src/game/Entities/Unit.cpp).

## Repository gaps and acceptance order

The qualified normal-client basic melee profile applies a fixed weapon-fixture amount, clipped by the
victim's remaining health. Its wire validator bounds applied damage by that
weapon amount. Ability hit/miss/deflect fixtures and fixed creature retaliation
are separate existing experiments. An explicit World experiment now handles ordinary AP/armour in both directions;
normal-client/transport adoption, resistance/absorption, the full melee attack
table and rank-coefficient catalogue remain pending.

After the strict combat rules pass:

- Add checked numerical attack/stat/outcome/defence types and explicit source-
  versioned fixtures. Cover the 118 example, rounding boundaries, zero damage,
  full absorption, partial prevention, overkill, overflow and ineligible outcomes.
- Bind committed player stats and creature definitions into gameplay identity.
  Use one bounded resolver from both damage directions; retain fixture profiles
  until real World tests pass at 20/30/60 Hz.
- Version affected weapon/ability data and result/event codecs before enabling
  damage exceeding the old fixed-weapon wire bound. Do not silently relax it.
- Prove replay/randomness stability, death/reset and contribution facts, actual
  encrypted UDP and normal logged-in equipment/defence feedback. Clearly separate
  verified Classic rules, Theia's level-80 extensions and provisional balance.

## Next numerical slice

Checked, renderer-independent arithmetic is implemented for weapon roll plus AP and
explicit base weapon speed, spell base plus a declared coefficient, and a fixed
creature starting amount. Preserve fractions until an explicitly named integer
rounding boundary. Validate magnitudes and intermediate arithmetic; invalid data
cannot silently clamp into a valid attack. Keep haste scheduling separate from
the weapon speed used for AP scaling.

The bounded fixture result breakdown is implemented for supplied impact and
prevention values. Next add checked category eligibility and versioned definition
loading before World adoption. The numerical stage does not invent outcome
probabilities or derive
armour/resistance percentages from unverified level formulas. Label fixture
ordering and rounding as provisional until supported by source-specific checks.
No new core helper authorizes a live health change or relaxes an existing codec.

All spatial inputs and UI remain metric. AP, coefficients and probabilities do
not rescale terrain or movement. Rich formula coverage does not imply working
classes, groups, PvP balance or MMO capacity.
