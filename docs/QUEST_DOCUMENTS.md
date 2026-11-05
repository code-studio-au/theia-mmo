# Readable quest documents

P3-05 / P3-11 / P3-15, 5 November 2026.

The user requested quest conditions for reading or viewing an item note/message.
Keep that historical action separate from possessing or looting the item. The
current playable quest-loot checkpoint is `65977bd`; readable document runtime
adoption through normal manifests and the native reader is still pending. Explicit
Store/worker, retained-owner and World support is implemented below. Earlier
prerequisite sections describe their own checkpoints.

## Implemented prerequisite: pure document catalogue

Core `documents::Catalog` schema 1 maps a known item to a stable document ID,
title and ordered plain UTF-8 pages. Each item has at most one document in a
catalogue. Unknown items, duplicate/aliased document definitions, ambiguous item
mappings, unknown fields and invalid schemas/text/bounds refuse. Document and
catalogue definition IDs cannot alias item definitions. Existing item schemas,
item bytes and inventory identities are unchanged; no catalogue is installed in
the normal development profiles yet.

Illustrative authoring data, whose item must already exist in the supplied item
catalogue:

```json
{
  "schema": 1,
  "id": "theia.example.documents",
  "revision": 1,
  "item_catalog": "theia.example.items",
  "documents": [
    {
      "id": "theia.example.roadside-note",
      "item": "theia.example.note-item",
      "title": "A roadside note",
      "pages": [
        "The eastern road is quiet.\nBring supplies."
      ]
    }
  ]
}
```

Initial development validation budgets:

| Content | Limit |
| --- | --- |
| Whole catalogue JSON | 32 KiB |
| Documents per catalogue | 16 |
| Pages per document | 4 |
| UTF-8 bytes per page | 2 KiB |
| UTF-8 bytes per title | 96 |

These are bounded implementation limits, not a permanent production allocation.
Titles are nonempty, trimmed and control-free. Pages must contain non-whitespace
text; ordinary LF/CRLF newlines and tabs are accepted and preserved. Other control
characters refuse. Pages contain plain text, without interpreted markup or
external resources. The catalogue revision must be positive.

`resolve_owned` checks a supplied bag against the captured item definitions,
the selected inventory revision, slot, document ID and actual item at that slot.
Moving, splitting or removing an item invalidates an old selected revision;
either retained split stack resolves at the new revision. It mutates no bag or
journal. This pure helper is **not** session authorization, a database lock,
an exact raw-byte profile binding, a successful client open or quest credit.

Six new tests cover text/order/lookup, forged or wrong/missing items, foreign bags,
move/split/delete behavior, unknown fields, aliases, count and UTF-8 byte bounds,
Windows line endings and malformed input. All 351 core tests pass at
`build/quest-documents-core-all-tests-v2-20261105.log`. Formatting and strict
workspace/all-target Clippy are recorded in the same checkpoint family. No new
SQL, native, UI, network or campaign-runtime claim.

## Implemented prerequisite: schema-4 logical read objectives

Quest catalogue schema 4 can represent a fixed count-one `read_document`
objective with a stable document selector:

```json
{
  "kind": "read_document",
  "id": "theia.example.objective.read-note",
  "label": "Read the roadside note",
  "document": "theia.example.roadside-note"
}
```

Schemas 1–3 reject the new kind. Do not add a quantity or consumption flag to
this objective; use a separate schema-3-or-newer possession requirement for
holdings/consumption. The existing all-required final-stage possession constraint
is preserved in schema 4. `Documents::validate_quests` checks known document
references, definition/reference namespace conflicts and possession item validity.
Offline quest parsing/compilation alone does not establish those external bindings.

Pure `prepare_document_read` observes only the currently eligible active stages
and seals their quest/version/stage IDs together with logical catalogue identities
and revisions. Its opaque result has no public constructor or deserializer.
`propose_document_read` updates only those original goals: no pre-accept credit,
future-stage cascading, duplicate progress, immediate XP or completion reward.
A new read may satisfy a later stage that asks for the same document. One read
can satisfy multiple eligible quests, respecting each stage's all/any/at-least
quorum. Losing all note items does not remove historical read credit; an independent
possession goal can still reverse Ready and refuse turn-in until reacquired.
Snapshot/restore preserves read history. Generic `propose_trusted_event` rejects
raw read selectors, just as it rejects unchecked area/possession events.

