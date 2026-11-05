Stopped safely at user request, 4 November 2026:
Qualified retained inventory/quest owner work is preserved at
`build/quest-item-owner-qualified-v1-20261104`: 1,271 workspace/all-target tests,
59 service skips and two actual OIDC/encrypted-ticket/PostgreSQL owner scenarios.
Subsequent work adds a confirmed destruction UI, exact confirmed-revision command,
and support for matching inventory acknowledgements carrying newer committed bags.
Eleven inventory bridge tests, seven encrypted account tests, 144 client tests,
formatting, strict all-target Clippy and optimized builds pass for those changes.
The first 1920x1200 native run stopped at an inventory Escape-close check before
testing destruction. It has zero GPU diagnostics on RX 7900 XTX / RADV Vulkan /
Mesa 26.2.6. Additional close-pending diagnostics and a settled-close wait are
built but not rerun. The destruction UI is work in progress, not native-qualified.
Preserve current source/build at `build/quest-item-ui-wip-stopped-v1-20261104`.
[Exact scope and resume work](QUEST_ITEM_OBJECTIVES.md).
Reading/viewing notes is planned separately from possession. Live item-quest
admission stays closed; repeatable loot and full native item flow remain pending.
All owned services and desktop input stopped. Nothing committed or pushed.

Item-quest retained owner prerequisite qualified locally, 4 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: inventory initializes before item-bound journal
admission; all supported owner bag mutations select compound transactions.
Generation-bound publication preserves newer committed bag/journal/XP projections
and command replies when responses arrive late. Actual owned OIDC/encrypted-ticket/
PostgreSQL lease checks cover split/destroy/reacquire, refused missing-item turn-in,
retained turn-in after logout, rejoin and replay without duplicate consumption/XP.
The earlier historical journal owner also passes its actual service regression.
All 1,271 workspace/all-target tests (59 service skips), formatting and strict
workspace/all-target Clippy pass. Preserve
`build/quest-item-owner-qualified-v1-20261104`.
[Exact evidence and remaining native work](QUEST_ITEM_OBJECTIVES.md).
NPC/event entitlement in owner checks is synthetic; separate channels may render
on separate frames. Live World/manifest item admission stays closed. Confirmed
UI, repeatable loot and normal encrypted/native item flow remain pending. Reading
notes is recorded as a distinct planned objective. Changes stay local.

Stopped safely at user request, 3 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: item-quest storage checkpoint is preserved, plus
checked quantity destruction in core, canonical IR02-only destruction requests
and compound SQL destruction/readiness reconciliation. A real split-one then
destroy-one SQL command changes Ready 3/3 to Active 2/3; retries never remove twice.
All 347 selected core/persistence/server tests, 14 inventory/journal codec checks,
19 actual owned PostgreSQL journal contracts, formatting and strict workspace/
all-target Clippy pass. Preserve
`build/quest-item-destruction-storage-qualified-v1-20261103`.
[Behavior, limits and resume work](QUEST_ITEM_OBJECTIVES.md).
Unfinished destruction UI is set aside as an ignored patch; the client UI is
restored to the prior qualified source. Item-quest live admission remains closed.
Resume with retained owner initialization/coherent publication, confirmed UI,
per-victim-life repeatable loot and normal encrypted/native item flow. Latest
native playable remains the area checkpoint. All local; nothing committed/pushed.

Item-quest durable storage prerequisite qualified locally, 3 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: exact item-profile binding, locked current-bag
reconciliation and atomic item consumption/completion/XP/action receipts. Optional
compound move/split/equipment, vendor and reward-claim APIs reconcile all active
item quests before the same COMMIT. Accepted compound jobs drain after response
loss. Existing unbound historical quest profiles retain their hashes and paths.
All 345 selected core/persistence/server tests (48 service skips), 18 actual owned
PostgreSQL journal contracts, formatting and strict workspace/all-target Clippy
pass; durable bag/journal/XP/receipt hashes match after database restart.
Preserve `build/quest-item-storage-qualified-v1-20261103`.
[Exact evidence and pending live work](QUEST_ITEM_OBJECTIVES.md).
SQL authorization/removal fixtures are synthetic. Live World/manifest item
admission remains closed; owner publication, destruction, repeatable loot and
normal encrypted/native item flows are pending. Latest playable remains the area
client. No save migration or new full-workspace/GPU/Windows claim. Local only.

Reversible item-quest core/codec prerequisite qualified locally, 3 November 2026:
P3-05 / P3-11 / P3-15: quest schema 3 adds explicit possession requirements and
consume-on-turn-in flags in an all-required final stage. Current holdings can
change Ready back to Active while permanent facts remain. Pure turn-in rechecks
current bags, prepares consumption/completion and recalculates other quests
sharing the materials. Completed quests stay completed.
All 335 core tests, nine codec checks, 72 authoring-tool checks, formatting and
strict workspace/all-target Clippy pass. Preserve
`build/quest-item-journal-qualified-v1-20261103`.
[Behavior, evidence and pending live work](QUEST_ITEM_OBJECTIVES.md).
Normal World/manifest admission still refuses unbound item quests. No new item
SQL transaction, deletion UI, repeatable loot, encrypted/native item quest,
save migration or full-workspace claim. Area client remains the latest playable.
Changes stay local; storage/ownership integration is next.

Quest-item possession/consumption prerequisite qualified locally, 3 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: checked backpack requirements count all stacks,
preserve totals across split/merge, distinguish kept from consumed requirements
and prepare all-or-nothing consumption against the supplied current bag.
All 327 core tests, seven new requirement cases, formatting and denied-warning
workspace/all-target Clippy pass. Preserve
`build/quest-item-possession-qualified-v1-20261103`.
[User semantics, pinned Classic-emulator evidence and remaining work](QUEST_ITEM_OBJECTIVES.md).
This pure helper is not live quest progress, item deletion, repeatable loot or
a SQL reward transaction. The area client remains the latest native playable.
Reversible item objectives, synchronized storage/owner publication and real
wolf-pelt loot/split/delete/reloot/turn-in are pending. Changes stay local.

Reach-area quest normal client qualified locally, 3 November 2026:
P3-02 / P3-11 / P3-12 / P3-15 / initial P3-16: opt-in manifest 16 binds static
metric volumes to exact quest/XP content. Current alive authoritative feet
produce bounded stage-scoped batches; the retained owner commits all changed
quests and replay receipts before private publication. Overlapping multi-goal
batches now preserve monotonic journal heads above every contained revision.
All 1,169 workspace tests / 54 service skips, 51 manifest / 36 launcher checks,
14 actual PostgreSQL journal contracts, formatting and strict all-target Clippy
pass. Normal unmodified data/native OS controls complete the original two quests
plus the overlook (about 134 m west), return and turn in: head 16 / 150 XP.
Database restart/rejoin preserves all three completions, six actions and ten
event receipts. Two native starts have zero GPU diagnostics on Linux RX 7900 XTX
/ RADV Vulkan / Mesa 26.2.6. Preserve
`build/quest-reach-client-qualified-v1-20261103`.
[Exact semantics, proof and limits](QUEST_REACH_AREAS.md).
Earlier schema-15 content/playables remain; no save migration, runtime packs,
native Windows, FPS benchmark or MMO-capacity claim. All changes local.
Next: user-requested possession-based item quests; splitting remains allowed.

