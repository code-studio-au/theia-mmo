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
[Exact scope and resume work](QUEST_ITEM_OBJECTIVES.md).
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
[Exact evidence and remaining native work](QUEST_ITEM_OBJECTIVES.md).
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
[Behavior, limits and resume work](QUEST_ITEM_OBJECTIVES.md).
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
[Exact evidence and pending live work](QUEST_ITEM_OBJECTIVES.md).
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
[Behavior, evidence and pending live work](QUEST_ITEM_OBJECTIVES.md).
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
[User semantics, pinned Classic-emulator evidence and remaining work](QUEST_ITEM_OBJECTIVES.md).
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
[Exact semantics, proof and limits](QUEST_REACH_AREAS.md).
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
[Evidence and remaining normal-launch/native work](QUEST_REACH_AREAS.md).
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
[Exact proof and pending owner/native integration](QUEST_REACH_AREAS.md).
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
[Evidence and remaining durable/live work](QUEST_REACH_AREAS.md).
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
[Semantics and pending live integration](QUEST_REACH_AREAS.md).
Normal World/manifest admission explicitly refuses unsupported area objectives.
No live area credit, runtime pack loading, migration, new GPU or Windows claim.
Normal playable remains directional v1; all changes remain local.

Reach-area geometry prerequisite qualified locally, 3 November 2026:
P3-11 / P3-12 / P3-15 / initial P3-16: strict finite XYZ volumes use checked
global metres, inclusive faces and bounded inputs; no renderer/physics/streaming
is created. All 310 core tests, six actual CLI cases, formatting and strict
workspace/all-target Clippy pass. Preserve
`build/quest-reach-area-geometry-qualified-v1-20261103`.
[Exact semantics, evidence and pending stage-bound/live work](QUEST_REACH_AREAS.md).
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
[Exact art/mapping/fixture scope, proof and remaining work](CLASSIC_DIRECTIONAL_LOCOMOTION.md).
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
[Asset proof and pending runtime adoption](CLASSIC_DIRECTIONAL_LOCOMOTION.md).
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
[Exact mapping, source uncertainty, compatibility and evidence](CLASSIC_GROUND_MOVEMENT.md).
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
[Exact normal/accelerated scopes and measurement corrections](MINIMAP.md).
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
[Exact Classic scope, measurements and remaining work](MINIMAP.md). Local only;
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
[Implementation, exact evidence and loss limitations](RENET_CONNECTION_CONFIRMATION.md).
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
baseline. [Workflow and limits](QUEST_PACK_AUTHORING.md). Local only.

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
Older bundles remain unchanged. [Exact scope and endpoint limitation](PUBLIC_AVATAR_LIFE.md).
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
[Scope, replay proof and same-endpoint limitation](PUBLIC_AVATAR_LIFE.md).
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
public-NPC schema 14, unchanged. [Scope](PUBLIC_AVATAR_LIFE.md). Local only; no
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
unchanged. [Exact scope and evidence](PUBLIC_CREATURE_ATTACKS.md). Synthetic
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
adoption remains next; threat v2 playable is unchanged. [Scope](PUBLIC_CREATURE_ATTACKS.md).
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
normal threat v2 playable is unchanged. [Scope](PUBLIC_CREATURE_ATTACKS.md).
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
`build/quest-compile-qualified-v1-20261103`. [Workflow](QUEST_PACK_AUTHORING.md).
No runtime pack loader, history expansion, automatic migration or normal content
replacement. Latest normal playable remains threat v2; changes stay local.

Offline quest pack/index workflow qualified, 3 November 2026:
P3-11 / P3-15: strict contained JSON index, bounded regular-file reads, global
IDs/prerequisites and exact hash report. Typed diagnostics identify files/IDs;
no narrative dump or source mutation. Seven core registry / seven file checks,
all 297 core and 44 existing gameplay-validator checks, formatting and Clippy
pass. Actual compiled CLI success/refusal and read-only hashes pass. Both example
validators are configured in Linux/Windows CI; only local Linux execution is
claimed. [Authoring workflow](QUEST_PACK_AUTHORING.md). Preserve
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
[Scope and evidence](CREATURE_THREAT.md).

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
The [expanded quest action/router and optional-XP kill Store](QUEST_ENGINE.md)
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
P1-06 / P1-08 / P3-11 / P3-12: [QK01/QP01 action boundaries](QUEST_ENGINE.md)
use 110-byte requests and 94-byte replies, carrying explicit actor/NPC/profile,
original intent and painted quest revision. Eight codec and eight focused World/
router checks pass, formatting and workspace/all-target denied-warning Clippy.
One pending command shares the event producer sequence; success waits for a
matching committed owner result. Channel tests use synthetic owner views; normal
UDP/client adoption remains open. Before replacement, preserve configured kill XP
and journal-event atomicity while retaining quest-only events without XP recipes.

Compact quest observation prerequisite qualified, 2 November 2026:
P1-08 / P3-11 / P3-15: [QJ01 journal snapshots](QUEST_ENGINE.md) encode the
current full development maximum in 925 bytes, retaining the existing 8-KiB
reliable channel budget. Five codec checks, all 290 core tests, formatting and
workspace/all-target denied-warning Clippy pass. Typed reconstruction validates
logical state; successor checks preserve completion/history and reject rollback.
Actual OIDC/ticket/SQL owner tests also confirm later events preserve the latest
command reply through logout. This is an unadopted snapshot codec, not new client
journal UI or live request support. Next: action codec and runtime/client adoption.

Explicit quest routing adapter qualified, 2 November 2026:
P3-02 / P3-11 / P3-12: [World-to-owner reservation](QUEST_ENGINE.md) refuses
full/closed/uninitialized owners before a semantic choice changes dialogue.
Kill routing reserves the whole recipient batch before sending and preserves
original IDs/sequence on refusal. Eight focused World/channel checks, all 290
core tests, formatting and workspace/all-target denied-warning Clippy pass.
Actual OIDC/ticket/SQL retirement is repeated and verifies Store/network exact
catalogue identity agreement. World routing tests use explicit synthetic owner
views, separate from the actual durable-owner proof. Normal runtime request
adoption, private journal codec/UI and combined native acceptance remain open.

Expanded quest World adapters qualified, 2 November 2026:
P3-11 / P3-12 / P3-15: [installed references and trusted occurrences](QUEST_ENGINE.md)
pass seven new focused checks, including 20/30/60-Hz semantic choices and actual
competing-player kill credit without XP configuration. Missing contacts/actions/
creatures refuse; stale/out-of-range/foreign-World input yields no occurrence.
6 existing core dialogue and 7 existing World dialogue checks also pass;
formatting and workspace/all-target denied-warning Clippy pass. This explicit API
is not routed through the normal runtime yet. Retain the qualified owner and
schema-11 playable. Next: bounded World-to-owner routing and private journal
transport/UI, with separate durable/encrypted/native acceptance.

Multi-quest retained owner qualified, 2 November 2026:
P3-02 / P3-11 / P3-12 / initial P3-16: [expanded journal ownership](QUEST_ENGINE.md)
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
P3-02 / P3-11 / P3-12 / initial P3-16: [bounded journal jobs](QUEST_ENGINE.md)
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
P3-02 / P3-11 / P3-12 / P3-15 / initial P3-16: [per-quest durable journals](QUEST_ENGINE.md)
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
P3-11 / P3-12 / P3-15 / initial P3-16: [bounded multi-quest proposals](QUEST_ENGINE.md)
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
expand quests. The [versioned multi-quest catalogue](QUEST_ENGINE.md) and offline validator
pass seven focused checks, all 282 core tests, formatting and workspace/all-target
denied-warning Clippy. Two development quests cover prerequisite links, ordered
stages and distinct NPC/kill selectors. This is pure offline code, not a new live
journal or persistence migration. Preserve `build/quest-catalog-qualified-v1-20261102`
(753 source files). The schema-11 playable remains preserved; the threat World
draft stays archived. Next: bounded multi-quest progress proposals, then separate
authority/SQL/transport/native acceptance. Changes remain local.

Creature victim-transfer prerequisites qualified, 2 November 2026:
P3-07 / P4-08: [authority-only transfer hooks](CREATURE_THREAT.md) pass six new
checks and all 275 core tests, formatting and workspace/all-target denied-warning
Clippy. Preserve leash anchor/deadline, cancel an old windup, retain recovery and
refuse invalid/late transfers atomically. These Theia hooks remain inactive in
normal World code. Preserve `build/creature-transfer-core-qualified-v1-20261102`
(748 source files). Next: prepare atomic World threat/pursuit/retaliation and
path-revision updates together. The schema-11 playable below remains qualified.
Changes stay local; no new live switching or network/native claim.

Creature threat arithmetic prerequisite qualified, 2 November 2026:
P3-07 / P4-08: [bounded damage-threat calculation](CREATURE_THREAT.md) passes ten
focused checks and all 269 core tests, formatting and workspace/all-target
denied-warning Clippy. Empirical 110%/130% thresholds retain explicit Theia
equality/tie/lifetime policies. This module is inactive in the live World; no
new victim-switching/native claim. Preserve
`build/creature-threat-core-qualified-v1-20261102`. The normal schema-11 reset
playable below remains qualified. Next: atomic opt-in World integration with
competing-player and encounter-lifecycle checks. Changes remain local.

Creature return-reset playable qualified, 2 November 2026:
P2-14 / P3-07 / P3-06 / P4-08: [normal schema-11 reset adoption](CREATURE_RETURN_RESET.md)
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
P3-07 / P3-06 / P4-08: [encrypted movement/reset observations](CREATURE_RETURN_RESET.md) pass
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
P3-07 / P3-06 / P4-08: [opt-in health/effect reset](CREATURE_RETURN_RESET.md) passes seven focused
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
P2-14 / P3-04 / P3-09 / P3-10 / P3-11 / P4-08: [normal schema-10 adoption](CREATURE_CREDIT.md)
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
P3-09 / P3-10 / P3-11 / P4-08: [public World plus owned PostgreSQL](CREATURE_CREDIT.md)
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
P3-09 / P3-10 / P3-11 / P4-08: [explicit schema-2 solo tag-owner credit](CREATURE_CREDIT.md)
passes six focused recipient checks and actual two-peer impaired encrypted UDP
at 20/30/60 Hz. All 259 core / 628 network (ten skips) / 123 client tests,
formatting and workspace/all-target denied-warning Clippy pass. Only the first
effective hitter receives XP/training-offer work; final hitters receive neither.
Queue routing is not COMMIT. Preserve
`build/creature-credit-recipient-qualified-v1-20261102`. Owned PostgreSQL/public
World acceptance follows. Normal manifests retain last-hit credit; party/shared
eligibility and general corpse loot remain separate. Changes are local.

Creature-credit World qualified, 2 November 2026:
P3-09 / P3-10 / P3-11 / P4-08: [opt-in first-effective-hit ownership](CREATURE_CREDIT.md)
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
P3-09 / P3-10 / P3-11 / P4-08: [bounded first-effective-hit ledger](CREATURE_CREDIT.md)
passes eight focused checks, all 258 core tests, formatting and workspace/all-target
denied-warning Clippy. Ownership and contribution facts are separate from future
XP/quest/loot eligibility and durability. Exact source/proof:
`build/creature-credit-core-qualified-v1-20261102`. This is inactive pure core;
normal rewards retain last-hit credit. Next: opt-in atomic World integration and
competing-player/reset tests. The playable checkpoint below remains preserved.

Normal combat-membership playable checkpoint, 2 November 2026:
P2-14 / P3-04 / P3-06 / P3-07: [normal PL02 adoption](COMBAT_MEMBERSHIP.md) adds a
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
P1-08 / P3-04 / P3-06 / P3-07: [PL02 owner-private life/combat observations](COMBAT_MEMBERSHIP.md)
pass 250 core / 606 network (ten skips) / 123 client tests, formatting and
workspace/all-target denied-warning Clippy. Six codec tests, two actual encrypted
UDP tests and a critical-queue check cover exact policy/life binding, prediction
barriers, owner isolation, kill release, version mismatch and full/closed queues.
New-mode 20/30/60 Hz impaired two-peer evidence uses explicit low-health and
zero-avoidance special fixtures with synthetic admitted inventory. Preserve
`build/combat-membership-transport-qualified-v1-20261102` and earlier checkpoints.
Normal manifest/launcher/client HUD and OIDC/PostgreSQL/native adoption are next;
no gameplay consumer is gated yet. The normal playable checkpoint below remains.

Ability damage and melee engagement checkpoint, 2 November 2026:
[Normal ability-damage adoption](CLASSIC_ABILITY_DAMAGE.md) uses the nineteen-role schema-8 bundle,
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

Ordinary melee playable checkpoint, 1 November 2026:
[Ordinary attack tables and normal client adoption](CLASSIC_ATTACK_TABLES.md)
pass 224 core / 556 network (ten skips) / 118 client / 41 Python checks,
workspace denied-warning Clippy, formatting and release. Six normal OIDC/PostgreSQL
native runs pass at actual 1920x1200, 3440x1440 and 1280x720, including controls,
casting, unmodified wolf perception/chase/attacks, kill and durable XP/loot offers.
Zero GPU diagnostics and combat event gaps. Linux AMD RX 7900 XTX / RADV Vulkan;
connected-display ultrawide fullscreen passed, other sizes windowed.
Preserve `build/classic-white-client-qualified-v1-20261101` and earlier checkpoints.
Native runs show normal hits, misses, dodges and criticals; level-1 wolf data does
not exercise block, parry, glancing or crushing. Those have numerical/World/codec
coverage. Static templates remain: derived gear/class stats, ability/spell starting
damage, special/spell tables and absorption/resistance are future work.
No loot-claim, Windows execution or MMO-capacity claim. Changes remain local.
Older dated entries below record previous checkpoints, not current pending work.