These are logical proposals, not proof of ownership, exact raw bytes, a rendered
view or a committed receipt. Session authorization, locked inventory and content
identity remain the durable owner's responsibility. Existing wire/storage
constructors, raw World preparation and normal gameplay manifests explicitly
refuse read objectives. No SQL/schema migration, command or UI is adopted yet.
The once-only character quest scope remains unchanged.

The registry/compiler preserves selectors and emits schema 4 only when the
selected closure needs reads. Selections using earlier objective kinds retain
their minimum schema 1/2/3. Update review classifies document-selector changes as
gameplay changes requiring version review; it grants no automatic migration or
World-reference validation. Existing unsupported-version tests now use schema 5.

Evidence: 357 core tests, 368 network-library tests, twelve server-library tests,
58 gameplay-validator tests, ten update-review tests and eleven pack-tool tests
pass at `build/quest-document-objective-*-tests-v*-20261105.log`. The six new
core cases cover strict schema/reference rules, original stage/replay, mixed
possession with complete item loss/reacquisition/consumption, changed revisions,
minimum-schema compilation and multi-quest/alternative completion. The server
boundary test proves legacy constructors and raw preparation remain closed; the
normal validator also refuses the new objective. Formatting and strict workspace /
all-target Clippy pass at `build/quest-document-objective-fmt-v1-20261105.log` and
`build/quest-document-objective-clippy-v2-20261105.log`. Preserve
`build/quest-document-objective-qualified-v1-20261105`. No new native/SQL claim;
qualified quest-loot playable remains `65977bd`.

## Implemented prerequisite: exact document/item/journal binding

Network `documents::Profile` and persistence `DocumentCatalog` now bind the same
raw document bytes to the exact raw inventory identity. Explicit journal
`from_json_with_documents` constructors bind that profile and attach inventory
for read-only quests as well as mixed possession/read quests. Old journal
constructors continue refusing read objectives. Rebinding an already bound
inventory or supplying an unused document context refuses.

The canonical hash dependency is:

```text
raw inventory -> document profile
raw quest + XP policy -> base journal
base journal -> optional exact area bytes -> exact inventory -> document profile
```

Each arrow uses the documented versioned domain prefix. Existing base/area/item
journal hash formulas stay unchanged. The new document suffix additionally binds
note content. Read journals with reach-area objectives require their area context;
read journals without them reject an unused area context. Document and area
catalogue/definition IDs cannot alias, including unused definitions. Bound
constructors validate known document/item references and the existing possession
restrictions before producing metadata.

Cross-layer tests change only whitespace in each of inventory, quest, XP,
document and area inputs. Every resulting journal identity changes and wire /
storage hashes agree; only inventory/document changes affect the document hash.
Read-only journals still attach the expected inventory hash, and missing,
foreign/unused or aliased contexts refuse.

At checkpoint `191d5e3`, this was **metadata-only adoption**. Legacy journal Store methods, compound
item/vendor/reward transactions and conditional generation/view/pickup refuse
read-bound profiles before SQL. Nonblocking and async Publisher paths refuse them
before acceptance into the critical queue, so unsupported calls do not fail the
shared worker. Exact binding also does not install the normal retained owner,
World or gameplay client. Boundary tests prove those gates remain closed.
No new opening command, document UI, read receipt or successful SQL read exists.

Evidence: fifteen server-library tests at
`build/quest-document-binding-server-tests-v3-20261105.log`; seven persistence
library tests at `build/quest-document-binding-persistence-tests-v1-20261105.log`.
The new disconnected Store check exercises twelve direct/compound entry points
and would panic if it reached SQL. The producer check exercises nineteen sync /
async paths and verifies the queue remains empty at its full sixteen-slot capacity.
These are refusal checks, not durable-open proof.

`build/quest-document-binding-postgres-v1-20261105/verification.json` passes all
60 existing actual PostgreSQL 16.15 contracts, seven non-SQL library tests,
reload examples and process restart. All 65 existing loot-roll rows preserve their
aggregate hash across restart. This is regression evidence for existing gameplay,
not successful read-document persistence. The validated area catalogue is retained
in the persistence profile for combined-reference checks; existing hash domains
and old profile behavior remain unchanged. Formatting and denied-warning
workspace/all-target Clippy pass at `build/quest-document-binding-fmt-v1-20261105.log`
and `build/quest-document-binding-clippy-v1-20261105.log`.
Preserve `build/quest-document-binding-qualified-v1-20261105`. Normal/native
quest-loot playable remains `65977bd`; no new native/Windows/capacity claim.

## Implemented: durable authorized document opening

