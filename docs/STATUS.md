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

Resumed at user request, 4 November 2026. Continue inventory-aware quest owner
integration from the qualified 3 November storage/destruction checkpoint.
Reading/viewing quest notes is added as a separate planned historical objective;
current possession and reading are distinct. See [item quest scope](QUEST_ITEM_OBJECTIVES.md).
No new live or native item/document behavior is claimed by this resume record.

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

Local minimap implementation in progress, 3 November 2026:
P2-14 / P3-13: standard Bevy UI reads resident mesh grids into a bounded local
map, uses checked global projection/manual heading and reuses committed guide
filters. No terrain requests, extra camera or gameplay entitlement. Backpack
layout preserves right-side space. All 135 client tests, formatting and strict
all-target Clippy pass; optimized native layout/input/performance checks at the
three supported resolutions are next. [Scope](MINIMAP.md).
The qualified connection-confirmation playable stays preserved. Local only.

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

Normal public avatar-life adoption in progress, 3 November 2026:
P3-04 / P3-14 / P3-16: explicit schema-15 normal CLI/bundle, standard Bevy remote
Death/fresh-life reset and life-aware pose discontinuity implemented. Private
owner state/controls stay authoritative. All 131 client / 47 validator / 35
Python checks, formatting and denied-warning Clippy pass. Optimized build and
owned native death/respawn checks are next; no new normal playable claim yet.
Preserve schema-14 and qualified AV01 transport. [Scope](PUBLIC_AVATAR_LIFE.md).
All changes stay local.

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

Public avatar visual-life prerequisite in progress, 3 November 2026:
P3-04 / P3-14 / P3-16: optional pre-admission World/profile and AV01 roster codec
project actor/life identity plus alive/dead only. Five wire and four actual
World cases pass, including death/respawn at 20/30/60 Hz, bounded connected roster
and authority refusal. No public transport, normal version or remote animation
adoption yet. Qualified normal public-NPC playable stays preserved. Full network
regression and Clippy are next. [Scope](PUBLIC_AVATAR_LIFE.md). Local only.

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

Normal public NPC presentation adoption in progress, 3 November 2026:
P3-07 / P3-14 / P4-08: explicit schema-14 bundle/CLI identity and standard Bevy
wolf public timing/facing implemented. Existing bundles remain unchanged.
129 client, 46 validator and 34 Python checks, formatting and all-target Clippy
pass. Optimized build and actual native victim/observer qualification are next.
Preserve the qualified public transport and threat-v2 playable; no new native
adoption claim. [Scope](PUBLIC_CREATURE_ATTACKS.md). Changes remain local.

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

Stopped safely at the user's request, 2 November 2026:
The schema-11 reset playable and inactive threat/transfer core prerequisites
below are qualified. The unfinished nine-file World threat adapter is archived
in `build/creature-threat-world-draft-v1-20261102` with source hashes, a reviewable
patch and resume notes. It is absent from the active source tree and is not
qualified. Its full network checks were interrupted; focused World acceptance
is still required. All 748 source hashes matched the qualified core checkpoint
before this status note, and formatting passed. No commit or push was made.

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

# Historical candidate — 2026-11-01: normal-client dodge adoption

P2-04 / P2-05 / P2-07 / P3-06: normal client/server profile selection, V input,
HUD cooldown hints and a sealed schema-4 gameplay bundle are implemented in the
local candidate. The final application build passes **100 client tests**;
the validator passes **18 checks**. Full regressions and native qualification
are in progress. The playable checkpoint remains
`build/camera-panels-qualified-v1-20261101`; dodge has no native acceptance yet.
[Current behavior and remaining checks](DODGE_MOVEMENT.md).
Changes remain local and unpushed.

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

# Headless prerequisite qualified — 2026-11-01: dodge replay

P2-04 / P2-07 / P3-06: immutable profile-bound prepared-terrain dodge prediction
and tick-aware replay pass nine new checks, with **416 network passes / 10 explicit
skips / 64 groups**, formatting and workspace/all-target denied-warning Clippy.
[Exact behavior, proof and remaining authority/transport gates](DODGE_MOVEMENT.md).
This does not enable live dodge. The panel/HUD candidate has 99 passing client
tests and built binaries at `build/camera-panels-candidate-v1-20261101`; native
checks await an agreed desktop window. The last qualified playable build remains
`build/camera-mode-qualified-v1-20261101`. Changes stay local.

# Active refinement — 2026-11-01: panel priority and control hints

P2-05 / P2-14: the source now gives dialogue/backpack priority over quitting,
allows classic world dragging around open panels, routes wheel ownership by
actual UI hover, and displays mode-specific HUD controls. **99 client tests**,
formatting and denied-warning workspace/all-target Clippy pass. Native rendering/
OS checks for this refinement remain pending; the qualified playable build is
still `build/camera-mode-qualified-v1-20261101` (97 client tests / seven native
starts). Desktop input has been released to the user; no new synthetic desktop
input will run until coordinated. Next independent work is shared dodge replay.
Changes stay local and unpushed.

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
[Behavior, exact evidence and limits](PLAYER_CONTROLS.md).
Matched source/binaries/art/proof: `build/camera-mode-qualified-v1-20261101`.
Earlier checkpoints remain. No new full-workspace, FPS/capacity, Windows or
physical-hardware claim. Live dodge remains uninstalled. Next: panel/input
refinements and shared dodge prediction. Changes are local and unpushed.

# Active direction — 2026-11-01: WoW/GW2 movement and camera

The user adopted familiar WoW/GW2 play with GW2-oriented movement/camera and an
optional action camera. [Behavior and acceptance](MOVEMENT_CAMERA_TARGET.md).
Dodge edge/codec prerequisites pass 407 network tests (10 explicit skips).
The camera-mode source candidate passes 97 client tests and workspace denied-
warning Clippy; native launcher/input validation is in progress.
The qualified playable build remains `build/world-input-qualified-v1-20261101`;
new camera modes are a source candidate and live dodge remains uninstalled.
Changes stay local.

# Active slice — 2026-11-01: restorable dodge state qualified

P2-04 / P2-05 / P2-07 / P3-06 / P3-14: checked cooldown/active-motion snapshots
resume with identical remaining distance at 20/30/60 Hz. **138 core tests / 21 groups**,
nine filtered kinematic checks, formatting and denied-warning workspace/all-target
Clippy pass. [Exact scope, proof and upcoming wire/authority gates](DODGE_MOVEMENT.md).
No live runtime currently selects dodge. The runnable mouse-control checkpoint
remains `build/world-input-qualified-v1-20261101`. Changes remain local.

# Active slice — 2026-11-01: shared dodge capsule prerequisite qualified

P2-04 / P2-05 / P2-07 / P3-06 / P3-14: eight new actual-Avian dodge-controller
checks pass, with **394 network all-target passes / 10 explicit skips in 61 groups**,
formatting and denied-warning workspace/all-target Clippy. [Behavior and remaining
adoption gates](DODGE_MOVEMENT.md). Pure policy remains 130 core passes. No server, wire,
prediction or client currently selects dodge. Restorable state and protocol are
next. Playable controls remain `build/world-input-qualified-v1-20261101`.

# Active slice — 2026-11-01: bounded dodge policy qualified

P2-04 / P2-05 / P2-07 / P3-06 / P3-14: eight new pure fixed-tick movement-policy
checks pass within **130 core tests / 20 groups**, with formatting and denied-
warning workspace/all-target Clippy. [Policy, exact proof and next gates](DODGE_MOVEMENT.md).
The profile is not selected by any runtime bundle; server/controller/protocol/
client adoption and native dodge checks remain. Playable mouse controls and quest
markers stay preserved in `build/world-input-qualified-v1-20261101`.

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

# Active slice — 2026-11-01: floating quest markers

P3-11 / P3-12 / initial P3-14: standard Bevy guide markers now derive from
committed private quest state, with bounded distance, residency and static
occlusion checks. Four new checks / 86 client tests, formatting and denied-warning
workspace/all-target Clippy pass. Native validation is in progress.
[Behavior, acceptance and current evidence](QUEST_MARKERS.md). The preserved playable
checkpoint remains `build/projectile-qualified-v1-20261031`. Development resumed
at the user’s request; changes remain local and unpushed.

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

# Active slice — 2026-10-31: pursuit policy qualified, server integration next

P3-07 / P4-03: pure perception/chase/attack/leash/return policy passes 108 core
all-target tests, denied-warning core Clippy and formatting. No gameplay profile
or server creature consumes it yet. [Contract, exact evidence and next gates](CREATURE_PURSUIT.md).
Matched source/proof: `build/pursuit-policy-qualified-v1-20261031`.
The checked navigation prerequisite and runnable grip checkpoint remain preserved.

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

# Current status — 2026-10-23

Saved continental terrain is the verified development default. The existing
16 km × 16 km world is baked into 15,625 versioned, hashed 128 m height files.
Visuals and collision load those samples on bounded workers; fixed simulation
performs no file I/O. Missing/corrupt data fails explicitly, with no recipe
fallback. `python3 tools/play.py` uses `build/graybox-terrain-v1` after its offline
bake; `--graybox-world` retains generated terrain. Prediction remains enabled.

