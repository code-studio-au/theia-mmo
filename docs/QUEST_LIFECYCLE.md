# Explicit quest acceptance and turn-in — P3-11 / initial P3-16

Status, 2026-10-31: the initial opt-in live journal lifecycle and normal launcher
now pass the native integration checks below. Final unchanged-source workspace
passes 464 tests / 33 explicit skips in 104 groups; formatting, denied-warning
workspace/all-target Clippy and twenty focused Python checks pass. Source, binaries
and proof are preserved at `build/quest-lifecycle-qualified-v1-20261031`.
Turn-in has no additional item/XP grant. Automatic
training objectives remain available as a separate mode. P3-17/18 are unspecified.

## Actual prerequisites and first lifecycle

Use the existing checked single-kill-objective `quests::Profile` and its stable IDs,
name, required count (1–100) and creature definition. Keep explicit lifecycle state
under `quests::lifecycle`, separate from the existing auto-start fixture, with a
distinct domain for exact-byte persistence/wire binding.

The initial once-per-character flow is Available → Active → Ready → TurnedIn.
A player requests acceptance or turn-in using a displayed revision; authoritative
credited kills advance Active only. Ready derives from accepted count reaching
the goal. No prior kills are counted by the pure available state, and no kills
advance Ready/TurnedIn. Refuse early turn-in, reacceptance, retaking completed
quests, stale commands, foreign profile IDs and impossible restored state. The
initial lifecycle has no abandonment, repeatables or administrative edits.

Acceptance increments quest revision once; each counted kill once; turn-in once.
Thus active revision equals count + 1 and turned-in revision count + 2 (maximum
102). Restore validates this relationship. Proposals leave their input untouched.
The canonical action bytes distinguish acceptance/turn-in and expected revision.
These proposals do not authorize NPC range/session, deduplicate victim lives,
commit a reward or acknowledge any client success.

## Adoption requirements

- Runtime verifies the owned character and current guide/session/range before
  accepting bounded work. The server chooses the objective and any reward recipe;
  clients supply neither counts, phase, recipient, items nor XP amounts.
- Exact-byte lifecycle profile/world binding must differ from auto-start mode.
  Existing saved objective progress must not silently become accepted progress.
- Persist explicit quest state and victim-life dispositions atomically with enabled
  kill XP. Record relevant kills observed before acceptance even if XP is capped;
  retrying an old fact after acceptance must never earn retrospective progress.
  A no-op without a receipt is insufficient for this guarantee.
- Persist acceptance/turn-in intent receipts. Recover exact retries without another
  mutation; reject conflicting or stale revisions. Preserve both user intent and
  trusted kills in accepted order across logout/retirement.
- If turn-in grants an item/XP recipe, commit it with the turned-in state and receipt
  atomically before success. Full bags/refusals must retain Ready state for retry.
- Publish private committed lifecycle state and use standard Bevy NPC/journal UI.
  A fresh character must accept before progressing; two real accounts must remain
  isolated through rejoin and database/server restart, with no repeated turn-in.
- Appropriate real SQL/worker/encrypted/native checks precede adoption. Existing
  three-kill automatic training and launch modes remain preserved until replaced.

## Pure prerequisite evidence

`crates/core/src/quests/lifecycle.rs` reuses the existing checked objective parser
and arithmetic. `crates/core/tests/quest_lifecycle.rs` covers acceptance/three later
kills/one turn-in, pre-accept and post-completion no-ops, early turn-in/reacceptance,
stale painted revisions and exact-retry ownership boundaries, invalid restores,
foreign IDs/unrelated creatures, min/max goals and canonical command identity.

`build/quest-lifecycle-core-tests-v1.log` records 77 core passes, including the seven
new tests. This is pure contract evidence, not working multiplayer or durability.
The launch-time manifest checkpoint remains separately frozen at
`build/gameplay-launch-qualified-v1-20261031` and uses the unchanged old runtime.

## Database prerequisite checkpoint

`QuestLifecycleCatalog` binds the exact existing objective bytes with a separate
lifecycle domain. `ensure_quest_lifecycle` initializes Available only; load/action/
trusted XP-and-lifecycle methods authorize owner/world and checked restored state.
Migration 0009 adds separate `quest_lifecycles`, `quest_action_receipts` and
`quest_lifecycle_kills` tables; old objective tables remain intact. The first
lifecycle is one nonrepeatable quest per character, with at most two command
receipts. Victim-life dispositions are durable history and grow with relevant
trusted kills; no production retention/partition budget is qualified here.

