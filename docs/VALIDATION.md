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

# Resumed diagnostics and ability-data prerequisite — 2026-10-26

Movement diagnostics pass 236 workspace tests (ten private-fixture skips),
denied-warning Clippy/fmt and actual saved/generated impaired UDP. The isolated
600-second eight-peer soak passes identity/retirement/counter/cleanup checks with
zero simulation and I/O missed deadlines. [Exact evidence](PREDICTION_INPUT_DIAGNOSTICS.md).
The subsequent P1-11 data-only addition passes all 16 core tests (including five
new contract tests), workspace/all-target Clippy/fmt and actual offline CLI
acceptance at 20/30/60 Hz plus rejection exits. [Scope and logs](ABILITY_DATA.md).
This is not a combined 241-test workspace result or live combat qualification.
Earlier stop/failure records below remain historical. Work stays local.

# Diagnostic safe stop — 2026-10-26

After the completed 235-test terrain/poor-link checkpoint below, bounded input
agreement diagnostics pass 11 focused history tests and workspace/all-target
denied-warning Clippy. The user requested stopping during the new full-suite
relink; that invocation was interrupted and is not a pass
(`build/prediction-input-workspace-v1.log`). New diagnostic real-UDP/native evidence
remains unqualified. [Exact status](PREDICTION_CONTRACT.md). No soak is running;
changes remain local and uncommitted.

# Resumed terrain/network qualification — 2026-10-26

The actual eight-peer 600-second saved-terrain soak passes return-identity and
bounded-residency checks, with zero missed simulation deadlines. Separate I/O pump
misses remain recorded. [Evidence](SAVED_TERRAIN_SOAK.md). The real encrypted UDP
poor-link test passes movement at 20/30 Hz and unsupported-latency suspension at
60 Hz. [Envelope and limits](PREDICTION_LATENCY.md). The fresh workspace suite
passes 235 tests, ten private-fixture skips; denied-warning all-target Clippy/fmt,
221 asset Python tests and four GLB alignment tests pass. This checkpoint precedes
the next input-agreement diagnostic extension; do not attribute unrun checks to it.

# Eight-peer saved-terrain checkpoint and safe stop — 2026-10-23

The actual encrypted eight-peer headless 180-second soak passes with no missed
server deadlines or movement recoveries, bounded collision residency and measured
retirement/relevance pruning. Sampled combined RSS peaks at 166.33 MiB.
[Exact evidence and limits](SAVED_TERRAIN_SOAK.md). The latest example-only
return-identity/trace additions build and pass workspace/all-target denied-warning
Clippy, but have not been runtime qualified. The full 234-test result below
predates this soak example and server telemetry. No longer soak was started;
work is stopped at the user’s request. Changes remain local and uncommitted.

# Movement-clock qualification — 2026-10-23

Saved terrain and the smoother prediction clock pass the full **234-test Rust**
workspace suite (ten private-fixture skips), denied-warning Clippy/fmt, 221 asset
Python tests and four alignment tests. Normal-speed native impaired-UDP steady
held frames improve from about 9% to 0% / 0.152%; ordinary VSync records 0.463%
at 60 FPS. A 1% scoped gate rejects the old baseline and accepts the final runs.
The full saved 16 km crossing still passes at 823.40 FPS uncapped on RX 7900 XTX,
with the established residency/retirement bounds. Rare packet-loss corrections
remain. [Exact measurements, hashes, scope and limits](PREDICTION_CONTRACT.md).

# Saved-terrain qualification — 2026-10-23

The bounded saved-height terrain slice passes 228 Rust tests (10 explicit
private-fixture skips), denied-warning Clippy/fmt, 221 asset-tool tests and four
GLB alignment tests. Native Linux full-continent crossing, two-client impaired
UDP/teleport/held-jump, corrupt-file shutdown, default launcher cleanup, retained
town and generated-source checks pass. [Exact evidence](SAVED_TERRAIN.md).
This is warm-cache graybox qualification, not production world/population/Windows
capacity. Earlier checkpoints below identify earlier binaries and behavior.

# Current qualification — 2026-10-22

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

The default graybox prediction slice passes full native crossing, two-client
impaired UDP/teleport/held-jump checks, retained town regression and final client
frame-guard/native launcher checks. Exact workload scope, binary hashes and
limits: [prediction evidence](PREDICTION_CONTRACT.md). Earlier evidence below
records earlier binaries and features; it is not a claim that every historical
private-fixture check was repeated in the final suite.

# Bootstrap qualification — 2026-10-16

This first section records the original offline Bevy bootstrap. The subsequent
multiplayer qualification is recorded below; neither is an MMO capacity claim.

## Executed checks

- Bevy 0.20.0, locked dependencies, Rust 1.99.0, Rust 2024, Linux debug build.
- `cargo fmt --check`, denied-warning workspace/all-target Clippy and build pass.
- Three CPU movement tests pass: normalized diagonal input, equivalent distance
  at 30/60 fixed ticks per second, and ground/boundary/idle invariants.
- Native Wayland in an owned private Weston headless/GL compositor on
  AMD Radeon RX 7900 XTX, RADV NAVI31, Vulkan device ID `0x744c`.
- `VK_LAYER_KHRONOS_validation` activation is recorded by Vulkan loader trace;
  `WGPU_VALIDATION=1`, `WGPU_DEBUG=1`. No Vulkan VUID/validation errors, renderer
  errors or panics occur in the successful final run. A benign window-decoration
  warning about an unknown button type remains.
- Scripted smoke input records `(0, 0.9, 0)` at frame 59, `(0, 0.9, -5)` at 119,
  and `(5, 0.9, -5)` at 179. The fixed-step movement checkpoints match within
  0.001 world units. The real rendered screenshot is requested at frame 239.
- The screenshot is 1280x720, contains 2561 distinct RGB colours and 477614 green
  pixels, and was visually inspected. The scene, blue capsule, road, shadow and
  control instructions are present. The image check establishes rendered content,
  not a pixel-exact geometry/lighting oracle or GPU transform readback.

Ignored local evidence lives in `build/qualification/bootstrap/`: `tests-020.log`,
`clippy-final.log`, `build-final.log` and `native-final/{run.log,report.json,scene.png,
verification.json,qualification.json}`. The final qualification records executable
and Cargo.lock hashes, device and compositor information. No generated binaries,
screenshots, dependency SDKs or private conversation files are committed.

## Dependency/platform observations

The initial 0.19.1 probe rendered and passed the movement/image checks, but emitted
undefined-present-layout and already-signaled-acquire-semaphore Vulkan diagnostics.
That run remains rejected under `native/`, with its original lock retained locally.
The fresh 0.20.0 runs pass the same target/device checks with ordinary Bevy rendering;
no custom Vulkan initialization patch or copied benchmark renderer was added.

This machine's Wayland/xkbcommon development packages were available as an
extracted third-party SDK. It was copied into ignored `build/native-sdk/` and used
only through `PKG_CONFIG_PATH`/`LIBRARY_PATH` during compilation. Normal installations
use the OS packages listed in the README. No old engine source or path dependency
is needed to compile or run the application. The local private-display host harness
was reused for qualification; it is not a game dependency.

Linux runtime is qualified on this GPU/compositor only. Linux/Windows CI checks
are configured but have not run on GitHub because no push was authorized. Native
Windows graphics, resize/minimize/recovery, manual keyboard interaction and other
GPUs remain unqualified. This is not a performance benchmark or capacity claim.

## Reproduce

After installing the README's native development packages:

```sh
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation \
WGPU_BACKEND=vulkan WGPU_VALIDATION=1 WGPU_DEBUG=1 \
cargo run --locked -p theia-client -- --smoke-test build/smoke
python3 tools/verify_smoke.py build/smoke
```

The run requires a graphical display or a separately prepared native Wayland
headless compositor. The image verifier needs Python 3 and Pillow. Runtime logs
must also be checked; an image/movement verification alone does not establish a
clean Vulkan validation run. The smoke mode scripts local intents and waits for
a real screenshot callback before success. It never starts or simulates a server.

## Local multiplayer qualification — 2026-10-16

Implemented and executed one authoritative 30 Hz Rust server and two independently
launched Bevy clients using Quinn QUIC, verified localhost certificate trust and
real input/snapshot datagrams. No mocked peers or synthetic proxy counts are used.

- Formatting, denied-warning workspace/all-target Clippy, workspace build and
  eleven tests pass: the three original offline movement tests, five server/wire
  unit tests and three actual loopback transport tests.
- Tests cover normalized movement, fixed server timestep, stale input rejection,
  movement expiry, bounds, player overlap/pass-through, capacity/ID lifecycle,
  bounded/versioned messages, two clients seeing movement, disconnect/reconnect,
  rejection of an untrusted certificate and closure on malformed input. A separate
  transport test reorders/replays movement and checks that it neither reverses
  movement nor refreshes the stale intent's timeout.
- Two independent 1280x720 Bevy windows run in the private native Wayland/Weston
  headless GL display on RX 7900 XTX / RADV NAVI31, Vulkan. Both receive distinct
  server identities and report matching final server poses. Local rendered poses
  match their authoritative snapshots, and interpolated remotes settle to the
  same positions. Actual PNGs contain both local blue and remote red players;
  screenshots were visually inspected as well as checked for coloured pixels.
- `VK_LAYER_KHRONOS_validation` activation is proved in **both** client logs by
  loader insertion messages. No VUIDs, validation errors, renderer errors or panics
  appear. The existing benign window-decoration warning remains.
- The original offline smoke mode is rerun with the connected implementation:
  all three movement checkpoints and the real screenshot still pass, with active
  Vulkan validation and no validation errors.

Ignored evidence: `build/qualification/multiplayer/{tests.log,clippy.log,
build-final.log,native-final.log}`, and `native-final/` containing server log,
per-client `a/` and `b/` reports/logs/PNGs, `verification.json` with executable,
lockfile and image hashes, and `offline/` regression evidence. The host compositor
harness is the same independent qualification tool described above. It is not a
runtime/build dependency of the game. Public ephemeral certificates are kept only
in ignored evidence; the private signing identity never leaves server memory.

This qualifies local two-player walking only. Native Windows, manual multiplayer
keyboard/camera interaction, adverse network loss/jitter, Internet exposure,
automatic session resume, local prediction/reconciliation and MMO scale remain
unqualified. NPCs do not exist yet: the player/NPC nonblocking rule is recorded
for future implementation, while the player/player case has an executed test.
World collision, imported character animation, targeting, combat and quests remain
future work; no interactive or moving environment is planned.

To reproduce the two-client GPU check after building on a prepared display:

```sh
VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation VK_LOADER_DEBUG=layer \
WGPU_BACKEND=vulkan WGPU_VALIDATION=1 WGPU_DEBUG=1 \
python3 tools/verify_multiplayer.py build/multiplayer
```

The tool requires Python 3/Pillow, launches/cleans up its own server and clients,
and checks diagnostic logs as well as received state and real screenshots. The
reusable smoke is not a performance or latency benchmark.

## Static-world and camera qualification — 2026-10-16

The renderer and server now use one immutable graybox box layout. The server owns
collision against the solid building, fixed door and rocks; players remain
nonblocking. This is conservative flat-ground clearance with X/Z wall sliding,
not general terrain physics. The local camera supports right-drag orbit, wheel
zoom, Home reset, camera-relative movement and static-box obstruction.

Eighteen workspace tests pass (six client, eight shared/core, four real QUIC).
New checks cover obstacle sweeps in both directions, sliding/retreat, clear roads,
camera obstruction/recovery, angle/zoom limits, rotated movement including
diagonals, cursor release on focus loss and close-body hiding/restoration. The new
transport test keeps submitting fresh movement into the door; snapshots remain
at contact while acknowledged input sequences advance.

Formatting, denied-warning all-target Clippy and workspace build pass. Native
Wayland/Weston headless GL on the same RX 7900 XTX / RADV device passes:

- The real server/two-client rendered smoke, with matching authoritative poses.
- The original offline movement smoke with unchanged expected checkpoints.
- The new `--world-smoke`: pawn reaches approximately `(-7,0.9,-4.54)` at the
  fixed door, camera retracts to approximately 0.232 units when pointed into the
  building, then restores to 10 units after orbiting away. A real 1280x720 PNG is
  saved and visually inspected; it shows the pawn outside the fixed door.

All four client processes prove Vulkan validation activation through loader
insertion logs, with no validation errors, VUIDs, renderer errors or panics. The
existing benign window-decoration warning remains. Ignored local evidence lives
under `build/qualification/static-world/`: test/lint/build logs and native reports,
PNGs, loader logs and artifact hashes. No dependency or hardware changes were
needed. The private host compositor harness remains a qualification tool only.

The camera's native test scripts angles/zoom and records CPU transforms. OS cursor
capture and physical mouse/keyboard operation are covered only by resource/system
tests, not manual native interaction. Native Windows, imported terrain/collision,
slopes/steps/jumps, jitter/loss and local prediction remain unqualified. No NPCs
exist yet; NPC nonblocking remains a requirement. Static environment items have no
opening, movement or destruction.

## Geometry-only GLB inspection — 2026-10-16

The user's 9884-byte `duskwoodhouse_medium.glb` was inspected read-only in Downloads,
then moved into the client's `assets/reference/` after actual rendering succeeded.
Original bytes/hash are recorded in that folder's README. The ZIP left in Downloads
contains the same GLB only; no loose BLP files were available for conversion.

Khronos glTF Validator `2.0.0-dev.3.10` reports zero errors/warnings: one mesh,
425 vertices, 672 triangles, no UVs/materials/images/animation/skin. Bevy's standard
loader instantiates it and generates flat normals, yielding 2016 render vertices.
Instantiated bounds match the source bounds; size is about 18.433 x 16.776 x 25.009
source units. No geometry, material assignments or unit scale were altered.
The inspector adjusts its camera/floor and supplies a separate 1.8-unit capsule.

The first native attempt was rejected: the minimal Bevy feature configuration did
not register reflected scene types, causing a scene-instantiation panic and abnormal
shutdown. Enabling Bevy's standard `reflect_auto_register` feature fixes that
configuration; no custom scene clone or manual per-type workaround was introduced.
Rejected logs remain under `build/qualification/gltf/native-rejected-reflection/`.
The subsequent actual GPU run loads/renders successfully with Vulkan validation
active and no validation errors. The plain-shaded house screenshot was inspected.
Reinhard tone mapping in the inspector keeps the untextured white material readable.

Twenty workspace tests pass, including GLB header preflight and framing/zooming a
large asset without snapping to the game's camera limit. Workspace build, formatting
and denied-warning all-target Clippy pass. Successful native evidence uses the same
RX 7900 XTX / RADV and owned private Wayland/Weston headless GL compositor. The
existing two-client, offline movement and world/camera smokes are also rerun after
enabling glTF/reflection support. A malformed GLB must exit unsuccessfully without
producing a successful report/screenshot.

Ignored evidence is under `build/qualification/gltf/`: validator report, build/test/
Clippy logs, rejected and passing native runs, per-client logs/reports/PNGs and the
final asset screenshot. The validator's pinned npm installation is a local ignored
qualification dependency, not part of the game runtime. The game remains independent
of the old engine; the existing external compositor harness is only a test host.

Only geometry import of this file is qualified. PNG/JPEG decoding is enabled but
textured import, BLP conversion, UV/material preservation, character animation,
custom WoW shaders, imported-world collision and native Windows remain unqualified.
Physical camera input is still not manually qualified; the asset run uses auto-fit.

## Textured building and character — 2026-10-16

The [two-model report](ASSET_CHECK_2026_10_16.md) records the exact fresh exports,
hashes, validation issues, image/mesh counts and stand/walk/run playback evidence.
Bevy's `gltf_animation` feature and standard animation graphs are now enabled in
the inspector. A requested clip must have a player and sampled transform changes
before a successful capture is written. Building playback is not automatic and
the authoritative game remains separate.

Twenty workspace tests, formatting, denied-warning all-target Clippy and build
pass. Four native checks render the textured building and the three character
clips on RX 7900 XTX / RADV, with active Vulkan validation and no validation errors.
Textures and changing skinned poses are visible in the PNGs; all source files are
preserved unchanged. These results supersede the previous geometry-only limit for
these particular new exports, but do not validate all character animations or
complete WoW asset compatibility. The character's strict glTF errors remain real;
successful Bevy rendering is not a substitute for a valid portable source export.

## RustyWolfPup — 2026-10-16

