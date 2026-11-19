Stopped safely at user request, 4 November 2026:
Qualified retained inventory/quest owner work is preserved at
`build/quest-item-owner-qualified-v1-20261104`: 1,271 workspace/all-target tests,
59 service skips and two actual OIDC/encrypted-ticket/PostgreSQL owner scenarios.
Subsequent work adds a confirmed destruction UI, exact confirmed-revision command,
and support for matching inventory acknowledgements carrying newer committed bags.
Eleven inventory bridge tests, seven encrypted account tests, 144 client tests,
formatting, strict all-target Clippy and optimized builds pass for those changes.
The first 1920x1200 native run stopped at an inventory Escape-close check before
testing destruction. It has zero GPU diagnostics on RX 7900 XTX / RADV Vulkan /
Mesa 26.2.6. Additional close-pending diagnostics and a settled-close wait are
built but not rerun. The destruction UI is work in progress, not native-qualified.
Preserve current source/build at `build/quest-item-ui-wip-stopped-v1-20261104`.
[Exact scope and resume work](../QUEST_ITEM_OBJECTIVES.md).
Reading/viewing notes is planned separately from possession. Live item-quest
admission stays closed; repeatable loot and full native item flow remain pending.
All owned services and desktop input stopped. Nothing committed or pushed.

Item-quest retained owner prerequisite qualified locally, 4 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: inventory initializes before item-bound journal
admission; all supported owner bag mutations select compound transactions.
Generation-bound publication preserves newer committed bag/journal/XP projections
and command replies when responses arrive late. Actual owned OIDC/encrypted-ticket/
PostgreSQL lease checks cover split/destroy/reacquire, refused missing-item turn-in,
retained turn-in after logout, rejoin and replay without duplicate consumption/XP.
The earlier historical journal owner also passes its actual service regression.
All 1,271 workspace/all-target tests (59 service skips), formatting and strict
workspace/all-target Clippy pass. Preserve
`build/quest-item-owner-qualified-v1-20261104`.
[Exact evidence and remaining native work](../QUEST_ITEM_OBJECTIVES.md).
NPC/event entitlement in owner checks is synthetic; separate channels may render
on separate frames. Live World/manifest item admission stays closed. Confirmed
UI, repeatable loot and normal encrypted/native item flow remain pending. Reading
notes is recorded as a distinct planned objective. Changes stay local.

Stopped safely at user request, 3 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: item-quest storage checkpoint is preserved, plus
checked quantity destruction in core, canonical IR02-only destruction requests
and compound SQL destruction/readiness reconciliation. A real split-one then
destroy-one SQL command changes Ready 3/3 to Active 2/3; retries never remove twice.
All 347 selected core/persistence/server tests, 14 inventory/journal codec checks,
19 actual owned PostgreSQL journal contracts, formatting and strict workspace/
all-target Clippy pass. Preserve
`build/quest-item-destruction-storage-qualified-v1-20261103`.
[Behavior, limits and resume work](../QUEST_ITEM_OBJECTIVES.md).
Unfinished destruction UI is set aside as an ignored patch; the client UI is
restored to the prior qualified source. Item-quest live admission remains closed.
Resume with retained owner initialization/coherent publication, confirmed UI,
per-victim-life repeatable loot and normal encrypted/native item flow. Latest
native playable remains the area checkpoint. All local; nothing committed/pushed.

Item-quest durable storage prerequisite qualified locally, 3 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: exact item-profile binding, locked current-bag
reconciliation and atomic item consumption/completion/XP/action receipts. Optional
compound move/split/equipment, vendor and reward-claim APIs reconcile all active
item quests before the same COMMIT. Accepted compound jobs drain after response
loss. Existing unbound historical quest profiles retain their hashes and paths.
All 345 selected core/persistence/server tests (48 service skips), 18 actual owned
PostgreSQL journal contracts, formatting and strict workspace/all-target Clippy
pass; durable bag/journal/XP/receipt hashes match after database restart.
Preserve `build/quest-item-storage-qualified-v1-20261103`.
[Exact evidence and pending live work](../QUEST_ITEM_OBJECTIVES.md).
SQL authorization/removal fixtures are synthetic. Live World/manifest item
admission remains closed; owner publication, destruction, repeatable loot and
normal encrypted/native item flows are pending. Latest playable remains the area
client. No save migration or new full-workspace/GPU/Windows claim. Local only.

Reversible item-quest core/codec prerequisite qualified locally, 3 November 2026:
P3-05 / P3-11 / P3-15: quest schema 3 adds explicit possession requirements and
consume-on-turn-in flags in an all-required final stage. Current holdings can
change Ready back to Active while permanent facts remain. Pure turn-in rechecks
current bags, prepares consumption/completion and recalculates other quests
sharing the materials. Completed quests stay completed.
All 335 core tests, nine codec checks, 72 authoring-tool checks, formatting and
strict workspace/all-target Clippy pass. Preserve
`build/quest-item-journal-qualified-v1-20261103`.
[Behavior, evidence and pending live work](../QUEST_ITEM_OBJECTIVES.md).
Normal World/manifest admission still refuses unbound item quests. No new item
SQL transaction, deletion UI, repeatable loot, encrypted/native item quest,
save migration or full-workspace claim. Area client remains the latest playable.
Changes stay local; storage/ownership integration is next.

Quest-item possession/consumption prerequisite qualified locally, 3 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: checked backpack requirements count all stacks,
preserve totals across split/merge, distinguish kept from consumed requirements
and prepare all-or-nothing consumption against the supplied current bag.
All 327 core tests, seven new requirement cases, formatting and denied-warning
workspace/all-target Clippy pass. Preserve
`build/quest-item-possession-qualified-v1-20261103`.
[User semantics, pinned Classic-emulator evidence and remaining work](../QUEST_ITEM_OBJECTIVES.md).
This pure helper is not live quest progress, item deletion, repeatable loot or
a SQL reward transaction. The area client remains the latest native playable.
Reversible item objectives, synchronized storage/owner publication and real
wolf-pelt loot/split/delete/reloot/turn-in are pending. Changes stay local.

Reach-area quest normal client qualified locally, 3 November 2026:
P3-02 / P3-11 / P3-12 / P3-15 / initial P3-16: opt-in manifest 16 binds static
metric volumes to exact quest/XP content. Current alive authoritative feet
produce bounded stage-scoped batches; the retained owner commits all changed
quests and replay receipts before private publication. Overlapping multi-goal
batches now preserve monotonic journal heads above every contained revision.
All 1,169 workspace tests / 54 service skips, 51 manifest / 36 launcher checks,
14 actual PostgreSQL journal contracts, formatting and strict all-target Clippy
pass. Normal unmodified data/native OS controls complete the original two quests
plus the overlook (about 134 m west), return and turn in: head 16 / 150 XP.
Database restart/rejoin preserves all three completions, six actions and ten
event receipts. Two native starts have zero GPU diagnostics on Linux RX 7900 XTX
/ RADV Vulkan / Mesa 26.2.6. Preserve
`build/quest-reach-client-qualified-v1-20261103`.
[Exact semantics, proof and limits](../QUEST_REACH_AREAS.md).
Earlier schema-15 content/playables remain; no save migration, runtime packs,
native Windows, FPS benchmark or MMO-capacity claim. All changes local.
Next: user-requested possession-based item quests; splitting remains allowed.

Reach-area queue/encrypted prerequisite qualified locally, 3 November 2026:
P3-11 / P3-15 / initial P3-16: fixed-step authority reserves one bounded owner
slot before sampling a typed stage-bound area batch. Full queues preserve retry;
unchanged batches do not repeat while commit is pending. The retained server
owner uses the qualified storage helper and publishes only committed views.
All 354 network-library tests, formatting and strict workspace/all-target Clippy
pass. New encrypted two-client delay/loss/duplicate cases at 20/30/60 Hz advance
one batch per stage, keep the other journal private and award XP only at turn-in.
Those encrypted cases use an explicit in-memory commit fixture; PostgreSQL was
qualified separately. The combined native/SQL path is still pending.
Preserve `build/quest-reach-owner-qualified-v1-20261103`.
[Evidence and remaining normal-launch/native work](../QUEST_REACH_AREAS.md).
No normal manifest area role, native area quest, save migration or new full
workspace/GPU/Windows/capacity claim. Changes remain local.

Reach-area storage/worker prerequisite qualified locally, 3 November 2026:
P3-02 / P3-11 / P3-15 / initial P3-16: area-aware persistence binds exact profile
bytes, stage-scoped observations and domain-separated replay receipts. All
affected quests and their receipt commit atomically; accepted jobs survive a
dropped response and worker shutdown. Area evidence grants no XP; normal turn-in
remains the reward transaction. Thirteen actual owned PostgreSQL journal
contracts pass, including three new area cases, plus four non-SQL checks,
formatting and strict workspace/all-target Clippy. Durable table hashes match
across an actual database restart with fsync/synchronous_commit/full_page_writes.
Preserve `build/quest-reach-store-qualified-v1-20261103`.
[Exact proof and pending owner/native integration](../QUEST_REACH_AREAS.md).
Trusted SQL observations are synthetic; no encrypted/native area credit,
manifest installation, save migration or new full-workspace/GPU claim. Local only.

Reach-area World adapter qualified locally, 3 November 2026:
P3-11 / P3-12 / P3-15 / initial P3-16: opt-in exact-area-byte quest profiles
validate selectors, installed namespaces and continental bounds. Samples use
current alive authoritative capsule feet, current World/actor binding and ready
collision coverage, with no client position argument or simulation I/O.
All 319 core / 352 network-library tests, 67 authoring-tool checks, formatting and
strict workspace/all-target Clippy pass; five new authority cases include
20/30/60-Hz predicate fixtures and dead-life exclusion. Preserve
`build/quest-reach-world-qualified-v1-20261103`.
[Evidence and remaining durable/live work](../QUEST_REACH_AREAS.md).
The immediately preceding full workspace passed 1,160 tests / 50 service skips;
this checkpoint does not claim a new full-workspace or GPU/native run.
Normal manifests still refuse area content; no durable area credit, save
migration or new Windows/capacity claim. Changes remain local.