P1-10/P2-06/07 and the initial P3-13 slice pass: full saved native crossing,
125 visited columns, maximum 81 terrain roots, 2,345 loads / 2,300 retirements,
zero movement recoveries or missed server deadlines. One-client Linux RX 7900
XTX / Ryzen 7800X3D, 1280×720, validation off: **818.59 FPS**, mean **1.222 ms**.
This is warm-cache empty-world evidence. Two clients under impaired UDP, held-jump
teleports, explicit corrupt-file exit, default launcher cleanup, retained town and
generated-source native regressions all pass. Wire/Quinn ALPN is **12**, with a
77-byte welcome carrying explicit terrain frame policy.

The initial saved-source checkpoint passed **228 Rust tests**, with ten private-fixture
tests explicitly ignored; the strengthened
saved non-spawn revisit tests also pass. Denied-warning Clippy, formatting,
221 asset-tool and four GLB alignment tests pass. [Saved terrain](SAVED_TERRAIN.md)
records exact binaries/content identities, commands, reports, failed attempts
and remaining gates. New work remains local.
P2-04 diagnostics now distinguish matched simulation ticks, pending readiness,
input/loading transitions and steady rendered movement. The qualified client
clock uses a continuous bounded rate and acquires its initial buffer after real
RTT/terrain readiness. Steady held frames on the impaired native route fall from
about 9% to **0% / 0.152%**; ordinary VSync records **0.463%** at 60 FPS. A scoped
1% gate rejects the measured old baseline and accepts these final runs. Rare
corrections under loss remain; broader latency/Windows/load limits are explicit.
[Measurement/clock evidence](PREDICTION_CONTRACT.md). That checkpoint passes
**234 Rust tests**, ten private-fixture skips and Clippy/fmt.

The subsequent actual eight-peer, 180-second saved-terrain headless soak passes:
zero server deadline misses/recoveries, maximum nine authority collision patches
and two per client query runtime, with actual retirement and relevance pruning.
Sampled combined RSS peaks at 166.33 MiB; long-term memory plateau remains open.
[Soak evidence](SAVED_TERRAIN_SOAK.md) separates the completed v2 run from the
latest example-only return-identity checks, which compile/Clippy but remain
runtime unqualified. The full 234-test checkpoint predates the new soak/telemetry.

**Stopped safely at the user’s request.** The build completed; no longer soak was
started. Changes remain local and uncommitted. Larger roadmap tasks remain partial.

## Earlier checkpoints

# Current status — 2026-10-22

The completed Linux development slice is the 16 km × 16 km graybox world with
POLYGON Base Locomotion and actual Renet client prediction/reconciliation enabled
by default. Run `python3 tools/play.py`; `--no-prediction` retains the qualified
server-pose interpolation baseline. Forest reproduction remains frozen.

P1-06/08 and bounded P2-04 acceptance now includes completed-step authority,
64-step replay, explicit movement streams, collision/readiness gating, real
impaired UDP at 20/30/60 Hz, native two-client animation/camera/teleport checks
and held-jump resets. The predicted full-continent crossing visits all 125 chunk
columns with at most 81 terrain roots, actual mesh retirement and zero missed
server deadlines or movement recoveries. Linux RX 7900 XTX / Ryzen 7800X3D,
1280×720, one client, validation off: **821.70 FPS**, mean **1.217 ms**;
this empty-world measurement does not qualify MMO population capacity.

The final client additionally bounds interpolation when a distant teleport's
snapshot precedes its correction. Its fresh native check passes. The default
launcher's two-client startup, real command traffic and owned process/ticket
cleanup pass after both interruption and injected client failure. POSIX shutdown
now allows the server to emit final transport statistics; Windows retains its
existing termination path and remains unqualified natively.

See [the prediction contract](PREDICTION_CONTRACT.md) for exact executable hashes,
checks and limitations, [the terrain baseline](GRAYBOX_WORLD.md), and the
[live task status](engineering/TASK_STATUS.md). Broader P2-04 latency/load/Windows,
authored saved-world streaming, navigation, combat and persistence remain open.
Raw correction diagnostics include readiness/clock catch-up and do not establish
a universal visual smoothness budget. The current slice is complete; further
feature work is stopped at the user's request. Commit/push is explicitly authorized.

Final repository verification, 2026-10-22: **224 Rust tests pass**, zero failures;
10 explicit private-fixture tests remain ignored. All **221 asset-tool Python tests**
and **4 GLB alignment tests** pass. Workspace/all-target denied-warning Clippy,
`cargo fmt --check` and diff whitespace checks pass. Logs:
`build/prediction-commit-workspace-tests.log`, `build/prediction-commit-clippy.log`,
`build/prediction-commit-fmt.log`, `build/prediction-commit-python-tests.log`,
`build/prediction-commit-align-tests.log`. Actual encrypted UDP reports are in
`build/prediction-commit-network`; these remain ignored local evidence, along with
private models, Unity exports, tickets and binaries. Linux execution is qualified;
no native Windows execution is claimed. No engine/plugin pins changed in this
prediction continuation.

## Earlier checkpoints (historical, superseded where noted)

Prediction foundation continuation, 2026-10-22 — P1-06/08, P2-04: completed-step
command timing and a bounded 64-step reconciliation core now pass headless Avian
tests at 20/30/60 Hz. Scheduled delivery faults exercise immediate local response,
terrain seam/jump correction and stopped convergence; teleport, readiness,
collision failure, overflow and stale/foreign-state guards pass. Movement streams
are separate from world incarnation. Full workspace **214 tests pass**, including
the retained real encrypted UDP regressions; denied-warning workspace/all-target
Clippy and fmt/diff pass. The scheduled prediction tests are not real-UDP or native
rendering evidence. Wire v11 and the live client remain on held input and smoothed
authority poses; actual command/correction integration is next. No renderer or
default walkthrough behavior changed. [Scope and evidence](PREDICTION_CONTRACT.md).
All changes remain local.

Continental graybox adoption, 2026-10-22: `python3 tools/play.py` now selects
16 km × 16 km procedural terrain and the POLYGON Base Locomotion development
player. Forest fidelity work is frozen. Bounded workers prepare terrain/Avian
patches; full-detail GPU-ready terrain gates authoritative travel. Standard Bevy
culling, distance LOD, actual mesh retirement, recipient-local snapshots and moving
render/query frames pass native tests. A real release client crosses all 125
horizontal chunk columns: maximum 81 terrain roots, 2,345 mesh publications /
2,300 retirements, zero movement recoveries or missed server deadlines.
At 1280×720 on RX 7900 XTX, one-client uncapped crossing averages 824.14 FPS
(1.213 ms; worst one-second p95 1.376 ms), without validation. Two-client validated
teleport/return/animation and retained town cases pass. Local/remote presentation
smoothing fixes 30 Hz stepping; readiness withdrawal now preserves airborne
gravity/landing. Delayed/lost/reordered two-peer terrain readiness and the actual
default launcher's startup/cleanup pass. Kinematic state replay is qualified as a
P2-04 prerequisite; client prediction remains unimplemented. Full workspace
201 tests (including 49 client), 221 Python tests,
denied-warning Clippy and fmt/diff pass. This is graybox terrain traversal, not
saved authored-map streaming, all-route/Windows/MMO capacity qualification.
[Behavior, evidence and limits](GRAYBOX_WORLD.md). All changes remain local.

# Earlier development record

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

Forest water continuation, 2026-10-21: four bindings / seven source water draws
join the qualified visual subset: **18,658 source draws / one unresolved**. Actual
Linux Unity proves built-in Plane identity/geometry; exact Editor FBX triangles,
raw source poses and explicit static PBR factors pass independent checks. Full
composition, eleven native Vulkan cases and four rehashed faults pass. Ten Khronos
files have zero errors/seven retained warnings. Fresh fmt, denied-warning Clippy
and 197 Python tests pass; runtime binaries remain unchanged. Emissive willow,
source water effects, lighting, rebase/resources/mips, collision and adoption remain
open. All work stays local. [Evidence](FOREST_WATER.md).

Forest sky continuation, 2026-10-21: the source skydome now joins the qualified
subset: **18,651 source draws / eight unresolved**. Actual Editor geometry, source
world-height gradient and placement pass independent checks. Standard unlit and
explicit Bevy shadow markers pass four native Vulkan cases; three legacy modes,
two native failure paths and encrypted town/farmhouse regressions pass. Fresh fmt,
denied-warning Clippy, 38 client Rust and 193 Python tests pass. Three Khronos
files have zero errors/seven retained warnings. Seven water draws and one emissive
willow remain, followed by lighting, rebase/resources/mips, collision and adoption.
All work stays local. [Evidence](FOREST_SKY.md).

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

Forest material-catalog continuation, 2026-10-21: the inspector supports bounded
schema-2 profiles alongside Standard materials while retaining schema 1. An actual
85+42-placement fixture applies two profiles to 127 primitives, keeps the building
Standard, and shares six images across eight slots. Native mapping/failure paths,
1,906 independent pixel samples, legacy/full-forest and encrypted town/farmhouse
regressions pass. Fresh fmt, denied-warning Clippy, 35 client Rust and 171 Python
tests pass. The main subset remains 18,079 draws / 580 unresolved. Next: batch
eligible triplanar bindings into the full scene, then rebase/resource/mip and
remaining material/collision/runtime gates. All work stays local.
[Contract, evidence and limits](FOREST_MATERIAL_CATALOG.md).

