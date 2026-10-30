# Initial live vendor barter qualified — 2026-10-30

The final optimized client and headless native probe pass actual two-account
OIDC/encrypted/native trades, own SQL/UI comparison, close, rejoin, clean database/
server restart, distant restored pose and backend failure at all three sizes:

- `build/vendor-native-small-v2-20261030`: 1280×720, no GPU diagnostics.
- `build/vendor-native-1920-v1-20261030`: 1920×1200, no GPU diagnostics.
- `build/vendor-native-ultrawide-v1-20261030`: 3440×1440 fullscreen, one specifically
  allowed upstream wgpu resize diagnostic per startup; other GPU errors fail.

Actual widgets were inspected on Linux, RX 7900 XTX/RADV NAVI31, Mesa 26.2.6,
Vulkan. Input is synthetic Bevy keyboard/picking, not physical OS input. Physical
4K, other DPI scales and native Windows remain unqualified. No new performance
or population-capacity claim is made. The preliminary small v1 run passed but
showed an unsupported trade-arrow glyph; the final client uses plain text and all
three final runs use identical executables. That preliminary run is not final
source/binary evidence.

Account A spends one token for three stones; account B spends two tokens for six
stones. Both recover the same bag revision/content without new clicks or charges
on rejoin and after database/server restart. Each new conversation starts closed
at revision zero. No reward/XP/objective records are manufactured. A successful
purchase changes the existing bag row and latest receipt only. Initial development
bags remain the explicit existing fixture; this does not establish production
onboarding or a wallet economy.

`build/vendor-final-postgres-v1-20261030` passes all twenty SQL contracts/twenty-six
commands. `build/items-retirement-v2-20261030`,
`build/rewards-retirement-regression-v1-20261030` and
`build/vendor-retirement-v2-20261030` pass actual OIDC/encrypted-ticket/SQL direct
lease checks: accepted work drains, exact pose fencing, fresh lease/latest receipt,
bounded queue, ordinary refusals and backend loss. The normal CLI's combined
vendor/quest/XP/reward/equipment/creature/status/feedback configuration passes three
cleanup modes in `build/vendor-mixed-launcher-v1-20261030`; it does not automate a
purchase in that mixed launcher check.

The native checks hold the first 3.2 seconds for restored-pose observation, then
approach the guide, inject actual Bevy E input, click the real Buy widget, compare
committed bags and close with Esc. Existing gameplay controls stay enabled.
`python3 tools/play.py --account-login --vendors` opts into matching development
items/equipment, guide and barter profiles. Opening a guide conversation opens the
backpack; I toggles it. Scrolling an open backpack does not zoom the camera.

The final unchanged-source workspace passes **433 tests / 30 explicit skips**
in 99 groups (`build/vendor-workspace-tests-v2-summary.json`). The thirty skips
include twenty SQL contracts executed separately above and ten private-asset
fixtures. Final formatting, denied-warning workspace/all-target Clippy and nine
Python helper tests pass. The purchase-activation test retains the displayed bag,
conversation, NPC and offer when a newer observation arrives. A preliminary
workspace v1 also passed, but the launcher qualifier changed during that run;
only the unchanged-source v2 is the final workspace proof.

Exact source, five executed binaries, non-story documentation and proof are frozen
at `build/vendor-qualified-v1-20261030`. The final server was built standalone;
its normal dependency graph excludes Bevy renderer/UI/audio and wgpu. Prior playable
checkpoints remain preserved. Changes remain local/uncommitted and story edits
are untouched. Production currency/selling/stock/buyback, general quest acceptance/
turn-in, occlusion, real player death and general NPC lifecycle remain open.
No new performance or population capacity is claimed. The earlier prerequisite
and integration records below are historical.

# Vendor integration in progress — 2026-10-30