Reach-area pure proposal prerequisite qualified locally, 3 November 2026:
P3-11 / P3-12 / P3-15 / initial P3-16: schema 2 adds count-one area selectors;
stage-scoped batches cannot credit later stages or later acceptances. Overlap,
quorums, delayed same-stage credit, 128-goal bounds and saved-count round trips
pass. Compiler selects the minimum schema; review detects selector changes.
All 1,160 workspace tests / 50 skips, 319 core tests, 66 authoring-tool checks,
formatting and strict workspace/all-target Clippy pass. Preserve
`build/quest-reach-stage-qualified-v1-20261103`.
[Semantics and pending live integration](../QUEST_REACH_AREAS.md).
Normal World/manifest admission explicitly refuses unsupported area objectives.
No live area credit, runtime pack loading, migration, new GPU or Windows claim.
Normal playable remains directional v1; all changes remain local.

Reach-area geometry prerequisite qualified locally, 3 November 2026:
P3-11 / P3-12 / P3-15 / initial P3-16: strict finite XYZ volumes use checked
global metres, inclusive faces and bounded inputs; no renderer/physics/streaming
is created. All 310 core tests, six actual CLI cases, formatting and strict
workspace/all-target Clippy pass. Preserve
`build/quest-reach-area-geometry-qualified-v1-20261103`.
[Exact semantics, evidence and pending stage-bound/live work](../QUEST_REACH_AREAS.md).
This is geometry only: quest schema 1 and its budgets remain, with no World area
credit, runtime packs, save migration or new GPU/Windows/capacity claim.
The qualified directional playable remains available; story IDs are untouched.

Classic directional locomotion runtime qualified locally, 3 November 2026:
P2-09 / P3-08 / P2-05: optional eight-way walking/running uses displayed manual
heading and checked global motion; backward run selects RunBack. Phase continuity,
standard Bevy finger/strike masks and death/jump priorities pass local/remote
native source-pose checks. All 1,145 workspace tests / 50 skips, 144 client tests,
formatting and strict all-target Clippy pass. Seven final native client starts
pass: two-account animation/life fixture, new-art ultrawide controls, and normal
unmodified quests/database restart/rejoin with both new art (1920) and old-art
fallback (720p). Normal new-art W traversal covers 207.9 m, streamed retirement
and an origin change. Zero GPU diagnostics on Linux RX 7900 XTX / RADV.
Preserve `build/classic-directional-client-qualified-v1-20261103`.
[Exact art/mapping/fixture scope, proof and remaining work](../CLASSIC_DIRECTIONAL_LOCOMOTION.md).
Crossfades/stride tuning and the existing skin-preview limitation remain.
Earlier checkpoints unchanged; local only, no native Windows/MMO-capacity claim.

Directional locomotion asset prerequisite qualified, 3 November 2026:
P2-09 / P3-08 / P2-05: separate 22-clip POLYGON asset retains all eight original
clips, skin, weights, binds, palette and sword sockets. Independent 702-frame
source comparison and 640 retained pose samples per artifact pass, with zero
retained matrix difference. Thirteen Python checks and standard glTF validation
pass. Fourteen native clip inspections have active Khronos validation and zero
GPU diagnostics on Linux RX 7900 XTX / RADV. The existing 8.8 mm four-weight
preview limitation remains; no production-fidelity claim.
Preserve `build/classic-directional-art-qualified-v1-20261103`.
[Asset proof and pending runtime adoption](../CLASSIC_DIRECTIONAL_LOCOMOTION.md).
Rust is unchanged from ground-v3; live directional selection is next. Local only.

Classic ground movement v3 qualified locally, 3 November 2026:
P2-04 / P2-05 / P2-07 / P2-09 / P2-11: shared authority/prediction uses
data-bound 7 m/s run, 4.5 m/s backward run and 2.5 m/s walk; schema-2 controls
default to run with NumpadDivide walk/run toggle, removing Classic Shift boost.
All 1,139 workspace tests / 50 skips, 138 client tests, 35 launcher/bundle checks,
formatting and strict all-target Clippy pass. Three native resolutions pass all
27 measured speed checks; a separate normal unmodified wolf/quest run and
database-restart rejoin preserve head 13 / 125 XP, followed by 207.7 m of normal
W traversal through streamed chunks and an origin change. Five final native
starts have zero GPU diagnostics on Linux RX 7900 XTX / RADV. Preserve
`build/classic-ground-client-qualified-v1-20261103`.
[Exact mapping, source uncertainty, compatibility and evidence](../CLASSIC_GROUND_MOVEMENT.md).
Earlier checkpoints unchanged; local only, no native Windows/MMO-capacity claim.

Local minimap traversal follow-up qualified, 3 November 2026:
P2-14 / P3-13: retained no-facing arrow now reads displayed character rotation;
Classic controls and gameplay remain unchanged. Actual normal W movement walks
208.3 m through three chunk owners and a render-origin change after OIDC/
PostgreSQL restart/rejoin, preserving head 13 / 125 XP. A separate explicit 32x
development-ticket run crosses the 16-km extent: 125 total origin changes,
62 independently sampled origins, bounded terrain retirement and correct dark
world edge. All 1,134 workspace tests (50 skips), 136 client / two checker tests,
formatting and strict all-target Clippy pass. Five final native starts have zero
GPU diagnostics on Linux RX 7900 XTX / RADV; normal UI checks retain all three
supported resolutions. Preserve
`build/minimap-traversal-client-qualified-v1-20261103`.
[Exact normal/accelerated scopes and measurement corrections](../MINIMAP.md).
Earlier checkpoints remain unchanged; local only, no native Windows/MMO-capacity claim.

Initial local minimap qualified, 3 November 2026:
P2-14 / P3-13: standard Bevy north-up local map, resident-mesh height sampling,
checked global projection, manual player heading and committed guide/quest icons.
Two fixed images use 17 KiB; no new camera, terrain read or residency request.
Health/mana and backpack layout remain usable at all three supported sizes.
All 1,133 workspace tests (50 skips), 135 client tests, fmt and all-target
denied-warning Clippy pass. Normal OIDC/PostgreSQL/native two-quest completion
and database-restart rejoin pass at 1280x720, 1920x1200 and 3440x1440 fullscreen:
four starts, zero GPU diagnostics. Visible map refresh leaves terrain reads and
residency unchanged; maximum raster 0.329 ms, active presentation mean
25.64–26.74 us on Linux RX 7900 XTX / RADV Vulkan. Preserve
`build/minimap-client-qualified-v1-20261103`.
[Exact Classic scope, measurements and remaining work](../MINIMAP.md). Local only;
no new native Windows/continent-crossing/population-capacity claim.

Renet lost-initial-acknowledgement fix qualified, 3 November 2026:
P1-06 / P1-08 / P3-04 / P3-16: bounded encrypted NC01 confirmation precedes
World/account admission; Renet transport revision 1 leaves movement wire/Quinn
12 and library pins unchanged. Full workspace: 1,128 passes / 50 explicit skips;
two focused loss-policy checks, formatting and all-target denied-warning Clippy
pass. Final baseline/seeded-loss comparison passes 48 cases at 20/30/60 Hz,
including 24 reused endpoints, 60 dropped initial KeepAlives and 3,607 old-packet
replays. Retain the separate periodic-loss result: 46/48, with snapshot erasure
after successful Welcome. Normal OIDC/PostgreSQL/native quests and database
restart/rejoin pass (head 13 / 125 XP); two 1920x1200 starts have zero GPU
diagnostics on Linux RX 7900 XTX / RADV Vulkan. Preserve
`build/connection-confirmation-client-qualified-v1-20261103`.
[Implementation, exact evidence and loss limitations](../RENET_CONNECTION_CONFIRMATION.md).
No native Windows/population/universal-loss-recovery claim. Local only.

Read-only quest update review qualified, 3 November 2026:
P3-11 / P3-15: compare two validated closed executable catalogues by stable IDs,
versions, exact hashes and review-only gameplay fingerprints. Distinguish rule,
presentation/layout and version changes; report missing bumps/regressions without
copying narrative text. Inputs are never rewritten; no save compatibility,
World-reference admission or automatic migration claim. Seven review checks and
all 65 authoring-example checks pass, along with formatting/all-target Clippy.
Actual built CLI proves deterministic hashes/output, input preservation and
unsupported/oversized refusal. Preserve
`build/quest-update-review-qualified-v1-20261103`. Normal schema-15 playable is
unchanged; its earlier 1,123-test/native qualification remains the runtime
baseline. [Workflow and limits](../QUEST_PACK_AUTHORING.md). Local only.

Normal public player death/fresh-life presentation qualified, 3 November 2026:
P3-04 / P3-14 / P3-16: schema-15 matched normal CLI/bundle, public visual life
roster and standard Bevy remote Death/fresh-life reset, with life-aware pose
reset and owner-private controls/vitals retained. Actual two-account native
observer sees a finished corpse in view (52 changing Death bones), then fresh
generation/pose; it stays at 100 HP with no private incoming windup/target. Owner
live casting is verified, then dead movement/new casts are blocked. Unmodified
quest completion and PostgreSQL restart/rejoin pass: two quests, three kills,
head 13 / 125 XP, no duplicate rewards. All 1,123 workspace tests (50 skips),
131 client / 47 validator / 35 Python checks, formatting and all-target Clippy
pass. Three final native runs at 1920x1200 have zero GPU diagnostics on Linux
RX 7900 XTX / RADV Vulkan. Preserve `build/avatar-life-client-qualified-v1-20261103`.
Older bundles remain unchanged. [Exact scope and endpoint limitation](../PUBLIC_AVATAR_LIFE.md).
Synthetic PID-bound X11 input; no native Windows/population/Classic corpse-run
claim. All changes remain local.

