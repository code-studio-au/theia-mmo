# Ordinary creature ownership and credit — 2 November 2026

P3-09 / P3-10 / P3-11 / P4-08. Prerequisites: typed authoritative outgoing
ordinary/special/spell health-loss facts, exact player life and NPC lifetime,
central combat membership, and the existing durable economy adapters. This is a
pure-core prerequisite; normal rewards still use the qualified last-hit fixture.
No gameplay, transport, account or persistence behavior changes in this slice.

## Adopted direction and explicit fixture policy

Follow the first qualifying effective damaging contribution decision in
[DECISIONS](DECISIONS.md). Tag ownership, contributions and downstream XP, quest
and loot decisions remain distinct. This implements Theia's selected policy,
not a verified reproduction of every Classic party/range/death rule. Distances
remain metric; this slice has no dimensional coefficient or range conversion.

`development-creature-credit.json` declares schema 1,
`creature_credit_fixture_v1` and a bounded maximum of 32 contributors. Strict
parsing rejects unknown fields, unsupported versions, invalid content IDs,
zero/oversized capacity and input over 4 KiB. No dependency is added.

`theia_core::creature_credit::Ledger` accepts trusted resolved damage facts only.
The first positive health loss tags the exact NPC encounter to the player's
session identity; avoided, prevented, overkill and zero effective damage cannot
establish ownership. A competitor's fatal hit does not transfer ownership.
Contributions aggregate actual health lost separately from ownership. A later
policy must decide eligibility and distribution; the ledger grants no entitlement.

White-melee and ability damage have separate source-life sequence spaces.
Duplicates/backward sequences and earlier source lives refuse atomically.
Connected player death alone does not clear ownership; a newer life contributes
to the same session total. Durable character IDs belong to the existing admission
adapter outside the simulation. This is not a cross-session/account claim.

Authoritative evade or tag-owner departure abandons the encounter and clears its
claim/contributions. This is an explicit initial Theia departure policy. A checked
encounter generation advances so delayed old facts cannot retag the reset NPC.
NPC respawn requires a fresh entity key and a fresh ledger. Finishing a confirmed
death yields one immutable ownership/contribution fact; repeated finishes refuse.
An untagged death produces no implicit owner. Death facts do not acknowledge XP,
quest or loot success; their separate durable transactions remain necessary.

A future World adapter must validate source admission/liveness and the current
encounter, convert immutable resolved damage, preflight a bounded ledger clone
before committing health/RNG, and commit both together. The ledger itself cannot
authenticate callers, prove death or infer final-hit eligibility. All refusals,
including full contributor capacity and counter exhaustion, leave it unchanged.
Zero damage allocates no slot and advances no sequence or observation watermark.
The authority must not reinterpret a previously immutable zero-damage fact as a
later positive-damage event with the same identity.

## Qualification and remaining acceptance

Eight focused checks cover competing hitters, zero damage, independent event
namespaces, duplicate/old-life rejection, connected-life replacement, capacity,
reset generations, fresh NPC identity, stale ticks/epochs/self hits, once-only
finish, untagged death and checked damage/generation exhaustion.
All 258 core tests, formatting and workspace/all-target denied-warning Clippy
pass. Source/proof: `build/creature-credit-core-qualified-v1-20261102`.
Network/client baselines are inherited from the preserved playable checkpoint;
this headless prerequisite has no rendering change or new native GPU claim.

Next: opt-in World adoption with pre-admission exact policy identity and atomic
ordinary/ability damage integration; competing-player, zero-damage, reset,
disconnect and respawn tests at 20/30/60 Hz. Then bind separate XP/quest/loot
eligibility adapters and actual durable competing-player acceptance before
replacing the normal last-hit fixture. Party/shared-target eligibility, range,
level filtering and disconnect/reconnect claims remain separate work.

## Opt-in World qualification

P3-09 / P3-10 / P3-11 / P4-08: pre-admission `enable_creature_credit` requires the
explicit typed-damage/membership World. The exact validated policy bytes wrap
World identity after membership and before later reward/progression wrappers.
Older Worlds, manifests and normal rewards remain unchanged.

Ordinary outgoing melee and typed Sword/Arcane resolution preview a bounded
ledger candidate before health, private damage RNG, retaliation, membership and
combat facts commit. Incoming NPC damage cannot tag the creature. A fatal hit
finishes one immutable ownership/contribution fact retained for the exact NPC
life; this is available to later policy adapters, with no grant or durable I/O
in the simulation step.

