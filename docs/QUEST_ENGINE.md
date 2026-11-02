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