Public avatar lifecycle transport qualified, 3 November 2026:
P3-04 / P3-14 / P3-16: shared bounded priority reliable channel, readiness gate,
full roster/changed-lifecycle publication, private-vitals separation and explicit
queue/upload/profile refusal. Actual encrypted observer sees death/respawn,
leave and fresh-endpoint rejoin under impairment at 20/30/60 Hz; old encrypted
traffic cannot restore the former life. Separate two-peer NPC/AV coexistence
passes at all three rates. All 670 network tests (ten skips), formatting and
workspace/all-target denied-warning Clippy pass. Preserve
`build/avatar-life-runtime-qualified-v1-20261103`. Normal CLI/bundle/remote Bevy
adoption remains next; normal schema-14 public-NPC playable is unchanged.
[Scope, replay proof and same-endpoint limitation](../PUBLIC_AVATAR_LIFE.md).
Trusted development profiles/synthetic ports; no new native/SQL/OIDC/Windows or
population-capacity claim. All changes remain local.

Public avatar life observation/codec prerequisite qualified, 3 November 2026:
P3-04 / P3-14 / P3-16: explicit pre-admission World/profile and bounded AV01
actor/life/alive-dead roster; no health, damage or respawn-control fields. Nine
focused wire/World checks cover actual death/respawn at 20/30/60 Hz, the bounded
connected set, stale/invalid data and unavailable authority. All 664 network
tests (ten skips), formatting and workspace/all-target denied-warning Clippy
pass. Preserve `build/avatar-life-codec-qualified-v1-20261103`. Public transport,
normal version and remote Bevy adoption remain next. Latest normal playable is
public-NPC schema 14, unchanged. [Scope](../PUBLIC_AVATAR_LIFE.md). Local only; no
new native/SQL/OIDC/Windows/population claim.

Normal public NPC attack presentation qualified, 3 November 2026:
P3-07 / P3-14 / P4-08: schema-14 exact-byte bundle and matching normal CLI identities
adopt the priority public timing stream with standard Bevy wolf clips and actual
victim facing. Native two-account observer stays at 100 HP, receives no private
incoming windup, keeps no target selected and renders remote-victim attacks.
Unmodified-wolf normal quest completion and database-restart rejoin also pass:
two quests, three kills, head 13 and 125 XP, no duplicate rewards. All 1,106
workspace tests (50 skips), 129 client / 46 validator / 34 Python checks,
formatting and all-target denied-warning Clippy pass. Three successful native
runs at 1920x1200 have zero GPU diagnostics on Linux RX 7900 XTX / RADV Vulkan.
Preserve `build/creature-attack-client-qualified-v1-20261103`; older bundles remain
unchanged. [Exact scope and evidence](../PUBLIC_CREATURE_ATTACKS.md). Synthetic
PID-bound X11 input; no native Windows or population-capacity claim. Local only.

Public NPC attack transport qualified, 3 November 2026:
P3-07 / P3-14 / P4-08: optional 8-KiB reliable priority channel, accepted-readiness
publication gate, exact profile/rate checks and bounded 16-event presentation
handoff. Actual encrypted victim/observer peers agree on windup/cancel/transfer
at 20/30/60 Hz under loss, jitter, duplication and reordering; incoming strikes
and vitals remain private. Four additional boundary checks cover full queues,
readiness and forged uploads on both channels. All 655 network tests (ten skips),
formatting and workspace/all-target denied-warning Clippy pass. Preserve
`build/creature-attack-runtime-qualified-v1-20261103`. Normal CLI/bundle/Bevy
adoption remains next; threat v2 playable is unchanged. [Scope](../PUBLIC_CREATURE_ATTACKS.md).
Synthetic admission/inventory ports; no native/SQL/OIDC/Windows/capacity claim.
All changes remain local.

Public NPC timing observation/codec prerequisite qualified, 3 November 2026:
P3-07 / P3-14 / P4-08: explicit pre-admission World profile/hash, read-only NPC
windup observation and strict 62/86-byte NA01 contract. No health/damage/outcome
or private player-life generation fields. Ten focused cases cover owner/observer,
transfer/cancel/death/respawn, malformed/stale/expired frames and 20/30/60 Hz.
All 650 network tests (ten skips), formatting and all-target Clippy pass. Preserve
`build/creature-attack-codec-qualified-v1-20261103`. Actual reliable public
replication, optional normal adoption and Bevy bystander animation remain next;
normal threat v2 playable is unchanged. [Scope](../PUBLIC_CREATURE_ATTACKS.md).
Local only; no new native/Windows/population claim.

Closed quest-subset authoring handoff qualified, 3 November 2026:
P3-11 / P3-15: deterministic compiler includes every transitive prerequisite,
preserves definition IDs/versions/behavior and revalidates the closed runtime
catalogue. Refuses 32-quest/512-objective/64-KiB overflow rather than truncating.
Explicit `--compile-selection` emits catalogue bytes plus source/selection/output
hashes; inputs stay unchanged. Actual compiled output passes normal schema-13
manifest validation, while an uninstalled guide correctly refuses admission.
All 1,089 workspace tests (50 skips), 304 core, 45 gameplay-validator and 11
authoring-tool checks, formatting and all-target Clippy pass. Preserve
`build/quest-compile-qualified-v1-20261103`. [Workflow](../QUEST_PACK_AUTHORING.md).
No runtime pack loader, history expansion, automatic migration or normal content
replacement. Latest normal playable remains threat v2; changes stay local.

Offline quest pack/index workflow qualified, 3 November 2026:
P3-11 / P3-15: strict contained JSON index, bounded regular-file reads, global
IDs/prerequisites and exact hash report. Typed diagnostics identify files/IDs;
no narrative dump or source mutation. Seven core registry / seven file checks,
all 297 core and 44 existing gameplay-validator checks, formatting and Clippy
pass. Actual compiled CLI success/refusal and read-only hashes pass. Both example
validators are configured in Linux/Windows CI; only local Linux execution is
claimed. [Authoring workflow](../QUEST_PACK_AUTHORING.md). Preserve
`build/quest-registry-cli-qualified-v1-20261103`. Runtime pack loading, history/
migration and world-reference admission are separate; normal playable unchanged.
Local only; next is a bounded reviewed subset handoff to the closed catalogue.

Offline quest pack-registry prerequisite qualified, 3 November 2026:
P3-11 / P3-15: bounded pure Registry validates global definition IDs, cross-pack
prerequisites/cycles and target aliases across up to 32 executable packs (2 MiB,
1,024 quests under current development bounds). Six new checks include a full
padded chain; all 296 core tests, formatting and all-target denied-warning Clippy
pass. Live `Catalog::from_json` stays closed/local; Registry exposes no Catalog
for journal loading. No runtime, save-history or migration change. Preserve
`build/quest-registry-core-qualified-v1-20261103`; normal playable remains the
schema-13 threat v2 checkpoint. Offline file/index validation is next. Local only.

Unmodified wolf/threat regression qualified, 3 November 2026:
P3-07 / P4-08: actual normal schema-13 OIDC/PostgreSQL/native flow also passes
with unchanged wolf perception, speed and retaliation profiles. Two quests,
three kills, head 13 and 125 XP survive a database process restart/rejoin; no
player death, zero GPU diagnostics. Fresh threat counters show 22 resolved facts,
300 actual HP lost and three clears; rejoin records none. A new actual World
passive body-pull case passes at 20/30/60 Hz: zero damage threat, no tag claim and
real retaliation. All 66 affected authority tests, formatting and all-target
Clippy pass; the 1,074-test workspace baseline is inherited with only test-helper
changes plus this case. Preserve `build/creature-threat-client-qualified-v2-20261103`.
Linux RX 7900 XTX / RADV Vulkan; native 1920x1200, synthetic OS input. Services
stopped; no Windows or population-capacity claim. All changes remain local.

Normal damage-threat playable qualified, 3 November 2026:
P3-07 / P4-08: schema-13 canonical journal/threat bundle, exact-byte launcher
snapshot and explicit server/client identity wiring pass normal OIDC/PostgreSQL
native completion and database-restart rejoin at 1920x1200. Fresh state has two
completed quests, head 13 and 125 XP; three kills commit 300 actual HP lost and
clear threat three times. Rejoin adds no progress/reward/damage. Both runs have
zero GPU diagnostics on Linux RX 7900 XTX / RADV Vulkan / Mesa 26.2.6 (fresh-run
adapter capture). The private combat fixture remains explicit; two-peer victim
transfer has separate impaired 20/30/60 Hz proof. Final workspace 1,074 tests pass
(50 ignored), plus 44 validator / 127 client / 33 Python checks, formatting and
all-target denied-warning Clippy. Preserve
`build/creature-threat-client-qualified-v1-20261103` and prior playables. Older
bundles/bare launcher stay unchanged. General/group/healing threat, streamed NPC
navigation, production capacity and Windows remain open. All changes stay local.

Normal creature threat adoption in progress, 3 November 2026:
P3-07 / P4-08: schema-13 canonical journal/threat bundle, exact-byte launcher
snapshot and explicit server/client `--creature-threat` identity wiring are
implemented. 44 validator, 127 client and 33 Python handoff checks pass, plus
formatting and workspace/all-target denied-warning Clippy. Optimized build and
normal OIDC/PostgreSQL native regression are pending. Preserve both qualified
threat World/runtime checkpoints and the schema-12 repeat-choice playable.
No new normal playable/native adoption claim yet. Changes stay local.

Encrypted creature threat competition qualified, 3 November 2026:
P3-07 / P4-08: two actual encrypted peers at 20/30/60 Hz under packet loss,
duplication and reordering observe victim transfer, old-windup cancellation and
new retaliation. Public outgoing results agree; incoming strikes remain private.
Damage threat matches actual NPC health lost; relay overflow is zero. Formatting,
all-target denied-warning Clippy and the three-rate test pass. Preserve
`build/creature-threat-runtime-qualified-v1-20261103`; ordinary schema-12 quest
playable is unchanged. Next is explicit versioned CLI/bundle/native adoption.
Synthetic admission/inventory ports; no new SQL/OIDC/native/capacity claim. Local.

Creature threat World adapter qualified, 3 November 2026:
P3-07 / P4-08: opt-in committed-health-loss threat, current metric reach,
prepared victim transfer, old-windup/path invalidation, retained membership and
independent tag ownership. Ten World and two transfer/path checks pass; full
workspace 1,071 tests pass (50 ignored), followed by two additional focused cases.
Formatting/all-target denied-warning Clippy pass. Fixed same-frame Return/death
cleanup. Preserve `build/creature-threat-world-qualified-v1-20261103`; latest normal
playable stays `build/quest-journal-repeat-qualified-v1-20261103`. Actual encrypted
competition and normal CLI/manifest/native adoption remain next. Local only.
[Scope and evidence](../CREATURE_THREAT.md).

