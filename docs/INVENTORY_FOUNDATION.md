Vendor continuation, 2026-10-30: purchases now share the existing bag revision,
sequence, pending controller and durable owner. The owner closes new acceptance
and drains accepted queued as well as executing item work before releasing a
retiring character. Actual ordinary-item/reward/vendor lease checks pass.
[Qualified barter integration and proof](VENDOR_BARTER_FOUNDATION.md): final
unchanged-source workspace passes 433 tests/30 skips. Earlier backpack
qualifications remain preserved.

# Inventory foundation — P3-05 initial durable backpack slice

Status: initial opt-in durable/native backpack slice qualified on Linux.
Broader P3-05 and production readiness remain open. Actual prerequisites are checked content identifiers,
character/account authorization, asynchronous PostgreSQL ownership and bounded
reliable commands. Creature AI, vendors, equipment and loot are separate features.
The imported task list is a scope guide, not a requirement to introduce those
systems before a backpack can work.

Build one opt-in development backpack with a fixed, bounded slot count and checked
item definitions. Development capacity and stack limits are test parameters,
not final game design. Use stable item definition IDs, checked positive stack
counts and explicit empty slots. Item instances needing independent identity or
modifiers belong to later equipment work. Item definitions have no renderer or
persistence dependencies.

The server owns contents. The first operations are moving a stack, merging
compatible stacks and splitting a checked amount into an empty slot. Preserve
item totals exactly, reject out-of-range slots, zero amounts and stale revisions,
and define self-moves explicitly. Commands must never specify authoritative item
contents, a different character owner, arbitrary grants or a final balance.
Development seed contents are an explicit authority fixture, applied once through
the durable boundary, not recreated on every join.

For the persistent path, acknowledge an accepted mutation only after the atomic
PostgreSQL transaction commits. Bind requests to the admitted character and a
bounded operation identity/revision. One intent may be outstanding per character. An exact retry of the latest
committed revision/operation returns its result; different input at that revision
conflicts. Older requests return stale and reload current authoritative contents,
without applying again. A single latest receipt bounds retained metadata. Concurrency must not duplicate
or lose items. SQL and awaited transport work stay outside World::step. While an
inventory mutation is pending, movement/combat continue; bounded queues reject
excess work rather than grow. A lost response after COMMIT must be recoverable by
retry/reload, with no client-side success prediction becoming authority.

Use new checksummed migrations rather than editing the qualified migration
history. Record the item catalog identity used to validate stored contents and
reject unknown/incompatible definitions before exposing a usable backpack. Keep
inventory state separate from superseding ordinary pose checkpoint batches:
critical item transactions cannot be overwritten by a later position snapshot.
Maintain one admitted owner and deny foreign-character access.

Present a standard Bevy backpack panel with authoritative slot/count labels and
simple selection/action controls. Pending/rejected/committed outcomes must be
visible. Initial acceptance does not require drag-and-drop, equipment bonuses,
vendor sales, trading, item use, currency, generated icons or final art.

Acceptance: definition/slot/stack bounds and conservation checks; exact replay
versus conflicting replay; concurrent mutation and stale revision rejection;
actual PostgreSQL commit/restart and lost-response recovery; foreign-owner denial;
actual encrypted native-client contents and actions; client reconnect preserving
committed contents; injected database failure producing no false success; bounded
work while movement/combat remain active. Run formatting, denied-warning Clippy,
appropriate automated tests and native GPU UI validation. Preserve the ordinary
launcher and previous qualified resource/account paths. Record exact source,
binaries, device, evidence and Linux/Windows limitations before calling the slice
qualified. Full P3-05 and persistent-zone completion remain separate claims.


Implemented prerequisite: checked schema-1 item catalog (32 KiB / 64 definitions,
1–10,000 stack limit, bounded temporary ASCII display labels), fixed sixteen-slot
checked snapshots (4 KiB), PostgreSQL-BIGINT-compatible revisions and pure tentative
move/merge/swap/partial-split proposals. A self-move fails without advancing the
revision. Merging moves up to destination capacity and retains any remainder;
different occupied items swap. A split must be positive, strictly partial and
into an empty slot. These are development mechanics, not final bag/item design.
The catalog byte digest must be bound by the persistence/transport envelope;
a matching logical ID alone does not prove identical definitions.

