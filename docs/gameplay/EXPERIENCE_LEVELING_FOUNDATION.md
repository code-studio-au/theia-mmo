# Experience and leveling — proposed P3-10 first slice

Status, 2026-10-30: initial live earned-XP slice qualified. Trusted current-life
fatal hits now reach durable atomic receipts, the bounded character owner and
committed private observations/Bevy level UI. Exact source/binaries/evidence:
`build/progression-qualified-v1-20261030`. 388 workspace tests pass with 24 explicit
skips; all fourteen SQL contracts/twenty commands separately pass. Native Linux
1920x1200 and 3440x1440 two-account checks pass. Broader leveling remains open.
Prior core/store and reward checkpoints remain preserved.
Development curves and names are fixtures, not approved production balance/story.

## Scope and actual prerequisites

Build on checked character identity, real OIDC/Renet admission, the existing
asynchronous critical worker, authoritative creature life/death facts and the
retirement fence. P3-09's native reward path supplies those boundaries; its item
claim is not an XP command. P2-12's existing standard Bevy UI is sufficient for
one private progression bar. No terrain, renderer, pathfinding or distributed
service work is needed for this slice. P3-17/18 remain unspecified.

Start with one small checked level curve: cumulative XP thresholds 0, 100 and
250 for levels one, two and three, and 25 XP per authoritative training-creature
kill. Treat these as proposed development values. The server awards kill XP
independently of opening the backpack or claiming an item. Repeated creature
lives can earn XP until the development cap; a previously consumed item reward
must not prevent XP. Keep last-hit credit explicit for the initial policy.
Group credit, rested XP, death penalties, quests, level/stat scaling, skill unlocks
and production balance remain separate work.

## Contracts to settle before implementing

Use bounded immutable definitions, stable content IDs and exact byte-bound
profiles. Levels begin at one with zero XP; thresholds are strictly increasing
and all integer arithmetic is checked. Derive level and progress from the saved
XP total and curve rather than accepting an independently mutable client level.
An award may cross several thresholds; cap it explicitly without overflow. A
capped character receives no additional XP/revision or ever-growing zero-award
receipt backlog. Initial proposed bounds: 32 levels, 10,000 total XP, positive grants up to
1,000 XP and a 16 KiB profile. These are development-slice limits, not a final
MMO level/experience range.

An authoritative fatal-hit fact must retain the victim's checked world incarnation
and entity identity, creature definition and live source-to-character binding.
A fresh respawn is a fresh eligible life. The client uploads neither kill IDs,
recipient IDs, XP quantities nor levels. Reusing a combat sequence alone is
insufficient: periodic damage and different creature lives must remain distinct.
Do not reuse the nonrepeatable item-claim ID as a repeatable kill receipt.

Persist the progression update and its earned-event receipt atomically on the
existing bounded async critical worker before advertising XP or level success.
Authorize the character owner/world/profile, lock consistently, and reject
foreign/conflicting event reuse. An exact old receipt must never award again,
even after later legitimate kills. A clean rejoin/restart restores the same XP;
new runtime actor identity does not reset durable progression. Preserve accepted
facts through logout/retirement and final character reuse. No awaited database or
transport operation belongs in World::step.

Bound positive-award receipts by the finite XP budget: with a minimum one-XP
positive award and a 10,000-XP cap, at most 10,000 awarded-event receipts can be
created per character. Exact old receipts return the current committed progression
without adding XP, even after later awards. At-cap new facts produce no mutation
or zero-award receipt. Check revision exhaustion before any positive mutation.
Consider a realm/world/victim-life uniqueness constraint for the initial one-winner
last-hit policy; a conflicting recipient must not turn the same kill into another
award. Group entitlement would require a separately specified receipt scope. Do not introduce
an unbounded in-memory kill history, silently evict receipts that could replay,
or invent a general event service before the slice needs one. Broader receipt
retention and production content upgrades need separately reviewed policies.

