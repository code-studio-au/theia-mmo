Experimental possession-item authoring prerequisite, 3 November 2026:
Schema 3 supports explicit possess_item/count/consume_on_turn_in data in an
all-required final stage. Registry/compiler/review preserve it and flag policy
changes; old-kind-only compilation retains the minimum old schema.
Core reconciliation/turn-in proposals and codec reversal rules are qualified,
but live World/manifest item admission remains blocked pending inventory/storage
binding. Do not ship campaign item quests yet.
[Exact supported scope and remaining work](QUEST_ITEM_OBJECTIVES.md).

Experimental reach-area authoring prerequisite, 3 November 2026:
Schema 2 parses a count-one `reach_area` objective and stable `area` selector.
Registry validation, subset compilation and update review preserve it; compiler
output uses schema 2 only when selected content needs it. Schema 1 remains valid
for existing kill/semantic content and refuses this new kind. An explicit area-aware World/profile and opt-in manifest-16 area role are
implemented; combined normal-native/SQL qualification passes for the opt-in development fixture. Existing
area-free manifests refuse this kind. Do not ship campaign area quests yet.
[Exact semantics and remaining live work](QUEST_REACH_AREAS.md).

# Executable quest packs and offline registry

The offline registry lets authors validate more than 32 quests across manageable
files. The current playable still loads one closed catalogue of at most 32 quests.
Passing this checker does not yet make a cross-pack campaign loadable in the game.
The [quest engine agreement](QUEST_ENGINE.md) defines the supported gameplay
subset; narrative planning documents remain separate from executable definitions.

## Write an index beside its packs

Existing content uses the schema-1 quest catalogue format, with stable IDs,
versions and the currently supported stages/objectives. The index is another
strict JSON file:

```json
{
  "schema": 1,
  "id": "theia.authoring.human-journey",
  "revision": 1,
  "catalogues": [
    "quests/starting-village.json",
    "quests/the-road.json"
  ]
}
```

These filenames illustrate the layout. Index validation creates or installs no
campaign files; explicit subset compilation is described below. Paths are relative to the index directory. Use portable forward slashes,
ASCII letters/digits, underscores, dots and hyphens, ending in `.json`. Empty,
absolute, parent/dot components, repeated separators, outside-root symlinks and
aliased files refuse. Index and pack inputs must be regular bounded files.

Catalogue, quest, stage and objective definition IDs must be globally unique.
NPC, creature and semantic-interaction IDs are references, so the same guide or
creature can be referenced in several packs. A world reference cannot double as
any definition ID. Prerequisites name quest IDs and may resolve in any supplied
pack, including a later entry. Missing prerequisites and cycles refuse; a valid
shared prerequisite diamond is allowed. Listing order does not change quest
meaning or owning catalogue.

Keep the current supported subset: once-only character quests, ordered stages,
all/any/at-least stage completion, kill and semantic NPC-interaction objectives,
and bounded XP completion recipes. New scripting, escort/exploration/item types,
party scope and general item/currency rewards require separate implementation.
Do not put a planning manifest, Markdown transcript or proposed scripting design
into an executable pack.

## Validate actual files

From the repository root:

```sh
cargo run --locked -p theia-network --example validate_quest_packs -- path/to/index.json
```

Success prints one JSON report and exits zero. Failure exits nonzero with no
success report. Global errors identify the affected source file and stable IDs,
for example:

```text
Quest pack validation failed: quests/the-road.json: Quest theia.quest.road in catalogue 2 has unknown prerequisite theia.quest.missing
```

The report contains the index and each pack's exact SHA-256, pack IDs/revisions,
quest counts, total definition count and a combined validation identity. Whitespace
edits change the relevant byte identity. Validation does not rewrite inputs or
print narrative prose. The report explicitly says `runtime_pack_loading: false`
and `world_references_validated: false`.

The report describes bytes read by this offline invocation. Re-run after edits;
it is separate from the launcher's immutable, revalidated execution snapshot.
The normal [gameplay bundle](GAMEPLAY_MANIFEST_LAUNCH.md) still performs its own
validation and exact-byte binding before launching either process.