Reach-area queue/encrypted prerequisite qualified locally, 3 November 2026:
P3-11 / P3-15 / initial P3-16: fixed-step authority reserves one bounded owner
slot before sampling a typed stage-bound area batch. Full queues preserve retry;
unchanged batches do not repeat while commit is pending. The retained server
owner uses the qualified storage helper and publishes only committed views.
All 354 network-library tests, formatting and strict workspace/all-target Clippy
pass. New encrypted two-client delay/loss/duplicate cases at 20/30/60 Hz advance
one batch per stage, keep the other journal private and award XP only at turn-in.
Those encrypted cases use an explicit in-memory commit fixture; PostgreSQL was
qualified separately. The combined native/SQL path is still pending.
Preserve `build/quest-reach-owner-qualified-v1-20261103`.
[Evidence and remaining normal-launch/native work](QUEST_REACH_AREAS.md).
No normal manifest area role, native area quest, save migration or new full
workspace/GPU/Windows/capacity claim. Changes remain local.

Reach-area storage/worker prerequisite qualified locally, 3 November 2026:
P3-02 / P3-11 / P3-15 / initial P3-16: area-aware persistence binds exact profile
bytes, stage-scoped observations and domain-separated replay receipts. All
affected quests and their receipt commit atomically; accepted jobs survive a
dropped response and worker shutdown. Area evidence grants no XP; normal turn-in
remains the reward transaction. Thirteen actual owned PostgreSQL journal
contracts pass, including three new area cases, plus four non-SQL checks,
formatting and strict workspace/all-target Clippy. Durable table hashes match
across an actual database restart with fsync/synchronous_commit/full_page_writes.
Preserve `build/quest-reach-store-qualified-v1-20261103`.
[Exact proof and pending owner/native integration](QUEST_REACH_AREAS.md).
Trusted SQL observations are synthetic; no encrypted/native area credit,
manifest installation, save migration or new full-workspace/GPU claim. Local only.

Reach-area World adapter qualified locally, 3 November 2026:
P3-11 / P3-12 / P3-15 / initial P3-16: opt-in exact-area-byte quest profiles
validate selectors, installed namespaces and continental bounds. Samples use
current alive authoritative capsule feet, current World/actor binding and ready
collision coverage, with no client position argument or simulation I/O.
All 319 core / 352 network-library tests, 67 authoring-tool checks, formatting and
strict workspace/all-target Clippy pass; five new authority cases include
20/30/60-Hz predicate fixtures and dead-life exclusion. Preserve
`build/quest-reach-world-qualified-v1-20261103`.
[Evidence and remaining durable/live work](QUEST_REACH_AREAS.md).
The immediately preceding full workspace passed 1,160 tests / 50 service skips;
this checkpoint does not claim a new full-workspace or GPU/native run.
Normal manifests still refuse area content; no durable area credit, save
migration or new Windows/capacity claim. Changes remain local.

Reach-area pure proposal prerequisite qualified locally, 3 November 2026:
P3-11 / P3-12 / P3-15 / initial P3-16: schema 2 adds count-one area selectors;
stage-scoped batches cannot credit later stages or later acceptances. Overlap,
quorums, delayed same-stage credit, 128-goal bounds and saved-count round trips
pass. Compiler selects the minimum schema; review detects selector changes.
All 1,160 workspace tests / 50 skips, 319 core tests, 66 authoring-tool checks,
formatting and strict workspace/all-target Clippy pass. Preserve
`build/quest-reach-stage-qualified-v1-20261103`.
[Semantics and pending live integration](QUEST_REACH_AREAS.md).
Normal World/manifest admission explicitly refuses unsupported area objectives.
No live area credit, runtime pack loading, migration, new GPU or Windows claim.
Normal playable remains directional v1; all changes remain local.

Reach-area expansion, 3 November 2026: the
[finite metric geometry prerequisite](QUEST_REACH_AREAS.md) is qualified.
Quest catalogue/schema/stage-sealed evidence and live credit remain pending;
the current runtime still supports kill and semantic NPC objectives only.

# Quest engine expansion — 3 November 2026

P3-11 / P3-12 / P3-15 / initial P3-16. The user reviewed a broader Rust quest
design and authorized implementation. Keep rules in the Bevy-independent
`theia-core::quests` domain, World integration in the existing authoritative
server and standard Bevy presentation on the client. No new service, ECS,
scripting dependency or crate split is needed for the first expansion.

The current explicit development flow uses the expanded multi-quest journal,
semantic NPC choices, prerequisite/stage objectives and atomic kill/completion XP.
Schema-12 normal OIDC/PostgreSQL/native completion and database-restart rejoin
are qualified; schema-13 adds separately qualified [damage threat](CREATURE_THREAT.md)
and passes final regression and native completion/rejoin. Separate schema-14
[public NPC timing](PUBLIC_CREATURE_ATTACKS.md) and schema-15
[public visual player life](PUBLIC_AVATAR_LIFE.md) retain these quest rules and
pass normal completion/restart regression. The previous single-quest mode and playable checkpoints
remain available. Later sections record the successive qualification boundaries;
earlier "next" steps there are historical. P3-17/18 remain unspecified.

## Catalogue prerequisite

The immutable `quests::catalog` catalogue supports
stable catalogue/quest/stage/objective IDs, catalogue revision, per-quest version,
all-completed quest prerequisites, giver/turn-in NPC IDs, XP recipes, ordered
stages and multiple objectives per stage. Completion supports `all`, `any`, or
`at_least` with a positive achievable count. Objectives are `kill` (1–100) and
`npc_interaction` (one distinct NPC/action pair). Explicit `once` repeat and
`character` credit scope are the only supported initial policies.

Strict JSON rejects unknown fields, unsupported policies, duplicate/aliased
definition IDs, repeated selectors within a stage, invalid counts, missing stage
references, stage cycles/unreachable stages and missing/cyclic prerequisites.
It builds a stable event-selector index; interaction credit matches both NPC and
semantic action, not any click on that NPC. Cross-quest selectors may overlap.
The catalogue is bounded to 64 KiB, 32 quests, eight stages per quest, eight
objectives per stage and 512 total objectives. These are development budgets.

`content/quests/development-journal.json` is an explicit two-quest fixture: meet
the guide through two distinct conversations, report in a later stage, then
unlock a three-kill plus interaction quest. It does not convert or approve the
human campaign's story quests. The legacy gameplay manifest does not load it.