XP observations have their own checked generation/revision and profile identity;
they do not advance the backpack revision or acknowledge a pending item intent.
The standard Bevy display shows only committed own-character level/XP. It may
aggregate presentation of saved progress but cannot predict a successful award.
A level change has no unrequested health/mana, gear, damage or movement effect.
Preserve P3-09's offer/claim protocol and legacy feature-disabled paths.

## Acceptance before qualification

- Core: malformed/nonmonotonic/oversized definitions, zero/overflow grants,
  boundary and multiple-level crossings, cap and unchanged source state.
- Real PostgreSQL: ownership/world/profile, duplicate and old-event replay,
  competing awards, forced rollback, dropped answers, full critical queue,
  actual blocked-backend failure and clean database/process restart.
- Authority/transport: direct and periodic fatal hits, live source/current life,
  fresh respawn, stale/foreign identities and bounded accepted facts; encrypted
  private observations remain separate from item replies and other characters.
- Real admission/retirement: executing and distinct queued XP facts survive
  disconnect/logout; saved pose alone cannot release a pending progression job.
- Native: actual authoritative kills advance each own character's Bevy bar/level;
  rejoin and database/server restart retain exact values, with no duplicate award
  from replay and no success on backend loss. State clearly when an accelerated
  test profile differs from the default development curve.
- Regressions: movement/readiness, backpack/equipment/rewards, combat/statuses,
  ordinary launchers/cleanup, required formatting/denied-warning Clippy and
  appropriate workspace tests; native GPU/device/limitations and exact source/
  executable evidence. Empty-world FPS is not MMO population capacity.


## Implemented pure core prerequisite

`theia_core::progression` checks 16 KiB definitions, two to 32 strictly increasing
cumulative thresholds starting at zero, a 10,000-XP cap and one to sixteen unique
creature definitions granting one to 1,000 XP. The development profile uses
0/100/250 thresholds and 25 XP for the training creature. State checks logical
profile identity, nonnegative bounded totals and a signed-database-compatible
revision. Storage/wire must separately bind the exact content digest.

Progress derives level and exact within-level amounts without floating-point
rounding. Trusted kill proposals preserve source state, advance one revision for
a positive grant, cross multiple levels if needed and clamp the final award to
cap. At cap, known new kills return the unchanged state with zero grant; invalid
creatures still reject. A positive award at exhausted revision rejects. Core
victim-life IDs bind both nonzero world incarnation and entity identity in sixteen
bytes; they are neither character IDs nor client-authoritative kill claims.

Six focused tests pass (`build/progression-core-v2.log`), with denied-warning
core/all-target Clippy (`build/progression-core-clippy-v2.log`). Tests cover malformed,
unknown/duplicate/oversized profiles, threshold boundaries, multi-level awards,
clamping, cap/no-op, source immutability, revision exhaustion, foreign profiles and
checked receipt words. Pure arithmetic deliberately does not deduplicate kills;
the durable receipt transaction below supplies that boundary. No new full-workspace
count or gameplay/native XP result is claimed.


## Qualified database/worker prerequisite

Migration 0007 adds character progression and positive-award receipts. The store
binds owner, character, world and the exact progression profile digest, locks the
progression row, and commits a positive award and its victim-life receipt together.
Realm/world/victim-life uniqueness permits one recipient under the development
last-hit policy. Conflicting recipient or creature reuse rejects. Exact older
receipts return current committed state with zero newly granted XP, even after
later awards. Initialization never reseeds existing progression. Positive receipt
growth is bounded by the finite XP budget; capped new facts create no rows.
Item revisions and reward claim consumption remain independent.

The adapter uses the existing sixteen-job critical worker queue. Closed replies
can skip work before execution; losing a reply after SQL begins does not cancel
the transaction. Backend/data/profile failures fail closed. These are store and
worker guarantees; runtime source entitlement and accepted-fact retention through
character retirement still require their own integration and qualification.