Inactive [ability starting-damage prerequisite](CLASSIC_ABILITY_DAMAGE.md):
231 core / 556 network (ten skips) / 118 client tests and workspace Clippy pass.
Explicit AP-only normalization, scale/flat ordering and bounded candidate recipes
are qualified; normal abilities still use fixed effects. Separate [spell landing and conditional critical](CLASSIC_SPELL_DAMAGE.md)
now pass 235 core tests and workspace Clippy; network/client baselines are inherited.
Main-hand-special outcomes and explicit mitigation remain before authority adoption.

Shared physical damage playable checkpoint, 1 November 2026:
[Normal physical damage authority and client adoption](CLASSIC_DAMAGE_AUTHORITY.md)
pass 189 core / 514 network (ten skips) / 115 client / 39 Python checks,
workspace Clippy, formatting, release and six normal OIDC/PostgreSQL native runs.
All three actual sizes (1920x1200, 3440x1440, 1280x720) pass controls/casting and
unmodified wolf fights with zero GPU diagnostics. Native server summaries confirm
physical hits both ways, one durable XP award and one loot offer per kill; claiming
loot was not tested in this slice. Exact source/art/binaries/proof:
`build/classic-damage-client-qualified-v1-20261101`. Earlier checkpoints remain.
Static templates and normal-contact physical reference arithmetic are implemented;
full attack tables, derived gear/class stats and spell coefficients remain next.
Linux AMD RX 7900 XTX / RADV Vulkan, disconnected-display uncapped/windowed checks;
no connected-monitor fullscreen, native Windows or MMO-capacity claim.
Changes remain local and unpushed. Older dated entries below are historical.

Attack-table prerequisites: [ordinary melee table and eligibility](CLASSIC_ATTACK_TABLES.md)
and authoritative creature facing pass 202 core / 516 network / 115 client tests
and workspace Clippy. Explicit outcome-aware World activation now also passes
224 core / 539 network (ten skips) / 115 client tests and workspace Clippy;
`build/classic-white-world-qualified-v1-20261101`. New-profile codec, UDP and
normal/native adoption remain pending. Actual CE07 encrypted two-peer UDP now
passes full regression: 224 core / 551 network (ten skips) / 115 client tests and
workspace Clippy; `build/classic-white-transport-qualified-v1-20261101`. The playable checkpoint above
remains unchanged.

Classic ability rules, 1 November 2026:
[Manual facing, cast interruption and weapon-timer rules](CLASSIC_ABILITY_RULES.md)
pass 156 core / 479 network (ten skips) / 114 client / 37 Python checks,
workspace Clippy and six normal OIDC/PostgreSQL native runs at actual three sizes.
All have zero GPU diagnostics; disconnected-display uncapped/windowed conditions
are explicit. The strict development profile now demonstrates thirteen checks;
full Classic damage formulas remain pending. Current source/art/binaries/proof:
`build/classic-ability-qualified-v1-20261101`. Next: [damage resolution](CLASSIC_DAMAGE_RESOLUTION.md).
Changes remain local and unpushed.

Damage arithmetic follow-on: [starting damage and provisional prevention](CLASSIC_DAMAGE_RESOLUTION.md)
and candidate definition loading pass 176 core tests and workspace Clippy.
Candidate source/data/proof: `build/classic-damage-catalog-qualified-v1-20261101`.
Physical reference arithmetic and static stat-template prerequisites now also
pass (189 core tests); source/data/proof:
`build/classic-damage-physical-stats-qualified-v1-20261101`.
Explicit physical World and encrypted Renet now pass 189 core / 511 network passes
(ten skips) / 114 client tests and workspace Clippy. New-profile impaired two-peer
UDP verifies both damage directions, owner privacy and reliable continuity at
20/30/60 Hz; backpressure and kill-fact boundaries pass. Source/proof:
`build/classic-damage-transport-qualified-v1-20261101`. The normal playable
checkpoint above remains unchanged. Manifest/launcher/client/presentation/native
adoption is next; full attack tables, derived stats and spell coefficients follow.

Basic-melee transport, 1 November 2026:
[Explicit Renet engagement/state/events](CLASSIC_MELEE_TRANSPORT.md) now
pass focused encrypted UDP checks at 20/30/60 Hz under loss, jitter and
duplication, plus presentation-queue backpressure. Formatting, workspace Clippy,
153 core / 464 network passes (ten skips) / 113 client tests pass. Exact source
and proof: `build/classic-melee-transport-qualified-v1-20261101`.
Normal application right-click/animation adoption remains pending;
playable checkpoint and strict milestone remain unchanged. Local and unpushed.

Broader world-system direction, 1 November 2026:
[Classic systems, repo assessment and working defaults](CLASSIC_WORLD_SYSTEMS_REFERENCE.md)
now cover combat state/resolution, tagging/credit, spawns, linked encounters,
resources, control effects, navigation and corpse recovery. These are scoped
plans and policy choices; the assessment separates them from implemented code.
Finish the current strict combat slice before expanding full-zone content.
Current [headless melee authority](CLASSIC_AUTOATTACK_AUTHORITY.md) passes
153 core / 458 network / 113 client tests; secure transport/client adoption is
next. The playable targeting checkpoint and five-of-thirteen milestone remain.
Changes stay local and unpushed.

Headless basic-melee authority, 1 November 2026:
[World weapon-timed combat](CLASSIC_AUTOATTACK_AUTHORITY.md) passes
153 core tests, 458 network passes / ten skips, 113 client tests and workspace
Clippy. Thirteen new checks prove committed gear, range/facing/LOS/readiness
pause/resume, casts/GCD, removal, retaliation/death/respawn, stale request streams,
reward/XP facts and work/counter limits. Normal transport explicitly refuses this
unadopted profile; next is secure engage/cancel/state and client right-click
adoption. Playable checkpoint: `build/classic-targeting-qualified-v2-20261101`.
Full strict combat acceptance remains five of thirteen. Headless source/proof:
`build/classic-autoattack-authority-qualified-v1-20261101`.
Changes remain local and unpushed.

Headless combat prerequisite, 1 November 2026:
[weapon data and independent swing timing](CLASSIC_MELEE_CLOCK.md) pass
153 core tests and workspace Clippy. Seven new checks cover timing, pause/resume,
retarget/re-engage abuse and overflow. These primitives are not enabled by the
normal executables yet. The playable build remains
`build/classic-targeting-qualified-v2-20261101`, with five of thirteen strict
combat checks. Next: authoritative World integration, then engage/cancel
transport and right-click autoattack. Metric units and deferred dodge remain.
All changes stay local and unpushed.

Targeting HUD follow-up, 1 November 2026: shortened the cycle hint to prevent
an overlap with the action bar at 1280×720. The final binary passes 113 client
tests, workspace Clippy, formatting and a fresh normal native 720p controls/
targeting run with zero GPU diagnostics; its capture is visually inspected.
Shared core/network sources are byte-identical to the qualified 146/445 result.
The preceding checkpoint retains the three-size feature checks. Current source,
binaries and proof: `build/classic-targeting-qualified-v2-20261101`.
Weapon-timed autoattack is next. Changes remain local and unpushed.

Current classic targeting checkpoint, 1 November 2026:
[persistent selection and forward/reverse Tab](CLASSIC_TARGETING.md) pass
146 core tests, 445 network passes / ten skips, 113 client tests, 36 Python checks,
workspace Clippy and three final normal native starts with zero GPU diagnostics.
Selection survives interest/terrain unloading and turning; explicit removal and
clear remain authoritative. The classic profile identity advances to v2.
The strict combat milestone is five of thirteen checks; next is
[weapon-timed melee autoattack](CLASSIC_AUTOATTACK_PLAN.md), then ability
facing/interaction rules. Metric scale remains unchanged. Dodge stays deferred.
Qualified source/binaries/proof: `build/classic-targeting-qualified-v1-20261101`.
Changes remain local, uncommitted and unpushed. Earlier entries retain their
historical scope; they do not supersede this checkpoint.

Formula direction: [Classic fidelity with metric units](CLASSIC_FORMULAS.md)
is adopted. Existing development values require sourced replacements; the world
stays at 1 unit = 1 metre. This does not change frozen checkpoint evidence.

Current classic development checkpoint, 1 November 2026:
[classic controls/manual facing](CLASSIC_MOVEMENT.md) pass 111 client and
45 Python checks, workspace Clippy and seven final native starts with zero GPU
diagnostics. The unchanged shared sources retain 142 core / 444 network passes.
Use `--classic`; dodge/action camera are disabled there. Movement/UI checks pass
at all three sizes, including an A/D-strafe settings file at 720p. Next: persistent
targeting and forward/reverse Tab, then weapon-timed autoattack and combat-facing
validation. [Enemy AI reference](ENEMY_AI_REFERENCE.md) is recorded for
subsequent threat/evade work. The full combat milestone remains incomplete.
Changes remain local, uncommitted and unpushed.

# Current direction — strict WoW Classic foundation, 1 November 2026

The user adopted [THEIA-COMBAT-001 v1.0](COMBAT_TARGETING_MOVEMENT_SPEC.md),
superseding the hybrid camera/movement plan. Initial controls require
character-relative movement, manual authoritative facing, persistent explicit
targeting and weapon-timed autoattack independent of abilities. Dodge, action
camera and reticle targeting are deferred. Dodge implementation remains stopped.

The [repository gap assessment and migration order](MOVEMENT_CAMERA_TARGET.md)
start with the combat-loop contract, then classic controls/facing, persistent
selection, melee autoattack and the thirteen-check one-hostile milestone.
The `--classic` profile now supplies qualified manual movement/camera controls;
persistent selection and weapon-timed combat remain before strict-prototype acceptance. Existing movement/network/terrain, NPC, abilities,
equipment and UI foundations are retained. Earlier checkpoints below record
experimental behavior and evidence; their old “next” steps are superseded.
The local dodge animation candidate is preserved with incomplete native checks.
Changes remain local, uncommitted and unpushed.

# Retained experimental checkpoint — 2026-11-01: predicted dodge

P2-04 / P2-05 / P2-07 / P3-06 / initial P3-14: V requests one server-owned,
collision-swept dodge in either camera mode, with bounded expiring input,
profile-bound correction replay and HUD cooldown feedback. The sealed schema-4
bundle enables the opt-in normal client/server path. **100 client tests**,
**436 network passes / 10 explicit skips / 66 groups**, **43 Python checks**,
formatting and workspace/all-target denied-warning Clippy pass.
All **seven final native starts have zero GPU diagnostics**, including actual
OIDC/PostgreSQL OS input at 1280×720, 1920×1200 and 3440×1440 fullscreen, plus
two-client melee/projectile regressions. Stationary dodge measures 3 m in both
camera modes at each size; panels and held-key/cooldown checks pass.
[Controls, proof and limits](DODGE_MOVEMENT.md).
Exact source/binaries/art/proof: `build/dodge-qualified-v1-20261101`.
Earlier checkpoints remain. No authored roll, invulnerability/endurance system,
new FPS/capacity, full-workspace or Windows claim. Next: authored movement
presentation and facing/tuning. Changes remain local and unpushed.

# Headless checkpoint — 2026-11-01: dodge driver and expiring input

P2-04 / P2-05 / P2-07: the shared prediction driver selects profile-bound
completed-tick dodge replay, with an input edge that expires after 200 ms or
acknowledgement and clears on focus/panel/readiness loss, disconnect or changed
actor/stream. **41 targeted checks / six groups**, formatting and workspace/
all-target denied-warning Clippy pass. Actual encrypted impaired-UDP tests carry
one expiring tap through the driver and authority at 20/30/60 Hz.
[Evidence and remaining normal-client gates](DODGE_MOVEMENT.md).
Exact headless source/proof: `build/dodge-driver-qualified-v1-20261101`.
The prior full-network result remains 427 passes / 10 explicit skips / 65 groups;
this is a targeted rerun. Normal gameplay bundles and client controls still do
not enable dodge. Playable camera: `build/camera-panels-qualified-v1-20261101`.
Next: normal client/server configuration, gameplay bundle and native controls.
Changes remain local and unpushed; no new GPU/FPS/capacity or Windows claim.

# Headless checkpoint — 2026-11-01: dodge authority and encrypted UDP

P2-04 / P2-07 / P3-06: opt-in World dodge authority and matching TC02/TP02
Renet selection pass **427 network tests / 10 explicit skips / 65 groups**,
formatting and workspace/all-target denied-warning Clippy. Eleven new checks
cover actual authority/prediction agreement at 20/30/60 Hz, readiness/collision
loss, teleport, death/respawn, stationary casts, and real encrypted impaired UDP.
[Behavior, evidence and remaining gates](DODGE_MOVEMENT.md).
Exact headless source/proof: `build/dodge-authority-qualified-v1-20261101`.
The normal gameplay client/launcher still does not enable dodge; the playable
camera checkpoint remains `build/camera-panels-qualified-v1-20261101`.
Next: bind the live prediction driver and bounded client input edges, then
normal gameplay adoption and native checks. No new GPU/FPS/capacity, Windows
or whole-workspace test claim. Changes remain local and unpushed.

# Current playable checkpoint — 2026-11-01: camera panels and HUD

P2-05 / P2-14: Escape closes dialogue, then backpack, before quitting;
classic world dragging works around open panels and HUD hints reflect the mode.
**99 client tests**, **416 network passes / 10 explicit skips / 64 groups**,
**41 Python checks**, formatting and workspace/all-target denied-warning Clippy
pass. All **three final native OS-input starts** at 1280×720, 1920×1200 and
3440×1440 fullscreen have **zero GPU diagnostics** on Linux RX 7900 XTX / RADV
NAVI31, Vulkan, Mesa 26.2.6 kisak. Exact source/binaries/art/proof:
`build/camera-panels-qualified-v1-20261101`.
[Controls, evidence and limits](PLAYER_CONTROLS.md).
The previous camera checkpoint separately retains actual two-client melee and
projectile proof; those starts were not rerun against this panel revision.
Shared dodge replay is headless-qualified; live dodge remains uninstalled.
No new full-workspace, sustained FPS/capacity, Windows or physical-input claim.
Changes stay local and unpushed. Historical entries below retain their dates.

# Current player-control priority — 2026-11-01