Four further native inspector runs qualify stand, walk, run and attack playback
for the user's external `creatures/RustyWolfPup.glb`. Each saves an actual textured
skinned screenshot and reports sampled transform changes. RX 7900 XTX / RADV,
private headless Wayland and active Vulkan validation were used; no validation
errors or panics occurred. Strict glTF validation separately fails with 3,084
alignment errors. [Exact input, counts, reproduction and limitations](ASSET_CHECK_2026_10_16.md#rustywolfpup).
Evidence: ignored `build/qualification/rusty-wolf-pup/`. No runtime changes or
additional workspace test run were needed for this asset/documentation check.

## Integrated reference scene — 2026-10-16

[Reference scene qualification](REFERENCE_SCENE.md) records prepared-file hashes,
remaining human skin errors, animated local/remote players, an idle nonblocking
wolf and the static farmhouse. Twenty-three Rust tests, four Python repair tests,
formatting, denied-warning all-target Clippy and build pass. Seven native client
runs pass on RX 7900 XTX / RADV with active Vulkan validation: reference locomotion,
imported-house world/camera smoke, two connected reference clients, two connected
graybox clients and the original offline movement smoke. All save actual rendered
screenshots. No VUIDs, Vulkan validation errors or panics occur in successful runs.

The shared building envelope is enlarged for sensible reference-model scale.
Door contact moves to Z=-2.54; both the actual QUIC door test and native camera
checks use the new geometry. The wolf has no collider. Native Windows, NPC AI/
authority, combat, traversable house interiors and terrain collision remain open.

## Map terrain visual import — 2026-10-16

The [map report](MAP_CHECK_2026_10_16.md) records the supplied Ruins of Lordaeron
tile, source/derived hashes, exact texture preservation and separate native Bevy
terrain capture. All 65,504 triangles and 256 textured patches survive conversion;
two identical PNGs share one image. Khronos validation has zero errors/warnings and
516 optional-buffer-target hints. RX 7900 XTX / RADV runs with active Vulkan
validation pass for both terrain rendering and a wolf animation regression.
The large-asset inspector now scales camera depth/zoom; loading/capture remain bounded.
Twenty-four Rust tests, seven Python asset-tool tests, format, denied-warning Clippy
and build pass. Map object placements, liquids and terrain-aware authoritative
movement remain unimplemented; this establishes visual import only.

## Static map objects — 2026-10-16

The [assembly report](MAP_OBJECTS.md) records primary-source placement conventions,
846 top-level and 624 nested placements, 183 distinct model inputs, source hashes
and an assembled GLB with 253,552 triangles. One smoke emitter has no source
geometry. Khronos validation reports no errors/warnings, one empty-node info and
2,322 optional buffer-target hints. Default/focused map captures and a wolf
animation regression use RX 7900 XTX / RADV with actual Vulkan validation active
and no validation errors or panics. Shared mesh references survive assembly.

Twenty-five Rust tests, 14 Python asset-tool tests, format, denied-warning Clippy
and build pass. The focus option records camera framing separately from whole-scene
statistics. Native Windows, liquids, map collision/movement, lights/particles and
exact WoW shader reproduction remain unqualified or unimplemented as detailed in
the report. The multiplayer simulation and protocol are unchanged by this slice.

## Static map liquids and capture readiness — 2026-10-16

[Liquid qualification](MAP_LIQUIDS.md) adds 30 source-masked patches, 1,825 vertices
and 2,744 triangles using absolute liquid heights and standard translucent PBR.
The original map buffer is retained unchanged. Khronos validation reports zero
errors/warnings, the existing empty smoke-node info and 2,393 optional buffer-target
hints. Raw liquid IDs are preserved without guessing their database classification.

Early fixed-warmup captures were visually incomplete despite successful CPU scene
instantiation. The inspector now observes Bevy render-pipeline readiness and records
pending/failed/compiled counts before allowing a capture. Final native lake and
matching no-water views, plus a wolf animation regression, use RX 7900 XTX / RADV
with active Vulkan validation. Saved PNGs are inspected; pre-readiness captures
are retained as rejected evidence. Twenty-seven Rust tests, 22 Python asset-tool
tests, format, denied-warning Clippy and build pass. Water rendering remains a
static material preview; terrain movement, water gameplay and native Windows are open.

## Bounded authoritative terrain walking — 2026-10-16

[Terrain walking qualification](TERRAIN_WALKING.md) connects a small source-map
patch to the existing Rust authority and two independent Bevy clients. Six textured
parts / 576 source triangles produce a 15,299,184-byte GLB and an exact matching
triangle profile. Strict Khronos validation has zero errors/warnings/infos and
21 optional-buffer-target hints. Actual prepared visual/profile triangles compare
bit-for-bit at float32 precision. Original OBJ/MTL/PNG hashes remain unchanged.

The server determines ground height and sampled slope/boundary legality. Wire
version/ALPN 2 adds authoritative Y and a world identity, while retaining bounded
intent-only inputs. Real QUIC tests check changing height and mismatched-world
rejection. Player overlap/pass-through also passes on sloped terrain. Thirty-two
Rust tests, 24 Python asset-tool tests, formatting, denied-warning all-target Clippy
and build pass.

Final native checks launch six actual GPU clients in three independent two-client/
server runs: terrain patch, procedural graybox and the existing human/wolf/farmhouse
reference scene. All run on RX 7900 XTX / RADV with actual Khronos Vulkan validation
active, no VUIDs/validation errors/panics, compiled render pipelines and saved
1280x720 PNGs. Local/remote XYZ agrees with authority. A separate Python triangle
interpolator checks terrain ground heights, and both players change server-owned Y.
Both final terrain PNGs are inspected. Existing reference actors still enter
walking animation. Final executable hashes are recorded by each verifier.

Evidence: ignored `build/qualification/terrain-walking/`, including `world/`,
`geometry.json`, `validator.json`, test/build/lint logs and
`native-final/{terrain,graybox,reference}/`. Earlier and interrupted native runs
remain separate from these final accepted reports. The private display harness
is a host qualification tool; no old-engine code/runtime dependency is added.

This qualifies a bounded terrain-only fixture with capsule actors, sampled square
clearance and a terrain camera. General static-object mesh clearance, animated
actors on this patch, interiors, steps/jumping/falling, full-map gameplay, water
rules, scale/performance, adverse networks and native Windows remain open.

## Animated players on terrain — 2026-10-16

The bounded [terrain walking slice](TERRAIN_WALKING.md#animated-human-players) now
accepts `--player-assets`, using Bevy's named file asset source for the prepared
human independently of the terrain assets. Both local and remote visuals inherit
existing server-owned XYZ roots, camera visibility, interpolation and lifetime.
Only the human is loaded in this mode; the full reference scene still includes
the wolf and house in its original world. Wire/authority and source assets are
unchanged. The known human skin-data validation failures remain unresolved.

Thirty-four Rust tests, 24 Python asset-tool tests, formatting, denied-warning
all-target Clippy and build pass. New tests verify foot/forward-axis placement at
several ground heights, child cleanup with the player and rejection of parent
terrain/root movement as sampled animation evidence. The network smoke now records
child-local transform changes by gait, only during smoke diagnostics.

Six native GPU clients run in three independent two-client/server checks: human
terrain, capsule terrain and the existing human/wolf/house reference scene. The
human terrain run uses a directory containing only `human.glb`, walks outward,
sprints back and stops. Both actors in both clients have sampled motion in stand,
walk and run, changing server-owned heights and matching rendered/server XYZ.
Both final 1280x720 human-terrain screenshots visibly contain both players and
are inspected. All final cases use RX 7900 XTX / RADV with actual Khronos Vulkan
validation active, stable compiled pipelines and no validation errors or panics.

An initial checker run exposed a Python option variable shadowed by the snapshot
player dictionary. It produced incorrect asset metadata and a false reference
scene assertion; this is corrected before the final accepted checks. Initial
widely separated gait captures and that checker output remain under `native/`;
accepted evidence is `build/qualification/terrain-players/native-final/`, with
source/prepared hashes, pose-change counts, executable hashes, logs, reports and
screenshots recorded in the surrounding qualification directory.

This establishes animated multiplayer presentation on the small terrain fixture.
Foot IK, animation blending, static object clearance, full-map collision/gameplay,
NPC authority, combat, performance/scale and native Windows remain open.

## Authored static farmhouse on terrain — 2026-10-16

The [static terrain object fixture](TERRAIN_STATIC_OBJECTS.md) adds one explicitly
authored farmhouse/foundation and a shared collision envelope. Source origin
`(350,37,2400)` gives a shallow foundation without changing terrain geometry.
The original terrain and house binary buffers are retained exactly; the model
is uniformly fitted to the authored envelope. The 24,194,636-byte GLB contains
2,143 triangles / 30 meshes / 19 materials / 18 images. Strict Khronos validation
reports zero errors/warnings/infos and 23 optional-buffer-target hints.

The bounded terrain profile now accepts at most 64 validated static solids.
Server movement uses expanded XZ sweeps against their ground-blocking footprints;
client camera clearance uses their finite 3D boxes. Primitive helpers are shared
with the graybox, preserving its existing height filtering and behavior. No
character/NPC enters world collision. New profile/visual pairs retain the existing
world-identity checks; wire version 2 and authority rules are unchanged.

36 Rust tests, 26 Python asset-tool tests, formatting, denied-warning all-target
Clippy and build pass. New checks cover slope-aware movement plus solid stopping,
sliding/retreat, camera clearance, rejected bounds/spawns, exact clipped foundation
heights, fitted visual bounds and unchanged binary payloads. An actual QUIC test
continues submitting movement at contact and confirms stationary authoritative
poses while applied input sequences advance.

Three independent server/two-client native checks pass on RX 7900 XTX / RADV:
terrain house with animated humans, existing human terrain-only and procedural
graybox. The house run reaches local Z=-3.35, continues pressing against the wall,
returns to idle from actual zero displacement, then records a real shortened
camera/hidden body and restored camera/body before screenshot capture. Local and
remote XYZ matches server snapshots and independently interpolated terrain height.
All accepted runs have actual Khronos Vulkan validation activation, ready compiled
pipelines, no validation errors/panics and real 1280x720 screenshots. Both house
captures are inspected. No benchmark/capacity or native Windows claim follows.

The first steep-footprint placement needed a tall foundation and was replaced
with a flatter source crop. Initial checker assumptions also required nontrivial
Y change on the nearly flat contact route and an absolute initial X=0; the final
checker instead verifies exact ground height and movement relative to the actual
server-assigned spawn. The existing sloped locomotion case retains its changing-Y
requirement. Initial candidates/checker outputs remain separate from accepted
`build/qualification/terrain-house/native-proof/` evidence.

The surrounding ignored directory records source/prepared hashes, GLBs, strict
validator results, tests/build/lint and qualification JSON. The source exports
remain unchanged and outside Git. This is a conservative solid building/foundation,
without interiors, walkable roofs, opening doors, mesh physics or full-map gameplay.

## Enterable farmhouse ground floor — 2026-10-16

The [enterable house fixture](ENTERABLE_HOUSE.md) replaces the single envelope in
a separate derived world with measured wall boundaries and an open doorway. The
model scale is increased to 0.6 to give the fixed 1.8 m player about 1.998 m of
entrance clearance. A fixed brown ramp bypasses the source model's steep textured
entrance face. The door/environment never moves. Original terrain/house buffers
and source files remain unchanged; proprietary assets are not added to Git.

Eighty source ground-floor triangles plus six authored ramp/landing/foundation
support triangles determine authoritative height. Seven blocking wall/exclusion
boxes and two camera-only ceiling/lintel boxes qualify this layout. Optional
profile fields default to the prior behavior; a bounded 0.2 m seam allowance is
enabled only for this fixture. The highest-support approach is not general
multi-floor physics, and the loft/stair area stays blocked. The old solid-house
and graybox scenes retain their previous behavior.

37 Rust tests, 28 Python asset-tool tests, formatting, denied-warning all-target
Clippy and build pass. New checks cover entry, raised-floor support, wall blocking,
exit, ceiling camera restriction, step bounds, measured door/ceiling clearance,
ramp slope, the open collision gap and source/output protection. Strict Khronos
validation reports zero errors/warnings/infos and 26 optional-buffer-target hints.

Six actual GPU clients across three independent two-client/server sessions pass
on RX 7900 XTX / RADV: entry/exit, previous solid-house collision and procedural
graybox. Client A enters the real model, presses against its interior wall, walks
back outside and enters again. Snapshots record interior Y=4.70959 for the pawn
centre and outside Y=1.98166 on the tested approach. Interior wall contact is near
X=-4.91566. Independent triangle interpolation, local/remote XYZ agreement,
advancing input sequences, actual ceiling-limited camera and sampled locomotion
are verified. The final interior image visibly places the human in the room.

All accepted native runs have actual Khronos Vulkan validation activation, ready
compiled pipelines, real 1280x720 screenshots and no validation errors/panics.
Ignored `build/qualification/house-interior/` records source hashes, measured
layout, prepared GLBs, strict validation, tests/build/lint, native logs/reports/
PNGs and aggregate qualification JSON. No general mesh collision, full-map,
performance/scale, NPC gameplay or native Windows qualification is claimed.

## Full supplied development tile — 2026-10-16

The [development-world report](DEVELOPMENT_WORLD.md) records complete supplied
terrain/objects/liquid presentation, authoritative terrain movement across the
actual tile bounds, conservative imported object envelopes, water exclusions and
normal launcher lifecycle. All 65,504 rendered terrain triangles match authority
at float32 precision. Embedded visual binary data is unchanged. The derived GLB
has zero glTF errors/warnings and the existing empty-emitter info.

Formatting, denied-warning all-target Clippy, workspace build, 40 Rust tests and
31 Python tests pass. Three native two-client/server sessions (full map, authored
interior regression and graybox) pass on RX 7900 XTX / RADV with active Khronos
Vulkan validation and no validation errors/panics. Final screenshots are inspected;
full-map players travel beyond the old patch, match source ground/server/rendered
XYZ and show sampled stand/walk/run. A normal launcher run separately proves map
readiness and clean server/client shutdown. Evidence lives under ignored
`build/qualification/development-world/`, including final accepted `native-final/`
and `launcher/` reports. The private display harness remains host tooling.

Wire/ALPN version 3 expands bounded snapshot coordinates; schema-2 profiles provide
actual map bounds and bounded spatial lookup. Schema-1 fixtures retain their prior
movement semantics. The full map's buildings are solid envelopes, water is blocked,
and exact foliage camera clearance is limited. Interior access, swimming, precise
mesh collision, performance/scale and native Windows are separate work.


## Actual full-map farmhouse access and camera visibility — 2026-10-16

The [map farmhouse report](MAP_FARMHOUSE.md) records the original placed building's
enterable ground floor, preserved furnishing collision, 133 source floor triangles,
a static ramp, 419 short conservative wall boxes and nearby development spawns.
The original map BIN remains an unchanged prefix; derived glTF validation has zero
errors/warnings. Terrain/profile rebasing error is below 0.000062 source units.

Formatting, denied-warning all-target Clippy, build, 42 Rust tests and 33 Python
tests pass. Three native two-client/server cases pass on RX 7900 XTX / RADV with
active Khronos Vulkan validation: actual map entry/wall/exit/re-entry, outdoor
movement and the prior interior fixture. Source ground/support heights, advancing
input sequences, authoritative/rendered XYZ, sampled animation, ready pipelines
and real inspected PNGs are checked. No validation errors or panics occur.
The default launcher also reaches readiness and cleans up both child processes.

Bevy's mesh ray caster supplements existing camera clearance for static scenery
only, and brighter ambient fill improves shaded views. Native queries execute;
these routes remain limited by existing boxes/terrain, while direct Bevy primitive
tests exercise additional mesh retraction, actor/translucency exclusion and body
hiding/recovery. Masked polygons are conservatively solid; exact texture-alpha
camera clearance is not established. The diagnostic checker now synchronizes
scene readiness before starting its routes. Normal interactive clients have no
test start gate. Evidence is ignored under `build/qualification/map-access/`.
Other map interiors, general mesh movement physics, stairs/lofts, swimming,
performance/scale and native Windows remain separate work.


## Original blacksmith main room with retained farmhouse — 2026-10-16

The [blacksmith report](MAP_BLACKSMITH.md) records a second original map building's
main-room access, with 226 source floor triangles, a fixed ramp, conservative
walls and exterior forge/bay/porch exclusions. The farmhouse's support, collision,
ceiling and visual data are retained and rebased together. Original BIN bytes are
preserved as a prefix. Derived glTF validation has zero errors/warnings and the
existing empty-emitter info. The new town profile stays within the existing
schema/wire budgets, with 3,972 movement and four camera-only boxes.

Formatting, denied-warning all-target Clippy, build, 42 Rust and 35 Python tests
pass. Both authoritative CPU routes pass in the new town profile. Two accepted
native server/two-client sessions pass on RX 7900 XTX / RADV with active Khronos
Vulkan validation: final blacksmith entry/contact/exit/re-entry in the new town
and a farmhouse regression in the prior profile. Actual source floor heights,
local/remote/server poses, input sequences, animation, camera clearance and real
inspected PNGs are checked; no validation errors/panics occur. The current default
launcher reaches readiness and cleans up its two child processes. Final client/
server hashes agree across accepted runs.

Broadening exterior exclusions correctly blocked the old secondary smoke route.
Its diagnostic walk/run now goes away from the entrance; gameplay controls and
server authority are unchanged. The final blacksmith route records 15 additional
static-mesh camera retractions and an interior camera distance near 8.68.
Ignored evidence is under `build/qualification/map-blacksmith/`, with earlier
and rejected runs separated from `native-final/`. Forge/side-bay access, other
interiors, general mesh movement physics, swimming, scale and native Windows
remain unqualified.

## Architecture alignment — 2026-10-19

The adopted handoff and [alignment record](engineering/ALIGNMENT.md) now govern
future work. Fresh Linux checks pass: `cargo fmt --check`, denied-warning
all-target workspace Clippy, workspace build, 51 Rust tests and 35 Python tests.
The existing `build/native-sdk` pkg-config/linker paths and `build/asset-tools`
Python environment resolved the earlier audit's dependency blockers; no system
installation or engine dependency was required. The CI workflow now includes the
35 synthetic Python asset/repair tests on Linux and Windows; those remote jobs
have not run because changes remain local.

The new Rust cases qualify checked f64 global/f32 local frame contracts, signed
chunk seams and origin changes; simulation dt, movement speed and input timeout
at 20/30/60 Hz; and actual configured-rate QUIC connections. Two additional Renet
2.0.0 tests use real UDP and secure netcode connect tokens with in-memory random
issuer keys. They observe authoritative movement, reliable welcome and disconnect
cleanup, and reject a wrong issuer. This does not qualify account identity,
single-use session tickets, Internet deployment, packet loss or MMO scale.

The release CPU-only eight-actor graybox baseline is under ignored
`build/alignment-baseline.jsonl`; its timings and exact limitations are in the
alignment record. It is not a rendered/networked load benchmark. Runtime
`simulation_summary` tracks work and deadline observations; full overloaded
scheduler qualification remains open.

Two real Bevy clients plus the authoritative server were rerun in an owned
private native Wayland compositor on **AMD Radeon RX 7900 XTX / RADV NAVI31,
Vulkan**, with active `VK_LAYER_KHRONOS_validation`, `WGPU_VALIDATION=1` and
`WGPU_DEBUG=1`. Graybox multiplayer passes with wire v4, validated 30 Hz snapshots,
matching rendered/server poses and actual 1280×720 captures. The town/blacksmith
route also passes entry, wall contact, exit and re-entry, retains both human
locomotion probes, and records 18 extra mesh-camera retractions for the entering
client with final camera distance about 8.77 m. No VUID, validation error or panic
markers were found by the checker. The qualification reused the previously
documented external display host harness only; the game has no engine dependency.

Accepted evidence is in `build/alignment-native/graybox/` and
`build/alignment-native/town-recheck/`. The first town checker run under
`town/` is rejected: rendered pose/route evidence completed but ambient logging
suppressed gait info markers. The harness now explicitly enables application info
logs and the complete rerun passes. Failed evidence is retained separately.

The launcher still uses Quinn/local f32 terrain movement. Full Renet client/session
migration, Avian/general collision, runtime floating origin, prediction, combat,
streaming and persistence remain open. Native Windows graphics remain unqualified.

## Pinned Avian runtime — 2026-10-19

Avian `0.8.0-dev` at `1070759b2145dc078abedace91dc7fe78013a45c` and exact Bevy
`0.20.0` are now compiled and used by the normal client and authoritative collision
queries. See [physics implementation/scope](PHYSICS_AVIAN.md). `cargo tree` confirms
one Bevy version; the standalone server's normal dependency graph has no
`bevy_render`, `bevy_winit` or `wgpu`. Renderer features moved to the client manifest.

Formatting, denied-warning all-target workspace Clippy, workspace build and
**53 Rust tests** pass: 20 client, 5 core, 18 network unit, 8 actual QUIC loopback
and 2 secure Renet loopback tests. The new cases exercise Avian capsule casts,
thin-wall blocking, retreat/layer filtering and the actual client plugin's
collider tree. Previous authority, terrain/ramp/interior and actor pass-through
tests continue to pass. Old exact box-contact assertions now allow 1 mm for the
capsule solver and 0.5 mm skin; the independent native collision verifier uses
actual capsule/box distance, including rounded corners and caps. Python checks
also pass: 31 asset tests plus four GLB repair tests.

Commands are the ordinary workspace checks with the existing ignored SDK supplied
through `PKG_CONFIG_PATH`/`LIBRARY_PATH` as documented above. Physics-only network
tests also compile/run without the window SDK. The Python suite uses the existing
`build/asset-tools` environment. No assets were regenerated or committed.

Three real server/two-client sessions pass in a private native Wayland display on
**AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan**, with active Khronos validation,
`WGPU_VALIDATION=1` and `WGPU_DEBUG=1`:

- Graybox walking, distinct identities, both rendered pawns and authoritative poses.
- Full-map blacksmith entry, wall contact, exit and re-entry with animated humans.
- Full-map farmhouse entry, wall contact, exit and re-entry with animated humans.

Each accepted run has actual 1280×720 screenshots, matching authoritative/rendered
poses, wire version 5 and `physics_backend` metadata containing the pinned revision.
Both town clients report 3,977 actual spawned static colliders; both farmhouse
clients report 2,937; graybox clients report eight. Native checks require this
metadata in addition to behaviour and image evidence. No Vulkan validation error,
VUID or panic markers occur in accepted runs. These are two-client qualification
results, not MMO capacity or performance benchmarks.

Accepted output is under ignored `build/avian-native-final/{graybox,town,farmhouse}`.
The earlier `build/avian-native/graybox` run is rejected: the short scripted input
window could expire during cold pipeline compilation, before movement was sent.
The diagnostic now waits for actual pipeline readiness before announcing its start
barrier; all three complete reruns pass with movement thresholds unchanged. The
external private-display host harness is reused only for qualification and is not
a runtime/path dependency.

Native Windows graphics, general multi-floor movement, gravity/jumping, swimming,
navmesh/AI, automatic all-building collision, prediction and platform motion remain
unqualified or unimplemented. The existing conservative collision profile is still
the content source. Avian adoption does not turn visual mesh counts into interiors.

## Renet executable foundation bridge — 2026-10-19

P1-06/07/08 and P2-02/03 now have an opt-in executable path. See
[bridge contract](NETWORKING_RENET.md) and the new
[foundation execution order](engineering/EXECUTION_ORDER.md). The imported four
source documents retain their recorded SHA-256 hashes. No full phase is complete.

Formatting, denied-warning all-target workspace Clippy, workspace build, Python
syntax checks and 35 Python asset/repair tests pass using the existing ignored
SDK/asset-tool environment. The server dependency graph remains headless, with no
normal `bevy_render`, `bevy_winit` or `wgpu` dependency. Local Rust runtime tests
add actual application-level Renet world matching, movement/acknowledgements,
departure/fresh-ticket reconnect, spent-ticket rejection, invalid intent, ticket
endpoint/expiry/truncation/trailing-data checks, exclusive file creation and Unix
0600 permissions. Runtime tests use wall-clock timers and real UDP sockets.

The full workspace passes **56 Rust tests**: 20 client, 5 core, 18 network unit,
8 Quinn loopback, 2 earlier Renet proof and 3 new executable-runtime integration
tests. The final run is `cargo test --locked --workspace -j 1` with the documented
SDK environment; its local log is `build/renet-tests-final.log`.

The native qualification uses the previously documented external private-display
host harness only, without introducing a game dependency. Accepted output is
under ignored `build/renet-native-final/`: Renet graybox walking and full-map
blacksmith access, plus the retained Quinn graybox regression. Each run uses a
real server and two real Bevy clients, 1280×720 screenshots and authoritative/
rendered pose comparison, with actual `transport` metadata. The map route checks
entry, wall contact, exit/re-entry and human locomotion. The device is **AMD Radeon
RX 7900 XTX / RADV NAVI31, Vulkan**, in owned private Wayland, with active
`VK_LAYER_KHRONOS_validation`, loader layer logging, `WGPU_VALIDATION=1` and
`WGPU_DEBUG=1`; accepted logs have no validation-error/VUID/panic markers.

The initial graybox output under `build/renet-native/` is rejected: server and
render poses agreed, but the short timed diagnostic could move the remote pawn
outside the screenshot. The short diagnostic now requests movement until its
observed displacement reaches 4 m; screenshot pawn-pixel and greater-than-3 m
movement acceptance checks remain unchanged. This does not change normal input,
authority, movement speeds or the building/gait routes. The complete reruns use
the corrected diagnostic. Parallel test linking competed with the two full-map
clients for memory; the test build was stopped and rerun with `-j 1`.

These are local two-client correctness checks, not latency/bandwidth benchmarks.
Controlled adverse networks, Internet flood resistance, production account/session
authorization, Windows execution, global runtime poses, prediction and MMO capacity
remain unqualified. Quinn remains the default until the replacement qualification
gate passes; local ticket issuance does not complete P3-01 authentication.

## Renet local replacement gate and default switch — 2026-10-19

The [qualification record](RENET_QUALIFICATION.md) defines the exercised profiles,
queue/traffic ceilings and limitations. Three new test cases use the real
executable runtime over actual authenticated UDP: baseline and two deterministic
impaired profiles; short/timeout outages and old-packet replay; sender channel
pressure without I/O. The tests observe monotonic snapshots/acknowledgements,
actual movement and stopped-position agreement, input expiry, removal/fresh-ticket
replacement, and no relay overflow. The replacement also continues to receive new
snapshots/acknowledgements during captured old-packet injection; a frozen or dead
client cannot satisfy that check. Secrets/ciphertext are not saved to reports.

Formatting, denied-warning all-target workspace Clippy, workspace build,
**59 Rust tests** and **35 Python tests** pass with the existing ignored SDK and
asset-tool environment. The Rust breakdown is 20 client, 5 core, 18 network unit,
8 Quinn loopback, 2 earlier secure Renet proof, 3 executable bridge and 3 adverse
network/queue cases. Final workspace log: `build/renet-default-final-tests.log`;
stronger targeted replay recheck: `build/renet-replay-final.log`. CI's test command
now uses `-j 1` to avoid the observed concurrent test-link memory pressure; the
workflow has not been pushed or executed remotely.

Reproduce the measured nonrendered network suite with:

```sh
THEIA_NETWORK_REPORT_DIR="$PWD/build/renet-qualification" \
  cargo test --locked -p theia-network --test renet_adverse -j 1 -- --nocapture
```

Accepted measurements are in the named ignored JSON reports. These use actual
wall-clock timers, two clients and a 30 Hz graybox authority. They do not represent
rendered impairment testing, wall-clock scheduling under MMO load, Internet
hosting or Windows execution. Normal client/server diagnostics now expose/log
observed queue high waters and client RTT/loss/byte-rate/snapshot counters. The
native verifier checks these diagnostics for Renet; wire remains version 5.

Native qualification is under `build/renet-default-final/`: default Renet graybox,
blacksmith and farmhouse routes; explicit Quinn graybox; and launcher cleanup.
Each rendered run uses a real server and two Bevy processes, 1280×720 screenshots,
authoritative/rendered pose agreement, active Khronos Vulkan validation plus
`WGPU_VALIDATION=1`, `WGPU_DEBUG=1` and loader layer logging. Device: **AMD Radeon
RX 7900 XTX / RADV NAVI31, Vulkan**, private native Wayland. No accepted validation
error, VUID or panic markers occur. The existing external display-host helper is
used only for qualification, without a runtime dependency.

The launcher qualifier executes the default launcher twice with two real clients.
Ctrl+C must return success and remove owned children/tickets; a deliberately
terminated owned client must cause a nonzero launcher exit and the same cleanup.
Its JSON evidence records both results. `/proc` provides Linux ownership evidence;
this is not a Windows lifecycle/ACL check or a manual gameplay walkthrough.

The initial `build/renet-default-native/farmhouse` result is rejected. It completed
entry, wall, exit/re-entry, but final idle drift was 0.20289 m against the existing
0.2 m goal tolerance. The diagnostic now keeps requesting ordinary input toward
its final waypoint until settled, rather than stopping after its first arrival
while a previous intent may still be in flight. Thresholds, normal controls,
movement authority and speeds are unchanged. The accepted reruns keep both the
original clearance checks and final waypoint tolerance. Earlier successful
artifacts in that initial directory remain earlier attempts, not final evidence.

The normal launcher and multiplayer verifier now select Renet by default. Explicit
Quinn and legacy standalone CLI meanings remain available for regressions. This
closes the bounded local replacement gate; provider-backed authorization,
exhaustive attack/rate-limit testing, broader outage profiles, automatic resume,
Internet deployment, native Windows and MMO capacity remain open. The next gate
is live global/local coordinate and entity/session identity integration.

## Runtime global coordinates and incarnation/frame identity — 2026-10-19

P1-02/03/04/08 and P2-03/05 bounded slice. See
[contract and limits](COORDINATES_IDENTITY.md). The authority stores checked global
f64 poses, queries Avian in a fixed bounded content frame, and emits explicit
f64-origin/f32-offset snapshots. Both transports use shared validated welcome and
incarnation/sequence guards. Wire and Quinn ALPN are version 6. The public epoch
is a lifecycle namespace; established transport admission still controls authority.

Formatting, denied-warning all-target workspace Clippy, workspace build,
**66 Rust tests** and **35 Python tests** pass with the documented ignored native
SDK/asset-tool environment. Rust breakdown: 20 client, 7 core, 21 network unit,
8 Quinn loopback, 3 adverse network/queue, 2 earlier Renet proof, 5 executable
Renet runtime tests. The final workspace log is
`build/coordinates-workspace-tests.log`. Tests use the same serialized protocol
in both transports; malformed-message tests now use v6/valid epochs so unrelated
schema failures are not hidden by a version mismatch.

Added checks cover checked incarnation/entity keys, negative region seams,
invalid/truncated/trailing welcomes, zero/wrong epochs, invalid/overflowing global
origins, out-of-frame projection, eight-peer datagram budget, non-reused lifecycle
and restart identity, and canonical authority equivalence across positive/negative
billion-metre origins with centimetre anchor offsets. The real Renet loop changes
wire origins multiple times while local speed/global reconstruction remain
continuous. Raw authenticated clients send a wrong-epoch maximum sequence before
a current-epoch low sequence; both executable transports must accept the latter.
Old-epoch traffic cannot poison coalescing/sequence state.

The deterministic loss/delay/duplicate/reorder, outage, sender-queue-pressure and
old encrypted-packet replay suites pass again on v6. This run's independent local
reports are under `build/coordinates-qualification/`; the earlier wire-v5 numbers
remain dated evidence in [Renet qualification](RENET_QUALIFICATION.md). These are
two-client graybox wall-clock correctness/traffic observations, not MMO load or
rendered impairment benchmarks.

Native output is under `build/coordinates-native/`:

- Renet graybox at positive and negative billion-metre anchors, 2 m wire span.
- Renet blacksmith access at a positive billion-metre anchor, 4 m wire span.
- Default-origin farmhouse access with the ordinary 128 m span.
- Explicit Quinn graybox at a negative billion-metre anchor, 2 m span.
- Actual default launcher Ctrl+C/child-failure cleanup with two admitted clients.

Each rendered check requires two real clients, matching rendered/server poses,
actual 1280×720 screenshots, v6 and same-incarnation metadata. Distant checks must
observe actual wire-frame changes and compare reported global positions against
independently subtracted local render coordinates. Map checks retain entry,
wall contact, exit/re-entry, animation, collision and camera requirements. Capture
samples raw/global and projected/local data from one mailbox state to prevent a
diagnostic cross-tick race. Normal input and movement authority are unchanged.

Device: **AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan**, private native Wayland,
active `VK_LAYER_KHRONOS_validation`, `VK_LOADER_DEBUG=layer`, `WGPU_VALIDATION=1`,
`WGPU_DEBUG=1`. Accepted logs contain no validation-error/VUID/panic markers. The
previously documented external display-host helper is qualification-only. Native
Windows and other GPUs remain unqualified. Imported source documents retain their
recorded hashes; no assets or credentials are added to Git.

Client rendering/Avian content anchors remain fixed to this admitted fixture.
Only relative replication origins recenter live: this is not an automatic
render/physics floating-origin system, chunk streaming, region ownership/handoff
or persistent character identity. Those interfaces remain open in the task overlay.

## Client render/static-Avian origin shifting — 2026-10-19

The [client rebase continuation](FLOATING_ORIGIN.md), P1-03/09 and P2-05/07,
reprojects canonical scenery roots and actual collider Positions/Transforms using
one client frame, while keeping global authority and wire v6 unchanged. The
static client physics plugins run in PostUpdate after placement. Camera follow,
transform propagation, mesh clearance, root-camera global synchronization and
Last-stage Avian probing/capture are ordered explicitly.

Formatting, denied-warning all-target workspace Clippy, workspace build,
**70 Rust tests** and **35 Python tests** pass with the documented ignored SDK
and asset-tool environment. Breakdown: 23 client, 8 core, 21 network unit,
8 Quinn loopback, 3 adverse network/queue, 2 earlier Renet proof, 5 executable
Renet runtime tests. Final workspace log: `build/rebase-workspace-tests.log`.
New tests cover f64 snap/reprojection across signed seams, a single-update actual
Avian BVH query after rebasing, all static collider Positions, unchanged child
poses and late-child propagation, global interpolation/camera-delta preservation,
and identical camera obstruction/visibility before/after rebasing. Client tests
use actual standard physics/transform plugins; no mocked collider tree is used.

Five real server/two-Bevy-client native checks pass under ignored
`build/rebase-native/`, using the same approved map/model bytes:

| Run | Configuration | Observed render shifts (entering/other peer) |
|---|---|---|
| Positive distant graybox | +1e9 m anchor, 2 m render span | 1 / 2 |
| Negative distant graybox | −1e9 m anchor, 2 m render span | 1 / 3 |
| Blacksmith | +1e9 m anchor, 2 m render span | 22 / 5 |
| Farmhouse | Ordinary anchor, 2 m render span | 24 / 3 |
| Quinn graybox | −1e9 m anchor, 2 m render span | 1 / 3 |

Each client reports global/content/render/wire origins, render and wire revision
counts, current physics frame revision and actual grounded-pawn BVH probe counts.
The verifier converts rendered positions through f64 origin differences before
comparing them with authoritative content/global poses. Physics revision must
match the render revision, probes must have run, and maximum ground error must
remain below 2 mm. The observed maximum was about **0.00865 mm** in the farmhouse
run; this is a measured consistency error, not a general precision guarantee.
The entering blacksmith/farmhouse clients completed 1,229/1,664 actual probes.

All prior image and behavior checks remain: real 1280×720 screenshots, distinct
identities, matched server/local/remote poses, source-ground/capsule clearance,
building entry/contact/exit/re-entry, sampled human animation and camera clearance.
Both actual default-launcher lifecycle checks also pass: Ctrl+C returns success,
an owned client failure returns nonzero, and each removes its three game child
processes plus temporary bearer tickets. The normal launcher uses the default
128 m render span; the 2 m span deliberately exercises frequent changes in native
regressions and does not change movement speed or server authority.

Device: **AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan**, owned private native
Wayland display, active `VK_LAYER_KHRONOS_validation`, `VK_LOADER_DEBUG=layer`,
`WGPU_VALIDATION=1`, `WGPU_DEBUG=1`. Accepted runs have no validation-error/VUID/
panic markers. The existing external display-host helper remains qualification
infrastructure only, with no game dependency. Native Windows and other GPUs
remain unqualified; remote CI has not been run or pushed.

This is automatic client render/static-physics rebasing for the bounded admitted
fixture. Server movement/query scenes remain in their immutable content frame;
the camera's shared content-space query cache is accessed through explicit
coordinate conversion. No dynamic-body contact-history rebase, streaming,
multi-chunk/region ownership, general movement, prediction, performance or MMO
capacity claim is made. Imported engineering source documents and source assets
remain unchanged; all work stays local.


## Scheduler and cold query-scene qualification — 2026-10-19

P1-05/P1-06/P1-12's bounded slice now passes against the actual executable Renet
loop: eight authenticated loopback UDP peers at 20/30/60 Hz, burst/replayed/wrong-
epoch intents, monotonic acknowledgements and bounded speed, simulation-time
input expiry, a 250 ms authority-only executor stall, recovery, and removal of
an over-budget sender while seven remain connected. Fixed-memory histogram
percentiles are inclusive upper bounds; exact maxima, skipped intervals and
separate 5 ms transport-pump timings are recorded. See
[contract, table and reproduction](SCHEDULER_QUALIFICATION.md).

Native CPU: AMD Ryzen 7 7800X3D, Linux x86_64 7.0.0-38-generic, Rust 1.99.0.
Final release evidence: ignored `build/scheduler-release-final/scheduler.json`
and `.log`; debug workspace qualification is in `build/scheduler-workspace-final.log`.
The three-second moving windows delivered 60/90/180 ticks. These are eight-peer
in-memory graybox results, not full-map, crowd, database, soak or MMO capacity.

Fresh commands pass:

```bash
cargo fmt --check
PKG_CONFIG_PATH="$PWD/build/native-sdk/usr/lib/x86_64-linux-gnu/pkgconfig" \
LIBRARY_PATH="$PWD/build/native-sdk/usr/lib/x86_64-linux-gnu" \
  cargo clippy --locked --workspace --all-targets -- -D warnings
PKG_CONFIG_PATH="$PWD/build/native-sdk/usr/lib/x86_64-linux-gnu/pkgconfig" \
LIBRARY_PATH="$PWD/build/native-sdk/usr/lib/x86_64-linux-gnu" \
  cargo test --locked --workspace -j 1
PKG_CONFIG_PATH="$PWD/build/native-sdk/usr/lib/x86_64-linux-gnu/pkgconfig" \
LIBRARY_PATH="$PWD/build/native-sdk/usr/lib/x86_64-linux-gnu" \
  cargo build --locked --workspace -j 1
THEIA_SCHEDULER_REPORT_DIR="$PWD/build/scheduler-release-final" \
  cargo test --locked --release -j 1 -p theia-network \
  --test scheduler_qualification -- --nocapture
build/asset-tools/bin/python -m unittest discover -s tools/asset_tests
build/asset-tools/bin/python -m unittest discover -s tools -p test_align_glb.py
```

Total **73 Rust tests**: 23 client, 9 core, 21 network unit, 8 Quinn loopback,
1 cold initialization regression, 3 adverse/queue, 2 secure Renet proof,
5 executable Renet runtime, and 1 scheduler qualifier (internally three rates).
The 31 asset and 4 GLB Python tests also pass. Build/Clippy logs are
`build/scheduler-build.log` and `build/scheduler-clippy-final.log`.

The initial workspace attempts exposed a pre-existing cold graybox query-scene
deadlock in parallel Bevy tests; these were interrupted, not counted as passes.
Captured stacks are in `build/scheduler-physics-deadlock-full.log`. A focused
regression reproduced the issue with eight simultaneous callers/two shared
compute workers. Building outside the scene publication lock and using standard
single-thread execution for its tiny startup schedules resolves it. The regression
passes, three further cold process repetitions pass, and the final full workspace
suite passes with its normal parallel test harness. The live client physics
schedule, collider geometry and wire v6 are unchanged.

A freshly built actual server/two-client native Renet graybox check also passes:
`build/scheduler-native/graybox/verification.json`, logs, captures and two actual
1280×720 screenshots. It checks distinct identities, matching rendered/server
poses, movement/stop and real origin shifts (2 m diagnostic span), plus current
Avian ground probes. Device: **AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan**,
owned private native Wayland display, active `VK_LAYER_KHRONOS_validation`,
`VK_LOADER_DEBUG=layer`, `WGPU_VALIDATION=1`, `WGPU_DEBUG=1`. No VUID, validation
error or panic markers. The existing external display host is qualification
infrastructure only. Processes/tickets are cleaned up. The earlier tile/building
captures remain historical; they were not rerun for this scheduler/startup change.
No Windows execution/cross-build, other GPU or remote CI evidence is claimed.
Imported source handoff checksums remain unchanged. All work stays local.


## Versioned content packages — 2026-10-19

The bounded P1-10/P2-08 package slice passes schema/identity/artifact error tests,
shared Rust/Python golden fingerprint, headless collision-only loading, real
secure admission and stale-revision rejection. Default development launch now
requires a manifest. The existing ignored outdoor/farmhouse/town/patch packages
are published at revision 1; visual/profile/source bytes remain unchanged.
See [contract and reproduction](CONTENT_PACKAGES.md).

Fresh fmt, denied-warning all-target workspace Clippy, workspace build and tests
pass. Logs: `build/content-clippy-final.log`, `build/content-build.log`,
`build/content-workspace-tests.log`. **79 Rust tests**: 23 client, 10 core,
21 network unit, 5 package, 8 Quinn loopback, 1 cold initialization, 3 adverse/queue,
2 secure proof, 5 Renet executable and 1 three-rate scheduler qualifier.
**39 Python tests** pass: 35 asset tests (including four publisher checks) and
4 GLB repair tests. Logs: `build/content-python-final.log`,
`build/content-python-glb.log`. Test binding payloads are explicitly synthetic;
they are not glTF rendering evidence.

Fresh native actual server/two-client checks pass:

- `build/content-native/town/verification.json`: packaged default town, animated
  human blacksmith route, distant global origin, 4 m wire/2 m render spans.
- `build/content-native/farmhouse/verification.json`: packaged farmhouse route,
  animation, movement/stop, collision/camera and 2 m render rebases.
- `build/content-native/quinn/verification.json`: retained explicit Quinn graybox,
  negative distant origin and 2 m wire/render spans.
- `build/content-native/launcher/verification.json`: both actual default-launcher
  cleanup cases, Ctrl+C success and owned client failure, each removing its three
  game children and temporary bearer tickets.

The verifier now derives expected content identity from the canonical package
contract and checks reported logical ID/revision, while preserving legacy fixture
checks. Captures still check actual rendered/server poses, source ground and
capsule clearance, sampled animation, camera clearance and real rebased Avian
queries. Device: **AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan**, private owned
native Wayland display, active `VK_LAYER_KHRONOS_validation`,
`VK_LOADER_DEBUG=layer`, `WGPU_VALIDATION=1`, `WGPU_DEBUG=1`. No validation errors,
VUIDs or panics. All owned processes/tickets are cleaned up; native/cross-built
Windows and remote CI remain unqualified. The external display-host helper is
qualification infrastructure only. Source handoff checksums remain unchanged.
No streamed tile, general collision, skeleton-contract or capacity claim is made.
All work stays local; next is bounded chunk ownership/residency proof.


## Isolated bounded chunk residency — 2026-10-19

The [headless precursor](CHUNK_RESIDENCY.md) passes against actual disk packages
and Avian query scenes on original geometry. It checks half-open ownership,
collision removal after unload, protected-owner/budget rejection, corrupt/missing
candidate rollback and explicit global/query-frame consistency. It does not use
the normal imported tile, the production simulation loop or client scene loading.
Visual-root descriptors are metadata, not evidence of rendering or streaming.

Fresh fmt, denied-warning all-target workspace Clippy and **84 Rust tests** pass:
23 client, 11 core, 21 network unit, 4 chunk residency, 5 content package,
8 Quinn loopback, 1 cold initialization, 3 adverse/queue, 2 secure proof,
5 Renet executable and 1 scheduler qualifier covering all three rates. Logs:
`build/chunk-clippy.log`, `build/chunk-workspace-tests.log`,
`build/chunk-residency-tests.log`. **41 Python tests** pass: 37 asset tests
(including two new original chunk-generator checks) and the unchanged four GLB
repair tests from the package verification. Log: `build/chunk-python.log`;
preceding GLB log: `build/content-python-glb.log`.

Three actual `cargo run --locked -p theia-network --example chunk_residency`
executions pass at query origins `[0,0,0]`, `[1e9,0,1e9]` and `[-1e9,0,-1e9]`.
The exact preparation/command lines are in the contract document. Logs:
`build/chunk-proof.log`, `build/chunk-positive-proof.log`,
`build/chunk-negative-proof.log`. Each commits three loads/three unloads, maximum
two active chunks, actual continuous flat seam-ground queries and a real thin-wall
capsule hit at fraction 0.27474624. The generator produces actual original GLBs;
Python verifies their triangles against collision data and package/index hashes.

Linux x86_64, AMD Ryzen 7 7800X3D, Rust 1.99.0. These are correctness proofs, not
load/soak or memory-capacity measurements. A two-chunk active limit does not mean
two chunks of peak staged memory; old/new sets and combined/per-chunk query worlds
can coexist. Normal server dependencies remain headless, verified in
`build/chunk-server-dependencies.txt` without bevy_render, bevy_winit or wgpu.
No rendering path changed in this isolated precursor, so no new GPU evidence is
claimed. The immediately preceding packaged town/farmhouse/Quinn/launcher native
runs above retain their exact scope and binary hashes. Windows remains unqualified;
no source handoff changes, commit or push. Next: bounded standard-Bevy visual
consumption/publication before normal chunk adoption or the general controller.

## Actual Bevy chunk publication — 2026-10-19

P2-06/P2-07/P2-08's original-fixture visual gate now passes in the real client:
`--chunk-smoke` uses one background Bevy I/O task for a verified residency candidate,
standard GLB loading/world scenes, hidden hierarchy staging, retained root/handle
reuse and one main-world publication with actual Avian static bodies. PostUpdate
physics updates before Last checks compare the live ground/capsule queries with
the published shared collision scene on every observed frame, including staging.
See [lifecycle, reproduction and limits](CHUNK_PRESENTATION.md).

Fresh fmt, denied-warning all-target Clippy, workspace build and **85 Rust tests**
pass: 23 client, 11 core, 21 network unit, 5 chunk residency, 5 content package,
8 Quinn loopback, 1 cold initialization, 3 adverse/queue, 2 secure proof,
5 Renet executable and 1 three-rate scheduler qualifier. The new staged-set test
checks that candidate publication/visual validation cannot mutate the old set.
Logs: `build/chunk-visual-workspace-tests.log`,
`build/chunk-visual-client-final-tests.log`, `build/chunk-visual-clippy-final.log`,
`build/chunk-visual-build.log`, `build/chunk-visual-build-corrected.log`.
**41 Python tests** pass (37 asset + 4 GLB), in
`build/chunk-visual-python.log` and `build/chunk-visual-python-glb.log`.

An initial native attempt failed the independent image checks: the diagnostic had
used the default LUT-dependent tonemapper without the corresponding feature,
producing magenta fallback pixels. That run is not accepted evidence. The camera
now explicitly uses the existing Reinhard path. All corrected native cases pass:

| Run under `build/chunk-visual-native/` | Actual checked behavior | Live frame checks |
|---|---|---|
| `origin/verification.json` | Red/green pair → green/blue pair → empty | 96 |
| `positive/verification.json` | Same publication at query origin `[1e9,0,1e9]` | 94 |
| `negative/verification.json` | Same publication at `[-1e9,0,-1e9]` | 93 |
| `asset-failure/verification.json` | Correctly bound invalid glTF rejects; old red/green/collision remain, then empty | 89 |

Every run has three actual 960×640 screenshots and raw `report.json`/`run.log`.
The independent verifier requires visible color counts, real ownership/state,
completed process success and no validation/panic markers. Positive runs remove
all red pixels and add 69,473 blue pixels while retaining about 74,490 green
pixels. Initial red count is 33,414. The failed replacement preserves the exact
initial red/green counts and never adds blue. Final unload has zero classified
floor pixels. Retained middle-root entity identity is asserted in the actual
client. This is evidence of real scene/physics publication and removal, not just
allocation counts. It does not prove immediate GPU allocation release or general
first-frame pipeline readiness.

The failure case edits only a temporary original fixture: it republishes the
third chunk with a consistent profile/manifest/index and invalid glTF bytes.
Bevy's actual asset decoder rejects it; only that expected asset-server error is
allowed by the verifier. Normal runs require no error log. Temporary assets and
all owned processes are removed on completion. Proofs record actual client/index
hashes. Driver activation is proved by `Insert instance layer
"VK_LAYER_KHRONOS_validation"` in loader output.

The freshly built normal Renet server/two-client graybox also passes at
`build/chunk-visual-native/normal-graybox/verification.json`, with its existing
pose/camera/render-rebase/live-physics checks. Device for every accepted native
run: **AMD Radeon RX 7900 XTX / RADV NAVI31, Vulkan**, private native Wayland,
active `VK_LAYER_KHRONOS_validation`, `VK_LOADER_DEBUG=layer`,
`WGPU_VALIDATION=1`, `WGPU_DEBUG=1`. No VUID/validation errors/panics. Corrected
qualification log: `build/chunk-visual-native-corrected.log`. The external display
host remains qualification-only infrastructure. Earlier packaged map/launcher
runs retain their earlier binary hashes and were not counted as rerun here.

Normal tile partitioning/adoption, authority activation with changing actors,
request cancellation/supersession, prediction, general character movement and
production memory/streaming capacity remain open. No Windows or remote CI execution
is claimed. Pins/default 30 Hz/wire v6 and imported handoff bytes remain unchanged;
all changes stay local. Continue with the shared general kinematic controller.

## Shared graybox kinematic controller — 2026-10-19

`theia-network::kinematic` now drives graybox authority and the offline sandbox.
Imported maps retain their previous grounded walker during migration; the
[controller contract](KINEMATIC_CONTROLLER.md) records that boundary and the next
content/activation gates. Wire/Quinn ALPN is v7 with held-level jump input.

Executed Linux checks on the final source:

- `cargo fmt --check`, `git diff --check`, locked workspace build and
  `cargo clippy --locked --workspace --all-targets -j 1 -- -D warnings` pass.
  Logs: ignored `build/kinematic-build-final.log` and
  `build/kinematic-clippy-final.log`.
- `cargo test --locked --workspace -j 1`: **98 tests pass**, including 12 new
  actual-Avian controller cases and real secure Renet jump replication at
  20/30/60 Hz. Log: `build/kinematic-workspace-final.log`. The exact stationary
  door regression passes; its assertion was retained after fixing contact-normal
  noise with bounded refinement and a 10-micrometre residual-slide cutoff.
- Python asset/GLB suites: **41 tests pass** (37 plus four), recorded in
  `build/kinematic-python-assets.log` / `build/kinematic-python-glb.log`.
  The updated multiplayer qualifier also passes Python compilation.
- Fresh release eight-peer scheduler qualification passes at 20/30/60 Hz with
  the actual controller. See [metrics and execution limits](SCHEDULER_QUALIFICATION.md#shared-controller-rerun--2026-10-19).
  30 Hz remains the default. This is local eight-peer graybox evidence, not MMO
  capacity or full-map movement performance.

The final native binary qualification uses an owned private Wayland compositor,
AMD Radeon RX 7900 XTX / RADV NAVI31 Vulkan, with active
`VK_LAYER_KHRONOS_validation` verified by loader trace and
`WGPU_VALIDATION=1`, `WGPU_DEBUG=1`. Artifacts live under ignored
`build/kinematic-native-final/`; each `verification.json` records binary, lockfile
and image hashes. There is no native Windows or Windows cross-build evidence.

Final Renet positive-billion-origin and explicit Quinn negative-billion-origin
two-client jump cases pass with 2 m wire/render thresholds. Actual local and
remote rendered jump peaks span approximately 1.229–1.260 m above content ground,
with observed render rebases and independent live Avian/content-frame agreement.
Captured graybox presentation was visually inspected. Peak samples establish
rendered jump observation; the later screenshots show the landed scene. Existing
camera, transport, snapshot/ack and independent screenshot-pixel checks remain.

Final-binary packaged town and farmhouse two-client routes also pass, preserving
human animation, map access, camera clearance and same-frame physics after render
rebases. These remain tests of the legacy imported-map walker. The town local
client records 24 render rebases / 1,972 live query probes with maximum ground-ray
disagreement below 0.2 micrometres. Both default launcher cases (interrupt and
injected child failure) pass owned-process and ticket cleanup. All five final
native verification records report `passed: true`; no Vulkan validation errors
or runtime panics occur. This does not qualify native Windows, imported-map
jumping, general source-mesh fidelity, prediction or normal chunk activation.

## Normal imported-map controller adoption — 2026-10-19

Normal terrain authority now uses the shared kinematic controller rather than
`Terrain::move_player` and per-step highest-surface selection. Existing profile
bytes/package fingerprints are unchanged. Runtime normal-world adoption adds four
movement-only envelope colliders; chunk scenes retain open ownership seams.
The [controller contract](KINEMATIC_CONTROLLER.md) specifies cached legacy spawn
selection, finite barriers, held jump and per-actor recovery.

Final-source Linux CPU checks:

- Locked workspace build, `cargo fmt --check`, `git diff --check` and denied-warning
  workspace/all-target Clippy pass. Logs: ignored `build/map-kcc-build-final.log`
  and `build/map-kcc-clippy-final.log`.
- **103 Rust tests pass** in `build/map-kcc-workspace-final.log`. Four new normal
  terrain tests cover shared jumping beneath an overpass at all qualified rates,
  finite fall recovery without held retrigger, boundary sliding/water exclusions,
  and idempotent world adoption with open chunk seams. A new authority unit test
  injects one invalid actor and verifies independent recovery/sequence progress.
  Existing exact stationary-door and contact assertions pass after suppressing
  solver-scale contact travel; no stationary assertion was relaxed.
- The explicitly ignored private-map test is executed separately with absolute
  `THEIA_MAP_CONTROLLER_FIXTURES` paths. It drives both town and farmhouse routes
  through actual normal authority at 20/30/60 Hz: all six waypoint/wall/re-entry
  stages pass in all six cases with **zero recoveries**. Log:
  `build/map-kcc-routes-final.log`. This is headless map movement evidence, not a
  rendered or network test; the default workspace run reports it ignored.
- **43 Python tests pass**: 39 asset tests plus four GLB tests, logs
  `build/map-kcc-python-assets.log` / `build/map-kcc-python-glb.log`. Two new
  independent geometry tests check Euclidean capsule/triangle clearance for
  flat/sloped/stacked/penetrating poses and face/edge/corner/intersection cases.
  The native qualifier uses these calculations rather than highest-height
  equality, while retaining capsule/box and authored route checks. Python
  compilation of the modified qualifier/modules passes.

Final-binary native evidence uses the same owned private Wayland compositor and
AMD Radeon RX 7900 XTX / RADV NAVI31 Vulkan, active Khronos validation proven by
loader trace. `build/map-kcc-native-final/graybox-jump/verification.json` passes
positive-billion-origin/2 m render-wire rebase jump qualification. The two early
map-jump attempts under `map-kcc-native/` and `map-kcc-native-final/` are rejected
qualification attempts: the first diagnostic still required horizontal travel,
the second still required walk/run clips from stationary actors. Their captures
show jumps without server recovery, but are not accepted verification records.
The corrected stationary case requires >1 m measured local/remote displacement
relative to each actor's initial height, unchanged XZ, two standing animated
characters and independent final mesh/body clearance. Walking-route cases retain
their full locomotion requirements. Jump animation remains unimplemented.

The fresh final-controller release scheduler rerun also passes all 20/30/60 Hz
eight-peer profiles. See [current metrics](SCHEDULER_QUALIFICATION.md#imported-controller-adoption-rerun--2026-10-19).
It ran concurrently with native map rendering; it is not an isolated performance
comparison or eight-peer imported-map authority benchmark.

Accepted stationary imported-map jump evidence:
`build/map-kcc-native-qualified/town-jump/verification.json`. Actual local/remote
peaks are about 1.200–1.222 m relative to initial heights; independent final mesh
clearance is 7.995–8.000 mm. Two characters remain animated in standing playback;
walking/run clips are not required for this stationary case. Screenshot inspected.
The ordinary town route also passes in `map-kcc-native-qualified/town/`, with its
walking/running, camera, entry/wall/exit/re-entry and no-recovery checks retained.

All final accepted native cases pass: graybox jump under
`build/map-kcc-native-final/graybox-jump/`, and stationary town jump, town route,
farmhouse route, explicit Quinn full-tile run and both default Renet launcher
cleanup cases under `build/map-kcc-native-qualified/`. Each native verification
records binary/lock/image hashes and active validation; no runtime panic,
validation error or unexpected movement recovery is reported. The final town
interior screenshot was visually inspected. Independent final mesh clearance
across all imported cases spans about **7.882–8.001 mm**. The Quinn full-tile
actors travel about **50.77 m / 30.61 m**, with 14/16 render rebases, preserving
travel beyond the old patch. This still does not qualify original mesh fidelity,
arbitrary collision authoring, normal chunk streaming, prediction, gameplay death/
respawn, native Windows or MMO-scale performance.


## General collision/schema-3 safe-stop checkpoint — 2026-10-19

The user requested stopping before the new native qualification/default-map gate.
[Format/runtime/import support](GENERAL_COLLISION.md) is implemented; staged map
revision-2 packages and the original mesh fixture remain local under ignored
`build/`. The normal launcher default was not changed. Source handoff documents
and legacy prepared packages are preserved.

Executed Linux checks on this checkpoint:

- Locked workspace build passes (`build/general-collision-build.log`), as do
  fmt, `git diff --check` and denied-warning all-target workspace Clippy
  (`build/general-collision-clippy.log`).
- **108 Rust tests pass** (`build/general-collision-workspace.log`), including
  combined floor/wall/ceiling motion at all rates, tight combined headroom,
  explicit lower/upper spawns, zero-spawn chunk admission, malformed geometry/
  spawn rejection and schema-3 collision/manifest binding without visual I/O.
- **46 Python tests pass** (42 asset plus four GLB):
  `build/general-collision-python-assets.log` / `build/general-collision-python-glb.log`.
  New importer tests cover node instances/transforms, visual/collision binding,
  source immutability, no implicit chunk spawns and exclusive revisioned migration.
  Modified Python modules compile successfully.
- The separately invoked private-map test passes all six staged-v3 town/farmhouse
  routes at 20/30/60 Hz with zero recoveries:
  `build/general-collision-v3-routes.log`. This is actual normal headless authority,
  not rendered/network evidence. The workspace run reports that private case ignored.

No new native schema-3 mesh/map/chunk or Windows qualification was started for
this slice before the stop request. Existing native results remain historical
baselines, not proof of this changed source. Resume with actual Bevy original mesh,
revision-2 map routes/jump/full-tile, chunk publication/rollback and launcher
checks under active Vulkan validation before default adoption. Moving-actor chunk
activation, streaming and prediction remain open.

## Schema-3 native qualification continuation — 2026-10-20

Resumed from the safe-stop checkpoint. Native execution found two issues that the
CPU controller tests had not exposed: zero-delta first-frame admission/rebasing
left live client queries stale, and legacy highest-surface camera sampling treated
a combined room's roof as ground. The client now refreshes the standard Avian BVH
when its anchored frame changes even at zero delta. Schema-3 camera clearance
uses the existing mesh sweep directly. Focused regressions cover first-frame and
subsequent zero-delta queries and independent camera clearance under a mesh ceiling.

Rejected artifacts are retained under ignored
`build/general-collision-native-20261020/` and `...-bvh/`, each with `REJECTED.md`.
The latter's original-room JSON results said passed under an insufficient visual
oracle: actual screenshot inspection showed a collapsed camera/hidden character.
Those are not accepted qualification. The corrected qualifier requires local
visibility and >1 m camera distance; actual original-room screenshots were then
visually inspected. A stationary jump does not claim an XZ wire rebase; moving
map cases explicitly exercise the wire threshold instead.

Accepted final-binary cases live under ignored
`build/general-collision-native-qualified/`, with binary/lockfile/image hashes in
each verification record. Native original-room walk and jump cases pass after the
fixes, with real animated humans, noncollapsed camera and independent mesh capsule
clearance. Positive-billion-origin stationary town jump also passes; it proves
render/physics rebasing, not stationary XZ wire rebasing. Device is AMD Radeon
RX 7900 XTX / RADV NAVI31 Vulkan, owned private Wayland, active Khronos validation
proved by loader trace. Exact remaining cases/checks are recorded below as completed.

Final accepted checks on this source:

- Locked workspace build, fmt, denied-warning workspace/all-target Clippy and
  **110 Rust tests pass**. Logs: `build/general-collision-qualified-build.log`,
  `...-tests.log`, `...-clippy.log`. **46 Python tests pass** (42 asset, four GLB)
  in the `general-collision-resume-python-*` logs; the modified launcher and
  multiplayer qualifiers pass compilation and their real process cases below.
- All six actual two-client native cases pass: original mesh walk/jump, positive
  billion-origin town jump and moving town route, farmhouse route and negative
  billion-origin explicit Quinn full-tile travel. Characters remain visible with
  camera distance >1 m; original-room camera distance is about 9.72 m. Original
  and town jumps show about 1.200–1.222 m local/remote displacement. Independent
  final capsule/mesh clearance across imported/original cases is about
  **7.882–8.001 mm**. No unexpected movement recovery occurs.
- Moving town records 22/5 render rebases and four wire rebases; full-tile Quinn
  records 15/16 render and 16 wire rebases, about **51.03 / 32.61 m** travel.
  Largest reported live Avian/content ground error in those cases is below
  0.3 micrometres. These are small loopback presentation/collision checks, not
  latency, crowd or performance capacity claims.
- Four schema-3, zero-spawn actual-client chunk cases pass at origin and signed
  billion-metre roots, plus failed-glTF rollback. Screenshot colour counts repeat
  initial red 33,414 / green 74,514, successful replacement red 0 / green 74,490 /
  blue 69,473; failure retains the initial pair; final unload yields zero geometry
  colour pixels. Their live BVH queries remain checked throughout. This is the
  isolated original-fixture diagnostic, not normal multiplayer chunk activation.
- Actual server/client executables reject a query-only zero-spawn package before
  session/GPU startup (`startup-rejection.json` and captured logs). Explicit-map
  launcher interrupt/child-failure cleanup also passes.
- After promotion, `build/general-collision-default-native/launcher/verification.json`
  passes both real launcher cases with `--world` omitted. Both verify
  `theia.development.lordaeron-town` **revision 2**, default Renet, removal of all
  three owned children and ticket cleanup. The normal path is now
  `build/development-world-town-v3/`; profile identity is
  `bcbc99cd36132ca2ca38ca4fbd524668d46ccc9debe9096ecfac28360c496bb4`.

Active Vulkan validation is recorded for all render cases; no VUID/renderer errors
or runtime panics occur in accepted final records. The intentionally rejected glTF
case reports only its expected asset failure. Native Windows and cross-builds
remain unqualified. Source-model collision fidelity, moving-actor chunk activation,
normal residency/streaming, prediction and jump animation remain open. Imported
map geometry remains its previous boxes/authored floors; the independent original
GLB mesh fixture establishes the general authoring/runtime path.

## Current-actor chunk activation — 2026-10-20

P1-09/P1-10/P2-06/P2-07 bounded authority continuation; see
[contract and next adoption gate](CHUNK_ACTIVATION.md). Linux CPU/headless scope.

- `cargo fmt --check`, `git diff --check` and locked denied-warning workspace/
  all-target Clippy pass (`build/chunk-activation-clippy-final.log`).
- Locked workspace tests pass: 116 tests and one explicitly ignored private-map
  test (`build/chunk-activation-workspace-tests.log`). The final edited chunk
  target passes all six tests (`build/chunk-activation-tests-final.log`), including
  its added identity/frame case: 117 distinct passing tests across current targets.
- Actual Avian cases cover preparation/publication races, full capsule footprint
  retention, body intrusion, grounded and airborne support loss, world/generation
  mismatch, abandoned/corrupt candidates and trusted spawn protection. Boundary
  movement preserves normalized diagonal speed, jump/gravity and input sequence
  progress at 20/30/60 Hz and signed billion-metre origins, with zero recoveries.
- The actual Renet runtime test uses two authenticated UDP clients, a distant
  origin, boundary waiting/jumping, worker preparation and live publication,
  concurrent management backpressure, rejected spawn unload/corrupt load and
  continued replicated movement/acknowledgements. Detailed stats are captured in
  `build/chunk-activation-renet-evidence.log`.
- No renderer source changes or new native/GPU qualification are part of this
  slice. Earlier RX 7900 XTX/RADV native records remain the qualified presentation
  baseline. Paired normal client/authority chunk adoption, imported tile
  partitioning, broader streaming/memory/latency budgets and Windows execution
  remain unqualified. Existing 30 Hz/default monolithic town selection is retained.

## Normal chunk startup and bounded common frames — 2026-10-20

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08 continuation. See
[shared startup and adoption limits](CHUNK_STARTUP.md) and
[current-actor activation](CHUNK_ACTIVATION.md).

- Fresh `cargo fmt --check`, `git diff --check`, locked workspace build and
  denied-warning workspace/all-target Clippy pass. Current logs:
  `build/chunk-wide-frame-build.log`, `build/chunk-wide-frame-clippy.log`.
- A frozen-source locked full workspace run passes **123 tests**, with one private
  prepared-map route test explicitly ignored:
  `build/chunk-wide-frame-workspace-tests.log`. The raw earlier
  `build/chunk-client-startup-workspace-tests.log` is discarded: it combined a
  changed admission test with the previous library during an in-flight build.
  It is not acceptance evidence. The subsequent coherent 120-test session run
  predates the additional wide-frame fix and is a historical checkpoint only.
- Python: 42 asset tests (`build/chunk-startup-python-assets.log`) and four GLB
  alignment tests (`build/chunk-client-startup-python-glb.log`) pass. The actual
  generator-to-Rust original fixture startup/load/unload also passes
  (`build/chunk-startup-interop.log`). Its earlier fingerprint output predates
  the version-2 initial-set binding; current native checks independently encode
  and verify that binding.
- The new source tests cover shared bounded startup, initial-set mismatch over
  real secure Renet, current actor activation, corrupt/stale candidate rollback,
  conservative boundary movement/normalized speed/jump/gravity, separate query
  frames, global margin and distant joins/departures. The pre-fix distant-actor
  snapshot panic is captured in `build/chunk-wide-frame-reproduction.log`;
  the fixed nine-case target passes in `build/chunk-wide-frame-tests.log` and
  the full workspace run. Client tests preserve required static anchors and
  previous presentation/interpolation endpoints through wide moves/recovery.
- Seven **actual two-client native cases**, all using the same rebuilt server and
  client binaries, pass on **AMD Radeon RX 7900 XTX, RADV NAVI31, Vulkan**. Khronos
  layer activation is present in loader traces, with WGPU debug/validation enabled
  and no VUID/validation errors. Current roots:
  `build/chunk-wide-frames-native-20261020/` (5.1 km separated actors),
  `build/chunk-wide-frames-regression-native-20261020/` (origin/positive/negative
  chunk startup, graybox and normal town route), and
  `build/chunk-wide-frames-edges-native-20261020/` (separate query frames and
  actual Bevy rejection of consistently bound invalid initial GLB).
- `build/chunk-wide-frame-native-summary.json` records the fourteen actual client
  reports, common binary hashes, live frame data and device. Every local actor is
  visible; maximum live ground agreement error is `9.5367431640625e-7` m. The
  ordinary cases require both actor colours; the separated case requires actual
  authoritative separation over 4,096 m, local actor pixels and correct remote
  entity projection, with no claim of distant offscreen pixels. Its actors are
  5,100 m apart and both clients/server retain common origins. Screenshots were
  inspected. Native ticket files and owned child processes are cleaned up.
- Normal executable chunk loading is **initial-only**. The launcher remains the
  qualified monolithic town; 30 Hz remains default. Dynamic paired readiness/
  replacement, geometry contributor ownership, actual tile partitioning, broader
  memory/load/streaming budgets, Windows native execution and MMO capacity remain
  separate gates. The source architecture/task/provenance files retain their
  original SHA-256 digests; all changes remain local.

## Initial per-client chunk readiness — 2026-10-20

Resumed the paused wire/ALPN-8 work and qualified the existing source without
changing runtime code during the checks. [Contract](CHUNK_READINESS.md).

- `cargo fmt --check`, `git diff --check` and denied-warning locked workspace
  all-target Clippy pass (`build/chunk-readiness-resume-clippy.log`). The paused
  final workspace build passed in `build/chunk-readiness-build.log`.
- Fresh locked full workspace tests pass: **126 passed, zero failed, one ignored**
  private-map test (`build/chunk-readiness-workspace-tests.log`). Coverage includes
  20/30/60 Hz readiness boundaries, independent peer release after authority
  replacement, empty withdrawal, fresh actor reset, epoch/unknown-ID sequence
  poisoning and malformed actor addressing over actual secure UDP. The full
  scheduler and impaired-transport regressions pass too.
- Fresh Python suites pass **42 asset tests plus four GLB tests** in
  `build/chunk-readiness-python-assets.log` and `...-python-glb.log`.
- **Eight actual two-client native cases** pass on **AMD Radeon RX 7900 XTX,
  RADV NAVI31, Vulkan**, with active Khronos validation and WGPU debug/validation:
  initial chunk worlds at ordinary and signed billion-metre coordinates, Renet
  graybox, normal town route, Quinn graybox, independent client query frames and
  actors separated by 5.1 km. Roots are `build/chunk-readiness-regression-native-20261020/`,
  `build/chunk-readiness-edges-native-20261020/` and
  `build/chunk-readiness-wide-native-20261020/`. Existing verifier checks include
  actual server acceptance of both client reports, scene roots/live BVH agreement,
  bounded frames and local actor pixels. The separated case deliberately checks
  remote entity projection without requiring offscreen remote pixels.
- `build/chunk-readiness-native-summary.json` verifies all sixteen reports used
  the same current client/server binary hashes and records the exact device.
  A consistently hashed invalid initial GLB exits with error, never publishes
  readiness and produces no ready state. Ordinary original-fixture and town
  screenshots were inspected. Owned processes and temporary tickets are cleaned
  up by the qualifier.
- Normal rendered chunk loading remains **initial-only**. The UDP replacement
  test uses headless readiness reports; it is not a rendered replacement proof.
  Acknowledged withdrawal, retained presentation, paired dynamic publication,
  geometry contributor ownership/tile partitioning and broader budgets remain
  open. The launcher remains monolithic, 30 Hz remains default, and native
  Windows/MMO capacity remain unqualified. Imported source MD hashes are unchanged.
  All changes remain local.

## Acknowledged chunk readiness — 2026-10-20

P1-06/P1-08/P2-06/P2-07 continuation, [contract](CHUNK_READINESS.md).
Wire/ALPN **9** now returns an exact epoch/sequence/set acknowledgement after a
fixed simulation step. The client confirmation barrier additionally requires a
snapshot at or after its application tick. Authority retains one coalesced pending
reply per peer when reliable-channel capacity is unavailable.

- Fresh locked workspace build, fmt, diff checks and denied-warning all-target
  Clippy pass (`build/chunk-ack-final-build.log`, `...-final-clippy.log`). Fresh
  full workspace tests pass **129 tests, zero failed, one private-map test ignored**
  (`build/chunk-ack-final-workspace-tests.log`). Python suites pass **42 asset plus
  four GLB tests** (`build/chunk-ack-python-assets.log`, `...-python-glb.log`).
- New unit cases reject malformed acknowledgements and prove that an early ack,
  stale epoch, mismatched set or A-to-other-to-A report cycle cannot release the
  barrier before the correct application snapshot. Projection preserves it.
  Real secure UDP tests prove empty withdrawal holds only its actor, input/other
  peer progress continues, regrant requires a fresh acknowledgement, and the
  latest valid burst report is acknowledged despite replay/unknown-ID attempts.
  Fresh final runtime coverage has ten tests; movement recoveries remain zero in
  the new withdrawal/regrant case.
- **Nine actual two-client native cases**, eighteen actual client reports, pass
  on **AMD Radeon RX 7900 XTX / RADV NAVI31 / Vulkan**, with active Khronos
  validation and WGPU debug/validation: origin and signed-billion-metre initial
  chunk worlds, Renet graybox, two normal town routes, Quinn graybox, independent
  client query frames, and actors separated by 5.1 km. The verifier independently
  checks the ack's version/epoch/sequence/set, application tick against the actual
  snapshot, and the client barrier, alongside existing root/live-BVH/camera/
  presentation checks. Roots: `build/chunk-ack-final-regression-native-20261020/`,
  `build/chunk-ack-edges-native-20261020/`, `build/chunk-ack-wide-native-20261020/`.
- `build/chunk-ack-native-summary.json` verifies all eighteen reports used the
  same current client/server binary hashes and records the device. Actual Bevy
  rejection of a consistently bound invalid initial GLB also passes, with no
  readiness publication. Town and wide-case screenshots were inspected; owned
  processes and temporary tickets are cleaned up.
- The first town run failed its existing 0.3 m sideways wall-contact bound:
  source-space x was 3.6920676848814966 versus the intended 4.0. That run is
  **not acceptance evidence** (`build/chunk-ack-regression-native-20261020/REJECTED.md`).
  Its smoke driver aimed far behind the wall, diluting tangential correction.
  The driver now corrects against the approach line while pressing into the wall,
  using ordinary input only. Collision, route geometry and verifier bounds are
  unchanged. Two rebuilt-binary town runs pass at source-space x
  4.20125140737567 and 4.186150073681556, respectively; final acceptance uses only
  the new binary set above.
- This qualifies the **authority acknowledgement barrier**, not actual dynamic
  client retirement. Normal rendering remains initial-only. Retained local/remote
  presentation, recovery-spawn coverage, gravity-driven horizontal boundary
  behavior, paired publication, geometry contributor ownership and tile
  partitioning remain open. Default launcher and 30 Hz are unchanged. Native
  Windows and MMO capacity remain unqualified. Imported source MD hashes are
  unchanged; changes remain local.

## Continuous chunk movement boundaries and support fallback — 2026-10-20

P1-09/P2-06/P2-07, [contract](CHUNK_BOUNDARIES.md). Wire/ALPN remains 9.

- The original stationary steep-slide reproduction crossed into client-unready
  collision at 20 Hz, x=127.67024 m (`build/chunk-gravity-boundary-reproduction.log`).
  Bounded coverage now participates in capsule sweeps, including vertical-motion
  projections and step-up attempts. Shared seams remain open; outer edges,
  concave missing corners and disconnected gaps are constrained continuously.
- Final `cargo fmt --check`, `git diff --check`, locked workspace build and
  denied-warning all-target Clippy pass (`build/chunk-boundary-fallback-build.log`,
  `...-fallback-clippy.log`). Fresh full workspace tests pass **134 tests, zero
  failures, one ignored private-map case**, in
  `build/chunk-boundary-fallback-workspace-tests.log`. Fresh Python suites pass
  **42 asset plus four GLB tests** (`...-fallback-python-assets.log`,
  `...-fallback-python-glb.log`).
- Gravity coverage tests exercise 27 combinations: origin/signed billion-metre
  frames, 20/30/60 Hz and missing authority collision/client-unready/full
  withdrawal. Free vertical fall and gravity state advance while XZ is blocked;
  actual loading/readiness release resumes sliding to supported ground. The
  corner test drives real Avian movement in nine frame/rate combinations and
  checks full footprint coverage on every step. Additional tests reject invalid
  coverage and sweep over a disconnected gap despite covered endpoints.
- The separately invoked private-map case passes both prepared town and farmhouse
  routes at **20/30/60 Hz**, all six stages with **zero recoveries**
  (`build/chunk-boundary-fallback-private-map-routes.log`). These retain their
  existing proxy/floor geometry, not new source-model collision fidelity.
- **Ten actual two-client native cases** pass on **AMD Radeon RX 7900 XTX,
  RADV NAVI31, Vulkan**, with active Khronos validation and WGPU debug/validation:
  ordinary/signed distant initial chunk worlds, actual loading-edge pressure,
  Renet graybox, town and farmhouse routes, Quinn graybox, independent client
  query frames and actors separated by 5.1 km. Current roots:
  `build/chunk-boundary-fallback-final-regression-native-20261020/`,
  `build/chunk-boundary-fallback-edges-native-20261020/`, and
  `build/chunk-boundary-fallback-wide-native-20261020/`.
- `build/chunk-boundary-fallback-native-summary.json` verifies all twenty reports
  used the same current client/server binary hashes. Maximum live ground agreement
  error is `9.5367431640625e-7` m. The edge case holds the leading actor at
  x=`127.64449310302734` m while applied input advances to sequence 183; actual
  rendered poses and live Avian queries agree. Invalid initial GLB rejection
  also passes without readiness publication. Edge/farmhouse screenshots were
  inspected, and owned processes/temporary tickets are cleaned up.
- The new corner regression exposed a large-convex-floor CCD/contact precision
  failure. Capsule-only validation was insufficient; a sole-first variant fixed
  support but changed exact persistent-contact results by sub-millimetres.
  Final source keeps ordinary capsule-first support and uses standard Avian sole
  validation/fallback for invalid candidates. The large-floor and unchanged exact
  Quinn door/uneven-terrain contact assertions all pass. Earlier failed workspace
  logs `build/chunk-boundary-workspace-tests.log` and
  `build/chunk-boundary-support-workspace-tests.log` are **not acceptance evidence**;
  corresponding native roots have `REJECTED.md` markers and are excluded from
  the final summary above. The diagnostic contact trace is retained in
  `build/large-floor-support-reproduction.log`.
- Normal executable rendering remains **initial-only**. The rendered edge case
  holds at its initial boundary; real rendered replacement/release is still open.
  Retained local/remote poses and recovery coverage, remote presentation while
  assets lag, paired publication, geometry contributors and tile partitioning
  remain next gates. Default launcher and 30 Hz are unchanged; native Windows,
  broad streaming budgets and MMO capacity remain unqualified. Imported source
  MD hashes are unchanged. All changes remain local.

## Bounded paired normal-client handover and wall-contact qualification — 2026-10-20

P1-06/P1-08/P1-09/P2-06/P2-07/P2-08, execution-order row 5, remain partial.
[Implemented behavior and adoption limits](CHUNK_HANDOVER.md). Wire/ALPN 10 adds
bounded authority offers and tagged readiness acknowledgements. Normal opt-in
clients now stage standard Bevy roots, protect local/remote/recovery coverage,
await the exact retirement barrier and publish matching actual Avian generations.
The normal town launcher remains monolithic.

- `cargo fmt --all --check`, denied-warning workspace/all-target Clippy and the
  locked client/server build pass. SDK environment is the extracted native SDK
  described above; Clippy/build logs are
  `build/chunk-pair-contact-qualified-clippy.log` and `...-build.log`.
- `cargo test --locked --workspace -j 1` passes **140 tests**, zero failures,
  with one private-content test explicitly ignored (`...-workspace-tests.log`).
  That private test is separately executed with
  `THEIA_MAP_CONTROLLER_FIXTURES=build/development-world-town-v3:build/development-world-access-v3`;
  both house routes pass at 20/30/60 Hz, six stages each, zero recoveries
  (`...-private-map-routes.log`). All commands use the locked dependencies.
- New checks exercise bounded offer envelopes, retained presentation/landing
  validation without mutation, idle versus falling/uphill chunk demand, actual
  encrypted two-client automatic activation with independent readiness, and
  zero-delta live Avian refresh after replacement. Existing movement, network
  impairment/session and map regressions remain intact. The unchanged Python
  suites pass 42 asset tests and four GLB tests in
  `build/chunk-pair-qualified-python-assets.log` and `...-python-glb.log`.
- The first wider workspace run is **rejected**:
  `build/chunk-pair-qualified-workspace-tests.log` exposed intermittent exact wall
  drift on sloped terrain. A new deterministic test reproduces it over approach
  positions at 20/30/60 Hz. Near-head-on Avian feature-manifold distance/normal
  refinement fixes the drift without loosening the exact assertions. Two-second
  retained contacts, grounding, minimum intentional tangential input and retreat
  pass, alongside unchanged diagonal-sliding and capsule/sole support checks.
  Earlier single-contact-normal variants are rejected because they blocked
  diagonal motion. Final evidence below uses the rebuilt corrected solver.

**Thirteen actual two-client native cases pass**, 26 client reports, plus actual
invalid-initial-GLB rejection. All 13 bind the same final client/server binaries
and current Cargo.lock hashes. The summary rechecks those hashes and active
validation: `build/chunk-pair-contact-qualified-native-summary.json`.
Native Vulkan execution uses AMD Radeon RX 7900 XTX / RADV NAVI31 and active
`VK_LAYER_KHRONOS_validation`, `WGPU_VALIDATION=1`, `WGPU_DEBUG=1`, with owned
private Wayland displays. This is Linux native execution, not Windows evidence.

- `build/chunk-paired-contact-qualified-native-20261020/`: delayed preparation
  (2.5 seconds on one client), consistently bound invalid replacement GLB, and
  negative distant coordinates with independent client/server query frames.
  Successful replacements preserve the retained hierarchy, use one staged root,
  acknowledge withdrawal before publication and confirm full readiness only with
  matching active/probed physics generations. The delayed client hides uncovered
  remote presentation while the ready peer crosses the seam. Screenshots and
  live loaded/removed column queries prove floor replacement and failed-candidate
  retention. Both failed clients keep generation 1 while rejecting the offered
  replacement; this is not asymmetric-failure coverage.
- `build/chunk-pair-contact-qualified-final-regression-native-20261020/`: initial
  origin/positive/negative chunk worlds, persistent loading-boundary pressure,
  graybox, town, farmhouse and explicit Quinn. Both actual house-access routes
  complete all six stages; ordinary animation/camera/query checks remain active.
- `build/chunk-pair-contact-qualified-edges-native-20261020/`: independent distant
  query frames plus a real Bevy consistently hashed invalid initial GLB. Startup
  fails cleanly without readiness; it is separate from failed replacement.
- `build/chunk-pair-contact-qualified-wide-native-20261020/`: two players 5.1 km
  apart with bounded common wire/render frames and retained static roots.

Successful runs have no unexpected renderer errors, panics or Vulkan validation
errors. The replacement-failure verifier permits exactly its expected candidate
GLTF loader error; it still rejects unrelated errors and all VUID/validation
failures. The first prototype verifier rejection of this expected error remains
rejected in `build/chunk-paired-native-20261020/REJECTED.md`. Inspected final
screenshots show replacement floor, rollback edge and the retained town interior.
The four imported engineering source documents retain their original SHA-256s.

Native scenarios use 30 Hz; multi-rate CPU/authority checks above cover 20/30/60 Hz.
No native Windows, crowd/soak or MMO capacity claim follows. General contributor
geometry crossing owner boundaries is not guaranteed by the schema-1 index.
Actual imported tile partitioning, shared asset/material budgets, native offer
reversal/supersession and asymmetric failure remain open before normal streaming
adoption. All source changes stay local; no commit or push was made.

## Contributor-complete static geometry — 2026-10-20

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08, execution-order row 5, remain partial.
[Index contract and limits](CHUNK_CONTRIBUTORS.md). Schema 2 binds combined local
XZ bounds into dataset identity, derives cell contributors and gates coverage,
demand, readiness, protected actors/recovery and retained presentation on complete
provider sets. Runtime checks actual colliders and transformed standard-Bevy mesh
bounds; ordinary GLB roots/renderer/Avian remain the implementation path. Wire/
ALPN stays 10; mismatched extent contracts change admission identity.

- `cargo fmt --all --check`, denied-warning workspace/all-target Clippy and locked
  client/server build pass. Logs: `build/chunk-contributors-final-clippy.log` and
  `...-build.log`, using the extracted native SDK environment described above.
- `cargo test --locked --workspace -j 1` passes **144 tests**, zero failures,
  one private-map test ignored (`...-workspace-tests.log`). The ignored test is
  separately run with both prepared map paths in
  `THEIA_MAP_CONTROLLER_FIXTURES`: town and farmhouse pass six stages each at
  20/30/60 Hz, zero recoveries (`...-private-map-routes.log`).
- New actual Avian cases require a nonlocal wall provider even when the owner
  floor is present, block missing readiness and release motion into the real wall
  from both sides at 20/30/60 Hz through origin and signed billion-metre frames.
  Bounds/order/query-frame identity, demand, recovery/presentation retention,
  incomplete startup, underspecified collider rejection without publication and
  contributor-budget rejection are checked. New client tests check actual nested
  mesh transforms, render-root independence, retained cached bounds after GPU
  extraction, morph rejection, invalid transforms and hierarchy budgets.
- Python suites pass **47 asset tests plus four GLB tests** in
  `build/chunk-contributors-python-assets.log` and `...-python-glb.log`.
  The authoring tool verifies immutable bindings, derives actual positions and
  transformed geometry, preserves source bytes, rejects bad/over-budget inputs
  and writes only a new index. Tests reproduce the pinned Trimesh `world`-node
  transform loss and ensure scratch renaming preserves real root transforms;
  false accessor extrema cannot hide geometry. No source package is rewritten.

**Fifteen actual two-client native cases pass**, 30 client reports, plus two
separate startup rejection cases. All 15 bind the same final binaries and current
Cargo.lock hashes; the summary checks them against the current files:
`build/chunk-contributors-qualified-native-summary.json`. Native execution uses
AMD Radeon RX 7900 XTX / RADV NAVI31 Vulkan with active
`VK_LAYER_KHRONOS_validation`, `WGPU_VALIDATION=1`, `WGPU_DEBUG=1`, on owned private
Wayland displays. Maximum live client ground-query error is
`9.5367431640625e-7` metres. This is Linux native evidence only.

- `build/chunk-contributors-final-native-20261020/`: schema-2 original geometry
  has a visible/collidable provider wall extending from its owner into the
  diagonal neighbour. Two clients publish the added neighbour while retaining
  both required roots, including a 2.5-second delayed peer and independent client
  query frame. Both retain real matching physics/readiness generations; the
  delayed client withholds uncovered remote presentation. A valid bound GLB with
  geometry outside the declared replacement extent is rejected after hierarchy
  staging: both clients retain their original two roots/generation, record one
  rejection and zero publications. A separate valid initial GLB with an
  underspecified extent exits with code 1 before readiness. The only error line
  there is its expected contributor-validation diagnostic; no VUID/panic occurs.
  Final screenshots were inspected and show the nonlocal wall and added floor.
- `build/chunk-contributors-paired-regression-native-20261020/`: all three
  existing delayed/invalid-GLB/negative-independent-frame handover cases pass.
- `build/chunk-contributors-qualified-final-regression-native-20261020/`: all
  eight origin/positive/negative/loading-boundary/graybox/town/farmhouse/explicit
  Quinn regressions pass; actual house access completes all six stages.
- `build/chunk-contributors-qualified-edges-native-20261020/`: separate query
  frames and real invalid-initial-GLB rejection before readiness pass.
- `build/chunk-contributors-qualified-wide-native-20261020/`: players 5.1 km
  apart retain bounded common frames/static roots. All successful scenarios reject
  unexpected renderer errors, panics and Vulkan validation diagnostics. Expected
  deliberate failed-asset diagnostics retain the previous narrow verifier rule.

The four imported engineering source SHA-256s remain unchanged. No commit/push was
made. Native runs use 30 Hz; multi-rate headless checks cover 20/30/60 Hz. Whole
phase/task completion, source-model collision fidelity, Windows, crowds/soak and
production streaming remain unproven. Conservative envelopes establish complete
contributors, not visual/collision mesh equality or imported partitioning.

Read-only town inventory in
`build/chunk-contributors-town-partition-inventory.json` records a 704,123,496-byte
GLB with 698,655,860 bytes in 491 embedded PNG image views (about 99.2%). It has
4,266 nodes, 756 meshes/primitives, no animations/skins/extensions; collision has
65,871 triangles and 3,972 conservative solids. PNG headers imply 1,524,977,664
base-level RGBA8 bytes, excluding mipmaps and driver overhead; this is an estimate,
not a GPU-memory measurement. Bind shared immutable images before actual terrain/
collision partitioning, preserving standard Bevy loading and measuring real
resident/staging/load budgets. The default town stays monolithic until that gate
passes; native reversal/supersession/asymmetric failure checks remain open.

## Shared immutable PNG dependencies and stable lookahead — 2026-10-20

P1-10/P2-06/P2-08 and the row-5 loading continuation remain partial.
[Manifest-v2 contract, budgets and scope](SHARED_IMAGES.md). Shared descriptors
bind image bytes/hash, colour space and glTF sampler settings into canonical
package identity. Normal prepared chunks use standard Bevy GLB/PBR/image loading
with editor metadata disabled; no renderer or image-loader replacement was added.
The headless collision path reads no PNGs, while admission binds their identity.

- Fresh `cargo fmt --all --check`, denied-warning workspace/all-target Clippy and
  locked client/server build pass. Logs:
  `build/shared-images-policy-final-clippy.log` and `...-build.log`, with the
  extracted native SDK environment described above.
- `cargo test --locked --workspace -j 1` passes **151 tests**, zero failures,
  **two explicitly ignored private-data tests**
  (`build/shared-images-policy-final-workspace-tests.log`). Both are separately
  executed: both prepared house routes complete all six stages at 20/30/60 Hz
  without recoveries (`...-private-map-routes.log`), and
  `THEIA_SHARED_WORLD_FIXTURE=build/shared-images-town-stage/town` verifies the
  actual staged manifest, collision, GLB and all 488 shared image dependencies
  (`...-private-town-validation.log`).
- Shared-resource tests cover a Python/Rust canonical golden identity, resource
  ordering, settings/byte sensitivity, duplicate/invalid/over-budget declarations,
  missing/corrupt files, URI/variant disagreement, unsafe aliases/paths and legacy
  downgrade rejection. Correctly hashed oversized/unsupported PNG headers fail
  before GPU loading. Headless collision loading still succeeds without images.
- Python suites pass **52 asset plus four GLB tests** in
  `build/shared-images-python-assets.log` and `...-python-glb.log`. New tests
  compare actual geometry accessor payloads, transforms, materials and PNG bytes,
  shared compatible paths across packages, split colour contexts, corruption,
  immutable revisions and source preservation. Image data is removed from the
  new embedded buffer rather than left as unused payload.

**Eighteen actual two-client native cases pass**, 36 client reports, plus both
retained startup rejection cases. All 18 bind the same final client/server and
current Cargo.lock hashes. The summary rechecks those against the current files:
`build/shared-images-policy-regression-native-summary.json`. Native Linux Vulkan
runs use AMD Radeon RX 7900 XTX / RADV NAVI31, active
`VK_LAYER_KHRONOS_validation`, `WGPU_VALIDATION=1`, `WGPU_DEBUG=1` and owned private
Wayland displays. Maximum live client ground-query error is
`9.5367431640625e-7` metres. Native runs use 30 Hz; the multi-rate authority checks
above cover 20/30/60 Hz. This provides no Windows or capacity evidence.

- `build/shared-images-policy-qualified-native-20261020/`: three original
  textured chunk scenarios. The delayed/independent-query-frame case shares one
  actual sRGB `Image` handle across two retained/new roots while the normal-map
  use has a separate linear handle (`Rgba8UnormSrgb` versus `Rgba8Unorm`). The
  sampler case retains different handles and actual repeat/linear versus clamp/
  nearest settings for the same PNG bytes. Invalid `.png.meta` files are ignored
  under the prepared-root policy. The corrupt-new-linear-image case preserves
  both old roots and one image handle, records one rejection/zero publications,
  and never grants the failed replacement. Actual material paths, handles,
  formats, dimensions and samplers match bound GLB references. Live generation,
  column-query, camera, two-player/pixel and validation checks stay enabled.
  The final sharing screenshot was visually inspected.
- `build/shared-images-policy-paired-regression-native-20261020/`: all three
  delayed/invalid-GLB/negative-frame paired handover cases pass.
- `build/shared-images-policy-contributors-regression-native-20261020/`: both
  nonlocal contributor publication and valid-GLB extent rollback cases pass,
  along with initial extent rejection before readiness.
- `build/shared-images-policy-regression-final-regression-native-20261020/`:
  all eight initial origin/positive/negative/boundary/graybox/town/farmhouse/Quinn
  cases pass, including all six actual house-access stages.
- `build/shared-images-policy-regression-edges-native-20261020/` and
  `...-wide-native-20261020/`: separate query frames, invalid initial GLB rejection
  and players 5.1 km apart with bounded common frames/static roots pass.
  Successful scenes reject unexpected renderer errors, panics and VUID/validation
  diagnostics. Earlier intentional GLB failure exceptions stay narrowly scoped;
  PNG hash rejection is an expected preparation warning, not a renderer exception.

Rejected evidence is retained rather than counted as success. The first sampler
verifier incorrectly required two compatible final roots after one had retired
(`build/shared-images-native-20261020/REJECTED.md`); exact per-binding checks remain
and the sharing fixture explicitly requires cross-root reuse. A second corruption
fixture wrote through a hard link, corrupting the retained variant too; clients
correctly rejected startup rather than exercising replacement rollback
(`build/shared-images-qualified-native-20261020/CORRUPT_FIXTURE_REJECTED.md`).
The final fixture replaces only the new variant before corruption.

The retained delayed-peer regression then exposed a genuine unload-only offer
churn case (`build/shared-images-paired-regression-native-20261020/REJECTED.md`).
A trailing peer moving within the current cell caused an intermediate unload
before the leading peer requested the neighbour; the delayed client stayed safe
but prepared twice and was still pending at capture. A deterministic test fails
against that demand (`build/shared-images-prefetch-churn-reproduction.log`). The
revised bounded policy retains spares until a new load requires them; signed-frame
and secure UDP arrival-order tests pass. Final evidence above uses the rebuilt
policy, not earlier image-only binaries. Explicit unload APIs are unchanged.

The actual town source remains unchanged. Offline extraction stages revision 3
in `build/shared-images-town-stage/town`: GLB bytes fall from 704,123,496 to
5,476,980, while 488 unique shared PNG resources retain 698,655,860 encoded bytes.
This is placement/deduplication, not compression of the map to 5.48 MB.
`build/shared-images-town-byte-comparison.json` confirms all 2,399 accessor
payloads, node/mesh/material/sampler/scene and collision semantics, and each of
491 texture entries' original PNG bytes. Unique PNG headers imply 1,514,491,904
base-level RGBA8 bytes, excluding mipmaps/driver overhead; no GPU residency
measurement is inferred. The Rust private-data check verifies the real staged
resource contract with the final code. Extraction output is in
`build/shared-images-town-extraction.log`.

The default remains revision-2 monolithic town at 30 Hz. Actual graph pruning,
terrain/collision partitioning, source-model collision fidelity, measured loading/
resident/staging budgets, full reversal/supersession/asymmetric failure, Windows
and MMO-scale behavior remain open. The four imported source-document hashes are
unchanged. All changes remain local; no commit or push was made.

## Offline real-world partition qualification — 2026-10-20

P1-10/P2-06/P2-07/P2-08, execution-order row 5. This is an offline actual-town
partition slice plus original-geometry runtime format qualification. It does not
qualify loading the actual town as chunks or change the default launcher.
[Contract, supported layouts and adoption gate](WORLD_PARTITIONING.md).

- `build/world-partition-python-assets.log`: **61 asset tests pass**;
  `build/world-partition-python-glb.log`: **four GLB tests pass**. Nine asset
  tests extend the preceding 52-test checkpoint. They reconstruct transforms,
  compare exact model accessor/index bytes and pruned image graphs, independently
  fit height/UV planes, conserve area/winding, test signed distant coordinates,
  immutable sources/new output, unsupported transforms/layouts and bad bindings.
  The independent oracles reject equal-area translated/reversed collision and
  changed UVs. A failed verification removes a previous proof result.
- `build/world-partition-fmt.log` and `build/world-partition-clippy.log`: fresh
  `cargo fmt --check` and locked denied-warning workspace/all-target Clippy pass.
  Rust source/binaries are unchanged from the preceding 151-pass/two-private-test
  checkpoint. That full suite was not rerun for these offline-only tools.
- `build/world-partition-town-verification.log` and
  `build/partitioned-town-qualified-stage/partition-verification.json`: actual
  source comparison passes all 2,532 model mesh instances, 256 terrain groups,
  exact model geometry/index payloads, hierarchy transforms and semantic material
  image/sampler bindings. Terrain source height/UV/area and complete oriented
  face checks pass. All 72,047 output collision faces lie on oriented source
  faces; total collision area difference is zero. All 3,972 movement plus four
  camera boxes occur exactly once in their original positions. Maximum terrain
  group area error is 0.00023984098743312643 square metres. This verifies existing
  source collision; it does not improve conservative model proxy fidelity.
- `build/world-partition-town-rust-packages.log`: the existing ignored
  `staged_shared_world_verifies_actual_manifest_collision_and_all_image_dependencies`
  test is executed separately for every output package using
  `THEIA_SHARED_WORLD_FIXTURE`. **All 26 actual packages pass** real Rust
  `Terrain::load`/manifest/collision/visual and shared PNG verification. These are
  26 executions of one existing test, not 26 new automated test definitions.
- `build/world-partition-native-20261020/proof/verification.json` and
  `build/world-partition-distant-native-20261020/proof/verification.json`: **two
  actual server/two-client cases pass**, four reports/screenshots, with AMD Radeon
  RX 7900 XTX / RADV NAVI31 Vulkan, active `VK_LAYER_KHRONOS_validation`,
  `WGPU_VALIDATION=1` and `WGPU_DEBUG=1` on owned private Wayland displays.
  The generated source has a sloped textured ground spanning three cells and
  whole static model boxes, including one crossing an ownership seam. Real output
  is pruned into packages; each case publishes one candidate after exact
  withdrawal acknowledgement while retaining the old root and shared image
  handle. Both end with two packages and matching live Avian generation/queries,
  correct two-pawn pixels, camera, mesh clearance and no unexpected recoveries.
  Peer B delays preparation 2.5 seconds and records 80 hidden-remote frames;
  ready peer A crosses while B remains behind its old coverage. The second case
  places content at global X=-1e9/Z=+1e9 with a separately shifted client query
  frame; render and replication span are both 2 metres. The ordinary screenshot
  was visually inspected. Logs: `build/world-partition-native.log` and
  `build/world-partition-distant-native.log`. No Windows execution is claimed.

An early independent verifier run rejected a valid terrain point because its
barycentric first weight omitted the second coordinate. The oracle was corrected
to `1-u-v`; explicit interior interpolation and deliberate UV corruption now
exercise it. No source geometry was changed or tolerance relaxed for that error.
Final actual source comparisons above use the corrected and strengthened oracle.

The town's 26 packages total 8,470,060 visual bytes and 12,805,430 collision bytes;
per-package textures peak at 139 versus 491 source entries. Shared PNG bytes are
unchanged. Current initial packages reference 377 unique variants / 474,111,320
encoded bytes; this is not resident CPU/GPU memory measurement. Contributor
closure requires up to nine providers per cell and ten for the current spawns.
The maximum union of spawn providers and one arbitrary cell is 16; actor/lookahead/
interpolation/staging unions still need separate measurement. Consequently only
`world.chunks.candidate.json` is written for the actual town; the runtime budget
is still four and the default remains monolithic revision 2 at 30 Hz.

All four imported engineering documents retain their original hashes. Private
assets and all generated packages, logs/screenshots and build outputs remain
ignored. All changes remain local. Actual-town runtime budgets/publication,
world-edge policy, remaining handover lifecycle checks, broader load/soak,
Windows and MMO capacity remain open. No whole task or phase is complete.

## Conservative content-budget analysis — 2026-10-20

P1-03/P1-10/P2-06/P2-07/P2-08, execution-order row 5. The offline budget
prerequisite now has executable evidence. No runtime cap, protocol, renderer,
world package or launcher changes are made by this slice.
[Measured results and proposed runtime gate](CHUNK_BUDGETS.md).

- `build/chunk-budget-python-assets.log`: **64 asset tests pass**;
  `build/chunk-budget-python-glb.log`: **four GLB tests pass**. Three new tests
  independently exercise separated actor unions, a capsule corner requiring
  four cell closures plus a nonlocal provider, real graph image deduplication
  and static collider counts. Source spawn reconstruction rejects falsified
  recovery sets. Incorrect actual geometry envelopes and changed artifact bytes
  reject measurement. The proposed 32-address JSON size estimate fits 1,200 bytes;
  this is not actual runtime encoding or widened-capacity transport evidence.
- `build/chunk-budget-fmt.log` and `build/chunk-budget-clippy.log`: fresh
  `cargo fmt --check` and locked workspace/all-target denied-warning Clippy pass.
  Rust application sources/binaries, content packages and rendering are unchanged
  from the preceding checkpoints. No full Rust or native GPU regression rerun
  was needed for this offline analysis; those earlier records remain historical.
- `build/world-partition-town-budgets-qualified.json` and `.log`: verifies all
  26 indexed package identities, actual GLB/collision bytes, canonical image
  references and actual PNG/header data. Shared variants are deduplicated by
  URI. Real combined geometry bounds are recomputed and compared with candidate
  envelopes. The unchanged source collision digest verifies all four original
  spawns; their full capsule footprints reproduce the ten-package recovery set.
- The finite-XZ breakpoint analysis produces 59 distinct actor-provider sets.
  Including recovery, one capsule can require **18 packages** and two can require
  **22**; the falling-lookahead analysis has the same maxima. Five conservative
  actor positions cover all **26** indexed packages, matching the finite index
  upper bound for the bounded up-to-eight-actor case. This ignores height,
  obstacles, reachable routes and prospective world edges. It is a conservative
  ownership analysis, not a live eight-player or town route qualification.
- All-indexed costs are **4,001 static colliders**, 72,047 triangles, 8,470,060
  GLB bytes, 12,805,430 collision JSON bytes, 488 unique image variants,
  **698,655,860 encoded PNG bytes**, and **1,514,491,904 estimated base RGBA8
  bytes**. Initial recovery costs are 2,783 colliders, 38,597 triangles,
  6,348,412 GLB bytes, 6,962,516 collision bytes, 377 unique variants,
  474,111,320 encoded bytes and 1,092,964,352 base RGBA8 bytes. The latter numbers
  exclude mipmaps, engine allocations, CPU/GPU copies and driver overhead;
  neither resident memory nor live loading latency is measured here. Timing
  fields describe warm-cache Python verification only.

The earlier 16-package bound was explicitly recovery plus one whole cell; it
was not a full capsule bound. The new 18-package corner measurement explains why
changing the runtime ceiling merely to 16 would be insufficient for all indexed
XZ footprints. A proposed 32 hard ceiling / 26 town session now has explicit
protocol/address/size, admission identity, finite-world, staging/retirement,
real loading/resource and native acceptance checks. None is implicitly certified
by these offline measurements. Runtime capacity remains four; no active town
chunk metadata or promotion is emitted. All work stays local and generated data
remain ignored. No whole task/phase, Windows or MMO capacity claim follows.

## Bounded 32-package runtime qualification — 2026-10-20

P1-03/P1-06/P1-08/P1-10/P2-06/P2-07/P2-08, execution-order row 5.
[Implemented contract and remaining adoption gate](CHUNK_CAPACITY.md).

- `build/chunk-capacity-workspace-tests.log`: **154 Rust tests pass**, three
  private tests ignored. `...-final-lib-tests.log`: all 27 library tests pass
  after current-version negative-input fixtures replace hardcoded old versions.
  Fresh fmt, denied-warning workspace/all-target Clippy and build pass in
  `...-fmt.log`, `...-clippy.log` and `...-build.log`. Python logs
  `...-python-assets.log` / `...-python-glb.log` pass **66 plus four tests**.
- Actual Rust control encodings round-trip 32 checked addresses and full-width
  epoch/sequence/tick headers within 926/979/955-byte readiness/wrapped-ack/offer
  bounds. The 33rd entry, invalid global addresses and stale versions reject.
  Real encrypted UDP sends full offers/readiness/acks, withdrawal/regrant and
  acknowledged movement at a negative billion-metre origin. Original disk/Avian
  tests load all 32 slots at three signed origins/query frames, preserve the
  unloaded 33rd-cell boundary/gravity and reject an over-budget replacement
  without changing its active generation. A 36-provider envelope fails the cap.
- Three private tests separately pass. `...-town-live-qualified.log` loads ten
  actual town packages, prepares/publishes all 26 against one live actor and
  preserves its exact position, then completes 90 ticks with zero recoveries.
  Single warm-cache samples: 1,525.138 ms initial preparation, 1,563.392 ms candidate
  preparation outside the fixed step, 11.433 ms publication. No latency percentile,
  cold load, rendered town chunks or memory claim follows. The explicit opt-in
  stage is `build/chunk-capacity-town-runtime-stage`, budget 26.
  `...-private-map-routes.log` passes both house routes at 20/30/60 Hz with zero
  recoveries. `...-private-shared-world.log` verifies all 488 source-town image
  dependencies. Prefix for these logs is `build/chunk-capacity`.
- `build/chunk-capacity-qualified-native-20261020/`: **eight actual two-client
  native cases**, 16 reports, use AMD Radeon RX 7900 XTX / RADV NAVI31 Vulkan,
  active `VK_LAYER_KHRONOS_validation`, `WGPU_VALIDATION=1`, `WGPU_DEBUG=1` and owned
  private Wayland displays. Cases are `full-startup` (32 actual standard roots,
  shared compatible image handle), `six-provider-publication` (one old root plus
  six required new roots, delayed peer), `negative-independent` (same at negative
  billion-metre positions/different client query frame), `shared-image-retention`
  (failed new dependency keeps old roots), `graybox`, `town`, `farmhouse` and
  `quinn` (ALPN 11). Both six-provider clients record exactly six staging roots,
  one publication and seven final roots. Delayed peers hide uncovered remote
  actors for 78/92 frames. Exact retirement acknowledgements, retained entities,
  live Avian generation/ground queries, actual image paths/formats/samplers,
  camera and two-player pixels remain checked. No unexpected recoveries or
  validation errors occur. The publication screenshot was visually inspected.
  Main log: `build/chunk-capacity-qualified-native.log`.

An initial private invocation used a relative fixture path from Cargo's test
working directory and failed before loading. The qualified invocation uses an
absolute path. The first native run produced both startup captures but the
verifier still expected version 10; its rejection is retained in
`build/chunk-capacity-native-20261020/REJECTED.md`. Both snapshot and acknowledgement
expectations now require 11; the final eight-case run above is fresh. No runtime
or fixture relaxation was made for that verifier error.

World identity v3 now binds the session budget, and wire/ALPN 11 signals the wider
contract; the index/package identities remain unchanged. All eight final native
proofs bind the same current client/server/Cargo.lock hashes. The actual town
candidate/publisher safety gate and monolithic default are unchanged. Explicit
finite-world bounds, actual-town client/resource qualification and the remaining
handover lifecycle cases precede adoption. No native Windows, full task/phase or
MMO capacity claim is made. All changes and evidence remain local; generated and
private assets stay ignored, and imported engineering source hashes are unchanged.

## Synty sample character conversion/preview — 2026-10-20

P1-10/P2-08/P2-09, user-directed asset branch alongside execution-order row 5.
[Contract and remaining quality/adoption limits](SYNTY_CHARACTER_PREVIEW.md).

- `build/synty-character-python-assets.log`: **71 asset tests pass**;
  `...-python-glb.log`: **four GLB tests pass**. New cases cover explicit Unity
  extraction with gzip extra headers, pathname trailers, source immutability,
  overwrite/traversal/alias/duplicate/missing-payload rejection, quaternion
  shortest-arc/step timing, hierarchy and known inverse-bind skin motion.
- Fresh fmt and denied-warning workspace/all-target Clippy pass in
  `...-fmt.log` / `...-clippy.log`. No Rust application/Cargo files changed for
  this slice; full Rust/native multiplayer suites are the preceding unchanged
  154-test/eight-case checkpoint, not rerun as new asset evidence.
- Seven selected private payloads are extracted under
  `build/synty-locomotion-source`. Original archive SHA-256 and extracted hashes
  remain unchanged. The Unity material's BaseMap/MainTex GUIDs match the selected
  dummy palette; embedded full/reference preview images preserve exact source
  **2048x2048 RGBA pixels**. `build/synty-character-source-palette-check.log`.
- Isolated Python 3.13 / Blender `bpy==5.2.2` tools are installed under `build/`,
  not system/game dependencies. `build/synty-conversion-tools-install.log` and
  `tools/synty-conversion-requirements.txt`. Conversion uses actual FBX sources,
  explicit standard material mapping and standard glTF export. Unity code is
  not executed. The FBX's absent image filename is resolved by the actual Unity
  material palette binding, rather than retaining the missing placeholder.
- `build/synty-character-canonical-stage`: one 50-joint skin, 4,896 triangles,
  10,287 exported vertices and four clips. The source has 2,477 vertices before
  splitting. `character-source-skin.glb` retains all skin influences;
  `character.glb` is the explicitly selected four-weight preview. Both Khronos
  validation reports have **zero errors/warnings**, one unused-UV1 informational
  message. Log: `build/synty-character-canonical-gltf-validation.log`.
- `build/synty-character-final-independent.log` and stage
  `character-verification.json`: independent NumPy glTF sampling, hierarchy,
  inverse-bind and skin comparison passes **175 source frames**. Maximum source
  joint-position difference is 0.000011538925291237988 metres. Exported source
  deformation deltas agree despite different fixed local bone bases. Clip
  durations are Idle 1.766667 s, Walk 1.033333 s, Run 0.7 s, Jump 2.2 s.
- Four-weight reduction affects **42 original vertices**, discarding at most
  0.04528161517368044 aggregate weight before renormalization. Sampled full-to-
  reduced differences are **1.309 / 2.164 / 3.588 / 6.974 mm** for Idle/Walk/Run/
  Jump. **Jump fails the initial 5 mm skin gate**. The default converter rejects
  it; the explicit preview flag permits a measured visual preview and records
  the failed gate. Full weights remain in the reference artifact. No production
  skin or all-times/all-clips accuracy claim follows. Animated bounds also show
  below-floor vertices (Jump reaches about -0.145 m): game-specific ground/root
  alignment, Unity importer/controller semantics and locomotion integration are
  still unqualified.
- `build/synty-character-qualified-native-20261020/`: **four actual native Bevy
  inspector cases** pass Idle/Walk/Run/Jump on AMD Radeon RX 7900 XTX / RADV NAVI31
  Vulkan with active Khronos validation, ready render pipelines and no validation/
  application errors. Each reports one animation player, sampled changing
  transforms, 50 joints, four clips, one actual image/material, UVs and metre
  bounds. Substantial golden source-palette body pixels remain checked. A Walk
  screenshot was visually inspected. Summary log:
  `build/synty-character-qualified-native.log`. All four bind the same final
  preview asset and unchanged actual client binary. These are inspector cases,
  not four multiplayer/gameplay cases; no native Windows evidence follows.

The first nonroot skin export failed the native metre-bounds gate: skin pixels
were human-sized while ordinary bounds applied the armature transform again.
The canonical export makes the mesh an identity scene root with its geometry
transform applied. It passes the original height gate and eliminates the glTF
nonroot-skin warning. Source pose baking also fixes the initially observed
connected-Hips constraint mismatch, and linear plus step samplers are verified.
Earlier output in `build/synty-character-stage` / `...-preview-stage` and initial
native capture are not the final artifacts; no tolerance/height gate was relaxed.

The qualified preview was opened on the user's desktop in GLB inspection mode,
playing Walk. It is not the active gameplay character or map. All private assets,
converted GLBs, Python tools environment and evidence stay ignored; all work is
local. The core finite-world/actual-town resource gate and the Elven Realm
assembled scene conversion remain open, as do full P1-10/P2-09 and phase completion.

## Elven source/dependency and representative static preview — 2026-10-20

P1-10/P2-08 bounded asset continuation. `unity_package.py --include-meta` stages
2,276 selected payloads and their metadata from the original private Elven
archive, preserving source hashes and paths. The recursive audit verifies staged
bindings and metadata GUIDs, inventories the actual scene and reports model
prefab-ID gaps plus missing serialized references. It does not instantiate the
scene. Qualification report: `build/synty-elven-qualified-dependency-audit.json`.

The isolated Blender converter exports one identity-root `SM_Bld_Pillar_01` to
standard GLB with an explicitly approximate base-colour material. Independent
comparison passes all 404 triangle positions, winding and UVs at 10⁻⁵
quantization; source 4096×4096 palette pixels match exactly. The exported bounds
are 0.646×3.006×0.646 m. Khronos reports zero errors, warnings or infos. Qualified
stage: `build/synty-elven-pillar-canonical-preview`, including conversion,
independent comparison and glTF validation reports. GLB SHA-256:
`8516faec5903deab0f244b1a0000bad6f543e0a56a44076208e16f916bc03994`.

One actual Bevy inspector case passes on AMD Radeon RX 7900 XTX / RADV NAVI31
Vulkan with active `VK_LAYER_KHRONOS_validation`, compiled/clear pipelines and
no error markers. The visually inspected 1280×720 capture contains 5,313 colours;
the inspector reports one mesh/material/image, 786 render vertices, 404 triangles,
normals and UV0, no animation/skin, and matching source-reference bounds. Reports:
`build/synty-elven-pillar-qualified-native-20261020/pillar`; log:
`build/synty-elven-pillar-qualified-native.log`.

Fresh checks pass: `cargo fmt --check`, denied-warning workspace/all-target Clippy,
**76 asset tests + 4 GLB tests = 80 Python tests**. New cases cover optional
metadata completeness/budget/immutability, signed/stripped Unity IDs, duplicate
IDs/keys, alias/object-constructor rejection, recursive model-prefab dependency
inspection, source tampering and static-oracle winding/UV/duplicate-face detection.
Logs: `build/synty-elven-fmt.log`, `build/synty-elven-clippy.log`,
`build/synty-elven-python-assets.log`, `build/synty-elven-python-glb.log`.

No Rust runtime/binary/engine-pin changes; the previous 154-Rust-test/eight-native
capacity checkpoint remains historical and was not rerun for offline Python
asset tools. No Windows, complete material/scene, collision/navigation, streaming
resource or game-adoption claim. [Contract and open gates](SYNTY_ELVEN_IMPORT.md).
All private source/outputs remain ignored and work stays local.

## Elven model identity/source-completeness continuation — 2026-10-20

P1-10/P2-08. `tools/unity_model_ids.py` reads only bounded binary FBX identities,
connections and model hierarchy. Modern signed XXHash candidates are checked
against distinct actual serialized source asset/fileID references. The candle
flame's legacy explicit metadata table binds its mesh by exact source name.
Collection/display-layer membership is excluded from transform parentage; true
cycles, ambiguous paths/IDs, malformed/extending records and unsupported mappings
reject. No decompression, source script execution or silent unresolved-ID fallback.

Current private source evidence:

- `build/synty-elven-dependency-vetted.json`: hash-checked dependency closure and
  full LFS pointer list with reachability flags.
- `build/synty-elven-model-ids-vetted.json`: 1,109 matches (517 Mesh, 222
  MeshRenderer, 185 Transform and 185 GameObject), 62 unmatched distinct references.
  Of these, 44 refer to missing source payloads and 18 have no source-derived
  candidate. These are not instantiated scene counts or Unity Editor equivalence.
- `build/synty-model-imported-hierarchy.log`: actual Blender hierarchy inspection
  of pillar, tree, eleven-mesh telescope and character source. No scene export or
  native map claim is implied by successful import/hierarchy inspection.
- `build/synty-elven-lfs-pointers.json`: 32 LFS-text payloads, 30 FBX/two PNG,
  declaring 1,429,558 missing bytes; 26 FBXs and one PNG are in the demo closure.
  This is unavailable content, not a permissible surrogate for geometry/images.
- `build/synty-model-ids-source-hash.log`: original archive SHA-256 remains
  `af62adb43b7e399d8ef8c514fa53d9a6ce715ee82fb1a79389e16e33a2dad1b1`.

Fresh fmt, denied-warning workspace/all-target Clippy and **85 Python tests**
(81 asset, four GLB) pass. New tests exercise published signed root-ID values,
32/64-bit binary hierarchy/geometry association with collection membership,
cycles/duplicate paths/invalid record bounds, exact legacy name-table mappings,
duplicate/missing legacy IDs and source LFS-pointer reporting/malformed rejection.
PyYAML 6.0.3 and xxhash 3.6.0 are pinned for offline tooling; the actual audits are
rerun with those versions. Logs: `build/synty-model-ids-{fmt,clippy,python-assets,
python-glb}.log`, `build/synty-elven-{dependency,model-ids}-vetted.log`.

No Rust application/renderer/binary changed, so Rust tests and GPU captures were
not repeated for identity/dependency-only tooling. The prior 154-test/eight-native
capacity checkpoint and separate Synty preview captures retain their scoped
meaning. Private source/outputs remain ignored. Full scene mapping/materials and
source completeness remain open. Core row-5 work can continue independently.

## Explicit finite-world startup/movement continuation — 2026-10-20

P1-03/P1-09/P1-10/P2-06/P2-07/P2-08, execution-order row 5. Shared startup
schema 2 now binds checked global XZ world bounds into identity v4; schema 1
retains exact v3 behavior/identity. Wire/ALPN stays 11. Shared authority movement
combines conservative local rectangle, loaded contributor coverage and client
readiness sweeps. Lookahead clamps to the finite rectangle. Trusted recovery
spawns and candidate actors retain full footprint validation. The launcher
remains monolithic town revision 2 at 30 Hz. [Contract](WORLD_EDGES.md).

Executed checks:

- Fresh workspace build, fmt and denied-warning workspace/all-target Clippy pass.
  Workspace tests: **157 passed, three ignored**; the final four-case edge test
  suite includes the three previous tests plus one new recovery case, yielding
  **158 distinct ordinary Rust tests**. The three ignored private tests separately
  pass. Logs: `build/world-edges-build.log`, `world-edges-workspace-tests.log`,
  `world-edges-final-tests.log`, `world-edges-final-fmt.log`,
  `world-edges-final-clippy.log`, `world-edges-private-tests.log` (all under `build/`).
- Original real-Avian edge/corner tests cover eight directions, zero and signed
  billion-metre positions and 20/30/60 Hz. They check jump height, landing,
  retreat, contained full footprints, vertical sweeps and open internal seams.
  Config tests cover legacy/changed-bound/query-frame-independent identities,
  invalid/out-of-frame rectangles and boundary spawns; clamped airborne/directional
  demand does not request an unnecessary outside provider.
- A new finite-world normal-authority test removes floor inside a still-resident
  cell, walks into the unsupported area, recovers to its validated in-bounds spawn
  and keeps held jump from retriggering. It passes at all three rates. This
  distinguishes unsupported-fall recovery from temporary loading barriers.
- The opt-in `build/world-edges-town-runtime-stage` copies the previous ten-to-26
  stage with shared immutable package/image bytes and a new session metadata file.
  Source playable bounds are exactly retained from the monolithic town profile:
  `[[-300.2397363603209,-113.47650036279128],
  [233.09359188674944,419.8567515903337]]`. All 26 packages publish against one
  live actor; 90 following ticks have zero recoveries and retained pose/support.
  Single concurrent-qualification samples: initial 2,068.804 ms, candidate
  preparation 2,104.401 ms, publication 14.084 ms. No latency percentile, cold
  loading, staging memory or capacity claim follows. Both private source routes
  pass six stages at 20/30/60 Hz; the shared-image verifier passes all 488 images.
- **Six actual native two-client cases / twelve reports** pass on AMD Radeon
  RX 7900 XTX / RADV NAVI31 Vulkan, with active Khronos validation, compiled/clear
  pipelines, two-pawn pixels, authoritative/rendered pose agreement, live Avian,
  camera/ground checks and zero unexpected recovery. Cases: finite +X and -Z
  pressure with 32 loaded original roots; delayed negative billion-metre
  six-provider publication with an independent client frame; town; farmhouse;
  Quinn. The bounded edge cases independently check every final actor's full
  rectangle clearance and contact with the expected plane. Delayed publication
  keeps exact retirement barriers, retained roots, shared image handles and
  matching Avian generation. Evidence:
  `build/world-edges-qualified-native-v2-20261020`,
  `build/world-edges-qualified-native-v2.log`. A publication capture was visually
  inspected. An earlier fixture rejected because it allowed less than the
  diagnostic's required three-metre travel; its `REJECTED.md` preserves that
  result and corrected fixtures retain the original assertions.
- **85 Python tests** pass (81 asset + four GLB), logs
  `build/world-edges-python-assets.log` and `world-edges-python-glb.log`.
  Imported engineering-source hashes remain unchanged; private assets/build
  outputs stay ignored and all work remains local.

Actual-town chunked Bevy loading/resource/staging measurements, full-town edge
accessibility, remaining handover lifecycle cases, default promotion and Windows
qualification remain open. No whole task/phase or MMO capacity claim follows.

## Actual partitioned-town client startup/resources — 2026-10-20

P1-10/P1-12/P2-06/P2-07/P2-08, execution-order row 5. Actual town startup
initially rejected strict Bevy contributor validation. Investigation found the
offline author used transformed vertices, while the runtime validates transformed
local AABB corners; rotated sparse geometry made those envelopes disagree.
`tools/index_chunk_contributors.py` now derives the latter from actual accessor
positions and the static hierarchy. The runtime guard is unchanged. A new sparse
rotated-triangle regression reproduces the difference; all corruption/animation/
large-envelope/over-budget tests still pass. No source asset or runtime change.

Only a new private stage's index changes: 25 conservative envelopes expand;
26 package identities, all GLB/collision/image bytes, trusted spawns and finite
bounds remain unchanged. Maximum cell contributors remain nine, initial recovery
providers ten. New index SHA-256:
`1a9061581932d3b562443b6a484f68e2a40b44d957e2d1f3b04338cfdf311c9b`.
Stage: `build/town-resource-runtime-stage-v2`; correction report:
`build/town-resource-envelope-correction.json`. The rejected first run remains at
`build/town-resource-qualified-native-20261020/REJECTED.md`, with failed startup
and partial resource observations explicitly excluded from qualified baselines.

**Two actual native two-client cases / four reports** now pass: ten-package
startup (2,783 static colliders, 377 shared image variants), then all 26 packages
(4,001 colliders, 488 variants). Both run ordinary secure Renet applications,
standard GLB/image loading and live Avian on **AMD Radeon RX 7900 XTX / RADV
NAVI31 Vulkan**, with active Khronos validation. Existing verifier requires exact
world identity/readiness acknowledgements, root ownership, bound real material/
image/sampler handles, authority/rendered pose agreement, independent capsule/
triangle clearance, ground/camera, both pawn pixels and no unexpected recovery.
The all-26 screenshot was visually inspected. This is full startup residency,
not ten-to-26 live client replacement or source-asset material fidelity.

`tools/process_metrics.py` records sampled Linux process RSS/high-water RSS/PSS/
swap and deduplicated DRM-client counters, following kernel fdinfo semantics.
The sample period requested is 250 ms with actual timestamps retained; missing
counters remain unavailable, aliases/duplicated FDs do not inflate resident
values, and categories are not additive. Each client exposes one measured amdgpu
DRM client; the headless server exposes none. These warm-cache concurrent debug
qualification samples are not cold/release benchmarks or percentile guarantees:

| Startup | Per-client observed peak RSS | Driver VRAM residency | Approx. first process observation to ready marker |
|---|---:|---:|---:|
| Ten packages | 1.82–1.83 GiB | 1.54–1.60 GiB | 15.7–15.9 s |
| All 26 | 2.30 GiB | 1.84–1.85 GiB | 22.2 s |

Server peak RSS is 111.5 / 152.3 MiB. PSS/raw DRM categories are retained in each
case's `*-resources/summary.json`/`samples.jsonl`; sampled marker observations
are in `*-events.json`. Maxima may occur at different times; driver buffers
include more than textures, and sampling can miss short peaks. Environment:
30.49 GiB OS-reported RAM, nominal 24 GiB discrete GPU, active Vulkan validation.
[Measurement contract and scope](TOWN_CLIENT_RESOURCES.md).

Fresh fmt, denied-warning workspace/all-target Clippy and **88 Python tests**
(84 asset plus four GLB) pass. New cases validate conservative rotated sparse
AABBs, measurement units, unavailable values and DRM descriptor/alias deduplication.
Logs: `build/town-resources-{fmt,clippy,python-assets,python-glb}.log` and
`build/town-resource-author-tests.log`. Rust runtime/application binaries and lockfile are
unchanged from the preceding 158-test/three-private-test checkpoint; Rust tests
were not rerun for offline authoring/measurement tools.

Native evidence: `build/town-resource-qualified-native-v2-20261020/`,
`build/town-resources-qualified-native-v2.log`,
`build/town-resources-qualified-summary.json`, with current binary/tool hashes.
The default launcher remains monolithic revision 2 at 30 Hz. Live candidate/
staging/retirement resource lifetimes, remaining handover lifecycle checks and
map regressions still precede adoption. No native Windows, sustained FPS/crowd/
soak, full source-world traversal or whole task/phase claim. All private assets/
outputs remain ignored; changes stay local.

## Actual-town live publication continuation — 2026-10-20

P1-10/P1-12/P2-06/P2-07/P2-08, execution-order row 5.
Fresh `cargo fmt --check`, denied-warning workspace/all-target Clippy, locked
workspace build and **160 ordinary Rust tests** pass. Three default-ignored
private tests separately pass with explicit prepared map/shared-image/partitioned
town sources. Python passes **91 tests** (87 asset plus four GLB). New regressions
cover resolved sloped recovery feet, exact decimal f64 bits, actual publication
primitive/root/image evidence and negative-cell nonlocal coverage.

Logs: `build/town-live-{fmt,clippy,qualified-build,qualified-workspace-tests,
private-qualified-tests,python-assets,python-glb}.log`. The initial private-test
invocation selected the parent shared-assets directory and failed with a missing
terrain.json; the corrected explicit `shared-images-town-stage/town` path passes.
No code or assertion was changed for that invocation error.

The initial native attempt held publication on raw sloped recovery guesses;
the next published but failed independent identity because of decimal parsing.
The third normal case passed, while its delayed case exposed a test assumption
using package-owner maxima as a coverage boundary. Rejected evidence remains
separate. The bound-geometry delay oracle now derives actual complete cells from
index envelopes; default green-floor boundary/pixel checks remain unchanged.
[Implementation, evidence contract and limitations](TOWN_LIVE_PUBLICATION.md).

Nine actual two-client native regression cases (18 reports) pass on current
binaries with active Vulkan validation: 32-root startup, delayed six-provider
publication, delayed negative independent-frame publication, missing image,
staged root cleanup after bounds rejection, all-26-package town startup,
monolithic town and farmhouse routes, and Quinn. The cleanup trace verifies
three roots before discard, two after rejection/settling, unchanged retained
entity IDs and original GLB/mesh/material asset counts after settling.
Evidence: `build/town-live-regression-native-20261020/` and
`build/town-live-regression-native.log`.

Both final actual-town cases pass (four additional reports): normal loading and
five-second delayed loading with client query origin [8,25,-8]. Together with
the nine regressions this is **11 cases / 22 reports**. Each actual town client
adds one hidden provider, publishes ten-to-eleven roots, retains original root
entities/shared image IDs, and reaches matching live physics/exact permission.
Source-bound new-root primitive counts pass; image assets remain 383 total
(377 shared variants plus existing application images), GLB assets 10→11 and
mesh assets 912→914. No unexpected recovery is reported.

Normal candidate publication occurs at 1.814–1.823 seconds from local preparation
start; delayed-peer publication occurs at 6.930 seconds, including its injected
five seconds. These are individual warm-cache debug observations with active
validation, not startup times, averages or latency guarantees. Sampled per-client
whole-run RSS peaks are 1.80–1.82 GiB normally and 1.85–1.87 GiB asymmetrically;
driver VRAM residency is 1.60–1.61 GiB. These maxima are not phase-specific.

Evidence: `build/town-live-qualified-native-v4-20261020/`,
`build/town-live-qualified-native-v4.log`, `build/town-live-qualified-summary.json`
and binary/source hashes in `build/town-live-qualified-hashes.json`. Native normal
capture was visually inspected. No default-map change, native Windows execution,
full-town live publication, worst-case staging or production capacity claim.

## Enchanted Forest target switch — 2026-10-20

P1-10/P2-08: the user's forest package replaces Elven Realm in the active import
queue. Inventoried 667 files, extracted 647 explicit data payloads plus metadata,
verified zero LFS pointers, recursively audited Demo_01 and matched all 264
queried model/fileID references (zero unmatched/model errors). Six external GUIDs
are absent from the archive; eight additional references are intentionally
unstaged shaders/script, recorded separately. The old simple-preview converter
rejects the forest material's missing `_BaseColor`; no forest preview or active
map replacement is claimed. [Source/acceptance record](SYNTY_ENCHANTED_FOREST_IMPORT.md).

Fresh fmt, denied-warning workspace/all-target Clippy and 91 Python tests
(87 asset plus four GLB) pass. Logs: `build/synty-enchanted-{audit,model-ids,
preview,fmt,clippy,python-tests,glb-tests}.log`. No Rust runtime change, new binary
or new rendering qualification; preceding town/native evidence remains separate.
All private sources remain ignored and changes local.


## Enchanted Forest opaque Standard material — 2026-10-20

P1-10/P2-08: actual SM_Prop_Ruins_01 passes source-imported triangle/winding/UV
comparison (7,844 triangles), exact 4096×4096 albedo and emission pixels, independent
source-bound material graph/scalars/culling/sampler checks and Khronos GLB validation
(zero errors/warnings/infos/hints). One actual native Bevy inspector capture passes
on RX 7900 XTX/RADV NAVI31 Vulkan with active validation and compiled pipelines.
The screenshot was visually inspected; this qualifies one source asset/PBR
adaptation, not the assembled forest or gameplay collision.

Fresh fmt, denied-warning workspace/all-target Clippy and **95 Python tests**
(91 asset plus four GLB) pass. Fault injection checks swapped slots, changed
scalars/culling/filtering/UV/emission and NaN; source planning rejects unsupported
features. Original Elven pillar conversion and its independent source oracle pass.
Rust code/binaries/pins remain unchanged; no Rust test rerun was required for
offline conversion tools. Prior 160 ordinary/three-private Rust tests retain
separate checkpoint evidence. No native Windows or default adoption claim.

Evidence: `build/synty-enchanted-ruins-pbr-qualified/`,
`build/synty-enchanted-ruins-qualified-native-20261020/`,
`build/synty-enchanted-pbr-gltf-validation.json`,
`build/synty-forest-material-final-*.log` and
`build/synty-forest-standard-material-coverage.json` (field-level eligibility,
12 of 55, not all texture/model qualification). Earlier uncorrected PBR trial is
not qualified. [Detailed contract](SYNTY_ENCHANTED_FOREST_IMPORT.md).


## Forest static hierarchy and instance planning — 2026-10-20

P1-10/P2-08: the actual Demo_01 source plan resolves 18,546 placed prefabs and
emits 18,480 static records after highest-detail LOD/active-state selection. All
emitted bindings/matrices pass an independent source/mathutils reference; maximum
absolute matrix-entry difference is 0.0000227346793 within the declared 0.0001
f32-reference threshold. This does not prove no omissions, Unity Editor parity,
FBX geometry/axes or rendered scene equivalence. The 423 unused flat-target
overrides are separately recorded; 179 FBX-variant and 30 particle placements stay
unresolved. Source hash bindings and errors are preserved in ignored reports.

Fresh fmt, denied-warning workspace/all-target Clippy and **103 distinct Python
tests** pass (98 asset, four GLB, one separate Blender reference fault test).
No Rust/runtime/render change; no new Rust test or native GPU claim. Evidence:
`build/synty-forest-static-instance-plan-v2.json`,
`build/synty-forest-static-reference-qualified.json`,
`build/synty-forest-override-target-analysis.json`,
`build/synty-forest-prefab-{unit-tests,reference-tests,python-assets,python-glb,fmt,clippy}.log`.
[Detailed acceptance and remaining work](UNITY_STATIC_PLACEMENTS.md).


## Basic forest foliage adaptation — 2026-10-20

P1-10/P2-08: the actual 160-triangle background-card set and eight-triangle vine
LOD1 pass source-bound branch geometry/winding/UV, scalar/material, exact derived
colour/alpha and standard-export normal-rounding checks. Independent raw-image
colour tables and cyclic-triangle normal matching are separate from the converter.
Khronos reports zero issues. Two native Bevy inspector cases pass on RX 7900
XTX/RADV Vulkan with active validation; both captures were visually inspected.
The first tree-card fixture incorrectly expected one primitive, is marked
rejected, and is superseded by a source-bound two-primitive case. A rejected vine
LOD0 trial belongs to a different shader family; qualified output explicitly uses
source LOD1. No active-map or high-detail replacement is claimed.

Fresh fmt, denied-warning workspace/all-target Clippy and **109 distinct Python
tests** pass: 104 asset, four GLB, one Blender reference test. Ten private fault
checks reject changed cutouts, culling, surface, filtering, branch/geometry/image
bindings and normals. A full-source highest-detail plan rerun is exactly equal to
the prior 18,480-record plan. Rust runtime/binaries are unchanged; prior Rust and
map qualification retain separate evidence, with no new Rust test rerun.
Custom normal-map output and Unity colour/BRDF equivalence remain unqualified.
[Detailed scope, normal bound and artifact paths](FOREST_FOLIAGE.md).


## Static model-variant placements — 2026-10-20

P1-10/P2-08: all four source branch variants and their 2,864 outer modifications
bind by the checked inherited-ID rule. Their 179 placements supply complete root
TRS, allowing source-coordinate compilation without imported default poses. All
179 pass independent source identity/category completeness/mathutils checks;
maximum matrix-entry difference is 0.0000147762358 within the 0.0001 reference
threshold. The 18,725-instance/18,659-record plan retains every old 18,480 record,
all 423 unused overrides and the 30 remaining particle-override errors exactly.

Fresh fmt, denied-warning workspace/all-target Clippy and **115 distinct Python
tests** pass (108 asset, four GLB, one previous Blender reference and two new model
reference tests). Missing/duplicate placement, identity/material/pose corruption,
NaN, inherited parent transforms and active overrides reject. No Rust/runtime/
renderer/binary change, no new GPU/Rust test run, native Windows or assembled
scene/collision claim. [Detailed evidence and reproduction](UNITY_MODEL_VARIANTS.md).


## Forest geometry and five actual placements — 2026-10-20

[Executed contract and exact ignored evidence paths](UNITY_GEOMETRY.md).
Pinned ufbx probe compiles with denied adapter warnings. Source LOD calibration
matches 52/54 with two explicit mismatches and no parser failures. Independent
canonical FBX point comparison and five-instance geometry/normals/winding/assets
checks pass. Final forest and asymmetric fixture Khronos checks have zero errors/
warnings. Two final native Bevy inspector cases pass on RX 7900 XTX/RADV Vulkan
with active validation, compiled pipelines and visually inspected captures;
maximum actual vertex-bound errors are 2.674 and 0.954 micrometres respectively.

Fresh `cargo fmt --check`, `cargo clippy --locked --workspace --all-targets -j1
-- -D warnings`, 114 asset tests, four GLB tests, one independent prefab reference
test and two variant reference tests pass (121 Python tests). Rust source/binaries
remain unchanged; prior Rust results were not rerun. This is Linux visual asset
qualification, not full-scene/collision, Windows or default-map adoption. All
source assets, third-party verifier inputs and output evidence remain ignored.


## Forest binary terrain and actual-ground preview — 2026-10-20

[Exact contract and evidence paths](UNITY_TERRAIN.md). A pinned offline reader
preserves every source height sample, five layer/control bindings and source
metadata. The independent verifier checks unique raw serialized height bytes,
all 1,365 patch extrema, 263,169 positions, 524,288 fixed-grid triangles, UVs,
raw control weights and 685 scalar source-colour samples. Maximum position error
is 0.605 micrometres; sampled colour-channel error is zero. The independent oracle
does not import the reader/converter.

Ground and ground-plus-five-ruins have clean Khronos validation. Two actual native
Bevy inspector cases pass on RX 7900 XTX/RADV Vulkan with active validation,
compiled pipelines, no VUID/errors/panics and visually inspected captures. Actual
transformed geometry bounds agree within 0.605 micrometres. This is a finite
base-colour preview with geometric normals and explicit fixed-grid tessellation;
it does not establish Unity shader/normal-map/TerrainCollider parity.

Fresh fmt, denied-warning workspace/all-target Clippy, 120 asset tests, four GLB
tests, one independent prefab reference and two model-variant reference tests
pass (127 Python). Rust source/binary/lock and imported planning hashes remain
unchanged; previous Rust tests were not rerun for offline tools. The current town
remains runnable. Matching terrain collision/full forest/native multiplayer and
Windows execution remain open; all sources/output remain ignored and work local.


## Full-sample forest ground collision — 2026-10-20

[Contract, exact paths and reproduction](FOREST_TERRAIN_COLLISION.md). Four
packages preserve 263,169 distinct source vertices, 524,288 oriented triangles,
exact UV/normals and one shared PNG; independent visual/collision/source coverage
and rebound missing-triangle fault tests pass. All package GLBs have zero Khronos
errors/warnings and one expected external-image informational message each.

Actual source Avian checks pass 81 raw-height probes (maximum error 3.815
micrometres), 18 X/Z/diagonal seam/jump cases at 20/30/60 Hz and two admitted
actors without recovery. Two actual encrypted two-client cases/four Bevy reports
pass on RX 7900 XTX/RADV Vulkan with active validation, compiled pipelines,
exact readiness/identity, matching live Avian and shared image handles. Captures
were visually inspected; the Z route changes height. Four actual launcher
admission/cleanup scenarios verify forest opt-in and the unchanged town default.

Fresh fmt, denied-warning workspace/all-target Clippy, 160 ordinary Rust tests,
the new private forest test and private old-map route regression pass (162
distinct Rust tests). Both town/farmhouse routes complete six stages at all three
tick rates without recovery. 131 Python tests pass: 124 asset plus four GLB, one
prefab reference and two variant reference tests. Application binaries, lock and
imported planning hashes are unchanged; all private assets/output remain ignored.
This is ground-only qualification, not full forest, Unity TerrainCollider/material
parity, dynamic ground residency, native Windows or final map adoption.


## Highest-detail full forest foliage — 2026-10-20

[Contract and exact evidence](FULL_FOREST_FOLIAGE.md). Two actual LOD0 prototypes
pass branch-bound geometry/winding/UV/mesh normals, exact derived colour/source
alpha, chosen normal reconstruction and independent ufbx mesh-local checks.
An observed 2.98e-8 UV rounding boundary changed oracle correspondence to exact
f32 cyclic positions plus bounded UV ULP checks; winding/position/UV faults reject.
Three trees and 612 exact mesh/material-binding fern instances pass independent
pose/geometry assembly checks, preserving the original source-plan hash.

Tree, fern and combined ground/ruins/vegetation subset pass native Bevy on
RX 7900 XTX/RADV Vulkan with active validation, compiled pipelines and visually
inspected captures. Maximum native assembled-bound error is 1.701 micrometres.
All three have zero Khronos errors/warnings; tree/assembly retain three unused-
attribute/tangent informational messages. A regenerated basic card is byte-
identical to the previously native-qualified artifact and passes the updated oracle.

Fresh fmt, denied-warning workspace/all-target Clippy and 137 Python tests pass
(130 asset plus seven GLB/reference). The earlier 162-test Rust checkpoint was
not rerun; application source/binaries/lock and ground/default-map content remain.
Wind/highlights, filtered/source-platform shading parity, four unresolved used
full-shader profiles and other shader families, complete scene/object collision,
native Windows and adoption remain open. All assets/evidence stay ignored/local.


## Forest foliage batch — 2026-10-20

46 exact source bindings / 15,156 draws pass prototype pixel/geometry/ufbx and
complete final-scene/source-override checks. Final scene-v3 has 2,517,360 triangles,
15,384 primitive instances and 17 shared image variants. Khronos checks 47 files:
zero errors, 14 runtime-tangent warnings, 101 informational messages. Two native
Bevy/RADV Vulkan cases pass with active validation and no tangent-generation
failures; transformed vertex bounds differ from the independent oracle by at most
0.0151 mm. Both captures were inspected. Fresh fmt, denied-warning Clippy and 149
Python tests pass; source rechecks and byte-identical basic-card regression pass.
Rust runtime/binaries unchanged; prior Rust/ground-network results are separate.
3,503 source draws, full object collision, resources and adoption remain open.
[Detailed contract, exact hashes, limitations and evidence](FOREST_FOLIAGE_BATCH.md).


## Forest Standard-material batch — 2026-10-20

66 additional bindings / 2,744 source draws join the static subset: 17,900 source
renderers, 5,151,519 triangles and 31 shared PNG variants. Independent source/
geometry/normal/palette checks pass for all 66, and complete final-scene source
coverage passes. Maximum canonical ufbx error is 12.48 micrometres; actual native
bound error is 0.0151 mm. Two final Bevy/RADV Vulkan captures pass active validation
and were visually inspected. Khronos checks 113 files with zero errors, 14 existing
runtime-tangent warnings and 117 informational messages. Fresh fmt, denied-warning
Clippy, 150 Python tests, six real-stage fault injections, full source rechecks and
byte-identical legacy ruins pass. Rust runtime/pins unchanged; old network/ground
qualification remains separate. 759 draws, object collision, resource budgets and
adoption remain open. [Exact evidence and limitations](FOREST_STATIC_BATCH.md).


## Forest model-variant geometry — 2026-10-20

Four actual branch meshes / 179 source poses pass raw inherited binding/material,
geometry/UV/normal/pixel, ufbx and complete placement checks. New canonical errors
are at most 4.33e-8 metres. Prior 112 prototype/placement/reference/conversion records
remain byte-identical. The combined 18,079-draw scene passes complete source
coverage and two native Bevy/RADV Vulkan views with active validation; actual
bounds agree within 0.0151 mm. Both captures inspected. Khronos checks 117 files:
zero errors, 14 existing tangent warnings, 117 informational messages. Fresh fmt,
denied-warning Clippy and 152 Python tests pass; raw sources rechecked at entry/
exit. Rust runtime/pins unchanged; existing Rust/networked-ground evidence separate.
580 draws, object collision, resources and adoption remain open.
[Exact contract and evidence](FOREST_VARIANT_GEOMETRY.md).


## Forest triplanar material prerequisite — 2026-10-20

One actual moss mesh (196 triangles) and one selected source instance pass raw
shader/material/PNG/geometry/UV/normal/tangent checks and independent ufbx positions
(maximum 2.76 micrometres). Six final Bevy/RADV Vulkan lit/colour/normal cases pass
active validation; native bounds agree within 0.00318 mm. Independent camera-ray
oracles check 588 actual diagnostic pixels with maximum one sRGB8 level difference.
Missing metadata options and unmarked fallback geometry reject without panic.
Khronos checks both GLBs with zero errors/warnings and one informational unused-
tangent message each; custom material semantics are covered by separate native/
pixel evidence. Five private fault injections, fresh fmt, denied-warning Clippy,
157 Python tests, 31 client Rust tests and full source/archive rechecks pass.
Combined-scene material/rebase/resource/mip and importer/root-pivot qualification
remain open; no triplanar draw is promoted into the main 18,079-draw subset.
[Exact evidence and source-pivot findings](FOREST_TRIPLANAR.md).

## Forest Editor-reference preparation — 2026-10-20

P1-10/P2-08: five actual Linux Unity 6000.6.6f1 batch runs compile/export all 112
scene FBXs / 214 meshes; every FBX-backed scene binding is covered. All original
metadata is unchanged. Independent point sets in recorded Unity frames match
within 11.02 micrometres. The old mesh-local policy differs for nineteen meshes,
affecting ten converted bindings / thirteen current batch draws. Five new
synthetic tests pass; the fresh total is 162 Python tests (155 asset, four GLB,
three Blender prefab/variant). Fresh fmt and denied-warning Clippy pass. No Rust/
binary/runtime/render change or new GPU/map regression occurred. All jobs are
terminal; the user requested a safe stop before converter replacement work.
[Scope, hashes, findings and resumption checks](FOREST_UNITY_REFERENCE.md).

## Forest mesh-frame continuation — 2026-10-21

P1-10/P2-08: ten corrected bindings / thirteen source draws, 106 byte-identical
prototype/placement pairs and all 116 actual Editor point-set checks pass (maximum
7.69 micrometres). Full combined scene source/pose/stream/material/image coverage
passes. Three native RX 7900 XTX/RADV Vulkan views pass active validation; 117
glTF files have zero errors and the fourteen retained tangent warnings. Actual
encrypted town/farmhouse two-client regressions pass. Standalone corrected moss
and all 85 source poses pass independent Editor/world checks, six final native
cases, Khronos (zero errors/warnings) and 512 final pixel checks (maximum one
sRGB8 level under the unchanged four-level bound). Rejected coarse overview/
intersecting-surface pixel samples are preserved; final eligibility uses geometry
alone. Fresh fmt, denied-warning Clippy, 169 Python tests, three private faults
and source/metadata rechecks pass. Rust binaries and runtime are unchanged.
Main subset remains 18,079 draws / 580 unresolved; no forest adoption/physics/
native Windows or production capacity claim. [Exact evidence](FOREST_MESH_FRAMES.md).

## Bounded forest material catalog — 2026-10-21

P1-10/P2-08/P2-10: actual 85+42 moss poses plus Standard building, complete input
composition and two native views × three modes pass. Actual 127 marked primitives,
one retained Standard primitive, two extended materials and six shared typed image
handles match the metadata. Independent colour/normal checks pass 1,906 samples
with at least thirty per profile, maxima one/two sRGB8 levels under the unchanged
four-level limit. Six native mapping failures and four rebound CPU composition
faults reject. Khronos, three legacy modes, three corrected forest views and
encrypted town/farmhouse two-client regressions pass on the final client. Fresh
fmt, denied-warning Clippy, 35 client Rust tests and 171 Python tests pass. Native
Linux RX 7900 XTX/RADV Vulkan only; server/lock unchanged. Main subset still
18,079 draws / 580 unresolved; no chunk/rebase/mip/whole-map adoption claim.
[Exact evidence and limitations](FOREST_MATERIAL_CATALOG.md).

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
[Evidence and limits](FOREST_TRIPLANAR_BATCH.md).


## Forest texture-default continuation — 2026-10-21

Forest texture-default continuation, 2026-10-21: actual Linux Unity 6000.6.6f1
resolves four Grass/Mud textures to bilinear/repeat with original PNGs/metadata
unchanged. The ten grass draws now join the qualified subset: **18,607 source
draws / 52 unresolved**. Fresh eighteen-binding source/Editor/placement checks,
four combined native Vulkan views, three grass modes and 274 pixel checks pass.
Fresh fmt, denied-warning Clippy and 175 Python tests pass. Runtime binaries remain
unchanged. Next: independent leaf-colour UV tiling for seventeen giant-tree draws;
remaining materials, rebase/mips/resources, collision and adoption remain open.
All work stays local. [Evidence](FOREST_TEXTURE_DEFAULTS.md).

## Forest colour-UV continuation — 2026-10-21

Forest colour-UV continuation, 2026-10-21: two exact giant-tree bindings /
seventeen placements now join the qualified visual subset: **18,624 source draws /
35 unresolved**. A second standard glTF channel preserves leaf colour tiling while
normal/AO retain UV0. Source/Editor/exact-placement/full composition checks and six
native Vulkan cases pass; actual loaded materials select the expected channels.
Legacy rendering and encrypted town/farmhouse regressions pass on the new client.
Fresh fmt, denied-warning Clippy, 35 client Rust and 179 Python tests pass. Remaining
materials, rebase/resources/mips/lighting, collision and adoption stay open. All work
is local. [Evidence](FOREST_COLOUR_UV.md).

## Forest glass continuation — 2026-10-21

Forest glass continuation, 2026-10-21: an explicit untextured Standard-to-glTF
BLEND adaptation adds the lantern glass: **18,625 source draws / 34 unresolved**.
Source geometry/material, actual Editor frame, exact placement/full composition
and four native Vulkan cases pass. The adaptation records Unity premultiplied
lighting/colour-space differences; no runtime shader/binary change is introduced.
Fresh fmt, denied-warning Clippy and 181 Python tests pass. Next: actual mixed
submesh-slot coverage for 25 crystal/geode draws. Remaining materials, lighting,
rebase/resources/mips, collision and adoption remain open. All work stays local.
[Evidence](FOREST_GLASS.md).

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
[Evidence](FOREST_MIXED_MATERIALS.md).

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
All work stays local. [Contract and evidence](STATIC_SHAPES.md).

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
[Packages](FOREST_SHAPE_PACKAGES.md), [standard materials](STATIC_MATERIALS.md).

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
[Evidence and scope](TRIPLANAR_PHASE.md).

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
open. All work stays local. [Contract and evidence](RUNTIME_MATERIALS.md).

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
default. All work stays local. [Evidence and limits](FOREST_RUNTIME.md).