The shared item owner now closes new acceptance and drains accepted executing and
queued work before releasing a retiring character. Actual OIDC/encrypted-ticket/
PostgreSQL lease checks pass for ordinary items and two barter purchases, including
committed-pose fencing, fresh lease/latest receipt and backend failure. Exact runs:
`build/items-retirement-v1-20261030` and `build/vendor-retirement-v1-20261030`.
These directly drive leases; they do not establish a native purchase.

The live world now binds the exact item/vendor bytes, matching guide NPC and
current private open conversation/revision, incarnation and three-dimensional
range before enqueue. VP01 has a fixed 61-byte intent and joins the existing item
sequence, pending limit, shared bag revision and committed IS01/IS02 observation.
A normal entitlement refusal does not disconnect or reach SQL. Accepted work does
not repeat NPC/session/range authorization halfway through completion. Closing or
retiring the guide preserves the installed world identity. Generic item proposals
and legacy item frames cannot turn a vendor intent into a grant.

Three new wire tests and two new authority/stream tests pass with existing network
regressions. An additional actual encrypted Renet test passes: closed-session
refusal, accepted purchase remaining pending after conversation closure, then an
owner response and an ordinary backpack move on the same ordered stream. Its
item owner is synthetic; real SQL/logout durability is the separate lease proof.
Logs: `build/vendor-runtime-tests-v2.log`, `build/vendor-encrypted-tests-v1.log`.

The opt-in `--vendors` launcher, checked client/server `--vendor-catalog`, Bevy
backpack offer widgets and native qualification driver are implemented and under
validation. All item UI actions share one controller/pending state; purchase clicks
retain the displayed bag revision, conversation revision, NPC and offer. Opening
a vendor conversation opens the backpack. Offers are scrollable with the existing
bag; success comes from a committed item response. No new native, full-workspace
or final formatting/Clippy qualification is claimed until those runs finish.
Changes remain local; earlier frozen playable checkpoints are preserved.

# Vendor barter — proposed P3-12 first purchasing slice

Status, 2026-10-30: pure exchange and database/critical-worker prerequisites are
qualified. Eight barter tests/36 core regressions and twenty actual SQL contracts/
twenty-six commands pass. Formatting, workspace/all-target check and denied-warning
Clippy pass. Live NPC entitlement, accepted item-owner integration, encrypted vendor
commands and shop widgets remain open. Exact database prerequisite source/proof:
`build/vendor-store-qualified-v1-20261030`. The preceding playable NPC checkpoint
remains `build/dialogue-qualified-v1-20261030`. Changes are local; no native purchase
or new full-workspace claim is made. P3-17/18 remain unspecified.

## Initial scope and actual prerequisites

Use the existing nonblocking guide, authoritative interaction range, private Bevy
presentation and durable backpack owner. Start with one fixed barter offer: one
Training Token for three Training Stones. These are existing development items;
no wallet balance, free purchase, sell-back, stock, bulk quantity, buyback, price
negotiation or quest reward is introduced. Normal gameplay must explicitly enable
its checked vendor profile. Pure content/arithmetic comes first, then real SQL
and accepted-work retirement, then encrypted/native purchasing.

Each offer names one positive cost item/count and one distinct output item/count.
Costs come from backpack slots, never the equipped main hand. Granting a checked
equippable output may place it in the backpack but must not auto-equip or replace
owned gear. One purchase uses one existing bag revision; cost removal and output
addition form one tentative state and one durable transaction.

## Existing transaction rules reviewed

`crates/persistence/src/store/inventory.rs` locks the character's backpack row and
checks owner, world, exact item profile and expected revision. Its bounded latest
receipt returns an exact last-command retry; different input at that prior revision
conflicts. Older/future revisions reject without mutation and require reloading
current state. This is not an unbounded historical intent ledger. Preserve one
outstanding item intent per character and the current shared revision across
backpack, equipment, reward claims and future purchases.