P2-04/05/07/09/11, P3-06/14: checked dodge edges/codecs and the first optional
action-camera slice pass. Next: Escape/panel priority, classic world dragging
around UI, control hints, shared dodge prediction/authority and combined
movement/facing/animation/latency tuning.
[Adopted WoW/GW2 feel and acceptance](MOVEMENT_CAMERA_TARGET.md).

# Current playable checkpoint — 2026-11-01: mouse controls and quest markers

P2-05 / P2-11 / P2-12 / P3-06 / P3-11 / P3-12 / initial P3-14:
mouse enemy/friendly selection, right-click basic sword/NPC interaction, F and
five ability slots now accompany committed floating quest markers. Standard Bevy
picking/UI, bounded hierarchy/intent handling and static occlusion preserve server
validation. Camera dragging waits for the click threshold and respects UI ownership.
**91 client tests**, formatting, denied-warning workspace/all-target Clippy and
**41 Python tests** pass. **14 final native starts have zero GPU diagnostics**,
including synthetic X11 OS input at 1280×720, 1920×1200 and 3440×1440 fullscreen,
actual OIDC/PostgreSQL/rejoin/restart, mouse melee and travelling-magic regressions.
[Behavior, exact evidence and limits](PLAYER_CONTROLS.md). Matched source/binaries/art/proof:
`build/world-input-qualified-v1-20261101`. The previous whole-workspace result
remains 608 passes / 40 explicit skips; it is not a new whole-workspace run.
Earlier checkpoints remain preserved. No new FPS/capacity or Windows claim.
Dodge is the next separate authoritative movement slice. Changes remain local;
no commit or push was made.

# Current work — 2026-11-01

Development resumed at the user’s request. Qualify [floating guide quest markers](QUEST_MARKERS.md)
(P3-11 / P3-12 / initial P3-14), then the user’s mouse selection, basic attack,
context interaction and five-slot controls. [Accepted scope and acceptance](COMBAT_WORLD_FOLLOWUPS.md).
Dodge follows as a separate authoritative movement slice. Keep the projectile
checkpoint available; changes remain local and unpushed.

# Current playable checkpoint — 2026-10-31: travelling magic

P3-06 / initial P5-15: wand-gated homing magic has finite speed/range/lifetime,
static collision and server damage/hit outcomes only at impact. Standard Bevy
shows the travelling bolt. **608 workspace passes / 40 explicit fixture skips in
115 groups**, formatting, denied-warning workspace/all-target Clippy and
**41 Python tests** pass. Real OIDC/PostgreSQL/two-client hit/miss/deflect travel,
legacy melee/miss-feedback and 3440×1440 fullscreen launcher fixture cleanup pass.
All **12 final native starts have zero GPU diagnostics**.
[Behavior, exact evidence, launch and limits](PROJECTILE_TRAVEL.md).
Matched source/binaries/art/proof: `build/projectile-qualified-v1-20261031`.
No new FPS, Windows, crowd or streaming-capacity claim. Earlier checkpoints remain.
Work stopped at the user's request after this slice; all changes remain local,
uncommitted and unpushed. Remaining requested features stay on the working list.

# Qualified playable checkpoint — 2026-10-31: hit/miss/deflect

P3-06 / initial P4-08: checked server-owned hit/miss/deflect now drives player
direct damage, versioned encrypted outcomes and actual visible Bevy labels.
**589 workspace passes / 40 fixture skips in 114 groups**, formatting,
denied-warning workspace/all-target Clippy and **41 Python tests** pass.
Real OIDC/PostgreSQL/two-client outcome cases, old melee/grip/life/legacy regressions
and 3440×1440 fullscreen launcher cleanup pass; all **31 final native starts have
zero GPU diagnostics**. [Behavior, evidence, launch and limits](HIT_OUTCOMES.md).
Matched source/binaries/art/proof: `build/outcomes-qualified-v1-20261031`.
No new FPS, Windows, crowd or streaming-capacity claim. Earlier checkpoints remain.

# Current playable checkpoint — 2026-10-31: player sword melee

P2-12 / P3-06 / P3-08 / initial P3-14: committed sword equipment now gates a 3 m
mobile melee action with authoritative windup, range/life/LOS and queue checks.
Standard Bevy upper-body attacks preserve standing/walking/running locomotion
locally and remotely. Terrain readiness follows actual terrain pipelines rather
than cosmetic compilation. [Behavior, proof, launch command and limits](PLAYER_MELEE.md).

Final unchanged source: **571 workspace passes / 40 explicit fixture skips in
113 groups**, formatting, denied-warning workspace/all-target Clippy and
**41 Python tests**. Real two-account OIDC/PostgreSQL/native melee, moving grip,
rewarded life/restart/backend-loss and legacy regressions pass; normal launcher
cleanup passes at captured 3440×1440 fullscreen. All 25 final starts record zero
GPU diagnostics. Quiet one-client near-spawn cadence is **711/713 FPS** at
1920×1200/3440×1440: [measurement and limits](PLAYER_MELEE_PERFORMANCE.md).
Matched source/binaries/art/proof: `build/melee-qualified-v1-20261031`.
Earlier checkpoints remain preserved. No crowd/streaming or Windows claim.
Changes remain local. Hit/miss/deflect resolution is the next small slice.

# Performance follow-up — 2026-10-31

Unchanged pursuit checkpoint: one native client with the combat rig and active
wolf/pursuit records **713 FPS at 1920×1200** and **712 FPS at 3440×1440 fullscreen**
near spawn. Each uses 15 steady one-second windows, 81 resident chunks and no
unloads; external validation is off. [Exact evidence and limits](CREATURE_PURSUIT_PERFORMANCE.md).
This is not population/streaming capacity. Player sword melee is the next slice.

# Current playable checkpoint — 2026-10-31: enemy perception, pursuit and return

P3-07 / P4-03 / initial P3-08 and P3-14: the opt-in wolf perceives a live player,
runs on checked navigation, attacks within reach, breaks off at its leash,
returns to its captured patrol position and resumes the saved route.
[Behavior, exact native evidence, launch command and limits](CREATURE_PURSUIT_NATIVE.md).

Final unchanged source: **557 workspace passes / 40 explicit fixture skips in
113 groups**, formatting, denied-warning workspace/all-target Clippy and
**41 Python tests**. Actual two-account OIDC/PostgreSQL/native pursuit passes at
1920×1200; moving sword grip, rewarded life/rejoin/restart/backend-loss and legacy
1280×720 regressions pass. Normal new-bundle launcher cleanup passes all three
modes at captured 3440×1440 fullscreen. All final starts have zero GPU diagnostics.
Matched source/binaries/art/proof: `build/pursuit-qualified-v1-20261031`.
Earlier checkpoints remain preserved. This is one NPC in an eight-player prototype,
not population/streamed-navigation capacity. Public NPC attack timing, player
melee/ranged enemy expansion and native Windows remain open. No standalone FPS
claim is made in this checkpoint. Changes remain local.

# Headless slice qualified — 2026-10-31: perception, chase, melee and return

P3-07 / P4-03: opt-in NPC perception/chase/attack/leash/return now drives real
collision/grounding and resumes saved patrol progress. 89 network unit tests,
workspace denied-warning Clippy, formatting and both content bundles pass.
Actual encrypted two-peer pursuit passes at 30 Hz; controller/lifecycle tests run
at 20/30/60 Hz. [Implemented behavior, exact proof and limits](CREATURE_PURSUIT.md).
Matched headless source/proof: `build/pursuit-authority-qualified-v1-20261031`.
Native OIDC/PostgreSQL/wolf qualification is in progress. The runnable grip
checkpoint stays preserved. No new whole-workspace, FPS/capacity or Windows claim.

# Checked prerequisite — 2026-10-31: bounded creature path worker

P3-07 / P4-03: the reviewed static Landmass query is shared with a dedicated
background worker. One outstanding request, bounded package/routes, exact actor /
target-life / destination-revision matching and stale-result expiry pass.
[Implemented contract, proof and limits](CREATURE_PATH_WORKER.md).

Final workspace: **539 passes / 40 explicit fixture skips in 112 groups**,
formatting and denied-warning workspace/all-target Clippy. The server's normal
dependency graph remains headless; library/engine versions are unchanged.
Matched source/proof: `build/navigation-worker-qualified-v1-20261031`.
No creature consumes dynamic paths yet; perception/chase/leash is next.
The runnable `build/grip-qualified-v1-20261031` remains preserved. No new native,
FPS, population-capacity or Windows claim is made. Changes remain local.

# Current playable checkpoint — 2026-10-31: equipped sword stance and moving grip

P2-09 / P3-08 / initial P3-14: server-owned public appearance now selects standing
SwordIdle on local and remote players. While moving, standard Bevy masks retain
the authored closed grip on eleven right-hand digit joints; the original gait
keeps the wrist, prop socket, arm, torso and legs. Wand replacement and unequip
remove the layer; death retains precedence. [Behavior, proof and limits](EQUIPPED_PLAYER_STANCE.md).

Final workspace: **537 passes / 40 explicit fixture skips in 111 groups**,
formatting, denied-warning workspace/all-target Clippy and **41 Python tests**.
Actual native local and remote Walk/Run poses match the source closed fingers and
original locomotion on every other sampled target. Two-account OIDC/PostgreSQL
rewarded life/rejoin/restart/backend-loss passes at 1920×1200; legacy rig at
1280×720; focused replacement, unequip and equipped movement at 1920×1200.
Normal launcher cleanup passes all three modes at captured 3440×1440 fullscreen.
All final starts have zero GPU diagnostics. Matched source/binaries/art/proof:
`build/grip-qualified-v1-20261031`. The socket checkpoint remains preserved.
Source art and the failed 5 mm production skin gate are unchanged. Upper-body
combat layers, authored equipped locomotion, public remote-player death and native
Windows remain open. No new FPS/capacity claim is made. Changes remain local.

# User-requested combat/world/HUD follow-ons — 2026-10-31

Continue the current animation/grip qualification, then prioritize perception,
chase/attack/leash and finite-range combat. Projectiles, hit/miss/deflect outcomes,
quest markers, minimap and a reviewed local HUD asset redesign are also recorded.
[Existing behavior, task mappings, prerequisites and acceptance](COMBAT_WORLD_FOLLOWUPS.md).
P3-17/P3-18 remain unspecified.

# Current playable checkpoint — 2026-10-31: authored prop sockets and player death

P1-10 / P2-09 / P3-08 / initial P3-14: the opt-in POLYGON combat conversion
preserves `Prop_L` / `Prop_R`, attaches development equipment to the authored
right-hand socket, and plays the local player's authoritative death once before
restarting animation for a fresh life. Actual two-account OIDC/PostgreSQL/native
gameplay passes at 1920×1200, the legacy rig passes at 1280×720, and normal
launcher cleanup passes all three modes at captured 3440×1440 fullscreen.
[Behavior, exact evidence and limits](PLAYER_ANIMATION_SOCKETS.md).

Final workspace: **532 passes / 40 explicit fixture skips in 111 groups**,
formatting, denied-warning workspace/all-target Clippy and **41 Python tests**.
All final native starts have zero GPU diagnostics. Exact matched source,
executables, art and proof: `build/sword-qualified-v1-20261031`. The wolf checkpoint
remains preserved. Remote-player death replication, equipped combat stance /
layering, actual sword mesh adoption, the failed 5 mm production skin-fidelity
gate and native Windows remain open. No new FPS or MMO-capacity claim is made.
Changes remain local.

# Current playable checkpoint — 2026-10-31: animated development wolf

P1-10 / P2-08 / P2-09 / initial P3-14: the opt-in MeshTint wolf now follows
existing authority patrol, incoming windup, hit, death and fresh respawn states
through standard Bevy animation. Actual two-account OIDC/PostgreSQL/native gameplay
passes with current and legacy bundles; normal launching passes all three cleanup
modes at captured 3440×1440 fullscreen. [Behavior, qualification and limits](WOLF_CREATURE.md).

Final workspace: **529 passes / 40 explicit fixture skips in 111 groups**, formatting,
denied-warning workspace/all-target Clippy and 28 Python tests. Quiet single-client
near-spawn baselines are **719 FPS at 1920×1200** and **720 FPS at 3440×1440
fullscreen**; these are not crowd/streaming-capacity results. Exact source,
executables, art and proof: `build/wolf-qualified-v1-20261031`. Prior checkpoints
remain preserved. Public NPC attack timing, general models, player death animation,
production skin fidelity and native Windows remain open. Changes remain local.

# Current playable checkpoint — 2026-10-31: transient player life and retaliation

P3-04 / P3-07: the opt-in thirteen-profile bundle now has authoritative player
health, first-attacker melee retaliation, dead-input suppression and supported
respawn with fresh life/movement streams. Standard Bevy health and incoming /
death / respawn text pass two-account OIDC/PostgreSQL/native gameplay, rewarded
quests, rejoin, server/database restart and backend-loss checks at 1280×720,
1920×1200 and 3440×1440 fullscreen. Normal launcher cleanup passes all three modes.
[Implemented behavior, exact GPU diagnostics and limits](PLAYER_LIFE_NATIVE.md).

Final unchanged-source checks: **525 workspace passes / 40 explicit fixture skips
in 111 groups**, formatting, denied-warning workspace/all-target Clippy and
28 Python tests. Exact matched binaries / source / proof:
`build/player-life-qualified-v1-20261031`. Health remains transient; production
combat logout policy, chasing/general threat, death animation and native Windows
remain open. Native validation ran with concurrent compilation; no standalone FPS
or MMO-capacity claim is made. Prior playable checkpoints remain preserved.
Changes stay local; no commits or pushes were made.

# Active development — 2026-10-31: shared life-aware prediction qualified

P3-04 / P3-07: initial/dead/unmatched life states now hold the opt-in shared driver,
clear old input/clock debt and keep completed authority corrections visible.
Actual impaired encrypted UDP resumes authoritative movement after respawn at
20/30/60 Hz. **108 relevant network tests**, formatting and denied-warning
workspace/all-target Clippy pass on unchanged final source. [Behavior and limits](PLAYER_LIFE_PREDICTION.md).
Exact source/proof: `build/player-life-prediction-qualified-v1-20261031`.
Prior full workspace evidence remains 519 passes / 40 skips, not rerun here.
Normal executable/bundle and Bevy/native adoption remain open. Changes stay local.

