# Combat membership — life-bound reasons and opt-in World

P3-04 / P3-06 / P3-07 / P4-08. The latest playable checkpoint remains
`build/classic-ability-client-qualified-v1-20261102`. This prerequisite supplies a
bounded pure contract, followed by an explicitly opted-in headless World slice.
The normal manifest/client indicator candidate is under native qualification.
Regeneration, resting, logout, mounting, assistance, tags and rewards have not
adopted membership. Those consumers require separate
acceptance. Selection and a running swing timer are not membership inputs.

## Membership and lifetime rules

`theia_core::combat_membership` owns one player's exact `LifeKey` and at most
32 opposing entity lifetimes. An authority-approved outgoing/incoming hostile
attempt supplies an independently expiring reason. Avoidance or zero damage
can still be hostile; rejected requests and mere targeting cannot enter combat.
A separate active-encounter reason keeps combat while an NPC pursues/attacks,
without fabricating damage or refreshing expired hostility reasons.

`advance` expires transient reasons at the exact fixed tick, even on idle steps.
Releasing the active-encounter reason retains unexpired hostility; confirmed
resolution, evade, retirement or disconnect can explicitly forget that exact
opponent and all its reasons. Ending the player's life clears membership and
refuses subsequent mutation. A fresh player life needs a fresh state, never a
copied old ledger. Stale owner life, backwards ticks, foreign world, self-target,
capacity and clock overflow fail atomically. Same-tick valid inputs are allowed.

Opponent keys must identify authoritative lifetimes. Current NPC respawns have
fresh entity keys; a future PvP adapter must explicitly retire old player lives.
This contract does not infer player-opponent life identity from a session key.
It is not a threat table, tag ledger, permissions service or replicated HP state.

The strict schema-1 `development-combat-membership.json` candidate is absent from
all manifests. Its `combat_membership_fixture_v1` policy sets a 32-opponent cap
and 5,000 ms hostility grace. Both are explicit Theia fixtures, not verified
Classic balance or an adopted global logout/regeneration timer. The parser is
bounded to 4 KiB, caps opponents at 32 and requires 100–60,000 ms grace. There is
no new dependency, service, wall-clock/I/O work or spatial-unit conversion.

## Acceptance and next slice

Seven focused tests pass: independent exact expiry at 20/30/60 Hz, a two-minute
encounter pin without damage, reason-specific release, exact-opponent retirement,
new/ended-life isolation, bounded capacity/reuse and complete-state refusal
atomicity, adversarial distinct targets, and profile schema/value/byte bounds.
All 250 core tests, workspace/all-target denied-warning Clippy and formatting
pass. Exact candidate source/proof:
`build/combat-membership-core-qualified-v1-20261102`.
No native GPU check is needed for this inactive renderer-independent module.
Network/client counts and native evidence remain those of the playable checkpoint.

The subsequent opt-in World source is described below; normal application and
combat-gated gameplay remain pending. The preserved core-only artifact records
its earlier boundary independently.

## Opt-in single-creature World — 2 November 2026

`World::enable_combat_membership` accepts the bounded profile before any player
admission or tick, requires the explicit ability/ordinary/player-life/pursuit
context and one typed-damage creature, and refuses unsupported status effects.
The exact bytes add a separate World-identity wrapper immediately after ability
damage, before rewards/progression and the final Classic wrapper. Duplicate,
invalid, missing-context and late activation fail without changing identity.
The normal manifest, launcher and client have no membership opt-in yet. The
World-only prerequisite originally supplied no combat codec; transport adoption
is recorded separately below.

Valid outgoing specials, Arcane impacts, white swings and incoming NPC white
swings preview the player's bounded membership state before health, private RNG
and retaliation commit. Valid avoidance still enters combat; ability start,
projectile launch, selection, rejected requests and a paused autoattack do not.
Projectile impact retains the existing source-life validation. Membership failure
refuses the hit before HP/RNG mutation. States exist only for admitted live
players, with each exact life separate; this is not a PvP adapter.

Each fixed step synchronizes actual pursuit and expires hostility, including
steps without attacks. Only the NPC's retained Chase/Attack target gets the
active-encounter reason. Another player's recent hostility can expire while the
first player remains engaged; no threat-switching rule is invented. Confirmed
Return/Blocked/Dead, creature death/retirement and replacement clear the correct
opponent. Player death/life replacement and disconnect remove the old player's
ledger. The server-only query validates the current live life and returns no
observation on authority error. No consumer uses that absence as permission.

Nine focused World checks pass, including 20/30/60 Hz selection/paused/rejected
and Arcane-impact cases, deterministic avoided specials, actual pursuit/incoming
swings, blocked-route reset, NPC kill/fresh respawn, player life/disconnect,
two-player isolation, exact-byte identity/admission and near-exhausted deadlines
refusing all three hit paths before health/RNG commits. These are headless
checked-terrain/Avian fixtures, not authenticated multiplayer or native UI checks.
All 250 core / 597 network (ten skips) / 123 client tests, formatting and
workspace/all-target denied-warning Clippy pass. Exact source/proof:
`build/combat-membership-world-qualified-v1-20261102`. New-profile encrypted
UDP/replicated observations are not yet qualified; existing UDP regression
exercises the preserved older profiles.
The normal playable checkpoint remains unchanged. Native GPU checks are not
required for this headless opt-in; transport observation and normal/client
adoption need their own acceptance before gating gameplay.