Forest mesh-frame continuation, 2026-10-21: the user resumed work. Ten bindings /
thirteen placements now use actual Unity mesh frames; the other 106 prototype/
placement pairs stay byte-identical. All 116 prototypes match Editor vertices
within 7.69 micrometres. Full scene coverage, three native Bevy/Vulkan views,
Khronos and retained encrypted town/farmhouse regressions pass. A corrected
85-placement moss group also passes source/world, six final native and 512
independent pixel checks, maximum one sRGB8 level. Fresh fmt, denied-warning
Clippy, 169 Python tests and three private faults pass. The corrected main subset
remains 18,079 draws / 580 unresolved; moss is still an opt-in single-profile
group. Multi-profile/rebase/resource integration, remaining materials, object
collision and adoption remain. All work stays local.
[Contract, evidence and next gates](FOREST_MESH_FRAMES.md).

Forest Unity-reference checkpoint, 2026-10-20: native Linux Unity 6000.6.6f1
compiled/exported 112 actual FBXs / 214 meshes, covering all 118 FBX-backed scene
bindings. Independent actual-frame point sets all match within 11.02 micrometres;
all metadata remains byte-identical. The old unadjusted mesh-local policy matches
195/214 meshes; ten current batch bindings / thirteen draws need frame correction.
Fresh fmt, denied-warning Clippy and 162 Python tests pass. All jobs completed.
The user requested a safe stop; no converter correction, runtime change or forest
promotion began. The 18,079-draw subset / 580 unresolved draws remain preserved,
with importer-frame corrections now explicit. All work stays local.
[Findings and resumption acceptance](FOREST_UNITY_REFERENCE.md).

Forest triplanar material prerequisite, 2026-10-20: a pinned world-projection
profile now renders one actual moss mesh and a transformed instance through
Bevy's standard PBR extension. Six native Vulkan cases and 588 independent
colour/decoded-normal pixel checks pass (maximum one sRGB8 level); missing options/
marks reject. Fresh fmt, denied-warning Clippy, 157 Python and 31 client Rust tests
pass. The 18,079-draw forest subset still has 580 unresolved draws. A newly exposed
FBX root/pivot-offset calibration gap, multiple-profile/streaming/rebase integration,
mips and object collision remain before promotion. No guessed centering is applied.
All work stays local. [Contract and evidence](FOREST_TRIPLANAR.md).

Forest branch-variant geometry continuation, 2026-10-20: all four inherited-ID
branch models and their **179 actual source placements** now join the qualified
static scene (18,079 source draws). Independent raw bindings, complete category/
scene coverage, ufbx and byte-preserved reuse checks pass. Two native Bevy/RADV
Vulkan views pass active validation; 117 glTF files have zero errors and the 14
existing tangent warnings. Fresh fmt, denied-warning Clippy and 152 Python tests
pass. 580 draws, authored object collision, resource review and adoption remain
open. Elven source-folder inspection is queued after forest completion, per user
instruction. All work stays local. [Evidence](FOREST_VARIANT_GEOMETRY.md).

Forest Standard-material continuation, 2026-10-20: **17,900 source draws** now
render in the combined static subset, adding 66 Standard bindings / 2,744 draws.
Canonical export avoids double-applying FBX node scale; independent source normal
direction checks qualify two nonunit-normal meshes without changing geometry.
All 66 Standard references and complete final-source coverage pass. Two native
Bevy/RADV Vulkan views pass active validation; 113 glTF files have zero errors
and 14 documented tangent warnings. Fresh fmt, denied-warning Clippy and 150
Python tests pass; legacy ruins bytes are unchanged. 759 draws, authored object
collision, resource review and full map adoption remain open. All work is local.
[Contract and evidence](FOREST_STATIC_BATCH.md).

Forest foliage batch continuation, 2026-10-20: **15,156 source draws across 46
mesh/material bindings** now join ground/ruins with 17 shared image variants.
Independent full-scene source/pose/material/tangent coverage passes; 3,503 draws
remain explicitly unresolved. Two final native Bevy/RADV Vulkan views pass active
validation; Khronos reports zero errors and 14 documented runtime-tangent warnings.
Fresh fmt, denied-warning Clippy and 149 Python tests pass; basic-card bytes remain
unchanged. Full object collision, remaining shader families/resources and default
map adoption remain open. All work is local. [Evidence](FOREST_FOLIAGE_BATCH.md).

Highest-detail forest foliage continuation, 2026-10-20: a pinned full-shader
static PBR profile now converts the actual LOD0 small tree and fern, preserving
source cutouts and applying explicit normal-map reconstruction. Independent
geometry/pixel/normal/FBX checks pass. Three exact-binding tree and 612 fern
placements join ground/ruins in a source-preserving visual subset. Three native
Bevy/RADV Vulkan cases pass with active validation. The legacy basic card remains
byte-identical. Fresh fmt, denied-warning Clippy and 137 Python tests pass.
Eight of twelve used full-shader property profiles are eligible; four and other
shader families remain unresolved. Wind/highlights, complete scene/object collision
and adoption remain open. All work is local. [Contract](FULL_FOREST_FOLIAGE.md).

Forest ground collision continuation, 2026-10-20: four bounded packages retain all
263,169 distinct source vertices and 524,288 ground triangles with one shared PNG.
Independent visual/collision/UV/normal coverage passes. Actual Avian source probes
and 18 seam/jump cases pass at 20/30/60 Hz. Two encrypted two-client native cases
cross X/Z seams with matching live collision and shared handles; opt-in launcher
and unchanged default-town admission/cleanup pass. Fresh fmt, denied-warning
Clippy, 160 ordinary plus two private Rust tests and 131 Python tests pass;
retained town/farmhouse routes pass all three tick rates. Full forest vegetation,
materials, object collision and adoption remain open. All work is local.
[Contract, commands and evidence](FOREST_TERRAIN_COLLISION.md).

Forest terrain continuation, 2026-10-20: binary TerrainData now decodes with all
263,169 source heights, five layer/control bindings and 1,365 independently checked
patch bounds. A full-sample 150 × 150 metre ground GLB and explicit baked-colour
preview pass independent geometry/UV/pixel checks and Khronos validation. Ground
and ground-plus-five-ruins pass native Bevy/RADV Vulkan checks with active
validation. Fresh fmt, denied-warning Clippy and 127 Python tests pass. Authored
terrain normal maps, full vegetation/scene, terrain collision and multiplayer
adoption remain open; the current town remains the runtime map. All work is local.
[Contract, limitations and evidence](UNITY_TERRAIN.md).

Forest geometry continuation, 2026-10-20: source-specific X-reflection calibration
matches 52 of 54 serialized LOD bounds; two centre mismatches remain explicit.
Independent ufbx positions and an affine-safe shared-mesh assembler now qualify
five actual ruins placements and an asymmetric shear/mirror fixture. Both pass
native Bevy/RADV Vulkan inspection with active validation and vertex bounds within
0.003 mm of independent source calculations. Fresh fmt, denied-warning Clippy
and 121 Python tests pass. TerrainData decoding, full vegetation, full scene and
collision remain open; the current town stays the runtime map. All work is local.
[Contract and evidence](UNITY_GEOMETRY.md).

Qualified model-variant placement continuation, 2026-10-20: inherited aliases now
bind all four branch variants and all 2,864 scene modifications. Requiring complete
authored root TRS resolves their 179 placements without assuming imported root
poses. All 179 pass independent category-completeness/binding/matrix checks.
The source plan now resolves 18,725 prefabs and emits 18,659 static records; the
old 18,480 records and 423 unused overrides are exactly unchanged. Thirty particle
override errors remain, alongside separately recorded unimplemented components.
Fresh fmt, denied-warning Clippy and 115 distinct Python tests pass. Geometry
axes, full materials, assembled scene and collision remain open; no Rust/binary/
renderer/default-map change or new GPU claim.
[Contract and evidence](UNITY_MODEL_VARIANTS.md).

Qualified static foliage continuation, 2026-10-20: actual background-tree cards
and a vine LOD1 card now convert to standard Bevy masked PBR with source-derived
colours, exact alpha, checked branch geometry/UVs and measured mesh-normal export
rounding. Two native inspector cases and Khronos validation pass on RX 7900 XTX
with active Vulkan validation. Custom normal-map output remains an explicit
approximation. Highest-detail vegetation/full scene adoption remain open.
Fresh fmt, denied-warning Clippy and 109 Python tests pass; the original default
scene plan is exactly unchanged. No Rust/binary/default-map change.
[Contract, evidence and limits](FOREST_FOLIAGE.md).

Qualified forest static-placement continuation, 2026-10-20: source hierarchy,
instance transforms/active/material overrides and highest-detail LOD planning now
resolve 18,546 of 18,755 prefab placements. All 18,480 emitted static records pass
independent mathutils/source binding checks. The report retains 423 documented
unused flat-target overrides and 209 unresolved model-variant/particle placements.
Fresh fmt, denied-warning Clippy and 103 distinct Python tests pass. This is an
offline source-coordinate plan; geometry axes, custom materials, assembled scene
and collision remain open. No Rust/runtime/default-map change or new GPU claim.
[Contract and evidence](UNITY_STATIC_PLACEMENTS.md).