# Active development — 2026-10-31: encrypted player-life runtime qualified

P3-04 / P3-07 owner-only health, incoming windup, death and respawn now pass actual
two-peer encrypted UDP at 20/30/60 Hz under delay/loss/reordering. Full workspace:
**519 passed / 40 explicit skips in 111 groups**, formatting and denied-warning
workspace/all-target Clippy on unchanged final source. Five new runtime checks
include one specifically identified server-queue unit test. [Contract, evidence scope and next acceptance](PLAYER_LIFE_RUNTIME.md).
Exact source/proof: `build/player-life-runtime-qualified-v1-20261031`. Normal
executable/bundle and Bevy/native adoption remain open; new-mode OIDC, SQL health
durability and native rendering are not claimed. The preserved playable checkpoint
remains `build/character-menu-qualified-v1-20261031`. Changes stay local.

# Active development — 2026-10-31: life codec/receive barrier qualified

P3-04 / P3-07: bounded owner-only PL01 frames and life/movement correction gating
pass eight wire tests, 100 core / 76 network-library tests, formatting and denied-
warning workspace/all-target Clippy on unchanged final source. Real completed
World health/death/respawn states round-trip through this codec. [Contract and
remaining adoption gates](PLAYER_LIFE_PROTOCOL.md). Exact source/proof:
`build/player-life-wire-qualified-v1-20261031`. New-mode Renet publication/client
bridge, executable/bundle and Bevy/native adoption remain open. No new-mode
multiplayer/native result is claimed; prior playable checkpoints stay preserved.
Changes remain local.

# Active development — 2026-10-31: player life/retaliation headless authority qualified

P3-04 / P3-07: the new opt-in World mode now runs checked retaliation, transient
health/death and supported streamed respawn with fresh life/movement streams.
Thirteen new World tests, all 100 core / 75 network-library tests, formatting and
denied-warning workspace/all-target Clippy pass on unchanged final source.
[Implemented behavior, ordering and remaining gates](PLAYER_LIFE_AUTHORITY.md). Source/proof:
`build/player-life-authority-qualified-v1-20261031`. Private wire/client/Bevy and
normal executable/bundle adoption remain open. Normal launching still uses the
preserved playable behavior; no new-mode multiplayer/native claim is made. Prior
checkpoints remain preserved; changes stay local.

# Active development — 2026-10-31: player health/life prerequisite qualified

P3-04 / P3-07: checked transient health, one scheduled respawn and nonzero per-actor
life generation pass eight new tests / 100 total core passes, formatting and denied-
warning workspace/all-target Clippy. Retaliation and damage application both refuse
old lives. [Contract and runtime acceptance](PLAYER_LIFE.md). Source/proof:
`build/player-life-core-qualified-v1-20261031`. World/transport/Bevy adoption remains
open; the current server still reports configured maximum player health. No new
full-workspace/native result is claimed. Prior playable checkpoints are preserved;
all changes remain local.

# Active development — 2026-10-31: creature retaliation policy prerequisite

P3-07 / P3-04: a bounded immutable recipe and fixed-tick retaliation state machine
now pass eight new tests / 92 total core passes, formatting and denied-warning
workspace/all-target Clippy. [Contract, acceptance and limits](CREATURE_RETALIATION.md). Source/proof:
`build/retaliation-core-qualified-v1-20261031`. This emits intents only; player
health/death/respawn and runtime/Bevy adoption are next. No new full-workspace or
native result is claimed. Prior playable checkpoints below stay preserved; all
changes remain local.

# Current checkpoint — 2026-10-31: existing-account character menu qualified

P3-03: the console launcher lists existing characters and offers N to create
another below the eight-character cap. A known incompatible-profile refusal returns
to selection without rewriting saves; temporary retirement conflicts keep their
bounded retry. Actual OIDC creation on an existing account, incompatible-to-compatible
menu recovery, two native players, rewarded gameplay and database/server restart
pass at 1920×1200 windowed with zero Vulkan diagnostics. Normal eleven-profile
launcher startup/cleanup passes all three modes. [Contract and proof](CHARACTER_LAUNCHER_MENU.md).

Thirty focused Python checks, formatting and denied-warning workspace/all-target
Clippy pass. Rust/content/binaries are byte-for-byte unchanged from the previous
476-pass / 40-skip workspace and 30-contract SQL checkpoint; those suites were not
rerun for the five Python-file changes. Matched binaries/source/evidence:
`build/character-menu-qualified-v1-20261031`. This is console workflow; physical
terminal/browser input, a complete Bevy login UI and native Windows remain open.
Changes remain local. Prior checkpoints stay preserved.

# Prior checkpoint — 2026-10-31: character gameplay-profile preflight qualified

P3-03 / P3-02: enabled saved inventory/XP/quest profiles are checked in a read-only
transaction while the Selecting reservation is held, before issuing a game ticket.
Expected content incompatibility returns a clear nonretryable HTTP 409 and leaves
the SQL worker available. Missing optional state remains compatible. No saved data
is initialized, migrated or reseeded by this operation; active-owner checks remain.

Actual OIDC/native/SQL at 1920×1200 windowed passes refusal with no ticket, unchanged
legacy journal data through database/server restart, then fresh rewarded training,
loot/barter/equipment and rejoin. Zero Vulkan diagnostics on Linux RX 7900 XTX/RADV.
The client/core/network/content source is unchanged from the prior three-size
reward checkpoint; a new three-size server test is not claimed.
Final unchanged-source workspace: **476 tests / 40 explicit skips**
in 108 groups. Formatting, denied-warning Clippy, 24 focused Python checks
and 30 actual SQL contracts / 33 commands pass, including cancelled-answer and
bounded-queue preflight tests. [Contract and proof](CHARACTER_PROFILE_PREFLIGHT.md). Matched binaries,
source, validator and evidence: `build/gameplay-profiles-qualified-v1-20261031`.
All changes remain local. General migrations, complete saved-value validation,
physical OS input and native Windows remain open. Prior checkpoints are preserved.

# Prior checkpoint — 2026-10-31: optional quest completion XP qualified

P3-10 / P3-11 / initial P3-16: the opt-in rewarded training quest now accepts at
the guide, counts three authoritative kills, then commits quest completion,
25 XP and its action receipt atomically. Ready at 75 XP becomes 100 XP / level 2
on the default XP curve. Recovered commands and rejoin/restart never award twice.
Standard Bevy guide controls display the configured reward. Item turn-in grants,
abandonment/repeatables, groups and a complete authored production zone remain open.

Actual two-account OIDC/encrypted/SQL native flows pass at 1280×720 and 1920×1200
windowed (zero Vulkan diagnostics), plus 3440×1440 borderless fullscreen (one
previously documented upstream surface-resize diagnostic per startup). Private
progress, personal loot, barter, equipment, reconnect, database/server restart and
backend-loss checks pass. Current binaries also pass the retained journal-only
native flow. The normal eleven-profile schema-2 launcher passes all three cleanup
modes; schema-1 ten-profile validation identity is preserved. Fresh compatible
characters are required; saved content profiles are never silently migrated.

Final unchanged-source workspace passes **475 tests / 37 explicit skips**
in 107 groups. Formatting, denied-warning workspace/all-target Clippy and
23 focused Python checks pass. Separately retained prerequisites cover 27 actual
SQL contracts / 32 commands and three actual OIDC/ticket/SQL retirement modes.
[Exact scope, prerequisites and proof](QUEST_TURN_IN_XP.md). Immutable source, matched optimized
binaries, validator and public evidence: `build/quest-turn-in-xp-qualified-v1-20261031`.
Native Windows, physical 4K, physical OS input and MMO population/performance
capacity are not qualified. All changes remain local; no commit/push this session.
Earlier journal-only and automatic checkpoints remain preserved.

# Stopped safely — 2026-10-30: initial persistent training-zone integration qualified

The combined two-account OIDC/encrypted/PostgreSQL native run passes at
1920×1200 on Linux (RX 7900 XTX, RADV/Vulkan). Each character independently
completes three kills, earns XP, claims one reward, barters and equips a distinct
item. Exact bags, hands, progress and latest item receipts survive reconnect and
database/server restart; ownership/replay fencing and backend-loss checks pass.
[Scope, fixture settings and limits](PERSISTENT_ZONE_INTEGRATION.md).

**441 workspace tests pass / 30 explicit skips**, with unchanged final source.
Formatting, denied-warning workspace/all-target Clippy and nine Python helper tests
pass. Exact source, binaries and proof are preserved in
`build/zone-qualified-v1-20261030`; prior vendor and content checkpoints remain.
This combined flow is qualified at 1920×1200 only; prior vendor checks cover
1280×720 and 3440×1440 separately. Physical OS input, native Windows, 4K and
MMO-scale capacity remain unqualified. Accelerated qualification fixtures do not
change default gameplay balance.

Development is stopped at the user's request; all current repository changes are
prepared for commit and push, including concurrent story/Obsidian edits. Private
conversation exports stay local and ignored. This is initial P3-16 training
integration, not a completed authored production zone. P3-17/18 remain unspecified.
Earlier notes below are historical.

# Active development — 2026-10-30: initial P3-15 offline gameplay validator qualified

The ten-profile development bundle now has an offline validator using the actual
core/runtime schemas. It checks item/NPC/creature references and ability/resource/
status compatibility, bounds file reads/paths and reports deterministic exact-byte
hashes. [Tool, acceptance and limits](GAMEPLAY_CONTENT_VALIDATION.md). Runtime profile loading stays as
qualified; this manifest is an authoring check, not an authoritative world identity.

**441 workspace tests pass / 30 explicit skips**, with unchanged final source.
Formatting, denied-warning workspace/all-target Clippy, eight focused tool tests
and eight actual CLI refusal cases pass. Source inputs remain unchanged. Exact
source, tool/test executables, docs and proof: `build/gameplay-content-qualified-v1-20261030`.
All previously qualified vendor/gameplay source files remain byte-identical; the
playable checkpoint remains `build/vendor-qualified-v1-20261030` (actual SQL and
native purchasing at 1280×720, 1920×1200 and 3440×1440). No new runtime/native,
performance or MMO-capacity claim is made for this offline tool.

Next is a bounded P3-16 persistent training-zone integration check combining the
existing initial gameplay systems through login, play, logout and reconnect;
its acceptance still needs implementation/verification. Production authoring UI,
navigation/terrain/assets, general quest acceptance/turn-in and other remaining
systems stay open. P3-17/18 remain unspecified. Changes remain local/uncommitted;
concurrent story edits are untouched. Earlier notes below are historical.

# Active development — 2026-10-30: initial P3-12 live vendor barter qualified

The guide offers one positive-cost barter through the existing shared item
owner/sequence: one Training Token for three Training Stones. Live guide/session/
range checks precede acceptance; accepted queued/executing work survives logout.
The Bevy shop retains displayed offer/revisions and acknowledges committed items.
[Exact current behavior, evidence and limitations](VENDOR_BARTER_FOUNDATION.md).

**433 workspace tests pass / 30 explicit skips**, with unchanged final source.
Formatting, denied-warning workspace/all-target Clippy, nine Python helper tests,
twenty actual SQL contracts/twenty-six commands, actual item/reward/vendor retirement
and mixed-launcher cleanup pass. Two actual OIDC/native accounts pass purchases,
own SQL/UI comparison, close, reconnect, database/server restart, distant restore
and backend failure at **1280×720, 1920×1200 and 3440×1440 fullscreen**. Actual
widgets were inspected. Windowed GPU validation is clean; fullscreen records only
one specifically allowed upstream wgpu resize diagnostic per startup. Physical
4K, physical OS input and native Windows remain unqualified.

Exact source, five executed binaries, non-story docs and proof are frozen at
`build/vendor-qualified-v1-20261030`; earlier checkpoints remain preserved. Currency,
selling, stock/buyback, general quest acceptance/turn-in and production NPC/player
lifecycle remain open. No new frame-rate or MMO-population claim is made.
Next is a small P3-15 offline gameplay-content validation slice using the existing
checked schemas. P3-17/18 remain unspecified. Changes remain local/uncommitted;
concurrent story edits are untouched. Earlier notes below are historical.

# Active development — 2026-10-30: P3-12 barter database prerequisite qualified

The positive-cost barter now has an atomic backpack cost/output/receipt transaction
and uses the existing bounded critical worker. Exact latest retries recover without
another charge; superseded/conflicting input rejects. [Exact scope/proof](VENDOR_BARTER_FOUNDATION.md).

**Twenty actual SQL contracts/twenty-six commands pass**, including three new
purchase contracts: rollback, owner/world/byte binding, shared-revision races,
full-bag recovery, cancelled answers, queue saturation, backend loss and actual
process/database restart. Eight barter/36 core regression tests remain qualified;
core source is unchanged. Workspace check, formatting and denied-warning Clippy
pass. Exact prerequisite source/probe/proof: `build/vendor-store-qualified-v1-20261030`.

Live NPC/range entitlement, accepted item-owner retention, encrypted purchase
commands and shop UI remain open. No native purchase or new full-workspace claim
is made. The prior playable NPC/display remains frozen at
`build/dialogue-qualified-v1-20261030`: 418 workspace passes/27 skips and actual
OIDC/native dialogue at 1280×720, 1920×1200 and 3440×1440. P3-17/18 remain unspecified.
All changes remain local/uncommitted; story edits are untouched. Earlier notes
below are historical.

# Active development — 2026-10-30: initial P3-12 informational dialogue qualified

One noncombat guide now supports private authoritative branches, bounded Renet
requests/acknowledgements and a standard Bevy panel. E talks, F1–F4 or buttons choose,
Esc closes without exiting, and walking away closes the session. Clicks retain the
observed choice/revision. [Exact source/evidence/limits](NPC_DIALOGUE_FOUNDATION.md).