Validate structure with:

```sh
cargo run --locked -p theia-core --example validate_quests -- content/quests/development-journal.json
```

Structural validation does not prove NPC/creature/action existence, reward
affordability, world admission or saved-state compatibility. World admission must
cross-check those references against installed content, bind exact validated
bytes and refuse incompatible saved versions. Definition version is distinct
from character progress revision. Migration is explicit future work.

## Production catalogue direction

The current 32-quest / eight-stage / eight-objective / 512-objective / 64-KiB
budgets are development safeguards, not permanent campaign limits. The user
raised campaign scale on 2 November 2026. Keep the consolidated planning
manifest separate from executable definitions. Recommended production direction:
author manageable quest files or regional packs, validate their combined stable
ID namespace and cross-pack prerequisites through a registry, and load only
validated executable content. A regional boundary must not break a quest chain
or erase a character's completed prerequisite history.

A bounded offline core registry and [file-index checker](QUEST_PACK_AUTHORING.md)
now validate the combined namespace and cross-pack prerequisite graph. The live
single-catalogue validator still requires prerequisite references within that
catalogue; simply splitting JSON does not introduce runtime pack loading. Today's journal
also binds exact catalogue bytes and retains at most 32 quest records. Campaign
scale therefore requires both content loading changes and a bounded active
journal with separately retained completed-quest history. Measure costs before
choosing larger limits; do not remove input/resource bounds wholesale.

Before production content updates, define compatibility and explicit migration
rules for saved per-quest versions and the outer catalogue identity. The current
exact-byte binding deliberately refuses edited content rather than silently
rewriting progress. No hot reload, transparent pack swapping or production
migration is implied by the development fixture.

## Authoring agreement for concurrent story work

Cross-check the concurrent [Human Rust authoring contract](story/Theia/Campaigns/Human_Rust_Quest_Authoring_Contract.md)
and [current planning manifest](story/Theia/Campaigns/Human_1_80_Current_Quest_Authoring_Manifest.json).
Those story documents remain proposed authoring material. Their Q-series ordering
IDs, H-series crosswalk and production runtime IDs are deliberately unresolved;
do not auto-assign them or treat illustrative null fields as executable content.
The `theia.dev.*` fixtures below are independent engineering examples. The actual
initial runtime schema is narrower than the proposed full authoring schema.


Continue drafting story Markdown in the existing story vault. Keep editorial
IDs such as `H1-14` and immutable runtime IDs such as
`theia.quest.human.h1-14` separate. A title/contact change must not rename an
existing runtime ID. Record prerequisites, giver/turn-in contacts, objective IDs,
ordered stages, exact completion evidence and intended credit scope. Narrative
level bands remain planning guidance unless an explicit eligibility gate is
approved. Do not infer runtime credit from free-form prose or Markdown links.

Ordinary conversations need stable semantic interaction IDs, so greeting a guide
cannot count as reviewing evidence. Future item objectives must explicitly state
historical acquisition versus current possession and consumption at turn-in.
Critical story evidence should use persistent objective credit as required by
the campaign register, rather than relying solely on a losable inventory item.

The first exportable schema is the checked-in fixture and Rust catalogue model.
Branching transitions/choices, flags, exploration/item/escort objectives, failure,
abandonment, repeatables, party/instance scope, currency/item reward bundles,
world phasing and Luau are planned capabilities. They are not accepted silently
by this version. Keep richer narrative requirements in Markdown until their
versioned runtime capabilities and tests exist. A visual editor can later emit
the same validated definitions.

## Implementation order and acceptance

1. Qualify the bounded catalogue and offline validator; preserve the live quest.
2. Add pure proposals for a bounded multi-quest journal: prerequisites, stage
   progress, kill/semantic interaction events, pending completion and turn-in.
   One event must not cascade through several stages. Reward proposals grant
   nothing themselves; event receipt deduplication belongs to durable ownership.
3. Migrate persistence through new tables keyed by character **and quest** with
   definition identity, independent revisions and per-event/recipient/objective
   dispositions. Preserve legacy rows. Completion, XP and receipts must commit
   atomically; item/currency rewards need their own coordinated transaction.
4. Add World-authorized semantic interactions and independent quest credit events.
   Keep simulation free of awaited I/O, bound queues and retain accepted work
   through disconnect. Selection or opening a dialogue is not completion proof.
5. Add versioned transport/manifest/client adoption. Prove multiple journals,
   prerequisite unlocks, stage ordering, duplicate/reordered input, reconnect,
   database/server restart and exactly-once XP through actual encrypted clients
   and native UI before replacing the qualified playable.

Party/shared objectives require distinct eligibility and receipts for each
recipient; they cannot reuse the existing single-recipient event primary key.
World commands, if later needed, require typed permission checks and durable
outbox delivery. Add no generic script execution to the initial standard quests.

## Catalogue qualification

Seven focused checks and all 282 core tests pass, including graph references,
selector indexing, strict completion fields, version/scope bounds, definition
identity and stable input ordering. Formatting and workspace/all-target
denied-warning Clippy pass. The actual bounded offline validator accepts the
development fixture. Preserve `build/quest-catalog-qualified-v1-20261102`
(753 source files). This is a catalogue prerequisite only; no new SQL, network,
UI or multiplayer behavior is claimed.


## Multi-quest journal pure prerequisite qualified

`quests::journal` now proposes acceptance, trusted event progress and turn-in for
up to sixteen active/ready quests and thirty-two total catalogue quests per
character. Completion prerequisites unlock only after turn-in. Each quest has its
own definition version, progress revision, active stage and objective counts;
Ready and Completed remain separate. No turn-in XP is applied by this engine:
it returns a typed completion proposal for the future transactional owner.

Only the original active stage consumes a given trusted occurrence. The selector
index can update several active quests independently, but the occurrence cannot
cascade into a newly activated stage. Completed/Ready quests, pre-accept events,
future stages, unrelated targets and wrong semantic interactions do not advance.
Each actual objective increment adds exactly one revision. Saved snapshots check
exact objective IDs/bounds, prior-stage completion, zero future-stage counts,
current status/quorum, per-quest versions, prerequisites and revision arithmetic.
Duplicate quest/objective snapshot rows, unsupported fields and oversized data
refuse. Logical snapshot restore still requires the outer owner to bind exact
catalogue bytes and stable character ownership before trusting saved state.

Seven journal checks and all 289 core tests pass, with formatting and workspace/
all-target denied-warning Clippy. Preserve
`build/quest-journal-core-qualified-v1-20261102` (755 source files). These are pure
proposals, not durable quest progress or live interaction authority. Duplicate
normalized occurrences must be deduplicated by original event identity in the
durable owner; proposing the same event kind again represents a new occurrence
and can legitimately count again. Multi-quest SQL receipts and atomic completion
are the next acceptance slice. Legacy live quests remain unchanged.


