# Reach-area quest objectives — qualified locally, 3 November 2026

P3-11 / P3-12 / P3-15 / initial P3-16. Prerequisites are the qualified
multi-quest journal/semantic evidence, durable event receipts, authoritative
checked global movement and normal quest/restart/rejoin flows.
The current opt-in playable is
`build/quest-reach-client-qualified-v1-20261103`; the earlier directional and
schema-15 playables remain preserved.

## Intended bounded slice

Add a versioned `reach_area` objective with stable area references. An area is
a finite static metric volume, independent of collision/navigation and visible
markers. The server observes its live actor's checked global position; client
coordinates, visual overlap, minimap state and clicks cannot grant area credit.
Do not add general world phasing, discovery history, scripts or a new service.

Adopted semantics are **sampled occupancy**, not a swept entry trigger.
Inclusive X/Y/Z faces count. An active objective can qualify while the actor is
already inside, including immediately after acceptance or authoritative load.
Once durably credited, leaving does not undo that historical fact. Dead actors
cannot produce new area evidence. This objective does not require a fresh
leave/re-enter edge or a particular path; those need a separate future policy.
Very brief crossings between authoritative samples are not continuous coverage.

Seal eligibility at observation: a bounded batch names each eligible quest,
definition version, original active stage and area. Later processing must not
credit a quest accepted afterward or a later stage activated in the meantime.
Apply against the original stage set, advancing each affected quest at most once
per batch. A later stage with the same occupancy predicate may qualify through a
new observation while still inside; authors needing a revisit must not model it
as two identical occupancy predicates.

Queue acceptance grants no success. Retain accepted work through the existing
durable owner, deduplicate original receipts, publish only committed progress and
keep awaited I/O out of simulation. Definition references, outer profile hashes,
alive/current actor context and static World bounds require explicit validation.
Once-character scope remains; future abandon/retry/repeat policies will need
their own attempt identity and compatibility design.

## Current implementation boundary

The qualified geometry prerequisite is `theia_core::quests::areas`. Its separate
strict schema-1 catalogue requires stable IDs, positive revision and 1–32 named
axis-aligned boxes. Each endpoint is a checked f64 global position. Extents are
1–1024 metres on all axes; the entire input is at most 16 KiB. These are
development safeguards, not final world/campaign limits. Overlap is allowed and
definition IDs are unique; the catalogue ID cannot alias an area.

The pure predicate checks all three dimensions without a global f32 conversion.
Its iterator is bounded by the area count and deterministic by stable ID.
Geometry does not establish actor authority, World placement, reachability,
event durability or quest completion. It creates no collision body, navigation,
terrain load or rendered marker.

`content/quests/development-quest-areas.json` is an independent engineering
example west of the development guide. It is **not loaded by normal World
admission**. No Human campaign runtime IDs, editorial crosswalks or level gates
are allocated/inferred. The story authoring contract remains proposed.

Quest catalogue schema 2 and stage-sealed pure proposals are now implemented
and qualified by the regression below. Offline registry/compiler and review
support retain the new area selector. Opt-in World/reference binding and alive
occupancy sampling and durable storage/worker proposals are now qualified below.
Retained owner routing, normal-launch installation and the combined native/SQL
area quest are qualified below. Existing quest schema 1, its
32-quest/512-objective/64-KiB bounds and the qualified normal manifests stay in
place. This slice does not implement runtime multi-pack loading or saved-content
migration.

## Concrete acceptance

1. Qualify finite 3D volumes, exact faces/height, large globals/render-origin
   independence, strict/bounded inputs, overlap and definition uniqueness.
2. Add versioned objective parsing and stage-sealed pure proposals. Test delayed
   observations across acceptance/stage changes, one-batch stage progression,
   overlap and no retroactive future credit. Preserve old schemas/compiled bytes.
3. Cross-check installed World area references and bind exact bytes. Produce
   evidence only from live current authority; test stale worlds/actors, missing
   geometry, vertical exclusion, boundaries and bounded owner pressure.
4. Prove durable receipts/replay, disconnect/restart, XP turn-in and no client
   position/event forgery over actual encrypted clients.
5. Demonstrate a separate development quest with normal OS movement to the area,
   return/turn-in and restart/rejoin; preserve the qualified normal playable until
   replacement acceptance passes. Document exact fixture and platform scopes.

## Geometry prerequisite qualified

