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

Ability-damage prerequisites, 2 November 2026:
[Special outcomes, impact and exact profile binding](../CLASSIC_ABILITY_DAMAGE.md)
pass 241 core and 231 network library tests, formatting and workspace Clippy.
These remain inactive; normal Sword Slash and Arcane Bolt still use fixed effects.
Opt-in World/RNG/life/projectile adoption now passes 241 core / 572 network
(ten skips) / 118 client tests and workspace Clippy. Preserve
`build/classic-ability-world-qualified-v1-20261102`. Next: CE08, UDP and normal/native
adoption; typed ability facts are currently refused by all older codecs.
CE08 now passes 241 core / 580 network (ten skips) / 118 client tests and workspace
Clippy; `build/classic-ability-codec-qualified-v1-20261102`. Actual CE08 UDP and
normal application/native adoption remain next.
Actual CE08 encrypted two-peer impaired UDP now passes at 20/30/60 Hz, with
241 core / 583 network (ten skips) / 118 client full regression and workspace
Clippy; `build/classic-ability-transport-qualified-v1-20261102`. Fixture inventory
is synthetic; normal OIDC/PostgreSQL/native replacement acceptance is next.
The ordinary-melee playable checkpoint below remains preserved. Changes are local.

Ordinary melee playable checkpoint, 1 November 2026:
[Ordinary attack tables and normal client adoption](../CLASSIC_ATTACK_TABLES.md)
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

Inactive [ability starting-damage prerequisite](../CLASSIC_ABILITY_DAMAGE.md):
231 core / 556 network (ten skips) / 118 client tests and workspace Clippy pass.
Explicit AP-only normalization, scale/flat ordering and bounded candidate recipes
are qualified; normal abilities still use fixed effects. Separate [spell landing and conditional critical](../CLASSIC_SPELL_DAMAGE.md)
now pass 235 core tests and workspace Clippy; network/client baselines are inherited.
Main-hand-special outcomes and explicit mitigation remain before authority adoption.

Shared physical damage playable checkpoint, 1 November 2026:
[Normal physical damage authority and client adoption](../CLASSIC_DAMAGE_AUTHORITY.md)
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

Attack-table prerequisites: [ordinary melee table and eligibility](../CLASSIC_ATTACK_TABLES.md)
and authoritative creature facing pass 202 core / 516 network / 115 client tests
and workspace Clippy. Explicit outcome-aware World activation now also passes
224 core / 539 network (ten skips) / 115 client tests and workspace Clippy;
`build/classic-white-world-qualified-v1-20261101`. New-profile codec, UDP and
normal/native adoption remain pending. Actual CE07 encrypted two-peer UDP now
passes full regression: 224 core / 551 network (ten skips) / 115 client tests and
workspace Clippy; `build/classic-white-transport-qualified-v1-20261101`. The playable checkpoint above
remains unchanged.

Classic ability rules, 1 November 2026:
[Manual facing, cast interruption and weapon-timer rules](../CLASSIC_ABILITY_RULES.md)
pass 156 core / 479 network (ten skips) / 114 client / 37 Python checks,
workspace Clippy and six normal OIDC/PostgreSQL native runs at actual three sizes.
All have zero GPU diagnostics; disconnected-display uncapped/windowed conditions
are explicit. The strict development profile now demonstrates thirteen checks;
full Classic damage formulas remain pending. Current source/art/binaries/proof:
`build/classic-ability-qualified-v1-20261101`. Next: [damage resolution](../CLASSIC_DAMAGE_RESOLUTION.md).
Changes remain local and unpushed.

Damage arithmetic follow-on: [starting damage and provisional prevention](../CLASSIC_DAMAGE_RESOLUTION.md)
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
[Explicit Renet engagement/state/events](../CLASSIC_MELEE_TRANSPORT.md) now
pass focused encrypted UDP checks at 20/30/60 Hz under loss, jitter and
duplication, plus presentation-queue backpressure. Formatting, workspace Clippy,
153 core / 464 network passes (ten skips) / 113 client tests pass. Exact source
and proof: `build/classic-melee-transport-qualified-v1-20261101`.
Normal application right-click/animation adoption remains pending;
playable checkpoint and strict milestone remain unchanged. Local and unpushed.

Broader world-system direction, 1 November 2026:
[Classic systems, repo assessment and working defaults](../CLASSIC_WORLD_SYSTEMS_REFERENCE.md)
now cover combat state/resolution, tagging/credit, spawns, linked encounters,
resources, control effects, navigation and corpse recovery. These are scoped
plans and policy choices; the assessment separates them from implemented code.
Finish the current strict combat slice before expanding full-zone content.
Current [headless melee authority](../CLASSIC_AUTOATTACK_AUTHORITY.md) passes
153 core / 458 network / 113 client tests; secure transport/client adoption is
next. The playable targeting checkpoint and five-of-thirteen milestone remain.
Changes stay local and unpushed.

Headless basic-melee authority, 1 November 2026:
[World weapon-timed combat](../CLASSIC_AUTOATTACK_AUTHORITY.md) passes
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
[weapon data and independent swing timing](../CLASSIC_MELEE_CLOCK.md) pass
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
[persistent selection and forward/reverse Tab](../CLASSIC_TARGETING.md) pass
146 core tests, 445 network passes / ten skips, 113 client tests, 36 Python checks,
workspace Clippy and three final normal native starts with zero GPU diagnostics.
Selection survives interest/terrain unloading and turning; explicit removal and
clear remain authoritative. The classic profile identity advances to v2.
The strict combat milestone is five of thirteen checks; next is
[weapon-timed melee autoattack](../CLASSIC_AUTOATTACK_PLAN.md), then ability
facing/interaction rules. Metric scale remains unchanged. Dodge stays deferred.
Qualified source/binaries/proof: `build/classic-targeting-qualified-v1-20261101`.
Changes remain local, uncommitted and unpushed. Earlier entries retain their
historical scope; they do not supersede this checkpoint.

Current classic development checkpoint, 1 November 2026:
[classic controls/manual facing](../CLASSIC_MOVEMENT.md) pass 111 client and
45 Python checks, workspace Clippy and seven final native starts with zero GPU
diagnostics. The unchanged shared sources retain 142 core / 444 network passes.
Use `--classic`; dodge/action camera are disabled there. Movement/UI checks pass
at all three sizes, including an A/D-strafe settings file at 720p. Next: persistent
targeting and forward/reverse Tab, then weapon-timed autoattack and combat-facing
validation. [Enemy AI reference](../ENEMY_AI_REFERENCE.md) is recorded for
subsequent threat/evade work. The full combat milestone remains incomplete.
Changes remain local, uncommitted and unpushed.

# Current direction — strict WoW Classic foundation, 1 November 2026

The user adopted [THEIA-COMBAT-001 v1.0](../COMBAT_TARGETING_MOVEMENT_SPEC.md),
superseding the hybrid camera/movement plan. Initial controls require
character-relative movement, manual authoritative facing, persistent explicit
targeting and weapon-timed autoattack independent of abilities. Dodge, action
camera and reticle targeting are deferred. Dodge implementation remains stopped.

The [repository gap assessment and migration order](../MOVEMENT_CAMERA_TARGET.md)
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
[Controls, proof and limits](../DODGE_MOVEMENT.md).
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
[Evidence and remaining normal-client gates](../DODGE_MOVEMENT.md).
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
[Behavior, evidence and remaining gates](../DODGE_MOVEMENT.md).
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
[Controls, evidence and limits](../PLAYER_CONTROLS.md).
The previous camera checkpoint separately retains actual two-client melee and
projectile proof; those starts were not rerun against this panel revision.
Shared dodge replay is headless-qualified; live dodge remains uninstalled.
No new full-workspace, sustained FPS/capacity, Windows or physical-input claim.
Changes stay local and unpushed. Historical entries below retain their dates.

# Current playable checkpoint — 2026-11-01: optional action camera

