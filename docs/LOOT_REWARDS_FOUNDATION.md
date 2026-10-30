# Loot and rewards — P3-09 proposed first slice

Status, 2026-10-30: the initial opt-in personal training-kill claim slice is
qualified, with exact source/executables/evidence frozen locally in
`build/rewards-qualified-v1-20261030`. This is not a complete loot system.
Development names are fixtures, not approved story or production balance.
The design/delivery order below describes the complete proposed first slice.

Start with one nonrepeatable personal training reward, offered after a qualifying
server-owned creature kill. The player claims it through a standard Bevy widget.
This is a bounded reward/durability proof; repeatable drop tables, shared/group
loot, corpse interaction, item instances and a complete loot system remain later
work. Equipment, stats and quests do not gain unrequested effects.

## Actual prerequisites and delivery order

The current inventory/equipment stream permits one pending client intent and a
latest committed reply. An unsolicited grant can advance the bag revision while
that reply waits to be sent, which requires a separately designed ordering rule.
Use an explicit claim in the same item-intent stream for the first slice. Do not
silently add background grants to the current coalescing observation channel.

1. Define a checked reward recipe referring to known items/counts, plus a pure
   all-or-nothing insertion proposal. Bound definitions, total granted quantity,
   JSON bytes and backpack capacity. Full bags reject without consuming a claim.
2. Persist one owner/character-bound claim per development reward definition.
   Create it through the async critical worker before advertising availability.
   Duplicate qualifying events return the same claim rather than minting more
   items. Keep migrations immutable and source/catalog identities checked.
3. Claim in a transaction that authorizes owner/world, locks the item state and
   claim consistently, checks the expected item revision, inserts every item,
   consumes the claim and writes its receipt before COMMIT. Exact response-loss
   retries recover success; old/conflicting claims cannot grant twice. A full
   backpack leaves the durable claim available for later pickup.
4. Add explicitly versioned encrypted claim/list observations and a Bevy widget,
   preserving IR01/IS01 and IR02/IS02. Requests contain only a server-issued opaque
   claim ID and checked intent metadata; clients never upload grants or recipients.
5. Connect one authoritative qualifying creature death to the existing admitted
   character owner. Retain accepted critical work through disconnect/retirement.
   World::step emits bounded gameplay facts; it performs no SQL, filesystem or
   awaited transport work. Prove capacity, actor/definition/lifetime fences and
   shutdown behavior before enabling the runtime adapter.

This starts with a single known reward per character/definition to bound the first
proof and avoid an unimplemented repeated-drop backlog/expiry policy. Ordinary
NPC death is not a successful item award. If availability or item COMMIT is not
confirmed, do not display a reward-success acknowledgement. Reconnect loads
existing durable availability/consumption instead of reseeding it. A crash before
an availability commit makes no successful reward promise; the nonrepeatable
fixture can be retriggered by a later qualifying kill. Once advertised, the claim
must survive clean process/database restart until claimed.

## Acceptance before claiming the slice qualified

Prove exact item quantities, stack merging, full-bag atomic failure, invalid recipe
rejection, revision exhaustion and unchanged equipment. Actual PostgreSQL checks
must cover ownership/world/catalog binding, repeated offers, competing intents,
lost responses, restart, full-bag retry and backend loss without false success.
Exercise real encrypted clients and both protocol regressions. Native checks must
use actual widgets and each own character's durable claim/items through rejoin and
restart. Retain movement, gear appearance, combat and launcher cleanup checks.
Record format, denied-warning Clippy, appropriate tests and exact source/binaries;
do not equate fixtures with production loot, a persistent zone or MMO capacity.


## Implemented core and database prerequisite — 2026-10-30

Checked recipes bind a logical item catalog, qualifying creature definition and
once-per-character policy. Bounds: 32 KiB catalog, sixteen definitions, eight
unique item types and 10,000 total units per reward. Definitions reference known
items; invalid/unknown/duplicate fields and unsupported policy values reject.
Insertion merges existing stacks then fills empty slots in deterministic order,
with at most two bounded scans per item type. It preserves equipment, advances
the shared item revision once and returns no partial result on capacity failure.
These are trusted pure proposals, not recipient authorization or successful awards.

Immutable migration `0006_reward_claims.sql` stores one claim per realm/character/
reward definition, a server-generated UUID, checked world/profile identity and
consumption/receipt. The profile binds exact reward bytes and item-catalog digest.
Offer/list/claim authorize the current character owner/world and validate item
state. Offer availability follows COMMIT; duplicate offers retain the same ID and
consumption. Pickup locks bag before claim, inserts all contents and updates both
item receipt and claim consumption in one transaction before success. Exact latest
retries recover; superseded/consumed requests never grant again. Full bags retain
availability. The existing bounded sixteen-job critical worker handles these jobs;
no additional service/queue or I/O in World::step was introduced.