All 310 core tests pass, including six new checks for exact faces/height,
large-global/render-origin independence, strict input and extent bounds, exact
byte/count limits, overlap/ordering/aliases and the independent development
fixture. Formatting and workspace/all-target denied-warning Clippy pass.
No fresh whole-workspace test claim is made at this pure prerequisite.

The bounded read-only CLI accepts the actual file and an exactly 16-KiB padded
input, and refuses invalid extents, 16-KiB+1 input, directories and symlinks.
All six actual CLI cases pass; the source file hash remains unchanged. Its JSON
report explicitly says World binding and quest credit are not implemented:

```sh
cargo run --locked -p theia-core --example validate_quest_areas -- \
  content/quests/development-quest-areas.json
```

The first focused attempt incorrectly classified a valid 8-m Y extent as an
invalid test input; the corrected case has zero Y extent. The implementation
retains its 1–1024 m bounds. The earlier log remains evidence, not a pass.

Proof: `build/quest-reach-area-core-tests-v2-20261103.log`,
`build/quest-reach-area-geometry-clippy-v2-20261103.log`,
`build/quest-reach-area-cli-cases-v1-20261103/verification.json`.
Preserved source/CLI/proof:
`build/quest-reach-area-geometry-qualified-v1-20261103`.
Client/network/server/persistence Rust is unchanged from the directional
playable. No new SQL, transport, native GPU, Windows, reachability or live
quest-credit qualification is claimed. Quest schema-2/stage-sealed proposals
are next; the normal playable is preserved. Local only.


## Stage-scoped proposal prerequisite — 3 November 2026

Schema 2 adds exactly `{"kind":"reach_area","id":"...","label":"...","area":"..."}`.
Its required count is one; extra count fields refuse. Schema 1 explicitly refuses
this kind. Existing kill/semantic objectives remain valid in both schemas.
The compiler emits schema 2 only when the selected prerequisite closure needs an
area objective; old-kind-only output remains schema 1. Stable area references
cannot alias quest/stage/objective definitions, including across source packs.

`Journal::prepare_reached_areas` seals eligible quest, definition version, active
stage and area. It is bounded to 32 input areas / 128 eligible goals (16 active
quests × 8 objectives). `propose_reached_areas` applies to those original stages
only. Same-stage progress changes do not invalidate the original observation;
later stages and later acceptances cannot consume it. Replays saturate without
new progress. Each affected stage advances at most once; any/at-least stages stop
at their quorum to preserve the existing saved-progress invariant. Ordinary
unscoped `propose_trusted_event(ReachArea)` refuses. These pure types do not prove
that an actor was inside: the server adapter must establish that separately.

Saved count envelopes retain schema 1 and round-trip the new counts, but exact
definition/profile compatibility is still required; no save migration is added.
A batch exposes read-only metadata and has no public raw constructor or
deserialization. Offline review includes the area selector in gameplay change
digests and version advice. Normal World and manifest validation explicitly
refuse area objectives until their installed area profile is supported.

This is not live area credit. Durable receipts, installed World bounds, current
alive actor checks, encrypted publication and native traversal are still pending.

Qualification: 1,160 workspace tests / 50 service-dependent skips,
319 core tests (eight new area-proposal cases and one new compiler case),
66 authoring-tool checks (including area-selector update review), formatting and
denied-warning workspace/all-target Clippy pass. Frozen source and proof:
`build/quest-reach-stage-qualified-v1-20261103`. No rendering change; GPU,
native Windows and live area/persistence evidence are not claimed.

## Authoritative World adapter — qualified 3 November 2026

An explicit experimental `quest_journal::Profile::from_json_with_areas` binds
exact area bytes into a domain-separated quest/XP hash. The existing constructor
and area-free hashes remain unchanged. Shared core validation resolves every
area selector and checks definition/contact/event namespace aliases. World
installation additionally requires inactive predicted continental authority and
volumes inside its actual 0–16,000 m XZ domain. This initial adapter supports the
existing procedural and saved continental backends, not arbitrary future maps.

`World::observe_quest_areas` accepts a prepared binding, current actor key and
logical journal; it has no caller-supplied position. It checks World incarnation
and identity, current admitted actor, alive state, available collision and the
actor's reported terrain footprint. It subtracts the standard capsule half-height
from stored body centre to obtain checked global feet, then prepares the bounded
stage-scoped batch. No disk I/O, geometry generation or new collision/nav/render
object occurs. Direct fixture placement checks geometry/authority predicates;
it does not demonstrate normal movement or network credit.