The explicit document-bound Store and shared critical worker now support
`DocumentOpenCommand`: nonzero original request ID, exact document-profile hash,
document ID, selected bag revision and slot. Structural and profile errors refuse
before SQL or queue admission. The transaction authorizes owner/character/world,
locks backpack before journal, resolves the currently owned item and seals only
then-active eligible stages. Possession reconciliation and the original open
receipt commit before success. Reads do not mutate items or grant XP.

Existing quest-event receipts are reused with separate domain-separated request
IDs and command hashes; no migration or parallel receipt system was added. Exact
successful retries recover the original authorization even after item loss, without
crediting a later accepted quest or stage. A changed original intent conflicts;
a fresh open still checks current ownership. Read history survives loss, while a
separate possession requirement can reverse Ready and block turn-in. Turn-in
consumption and XP retain their existing atomic transaction.

Bound document journals now use existing initialization, bag, reward, vendor and
journal operations. Those operations cannot create historical read credit; the
unchecked generic read-event path stays closed. Accepted opens remain in the
bounded FIFO after reply loss and drain before shutdown. Missing/stale items are
ordinary rejections and do not poison the worker.

Evidence: `build/quest-document-open-postgres-v3-20261105/verification.json`
passes 66 actual PostgreSQL 16.15 contracts (six document cases), seven library
tests, seven document-character snapshots and an exact open replay across a real
database restart. Coverage includes pre-accept reads, same-note sequential stages,
wrong owner/world/profile/XP, changed slots, deletion, full backpacks, a mutation
while an open waits for the bag lock, reacquisition, forced rollback, lost replies,
and shutdown drain. All 65 existing loot-roll rows remain unchanged across restart.
The first delayed-mutation fixture violated the backpack receipt constraint before
exercising the scenario; its corrected valid update passes in this v3 run.

Fifteen server-library regression tests preserve owner/World/client refusal.
Formatting and denied-warning workspace/all-target Clippy pass in
`build/quest-document-open-{fmt,clippy}-v1-20261105.log`;
server evidence is `build/quest-document-open-server-tests-v1-20261105.log`.
Preserve `build/quest-document-open-qualified-v1-20261105`.
This proves headless storage/worker behavior, not a connected document reader:
retained owner, encrypted document commands, World admission, normal manifests
and Bevy UI remain pending. No native/Windows/capacity claim; qualified normal
quest-loot playable remains `65977bd`.

## Implemented prerequisite: bounded document transport bridge

`DO01` carries an 86-byte actor-bound open intent: ordered item sequence, exact
32-byte document hash, original nonzero 128-bit request ID, bag revision, slot and
bounded document ordinal. Ordinals follow the catalogue's stable ID ordering and
are resolved only against that exact profile. Legacy item frames refuse document
commands; generic bag proposals/storage mutations require document authority.

Explicit inventory profiles may attach documents without changing raw item,
vendor or reward identities. Client preflight requires the exact document-bound
journal. Owner ports check matching inventory; peer admission additionally checks
the authoritative World's document hash. The existing bounded control channel,
rate limit and inventory pending gate carry reads and mutations in one order.

A document-aware `DS01` inventory response appends the exact document hash. It
permits a successful read at bag revision zero; ordinary `IS01`/`IS02` responses
retain their original positive-success revision checks. Client acknowledgement
validation permits an unchanged bag only for a pending document command. Normal
mutations still require at least the next bag revision. Foreign, unsent or
unmatched replies cannot approve content; equal-revision conflicting bags refuse.
A matching committed success publishes only the retained command/profile, never
server-supplied note text. UI consumers must resolve exact local authored pages.

Evidence: all 374 then-current network-library tests pass in
`build/quest-document-bridge-network-tests-v2-20261105.log`, including three
synthetic bridge cases. Four codec tests, including an additional document-response
case, pass in `build/quest-document-bridge-wire-tests-v1-20261105.log`.
Fifteen server-library regressions, formatting and denied-warning workspace /
all-target Clippy pass in `build/quest-document-bridge-{server-tests,fmt}-v1-20261105.log`
and `build/quest-document-bridge-clippy-v2-20261105.log`.
The full core regression is recorded in
`build/quest-document-bridge-core-full-tests-v1-20261105.log`.

At checkpoint `6cc009e`, these acknowledgements used synthetic owner publications
while retained-owner and World installation remained closed. The following slice
opens those explicit boundaries; these bridge tests still establish no connected
read-credit or native UI evidence. Next connect those qualified storage
and transport boundaries, then adopt normal manifests and standard Bevy pages.
No new GPU/Windows/capacity claim; playable checkpoint remains `65977bd`.