P2-04 / P2-05 / P2-07 / P2-11, P3-06 / P3-14: F8 toggles action camera,
Left Ctrl temporarily releases the cursor, panels/focus suspend mouse-look,
and a raised collision-bounded aiming pivot keeps the player below the reticle.
Classic left-orbit preserves held travel; reticle clicks reuse checked targeting.
**97 client tests**, **407 network passes / 10 explicit skips / 63 groups**,
formatting, denied-warning workspace/all-target Clippy and **41 Python checks**
pass. All **seven final native starts have zero GPU diagnostics**, including
OS-synthetic input at 1280×720, 1920×1200, 3440×1440 fullscreen and actual
OIDC/PostgreSQL two-client melee/projectile regressions.
[Behavior, exact evidence and limits](../PLAYER_CONTROLS.md).
Matched source/binaries/art/proof: `build/camera-mode-qualified-v1-20261101`.
Earlier checkpoints remain. No new full-workspace, FPS/capacity, Windows or
physical-hardware claim. Live dodge remains uninstalled. Next: panel/input
refinements and shared dodge prediction. Changes are local and unpushed.

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
[Behavior, exact evidence and limits](../PLAYER_CONTROLS.md). Matched source/binaries/art/proof:
`build/world-input-qualified-v1-20261101`. The previous whole-workspace result
remains 608 passes / 40 explicit skips; it is not a new whole-workspace run.
Earlier checkpoints remain preserved. No new FPS/capacity or Windows claim.
Dodge is the next separate authoritative movement slice. Changes remain local;
no commit or push was made.

# Current checkpoint — 2026-10-31: existing-account character menu qualified

P3-03: the console launcher lists existing characters and offers N to create
another below the eight-character cap. A known incompatible-profile refusal returns
to selection without rewriting saves; temporary retirement conflicts keep their
bounded retry. Actual OIDC creation on an existing account, incompatible-to-compatible
menu recovery, two native players, rewarded gameplay and database/server restart
pass at 1920×1200 windowed with zero Vulkan diagnostics. Normal eleven-profile
launcher startup/cleanup passes all three modes. [Contract and proof](../CHARACTER_LAUNCHER_MENU.md).

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
bounded-queue preflight tests. [Contract and proof](../CHARACTER_PROFILE_PREFLIGHT.md). Matched binaries,
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
[Exact scope, prerequisites and proof](../QUEST_TURN_IN_XP.md). Immutable source, matched optimized
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
[Scope, fixture settings and limits](../PERSISTENT_ZONE_INTEGRATION.md).

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
hashes. [Tool, acceptance and limits](../GAMEPLAY_CONTENT_VALIDATION.md). Runtime profile loading stays as
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
[Exact current behavior, evidence and limitations](../VENDOR_BARTER_FOUNDATION.md).

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
another charge; superseded/conflicting input rejects. [Exact scope/proof](../VENDOR_BARTER_FOUNDATION.md).

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
observed choice/revision. [Exact source/evidence/limits](../NPC_DIALOGUE_FOUNDATION.md).

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
[Exact behavior, evidence and limits](../QUEST_OBJECTIVES_FOUNDATION.md).

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
the bounded [P3-12 dialogue foundation](../NPC_DIALOGUE_FOUNDATION.md), currently
plan only. General quest acceptance, dialogue/turn-in rewards and vendors remain
open. P3-17/18 remain unspecified. All changes remain local/uncommitted; concurrent
story edits are untouched. Nothing was pushed. Earlier notes below are historical.

# Active development — 2026-10-30: P3-11 objective core/database prerequisite qualified

One checked development objective and its durable kill receipts now share an
atomic transaction with enabled XP. Completion stays finite; unfinished objectives
can advance after XP caps. Existing XP-only callers retain their own transaction.
[Exact scope and next acceptance](../QUEST_OBJECTIVES_FOUNDATION.md).

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
[Exact behavior and limits](../EXPERIENCE_LEVELING_FOUNDATION.md).

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
[Window options and exact display evidence](../DEVELOPMENT_WORLD.md).

Linux RX 7900 XTX / RADV / Mesa 26.2.6 / Vulkan was inspected. Windowed validation
is clean; fullscreen records one specifically documented wgpu resize diagnostic
per startup and rejects other errors. The same release client passes a validation-
off, single-client empty-world tour at **808.60 FPS uncapped, 1920×1200**, with
834 loaded / 753 retired chunks and zero missed server simulation deadlines.
This is not crowded MMO capacity or a native Windows result.

Exact source, binaries and evidence: `build/progression-qualified-v1-20261030`.
Prior reward/core-store checkpoints remain preserved. Next is the bounded
[P3-11 objective foundation](../QUEST_OBJECTIVES_FOUNDATION.md); no quest
code or journal is implemented yet. P3-17/18 remain unspecified. Changes are local/
uncommitted; concurrent story edits are untouched. Nothing was pushed.
Earlier checkpoint notes below are historical.

# Paused — 2026-10-30: P3-10 experience core/database prerequisite qualified

Stopped at the user's request after finishing this prerequisite. Checked progression
profiles, integer level/XP proposals and durable atomic victim-life receipts are
implemented on the existing bounded async worker. Old-event replay returns current
committed progress without another award; cap, rollback, cancelled replies, queue
saturation, competing recipients, backend loss and real restart checks pass.
[Exact scope and remaining acceptance](../EXPERIENCE_LEVELING_FOUNDATION.md).

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
[Exact behavior and limits](../LOOT_REWARDS_FOUNDATION.md).

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

Next is [P3-10 experience and leveling](../EXPERIENCE_LEVELING_FOUNDATION.md):
its proposed small curve, durable kill receipts, retirement and independent own-
character UI contract are documented. No progression code/award/UI is implemented
in the reward checkpoint. Broader loot/groups/instances and production balance
remain separate work; P3-17/18 stay unspecified.

Earlier checkpoint notes below are historical.

# Active development — 2026-10-29: initial status-effect ability qualified

P3-06's first opt-in Scorch slice is qualified: checked definitions, server-owned
periodic damage and refresh, bounded independent sources, source/target cleanup,
encrypted status observations and standard Bevy selected-target indicators.
[Behavior, exact evidence and limitations](../STATUS_EFFECTS_FOUNDATION.md).

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
prerequisite are now in [offline bake/route prerequisite work](../CREATURE_AI_FOUNDATION.md);
offline central/far saved-terrain packages and headless patrol timing/replication
checks pass; client integration and native qualification are in progress. P3-17/18 stay unspecified. Changes remain local
and concurrent story edits are untouched. Earlier checkpoints below are historical.

# Active development — 2026-10-29: initial durable backpack qualified

P3-05's first opt-in backpack slice is qualified: checked item definitions and
sixteen slots, conserved move/merge/swap/split proposals, durable atomic mutations,
latest-intent recovery and standard Bevy widgets. Neither queue acceptance nor a
client-side proposal is acknowledged as a committed item change.
[Exact behavior, proof and limits](../INVENTORY_FOUNDATION.md).

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
preserved. [Resources and their separate performance proof](../STATS_RESOURCES.md).

This is the first development backpack, not all of P3-05: equipment, instances,
item use/destruction, currency, trade, vendors and rewards remain open. Durable
items do not persist transient vitals/combat or complete the persistent zone.
Production account hardening and native Windows remain open. Next is the scoped
[P3-06 status-effect ability](../STATUS_EFFECTS_FOUNDATION.md); its checked definitions, authoritative timing and encrypted two-client replication
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
closed. [Exact behavior and evidence](../PERSISTENCE_FOUNDATION.md).

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
[AUTHENTICATION_FOUNDATION.md](../AUTHENTICATION_FOUNDATION.md). The provider
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
movement/targeting/damage/death/respawn/critical cues. [Exact evidence](../COMBAT_UI.md).

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
[Exact native evidence, hashes, prior failure and limits](../SPELL_FEEDBACK.md).

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

# Current task checkpoint — 2026-10-26

Movement input diagnostics pass 236 full workspace tests and an uncontended
600-second eight-peer soak with zero simulation/I/O misses. The P1-11 ability-data
prerequisite passes 16 core tests, actual offline validation and Clippy/fmt.
P2-11's server-owned targeting prerequisite passes four actual-World tests;
the combined full regression now passes 245 tests and Clippy/fmt. Network/UI adoption remains open.
See [input evidence](../PREDICTION_INPUT_DIAGNOSTICS.md),
[ability data](../ABILITY_DATA.md) and [targeting](../TARGETING_FOUNDATION.md).
Earlier stop records are historical; current work remains local.

# Current checkpoint — stopped safely, 2026-10-26

The user requested stopping. No soak is running; the final diagnostic full-suite
relink was safely interrupted. Changes remain local and uncommitted.

The actual 600-second, eight-peer saved-terrain soak passes identity return,
bounded residency and zero missed simulation deadlines. Separate I/O pump misses
under concurrent builds remain recorded. [Soak evidence](../SAVED_TERRAIN_SOAK.md).
The poor-link test passes movement/convergence at 20/30 Hz and safe suspension
outside the 60 Hz latency window. [Envelope](../PREDICTION_LATENCY.md). That checkpoint
passes 235 workspace tests (ten private-fixture skips), denied-warning Clippy/fmt,
221 asset Python tests and four GLB alignment tests.

