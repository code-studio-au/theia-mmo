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

# Current playable checkpoint — 2026-10-31: travelling magic

P3-06 / initial P5-15: wand-gated homing magic has finite speed/range/lifetime,
static collision and server damage/hit outcomes only at impact. Standard Bevy
shows the travelling bolt. **608 workspace passes / 40 explicit fixture skips in
115 groups**, formatting, denied-warning workspace/all-target Clippy and
**41 Python tests** pass. Real OIDC/PostgreSQL/two-client hit/miss/deflect travel,
legacy melee/miss-feedback and 3440×1440 fullscreen launcher fixture cleanup pass.
All **12 final native starts have zero GPU diagnostics**.
[Behavior, exact evidence, launch and limits](../PROJECTILE_TRAVEL.md).
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
zero GPU diagnostics**. [Behavior, evidence, launch and limits](../HIT_OUTCOMES.md).
Matched source/binaries/art/proof: `build/outcomes-qualified-v1-20261031`.
No new FPS, Windows, crowd or streaming-capacity claim. Earlier checkpoints remain.

# Current playable checkpoint — 2026-10-31: player sword melee

P2-12 / P3-06 / P3-08 / initial P3-14: committed sword equipment now gates a 3 m
mobile melee action with authoritative windup, range/life/LOS and queue checks.
Standard Bevy upper-body attacks preserve standing/walking/running locomotion
locally and remotely. Terrain readiness follows actual terrain pipelines rather
than cosmetic compilation. [Behavior, proof, launch command and limits](../PLAYER_MELEE.md).

Final unchanged source: **571 workspace passes / 40 explicit fixture skips in
113 groups**, formatting, denied-warning workspace/all-target Clippy and
**41 Python tests**. Real two-account OIDC/PostgreSQL/native melee, moving grip,
rewarded life/restart/backend-loss and legacy regressions pass; normal launcher
cleanup passes at captured 3440×1440 fullscreen. All 25 final starts record zero
GPU diagnostics. Quiet one-client near-spawn cadence is **711/713 FPS** at
1920×1200/3440×1440: [measurement and limits](../PLAYER_MELEE_PERFORMANCE.md).
Matched source/binaries/art/proof: `build/melee-qualified-v1-20261031`.
Earlier checkpoints remain preserved. No crowd/streaming or Windows claim.
Changes remain local. Hit/miss/deflect resolution is the next small slice.

# Performance follow-up — 2026-10-31

Unchanged pursuit checkpoint: one native client with the combat rig and active
wolf/pursuit records **713 FPS at 1920×1200** and **712 FPS at 3440×1440 fullscreen**
near spawn. Each uses 15 steady one-second windows, 81 resident chunks and no
unloads; external validation is off. [Exact evidence and limits](../CREATURE_PURSUIT_PERFORMANCE.md).
This is not population/streaming capacity. Player sword melee is the next slice.

# Current playable checkpoint — 2026-10-31: enemy perception, pursuit and return

P3-07 / P4-03 / initial P3-08 and P3-14: the opt-in wolf perceives a live player,
runs on checked navigation, attacks within reach, breaks off at its leash,
returns to its captured patrol position and resumes the saved route.
[Behavior, exact native evidence, launch command and limits](../CREATURE_PURSUIT_NATIVE.md).

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
at 20/30/60 Hz. [Implemented behavior, exact proof and limits](../CREATURE_PURSUIT.md).
Matched headless source/proof: `build/pursuit-authority-qualified-v1-20261031`.
Native OIDC/PostgreSQL/wolf qualification is in progress. The runnable grip
checkpoint stays preserved. No new whole-workspace, FPS/capacity or Windows claim.

# Active slice — 2026-10-31: pursuit policy qualified, server integration next

P3-07 / P4-03: pure perception/chase/attack/leash/return policy passes 108 core
all-target tests, denied-warning core Clippy and formatting. No gameplay profile
or server creature consumes it yet. [Contract, exact evidence and next gates](../CREATURE_PURSUIT.md).
Matched source/proof: `build/pursuit-policy-qualified-v1-20261031`.
The checked navigation prerequisite and runnable grip checkpoint remain preserved.

# Checked prerequisite — 2026-10-31: bounded creature path worker

