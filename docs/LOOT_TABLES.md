Current item-quest integration, 5 November 2026: opt-in manifest schema 18 now
executes the inventory-bound pelt quest through normal pickup, deletion,
reacquisition, consumption and restart/rejoin. [Evidence and fixture limits](QUEST_ITEM_OBJECTIVES.md#normal-item-quest-qualification-complete--5-november-2026).
Pelts remain ordinary loot; quest-only generation/paging/pickup checks below are
still planned. Earlier live-possession gate statements are historical.

# Creature loot tables

P3-02 / P3-05 / P3-09 / P3-11 / P3-15, 5 November 2026.
The user requested authored chances rather than a quest item on every kill.

## Reference and adopted policy

Reviewed CMaNGOS Classic at revision
[ec18be310645042d085032e276454b36ecef6269](https://github.com/cmangos/mangos-classic/blob/ec18be310645042d085032e276454b36ecef6269/src/game/Loot/LootMgr.cpp).
`LootStoreItem::Roll` applies a drop chance during generation;
`LootTemplate::LootGroup::Roll` selects a group entry. Its
[Player::HasQuestForItem](https://github.com/cmangos/mangos-classic/blob/ec18be310645042d085032e276454b36ecef6269/src/game/Entities/Player.cpp#L13784)
checks an incomplete quest and insufficient required holdings. These are
Classic-targeted emulator rules, not access to Blizzard's server implementation.
The [AzerothCore loot-table reference](https://www.azerothcore.org/wiki/loot_template)
also documents independent chances, exclusive groups and quest eligibility;
AzerothCore's reference is expansion-specific and is supplementary evidence.
No emulator source is copied into Theia.

Adopt independent item chances, optional exclusive groups, server-owned rolls
and explicit pickup. A 60% chance does not guarantee three items after five
kills. A 100% authored entry remains valid. Changing a drop chance is content
balancing, not a change to quest possession/completion rules.

## Authored format and exact semantics

Reward schema 3 extends the existing fixed-quantity recipe with:

- `chance_basis_points`: integer 0 through 10000; 6000 means 60%. Default 10000.
- `drop_group`: integer 0 through 255; default 0 rolls independently. Nonzero
  values identify exclusive groups within one creature definition.
- An exclusive group's chances must total at most 10000. A shared uniform
  ticket selects at most one member; unused chance produces no item. Entries
  occupy contiguous intervals ordered by stable reward ID. Weights are absolute
  chances, not values that are silently normalized. For 3500 + 2500, the remaining
  4000 means no item. Group IDs on different creatures are independent.

Roll metadata is permitted only for per-creature-life recipes. Schema 1/2
preserve their guaranteed fixed recipes and reject these new fields. Quantity
ranges, reference tables, equal-chance fallback entries, rarity multipliers and
party need/greed distribution are not supported by this slice. Fixed `count`
continues to determine each winning recipe's contents.

The grouped integer intervals, basis-point precision, recipient-private loot
and durability policy are explicit Theia choices. They reproduce the adopted
chance/group principles, not every emulator edge case. This rule has no distance
units; corpse positions, interest and pickup remain metric.

## Persistence and authority

The async authority owner calls the retained roll adapter after the existing
trusted kill/recipient checks. The fixed simulation does not wait for SQL or
accept a client-supplied roll, item list or seed. OS entropy uses bounded
rejection sampling to avoid modulo bias.

Migration 13 records one uniform ticket per character, original victim life
and entry/group key. World/profile hashes and the original corpse position are
bound to that record. Winning availability and its roll commit in one SQL
transaction. A miss also commits a record. Duplicate kill facts, lost responses,
reopening a corpse and reconnecting cannot reroll that life. A profile mismatch
or changed corpse position rejects instead of replacing an old result. A winning
claim keeps the existing durable consumed receipt. The older direct guaranteed
offer adapter refuses schema-3 per-life recipes.

Availability is not an inventory grant. Opening the window is read-only; Take
or Loot all sends normal owned pickup commands. Full bags preserve the loot.
Possession credit changes only with committed inventory mutations.

Existing limitation: the world kill queue is not a durable world-event journal;
a crash before an offer/roll is accepted by the persistence worker can lose a
fact. Group members arrive as bounded individual opportunities; successful
members share a stored ticket, but this is not an atomic whole-corpse event log.

## Development profiles and remaining adoption

`content/development-gameplay-classic-creature-loot.json` deliberately retains
schema-2 guaranteed pelt/token drops for repeatable UI qualification.
`content/development-gameplay-classic-random-loot.json` selects schema-3 ordinary
loot: 60% pelt and 25% token, independently, plus the existing once-character
training reward. These are provisional Theia rates, not claimed Classic values.

These pelts are currently ordinary inventory items. Quest-only loot eligibility
must be bound to the admitted inventory/journal profile before a quest-only rule
is enabled. Live possession quests remain gated separately; authoring a chance
does not open that gate. Active-and-short eligibility, deletion/reacquisition
and turn-in reconciliation are the next combined item-quest slice. Do not claim
that this development profile already suppresses drops after a quest is ready.

## Evidence

The owned PostgreSQL qualification at
`build/loot-roll-postgres-v4-20261105/verification.json` passes the new roll
contract alongside existing storage contracts. Sixteen life fixtures produce
one exclusive-group winner apiece and a guaranteed independent miss. Repeated
requests agree with the stored tickets, availability alone preserves the bag,
pickup/consumed replay preserve one grant, changed position/profile reject and
reconnecting preserves a miss. All 32 roll rows have identical full-row hashes
before and after an actual PostgreSQL 16.15 process restart. The trusted
kill/recipient facts in this storage contract are fixtures; it is not live
combat, encrypted transport, native interaction or a capacity measurement.

## Next quest-only eligibility slice — planned

Keep roll generation separate from current eligibility. The pinned Classic
reference's `LootItem::AllowedForPlayer` uses the current quest-item check when
accessing loot. Theia should retain the rolled candidate, then authoritatively
filter quest-only visibility and pickup against the admitted journal and bag.
Deleting an item may make an existing winning candidate eligible again; it must
never change a recorded losing roll. Ordinary drops stay independent of quests.

Bind exact quest/item profiles before enabling a quest-only flag. Check missing,
completed and sufficient-holdings cases, stack split/merge invariance, destruction
and reacquisition, concurrent pickup/turn-in, rejoin and stale request rejection.
Apply eligibility before the bounded nearby page limit so hidden quest-only
claims cannot displace ordinary nearby loot. Inventory/journal commits must
refresh the relevant private availability view. These are acceptance requirements,
not behavior implemented by the current ordinary-loot profiles.

Combined implementation checks pass 1,295 workspace/all-target tests, with 65
service-dependent skips, formatting, Python compilation and denied-warning
workspace/all-target Clippy. Logs:
`build/loot-roll-workspace-tests-v2-20261105.log`,
`build/loot-roll-fmt-v2-20261105.log` and
`build/loot-roll-clippy-v1-20261105.log`. The full build used two jobs after the
initial default-concurrency build was interrupted safely when parallel linking
filled swap. The separate actual PostgreSQL run passes 55 storage contracts;
service skips are not represented as executed tests.


Actual local OIDC/encrypted-ticket/PostgreSQL owner proof passes at
`build/loot-roll-owner-v1-20261105/verification.json`. Its schema-3 100% fixture
and repeated 0% life produce 19 distinct roll records, 18 winning claims and no
advertised missed loot. Retained movement, pickup across retirement and rejoin
preserve the original results. It drives real owners with trusted synthetic
facts, not the World kill adapter or native random drops. Owned services stop.
The actual-validator launcher suite also passes all 38 tests, including exact
random-recipe bytes and explicit pickup mode; see
`build/loot-roll-bundle-tests-v1-20261105.log`.

Optimized Linux client/server build passes. Preserve
`build/quest-loot-qualified-v1-20261105` for the combined source, binaries, art
and evidence references. Its native presentation proof is the earlier guaranteed
fixture with identical client source; the new ordinary random-loot path has
math, actual SQL/owner and manifest validation evidence, not a claimed native
random-drop walkthrough. Live quest-only eligibility remains the next slice.

## Quest-only eligibility core prerequisite — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15: `Journal::needed_possession_items` now derives a
bounded, unique ordered item set from accepted unfinished quests and the supplied
checked current backpack. It excludes unaccepted/completed quests, excludes
satisfied quantities and never changes the journal, inventory, roll or receipt.
Splitting/merging without changing holdings preserves eligibility; destruction
can restore it even when supplied journal counters still say Ready. Several
quests needing the same item share holdings rather than adding their quantities.
Unknown item references and foreign backpacks refuse.

Explicit Theia stage policy: accepted quests may collect their final-stage
possession items before reaching that stage. Eligibility does not satisfy earlier
kill, interaction or area stages. More restrictive staged drop conditions need
separate authored support. Equipped items and future bank storage remain outside
the current backpack predicate, consistently with possession reconciliation.

343 core tests, formatting and strict workspace/all-target Clippy pass; logs are
`build/quest-loot-eligibility-core-tests-v2-20261105.log`,
`build/quest-loot-eligibility-fmt-v1-20261105.log` and
`build/quest-loot-eligibility-clippy-v1-20261105.log`.
These are pure checked proposals, not new SQL, encrypted or native evidence.
No runtime gate, packet or content schema changes here: quest-only entries and
live possession quests remain refused until exact profile binding, eligibility
before nearby paging, pickup checks and committed refresh are installed together.
Ordinary schema-17 corpse pickup and random drops remain the separately qualified
`4dbeefd` checkpoint. Continue implementation with local commits only.

### Quest-only generation rights require a separate decision

The pinned CMaNGOS source distinguishes more than the chance roll and current
item need. `Loot::FillLoot` assigns each generated non-chest item's allowed-player
set using `AllowedForPlayer`; `LootItem::IsAllowed` subsequently consults that
stored set for corpses. See [LootMgr.cpp at ec18be31](https://github.com/cmangos/mangos-classic/blob/ec18be310645042d085032e276454b36ecef6269/src/game/Loot/LootMgr.cpp)
(`FillLoot`, approximately lines 960–1000; `IsAllowed`, approximately 600–607).
This does not establish that accepting a quest later should unlock an old corpse.
The current pure needed-items helper establishes no generation rights.

Recommended next integration policy, not implemented: require an accepted quest
and a quantity shortfall when the trusted generation opportunity is first
committed; persist that decision alongside its single roll. Then recheck current
need before visible paging and pickup. A previously entitled winning candidate
may become visible again after deletion, but an ineligible generation or losing
roll cannot become a new entitlement on replay, opening or later acceptance.
Theia's strict current-need pickup guard is an explicit policy beyond the cited
emulator's stored corpse permission set. Current development pelts remain ordinary
loot. Validate commit ordering across the inventory and journal owners before
admitting any quest-only profile; do not infer death-time eligibility merely from
an asynchronous worker's later observation.


## Quest-required metadata prerequisite — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15: core reward schema 4 adds optional boolean
`quest_required` (default false). A true entry must be per-creature-life, grant
one known item type and use independent chance (`drop_group` zero). Initial
single-item/independent restrictions are deliberate Theia scope, not a claim that
Classic cannot use other configurations. Older schemas refuse the new metadata.
Ordinary schema-1/2/3 recipes retain their previous behavior.

`Definition::quest_item` exposes the one conditional item and
`Catalog::has_quest_required` classifies the catalogue. These are metadata, not
acceptance, current possession, generation rights or a grant. Ticket/chance
classification remains identical for ordinary and conditional entries, including
the exact 60% boundary; eligibility must not reroll it.

Until the actual integration is complete, all wire/inventory reward profile
constructors, storage reward profile construction and normal gameplay validation
refuse quest-required entries. The core parser accepting schema 4 does not expose
them to an ordinary corpse page or permit pickup. No catalogue on disk changes
and no packet or database schema changes in this prerequisite.

345 core tests, ten reward-wire tests, four non-SQL persistence-library tests and
56 gameplay-validator tests pass. Formatting and denied-warning workspace/
all-target Clippy pass. Logs:
`build/quest-required-all-core-tests-v1-20261105.log`,
`build/quest-required-wire-tests-v1-20261105.log`,
`build/quest-required-persistence-lib-tests-v1-20261105.log`,
`build/quest-required-validator-tests-v1-20261105.log`,
`build/quest-required-fmt-v1-20261105.log` and
`build/quest-required-clippy-v1-20261105.log`.
Preserve `build/quest-required-core-qualified-v1-20261105`. No new actual SQL,
owner, encrypted peer, native/Windows or capacity evidence is claimed.
Normal inventory-bound item quests remain the separately qualified `85fd967`.
Next: exact item/journal/reward binding, durable initial generation rights,
eligibility before bounded nearest paging and a fresh locked pickup check. Keep
server commit ordering explicit; a later async view is not a death-time snapshot.
Commit locally only; no push.


## Quest-only profile binding prerequisite — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15. Explicit wire and persistence
`from_json_with_quest_journal` constructors now check a conditional recipe against
the exact inventory-bound journal. Every conditional grant item must occur in a
possession objective. Unbound/foreign item profiles, unreferenced conditional
items and unused bindings on ordinary recipes refuse. These are immutable data
constructors, not runtime installation.

Identity is acyclic: raw items → inventory-bound journal (including quest, XP and
optional area source identity) → conditional reward profile. The final reward
hash uses `theia-quest-loot-profile-v1\0`, its existing item/reward byte hash and
the exact journal hash. Wire/storage hashes agree. Changing even whitespace in
items, quests, XP, areas or rewards changes the bound identity; old ordinary
hashes remain unchanged. No mutable quest state enters the content identity.

Creating a bound profile does not bypass the integration gates. Both World reward
installation paths and the retained inventory owner refuse conditional content.
All legacy Store offer/roll/list/pickup paths refuse before database access;
a disconnected Store test covers those public paths, including compound journal
pickup. Normal manifests and old profile constructors remain closed. This slice
adds no SQL, packet, source catalogue or native behavior.

Eleven server-library tests, five non-SQL persistence-library tests, ten reward-
wire tests, 56 gameplay-validator tests and 13 focused reward-core tests pass.
Formatting and denied-warning workspace/all-target Clippy pass. Evidence:
`build/quest-loot-binding-server-tests-v1-20261105.log`,
`build/quest-loot-binding-persistence-lib-tests-v1-20261105.log`,
`build/quest-loot-binding-wire-tests-v1-20261105.log`,
`build/quest-loot-binding-validator-tests-v1-20261105.log`,
`build/quest-loot-binding-core-tests-v1-20261105.log`,
`build/quest-loot-binding-fmt-v1-20261105.log` and
`build/quest-loot-binding-clippy-v1-20261105.log`.
Preserve `build/quest-loot-binding-qualified-v1-20261105`. No new actual SQL,
encrypted-owner, native/Windows or capacity claim. The normal schema-18 playable
remains `85fd967`. Next: durable first-generation eligibility, then eligibility
before bounded paging and a fresh locked pickup check. Keep the generation
ordering policy explicit rather than claiming an asynchronous observation was
captured at the exact death tick. Commit locally only; no push.