Repeated NPC interaction qualified, 3 November 2026:
P2-14 / P3-11 / P3-12: standard Bevy click-on-release now also handles shared
NPC dialogue choices. Actual 1920x1200 normal OIDC/SQL native input repeats the
same choice twice before acceptance, then completes both quests normally and
rejoins after a database restart. The two earlier conversations grant no quest
credit or XP; final state has head 13, nine events, four actions and 125 XP.
127 client tests, formatting, all-target denied-warning Clippy and two native
runs pass; GPU diagnostics are zero. Prior wide/small layout proof is inherited
unchanged. The private combat fixture remains explicit. Preserve
`build/quest-journal-repeat-qualified-v1-20261103`; all work stays local.

Expanded journal playable qualified, 3 November 2026:
P2-14 / P3-02 / P3-11 / P3-12 / P3-15: schema-12 canonical bundle,
authenticated World/owner wiring, standard Bevy journal/actions and aggregated
markers pass 127 client / 41 validator / 31 Python checks, formatting and
workspace/all-target denied-warning Clippy. Five normal OIDC/PostgreSQL native
runs cover fresh completion at 1920x1200, 3440x1440 fullscreen and 1280x720, plus
two post-database-restart rejoins. Each fresh character completes both quests:
three kills, four semantic choices, four durable actions, seven events, head 11
and 125 XP. No extra offers/rewards on rejoin; GPU diagnostics are zero on Linux
RX 7900 XTX / RADV Vulkan / Mesa 26.2.6, connected display. The private combat
fixture reduces perception/speed/retaliation policy damage; quest/reward data are
unchanged. The 1,057-test runtime and 40-contract SQL baselines are inherited.
Preserve `build/quest-journal-client-qualified-v1-20261103` and the schema-11
playable. Bare launcher stays a walking sandbox; the new bundle is explicit.
No production campaign-scale, physical-input hardware or Windows claim. Local.

Normal expanded-journal adoption in progress, 3 November 2026:
P2-14 / P3-02 / P3-11 / P3-12 / P3-15: schema-12 canonical content bundle,
authenticated CLI/owner wiring, standard Bevy journal/NPC actions and committed
marker aggregation are implemented as an explicit mode. All 127 client, 41
validator and 31 Python handoff tests pass; formatting and all-target denied-
warning Clippy pass. Optimized build and owned native acceptance are pending.
No new playable/default adoption claim yet. Preserve the schema-11 playable and
`build/quest-journal-runtime-qualified-v1-20261103`. Changes stay local.

Expanded journal transport qualified, 3 November 2026:
P1-06 / P1-08 / P3-11 / P3-12: installed World/owner identity, bounded
semantic/atomic-kill routing and private reliable state/action dispatch pass
1,057 workspace tests (50 ignored), formatting and workspace/all-target
denied-warning Clippy. Fifteen focused checks include two encrypted peers with
measured loss/duplication/reordering at 20/30/60 Hz and a no-kill-XP 30-Hz case.
Transport uses a synthetic commit adapter; actual OIDC/SQL retirement also passes
separately. The 40 SQL contracts/restart proof are inherited unchanged from the
kill checkpoint. Preserve `build/quest-journal-runtime-qualified-v1-20261103`.
Normal manifest/CLI/UI and combined native/durable acceptance remain open.
Services are stopped; all changes remain local.

Original kill routing qualified, 3 November 2026:
P3-02 / P3-10 / P3-11 / P3-12: original victim-life facts now use the bounded
journal router, retained owner and shared critical worker's atomic optional-XP
transaction. All 1,050 workspace tests pass (50 ignored; 40 SQL contracts
executed explicitly), formatting and workspace/all-target denied-warning Clippy.
Actual OIDC/SQL logout/reconnect preserves command replies and grants kill XP
once; new atomic journal/receipt fixtures survive database process restart.
Preserve `build/quest-journal-kill-qualified-v1-20261103`. Normal World profile,
UDP journal/client and native adoption remain open. Changes remain local.

Stopped safely at the user's request, 2 November 2026:
The [expanded quest action/router and optional-XP kill Store](../QUEST_ENGINE.md)
prerequisites are qualified locally. All 290 core, eight codec, eight focused
World/router and one competing-player credit/routing check pass, plus all 39
owned PostgreSQL contracts, formatting and workspace/all-target denied-warning
Clippy. Test services are shut down. Commit `4ceff46` is pushed; the later action
and atomic-kill slice remains local. Preserve
`build/quest-journal-atomic-qualified-v1-20261102`. Normal runtime/client adoption
remains open; legacy playable is unchanged. Resume with original victim-life
facts through the bounded worker/router, then encrypted/client/native acceptance.
No further implementation is authorized until the user resumes.

Quest action codec/router prerequisite qualified, 2 November 2026:
P1-06 / P1-08 / P3-11 / P3-12: [QK01/QP01 action boundaries](../QUEST_ENGINE.md)
use 110-byte requests and 94-byte replies, carrying explicit actor/NPC/profile,
original intent and painted quest revision. Eight codec and eight focused World/
router checks pass, formatting and workspace/all-target denied-warning Clippy.
One pending command shares the event producer sequence; success waits for a
matching committed owner result. Channel tests use synthetic owner views; normal
UDP/client adoption remains open. Before replacement, preserve configured kill XP
and journal-event atomicity while retaining quest-only events without XP recipes.

Compact quest observation prerequisite qualified, 2 November 2026:
P1-08 / P3-11 / P3-15: [QJ01 journal snapshots](../QUEST_ENGINE.md) encode the
current full development maximum in 925 bytes, retaining the existing 8-KiB
reliable channel budget. Five codec checks, all 290 core tests, formatting and
workspace/all-target denied-warning Clippy pass. Typed reconstruction validates
logical state; successor checks preserve completion/history and reject rollback.
Actual OIDC/ticket/SQL owner tests also confirm later events preserve the latest
command reply through logout. This is an unadopted snapshot codec, not new client
journal UI or live request support. Next: action codec and runtime/client adoption.

Explicit quest routing adapter qualified, 2 November 2026:
P3-02 / P3-11 / P3-12: [World-to-owner reservation](../QUEST_ENGINE.md) refuses
full/closed/uninitialized owners before a semantic choice changes dialogue.
Kill routing reserves the whole recipient batch before sending and preserves
original IDs/sequence on refusal. Eight focused World/channel checks, all 290
core tests, formatting and workspace/all-target denied-warning Clippy pass.
Actual OIDC/ticket/SQL retirement is repeated and verifies Store/network exact
catalogue identity agreement. World routing tests use explicit synthetic owner
views, separate from the actual durable-owner proof. Normal runtime request
adoption, private journal codec/UI and combined native acceptance remain open.

Expanded quest World adapters qualified, 2 November 2026:
P3-11 / P3-12 / P3-15: [installed references and trusted occurrences](../QUEST_ENGINE.md)
pass seven new focused checks, including 20/30/60-Hz semantic choices and actual
competing-player kill credit without XP configuration. Missing contacts/actions/
creatures refuse; stale/out-of-range/foreign-World input yields no occurrence.
6 existing core dialogue and 7 existing World dialogue checks also pass;
formatting and workspace/all-target denied-warning Clippy pass. This explicit API
is not routed through the normal runtime yet. Retain the qualified owner and
schema-11 playable. Next: bounded World-to-owner routing and private journal
transport/UI, with separate durable/encrypted/native acceptance.

Multi-quest retained owner qualified, 2 November 2026:
P3-02 / P3-11 / P3-12 / initial P3-16: [expanded journal ownership](../QUEST_ENGINE.md)
now shares the existing character-bound progression FIFO and retirement gate.
Actual OIDC, encrypted tickets and owned SQL prove logout retains accepted
acceptance/events/turn-in/prerequisite work, reconnect waits, and retries add no
XP. Read-only admission checks exact journal identity. All 1,072 workspace tests
pass (46 skipped; 36 SQL contracts then executed explicitly); formatting and
workspace/all-target denied-warning Clippy pass. Normal World semantic events,
combat credit routing, journal transport/client and native adoption remain open.
Legacy schema-11 playable is preserved. Production catalogue/registry and saved
completion-history scaling remain planned, not a permanent 32-quest limit.

Journal worker prerequisite qualified, 2 November 2026:
P3-02 / P3-11 / P3-12 / initial P3-16: [bounded journal jobs](../QUEST_ENGINE.md)
use the shared 16-slot critical FIFO, retain accepted journal mutations after
response cancellation, and drain accepted work on graceful worker shutdown.
Five journal / 35 total owned SQL contracts pass, including lock-delayed
acknowledgement, saturation, cancelled answers, completion XP and prior process
restart checks. Formatting and workspace/all-target denied-warning Clippy pass.
World/character retirement, transport/UI and native adoption remain separate.
Production catalogue scale is planned as validated content packs/registry with
cross-pack prerequisites, version compatibility and completed-quest history;
current development bounds remain. Legacy playable is preserved.

Multi-quest persistence prerequisite qualified, 2 November 2026:
P3-02 / P3-11 / P3-12 / P3-15 / initial P3-16: [per-quest durable journals](../QUEST_ENGINE.md)
pass four new and thirty legacy owned PostgreSQL contracts (34 total), exact
snapshot/receipt/XP restart checks, atomic completion rollback and a lock barrier.
Events are independent of XP kill recipes and deduplicate per character across
all affected quests. Legacy data is preserved. All 290 core / 641 network (ten
skips) / 123 client / five persistence unit tests, formatting and workspace/
all-target denied-warning Clippy pass. Ordinary Cargo's 34 ignored SQL tests were
executed explicitly in the owned runner. Preserve
`build/quest-journal-store-qualified-v1-20261102` (759 source files). Live World,
owner queues, transport/UI and native adoption remain separate; current playable
is still the schema-11 reset checkpoint. Next: bounded retained owner integration,
then World-authorized semantic events. Concurrent story documents remain planning
material with unresolved production IDs. Commit/push authorized by the user.