`crates/core/src/rewards.rs` already grants by merging matching stacks in slot
order and then filling empty slots. Reuse that reviewed bounded policy after
removing the positive cost. Do not route repeatable purchases through nonrepeatable
kill-reward claims, and do not report a pure proposal or accepted queue as success.

## Contracts and concrete acceptance

1. Checked content: at most 16 KiB and eight offers, bounded stable IDs/names,
   exactly one cost/output leg, positive quantities at most 1,000 and within the
   sixteen-slot item capacity. Reject unknown items, aliasing definitions, equal
   cost/output IDs, equippable cost items, unsupported fields and impossible
   quantities. The pure profile binds the checked item definitions; runtime/store
   must additionally bind exact item/vendor bytes and the current guide's NPC ID.
2. Pure exchange: validate the source and expected revision, remove cost from
   matching slots in increasing order, merge output then fill empty slots, preserve
   equipment and advance exactly one revision. Any insufficient cost, no capacity,
   stale/foreign state or exhausted revision returns no partial result. A cost
   that empties a slot can make a previously full bag able to receive the output.
   Work is bounded by slot/offer counts, not by the number of item units.
3. Durable boundary: authorize owner/world/catalogues, lock the shared row, check
   the latest receipt and commit cost/output/receipt together before acknowledgement.
   Canonical purchase identity includes expected revision and exact vendor/offer
   binding; the client never supplies a recipe or price. Test forced rollback,
   last retry, conflicting/older retries, concurrent purchases/transfers, full-bag
   recovery, cancelled answers, bounded queue saturation, backend failure and real
   database/process restart. Preserve all existing item/quest/XP contracts.
4. Runtime: the server binds the admitted actor and current guide/profile/range
   before accepting a purchase into the existing critical owner queue. Retain
   accepted executing/queued purchases through close, walkaway and logout until
   durable completion; do not re-authorize an already accepted cost halfway through
   its transaction. No SQL or awaited I/O in the fixed step. A dialogue-node reply
   is not a purchase-success acknowledgement.
5. Client: standard Bevy offer/cost/output widgets use observed bag state and
   immutable profiles, retain displayed offer identity/revision, show pending and
   publish success only from the committed item response. Actual two-account OIDC/
   encrypted/native checks must compare own widgets to SQL before/after purchases,
   reconnect/restart and dropped replies. Inspect 1280×720, 1920×1200 and 3440×1440;
   record exact GPU policy/device and physical OS input/Windows limitations.
6. Formatting, denied-warning Clippy and appropriate tests pass. Freeze exact
   source, executed binaries and proof before qualifying the next slice. Production
   currency/economy and general NPC/quest lifecycle remain separate work.

## Qualified pure exchange prerequisite

`theia_core::vendors` checks a schema-1 catalogue with one positive cost and one
distinct output per offer. Vendor/NPC/offer definition IDs cannot alias each other,
the item catalogue ID or known item definitions. Eight offers and 16 KiB bound
content; names are at most 64 UTF-8 bytes without control/edge whitespace. Each
leg is one to 1,000 units and cannot exceed that item's sixteen-slot capacity.
Equippable cost definitions reject; equippable outputs are permitted into the
backpack without changing the equipped main hand. Unsupported/free/currency
fields, duplicate keys, unknown items and ambiguous offers reject.

The profile retains the checked item definitions and compares them on proposal,
so changing stack/equipment metadata under the same logical catalogue name cannot
silently rebind a price. Raw byte hashes, actual NPC identity, range, character
ownership and durable receipts still belong to outer layers. A pure repeated
proposal is not receipt deduplication or permission to grant an output.

A proposal clones the sixteen slots, removes cost in slot order, merges output
in slot order, then fills empty slots and advances exactly one signed-database-
bounded revision. Work uses at most three scans of sixteen slots, independently
of unit count. Insufficient cost or output capacity leaves the original cost,
items, revision and equipment unchanged. A cost that empties a slot can allow a
previously full bag to receive the purchase. Legacy schema-1 backpacks retain
that schema and do not gain an equipment field. The reviewed grant policy follows
`core/rewards.rs`; qualified reward code was preserved.