P3-07 / P4-03: the reviewed static Landmass query is shared with a dedicated
background worker. One outstanding request, bounded package/routes, exact actor /
target-life / destination-revision matching and stale-result expiry pass.
[Implemented contract, proof and limits](../CREATURE_PATH_WORKER.md).

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
remove the layer; death retains precedence. [Behavior, proof and limits](../EQUIPPED_PLAYER_STANCE.md).

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
[Existing behavior, task mappings, prerequisites and acceptance](../COMBAT_WORLD_FOLLOWUPS.md).
P3-17/P3-18 remain unspecified.

# Current playable checkpoint — 2026-10-31: authored prop sockets and player death

P1-10 / P2-09 / P3-08 / initial P3-14: the opt-in POLYGON combat conversion
preserves `Prop_L` / `Prop_R`, attaches development equipment to the authored
right-hand socket, and plays the local player's authoritative death once before
restarting animation for a fresh life. Actual two-account OIDC/PostgreSQL/native
gameplay passes at 1920×1200, the legacy rig passes at 1280×720, and normal
launcher cleanup passes all three modes at captured 3440×1440 fullscreen.
[Behavior, exact evidence and limits](../PLAYER_ANIMATION_SOCKETS.md).

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
modes at captured 3440×1440 fullscreen. [Behavior, qualification and limits](../WOLF_CREATURE.md).

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
[Implemented behavior, exact GPU diagnostics and limits](../PLAYER_LIFE_NATIVE.md).

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
workspace/all-target Clippy pass on unchanged final source. [Behavior and limits](../PLAYER_LIFE_PREDICTION.md).
Exact source/proof: `build/player-life-prediction-qualified-v1-20261031`.
Prior full workspace evidence remains 519 passes / 40 skips, not rerun here.
Normal executable/bundle and Bevy/native adoption remain open. Changes stay local.

# Active development — 2026-10-31: encrypted player-life runtime qualified

P3-04 / P3-07 owner-only health, incoming windup, death and respawn now pass actual
two-peer encrypted UDP at 20/30/60 Hz under delay/loss/reordering. Full workspace:
**519 passed / 40 explicit skips in 111 groups**, formatting and denied-warning
workspace/all-target Clippy on unchanged final source. Five new runtime checks
include one specifically identified server-queue unit test. [Contract, evidence scope and next acceptance](../PLAYER_LIFE_RUNTIME.md).
Exact source/proof: `build/player-life-runtime-qualified-v1-20261031`. Normal
executable/bundle and Bevy/native adoption remain open; new-mode OIDC, SQL health
durability and native rendering are not claimed. The preserved playable checkpoint
remains `build/character-menu-qualified-v1-20261031`. Changes stay local.

# Active development — 2026-10-31: life codec/receive barrier qualified

P3-04 / P3-07: bounded owner-only PL01 frames and life/movement correction gating
pass eight wire tests, 100 core / 76 network-library tests, formatting and denied-
warning workspace/all-target Clippy on unchanged final source. Real completed
World health/death/respawn states round-trip through this codec. [Contract and
remaining adoption gates](../PLAYER_LIFE_PROTOCOL.md). Exact source/proof:
`build/player-life-wire-qualified-v1-20261031`. New-mode Renet publication/client
bridge, executable/bundle and Bevy/native adoption remain open. No new-mode
multiplayer/native result is claimed; prior playable checkpoints stay preserved.
Changes remain local.

# Active development — 2026-10-31: player life/retaliation headless authority qualified

P3-04 / P3-07: the new opt-in World mode now runs checked retaliation, transient
health/death and supported streamed respawn with fresh life/movement streams.
Thirteen new World tests, all 100 core / 75 network-library tests, formatting and
denied-warning workspace/all-target Clippy pass on unchanged final source.
[Implemented behavior, ordering and remaining gates](../PLAYER_LIFE_AUTHORITY.md). Source/proof:
`build/player-life-authority-qualified-v1-20261031`. Private wire/client/Bevy and
normal executable/bundle adoption remain open. Normal launching still uses the
preserved playable behavior; no new-mode multiplayer/native claim is made. Prior
checkpoints remain preserved; changes stay local.

# Active development — 2026-10-31: player health/life prerequisite qualified

