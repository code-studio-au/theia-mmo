# Creature damage threat — 3 November 2026

P3-07 / P4-08. Prerequisites: checked resolved health loss, exact player/NPC
lifetimes, pursuit/retaliation, membership, credit and return/reset boundaries.
The latest normal playable remains
`build/creature-threat-client-qualified-v2-20261103` (explicit schema-13 bundle).
The schema-12 quest playable and earlier modes remain preserved. The sections
below record each qualification boundary; their earlier "next" steps are historical.

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

## Opt-in World adapter qualified — 3 November 2026

P3-07 / P4-08. Reviewed the nine-file archived draft against the newer quest
runtime and applied its patch selectively. `World::enable_creature_threat` is
pre-admission, Classic-only and requires typed ordinary/special damage, combat
membership, creature credit and return reset. Exact bounded policy bytes wrap
world identity after reset. Existing modes retain `None`; no ordinary launcher,
manifest or client has opted into threat switching at this checkpoint.

Committed actual health loss prepares the bounded threat table alongside credit,
HP, membership, retaliation and private random draws. Preparation failure commits
none of those effects. Valid avoided hostility can explicitly register a zero
entry; overkill and prevented damage do not create damage threat. Client Cast
replay guards prevent duplicate public commands. Internal prepared Hit fixtures
are trusted authority inputs, not a separate replay protocol; older spell
sequences may legitimately land after newer instant abilities.

Selection runs after movement and before pursuit/retaliation evaluation. It uses
exact living connected player lives, checked visibility and current metric NPC
reach. Current-victim LOS grace remains; a new victim must be visible and inside
the original leash. Moving into melee can make existing 120% threat qualify
without another hit. Exact 110%/130% equality retains the victim. Pursuit and
retaliation are cloned and validated before switching; their original anchor and
soft deadline remain. A switch cancels an old windup, preserves the adopted
recovery policy, advances the checked route revision and invalidates cached paths,
requested goals and pending replies. Late old replies cannot steer the new chase.

Connected participants stay in combat through victim changes. Dead/disconnected
exact lives are pruned; respawns inherit no prior-life threat. NPC death, fresh
NPC lifetime, Return and Blocked clear the table. Lifecycle cleanup also runs
after transitions so stale entries never survive the Return/death frame. Credit
ownership remains separate: transferring victims does not steal the tag; tag-owner
departure retires old credit-generation casts while preserving a remaining live
victim's threat. No new party, healing, social or reward policy is introduced.

Ten focused World checks and two transfer/path checks pass, including 20/30/60 Hz
cases, saturation before HP/RNG mutation, actual damage accounting, movement,
credit independence, exact lives and reset. Full workspace checks passed 1,071
tests with 50 service-dependent tests ignored, followed by two additional focused
World cases; formatting and all-target denied-warning Clippy pass for the final
source. Preserve `build/creature-threat-world-qualified-v1-20261103` and the latest
quest playable. Existing SQL/native evidence is inherited; this slice makes no
new SQL, normal client, native GPU, Windows or capacity claim. Next acceptance is
competing actual encrypted peers under impairment, then versioned normal adoption.

## Encrypted competition qualified — 3 November 2026

Two actual encrypted Renet peers at each of 20/30/60 Hz use the opt-in World
adapter through public casts and weapon-timed autoattack. Deterministic loopback
relays apply 20 ms delay, 30 ms jitter, every-eleventh-packet loss and every-
seventeenth-packet duplication. Each run observes the original victim's private
windup, competing damage, its cancellation, the new victim's private windup and
an actual retaliation strike. Both participants retain combat membership; public
outgoing damage results agree and incoming NPC strikes stay private to their
victim. Committed damage-threat totals equal public actual NPC health lost.
Every rate records actual loss, duplication and reordering with zero relay queue
overflow. ServerStats/shutdown summaries expose bounded threat counters only.

Final formatting, all-target denied-warning Clippy and the impaired three-rate
test pass. Earlier assertion failures confused unchanged private-state timestamp
suppression and private incoming strikes with public results; the final test
checks the existing transport contract without changing it. Admission/inventory
are synthetic committed ports; this is not OIDC, SQL durability, native rendering,
Windows or capacity evidence. Preserve `build/creature-threat-runtime-qualified-v1-20261103`.
Next: explicit versioned normal bundle/CLI adoption and native regression; the
schema-12 quest playable remains preserved and has no threat mode enabled.

