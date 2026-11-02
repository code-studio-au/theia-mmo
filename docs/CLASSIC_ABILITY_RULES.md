# Ability facing and weapon-timer interaction — qualified

P2-11 / P2-12 / P3-06 / P3-08 builds on qualified classic controls, persistent
targeting and basic melee. Schema-7 ability rules now pass the normal logged-in
client, authority and transport acceptance. The current source, binaries, art and
proof are frozen in `build/classic-ability-qualified-v1-20261101`. Earlier
checkpoints remain historical and unchanged. [Damage resolution](CLASSIC_DAMAGE_RESOLUTION.md)
is next; live damage is still the existing fixed development fixture.

## Implemented behavior and evidence

Activation, queued start and cast completion validate manual facing and minimum
range. New positional rejection reasons require ability schema 7; bounded packet
layouts are preserved and exact data bytes participate in gameplay identity.
Timer changes are planned before resource/cooldown commit. Suspension resumes on
completion/cancellation even after selection is cleared. Repeated delays have a
total future-readiness bound. A real committed weapon change resets the clock,
including a swap through a non-melee wand. Loading observations and same-item
revision updates do not reset it. Exhausted deadlines stop engagement safely.

The development profile explicitly declares instant Sword Slash with continue
and a 500ms stationary Arcane Bolt with suspend, both using manual frontal
eligibility. Movement rejects or interrupts the latter; other abilities can
still explicitly permit mobile casting. Timing and balance remain Theia fixtures.

Formatting, denied-warning workspace Clippy, release builds, **156 core tests,
479 network passes / ten explicit skips / 69 groups, 114 client tests and 37
Python checks** pass. Focused evidence includes eleven new World tests covering
20/30/60 Hz, one wire check, three schema checks and a new encrypted impaired UDP
scenario at all three rates. See `build/classic-ability-world-v5.log` and
`build/classic-ability-udp-v1.log`, plus the full v2 regression logs.

Six final normal OIDC/PostgreSQL starts pass in `build/classic-ability-native-v6`
and `build/classic-ability-native-v7`: controls/casting and unmodified wolf fights
at actual 1920×1200, 3440×1440 and 1280×720 sizes. The 720p run uses the A/D-strafe
preference. Tests use OS input to owned PID-matched X11 windows and read-only
telemetry. They prove committed wand equipment, rejected rear-facing casts
without mana loss, a valid timed spell/projectile, a separate instant sword
ability and subsequent wolf death; the unmodified profile separately proves
perception, chase, retaliation and kills without player death.

All six report zero GPU diagnostics on Linux, AMD Radeon RX 7900 XTX / RADV
NAVI31, Vulkan, Mesa 26.2.6 kisak. No physical monitor was connected: the harness
explicitly used the existing uncapped presentation diagnostic and windowed
3440×1440. It asserts actual window dimensions. Earlier fullscreen fallback to
800×600 is excluded from final evidence, as are the stalled VSync and interrupted
telemetry-read attempts. Captures were inspected. This does not establish normal
VSync performance, physical input, connected-display fullscreen or Windows.

All thirteen strict prototype checks are demonstrated for this development
profile. This does not implement every Classic damage formula, classes, PvP or
MMO capacity. P3-17/18 remain unspecified. Changes remain local and unpushed.
The original acceptance contract follows.

## Data and authority

Use an explicit new ability schema for minimum metric range, positional facing
eligibility and autoattack interaction. Existing schemas retain their behavior;
new fields cannot silently alter old data. Each new-schema ability explicitly
chooses any-facing or a checked frontal arc, minimum range below maximum range,
and a continue, suspend, delay or reset rule. Exact profile bytes participate in
the existing world identity. Require classic independent-facing authority for
these rules. No target-facing assistance or camera dependence is introduced.

Validate current authoritative facing/minimum range on activation and completion,
and again when a queued ability actually starts. Return distinct feedback for
being too close or facing away. A rejected request cannot mutate a swing clock
or spend ability resources. A foreign/replayed request cannot reroll or restart
an attack. Keep queue, deadline, cast/event and per-tick budgets unchanged.

The initial explicit timer semantics are Theia development policies, not a claim
of exact Classic spell behavior:

| Rule | Timer at a successful ability start | During a cast | Completion or cancellation |
| --- | --- | --- | --- |
| Continue | Preserve deadline | Basic swings remain eligible if other checks pass | Preserve clock |
| Suspend | Freeze remaining countdown | Pause basic swings | Resume remaining countdown |
| Delay | Add one bounded declared delay | Countdown elapses; basic swings pause | Eligibility resumes without a stored burst |
| Reset | Replace countdown with a full applicable weapon period | Countdown elapses; basic swings pause | Eligibility resumes without a stored burst |

Queued intent applies no timer change until actual start. GCD is independent of
the weapon clock. Switching committed weapons preserves the existing full-period
reset rule; equipping the same item at a newer revision does not reset it. Resolve
suspension before applying another ability's rule and after a cast ends. Clock
arithmetic must fail atomically before cost/cooldown acknowledgement. Bound total
future readiness, not merely an individual delay argument. Preserve old cast
pause behavior in legacy profiles.

## Acceptance

- Numerical schema checks for required/non-null fields, invalid ranges/arcs,
  unknown rules/fields, delay bounds, legacy incompatibility and tick conversion.
- Actual World/Avian checks at 20/30/60 Hz for front/behind/side, minimum-range
  boundary, turn during a cast and queued-start revalidation. Target selection and
  character heading remain independent. Invalid requests preserve health/mana/
  clock state except their bounded rejection acknowledgement.
- Real weapon clocks for all four interactions, instant and cast-time abilities,
  GCD, cancellation, repeated/queued actions, equipment changes, death/respawn and
  exhausted deadlines/counters. No catch-up burst or unbounded postponed timer.
- Strict wire feedback/profile compatibility, actual encrypted impaired UDP,
  normal OIDC/PostgreSQL and native instant/cast-time play with committed sword/
  wand equipment. Update the combat acceptance checklist only from this evidence.

Classic coefficients, outcome tables, armour/resistance and absorption remain the
following damage-resolution slice. New timing/facing primitives must not pretend
to implement those formulas, full classes or PvP. P3-17/18 stay unspecified.