**418 workspace tests pass / 27 explicit skips**, with unchanged final source.
All seventeen SQL contracts/twenty-three commands separately pass; ten private-asset
cases remain skipped. Formatting, denied-warning workspace/all-target Clippy and
nine helper tests pass. Two actual OIDC accounts/native encrypted clients pass at
**1280×720, 1920×1200 and 3440×1440 fullscreen**: distinct branches, actual Bevy
keyboard/picking, close/reopen/walkaway, fresh sessions after reconnect/database/
server restart, distant restore and backend loss. Informational dialogue creates
no item/reward/XP/objective records. Actual panels were inspected. Windowed GPU
validation is clean; fullscreen has one or two specifically allowed upstream wgpu
resize diagnostics per startup and no other errors. Physical 4K/Windows remain open.

The mixed Scorch/resources/statuses/creature/equipment/reward/XP/objective/dialogue
launcher passes all three cleanup modes. Exact source, three executed binaries,
docs and proof: `build/dialogue-qualified-v1-20261030`. Previous checkpoints stay
preserved. This is the initial informational slice: vendors, quest acceptance/
turn-in, production NPC art, occlusion and real player-death closure remain open.
No new frame-rate or MMO-population claim is made. P3-17/18 remain unspecified.
All changes stay local/uncommitted; story edits are untouched. Earlier notes below
are historical.

# Active development — 2026-10-30: initial P3-11 quest objective slice qualified

One opt-in three-kill objective now uses trusted current-life facts, an atomic
XP/objective transaction, accepted-work retirement, private committed observations
and a standard Bevy journal. Completion grants no extra items or XP.
[Exact behavior, evidence and limits](QUEST_OBJECTIVES_FOUNDATION.md).

**396 workspace tests pass / 27 explicit skips**, with unchanged final source.
All seventeen actual PostgreSQL contracts/twenty-three commands separately pass;
ten private-asset skips remain skips. Formatting, denied-warning workspace/all-target
Clippy and nine Python helper tests pass. Actual OIDC/encrypted-ticket/SQL lease
checks retain accepted objective work through logout without duplicate credit.

Two actual accounts/native encrypted clients pass at **1920×1200 windowed** and
**3440×1440 fullscreen**: distinct own 1/3 versus 3/3 journals, completion after
XP caps, reconnect, database/server restart, distant restore and backend failure.
Actual widgets were inspected. Windowed GPU validation is clean; fullscreen has
one documented upstream wgpu resize diagnostic per startup and no other errors.
The ordinary 1280×720 mixed Scorch/resources/statuses/equipment/reward/XP/objective
launcher passes all three cleanup modes. Physical 4K and native Windows remain
unqualified. No new performance or MMO population capacity claim is made.

Exact source, four executables, documentation and proofs:
`build/quests-qualified-v1-20261030`. Prior checkpoints remain preserved. Next is
the bounded [P3-12 dialogue foundation](NPC_DIALOGUE_FOUNDATION.md), currently
plan only. General quest acceptance, dialogue/turn-in rewards and vendors remain
open. P3-17/18 remain unspecified. All changes remain local/uncommitted; concurrent
story edits are untouched. Nothing was pushed. Earlier notes below are historical.

# Active development — 2026-10-30: P3-11 objective core/database prerequisite qualified

One checked development objective and its durable kill receipts now share an
atomic transaction with enabled XP. Completion stays finite; unfinished objectives
can advance after XP caps. Existing XP-only callers retain their own transaction.
[Exact scope and next acceptance](QUEST_OBJECTIVES_FOUNDATION.md).

Thirty focused core/item tests pass, including five objective tests. All seventeen
actual PostgreSQL contracts/twenty-three commands pass: coupled rollback, old-event
replay, competing recipients, cancelled answers, queue saturation, terminated
backend and actual database/process restart. Formatting, workspace/all-target
check and denied-warning Clippy pass. Exact prerequisite source, probe and proof:
`build/quests-store-qualified-v1-20261030`.

Live quest credit, accepted-work retirement integration, private observations and
the Bevy journal remain open. No new full-workspace or native quest result is
claimed. The qualified P3-10 game/display source and executables remain frozen at
`build/progression-qualified-v1-20261030` (388 tests/24 skips, native 1920×1200 and
3440×1440, 808.60 FPS empty-world baseline). Changes are local/uncommitted; story
edits remain untouched. Earlier notes below are historical.

# Active development — 2026-10-30: initial earned XP and display slice qualified

P3-10's first opt-in slice now connects authoritative fatal-hit facts to atomic
victim-life receipts, committed private XP observations and a standard Bevy
level/bar. Accepted executing/distinct queued XP survives logout and prevents
character reuse until completion. Item pickup and item revisions are independent.
[Exact behavior and limits](EXPERIENCE_LEVELING_FOUNDATION.md).

**388 workspace tests pass / 24 explicit skips**. All fourteen SQL contracts/twenty
commands separately pass; ten private-asset skips remain skips. Formatting,
denied-warning workspace/all-target Clippy, five account-helper tests and four
GPU-policy tests pass. Actual OIDC/encrypted-ticket/SQL retirement proof passes.

Two actual OIDC/native encrypted clients pass kill-to-level, each own UI,
rejoin/database/server restart, distant restore and backend-loss checks at both
**1920×1200 windowed** and **3440×1440 borderless fullscreen**. The normal window
now defaults to 1920×1200; `--window-size` selects dimensions and F11 toggles
fullscreen. Mixed Scorch/resources/equipment/reward/XP launcher checks retain
1280×720 and pass all three cleanup modes. 3840×2160 is accepted configuration;
this 3440×1440 desktop clamps oversized windows, so physical 4K is unqualified.
[Window options and exact display evidence](DEVELOPMENT_WORLD.md).

Linux RX 7900 XTX / RADV / Mesa 26.2.6 / Vulkan was inspected. Windowed validation
is clean; fullscreen records one specifically documented wgpu resize diagnostic
per startup and rejects other errors. The same release client passes a validation-
off, single-client empty-world tour at **808.60 FPS uncapped, 1920×1200**, with
834 loaded / 753 retired chunks and zero missed server simulation deadlines.
This is not crowded MMO capacity or a native Windows result.

Exact source, binaries and evidence: `build/progression-qualified-v1-20261030`.
Prior reward/core-store checkpoints remain preserved. Next is the bounded
[P3-11 objective foundation](QUEST_OBJECTIVES_FOUNDATION.md); no quest
code or journal is implemented yet. P3-17/18 remain unspecified. Changes are local/
uncommitted; concurrent story edits are untouched. Nothing was pushed.
Earlier checkpoint notes below are historical.

# Paused — 2026-10-30: P3-10 experience core/database prerequisite qualified

Stopped at the user's request after finishing this prerequisite. Checked progression
profiles, integer level/XP proposals and durable atomic victim-life receipts are
implemented on the existing bounded async worker. Old-event replay returns current
committed progress without another award; cap, rollback, cancelled replies, queue
saturation, competing recipients, backend loss and real restart checks pass.
[Exact scope and remaining acceptance](EXPERIENCE_LEVELING_FOUNDATION.md).

**25 focused core/item tests pass**, including six progression tests. All fourteen
actual PostgreSQL contracts/twenty qualification commands pass. Formatting,
workspace/all-target checks and denied-warning Clippy pass. Exact source, store
probe and evidence: `build/progression-store-qualified-v1-20261030`.
Live kill-to-XP integration, admission/retirement integration, encrypted XP
observations and Bevy XP UI remain open. No new full-workspace or native XP result
is claimed; this prerequisite does not establish playable progression.

The qualified P3-09 source/executables remain frozen at
`build/rewards-qualified-v1-20261030` (**379 tests / 21 skips**, actual SQL,
native two-account reward/rejoin/restart, ordinary launchers and 805.97 FPS empty-
world baseline). Changes remain local/uncommitted; concurrent story edits are
untouched. Nothing was pushed. Earlier checkpoint notes below are historical.

# Active development — 2026-10-30: initial personal reward slice qualified

P3-09's first opt-in training-kill reward is qualified: authoritative fatal-hit
credit, durable nonrepeatable availability, explicit ordered pickup, atomic items/
consumption/receipt and a standard Bevy widget. Two actual OIDC/native encrypted
clients retain each own items through rejoin, real database/server restart,
distant restore and backend-loss fail-closed behavior. Scorch-to-reward pickup
and ordinary CLI interruption/child-failure/cancelled-login cleanup pass too.
[Exact behavior and limits](LOOT_REWARDS_FOUNDATION.md).

**379 workspace tests pass / 21 explicit skips**, formatting and denied-warning
workspace/all-target Clippy pass. All eleven SQL contracts/seventeen commands
separately execute successfully, including full-bag retention, rollback and
latest-receipt recovery; ten private-asset skips remain skips. Accepted executing
and distinct queued offers, as well as pickup, retain character ownership through
logout/retirement. The ordinary legacy backpack launcher also passes.

Native screenshots were inspected on Linux RX 7900 XTX / RADV / Mesa 26.2.6,
1280×720, with Vulkan validation. The same release binaries pass an empty-world
one-client movement baseline at **805.97 FPS uncapped**, validation off. This is
not MMO capacity or a native Windows result. Exact source/binaries/evidence:
`build/rewards-qualified-v1-20261030`. Prior equipment/store checkpoints remain
preserved. Changes stay local/uncommitted; story edits remain untouched.

Next is [P3-10 experience and leveling](EXPERIENCE_LEVELING_FOUNDATION.md):
its proposed small curve, durable kill receipts, retirement and independent own-
character UI contract are documented. No progression code/award/UI is implemented
in the reward checkpoint. Broader loot/groups/instances and production balance
remain separate work; P3-17/18 stay unspecified.

Earlier checkpoint notes below are historical.

# Active development — 2026-10-29: initial status-effect ability qualified

P3-06's first opt-in Scorch slice is qualified: checked definitions, server-owned
periodic damage and refresh, bounded independent sources, source/target cleanup,
encrypted status observations and standard Bevy selected-target indicators.
[Behavior, exact evidence and limitations](STATUS_EFFECTS_FOUNDATION.md).

**336 workspace tests pass**, with fifteen explicit skips. All five PostgreSQL
contracts separately execute and pass; ten private-asset skips remain skips.
Formatting, workspace/all-target denied-warning Clippy and five HTTP helper tests
pass. The standalone server dependency graph remains headless.

Two native encrypted clients pass impaired UDP, Vulkan validation, moving casts,
mana depletion/recovery, 84 periodic hits and four deaths/respawns each, with
protected hit cues under 32,768 cosmetic requests per client. Own/other status
labels and cleared state are visible; screenshots were inspected on Linux
RX 7900 XTX / RADV / Mesa 26.2.5 kisak, 1280×720. Ordinary status/resource/zero-cost
launchers and the actual OIDC/PostgreSQL mixed backpack launcher pass cleanup.
Exact source, binaries and evidence: `build/status-qualified-v1-20261029`.

This is an initial periodic-damage slice, not all of P3-06 or persistent combat.
Player damage/death, healing, dispels, crowd control, class scaling, durable combat
and native Windows remain open. P3-07 creature AI and its minimum P4-03 navigation
prerequisite are now in [offline bake/route prerequisite work](CREATURE_AI_FOUNDATION.md);
offline central/far saved-terrain packages and headless patrol timing/replication
checks pass; client integration and native qualification are in progress. P3-17/18 stay unspecified. Changes remain local
and concurrent story edits are untouched. Earlier checkpoints below are historical.

# Active development — 2026-10-29: initial durable backpack qualified

P3-05's first opt-in backpack slice is qualified: checked item definitions and
sixteen slots, conserved move/merge/swap/split proposals, durable atomic mutations,
latest-intent recovery and standard Bevy widgets. Neither queue acceptance nor a
client-side proposal is acknowledged as a committed item change.
[Exact behavior, proof and limits](INVENTORY_FOUNDATION.md).

**322 full workspace tests pass**, with fifteen explicit skips; formatting and
workspace/all-target denied-warning Clippy pass. All five PostgreSQL contracts
are separately executed on the final source, including dropped-response recovery,
concurrent conflicts, queue saturation, actual blocked-backend loss and clean
process restart. The other ten skips require private assets. Five HTTP helper tests
pass. Skips are never counted as passes.

Actual OIDC/encrypted-ticket/PostgreSQL lease checks retain both joined and
never-joined characters until accepted item work finishes. An already committed
pose alone cannot release a pending item transaction. Two real native encrypted
clients perform six durable changes and one rejected split each through Bevy
widgets; distinct arrangements match each own character's SQL contents through
rejoin, actual database/authority restart, changed spawn and backend loss.
Screenshots were inspected; Vulkan validation and 4×4 cell-layout checks pass on
Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.5 kisak, 1280×720.

`python3 tools/play.py --account-login --inventory` opts in using the configured
local OIDC/PostgreSQL boundary. Press I, select an item then a destination; Split
one changes the next operation. Add `--resources --spell-feedback` to load those
profiles too. The ordinary mixed launcher passes interruption/child failure and
pending-browser cancellation, with owned children/tickets removed and database
ownership closed. Exact source/binaries/evidence:
`build/inventory-qualified-v1-20261029`. Prior resource/admission checkpoints remain
preserved. [Resources and their separate performance proof](STATS_RESOURCES.md).

This is the first development backpack, not all of P3-05: equipment, instances,
item use/destruction, currency, trade, vendors and rewards remain open. Durable
items do not persist transient vitals/combat or complete the persistent zone.
Production account hardening and native Windows remain open. Next is the scoped
[P3-06 status-effect ability](STATUS_EFFECTS_FOUNDATION.md); its checked definitions, authoritative timing and encrypted two-client replication
pass focused tests. Native presentation and integrated regression qualification are
in progress. Engineering changes remain local/unpushed and concurrent story
edits are untouched. Earlier checkpoint notes below are historical.

# Current development checkpoint — 2026-10-29: persistence qualified

