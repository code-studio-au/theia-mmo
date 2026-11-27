# Equipment and appearance — P3-08 first slice

Status, 2026-10-30: the initial one-main-hand slice is qualified. Checked item
transfers, durable shared state, encrypted private/public observations, standard
Bevy widgets/attachments, two native clients and final workspace checks pass.
This is not complete equipment, appearance customization or production art.
Development names/appearance IDs are fixtures, not story approvals.
The dated prerequisite/integration notes below preserve their historical context.

Start with one main-hand slot and a simple original mesh attached to the existing
POLYGON development character. Reuse standard Bevy hierarchy/transforms, meshes,
materials and animation. The server owns equipped contents; a visual attachment
never grants an item or determines combat power. Keep existing movement, backpack,
resource/status and patrol profiles available as regressions.

Actual prerequisites are the checked item/backpack proposals, character ownership,
durable latest-intent receipt, bounded asynchronous inventory worker, encrypted
inventory observations and the loaded character hierarchy. Broader creature AI,
loot, classes and a complete equipment-slot system are not prerequisites.

## State and durability contract

- Keep backpack and equipment in one revisioned item state. Equip transfers one
  eligible nonstackable item from a backpack slot to main hand; replacing an item
  returns the displaced item to the vacated source slot. Unequip needs an explicit
  empty backpack destination. Failed operations leave all contents unchanged.
- Bind eligibility to checked catalog definitions, not client-provided slot types,
  item contents or asset paths. Ordinary stones/tokens cannot equip. Initially no
  stat modifiers, durability, unique item instances, two-handed rules or inventory
  expansion are implied.
- Reuse the existing transaction/owner/world checks and single-outstanding-intent
  contract. Atomically commit the complete item state and exact intent receipt
  before success acknowledgement; disconnect/rejoin and response loss must conserve
  every item. No database I/O or awaited work enters the simulation step.
- Version catalog/state/wire changes explicitly. Preserve schema-1 backpack data
  and IR01/IS01 behavior; never silently reinterpret old frames or reseed existing
  characters. Definition changes still require matching byte identity. A separate
  opt-in development catalog/fixture is acceptable; production catalog migration
  and existing-character upgrades require their own explicit qualification.

## Acceptance before claiming the slice qualified

Prove equip, replacement and unequip conservation, wrong-slot/ordinary-item
rejection, full-backpack behavior, stale revision, exhaustion and checked decoding.
Exercise real PostgreSQL commit, retry/conflict, restart and owner separation.
Use two actual encrypted clients to verify distinct equipment observations and
fresh actor/rejoin behavior. Native Linux checks must show a moving animated
character attachment, remote appearance, unequip/removal and origin shifts, while
retaining standard skin/animation behavior. Prove ordinary backpack/status/patrol
launcher regressions and cleanup, formatting, denied-warning Clippy and appropriate
tests. Record exact source, binaries and device; native Windows remains separate.

Missing joints or assets must be explicit readiness/errors, not a guessed bone or
silent world-space prop. Inspect the actual converted skeleton before choosing
its attachment contract. A placeholder mesh does not qualify final weapon art,
skinning fidelity, combat animation layering or general appearance customization.

## Inspected character prerequisite — 2026-10-30

The actual converted `build/synty-character-canonical-stage/character.glb` has one
skin with fifty joints and exactly one `Hand_R` joint (node 26), descending through
Elbow_R, Shoulder_R, Clavicle_R, Spine_03/02/01, Hips, Root and SyntyRig. This is an
asset inspection, not a rendered attachment proof. The imported hierarchy carries
scale; joint-local translations are not directly world metres. Verify complete
ancestor transforms, metre size and orientation under animation before adopting
a socket transform. Do not modify the existing character GLB or its skin weights.


## Implemented core and database prerequisite — 2026-10-30

Catalog schema 2 binds main-hand eligibility and a stable appearance ID to
nonstackable items. It requires at least one equippable definition. The separate
development equipment catalog contains stones/tokens plus a sword and wand;
ordinary schema-1 catalogs and bag bytes retain their old layout/strictness.
Schema-2 item state contains sixteen backpack slots plus one main hand inside
the existing checked 4 KiB serialized state. There is no new database table or
migration and no separate equipment revision. Equip swaps the displaced item
into the vacated source slot; unequip requires an empty explicit destination.
Existing backpack proposals preserve equipment. All failures preserve state.

