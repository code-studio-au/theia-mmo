# Classic formula fidelity and metric policy — 1 November 2026

The user requested WoW Classic formulas wherever possible, with Theia remaining
metric. This refines [THEIA-COMBAT-001](COMBAT_TARGETING_MOVEMENT_SPEC.md) and the
[enemy AI reference](ENEMY_AI_REFERENCE.md). Prototype values do not establish
Classic formula fidelity.

The [broader world-system notes](CLASSIC_WORLD_SYSTEMS_REFERENCE.md) prioritize
attack categories, eligibility, combat membership and progression interfaces
before full coefficient tuning. Exact weapon-skill tables and unusual outcomes
can follow evidence/balance work. This preserves the Classic-oriented baseline
and metric policy; it does not promote unsourced examples into verified rules.

The adopted [damage-resolution stages and acceptance](CLASSIC_DAMAGE_RESOLUTION.md)
separate base/stat scaling, attack outcomes and defender mitigation. Live damage
remains fixture-based until versioned numerical, World, transport and native
checks pass.

## Reference policy

- Prefer original-style Classic/1.12. Record the actual source build and later
  changes; do not silently mix retail, seasonal, TBC and Wrath rules. Theia's
  level-80 direction stays; extensions beyond evidenced levels need explicit tests.
- Prefer Blizzard statements/client data. Original measured Classic research can
  fill gaps with its build, sample and uncertainty recorded. Emulator behavior
  alone does not establish Blizzard behavior.
- Keep formulas/data authoritative and add numerical fixtures for thresholds,
  rounding and boundary cases. Separate confirmed values, empirical formulas,
  Theia adaptations and temporary fixtures. Do not label an invented leash,
  attack arc or Tab search as exact Classic behavior.
- Implement formulas as each playable slice needs them; no wholesale emulator
  port or new simulation framework is implied.

## Metric convention

Y-up, **1 world unit = 1 metre**, checked f64 global coordinates and bounded f32
local frames remain unchanged. Use m/km, m/s and explicit seconds/ms/ticks.
Percentages, level/skill differences and ratios are dimensionless.

Each dimensional rule needs a recorded source-unit-to-metre mapping before
adoption. Distinguish gameplay map units/localized labels from literal physical
imperial measurements. Do not double-convert metric-equivalent gameplay values
or rescale the world. This record does not itself adopt new speed/range numbers.

## Initial evidence register

| Rule | Source evidence and limits | Adoption gap |
| --- | --- | --- |
| Run reference | Vanilla character-frame UI declares base movement rate 7. This is a client reference, not the complete server movement algorithm. [Pinned extracted Blizzard UI source](https://github.com/Gethe/wow-ui-source/blob/4192ac60067fc55566182291bc67c85dd9b6794d/Interface/AddOns/Blizzard_CharacterFrame/Vanilla/PaperDollFrame.lua#L7) | [Ground movement v3](CLASSIC_GROUND_MOVEMENT.md) is qualified for explicit metric mapping and shared walk/run/backward/strafe rates; complete parity and other movement modes remain open |
| Creature avoidance and skill | Blizzard's reference comparison reports equal-level creature dodge 5%, +0.5 percentage points per higher creature level; +3-level parry 14% and base miss 8%. It separately explains first-1%-hit suppression above a 10-point defense/weapon-skill gap. Conditions matter; these are not universal PvP/low-level formulas. [Archived original Blizzard QA replies](https://www.bluetracker.gg/wow/topic/us-en/185675-bug-hit-tables/) | Replace generic hit/miss/deflect fixtures with skill/level/attack-type rules; passive dodge outcomes remain distinct from deferred active dodge |
| Threat | Original Classic research reports 110%/130% melee/ranged transfer thresholds with tests/build context, damage at 1:1 threat and effective healing at 0.5 before relevant distribution/modifiers. [Original research](https://github.com/magey/classic-warrior/wiki/Threat-Mechanics) | Add bounded per-NPC threat and exact threshold tests; current acquired-target policy is not a threat table |
| Attack tables | Original research separates confirmed observations from estimates and records build-dependent corrections. [Original attack-table research](https://github.com/magey/classic-warrior/wiki/Attack-table) | Carry white/special, single/dual weapon, skill, defender and positional distinctions explicitly |
| Aggro, leash, arcs, Tab | Supplied notes establish behavior; exact algorithms are not verified here | Verify where possible, label adaptations; 60 m / 12 s leash examples remain illustrative |

Checked 1 November 2026. Direct retrieval of the original Blizzard forum links
failed; the QA replies are available through the linked archive. The pinned Era
UI mirror contains later additions too; its constants do not reveal server code.
No proprietary implementation was copied or new dependency introduced.

Persistent targeting is [qualified](CLASSIC_TARGETING.md). Weapon-timed melee
and facing validation remain next.
Movement calibration and combat/AI formulas now follow this evidence policy;
frozen checkpoints retain their historical values and qualification.

The [damage-only threat arithmetic prerequisite](CREATURE_THREAT.md) now pins
the original research revision and separates empirical thresholds from explicit
Theia equality/tie/lifetime rules. It is inactive in normal World combat; no
healing/class modifier, live target transfer or new distance formula is adopted.


Quest item delivery research (3 November 2026) is recorded in
[QUEST_ITEM_OBJECTIVES.md](QUEST_ITEM_OBJECTIVES.md). Pinned VMaNGOS/CMaNGOS
Classic implementations provide evidence for item-removal incompletion, bag
rechecking and consumption at reward; emulator behavior is not Blizzard-server
proof. Counts are dimensionless. Initial backpack-only possession and final-stage
delivery scope are explicit Theia choices; live adoption remains in progress.