Evidence on the final prerequisite source:

- `build/progression-core-and-item-tests-v1.log`: 25 passing tests, comprising six
  progression, six equipment, five inventory and eight reward tests, with no skips.
- `build/progression-postgres-v1-20261030/verification.json`: twenty commands and
  fourteen actual SQL contracts pass on PostgreSQL 16.15. Three new progression
  contracts cover old-event replay, competing recipients, exact profile/world/
  owner binding, finite cap, forced atomic rollback, revision exhaustion, dropped
  replies, queue saturation and terminating a blocked database backend. Separate
  writer/reader processes retain exact XP and old receipts through database restart.
  The owned cluster is stopped and plaintext password files removed.
- `build/progression-store-fmt-v1.log`,
  `build/progression-workspace-check-v1.log` and
  `build/progression-workspace-clippy-v1.log`: formatting, workspace/all-target
  check and denied-warning Clippy pass.

Exact source, non-story documentation, the executed store probe and proof hashes
are frozen in `build/progression-store-qualified-v1-20261030`. This checkpoint does
not replace the qualified P3-09 game binaries or claim a new full-workspace/native
XP run. No rendering changed in this prerequisite. All changes remain local;
nothing was committed or pushed. Work stopped here at the user's request.


## Implemented live integration

The opt-in world binds the exact progression profile into its network identity.
Current installed-creature fatal damage from a live player produces a victim-life
fact, including periodic damage. The runtime verifies the admitted source and
profile before trying the sixteen-fact character queue. Failure rejects authority;
no client amount, recipient or XP event command exists. Fresh respawns have fresh
victim IDs. Item availability/claims and XP use independent observations/revisions.

The generation-bound XP owner initializes saved progression and awaits SQL outside
the simulation. It publishes the committed result through a superseding watch
observation. On logout, it closes the incoming fact queue and drains accepted
executing/queued work. Character release requires its completion in addition to
item-owner completion and the exact final-pose acknowledgement. A panic or dropped
owner fails admission closed. Missing initial XP is loading, never invented zero.

`XP01` is an exact 64-byte private reliable frame: actor epoch/entity, profile
digest, durable revision and integer total XP. The client verifies actor/profile,
bounds, nonregression and same-revision identity. Coalesced committed revisions
may skip intermediates. A missing initial observation times out after eight
seconds; inventory intent/reply ordering is unaffected. Standard Bevy text/bar
shows committed own-character level and within-level XP; no optimistic XP grant.

`python3 tools/play.py --account-login --progression` enables the development
profile and implies creature combat, retaining the normal OIDC/PostgreSQL setup.
It can combine with `--rewards`, equipment, resources and statuses. Native
qualification uses a separately byte-bound 0/25/50 curve with the same 25-XP kill
amount, solely to demonstrate a real level change promptly. The shipped development
profile remains 0/100/250. No class/stat/resource/gear/movement scaling is added.

Current integration evidence:

- `build/progression-authority-tests-v1.log`: 42 network unit tests and two wire
  tests pass, including live-source/current-life rejection and stale respawn facts.
- `build/progression-leases-v1-20261030/verification.json`: actual OIDC, encrypted
  ticket and PostgreSQL pass. An executing positive award and a distinct queued
  award survive logout; a duplicate older event grants nothing. No XP is published
  before COMMIT, committed pose alone cannot release the character, fresh lease
  restores XP, and the retired generation rejects new facts. This directly drives
  leases and is not native game proof.
- `build/progression-runtime-clippy-v2.log`: workspace/all-target denied-warning
  Clippy passes. Final native/restart and regression results are recorded below.


## Qualified live slice — 2026-10-30

- `build/progression-workspace-tests-v5-summary.json`: 388 passed, zero failed,
  24 explicit skips, 88 groups, exit zero and unchanged source hashes. All fourteen
  skipped SQL contracts run separately; ten private-asset cases remain skipped.
