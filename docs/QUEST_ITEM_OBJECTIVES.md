# Possession-based quest items

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


## Confirmed destruction UI qualified — 4 November 2026

P3-05 / P3-09 / P3-11 / P3-15. The resumed native check passes with the settled
NPC-close wait. Closing waits for both server dialogue pending state and the
local deferred-close flag before testing the backpack's separate Escape action.
The earlier timeout remains retained evidence; it is not reproduced by these
runs and no new cause beyond the unsettled test prerequisite is asserted.

Current-checkout optimized client/server and validator builds, all 144 client
tests, all 11 focused inventory transport tests, formatting, Python compilation
and denied-warning workspace/all-target Clippy pass. The merged development
branch's eight encrypted account tests passed during repository alignment;
these selected checks are not a new full-workspace/service qualification.

Normal production-content launcher, actual local OIDC, owned PostgreSQL and
PID-bound synthetic X11 input pass at 1920x1200, 3440x1440 fullscreen and 1280x720.
Each run completes the unchanged two historical quests, three wolf kills and
125 XP, then splits a stone into another stack without changing total holdings.
The first Destroy click only arms the exact slot/revision confirmation; Cancel
preserves it; closing/reopening cancels the old confirmation and requires a fresh
one. Confirming removes exactly one stone (28 to 27), advances the backpack once,
and preserves equipment and completed quest state. Database restart and normal
rejoin retain 27 stones with no restored confirmation. All four native starts
have zero GPU diagnostics on Linux AMD Radeon RX 7900 XTX / RADV Vulkan /
Mesa 26.2.6. Screenshots were reviewed at 1920 and 720p.

Freeze current source, binaries and proof at
`build/quest-item-ui-qualified-v1-20261104`; raw final native evidence is
`build/quest-items-ui-current-native-v1-20261104`. The first resumed diagnostic
using the previous preserved binaries is separate evidence under
`build/quest-items-ui-resume-native-v1-20261104`. No files in earlier checkpoints
are replaced. All owned services and desktop input are released after each run.

This qualifies ordinary confirmed item destruction and transport coexistence;
it is not a live possession quest or per-life loot demonstration. Keep item-quest
World/manifest admission closed. Next implement repeatable loot tied to the
actual authoritative victim life, preserving once-per-character recipes and
bounded owner/availability queues, then the normal pelt quest. Reading documents
remains a distinct planned condition. Work continues with local commits only;
no pushes, native Windows, MMO capacity or new save-migration claim.


## Per-life loot core prerequisite — 4 November 2026

Reward schema 2 admits an explicit `per_creature_life` scope alongside the
unchanged `once_per_character` policy. Schema 1 refuses the new scope. A pure
creature-selector helper derives either the once-character discriminator or the
existing `KillId` victim-life key from a trusted kill fact. A different entity
or world epoch is a different occurrence. A mismatched creature refuses.
This helper grants no recipient authority, death proof, durable offer or items.

All 339 core tests, three reward-wire tests and the non-SQL storage admission
check pass. The three existing reward database tests are skipped in this pure
prerequisite run; no new SQL proof is claimed. Formatting and denied-warning
workspace/all-target Clippy pass. Evidence logs are
`build/quest-loot-scope-{core,wire,store-gate,fmt,clippy}-v1-20261104.log`.
Runtime and storage profile constructors deliberately refuse per-life recipes
until their durable occurrence/paging and runtime publication paths are installed.
Schema-2 once-character recipes remain valid. Existing content bytes and wire
layouts are unchanged. No new native rendering or live repeatable-loot claim.

Next add an explicit durable scope/life discriminator with once-character and
per-recipient-life uniqueness, atomic claim/possession reconciliation, bounded
pending availability and replay-safe retained work. Keep consumed per-life
receipts durable without loading all historical kills into the owner. Then bind
actual fatal victim lives in World/runtime and qualify normal pelt quest play.


## Added requirement — explicit corpse-loot pickup, 4 November 2026

