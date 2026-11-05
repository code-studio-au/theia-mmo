Normal quest-only loot qualified locally, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: opt-in manifest schema 19 binds the exact item
journal before conditional loot on client/server and selects the retained owner.
Linux native pickup, split/delete reversal, replacement pickup, consumption and
PostgreSQL restart/rejoin pass at 1280x720, 1920x1200 and 3440x1440. First three
unaccepted lives stay denied; four later lives retain their original rights.
57 validator, 41 bundle, 147 client and eleven server tests, formatting, strict
Clippy and optimized builds pass. [Evidence and limits](LOOT_TABLES.md#normal-quest-only-loot-qualified--5-november-2026).
The native fixture guarantees loot and uses low damage; ordinary random gameplay
content remains 60% pelt / 25% token. Continue readable quest documents next;
commit locally only, no push. Earlier gate/stop entries describe prior checkpoints.

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


## Quest-only generation storage prerequisite — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15. Explicit trusted
`Store::generate_quest_loot_for_kill` now accepts the exact bound conditional
profile. For a conditional entry, it locks the checked backpack before the
journal, derives current accepted unfinished-quest shortfalls, and persists that
first generation decision alongside the one roll. Generation itself changes
neither items, quest counters nor XP. Winning eligibility creates only a claim.
The generic Store offer/roll/list/pickup methods still refuse conditional
profiles; worker, retained owner, World and normal manifest adoption stay closed.

Theia ordering policy: this is the first committed generation transaction,
serialized with bag/journal mutations, not a snapshot claimed to have been taken
at the simulation's fatal tick. Acceptance committed before that transaction can
qualify; acceptance afterward cannot unlock its denied result. A life generated
while enough items are held remains ineligible after later deletion; the player
must obtain a previously entitled winning candidate or a new qualifying kill.
A missing/uninitialized journal is an error and creates no denied receipt.
This policy follows the separation of initial corpse rights from need in the
cited Classic-targeted source while explicitly adapting the async boundary.

Migration 14 adds `generation_eligible` and nullable bag/journal audit revisions
to loot rolls. Existing ordinary rows default to eligible with no quest snapshot;
conditional rows require both nonnegative revisions, including denied results.
The snapshot is recorded at initial insertion. Replay validates the original
world/profile/life/metric source and recovers its stored rights and ticket rather
than replacing them with current need. A denied generation or losing roll stays
empty. Winning claim and roll/snapshot share a transaction; forced claim-insert
failure rolls all of them back. Grouped ordinary rolls retain their earlier
behavior; conditional entries remain independent single-item recipes.

Actual owned PostgreSQL 16.15 proof:
`build/quest-loot-generation-postgres-v3-20261105/verification.json` passes 57 SQL
contracts, five non-SQL library tests, existing example reloads and actual database
process restart. All 38 full roll rows retain their hash across restart, including
five conditional snapshots and the ordinary migration fixture. The new tests
cover empty/unaccepted, accepted/needed, already-held/Ready, deletion reversal,
zero-chance miss, replay, source-position mismatch, missing journal, reconnect,
rollback after claim-insert failure and rejected incomplete audit metadata.
The migration fixture restores a real version-13 ordinary row, reapplies migration
14 and recovers the identical ticket and claim. v1 is retained as negative
fixture evidence: pre-held acceptance reconciles a second journal head, so its
initial hard-coded audit expectation was corrected; v2 passes, and v3 adds the
fault/constraint checks. The generation and migration tests use explicit 100%/0%
fixtures, not native sampling of provisional drop percentages.

Formatting and denied-warning workspace/all-target Clippy pass at
`build/quest-loot-generation-fmt-v2-20261105.log` and
`build/quest-loot-generation-clippy-v2-20261105.log`. Preserve
`build/quest-loot-generation-qualified-v1-20261105`. No new worker/encrypted-owner,
live death-to-generation, native/Windows or capacity claim. Accepted async
opportunity durability before worker acceptance and whole-corpse generation
atomicity remain outside this slice. The normal schema-18 playable remains
`85fd967`; no source gameplay catalogue changes. Next: eligibility before bounded
nearest paging and a fresh locked pickup check, then worker/owner/native adoption.
Commit locally only; no push.


## Quest-only paging and pickup storage prerequisite — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15. New explicit trusted Store operations
`list_quest_reward_claims_nearby` and `pickup_quest_reward` use the exact bound
conditional profile. The generic Store operations remain closed. Worker, retained
owner, World and normal manifest admission are not installed for conditional loot.
No source gameplay catalogue, wire format or renderer changes in this slice.

The view locks one checked bag/journal pair and compiles at most 16 catalogue IDs
into allowed ordinary/currently-needed conditional entries. SQL applies that
filter before the exact 3D nearest-16 pending-life limit. Once-character history
stays separately bounded and includes consumed receipts. Unknown IDs deliberately
reach checked decoding rather than disappearing in the filter. A single joined
query retrieves each selected conditional claim's generation receipt; its world,
profile, life, metric source, winning ticket, eligibility and audit revisions must
validate. Ordinary repeatable and once-character entries retain their semantics.

Pickup locks the checked backpack and claim, validates original conditional
rights/ticket, and checks current accepted unfinished-quest need before applying a
fresh grant. `RewardNotNeeded` rejects an outdated visible intent without changing
the bag, journal or receipt. A winning candidate generated while eligible can
reappear after deletion; denied generations do not gain rights. Completion hides
leftovers even after turn-in consumes all pelts. The exact latest successful
pickup retry is recovered before the fresh-need guard: becoming Ready must not
invalidate acknowledgement of the third successful pickup. Older superseded
retries remain stale. Bag, claim receipt and journal reconciliation commit together;
no capacity leaves all three unchanged and the candidate remains available.

Owned PostgreSQL 16.15 proof:
`build/quest-loot-pickup-postgres-v2-20261105/verification.json` passes 59 SQL
contracts and five non-SQL library tests, existing reload examples and actual
process restart. All 60 full roll rows retain their hash across restart. New
contracts generate 20 entitled nearby pelt candidates, collect three, then prove
that 17 hidden nearer claims cannot hide a farther ordinary token. They cover a
refused fourth pickup, exact Ready retry, stale older retry, deletion/reveal,
replacement pickup, turn-in consumption/completion, preserved ordinary once and
repeatable rewards, reconnect, full-bag rollback/freeing space, invalid original
rights and unknown-ID refusal. The one-time reward recovers the same claim across
different kill lives and its consumed history remains after pickup. v1 passes;
v2 adds the mixed ordinary/once-history coverage. A preliminary build failed only
a malformed decimal literal in the new test fixture, fixed before SQL execution.

Formatting and denied-warning workspace/all-target Clippy pass at
`build/quest-loot-pickup-fmt-v1-20261105.log` and
`build/quest-loot-pickup-clippy-v1-20261105.log`.
Preserve `build/quest-loot-pickup-qualified-v1-20261105`. These are trusted storage
calls with synthetic kill/NPC authorization, not live actor range, encrypted-owner
or native proof. `RewardNotNeeded` still needs an ordinary owner rejection mapping
when that adapter is installed. The normal schema-18 playable remains `85fd967`;
no new native/Windows or capacity claim. Next: bounded worker operations, accepted
request drain after lost replies, owner refresh after committed bag/journal changes,
then explicit World/manifest/client adoption. Commit locally only; no push.


## Quest-only worker prerequisite — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15. Publisher mirrors the explicit Store generation,
nearby view and pickup APIs on the existing 16-entry critical inventory FIFO.
Bound profiles, checked world, creature selector and revision budgets validate
before enqueue. Unsupported generic/old compound reward routes reject the
conditional profile before queueing rather than failing the shared SQL worker.
No new service, simulation I/O, wire message or source gameplay catalogue.

Nonblocking enqueue helpers report full/closed queues without acceptance. Async
critical producers reserve capacity in the same bounded queue before sending;
only then does the five-second answer timeout begin. Cancellation before enqueue
has accepted no work. Accepted generation and conditional pickup execute even if
the answer receiver closes while queued, including during graceful shutdown drain.
Read-only cancelled views may skip. Queue acceptance is neither inventory success
nor a committed generation receipt. `RewardNotNeeded` is an ordinary nonfatal
rejection and increments rejection counters; bad persistent data/backend failures
retain the existing fail-closed handling.

Owned PostgreSQL 16.15 proof:
`build/quest-loot-worker-postgres-v3-20261105/verification.json` passes 60 SQL
contracts, five non-SQL library tests, reload examples and actual process restart.
All 65 full roll rows retain their hash across restart. The new worker contract
blocks a backpack row, drops the first generation answer before execution,
queues acceptance afterward and fills the remaining 15 FIFO slots. Seventeen
additional nonblocking sends refuse immediately; an async producer waits for
capacity. Once released, the original denied generation and dropped duplicate
answers drain without changing its initial rights after acceptance.

A winning generation is then blocked so pickup is accepted but cannot begin;
its answer is dropped and shutdown starts before release. Pickup still commits
bag/journal/claim before ownership release. Reconnect recovers the receipt; a new
worker collects the next two pelts, rejects an unnecessary fourth without failing,
answers another view and recovers the exact third-pickup retry while Ready.
Deleting one afterward reveals the retained winning candidate. v1/v2 preserve
successful queued-drain proof; v3 additionally proves bounded async backpressure.

Eleven server-library tests, formatting and denied-warning workspace/all-target
Clippy pass at `build/quest-loot-worker-server-tests-v2-20261105.log`,
`build/quest-loot-worker-fmt-v2-20261105.log` and
`build/quest-loot-worker-clippy-v2-20261105.log`. Preserve
`build/quest-loot-worker-qualified-v1-20261105`. Retained generation authorization,
stationary owner refresh after bag/journal changes, World installation, normal
manifest/client admission and native conditional loot are still pending. This is
trusted worker/SQL proof, not a live kill, encrypted owner, native/Windows or
capacity claim. Normal schema-18 playable remains `85fd967`. Continue locally;
no push.


## Quest-only retained owner — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15. Explicit headless `with_quest_loot` now requires
an installed exact inventory-bound journal in both owners, matching reward wire /
storage identities and exclusive reward installation. Missing journals, mismatched
source bytes and duplicate installation refuse before adoption. The ordinary
constructor stays closed for conditional entries; no normal manifest changes.

The retained async inventory owner routes generation, nearby views and pickup
through the explicit conditional Publisher APIs. Accepted pickup and executing /
queued fatal facts drain through logout before the character slot is released.
`RewardNotNeeded` maps to the existing ordinary Conflict rejection; it does not
fail the owner. Before normal UI adoption, replace the existing overly specific
Conflict text with suitable pickup feedback. Latest successful pickup retries
still recover while Ready. Ordinary rewards remain available independently.

The 500ms interest timer also refreshes conditional views after committed bag or
journal high-water changes, without requiring the existing eight-metre movement
threshold. Revisions are dirty hints only; SQL rechecks locked current need. The
owner releases the admission lock before borrowing watch projections and samples
hints before the query so a concurrent publication remains dirty for the next
refresh. This is bounded polling, not a new event or streaming system.

`build/quest-loot-owner-oidc-v3-20261105/verification.json` passes actual local OIDC,
encrypted ticket and PostgreSQL 16.15 lease checks: unaccepted death stays denied
after acceptance/replay, five entitled candidates require explicit pickup, Ready
hides leftovers, deleting one reveals loot while stationary, stale fresh pickup
rejects normally, accepted pickup commits through logout, rejoin restores Ready,
latest pickup retry replays, turn-in consumes three and awards 25 XP. Completion
keeps the remaining entitled pelt hidden. Two further accepted fatal facts drain
through logout with denied generation receipts. Nine unique rolls persist within
this owner proof; this is not another database restart test. v1/v2 retain earlier
passing checks; v3 adds source mismatch and duplicate-install refusal.

Ordinary per-life loot and inventory-bound journal owner regressions pass at
`build/quest-loot-owner-ordinary-regression-v1-20261105/verification.json` and
`build/quest-loot-owner-items-regression-v1-20261105/verification.json`.
Eleven server-library tests, formatting and denied-warning workspace/all-target
Clippy pass (`build/quest-loot-owner-server-tests-v1-20261105.log`,
`build/quest-loot-owner-fmt-v1-20261105.log`,
`build/quest-loot-owner-clippy-v1-20261105.log`). Preserve
`build/quest-loot-owner-qualified-v1-20261105`.

These checks drive trusted owner facts/NPC authorization, not simulation kills or
native interactions. World installation, normal schema/client adoption and native
conditional loot remain pending. No rendering change or new Windows/capacity
claim. Normal schema-18 playable remains `85fd967`. Continue locally with explicit
World validation and encrypted runtime coverage; no push.


## Quest-only World admission — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15. Explicit headless
`World::enable_quest_loot` requires the exact installed inventory-bound journal,
conditional reward metadata, inactive combat and matching installed creature.
Legacy `enable_rewards` / `enable_creature_loot` continue rejecting conditional
profiles. Refused admission leaves rewards and World identity unchanged. Accepted
content participates in the existing reward/journal identity composition.

The fixed step only produces original server fatal-life facts for the retained
recipient. It does not evaluate saved quest need, roll tickets or grant items;
those decisions stay in the already qualified async owner/SQL path. Existing
pickup guards continue requiring a matching profile, alive loaded player and
three-dimensional three-metre range. This interaction distance is Theia policy.

All 365 network-library tests pass at
`build/quest-loot-world-network-tests-v2-20261105.log`. Two new checks cover missing /
foreign journal refusal, legacy gates, duplicate admission, joined or ticked
World refusal and identity preservation. Real casts kill the current creature at
20/30/60 Hz and emit the original actor, life, creature and position for each
reward recipe, including conditional entries without any saved quest in World.
Duplicate events deduplicate; nonfatal and respawn-stale facts produce no offers.
Existing pickup profile/readiness/range logic works with the conditional profile.
Formatting and denied-warning workspace/all-target Clippy pass at
`build/quest-loot-world-fmt-v1-20261105.log` and
`build/quest-loot-world-clippy-v1-20261105.log`.

Preserve `build/quest-loot-world-qualified-v1-20261105`. This is headless simulation
proof, separate from the OIDC/SQL owner checkpoint. No combined conditional
encrypted-runtime/native claim yet. Normal CLI/manifest/client adoption remains
pending; preserve playable `85fd967`. Continue client binding and transport
qualification locally; no push.


## Quest-only client binding and encrypted bridge — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15. Explicit inventory
`Profile::with_quest_loot` binds reward bytes to the exact already inventory-bound
journal. It preserves raw item identity/catalogue and an attached vendor identity,
refuses replacing rewards and leaves ordinary constructors unchanged. Gameplay
client startup checks the actual configured journal and inventory before reading
its ticket: missing inventory for an item journal, a foreign inventory, or missing /
foreign journal for conditional rewards refuse before connection.

All 368 network-library tests pass at
`build/quest-loot-client-binding-network-tests-v2-20261105.log`. New constructor
and actual startup checks exercise missing/foreign bindings, preserved vendor
identity and old constructor gates. The encrypted two-peer credit fixture now
also runs conditional per-life loot at 20/30/60 Hz with 20ms delay, 30ms jitter,
every-eleventh loss and every-seventeenth duplication. It installs the exact
journal/guide/rewards on both sides and receives owner-private journal views.
The original tag owner receives the fatal-life loot fact and journal kill;
the competing final hitter receives neither. A synthetic committed available
claim arrives without bag mutation; the other peer cannot collect it. Full-bag
refusal preserves availability, later explicit pickup publishes a new bag and
removes the offer, and a consumed-claim request cannot enqueue again. Existing
ordinary/once variants remain in the same regression suite.

These encrypted tests use synthetic owner publications, not database commits.
The actual OIDC/SQL retained owner proof remains the separate `56b6a22` checkpoint;
this does not yet establish one combined normal native conditional flow. No new
wire format, renderer or UI behavior. Formatting and denied-warning workspace /
all-target Clippy pass at `build/quest-loot-client-binding-fmt-v1-20261105.log` and
`build/quest-loot-client-binding-clippy-v1-20261105.log`. Preserve
`build/quest-loot-client-binding-qualified-v1-20261105`.

Next: opt-in normal schema/CLI admission must bind journal before conditional
rewards on both sides, select the explicit retained owner and update suitable
pickup rejection text. Validate the exact launch snapshot and run the full
native pickup/deletion/reacquisition/turn-in/rejoin flow. Ordinary schema-18
playable `85fd967` remains the qualified reference. Commit locally only; no push.


## Normal quest-only loot qualified — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15. Opt-in manifest schema 19 selects `--quest-loot`
alongside `--quest-items`, `--quest-journal` and `--creature-loot`. Both normal
executables defer conditional reward construction until the exact inventory-bound
journal exists. Server World installation and account service then select the
already qualified explicit conditional paths. Old schema versions, constructors,
ordinary hashes and gameplay content remain valid; schema 18 refuses conditional
entries. Schema 19 requires at least one conditional entry referenced by a known
possession objective. No new network messages or migration in this slice.

Use `content/development-gameplay-classic-quest-loot.json` for provisional
60% pelt / 25% ordinary-token gameplay; the pelt entry is quest-required and the
token remains ordinary. `...quest-loot-fixture.json` uses explicitly guaranteed
per-life chances for native checks. Their reward-schema-4 files are
`content/gameplay/development-quest-required-loot[-fixture].json`. Existing
schema-18 item quests with ordinary pelt drops remain available. Authoring
bounds and conditional restrictions (independent single-item per-life entries)
remain as documented above. The normal launcher validates and snapshots all 23
exact-byte roles, deriving the modes from schema 19; older bundles reject an
explicit conditional override. Loot Conflict feedback now mentions quest need
as well as range instead of always instructing the player to move closer.

Native Linux proofs:
`build/quest-loot-client-native-v1-20261105/verification.json` passes fresh
1920x1200 and a real PostgreSQL restart/fresh login at that size.
`build/quest-loot-client-native-v3-20261105/verification.json` passes fresh
1280x720, fullscreen 3440x1440 and PostgreSQL restart/fresh login at 1280x720.
All five sessions use AMD Radeon RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6
(kisak PPA), with zero GPU diagnostics. The tests use owned OIDC, encrypted game
admission and PostgreSQL 16.15 through the normal launcher. Input is PID/focus-
guarded XTEST pointer/keyboard plus a temporary uinput wheel; this is not physical
hardware or native Windows evidence. Device removal, service shutdown and secret
file cleanup pass. The display is restored to 2560x1080 at 60 Hz.

Every fresh character completes the earlier three quests/area regression. Its
first three wolf lives create no pelt rights before acceptance; ordinary tokens
can be viewed/collected. Accepting the delivery quest never unlocks those old
pelts. Three further kills only create available items; opening the corpse still
grants no possession credit. Explicit pickup yields Ready 3/3. Split-one keeps
Ready; confirmed deletion changes it to Active 2/3. A seventh kill creates the
replacement, whose explicit pickup restores Ready. Turn-in consumes all three
pelts. Prior quest records remain unchanged, all eight quest actions apply with
zero rejections, seven deaths have no player death/event gaps, and the backpack
ends with zero pelts and 106 tokens (99 seed plus seven repeatable token drops).
The separate once-character reward remains uncollected by corpse UI.

Each saved journal ends at head 27, four completed quests, eight actions and
14 historical events. XP reaches the current development cap of 250; the final
25-XP delivery therefore grants no additional XP. SQL records exactly three
denied and four entitled pelt generations, all with observed bag/journal heads.
Tickets, rights, heads, reward profile and original corpse positions match across
actual process restart: 14 roll rows in v1 and 28 across the two v3 characters.
Fresh login restores completion/consumption without offering the quests again.

These tests guarantee loot and use copied, explicitly modified slow-wolf combat
content: perception 1m, chase/patrol speed 0.1m/s, retaliation base damage 1 and
creature physical damage range 1–1. Source gameplay balance is unchanged. They
prove integration, not random-drop rates, combat balance, multiplayer capacity or
all quest content. Backend random tickets and strict current-need/replay behavior
retain the independent storage/worker/owner qualifications above. The first 720p
v2 stopped after three kills when a lingering loot pane correctly blocked the
guide's pointer hit. The qualifier now closes ordinary panels before reopening
dialogue; v3 passes without any product input-blocking change. Frozen game source
matches the candidate build; only that qualifier changed after candidate capture.

57 Rust validator tests, 41 actual-validator bundle/launcher tests, 147 client
and eleven server-library tests pass at `build/quest-loot-manifest-*-tests-v*-20261105.log`.
Formatting and denied-warning workspace/all-target Clippy pass at
`build/quest-loot-manifest-fmt-v1-20261105.log` and
`build/quest-loot-manifest-clippy-v1-20261105.log`. Optimized executables/validator
build at `build/quest-loot-manifest-release-v1-20261105.log`. Earlier 368 network
checks include conditional impaired encrypted privacy/pickup; the actual normal
native tests now connect the SQL owner and simulation as well. Preserve
`build/quest-loot-client-qualified-v1-20261105` and its source, executables/art,
proof references and hashes. Continue readable quest notes next; no push.