Store/Publisher `mutate_equipment` uses the same checked owner/world/catalog,
transaction, locked item row, revision and bounded critical-worker queue as
backpack changes. An equipment-specific intent namespace prevents aliasing the
existing backpack receipt. Success follows COMMIT; exact latest retries recover
it, conflicting or stale intents reject. Dropping an answer after SQL begins does
not cancel executing critical work. Backend loss fails the worker closed.

Six equipment proposal tests and all five legacy inventory tests pass
(`build/equipment-core-proposals-v2.log`), including 10,000 mixed operations,
item conservation, full bags, failures, revision exhaustion, malformed states and
maximum-length IDs within the durable byte budget. Four legacy wire tests pass
(`build/equipment-legacy-wire-v1.log`), including explicit rejection of equipment
state by IS01 and of unsupported equipment profiles at the existing admission
boundary. Three persistence unit tests pass. Formatting and workspace/all-target
denied-warning Clippy pass (`build/equipment-core-store-fmt-v1.log`,
`build/equipment-workspace-clippy-v2.log`).

Actual PostgreSQL qualification passes fourteen commands:
`build/equipment-postgres-v2-20261030/verification.json`. All eight ignored SQL
contracts separately execute (five existing plus three equipment), with real
owner/catalog/world checks, COMMIT visibility from an independent connection,
reconnect/replay, concurrent intents through the sole realm-owner worker,
cancelled responses and actual backend termination. Separate writer/read
processes and a clean PostgreSQL restart preserve owner/character IDs, complete
item contents and the latest equipment receipt. Credentials are removed and the
owned cluster is stopped. The first attempt's failed two-owner race fixture is
retained as unqualified evidence; the existing ownership lock correctly refused
it. No production owner bypass was introduced.

Exact source, store probe and evidence are retained in
`build/equipment-store-qualified-v1-20261030`. This is a core/database prerequisite,
not native equipment gameplay. The 351-test full-suite/native patrol result belongs
to `build/creature-qualified-v1-20261030`, preceding these equipment changes; no
new full-workspace count is claimed here. Next: explicitly versioned encrypted
commands/state and a character attachment, keeping old profiles available.


## Encrypted/presentation integration in progress

IR02 carries backpack/equipment intents through one pending stream; IS02 retains
all seventeen slots in 120 bytes. IR01/IS01 keep their existing layout and reject
cross-profile frames. EA01 separately binds sorted runtime actor/appearance IDs
to the equipment catalog and world epoch, with at most eight rows/125 bytes. It
contains no private contents or item revisions, follows existing movement interest,
and suppresses ordinary backpack-only changes. Typed shared item commands preserve
the existing durable receipt namespaces.

Fifty focused network unit/codec checks pass (`build/equipment-public-wire-v2.log`)
and all three actual encrypted admission/item-stream tests pass
(`build/equipment-public-encrypted-v1.log`). The new two-client test uses real
impaired UDP and deliberately synthetic item owners: it proves ordered routing,
independent actors, pending-state behavior, remote equip/unequip/departure and no
public update for private moves; it is not SQL or native appearance evidence.
The database prerequisite above remains separate. Attachment/UI compile checks
pass; new native widget/mesh proofs are being prepared. Do not treat this section
as final native qualification or a new full-workspace result.


Native qualification preparation now includes original standard Bevy sword/wand
props parented to the inspected Hand_R skin joint, with explicit source-scale
validation and no colliders. Equip/Unequip widgets share the committed item stream;
new development characters seed sword/wand in backpack slots 4/5 (one-based).
`tools/play.py --account-login --equipment` selects the opt-in catalog, which
requires matching stored definitions; existing backpacks are never silently
upgraded/reseeded. A fresh owned development fixture is used for qualification.
The native widget script observes equip, replacement, rejection and unequip,
checks full-pane bounds and records actual visible mesh/socket/motion evidence.
The native run has not yet executed; compilation alone does not qualify appearance.


## Native integration passed; final workspace qualification pending

The final hardened decoder passes the six equipment/five inventory core tests
(`build/equipment-core-proposals-v3.log`) and all eight actual PostgreSQL contracts,
writer/read processes and clean restart (`build/equipment-postgres-v3-20261030`).
An omitted schema-2 main-hand field is invalid, including in an actual stored row;
only an explicit JSON null represents an empty hand. Existing bag bytes retain
schema 1. No successful response precedes a durable item commit.