P3-04 / P3-07: checked transient health, one scheduled respawn and nonzero per-actor
life generation pass eight new tests / 100 total core passes, formatting and denied-
warning workspace/all-target Clippy. Retaliation and damage application both refuse
old lives. [Contract and runtime acceptance](../PLAYER_LIFE.md). Source/proof:
`build/player-life-core-qualified-v1-20261031`. World/transport/Bevy adoption remains
open; the current server still reports configured maximum player health. No new
full-workspace/native result is claimed. Prior playable checkpoints are preserved;
all changes remain local.

# Active development — 2026-10-31: creature retaliation policy prerequisite

P3-07 / P3-04: a bounded immutable recipe and fixed-tick retaliation state machine
now pass eight new tests / 92 total core passes, formatting and denied-warning
workspace/all-target Clippy. [Contract, acceptance and limits](../CREATURE_RETALIATION.md). Source/proof:
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

# Current development checkpoint — 2026-10-26

Resumed after user commit `9497a18`. The input-agreement diagnostic gate now passes
236 full workspace tests (ten private-fixture skips), denied-warning Clippy/fmt,
actual saved/generated impaired UDP and an uncontended 600-second eight-peer soak.
Both simulation and I/O record zero missed deadlines in the uncontended run;
identity return, bounded terrain residency, diagnostic counts and cleanup pass.
[Input evidence](../PREDICTION_INPUT_DIAGNOSTICS.md), [soak evidence](../SAVED_TERRAIN_SOAK.md).
Severe-link errors up to 1.6085 m remain documented; stopped convergence is not a
smoothness guarantee. Native Windows and MMO population capacity remain open.

P1-11's bounded ability-data prerequisite is now qualified: immutable validated
JSON, mobile-casting defaults, off-GCD options and ceiling simulation-tick timing.
All 16 core tests, workspace/all-target Clippy/fmt and actual offline validation at
20/30/60 Hz pass. [Ability data](../ABILITY_DATA.md). These checks follow the movement
checkpoint; no combined 241-test result is claimed. Live targeting/casts/damage
and the rest of P1-11/P2-11–14 remain open. All new changes stay local.

## Previous checkpoint

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

User-directed current slice, 2026-10-22: [continental graybox](../GRAYBOX_WORLD.md),
P1-03/09/10/12 and P2-06/07/08/09. Actual prerequisites are checked global/local
coordinates, secure authority, the Avian controller and the existing POLYGON
conversion. Acceptance: 16 km extent, seams, bounded asynchronous visual/collision
residency, moving frames, full crossing/return, teleport/revisit, actual animation,
culling/unloading and measured release GPU behavior. Forest fidelity work is frozen.
This slice does not complete the source tasks or qualify MMO population capacity.

The bounded Linux terrain and prediction slices now pass; see
[TASK_STATUS](TASK_STATUS.md) and the [prediction evidence](../PREDICTION_CONTRACT.md).
The user requested stopping after completion. Do not start another slice as part
of the repository alignment/commit. Broader task acceptance remains open.

# Foundation execution order

Adopted working order, 2026-10-19. The user currently prioritizes engineering
foundations over playability. Use this document to select work; use
[TASK_STATUS](TASK_STATUS.md) to record delivery. The 112 imported entries and
historical experiments remain intact. No entire phase is complete.

The source register chains many tasks to the preceding number. Those chains are
preliminary: for example, instrumentation does not require combat schemas, and
network session validation does not require terrain streaming. The table below
states actual prerequisites for bounded implementation slices. A delivered slice
closes its own acceptance checks, not every task ID it touches.