## Multi-quest Store and receipts qualified

Migration 0011 adds new journal heads, per-character/per-quest entry rows,
per-recipient event receipts and action receipts. It leaves legacy quest tables
and data intact. The exact catalogue and XP-policy digest are bound at journal
creation; incompatible bytes/world/owner refuse rather than reseeding progress.
Canonical per-quest snapshot envelopes check persisted metadata and core state.
A journal-head state digest also refuses missing or logically valid rewound rows.
No active-character content migration is provided by this initial version.

Trusted normalized events advance all matching active quests in one transaction,
independently of XP kill recipes. Every original event has a durable disposition,
including pre-accept no-ops; replay cannot retrospectively count it. The receipt
key includes the character, so the same occurrence can be submitted separately
for each legitimately eligible character. The Store does not decide party or
combat eligibility: World must establish that before trusted submission.
Conflicting payloads under the same event identity refuse.

Completion locks XP before the journal and commits the quest transition, actual
capped XP and exact action receipt together. Zero-XP recipes and capped awards
still complete. Exact acceptance/turn-in retries return current progress with no
new award, even after later quests advance. Request identity conflicts refuse.
All Store operations remain async and outside `World::step`; live owner queues,
World event/NPC authority and transport/client adoption are still the next slice.

Four new owned SQL checks pass with all thirty existing SQL contracts (34 total),
including legacy migration upgrade fixtures. Actual PostgreSQL process restart
preserves exact quest snapshot fingerprints, rows, receipts and XP for six
synthetic characters. Rollback injection prevents partial completion/reward;
a held XP row prevents acknowledgement until release. All 290 core / 641 network
(ten skips) / 123 client tests and five persistence unit tests pass; the 34 ignored
SQL tests in ordinary Cargo runs are explicitly executed by the owned runner.
Formatting and workspace/all-target denied-warning Clippy pass. Proof:
`build/quest-journal-postgres-v3-20261102`; full source/proof checkpoint:
`build/quest-journal-store-qualified-v1-20261102` (759 source files).

These are trusted synthetic Store inputs, not live NPC interactions, group-credit
eligibility, new quest UI or native/Windows/MMO-capacity evidence. The previously
qualified schema-11 playable stays unchanged until normal adoption passes.


## Journal worker queue qualified

The new journal Publisher APIs use the existing shared 16-slot critical FIFO.
Commands and trusted events enqueue synchronously, returning a response receiver;
queue acceptance is distinct from commit. Full/closed queues refuse before
acceptance. Accepted journal operations run even when their receiver is dropped.
Graceful worker shutdown closes the FIFO and drains accepted work before closing
the SQL connection. Forced task/process termination is separate; this queue is
in memory and is not a durable outbox. The future character owner must retain
ownership and original request identities until outcomes resolve.

Five journal SQL contracts now pass (35 total including legacy contracts). The
new test holds a real journal row lock, proves no early acknowledgement, fills
all 16 queued slots, refuses overflow, drops queued responses and requests
shutdown. After release, all accepted work commits in order. Reload verifies
progress; a second drain commits completion and 25 XP despite dropped responses.
Stale commands and conflicting event receipts reject without stopping the worker.
The owned runner also repeats PostgreSQL restart checks for the prior journal
characters. Formatting and workspace/all-target denied-warning Clippy pass.
Evidence: `build/quest-journal-worker-postgres-v1-20261102` and
`build/quest-journal-worker-clippy.log`.

This is the SQL worker boundary only. Live character-slot retirement, World NPC
entitlement, independent combat credit routing, transport and UI adoption remain
unimplemented for the expanded journal. The qualified legacy playable is retained.


## Expanded journal character owner qualified

The optional expanded mode now uses the existing generation-bound progression
owner and its 16-slot authority-input FIFO. Legacy automatic/explicit quest modes
are exclusive with it. Trusted journal input carries actor identity, exact
catalogue identity, bounded monotonic sequence and a typed command or original
normalized occurrence. These are server interfaces; no client decoder admits
them yet. Command proposal types live in pure core and retain persistence aliases.

The async owner initializes saved progress outside simulation, processes its
FIFO in order, waits for space in the bounded SQL FIFO and publishes only Store
results. It retains the character reservation during logout and SQL waits.
Retirement closes admission to new inputs, drains accepted ones and releases the
slot only after that owner and the final movement checkpoint have finished.
Actor/catalogue/sequence mismatches fail the admission gate closed. Normal rule
refusals and receipt conflicts produce private refusals without corrupting the
journal. Forced process termination is not a durable input outbox guarantee.

Read-only character preflight now checks the fifth saved identity role, the
journal head. Missing rows remain compatible without initialization; changed
catalogue bytes refuse before ticket admission. Expanded mode requires XP and
cannot combine with either legacy quest mode. Core per-quest snapshot checks and
outer exact binding still apply during owner initialization; migration remains
explicit future work.

The actual provider/session plus encrypted-ticket qualification holds a journal
row lock while acceptance executes, queues three semantic interactions, an event
retry, completion and acceptance of its prerequisite successor, then revokes the
login and drops the lease. No uncommitted state/XP is published and fresh login
cannot bypass retirement. After release, all accepted work drains: the first
quest completes, the successor becomes active and XP increases by 25 exactly.
A second encrypted admission restores both quests and retries completion without
another award. The earlier actor port stays closed. Trusted synthetic NPC/action
inputs are explicit; this is not live dialogue or combat-credit evidence.

All 1,072 workspace tests pass, with 46 ordinary skips. The owned SQL runner
executes all 36 SQL contracts explicitly, leaving the ten network fixture skips
as stated. Formatting and workspace/all-target denied-warning Clippy pass. Proof:
`build/quest-journal-owner-postgres-v1-20261102`,
`build/quest-journal-owner-lease-v1-20261102` and
`build/quest-journal-owner-qualified-v1-20261102`. No rendering change or new native
GPU/Windows/capacity claim. Next: World reference admission, semantic NPC choice
entitlement and quest credit independent of XP; then versioned transport/UI and
normal playable acceptance.


## Expanded quest World adapters qualified

An explicit pre-admission reference check resolves all giver/turn-in contacts,
kill definitions and semantic actions against the installed initial single guide
and creature. Dialogue choice IDs are semantic action IDs; the new opt-in
`content/quests/development-journal-dialogue.json` offers the four development
interactions using the existing schema. Informational legacy dialogue does not
silently satisfy those references. The checker is read-only and activates no
new World mode. Conversation-only catalogues require neither creature nor XP.

Preparation captures the immutable logical catalogue, World incarnation and
content identity. The explicit `interact_quest_journal` adapter uses the existing
actor-life, guide, offered-node, range, revision, sequence and rate checks.
Opening/closing dialogue yields no occurrence. A successful matching choice
returns a normalized NPC/action occurrence with a server-derived original ID;
replaying the request is refused, while a later legitimate visit has a new ID.
Foreign incarnation/content tokens refuse. The current range remains the existing
3D three-metre informational interaction policy; no additional LOS claim is made.