## PL02 and encrypted transport qualification — 2 November 2026

P1-08 / P3-04 / P3-06 / P3-07: the explicit `combat_membership_wire` profile
checks the core policy bounds before hashing exact bytes. PL02 extends the
existing owner-only life frame with that policy hash, opponent count and three
reason bits. It is 120–144 bytes, retaining the PL01 health, actor/life, tick,
movement-stream, death/respawn and windup validation. Unknown/reserved bits,
contradictory count/reasons, combat on a dead life, wrong policy and malformed
frames fail. PL01 continues to refuse any membership field; neither receive
mode silently accepts the other version.

The server selects PL02 only for an opted-in World. Health and membership are
captured together at the end of the fixed step. A membership authority error
halts publication; it never manufactures an out-of-combat permission. The
existing owner-private reliable channel and capacity-16 presentation queue carry
the composite observation. Full/closed queues disconnect before advancing
retained receive state. Duplicates/stale observations consume no queue slot.
Clients require the exact membership policy plus explicit ability-damage mode.
A combat-only update leaves the prediction correction barrier unchanged;
death/life/stream changes retain their existing barriers.

Six codec tests, two actual encrypted UDP tests and a critical-queue test pass.
At 20/30/60 Hz, two admitted peers under 20 ms delay / 30 ms jitter, periodic
loss/duplication and observed reordering receive matching public ability facts,
but only the fighting player receives in-combat membership and private incoming
white damage. Membership clears on the second-hit kill and ordinary movement
prediction remains permitted. The test deliberately uses a 20-HP NPC and a
zero-avoidance/noncritical special, with synthetic committed inventory and
fixture admissions; this is not normal balance or OIDC/PostgreSQL evidence.
Both PL01/PL02 receive-mode mismatches disconnect without partial publication.

All 250 core / 606 network (ten skips) / 123 client tests, formatting and
workspace/all-target denied-warning Clippy pass. Exact source/proof:
`build/combat-membership-transport-qualified-v1-20261102`.
Normal manifests, launcher/server/client flags, the visible combat indicator,
OIDC/PostgreSQL and native checks remain next. No gameplay consumer is gated by
this candidate; the qualified playable checkpoint remains preserved.


## Normal application qualification — 2 November 2026

P2-14 / P3-04 / P3-06 / P3-07: schema-9
`development-gameplay-classic-membership.json` adds `combat_membership` to the
nineteen-role ability bundle. Rust validation binds all twenty exact-byte roles,
requires the complete supported schema/context, and checks the 4 KiB policy
budget, rule version and bounds. Three additional validator tests pass (34 total).
The normal launcher snapshots and forwards `--combat-membership` to both server
and client; missing ability/life/classic context refuses before children start.
Two additional Python checks cover staged forwarding and unbound refusal.

Server/client loads are bounded and use the same exact policy identity after
ability damage and before later rewards/progression and the final Classic
wrapper. Both require explicit compatible life/ability profiles; offline mode
cannot opt in. The normal player's reliable life mode selects PL02 with the
same immutable policy. Older manifests remain PL01.

Standard Bevy UI now displays “In combat” or “Out of combat” in the existing
health panel, with a compact incoming-attack countdown. Death, respawn barriers,
loading and disconnect retain their own statuses. The UI reads the authoritative
own-life observation, never selection or autoattack intent. Entries/exits and
nonempty laid-out combat/peace glyph frames are recorded for native validation;
these are presentation observations, not permissions or new gameplay authority.
The native helper's `--membership` mode selects schema 9 and requires real combat
entry and post-kill release, or sustained contact in the separate passive check.

All 250 core / 609 network (ten skips) / 123 client / 45 Python checks,
formatting, workspace/all-target denied-warning Clippy and release pass.
Seven normal OIDC/PostgreSQL native cases pass: controls/casting and unmodified
wolf fights at actual 1920x1200, 3440x1440 fullscreen and 1280x720, plus a separate
35-second passive melee check. Every completed fight records authoritative
entry/release, visible nonempty combat/peace glyphs, zero returns, a typed ability
killing hit, one durable XP commit and one loot offer. The passive check records
one entry, zero exits, one acquisition and zero returns without outgoing swings
or abilities. An earlier interrupted attempt included user-supplied abilities;
it remains outside the qualified proof. The helper now rejects outgoing
abilities as well as autoattacks during passive qualification.

Screenshots were inspected at 1920x1200, ultrawide and 1280x720. Combat-event gaps
and GPU diagnostics are zero. Device: Linux AMD Radeon RX 7900 XTX / RADV Vulkan /
Mesa 26.2.6, connected display; ultrawide is fullscreen. No native Windows,
loot-claim or MMO-capacity claim. Exact source/binaries/public proof:
`build/combat-membership-client-qualified-v1-20261102`.
No new resource, mounting, resting, logout or reward rule is gated by the
indicator. First-effective-hit ownership and separate credit interfaces follow;
normal rewards still use the preserved last-hit development policy.