`build/vendor-core-tests-v2.log` records 44 passes/zero failures: eight barter
checks and 36 dialogue/objective/XP/equipment/backpack/reward regressions. It covers
strict definitions and byte/count/capacity limits, deterministic multi-slot costs
and output merges, full-bag rollback and freed-slot success, insufficient funds,
stale/unknown/exhausted/foreign input, same-name changed definitions, maximum unit
counts, preserved equipment and legacy schema. `build/vendor-core-fmt-v1.log` and
`build/vendor-core-workspace-clippy-v1.log` record successful formatting and
denied-warning workspace/all-target Clippy. No rendering changed and no new
full-workspace, SQL purchase or native vendor claim is made. Exact prerequisite
source, non-story docs, the executed barter test binary and proof hashes are
frozen in `build/vendor-core-qualified-v1-20261030`. Previous playable sources
and executables remain preserved. No commit/push or story edits were performed.

## Qualified database and critical-worker prerequisite

`VendorCatalog` binds the exact vendor bytes and exact item-byte digest, while
retaining the checked pure catalogue. Canonical latest purchase identity includes
that profile digest, expected bag revision and length-delimited stable offer ID.
It excludes ephemeral actor/NPC incarnations so an exact latest receipt can recover
after process restart. Current NPC/range entitlement still belongs to the future
live adapter; calling this trusted SQL API does not establish that entitlement.

`Store::purchase_vendor` checks owner/world/item bytes, locks the existing backpack
row and commits cost removal, output addition, one shared revision and the latest
receipt in one transaction. There is no new schema, wallet, historical purchase
ledger or second receipt/revision stream. An exact last-command retry returns the
committed bag without another cost/output; different input/profile at that prior
revision conflicts. Older/superseded or future revisions reject and require reload.
A new enabled price may be used by a new intent at the current revision; an old
request cannot silently replay under changed vendor/item bytes. Gear, transfers
and reward claims continue sharing the same row/revision.

The publisher uses the existing sixteen-job critical queue and existing worker.
Closed answers skip before execution; once SQL starts, dropping the answer does
not cancel the transaction. Insufficient cost, no capacity, unknown offer and
stale/exhausted commands are expected refusals. Corrupted trusted state/content
and backend failure fail the worker closed. Purchase/replay/refusal counters and
shared item counters record outcomes. Accepted network work must still be held
through retirement by the live owner adapter; no new accepted-work logout proof
is claimed at this prerequisite. No awaited I/O was added to the simulation.

`build/vendor-postgres-v2-20261030/verification.json` records twenty actual SQL
contracts and twenty-six commands. Three new contracts cover exact byte/owner/world
binding, changed/superseded receipts, a full bag whose cost frees a slot, no-capacity
rollback, an AFTER UPDATE trigger that forces transaction rollback, competing
purchase/transfer producers, continued worker service after ordinary refusals,
cancelled executing answers, exact latest recovery, sixteen-job saturation and
actual blocked-backend termination without a purchase-success response. Prior
seventeen contracts pass too. Separate writer/read processes retain revision two,
four tokens, twenty-four stones, equipped sword and the latest receipt through
an actual clean database restart. The owned cluster is stopped and password files
removed. This is real SQL/worker proof, without NPC simulation, OIDC or rendering.

`build/vendor-store-check-v1.log`, `build/vendor-store-fmt-v1.log` and
`build/vendor-store-clippy-v2.log` record successful workspace/all-target check,
formatting and denied-warning Clippy. The pure-core source is unchanged from
its qualified prerequisite. Exact source, non-story docs, the executed restart
probe and proof digests are frozen at `build/vendor-store-qualified-v1-20261030`.
No commit/push or concurrent story editing was performed.