Qualified Enchanted Forest material continuation, 2026-10-20: the first actual
ruins prefab converts through a reusable opaque Standard-to-glTF PBR adaptation,
preserving source albedo/emission pixels, surface scalars, culling and sampling.
Independent geometry/UV/material checks and Khronos validation pass; one native
Bevy capture passes on RX 7900 XTX/RADV Vulkan with active validation.
Fresh fmt, denied-warning Clippy and 95 Python tests pass; legacy pillar source
conversion still passes. No Rust runtime/binary/default-map change.
Next: custom forest materials and source hierarchy/LOD/override instantiation
before assembled scene and collision qualification.
[Contract, evidence and scope](SYNTY_ENCHANTED_FOREST_IMPORT.md).

Enchanted Forest replaces Elven Realm as the user-selected environment import
target, 2026-10-20. Its 225 FBXs and selected images contain actual data; no LFS
pointers occur in the 647 staged data payloads. Source/demo audit is complete;
prefab/material conversion and native scene qualification precede map adoption.
The current town remains available during replacement. Elven missing-source
recovery is no longer in the active queue.
[Source evidence and next acceptance](SYNTY_ENCHANTED_FOREST_IMPORT.md).

Qualified actual-town live-publication continuation, 2026-10-20: ordinary encrypted
two-client sessions now publish ten-to-eleven town packages, retaining existing
roots/image handles and matching live Avian. Normal loading and a five-second
asymmetric delay with independent client query frame pass. Shared startup exposes
resolved recovery feet; exact round-trip decimal JSON parsing fixes a reproduced
one-bit admission-identity mismatch without relaxing the independent oracle.

Fresh fmt, denied-warning Clippy/build, **160 ordinary Rust tests**, **three
private-source tests** and **91 Python tests** pass. **Eleven actual native
cases, 22 reports**, pass on RX 7900 XTX/RADV Vulkan with active validation,
including staged-candidate asset cleanup, all-26 startup, town/farmhouse and Quinn.
These bounded debug samples do not establish production streaming or latency.

Next: remaining retirement/reversal/supersession/asymmetric-failure lifecycle
checks before adoption. The normal launcher remains monolithic revision 2 at
30 Hz. All changes stay local. The user confirms no alternate Elven Realm source
is available; missing Git LFS payloads leave full equivalent import pending.
[Live-publication contract](TOWN_LIVE_PUBLICATION.md).

Qualified actual-town client startup/resource continuation, 2026-10-20: ten
packages and all 26 now load in ordinary encrypted two-client Bevy sessions.
The first actual run exposed an offline bounds mismatch; authoring now binds
transformed mesh-local AABB corners, matching the unchanged strict client guard.
Only a new stage's index changes; package/image bytes and the launcher remain.

Two actual native cases, four reports, pass on RX 7900 XTX/RADV Vulkan with active
validation, exact readiness/identity, shared-image handles, live Avian, camera,
pose/mesh-clearance checks and zero unexpected recovery. Sampled per-client peak
RSS is 1.82–1.83 GiB initially and 2.30 GiB for all packages; driver VRAM residency
is 1.54–1.60 / 1.84–1.85 GiB. These are warm-cache debug qualification samples,
not release/latency percentiles or hardware minimums. Fresh fmt, denied-warning
Clippy and 88 Python tests pass; Rust binaries retain the preceding checkpoint.

Next: actual live candidate/staging/resource lifetimes and remaining handover
lifecycle cases, then map regressions before adoption. The normal launcher stays
monolithic revision 2 at 30 Hz. All work stays local.
[Contract, measurements and limitations](TOWN_CLIENT_RESOURCES.md).

Qualified finite-world continuation, 2026-10-20: chunk-session schema 2 binds an
explicit checked global rectangle into admission identity v4. Conservative local
planes constrain shared authority movement independently of loaded/ready chunk
coverage; internal seams and vertical motion stay open, and lookahead clamps to
the finite world. Schema-1 identities and wire/ALPN 11 remain unchanged.

Fresh fmt, denied-warning Clippy/build, **158 distinct Rust tests** and **85 Python
tests** pass. The workspace run passes 157 tests, followed by a four-case targeted
run adding the new finite-world recovery case. Three private tests ignored by
default separately pass, including source-town rectangle preservation and
ten-to-26-package publication. **Six actual two-client native cases**, twelve
reports, pass on RX 7900 XTX/RADV Vulkan with active validation: finite X/Z wall
pressure, delayed negative independent-frame publication, town, farmhouse and
Quinn. No unexpected recovery occurs; all changes remain local.

Next: actual-town client loading/staging memory and latency, then remaining
handover reversal/supersession/asymmetric-failure checks before adoption. The
normal launcher remains monolithic revision 2 at 30 Hz. No whole task/phase,
full-town perimeter, native Windows or MMO capacity claim follows.
[Contract and evidence](WORLD_EDGES.md).

Elven model/source continuation, 2026-10-20: checked modern model-ID candidates
and an older explicit metadata table match 1,109 distinct asset/fileID references.
The source package contains 30 FBX and two PNG Git LFS pointers; the demo closure
reaches 26 missing FBXs and one PNG. Expected blob hashes/sizes are recorded;
18 additional override-target IDs remain unmapped. Full equivalent scene export
therefore remains open. Fresh fmt, denied-warning Clippy and 85 Python tests pass.
No Rust runtime/render changes or default-map adoption occurred. Core row-5 work
can continue independently while complete sources are located.
[Evidence and source-completeness gate](SYNTY_ELVEN_IMPORT.md).

Elven Realm source/import checkpoint, 2026-10-20: selected source payloads now
retain hash-bound Unity metadata. The recursive dependency audit identifies
model-prefab ID mapping and unresolved serialized references before full-scene
assembly. One actual pillar converts to standard GLB with its source palette;
all 404 triangle positions/winding/UVs and palette pixels pass independent export
comparison. Khronos validation is clean. A native Bevy capture passes on RX 7900
XTX/RADV Vulkan with active validation. The material is explicitly base-colour
only; the full scene, collision and game adoption remain open.
Fresh fmt, denied-warning Clippy and 80 Python tests pass; Rust runtime/binaries
are unchanged. All assets/output remain ignored and changes stay local.
[Scope, evidence and remaining acceptance](SYNTY_ELVEN_IMPORT.md).

Qualified isolated Synty preview, 2026-10-20: selected Base Locomotion FBXs
now convert to standard GLB with their own palette, 50-joint skin and four
baked Idle/Walk/Run/Jump clips. Source bind poses differ, so source world poses
are baked onto the sample skin. An identity mesh root fixes ordinary bounds/
skinning agreement at 1.793552 metres. Both GLBs validate without errors/warnings.
Independent source-pose comparison passes all 175 sampled frames, and four actual
native Bevy animation cases pass on RX 7900 XTX/RADV Vulkan with active validation.
The walk preview was opened on the user's desktop.

The full source skin is retained separately. The standard-renderer four-weight
preview changes 42 source vertices; sampled Jump deformation reaches 6.974 mm,
so the initial 5 mm skin gate stays failed. An explicit preview flag permits
rendering without claiming exact source skin or production character adoption.
Fresh fmt, denied-warning Clippy and **75 Python tests** pass. Rust binaries are
unchanged from the preceding 154-test/eight-native-case checkpoint; that suite
was not rerun for these offline tools/assets. All private assets/build outputs
remain ignored and all work stays local.

Next asset step: Elven Realm representative mesh/material and assembled prefab
scene conversion. The core finite-world edge, actual-town client/resource and
remaining handover gates remain open. The normal game character/map and 30 Hz
default are unchanged. [Preview contract and commands](SYNTY_CHARACTER_PREVIEW.md).
[Evidence](VALIDATION.md).

Qualified runtime capacity continuation, 2026-10-20: the hard ceiling is now
32 with explicit per-session budgets, checked global control addresses, world
identity v3 binding the budget, and wire/ALPN 11. Existing fixture budgets and
bounded datagram/input/snapshot/queue limits remain unchanged. Actual native
startup supports 32 roots; delayed publication adds six required roots while
retaining the old root, shared images and matching live Avian.

Fresh fmt, denied-warning Clippy/build, **154 Rust tests** and **70 Python tests**
pass. Three private tests are ignored by default and separately pass: actual town
ten-to-26-package live authority publication, both house routes at 20/30/60 Hz
without recovery, and all staged shared-image dependencies. A final 27-test
library run verifies corrected current-version input-negative fixtures.
**Eight actual two-client native cases**, 16 reports, pass on RX 7900 XTX/RADV
Vulkan with active validation, including negative independent frames, failed
image retention, both map regressions and Quinn. Imported sources retain their
hashes. All work remains local.

The default remains monolithic town revision 2 at 30 Hz. The 26-package town
runtime session is opt-in/headless-qualified only. Next: explicit finite-world
bounds and edge/corner qualification, then actual-town client loading/resource
measurements and remaining handover lifecycle cases before adoption. No complete
task/phase, Windows or MMO capacity claim follows.
[Contract and scope](CHUNK_CAPACITY.md). [Executed evidence](VALIDATION.md).

Qualified budget-analysis continuation, 2026-10-20: the new offline tool
verifies actual package/image bytes and geometry envelopes, reconstructs source
spawn footprints and measures conservative actor/recovery/lookahead unions.
The town needs 18 providers for one capsule near a corner, 22 for two actors,
and at most all 26 packages for this finite index. Five conservative XZ actor
placements with recovery already reach that upper bound; these ignore terrain
height, obstacles, reachability and future finite-world bounds.