`build/equipment-native-v3-20261030/verification.json` passes: two actual local
OIDC accounts, owned PostgreSQL and native encrypted clients use Bevy widgets to
equip, replace, reject an ordinary item, unequip and re-equip. Each performs ten
pointer presses, four commits and one rejection. Distinct arrangements and main
hands match each own character's SQL contents through fresh actor rejoin, actual
database/authority restart and changed spawn near 15 km; backend loss fails closed.
Local/remote props are actually visible and move with the standard animated hand.
Grip offset is 3 cm; observed socket-position error is zero and maximum scale error
is below 0.000019 (under 0.002%). Complete widget-pane bounds and Vulkan validation
pass. Owned services/tickets/credentials are cleaned up. Screenshots were inspected.
This does not resolve the existing character skin approximation or qualify authored
holding/combat poses, stat bonuses, general equipment slots or production art.

Both the equipment and legacy backpack mixed launchers pass all three cleanup
modes with patrol/status/resource/feedback profiles enabled:
`build/equipment-mixed-launcher-v2-20261030/verification.json` and
`build/equipment-legacy-backpack-launcher-v1-20261030/verification.json`.
The mixed launcher loads combat but does not cast. A separate real native combat
widget regression (`build/equipment-combat-widget-regression-v1-20261030`) passes
seven pointer/seven keyboard casts, thirteen hits and two deaths/respawns. Empty
full-width combat layout containers explicitly ignore picking; their actual child
button remains interactive and the backpack buttons receive input underneath.

Failed first native capture and mixed-launcher attempts are preserved as
unqualified evidence. The first requested two screenshots of one window in one
frame; captures now occur on separate frames. The mixed run exposed transparent
combat layout containers intercepting Equip; the pointer policy is fixed and both
inventory/combat widget paths pass. These failures are not counted as passes.

The final equipment native logs report Linux Ryzen 7800X3D / RX 7900 XTX / RADV
NAVI31 / Mesa **26.2.6** kisak Vulkan, 1280×720. Earlier patrol native/baseline logs
report Mesa 26.2.5; do not attribute a driver change to the game source. The final
same-binary one-client empty saved-terrain baseline passes at **782.33 FPS uncapped**,
validation off (`build/equipment-legacy-baseline-v1-20261030/verification.json`).
This is a bounded empty-world traversal, not populated-world or MMO capacity.
No native Windows or physical OS mouse/browser qualification is claimed.

The final full-workspace/all-target run is in progress. No new full-suite count or
final equipment checkpoint is claimed yet. The prior qualified patrol and scoped
core/database snapshots remain intact. The proposed next P3-09 scope is in
[LOOT_REWARDS_FOUNDATION.md](LOOT_REWARDS_FOUNDATION.md).


## Initial main-hand slice qualified — 2026-10-30

The final full-workspace/all-target run passes **367 tests, zero failures and
eighteen explicit skips**, across eighty groups
(`build/equipment-workspace-tests-v1.log`). All eight PostgreSQL contracts execute
separately and pass in `build/equipment-postgres-v3-20261030`; ten private-asset
skips remain skips. Formatting and denied-warning workspace/all-target Clippy pass
(`build/equipment-final-fmt-v1.log`, `build/equipment-final-clippy-v1.log`). The
standalone server normal dependency graph remains headless and excludes the
navigation backends (`build/equipment-server-dependencies-v1.txt`).

The native/launcher/combat/baseline proofs in the preceding section apply to the
exact final executables: client
`3815c4af1009a1b4066f088bcd6579f2c970fd3f10b6a192b2a79b380870e9ac`, ordinary server
`320e80d4c2557e791cc73668a651ef354fa57784dd357b1d86f9593b66c45695`, and account-native
qualification server
`f318e56a378b3502a51634b9bd3238d7733b5d3654f0d3fdc182cd3f3a30f012`.
The qualification executable uses the ordinary authenticated authority runner;
it is separately identified from the CLI server tested by mixed launchers.

Exact source, binaries, engineering docs, selected native images and proof hashes
are frozen in `build/equipment-qualified-v1-20261030`. The prior patrol and scoped
store checkpoints remain preserved. All owned game/provider/database processes
have stopped. Changes remain local and uncommitted; concurrent story edits remain
untouched. No commit/push was performed.

Use `python3 tools/play.py --account-login --equipment` with the configured local
OIDC/PostgreSQL boundary and matching development catalog. Press I, select a
backpack item and Equip selected; Unequip then an empty slot returns it. Saving
continues until the authoritative durable reply; invalid operations do not change
contents. The equipped prop is cosmetic and grants no stat/damage bonus. Existing
backpacks are never silently migrated/reseeded. General slots, item instances,
combat holding poses, final art, gear statistics, production upgrades and native
Windows remain open. Next is the separately scoped P3-09 reward/claim prerequisite.