## Compile a closed playable subset

An explicit selection can produce a catalogue for the existing runtime format.
It selects roots, then includes **every transitive prerequisite**. Put the output
catalogue ID and revision in a separate bounded selection file:

```json
{
  "schema": 1,
  "id": "theia.runtime.starting-area",
  "revision": 1,
  "quests": ["theia.quest.the-road"]
}
```

Compile after validating the index:

```sh
cargo run --quiet --locked -p theia-network --example validate_quest_packs -- \
  path/to/index.json --compile-selection path/to/selection.json \
  > build/reviewed-quests.json
```

Compilation writes a compact closed catalogue to stdout, and one JSON provenance
summary to stderr. The summary binds exact registry/selection/output hashes and
lists selected roots and the included quest count. Inputs are unchanged. A
validation/selection/budget failure emits no catalogue and exits nonzero.

Definition IDs, per-quest versions, titles, descriptions, contacts, objectives,
prerequisites, completion rules and XP stay intact. Output metadata is explicit.
Source/root ordering does not change compiled bytes. Source whitespace changes
provenance while leaving compiled bytes unchanged when definitions are unchanged.
Objective presentation order is preserved. The result passes the existing closed
`Catalog::from_json` parser; no partially linked catalogue is exposed to journals.

The whole dependency closure must fit **32 quests, 512 objectives and 64 KiB**.
The compiler refuses larger closures rather than dropping prerequisites, stages
or objectives. A long campaign chain can therefore validate globally yet still
require the future runtime/history work before it can be played across packs.
Selection files are regular JSON files bounded to 8 KiB, schema 1, with 1–32
distinct valid root IDs and a positive output revision.

Run the existing gameplay manifest validator with the compiled catalogue selected
as its `quest` role before normal admission. Compilation alone does not establish
installed NPC/creature/action references, reward context or saved-state compatibility.
It does not install the output or change a normal bundle. Changing output metadata
or definitions can change the catalogue hash; current saved-profile checks refuse
incompatible content instead of migrating character progress automatically.

## Current development bounds

| Input | Bound |
| --- | --- |
| Index | 16 KiB; schema 1; positive revision |
| Pack paths | 1–32 distinct files; at most 256 ASCII bytes per path |
| Individual catalogue | 64 KiB; 1–32 quests; at most 512 objectives |
| Combined catalogue bytes | 2 MiB; index is separate |
| Combined quests | At most 1,024 under the above bounds |
| Per quest/stage | Eight stages per quest, eight objectives per stage, 32 prerequisites per quest |
| Successful report | 32 KiB |

These are offline development safeguards. A synthetic full 1,024-quest chain at
the 2-MiB input boundary is tested; it is not proof of authored campaign quality,
runtime loading or server population capacity. Production limits require
measurements and an explicit loading/history design.

## What remains before runtime packs

`Catalog::from_json` keeps its existing local-prerequisite validation. The pure
Registry privately links definitions across packs and exposes definitions and
metadata, never an internal `Catalog` that a live journal could mistake for a
closed catalogue. No normal CLI flag loads this index into a World or journal.

The bounded subset compiler provides the closed-catalogue handoff above.
Measured runtime pack selection, separately retained completion history and explicit
saved-version/catalogue compatibility/migration remain separate future work. No hot reload,
transparent pack swapping, automatic save migration or whole planning-document
execution is implemented. World admission must still check declared NPCs,
creatures and semantic actions against installed gameplay content.

## Earlier registry/index prerequisite checkpoint

P3-11 / P3-15. Seven core registry checks cover closed-parser preservation,
canonical split linking, global collisions/aliases, missing/cyclic/shared
prerequisites, source/definition bounds, a full iterative 1,024-quest chain and
bounded ID/source-index diagnostics. Seven file-validator tests cover exact hashes,
strict index fields, path containment/aliases, missing/nonregular/oversized files,
Unix symlinks and useful errors without echoing prose. Actual compiled CLI success
and missing-prerequisite refusal leave both sets of source files unchanged.