All 26 packages together contain 4,001 static colliders, 698,655,860 shared PNG
bytes and an estimated 1,514,491,904 base RGBA8 bytes. These are verified content
counts/estimates, not live preparation latency or measured CPU/GPU residency.
Fresh fmt, denied-warning Clippy and **68 Python tests** pass. No Rust application,
renderer or asset files changed; the previous native/Rust checkpoints remain
historical evidence. A 32-package hard ceiling / 26-package town session is now
a concrete proposal with explicit protocol, admission, world-edge, staging and
native acceptance checks. Runtime capacity remains four and the launcher stays
monolithic revision 2 at 30 Hz. Continue with that runtime qualification; no task
or phase is complete. All changes remain local. [Measurements and next gate](CHUNK_BUDGETS.md).

Qualified offline partition continuation, 2026-10-20: the actual town now has
26 new source-preserving packages. Explicit terrain and matching collision clip
at 128-metre planes; each static model mesh instance has one owner and retains
its ancestor transforms and original buffers. Per-package asset graphs are
pruned, while canonical shared PNG dependencies remain shared.

Independent source comparison verifies all 2,532 model mesh instances, 256
terrain groups, terrain height/UVs/surface winding, collision surfaces/winding
and 3,976 movement/camera boxes. All 26 actual packages separately pass the
headless Rust manifest/collision/image verifier. Fresh fmt, denied-warning
Clippy and **65 Python tests** pass. Two new actual two-client native cases pass
on RX 7900 XTX/RADV Vulkan with active validation: generated original geometry,
then signed billion-metre global positions with independent query frames. These
are four reports with delayed publication, shared image handles, matching live
Avian, retained roots and no unexpected recoveries. Rust application sources and
binaries are unchanged from the preceding 151-test checkpoint.

The town output totals 8,470,060 GLB bytes and 12,805,430 collision JSON bytes.
Each package uses at most 139 textures, compared with 491 in the source. Actual
content requires up to nine providers per cell and ten at the current spawns;
the four-package proof budget therefore prevents active metadata publication.
Only a candidate town index is emitted. The launcher remains the qualified
monolithic revision 2 at 30 Hz. Next: measured residency/staging budgets,
world-edge policy, remaining handover lifecycle checks and actual-town runtime
qualification before adoption. No entire task/phase or Windows/MMO capacity
claim follows. All changes remain local. [Contract](WORLD_PARTITIONING.md).
[Executed evidence](VALIDATION.md).

Qualified shared image continuation, 2026-10-20: manifest schema 2 binds canonical
PNG dependencies, colour space and sampler settings. Normal chunk clients verify
URI/path/hash/header budgets before standard Bevy loading; prepared roots ignore
unbound editor metadata. Native capture proves compatible handles are shared and
incompatible colour/sampler variants remain distinct. Failed dependencies retain
the active scene. An arrival-order regression also fixed unload-only offer churn:
spare packages remain until a new load needs the same bounded resident budget.

Fresh fmt, denied-warning Clippy/build, **151 Rust tests** and **56 Python tests**
pass. Two private-data tests are ignored by default and separately pass: both
house routes at 20/30/60 Hz with zero recoveries, and actual staged town package/
image validation. **Eighteen actual two-client native cases**, 36 reports, plus
both retained startup rejection checks pass on RX 7900 XTX/RADV Vulkan with active
validation and the same final binaries. All four imported source documents retain
their original hashes. All changes remain local.

The unchanged town was staged as a 5,476,980-byte GLB plus 488 shared resources
containing the original 698,655,860 PNG bytes. Independent comparison confirms
original geometry, transforms, material and collision semantics, and PNG bytes.
This changes placement, not total image compression or terrain partitioning.
Next: prune real node/mesh/material/texture graphs, partition actual terrain and
matching collision, and measure residency/load budgets. The launcher remains the
revision-2 monolithic town at 30 Hz. Windows/crowds/production streaming and whole
phase/task completion remain unqualified. [Contract](SHARED_IMAGES.md).
[Executed evidence](VALIDATION.md).

Qualified geometry contributor continuation, 2026-10-20: schema-2 chunk bounds
now bind dataset identity and determine complete-cell coverage, demand, readiness,
protected actors/recovery and retained presentation. A loaded owner floor alone
does not permit movement without a crossing geometry provider. Off-thread
preparation checks actual collider extents; normal clients check actual nested
Bevy mesh bounds before initial readiness or replacement publication. The new
offline tool derives combined static GLB/collision bounds without changing
packages. Schema-1 experiments retain their original identity and limited scope.

Fresh fmt, denied-warning Clippy/build, **144 Rust tests** and **51 Python tests**
pass. One private-map test is ignored by default and separately passes both house
routes at 20/30/60 Hz with zero recoveries. **Fifteen actual two-client native
cases**, 30 reports, plus invalid initial GLB and initial extent rejection pass
on RX 7900 XTX/RADV Vulkan with active validation and the same final binaries.
New cases cover nonlocal geometry, delayed independent frames and valid GLB extent
rejection/retention. All four imported engineering sources retain their hashes.

Next: shared immutable image bindings, then actual imported terrain/collision
partitioning and measured residency/load budgets. The town's 704.1 MB GLB contains
698.7 MB of embedded images; copying that image set into each chunk is not an
adoption path. The launcher remains monolithic, 30 Hz remains default, native
Windows/crowds/production streaming remain unqualified, and no whole task or phase
is complete. All changes remain local. [Contract](CHUNK_CONTRIBUTORS.md).
[Executed evidence](VALIDATION.md).

Qualified paired handover continuation, 2026-10-20: wire/ALPN 10 adds authority
chunk offers and an opt-in `--chunk-stream` policy. Normal Bevy clients stage one
replacement, protect local/remote/recovery coverage, await exact withdrawal
acknowledgement and publish matching roots/actual Avian generations. Full
readiness waits for live BVH and render readiness. Delayed preparation, failed
replacement retention and distant independent frames pass on original fixtures.

The wider suite exposed sloped-wall contact drift. A deterministic multi-rate
regression and head-on Avian feature-manifold refinement fix it while preserving
exact contacts, tiny tangential input, diagonal sliding and retreat. Fresh fmt,
denied-warning Clippy/build and **140 Rust tests** pass (one private-map test
ignored by default and passed separately for both house routes at 20/30/60 Hz,
zero recoveries). The unchanged Python suites pass 46 tests. **Thirteen actual
two-client native cases**, 26 reports, plus invalid initial GLB rejection pass on
RX 7900 XTX/RADV Vulkan with active validation and the same final binaries.

The launcher remains the monolithic town; 30 Hz stays default. General geometry
contributors, imported tile partitioning, measured content budgets and wider
handover/streaming qualification remain open. Native Windows and MMO capacity
remain unqualified. No whole task or phase closes here. All changes remain local.
[Handover contract](CHUNK_HANDOVER.md). [Executed evidence](VALIDATION.md).

Qualified movement continuation, 2026-10-20: capsule sweeps now enforce authoritative/client chunk
coverage for gravity-driven horizontal motion as well as input. A reproduced
steep-slide boundary failure is fixed; targeted signed/distant 20/30/60 Hz cases
pass, including missing-corner coverage and release after loading. New support
validation/fallback avoids the large-convex-floor regression while preserving
exact persistent contacts. Fresh fmt, denied-warning Clippy/build, 134 Rust tests
and 46 Python tests pass (one private-map test ignored by default and passed
separately across both houses at 20/30/60 Hz with zero recoveries). Ten actual
two-client native cases and invalid-GLB rejection pass on RX 7900 XTX/RADV Vulkan
with active validation. Next: retain local/remote presentation and recovery
coverage, then pair live standard-Bevy/Avian publication with authority offers.
Normal chunk rendering remains initial-only; the launcher stays monolithic and
30 Hz remains default. Native Windows and MMO capacity remain unqualified.
[Contract](CHUNK_BOUNDARIES.md). [Evidence](VALIDATION.md). All changes remain local.

Qualified continuation, 2026-10-20: wire/ALPN 9 adds exact readiness acknowledgements emitted
after a fixed simulation step. Client confirmation requires the latest report and
a snapshot at or after its application tick, preventing old acknowledgements from
confirming a new request. Fresh fmt, denied-warning Clippy/build, 129 Rust tests
and 46 Python tests pass (one private-map test ignored). Nine actual two-client
native cases and invalid-GLB rejection pass on RX 7900 XTX/RADV Vulkan with active
validation. A diagnostic wall-steering fix passes two actual town runs against
unchanged contact limits; the earlier failed run is retained as rejected evidence.
The version-8 checkpoint below remains historical. Next: retained presentation,
recovery/gravity boundary coverage and paired dynamic publication, then explicit
geometry contributors/tile partitioning. Normal rendering remains initial-only;
the launcher stays monolithic and 30 Hz remains default. Windows remains
unqualified. [Contract](CHUNK_READINESS.md). [Evidence](VALIDATION.md).

