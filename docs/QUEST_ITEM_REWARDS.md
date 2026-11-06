# Fixed item rewards at quest turn-in

P3-05 / P3-09 / P3-11 / P3-15, staged implementation, 6 November 2026.
Core proposals, exact content binding and atomic Store/worker turn-in are now
qualified. Fixed bundles remain unavailable in normal gameplay until retained
ownership, World/network admission and presentation are qualified.

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