The independent kill adapter consumes bounded trusted authoritative combat
batches, follows the installed recipient policy and deduplicates the same victim
life within the batch. It requires no matching XP recipe or enabled XP component.
Actual special attacks with competing players at 20/30/60 Hz verify that a solo
first tag-owner receives the occurrence when another player supplies the fatal
hit; a departed owner has no fallback. Nonfatal hits yield none. Event receipts
remain the durable owner's responsibility. Interaction and kill original IDs
use distinct versioned SHA-256 domains, truncated to nonzero 128-bit identities;
identity alone grants no eligibility.

Seven new focused checks pass, with the existing core/World dialogue checks,
formatting and workspace/all-target denied-warning Clippy. The 1,072-test full
workspace, 36 owned SQL and actual OIDC/ticket/retirement evidence remain in the
prior owner checkpoint; this later slice adds explicit World adapter checks,
not another full-workspace claim. Evidence:
`build/quest-journal-world-qualified-v1-20261102`.

Normal Renet routing does not invoke these APIs yet. A caller must reserve bounded
owner capacity before processing a semantic choice, retain the returned fact,
and acknowledge journal success only after SQL. The reference token is not a
saved-profile compatibility assertion or transport identity. Normal manifest,
World-to-owner routing, private journal codec/client UI and combined durable/
encrypted/native acceptance still precede replacement of the legacy playable.


## Explicit reservation/routing adapter qualified

`theia_network::quest_journal::Profile` binds the same exact catalogue bytes and
XP-policy digest as the Store, using a shared pure-core identity domain. The
actual authenticated owner qualifier asserts equality and repeats retirement/
replay acceptance with that identity. This introduces no new wire decoder or
normal manifest role.

A prepared `Router` binds one World/actor and one trusted-producer sequence.
It checks the initialized committed owner view and XP/catalogue identity, then
reserves a bounded owner FIFO slot **before** processing dialogue. A full/closed
or uninitialized owner therefore leaves the dialogue view and sequence unchanged;
the same originally refused choice can be processed after capacity returns.
A valid semantic choice forwards its original World-derived occurrence. Kill
forwarding reserves all recipient slots atomically, preserving the original
batch and producer sequence on refusal. The caller must retain refused earned
facts for retry or fail closed. Queue acceptance is never a durable success.
Only one router/producer sequence may feed a given character owner; later action
requests must share that sequence rather than creating a competing producer.

Eight focused World/channel checks pass (seven reference/interaction/reservation
checks plus actual competing-player kill/routing at all three rates), along with
all 290 core tests, formatting and workspace/all-target denied-warning Clippy.
Actual OIDC/encrypted ticket/SQL retirement is repeated successfully. Source/proof:
`build/quest-journal-router-qualified-v1-20261102`. The World routing checks use
synthetic initialized owner views and actual bounded channels; they do not prove
combined live World+SQL or multiplayer adoption. The authenticated owner proof
uses explicitly trusted synthetic quest events as before. The 36-contract SQL
checkpoint is inherited; no migration or receipt shape changed in this slice.

Normal Renet request dispatch, private journal snapshots/action codecs, standard
Bevy UI and combined durable/encrypted/native acceptance remain next. The legacy
playable remains the normal development launch. Production pack registry,
completed-history scaling and compatibility migrations remain planned separately.


## Compact private snapshot prerequisite qualified

The QJ01 codec sends actor identity, exact profile digest, durable journal-head
revision and canonical catalogue-indexed progress. Each row carries quest and
stage indices, independent progress revision/status and all of that quest's
bounded objective counts. Text/definitions come from validated content. Per-quest
versions are inferred only under the exact catalogue digest. Header/metadata plus
all 512 possible counts across 32 quest records occupy at most **925 bytes**,
proven by a structurally valid maximum fixture with sixteen completed and sixteen
active quests. No 64-KiB saved JSON upload or reliable-channel enlargement is used.

Strict decoding rejects malformed/truncated/trailing data, invalid identities,
wrong profiles, duplicate/noncanonical quest indices, unsupported status/stage,
invalid counts, future-stage credit and inconsistent revision/prerequisite state.
Pure typed reconstruction uses the same logical journal validation as saved state.
Successor checks bind actor/profile, forbid disappearing history or regressing
head/quest/counts, refuse conflicting same-revision states and freeze Ready/
Completed progress appropriately. Coalesced updates and no-op receipt-head
increments remain valid. Five codec checks and all 290 core tests pass, with
formatting and workspace/all-target denied-warning Clippy.

Command acknowledgement is now distinct from event progress. Events update the
journal while preserving the last command reply; a later event cannot hide an
accepted/turned-in reply in a coalesced observation. Trusted event conflicts fail
the character gate closed rather than pretending to be player-command refusals.
Actual OIDC/ticket/SQL qualification queues an event after the last command,
revokes login, drains retirement and verifies that the command acknowledgement
survives. Original receipt replay still adds no XP. Proof/source:
`build/quest-journal-wire-qualified-v1-20261102`.

The codec is not yet called by normal Renet/client dispatch. Action/request and
reply formats, shared router command sequencing, manifest/content binding and UI
remain separate acceptance. Its small catalogue indices are specific to this
bounded development version; production registry/history scaling may require a
new wire version. It does not make 32 quests a permanent campaign limit. No new
native GPU, Windows, Internet or MMO-capacity claim is made.


## Action/reply codec and shared producer prerequisite qualified

QK01 actions are exactly 110 bytes: current actor and NPC incarnations, exact
profile digest, client sequence, dialogue revision, original nonzero request ID,
quest index, painted quest revision and Accept/TurnIn. Logical contact IDs come
from the validated definition. No objective counts, arbitrary events or reward
amounts can be uploaded through this frame. QP01 replies are 94 bytes and bind
actor/profile, client sequence, original request ID, quest, durable journal and
quest revisions, plus a bounded refusal code. Reply validation rejects a different
pending intent or success without progress beyond the proposed quest revision.

The explicit Router checks current World/content, actor, own guide session and
range before queueing a command. It allows one pending command and shares its
internal producer sequence with semantic/kill events. Queue acceptance gives no
success reply; `take_reply` maps only the matching committed owner response back
to the original client intent. Later event updates may advance the returned view
while retaining that command acknowledgement. Caller retains a taken reply until
publication succeeds. Normal refusals remain typed; unexpected owner failures
fail closed. This still is not wired into normal UDP or the playable UI.