All 297 core tests, 44 existing gameplay-validator checks, seven file-validator
checks, formatting and all-target denied-warning Clippy pass. CI is configured to
run both authoring validator examples on Linux and Windows; this local evidence
is Linux only, with no hosted CI or native Windows execution claim. The normal
playable remains `build/creature-threat-client-qualified-v2-20261103`.

## Closed-subset handoff qualified — 3 November 2026

Seven pure compilation checks pass for canonical fidelity and event indexes,
deterministic output, bad selections/metadata, complete prerequisite closure,
32-quest/512-objective/64-KiB boundaries and all stage-completion forms. All 304
core tests, 45 gameplay-validator tests, 11 file/compiler tests, formatting and
all-target denied-warning Clippy pass. Actual compiled CLI output/provenance,
repeat identity, missing-selection refusal and normal schema-13 manifest
acceptance pass; an uninstalled contact compiles structurally but correctly
fails normal World-reference validation. All 1,089 workspace tests pass (50
service-dependent skips). Normal playable and saved state remain unchanged.
Source, both authoring executables and proof: `build/quest-compile-qualified-v1-20261103`.
Raw actual handoff evidence: `build/quest-compile-cli-actual-v1-20261103`.

## Review a closed catalogue update before installation

P3-11 / P3-15. After compiling a reviewed subset, compare it with the exact prior
executable catalogue. The read-only tool validates both closed supported-schema inputs,
reports their exact hashes and compares stable quest IDs/versions. It does not
write either source, install content, inspect saves or migrate character data.

```sh
cargo run --locked -p theia-network --example review_quest_update -- \
  previous-catalogue.json candidate-catalogue.json > quest-update-review.json
python3 -m json.tool quest-update-review.json
```

Keep these independent version numbers distinct:

| Field | Meaning |
| --- | --- |
| Quest JSON `schema` | Executable data format; currently 1 |
| Catalogue `revision` | Authored catalogue release revision |
| Quest `version` | Immutable definition revision for a stable runtime quest ID |
| Gameplay manifest `schema` | Launcher/runtime capability bundle; currently explicit versions through 15 |
| Character journal `revision` | Server-owned durable progress/action revision; never author it |

The report lists added/removed IDs, changed gameplay fields, presentation edits,
layout/order changes and version-only changes. A review-only gameplay SHA strips
titles/descriptions/objective labels, normalizes terminal `next` omission/null,
and sorts the catalogue's unordered prerequisite and identity-indexed stage/objective
arrays. Object keys sort recursively. This fingerprint is **not** the runtime
catalogue digest or a saved-state compatibility verdict. Original source bytes,
IDs, versions and ordering are never rewritten.

`quest_version_review_issues` highlights version regression or a changed gameplay/
presentation definition without a higher version. Root catalogue identity/revision
review is separate. Advisories return a report successfully; invalid executable
catalogues, unsupported planning fields, nonregular/oversized inputs and output
budget overflow refuse. Exit success means a valid review was produced, not that
an update is approved or safe to apply. No narrative text is copied into reports.

Each input keeps the current 64-KiB/32-quest closed-catalogue limits. The union has
at most 64 quest IDs and output is bounded to 32 KiB. Seven focused checks include
maximum-length IDs/union size, presentation/layout normalization, missing version
bumps, regressions, unsupported input and deterministic output. All 65 current
authoring-example checks (47 gameplay, 11 pack/compiler, 7 update review), formatting
and workspace/all-target denied-warning Clippy pass. Actual built CLI proof
verifies exact hashes, input preservation, deterministic output and refusals.
Preserve `build/quest-update-review-qualified-v1-20261103`.

**Today's runtime still refuses changed catalogue bytes against existing saved
journal identity, including presentation/layout-only edits.** A higher version or
a matching review gameplay fingerprint does not bypass that check. Explicit save
migration remains future work. World/NPC/creature/action existence still requires
the normal gameplay validator. Review compiled closed subsets here; the much
larger planning manifest and unlinked regional packs remain separate inputs for
the planning/registry/compiler workflow. No runtime pack/history limit changes.