The user clarified that a creature can die and drop loot without the player
collecting it. Adopt separate states: authoritative death creates loot availability
at that victim life's corpse; opening loot only views it; explicit pickup commits
items into the backpack. Possession objectives change only after the inventory
transaction commits. Uncollected drops, viewing the corpse and closing the loot
window grant no possession credit. This does not change historical kill objectives.

The recommended initial interaction is right-click corpse, then take an item or
Loot all. Authority must check the recipient, exact corpse/life, current range,
world/content binding and backpack capacity. A failed/full-bag pickup leaves the
uncollected claim available. Bind the original death position so a new creature
life or later movement cannot retarget an old drop. Keep accepted claim/receipt
handling replay-safe; continue to support split/delete/reacquire after pickup.

This is adopted scope under implementation, not a working corpse/loot window.
The existing qualified training Claim button and once-character content remain
legacy behavior. Per-life live admission stays closed until corpse presentation,
authoritative interaction/pickup and normal native tests pass. Corpse expiry,
party/shared loot, drop chance balance and general ground-item dropping require
separate policies; no exact Classic parity for those rules is claimed here.


## Per-life loot storage prerequisite qualified — 4 November 2026

Migration 12 preserves once-character claims and adds a checked scope, victim-life
key and immutable checked f64 corpse position in global Y-up metres. Separate
partial unique indexes enforce once per character/recipe or once per character/
recipe/victim life, including consumed history. Reoffering the same life returns
the original claim; a changed corpse position, wrong creature or foreign owner
refuses. A new life creates a new entitlement. The trusted API does not establish
World kill credit or current pickup range by itself.

Creating an entitlement commits availability only. It does not mutate the bag or
quest possession. Explicit claim consumption uses the existing atomic bag/receipt/
optional-journal transaction. Tests take three drops to reach Ready, destroy one
to become Active, create a replacement drop while remaining Active, then pick it
up to become Ready. A forced journal-write failure rolls back both the pickup and
claim consumption. A full bag leaves uncollected loot available. Split/delete and
normal completed-quest rules remain unchanged.

Accepted trusted offer jobs now survive lost response receivers, including jobs
still queued behind a blocked transaction, and drain during worker shutdown.
Listing keeps complete once-character history (at most 16 definitions) plus the
oldest 16 unconsumed life claims. Consumed life receipts and unshown pending drops
stay in SQL; taking a visible entry exposes later entries on reload. This page is
a bounded storage primitive, not general spatial corpse interest or live-world
capacity proof. Selected-corpse/spatial lookup is still needed for live access to
arbitrary nearby corpses without collecting older distant drops first.

The actual owned PostgreSQL qualifier passes all 34 commands and 53 database
contract tests, including seven reward contracts and the existing quest/inventory/
XP/vendor regressions. Migration upgrade fixtures preserve an old consumed
once-character claim, bag and quest rows; the schema-nine fixture now uses its
own disposable database. Failure-injection constraints are scoped to their test
realm so unrelated XP rows remain valid. Explicit restart hashes match for 28
life claims (26 consumed, two pending), their corpse sources, backpacks, journals
and XP. Durability flags remain fsync/synchronous_commit/full_page_writes on.
Owned services stop and plaintext password files are removed.

One non-SQL profile check, eight server tests and 368 selected network tests pass
(377 selected tests, seven database skips in the non-service command). Formatting,
Python compilation and denied-warning workspace/all-target Clippy pass. The broad
all-target execution build was cancelled while relinking historical executables;
it is not counted as a passing full-workspace test run. The actual SQL contracts
were executed separately above. Source/proof checkpoint:
`build/quest-loot-storage-qualified-v1-20261104`; raw database proof:
`build/quest-loot-storage-postgres-v4-20261104`.

Storage accepts explicit per-life profiles now; the network runtime constructor
still refuses them. No live corpse, loot window, pickup range or native pelt-quest
adoption is claimed. Keep that gate until corpse interaction, recipient/life/range
validation, explicit item/Loot-all pickup and native tests pass. Earlier native
confirmed destruction remains available at its frozen checkpoint. No quest-profile
migration, native Windows or MMO-capacity claim. Continue with local commits only.