| Order | Slice / source IDs | Actual prerequisites | Acceptance before proceeding |
|---|---|---|---|
| 1 | Real application Renet bridge: P1-06/07/08, P2-02/03 | Existing fixed-step authority, bounded wire/intents, secure UDP proof and working Bevy bridge | Real server and two Bevy clients on Renet; world matching, authoritative poses/acks, distinct IDs, departure/fresh-ticket reconnect; invalid intent/ticket/world rejection. Keep Quinn default until row 2 passes. |
| 2 | Renet qualification/default switch: P1-06/07/12, P2-02/03 | Row 1 | Repeatable controlled loss, delay, duplication/reorder, outages and reconnect; measured bandwidth/RTT/queue bounds; stale commands cannot move a replacement session; rerun map regressions and launcher cleanup. Record exact tested profiles and limits before default switch. |
| 3 | Live coordinate and identity contracts: P1-02/03/04/08 | Existing checked coordinate types and admitted-session contract | Checked global truth through authority/wire/client, bounded local frames, explicit entity/session lifetime and region/chunk identity. Positive/negative large coordinates, local rebases, chunk seams, actor removal/reconnect and stale-session traffic tests; no renderer dependencies in server. |
| 4 | Scheduler and command qualification: P1-05/06/12 | Existing configurable scheduler and rows 1–3 contracts | Loaded 20/30/60 Hz runs with explicit actor/traffic counts, tick percentiles/lateness/deadlines, bounded queues, input expiry and overload recovery. Separate CPU simulation, wall-clock scheduling and rendered network evidence. |
| 5 | Content and general controller: P1-09/10, P2-06/07/08 | Coordinate/chunk ownership from row 3; pinned Avian already active | Versioned stable content/collision identity; bounded chunk load/unload proof; shared kinematic movement for walls, headroom, steps, slopes, stacked floors, falling/jumping and nonblocking actors. Retain authored house routes as regression fixtures. |
| 6 | Prediction and presentation: P2-04/05/09/10 | Qualified input/ack contract and shared movement from row 5 | Bounded history/replay, corrections under row 2 network profiles, camera and animation follow corrected motion; native Linux evidence and separately qualified Windows support. |
| 7 | Ability schemas and first authoritative gameplay: P1-11, P2-11–14 | Identity/lifecycle, commands, controller and corrected presentation | Data validation and server target/range/cooldown/health authority; mobile casting rules; death/respawn and UI/VFX/audio derived from server events. |
| 8 | Persistent zone and later phases | Relevant foundation gates above | Refine actual dependencies and concrete checks from Phase 3 onward before implementing. External identity provider and durable economy require their own design/validation; no premature distributed-service scaffold. |

Rows 3 and 4 may overlap once their specific interfaces are stable. Content
investigation may also overlap; crossing a gate still requires executed evidence.
Early navigation contracts belong with content ownership; full tiled navigation
remains P4-03. Existing camera/art can be reused throughout. Neither gameplay nor
more manually opened buildings is the next acceptance gate.

## Current slice

Row 1 supplies the executable path through `tools/play.py --transport renet`.
It retains existing f32 fixture poses and wire v5, eight-peer capacity and local
snapshots. Local tickets are a development issuer, not external account login.
See [Renet bridge](../NETWORKING_RENET.md) for behavior, evidence and limitations.
Row 2's bounded local replacement gate now passes: deterministic impaired UDP
profiles, measured RTT/traffic/queue bounds, outages and captured old-session
packet replay. Renet is the default for the launcher and multiplayer qualifier;
Quinn remains an explicit regression path. See [qualification](../RENET_QUALIFICATION.md).
This closes those local slice gates, not full production transport/session scope.
Row 3 now has a bounded [runtime coordinate/identity slice](../COORDINATES_IDENTITY.md):
checked global authority, epoch/origin wire v6, live relative-wire rebases and
fixed-anchor presentation at distant origins. The [client rebase continuation](../FLOATING_ORIGIN.md) now recentres render and
static Avian worlds together. Server query scenes remain in the fixed content
frame; general multi-chunk ownership is open. Refine those remaining interfaces
before claiming the entire row complete; row 4 scheduler
qualification can proceed where these existing interfaces suffice.

Row 4's bounded scheduler gate now has a real eight-peer graybox qualification at
20/30/60 Hz, with explicit burst traffic, bounded timing histograms, expiry,
controlled authority stall/recovery and input-budget removal. See
[scheduler qualification](../SCHEDULER_QUALIFICATION.md). Larger populations,
full-map/crowd/render/soak baselines and action schemas remain open. Continue with
row 5's shared content/collision and general controller contracts; preserve the
existing map fixtures and regressions.

Row 5's first prerequisite now has [versioned bounded content packages](../CONTENT_PACKAGES.md):
stable logical IDs/revisions, independent artifact binding, headless collision
verification and normal map adoption. Continue with chunk visual/collision
ownership and bounded residency/load/unload before general movement. This is a
partial row, not completed chunk streaming or controller acceptance.

