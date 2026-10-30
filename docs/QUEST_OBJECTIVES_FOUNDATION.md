# Quest objectives — proposed P3-11 first slice

Status, 2026-10-30: the initial live three-kill objective slice is qualified.
396 workspace tests pass with 27 explicit skips; all seventeen SQL contracts and
twenty-three commands separately pass. Actual two-account OIDC/encrypted native
checks pass at 1920×1200 and 3440×1440 fullscreen, including distinct private
journals, XP-cap independence, reconnect, database/server restart and backend loss.
Exact source, binaries, documentation and evidence are frozen at
`build/quests-qualified-v1-20261030`. Broader acceptance/turn-in lifecycle remains
open. Names/objectives are development fixtures; P3-17/18 remain unspecified.

## Small initial scope and prerequisites

Start with one opt-in development objective: defeat three training-creature lives.
Initialize it only when its checked objective profile is first enabled for a
character. This explicit automatic development fixture proves objective credit;
it does not establish production quest acceptance, dialogue or a quest giver.
P3-12 can add those interactions separately. Completion grants no extra items,
XP, money, stats or unlocks. Existing per-kill XP and personal item claims retain
their own rules and revisions.

Use P3-10's live source/current victim-life facts, durable character binding,
bounded async owner/critical worker and retirement fence. Reuse the existing
standard Bevy presentation and encrypted private observation boundary. World
collision/navigation, terrain, rendering infrastructure and distributed services
need no expansion. NPCs and players remain physically nonblocking.

## Contracts to check before implementation

- One bounded immutable profile with stable quest/objective/creature IDs, a positive
  integer goal and exact content digest. Start with a single kill objective; do not
  invent a general quest scripting engine. Bound definition bytes and the maximum
  goal. Incompatible content fails closed rather than reseeding saved progress.
- Derive active/complete from checked committed count and goal. A new eligible
  victim life adds one, clamps at goal and advances one revision only when progress
  changes. Pure proposals never authorize credit or deduplicate events themselves.
- Persist each positive objective update and its victim-life receipt atomically.
  Exact old receipts remain harmless after later kills. At completion, new kills
  create neither revisions nor an ever-growing zero-progress receipt log. A finite
  goal bounds positive receipt growth.
- Reuse the same bounded earned-fact queue rather than introducing another event
  service. Review an atomic XP/objective transaction for one eligible kill so a
  crash cannot commit only one of its enabled progression effects. Retain separate
  XP/objective state and revisions; a capped XP total must not stop an unfinished
  objective. Queue/SQL acceptance is never objective-success acknowledgement.
- Initialize the opt-in fixture before processing its accepted facts. Credit comes
  from live authority, never client event IDs, recipient UUIDs or counters. Fresh
  respawns are distinct; stale/static targets and foreign sources reject. State
  restores for the same character despite new runtime actor/input identities.
- Preserve accepted executing/distinct queued work through logout/retirement, plus
  the final pose and existing item-owner fence. No SQL or awaited transport in the
  fixed step. Fail ambiguous persistence closed.
- Send a bounded independent private committed objective observation. The Bevy
  journal/counter shows loading until a saved observation exists, then exact count,
  goal and completion. It must not acknowledge an item intent, predict credit or
  reveal another character's objectives. Keep the 1920x1200 default, 1280x720
  regression and 3440x1440 ultrawide anchors usable.

Quest acceptance/abandonment/retakes, branching, dialogue, vendors, collection and
exploration objectives, group credit, turn-in rewards and production story remain
separate scope. Introducing any of them requires explicit lifecycle/receipt rules;
this automatically initialized fixture must not silently become that policy.

## Concrete acceptance

1. Core: invalid/duplicate/oversized definitions, checked count/revision bounds,
   source immutability, exact completion and complete no-op.
2. Real SQL: owner/world/profile binding; old and competing event replay; forced
   rollback across enabled XP/objective effects; XP cap with unfinished objective;
   bounded receipts, dropped replies, queue saturation, backend loss and real
   process/database restart.
3. Authority/encrypted runtime: direct and periodic kill credit, fresh life,
   stale/foreign rejection, bounded fan-out, private independent observations and
   feature-disabled XP/item regressions.
4. Actual OIDC/leases: executing and distinct queued facts survive logout; pose
   acknowledgement cannot release a pending objective; fresh lease restores state.
5. Native Linux: actual kills visibly advance each own objective and complete it;
   later kills preserve completed state; reconnect/restart retain exact counters.
   Inspect actual Bevy widgets at the development and ultrawide sizes, and document
   GPU validation policy/device, physical input limitations and Windows evidence.