## Bounded loot-availability codec prerequisite — 4 November 2026

Opt-in RW02 observations carry per-claim victim-life keys and checked f64 global
corpse positions. Different lives may share a recipe; each claim and each
recipe/life pair must be unique. Several recipes for one corpse must agree on
its location. Scope mismatches, unknown kinds, nonfinite/out-of-bounds positions,
zero life halves, malformed/truncated/trailing data and noncanonical ordering
refuse. Shared core corpse metadata and the 16-life visible-page bound retain
the persistence API through a re-export.

Once-character RW01 and RC01 layouts remain unchanged; a fixed RW01 byte sample
passes. A mixed profile admits at most 16 once-definition rows and 16 visible
life rows, within a conservative 1,277-byte frame bound. The maximal realizable
mixed fixture (15 once recipes plus 16 life claims for one repeatable recipe)
round-trips 31 rows / 1,259 bytes; the seventeenth life claim refuses. This is
bounded encoding evidence, not encrypted RW02 delivery or corpse streaming proof.

All 357 network-library tests, eight availability codec tests and the non-SQL
storage profile check pass (366 selected checks; seven database skips in the
non-service storage command). Formatting and denied-warning workspace/all-target
Clippy pass. Existing encrypted tests in the library exercise their old profiles;
no new encrypted per-life or SQL rerun is claimed here. Evidence is retained in
`build/quest-loot-wire-{tests,storage-api,fmt,clippy}-v2-20261104.log`.

A separate explicit constructor admits only the codec prerequisite. Default
runtime profile construction still rejects per-life content, and World refuses
it even when that explicit constructor is used, without changing World identity.
Do not open those gates until original fatal-life forwarding, bounded selected/
nearby-corpse availability, authoritative recipient/range/session checks and
explicit pickup UI are installed and qualified. Viewing availability grants no
items or possession credit. Native destruction and durable storage checkpoints
remain separately preserved.


## Original fatal-life forwarding — 4 November 2026

P3-05 / P3-09 / P3-11 / P3-15: the bounded reward opportunity queue now
retains the authoritative recipient, original victim-life key, creature definition
and checked global death position. Existing fatal/current-life/tag-owner checks
still precede forwarding. The async character owner validates the creature,
selects once-character versus victim-life deduplication and persists the original
fact through the trusted kill-offer API. It reloads the bounded pending page
instead of accumulating life history or suppressing later kills by recipe ID.
Availability alone still changes neither inventory nor possession objectives.

All 357 network-library tests pass, including encrypted two-peer competing-hitter
routing at 20/30/60 Hz. Those encrypted fixtures still use once-character recipes;
source-life metadata is asserted but no live repeatable-loot claim is made.
The actual owned OIDC/encrypted-ticket/PostgreSQL reward-owner qualification
passes: accepted executing and distinct queued offers survive logout, availability
waits for commit, unclaimed offers restore and claims do not grant twice.
Evidence: `build/quest-loot-fatal-network-v2-20261104.log` and
`build/quest-loot-fatal-owner-v1-20261104/verification.json`. Formatting and strict
workspace/all-target Clippy pass. No rendering change or new native check.

Default runtime per-life admission remains closed. Next: explicit corpse-life
pickup validation and selected/nearby availability, then corpse presentation and
loot UI. The current oldest-pending storage page remains a storage primitive,
not spatial corpse interest. This slice does not establish durable world-kill
recording if the bounded fact queue fails before enqueue. Local commits only.


## Safe stop: explicit pickup authority prerequisite — 4 November 2026

Stopped at user request. P3-05 / P3-09 / P3-11 / P3-15 now have an explicit
68-byte RC02 pickup frame binding actor, shared inventory sequence, expected bag
revision, private claim ID and original victim-life key. The server dispatcher
recognizes it separately from unchanged RC01/IR01/IR02. Per-life requests require
an installed per-life profile and matching private committed availability; an
RC01 request cannot bypass a life-bound claim. Recipient/claim/life/scope,
current World profile, living loaded actor and three-dimensional 3m range are
checked before owner enqueue. This is the current metric Theia interaction
policy, not a sourced exact Classic loot-distance claim. Accepted SQL pickup
uses the existing atomic bag/receipt/journal operation; later walking away does
not cancel an already accepted critical intent.