Seven new pure grant tests plus eleven inventory/equipment regressions pass
(`build/rewards-core-proposals-v1.log`). Formatting and workspace/all-target
denied-warning Clippy pass (`build/rewards-final-fmt-v1.log`,
`build/rewards-workspace-clippy-v1.log`). Actual PostgreSQL qualification passes
seventeen commands and all eleven SQL contracts, including the five preceding
persistence/inventory and three equipment contracts:
`build/rewards-postgres-v1-20261030/verification.json`.

The three new SQL contracts prove owner/world/profile binding, one durable offer,
latest-receipt recovery, a shared transfer/claim revision, full-bag retry, forced
failure after the tentative bag update with atomic rollback, cancelled answers,
queue saturation and actual backend termination without false success. Distinct
writer/read processes plus clean database restart preserve both an unclaimed offer
and a consumed claim/items/receipt. Old migration upgrades/checksums, account/
character/inventory/equipment regressions pass; earlier migrations were not edited.
Owned cluster stopped and password files were removed.

Exact source/store probe/evidence: `build/rewards-store-qualified-v1-20261030`.
This is a core/database prerequisite. It does not prove qualifying kills, gameplay
rewards, claim networking/UI or live character-retirement fencing. The preceding
367-test/native equipment result belongs to `build/equipment-qualified-v1-20261030`;
no new full-workspace count or native reward result is claimed here. Next: connect
the typed claim intent and bounded owned-claim observations to the existing secure
item stream and per-character owner before publishing any live reward offer.


## Runtime integration in progress — 2026-10-30

RC01 carries only runtime actor identity, ordered item sequence, expected revision
and the opaque nonzero claim key (52 bytes). RW01 carries a private bounded owned
availability list, reward-profile digest and monotonic observation generation
(61 bytes plus seventeen per available recipe, at most sixteen). Nil identities,
unknown recipes, duplicate/unordered rows, mismatched profiles, generation aliases,
truncation/trailing bytes and legacy-protocol claim aliases reject. Availability
never acknowledges an item intent or advances its bag revision. A claim shares
one pending request with backpack/equipment and uses the existing durable reply.

The opt-in development policy credits the last fatal hit on the installed current
creature life to its live player source. Static targets and stale lives/sources
cannot qualify. A bounded trusted owner queue accepts these facts outside
World::step. The existing character owner commits availability before publishing;
it drains accepted facts on normal exit even after logout/retirement and retains
the same generation/binding until completion. Duplicate facts reuse the same
nonrepeatable claim. Pickup uses the atomic store operation; a failed observation
refresh after commit closes the session rather than inventing success.

`tools/play.py --account-login --rewards` selects equipment and creature
prerequisites. The backpack claim widget uses ordinary Bevy picking. It displays
saved availability, waits for the durable reply and retains availability on a
full bag. This source path is not yet native qualified. Story and production
reward policy remain unchanged.

Eight pure reward tests pass (`build/rewards-core-shared-id-v1.log`); focused
wire checks, six item-stream regressions and fatal-life/player-credit checks pass
(`build/rewards-wire-v1.log`, `build/rewards-item-stream-v1.log`,
`build/rewards-authority-tests-v1.log`). Workspace/all-target denied-warning
Clippy passes (`build/rewards-workspace-clippy-v2.log`). Actual verified OIDC,
encrypted ticket and PostgreSQL retirement qualification passes in
`build/rewards-leases-v1-20261030/verification.json`: one blocked offer plus a
queued duplicate survive logout/retirement, no availability before COMMIT,
character reuse waits for offer/pickup completion, closed generations reject new
facts, fresh leases recover availability/consumption and an unacknowledged pickup
replays once. Owned services stopped. This drives real leases directly and does
not establish encrypted gameplay delivery, native widget success or capacity.


## First live native evidence; final regression work continues

`build/rewards-native-v2-20261030/verification.json` passes two actual OIDC
accounts, two native encrypted clients and PostgreSQL: independent authoritative
kills, each own Bevy claim click, exactly one additional token each, distinct
server-issued claims, saved consumption at item revision five, fresh runtime
actors on reconnect, actual database/server restart, changed default spawn and
backend-loss fail-closed behavior. Screenshots show sword/wand attachments,
100 tokens in the bag, “Reward claimed” and no remaining claim. Linux
RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 kisak, 1280×720; Vulkan validation enabled
and no reported GPU validation errors. Both bag layouts stay inside the window.
The ordinary authority runner is shared with the CLI; this qualification binary
is an owned probe, with CLI launcher checks still pending. This proves a bounded
development slice, not Windows execution or MMO capacity.