Action retries match retained canonical command hashes before replay recovery and
return current state without repeating a transition. Turn-in currently marks the
journal only. It grants no item/XP recipe and must not be presented as a rewarded
quest until the atomic reward extension is implemented. Kill XP keeps its existing
server-owned curve. The transaction locks XP first, then lifecycle, records pre-
accept/capped no-ops and refuses cross-recipient facts; a previously positive XP
receipt without lifecycle history cannot retrospectively count either.

The existing critical FIFO remains sixteen queued jobs; no awaited SQL enters
`World::step`. Ordinary action refusal leaves the worker live; corrupt state and
backend loss fail closed. Executing SQL survives a lost answer; runtime ownership
and accepted-queue retirement still require integration before game adoption.

`build/quest-lifecycle-postgres-v1-20261031/verification.json` records 23 actual SQL
contract passes (three new lifecycle contracts) across 29 commands. Tests prove
XP/lifecycle rollback, byte/world/owner binding, old capped/pre-accept and older XP
receipt handling, at-most-two action receipts, immutable completion, isolated
characters, FIFO acceptance-before-kill, lost executing answers, bounded queue
saturation and terminated-backend non-acknowledgement. Existing migration/upgrade,
character, equipment, rewards, automatic quests, XP, vendor and worker regressions
also pass with migration 9.

The new clean process/database restart probe compares Available, Active and
TurnedIn characters, capped XP, exact prior kill/action receipts and private logical
identities after fresh processes and actual owned PostgreSQL restart. Owned cluster
stops and plaintext password files are removed. This is store evidence using
trusted synthetic facts, not live NPC entitlement or multiplayer quest adoption.

## Adopted live slice design

Keep explicit lifecycle an opt-in mode, mutually exclusive with automatic objective
mode. Use the same checked objective source and exact lifecycle domain on store and
wire; include the selected mode/profile in world compatibility identity. Require
actual owned account admission, progression, creature combat and the informational
guide. Do not silently migrate old automatic progress or change default profiles.

Route trusted kills and accepted NPC actions through **one per-character FIFO**
in the existing generation-bound progression owner. Separate fact/action channels
with select priority cannot preserve acceptance-before-kill semantics. The fixed
step only validates/copies bounded requests. Retirement closes/drains that one
queue under retained ownership before slot reuse. The sixteen-job SQL queue stays
unchanged. Initial turn-in remains journal-only; the UI must not promise an item
or XP reward that this transaction does not grant.

A client action carries actor, sequence, painted quest revision, painted dialogue
revision, current NPC incarnation and Accept/TurnIn, never counts or rewards. Check
current NPC, current open dialogue revision and 3D interaction range before queue
acceptance. Malformed/foreign frames reject; normal stale/not-ready/not-entitled
refusals acknowledge authoritative state without disconnecting. Sequence/revision
checks prevent delayed replies from resolving a newer request.

Publish private committed lifecycle state before the matching action reply on the
existing reliable encrypted channel. Client successors permit acceptance/turn-in
revision increments without a kill-count increment, while rejecting rollback,
foreign actor/profile and impossible phase/count combinations. Standard Bevy guide
buttons and the journal display authoritative Available/Active/Ready/TurnedIn state;
only one pending quest action is allowed. Reconnect starts with closed dialogue
and restores committed progress without another acceptance or turn-in.

Qualification must cover real guide actions, out-of-range/stale-session refusal,
old facts at the XP cap, nonblocking guide collision, retained accepted work at
logout, owner isolation and database/server restart. Preserve the earlier playable
training and manifest-launcher binaries until the new live slice passes.

## Implemented wire/world prerequisite

State is exactly 65 bytes (tag, actor incarnation, exact profile digest, revision,
count and stored status); Ready derives from accepted progress. Requests are
exactly 61 bytes (tag, actor incarnation, sequence, painted quest/dialogue revisions,
NPC incarnation and Accept/TurnIn). Replies are exactly 69 bytes (tag, actor,
profile digest, sequence, committed quest revision and bounded normal refusal).
Parsing establishes no NPC entitlement or durable success. Both explicit mode and
exact profile bytes contribute a different world identity from auto-start mode.