358 network-library tests and nine reward-wire tests pass (367 total), as do
formatting and denied-warning workspace/all-target Clippy. The new frame tests
reject truncation, trailing bytes, invalid identity halves/revisions and protocol
downgrades. A synthetic World guard fixture checks absent/mismatched profile,
unready/missing terrain footprint, vertical range boundary and departure, including
a durable corpse from an older world epoch. That fixture sets the private profile
directly for testing; normal per-life World admission remains closed. Evidence:
`build/quest-loot-pickup-{tests,fmt,clippy}-v1-20261104.log`.

This is a checked prerequisite, not a completed corpse interaction feature. There
is no client pickup queue, rendered private corpse, loot window or Loot-all yet.
Positive life-bound peer enqueue, death refusal and impaired encrypted per-life
pickup still need integration qualification; then selected/nearby availability,
client UI and normal native possession/reversal/turn-in/rejoin tests. Keep live
item-quest and per-life content gates closed until those pass. The prior real
OIDC/PostgreSQL reward-owner regression remains separately qualified, not proof
of this new RC02 path. No new native, Windows or capacity claim.

Preserve source/proof at `build/quest-loot-pickup-stopped-v1-20261104` and runnable
native destruction at `build/quest-item-ui-qualified-v1-20261104`. Owned test
services and desktop input are stopped/released. Local commits only; nothing
pushed. Resume only on user instruction.


## Explicit headless pickup integration — 5 November 2026

Resumed at user request. P3-05 / P3-09 / P3-11 / P3-15: an explicit per-life
inventory profile and World installation now support headless corpse pickup.
Default profile construction, legacy World reward installation and normal
manifest admission retain their gates. The optional capacity-one pickup inlet
shares the existing backpack sequence, pending deadline and committed-revision
acknowledgement rules; reward availability alone cannot clear a pending action.
Existing normal clients supply no pickup inlet yet.

Real World/bridge checks cover accepted exact pickup, protocol downgrade,
wrong life/claim, closed/failed owner, unavailable terrain, three-dimensional
range, missing availability and actual player death. Rejection leaves the bag
unchanged and does not enqueue owner work. A full-bag acknowledgement preserves
availability while releasing the shared stream for the next action.

The actual encrypted two-peer test runs at 20/30/60 Hz with delayed/jittered/lost/
duplicated UDP. A competing final hitter cannot pick up the tag owner's claim;
viewing an offer does not mutate the bag; an owner full-bag refusal preserves
availability; retry success waits for the committed observation; the collected
claim cannot be enqueued again. Inventory owner/commit observations in this test
are synthetic. They do not establish SQL or native corpse interaction. Fatal
recipient routing uses the real combat World. Durable SQL evidence remains the
separate 4 November qualification.

381 selected tests pass: 361 network-library, eight account-admission, three
melee transport and nine reward-wire tests. Formatting and strict workspace/
all-target Clippy pass. Logs: `build/quest-loot-headless-{tests,fmt,clippy}-v1-20261105.log`.
No rendering change or new native/Windows claim. Preserve
`build/quest-loot-headless-qualified-v1-20261105`. Next: bounded nearby/selected
corpse availability, normal client selection/pickup/loot UI, then live item-quest
admission and native possession/reversal/turn-in/rejoin proof. Local commits only.


## Nearby durable loot interest — 5 November 2026

P3-02 / P3-05 / P3-09 / P3-11 / P3-15: the async item owner now loads nearest
pending life claims within a 64m three-dimensional presentation radius around
the server-retained character pose. It preserves complete bounded once-character
history and at most 16 pending life rows. The old oldest-pending store API remains
available for recovery/tests, but no longer selects the per-life owner's visible
page. Consumed history and distant/unshown loot remain durable. Wire order stays
canonical by recipe/claim after nearest-page selection.