A server-issued resolution serial, separate from cast/request and autoattack
sequences, supplies contribution ordering. A legitimate slow spell can land
after a later instant cast. Cast start captures the encounter generation; that
capture travels with the projectile through impact. Owner departure and pursuit
Return/Blocked clear the ledger and advance its generation before damage runs.
Old in-flight contributions refuse without health/RNG or fact commits. This
opt-in refuses outgoing damage while Return/Blocked so a new attack cannot
immediately restore a cleared claim; older generic damageable-return fixtures remain preserved; ordinary/typed
paths already rejected return damage independently of credit. This is an explicit initial Theia reset boundary, not full
Classic evade/heal/immunity behavior. A fresh NPC life starts an empty ledger.
Connected player death alone keeps the session claim; a subsequent pursuit reset
can independently abandon it. Fatal facts remain available after owner departure
until NPC replacement; downstream eligibility remains a separate decision.

Twelve focused checked-terrain/Avian tests pass, including competing-player
ownership, zero-damage hostility, actual projectile impact and delayed old flight,
white incoming/outgoing distinction, exact identity/admission, atomic capacity
refusal, reset, life replacement and fresh NPC respawn. Multi-rate cases use
20/30/60 Hz. Specials deliberately have zero avoidance/critical probabilities
(or forced misses in the zero-damage case); deterministic Arcane draws retain
its existing landing rules. These are fixture World tests, not normal balance,
authenticated transport, native competing-player or durable reward evidence.
All 258 core / 621 network (ten skips) / 123 client tests, formatting and
workspace/all-target denied-warning Clippy pass. Source/proof:
`build/creature-credit-world-qualified-v1-20261102`. Existing UDP checks regress
older modes; new-profile encrypted/durable competing-player acceptance remains.
No new transport or normal launcher/client flag is adopted yet.

The next recipient slice must keep ownership and eligibility distinct: an
ineligible or departed owner must not silently hand rewards to the last hitter.
Bind an explicit versioned solo-recipient policy before changing adapters. The
initial quest currently shares the XP kill transaction/queue; that coupling must
be stated, with party/shared-quest distributions deferred until independently
versioned eligibility and receipts exist. Existing loot offers are nonrepeatable
training rewards, not a general per-corpse loot system. Preserve those durability
and consumption guarantees rather than relabeling them as completed MMO loot.

## Explicit solo-recipient qualification

Schema 2 `development-creature-credit-solo.json` declares
`solo_tag_credit_fixture_v1` with `recipient: solo_tag_owner`. Schema 1 remains
ownership-only and retains last-hit reward adapters. Missing/null policies,
wrong schema/ruleset pairs and unsupported recipients reject; no silent upgrade.
Both modes retain the bounded contribution and encounter contracts above.

The new mode retains the exact immutable fatal World event alongside the
ownership fact. XP and training reward adapters accept only that event for the
current creature lifetime, resolve its tag owner, and require the same admitted
session. A competitor's fatal hit changes neither recipient. Changed actor,
life, sequence or damage facts, nonfatal observations and old NPC lives cannot
authorize it. No current owner means no fallback to the final hitter. A connected
dead owner remains eligible in this initial Theia fixture; party, level/range and
shared-target eligibility are unimplemented. Existing accepted-quest rules still
apply in the XP/quest transaction. These remain coupled for this solo fixture;
independent quest-sharing distributions require a later versioned interface.
Training reward offers remain nonrepeatable per character/recipe, with no new
per-corpse loot system or loot-consumption claim.

Six focused World recipient checks and one actual encrypted UDP test pass. The
UDP test uses two separately bound synthetic XP/reward owner ports at 20/30/60 Hz,
20 ms delay / 30 ms jitter, periodic loss/duplication and observed reordering.
Only the first effective hitter receives one XP kill and one training offer;
the final hitter receives neither. Both see the same typed public combat facts.
Queueing does not publish committed XP: the synthetic owner still reports zero.
These are deliberately separate from PostgreSQL and normal OIDC/native evidence.
All 259 core / 628 network (ten skips) / 123 client tests, formatting and
workspace/all-target denied-warning Clippy pass. Source/proof:
`build/creature-credit-recipient-qualified-v1-20261102`. An owned
PostgreSQL/public-World qualifier follows to check actual commit, replay and
reload boundaries. No normal launcher/client adoption or new durable/native
claim is made by this checkpoint.


## Owned World/PostgreSQL qualification