6. Formatting, denied-warning workspace Clippy and appropriate tests pass; freeze
   exact source/binaries/proofs before the next slice. Keep changes local.


## Implemented pure core prerequisite

`theia_core::quests` reads one immutable schema-1 kill objective, at most 4 KiB,
with distinct stable profile/objective/creature IDs, a checked display name and a
goal of one to 100 kills. The development fixture requires three training-creature
lives. Saved state validates logical profile, count at/below goal and revision
at/below signed database maximum. Storage/wire must separately bind exact bytes.

One matching trusted proposal adds one count/revision, derives completion at the
exact goal and leaves source state unchanged. Nonmatching kills and completed
objectives produce no mutation, even at exhausted revision; a positive mutation
at exhausted revision rejects. There are no client counter/grant commands, reward
side effects, acceptance/retake lifecycle or in-memory receipt history in core.

`build/quests-core-tests-v2.log` has thirty passing tests: five new objective tests
and 25 XP/equipment/backpack/reward regressions. `build/quests-core-clippy-v1.log`
passes denied-warning core/all-target Clippy. Tests cover ambiguous/unsupported/
oversized definitions, distinct IDs, bounded goals/counts/revisions, exact
completion, unrelated/completed no-ops and foreign logical profiles. Pure repeated
proposals still require durable victim-life deduplication; no native quest or new
full-workspace qualification is claimed yet.


## Qualified database/worker prerequisite

Migration 0008 adds one objective state per character and positive victim-life
receipts. Owner/world/exact profile binding is checked; initialization creates
missing zero progress only. Exact old receipts return current saved state without
advancing or repeating completion. Receipt state at the latest revision must agree
with saved count; corrupt receipt/profile/state fails closed. A completed objective
and unrelated creature create no positive receipt or revision. One-to-100 kill
goals bound positive objective receipt growth to at most 100 per character.

`award_progression_and_quest` locks XP before the objective and commits both enabled
updates and both positive receipts together. The existing XP transaction body is
shared internally; the public XP-only method still owns its own COMMIT. A forced
failure in either XP or objective update rolls everything back. Capped XP does not
prevent objective progress, and completed objectives do not prevent later XP.
This is the initial one-recipient/current-life policy, not group quest entitlement.

The adapter uses the existing sixteen-job critical queue. Before execution, closed
answers skip work; after SQL starts, dropping an answer does not cancel the bundled
transaction. Expected missing/conflict/command rejection is nonfatal; corrupted
state/content/backend failure fails the worker closed. There is no new worker,
event service, queue or awaited database call in the simulation. Runtime accepted-
fact retention is qualified separately by the live integration evidence below.

Final prerequisite evidence:

- `build/quests-core-tests-v2.log`: thirty passing focused tests, comprising five
  objective tests and 25 XP/equipment/backpack/reward regressions.
- `build/quests-postgres-v2-20261030/verification.json`: seventeen actual SQL
  contracts and twenty-three commands pass. Three objective contracts verify old
  receipts after later kills, exact byte/profile/world/recipient binding, positive
  receipt bounds, objective completion after XP cap, forced rollback from either
  effect, conflicting/latest receipt rejection, competing producers, cancelled
  answers, shared queue overflow and actual blocked-backend termination. Separate
  writer/read processes retain completed/active counters, XP and old receipts
  through a clean database restart. Prior fourteen contracts pass too. The owned
  cluster is stopped and plaintext password files are removed.
- `build/quests-store-fmt-v1.log`, `build/quests-workspace-check-v1.log` and
  `build/quests-workspace-clippy-v1.log`: formatting, workspace/all-target check
  and denied-warning Clippy pass. No rendering changed in this prerequisite.

Source, non-story documentation, the executed store probe and proof hashes are
frozen in `build/quests-store-qualified-v1-20261030`. All changes remain local;
nothing was committed/pushed, and concurrent story work is untouched.


## Qualified live integration

Enabling the objective requires the inactive world's matching progression/patrol
profile and binds exact objective bytes into its transport identity. Reuse the
same sixteen-fact character queue and live current-victim source checks as XP.
The async owner initializes both saved states and commits enabled effects together;
its existing retirement completion flag now includes objective work. No separate
queue, event service, simulation SQL or client credit/count command was added.

`QO01` is an independent 64-byte private reliable observation: own actor identity,
profile digest, durable revision and integer count. It has its own coalescing guard
so XP at cap cannot hide further objective progress. Client validation rejects
foreign/profile/regressed/aliased observations and bounds count to the known goal;
missing initial state times out after eight seconds. The standard Bevy journal
shows loading until committed state arrives, then count/goal and exact completion.
No item intent is acknowledged and no reward/extra XP is granted by this panel.