Multi-quest journal pure prerequisite qualified, 2 November 2026:
P3-11 / P3-12 / P3-15 / initial P3-16: [bounded multi-quest proposals](../QUEST_ENGINE.md)
pass seven journal checks and all 289 core tests, formatting and workspace/
all-target denied-warning Clippy. Prerequisites unlock on completed turn-in;
indexed kill/semantic interaction events respect stages, versions and independent
quest revisions. One occurrence never cascades to a later stage. Snapshot checks
refuse inconsistent progress. Completion returns a proposal only; durable event
receipts, rewards, World and native adoption remain separate. Preserve
`build/quest-journal-core-qualified-v1-20261102` (755 source files). Next: new
multi-quest SQL ownership/receipt tables, preserving the live legacy quest.
Changes remain local; the threat draft stays archived.

Quest catalogue prerequisite qualified, 2 November 2026:
P3-11 / P3-12 / P3-15 / initial P3-16: resumed with the user's authorization to
expand quests. The [versioned multi-quest catalogue](../QUEST_ENGINE.md) and offline validator
pass seven focused checks, all 282 core tests, formatting and workspace/all-target
denied-warning Clippy. Two development quests cover prerequisite links, ordered
stages and distinct NPC/kill selectors. This is pure offline code, not a new live
journal or persistence migration. Preserve `build/quest-catalog-qualified-v1-20261102`
(753 source files). The schema-11 playable remains preserved; the threat World
draft stays archived. Next: bounded multi-quest progress proposals, then separate
authority/SQL/transport/native acceptance. Changes remain local.

Creature victim-transfer prerequisites qualified, 2 November 2026:
P3-07 / P4-08: [authority-only transfer hooks](../CREATURE_THREAT.md) pass six new
checks and all 275 core tests, formatting and workspace/all-target denied-warning
Clippy. Preserve leash anchor/deadline, cancel an old windup, retain recovery and
refuse invalid/late transfers atomically. These Theia hooks remain inactive in
normal World code. Preserve `build/creature-transfer-core-qualified-v1-20261102`
(748 source files). Next: prepare atomic World threat/pursuit/retaliation and
path-revision updates together. The schema-11 playable below remains qualified.
Changes stay local; no new live switching or network/native claim.

Creature threat arithmetic prerequisite qualified, 2 November 2026:
P3-07 / P4-08: [bounded damage-threat calculation](../CREATURE_THREAT.md) passes ten
focused checks and all 269 core tests, formatting and workspace/all-target
denied-warning Clippy. Empirical 110%/130% thresholds retain explicit Theia
equality/tie/lifetime policies. This module is inactive in the live World; no
new victim-switching/native claim. Preserve
`build/creature-threat-core-qualified-v1-20261102`. The normal schema-11 reset
playable below remains qualified. Next: atomic opt-in World integration with
competing-player and encounter-lifecycle checks. Changes remain local.

Creature return-reset playable qualified, 2 November 2026:
P2-14 / P3-07 / P3-06 / P4-08: [normal schema-11 reset adoption](../CREATURE_RETURN_RESET.md)
binds twenty-two exact-byte roles. 259 core / 641 network (ten skips) / 123 client /
49 Python tests, formatting, workspace/all-target denied-warning Clippy and release
pass. Seven normal OIDC/PostgreSQL native runs pass: passive 35-second melee,
controls/casting at three sizes and return/reacquire/kill at three sizes. Each
return run restores the same NPC from 89 to 100 HP, clears combat, reacquires and
later yields one durable XP grant and one training offer. Exactly one reset and
two acquisitions; no player death/life change. Passive contact has zero returns
and resets. GPU diagnostics and combat-event gaps are zero. Actual 1920x1200,
3440x1440 fullscreen and 1280x720 on Linux RX 7900 XTX / RADV Vulkan / Mesa 26.2.6,
connected display. Preserve `build/creature-reset-client-qualified-v1-20261102`
(744 source files) and earlier checkpoints. Injected status cleanup and two-peer
impaired UDP have separate headless evidence. No native DoT, loot consumption,
party, Windows or MMO-capacity claim. Next: bounded threat/target transfer before
population expansion. Changes are local; entries below are historical.

Creature return-reset transport qualified, 2 November 2026:
P3-07 / P3-06 / P4-08: [encrypted movement/reset observations](../CREATURE_RETURN_RESET.md) pass
at 20/30/60 Hz with two real peers under delay, jitter, loss, duplication and
reordering. Moving away causes return and restores the same NPC to maximum
health, releases combat and preserves the player's life/movement stream. No
false death, respawn or award occurs. Explicit synthetic inventory and low-HP,
zero-avoidance fixtures apply; no OIDC/native/persistence claim is made here.
259 core / 639 network (ten skips) / 123 client tests, formatting and workspace/
all-target denied-warning Clippy pass. Preserve
`build/creature-reset-transport-qualified-v1-20261102` (743 source files).
Next: schema-11 normal bundle and native return/heal acceptance. The schema-10
playable checkpoint remains preserved. Changes are local.

Creature return-reset World qualified, 2 November 2026:
P3-07 / P3-06 / P4-08: [opt-in health/effect reset](../CREATURE_RETURN_RESET.md) passes seven focused
World checks and 259 core / 638 network (ten skips) / 123 client tests, formatting
and workspace/all-target denied-warning Clippy. Living NPC health restores once
on Return/Blocked; target effects clear, membership/tag release and old projectile
contexts remain invalid. Death/respawn is separate. Multi-rate cases use
20/30/60 Hz; status cleanup uses an explicit trusted Registry injection because
the current typed catalogue has no DoT ability. Preserve
`build/creature-reset-world-qualified-v1-20261102`. New-profile encrypted
observation checks come next; no normal reset manifest/client adoption yet.
The schema-10 playable checkpoint below remains preserved. Changes are local.

Solo tag-owner playable checkpoint, 2 November 2026:
P2-14 / P3-04 / P3-09 / P3-10 / P3-11 / P4-08: [normal schema-10 adoption](../CREATURE_CREDIT.md)
binds twenty-one exact-byte roles and authoritative solo tag-owner XP, accepted-
quest and training-offer recipients. 259 core / 631 network (ten skips) / 123
client / 47 Python tests, formatting, workspace/all-target denied-warning Clippy
and release pass. Seven normal OIDC/PostgreSQL native runs pass: controls/casting
and full fights at actual 1920x1200, 3440x1440 fullscreen and 1280x720, plus a
35-second passive check. Full fights have zero returns, one combat entry/exit,
one durable XP commit and one training offer. Passive contact has zero returns
and no outgoing attacks. GPU diagnostics and combat-event gaps are zero.
Linux AMD RX 7900 XTX / RADV Vulkan / Mesa 26.2.6, connected display.
Preserve `build/creature-credit-client-qualified-v1-20261102` and earlier builds.
Two-peer recipient routing and owned PostgreSQL restart/replay are separately
qualified below. Native cases are single-player; loot consumption, parties,
shared eligibility, native Windows and MMO capacity remain unqualified.
Next: bounded encounter threat/reset before population expansion. Changes are
local. Entries below record historical checkpoints and their stated limits.

Solo credit durability qualified, 2 November 2026:
P3-09 / P3-10 / P3-11 / P4-08: [public World plus owned PostgreSQL](../CREATURE_CREDIT.md)
confirms first tag-owner XP 25 / accepted-quest count 1 / one training claim;
the final hitter receives 0 / 0 / 0. SQL locking prevents acknowledgement while
fixed simulation steps advance. Replays do not duplicate; wrong-character replay
refuses. Recipient state and receipts survive PostgreSQL process restart with
fsync/synchronous_commit/full_page_writes on. Explicit synthetic account,
unequipped/no-cost Arcane and trusted quest-acceptance fixtures are used.
Formatting, workspace/all-target denied-warning Clippy, one server test, build
and Python compile pass; 259 core / 628 network (ten skips) / 123 client results
are inherited unchanged. Preserve
`build/creature-credit-durable-qualified-v1-20261102`. No combined OIDC/native or
loot-consumption claim. Next: versioned normal manifest/launcher/client adoption.
Normal manifests still retain last-hit credit. Changes remain local.

Solo-recipient adapters qualified, 2 November 2026:
P3-09 / P3-10 / P3-11 / P4-08: [explicit schema-2 solo tag-owner credit](../CREATURE_CREDIT.md)
passes six focused recipient checks and actual two-peer impaired encrypted UDP
at 20/30/60 Hz. All 259 core / 628 network (ten skips) / 123 client tests,
formatting and workspace/all-target denied-warning Clippy pass. Only the first
effective hitter receives XP/training-offer work; final hitters receive neither.
Queue routing is not COMMIT. Preserve
`build/creature-credit-recipient-qualified-v1-20261102`. Owned PostgreSQL/public
World acceptance follows. Normal manifests retain last-hit credit; party/shared
eligibility and general corpse loot remain separate. Changes are local.

Creature-credit World qualified, 2 November 2026:
P3-09 / P3-10 / P3-11 / P4-08: [opt-in first-effective-hit ownership](../CREATURE_CREDIT.md)
passes twelve focused World checks and 258 core / 621 network (ten skips) /
123 client tests, formatting and workspace/all-target denied-warning Clippy.
Atomic ordinary/typed ability contributions, competing hitters, delayed spell
reset boundaries, disconnect and NPC/player life cases pass. These use explicit
fixtures at 20/30/60 Hz; no new-profile encrypted/durable or native claim.
Preserve `build/creature-credit-world-qualified-v1-20261102`. Normal rewards still
use last-hit credit. Next: versioned solo-recipient adapters and competing-player
durability; party/shared credit and per-corpse loot remain future work.
The normal playable checkpoint below remains preserved. Changes are local.

Creature-credit core prerequisite qualified, 2 November 2026:
P3-09 / P3-10 / P3-11 / P4-08: [bounded first-effective-hit ledger](../CREATURE_CREDIT.md)
passes eight focused checks, all 258 core tests, formatting and workspace/all-target
denied-warning Clippy. Ownership and contribution facts are separate from future
XP/quest/loot eligibility and durability. Exact source/proof:
`build/creature-credit-core-qualified-v1-20261102`. This is inactive pure core;
normal rewards retain last-hit credit. Next: opt-in atomic World integration and
competing-player/reset tests. The playable checkpoint below remains preserved.

