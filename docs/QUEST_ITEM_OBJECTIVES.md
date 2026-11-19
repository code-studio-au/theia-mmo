# Possession-based quest items — implementation in progress

User request, 3 November 2026: support quests requiring looted items, recalculate
progress when an item is dropped/deleted, and permit turn-in only while the
required items are in the player's inventory. The recommended direction
is to allow splitting: three items split into stacks of two and one still count
as three; deleting the single item reduces possession to two.

## Evidence reviewed

These are pinned Classic-targeted emulator implementations, not Blizzard's
private server source. VMaNGOS provides item-add/remove quest checks, reverting
completion when required holdings fall short, a separate bag-count check before
reward, and required-item destruction during reward. Its normal stack-splitting
path has no general quest-item splitting ban. Source:
[VMaNGOS Player.cpp, revision 6bd08b92](https://github.com/vmangos/core/blob/6bd08b92c2a8909c696cb398b124943198ca9514/src/game/Objects/Player.cpp#L10961)
(SplitItem, CanRewardQuest, RewardQuest, ItemAddedQuestCheck,
ItemRemovedQuestCheck; lines 10961, 12682, 13081, 13905, 13949).

CMaNGOS Classic independently checks required bag counts before rewarding,
reverts completion on item removal, and destroys required items during reward.
[CMaNGOS Player.cpp, revision 5145613a](https://github.com/cmangos/mangos-classic/blob/5145613a3f68f200664c364dbe136caf413bd2ba/src/game/Entities/Player.cpp#L12378)
(CanRewardQuest, RewardQuest, ItemRemovedQuestCheck; lines 12378, 12611,
13488). Both headers default the count query's bank inclusion to false.
Exact downloaded source hashes and revisions are retained under ignored
build/quest-item-classic-research-v1-20261103; no emulator code is copied into
the Rust implementation.

## Required Theia behavior — not implemented yet

- Count current committed bag holdings across every stack, capped for display at
  the required count. Splitting/merging/reordering without changing the total
  leaves quest progress unchanged.
- Item removal can change Ready back to Active. Losing one of three required
  pelts makes progress 2/3; acquiring another restores readiness. Existing
  permanent kill, interaction and area facts must remain intact.
- Recheck authoritative holdings at turn-in. Consume the required quantities
  across stacks and commit completion, rewards and replay receipt atomically.
  Concurrent deletion/turn-in, request replay and failed transactions must never
  duplicate rewards or consume items without completion.
- Reconcile after relevant loot, destruction and other supported bag mutations,
  acceptance, load/rejoin and consumption by another quest. One set of items may
  satisfy several quest displays; turning one in can make the others incomplete.
- Completed/rewarded quests stay completed after their items are consumed.
  Initial storage scope is the existing backpack; banking, mail and trade are
  separate future systems and confer no current possession credit.
- Keep item delivery distinct from historical “loot this item once” events.
  Authoring must identify which rule is intended.

P3-05 / P3-09 / P3-11 / P3-12 / P3-15 / initial P3-16. Before implementation,
review the existing inventory mutation/ownership/lock order and journal
invariants. Current journal counts/revisions and wire successor checks assume
monotonic historical credit: reversible possession must be modeled explicitly,
with versioned content/profile/save/wire compatibility where required.

Acceptance includes core possession/stack cases, actual PostgreSQL rollback and
concurrent mutation/turn-in, retained bounded owners, encrypted privacy and
forgery refusal, plus a normal native wolf-pelt loot/split/delete/reloot/turn-in
and restart/rejoin flow. The area-quest client qualification is complete; its playable is preserved.
This request section states desired behavior; qualified implementation is recorded
below. No migration, native item proof or campaign source-ID allocation is claimed.

## Pure inventory prerequisite qualified — 3 November 2026

The inactive core requirements module validates 1–8 unique item requirements,
positive per-item counts within the backpack capacity, and explicit
consume-on-turn-in flags. It sums current backpack stacks, retains actual surplus
for validation and caps only displayed progress. Equipped items are outside this
initial backpack-only predicate, as are future banks/mail/trade. This is an
explicit Theia scope choice, not a claim of complete Classic inventory parity.

Consumption rechecks the supplied current bag, checks every kept and consumed
requirement first, then removes quantities in stable slot order in one pure
proposal. It preserves unrelated stacks/equipment and increments the shared bag
revision once if anything is consumed. A stale sufficient observation cannot be
used as an input to consume a different insufficient bag. The proposal grants
no actor entitlement, SQL COMMIT, quest completion or reward.

All 327 core tests and seven new requirement cases, formatting and denied-warning
workspace/all-target Clippy pass. Cases include real existing split/merge
operations, a reduced-quantity fixture, current-bag rechecking, surplus/cross-stack
consumption, mixed keep/consume, invalid/duplicate/unknown/over-budget requirements,
equipped-item exclusion, exhausted mutation revisions and exact maximum bag
quantity. Frozen source/proof:
build/quest-item-possession-qualified-v1-20261103.
There is no new full-workspace, SQL, transport or native claim for this helper.

## Next integration boundaries

Initially support possession requirements in an all-required final hand-in stage.
Earlier permanent kill, interaction and area stages remain complete when items
are lost. The final stage may move Ready back to Active when its current item
requirements cease to hold. Consuming versus keeping items stays explicit in
authoring data, aligned with the proposed authoring agreement. Broader reversible
multi-stage/any-of delivery semantics require their own defined rules.

The current backpack supports split and move/merge, but has no destruction
command. The current reward catalogue supports once-per-character claims only.
The requested wolf-pelt loop therefore also needs a checked destruction flow and
repeatable loot bound to an authoritative creature life; inventing more
once-character claims would not demonstrate replenishment by killing another
wolf. Keep old schemas/content working until these replacements pass.

Before storage integration, preserve the existing lock order and ownership
guarantees. Backpack, reward claim, XP and journal changes cross multiple current
owners. Every relevant mutation must reconcile possession, and final turn-in must
recheck and consume under the same durable transaction as completion/reward.
Publication must not regress under concurrent owner completions. These are
requirements still to implement and qualify.


## Reversible journal / codec prerequisite qualified — 3 November 2026

Quest catalogue schema 3 adds possess_item with item ID, required quantity and an
explicit boolean consume_on_turn_in. The current 1–100 objective-count bound
remains. Schemas 1/2 refuse this kind; schema 3 can also retain old historical
kinds. Initial possession objectives must be in an all-required final stage.
The source-pack compiler emits schema 3 only for a selected closure needing this
kind; item selectors and consume flags survive, and alias checks remain strict.

Journal inventory reconciliation derives capped possession counts from the
supplied current bag. Ready may become Active and later Ready again; earlier
historical facts and completed quests do not change. Historical-only definitions
retain their old exact revision validation. Possession definitions use monotonic
mutation revisions even as quantities decrease, with checked lower bounds for
acceptance, historical facts, nonzero holdings and completion. Saved JSON count
envelopes keep the same shape; schema-3 definition semantics and outer exact
profile binding are required. No existing character profile is migrated.

Generic raw possession events and ordinary unchecked item-quest turn-in refuse.
The dedicated pure turn-in rechecks the current bag, rejects stale cached Ready,
prepares required consumption and once-only completion/reward, then recalculates
other quests against the resulting bag. Consuming shared materials can make
another ready quest Active. Explicit nonconsuming requirements keep the items.
These remain tentative values: the durable caller must commit every participant
together and establish actor/NPC ownership.

The existing codec layout can carry these counts/revisions. Successor checks
allow item-count decreases and same-final-stage Ready→Active only for possession
definitions with advancing versions; permanent fact regression, conflicting
heads and changes after completion still refuse. This does not add a client
credit-upload operation.

All 335 core tests (seven new journal and one compiler cases), nine codec checks
(one new reversal/permanent-fact case), 72 authoring-tool checks (including consume
policy change review and fail-closed manifest admission), formatting and strict
workspace/all-target Clippy pass. Core cases cover missing-current-item refusal,
historical fact preservation, cross-stack consumption, shared materials,
nonconsuming requirements, revision exhaustion, saved round trips and impossible
item references/capacity. Frozen source/proof:
build/quest-item-journal-qualified-v1-20261103.

Normal World/manifest admission explicitly refuses item objectives until their
inventory profile and durable owner are installed. No item-aware SQL transaction,
destruction command, per-life repeatable loot, actual encrypted item quest or
native item flow is claimed. The qualified area client remains available.

Additional official evidence: Blizzard's January 2022
[TBC Classic support reply](https://us.forums.blizzard.com/en/wow/t/quest-item-deleted-tbc-classic/1174989/8)
explains that a deleted quest item may require killing its source boss again.
This is a later Classic expansion example, not a Vanilla server-formula proof.
The linked support article could not be read through its current redirect.


## Durable storage prerequisite qualified — 3 November 2026

Network and storage profile constructors bind the prior quest/XP/optional-area
hash plus exact inventory hash in a new inventory domain. Unknown, impossible
and reserved-selector item references refuse binding. Old profiles retain their
hashes. Unbound possession journals refuse storage operations; no save migration.

Item-bound journal operations lock the backpack before journal rows (and after
XP when XP is needed). Acceptance, load, event/action replay and historical facts
reconcile against that current locked bag. Turn-in rechecks possession, consumes
across stacks, recalculates other active quests, completes and grants XP with the
original action receipt in one transaction. A separate internal consumption
receipt domain supersedes the latest player-item receipt while retaining the
existing positive-revision SQL constraint. Exact action retry returns current
committed views and grants/consumes nothing again.

Optional compound item APIs cover ordinary move/split/equipment, vendor and reward
claim writes, including their replay paths, and return bag and journal from the
same transaction. The existing legacy APIs remain available for old profiles;
the live item-aware owner must select compound APIs for every relevant mutation.
The new bounded critical-worker jobs retain accepted work after response loss and
drain on shutdown. No database I/O enters simulation.

Four new owned PostgreSQL contracts check cross-stack consumption, receipt-trigger
rollback of bag/XP/journal, reopen/action replay, missing bag rejection, cached
Ready refusal and reconciliation after an explicitly administrative removal,
vendor acquisition plus journal-write rollback, and retained compound split work.
All 18 journal SQL contracts pass; exact durable hashes of backpacks, journals,
receipts and XP match across actual database restart with fsync,
synchronous_commit and full_page_writes enabled. The administrative removal is
not a player destruction command. Vendor authorization is a trusted fixture,
not live NPC range proof. All 345 selected core/persistence/server tests pass
(48 service skips), including three cross-crate profile-binding cases; formatting
and denied-warning workspace/all-target Clippy pass.
Frozen source/proof: build/quest-item-storage-qualified-v1-20261103.

Live World/manifest item admission remains closed. Generation-bound coherent
publication across inventory and progression owners, player destruction,
per-victim-life repeatable loot, encrypted item quest and normal native
loot/split/delete/reloot/turn-in/rejoin remain required. Latest playable stays
build/quest-reach-client-qualified-v1-20261103. No new full-workspace, native GPU,
Windows, MMO capacity or campaign-ID allocation claim.


## Destruction core / codec / storage checkpoint — stopped 3 November 2026

At the user's stop request, preserve the checked Destroy {from, amount} backpack
proposal: positive amount up to the current stack quantity, current revision,
valid source and bounded revision are required. Partial removal preserves the
remaining stack; exact removal clears it. One operation advances one bag revision.
Other slots and equipment remain untouched. The durable item-command hash includes
a distinct ordinal and exact quantity. This is destruction, not a world ground
item or recoverable drop system.

IR02 item requests use ordinal 4, a canonical zero destination and checked positive
amount. Legacy IR01 still refuses destruction, and all original frames/layouts
remain unchanged. Existing equipment-capable content is the intended client path;
no new manifest or live item-quest admission is enabled. Core and codec malformed,
truncated, stale, exhausted and invalid amount/source cases pass.

The fifth new owned PostgreSQL contract actually splits one of three items into
another slot, submits its destruction through the compound item API and observes
Ready→Active with 2/3. A forced journal-write failure rolls back removal; retrying
successful destruction returns the current bag without removing another item.
The old ready turn-in fails and XP remains zero. All 19 journal SQL contracts
pass and durable bag/journal/XP/receipt hashes match after actual database restart.
All 347 selected core/persistence/server tests (48 service skips), 14 inventory/
journal codec checks, formatting and strict workspace/all-target Clippy pass.
Frozen source/proof: build/quest-item-destruction-storage-qualified-v1-20261103.

An unfinished two-step destruction button patch is retained only at ignored
build/quest-items-destruction-ui-pending-20261103.patch. Client inventory source
was restored byte-for-byte from the preceding storage checkpoint before final
validation, so this stop adds no renderer/UI change or native GPU claim.

Work is stopped at the user's request. Resume with generation-bound retained
owner initialization and monotonic cross-owner committed publication; select
compound APIs for every supported bag mutation. Then finish and natively verify
the confirmation UI, implement repeatable loot bound to real creature lives,
install exact manifest/World bindings and run normal encrypted/native
wolf-pelt loot/split/delete/reloot/turn-in/rejoin. Do not apply the set-aside patch
blindly after other UI edits. Latest normal playable remains the area checkpoint;
no campaign-ID allocation, save migration, native Windows or MMO-capacity claim.


## Added requirement — readable quest documents, 4 November 2026

User requested quest conditions for reading/viewing a note or message. Planned
scope: a separate stable document-read objective, distinct from current item
possession. Opening a document requires current server-validated ownership (or an
explicit world-document interaction), exact document/content binding and active
quest-stage eligibility. Viewing records a bounded, replay-safe historical fact;
it cannot prove human attention. Closing the panel or later destroying the note
does not erase that fact. A separate possession objective may still require the
note at turn-in. Merely looting/hovering the item grants no read credit. Reopening
cannot duplicate progress/rewards, and pre-accept or future-stage views receive
no retroactive credit unless a later explicit content policy permits it.

This is planned behavior, not an implemented document UI or objective kind.
P3-05 / P3-11 / P3-15: implement after the current inventory-aware owner and loot
slice. Acceptance must include unauthorized/forged document requests, wrong
item/document binding, duplicate opens, stage boundaries, save/rejoin and native
open/read credit. Existing NPC interaction objectives must not be repurposed as
unchecked client read events. No campaign IDs are allocated here.


Readable-item evidence: pinned VMaNGOS
[ItemHandler.cpp at 6bd08b92](https://github.com/vmangos/core/blob/6bd08b92c2a8909c696cb398b124943198ca9514/src/game/Handlers/ItemHandler.cpp)
checks the item at the requested bag/slot, that it has page text, and CanUseItem
before returning read success or failure. That handler alone does not grant a
general quest objective. This supports server-authorized document opening;
Theia's separately authored read-objective credit remains an explicit extension,
not a claim that every Classic note grants completion. Exact source/hash retained
under ignored build/quest-document-research-v1-20261104; no C++ is copied.


## Retained owner prerequisite qualified — 4 November 2026

The item-aware inventory owner initializes the bag and reconciled journal before
publishing bag readiness. Its progression peer waits for that initialization.
Exact inventory hashes must agree when admission is configured. Move/split,
destruction, equipment, vendor and reward claims select the compound SQL APIs.
Normal refusals reload both committed bag and journal before private publication.

A retained generation/binding check guards cross-owner publication. Bag, journal
and XP high-water versions prevent late responses restoring removed items or
older readiness; equal-version conflicts refuse. Background updates preserve
command replies. A delayed command reply attaches to the newest committed
journal, rather than overwriting it. Pure tests cover these reorderings and XP
reward preservation. These are separate watch/network projections: a renderer
may sample them on different frames. This is not an atomic multi-channel wire
snapshot claim. Gameplay turn-in remains checked and committed in SQL.

The actual service qualifier uses verified local OIDC, an encrypted connection
ticket and owned PostgreSQL with the production admission/owner code. An explicit
28-stone fixture accepts a possession quest, splits and destroys one (27/incomplete),
rejects old-ready turn-in, buys three through the real owner, then turns in while
logout/retirement waits on a deliberately locked journal row. Character reentry
is blocked until accepted work drains. Completion consumes 28, leaves two, grants
25 XP and reaches bag revision 4. Rejoin and retry preserve those values without
another consumption/reward. The older historical-journal retirement scenario
also passes. Both use trusted synthetic NPC/event entitlement, not live World
range/combat or a native client. All owned services stop cleanly.

All 1,271 workspace/all-target tests pass (59 service skips), plus the two actual
service qualifier modes, formatting, Python launcher compilation and denied-warning
workspace/all-target Clippy. Earlier 19 journal SQL contracts remain separate evidence; this owner slice adds no SQL
migration. Frozen source/proof: build/quest-item-owner-qualified-v1-20261104.
Live World/manifest item admission remains closed until confirmed UI, real
per-creature-life repeatable loot and normal encrypted/native item adoption pass.
No new native GPU/Windows, capacity or save-migration claim. Local/unpushed.


## UI / transport work in progress — stopped 4 November 2026

The user requested a safe stop. Current source adds a two-click Destroy selected
stack control with Cancel; closing the pane, losing connection, changing the bag
or choosing another item action clears confirmation. The confirmed slot and bag
revision are retained. The client sends that exact revision rather than sampling
a newer revision during enqueue, so a replacement item cannot become the target
of an older confirmation. The server still checks ownership/current revision and
quantity. Equipment-capable IR02 content is required. This is built work in
progress; normal player destruction is not yet native-qualified.

A related transport prerequisite allows an exact matching successful reply to
carry a bag newer than expected_revision+1. Item and quest writes share a bag, so
another committed write may advance it before the original command reply is
published. The minimum committed revision, exact pending sequence, actor/profile
binding and nonregression checks remain. A newer background bag alone never
acknowledges the pending command. Eleven focused bridge tests pass, including
coalesced removal preservation, missing-commit refusal and exact confirmed-revision
retention. Seven encrypted account tests pass, including a new two-player
coalescing case that preserves the later removal, recipient privacy and movement
while waiting. The owner in these transport tests is explicitly synthetic; the
SQL owner has separate qualified evidence. No wire layout changes.

All 144 client tests, formatting, Python compilation, strict workspace/all-target
Clippy and optimized combined client/server plus validator builds pass. These are
selected follow-up checks, not another full-workspace qualification after the
1,271-test owner checkpoint.

The first normal production-content native run at 1920x1200 reached original
quest acceptance but timed out in the existing inventory Escape-close check,
before the new destruction helper ran. The reviewed screenshot shows the new
control rendered, and strict log review has zero GPU diagnostics on AMD Radeon
RX 7900 XTX / RADV Vulkan / Mesa 26.2.6. This failed flow does not prove destruction,
cancel/reopen or item persistence. Extra dialogue pending/UI diagnostics and a
wait for fully settled dialogue close are now built; their rerun was deferred by
the stop request. The cause of the initial close timeout is not established.

Keep the failed evidence at build/quest-items-destruction-native-v1-20261104 and
its first candidate unchanged. Preserve final work-in-progress source, exact
binaries and proof at build/quest-item-ui-wip-stopped-v1-20261104. No desktop/game/
owned provider/database process remains running. Latest fully qualified item-owner
source remains build/quest-item-owner-qualified-v1-20261104; latest normal playable
qualification remains the earlier area client.

Resume only after the user asks: diagnose/rerun native panel close, then complete
split/confirm/cancel/reopen/destroy/restart checks at 1920x1200, 3440x1440 and
1280x720. Continue with real per-creature-life repeatable loot, exact manifest/
World binding and full wolf-pelt item quest. Then implement the separately
planned note-reading condition. Item-quest live admission remains closed.
Nothing committed or pushed; no new Windows, capacity or save-migration claim.