Successor checks allow coalesced acceptance/turn-in updates that increment revision
without incrementing count, while rejecting rollback, foreign profiles/actors and
impossible restored phases. Reply sequence checks reject unsent/regressed/different
same-sequence acknowledgements. Current sole-guide entitlement rechecks the actor's
actual 3D range before a pruning step and binds the open dialogue's NPC incarnation
and revision. Activation requires inactive matching creature/progression/guide
configuration and excludes the retained automatic objective mode in both directions.

`build/quest-lifecycle-wire-tests-v1.log`: six protocol passes.
`build/quest-lifecycle-authority-tests-v2.log`: 55 network-library passes, including
two new explicit-mode/session/range tests and existing vendor, nonblocking-guide,
combat, patrol and movement regressions. The first authority build referenced a
nonexistent convenience method; it failed compilation, was corrected to the actual
`enable_combat` API and is not used as passing evidence. No new full-workspace,
encrypted live lifecycle, native display or SQL run is claimed for this prerequisite.

## Implemented async owner prerequisite

The progression port now has one `Input` FIFO containing trusted kills or accepted
quest actions. A single generation-bound owner initializes committed state, checks
retained binding/actor/sequence, then awaits one SQL job at a time outside the tick.
Retirement closes/drains the queue before the gate can reuse its slot; NPC/session/
range are not rechecked at commit after a valid acceptance. Normal quest refusal
publishes a bounded sequence/revision result and leaves the owner live. Unexpected
profile/state/ownership/backend errors fail closed. The normal auto-start quest and
XP-only branches use the same retained owner and remain available.

`AccountService::GameProfiles` provides typed optional store/wire pairs without
extending the existing positional constructor ladder; old constructors delegate
with lifecycle disabled. The authenticated adapter and peer admission require
matching world/store/wire lifecycle profiles. At that checkpoint, normal executable
selection and private lifecycle transport/controller observations/actions were
still open; the later checkpoints below cover their adoption.

`build/quest-lifecycle-retirement-v1-20261031/verification.json` passes actual
verified OIDC identity, encrypted ticket consumption and owned SQL against the
production gate source. It proves no acceptance/turn-in observation before COMMIT,
FIFO acceptance-before-three-kills, retained executing/queued work after logout,
fresh authorization fenced during retirement, restored Ready/TurnedIn state and
at-most-two durable actions after old-command recovery. SQL retains six victim
receipts, exactly three counted kills and one capped positive XP award. This is a
direct lease contract with synthetic already-authorized inputs, not native gameplay.
The same binary passes the retained automatic-objective retirement proof at
`build/quest-lifecycle-legacy-retirement-v1-20261031`.

## Implemented encrypted bridge/controller

The existing reliable channel now accepts bounded QA01 actions only on a peer with
a matching explicit lifecycle owner. Actor, increasing sequence and a maximum gap
of 128 are checked; the existing control batch is bounded and lifecycle requests
are limited to eight per peer per second. Current guide/session/3D range is checked
before nonblocking queue submission. Normal stale context or queue pressure returns
a refusal; malformed/foreign/unsupported control frames retire that peer.

State and matching reply are published in order, with capacity checked for both
before sending. A later local refusal fences an earlier owner reply so delayed
accepted-work observations cannot roll back the acknowledged sequence. The client
retains the painted quest/dialogue/NPC intent, allows one pending request, bounds
its UI queue to four and fails pending/initial state deadlines after eight seconds.
A successful acknowledgement requires committed state matching acceptance or
turn-in; replies alone never predict quest progress.

`build/quest-lifecycle-controller-tests-v2.log` passes the six new controller cases
alongside all existing 55 library tests. `build/quest-lifecycle-runtime-tests-v1.log`
then passes all 74 selected tests including the new encrypted test. Two actual
Renet clients play a real authoritative moving-creature loop, with explicit instant
100-damage and capped-XP fixtures, synthetic journal commit adapters and no assets.
A's first pre-accept kill stays uncounted; three later kills and a turn-in complete
A while B remains private/zero. B refuses premature turn-in, accepts, completes and
turns in independently. Stale closed-guide context refuses without disconnect.
The exact seven authority kill facts, five forwarded actions, one server-side
context refusal and six action replies are checked. This is real encrypted runtime
behavior, not real SQL/admission identity or native Bevy input evidence. The test's
first build used wrong health/container fields, and its first executable attempt
omitted the required prediction setup; these failed attempts are retained and
superseded by passing `quest-lifecycle-encrypted-tests-v3.log` and the final suite.