Normal combat-membership playable checkpoint, 2 November 2026:
P2-14 / P3-04 / P3-06 / P3-07: [normal PL02 adoption](../COMBAT_MEMBERSHIP.md) adds a
schema-9 twenty-role bundle and an authoritative “In combat” / “Out of combat”
health-panel indicator. 250 core / 609 network (ten skips) / 123 client / 45 Python
checks, formatting, workspace/all-target denied-warning Clippy and release pass.
Seven normal OIDC/PostgreSQL native runs pass: controls/casting and full fights
at actual 1920x1200, 3440x1440 fullscreen and 1280x720, plus 35 seconds of passive
melee. Each fight enters and exits combat, has zero returns, one durable XP commit
and one loot offer. Passive melee has one entry, zero exits and zero returns.
Combat/peace glyphs render; GPU diagnostics and combat-event gaps are zero.
Linux AMD RX 7900 XTX / RADV Vulkan / Mesa 26.2.6, connected display.
Preserve `build/combat-membership-client-qualified-v1-20261102` and earlier builds.
No gameplay permission consumer is gated yet; loot claim, native Windows and MMO
capacity are not qualified. Next: first-effective-hit tag and credit interfaces.
Changes remain local. Entries below are historical checkpoints.

Combat-membership transport qualified, 2 November 2026:
P1-08 / P3-04 / P3-06 / P3-07: [PL02 owner-private life/combat observations](../COMBAT_MEMBERSHIP.md)
pass 250 core / 606 network (ten skips) / 123 client tests, formatting and
workspace/all-target denied-warning Clippy. Six codec tests, two actual encrypted
UDP tests and a critical-queue check cover exact policy/life binding, prediction
barriers, owner isolation, kill release, version mismatch and full/closed queues.
New-mode 20/30/60 Hz impaired two-peer evidence uses explicit low-health and
zero-avoidance special fixtures with synthetic admitted inventory. Preserve
`build/combat-membership-transport-qualified-v1-20261102` and earlier checkpoints.
Normal manifest/launcher/client HUD and OIDC/PostgreSQL/native adoption are next;
no gameplay consumer is gated yet. The normal playable checkpoint below remains.

## P2-12 / P3-06 / P3-07 / P3-09 — qualified ability and engagement slice

Ability damage and melee engagement checkpoint, 2 November 2026:
[Normal ability-damage adoption](../CLASSIC_ABILITY_DAMAGE.md) uses the nineteen-role schema-8 bundle,
CE08, separately resolved Sword Slash and Arcane Bolt, and bounded typed combat
labels/effects. The wolf's 30-second allowance now refreshes on valid visible
melee contact; its captured 35 m hard leash, LOS grace and blocked-path resets
remain. This refresh rule and current coefficients are explicit Theia policies.
243 core / 588 network (ten skips) / 123 client / 43 Python checks, formatting,
workspace/all-target denied-warning Clippy and release pass. Seven normal
OIDC/PostgreSQL native runs pass: controls/casting and complete fights at actual
1920x1200, 3440x1440 fullscreen and 1280x720, plus a 35-second passive melee check.
All fights have zero returns, one typed ability killing hit, one durable XP commit
and one loot offer; the passive check has one acquisition and zero returns.
Zero GPU diagnostics and combat event gaps. Linux AMD RX 7900 XTX / RADV Vulkan,
connected display; no native Windows, loot-claim or MMO-capacity claim.
Preserve `build/classic-ability-client-qualified-v1-20261102` and earlier builds.
Derived gear/class stats, resistance/absorption, ranged leash refresh and full
threat/evade/reset remain. Next: central combat membership and credit interfaces.
Changes remain local. Dated entries below describe earlier checkpoints.

## Ordinary melee playable checkpoint — 1 November 2026

P2-11 / P2-12 / P3-06 / P3-08 / P3-09: normal schema-7 manifest, launcher,
server/client loading, CE07 identity and bounded ordinary combat presentation
are adopted. 224 core / 556 network (ten skips) / 118 client / 41 Python checks,
workspace Clippy, formatting, release and six normal OIDC/PostgreSQL native runs
pass. Preserve `build/classic-white-client-qualified-v1-20261101`.
[Exact behavior and native limits](../CLASSIC_ATTACK_TABLES.md).
Next: ability/spell starting damage and category-specific outcomes, followed by
actual gear/class/level stat derivation. Keep physical/ordinary reference rules
separate from generic ability Hit/Miss/Deflect fixtures. P3-17/18 stay unspecified.
Inactive ability starting recipes now pass 231 core / 556 network (ten skips) /
118 client tests and workspace Clippy. Preserve
`build/classic-ability-starting-qualified-v1-20261101`;
[scope and next acceptance](../CLASSIC_ABILITY_DAMAGE.md). Implement separate spell
landing/conditional critical and main-hand-special policies before live adoption.
[Spell landing/conditional critical](../CLASSIC_SPELL_DAMAGE.md) now passes
235 core tests and workspace Clippy as an inactive numerical prerequisite.
Older sections below record historical prerequisite acceptance.

## Ordinary melee outcomes — World qualified, transport next

P2-11 / P2-12 / P3-06 / P3-09: [ordinary table, rates, amounts and World](../CLASSIC_ATTACK_TABLES.md)
now pass 224 core / 539 network (ten skips) / 115 client tests, formatting and
workspace Clippy. Exact source/proof: `build/classic-white-world-qualified-v1-20261101`.
Keep the normal-physical playable checkpoint below. CE07 and Renet now also pass
224 core / 551 network / 115 client full regression (ten skips), including actual
impaired two-peer UDP, owner privacy/continuity and backpressure. Preserve
`build/classic-white-transport-qualified-v1-20261101`. Implement normal
manifest/launcher/client and native acceptance next. Do not equate the
candidate template/rate policies with full class/gear statistics or every Classic
formula; lower-level critical targets, special/spell tables, control/disarm state,
absorption/resistance and aura modifiers remain explicit gaps. P3-17/18 stay open.

## Shared physical damage checkpoint — 1 November 2026

P2-11 / P2-12 / P3-06 / P3-08 / P3-09 now have normal application and native
physical-result adoption: schema-6 exact-byte manifest, matching server/client
profiles and identity, white outgoing/red incoming damage labels, sword cues and
creature reactions. Preserve `build/classic-damage-client-qualified-v1-20261101`.
189 core / 514 network (ten skips) / 115 client / 39 Python checks, workspace
Clippy, formatting and release pass. Six logged-in native runs at three actual
sizes pass with zero GPU diagnostics; normal wolf fights prove durable XP and
loot offers. The physical template is level 1 and normal-contact-only; abilities
still use their existing damage fixture. Next: category-specific outcome tables
and eligibility, before full stat derivation and spell coefficient adoption.
P3-17/18 remain unspecified. Earlier entries below are historical.

## Classic ability rules checkpoint — 1 November 2026

P2-11 / P2-12 / P3-06 / P3-08 [ability rules](../CLASSIC_ABILITY_RULES.md) now
pass authority, encrypted impaired UDP and normal logged-in native acceptance.
Preserve `build/classic-ability-qualified-v1-20261101`. The initial checked
[damage arithmetic](../CLASSIC_DAMAGE_RESOLUTION.md) passes six new numerical
fixtures and seven prevention/result checks (169 core tests total). Continue
category eligibility and actor-stat binding; the bounded candidate definition
loader now passes seven further tests (176 core total). Continue [shared physical damage authority](../CLASSIC_DAMAGE_AUTHORITY.md):
reference armour/stats now pass thirteen further checks (189 core total). Next
the candidate backend configuration passes nine new profile checks (163 network
library tests). Active World identity and both physical damage directions now
pass twelve World checks at 20/30/60 Hz and full regression (501 network passes).
The explicit CE06 result codec now passes seven checks (183 network-library
tests), strict profile/life/math validation and a 1189-byte maximum batch. Next
Renet selection, per-recipient privacy/continuity and actual impaired UDP now
pass at 20/30/60 Hz, including backpressure and existing kill-fact boundaries
(511 network passes in the full regression). Implement manifest/launcher/server/
client loading and presentation, then logged-in native acceptance before
replacing the playable checkpoint. Keep full attack tables and spell
coefficient adoption as explicit following slices. The strict development prototype demonstrates
thirteen checks; full damage formulas remain pending. P3-17/18 stay unspecified.

## World-system scope refinement — 1 November 2026

Use [the current world-system assessment](../CLASSIC_WORLD_SYSTEMS_REFERENCE.md)
for priority, working policies, actual gaps and acceptance. Complete strict
combat transport/client adoption first. Refine P3-04/06/07/09/10/11/15/16 and
P4-03/07/08/09/10/11 for combat membership, attack tables, first-tag/shared quest
credit, bounded spawns/encounters, resource/control rules and corpse recovery.
Preserve existing development credit/respawn fixtures until replacements pass.
Use competing-player, life/reset, persistence and actual terrain/navigation
checks. Recommendations and research claims are not completed task evidence.
P3-17/P3-18 stay unspecified; actual prerequisites govern overlapping slices.

# Theia MMO — Codex Dependency and Acceptance Register

**Current acceptance amendment, 1 November 2026:** follow
[THEIA-COMBAT-001](../COMBAT_TARGETING_MOVEMENT_SPEC.md) and its
[repository migration slices](../MOVEMENT_CAMERA_TARGET.md) for initial controls
and combat. Earlier hybrid/dodge requirements are deferred. Prior experiment
results do not complete the new strict-WoW milestone. P3-17/18 remain unspecified.

**Status:** proposed engineering task decomposition. All entries default **NOT STARTED / NOT VERIFIED** until repository audit. The voice session approved the phase/wave structure and decisions but did not verify 112 individual implementation tasks. Dependency chains below are conservative initial proposals; Codex must refine per repository and identify safe parallelism. Missing P3-17/18 are intentionally blocked.