Resumed at the user's request, 2026-10-20. Wire/ALPN version 8 now gates each
networked chunk actor's intentional horizontal movement on its own client's
reported coverage, in addition to authority residency. The normal Bevy client
reports only after initial assets and live Avian queries are ready. Fresh fmt,
denied-warning workspace Clippy, 126 Rust tests and 46 Python tests pass (one
private-map test explicitly ignored); the final workspace build also passed.
Eight actual two-client native cases and invalid-GLB rejection pass on RX 7900
XTX/RADV Vulkan with active validation. Windows remains unqualified. Normal chunk
rendering is initial-only; next are acknowledged withdrawal/retained presentation,
paired dynamic publication and geometry contributors/tile partitioning. The
launcher remains monolithic and 30 Hz stays default. [Contract](CHUNK_READINESS.md).
[Evidence](VALIDATION.md). All changes remain local.

Opt-in normal chunk startup, 2026-10-20: `--chunk-world` now uses shared bounded
startup metadata in the actual server and Bevy client. Initial standard GLB roots,
trusted spawns, matching live Avian collision, camera and ordinary encrypted player
presentation work through signed distant frames and render/wire rebases. The
version-2 session fingerprint also binds initial chunk IDs so an initial-only
client cannot admit with a different loaded set. Stable dataset identity and
client/server query frames remain separate. That admission guard and the initial
native path pass. A continuation now
protects wire/render frames for distant loaded actors and previous presentation
poses: the original 5.1 km two-client case passes with bounded common frames.
Fresh fmt, denied-warning Clippy/build, 123 Rust and 46 Python tests pass
(one private-map test explicitly ignored). Seven actual two-client native cases,
including the normal town route, and real GLB startup failure pass on RX 7900 XTX/
RADV Vulkan with active validation. Windows remains unqualified. 30 Hz stays
default. [Contract](CHUNK_STARTUP.md).
Normal executable loading is initial-only. Dynamic paired readiness/publication,
actual imported tile partitioning and broader streaming budgets remain open;
the launcher retains the monolithic town. All changes remain local.

Resumed at the user's request, 2026-10-20. The chunk-activation scaffold is now
integrated into fixed-step authority and a bounded owner-side management path in
the actual Renet runtime. Current capsule footprints/support/clearance and
world/generation guards protect publication; missing-neighbor movement preserves
jump/gravity. Targeted original-geometry and real two-client UDP cases pass.
Workspace checks pass: 116 tests plus the final expanded six-test chunk target
(one new case; 117 distinct passing tests), with the existing private-map test
explicitly ignored. Fresh fmt and denied-warning Clippy pass. Normal map/client
residency adoption is the next gate; the qualified town remains monolithic.
[Contract and scope](CHUNK_ACTIVATION.md). All changes remain local.

Schema-3 native qualification/adoption, 2026-10-20: general mesh collision and
explicit spawns now pass real two-client mesh/map checks, signed distant rebases,
spawn-free chunk publication/replacement/rollback/unload and normal-world startup
rejection. The launcher now selects `build/development-world-town-v3`, logical
`theia.development.lordaeron-town` revision 2; both actual default launch cleanup
cases verify that package. The tile still uses its existing conservative boxes
and authored floors; the new format does not infer complete model collision.
Native inspection fixed first-frame zero-delta Avian BVH freshness and a
highest-surface camera error inside general mesh rooms. Regressions and native
visibility/camera-distance checks now cover both. [Contract](GENERAL_COLLISION.md).
Fresh fmt, denied-warning Clippy/build, 110 Rust and 46 Python tests pass. Six
native two-client cases, four chunk cases, startup rejection and explicit/default
launcher cleanup pass on RX 7900 XTX/RADV Vulkan with active validation. Windows
remains unqualified. 30 Hz stays default. Next: live moving-actor chunk activation
and normal residency adoption; prediction remains open. [Evidence](VALIDATION.md).

Paused at the user's request, 2026-10-19, after a safe verification checkpoint.
Collision schema 3, explicit validated spawns, query-only chunks and static GLB
import/migration tooling are implemented. A combined-mesh low-ceiling grounding
bug is fixed. Staged revision-2 development maps preserve their geometry/visuals;
the launcher still defaults to the previously qualified schema-2 town. No new
native schema-3 qualification/default adoption is claimed. Resume with the
[checkpoint and next gate](GENERAL_COLLISION.md), then moving-actor chunk activation.
Fresh locked build, fmt, denied-warning Clippy, 108 Rust and 46 Python tests pass.
Separate headless staged-v3 town/farmhouse routes pass at 20/30/60 Hz with zero
recoveries. No native/GPU/Windows qualification was started for this new slice
before the user requested stopping. Earlier native records below are historical
baselines. Changes remain local. [Evidence](VALIDATION.md).

Imported-map controller adoption, 2026-10-19: normal imported authority now uses
the shared capsule controller, including gravity/jumping and nearby body support.
Trusted legacy spawn locations are resolved/validated once. Normal-world adoption
adds finite collision barriers; individual residency chunks retain open seams.
Unsupported falls or rejected solver states recover only that actor to its trusted
spawn, preserving input sequence/session and held jump. Recovery is observable and
ordinary route qualifiers reject it. [Contract](KINEMATIC_CONTROLLER.md). Content
profiles/packages stay unchanged; conservative boxes are still proxies. Explicit
spawn metadata, arbitrary collision mesh authoring and live actor/chunk activation
remain open. 30 Hz stays default; prediction and jump animation remain open.
Fresh fmt, denied-warning Clippy/build, 103 Rust and 43 Python tests pass. The
separate private-map test passes all six house-route cases at 20/30/60 Hz with zero
recoveries. Final native graybox/distant-rebase, stationary map jump, town/farmhouse
routes, full-tile Quinn and both launcher cleanup checks pass on RX 7900 XTX/RADV
with active Vulkan validation and zero unexpected recoveries. Windows remains
unqualified. [Evidence](VALIDATION.md).

Shared-controller continuation, 2026-10-19: graybox authority and the offline
sandbox now use one general Avian capsule controller for walls, headroom,
steps/slopes, stacked floors, gravity/jumping and nonblocking actors. Wire v7
adds held jump input; authority owns vertical/grounding state. Connected clients
present authoritative motion. [Contract](KINEMATIC_CONTROLLER.md). The imported
map deliberately retains its legacy grounded walker during migration; no normal
map jumping, prediction or live authority chunk adoption is claimed. Next:
qualify imported content/spawn/fall recovery with this controller, then moving
actor/chunk activation. 30 Hz remains default.
Fresh fmt, denied-warning Clippy/build, 98 Rust and 41 Python tests pass. Release
eight-peer scheduler qualification passes at 20/30/60 Hz. Final-binary native
Renet/Quinn jump cases at signed billion-metre origins, packaged town/farmhouse
regressions and both default launcher cleanup cases pass on RX 7900 XTX/RADV with active Vulkan validation. Windows
remains unqualified. [Evidence](VALIDATION.md).

Bevy chunk-publication continuation, 2026-10-19: the actual client now has an
opt-in `--chunk-smoke` diagnostic that stages one verified residency candidate on
Bevy's I/O task pool, loads standard GLB scenes, preserves retained roots and
publishes ready hierarchy visibility with matching actual Avian bodies. Last-stage
queries check the live BVH throughout transitions. Native screenshots independently
prove red unloads/blue loads, retained green remains, failed glTF preserves the old
pair and final unload removes all geometry. Origin and ±1-billion-metre cases pass.
[Contract](CHUNK_PRESENTATION.md). This is original-fixture qualification, not normal
tile partitioning, production streaming or multiplayer chunk adoption. Next: shared
general kinematic movement, then integrate bounded residency with live authority/
presentation after its activation/owner checks are qualified. 30 Hz remains default.
Fresh fmt, denied-warning Clippy/build, 85 Rust and 41 Python tests pass. Four
native chunk cases and the normal Renet two-client graybox pass on RX 7900 XTX/RADV
with active Vulkan validation. Windows remains unqualified. [Evidence](VALIDATION.md).

Chunk-residency precursor, 2026-10-19: an isolated original three-chunk fixture
now proves half-open ownership, explicit bounded residency and real Avian collision
load/unload. Protected actor owners cannot be removed; corrupt candidates preserve
the active scene. Origin and ±1-billion-metre runs pass with at most two active
chunks and a real thin-wall capsule hit. [Contract](CHUNK_RESIDENCY.md). Visual-root
descriptors track active ownership, but are not rendered/streamed assets. The normal
imported tile remains monolithic; synchronous proof loading never runs in its
simulation loop. Next: bounded Bevy visual consumption/publication, then the general
shared controller. The 30 Hz default is unchanged.
Fresh fmt, denied-warning Clippy and all 84 Rust tests pass; 41 Python tests pass
(37 asset plus four GLB). All three headless executable residency proofs pass.
Normal server dependencies remain renderer-free. No new renderer path is claimed
by this precursor; the immediately preceding native map checks remain valid
historical evidence. [Validation](VALIDATION.md).

Content-package continuation, 2026-10-19: the normal map launcher now requires a
versioned manifest with stable logical ID/revision and separate collision/visual
lengths and SHA-256 bindings. Shared Rust loading verifies collision packages;
the client streams visual verification before standard Bevy asset loading. The
headless server does not read visual files. Wire v6 carries the package fingerprint;
real secure admission rejects stale revisions. Existing ignored map fixtures have
revision-1 packages; malformed present manifests fail closed, while standalone
historical unmanifested fixtures retain legacy identity. [Contract](CONTENT_PACKAGES.md).
This is the row-5 content prerequisite. Chunk residency, general mesh collision,
stacked floors and falling/jumping remain open; 30 Hz remains the server default.
Fresh checks pass fmt, denied-warning Clippy/build, 79 Rust and 39 Python tests.
Packaged town/farmhouse native routes, Quinn graybox and both default launcher
cleanup cases pass on RX 7900 XTX/RADV with active Vulkan validation. Windows
remains unqualified. [Evidence](VALIDATION.md).

