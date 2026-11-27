# Melee weapon data and swing clock — headless foundation

P2-12 / P3-06 / P3-08. This implements the first numerical prerequisite in the
[autoattack plan](CLASSIC_AUTOATTACK_PLAN.md), using existing checked identities,
simulation tick rates, item definitions and facing arcs. The playable checkpoint
remains `build/classic-targeting-qualified-v2-20261101`; normal client/server
profiles do not enable this weapon file or clock yet.

`theia_core::melee::Catalog` validates bounded immutable weapon data keyed by
item ID. Cross-validation requires a known nonstackable main-hand item. This is
content validation; actual equipment ownership must come from the committed
inventory observation when the authority adopts it. Byte/count limits, duplicate
IDs, unknown fields, invalid timing/damage/reach and malformed arcs fail closed.
Periods round upward to fixed ticks. Reach is in metres; orientation uses the
existing character-facing contract.

`melee::timer::Clock` keeps engagement independent from readiness. Initial
readiness is immediate; stop/re-engage, duplicate enable and retarget preserve
the existing deadline. Authoritative eligibility pauses preserve the target.
Re-entry produces at most one swing and schedules the next from the actual
current tick; it cannot bank a burst. Eligibility remains the responsibility of
server range, facing, LOS, life, readiness, equipment and status validation.
A returned swing token is not permission for a client to apply damage.

Ability suspension freezes remaining time. Explicit resume restores that
remaining time; delay adds bounded time; reset replaces the period/countdown.
Ordinary eligibility pauses let time elapse. Repeated suspension is idempotent.
Callers must handle overlapping ability/status suspension ownership rather than
calling resume when another active restriction still owns the pause. Time
regression, foreign/self target IDs, excessive periods/delays and deadline
overflow fail without partial clock mutation. Matching entity invalidation ends
engagement, preserving cooldown. Owner life/session handling remains an
integration requirement.

The separate `development-melee-weapons.json` is a **Theia test fixture**:
training sword, 2400 ms period, 15 base damage, 3 m centre-distance reach and a
180-degree frontal arc. These numbers and immediate initial readiness are not
verified Classic formulas or final balance. Skill/level/AP modifiers and actual
Classic calibration follow [the evidence policy](CLASSIC_FORMULAS.md).
This fixture does not rescale the world or change existing gameplay manifests.

Seven new numerical checks pass, including 20/30/60 Hz timing, rounding,
metric/facing/item references, abuse/re-entry boundaries, explicit ability
interactions and atomic arithmetic failure. **153 core tests**, formatting and
workspace/all-target denied-warning Clippy pass. Proof:
`build/classic-melee-core-results-v1.json` and matching logs. Exact source/proof:
`build/classic-melee-clock-qualified-v1-20261101`. No rendering change or new
native/GPU, multiplayer combat or full-spec completion claim is made.

Next: bind this state to actual World actors and committed equipment; validate
swings against the authoritative scene and reuse damage/death accounting. Then
add bounded engage/cancel/state transport and normal right-click presentation.
Abilities need explicit swing interaction and facing rules before the complete
one-hostile milestone can pass. All changes remain local and unpushed.