The immutable profile can be installed explicitly by trusted Rust callers.
Normal manifests and server launcher roles still lack area installation and
continue to fail closed. At that World-only checkpoint, async ownership, SQL commit, encrypted publication
and a native area quest were pending; subsequent prerequisites are below.


Qualification: 319 core / 352 network-library tests, 67 authoring-tool tests,
formatting and denied-warning workspace/all-target Clippy pass. Five new
headless authority cases cover exact-byte hashes/legacy hash preservation,
unresolved references and namespace/bounds refusal, direct predicate placement at
20/30/60 Hz, current World/actor and collision/readiness gates, and dead-life
exclusion. They are not actual locomotion/transport/SQL/native area credit.
The first authority fixture correctly refused its unready newly admitted peer;
the corrected fixture reports coverage before evaluating occupancy and retains
an explicit readiness-negative assertion. Frozen source and logs:
`build/quest-reach-world-qualified-v1-20261103`. The prior stage checkpoint's
1,160-test full-workspace result is historical evidence, not a fresh full run
for this source. Existing playable and story authoring IDs remain unchanged.


## Durable observation storage/worker — qualified 3 November 2026

`QuestJournalCatalog::from_json_with_areas` shares core reference validation and
the network profile domain/exact-byte construction. Area-free profiles retain
their old hashes. Unbound or empty observations refuse before mutation.
`Store::record_reached_areas` authorizes the character/World, locks and checks its
exact-profile journal, applies the sealed pure proposal and commits every changed
quest row with a domain-separated observation receipt. Reusing an ID with a
different stage-bound batch conflicts; exact receipt replay returns current
progress without repeating credit. The existing receipt table is sufficient:
no SQL migration or general script/event decoder is added.

The shared bounded critical worker accepts the typed batch synchronously, retains
accepted work after response loss and drains on shutdown. Its async owner helper
waits for capacity outside simulation. New receipts may advance the journal head
even when a delayed old-stage observation changes no quest counts; replaying the
same receipt does not. Area observations never grant XP. Normal turn-in still
commits completion, XP and its action receipt atomically.

Thirteen actual owned PostgreSQL journal contracts pass, including three new
cases: forced receipt-insert rollback plus same-ID replay/conflict and turn-in;
dropped response while SQL is locked followed by shutdown/drain/reopen; delayed
acceptance exclusion plus one atomic two-quest batch, wrong-World/unbound/raw-event
refusal. All four non-SQL persistence/profile checks, formatting and strict
workspace/all-target Clippy pass. Hashes of heads, entries, event/action receipts
and XP match across an actual PostgreSQL restart with durability flags enabled.
Raw proof: `build/quest-reach-store-postgres-v2-20261103`; frozen source/proof:
`build/quest-reach-store-qualified-v1-20261103`.

These SQL observations are already-authorized synthetic facts. The World adapter
and storage path are individually tested; the retained server owner, encrypted
clients, launcher/manifest area role and normal native area quest remain pending.
No full-workspace, GPU, native Windows, capacity or save-migration claim is added.


## Retained owner queue / encrypted fixture — qualified 3 November 2026

A private Rust `JournalOperation::Areas` carries the sealed typed observation and
domain-separated per-World/actor/producer-sequence receipt. There is no client
decoder for this operation. At fixed-step sampling, the router checks its exact
profile and actor binding, current alive/prepared authority and initialized
committed journal, then reserves one bounded Port slot before preparing credit.
Full queues consume no observation or sequence and retry while still eligible.
The last accepted eligible batch suppresses unchanged repeats while asynchronous
commit is pending. An empty observation grants nothing. Occupancy can qualify a
fresh later stage through its own later observation, as specified above.

The server's retained character owner sends that original batch through the
qualified asynchronous worker helper and publishes the resulting committed view;
queue acceptance alone never updates client progress. Occurrences preserve a
pending command's reply. Normal kill and semantic event behavior remains; generic
raw area events refuse the typed-authority path.

All 354 network-library tests, formatting and denied-warning workspace/all-target
Clippy pass. A new bounded-router case proves full-queue retry, sequence
preservation, 100 unchanged uncommitted observations without repeats and no
premature watch publication. New actual encrypted two-peer cases under
delay/loss/duplication at 20/30/60 Hz prove two successive same-area stages receive
exactly two distinct batches, the unaccepted other journal remains private and
only normal turn-in awards XP.

Those encrypted cases use an explicit **in-memory commit fixture**. The actual
PostgreSQL transaction/worker/restart cases are separate evidence, not one
combined run. Frozen source/proof:
`build/quest-reach-owner-qualified-v1-20261103`. Launcher schema/area role,
fresh development content, combined native/SQL/rejoin and any area destination
presentation remain pending. Existing normal manifests and playable checkpoint
remain unchanged.