## Executable and Bevy adoption

The server, client and `tools/play.py` now accept opt-in `--quest-lifecycle`.
It requires the existing progression, creature and guide profiles and authenticated
predicted movement. The launcher enables the required systems; direct executable
arguments must supply their profiles. The previous automatic `--quests` workflow
remains available. Explicit mode uses a separate identity and persistence row.

The standard Bevy journal displays Available, counted Active, Ready and completed
training. The existing guide panel offers acceptance or turn-in according to
committed state, with one pending action, retained painted intent and bounded
normal-refusal text. Completing the journal grants no additional item or XP;
existing kill XP and personal loot claims remain independent.

The native-qualified two-account driver clicks the actual
Bevy buttons, completes three authority kills, claims personal loot, barters,
equips, turns in and then checks reconnect/database restart without repeating
quest actions. It checks exact receipt hashes and that the old automatic-objective
table remains empty. The following combined proof complements the earlier
separate encrypted and SQL checkpoints.

## Combined native and normal-launcher evidence

| Window | Actual provider/SQL/native proof | Vulkan errors |
| --- | --- | --- |
| 1280×720 | `build/quest-lifecycle-native-1280-v1-20261031/verification.json` | 0 |
| 1920×1200 | `build/quest-lifecycle-native-1920-v2-20261031/verification.json` | 0 |
| 3440×1440 | `build/quest-lifecycle-native-3440-v2-20261031/verification.json` | 0 |

All use the same optimized client/probe on RX 7900 XTX, RADV NAVI31, Mesa 26.2.6,
Vulkan/Linux, Ryzen 7800X3D, 128 GB RAM and a 3440×1440 desktop. Two actual native
clients use verified local OIDC accounts, encrypted tickets and owned PostgreSQL.
Bevy keyboard/picking events click acceptance/turn-in and the existing loot/barter/
equipment controls. A completes while B remains Available with zero XP and no item mutations;
then B completes independently. Both restore TurnedIn revision 5 with three counted
kills, exact bags/hands and exactly two canonical action receipts (accept revision
0, turn-in revision 4) through rejoin and database/server restart. The old automatic
objective table stays empty. Revocation, spent-ticket refusal and backend-loss
closure pass; owned services and private ticket/password files are cleaned up.
Acceptance and completed-state screenshots were inspected at all three sizes.
This is synthetic native Bevy input; physical OS input/browser, Windows, 4K,
production balance and MMO capacity are not qualified.

`build/quest-lifecycle-launcher-v1-20261031/verification.json` separately passes all
three actual normal-launcher cleanup modes with the validated ten-profile bundle
and `--quest-lifecycle`. Process argv proves matching profile bytes, snapshots are
removed after children stop, and native environments exclude SQL/libpq/provider/
test-browser configuration. Its equipment fixture kills without accepting, so SQL
stays Available revision/count zero with no action receipts despite earning XP.
This proves normal mode selection and pre-accept behavior; the native runs above
prove the explicit buttons and complete lifecycle. Pending-login cleanup starts no
native child.

The first native run failed world-identity checks because the server qualification
fixture selected accelerated abilities for the old objective flag only. It now
selects them for either quest mode; production defaults are unchanged. The failed
run and diagnostic are retained. The first ultrawide run was interrupted before
final restart verification; its orphaned owned provider was stopped and temporary
credentials removed. Only its clean rerun is counted above. A workspace attempt
was also terminated during compilation; its incomplete log is not a passing gate.
The final unchanged-source workspace gate passes 464 tests / 33 skips in 104 groups;
formatting, denied-warning workspace/all-target Clippy and twenty focused Python
checks pass. Core/persistence source is byte identical to the separately qualified
23-contract SQL checkpoint. The earlier pure/store/wire/owner/runtime checkpoints
remain preserved with their narrower scope.

## Final playable checkpoint

`build/quest-lifecycle-qualified-v1-20261031` contains the exact source hash map,
optimized client, normal server, native probe, preserved offline validator, current
non-story documentation and selected public proof logs/reports/screenshots.
Private provider/database data and credential files are excluded. The wrapper
records 537 non-document source files and verifies executed binary hashes against
all native/normal-launcher reports. This is an initial accepted training journal
and persistent integration fixture; quest turn-in grants, abandonment/repeatables,
groups and an authored production zone remain open. Changes are local and no new
engine pins, default gameplay profiles or population/performance claims are added.