The subsequent bounded input-agreement diagnostics are implemented; all 11 focused
history tests and workspace/all-target denied-warning Clippy pass. Their final
full-suite/real-UDP run was interrupted, so these new counters have no completed
runtime qualification. [Diagnostic checkpoint](../PREDICTION_CONTRACT.md). Resume with
that verification before interpreting the remaining correction outliers. No next
feature is active; no native Windows or MMO population capacity is qualified.

Current continuation, 2026-10-23 — P1-10/P2-06/07 and initial P3-13:
[saved continental terrain](../SAVED_TERRAIN.md), with the existing coordinate,
controller, readiness and prediction prerequisites. Acceptance is recorded there;
saved terrain now passes full runtime/default adoption checks. The user resumed work.
The matched-tick/render diagnostics and bounded rate prediction clock also pass
234 Rust tests and native Linux comparisons; [clock evidence](../PREDICTION_CONTRACT.md).
The actual eight-peer, 180-second headless saved-terrain soak passes;
[scope, remaining checks and safe stop](../SAVED_TERRAIN_SOAK.md). Work is stopped
at the user’s request; changes remain local and uncommitted.
Earlier queues/stops below are dated historical context. No region/shard service
or complete P3-13 acceptance is implied.

Current bounded slice, 2026-10-22 — P1-03/09/10/12, P2-05/06/07/08/09:
[continental graybox adoption](../GRAYBOX_WORLD.md) passes headless and native
Linux checks. Default launcher selects the 256 km² procedural world and POLYGON
player; bounded visual/collision preparation, GPU readiness, culling/retirement,
full native crossing, distant teleports/revisit and local/remote presentation
smoothing are implemented. Release predicted single-client 720p crossing: 821.70 FPS;
this is an empty-world baseline, not MMO population capacity. Whole Pn tasks and
broader P2-04 qualification remain open. Forest work is frozen. Impaired-UDP
traversal/readiness and the input-tick/kinematic-state prerequisites now pass.
The actual Renet authority/client now execute completed-step commands and bounded
reconciliation. Native impaired-UDP, held-jump/teleport and full-continent checks
pass; the graybox launcher selects prediction with a retained `--no-prediction`
baseline. This closes the bounded Linux slice; broader P2-04 qualification remains. [Contract and limits](../PREDICTION_CONTRACT.md).

# Live engineering task status

Updated 2026-10-28. This is the active status overlay for all 112 imported task
entries. The supplied register stays unchanged for provenance. “Partial” and
“Precursor only” do not satisfy the original task's acceptance criteria. No phase
or wave is complete. Two source tasks remain unspecified.

Use [ALIGNMENT](ALIGNMENT.md) for implementation evidence and remaining gates,
[the index](README.md) for the immediate queue, and the imported
[plan](IMPLEMENTATION_PLAN.md) for phase/wave scope. Refine actual dependencies and
specific behavioural/failure/performance acceptance before implementing each task;
list order is not a dependency graph. Follow [execution order](EXECUTION_ORDER.md). Scheduler work is P1-05, inventory P3-05,
and loot P3-09. Do not invent scope for P3-17/18.

## Phase 1

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P1-01 | Repository audit and reusable/obsolete classification | Audit delivered | Pre-alignment inventory is recorded; generic source acceptance is narrowed to documentation audit. |
| P1-02 | Rust workspace, Bevy client, headless server and shared crates | Partial | Independent workspace and core crate exist; remaining domain boundaries and later gameplay integration are open. |
| P1-03 | World coordinates and spatial conventions | Partial | Live checked global authority and local projection/wire rebasing; chunk/region ID contracts tested. Normal chunk startup supports independent client query frames and guarded common wire/render frames for distant loaded actors. General query-scene rebasing and region ownership remain. [Scope](../CHUNK_STARTUP.md). |
| P1-04 | Entity identity and lifecycle | Partial | Checked incarnation/entity keys and monotonic actor lifecycle active; restart/stale-epoch rejection tested. General entities and persistent identity remain. |
| P1-05 | 30Hz fixed timestep, bounded input queues | Partial | Bounded eight-peer graybox scheduler qualified at 20/30/60 Hz, with burst coalescing, expiry, controlled stall/recovery and budget removal; broader scene/soak qualification remains. See [evidence](../SCHEDULER_QUALIFICATION.md). |
| P1-06 | Command schemas, sequencing, acknowledgements and loss tests | Partial | Bounded intents, monotonic acknowledgements, expiry and controlled impaired UDP tested; incarnation guards now active; eight-peer burst/replay/budget qualification now passes; action schemas and broader traffic qualification remain. |
| P1-07 | Renet integration and validation (replaces comparative transport study) | Partial | Renet is the launcher default after real UDP loss/delay/reorder/outage/RTT/bandwidth/queue qualification; Internet security/load and production sessions remain. |
| P1-08 | Replication protocol contracts | Partial | Wire/ALPN 12 adds explicit terrain frame policy independent of content digest; bounded authority offers and tagged readiness acknowledgements remain. Confirmation requires the exact latest report and a post-application snapshot. General entity/delta/interest/session contracts remain. [Handover](../CHUNK_HANDOVER.md). [Readiness](../CHUNK_READINESS.md). |
| P1-09 | Kinematic movement and collision contracts | Partial | Shared Avian capsule controller drives normal maps, graybox and offline motion. Coverage sweeps constrain input/gravity and missing-corner shortcuts; support validation/fallback handles imprecise convex contacts. Retained actor/recovery validation now gates opt-in paired replacement. Head-on feature-manifold refinement addresses reproduced sloped-wall drift; see current qualification evidence. Schema-2 complete-cell coverage now requires all geometry contributors for input/gravity/readiness. Platforms/prediction and full movement scope remain. [Contributors](../CHUNK_CONTRIBUTORS.md). [Boundaries](../CHUNK_BOUNDARIES.md). [Handover](../CHUNK_HANDOVER.md). |
| P1-10 | Asset/terrain/animation content contracts | Partial | Saved height tiles have a versioned, hashed manifest; offline bake/full-seam verification and worker read validation feed shared visual/collision samples. [Saved terrain](../SAVED_TERRAIN.md). Stable package IDs/revisions, schema-3 general collision/explicit spawns and schema-2 geometry contributors exist. Manifest schema 2 now binds shared PNG variants (bytes/hash/colour space/sampler) with actual URI/path/header verification and standard Bevy loading. The offline extractor preserves source geometry/material/image bytes and stages the actual town separately. Real graph/terrain/collision partitioning, source collision fidelity, skeleton contracts and measured budgets remain. [Shared images](../SHARED_IMAGES.md). [Contributors](../CHUNK_CONTRIBUTORS.md). |
| P1-11 | Data-driven ability/combat schemas | Partial | Bounded immutable schema-1 JSON definitions, mobile/off-GCD rules, damage/range budgets and ceiling tick timing pass five new contract tests, all 16 core tests, denied-warning Clippy and an actual offline validator at 20/30/60 Hz. Bounded server execution, encrypted transport and native ability integration now pass the 272-test/full Linux slice; broader combat schemas remain. [Data](../ABILITY_DATA.md), [authority](../COMBAT_AUTHORITY.md). |
| P1-12 | Tests, instrumentation and benchmark baselines | Partial | Fixed-memory timing quantiles, separate simulation/pump metrics, eight-peer scheduler qualification and a 600-second saved-terrain identity/retirement soak pass. Uncontended simulation and I/O deadlines both pass. Wider populations, authored content, rendered crowd and production/long-soak budgets remain. |