Owner interest checks run outside simulation on a 500ms skipped-missed-tick timer;
SQL reload occurs after at least 8m displacement, on offers, on pickup or at rejoin.
The centre comes from the retained generation-bound pose, never a client request.
Availability generations advance only when the selected claim set changes.
Viewing/moving changes neither the bag nor item-objective progress. The 64m radius,
16-row page and refresh policy are explicit initial Theia presentation bounds,
not verified Classic constants or production crowd capacity. A dense pile can
exceed the visible page; pickup/closer interest reveals additional pending rows.
No corpse expiry/cleanup policy or general spatial index is established here.

The actual PostgreSQL qualification passes 54 contracts and a real restart with
exact loot/bag/journal row hashes. A new mixed-profile fixture preserves 39 claims
(38 pending/one consumed), checks nearest selection despite older remote claims,
exact vertical radius vs diagonal/outside rejection, once-character history,
worker parity, pickup and return to distant uncollected loot. Evidence:
`build/quest-loot-nearby-postgres-v2-20261105/verification.json`. The first run found
an SQL parameter-type ambiguity; explicit double-precision casts corrected it.

Actual OIDC/encrypted-ticket/PostgreSQL owner qualification also passes: 17 remote
old lives do not hide the fresh local drop; retained movement switches pages;
returning restores the same claim; accepted pickup completes across logout;
fresh admission restores the committed bag and remote pending loot; replaying a
consumed life creates neither items nor availability. This drives the real owner
through synthetic trusted facts/poses and bypasses the World request adapter, so
it is not native pickup or end-to-end World-to-SQL proof. The old once-character
reward-owner retirement qualification passes unchanged. Both owned test service
sets stop and remove plaintext password files. Proofs:
`build/quest-loot-nearby-owner-v1-20261105/verification.json` and
`build/quest-loot-nearby-legacy-owner-v1-20261105/verification.json`.

394 selected library tests (22 core, 361 network, three persistence and eight
server), formatting, Python compilation and strict workspace/all-target Clippy
pass. Preserve `build/quest-loot-nearby-qualified-v1-20261105`. No new rendering,
native/Windows or capacity claim. Continue with normal client corpse selection,
explicit loot UI and manifest adoption; live item quests remain gated until
normal native possession/reversal/turn-in/rejoin succeeds. Local commits only.

## Explicit corpse pickup UI — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15: the normal opt-in schema-17 development manifest
now installs recipient-private corpse visuals and a standard Bevy loot pane.
Right-click a corpse to view; Take requests one displayed claim; Loot all sends
serial pickup commands through the same acknowledgement stream as backpack
operations. Merely killing, opening, closing or pressing Escape grants no items.
Escape closes loot before the other open panes. Pending commands retain their
identity; full-bag rejection stops Loot all and leaves availability intact.

Private corpses use the original checked death position and frozen final death
animation, independently of the public respawning creature life. Only resident,
nearby committed loot is shown. A standard transparent mesh provides the body
picking volume: Bevy's CPU mesh raycaster reads unskinned vertex positions, so
picking the frozen skinned death pose directly missed visible bodies. This is a
presentation/picking proxy, with no physics collider or combat-target authority.
The bounded pane uses standard UI scrolling. Existing quest and vendor panes
retain their own ownership; an obstructed body requires a normal camera orbit.

Actual normal launcher/local OIDC/encrypted Renet/PostgreSQL/native Linux proof:
`build/quest-loot-ui-native-v8-20261105/verification.json`. The 1280x720,
1920x1200 and fullscreen 3440x1440 flows complete the unchanged three development
quests and three ordinary wolf kills, then demonstrate read-only opening,
Escape ownership, one-item pickup, serial Loot all and partial corpse retention.
An actual database process restart/fresh login restores the same uncollected
bodies and items; collecting them leaves no pending bodies. No inventory grant
is inferred from a visual marker. Journal state/revision remain unchanged by
these ordinary item pickups. All four runs report zero GPU diagnostics on
AMD Radeon RX 7900 XTX / RADV Vulkan / Mesa 26.2.6, using PID-bound synthetic X11
input. This is Linux native evidence, not physical input, Windows or capacity
qualification. Owned services/input stop and plaintext password files are removed.