## How to execute
For each task: inspect existing code first; write a short design and tests; implement smallest vertical slice; run format/lint/test/build and relevant headless/client integration; capture benchmark evidence; update status only after verification. Preserve ADRs and avoid revisiting locked decisions without explicit approval.

## Phase 1

### P1-01 — Repository audit and reusable/obsolete classification
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** repository audit and architecture baseline.
- **Codex task:** Implement and document repository audit and reusable/obsolete classification using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for repository audit and reusable/obsolete classification; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-02 — Rust workspace, Bevy client, headless server and shared crates
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-01.
- **Codex task:** Implement and document rust workspace, bevy client, headless server and shared crates using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for rust workspace, bevy client, headless server and shared crates; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-03 — World coordinates and spatial conventions
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-02.
- **Codex task:** Implement and document world coordinates and spatial conventions using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for world coordinates and spatial conventions; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-04 — Entity identity and lifecycle
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-03.
- **Codex task:** Implement and document entity identity and lifecycle using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for entity identity and lifecycle; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-05 — 30Hz fixed timestep, bounded input queues
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-04.
- **Codex task:** Implement and document 30hz fixed timestep, bounded input queues using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for 30hz fixed timestep, bounded input queues; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-06 — Command schemas, sequencing, acknowledgements and loss tests
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-05.
- **Codex task:** Implement and document command schemas, sequencing, acknowledgements and loss tests using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for command schemas, sequencing, acknowledgements and loss tests; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-07 — Renet integration and validation (replaces comparative transport study)
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-06.
- **Codex task:** Implement and document renet integration and validation (replaces comparative transport study) using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for renet integration and validation (replaces comparative transport study); automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-08 — Replication protocol contracts
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-07.
- **Codex task:** Implement and document replication protocol contracts using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for replication protocol contracts; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-09 — Kinematic movement and collision contracts
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-08.
- **Codex task:** Implement and document kinematic movement and collision contracts using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for kinematic movement and collision contracts; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-10 — Asset/terrain/animation content contracts
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-09.
- **Codex task:** Implement and document asset/terrain/animation content contracts using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for asset/terrain/animation content contracts; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-11 — Data-driven ability/combat schemas
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-10.
- **Codex task:** Implement and document data-driven ability/combat schemas using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for data-driven ability/combat schemas; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P1-12 — Tests, instrumentation and benchmark baselines
- **Primary wave:** W1; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-11.
- **Codex task:** Implement and document tests, instrumentation and benchmark baselines using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for tests, instrumentation and benchmark baselines; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 2

### P2-01 — Bevy client bootstrap
- **Primary wave:** W2; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P1-05, P1-08, P1-09.
- **Codex task:** Implement and document bevy client bootstrap using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for bevy client bootstrap; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-02 — Connection/session management
- **Primary wave:** W2; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-01, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document connection/session management using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for connection/session management; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-03 — Entity replication and lifecycles
- **Primary wave:** W2; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-02, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document entity replication and lifecycles using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for entity replication and lifecycles; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-04 — Prediction and reconciliation
- **Primary wave:** W2; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-03, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document prediction and reconciliation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for prediction and reconciliation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-05 — Third-person camera/controls
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-04, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document third-person camera/controls using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for third-person camera/controls; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-06 — Terrain chunk loading
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-05, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document terrain chunk loading using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for terrain chunk loading; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-07 — Collision integration
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-06, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document collision integration using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for collision integration; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-08 — GLB and asset loading
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-07, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document glb and asset loading using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for glb and asset loading; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-09 — Character skeletal animation
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-08, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document character skeletal animation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for character skeletal animation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-10 — Lighting/materials/atmosphere
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-09, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document lighting/materials/atmosphere using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for lighting/materials/atmosphere; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-11 — Targeting and interactions
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-10, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document targeting and interactions using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for targeting and interactions; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-12 — One authoritative ability, damage, death and respawn
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-11, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document one authoritative ability, damage, death and respawn using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for one authoritative ability, damage, death and respawn; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-13 — Spell VFX and audio hooks
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-12, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document spell vfx and audio hooks using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for spell vfx and audio hooks; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P2-14 — Health/target/action-bar UI and integrated demo
- **Primary wave:** W3; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-13, P1-05, P1-08, P1-09.
- **Codex task:** Implement and document health/target/action-bar ui and integrated demo using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for health/target/action-bar ui and integrated demo; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 3

### P3-01 — Authentication and sessions
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-12, P1-11.
- **Codex task:** Implement and document authentication and sessions using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for authentication and sessions; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-02 — PostgreSQL persistence/migrations
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-01, P2-12, P1-11.
- **Codex task:** Implement and document postgresql persistence/migrations using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for postgresql persistence/migrations; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-03 — Character creation/selection
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-02, P2-12, P1-11.
- **Codex task:** Implement and document character creation/selection using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for character creation/selection; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-04 — Stats and resources
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-03, P2-12, P1-11.
- **Codex task:** Implement and document stats and resources using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for stats and resources; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-05 — Inventory and items
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-04, P2-12, P1-11.
- **Codex task:** Implement and document inventory and items using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for inventory and items; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-06 — Abilities and status effects
- **Primary wave:** W4; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-05, P2-12, P1-11.
- **Codex task:** Implement and document abilities and status effects using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for abilities and status effects; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-07 — Creature AI
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-06, P2-12, P1-11.
- **Codex task:** Implement and document creature ai using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for creature ai; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-08 — Equipment and appearance
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-07, P2-12, P1-11.
- **Codex task:** Implement and document equipment and appearance using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for equipment and appearance; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-09 — Loot and rewards
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-08, P2-12, P1-11.
- **Codex task:** Implement and document loot and rewards using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for loot and rewards; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-10 — Experience and leveling
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-09, P2-12, P1-11.
- **Codex task:** Implement and document experience and leveling using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for experience and leveling; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-11 — Quest objectives
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-10, P2-12, P1-11.
- **Codex task:** Implement and document quest objectives using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for quest objectives; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-12 — NPC dialogue and vendors
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-11, P2-12, P1-11.
- **Codex task:** Implement and document npc dialogue and vendors using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for npc dialogue and vendors; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-13 — Terrain streaming and initial LOD
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-12, P2-12, P1-11.
- **Codex task:** Implement and document terrain streaming and initial lod using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for terrain streaming and initial lod; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-14 — Animation, VFX and audio expansion
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-13, P2-12, P1-11.
- **Codex task:** Implement and document animation, vfx and audio expansion using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for animation, vfx and audio expansion; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-15 — Content authoring and validation
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-14, P2-12, P1-11.
- **Codex task:** Implement and document content authoring and validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for content authoring and validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-16 — One complete persistent zone and login/logout proof
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-15, P2-12, P1-11.
- **Codex task:** Implement and document one complete persistent zone and login/logout proof using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for one complete persistent zone and login/logout proof; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-17 — UNSPECIFIED in original transcript: needs reconciliation
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-16, P2-12, P1-11.
- **Codex task:** Implement and document unspecified in original transcript: needs reconciliation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** BLOCKED: obtain missing original approved scope; do not implement guessed task.
- **Status:** NOT STARTED / NOT VERIFIED.

### P3-18 — UNSPECIFIED in original transcript: needs reconciliation
- **Primary wave:** W5; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-17, P2-12, P1-11.
- **Codex task:** Implement and document unspecified in original transcript: needs reconciliation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** BLOCKED: obtain missing original approved scope; do not implement guessed task.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 4

### P4-01 — World region ownership
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-16, P1-03.
- **Codex task:** Implement and document world region ownership using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for world region ownership; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-02 — Predictive terrain/object streaming
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-01, P3-16, P1-03.
- **Codex task:** Implement and document predictive terrain/object streaming using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for predictive terrain/object streaming; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-03 — Streamed navigation and spatial queries
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-02, P3-16, P1-03.
- **Codex task:** Implement and document streamed navigation and spatial queries using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for streamed navigation and spatial queries; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-04 — Advanced interest management
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-03, P3-16, P1-03.
- **Codex task:** Implement and document advanced interest management using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for advanced interest management; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-05 — Adaptive replication
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-04, P3-16, P1-03.
- **Codex task:** Implement and document adaptive replication using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for adaptive replication; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-06 — Cross-region handoff
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-05, P3-16, P1-03.
- **Codex task:** Implement and document cross-region handoff using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for cross-region handoff; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-07 — Population and overload management
- **Primary wave:** W6; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-06, P3-16, P1-03.
- **Codex task:** Implement and document population and overload management using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for population and overload management; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-08 — Advanced combat interrupts/CC/AoE/healing/threat
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-07, P3-16, P1-03.
- **Codex task:** Implement and document advanced combat interrupts/cc/aoe/healing/threat using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for advanced combat interrupts/cc/aoe/healing/threat; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-09 — Classes and specializations
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-08, P3-16, P1-03.
- **Codex task:** Implement and document classes and specializations using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for classes and specializations; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-10 — Group encounters
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-09, P3-16, P1-03.
- **Codex task:** Implement and document group encounters using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for group encounters; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-11 — Parties and group rules
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-10, P3-16, P1-03.
- **Codex task:** Implement and document parties and group rules using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for parties and group rules; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-12 — Chat/moderation
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-11, P3-16, P1-03.
- **Codex task:** Implement and document chat/moderation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for chat/moderation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-13 — Friends and presence
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-12, P3-16, P1-03.
- **Codex task:** Implement and document friends and presence using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for friends and presence; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-14 — Guild persistence
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-13, P3-16, P1-03.
- **Codex task:** Implement and document guild persistence using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for guild persistence; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-15 — Secure trading
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-14, P3-16, P1-03.
- **Codex task:** Implement and document secure trading using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for secure trading; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-16 — Animation LOD and crowd policies
- **Primary wave:** W7; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-15, P3-16, P1-03.
- **Codex task:** Implement and document animation lod and crowd policies using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for animation lod and crowd policies; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-17 — VFX density budgets
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-16, P3-16, P1-03.
- **Codex task:** Implement and document vfx density budgets using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for vfx density budgets; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-18 — Time/weather integration
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-17, P3-16, P1-03.
- **Codex task:** Implement and document time/weather integration using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for time/weather integration; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-19 — Multi-zone scale profiling
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-18, P3-16, P1-03.
- **Codex task:** Implement and document multi-zone scale profiling using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for multi-zone scale profiling; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P4-20 — World and multiplayer integration gate
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P4-19, P3-16, P1-03.
- **Codex task:** Implement and document world and multiplayer integration gate using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for world and multiplayer integration gate; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 5