Five focused tests pass, including 10,000 mixed operations checking per-item
conservation, unchanged published input, checked round trips, stale revisions,
invalid data and revision exhaustion (`build/inventory-core-tests-v1.log`). All
core tests and denied-warning core/all-target Clippy pass. The core prerequisite itself commits or grants nothing. The owned PostgreSQL
qualification below commits synthetic fixture records; no live game session
currently requests inventory data and no item state is replicated.


Implemented store/worker boundary: new immutable migration 0005 adds one checked
backpack row per character, a byte-bound catalog digest, checked revision/state and
a bounded latest-intent SHA-256 receipt. Earlier migration bytes are unchanged.
Owner/world checks and row locking precede initialization/load/mutation. Only a
trusted revision-zero seed can initialize a missing bag; re-entry never reseeds
existing contents. The worker's separate sixteen-request item queue cannot be
superseded by ordinary pose snapshots. Success and metrics follow COMMIT.
Cancelled queued callers are skipped; if a caller disappears during an executing
transaction, the transaction finishes and its receipt permits recovery. Queued or
timed-out requests never receive a fabricated committed acknowledgement.

`build/inventory-postgres-v3-20261029/verification.json` passes actual PostgreSQL
16.15 checks: both inventory contracts, all earlier migration/store/worker/character
contracts, independent-connection visibility before success, concurrent conflicting
commands, exact replay, stale commands, byte-level catalog mismatch, foreign owner,
corrupt state, revision bounds, dropped-response recovery, queue saturation and
termination of the actual blocked worker backend with no false success. A separate
write/read probe survives a real clean database process restart with identical
contents/revision/UUIDs and a replayable latest receipt; a second seed is ignored.
These are store/worker checks, not real OIDC or native inventory demonstrations.
Owned cluster stopped and plaintext credential files removed. Denied-warning
persistence/all-target Clippy passes in `build/inventory-persistence-clippy-v5.log`.

The first restart runner attempt exposed a diagnostic UUID serialization build
error; the probe now explicitly prints UUID strings without enabling a new crate
feature. Only v3 is the complete passing store/restart proof. Native integration
must also keep a retiring character reserved until any in-flight item transaction
has finished, in addition to its final pose acknowledgement; releasing ownership
on pose commit alone would be insufficient for the new item boundary.


Native integration, final workspace and ordinary launcher qualification pass.
`play.py --account-login --inventory` selects the explicit development item catalog;
`--items` overrides its file. `--resources --spell-feedback` can be combined with
it. Press I or click Backpack, select an item then a destination; Split one changes
the next destination operation. Different occupied items swap, matching stacks
merge to capacity, and Cancel clears an unsubmitted selection. Saving is pending
until an authoritative durable reply arrives; the panel never predicts contents.

The item owner is attached to a generation-bound admission lease. Both joined and
never-joined retirement retain ownership until executing item work finishes;
joined leases additionally require their exact final pose acknowledgement.
Critical SQL is not cancelled merely because a peer leaves. Ambiguous deadlines,
closed worker channels or task panic poison admission rather than release a slot
early. A genuinely full queue is a known unsubmitted busy rejection.
`build/inventory-admission-leases-v1-20261029/verification.json` verifies these
fences using actual OIDC, encrypted ticket user data and owned PostgreSQL: an
already committed pose cannot release a blocked item mutation, initialization
retains a never-joined slot, fresh admission loads committed contents/replays the
latest intent, and backend loss denies all further admission. The original
resource-disabled direct lease suite also passes in
`build/inventory-admission-legacy-v1-20261029/verification.json`. These direct lease
checks are separate from native gameplay.