The native binary is `build/quest-loot-ui-candidate-v4-20261105`; its client source
hashes exactly match this UI implementation. It predates the separate authored
random-roll storage change. The guaranteed schema-2 pelt/token recipe is an
intentional repeatable test fixture. It is not evidence of probabilistic rates,
quest-only eligibility or item-based quest progression. [Loot tables](LOOT_TABLES.md)
define those separate semantics and the new random-profile/storage evidence.
Earlier native v1–v7 failures remain negative evidence: CLI flag validation,
saved-pose assumptions, clipped dialogue choices, bind-pose picking and body/
guide occlusion were resolved through code fixes and real input, not direct
state mutation. Screenshots at small and ultrawide sizes were inspected.

Pre-roll UI checks pass 147 client tests, 53 validator checks, 37 bundle tests,
formatting and strict workspace/all-target Clippy. Combined roll/UI checks pass 1,295 workspace/all-target tests (65 service
skips), 55 actual PostgreSQL contracts, the retained-owner roll scenario,
38 bundle tests, formatting and strict all-target Clippy; see [loot evidence](LOOT_TABLES.md). Live
possession-quest admission remains gated pending its combined pickup, deletion,
reacquisition, turn-in and restart/rejoin proof. Reading notes remains a distinct
future authored objective. Commit locally only; no push.

## Explicit headless item-journal admission — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15: new explicit
`World::enable_item_quest_journal` and `Profile::prepare_with_inventory` paths
validate possession references against the exact inventory profile already bound
to the journal. Equal catalogue IDs/definitions with different source bytes do
not substitute for that binding. Refused bindings leave World state/identity
unchanged. A successful inactive installation retains the bound profile in World
identity and creates normal generation-bound actor routers. Live/repeated
replacement stays refused.

The old enable, prepare and raw reference paths still refuse possession quests.
The existing area and guide/creature reference checks, XP binding and inactive
World requirements remain in force. This is an explicit headless/library
prerequisite: it does not initialize SQL, prove a saved player, install a normal
CLI flag/manifest, create item-quest content or implement quest-only loot filters.
Normal client/server and manifest admission remain closed pending those slices.

363 network-library tests and nine journal-wire tests pass, including existing
World/Avian and impaired encrypted journal regressions. Formatting and denied-
warning workspace/all-target Clippy pass. Evidence:
`build/quest-item-admission-network-tests-v2-20261105.log`,
`build/quest-item-admission-wire-tests-v1-20261105.log`,
`build/quest-item-admission-fmt-v1-20261105.log` and
`build/quest-item-admission-clippy-v1-20261105.log`.
No new actual SQL, native/Windows or capacity claim. Next: explicit normal
inventory/journal binding and development-only item quest, then native pickup,
split/delete reversal, replacement pickup, atomic turn-in and restart/rejoin.
Keep quest-only candidate filtering separate from rolling and perform it before
bounded paging and again at pickup. Local commits only; no push.

## Normal item-quest admission — 5 November 2026

P3-05 / P3-09 / P3-11 / P3-15. Prerequisites are the checked possession
catalogue, atomic backpack/journal reconciliation and turn-in, explicit bound
World admission, corpse pickup and durable loot rolls described above. Normal
client and server now accept opt-in `--quest-items` with the exact inventory-bound
journal. Gameplay manifest schema 18 selects that mode; earlier manifest versions
continue to refuse possession objectives. Validation requires known item
references and at least one possession objective, preserving existing guide,
creature, area, progression and inventory checks.

