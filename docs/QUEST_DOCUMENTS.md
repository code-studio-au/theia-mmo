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

1. Add an explicit versioned read objective and sealed stage observation. Test
   pre-accept/future/completed stages, duplicate credit, document references and
   persistence/authoring compatibility; keep unbound runtime paths closed.
2. Bind exact inventory/document/journal bytes on both sides. Resolve ownership
   and stage eligibility under the existing async authority's bag/journal locks,
   commit replay-safe original-open receipts and affected journal before success.
   Test forged requests, moved/deleted items, concurrent changes, rollback,
   response loss, retirement and actual database restart.
3. Add bounded wire commands and standard Bevy document presentation. Display
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