## Phase 2

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P2-01 | Bevy client bootstrap | Partial | Existing Bevy bootstrap qualified; complete target foundation integration before closing. |
| P2-02 | Connection/session management | Partial | Default executable Renet lifecycle and impaired-link/fresh-ticket recovery qualified; automatic resume, provider/character authorization and production sessions remain. |
| P2-03 | Entity replication and lifecycles | Partial | Quinn and executable Renet player snapshots with world actor IDs/acks; incarnation/frame contracts now active; bounded opt-in static targets now include reliable selection/state/removal with actual UDP and native evidence; general entity replication remains. [Runtime](../TARGETING_RUNTIME.md). |
| P2-04 | Prediction and reconciliation | Partial | Actual Renet command/correction profile, 64-step replay, bounded client clock/queues, readiness overlap and stream resets run in the normal procedural client/server. Actual impaired UDP at 20/30/60 Hz and native Linux 30 Hz traversal/camera/animation/teleport/held-jump checks pass. Graybox default adopts it; legacy interpolation remains. Completed-input diagnostics and real poor-link checks now distinguish 20/30 Hz movement/convergence from unsupported-RTT suspension at 60 Hz. Severe-link corrections up to 1.6085 m remain; authored-map prediction, dense load and native Windows remain. [Evidence and limits](../PREDICTION_CONTRACT.md). |
| P2-05 | Third-person camera/controls | Partial | Camera/controls qualified across client origin changes and native graybox prediction; broader authored-map prediction and controls qualification remains. |
| P2-06 | Terrain chunk loading | Partial | Saved continental height files stream through bounded workers with 81 visual roots maximum, actual mesh retirement and 3 × 3 collision patches; full native crossing, teleports and impaired prediction pass. [Saved terrain](../SAVED_TERRAIN.md). Normal opt-in publication protects readiness, actors/recovery and complete geometry contributors. Compatible PNG variants now share actual Bevy image handles; invalid dependencies retain old scenes. Lookahead retains spare packages until new providers require the budget, avoiding unload-only offer churn. Actual graph/terrain/collision partitioning, wider lifecycle checks and measured production budgets remain; town launcher stays monolithic. [Shared images](../SHARED_IMAGES.md). [Handover](../CHUNK_HANDOVER.md). |
| P2-07 | Collision integration | Partial | Shared controller drives normal maps and graybox/offline motion. Paired publication replaces actual Avian bodies, with readiness withheld until live generation/column checks pass. Schema-2 collider envelopes and contributor-complete coverage protect nonlocal walls, actors, recovery and presentation. Source-mesh fidelity and measured production collision/streaming budgets remain. [Contributors](../CHUNK_CONTRIBUTORS.md). [Boundaries](../CHUNK_BOUNDARIES.md). |
| P2-08 | GLB and asset loading | Partial | Standard Bevy GLB/PBR/image loading follows immutable manifest and dependency verification. Actual nested extents gate initial/replacement readiness; shared PNG variants preserve compatible handle reuse and distinct colour/sampler settings. New packages extract original images and geometry without changing the active map. Imported graph pruning/partitioning, skeleton/character contracts and measured asset budgets remain. [Shared images](../SHARED_IMAGES.md). [Contributors](../CHUNK_CONTRIBUTORS.md). |
| P2-09 | Character skeletal animation | Partial | POLYGON Idle/Walk/Run/Jump, authored prop sockets, own-player death/fresh-life restart, public local/remote sword stance and masked moving finger grip, and the independent six-clip wolf are qualified development animations. Production skin fidelity, public remote-player death, full equipped locomotion/attack, blending, layered mobile casting and general skeleton contracts remain. |
| P2-10 | Lighting/materials/atmosphere | Partial | Existing PBR light/shadows and water preview; atmosphere/quality requirements remain. |
| P2-11 | Targeting and interactions | Partial | Optional static training target and authoritative selection/clear now run through real encrypted Renet and standard Bevy presentation. The 253-test regression, impaired two-client native GPU selection/rebase/relevance retirement/return and launcher cleanup pass. Owner removal and malicious traffic pass actual UDP tests; general interactions remain open. [Runtime](../TARGETING_RUNTIME.md). |
| P2-12 | One authoritative ability, damage, death and respawn | Partial | The bounded NPC ability loop passes 272 full workspace tests, real encrypted adverse UDP, two native Linux Vulkan clients and launcher cleanup: mobile casting, range/LOS, queue/cooldowns, shared health, same-step death and fresh-ID respawn. Broader combat/player lifecycle and native Windows remain open. [Authority](../COMBAT_AUTHORITY.md). |
| P2-13 | Spell VFX and audio hooks | Initial slice qualified | Actual encrypted profile-2 events drive bounded original Bevy pulses and spatial voices; native impaired two-client saturation, critical-only quality, live rebases and isolated PCM pass. 278 workspace tests plus final client tests/Clippy/fmt pass. Production content/readability/audio design and native Windows remain open; see `docs/SPELL_FEEDBACK.md`. |
| P2-14 | Health/target/action-bar UI and integrated demo | Initial slice qualified | Standard Bevy clickable action slot, Digit 1 input, target health/cast/cooldown bars and numeric/state alternatives pass actual impaired two-client picking/layout, authority integration, feedback saturation, critical-only and cleanup checks. Full suite: 280 passed, ten private-fixture skips; Clippy/fmt pass. Multi-bar customization, full accessibility/responsive UI and broader player gameplay remain. [Evidence](../COMBAT_UI.md). |