`content/development-gameplay-classic-quest-items.json` adds a development-only
“Pelts for the guide” quest after the western overlook: bring three wolf pelts,
consume them on turn-in and award 25 XP. It retains the three earlier quest
definitions and uses the existing independent 60% pelt / 25% token loot profile.
Those rates are provisional Theia content values. The explicitly named
`development-gameplay-classic-quest-items-fixture.json` instead uses guaranteed
schema-2 loot for repeatable native checks. Neither profile makes pelts quest-only:
ordinary pelts can drop before acceptance or after completion.

The normal flow counts committed backpack holdings across stacks. Opening a
corpse and killing another wolf give no possession credit. Splitting three pelts
into two plus one preserves Ready; confirmed deletion of the single pelt returns
the quest to Active at 2/3. A replacement pickup restores Ready. Turn-in rechecks
and consumes the three held pelts atomically with completion and XP; completed
history remains complete after consumption. Earlier permanent quest facts remain
unchanged. No banking, mail, trade or item-reading objective is added here.

Native testing exposed a presentation bug: a loot button using ActivateOnPress
could retain Bevy's Pressed component when the panel hid before pointer release.
Hidden loot buttons now clear that transient state. Reopening a corpse must work
with one normal click; the final qualification helper does not retry Loot all.
Read-only input telemetry additionally reports window focus, cursor position and
hovered UI/actor information; it cannot inject gameplay state.

Qualification is paused at the user's request before completion. Acceptance checks cover normal OIDC/account entry,
three existing quests, explicit corpse pickup, split/delete reversal, a fourth
ordinary wolf kill without item credit, replacement pickup, consumption at turn-in,
three native window sizes and actual PostgreSQL process restart/fresh login.
The expected final journal head is 24: eight actions, eleven historical events
(including the fourth kill), and five possession-changing reconciliations. Four
quests complete at 200 total XP, with zero pelts and 103 training tokens held.
Quest-only eligibility before paging and again at pickup remains the next slice.


Paused evidence and resume point:

- 147 client tests pass in `build/quest-item-client-app-tests-v2-20261105.log`;
  55 validator tests and 40 actual-validator bundle tests pass in
  `build/quest-item-client-validator-tests-v2-20261105.log` and
  `build/quest-item-client-bundle-tests-v1-20261105.log`. The later additions are
  read-only dialogue bounds/scroll telemetry; the latest optimized build and
  strict Clippy include it. Formatting and Python compilation also pass.
- `build/quest-item-client-native-v10-20261105/1920x1200/proof.json` and
  `sql.txt` qualify that complete normal-combat size only: four completed quests,
  journal head 24, 200 XP, eight actions and eleven historical events. The older
  report's top-level XP/actions/kills describe the baseline flow; final results
  are in its item proof, SQL and captured state. The report now preserves an
  explicitly named baseline and reports final counters separately.
- `build/quest-item-client-native-v11-20261105/1920x1200/proof.json` and
  `sql.txt` repeat the complete flow with the explicitly marked low-damage,
  slow-creature test fixture. Those overrides exist only in ignored test content;
  shipping/development source profiles were not weakened. Both proofs use the
  same gameplay client candidate v3. Current candidate v4 adds read-only dialogue
  geometry telemetry; its first run v14 stopped on focus loss before qualification.
- Native v1–v14 retain negative evidence for setup/input guards, layout timing,
  the corrected hidden-button Pressed state, test-only assumptions about health
  regeneration and the fourth kill's journal/threat counters, frontmost corpse
  occlusion, and the still-unresolved clipped small-window training choice.
  The helper uses supported Tab selection when an old corpse obscures the living
  wolf; it does not select through the corpse or bypass gameplay authority.
- Next: diagnose the 1280x720 dialogue body's measured bounds/content/scroll,
  expose the fourth choice with ordinary wheel input, then complete small and
  ultrawide flows plus actual database restart/rejoin. No new native Windows,
  random-rate walkthrough or capacity claim. No final qualified checkpoint yet.

The desktop was restored to 2560x1080 at 60 Hz and owned game/provider/database
processes are stopped. Preserve work locally; do not push or resume until asked.


## Normal item-quest qualification complete — 5 November 2026