`python3 tools/play.py --account-login --quests` opts into the three-kill fixture,
implying progression and creature combat. Existing provider/database configuration
is required. It can combine with rewards, equipment, resources and statuses.

Current integration evidence:

- `build/quests-authority-tests-v1.log`: 43 network unit tests, two XP wire tests
  and two objective wire tests pass. Profile binding and current-life facts are
  checked; unknown/truncated/foreign/regressed objective observations reject.
- `build/quests-leases-v1-20261030/verification.json`: actual OIDC, encrypted ticket
  and PostgreSQL pass. Two accepted distinct lives and an old duplicate drain
  through logout; no uncommitted XP or count is advertised, pose success cannot
  release pending work, the retired port closes and a fresh lease restores count
  two without re-awarding. This directly drives leases; native gameplay is separate.
- `build/quests-runtime-clippy-v1.log`: workspace/all-target denied-warning Clippy
  passes. Final formatting and workspace/all-target denied-warning Clippy also
  pass in `build/quests-final-fmt-v1.log` and `build/quests-final-clippy-v1.log`.

The native fixture keeps the real three-life objective, uses the explicit 0/25/50
XP curve, and sets its first ability to instant 100 damage for short test duration.
Default development abilities/quest goal remain unchanged. One client pauses at
1/3 while the other completes 3/3, allowing each journal to be checked against its
own distinct SQL state before both finish. The third credited kill must still
advance the objective after XP reaches its test cap. These fixture changes are
not production balance or accelerated server simulation.

## Native integration evidence

`build/quests-native-1920-v2-20261030/verification.json` and
`build/quests-native-ultrawide-v1-20261030/verification.json` pass on Linux,
RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 / Vulkan. Two actual OIDC accounts and
native encrypted clients earn independent credit through ordinary cast intents.
One is held at 1/3 while the other completes 3/3; SQL and each actual journal agree.
Both then complete, reconnect and retain exact counters through actual database
and server restart, including a changed spawn about 15 km away. Backend failure
fails closed. Objective counts progress 0/1/2/3 while explicit fixture XP is
0/25/50/50, proving independent progress after the XP cap. Item/equipment/reward
widget checks remain enabled. Actual 1/3 and 3/3 screenshots were inspected at both
sizes; XP, journal, backpack and combat panels fit without overlap.

Windowed validation has no GPU diagnostics. Fullscreen records one startup
`VUID-VkSwapchainCreateInfoKHR-pNext-07781` per client, subject to the narrow
[existing upstream wgpu resize diagnostic policy](EXPERIENCE_LEVELING_FOUNDATION.md).
All other GPU errors reject. The display is physically 3440×1440; configurable
3840×2160, native Windows and non-default DPI remain unqualified. Scripted input
uses actual game requests; physical mouse/browser interaction is not established.

`build/quests-mixed-launcher-v1-20261030/verification.json` passes the ordinary
1280×720 CLI with Scorch, resources, statuses, creatures, spatial feedback,
equipment, rewards, XP and objectives enabled. Interrupt, child failure and pending
login cancellation retain owned-process cleanup, credential removal and accepted
final work. These checks use their explicit local OIDC provider and PostgreSQL;
they establish no production identity-provider or MMO population capacity claim.

## Final regression and frozen checkpoint

`build/quests-workspace-tests-v2.log` and its summary record 396 passes, zero
failures and 27 explicit skips across 92 test groups, with all 491 captured source
files unchanged. Seventeen skipped SQL contracts were separately executed in the
real database qualification; ten private-asset cases remain unexecuted. The 65
core/persistence/SQL-qualifier files are byte-identical to their qualified store
prerequisite. `build/quests-python-tests-v2.log` records nine passing account/GPU
helper tests. The normal server dependency graph retains no renderer/UI/audio
or wgpu dependency (`build/quests-server-dependencies-v1.txt`).

`build/quests-qualified-v1-20261030` preserves exact source, non-story Markdown,
four executed binaries and proof digests. The first workspace run also passed but
its starting source map predates the screenshot-capture retry; it is not the final
unchanged-source proof. The first 1920 native attempt failed on a skipped screenshot
capture, was corrected, and is excluded from the qualified checkpoint. The final
fixture retries capture while that committed count remains visible.

No new frame-rate benchmark was run for this objective binary. The prior 808.60 FPS
empty-world result belongs to the frozen XP/display binary and retains its stated
limits. All changes are local; no commit/push or story editing was performed.