Eight codec checks and eight focused reference/World/router checks pass, with
formatting and workspace/all-target denied-warning Clippy. The new channel test
uses an explicit synthetic initialized/committed owner view to prove shared
sequence, pending exclusion, no early success, later-event coalescing and mapped
rule refusals. Actual SQL/OIDC retirement remains separately qualified in the
snapshot checkpoint. Source/proof:
`build/quest-journal-actions-qualified-v1-20261102`. No new native or combined
World+SQL claim.

Before normal adoption, preserve the legacy atomic configured-kill XP/objective
contract for the expanded journal. The independent normalized event API must
continue supporting quest-only creatures; that does not require weakening a
configured XP-plus-quest award into two independent commits. Add an optional-XP
kill transaction and original receipt checks, then connect normal runtime/private
journal/client dispatch and qualify real encrypted/durable/native flows.


## Optional-XP kill Store prerequisite qualified; stopped safely

`Store::award_progression_and_quest_journal` now authorizes one original victim
life and commits configured XP plus all matching journal progress and receipts
in one transaction. It locks XP before the journal. A creature with no XP recipe
still receives quest-only processing without requiring an XP row. Existing
migrations/tables remain unchanged; the generic normalized event path shares the
same transaction helper. The victim-life occurrence domain matches the explicit
World adapter and is centralized in pure core.

Paired kill receipts use a distinct versioned hash from independent events.
Replays grant no extra XP or quest progress; pre-accept no-ops cannot later count.
An existing XP-only receipt cannot silently become new quest credit, and an
independent quest-event receipt cannot silently become a paired award. The XP cap
still permits journal progress and retains the event disposition. XP policy is
exactly bound; changing it remains an explicit migration, not a replay loophole.

Three new owned contracts pass with all 36 prior contracts (39 total): positive
and capped XP with pre-accept/replay/completion, quest-only recipients without XP
state, operation conflicts, injected rollback after XP update and a held XP-lock
acknowledgement barrier. The owned runner also repeats its prior journal/process
restart checks; it does not fingerprint the new Atomic-prefixed fixtures across
restart. All 290 core, eight codec, eight focused World/router and the competing-
player credit/routing check pass, plus formatting and workspace/all-target
denied-warning Clippy. Proof: `build/quest-journal-atomic-postgres-v2-20261102` and
`build/quest-journal-atomic-qualified-v1-20261102`. The first run exposed a fixture
attempt to create two owners for one realm; the corrected fixture retains the
single owner and observes its blocked transaction through the owned connection.

The user requested a stop. Current source is qualified; services are shut down.
The new Store method is not yet called by the bounded worker/router. Resume by
carrying the original victim life through those adapters, preserving command
acknowledgements and routing one combined award. Then qualify normal encrypted
request/state dispatch, client UI and native gameplay before replacing the legacy
launch. Additional restart fingerprints for the new combined fixtures remain a
specific acceptance check. Commit `4ceff46` contains the earlier snapshot/owner/
World groundwork and is pushed; later action/atomic changes remain local.


## Original kill facts through retained owners — 3 November 2026

P3-02 / P3-10 / P3-11 / P3-12: explicit World kill routing now preserves the
original typed victim-life `KillEvent`, rather than converting it into an
independent quest-only event. It retains installed solo tag ownership, batch
bounds, all-or-none queue reservation and the shared producer sequence. Kills
without matching objectives are retained too, so a conversation-only catalogue
cannot suppress configured kill XP. The competing-hitter fixture checks this at
20/30/60 Hz, including a full queue and retry with the same victim life.

`JournalOperation::Kill` reaches the character's retained owner and shared
critical persistence FIFO. The worker calls the single optional-XP Store
transaction. Dropping a response does not cancel accepted work; graceful shutdown
drains it. Journal owners refuse legacy XP-only input and generic Kill events,
which would bypass the combined original-life transaction. Semantic interactions
remain independent events. Kill updates preserve the latest command reply.