The user resumed implementation. The preceding pause record and unresolved
small-window diagnosis are historical. Opt-in schema 18 is now qualified for the
normal inventory-bound journal flow described above, including live corpse
pickup, Ready reversal after deletion, replacement pickup and consumption.
Existing manifest versions remain unchanged; this does not install a campaign,
perform save migration or make drops quest-only.

The small-window failure was synthetic wheel input, not an overflowing-content
calculation error: native v16 recorded zero MouseWheel messages while the dialogue
had 231 px of content in a 194 px viewport. A temporary Linux uinput mouse now
emits wheel-axis events instead of XTEST wheel-button clicks. The existing Bevy
scroll code then moves 37 px, exposing the fourth choice at y=366 instead of the
clipped y=403. The helper clicks that exposed ordinary choice; no dialogue intent,
quest fact, inventory state or health is injected. Focus/pointer checks guard
input to the owned X11 window. Device creation requires writable `/dev/uinput`;
normal game input has no such requirement. Device destruction/descriptor closure
and owned service cleanup run after qualification.

Completed proofs using `build/quest-item-client-candidate-v5-20261105`:

- `build/quest-item-client-native-v17-20261105/verification.json`: fresh 1280x720
  flow and actual PostgreSQL process restart/fresh login at that size.
- `build/quest-item-client-native-v18-20261105/verification.json`: fresh
  3440x1440 fullscreen and 1920x1200 flows, then database restart/fresh login at
  3440x1440. It also asserts descriptor closure and virtual-device removal.

All five sessions report zero GPU diagnostics on Linux, AMD Radeon RX 7900 XTX,
RADV Vulkan, Mesa 26.2.6. Small and ultrawide screenshots were inspected. Each
fresh run ends at journal head 24, 200 XP, four completed quests, eight actions
and eleven historical events in actual SQL; held pelts are zero and tokens 103.
Rejoin restores completion and consumption without replaying rewards. Opening a
corpse and the fourth kill leave possession at 2/3 until replacement pickup;
split leaves 3/3 Ready, deletion restores Active 2/3, and turn-in consumes three.
The original three completed quest states remain unchanged.

Native proofs deliberately use guaranteed schema-2 loot and the existing
low-damage/slow-creature test fixture. The earlier v10 1920x1200 proof separately
passed unchanged normal combat; the completed multi-size checks are not a claim
of native random-rate sampling, physical mouse hardware, Windows execution or MMO
capacity. The normal schema-18 manifest retains independent 60% pelt / 25% token
content; actual random-roll SQL/owner evidence remains in [loot tables](LOOT_TABLES.md).

Current checks: 147 client tests (`build/quest-item-client-app-tests-v3-20261105.log`),
55 validator tests and 40 actual-validator bundle tests (v2/v1 logs cited in the
pause record), formatting (`build/quest-item-client-fmt-v5-20261105.log`), strict
workspace/all-target Clippy (`build/quest-item-client-clippy-v8-20261105.log`),
optimized build (`build/quest-dialogue-scroll-release-v1-20261105.log`) and Python
compilation pass. The new client delta after the earlier checks is read-only
scroll telemetry. Preserve `build/quest-item-client-qualified-v1-20261105` for
source, binary/art references and combined evidence. Continue implementation
locally with quest-only generation entitlement and fresh current-need checks;
reading/viewing notes remains a separate authored objective. No push.


## Quest-only loot integrated — 5 November 2026

Opt-in schema-19 normal development content now connects possession objectives
to conditional creature-loot availability. The native flow covers three denied
pre-accept lives, three explicit pickups after acceptance, split/delete reversal,
a fourth entitled replacement pickup, consumption and restart/rejoin at all three
supported test sizes. [Exact evidence and restrictions](LOOT_TABLES.md#normal-quest-only-loot-qualified--5-november-2026).
No new quest objective kind or campaign IDs. The earlier readable-document
requirement remains the next P3-05 / P3-11 / P3-15 slice; historical read credit
must stay separate from possession. Local commits only; no push.