## Normal bundle and native adoption qualified — 3 November 2026

Schema-13 `content/development-gameplay-classic-threat.json` preserves all
schema-12 journal bindings and adds only `creature_threat`. Its 22 roles exclude
the legacy completion-XP recipe. Bounded Rust validation checks the threat policy,
complete distinct prerequisites, role exclusivity and exact bytes. Python snapshots
and revalidates the whole bundle before spawning either child. An individual
`--creature-threat` requires return reset; mixed bundle/profile overrides refuse.
The client reads the policy only to bind expected world identity after reset.
Victim selection, HP, credit, windup and path authority remain server-owned. Older
bundles and the bare walking launcher are unchanged.

Final formatting, workspace/all-target denied-warning Clippy and all 1,074
workspace tests pass (50 service-dependent skips). Forty-four validator, 127
client and 33 Python handoff checks pass separately. The existing 40-contract
owned SQL baseline is inherited; no persistence schema change is introduced.

Actual normal OIDC/PostgreSQL native mouse/keyboard flow at 1920x1200 accepts both
quests, repeats two conversations before acceptance without granting progress,
completes semantic objectives and kills three wolves. Fresh SQL reaches two
completed quests, four actions, nine events, head 13 and 125 XP. Server counters
show 19 resolved damage facts, 300 actual health lost and three threat clears.
After a database process restart, native rejoin retains the same state and
receipts with zero new damage facts, health lost or XP. Both runs have zero GPU
diagnostics. The fresh run records AMD RX 7900 XTX / RADV Vulkan / Mesa 26.2.6 on
Linux; connected display, synthetic PID-bound X11 input, not physical hardware
or Windows execution. The standard UI screenshot was inspected. Test services
stopped and owned plaintext password files were removed.

The private combat fixture retains the earlier low perception/speed and
retaliation-policy damage so this check isolates normal integration and persistence.
Quest/reward definitions are unchanged. This single-native-player proof does not
prove competing native victims; actual competing encrypted peers have their own
three-rate evidence above. Prior wide/small UI geometry is inherited unchanged.
No production population, healing/taunt/social/group threat or streamed-navigation
capacity is claimed. Source, optimized binaries, assets and proof:
`build/creature-threat-client-qualified-v1-20261103`; native proof:
`build/creature-threat-launch-native-v1-20261103`.

With the existing local account-provider configuration, launch explicitly:

```sh
python3 tools/play.py \
  --binaries build/creature-threat-client-qualified-v1-20261103 \
  --gameplay-manifest content/development-gameplay-classic-threat.json \
  --players build/creature-threat-client-qualified-v1-20261103/art/player \
  --creature-asset build/creature-threat-client-qualified-v1-20261103/art/wolf/wolf.glb
```

## Passive body pull and unchanged native profiles — 3 November 2026

A further actual World case enables normal-range perception without any player
attack. At 20/30/60 Hz the NPC acquires the player, registers one zero-threat life,
starts a private windup and resolves a real ordinary retaliation attempt. No
outgoing damage fact, damage threat or tag ownership is manufactured. All 66
affected ordinary/ability authority tests pass after factoring the existing test
fixture's perception parameter; production code and binaries are unchanged.
Formatting and all-target denied-warning Clippy pass. The prior full 1,074-test
workspace result is inherited, with one added focused case.

The normal native qualifier also ran with `--production`: no creature/retaliation
profile override, unchanged canonical schema-13 bindings, quest and reward data.
At 1920x1200 the player completes both quests and all three kills without dying
(final health 48/100), then rejoins after a database process restart. Both SQL
checks retain head 13, four actions, nine events, two completions and 125 XP.
Fresh server counters record 22 damage facts, 300 actual HP lost and three threat
clears; rejoin records zero new facts or damage. Both GPU diagnostic counts are
zero; the fresh Linux adapter capture is RX 7900 XTX / RADV Vulkan / Mesa 26.2.6.
Owned services stop and password files are removed. This is a single-player
native integration check, not competing native targets or population capacity.
The separate encrypted two-peer transfer proof remains applicable.

Preserve `build/creature-threat-client-qualified-v2-20261103`, including unchanged
optimized binaries/assets, final source and proof. Raw new native evidence is
`build/creature-threat-production-native-v1-20261103`; earlier private-fixture
proof is retained separately. The launch command above may use the v2 checkpoint.