- `build/progression-postgres-v2-20261030/verification.json`: all fourteen actual
  SQL contracts/twenty commands pass, including rollback, cap, old receipts,
  competing recipients, lost answers, queue bounds, blocked-backend termination
  and real process/database restart. Owned cluster stopped and passwords removed.
- `build/progression-leases-v1-20261030/verification.json`: actual OIDC/encrypted
  tickets and SQL retain executing and distinct queued facts through logout,
  deduplicate old events and restore a fresh lease. Direct lease proof is separate
  from native gameplay. Its exact executed executable is retained in the checkpoint.
- `build/progression-native-1920-v2-20261030/verification.json` and
  `build/progression-native-ultrawide-v2-20261030/verification.json`: two actual
  OIDC accounts/native encrypted clients earn 25 XP/revision one each, displaying
  level two on the explicit 0/25/50 qualification curve. Each own observation
  matches SQL through fresh actor identities, rejoin and actual database/server
  restart with changed default spawn. Backend loss closes authority without
  changing saved XP. Reward/equipment widgets and conserved items also pass.
  All eleven pointer actions per initial client use Bevy picking/widgets; these
  are synthetic native input events, not physical OS mouse delivery.
- `build/progression-mixed-launcher-v3-20261030/verification.json`: ordinary CLI
  Scorch/resources/feedback/creatures/equipment/rewards/XP passes interruption,
  child failure and pending-login cancellation. The default curve remains
  0/100/250; the periodic fatal kill commits 25 XP/revision one, independently of
  reward pickup. Later launcher modes reload saved XP rather than re-awarding it.
- `build/progression-baseline-1920-v1-20261030/verification.json`: same release
  client, 1920x1200, one client, uncapped, validation off, empty-world tour:
  808.5973 FPS mean (1.23671 ms), worst one-second p95 1.42689 ms, 834 loaded /
  753 retired chunks, peak 81 resident, zero missed server simulation deadlines.
  This accelerated route verifies feature-disabled streaming regression; it does
  not establish crowded MMO capacity or ordinary gameplay reconciliation quality.
- Final formatting/denied-warning Clippy pass in
  `build/progression-final-fmt-v3.log` and
  `build/progression-final-clippy-v2.log`; the four explicit GPU-policy negative/
  exception tests pass in `build/window-gpu-policy-tests-v1.log`. Five account
  helper tests pass in `build/progression-account-helper-tests-v1.log`.
  `build/progression-server-dependencies-v1.txt` preserves the normal headless
  server graph without render/audio/UI/wgpu/navigation backends.

Screenshots were inspected on Linux RX 7900 XTX (RADV NAVI31), Mesa 26.2.6 kisak,
Vulkan. Actual physical extents match 1920x1200 windowed and 3440x1440 fullscreen;
1280x720 remains the mixed-launcher regression size. Windowed Vulkan validation
has no reported errors. Fullscreen's forced external validation layer records one
`VUID-VkSwapchainCreateInfoKHR-pNext-07781` per startup. The pinned wgpu-hal 30.0.1
[source explicitly handles this surface-resize race](https://github.com/gfx-rs/wgpu/blob/40f4a34ebaf56f9a046231f54125ad046239d3f3/wgpu-hal/src/vulkan/instance.rs#L44-L50).
The qualifier counts only that exact surface diagnostic during fullscreen, rejects
more than two occurrences per client, and fails every other validation ID/error,
panic, wrong actual extent or invalid UI bound. No renderer fork or dependency
change was introduced. Physical 4K and native Windows execution remain unqualified.

This is a finite development curve, not production balance, class/stat scaling,
group credit, rested/death/quest XP or a complete persistent combat system. Normal
logout/shutdown drains accepted work; a hard crash before COMMIT can lose an
unacknowledged kill fact. No general durable combat-event/outbox system is claimed.
All edits remain local, with no commit/push; concurrent story work stays untouched.