IR01 commands are fixed 43-byte actor/sequence/revision/slot operations. IS01
observations hold all sixteen slots in 117 bytes using sorted catalog ordinals;
no arbitrary item contents or persistent character UUID is accepted in a command.
The configured byte identity and checked definitions validate every observation.
The runtime routes only an admitted actor's port, forwards one pending intent,
bounds rates/queues and disconnects a stalled reliable consumer. The client checks
actor/catalog/sequence/revision, rejects false success/rollback and times out
unacknowledged work without claiming success. Three codec tests and four private
bridge/client state contracts pass. The encrypted two-actor routing test passes
with synthetic item owners: the other actor and movement continue during a delayed
response. It is explicitly not SQL/OIDC durability evidence.

`build/inventory-native-live-v1-20261029/verification.json` passes two actual OIDC
accounts, owned PostgreSQL and native encrypted Linux clients. Each first client
uses sixteen synthetic native pointer presses through Bevy picking/widgets for
six durable moves/splits/swaps/merges and one rejected occupied-slot split. Distinct
final arrangements are checked against each client's own persistent character,
then preserved through logout/rejoin, actual database/authority restart and changed
default spawn. Backend loss fails closed. Sixteen visible cells form four rows and
four columns with no layout failures; final screenshots were inspected. These are
native widget tests, not operating-system mouse/browser interaction. Vulkan
validation is enabled; device is RX 7900 XTX / RADV NAVI31 / Mesa 26.2.5 kisak,
1280×720, Linux Ryzen 7800X3D. Native Windows and MMO population capacity are not
claimed. Owned provider/database stopped and private ticket files were removed.

After this initial backpack slice is qualified, the proposed next scope is
[P3-06's first status-effect ability](STATUS_EFFECTS_FOUNDATION.md). It is scoped
only; no status-effect implementation is claimed yet.


Final qualification — 2026-10-29:

- `build/inventory-workspace-tests-v1.log`: **322 passing / 15 explicit skips**,
  all workspace targets, `-j1`. `inventory-fmt-final-v1.log` and
  `inventory-workspace-clippy-v2.log` pass formatting and workspace/all-target
  denied-warning Clippy. Five HTTP helper tests pass in
  `inventory-account-helper-tests-v1.log`.
- `build/inventory-postgres-final-v1-20261029/verification.json` reruns all five
  actual PostgreSQL contracts on the final source, plus the clean-restart probes.
  Those five default skips are separately executed, not counted as default
  passes. The remaining ten skips need private asset fixtures.
- `build/inventory-mixed-launcher-v1-20261029/verification.json`: ordinary
  `play.py --account-login --inventory --resources --spell-feedback` passes
  interruption, child failure and pending-browser cancellation. Joined modes run
  the real backpack widget script, preserve revision-six contents, commit the
  final pose, close database ownership and remove private tickets/child processes.
  The resource/combat/feedback profiles load alongside the backpack; this launcher
  test does not repeat the earlier resource depletion/casting/cosmetic-stress
  proof. The mixed screenshot was inspected with health/mana and all panels clear.
- `inventory-server-dependencies-v1.txt` confirms the standalone normal server
  graph remains renderer/audio/UI/wgpu free. No critical SQL/provider/awaited
  transport work was added to World::step.

Final native client SHA-256:
`d2b1f36ecc478d068938705a5194aee78f512a52fd59fac53bd3f7ac1eefdce2`.
Ordinary server SHA-256:
`061f6696110715cb10cead885a19471f1fb920c9e01f6b913ab01130ff34b07f`.
Native qualification fixture SHA-256:
`3192ffb4cb51298a2dc549343dbb81ab0c7bf2ea07ea70803d0adb877f567f8e`.
Exact source/binaries/evidence are preserved in
`build/inventory-qualified-v1-20261029`. Earlier resource and store-only
checkpoints remain separate. No commit or push accompanies this checkpoint.

The first bag has sixteen slots and once-only synthetic training items, not a
reward/grant system. A custom `--items` file currently must retain the two training
definitions expected by that development seed. Item instances/modifiers, equipment,
item use/destruction, containers, currency, trade, vendors, loot/rewards, production
identity/HTTPS hardening and native Windows remain separate work. Resources and
combat still use their documented transient fresh-actor policy; durable items do
not make the whole persistent zone complete.