An isolated [chunk-residency precursor](../CHUNK_RESIDENCY.md) now proves bounded
synchronous package/collision loading, unload effects, protected owners and signed/
distant seams on original geometry. Visual descriptors remain unconsumed; add a
bounded standard-Bevy visual/publication proof before normal tile adoption. Do
not count this as asynchronous streaming or completed P2-06/general controller.

The [actual-client original-fixture diagnostic](../CHUNK_PRESENTATION.md) now passes
standard Bevy GLB readiness/retention and publication with live Avian, including
rendered replacement, failed-asset rollback and unload. This closes the bounded
visual-consumer proof; normal tile/authority adoption and moving-actor activation
are still open. Continue row 5 with shared general kinematic movement so live
chunk activation can be qualified against its real collision/grounding needs.

A [shared capsule controller](../KINEMATIC_CONTROLLER.md) now drives graybox
authority and offline movement, with actual Avian fixture coverage for walls,
headroom, steps/slopes, stacked floors, falling/jumping and nonblocking actors.
Wire v7 carries a held jump level. The imported map still uses its legacy walker;
continue row 5 by migrating its collision/controller contract and qualifying live
actor/chunk activation before prediction. This bounded controller slice does not
close row 5, P1-09 or P2-07.

Normal imported-map authority now also uses the shared controller. Trusted cached
spawns, finite-world-only collision barriers and observable per-actor fall/invalid-
state recovery replace the old highest-surface walker in ongoing simulation.
Individual chunk scenes remain open at their seams. Continue row 5 with explicit
spawn/general collision authoring contracts and live moving-actor chunk activation;
normal streaming and prediction remain open. See [movement contract](../KINEMATIC_CONTROLLER.md).

Safe-stop checkpoint: [schema-3 collision and explicit spawns](../GENERAL_COLLISION.md)
are implemented with authoring/migration tooling and staged revision-2 maps.
The user requested stopping before native qualification/default adoption. Resume
with actual Bevy mesh/map/chunk regression checks; retain the existing schema-2
launcher default until those pass. Then continue moving-actor chunk activation.

The schema-3 native/default gate now passes (2026-10-20): original mesh and map
motion/camera, signed distant frames, spawn-free chunk lifecycle/rollback and
actual launcher default package/cleanup. Default town is revision 2 with explicit
spawns. Continue row 5 with moving-actor chunk activation, then normal residency
adoption; prediction follows its acceptance gate. [Scope](../GENERAL_COLLISION.md).

## Moving-actor authority activation continuation — 2026-10-20

[Bounded authority activation](../CHUNK_ACTIVATION.md) now connects residency to
actual fixed-step actors and the real secure Renet loop. Publication revalidates
current footprints/support/clearance and epoch/generation, retaining active state
on failure. One owner-side transaction prepares collision off-tick; unknown
neighbors hold horizontal motion while preserving jump/gravity. Original Avian
fixtures and actual two-client secure UDP checks pass. Normal development map
selection remains monolithic. Continue row 5 with paired Bevy/authority residency
and actual tile ownership/partitioning; prediction follows that acceptance gate.

## Shared normal initial chunk startup — 2026-10-20

[The shared startup descriptor](../CHUNK_STARTUP.md) now connects normal executable
server/client startup, trusted global spawns, initial standard GLB scenes, live
Avian/camera and actual encrypted sessions. Client and server query frames may
differ without changing physical content; the initial selection is bound in the
version-2 session fingerprint to reject mismatched initial-only clients. Normal
loading remains initial-only and the launcher remains the monolithic town. Next:
paired dynamic readiness/publication, then actual tile geometry ownership and
partitioning with measured budgets. Do not infer complete streaming from these
bounded original fixtures or move to prediction before the row-5 adoption gate.

## Per-client chunk readiness — 2026-10-20

Row 5 now has [initial per-client readiness](../CHUNK_READINESS.md) on wire/ALPN 8.
The actual normal Bevy client waits for assets and live Avian before reporting;
authority gates each actor's horizontal movement independently. Real UDP tests
cover server replacement, delayed client release and replay/malformed reports;
the peer in that replacement test is headless. Native qualification covers normal
initial scenes, distant/query frames, invalid assets and monolithic regressions.
Continue with acknowledged withdrawal and retained presentation, then paired
dynamic publication and explicit geometry contributors/tile partitioning. Normal
rendered chunk loading remains initial-only; no row or whole task closes here.

## Acknowledged readiness continuation — 2026-10-20