## Implemented: retained document owner and explicit World admission

`GameAdmissions::with_document_quest_journal` and the account service accept only
the exact wire/storage document context attached to the inventory-bound journal.
Legacy journal configuration still refuses documents. Authenticated gameplay
derives the wire context from the installed matching World journal, so a caller
cannot select unrelated text. The item owner maps bounded ordinals to stable
IDs and calls the qualified async Store operation; its resulting bag and journal
use the existing generation-bound monotonic publications.

Reads retain original intent while waiting for queue space or SQL. Already
accepted queued/executing work drains before character ownership is released.
Retired sessions receive no new successful answer; rejoining and replaying the
original intent recovers its committed authorization. Stale, wrong or missing
items are ordinary nonfatal rejections. Profile/data/backend failures retain the
existing closed admission policy.

`World::enable_document_quest_journal` requires explicit exact item/document
context, matching XP, installed contacts/creatures/semantic references and an
inactive exclusive World. Even unused note definitions cannot alias installed
NPC/creature/choice IDs. Failed installation preserves identity. Legacy raw/item
installation remains closed. World identity already includes the document-bound
journal hash; simulation performs no document SQL or read-credit mutation.

`build/quest-document-owner-oidc-v4-20261105/verification.json` passes against
actual Keycloak login, an encrypted ticket handshake and PostgreSQL. Direct lease
requests prove pre-accept replay exclusion, same-note sequential stages, an open
waiting under the bag lock followed by queued deletion during retirement,
retained history, fresh-missing rejection, original retry after rejoin and normal
turn-in/retry with XP remaining 25. There are four unique open receipts for six
successful processed opens (two retries); repeated turn-in creates no second
reward. The owned services stop on exit. This fixture does not send document
messages through the game transport and does not establish native UI behavior.

375 network-library and sixteen server-library tests pass in
`build/quest-document-owner-{network-tests-v1,server-tests-v2}-20261105.log`.
Formatting and strict workspace/all-target Clippy pass in
`build/quest-document-owner-{fmt,clippy}-v2-20261105.log`.
The actual OIDC/SQL conditional-loot owner regression also passes at
`build/quest-document-owner-loot-regression-v1-20261105/verification.json`.
Preserve `build/quest-document-owner-qualified-v1-20261105`.
Normal manifests and the Bevy reader remain pending; no GPU/Windows/capacity
claim. Qualified normal quest-loot playable remains `65977bd`.

## Planned read-credit semantics

An authenticated server must validate current ownership and exact item/document
content before acknowledging a readable item open. An owned document may be
viewed without an active quest; reading credit is restricted to an eligible active
quest stage. Reading records a historical fact and cannot establish human
attention. Merely acquiring an item, viewing loot or hovering a slot grants no
read credit. Later destruction or loss does not erase a recorded read. A separate
possession objective may still require the item at turn-in, with independent
consumption policy.

Do not apply pre-accept or future-stage reads retroactively. Replaying an original
open request must preserve its original eligibility decision rather than credit a
new stage. A new authorized open can observe the then-current stage. Duplicate
reads cannot grant extra progress or rewards. Keep historical read credit out of
generic unchecked client/NPC event paths.

Remaining slices and concrete acceptance checks:

1. Qualify document intents and read approval through the actual encrypted game
   runtime. Keep synthetic transport publications distinct from the real OIDC/SQL
   retained-owner proof above, then qualify their normal combined path.
2. Adopt opt-in normal manifests and standard Bevy document presentation. Display
   only exact locally bound content after server approval; preserve current
   controls and UI input ownership. Qualify normal open/read, repeat open,
   later deletion, turn-in and save/rejoin through encrypted/native flows.

No campaign IDs or story edits are allocated here. Explicit world-document
interactions, arbitrary books/scroll scripting and quest-driven item usability
conditions are separate scope; they are not established by this item-note slice.

## Classic reference and explicit extension

The already pinned [VMaNGOS item handler at revision 6bd08b92](https://github.com/vmangos/core/blob/6bd08b92c2a8909c696cb398b124943198ca9514/src/game/Handlers/ItemHandler.cpp#L417)
locates the item at the requested bag/slot, checks page text and item usability,
and returns success or failure. The local source hash still matches the retained
record in `build/quest-document-research-v1-20261104/source.json`. That handler
does not establish a universal read-note quest objective. Server-validated
opening follows that evidenced boundary; Theia's separately authored historical
read objective is an explicit extension. No emulator code is copied.
