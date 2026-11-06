# Fixed item rewards at quest turn-in

P3-05 / P3-09 / P3-11 / P3-15, staged implementation, 6 November 2026.
Core proposals, exact content binding, atomic Store/worker turn-in and explicit
retained ownership, explicit World admission and the encrypted bridge are now
qualified. Opt-in manifest/CLI configuration is now implemented as a prerequisite;
reward/capacity UI and combined native qualification remain pending.

Use an explicit bounded catalogue associating a quest with fixed grants of known
items. Keep the existing quest schemas and legacy profiles intact. Bind exact
item, quest, reward, XP and optional area/document content before live admission;
matching logical IDs alone must not authorize a different saved reward policy.
A changed reward policy must never retroactively reward a completed quest.

Start with fixed rewards only. Choices, currencies, mailed overflow, automatic
equipping and new item restrictions require separate designs. Successful turn-in
must consume required delivery items, add all rewards, reconcile other item
quests, complete the selected quest, update XP and save the original action
receipt in one durable transaction. The server acknowledges only after COMMIT.
Full bags, missing delivery items and stale or conflicting commands leave every
part unchanged. Exact retries restore committed results without another grant.
Accepted transactions must drain through logout using the existing owner FIFO.

The pure proposal will tentatively consume delivery items before checking reward
capacity, so a slot freed by delivery can hold its reward. Merge compatible
backpack stacks before filling empty slots; preserve equipment. Check the whole
bundle, not each grant against the original unchanged bag. Use one final bag
revision for the combined mutation. These are explicit Theia transaction rules.

## Classic reference and limits