P1-06/P1-08/P2-06/P2-07 now have exact epoch/sequence/set acknowledgements emitted
after a fixed simulation step. Authority coalesces one pending reply per peer;
the client confirms only the latest report with a post-application snapshot.
Real UDP withdrawal/regrant tests preserve independent actor progress and input
acknowledgements. The normal Bevy capture/verifier consumes the same barrier.
This supplies the authority half of retirement, not actual rendered retirement.
Continue the [paired publication sequence](../CHUNK_READINESS.md), protecting
local/remote interpolation, recovery spawns and gravity-driven boundary motion
before unloading. Native scope and checks are in [validation](../VALIDATION.md).

## Continuous loading boundaries — 2026-10-20

P1-09/P2-06/P2-07 now constrain capsule sweeps to bounded authority/client
coverage, including gravity projections, step-up attempts, missing corners and
disconnected gaps. Capsule-first support uses sole validation/fallback for
imprecise convex contacts while retaining exact persistent wall contacts.
[Contract](../CHUNK_BOUNDARIES.md). Native edge pressure and existing house/map
regressions remain required evidence, not a streaming-capacity claim.

Continue row 5 with actual client retention of local poses, remote interpolation
endpoints and recovery spawns. Pair that coverage with the acknowledgement
barrier, handle newly arriving remote poses while assets lag, then connect
bounded standard-Bevy/Avian publication to authority offers. Actual tile
contributors/partitioning and wider budgets remain before normal adoption.

## Paired normal-client handover — 2026-10-20

P1-06/P1-08/P1-09/P2-06/P2-07/P2-08 now have an opt-in normal executable path:
authority lookahead and reliable generation offers, one hidden standard-Bevy
candidate, exact retirement acknowledgement, retained local/remote/recovery
coverage, matching Avian replacement and live generation readiness. Original
three-chunk fixtures exercise delayed preparation, invalid replacement rollback
and independent distant query frames. [Contract](../CHUNK_HANDOVER.md).
Current executed qualification, including the sloped-wall contact regression
found by the wider suite, is recorded in [status](../STATUS.md) and
[validation](../VALIDATION.md); implemented behavior alone is not acceptance.

Continue row 5 by specifying geometry contributors before importing actual tile
streaming. Schema-1 package geometry may extend beyond its owner cell, so owner
lookahead alone cannot prove collision completeness. Acceptance must cover a
building/mesh spanning cell edges, arrival from either side and a diagonal corner,
matching visible/collision contributors, candidate rejection, delayed peers and
retirement safety. Bind contributor metadata into dataset identity and validate
it against actual payloads; retain the qualified monolithic map until real tile
partitioning and memory/load budgets pass. Native reversal/superseding offers and
asymmetric failed assets remain explicit handover checks, not implied by the
original three cases. Prediction remains the next major foundation after the
relevant row-5 gates; no entire task or phase is closed here.

## Geometry contributor continuation — 2026-10-20

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08 now implement schema-2 bound package extents,
complete-cell coverage/demand/readiness and actual collider/Bevy hierarchy extent
checks. Original nonlocal-wall fixtures cover straight/diagonal entry, independent
frames, protected presentation/recovery and invalid extent retention. Current
executed checks are recorded separately in [status](../STATUS.md).
[Contract](../CHUNK_CONTRIBUTORS.md). The town remains monolithic.

The actual town inventory is 704.1 MB, including 698.7 MB of embedded images.
Before real partitioning, bind shared image resources and validate immutable
relative paths/hashes for standard Bevy loading. Then split actual terrain and
matching collision at cell edges, retain model transforms/contributors, and
measure resident/staging/load budgets. Keep whole task/phase and production
streaming claims open; refine the four-package proof budget only against actual
content requirements rather than duplicating the town's image payload per chunk.

## Shared image continuation — 2026-10-20

P1-10/P2-06/P2-08 implement manifest-v2 shared PNG bindings with canonical
colour-space/sampler variants and normal Bevy loading. The offline extractor
preserves source geometry and PNG bytes, removes image payloads from new GLBs
and publishes a new immutable representation. The actual town stage is a
5.48 MB GLB plus 488 shared images; it remains monolithic geometry and is not
promoted. [Contract](../SHARED_IMAGES.md). Current executed checks remain separate
in [status](../STATUS.md) and [validation](../VALIDATION.md).