P3-02's initial ordinary-position persistence slice is qualified against actual
PostgreSQL 16.15. Two native encrypted clients save authoritative checked f64
positions, restart both database and server, reconnect in reversed order and
restore their own positions around 15 km despite a changed default spawn. Stable
character/owner IDs survive; runtime actor/world identity is fresh. Wrong-world
startup rejects before listen, and actual database backend loss fails the session
closed. [Exact behavior and evidence](PERSISTENCE_FOUNDATION.md).

**284 workspace tests pass**, with 12 explicit skips (ten private fixtures and two
actual-PostgreSQL tests separately executed successfully); formatting and denied-
warning all-target Clippy pass. Native integrated persistence/combat/UI/feedback
passes under impaired UDP with 9,728 cosmetic requests per client and all critical
cues delivered. Launcher interruption/child-failure cleanup closes database
ownership too. The persistence-disabled one-client empty-world baseline passes at
821.84 FPS uncapped, validation off; this does not establish MMO capacity.
Qualified artifacts: `build/persistence-qualified-v1-20261029`.

`python3 tools/play.py --persistence` opts into the development database boundary
using private `THEIA_DATABASE_URL` (numeric loopback only); add `--spell-feedback`
for combat/UI/effects. Ordinary launch still works without database configuration.
This uses trusted development issuer slots, not external accounts. Character
creation/selection, inventory/economy durability, power-loss recovery, process
handoff, native Windows and production database transport remain open. Owned test
clusters/processes stopped and plaintext password files were removed.

Next is P3-01 provider-neutral OIDC, validated against an owned local Keycloak
fixture, as selected by the user. Its initial scope and acceptance checks are in
[AUTHENTICATION_FOUNDATION.md](AUTHENTICATION_FOUNDATION.md). The provider
fixture has passed discovery and a real authorization-page/PKCE code exchange;
application verification, internal sessions and game admission remain unimplemented.
Production identity-provider selection stays open. Engineering changes remain
local/uncommitted; nothing was pushed and story edits remain untouched.
Earlier checkpoints below are historical.

# Current development checkpoint — 2026-10-29

P2-14's initial action-slot/health/cast/cooldown UI is qualified. Standard Bevy
widgets and the Digit 1 shortcut share the existing bounded cast-request path;
all values come from authoritative health/tick deadlines. Two actual clients pass
impaired encrypted UDP, five pointer presses/five widget requests plus five
keyboard requests each, <0.47 px actual fill-width error and integrated
movement/targeting/damage/death/respawn/critical cues. [Exact evidence](COMBAT_UI.md).

**280 workspace tests pass**, ten private-fixture skips, denied-warning
workspace/all-target Clippy and formatting. A first full-build Rust linker
segmentation fault is retained; the serial retry completed successfully.
Native critical-only quality, isolated PCM, same-binary ordinary baseline
(832.17 FPS uncapped, one empty-world client, validation off) and real two-client
UI-only/feedback launcher interruption/child-failure cleanup pass. Binaries and
original data are preserved in `build/combat-ui-qualified-v1-20261029`.

Run `python3 tools/play.py --combat` for the clickable slot and bars, or
`--spell-feedback` for the original spatial effects/audio too. Tab selects,
Backspace clears; press the slot or Digit 1 to cast. This is one development slot,
not a production multi-bar editor, full accessibility/responsive UI, player HP,
physical OS input, Windows execution or population-capacity qualification.

P3-02 persistence is now in progress. Actual PostgreSQL store/migration/worker
checks pass, including ownership, stale/invalid writes, transactional rollback,
bounded coalescing and cancellation/backend-loss behavior. The server adapter is
wired; live save/restore acceptance is pending. See `docs/PERSISTENCE_FOUNDATION.md`. No PostgreSQL system service was
installed; the owned test server is stopped. External identity-provider choice
remains open. All changes stay local/uncommitted; story edits remain untouched.
Earlier checkpoints below are historical.

# Current development checkpoint — 2026-10-28

The initial P2-13 spell feedback slice is qualified: original standard Bevy
meshes/materials and spatial audio follow actual encrypted server outcomes,
including damage/death after target retirement. Combat profile 2 binds checked
global cue positions and ability IDs; old combat-profile clients fail admission.
The full workspace run passed **278 tests** (ten private-fixture skips); final
client tests, denied-warning all-target Clippy and formatting also pass.
[Exact native evidence, hashes, prior failure and limits](SPELL_FEEDBACK.md).

Run `python3 tools/play.py --spell-feedback` (optionally `--clients 2`): Tab selects,
Digit 1 casts, Backspace clears. The normal saved-terrain/prediction launch,
selection-only `--targeting` and UI-only `--combat` remain available; these modes
keep audio disabled. Two native Linux/Vulkan clients passed impaired UDP with
10,240 synthetic cosmetic requests each: all 18 critical cues per client played
and appeared under concurrent cosmetic load, with zero critical overflow/event
gaps/large movement displacements. Peak visuals/voices were 50/26, within 64/32.
Critical-only quality and live visual/audio origin shifts also pass.

Owned isolated audio captures contain actual cue PCM, no foreign streams or
default-sink changes; private sinks were removed. The same-binary one-client empty
baseline passed at 826.78 FPS uncapped with validation off; this is not MMO
capacity. Actual two-client feedback launcher interruption/child-failure cleanup
passed. Binaries/data are preserved in `build/spell-feedback-qualified-v1-20261028`.

Stopped safely after this completed slice at the user's request. Next planned
work is P2-14 clickable action-slot and clearer health/cast presentation; it has
not started. Broader combat/AI/navigation, persistence/rewards, native Windows,
physical keyboard/mouse delivery, production sound design and population scale
remain open. All engineering changes remain local and uncommitted; nothing was
pushed. Story edits were left untouched. Earlier checkpoints below are historical.

# Current development checkpoint — 2026-10-26

Resumed after user commit `9497a18`. The input-agreement diagnostic gate now passes
236 full workspace tests (ten private-fixture skips), denied-warning Clippy/fmt,
actual saved/generated impaired UDP and an uncontended 600-second eight-peer soak.
Both simulation and I/O record zero missed deadlines in the uncontended run;
identity return, bounded terrain residency, diagnostic counts and cleanup pass.
[Input evidence](PREDICTION_INPUT_DIAGNOSTICS.md), [soak evidence](SAVED_TERRAIN_SOAK.md).
Severe-link errors up to 1.6085 m remain documented; stopped convergence is not a
smoothness guarantee. Native Windows and MMO population capacity remain open.

P1-11's bounded ability-data prerequisite is now qualified: immutable validated
JSON, mobile-casting defaults, off-GCD options and ceiling simulation-tick timing.
All 16 core tests, workspace/all-target Clippy/fmt and actual offline validation at
20/30/60 Hz pass. [Ability data](ABILITY_DATA.md). These checks follow the movement
checkpoint; no combined 241-test result is claimed. Live targeting/casts/damage
and the rest of P1-11/P2-11–14 remain open. All new changes stay local.

## Previous checkpoint

# Current checkpoint — stopped safely, 2026-10-26

The user requested stopping. No soak is running; the final diagnostic full-suite
relink was safely interrupted. Changes remain local and uncommitted.

The actual 600-second, eight-peer saved-terrain soak passes identity return,
bounded residency and zero missed simulation deadlines. Separate I/O pump misses
under concurrent builds remain recorded. [Soak evidence](SAVED_TERRAIN_SOAK.md).
The poor-link test passes movement/convergence at 20/30 Hz and safe suspension
outside the 60 Hz latency window. [Envelope](PREDICTION_LATENCY.md). That checkpoint
passes 235 workspace tests (ten private-fixture skips), denied-warning Clippy/fmt,
221 asset Python tests and four GLB alignment tests.

The subsequent bounded input-agreement diagnostics are implemented; all 11 focused
history tests and workspace/all-target denied-warning Clippy pass. Their final
full-suite/real-UDP run was interrupted, so these new counters have no completed
runtime qualification. [Diagnostic checkpoint](PREDICTION_CONTRACT.md). Resume with
that verification before interpreting the remaining correction outliers. No next
feature is active; no native Windows or MMO population capacity is qualified.

Current continuation, 2026-10-23: [saved height terrain](SAVED_TERRAIN.md) passes
its bounded Linux adoption checks. Same-tick diagnostics and the smoother prediction
clock pass. The actual eight-peer, 180-second headless saved-terrain soak also
passes; [scope and remaining checks](SAVED_TERRAIN_SOAK.md). Work is stopped at the
user’s request, with changes local and uncommitted. The saved slice
extends the existing continental loader (P1-10/P2-06/07, initial P3-13). Preserve
procedural and imported-map regressions. Zone ownership, sharding, navigation,
terrain editing/materials and population capacity remain separate work.

Current priority, 2026-10-22: [256 km² graybox terrain](GRAYBOX_WORLD.md) and the
POLYGON Base Locomotion development player. Forest reproduction is frozen. Complete
and qualify bounded visual/collision generation, culling/unloading, long-distance
frames and authoritative traversal before returning to later gameplay milestones.
This is one terrain foundation slice, not completion of P1/P2 or MMO-scale capacity.
That traversal slice now passes. P1-06/08 and the bounded P2-04 continuation now
include actual Renet prediction/reconciliation, impaired-UDP and native Linux
traversal/reset checks. The graybox launcher adopts prediction and retains
`--no-prediction`; broader latency/load/Windows qualification remains open.
See the [prediction contract](PREDICTION_CONTRACT.md).

# Roadmap

Forest performance correction, 2026-10-22: the reported ~10 FPS walkthrough
used unoptimized binaries and redundant render-mesh camera scans. The launcher
now uses release binaries; chunk cameras use existing accelerated collision
geometry while legacy maps retain visual refinement. The unchanged full scene
measures 99.65 FPS uncapped at 1280×720 on the Linux RX 7900 XTX, versus 64.90 FPS
with optimized visual refinement. The reopened desktop window averages 57.89 FPS
with normal VSync, with frame-time spikes retained in the evidence. All 45 client
tests, denied-warning workspace/all-target Clippy, fmt/diff and 221 asset Python
tests pass. This is one machine/view, not MMO capacity or every-route performance.
Broader quality/performance gates remain. [Evidence](FOREST_PERFORMANCE.md).

Forest normal-runtime continuation, 2026-10-21: ten bound material packages
preserve complete source geometry/collision; independent comparisons, four
rehashed faults, all 804,029 source rays and glTF validation pass. A separate
candidate excludes exactly one invisible water collider. Two real encrypted
Linux/Vulkan clients load the full scene, share four projection materials and
47 actual images, and pass visibility/readiness/rebase checks. Per-client base
RGBA CPU image data is 1,589,649,408 bytes (not total memory). The actual-map
survey records 27 clear starts and 41 rejected routes; movement, lighting,
mips/resources, water and broader camera/adoption remain open. Town stays the
default. All work stays local. [Evidence and limits](FOREST_RUNTIME.md).

Forest runtime-material continuation, 2026-10-21: package schema 3 now binds
self-contained projection parameters, global f64 source anchors and canonical
PNG dependencies; a saved older reader rejects it before startup. The normal
client gates root readiness, shares actual materials/images and updates bounded
phase uniforms with live render rebasing. Six encrypted two-client fixture views
at ±128 billion metres pass, including 828 independent source pixels (colour ≤1
RGB8 level, normals exact). Five native failure paths, fresh town/farmhouse and
three retained handover cases pass. Position-based gait diagnostics preserve the
existing movement/animation acceptance checks. Complete default network tests,
44 client tests, 221 asset Python tests, fresh fmt/diff and denied-warning Clippy
pass. This qualifies the original runtime slice; forest embedding, actual forest
resources/rebase, flags/lighting/mips, water and movement/camera/adoption remain
open. All work stays local. [Contract and evidence](RUNTIME_MATERIALS.md).

Forest texture-phase continuation, 2026-10-21: the standard Bevy material
extension now accepts a bounded source translation and uploads only its periodic
texture phase. Fifteen native Vulkan views pass: 1,056 independent source-pixel
checks, nine paired colour/normal/lit cases with at most one RGB8 level difference,
a deliberately wrong origin rejected, and both complete forest views pixel-exact
outside HUD. Three translated GLBs have zero Khronos errors/warnings. Fresh fmt/
diff, denied-warning Clippy, 42 client and 218 asset Python tests pass. This is
bounded inspection qualification; normal-world automatic frame updates and bound
runtime material/image admission remain next, followed by flags/lighting,
resources/mips, water and actual-map movement/camera/adoption. All work stays local.
[Evidence and scope](TRIPLANAR_PHASE.md).

Forest package continuation, 2026-10-21: ten source-preserving candidate packages
retain all 18,660 draws, 18,925 primitive instances, 6,137,378 triangles and complete
object/full-sample collision, with at most eight cell contributors. Independent
stream/pose/material/collision checks and four rebound faults pass; actual package
loading plus all 804,029 source rays pass. Ten Khronos files have zero errors and
22 retained generated-tangent warnings. Standard unlit/HDR-emissive shared-image
admission also passes native encrypted material/town/farmhouse cases, ten semantic
faults, the complete default network suite and 41 client tests. Fresh fmt/diff,
denied-warning Clippy and 227 Python tests pass. No forest session/default is
published; custom material rebasing/admission, flags/lighting, resources/mips,
water and actual map movement/camera remain open. All work stays local.
[Packages](FOREST_SHAPE_PACKAGES.md), [standard materials](STATIC_MATERIALS.md).

Static-shape admission continuation, 2026-10-21: collision schema 4 now uses
standard Avian shared mesh inputs, oriented boxes, capsules and full-sample fields
through the normal package/client/server paths. Complete source binding covers
1,731,286 mesh vertices and 263,169 unique heights; all 804,029 source object/
terrain rays pass. Four actual Avian faults and five independent geometry faults
reject. Original encrypted two-client, ±128-billion-metre contributor, town and
farmhouse cases pass on Linux RX 7900 XTX with Vulkan validation. Fresh format,
denied-warning Clippy, complete default network suite, 41 client and 226 Python
tests pass. Existing budgets/defaults remain preserved. Forest visual/material
publication, rebase/resources, water policy and actual-map adoption remain open.
All work stays local. [Contract and evidence](STATIC_SHAPES.md).