The earlier v1 run completed both kills/widget claims but then honored the test
provider's sixty-second token expiry during rejoin; it is not a passing complete
run. V2 renews the second test login before collecting the next lease, without
changing provider/session lifetime or bypassing authorization. Final source now
uses the configured ability in the qualification automation so the ordinary mixed
launcher can cover periodic-damage credit as well. Requalification and the full
workspace suite remain pending before freezing the final runtime checkpoint.

Shared opaque claim-key conversion also passes all seventeen actual PostgreSQL
qualification commands/eleven contracts and clean restart:
`build/rewards-postgres-v2-20261030/verification.json`. All owned services stop;
plaintext password/ticket files are removed. Existing encrypted admission/item/
equipment regressions pass (`build/rewards-network-regressions-v1.log`), as do
five account-helper tests (`build/rewards-account-helper-tests-v1.log`).


The stronger retirement contract now passes in
`build/rewards-leases-v3-20261030/verification.json`: two distinct recipes (one
executing, another queued) plus a duplicate survive logout/retirement. Exactly
two offers commit; the remaining unclaimed recipe survives fresh leases. This
catches a dropped queued fact that a duplicate-only fixture could conceal.
The fixture's second recipe is test data, not a second live development reward.

The ordinary CLI/mixed launcher passes interruption, actual child failure and
pending-login cancellation:
`build/rewards-mixed-launcher-v2-20261030/verification.json`. Equipment, resources,
Scorch and feedback load together; Scorch periodic damage earns the live claim,
its actual Bevy widget saves the extra token, and the next launch restores the
consumed claim. The first mode records nine accepted casts, one target selection,
one earned reward, 27 status observations and no missed simulation deadlines.
All owned children/tickets/database ownership close. The initial mixed v1 attempt
hit the unchanged control-message limit because test automation selected on every
uncapped frame; the automation now selects only when necessary with a bounded
retry. The server's rate limits were preserved. Final two-account native, baseline
and full-workspace qualification still precede the final frozen runtime checkpoint.


## Final qualified runtime checkpoint — 2026-10-30

`build/rewards-qualified-v1-20261030` preserves the exact source, four qualified
executables, non-story docs, asset digests and copied/hash-bound evidence. The
workspace run passes **379 tests, zero failures, 21 explicit skips in 84 groups**,
with source identity unchanged before/after the check. Eleven skipped SQL
contracts separately execute successfully in seventeen actual PostgreSQL
qualification commands; ten private-asset skips remain skips. Final formatting
and workspace/all-target denied-warning Clippy pass. The standalone server's
normal dependency graph excludes renderer/audio/UI/wgpu and offline nav backends.
Five account-helper tests pass.

Primary final-source native proof:
`build/rewards-native-v4-20261030/verification.json`. Both real accounts perform
one actual reward widget click, eleven pointer presses, five durable item changes
and one rejected ordinary item operation each. Each private bag reaches revision
five with exactly 100 training tokens (99 seed plus one reward), conserving both
weapon items. Distinct claimed IDs and consumption survive rejoin and actual
server/database restart. Own/remote animated gear metrics, layout bounds and
Vulkan validation pass. The same corrected client runs both this probe and the
ordinary CLI launcher. Five target selections suffice across the native run;
control limits and short test-session validation were preserved.

The stronger accepted-work retirement proof is
`build/rewards-leases-v3-20261030/verification.json`. Actual Scorch/reward/equipment/
resources/feedback CLI cleanup passes all three modes in
`build/rewards-mixed-launcher-v2-20261030/verification.json`; the legacy schema-one
backpack/mixed launcher also passes all three modes in
`build/rewards-legacy-backpack-launcher-v1-20261030/verification.json`.

The same exact release binaries pass the legacy one-client empty saved-terrain
movement tour at **805.97 FPS uncapped**, validation off:
`build/rewards-legacy-baseline-v1-20261030/verification.json`. This is a bounded
regression baseline, not populated-world capacity. Hardware remains Linux
Ryzen 7800X3D / RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 kisak / Vulkan / 1280×720.
Native Windows and physical OS mouse/browser delivery remain untested. Native
screenshots were inspected. All owned services/children stopped and private
password/ticket files were removed. No commit or push was performed.

Earlier in-progress notes/attempts above describe their own source and scope; the
v4 native, v3 retirement and v2 mixed-launcher proofs are final. Production loot,
repeatable/group entitlement, corpses, item instances, content upgrades and MMO
capacity remain open. [P3-10 progression](EXPERIENCE_LEVELING_FOUNDATION.md) is a
separate proposed next slice and has no implementation in this checkpoint.
