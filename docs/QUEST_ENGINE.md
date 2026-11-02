# Quest engine expansion — 2 November 2026

P3-11 / P3-12 / P3-15 / initial P3-16. The user reviewed a broader Rust quest
design and authorized implementation. Keep rules in the Bevy-independent
`theia-core::quests` domain, World integration in the existing authoritative
server and standard Bevy presentation on the client. No new service, ECS,
scripting dependency or crate split is needed for the first expansion.

The current single three-kill quest, PostgreSQL receipts, XP-only atomic turn-in,
private journal and guide markers remain the qualified live flow. Do not replace
them until the expanded persistence/authority/transport/native slice passes.
The archived threat World draft remains separate; this continuation prioritizes
the user's quest-design alignment. P3-17/18 remain unspecified.

## Catalogue prerequisite

The initial `quests::catalog` catalogue is opt-in pure/offline code. It supports
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

This registry and cross-pack validation are planned, not implemented. Today's
single catalogue validator requires prerequisite references within that catalogue;
simply splitting the JSON does not introduce cross-pack support. Today's journal
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