Scheduler continuation, 2026-10-19: the existing Renet authority now reports
bounded timing histograms, skipped intervals and separate simulation/transport-pump
diagnostics. A real eight-client UDP qualifier covers 20/30/60 Hz, burst/replayed/
wrong-epoch traffic, simulation-time input expiry, a controlled 250 ms authority
stall, recovery and removal of an over-budget sender while seven peers continue.
[Contract, measurements and limits](SCHEDULER_QUALIFICATION.md). This qualifies the
bounded graybox scheduler, not MMO capacity or full-map performance. Verification
also exposed and fixed cold query-scene initialization deadlock under concurrent
Bevy callers, with a two-worker/eight-caller regression. General action
schemas and crowd/render/soak baselines remain open. Next: shared content/collision
and general controller contracts, preserving existing map regressions.
Fresh verification passes fmt, denied-warning workspace Clippy/build, 73 Rust and
35 Python tests, the release eight-client qualifier and a native two-client
Renet graybox check with active Vulkan validation on RX 7900 XTX/RADV. Linux only;
Windows remains unqualified. [Evidence](VALIDATION.md).

Client render/physics rebase continuation, 2026-10-19: connected clients now
recentre local scenery, pawn/interpolation presentation and actual Avian static
collider Positions/Transforms together. Canonical globals and wire v6 are unchanged.
Standard PostUpdate physics refreshes the collider tree before Last-stage ground
probes/capture; camera queries explicitly cross the render/content-frame boundary.
[Contract and scope](FLOATING_ORIGIN.md). Server queries remain in the fixed bounded
content frame; multi-chunk ownership, streaming and dynamic-body rebasing remain
open. Earlier coordinate notes below describe the prior fixed client anchor.
Fresh verification passes fmt, denied-warning Clippy/build, 70 Rust and 35 Python
tests, five native two-client checks and both actual launcher cleanup cases.
The blacksmith/farmhouse routes exercised 22/24 shifts on their entering clients,
with actual rebased Avian queries and intact animation/camera checks. Device:
RX 7900 XTX/RADV with active Vulkan validation. Windows remains unqualified.
[Evidence](VALIDATION.md).

Coordinate/identity slice, 2026-10-19: connected actors now store checked f64
global positions; both transports carry an incarnation epoch and explicit snapshot
origin. The client reconstructs global positions before projecting into its fixed
local content frame. Wire origins rebase without moving authority poses or static
geometry. Protocol/ALPN is now v6 with a shared 76-byte welcome. Wrong-epoch input
is rejected before sequence/pending-input state changes. Renet remains the default.
[Contract, evidence and limits](COORDINATES_IDENTITY.md). Content/render/physics
anchors remain fixed for this fixture: automatic floating origin, multi-chunk
ownership, streaming and persistent character identity remain open.
Fresh checks pass fmt, denied-warning Clippy/build, 66 Rust and 35 Python tests.
Five native two-client checks pass (positive/negative distant Renet graybox,
distant blacksmith, ordinary farmhouse, distant Quinn), plus both default launcher
cleanup cases. Device: RX 7900 XTX/RADV with active Vulkan validation; Windows
execution remains unqualified. [Validation](VALIDATION.md).

Earlier Renet replacement qualification follows.

Renet replacement qualification, 2026-10-19: `python3 tools/play.py` and the
multiplayer qualifier now default to Renet. Deterministic real UDP profiles pass
10/20% loss, delay, duplication/reorder, monotonic acknowledgement, movement/stop
agreement and bounded queue/traffic checks. Two-second outages expire input and
recover the same session; a longer outage removes it after about five seconds.
Replaying 384 old-session packets cannot move a fresh-ticket replacement actor.
[Measured scope](RENET_QUALIFICATION.md). Quinn remains an explicit regression
option; standalone legacy CLI meanings are preserved. Next is live global/local
coordinate and entity/session identity integration, not combat or more houses.
Fresh qualification passes fmt, denied-warning Clippy/build, 59 Rust tests and
35 Python tests. Four native two-client checks pass (default Renet graybox,
blacksmith, farmhouse; explicit Quinn graybox), plus both actual launcher cleanup
cases. Device: RX 7900 XTX/RADV with active Vulkan validation. Windows execution
remains unqualified. [Evidence](VALIDATION.md).

Earlier executable-bridge continuation, 2026-10-19: the active task list now has a
[dependency-based execution order](engineering/EXECUTION_ORDER.md), with
foundations prioritized over gameplay. An opt-in [Renet executable bridge](NETWORKING_RENET.md)
connects the actual server, Bevy client and launcher using secure local tickets,
world checks and existing authority. `tools/play.py --transport renet --clients 2`
selects it. Quinn remains the default pending controlled network qualification.
Fresh continuation checks pass fmt, denied-warning Clippy/build, 56 Rust tests,
35 Python tests and three native server/two-client sessions (Renet graybox and
blacksmith route, Quinn graybox) on RX 7900 XTX/RADV with active Vulkan validation.
See [validation](VALIDATION.md). Windows execution remains unqualified.
No entire roadmap phase is complete; global runtime coordinates, general movement,
external identity and prediction remain outstanding.

Avian migration, 2026-10-19: the normal client now installs standard Avian physics
plugins and static world colliders. Avian `0.8.0-dev` is pinned to Git revision
`1070759b2145dc078abedace91dc7fe78013a45c` alongside exact Bevy `0.20.0`.
Server/offline movement, ground queries and camera volume queries use the shared
Avian backend. Characters remain nonblocking; connected movement remains server
owned. Existing terrain/building profiles are retained, with capsule sweeps
replacing custom box sweeps. Wire/ALPN version 5 prevents mixing old/new backends.
See [physics scope](PHYSICS_AVIAN.md). General multi-floor movement, gravity/jumping,
automatic building collision, prediction and navigation remain unfinished.
Earlier alignment notes below describe the prior implementation state.

Fresh qualification passes formatting, denied-warning workspace Clippy/build,
53 Rust and 35 Python tests, plus three native server/two-client Avian sessions
(graybox, blacksmith and farmhouse) on RX 7900 XTX/RADV with active Vulkan
validation. Windows graphics remain unqualified. [Evidence](VALIDATION.md).

## Earlier foundation migration

Alignment implementation, 2026-10-19: the handoff is now adopted as the active
architecture. Contributor rules and the future queue are aligned; scripted
platforms are allowed but unimplemented. A new headless `theia-core` supplies
checked f64/global and f32/local coordinate/frame contracts, seam tests and
timestep utilities. The existing authority now supports configurable 20/30/60 Hz
(30 default), preserved movement speed/input expiry and shutdown deadline metrics.
Wire/ALPN version 4 includes validated tick rate, used by client interpolation.
Renet/netcode 2.0.0 secure transport helpers and a two-client real UDP proof pass;
the desktop launcher still uses Quinn. The running map remains the local f32
terrain fixture, with no Avian/general collision or prediction yet.

Fresh full-workspace checks now pass using the existing ignored native SDK and
asset-tool environment: formatting, denied-warning Clippy, build, 51 Rust tests
and 35 Python tests. This resolves the environment blockers from the earlier audit
below. See [current alignment](engineering/ALIGNMENT.md),
[live task status](engineering/TASK_STATUS.md) and [validation](VALIDATION.md).

## Earlier planning and audit snapshot

Planning update, 2026-10-19: the user supplied a more defined engineering handoff.
Its four Markdown documents are preserved under [engineering](engineering/README.md),
with a [local repository assessment](engineering/REPOSITORY_ASSESSMENT.md). Future
work follows that reconciled queue. This documentation integration changes no
runtime behaviour: Renet, Avian, f64 global/local coordinates, prediction and the
planned gameplay systems remain unimplemented. It is not a fresh repository or
a rewrite. The assessment recommends preserving this repo and migrating foundations.

Fresh audit checks pass formatting, denied-warning headless Clippy and 23 headless
Rust tests (16 unit, 7 actual QUIC loopback). Full workspace Clippy/tests are
blocked by missing `wayland-client.pc`; the default Python asset suite cannot
import nine modules without its numpy/trimesh dependencies. Historical full-suite
and GPU results below remain dated evidence, not fresh audit results.

## Prototype implementation record — 2026-10-16

Bevy 0.20.0 client, local multiplayer, full-tile outdoor development world, orbit
camera and a separate GLB inspector,
2026-10-16.

Normal development launch: `python3 tools/play.py`, using the complete supplied
tile with enterable farmhouse and blacksmith main room, starting near the latter.

Implemented: a standard Bevy PBR scene made from original primitive geometry,
directional light/shadows, a local capsule pawn, fixed-step keyboard movement,
orbit/zoom camera, camera-relative controls, reset/quit and screenshot smoke
mode. There is no dependency on the old custom engine or its benchmark crate.