`apps/server/examples/creature_credit_durable_qualification.rs` constructs the
public saved-terrain/Avian World with checked baked navigation and the explicit
solo policy. It drives two player sessions: one establishes the first effective
tag and the other lands the killing Arcane impact. Creature speed/perception and
an unequipped, zero-cost Arcane catalogue are explicit fixture data; no private
equipment setter or normal equipment rule is exposed. Synthetic accounts and
trusted Store quest acceptance are used. This does not claim OIDC, normal NPC
acceptance, encrypted transport or native player evidence in this process.

Actual World XP/training opportunities select the first player. The qualifier
maps its session to a created character and submits work through the existing
asynchronous persistence Publisher/Worker. A held XP row lock prevents completion
while ten further fixed steps advance. After release, XP and accepted-quest count
commit atomically; the training offer has its separate existing durable receipt.
Replaying both paths yields no duplicate. A fresh Store connection verifies
owner XP 25 / quest count 1 / one unconsumed claim and final hitter 0 / 0 / 0.

`tools/qualify_creature_credit.py` requires a new owned output directory, the
existing disposable PostgreSQL SDK and checked terrain/navigation. It verifies
fsync, synchronous_commit and full_page_writes, then restarts PostgreSQL after
the example exits and reads recipient state and receipt counts again. Password
files are removed on exit. The initial run remains preserved. The final run in
`build/creature-credit-durable-v2-20261102` additionally rejects a wrong-character
replay without partially changing its XP/quest state. Formatting, workspace/all-
target denied-warning Clippy, the server test, qualifier build and Python compile
pass. Core/network/client code is unchanged from the 259 / 628 (ten skips) / 123
recipient checkpoint; those counts are inherited, not rerun for this example.
Exact source, executable and public proof:
`build/creature-credit-durable-qualified-v1-20261102`.
Loot consumption, general corpse loot and normal scene adoption remain separate.

## Normal application qualification

P2-14 / P3-04 / P3-09 / P3-10 / P3-11 / P4-08: schema-10
`development-gameplay-classic-credit.json` adds the explicit `creature_credit`
role to the twenty-role membership bundle. Rust validation requires the complete
schema/context, exact distinct files and a supported bounded solo-recipient
policy. Normal application loading refuses the ownership-only schema-1 policy.
Older manifests retain their behavior.

The normal launcher snapshots and forwards `--creature-credit` to server and
client. Both load at most 4 KiB plus a refusal byte and validate the exact policy;
client identity and authoritative World bind it immediately after membership,
before reward/progression and later Classic wrappers. Membership's existing
secure/predicted/classic context remains required. No new packet or cosmetic
inference of ownership is introduced: PL02 life/combat and CE08 typed damage
remain the existing transport contracts. XP, accepted-quest count and training
offers use the versioned authoritative solo-recipient adapter.

Three validator and two Python cases cover twenty-one-role staging, exact byte
hashing, complete context, unsupported/null policies, aliasing and byte/capacity
bounds. The native helper's `--solo-credit` mode selects the new bundle while
retaining its actual OS controls, combat/peace glyph, typed kill, durable XP/offer,
GPU diagnostic and passive-contact checks. All 259 core / 631 network (ten
skips) / 123 client / 47 Python tests, formatting, workspace/all-target denied-
warning Clippy and release pass. The legacy status UDP regression now uses an
explicit nine-second fixture and refreshes the remaining source before source
departure, distinguishing retirement from natural expiry under packet loss.
Normal status duration/data is unchanged.

Seven normal OIDC/PostgreSQL native cases pass: controls/casting and unmodified
wolf fights at actual 1920x1200, 3440x1440 fullscreen and 1280x720, plus a 35-second
passive contact check. Every full fight has one authoritative entry/exit, zero
returns, a typed ability fatal hit, one durable XP commit and one training offer.
Passive contact has one entry, zero exits, zero returns and no outgoing attacks.
Combat/peace glyphs render; combat-event gaps and GPU diagnostics are zero.
Captures were inspected at all three sizes. Device: Linux AMD Radeon RX 7900 XTX /
RADV Vulkan / Mesa 26.2.6, connected display. The reported user interruption did
not invalidate the assertions; no native rerun was needed.

Exact source, four normal executables, art and public proof:
`build/creature-credit-client-qualified-v1-20261102`. Earlier checkpoints remain
preserved. Competing-player routing and PostgreSQL restart/replay are covered
separately above; the native cases use one player. No native Windows, tag-color
UI, party/shared eligibility, loot consumption, general per-corpse loot or
MMO-capacity claim. Combat permission consumers remain ungated. Follow with
bounded encounter threat/reset before expanding NPC population.