Continue row 5 by pruning each package's real node/mesh/material/texture graph
and partitioning actual terrain and matching collision at cell edges. Preserve
model transforms, UVs and contributor completeness. Measure resident/staging
bytes, required provider counts and loading behavior before changing the default
map or proof budgets. Keep reversal/supersession/asymmetric-failure checks open;
prediction remains after the relevant content/controller gates rather than a
claim that this packaging slice completes a phase.


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


## Actual-content budget analysis — 2026-10-20

P1-03/P1-10/P2-06/P2-07/P2-08 now have verified package/image/geometry cost
analysis and conservative complete-footprint/recovery/lookahead unions. The
finite town reaches 18 providers for one capsule corner, 22 for two actors and
26 for the bounded actor set. All-indexed content has 4,001 colliders and an
estimated 1.51 GB base RGBA8 image payload; actual residency/loading is unmeasured.
[Evidence and concrete runtime acceptance](../CHUNK_BUDGETS.md).

Continue by implementing and qualifying a 32-package hard ceiling with a
26-package town session, checked control-message addresses and size bounds,
budget-bound admission, explicit world-edge policy and real multi-root staging/
retirement. This is a proposal, not a runtime cap change. Preserve the small
fixtures, actual map regressions and 30 Hz default. Then qualify real town
loading/resources and remaining handover lifecycle cases before promotion.


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

## Synty asset branch: model identity/source completeness — 2026-10-20

P1-10/P2-08: checked source-derived IDs now match 1,109 distinct Elven asset/fileID
references, including a legacy explicit metadata table. However, the package has
32 Git LFS pointer payloads, and the demo closure reaches 26 missing FBXs plus
one missing PNG. Eighteen other override-target IDs remain unmapped. Resolve
these and qualify actual hierarchy/material conversion before claiming full scene
import or adopting this map. [Evidence and acceptance](../SYNTY_ELVEN_IMPORT.md).
This asset-specific source gap does not block row 5's core finite-world, actual-
town client/resource and handover-lifecycle work. No task/phase is complete.

## Explicit finite-world bounds — 2026-10-20

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08 add schema-2 chunk-session rectangles,
admission identity v4, conservative local planes in shared movement and clamped
lookahead. Legacy session identities and open internal seams remain. Real Avian
edge/corner/jump/retreat and finite-world fall recovery qualify at 20/30/60 Hz;
source-town bounds are retained in a separate opt-in headless stage.
[Contract and scoped evidence](../WORLD_EDGES.md).

Continue row 5 with actual-town client preparation/staging memory and latency,
then reversal/supersession/asymmetric-failure lifecycle checks and map regressions
before promotion. Finite-world original-fixture tests do not establish the full
town perimeter's traversability, rendered resource budgets or MMO capacity. Keep
the monolithic launcher at 30 Hz. No complete task/phase or Windows claim.

## Actual-town startup/resource checkpoint — 2026-10-20

P1-10/P1-12/P2-06/P2-07/P2-08 now qualify ten-package and all-26-package
normal two-client startup on RX 7900 XTX/RADV Vulkan. A measured failure corrected
offline authoring to use transformed local mesh AABBs, matching unchanged runtime
validation; package/image bytes remain untouched in a new index stage.
Per-client RSS/DRM VRAM and approximate ready-marker times are sampled, with
scope/limitations in [the resource record](../TOWN_CLIENT_RESOURCES.md).

Continue row 5 with actual live candidate preparation/staging/publication,
retained shared-image/resource lifetimes and cleanup, then remaining reversal/
supersession/asymmetric-failure checks and map regressions before promotion.
All-index startup does not establish ten-to-26 live client replacement or
maximum staging memory. Keep the monolithic launcher and 30 Hz default.

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

## Interactive performance gate — 2026-10-22

P1-12 / P2-05 / P2-08 / P2-10. Require representative optimized, native frame-time
measurements before scene adoption. Correctness runs with validation, functional
screenshots and asset counts do not establish acceptable performance. Preserve
legacy camera regressions and use the existing accelerated CAMERA query geometry
for normal chunk worlds. Keep explicit debug/visual-refinement modes for diagnosis.
Current measured scope and pending checks are in [the performance record](../FOREST_PERFORMANCE.md).