Forest mixed-material continuation, 2026-10-21: nine crystal/geode bindings /
25 draws join the qualified subset: **18,650 source draws / nine unresolved**.
Actual Editor submesh triangles prove source-slot fidelity; four differing quad
diagonals use Editor topology with original corner normals/UVs. Crystal appearance
is an explicit static standard-PBR preview, with refraction/view effects omitted.
Full composition, nine native Vulkan cases, five rebound faults and 19 Khronos
files pass (zero errors/seven retained warnings). Fresh fmt, denied-warning Clippy
and 189 Python tests pass. Water/sky/emissive willow, lighting, rebase/resources/mips,
collision and adoption remain open. All work stays local.
[Evidence](FOREST_MIXED_MATERIALS.md).

Forest glass continuation, 2026-10-21: an explicit untextured Standard-to-glTF
BLEND adaptation adds the lantern glass: **18,625 source draws / 34 unresolved**.
Source geometry/material, actual Editor frame, exact placement/full composition
and four native Vulkan cases pass. The adaptation records Unity premultiplied
lighting/colour-space differences; no runtime shader/binary change is introduced.
Fresh fmt, denied-warning Clippy and 181 Python tests pass. Next: actual mixed
submesh-slot coverage for 25 crystal/geode draws. Remaining materials, lighting,
rebase/resources/mips, collision and adoption remain open. All work stays local.
[Evidence](FOREST_GLASS.md).

Forest colour-UV continuation, 2026-10-21: two exact giant-tree bindings /
seventeen placements now join the qualified visual subset: **18,624 source draws /
35 unresolved**. A second standard glTF channel preserves leaf colour tiling while
normal/AO retain UV0. Source/Editor/exact-placement/full composition checks and six
native Vulkan cases pass; actual loaded materials select the expected channels.
Legacy rendering and encrypted town/farmhouse regressions pass on the new client.
Fresh fmt, denied-warning Clippy, 35 client Rust and 179 Python tests pass. Remaining
materials, rebase/resources/mips/lighting, collision and adoption stay open. All work
is local. [Evidence](FOREST_COLOUR_UV.md).

Forest texture-default continuation, 2026-10-21: actual Linux Unity 6000.6.6f1
resolves four Grass/Mud textures to bilinear/repeat with original PNGs/metadata
unchanged. The ten grass draws now join the qualified subset: **18,607 source
draws / 52 unresolved**. Fresh eighteen-binding source/Editor/placement checks,
four combined native Vulkan views, three grass modes and 274 pixel checks pass.
Fresh fmt, denied-warning Clippy and 175 Python tests pass. Runtime binaries remain
unchanged. Next: independent leaf-colour UV tiling for seventeen giant-tree draws;
remaining materials, rebase/mips/resources, collision and adoption remain open.
All work stays local. [Evidence](FOREST_TEXTURE_DEFAULTS.md).

Forest triplanar batch continuation, 2026-10-21: seventeen new bindings / 518
placements now join the corrected visual subset: **18,597 source draws / 62
unresolved**. Fresh independent source/Editor/placement and full composition checks,
four combined native Vulkan views, three dirt-profile modes and 304 pixel checks
pass. Three profiles share eight extension images; 31 Standard image variants
remain distinct. Khronos: 35 files, zero errors, seven runtime-tangent warnings.
Fresh fmt, denied-warning Clippy and 173 Python tests pass. Runtime binaries stay
unchanged; collision/rebase/mips/resource and adoption gates remain. Next: actual
Editor sampler defaults for ten grass draws. All work stays local.
[Evidence and limits](FOREST_TRIPLANAR_BATCH.md).

Forest P1-10/P2-08/P2-10 catalog continuation, 2026-10-21: actual mixed Standard/
two-profile rendering, shared images, negative mapping cases and independent pixel
checks qualify schema 2 while retaining schema 1. Batch remaining eligible
triplanar bindings and compose the full visual scene next; rebase texture phase,
resources/mips, remaining materials, collision and adoption remain.
[Contract and evidence](FOREST_MATERIAL_CATALOG.md).

Forest P1-10/P2-08 mesh-frame continuation, 2026-10-21: thirteen placements are
corrected in a new source-preserving subset; all 116 prototypes, full scene,
native Vulkan and retained multiplayer regressions pass. The corrected moss
prototype/all-85 group also qualifies with independent pixel checks. Continue
multiple triplanar profiles, rebased texture phase and resource/mip integration,
remaining shaders/slots, then object collision and runtime adoption. Main subset:
18,079 draws / 580 unresolved. [Scope and evidence](FOREST_MESH_FRAMES.md).

Forest P1-10/P2-08 checkpoint: actual Linux Editor references cover all 112 scene
FBXs / 214 meshes; independent recorded-frame point sets match with byte-identical
metadata. Thirteen current batch draws need converter frame correction. The user
requested a safe stop before that work. Resume with source-bound corrections,
independent replacement geometry/placement checks and native regressions before
promotion. Existing town/forest-ground remain available.
[Acceptance and evidence](FOREST_UNITY_REFERENCE.md).

As of 2026-10-19, the [engineering plan](engineering/README.md) is the entry point
for future work. It incorporates the supplied architecture, six-phase plan and
112-entry task register. Follow the refined [foundation execution order](engineering/EXECUTION_ORDER.md),
which prioritizes dependencies and engineering evidence over playability. It includes a [repository assessment](engineering/REPOSITORY_ASSESSMENT.md)
and immediate foundation queue. The user has adopted the handoff architecture;
the [alignment record](engineering/ALIGNMENT.md) and [live task status](engineering/TASK_STATUS.md)
separate delivered foundations from remaining migrations and game features.

- [ ] Import the user-selected [Enchanted Forest](SYNTY_ENCHANTED_FOREST_IMPORT.md)
  in place of Elven Realm: source audit delivered; material/prefab conversion,
  native scene and authoritative collision qualification precede map adoption.
- [x] Shared bounded chunk startup metadata and opt-in normal executable loading:
  ordinary two-client sessions, GLB roots, live Avian/camera and explicit client
  query frames. Initial selection is bound into admission. [Scope](CHUNK_STARTUP.md).
- [x] Initial per-client chunk readiness on secure Renet, with bounded control
  messages, independent movement gates and actual Bevy/live-Avian startup checks.
  [Contract and remaining publication gate](CHUNK_READINESS.md).
- [x] Exact post-simulation readiness acknowledgements and a client confirmation
  barrier tied to the latest report and a post-application snapshot. Rendered
  retention and paired replacement remain open. [Contract](CHUNK_READINESS.md).
- [x] Shared coverage sweep constraints for input and gravity-driven motion,
  missing-corner/gap protection and validated support fallback. Native loading-edge
  pressure and retained map regressions pass. [Scope](CHUNK_BOUNDARIES.md).
- [x] Bounded paired normal-client handover on original fixtures: authority
  offers, retained presentation/recovery coverage, exact retirement barriers,
  standard Bevy roots and live Avian generations. Delayed preparation, failed
  replacement retention and distant frames qualify. [Scope](CHUNK_HANDOVER.md).
- [x] Bounded contributor-complete geometry: schema-2 bound package extents,
  real collider/Bevy validation and shared coverage/demand/readiness. Original
  straight/diagonal nonlocal geometry and extent failures qualify.
  [Scope](CHUNK_CONTRIBUTORS.md).
- [x] Bound shared PNG variants and standard Bevy handle reuse: immutable
  dependencies, actual image format/sampler checks, failed-dependency retention
  and source-preserving extraction of the actual town. [Scope](SHARED_IMAGES.md).
- [x] Offline actual terrain/collision partitioning, whole model ownership and
  per-package graph pruning: independent source comparison and all 26 Rust
  package checks pass; two generated original-fixture native cases qualify the
  format. [Scope and measured provider requirements](WORLD_PARTITIONING.md).
- [x] Offline provider/resource budget analysis: source-verified recovery and
  actual package/image costs, complete capsule corners and bounded actor/
  lookahead unions. [Measured 18/22/26-provider requirements](CHUNK_BUDGETS.md).
- [x] Bounded 32-package runtime capacity, budget-bound admission and wire/ALPN
  11: actual secure UDP, 32 native roots, six-provider delayed publication and
  opt-in ten-to-26-package town authority checks. [Scope](CHUNK_CAPACITY.md).
- [ ] Qualify actual-town runtime residency/staging resources and world-edge policy,
  remaining handover lifecycle checks and real town adoption. The current
  town authority fits the new budget; client resources and adoption remain open.
  The normal launcher stays monolithic until the runtime adoption gate passes.
- [x] Explicit finite chunk-world bounds: admission identity v4, conservative
  local boundary sweeps, bounded lookahead, edge/corner motion and fall recovery.
  Native original fixtures and opt-in town authority qualify the bounded slice;
  actual-town client resources and adoption remain open. [Scope](WORLD_EDGES.md).
- [x] Bounded live-publication diagnostics, resolved recovery feet and exact decimal
  identity parsing; staged-candidate cleanup and retained map regressions pass.
  Broader lifecycle cases and adoption remain open. [Scope](TOWN_LIVE_PUBLICATION.md).
- [x] Actual town ten/all-26-package two-client startup and sampled process/DRM
  resources, with corrected conservative offline visual envelopes. Live staging/
  retirement and adoption remain open. [Measurements](TOWN_CLIENT_RESOURCES.md).

- [x] Bounded moving-actor authority chunk activation: current capsule/support
  checks, stale candidate rollback, conservative unloaded-boundary movement and
  background loading in the actual two-client Renet loop. Normal paired client/
  authority residency and tile partitioning remain. [Scope](CHUNK_ACTIVATION.md).

- [x] Native qualification/adoption of schema-3 general collision and explicit
  spawns: original mesh and imported map cases, distant frames, spawn-free chunk
  publication/rollback, startup rejection and actual default launcher package/
  cleanup pass. [Scope](GENERAL_COLLISION.md). Moving-actor activation remains.

- [x] Normal imported maps now use shared capsule movement/jumping, validated
  cached development spawns, finite-world barriers and observable per-actor fall/
  invalid-state recovery. [Scope](KINEMATIC_CONTROLLER.md). Live chunk/actor
  activation and normal residency adoption remain open.

- [x] Shared general capsule controller in graybox authority/offline sandbox:
  actual Avian walls, headroom, steps, slopes, stacked floors, gravity/jumping and
  nonblocking actors; wire v7 jump input. [Scope](KINEMATIC_CONTROLLER.md).
  Live chunk/actor activation remains open.

- [x] Real Bevy original-chunk publication diagnostic: one background candidate,
  standard GLB hierarchies, live Avian body/BVH agreement, retained roots and
  screenshot-proven replacement, failed-asset rollback and unload. [Scope](CHUNK_PRESENTATION.md).
  Normal tile partition/adoption, actor activation and production streaming remain.

- [x] Isolated bounded chunk-residency precursor on original geometry: signed
  ownership seams, real collision load/unload, protected-owner and corrupt-package
  rejection, and distant query frames. [Scope](CHUNK_RESIDENCY.md). Normal tile
  partitioning, client visual consumption and asynchronous streaming remain open.

- [x] Bounded content-package identity slice: versioned manifest, stable logical
  IDs/revisions, independent collision/visual verification and stale-revision
  admission rejection. [Contract](CONTENT_PACKAGES.md). Chunk ownership/residency,
  general collision and character/skeleton contracts remain open.

- [x] Bounded eight-peer Renet scheduler qualification at 20/30/60 Hz, input
  coalescing/expiry/budget enforcement, controlled stall/recovery and bounded
  simulation/pump timing diagnostics. [Evidence](SCHEDULER_QUALIFICATION.md).
  Full-map, crowd, action-command and sustained-overload capacity remain open.

- [x] Automatic client render/static-Avian rebasing, canonical root placement,
  stable global interpolation and same-frame camera/query verification.
  [Scope](FLOATING_ORIGIN.md). Server/query-scene rebasing and multi-chunk content
  ownership remain open.

- [x] Bounded runtime coordinate/identity slice: checked global authority, explicit
  epoch/origin wire v6, relative replication rebases and stable local presentation.
  Multi-chunk ownership and server query-scene rebasing remain open.
  [Contract](COORDINATES_IDENTITY.md).

- [x] Renet local replacement gate: measured real UDP delay/loss/duplicate/reorder,
  outages, bounded queues, captured stale-session packet rejection and default
  launcher switch. [Qualification](RENET_QUALIFICATION.md).

- [x] Executable Renet bridge for server/client/launcher, secure local tickets,
  world matching and authority lifecycle. Production identity, Internet security
  and load qualification remain open. [Scope](NETWORKING_RENET.md).

- [x] Pin Bevy 0.20-compatible Avian development revision and migrate normal
  client plugins/colliders plus shared server/offline movement, ground and camera
  queries. Retain grounded/profile limits. [Scope and evidence](PHYSICS_AVIAN.md).

The milestones below preserve the earlier walking-prototype plan and evidence.
Their former next-step ordering is superseded by the engineering queue. Keep the
map and building fixtures as regressions while replacing the general movement
foundation; adding more individually authored interiors is not the default path.

## Earlier first playable milestones

These are game outcomes, not a continuation of the old engine's M0–M6 schedule.

## Bootstrap — runnable local client

- [x] Independent Rust workspace and pinned Bevy application.
- [x] Original graybox scene, lit geometry, local movement and follow camera.
- [x] Controls, reset/quit, scripted movement and rendered screenshot path.
- [x] Execute native qualification and record limitations.

## First multiplayer slice — two players walking together

- [x] One local Rust authoritative server with a fixed simulation tick.
- [x] Review/reuse suitable protocol and Quinn code from the old project, preserving
  provenance and relevant tests. Do not copy the whole engine or assume its
  experimental protocol is production-ready.
- [x] Two independently launched clients connect, receive distinct player identities
  and see each other's server-owned movement in the same scene.
- [x] Clients submit bounded input intents; server controls speed, bounds and state.
- [x] Basic remote interpolation, direct local server poses, disconnect/reconnect
  and malformed/stale input tests. Rendered poses agree with server snapshots.