## Phase 3

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P3-01 | Authentication and sessions | Initial account/native slice qualified | Actual two-user HTTP/OIDC/PostgreSQL, proof-bound session collection/cancellation, ownership, revocation/supersession, bounds/deadlines, fail-closed/graceful shutdown and restart pass. 294 workspace tests and final Clippy pass; exact source/executable frozen. Native encrypted ticket/session/character binding, real restart/revocation/outage and browser-command fixture launcher cleanup now pass; 297 workspace tests pass; exact checkpoint evidence is retained. Physical browser, renewal/production transport and native Windows remain open. [HTTP evidence](../ACCOUNT_HTTP_FOUNDATION.md), [native admission](../AUTHENTICATED_ADMISSION.md). |
| P3-02 | PostgreSQL persistence/migrations | Initial slice qualified | Real migrations/upgrade, ownership/world checks, monotonic atomic checkpoints, bounded worker and actual two-native-client server/database restart restore pass. Wrong-world and live backend-loss fail closed; integrated combat/UI/feedback and cleanup pass. 284 workspace tests; exact artifacts/limits in [contract](../PERSISTENCE_FOUNDATION.md). External accounts/economy/Windows remain open. |
| P3-03 | Character creation/selection | Initial database/native/launcher, preflight and console menu qualified | Owned selection now holds its reservation across read-only enabled-profile checks before encrypted ticket issuance. Incompatibility returns a clear nonretryable refusal without failing the worker or rewriting saves. Actual OIDC/native/SQL refusal, fresh entry, rewarded gameplay and restart pass at 1920×1200; 476 workspace tests/40 skips. [Contract and evidence](../CHARACTER_PROFILE_PREFLIGHT.md). Console creation on existing accounts and return-to-selection after incompatibility also pass actual OIDC/native/launcher checks; [menu evidence](../CHARACTER_LAUNCHER_MENU.md). Complete in-game creation/login UI, production provider/renewal and native Windows remain open. |
| P3-04 | Stats and resources | Initial resources and opt-in transient player life qualified; broader scope open | [Resource foundation](../STATS_RESOURCES.md) plus [private player life](../PLAYER_LIFE_NATIVE.md): checked health/mana, cast costs/regen, player damage/death, prepared streamed home respawn and fresh movement streams with standard Bevy UI. Two-account native gameplay/rejoin/restart/backend-loss passes at three sizes; full workspace 525 passes / 40 explicit skips. Persistent vitals, production combat logout policy and class/gear attributes remain open. |
| P3-05 | Inventory and items | Initial durable/native backpack qualified; broader scope open | [Actual SQL/verified-admission/native restart/widget proof](../INVENTORY_FOUNDATION.md). Equipment, instances, use/destruction, currency, trade and reward systems remain separate. |
| P3-06 | Abilities and status effects | Initial status, equipment-gated melee and player hit outcomes qualified | [Sword Slash, schema-4 requirements and native proof](../PLAYER_MELEE.md). [Server-owned hit/miss/deflect and actual native labels](../HIT_OUTCOMES.md). [Authority, encrypted replication, native indicators/feedback and limits](../STATUS_EFFECTS_FOUNDATION.md). Healing/dispels/CC/class scaling/durable combat remain open. |
| P3-07 | Creature AI | Initial single-NPC patrol/pursuit/reset and damage threat qualified; broader scope open | [Pursuit](../CREATURE_PURSUIT_NATIVE.md), [return reset](../CREATURE_RETURN_RESET.md), [damage threat](../CREATURE_THREAT.md): metric current-reach 110%/130% transfer, prepared windup/path invalidation, exact-life retirement, retained membership and separate tag credit. Actual impaired encrypted two-peer transfer at 20/30/60 Hz; schema-13 unmodified native three-kill/quest/restart regression passes; passive zero-threat body pulls also pass at 20/30/60 Hz. Final workspace 1,074 tests pass (50 ignored); preserve `build/creature-threat-client-qualified-v2-20261103` (66 affected authority tests plus inherited full regression). Normal public NPC attack timing/facing is now [qualified](../PUBLIC_CREATURE_ATTACKS.md) through schema 14 and actual native victim/observer views. Multi-NPC/bosses, social/healing/taunt rules, streamed navigation and population capacity remain open. |
| P3-08 | Equipment and appearance | Initial main-hand slice qualified | [Exact behavior/source/evidence](../EQUIPMENT_FOUNDATION.md): shared durable item revision, conserved transfers, encrypted private/public state, standard Bevy props/widgets and actual two-account native rejoin/restart. 367 workspace tests/18 skips; eight SQL contracts separately pass. Mixed/legacy launchers and combat-widget regression pass. Authored sockets, public sword stance and closed local/remote moving grip also pass [current native checks](../EQUIPPED_PLAYER_STANCE.md). General slots/stats/instances/art/Windows remain open. |
| P3-09 | Loot and rewards | Initial personal claim slice qualified | [Exact source/evidence/limits](../LOOT_REWARDS_FOUNDATION.md): authoritative kill/Scorch credit, durable offer and atomic ordered pickup, real two-account Bevy widget/rejoin/database restart, accepted-work retirement and CLI cleanup pass. 379 workspace tests/21 skips; all eleven SQL contracts separately execute. Full-bag retry, rollback, replay and backend loss pass. General/repeatable/group/corpse loot and instances remain open. |
| P3-10 | Experience and leveling | Initial earned and optional completion XP qualified; broader leveling open | [Completion XP](../QUEST_TURN_IN_XP.md): three normal kills reach 75 XP, atomic once-only turn-in reaches 100 / level 2; retained earned-XP behavior remains. Actual two-account native completion/rejoin/restart passes at 1280×720, 1920×1200 and 3440×1440 fullscreen. Class/stat scaling, groups and production balance remain open. |
| P3-11 | Quest objectives | Initial expanded persistent multi-quest journal qualified; broader quests open | [Current engine and authoring agreement](../QUEST_ENGINE.md): immutable definitions, prerequisites, ordered all/any/at-least stages, kill and semantic NPC objectives, committed private state/actions, aggregate markers and atomic idempotent kill/completion XP.40 owned SQL contracts and normal OIDC/native two-quest completion plus database-restart rejoin pass; repeated preaccept conversations grant no credit. Current fixture reaches 125 XP/head 13. Abandonment, repeatables, item/currency grants, groups, production registry/history scaling and explicit content migrations remain open. |
| P3-12 | NPC dialogue and vendors | Initial live semantic dialogue and fixed barter qualified; broader scope open | [Dialogue](../NPC_DIALOGUE_FOUNDATION.md), [barter](../VENDOR_BARTER_FOUNDATION.md) and [expanded quest integration](../QUEST_ENGINE.md): standard release-click choices, exact NPC/choice authority and committed objective events. Repeated choices before acceptance and later valid progress pass normal native/SQL/restart checks. Prior atomic barter/equipment/owner-retirement proof remains inherited. Production currency/selling/stock/buyback and broader NPC lifecycle remain open. |
| P3-13 | Terrain streaming and initial LOD | Partial | Generated and saved 16 km graybox terrain have bounded streaming, distance LOD, geometric seam validation, independent physics/render origins and actual retirement. Full native saved crossing and two-client impaired-network checks pass. Hierarchical screen-error LOD, long vistas/flying, objects/materials and production budgets remain. [Evidence](../SAVED_TERRAIN.md). |
| P3-14 | Animation, VFX and audio expansion | Partial | Opt-in wolf and authored-socket player death/fresh-life presentation pass actual two-account gameplay and fullscreen launching; public sword stance and closed moving grip also pass; see [latest evidence](../EQUIPPED_PLAYER_STANCE.md). Standard upper-body SwordAttack now passes local/remote standing, walking and running [native/source-curve checks](../PLAYER_MELEE.md). Public NPC timing has an opt-in World/NA01 contract and bounded reliable priority transport: actual encrypted victim/observer transfer at 20/30/60 Hz plus queue/upload refusal checks; 655 network tests pass. [Normal schema-14 CLI/Bevy adoption](../PUBLIC_CREATURE_ATTACKS.md) passes actual two-account native victim/observer views and normal unmodified-wolf quest/restart regression, plus 1,106 workspace tests. Public remote-player death/fresh life now passes [normal schema-15/Bevy native acceptance](../PUBLIC_AVATAR_LIFE.md): actual finished corpse in view, 52 changing Death bones, fresh generation/pose reset, private owner dead-input barrier and healthy observer. Normal quest/database-restart regression plus 1,123 workspace tests pass. Impaired three-rate transport remains separately qualified; immediate same-backend-endpoint reconnect remains unqualified. Full equipped locomotion/blending and broader VFX/audio remain. |
| P3-15 | Content authoring and validation | Initial bounded runtime catalogues and exact-byte launch bundles qualified | [Validator](../GAMEPLAY_CONTENT_VALIDATION.md), [launch adoption](../GAMEPLAY_MANIFEST_LAUNCH.md), [quest limits/authoring agreement](../QUEST_ENGINE.md): schema12 canonical nested quest files; schema13 journal plus explicit threat policy; schema14 derived public NPC timing; schema15 public visual avatar life. 47 validator and 35 actual Rust-to-Python handoff checks pass with exclusive roles, references, budgets, paths and exact hashes; normal native snapshot cleanup passes. Bounded registry, index validator and closed-subset compiler pass 304 core / 45 gameplay / 11 file/compiler checks and full 1,089-test regression. Actual compiled output passes normal manifest admission; see [pack authoring](../QUEST_PACK_AUTHORING.md). Read-only closed-catalogue update review now adds exact hashes, presentation/gameplay classification and version advisories; all 65 authoring-example checks pass. Runtime pack loading/history scaling, saved-version migrations, nav/terrain/assets and general authoring UI remain open. |
| P3-16 | One complete persistent zone and login/logout proof | Initial expanded rewarded training integration qualified; authored zone open | [Expanded quest proof](../QUEST_ENGINE.md): actual OIDC/SQL/native semantic conversations, two quests, three kills, four durable actions and 125 XP; database process restart/rejoin preserves exact head/history with no extra reward. Existing loot/barter/equipment and earlier multi-account proofs remain inherited. Current schema15 regression uses an unmodified wolf, actual normal quest completion/restart and separate two-account public NPC/remote death/fresh-life native views. Two-peer threat transfer has separate impaired encrypted runtime proof. A complete authored production zone remains open. |
| P3-17 | UNSPECIFIED in original transcript: needs reconciliation | Blocked: scope unknown | Obtain missing scope from the owner; do not block unrelated defined tasks. |
| P3-18 | UNSPECIFIED in original transcript: needs reconciliation | Blocked: scope unknown | Obtain missing scope from the owner; do not block unrelated defined tasks. |

## Phase 4

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P4-01 | World region ownership | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-02 | Predictive terrain/object streaming | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-03 | Streamed navigation and spatial queries | Local bake/query/worker and initial pursuit consumer qualified; streaming scope open | [Checked worker contract](../CREATURE_PATH_WORKER.md) and [native pursuit](../CREATURE_PURSUIT_NATIVE.md). Bounded static package, one outstanding job, expiry and full identity/revision checks; searches outside fixed ticks, collision-grounded movement. No all-world streamed navigation, hard query CPU deadline or population-capacity claim. |
| P4-04 | Advanced interest management | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-05 | Adaptive replication | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-06 | Cross-region handoff | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-07 | Population and overload management | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-08 | Advanced combat interrupts/CC/AoE/healing/threat | Initial damage outcomes and damage-only creature threat qualified; broader scope open | [Outcomes](../HIT_OUTCOMES.md), [damage threat](../CREATURE_THREAT.md): authoritative actual-health-loss arithmetic and prepared victim transfer, independent of tag/reward credit. Metric reach and exact lifetimes; impaired encrypted two-peer and normal native regression evidence are separate. General interrupts/CC/AoE/healing, stance modifiers, taunts, social assistance and group threat remain open. |
| P4-09 | Classes and specializations | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-10 | Group encounters | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-11 | Parties and group rules | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-12 | Chat/moderation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-13 | Friends and presence | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-14 | Guild persistence | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-15 | Secure trading | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-16 | Animation LOD and crowd policies | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-17 | VFX density budgets | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-18 | Time/weather integration | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-19 | Multi-zone scale profiling | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P4-20 | World and multiplayer integration gate | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |

## Phase 5

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P5-01 | Rendering benchmarks | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-02 | Lighting and shadows | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-03 | Atmosphere and fog | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-04 | Stylized materials | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-05 | Water | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-06 | Terrain LOD and seams | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-07 | Streaming memory budgets | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-08 | Vegetation instancing and wind | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-09 | Visibility/culling | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-10 | Animation blending and layers | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-11 | Equipment visuals | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-12 | Animation LOD | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-13 | Character readability | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-14 | Particle expansion | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-15 | Projectiles/AoE effects | Initial authoritative travelling magic qualified | [Finite travel, impact-only damage, encrypted/native evidence and limits](../PROJECTILE_TRAVEL.md). Bow/arrow equipment and presentation, ranged NPC casting, general volumes and AoE remain open. |
| P5-16 | VFX priority and density limits | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-17 | World editor | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-18 | Creature/item/ability/quest editors | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-19 | Asset conversion and license validation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-20 | Developer inspection tools | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-21 | Client performance tuning | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-22 | Server profiling | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-23 | Deployment/monitoring/recovery | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P5-24 | Integrated visual and load validation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |

## Phase 6

| Task | Scope | Status | Remaining work / evidence |
|---|---|---|---|
| P6-01 | Release scope and quality gates | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-02 | Minimum playable content | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-03 | Internal playtests/defect tracking | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-04 | Class/combat balance | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-05 | Progression and economy validation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-06 | Quest/encounter chains | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-07 | Onboarding/accessibility | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-08 | Closed alpha accounts/environment | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-09 | Installer/patcher | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-10 | Closed alpha stabilization | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-11 | Concurrent gameplay load test | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-12 | Crowded encounter test | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-13 | Recovery drills | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-14 | Security/exploit testing | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-15 | Closed beta population | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-16 | Long soak testing | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-17 | Production deployment/support | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-18 | Moderation/support workflows | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-19 | Release candidate validation | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-20 | Operational readiness review | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-21 | Go/no-go | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-22 | Final load test | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-23 | Launch | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |
| P6-24 | Launch stabilization | Not started | Implement the named scope after reviewing real prerequisites and defining concrete acceptance checks. |



## Offline actual-world partition continuation — 2026-10-20

P1-10/P2-06/P2-07/P2-08, row 5: actual terrain/collision clipping, whole model
ownership and per-package graph pruning now have independent source comparisons.
The town produces 26 packages; all 2,532 model mesh instances and original
transforms/buffers/materials remain, with terrain UV/height/winding and collision
surface/box checks. Each actual package passes the Rust verifier. Generated
original partition output also passes two actual two-client Bevy/Avian cases,
including signed distant/independent frames and delayed publication.
[Contract and measured budgets](../WORLD_PARTITIONING.md).

The town needs nine providers per cell and ten initially, so only a candidate
index is emitted under the four-package proof budget. This closes the offline
partition slice, not actual-town runtime adoption or a whole task. Next: measure
combined actor/lookahead/recovery/staging demand and resource costs, qualify an
appropriate bounded runtime budget and world-edge policy, remaining handover
lifecycle cases, then real town publication/map regressions before default
adoption. Keep prediction behind the relevant row-5 gate. Current checks and
limitations remain separate in [validation](../VALIDATION.md).


## Verified content-budget analysis — 2026-10-20

P1-03/P1-10/P2-06/P2-07/P2-08: source-verified recovery, actual package/image
costs and conservative actor footprint/lookahead unions now have executable
offline analysis. Town union maxima are 18/22/26 packages for one/two/up-to-eight
actors including recovery. All-indexed content has 4,001 static colliders and
1,514,491,904 estimated base RGBA8 bytes; these are not measured live allocations.
A 32-package hard ceiling / 26-package town budget is proposed with concrete
protocol/admission/world-edge/staging/native acceptance in
[the budget contract](../CHUNK_BUDGETS.md). Runtime remains limited to four;
actual town adoption, full task/phase completion and Windows/MMO capacity remain
open. Continue the runtime gate in execution order, retaining map regressions.


## Qualified bounded runtime capacity — 2026-10-20

P1-03/P1-06/P1-08/P1-10/P2-06/P2-07/P2-08 now qualify a 32-package hard
ceiling, unchanged smaller session budgets, checked control addresses,
budget-bound admission identity v3 and wire/ALPN 11. Actual secure UDP carries
full sets; disk/Avian tests protect the 33rd unloaded cell. The opt-in private
town authority loads ten packages then publishes all 26 against a live actor.
Eight actual two-client native cases qualify 32 initial roots, six-provider
delayed publication, independent negative frames, failed-image retention and
normal map/Quinn regressions. [Contract and evidence](../CHUNK_CAPACITY.md).

Continue row 5 with explicit finite-world bounds and edge/corner qualification,
then actual-town client loading/memory/staging measurements and remaining
reversal/supersession/asymmetric-failure checks before promotion. Preserve the
monolithic default at 30 Hz. Native Windows, full task/phase completion and MMO
capacity remain open. New Unity/Synty asset inspection can proceed independently
without changing the adopted runtime architecture or promoting an unqualified map.


## User-supplied Synty source inspection — 2026-10-20

P1-10/P2-08/P2-09: reviewed Base Locomotion, Elven Realm and Modular Fantasy Hero
archives from the user-specified Downloads folder. The locomotion sample has a
50-bone skinned model and matching selected animation bone names. The Elven demo
contains 15,416 serialized prefab instances; preserving scene structure matters
in addition to converting meshes. Modular source scale/rig differences need
qualification. [Review, evidence and concrete import acceptance](../SYNTY_ASSET_REVIEW.md).
No converted/native Synty result or default replacement is claimed. The core
row-5 finite-world and actual-town client/resource gates remain open.


## Isolated Synty locomotion preview — 2026-10-20

P1-10/P2-08/P2-09: explicit Unity payload extraction, source-pose baking into a
50-joint sample skin, standard GLB export, independent frame comparison and four
native Bevy playback cases now qualify the bounded preview. Full weights are
retained; the four-weight Jump preview exceeds the initial 5 mm gate at 6.974 mm
and is explicitly not production-character adoption. [Contract](../SYNTY_CHARACTER_PREVIEW.md).
Default gameplay/character/map stay unchanged. Elven scene conversion can proceed
as the user's asset branch while core world-edge/resource gates remain open.

## Elven Realm source and representative static preview — 2026-10-20

P1-10/P2-08: hash-bound metadata extraction and bounded recursive Unity dependency
inspection now identify model-prefab ID gaps and unresolved serialized references.
A single pillar's standard GLB/base-colour preview passes independent geometry,
winding, UV and palette comparison, clean Khronos validation and one actual native
Bevy capture. Full scene instantiation, complete materials, collision/navigation
and map adoption remain open. [Acceptance and scope](../SYNTY_ELVEN_IMPORT.md).
This is an isolated asset branch; no task or core foundation gate is complete.

The follow-on model-ID audit matches 1,109 distinct serialized references, with
62 remaining explicit gaps. Source completeness is now a concrete Elven gate:
32 selected payloads are LFS pointers; 26 FBXs and one PNG are reachable from the
demo closure. Missing blob hashes/sizes are recorded privately. This blocks a
complete equivalent scene export, not unrelated core row-5 work. [Details](../SYNTY_ELVEN_IMPORT.md).

## Explicit finite chunk-world bounds — 2026-10-20

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08: schema-2 session rectangles participate
in admission, conservative local swept planes constrain shared authority motion,
lookahead clamps to the rectangle and recovery spawns retain full clearance.
Legacy identities/internal seams remain. Edge/corner, distant-frame, jump/retreat
and unsupported-fall/held-jump recovery checks pass at 20/30/60 Hz. A separate
opt-in town session preserves its source rectangle. [Scope](../WORLD_EDGES.md).
Actual-town client preparation/staging resources and remaining handover lifecycle
checks still precede promotion; no complete task/phase claim follows.

## Actual-town client startup/resource checkpoint — 2026-10-20

P1-10/P1-12/P2-06/P2-07/P2-08: actual ten/all-26-package ordinary two-client
startup now passes with measured Linux process/DRM memory and approximate ready
times. The new index binds transformed mesh-local AABBs after a strict runtime
rejection exposed the tighter vertex-only authoring envelope. Package/image
bytes remain unchanged. [Contract and measurements](../TOWN_CLIENT_RESOURCES.md).
Live client candidate/staging/retirement memory and remaining lifecycle checks
still precede default promotion. No whole task/phase or capacity claim follows.