## Normal-launch bundle / multi-objective revision follow-up — qualified

Opt-in gameplay manifest 16 adds one exact-byte `quest_areas` role to the
schema-15 roles and public creature/player life contracts. The separate
`development-journal-areas.json` keeps both existing quest definitions unchanged
and adds the once-character western-overlook engineering quest. No campaign IDs
or story contract are rewritten. Existing schema-15 manifests/quest bytes stay
unchanged; exact-profile saves are not migrated.

Both executable CLIs now accept bounded `--quest-areas` bytes with expanded
journals; server startup requires exact network/persistence profile agreement and
installed World checks. The launcher snapshots the area file with the other
validated roles. Fifty-one Rust manifest tests and 36 actual-validator Python
launcher checks pass. Optimized/native/combined SQL qualification passes as recorded below.

A new three-objective overlapping-area SQL regression exposed a journal-head
invariant missing from the earlier storage checkpoint: one batch can advance a
quest's logical revision by more than one. A committed head must be at least as
high as every contained quest revision, and remain strictly monotonic.
`Journal::next_head_revision` now centralizes that rule for storage and the
encrypted in-memory fixture; the header is a snapshot version, not a receipt
count. Existing single-objective/event sequences keep their previous revisions.
The added regression failed on the prior storage source and passes after this
fix, along with all 14 actual owned PostgreSQL journal contracts and identical
durable table hashes across database restart
(`build/quest-reach-store-postgres-v4-20261103`). Earlier storage/owner checkpoints
do not cover the overlapping multi-goal invariant; use the corrected source.
Final full-workspace, Clippy and native proof pass as recorded below.

The optional normal-OS area harness extends the existing quest/login runner:
accept via the real quest button, manually face west, hold normal W to the
overlook, verify Ready without XP, walk back and turn in, then restart PostgreSQL
and rejoin the same completed character. It does not inject authoritative
positions, speed changes or debug quest events.


## Final normal client / SQL qualification — 3 November 2026

All 1,169 workspace tests / 54 service-dependent skips, 320 core / 354 network
library / 144 client tests, 51 manifest tests, 36 actual-validator Python launcher
checks, formatting and denied-warning workspace/all-target Clippy pass.
Fourteen owned PostgreSQL journal contracts, including the corrected overlapping
multi-objective case, pass with identical durable hashes across database restart.
The prior failed overlap regression is retained as evidence of the fixed
journal-head invariant; earlier store/owner checkpoints do not include that fix.

Two Linux native starts use the optimized normal launcher, actual local OIDC and
owned PostgreSQL with unchanged gameplay data. At 1920×1200, real UI inputs and
normal manual camera/W movement complete both original quests, accept the
overlook, walk from authoritative feet [8004.7853, -12.5318, 8000.0997] to
[7871.0863, -3.6001, 8000.2352] in 19.23 seconds of held W, then return and turn
in. Ready grants no XP; final turn-in reaches total 150 XP / journal head 16.
The player has not died; ordinary creature perception, movement and damage stay
unchanged. Saved rows record three completions, six actions and ten event
receipts. The second native start follows an actual database process restart and
rejoins the same character with that state intact and no repeated quest actions.
Both starts have zero GPU diagnostics on AMD Radeon RX 7900 XTX, RADV Vulkan,
Mesa 26.2.6. Screenshots of Ready and Completed were visually inspected.
This is not a physical-input, native Windows, FPS benchmark or population test.

Raw normal proof: `build/quest-reach-native-v1-20261103`.
Frozen full source, optimized binaries, art and selected proof:
`build/quest-reach-client-qualified-v1-20261103`.
Native command:

    python3 tools/qualify_quest_journal_os.py build/NEW-OWNED-OUTPUT       --binaries build/quest-reach-client-qualified-v1-20261103       --size 1920x1200 --production --repeat-interaction --reach-areas       --gameplay-manifest content/development-gameplay-classic-quest-areas.json

The opt-in bundle is deliberately separate. Existing manifests/quest definitions
remain unchanged and no character save is migrated automatically. Initial area
markers are the existing quest-giver marker and text tracker; a destination pin,
map discovery history, arbitrary region support and swept-entry triggers are not
implemented. User-requested possession-based item objectives are next; see
[research and required behavior](QUEST_ITEM_OBJECTIVES.md).
