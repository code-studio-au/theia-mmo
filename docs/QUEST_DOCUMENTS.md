# Readable quest documents

P3-05 / P3-11 / P3-15, 5 November 2026.

The user requested quest conditions for reading or viewing an item note/message.
Keep that historical action separate from possessing or looting the item. The
current playable quest-loot checkpoint is `65977bd`; readable document runtime
adoption is still pending.

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

1. Bind exact inventory/document/journal bytes on both sides. Resolve ownership
   and stage eligibility under the existing async authority's bag/journal locks,
   commit replay-safe original-open receipts and affected journal before success.
   Test forged requests, moved/deleted items, concurrent changes, rollback,
   response loss, retirement and actual database restart.
2. Add bounded wire commands and standard Bevy document presentation. Display
   only exact locally bound content after server approval; preserve current
   controls and UI input ownership. Qualify normal open/read, repeat open,
   later deletion, turn-in and save/rejoin through actual OIDC/SQL/native flows.

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