Actual-town live-publication continuation (row 5, P1-10/P1-12/P2-06/07/08):
shared startup preparation now exposes authority-resolved recovery feet, and exact
round-trip JSON parsing protects admission identity for decimal source heights.
Bounded Bevy lifecycle traces and synchronized two-client captures support source
primitive, retained-root/image and rejected-candidate cleanup checks.
See [the contract and qualification scope](../TOWN_LIVE_PUBLICATION.md).
Broader handover lifecycle checks and default adoption remain open.

Environment-source steering, 2026-10-20: the user selected
[Enchanted Forest](../SYNTY_ENCHANTED_FOREST_IMPORT.md) instead of Elven Realm.
Forest conversion/qualification is the active asset branch; do not wait for Elven
LFS recovery. Preserve town regressions until forest adoption passes.

Forest P1-10/P2-08 continuation: reusable opaque Standard material adaptation
and one actual ruins prefab now pass independent geometry/pixel/material checks
and native Bevy Vulkan inspection. Full scene/foliage/hierarchy/collision gates
remain; see [qualification](../SYNTY_ENCHANTED_FOREST_IMPORT.md).

Forest P1-10/P2-08 source-placement continuation now resolves ordinary prefab
hierarchy/overrides/LOD selection with independently checked emitted source
matrices. Four FBX-backed variants, effects, geometry axes, custom materials and
assembled-scene/collision acceptance remain open.
[Scope and evidence](../UNITY_STATIC_PLACEMENTS.md).

Forest P1-10/P2-08 basic-foliage continuation qualifies real tree/vine cards
through source-derived masked standard PBR, independent pixels/geometry and two
native captures. Explicit preview LOD selection preserves the entire default
highest-detail scene plan. Full vegetation, custom normals, geometry coordinates
and assembled scene/collision adoption remain open. [Scope](../FOREST_FOLIAGE.md).

Forest P1-10/P2-08 model-variant continuation now resolves the four branch
variants through checked inherited aliases and complete authored root TRS. All
179 placements pass independent completeness/binding/matrix checks; old records
are unchanged. Geometry/importer coordinates, full materials and assembled
scene/collision adoption remain open. [Evidence](../UNITY_MODEL_VARIANTS.md).


Forest P1-10/P2-08 geometry continuation: independent mesh-local calibration
matches 52/54 source LOD bounds with two explicit centre mismatches. A shared
standard-glTF assembler preserves shear and mirrored winding; five authored ruins
placements and an asymmetric fixture pass native Bevy/Vulkan checks. Binary
TerrainData, full vegetation, assembled-scene/collision/multiplayer adoption
remain open; preserve town regressions. [Contract](../UNITY_GEOMETRY.md).


Forest P1-10/P2-06/P2-07/P2-08 terrain prerequisite now preserves all 263,169
binary source height samples, five control/layer bindings and 1,365 patch extrema.
Full-grid ground and ground-plus-five-ruins render in native Bevy; the finite
colour bake/geometric normals remain explicit approximations. Partition matching
collision under current budgets, complete full vegetation/source assembly and
qualify multiplayer before adoption. [Contract](../UNITY_TERRAIN.md).


Forest P1-10/P2-06/P2-07/P2-08 ground collision continuation now qualifies four
full-sample packages, independent complete source coverage, real Avian probes/
seam/jump checks and two encrypted native two-client cases. An opt-in launcher
loads the ground; the default town and old map routes remain qualified. Complete
vegetation/materials, authored object collision and representation/budget review
still precede forest adoption. [Contract](../FOREST_TERRAIN_COLLISION.md).


Forest P1-10/P2-08 highest-detail foliage continuation qualifies a static full-
shader PBR profile, actual tree/fern geometry/normal/cutout checks and three native
cases. Three trees and 612 exact-binding ferns now join the visual subset; eight
of twelve used full-shader property profiles are eligible, four remain explicit.
Batch remaining mesh/material overrides and shader families, then complete scene/
collision/resource/multiplayer acceptance before adoption. [Scope](../FULL_FOREST_FOLIAGE.md).


Forest P1-10/P2-08 bulk foliage continuation now qualifies 15,156 source draws in
46 exact mesh/material bindings, actual scene overrides and a shared-image visual
assembly. Independent complete source/category/material/tangent checks and two
native Bevy/Vulkan views pass; 3,503 draws retain explicit unresolved reasons.
Continue remaining Standard/other shader families and mixed slots, then review
object collision and resource budgets before full multiplayer/default adoption.
Town and opt-in forest-ground remain available. [Evidence](../FOREST_FOLIAGE_BATCH.md).


Forest P1-10/P2-08 Standard-material continuation qualifies another 66 bindings /
2,744 source draws, bringing the static subset to 17,900 draws with 31 shared
image variants. Canonical mesh-local export excludes imported FBX node placement;
source normal directions, exact palettes, full scene poses and two native Bevy/
Vulkan views qualify. 759 draws remain explicit. Continue four model variants,
other shader families/mixed slots, then object collision and resource/multiplayer
acceptance before adoption. [Evidence](../FOREST_STATIC_BATCH.md).


Forest P1-10/P2-08 canonical model-variant continuation now qualifies all four
branch models / 179 poses, with inherited IDs, raw nested/outer material bindings,
independent ufbx/source geometry and byte-preserved prior content. The combined
18,079-draw scene passes complete source and native Bevy/Vulkan checks. 580 draws
remain explicit; continue triplanar cliffs/rocks and other shader/mixed-slot work,
then object collision and resource/multiplayer acceptance before adoption.
[Contract](../FOREST_VARIANT_GEOMETRY.md). The user queues the new Elven Realms
folder under Downloads / Synty / Biomes only after forest completion; its contents
have not yet been verified.


Forest P1-10/P2-08/P2-10 triplanar prerequisite now has a source-hash-bound world
projection profile and opt-in Bevy StandardMaterial extension. Actual canonical/
transformed mesh cases pass native lit/diagnostic rendering and independent GPU
pixel checks. The main subset remains 18,079 draws with 580 unresolved. Before
batch adoption, calibrate the exposed FBX mesh/root-pivot offset (including existing
LOD-bound evidence), then integrate multiple profiles, f64 rebase texture phases,
resources/mips and complete scene/object collision acceptance. No automatic mesh
centering or default-map change. [Contract](../FOREST_TRIPLANAR.md). Elven folder
inspection remains queued after forest completion.

Forest P1-10/P2-08 importer checkpoint: actual Linux Unity 6000.6.6f1 compilation/
export and independent recorded-frame point sets pass for all 112 scene FBXs /
214 meshes, with original metadata unchanged. Thirteen existing batch draws need
frame correction. All jobs are terminal; the user requested a safe stop before
converter changes. Resume with source-bound frame adaptation and independent/
native replacement checks, then remaining material/collision/resource gates.
[Findings and acceptance](../FOREST_UNITY_REFERENCE.md).

Resumed 2026-10-21: P1-10/P2-08 now applies recorded Unity mesh frames to ten
bindings / thirteen placements while retaining 106 prototype/placement pairs.
All 116 prototypes and complete rebuilt scene pass source/native checks; town/
farmhouse regressions remain. Corrected moss/all-85 source poses pass as a
standalone single-profile material group, not main-scene adoption. Continue
multi-profile/rebased texture phase/resources/mips, remaining materials, then
object collision/runtime gates. [Evidence](../FOREST_MESH_FRAMES.md).

P1-10/P2-08/P2-10 catalog continuation now qualifies actual mixed Standard and
two-profile rendering, shared typed image handles, complete source composition,
negative mappings and per-profile pixel samples. Legacy and map regressions pass
on the final client. Continue eligible binding batches/full-scene composition,
then rebased texture phase/resources/mips and remaining material/collision/runtime
gates. [Evidence](../FOREST_MATERIAL_CATALOG.md). Main subset counts are unchanged.

## Forest triplanar batch continuation — 2026-10-21

Forest triplanar batch continuation, 2026-10-21: seventeen new bindings / 518
placements now join the corrected visual subset: **18,597 source draws / 62
unresolved**. Fresh independent source/Editor/placement and full composition checks,
four combined native Vulkan views, three dirt-profile modes and 304 pixel checks
pass. Three profiles share eight extension images; 31 Standard image variants
remain distinct. Khronos: 35 files, zero errors, seven runtime-tangent warnings.
Fresh fmt, denied-warning Clippy and 173 Python tests pass. Runtime binaries stay
unchanged; collision/rebase/mips/resource and adoption gates remain. Next: actual
Editor sampler defaults for ten grass draws. All work stays local.
[Evidence and limits](../FOREST_TRIPLANAR_BATCH.md).