The pinned [VMaNGOS Player quest reward checks](https://github.com/vmangos/core/blob/6bd08b92c2a8909c696cb398b124943198ca9514/src/game/Objects/Player.cpp#L12741)
check fixed and choice rewards against inventory capacity and report inability
to carry them. This is a Classic-targeting emulator reference, not proof of
Blizzard's source or SQL ordering. It does not establish Theia's consume-before-
capacity policy or durable transaction model. The reviewed source and SHA-256 are
retained in `build/quest-item-reward-research-v1-20261106/source.json`; no emulator
code is copied. No dimensional formula changes; metres remain unchanged.

## Acceptance sequence

1. Pure catalogue/proposal: known quest/items, strict fields and bounds, duplicate
   entries/namespace refusal; normal and delivery turn-in; deterministic stack
   merge; full-bag refusal; freed-slot success; multiple grants sharing capacity;
   revision exhaustion; no partial input mutation; other quests reconciled.
2. Exact profile binding and actual SQL: owner/world/content checks, one bag +
   journal + XP + receipt transaction, rollback injection, full bag/missing items,
   competing retries, lost replies, logout drain and process/database restart.
3. Retained owner and encrypted bridge: shared inventory ordering, no foreign
   private reward state, accepted work during retirement and retry at 20/30/60 Hz.
4. Opt-in manifest/CLI/standard Bevy display: honest configured reward text,
   capacity feedback, normal native completion and save/rejoin at all supported
   display anchors, plus unchanged legacy profiles.

Do not promote prerequisite tests to a working reward flow. Keep work local;
P3-17/18 remain unspecified and campaign story IDs are not allocated here.

## Implemented metadata prerequisite

`quests::item_rewards::Catalog` schema 1 validates a separate fixed-reward file
against supplied immutable quest and item catalogues. It retains those checked
contexts, the catalogue ID/revision and a quest-to-grants map. It does not create
a profile digest, a proposal, a saved receipt, an item or quest completion.

Illustrative format (referenced quests/items must already exist):

```json
{
  "schema": 1,
  "id": "theia.example.quest-item-rewards",
  "revision": 1,
  "quest_catalog": "theia.example.quests",
  "item_catalog": "theia.example.items",
  "rewards": [
    {
      "quest": "theia.example.quest-deliver-pelts",
      "grants": [{"item": "theia.example.training-token", "count": 3}]
    }
  ]
}
```

Budgets: 32 KiB source, at most 32 quest entries, eight distinct grants per entry,
positive counts at most 10,000 each. A complete bundle must fit sixteen empty
backpack slots under the actual item stack limits. Exactly sixteen required
slots are valid metadata; current occupancy is a later proposal check. Empty or
duplicate entries/grants, unknown contexts/quests/items, unsupported fields,
invalid IDs, namespace aliases and impossible bundles refuse. These are initial
bounded development allocations, not production capacity claims. Existing quest
schemas, loot definitions, item catalogues and runtime profiles are unchanged.

Six new metadata tests and all 363 core tests pass at
`build/quest-item-reward-catalogue-core-tests-v4-20261106.log`. Formatting and
strict workspace/all-target Clippy use the catalogue `fmt-v2` and `clippy-v2`
logs for 20261106. The earlier v1–v3 tests cover narrower intermediate snapshots.
This headless metadata change requires no new GPU claim. No persistence, live
admission, authoring-CLI support or playable item reward is established here.
Next implement and test the combined pure consumption/reward proposal, then
qualify durable ownership and the runtime sequence above.

## Implemented pure turn-in proposal

`Catalog::propose_turn_in` accepts the checked journal, current bag and a `TurnIn`
containing quest/NPC IDs plus expected quest and bag revisions. It rechecks
readiness/current delivery holdings, tentatively consumes required materials,
merges the fixed bundle into the remaining bag and returns completion, the final
bag and the existing completion-XP proposal together. The final bag advances once,
including when the old revision is the last usable value before exhaustion.
An error returns no partial candidate and leaves every input unchanged.

The loot recipe's existing merge algorithm is factored into a crate-private
shared helper without changing its policy. Existing possession turn-in retains
its behavior; a private preparation step allows the reward path to reconcile
other quests only against final holdings. A consumed item immediately replaced
by its reward therefore does not temporarily invalidate another quest or bump
its revision. Rewards can independently make another accepted item quest ready.
Equipment remains unchanged; an unknown reward quest, stale intent, wrong NPC,
missing delivery items, completed quest, full bundle or exhausted revision refuses.

Eight new proposal tests and all 371 core tests pass at
`build/quest-item-reward-proposal-core-tests-v3-20261106.log`. The initial v2
ordinary-quest fixture used an invalid Ready revision; corrected valid progress
passes v3. All 376 network-library regression tests, including existing encrypted
item/document flows, pass at `build/quest-item-reward-proposal-network-tests-v1-20261106.log`.
Formatting and strict workspace/all-target Clippy pass in the proposal `fmt-v1`
and `clippy-v1` logs for 20261106. These are regression checks, not new reward-wire
or saved item reward evidence. The pure proposal grants no items or XP itself,
checks no live NPC distance/owner and creates no durable retry receipt. Exact
profile binding, authorized SQL, retained owner and playable reward adoption
remain pending; the qualified note reader stays available unchanged.

## Implemented exact profile binding (runtime still closed)

Wire `quest_journal::Profile` and stored `QuestJournalCatalog` now expose explicit
`with_item_rewards(raw_reward_bytes, inventory)` metadata binding. Both retain
the checked fixed-reward catalogue and produce the same digest. Reward-only
quests still bind inventory even without a possession objective. An already
bound inventory must have the same exact hash; repeated reward binding refuses.

Let J be the existing journal digest, including its exact quest/XP and any
area/document context. If inventory is not already bound, first calculate
`J_items = SHA256(existing_item_domain || J || inventory_hash)`; otherwise retain
the existing item-bound J. The new digest is
`SHA256(reward_domain || J_items || raw_reward_bytes)`. The reward domain is
UTF-8 `theia-quest-item-rewards-v1` followed by one zero byte. No legacy hash changes when rewards are absent.
The six raw contexts (quest, inventory, XP, area, document, fixed rewards) each
change the resulting exact identity. Additional area/document definitions,
including unused definitions, cannot alias the reward catalogue or grant items.

This is metadata only. Existing World installation, normal client progression
admission, retained-owner setup and generic Store/worker/document admission refuse
reward-bearing profiles. These gates prevent a policy from being admitted while
completion could still omit its item grants. No schema migration, SQL reward
transaction, new wire message, executable flag or normal manifest role is added.

Five cross-layer server cases (22 server tests total), 371 core tests, 377 network
and eight persistence-library tests pass. Logs are
`build/quest-item-reward-binding-server-tests-v1-20261106.log`,
`build/quest-item-reward-binding-core-tests-v1-20261106.log` and
`build/quest-item-reward-binding-library-tests-v1-20261106.log`.
Formatting and strict workspace/all-target Clippy pass in the binding `fmt-v1`
and `clippy-v1` logs. The new private guards are exercised directly; actual SQL
reward/profile compatibility remains unqualified. No new native or Windows claim.


## Implemented atomic Store/worker turn-in

`Store::command_quest_journal` now selects the fixed-reward proposal only for
explicitly bound quests with a configured bundle. Other quests and legacy
profiles retain their existing path. Store context checks admit the checked
reward policy; World, ordinary client and server retained-owner gates remain
closed. This slice adds no migration, live flag or manifest role.

The existing transaction locks XP, backpack and journal in that order and checks
owner/world/content. It looks up the original action receipt before attempting
fresh capacity or delivery validation. A new turn-in uses the current locked bag,
then persists consumption and the complete reward bundle with one bag revision,
final-holdings reconciliation, completion, XP and the action receipt before
COMMIT. The existing quest intent has an expected quest revision, not a captured
bag revision; stack placement uses current saved inventory. A successful retry
returns current saved journal/bag state and grants nothing further, including
when its reward has since moved. Failed capacity attempts save no receipt, so the
same intent can succeed after space is freed. Accepted worker commands drain on
shutdown even after their first response receiver is dropped.

Seven new real SQL contracts cover injected rollback, whole-bundle/full-bag
refusal, freed delivery slots, ownership/world/exact-content mismatch, stale or
wrong-NPC intent, lost replies, queued duplicates followed by inventory movement,
nonfatal worker capacity refusal, historical read receipts after note consumption
and item rewards at the XP cap. Together with existing regression contracts,
**73 PostgreSQL contracts and eight library tests pass** in
`build/quest-item-reward-storage-postgres-v2-20261106/verification.json` and its
individual logs. The owned PostgreSQL cluster uses fsync, synchronous commit and
full-page writes, rejects an incorrect password, and performs a real restart.
The restart probe retains three tokens in moved slot 15, bag revision 2, journal
head 3 and XP 25. SQL fingerprints remain identical after restart and original
turn-in replay. The server library's 22 tests also pass in the storage
`server-tests-v1` log. Formatting and denied-warning workspace/all-target Clippy
pass in the storage `fmt-v2` and `clippy-v2` logs.

The initial storage v1 restart probe incorrectly expected journal head 2; ready
quest revision 2 advances to completed revision 3. Its seven SQL contracts passed,
but that overall run failed. Correcting only the probe and matching SQL
expectation produces the successful v2 run. No production revision rule changed.
Owned cluster processes are stopped and temporary password files removed.

SQL NPC entitlement in these tests is synthetic trusted fixture scope. These
checks do not establish live NPC range, session ownership, encrypted transport,
client presentation or Windows execution. Continue retained-owner/World and
headless bridge qualification next; desktop/mouse tests are paused at the user's
request. Keep commits local and retain the working note-reader path.


## Implemented retained owner adoption

`GameAdmissions::with_item_reward_quest_journal` is an explicit internal opt-in:
it requires reward-bearing stored and wire profiles with identical exact hashes,
then binds the existing inventory and progression owners. Optional documents use
the same checked wire context. Existing ordinary/document owner constructors
continue to reject reward-bearing profiles. Normal executable configuration,
World installation and client progression admission remain closed.

The progression owner now treats reward capacity as `InventoryFull`, reloads the
current saved projection and continues serving commands. Reward bag revision
exhaustion maps to the existing exhausted refusal. Unexpected policy/storage
errors still fail closed. The journal reply appends `InventoryFull` at code 10;
codes 1–9 and frame sizes remain unchanged. Router and wire tests cover that
specific refusal separately from the existing journal capacity rule. The normal
client UI has not yet adopted reward text or inventory-full guidance.

Actual headless qualification passes at
`build/quest-item-reward-owner-leases-v2-20261106/verification.json` using verified
OIDC, real encrypted admission tickets and owned PostgreSQL. Its deliberately
full seed bag cannot hold the reward after consuming thirteen stones. Refusal
leaves XP/bag unchanged and the owner healthy. Two ordinary filler deletions free
space; the same quest request then waits on a held SQL bag lock. No reward or XP
is published before COMMIT. A queued item move and turn-in drain through session
retirement. Rejoining and retrying the original request retain 102 total tokens,
bag revision 4, moved reward slot 15 and XP 25 without another grant. Reopening
the Store confirms original receipt replay. A changed reward file is refused
before owner setup. Services and temporary credentials are cleaned up.

The initial owner v1 fixture tried a 100-token stack although this catalogue's
maximum is 99; `Stack::new` returned None, so that qualification failed. Corrected
v2 uses valid 99-token input and explicitly asserts every seed slot is populated.
No production stack rule changed. The unchanged document-owner regression passes
at `build/qir-doc-v2-20261106/verification.json`. Its first attempted output path
was too long for PostgreSQL's Unix socket; the cluster refused startup and was
cleaned up. The shorter v2 path runs the full regression successfully.

377 network-library tests, nine journal-wire integration tests and 22 server
library tests pass in the owner `network-tests-v1` and `server-tests-v1` logs.
Formatting and strict workspace/all-target Clippy pass in owner `fmt-v1` and
`clippy-v1`. These proofs drive direct retained leases after a ticket handshake;
they do not exercise encrypted game commands, World-authorized NPC range, the
normal client, native display, Windows or MMO capacity. Next qualify explicit
World and encrypted bridge adoption at 20/30/60 Hz, preserving all legacy gates.
Desktop/mouse tests remain paused at the user's request. No push.


## Implemented explicit World and encrypted bridge

`World::enable_item_reward_quest_journal` is an explicit opt-in requiring a
reward-bound profile, matching inventory and XP, all installed quest references,
optional exact documents and an inactive exclusive World. It also checks reward
catalogue/grant IDs against installed guide, creature and dialogue-choice
namespaces, including unused choices. Prepared routers use the same namespace
checks. Ordinary, item-only and document-only World constructors still refuse
reward-bearing profiles. Successful installation binds the existing World
identity to the exact reward-bearing journal; it creates no inventory or XP.

The runtime client can now validate the explicitly reward-bound journal against
its exact XP and inventory. It retains normal prediction/dialogue, private owner
state and pending-command constraints. No new client ability to grant items,
claim completion or upload counters is added. This is library/runtime adoption;
normal flags, sealed manifests and Bevy reward/capacity presentation are pending.

Two new tests pass in
`build/quest-item-reward-world-bridge-tests-v2-20261106.log`. The first checks
20/30/60 Hz explicit installation, foreign inventory/XP, missing reward policy,
legacy gate refusals, installed choice aliases, repeated/live installation and
router creation. Refusals preserve World identity and installed mode.

The second runs two actual encrypted Renet peers through the normal gameplay
client/server loops at 20/30/60 Hz with 20 ms delay, 30 ms jitter, every eleventh
packet dropped and every seventeenth duplicated. Explicit synthetic owner
publications hold a valid full-bag refusal and later successful completion. The
client receives inventory-full code 10, frees space, retries its original intent
and keeps success pending until the held projection is published. Core proposals
produce the whole consumed/rewarded bag and completion/XP. After deleting the
reward, retrying the original successful request retains the deleted state and
XP 25. The other peer retains its original bag, empty journal and XP 0, with no
private replies. Queues remain bounded without overflow; impairment is observed.

The v1 encrypted fixture selected inventory schema 1, which deliberately does
not support destruction, so it failed before forwarding its deletion command.
The corrected v2 fixture explicitly selects existing equipment/item schema 2;
no legacy inventory capability or production mutation rule changed.

379 network-library tests, nine journal-wire tests and 22 server-library tests
pass in the World `network-tests-v1` and `server-tests-v1` logs for 20261106.
Formatting and strict workspace/all-target Clippy pass in World `fmt-v1` and
`clippy-v1`. These transport tests use synthetic identity/commit projections;
the actual OIDC, SQL, restart and retained-lease proofs above remain separate.
They are not a combined normal-launch or native game qualification. Desktop and
mouse tests remain paused. Next adopt exact normal manifest/CLI content, then
honest reward/capacity UI and native qualification when desktop use is available.
No push or Windows/MMO-capacity claim.


## Implemented normal configuration prerequisite

Opt-in manifest schema 21 adds the bounded `quest_item_rewards` role to all 24
schema-20 roles. The separate catalogue maps the existing development pelt quest
to three training tokens and roadside orders to two. Earlier manifests, quest
XP, ordinary drop chances and guaranteed low-danger fixture loot are unchanged.
Both new manifests are checked in:

- `content/development-gameplay-classic-item-rewards.json`
- `content/development-gameplay-classic-item-rewards-fixture.json`

This cumulative manifest requires the document/item/area context; the underlying
runtime library also supports reward-only quests without notes. The recipe lives
at `content/quests/development-quest-item-rewards.json`. Fixed rewards belong to
this separate role, not extra fields in quest definitions or creature drop tables.
Choice rewards, currency, campaign exports and saved-profile migration are absent.

The Rust validator checks the complete quest/item/document/area context, unused
installed World namespaces, strict version/role presence, paths and the role's
32 KiB bound. The sealed launcher hashes/copies/revalidates all 25 roles and passes
`--quest-item-rewards` with its retained snapshot path to both normal binaries.
Individual profile overrides cannot be mixed with a sealed manifest. Both
binaries read bounded bytes and bind the reward policy after document/item-aware
journal setup, before attaching conditional quest loot. Server World installation
uses explicit reward mode. The account service requires exact stored/wire
reward-journal agreement before choosing the retained fixed-reward owner.
No default catalogue or legacy profile hash changes, and no automatic adoption
of a different saved policy.

**This is a configuration prerequisite, not a qualified playable reward release.**
The Bevy journal still lacks configured reward lines and specific inventory-full
guidance. No normal native reward completion/save/rejoin run has been performed;
the user has paused all desktop/mouse tests. The preserved note-reader candidate
remains the most recent fully qualified native path. New UI and combined native
checks must be completed before promoting this reward candidate.

Headless evidence for 20261106:

- 62 Rust validator tests pass in manifest `validator-tests-v2`; 45 actual-validator
  Python snapshot/handoff tests pass in `bundle-tests-v2`.
- 24 server library tests pass in `server-tests-v2`, including missing, unexpected
  and foreign exact wire policies at the normal service boundary, plus the exact
  normal document/fixed-reward/conditional-loot context. All 150 client unit tests
  pass in `client-tests-v1`; these headless units create no game window.
- Current debug client/server binaries build (`binaries-v1`, with server corrected
  in `server-binary-v2`). Three Linux CLI tests cover eight missing/unbound,
  duplicate and reordered-option cases in `cli-v2`. Display variables and service
  configuration are removed from child environments; no window is created.
- Four reports (schema 1/12/19/20) match the frozen reader validator byte-for-byte
  as parsed JSON in `build/quest-item-reward-manifest-legacy-reports-v1-20261106.json`.
  Both new manifest reports contain schema 21 and exactly 25 roles.
- Formatting and strict workspace/all-target Clippy pass in manifest `fmt-v3` and
  `clippy-v3`. Workspace/all-target compilation also passes (`check-v1`).

The first validator test expected a World-alias error for a choice already
referenced by the quest catalogue; Core correctly refused it earlier. The v2 test
removes that reference and verifies refusal of an unused installed choice. The
first ad-hoc CLI smoke also caught a prerequisite check accidentally inserted
inside document-argument parsing. It has been moved after all arguments, and
permanent CLI cases cover all three flag orders. That failed smoke's headless
server was killed/reaped and its temporary public certificate removed. These
failed attempts are not successful qualification evidence.


## Implemented read-only authoring handoff

The existing note/item/quest reference checker now accepts optional
`--item-rewards FILE`, validating the fixed recipe against the supplied closed
catalogue, known inventory and optional area/document context. It refuses
unknown quests/items, context mismatches, namespace aliases and nonregular or
oversized reward files. No reward option preserves the old report exactly;
with it, the report adds only raw hash/count metadata. There is no XP/World
binding or saved admission. [Command, constraints and nine-test/CLI evidence](QUEST_PACK_AUTHORING.md#optional-fixed-turn-in-reward-references).
No desktop interaction or UI changes are needed for this tool. Normal reward UI
and combined native checks remain pending; continue positive server configuration
qualification headlessly while desktop access remains paused.