### P5-01 — Rendering benchmarks
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P2-14, P4-07.
- **Codex task:** Implement and document rendering benchmarks using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for rendering benchmarks; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-02 — Lighting and shadows
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-01, P2-14, P4-07.
- **Codex task:** Implement and document lighting and shadows using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for lighting and shadows; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-03 — Atmosphere and fog
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-02, P2-14, P4-07.
- **Codex task:** Implement and document atmosphere and fog using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for atmosphere and fog; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-04 — Stylized materials
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-03, P2-14, P4-07.
- **Codex task:** Implement and document stylized materials using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for stylized materials; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-05 — Water
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-04, P2-14, P4-07.
- **Codex task:** Implement and document water using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for water; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-06 — Terrain LOD and seams
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-05, P2-14, P4-07.
- **Codex task:** Implement and document terrain lod and seams using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for terrain lod and seams; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-07 — Streaming memory budgets
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-06, P2-14, P4-07.
- **Codex task:** Implement and document streaming memory budgets using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for streaming memory budgets; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-08 — Vegetation instancing and wind
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-07, P2-14, P4-07.
- **Codex task:** Implement and document vegetation instancing and wind using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for vegetation instancing and wind; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-09 — Visibility/culling
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-08, P2-14, P4-07.
- **Codex task:** Implement and document visibility/culling using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for visibility/culling; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-10 — Animation blending and layers
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-09, P2-14, P4-07.
- **Codex task:** Implement and document animation blending and layers using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for animation blending and layers; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-11 — Equipment visuals
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-10, P2-14, P4-07.
- **Codex task:** Implement and document equipment visuals using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for equipment visuals; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-12 — Animation LOD
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-11, P2-14, P4-07.
- **Codex task:** Implement and document animation lod using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for animation lod; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-13 — Character readability
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-12, P2-14, P4-07.
- **Codex task:** Implement and document character readability using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for character readability; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-14 — Particle expansion
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-13, P2-14, P4-07.
- **Codex task:** Implement and document particle expansion using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for particle expansion; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-15 — Projectiles/AoE effects
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-14, P2-14, P4-07.
- **Codex task:** Implement and document projectiles/aoe effects using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for projectiles/aoe effects; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-16 — VFX priority and density limits
- **Primary wave:** W8; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-15, P2-14, P4-07.
- **Codex task:** Implement and document vfx priority and density limits using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for vfx priority and density limits; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-17 — World editor
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-16, P2-14, P4-07.
- **Codex task:** Implement and document world editor using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for world editor; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-18 — Creature/item/ability/quest editors
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-17, P2-14, P4-07.
- **Codex task:** Implement and document creature/item/ability/quest editors using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for creature/item/ability/quest editors; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-19 — Asset conversion and license validation
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-18, P2-14, P4-07.
- **Codex task:** Implement and document asset conversion and license validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for asset conversion and license validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-20 — Developer inspection tools
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-19, P2-14, P4-07.
- **Codex task:** Implement and document developer inspection tools using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for developer inspection tools; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-21 — Client performance tuning
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-20, P2-14, P4-07.
- **Codex task:** Implement and document client performance tuning using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for client performance tuning; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-22 — Server profiling
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-21, P2-14, P4-07.
- **Codex task:** Implement and document server profiling using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for server profiling; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-23 — Deployment/monitoring/recovery
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-22, P2-14, P4-07.
- **Codex task:** Implement and document deployment/monitoring/recovery using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for deployment/monitoring/recovery; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P5-24 — Integrated visual and load validation
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P5-23, P2-14, P4-07.
- **Codex task:** Implement and document integrated visual and load validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for integrated visual and load validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## Phase 6

### P6-01 — Release scope and quality gates
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P3-16, P5-24.
- **Codex task:** Implement and document release scope and quality gates using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for release scope and quality gates; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-02 — Minimum playable content
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-01, P3-16, P5-24.
- **Codex task:** Implement and document minimum playable content using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for minimum playable content; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-03 — Internal playtests/defect tracking
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-02, P3-16, P5-24.
- **Codex task:** Implement and document internal playtests/defect tracking using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for internal playtests/defect tracking; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-04 — Class/combat balance
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-03, P3-16, P5-24.
- **Codex task:** Implement and document class/combat balance using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for class/combat balance; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-05 — Progression and economy validation
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-04, P3-16, P5-24.
- **Codex task:** Implement and document progression and economy validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for progression and economy validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-06 — Quest/encounter chains
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-05, P3-16, P5-24.
- **Codex task:** Implement and document quest/encounter chains using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for quest/encounter chains; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-07 — Onboarding/accessibility
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-06, P3-16, P5-24.
- **Codex task:** Implement and document onboarding/accessibility using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for onboarding/accessibility; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-08 — Closed alpha accounts/environment
- **Primary wave:** W9; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-07, P3-16, P5-24.
- **Codex task:** Implement and document closed alpha accounts/environment using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for closed alpha accounts/environment; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-09 — Installer/patcher
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-08, P3-16, P5-24.
- **Codex task:** Implement and document installer/patcher using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for installer/patcher; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-10 — Closed alpha stabilization
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-09, P3-16, P5-24.
- **Codex task:** Implement and document closed alpha stabilization using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for closed alpha stabilization; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-11 — Concurrent gameplay load test
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-10, P3-16, P5-24.
- **Codex task:** Implement and document concurrent gameplay load test using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for concurrent gameplay load test; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-12 — Crowded encounter test
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-11, P3-16, P5-24.
- **Codex task:** Implement and document crowded encounter test using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for crowded encounter test; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-13 — Recovery drills
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-12, P3-16, P5-24.
- **Codex task:** Implement and document recovery drills using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for recovery drills; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-14 — Security/exploit testing
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-13, P3-16, P5-24.
- **Codex task:** Implement and document security/exploit testing using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for security/exploit testing; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-15 — Closed beta population
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-14, P3-16, P5-24.
- **Codex task:** Implement and document closed beta population using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for closed beta population; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-16 — Long soak testing
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-15, P3-16, P5-24.
- **Codex task:** Implement and document long soak testing using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for long soak testing; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-17 — Production deployment/support
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-16, P3-16, P5-24.
- **Codex task:** Implement and document production deployment/support using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for production deployment/support; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-18 — Moderation/support workflows
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-17, P3-16, P5-24.
- **Codex task:** Implement and document moderation/support workflows using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for moderation/support workflows; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-19 — Release candidate validation
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-18, P3-16, P5-24.
- **Codex task:** Implement and document release candidate validation using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for release candidate validation; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-20 — Operational readiness review
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-19, P3-16, P5-24.
- **Codex task:** Implement and document operational readiness review using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for operational readiness review; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-21 — Go/no-go
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-20, P3-16, P5-24.
- **Codex task:** Implement and document go/no-go using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for go/no-go; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-22 — Final load test
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-21, P3-16, P5-24.
- **Codex task:** Implement and document final load test using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for final load test; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-23 — Launch
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-22, P3-16, P5-24.
- **Codex task:** Implement and document launch using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for launch; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

### P6-24 — Launch stabilization
- **Primary wave:** W10; work may begin earlier when prerequisites permit.
- **Dependencies (initial):** P6-23, P3-16, P5-24.
- **Codex task:** Implement and document launch stabilization using approved architecture; add automated tests and observability; avoid blocking simulation or leaking renderer dependencies into headless server.
- **Acceptance:** Implementation demonstrated for launch stabilization; automated correctness/error-path test; metrics or logs captured; integration with authoritative server and relevant Bevy client verified; documentation updated.
- **Status:** NOT STARTED / NOT VERIFIED.

## High-priority cross-cutting acceptance suites
1. Fixed 30Hz scheduling under load; bounded queues, ordering, 20/30/60Hz behaviour.
2. Renet encrypted connection, auth ticket expiry/replay, loss/reorder/reconnect, bandwidth budgets.
3. f64 global/f32 local precision, origin shift, seam collision, long-distance flying.
4. Avian kinematic slopes/jumps/swim/fly and scripted elevator boarding.
5. Chunk streaming and LOD cracks, predictive flight loading, memory pressure, visual/collision separation.
6. Character run-and-cast layering, mounts, equipment, animation LOD.
7. Boss critical telegraphs and audio retained at lowest quality under saturated effects.
8. Durable economic transactions, idempotent retries, crash injection and backup restores.
9. Spatial interest correctness, delta baseline recovery, 200+ crowded-scene benchmark (target workload, not capacity claim).
10. Multi-region ownership uniqueness, cross-boundary interactions and future handoff contract.

## Known reconciliation issues
- Phase 3 was called 18 stages but only 16 stage descriptions were spoken; P3-17/18 are unknown.
- Phase 4–6 descriptions were thematic, so per-stage numbering here is newly proposed.
- Earlier shorthand mapped Phase 3 inventory to stage 9, but detailed enumeration puts inventory at P3-05 and loot at P3-09. Detailed enumeration takes precedence pending review.
- A spoken total of 88 for phases 1–5 conflicts with 12+14+18+20+24=88 (correct), plus 24 =112.
- Original 'M1-04' scheduler task ID differs from this document's P1-05 due to differing milestone schemes. Resolve before creating issue IDs.

## User-requested follow-on refinements — 2026-10-31

The user requested perception/chase/leash, range-constrained player/enemy melee
and magic, travelling magic/arrow projectiles, hit/miss/deflect outcomes, floating
quest markers, a minimap and an improved HUD using the local Synty HUD sources
where useful. [Concrete existing-task slices and acceptance](../COMBAT_WORLD_FOLLOWUPS.md)
refine P2-12/14, P3-06/07/11/12/13/14, P4-03/08 and P5-13/15/19. Continue the
current grip slice first; imported list order is not a rigid dependency gate.
These requests do not assign guessed scopes to P3-17/P3-18.