## Forest texture-default continuation — 2026-10-21

Forest texture-default continuation, 2026-10-21: actual Linux Unity 6000.6.6f1
resolves four Grass/Mud textures to bilinear/repeat with original PNGs/metadata
unchanged. The ten grass draws now join the qualified subset: **18,607 source
draws / 52 unresolved**. Fresh eighteen-binding source/Editor/placement checks,
four combined native Vulkan views, three grass modes and 274 pixel checks pass.
Fresh fmt, denied-warning Clippy and 175 Python tests pass. Runtime binaries remain
unchanged. Next: independent leaf-colour UV tiling for seventeen giant-tree draws;
remaining materials, rebase/mips/resources, collision and adoption remain open.
All work stays local. [Evidence](../FOREST_TEXTURE_DEFAULTS.md).


## Forest colour-UV continuation — 2026-10-21

Forest colour-UV continuation, 2026-10-21: two exact giant-tree bindings /
seventeen placements now join the qualified visual subset: **18,624 source draws /
35 unresolved**. A second standard glTF channel preserves leaf colour tiling while
normal/AO retain UV0. Source/Editor/exact-placement/full composition checks and six
native Vulkan cases pass; actual loaded materials select the expected channels.
Legacy rendering and encrypted town/farmhouse regressions pass on the new client.
Fresh fmt, denied-warning Clippy, 35 client Rust and 179 Python tests pass. Remaining
materials, rebase/resources/mips/lighting, collision and adoption stay open. All work
is local. [Evidence](../FOREST_COLOUR_UV.md).

## Forest glass continuation — 2026-10-21

Forest glass continuation, 2026-10-21: an explicit untextured Standard-to-glTF
BLEND adaptation adds the lantern glass: **18,625 source draws / 34 unresolved**.
Source geometry/material, actual Editor frame, exact placement/full composition
and four native Vulkan cases pass. The adaptation records Unity premultiplied
lighting/colour-space differences; no runtime shader/binary change is introduced.
Fresh fmt, denied-warning Clippy and 181 Python tests pass. Next: actual mixed
submesh-slot coverage for 25 crystal/geode draws. Remaining materials, lighting,
rebase/resources/mips, collision and adoption remain open. All work stays local.
[Evidence](../FOREST_GLASS.md).

## Forest mixed-material continuation — 2026-10-21

Forest mixed-material continuation, 2026-10-21: nine crystal/geode bindings /
25 draws join the qualified subset: **18,650 source draws / nine unresolved**.
Actual Editor submesh triangles prove source-slot fidelity; four differing quad
diagonals use Editor topology with original corner normals/UVs. Crystal appearance
is an explicit static standard-PBR preview, with refraction/view effects omitted.
Full composition, nine native Vulkan cases, five rebound faults and 19 Khronos
files pass (zero errors/seven retained warnings). Fresh fmt, denied-warning Clippy
and 189 Python tests pass. Water/sky/emissive willow, lighting, rebase/resources/mips,
collision and adoption remain open. All work stays local.
[Evidence](../FOREST_MIXED_MATERIALS.md).

## Forest source sky — 2026-10-21

Forest sky continuation, 2026-10-21: the source skydome now joins the qualified
subset: **18,651 source draws / eight unresolved**. Actual Editor geometry, source
world-height gradient and placement pass independent checks. Standard unlit and
explicit Bevy shadow markers pass four native Vulkan cases; three legacy modes,
two native failure paths and encrypted town/farmhouse regressions pass. Fresh fmt,
denied-warning Clippy, 38 client Rust and 193 Python tests pass. Three Khronos
files have zero errors/seven retained warnings. Seven water draws and one emissive
willow remain, followed by lighting, rebase/resources/mips, collision and adoption.
All work stays local. [Evidence](../FOREST_SKY.md).

## Forest static water — 2026-10-21

Forest water continuation, 2026-10-21: four bindings / seven source water draws
join the qualified visual subset: **18,658 source draws / one unresolved**. Actual
Linux Unity proves built-in Plane identity/geometry; exact Editor FBX triangles,
raw source poses and explicit static PBR factors pass independent checks. Full
composition, eleven native Vulkan cases and four rehashed faults pass. Ten Khronos
files have zero errors/seven retained warnings. Fresh fmt, denied-warning Clippy
and 197 Python tests pass; runtime binaries remain unchanged. Emissive willow,
source water effects, lighting, rebase/resources/mips, collision and adoption remain
open. All work stays local. [Evidence](../FOREST_WATER.md).

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
stays local. [Evidence](../FOREST_EMISSIVE.md).

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
[Evidence](../FOREST_RESOURCES.md).

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
All work stays local. [Evidence](../FOREST_LIGHTING.md).

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
remain open. All work stays local. [Evidence](../FOREST_COLLIDER_REFERENCE.md).

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
work stays local. [Evidence](../FOREST_COLLISION_GEOMETRY.md).

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
defaults remain preserved. All work stays local. [Evidence](../FOREST_HEIGHTFIELDS.md).

## Bounded static-shape admission — 2026-10-21

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
All work stays local. [Contract and evidence](../STATIC_SHAPES.md).

## Forest complete reference packaging — 2026-10-21

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
[Packages](../FOREST_SHAPE_PACKAGES.md), [standard materials](../STATIC_MATERIALS.md).

## Forest source texture phase — 2026-10-21

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
[Evidence and scope](../TRIPLANAR_PHASE.md).

## Bound runtime projection materials — 2026-10-21

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
open. All work stays local. [Contract and evidence](../RUNTIME_MATERIALS.md).

## Complete forest runtime candidate — 2026-10-21

Forest normal-runtime continuation, 2026-10-21: ten bound material packages
preserve complete source geometry/collision; independent comparisons, four
rehashed faults, all 804,029 source rays and glTF validation pass. A separate
candidate excludes exactly one invisible water collider. Two real encrypted
Linux/Vulkan clients load the full scene, share four projection materials and
47 actual images, and pass visibility/readiness/rebase checks. Per-client base
RGBA CPU image data is 1,589,649,408 bytes (not total memory). The actual-map
survey records 27 clear starts and 41 rejected routes; movement, lighting,
mips/resources, water and broader camera/adoption remain open. Town stays the
default. All work stays local. [Evidence and limits](../FOREST_RUNTIME.md).

## Interactive performance correction — 2026-10-22

P1-12 / P2-05 / P2-08 / P2-10: release launch defaults and authored collision-based
chunk camera clearance address the reported slow debug walkthrough. The full
forest measures 99.65 FPS uncapped at 1280×720; a normal desktop relaunch averages
57.89 FPS with VSync. Frame-time spikes and measurement limits are retained.
All 45 client tests, denied-warning Clippy and 221 asset Python tests pass.
This does not complete the task rows or establish MMO capacity. See the
[performance record](../FOREST_PERFORMANCE.md).

### P2-04 prerequisite continuation — 2026-10-22

Gameplay-only kinematic state capture/restore and exact same-frame Avian replay
now pass at 20/30/60 Hz, with invalid-state/diagnostic injection rejection. The
new procedural two-peer impairment check passes readiness, seam crossing,
withdrawal/regrant and stopped convergence under delay/jitter/10% loss/duplication/
reorder. An airborne withdrawal keeps gravity and landing active while holding XZ.
These are prerequisites, not prediction adoption. Wire v11 still carries held input
and position snapshots; completed-tick/command and authoritative motion-state
correction messages remain next. [Contract](../PREDICTION_CONTRACT.md).

### P1-06/08, P2-04 bounded replay continuation — 2026-10-22

Completed-step input timing, a separate movement-stream epoch and 64-step
reconciliation history now pass headless shared-controller checks at 20/30/60 Hz.
Seven replay/reset checks and six timing checks pass, including real Avian terrain
seam/jump queries under scheduled packet faults. Full workspace: 214 tests pass;
denied-warning Clippy and fmt/diff pass. These simulated delivery schedules are not
new real-UDP prediction evidence. Actual authority/Renet/client integration and
native presentation remain next; P2-04 stays precursor-only. Existing wire v11 and
qualified default release behavior are preserved. [Evidence](../PREDICTION_CONTRACT.md).