Linux build, denied-warning Clippy, formatting, 42 Rust tests and 35 offline
Python asset-tool tests pass.
Private native Wayland runs on RX 7900 XTX / RADV pass scripted movement,
static-world/camera checks and real 1280x720 screenshots with active Vulkan validation
and no validation errors.
The initial Bevy 0.19.1 probe exposed startup presentation diagnostics; the final
0.20.0 run passes without an application renderer workaround.
[Executed evidence and reproduction](VALIDATION.md).

The connected mode now uses one Rust authoritative server at 30 Hz, Quinn QUIC,
server-assigned identities, bounded input intents and full shared snapshots.
Clients cannot submit coordinates. Server speed, diagonal normalization, bounds,
stale-input rejection and input expiry control movement. The local pawn follows
server positions directly; remote pawns interpolate over a snapshot interval.
Disconnect removes the player; relaunching reconnects with a new ID. The offline
sandbox remains separate and R reset is disabled when connected.

Tests include player overlap/pass-through, actual encrypted loopback two-client movement,
disconnect/reconnect, untrusted-certificate rejection, malformed input and reordered
input/expiry. Two independent Bevy clients ran on RX 7900 XTX / RADV in a private
headless Wayland display. Both rendered both players and reported matching server
poses; Vulkan validation was active with no validation errors. This establishes
local multiplayer walking, not MMO scale or Internet deployment readiness.

Rendering and simulation now share immutable box geometry. The server stops
movement at the building, fixed door and rocks, with conservative pawn clearance
and wall sliding. Thin roads remain walkable on the flat ground. The camera
orbits/zooms and retracts against static geometry; the local body hides when the
eye gets very close. Tests cover door collision through an actual QUIC connection,
camera-relative movement, orbit limits, focus/cursor state and obstruction recovery.
The native world smoke reaches the fixed door and records camera retraction and
restoration before saving its screenshot. [Scope/limitations](WORLD_AND_CAMERA.md).

The user-provided `duskwoodhouse_medium.glb` passes Khronos validation and renders
through Bevy's standard loader in an independent inspection mode. It was moved
from Downloads into `apps/client/assets/reference/` after rendering succeeded,
with original bytes preserved. The file has 425 source positions, 672 triangles
and no UVs, authored normals, materials or textures. Bevy generates flat normals
and a default material. This qualifies geometry-only import, not a textured house.
The inspector frames actual scene bounds, shows a scale reference and can save
a real screenshot/report. It does not create gameplay collision or alter the
authoritative world. [Import workflow and export guidance](ASSET_IMPORT.md).

The newer farmhouse and re-exported human character also render with embedded
textures. The inspector now supports explicit named-clip playback with Bevy's
standard animation graphs. Stand, walk and run have actual sampled joint motion
and native rendered-pose evidence. The farmhouse passes glTF validation cleanly;
the character still has empty clips and incorrect accessor metadata, despite the
selected animations working. Large redundant vertex buffers also need cleanup.
Both source exports remain outside Git and unchanged. [Detailed model checks](ASSET_CHECK_2026_10_16.md).

The additional RustyWolfPup GLB also renders its embedded textures and plays stand,
walk, run and attack in the inspector on RX 7900 XTX / RADV with active Vulkan
validation. Its 66-joint skin and selected clips work, but the source fails strict
glTF validation with 3,084 accessor alignment errors. It remains an external local
compatibility reference, with no NPC/gameplay integration. The same detailed report
records the exact source hash and limitations.

An opt-in [reference game scene](REFERENCE_SCENE.md) now uses the human for local
and remote players, an idle nonblocking wolf enemy placeholder and a static
farmhouse. Player stand/walk/run follows actual movement. Both clients and server
use an enlarged shared solid building envelope; native two-client, locomotion,
wolf pass-through and house/camera checks pass. Derived models stay under ignored
`build/reference-scene/`; the default scene remains procedural. The prepared wolf
has zero glTF errors, while deeper human validation still finds skin-data errors.

The supplied Ruins of Lordaeron map tile also converts from textured OBJ/MTL into
a separately inspected GLB: 65,504 triangles and 256 textured patches, with zero
glTF errors/warnings. A further [static map assembly](MAP_OBJECTS.md) now includes
846 top-level placements and 624 nested building props using shared model parts.
The combined GLB renders through Bevy with no glTF errors/warnings. One empty
smoke emitter has no visible effect. [Static liquid surfaces](MAP_LIQUIDS.md) now
add 30 masked patches using source heights and a generic translucent PBR material.
The inspector waits for render pipeline readiness before captures; CPU asset loading
alone was insufficient for reliable screenshots. The original assembled GLB remains available in the inspector. A separate bounded
[terrain walking patch](TERRAIN_WALKING.md) now has server-owned height and slope
checks, matching visual/collision triangles, world identity checks and two rendered
multiplayer clients. An optional `--player-assets` mode now attaches the existing
human to both local and remote terrain players, with sampled stand/walk/run
evidence from real connected clients. A separate [authored farmhouse placement](TERRAIN_STATIC_OBJECTS.md)
now adds a fixed foundation and shared solid boundary, with server collision and
real camera obstruction/body visibility checks. A new [enterable ground-floor
fixture](ENTERABLE_HOUSE.md) uses measured wall boundaries, source floor triangles,
a permanently open doorway and a fixed ramp. Native networked entry, interior wall
contact, exit and re-entry pass with camera clearance below the ceiling. The subsequent
[full-tile development world](DEVELOPMENT_WORLD.md) now connects the
complete visual assembly to terrain movement, conservative imported object bounds
and blocked water cells. Precise mesh collision, general interiors and swimming
remain outstanding.
[Original map inventory](MAP_CHECK_2026_10_16.md).

No local prediction, general world mesh collision, animation blending,
targeting, combat, quests, inventory, accounts or persistence is implemented.
The wolf currently supplies presentation only, without server-owned NPC state or AI;
the older solid-house fixtures remain closed. The isolated ground-floor fixture
and the original farmhouse and blacksmith main room on the full map now have
authored access. Housing ownership/persistence and loft access are absent.
The network executables currently allow loopback only; eight peers is a prototype
cap, not a qualified capacity. Manual multiplayer keyboard interaction, adverse
network behaviour and the user-visible walkthrough remain outstanding.

Confirmed gameplay constraints: environment items are static (including doors),
and players never physically collide with other players or NPCs. NPC pass-through
is a requirement for future NPCs; this slice contains players only.

Earlier next-step plan (superseded by the October 19 engineering queue): use the
full supplied tile for the shared target/ability/health loop, with quests following.
The current queue first defines and qualifies the foundation migrations while
retaining this map as a fixture. Remaining character skin errors and replacement
original/licensed art still require qualification. The direct GLBs contain PNGs and preserve UVs;
the earlier missing-texture problem is resolved for these new exports.
The default/reference world still uses flat-ground boxes. The opt-in terrain patch
adds sampled height/slope movement; general stair climbing, jumping and arbitrary mesh collision remain unimplemented;
the new house permits only bounded small support seams and its authored ramp.
The [WoW-style game direction](GAME_DIRECTION.md), [network slice](NETWORKING.md)
and [roadmap](ROADMAP.md) distinguish intended gameplay from current capability.

Native Windows graphics and other GPUs remain unqualified.

The full supplied tile is now prepared under ignored `build/development-world/`,
with all 65,504 terrain triangles, the existing assembled props/buildings/liquids,
1,144 conservative object boxes and 1,372 blocked water cells. Runtime profile
schema 2 adds actual tile bounds and nearby spatial lookup. Schema-1 fixtures keep
their prior collision behavior. Wire/ALPN version 3 supports the larger coordinates;
clients and server must be rebuilt together. `python3 tools/play.py` launches the
world with an authoritative server and animated human; `--clients 2` adds a second
client. Native full-map, interior and graybox multiplayer checks and launcher
cleanup pass on RX 7900 XTX / RADV with active Vulkan validation. Buildings in this
map remain solid envelopes, while the separate authored
house fixture retains its interior. This replaces the full-map inspector-only
limitation for outdoor development, without adding NPCs, combat or swimming.

The default launcher now selects `build/development-world-access`, the complete
map with its original placed farmhouse ground floor made enterable. The building
retains its original scale/rotation and furnishings. Source floor triangles,
419 short rotated-wall envelope pieces and a fixed visible ramp qualify entry,
wall contact, exit and re-entry; other buildings remain solid and the loft stays
blocked. Development spawns move to the outdoor approach. Terrain ambient fill
is brighter, and Bevy mesh ray casts supplement static camera clearance without
including characters. Native two-client map-access, outdoor and prior interior
checks pass on RX 7900 XTX / RADV with active Vulkan validation. The diagnostic
harness now waits for both scenes to load before starting timed routes.
[Measured layout, evidence and limitations](MAP_FARMHOUSE.md).

The default launcher now selects `build/development-world-town`: the same complete
tile with both farmhouse access retained and the original blacksmith main room
opened. The new building contributes 226 source floor/four ramp triangles, 576
short main-room wall/exclusion boxes and 463 exterior forge/bay/porch exclusions.
Original source art and furnishings remain intact. Spawns move to its outdoor
approach. The new map's CPU routes pass for both buildings; native two-client
blacksmith entry/contact/exit/re-entry and normal launcher lifecycle checks pass
on RX 7900 XTX / RADV with active Vulkan validation. Camera mesh retractions are
observed on the actual blacksmith route. The forge annex and other unqualified
buildings remain blocked. [Scope and evidence](MAP_BLACKSMITH.md).