All 40 owned PostgreSQL contracts pass, including a new held-lock/17-job worker
case: no early acknowledgement, a full 16-slot FIFO, cancelled answers, original
kill replay, XP cap, continued quest progress and drain before shutdown. The
runner now fingerprints the complete journal heads, entries, action/event and XP
receipt rows for the Atomic-prefixed fixtures before and after a real database
process restart. Quest-only cases retain the absence of an XP row. The first
runner invocation used an incorrect expected fixture total (25 instead of the
fixture's deliberately established 50); the corrected runner passes.

Actual OIDC/encrypted-ticket/SQL owner qualification queues three kills, a replay
and a no-XP occurrence after the final command, revokes login, blocks reconnect
until retirement and then replays after reconnect. Completion XP is 25, kill XP
is 75; total stays 100 through replay, the follow-up is Ready and the command
acknowledgement survives. These are trusted synthetic owner inputs; actual World
recipient routing has separate headless evidence. This does not yet establish a
combined World/SQL/UDP/native quest flow or normal client adoption.

Proof: `build/quest-journal-kill-postgres-v2-20261103` and
`build/quest-journal-kill-owner-v2-20261103`. Owned services stop and password files
are removed. Formatting and workspace/all-target denied-warning Clippy pass;
all 1,050 workspace tests pass (50 ignored; the 40 SQL contracts were executed explicitly). Next: installed World/profile binding
and private reliable state/action dispatch, then client journal UI and native
acceptance. The prior schema-11 playable stays available; changes remain local.


## Installed expanded-journal transport qualified — 3 November 2026

P1-06 / P1-08 / P3-02 / P3-10 / P3-11 / P3-12: the explicit runtime mode binds
an exact catalogue and XP policy into the World identity. Installation validates
local quest contacts, semantic choices and creature selectors before admission,
refuses legacy quest modes and refuses replacing a different installed XP policy.
The journal installation can supply its XP policy directly: a creature without
an XP recipe and a conversation-only catalogue remain valid. Legacy XP-only
installation retains its stricter installed-creature checks. No new production
content registry or cross-pack reference validation is claimed.

Character ports now carry immutable journal identity before asynchronous saved
state initialization. Admission checks it against the World. Trusted kills can
queue before the first journal observation, retaining pre-accept disposition;
player commands and dialogue mutations still wait for initialized state. The
normal runtime routes semantic choices and original kill lives through the same
producer. Instant ability results are routed after the bounded peer input batch,
so recipient lookup uses the actual credit owner rather than assuming the final
caster owns XP/rewards. Critical work remains outside simulation/SQL awaits.

The existing 8-KiB reliable channel carries owner-private QJ01 snapshots and
QP01 replies, with state sent before acknowledgement. Replies taken from the
retained owner survive a failed publication attempt. The normal slow-peer policy
then disconnects the slow connection and retains accepted work through character
retirement. One original command is in flight on the opt-in client transport;
matching actor/profile/request ID/quest, committed revisions and the actual action
state are required before clearing it. Replayed identical replies are harmless;
a snapshot by itself cannot acknowledge an intent. Four queued UI intents and
150-ms send pacing stay bounded. The client bridge exposes private journal,
reply and pending state, without introducing a normal Bevy journal panel yet.

Fifteen focused checks pass. Actual encrypted two-peer runtime cases at 20/30/60 Hz
use 20-ms delay, up to 30-ms jitter, every-eleventh-packet loss and
every-seventeenth-packet duplication. Relay counters confirm drops, duplication,
reordering and no overflow. They exercise pre-accept kills, prerequisite refusal,
ordered semantic stages, three kills, both turn-ins, private progress and replies,
and retained command acknowledgement through later progress. A fourth 30-Hz case
has no matching local XP recipe: all kill objectives still advance and both
completion rewards commit in the explicit in-memory owner. These cases use a
synthetic admission/commit adapter, not OIDC or PostgreSQL. Real OIDC/SQL retained
ownership and atomic durability have separate evidence above.

Current focused proof: `build/quest-journal-runtime-tests-v5-20261103.log`.
Formatting, denied-warning all-target Clippy, all 1,057 workspace tests (50
ignored) and the updated actual OIDC/SQL owner regression pass against final
source. The 40 SQL contracts and restart checks are inherited unchanged from the
kill checkpoint. Source/proof: `build/quest-journal-runtime-qualified-v1-20261103`.
The owned identity/database services are stopped. Normal launch
manifest/CLI, authenticated World wiring, standard Bevy journal/NPC actions and
markers, and combined native/durable acceptance remain open. The schema-11
playable and original-kill checkpoint remain preserved; changes stay local.


## Normal journal launcher/UI candidate — 3 November 2026

P2-14 / P3-02 / P3-11 / P3-12 / P3-15: schema 12 uses 21 exact-byte roles,
retaining the reset/combat bundle and replacing the legacy quest/extra completion
recipe interpretation with the expanded catalogue's embedded completion rewards.
The content-root manifest `content/development-gameplay-classic-journal.json`
references the canonical `content/quests/` definitions and semantic guide. It
never copies campaign planning content into executable definitions. Offline checks
resolve contacts, choices and creatures and refuse legacy/new role mixing.

The explicit `--quest-journal` CLI mode and authenticated service now supply the
same catalogue to World and retained persistence owners. The launcher selects it
from schema 12; earlier manifests retain their modes. The Bevy client presents
committed per-quest stages/counts/completion, guide acceptance/turn-in buttons,
retained painted intent identities and bounded pending feedback. A scrollable
tracker and aggregated guide marker prioritize Ready, then available offers,
then active work; completed quests disappear from offered actions. This marker
priority is an explicit Theia presentation policy, not an exact Classic claim.

All 127 client tests, 41 validator tests and 31 Python bundle/handoff tests pass,
plus formatting and workspace/all-target denied-warning Clippy. These include
original-intent widget behavior, missing prerequisites/current guide context,
completed prerequisite offers, committed marker aggregation, canonical nested
content snapshots and exclusive launcher mode. Proof logs use
`build/quest-journal-launch-*-v4-20261103`. Read-only owned-X11 telemetry was
extended for visible journal/actions/choices and actual target health; it grants
no progress and injects no commands.

Optimized client/server build and native acceptance are in progress. The new
`tools/qualify_quest_journal_os.py` drives actual PID-bound mouse/keyboard input,
normal login/launcher and owned SQL. Its initial private combat fixture reduces
wolf perception, speed and retaliation damage to isolate UI behavior; catalogue,
objectives and rewards stay unchanged. No native success, normal default switch,
production campaign scale or new Windows execution is claimed yet. The previous
schema-11 playable and qualified runtime checkpoints remain preserved; all
changes remain local.


## Normal expanded-journal native checkpoint qualified — 3 November 2026

The schema-12 launcher and Bevy journal candidate above now has actual normal
OIDC/PostgreSQL/encrypted/native acceptance. Fresh play passes at 1920x1200,
3440x1440 fullscreen and 1280x720. Two additional native rejoins occur after real
PostgreSQL process restarts. All five runs have zero GPU diagnostics on connected
Linux AMD Radeon RX 7900 XTX / RADV Vulkan / Mesa 26.2.6. These use PID-bound X11
synthetic mouse/keyboard events, not physical hardware or native Windows.

Every fresh flow accepts the introduction, chooses road/supplies/report, turns it
in, accepts the unlocked follow-up, equips the sword, kills three actual World
wolves, returns to the guide, chooses training and turns in. SQL has two completed
entries, four original command receipts, seven semantic/kill receipts, journal
head 11 and 125 total XP. The post-restart normal client sees the same completion
with no new offered actions, submissions or rewards. Tracker text and actual
buttons are screenshot-checked at all three sizes. The current normal launcher
remains explicit: bare `tools/play.py` still runs its walking sandbox.

The private copied combat fixture changes wolf perception to 1 m, chase/patrol
speed to 0.1 m/s and retaliation policy damage to 1, isolating quest UI from
unrelated combat balance. All canonical quest definitions, objectives, XP policy
and completion rewards remain byte-identical. This is a combined real World,
UI, identity and SQL quest proof; it does not replace prior unmodified Classic
combat/native qualification or establish campaign-scale/party/loot behavior.

The first 1920 run reached Ready after all kills but its final button did not
activate. Quest actions now use standard Bevy click-on-release, avoiding hiding
the widget while a press is held. Repeated actual actions pass. The tracker uses
ASCII separators supported by the current font, suppresses the redundant legacy
guide hint in this mode and remains above the lower-left HUD at 1280x720 with
bounded scrolling. The three initial headless intent tests, new marker test and
all 127 client tests pass after the fixes, with formatting and all-target
Clippy. The 41 validator and 31 Python results are inherited unchanged from the
same launcher slice; 1,057 workspace / 40 owned SQL checks remain the earlier
runtime/kill baseline. No fictitious fresh full-workspace total is claimed.

Proof: `build/quest-journal-launch-native-v2-20261103` (1920 + rejoin),
`build/quest-journal-launch-native-v3-20261103` (3440/1280 + rejoin), and
`build/quest-journal-client-qualified-v1-20261103` (source, optimized binaries,
prepared assets and proof). Owned services stop and password files are removed.
All changes remain local. The earlier schema-11 playable stays preserved.

With the normal database/provider environment configured as described in
[authenticated admission](AUTHENTICATED_ADMISSION.md), this local qualified kit
can run the canonical development content with:

```sh
python3 tools/play.py --classic --account-login \
  --binaries build/quest-journal-repeat-qualified-v1-20261103 \
  --gameplay-manifest content/development-gameplay-classic-journal.json \
  --players build/quest-journal-repeat-qualified-v1-20261103/art/player \
  --creature-asset build/quest-journal-repeat-qualified-v1-20261103/art/wolf/wolf.glb
```

The kit supplies the validator as well as the matching binaries. Raw development
launches use their configured combat data; the reduced-aggression proof copy is
confined to the owned qualification directory. Existing legacy quest characters
still require explicit saved-content compatibility handling; no automatic legacy
journal migration or campaign-ID allocation is introduced.


## Repeated guide choices and pre-accept disposition qualified

The shared informational choice widgets now also use the standard Bevy Button
click-on-release behavior. The same native mouse-selected choice is visited
twice before acceptance and a third time during its actual stage. Those earlier
legitimate occurrences create two no-op receipts, not retroactive quest progress
or XP. Each new visit retains its own original authoritative occurrence identity;
this is distinct from replaying the same receipt, which remains idempotent.

An actual normal OIDC/SQL native 1920x1200 flow completes both quests, makes all
four command actions once and obtains 125 total XP. Two pre-accept conversations
increase the final head from 11 to 13 and event receipts from seven to nine; final
per-quest revisions/counts are unchanged. PostgreSQL process restart and native
rejoin retain completion, receipt counts and XP with no new offers/actions. Both
runs have zero GPU diagnostics on the same connected Linux RX 7900 XTX / RADV
Vulkan / Mesa 26.2.6. The earlier 3440/1280 rendering/layout evidence is inherited;
this change alters input activation timing, not layout. It retains the explicit
private reduced-aggression combat fixture and unchanged canonical quest content.

All 127 client tests, formatting and workspace/all-target denied-warning Clippy
pass against the updated source. Proof: `build/quest-journal-repeat-native-v1-20261103`;
source/optimized binaries/assets: `build/quest-journal-repeat-qualified-v1-20261103`.
Services are stopped, password files removed and all changes remain local. The
launch command above selects this latest kit. Earlier checkpoints stay preserved.

## Bounded offline pack-registry prerequisite — 3 November 2026

P3-11 / P3-15. `quests::catalog::registry::Registry` validates immutable
executable catalogues together. Current development budgets are 32 packs, each
with the existing 32-quest / 512-objective / 64-KiB limits, and 2 MiB total input
(up to 1,024 quests). These are bounded offline development guards, not chosen
production limits or campaign-capacity evidence. Existing per-quest/stage/objective
capabilities and prerequisite-count bounds remain unchanged.

Catalogue, quest, stage and objective definition IDs share one global namespace.
Contacts and objective targets cannot alias a definition in another pack.
Prerequisites may point forward/backward across supplied packs; unknown IDs,
self/cross-pack cycles and duplicate IDs refuse. Iterative global traversal
handles a full 1,024-quest chain without recursive graph calls and permits valid
shared prerequisite diamonds. Pack/source ordering does not change quest lookup
or ownership. Each source is bounded before parsing; all prior structural and
unsupported-feature validation still applies.

Only the private registry path defers prerequisite linking. Public
`Catalog::from_json` remains closed and unchanged for runtime journals. Registry
exposes quest definitions, owning catalogue IDs and immutable metadata, never its
internal partially linked `Catalog` values. No World/transport/client/SQL/journal
loading change is made. This does not implement production pack loading,
completed-history scaling, content compatibility/migration, hot reload or the
consolidated narrative planning manifest as executable data.

Six new checks cover canonical split definitions with retained standalone
refusal, missing/cyclic/shared prerequisites, all definition kinds and cross-role
collisions, cross-pack world-target aliasing, retained strict budgets/capabilities,
and the full padded 2-MiB/1,024-quest chain plus cycle. All 296 core tests,
formatting and workspace/all-target denied-warning Clippy pass. The six-test run
was 0.11 s on this machine; this is a simple synthetic validation fixture, not
runtime/population capacity. Preserve `build/quest-registry-core-qualified-v1-20261103`.
Next is a bounded offline file/index validator with exact hashes and author-facing
errors. Ordinary development remains the qualified schema-13 threat playable.

## Offline file-index authoring workflow qualified — 3 November 2026

P3-11 / P3-15. [Executable pack/index guide](QUEST_PACK_AUTHORING.md) documents
strict schema-1 index files, current supported definitions, bounds and exact CLI
usage. `validate_quest_packs` reads 1–32 regular contained files, retains per-pack
64-KiB/32-quest safeguards, resolves their global namespace/graph and emits exact
input hashes and metadata without modifying sources. It rejects outside/aliased
paths, oversized/nonregular/missing inputs and unsupported planning fields.
Bounded typed errors identify source indexes and offending stable IDs; CLI errors
include filenames and never dump narrative prose. Successful reports explicitly
state that runtime pack loading and world-reference validation are not provided.

Seven pure registry checks and seven file/index checks pass, along with all 297
core and 44 existing gameplay-validator tests, formatting and all-target denied-
warning Clippy. The configured CI command for both validator examples passes
locally on Linux; hosted CI and Windows execution remain unverified. The actual
compiled CLI validates a split copy of the two canonical quests, reports their
exact hashes, refuses a missing prerequisite with no success output, and leaves
both valid/invalid source sets unchanged. This proves offline authoring validation,
not World loading, campaign runtime scale or saved-state migration. Current
normal gameplay/source/binaries remain the threat v2 playable.

Preserve `build/quest-registry-cli-qualified-v1-20261103`; raw executable proof is
`build/quest-registry-cli-actual-v1-20261103`. Next is a bounded closed-catalogue
handoff for reviewed playable subsets; production pack loading and separate
completion history/migration remain distinct future slices.

## Closed runtime-catalogue handoff qualified — 3 November 2026

P3-11 / P3-15. `Registry::compile_catalogue` produces a closed, bounded reviewed
subset with explicit output catalogue ID/revision and every transitive quest
prerequisite. It preserves definition IDs, versions, stage rules, objective order,
contacts, XP and event indexes. Root/source ordering is deterministic; 32-quest,
512-objective and 64-KiB overflow refuses without truncation. Only the fully
reparsed closed `Catalog` can be exposed to a live journal.

The existing offline tool adds explicit `--compile-selection`, with an 8-KiB
strict request, catalogue stdout and bounded stderr provenance binding exact
registry/selection/output hashes. Default validation remains unchanged. Seven
core compilation and four additional file/compiler checks pass; actual CLI
repeat bytes/hashes, no-input-mutation, missing-root refusal, normal schema-13
manifest acceptance and unknown installed-contact refusal pass. All 1,089
workspace tests (50 skips), 304 core, 45 gameplay-validator and 11 authoring-tool
checks, formatting and all-target denied-warning Clippy pass. No rendering or
persistence change; native/SQL evidence is inherited.

[Current authoring workflow](QUEST_PACK_AUTHORING.md). Preserve source, tools and
proof at `build/quest-compile-qualified-v1-20261103`. This is an offline handoff,
not runtime pack loading, 1,024-quest active journals or saved-version migration.
Compiled metadata/definition changes can still cause exact-byte save-profile
incompatibility; the compiler never rewrites character progress or installs a
new normal bundle automatically.