- [x] Repeatable local two-client/server verification with actual GPU screenshots.
- [ ] Local prediction/reconciliation when needed for responsiveness; qualify
  jitter/loss/delay and recovery. Direct server poses currently provide correction.
- [ ] Manual two-client walkthrough on request; current GPU checks use an owned
  private headless display. Automatic session resume is not implemented.

## Representative game scene

- [x] Enable ordinary Bevy glTF loading and an independent bounds/scale inspector.
- [x] Qualify the user's geometry-only house GLB and move it into the repo as a
  compatibility reference. No authored UVs/materials/textures exist in that file.
- [ ] Qualify textured static assets and animation loading with original or
  appropriately licensed character, terrain, building and props.
  Local compatibility checks now render the user's textured farmhouse and character;
  stand/walk/run playback works in the inspector. The character export still fails
  strict glTF validation. Representative game content remains a separate step.
- [x] First explicit reference placement with conservative server collision:
  textured farmhouse inside the shared solid envelope, human players and a
  nonblocking idle wolf placeholder. [Scope and checks](REFERENCE_SCENE.md).
- [ ] Qualify final representative asset placement and colliders; viewing imported
  meshes does not automatically create server collision or usable interiors.
- [x] First static-world collision pass for the flat procedural scene: solid
  building, fixed door and rocks. Rendering and authority share authored geometry.
- [x] Orbit/zoom camera, camera-relative movement and static camera obstruction.
- [x] First bounded terrain-only walking patch: shared source triangles, server-owned
  height/slope checks and two rendered multiplayer clients. [Scope](TERRAIN_WALKING.md).
- [x] Animated local/remote human players on the bounded terrain patch, with actual
  stand/walk/run sampling alongside server-owned height.
- [x] One authored farmhouse/foundation on terrain with shared server/camera solid
  boundaries and actual connected contact checks. [Scope](TERRAIN_STATIC_OBJECTS.md).
- [x] One enterable farmhouse ground floor: measured open doorway, fixed ramp,
  source floor support and networked entry/exit with interior wall/camera checks.
  [Scope](ENTERABLE_HOUSE.md).
- [ ] Qualify general authored-world collision alongside imported assets; steps,
  jumping and general mesh colliders remain outstanding.
  The Ruins of Lordaeron terrain and 846 top-level placements plus 624 nested props
  now have a qualified [static visual assembly](MAP_OBJECTS.md). Exported liquid
  geometry also has a [static PBR preview](MAP_LIQUIDS.md). The
  [full development world](DEVELOPMENT_WORLD.md) now connects all source terrain
  and the complete assembly to authoritative outdoor movement, conservative object
  envelopes and blocked water cells. Precise mesh collision, interiors and swimming
  remain outstanding.
- [x] First movement-driven character animation states: human stand/walk/run for
  local and remote players, alongside the existing controls/connection HUD.
- [x] Open the original full-map farmhouse ground floor with source floor support,
  measured rotated walls, retained furnishing collision, a fixed ramp and outdoor
  approach spawns. Improve static-mesh camera clearance and shaded visibility.
  [Scope and checks](MAP_FARMHOUSE.md).
- [x] Retain farmhouse access and open the original blacksmith main room, with
  source floors, a fixed ramp, wall contact and exterior forge/bay exclusions.
  [Scope and checks](MAP_BLACKSMITH.md).
- [ ] Animation blending/combat states and the gameplay target/health/ability HUD.
  Environment geometry stays static, including doors. Players do not physically
  collide with either players or NPCs; targeting/hit detection is separate.
- Define an asset layout and stable game identifiers; avoid custom cooked formats
  until a measured requirement needs them.
- Verify the same small scene and connection flow on native Windows.

- [x] Make the complete supplied tile the outdoor development world, with animated
  multiplayer players, bounded terrain/prop lookup, conservative static collision,
  water exclusions and a single local launch command. Buildings remain solid.

## Gameplay loop, then measured scale

Add a server-owned interaction/combat loop, then persistence and accounts when the
playable experience needs them. Increase entities/clients while measuring actual
simulation, replication, asset and rendering costs. Interest management, larger
world streaming and services follow observed constraints. No MMO-scale capacity
is claimed by the bootstrap or first two-client milestone.

The intended game is WoW-style: begin with a targetable NPC, one server-validated
ability, health/cooldown, then one quest and reward before broadening systems.
See [GAME_DIRECTION.md](GAME_DIRECTION.md) for the small starting-zone outcome.


Forest geometry prerequisite: [calibrated source poses and five-instance native
assembly](UNITY_GEOMETRY.md) now qualify. Full binary terrain import, vegetation,
scene coverage and authoritative collision still precede replacement-map adoption.


Forest terrain prerequisite: [full-sample binary terrain decoding and native
ground preview](UNITY_TERRAIN.md) now qualify. Matching bounded collision,
authored terrain normals/material fidelity and full vegetation/scene/multiplayer
remain before replacement-map adoption.


Forest ground prerequisite: [full-sample authoritative collision and native
multiplayer ground](FOREST_TERRAIN_COLLISION.md) now qualify in four opt-in
packages. Full vegetation/materials and object collision remain before forest
adoption; the default town remains available.


Forest visual continuation: [LOD0 full-shader tree/fern prototypes and actual
planted subset](FULL_FOREST_FOLIAGE.md) now qualify. Complete material variants,
other shader families, authored object collision and scene adoption remain open.


Forest P1-10/P2-08 bulk foliage continuation now qualifies 15,156 source draws in
46 exact mesh/material bindings, actual scene overrides and a shared-image visual
assembly. Independent complete source/category/material/tangent checks and two
native Bevy/Vulkan views pass; 3,503 draws retain explicit unresolved reasons.
Continue remaining Standard/other shader families and mixed slots, then review
object collision and resource budgets before full multiplayer/default adoption.
Town and opt-in forest-ground remain available. [Evidence](FOREST_FOLIAGE_BATCH.md).


Forest P1-10/P2-08 Standard-material continuation qualifies another 66 bindings /
2,744 source draws, bringing the static subset to 17,900 draws with 31 shared
image variants. Canonical mesh-local export excludes imported FBX node placement;
source normal directions, exact palettes, full scene poses and two native Bevy/
Vulkan views qualify. 759 draws remain explicit. Continue four model variants,
other shader families/mixed slots, then object collision and resource/multiplayer
acceptance before adoption. [Evidence](FOREST_STATIC_BATCH.md).


Forest P1-10/P2-08 canonical model-variant continuation now qualifies all four
branch models / 179 poses, with inherited IDs, raw nested/outer material bindings,
independent ufbx/source geometry and byte-preserved prior content. The combined
18,079-draw scene passes complete source and native Bevy/Vulkan checks. 580 draws
remain explicit; continue triplanar cliffs/rocks and other shader/mixed-slot work,
then object collision and resource/multiplayer acceptance before adoption.
[Contract](FOREST_VARIANT_GEOMETRY.md). The user queues the new Elven Realms
folder under Downloads / Synty / Biomes only after forest completion; its contents
have not yet been verified.


Forest P1-10/P2-08/P2-10 triplanar prerequisite now has a source-hash-bound world
projection profile and opt-in Bevy StandardMaterial extension. Actual canonical/
transformed mesh cases pass native lit/diagnostic rendering and independent GPU
pixel checks. The main subset remains 18,079 draws with 580 unresolved. Before
batch adoption, calibrate the exposed FBX mesh/root-pivot offset (including existing
LOD-bound evidence), then integrate multiple profiles, f64 rebase texture phases,
resources/mips and complete scene/object collision acceptance. No automatic mesh
centering or default-map change. [Contract](FOREST_TRIPLANAR.md). Elven folder
inspection remains queued after forest completion.

## Forest source sky — 2026-10-21

Forest sky continuation, 2026-10-21: the source skydome now joins the qualified
subset: **18,651 source draws / eight unresolved**. Actual Editor geometry, source
world-height gradient and placement pass independent checks. Standard unlit and
explicit Bevy shadow markers pass four native Vulkan cases; three legacy modes,
two native failure paths and encrypted town/farmhouse regressions pass. Fresh fmt,
denied-warning Clippy, 38 client Rust and 193 Python tests pass. Three Khronos
files have zero errors/seven retained warnings. Seven water draws and one emissive
willow remain, followed by lighting, rebase/resources/mips, collision and adoption.
All work stays local. [Evidence](FOREST_SKY.md).

## Forest static water — 2026-10-21

Forest water continuation, 2026-10-21: four bindings / seven source water draws
join the qualified visual subset: **18,658 source draws / one unresolved**. Actual
Linux Unity proves built-in Plane identity/geometry; exact Editor FBX triangles,
raw source poses and explicit static PBR factors pass independent checks. Full
composition, eleven native Vulkan cases and four rehashed faults pass. Ten Khronos
files have zero errors/seven retained warnings. Fresh fmt, denied-warning Clippy
and 197 Python tests pass; runtime binaries remain unchanged. Emissive willow,
source water effects, lighting, rebase/resources/mips, collision and adoption remain
open. All work stays local. [Evidence](FOREST_WATER.md).

## Forest emissive/static coverage — 2026-10-21

Forest emissive continuation, 2026-10-21: the willow completes **18,659 planned
static source draws / zero unresolved static draws**. Source mask HDR emission
uses standard glTF strength 2.6 with an explicit base-floor preview omitting spatial
colour noise. Exact source/Editor geometry, full composition, seven native Vulkan
cases, four rehashed faults and encrypted town/farmhouse regressions pass. Fresh
fmt, denied-warning Clippy, 38 client Rust and 201 Python tests pass; four Khronos
files have zero errors/seven retained warnings. Source effects/particles, lighting,
resources/rebase/mips, object collision and normal map adoption remain open. Next:
qualify shared images against the measured duplicate-image inventory. All work
stays local. [Evidence](FOREST_EMISSIVE.md).

## Forest shared image resources — 2026-10-21

Forest resource continuation, 2026-10-21: the complete static scene now shares
**37 actual core image handles instead of 77** across 298 slots. Loaded CPU Image
payloads reduce 3,124,764,672 → 1,421,877,248 bytes; both paired native views are
pixel-identical outside the HUD. Complete 944-accessor/material/PNG/role/sampler
checks, four rehashed faults and three legacy modes pass. Fresh fmt, denied-warning
Clippy, 38 client Rust and 202 Python tests pass; Khronos has zero errors/seven
retained warnings. All 18,659 static source draws remain covered. This is inspector
sharing; package admission stays unchanged. Lighting, authored collision, rebase/
mips and runtime adoption remain open. All work stays local.
[Evidence](FOREST_RESOURCES.md).

## Forest source lighting boundary — 2026-10-21

Forest lighting continuation, 2026-10-21: actual Unity imports qualify source
renderer layers and cookie alpha; all 26 offline source lights are preserved.
Native pixels expose unsupported selective receiver masks and a disabled cookie
feature. The client enables standard Bevy light textures and rejects selective
masks; a separately verified inspector copy explicitly omits three masked suns,
retaining 22 point lights, one spotlight, ambient and fog. Two full-scene and two
cookie GPU cases, three legacy modes, three failure paths and encrypted town/
farmhouse regressions pass. Four rehashed faults reject. Fresh fmt, denied-warning
Clippy, 41 client Rust and 204 Python tests pass. This is an explicit lighting
approximation; selective suns, authored collision, rebase/mips and normal forest
adoption remain open. Next: source collider geometry and measured package capacity.
All work stays local. [Evidence](FOREST_LIGHTING.md).

## Forest authored collision reference — 2026-10-21

Forest authored-collision reference, 2026-10-21: actual Linux Unity now covers
all 79 raw prefab mesh references, including 39 new collision FBXs and 17 exact
serialized meshes. The actual scene and independent raw hierarchy agree on all
2,776 collider components, with 2,763 active/enabled and maximum matrix difference
0.046 mm. Native targeted rays provide 16,578 reference cases. Four rebound
pose/mesh/state/capsule faults reject; 213 Python tests pass. The invisible downward
water collider, nonuniform primitives, mirrored/sheared meshes and full terrain
budgets remain explicit constraints. Next: native Avian object-query comparison
and measured representation/capacity. Runtime object admission and forest adoption
remain open. All work stays local. [Evidence](FOREST_COLLIDER_REFERENCE.md).

## Forest Avian source geometry — 2026-10-21

Forest Avian geometry continuation, 2026-10-21: a real headless Bevy/Avian
world matches 2,762 active source object colliders across 16,572 targeted rays,
maximum distance difference 0.27 mm. A float64 source-surface oracle separately
proves all 17 convex inputs are closed hulls. Four rehashed expected-hit/distance/
normal/mesh faults reject in actual Avian. Strict Unity ray equality remains a
recorded diagnostic failure because cooking, backfaces and sheared-mesh behaviour
differ; it is not promoted as parity. Fresh format/diff checks, denied-warning
workspace/all-target Clippy and 215 Python tests pass. Terrain, package budgets,
water policy, movement/camera and forest adoption remain separate gates. Next:
measure the source representation against existing runtime package limits. All
work stays local. [Evidence](FOREST_COLLISION_GEOMETRY.md).

## Forest full-sample heightfield prerequisite — 2026-10-21

Forest heightfield continuation, 2026-10-21: standard Avian full/adjoining
heightfields retain every source sample and oriented triangle surface. All
2,362,371 source-vertex/centroid queries and 54 capsule seam route runs at 20/30/60
Hz pass against retained triangle packages; maximum heightfield error is 7.16
micrometres. Rectangular axis/diagonal checks pass; two rehashed sample/scale faults
reject. Fresh fmt, denied-warning workspace/all-target Clippy and 217 Python tests
pass. Offline collision-only layout measures at most 47,245 pooled input triangles,
259,623 instantiated triangles, 66,049 height samples per provider and eight
contributors per finite terrain cell. These are not runtime capacity/publication
claims. Next: bounded versioned shape/heightfield admission; existing packages and
defaults remain preserved. All work stays local. [Evidence](FOREST_HEIGHTFIELDS.md).
