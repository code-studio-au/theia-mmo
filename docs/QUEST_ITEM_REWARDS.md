# Fixed item rewards at quest turn-in

P3-05 / P3-09 / P3-11 / P3-15, planned slice, 6 November 2026.
This design is not runtime support. Existing quest completion awards XP; item
consumption is already coordinated with journal updates. Item reward bundles
remain unavailable until the following prerequisites are qualified.

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
