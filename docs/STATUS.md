P3-09 / P3-11 / P3-15 explicit frozen population quest launch,
14 November 2026: the account population wrapper accepts an explicit inventory-bound
expanded quest catalogue, matching dialogue catalogue and guide position together.
It freezes both exact files beside the combat/item/reward profiles and records
hashes, selected journal/item/conditional modes and contact placement. Conditional
per-life rewards select the existing quest-loot mode; missing journal/contact
inputs and oversized files refuse before creating output. Normal application
preflight remains responsible for complete catalogue/cross-profile validation.
Default editor population launches retain their existing setup; this does not
publish authored contacts/quest areas or load a template implicitly.

The native wrapper qualifier verifies all 20 frozen hashes, first kill before
acceptance (ordinary token only), visible acceptance, eligible pelt pickup,
turn-in consumption and two PostgreSQL process restarts/same-character rejoins.
Both kills took five normal casts; XP 75, Completed, inventory revision 6,
102 tokens and no pelts survive. Denied and eligible original generation records
stay exact; four claims remain consumed. Health is transient, not restored.
Evidence: build/population-quest-package-native-v1/qualification.json, screenshots,
phase SQL rights and owned/launch.json. Linux 1280x720 RX 7900 XTX / RADV NAVI31 /
Vulkan / Mesa 26.2.6, zero GPU diagnostics; explicit one-pelt/guaranteed-loot fixture,
normal combat/pickup requests and PID-bound quest-button clicks. No new Rust or
gameplay policy. 27 Python launcher tests pass (one skipped), compile, fmt, strict
workspace/all-target Clippy and diff checks pass; see build/population-quest-package-
{tests,clippy}.log. Earlier client/network suites remain valid for the unchanged
binaries. Next: native accepted kill-objective credit from population deaths,
then deliberate authored contact/quest integration. Local commits only.

P3-05 / P3-09 / P3-11 / P3-15 normal population quest-required loot,
14 November 2026: normal population launch now accepts the existing conditional
loot mode. The launcher accepts an explicit --quest-catalog with quests/journal,
refusing a catalogue without its mode or alongside a frozen gameplay manifest.
The existing exact inventory/journal/reward bindings and first-tag eligibility
remain authoritative. Editor population exports do not silently attach quests.

Owned OIDC/PostgreSQL and native 1280x720 qualification uses the unchanged pursuing
five-creature package, an explicit one-pelt journal, nearby guide and guaranteed
conditional-pelt/ordinary-token fixture. Before acceptance, eight normal casts
kill one NPC, award 25 XP and only ordinary loot: bag revision 3, 101 tokens,
no pelt. After actual PostgreSQL restart and same-character rejoin, PID-bound
quest-button acceptance succeeds. Five normal casts kill a fresh life; two corpse
pickups make the quest Ready. Visible turn-in consumes the pelt and awards 25 XP.
A second process restart/rejoin preserves Completed, 75 XP, revision 6, 102 tokens
and no pelts. Both deaths leave the four other creatures undamaged. Health is
transient: 44/58/78 at the three observations. Ordinary equipment, mana, facing,
range and cast validation apply; combat/pickup uses normal test-driver requests.

SQL retains the original denied pelt generation unchanged after acceptance and
restart, one later eligible generation and four consumed claims. Exact observed
bag/journal heads are recorded. Evidence: build/population-quest-loot-native-v1/
qualification.json, phase screenshots and generation-rights.txt, binary hashes,
transport/persistence summaries. Linux RX 7900 XTX / RADV NAVI31 / Vulkan /
Mesa 26.2.6, zero GPU diagnostics. This is native Linux/30 Hz and explicit fixture
integration, not random rates, editor quest publishing, Windows or MMO capacity.
The bounded test driver now checks one or two pickups against the committed
pre-combat bag, supporting resumed characters without changing production loot.

All 509 default network tests (17 fixture-dependent ignored), 265 client tests
(four ignored), 23 Python launcher checks (one skipped), fmt, strict workspace/
all-target Clippy and diff checks pass. Logs: build/population-quest-{network,
client}-tests.log, population-quest-launcher-tests.log and
population-quest-loot-clippy.log. Retained actual encrypted two-recipient
conditional population routing also passes at 20/30/60 Hz on the authored map:
build/population-quest-encrypted-library-tests.log. Those synthetic admission
ports prove routing separately from the native SQL flow. An initial unnecessarily
broad cargo test build was interrupted during unrelated example linking because
free disk space was low; the library-only rerun passes. Continue with explicit frozen account-launch
quest profiles and native reuse; general quest authoring remains separate. Local only.

P3-06 / P3-09 / P3-10 / P5-17 authenticated native population competition,
14 November 2026: two distinct OIDC accounts now qualify the ordinary pursuing
editor population together through one normal authoritative server. The owner
lands one Arcane cast, stops attacking, then the other player lands four casts
and finishes the NPC. Shared damage threat transfers once; both clients observe
the public windup toward the finisher, its private incoming windup and remote-victim
wolf animation. Actual health lost totals 100; four neighbouring NPCs remain
undamaged. Normal equipment, mana, facing/range checks and cast timing apply.

Only the original tagger receives 25 XP, the training reward and both corpse
pickups: inventory revision 4, one pelt and 101 tokens. The finisher retains zero
XP/rewards and equipment revision 1. Actual SQL has one XP receipt, three consumed
owner claims and two corpse claims for one victim life. PostgreSQL process restart,
fresh server, separate account login and both character rejoins preserve exact
inventories/XP without additional grants. Fresh health is 62/85; health is transient.

Evidence: build/population-competing-accounts-native-v3/qualification.json, matching
binary hashes, fresh/transfer-sample.json, both native screenshots and fresh/resumed
SQL records. Linux RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6, zero GPU
diagnostics. The test-only client driver can request an exact currently live target
and wait for another player's kill before collecting; it still sends ordinary
selection, equipment, cast and pickup requests. No OS input or prepared damage.
The initial harness cutoff was too strict at the 2.9 m NPC stopping boundary;
match existing 2.95 m selection and await committed diagnostic publication after
pickup. All 265 client tests pass (four fixture-dependent ignored), strict workspace/
all-target Clippy, fmt, Python compile and diff checks pass. Evidence:
build/population-competing-driver-tests.log and population-competing-driver-checks.log.
This is native Linux/30 Hz normal grants and loot, not a new gameplay policy,
Windows or MMO capacity claim. Multi-rate impaired transport and SQL contracts
remain separate. Next: close the remaining normal population quest-only loot
integration gate, preserving exact journal/owner eligibility. Local only.

P3-06 / P3-09 / P3-10 / P3-11 pursuing population durable competition,
14 November 2026: extend the retained owned World/PostgreSQL qualifier with an
explicit --threat population mode and 20/30/60 Hz selection. It uses the normal
configured navigation, retaliation, reset, shared damage threat and public NP01
observation. Accepted Arcane casts move the victim from the first tagger to the
finisher; exact public/private windups agree and an incoming ordinary strike reaches
the finisher. Every live sample checks damage-threat totals against actual health
lost and unchanged tag ownership. Death clears the table; neighbouring NPC health
is unchanged. The existing no-cost/unequipped Arcane and synthetic account/accepted
legacy quest fixtures remain explicit, without production gameplay changes.

At all three rates the first tagger alone receives 25 XP, one accepted quest count
and one unconsumed training claim. SQL lock withholding, continued fixed steps,
replay idempotency, wrong-recipient refusal, fresh Store connection and PostgreSQL
process restart all pass. Passive-population and singleton receipt regressions
also pass. Evidence: build/population-threat-receipts-{20,30,60,passive,singleton}-v1/
verification.json. Strict workspace/all-target Clippy, fmt, 25 server tests, Python
compile and diff checks pass; see population-threat-receipt-final-checks.log and
population-threat-receipt-server-tests.log. Socket-dependent tests were rerun outside
the restricted sandbox. This adds no OIDC, encrypted-client, corpse-consumption,
native Windows or MMO capacity claim. Next: competing authenticated native clients
with normal durable grants/loot, retaining these separate receipt contracts. Local only.

P3-06 / P3-09 / P3-14 / P5-17 public population attack presentation,
14 November 2026: the normal population launch accepts the existing public attack
flag, selecting the distinct bounded NP01 profile. Pursuing editor account launches
automatically select it; passive definitions retain their existing private setup.
Existing standard Bevy wolf clips resolve each actor's public victim independently,
including remote observers, cancellation, retirement and expiry. Player facing and
selection remain manual. No new attack/damage/loot rules or timing formula.

Three batch presentation contracts pass at 20/30/60 Hz alongside the retained
singleton/private tests. Two actual encrypted native clients observe simultaneous
NPC attacks, correct remote-victim facing and advancing Bevy playback at every
supported rate. The copied native fixture uses 12 m perception and one damage per
strike; it proves presentation, not balance, OIDC or durable competition. Exact
binary hashes, per-NPC playback samples, transport counters and screenshots are in
build/population-public-native-{20,30,60}-v2/qualification.json. Linux RX 7900 XTX /
RADV NAVI31 / Vulkan / Mesa 26.2.6 reports zero GPU diagnostics. No OS input used.

The unchanged pursuing editor export also passes normal owned OIDC/PostgreSQL
launch, 18 frozen source profiles, public attack animation, five-cast kill, both
corpse pickups and database restart/character-picker rejoin. XP 25 and inventory
revision 4 survive; observed transient health is 53 then 82. Evidence:
build/population-public-account-native-v1/qualification.json. All 509 default
network tests (17 fixture-dependent ignored), 265 client tests (four ignored),
22 Python launcher checks (one skipped), strict workspace/all-target Clippy, fmt
and diff checks pass. See build/population-public-adoption-checks.log and
population-public-launch-python-tests.log. Retained authored multi-victim/transfer
and impaired-network fixtures remain separate evidence. Windows, native competing
attackers with durable grants and MMO capacity remain unqualified. Next: exercise
shared damage threat and first-tag durable recipients together. Local only.

P3-06 / P3-09 public population presentation resumed, 14 November 2026:
The user authorizes continued development, local commits and mouse/display use,
superseding the stop checkpoint below. Three explicit batch presentation tests
cover independent remote victims and target transfer, public cancellation and
retirement suppressing old private timing, missing/foreign victims and snapshot
expiry at 20/30/60 Hz. Retained singleton/private and corpse tests pass. Bounded
native diagnostics now expose exact NPC identity, clip/serial, animation playback,
position/facing and resolved public victim. Normal launch remains gated pending
actual GPU qualification. These are client presentation contracts, not new native
or multi-client gameplay evidence. Continue with native multi-NPC animation, then
normal/editor public timing adoption. All 265 default client tests pass (four
ignored); strict workspace/all-target Clippy, fmt and diff checks pass. Evidence:
build/population-public-client-tests.log and population-public-presentation-lint.log.
No push authorized.

USER-REQUESTED STOP CHECKPOINT — 13 November 2026.
Completed gameplay/editor slices are committed: configured population pursuit,
independent retaliation/return/reset/threat, ordinary corpse loot and durable restart.
Public population timing authority, bounded codec and encrypted priority delivery
are committed through 573e4e9c, with 509 network tests and 17 authored checks passing.

The final presentation work is saved as an explicit unfinished checkpoint. It adds
a bounded population frame queue to the native bridge, drains it in combat Update,
and lets existing standard Bevy creature attack cues select their own public actor/
victim, including cancellation, expiry and retired actors. Client/server branches
and profile identity preparation exist, but normal population configuration still
rejects --public-creature-attacks. The editor wrapper does not enable it. This gate
was retained at the user's stop request; public population animation is not adopted.
Six retained creature-visual tests pass; workspace/all-target denied-warning Clippy,
fmt and diff checks pass. Evidence:
build/population-public-presentation-stop-checks.log. No native GPU qualification
was run for this unfinished presentation change. No native multi-NPC animation or
complete public-presentation claim is made.

Restart here: add explicit batch attack-cue tests (independent remote victims,
retirement/private suppression/expiry), expose matching native diagnostics, then
qualify population animation on the actual GPU. Only after those checks pass,
remove the population public-timing launch gate and enable the existing shared flag
through the editor wrapper. Preserve old singleton/public and private fallback tests.
All work stays local. No push authorized. Stop requested: do not resume autonomously.

P3-06 / P3-09 reliable priority delivery for public population timing,
13 November 2026: authenticated gameplay sessions now distinguish singleton and
population timing modes. Population frames use the existing reliable ordered priority
channel, bind presentation state to admitted world/profile/rate, and reach a bounded
frame queue. Movement corrections/snapshots retain rate agreement checks. Whole
frames wait for presentation readiness; unchanged contents are suppressed. Changes
between fixed ticks defer until the next authority tick to avoid same-tick conflicts.
Slow server peers are explicitly retired rather than losing critical transitions.
Both singleton and population timing upload tags are rejected on event and priority
channels. The retained native app construction still selects singleton mode; normal
population/editor public timing remains gated pending standard Bevy presentation.
Two actual encrypted peers receive full five-NPC frames at 20/30/60 Hz under loss,
jitter, duplication and reordering. Both observe the old/new victim windups around
competing accepted attacks, private cancellation and a real incoming strike. Other
NPCs are idle in this transport fixture; multiple simultaneous public authorities
remain the separate authored World test. Synthetic inventory/admission ports are
explicit: no SQL/OIDC, native visuals or durable competing grants claim here.
An authored authority/Renet queue check qualifies readiness, unchanged publication
and full critical-queue retirement. Client saturation/closed-queue handoff and both
upload families on both channels pass. All 509 default network tests pass (17 ignored)
and all 17 authored population checks pass separately. Strict workspace/all-target
Clippy, fmt and diff pass. Evidence: build/population-public-encrypted-session-tests.log,
population-public-session-safety-tests.log and population-public-session-final-checks.log.
Next: standard Bevy animation/event consumption, native observation and then normal
launcher/editor public timing adoption. Local only.

P3-06 / P3-09 bounded client handoff for public population timing,
13 November 2026: added population timing receive validation and a bounded full-frame
presentation queue using the retained critical-event capacity. Profile, world epoch,
world identity and movement/correction rate bind the current presentation state.
Changed validated frames enter the queue before state/counters commit. Duplicates
and stale frames do not add presentation events. Full/closed queues return an error
without committing or silently dropping a critical transition. Future session
integration must treat that error as disconnect, as the retained singleton path does.
Two focused receiver contracts pass: ordinary/duplicate/stale/admission checks at
20/30/60 Hz, and queue saturation, unchanged state, successful handoff after test-only
drain, closed receiver and invalid capacity. Codec bounds/lifecycle tests remain
separate. Strict workspace/all-target Clippy, fmt and diff pass. Evidence:
build/population-public-client-handoff-checks.log. This is a direct receiver/queue
contract, not session/channel wiring, network delivery, UI animation or native proof.
Next: attach this receiver to authenticated client sessions and publish whole frames
on the existing reliable priority channel. Local only.

P3-06 / P3-09 authoritative public population timing, 13 November 2026:
Added an explicit authority-only population timing installer/profile and sorted full
snapshots. Both singleton and population snapshots reuse the same observation builder:
windups require a living NPC, attack-capable pursuit phase and exact living player
life. Population identity is distinct, and singleton/population public modes cannot
be installed together. Full snapshots validate through the bounded population codec.
An actual authored 20/30/60 Hz check observes at least two independent NPC windups,
compares public completion/creature identities with the private victim windup, clears
all windups after departure, and replaces the old NPC ID after ordinary weapon kill
and fresh-life respawn. The decoded presentation state cannot retain its old windup.
All 507 default network tests pass (15 ignored), including retained encrypted public
timing/privacy contracts. The authored population authority check passes separately;
strict workspace/all-target Clippy, fmt and diff checks pass. Evidence:
build/population-public-attack-authority-tests.log and population-public-authority-final-checks.log.
Prerequisites: shared pursuit/retaliation, independent threat/reset and codec 76f3d884.
Normal launcher/editor public attack support remains gated. No population priority
transport, animation, native presentation or capacity claim yet. Next: reliable
priority batch delivery with bounded presentation queues and explicit backpressure.
Local only.

P3-06 / P3-09 bounded public population-windup codec, 13 November 2026:
Added a separate, explicitly identified whole-population timing frame/profile.
Up to 16 sorted exact NPC identities share one authority tick; frames contain only
public windup targets/start/completion times. Maximum encoded size is 582 bytes,
with a compile-time check against the existing message bound. Singleton wire/profile
identities remain distinct. Omitted actors cancel their presentation; a monotonic
entity watermark rejects later resurrection of retired IDs while retaining older
still-live IDs. Empty frames retire all actors without an unbounded tombstone list.
This assumes full admitted-pool snapshots, not interest-area exit/re-entry.
Three codec/state contracts pass at 20/30/60 Hz: maximum simultaneous round-trip,
truncation/foreign/duplicate/invalid/trailing data with unchanged state, and stale
frames, same-tick conflicts, replacement, retirement, empty frames and expiry.
Strict workspace/all-target Clippy, fmt and diff pass. Evidence:
build/population-public-attack-codec-final-checks.log. This is codec groundwork;
no authority publication, network delivery, animation or editor adoption claim.
Next: shared authority observations, priority transport/backpressure and standard
Bevy presentation. Local only.

P3-06 / P3-09 / P5-17 shared threat in normal editor population launches,
13 November 2026: normal population configuration now accepts the existing damage
threat profile with its pursuit/reset/typed credit prerequisites. The editor account
wrapper freezes and selects reset plus threat for pursuing definitions (18 profiles);
stationary definitions retain their prior 16-profile setup. No alternate spawn-group
combat, damage, ownership or loot rules were introduced.
Two actual encrypted peers now qualify a pursuing authored five-NPC population at
20/30/60 Hz under delay, jitter, packet loss, duplication and reordering. Accepted
casts and weapon attacks change the NPC's victim; private old-victim windup cancels,
new-victim windup arrives, and an actual incoming ordinary strike hits the new life.
Both peers retain equal public damage facts and exact damage-to-threat accounting;
queues do not overflow. Synthetic committed inventory/admission ports are explicit:
this encrypted competition fixture is not SQL/OIDC or native rendering evidence.
All 504 network tests pass (14 ignored); all 14 authored population checks pass
separately. Strict workspace/all-target Clippy, fmt and 22 Python launcher tests
pass (one skipped). Evidence: build/population-threat-launch-final-checks.log and
population-threat-encrypted-world-tests.log. The shared fixture preserves the
retained singleton/public timing scenarios and original passive grant fixtures.
Actual exported editor account launch with 18 frozen profiles passes native Linux
admission, checked chase, retaliation, six-cast kill and both corpse pickups.
Threat records six damage facts and exactly 100 health lost, then clears on death.
25 XP and inventory revision 4 (one pelt, 101 tokens) survive PostgreSQL restart and
character-picker rejoin. Observed health is 50 at first exit and 80 after new-runtime
attacks on rejoin; health is transient. Zero GPU diagnostics on RX 7900 XTX / RADV
NAVI31 / Vulkan / Mesa 26.2.6; no OS input automation. Evidence:
build/population-threat-native-v1/qualification.json and fresh/resumed owned launch
records/server logs. Native competition between multiple human clients, contested
durable grants with threat enabled, Windows and MMO capacity remain unqualified.
Next: bounded public attack timing/animation for multiple NPCs, retaining priority
and fail-closed backpressure, plus broader competing durable-gameplay coverage.
Local only.

P3-06 / P3-09 ordinary typed damage into independent threat, 13 November 2026:
The population threat selection check now drives accepted character-facing commands,
equipment observations and ordinary sword-slash cast requests for two player lives
against different NPCs. Each table's threat equals its own target's actual health
lost, unrelated participants are absent, victims remain independent, and departure
transfers the remaining engagement. The fixture uses explicit guaranteed-hit special
policy data to separate integration from chance/balance; no prepared threat facts
or direct table updates drive selection. Later reset isolation still uses its
explicit trusted effect/damage fixture, followed by an ordinary weapon kill.
All thirteen authored population checks pass at their configured 20/30/60 Hz rates,
including retained encrypted grant/loot/privacy fixtures. Strict workspace/all-target
Clippy, fmt and diff pass. Evidence: build/population-threat-typed-world-tests.log.
This qualifies the World command/cast-to-threat path, not competing live network
clients. Normal/editor threat adoption stays gated pending that remaining check.
Local only.

P3-06 / P3-09 independent bounded population threat tables, 13 November 2026:
The shared threat adapter now stores an independent bounded table per configured
pursuing creature. Exact-victim facts, victim/path transfer, participant retirement,
death/return cleanup and fresh-ID rebinding use the existing shared rules. Membership
keeps participants active only for their own enemy's table. Profile identity is
composed once, while diagnostics aggregate table counters and offer exact-NPC views.
Normal population/editor threat launch remains gated pending ordinary typed-cast and
live competing-attacker qualification. This core slice does not enable that flag.
The authored-world 20/30/60 Hz check uses explicit trusted prepared threat facts to
isolate table selection: two enemies choose different player lives, retain separate
100-versus-1 threat contributions, remove the departed life and transfer back to the
remaining life independently. It then passes return/reset isolation, ordinary weapon
kill/credit and fresh-life respawn. The facts are not network damage input or ability
timing evidence. The retained suite caught an old-key window on singleton respawn;
shared respawn rebinding now updates both singleton and population tables immediately.
All 504 network tests pass (13 ignored), the new authored-world check passes, and
strict workspace/all-target Clippy, fmt and diff checks pass. Evidence:
build/population-threat-tables-final-checks.log and population-threat-tables-world-tests.log.
No native, competing live-client, public attack presentation or capacity claim for
population threat yet. Next: ordinary typed damage-to-threat integration and then
normal launcher/editor admission. Local only.

P3-06 / P3-09 victim-bound prepared threat facts, 13 November 2026:
Damage-to-threat preparation now receives the exact victim NPC from the existing
credit fact, validates that encounter, and retains the NPC identity through commit.
A rejected victim cannot change the live table or its counters; preparation remains
side-effect free. All nineteen filtered threat tests, including encrypted competing
attacker/timing scenarios and a new 20/30/60 Hz wrong-victim/valid-commit contract,
pass. Strict workspace/all-target Clippy, fmt and diff pass. Evidence:
build/population-threat-fact-binding-checks.log. This is the damage-fact prerequisite
for independent population tables; normal population threat selection remains gated.
Local only.

P3-06 / P3-09 exact-creature threat-transfer routing, 13 November 2026:
Threat candidate checks, victim lookup and prepared pursuit/retaliation transfer
now accept an exact NPC identity. Prepared transfers retain that identity, and
commit locates the matching authority across the retained singleton and bounded
population states. The existing singleton threat adapter delegates to these shared
operations. Candidate range/LOS, anchor/leash, manual player facing, windup
cancellation and path revision invalidation rules are unchanged.
A 20/30/60 Hz routing contract moves the existing world creature authority into
the pool: preparation leaves its victim unchanged, commit changes only its victim
and cancels its old windup, and an unrelated idle authority remains untouched.
The extra idle state is not an admitted second world creature. No population threat
table/selection or competing-player gameplay claim yet.
Thirteen pursuit tests, eleven retained pure threat tests and all eighteen filtered
threat checks including encrypted competing-player/public timing scenarios pass.
Strict workspace/all-target Clippy, fmt and diff checks pass. Evidence:
build/population-threat-routing-tests.log and population-threat-routing-final-checks.log.
Next: independent bounded threat tables and lifecycle synchronization per NPC,
then normal-launch/editor integration and competing-attacker qualification. Local only.

P3-06 / P3-09 shared per-creature return reset, 13 November 2026:
The existing explicit return-reset adapter now retains one bounded reset latch per
creature life. It validates all transitions before restoring health or removing
effects; returning one NPC leaves neighbouring NPCs unchanged. Reset never changes
entity identity, resurrects a corpse or emits death/grant events. Fresh-ID respawns
start a fresh latch. Singleton threat/public timing adapters retain their explicit
single-encounter prerequisite. Population attacks fail closed after reset errors.
Normal population configuration accepts the reset profile when matching pursuit,
typed damage, membership and credit are installed; passive definitions refuse it.
The editor account wrapper freezes and selects the shared development reset profile
automatically for pursuing definitions (17 profiles, versus 16 for stationary ones).
Bounded package behavior reads reject escaping paths and oversized inputs. Client
reset admission now resolves the same creature definition for patrols/populations.
All 502 network tests pass (12 ignored); all 12 authored population checks pass
separately. The new 20/30/60 Hz check uses explicit trusted damage/effect fixtures
to isolate reset: one returning NPC restores 20 health/removes its effect while a
second remains damaged with its effect; repeated return frames do not reset twice.
Departure clears the other source-owned effect, remaining returns restore 30 health,
and ordinary weapon kill/credit/fresh-life respawn still pass. Seven retained
singleton reset checks, strict workspace/all-target Clippy, fmt and 22 Python
launcher tests pass (one skipped). Evidence: build/population-reset-final-checks.log,
population-reset-controller-tests.log and population-reset-world-tests.log.
Native exported editor account launch, automatic 17-profile freeze, configured
chase/retaliation, eight-cast kill, two corpse pickups, 25 XP, inventory revision 4,
PostgreSQL restart and matching character rejoin pass. The first observed health is
45; after fresh-runtime attacks on rejoin it is 77. Health remains transient. Zero
GPU diagnostics on Linux RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6; no OS input.
Evidence: build/population-reset-native-v1/qualification.json and owned launch.json.
Native qualification proves reset-profile admission and retained combat/loot; the
damaged-return/effect cleanup evidence is the actual authored headless world test.
No native Windows, population threat transfer/public attack presentation or MMO
capacity claim. Next: adapt shared threat selection and target/path transfer to
exact per-NPC engagements. Local only.

P3-06 / P3-09 / P5-17 configured pursuit for editor populations, 13 November 2026:
Spawn groups can now bind definitions with existing aggro settings. The normal
server prepares one shared navigation worker plus independent collision bodies
before admission; matching player-life setup installs the ordinary pursuit and
retaliation controller per creature. Definitions without aggro retain stationary
behavior. A group controls placement/count/respawn, not alternate combat rules.
Pursuing creatures perceive, chase, face their own target in melee, return to their
own anchor and respawn at their checked home with a fresh entity/path identity.
Movement reuses the patrol's static-world controller and prepared coverage. No
search, file access or awaited I/O was added to simulation steps. Membership and
credit now observe each creature's phase; return/blocked phases refuse new attacks
and reset that encounter's credit without changing other creatures' records.
Editor creature inspection resolves pursuing definitions, and export carries those
settings through the existing owned account launch. Existing singleton threat,
evade-health/reset and public attack-stream adapters remain explicitly gated for
populations; this slice does not claim those features or general AI authoring.
All 502 network library tests pass (11 ignored); all 11 authored population checks
pass separately, including encrypted ownership/private grant fixtures. The new
actual-world test runs at 20/30/60 Hz: several enemies chase and retaliate, disconnect
causes independent returns, credit encounters reset, ordinary weapon damage kills,
kill credit belongs to the attacker, and the fresh life returns to its own home
without an old navigation slot. It uses explicit 1000-health players to separate
lifecycle checks from encounter balance. All 242 editor tests pass (11 asset checks
ignored), 20 Python launcher tests pass (one skipped), strict workspace/all-target
Clippy and fmt pass. Evidence: build/population-pursuit-final-checks.log and
population-pursuit-world-tests.log.
The actual headless editor export and native owned account launcher pass with a
pursuing definition: one nearby enemy follows a checked path, attacks, dies after
six casts, leaves two consumed corpse claims, and awards 25 XP. Four bystanders
remain undamaged. Inventory revision 4, one pelt and 101 tokens survive PostgreSQL
restart/character rejoin. Health is transient: 50 at first exit; the new runtime
attacks again after rejoin (81 when observed), not persisted health. Both launches
report acquisition/chase/published routes with zero blocked/failed paths. Zero GPU
diagnostics on Linux RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6; screenshot
inspected, no OS input automation. Evidence: build/population-pursuit-native-v3/
qualification.json, pursuit-evidence.json and fresh/resumed server logs. The first
native attempts exposed old harness assumptions (initially out of combat and a
2.8 m corpse filter); the harness now permits configured acquisition and selects
inside the actual 3 m three-dimensional pickup limit. The v3 supplemental evidence
corrects its older report scope string using saved authoritative metrics.
No native Windows, MMO-scale capacity, population threat/evade-reset/public attack
presentation, or movement beyond prepared collision coverage claim. Next: connect
the retained shared evade/reset and threat adapters per creature, then qualify
multiple competing live attackers and public attack presentation. Local only.

P3-06 / P3-09 shared checked creature displacement, 13 November 2026:
Extracted the existing patrol's body-centre/feet conversion and Avian/static-world
step into a shared creature movement function. It uses the same prepared patch,
coverage boundary, intent and tick rate; failed conversion/controller steps leave
the caller's motion state unchanged. Patrol waypoint, facing, arrival, blocked and
return policies are unchanged. This prepares reuse by independently spawned NPCs;
it does not yet enable population pursuit. Thirteen patrol/movement and twelve
pursuit tests pass, plus strict workspace/all-target Clippy, fmt and diff checks.
Evidence: build/creature-shared-motion-tests.log. Local only.

P3-06 / P3-09 pursuit controller on shared navigation, 13 November 2026:
Moved navigation ownership from each pursuit controller to the combat owner.
The owner pump dispatches through one bounded scheduler and delivers exact-NPC
replies; controllers retain their existing perception/chase/melee/leash/return
policies. Changed engagements can replace obsolete queued work. Death, despawn
and fresh-ID respawn retire old navigation slots. The retained singleton runtime
now exercises this same shared transport instead of owning a private worker.
A two-controller contract passes at 20/30/60 Hz with distinct target lives and
route endpoints. This test does not install or move a second world creature.
All 502 network library tests pass (10 ignored), all six navigation-worker tests,
strict workspace/all-target Clippy and fmt pass. Existing pursuit lifecycle and
encrypted threat/reset regressions remain green. Evidence:
build/pursuit-shared-scheduler-final-checks.log.
Two actual native clients with owned OIDC/PostgreSQL pass the existing chase,
melee, leash and saved-patrol return qualification: one acquisition/return/arrival,
31 published routes, zero blocked/failed paths, zero GPU diagnostics; owned
services stop and private tickets are removed. Linux RX 7900 XTX / RADV NAVI31 /
Vulkan only; no Windows or capacity claim. No OS input automation.
Evidence: build/pursuit-shared-scheduler-native-v1/verification.json.
Prerequisite: bounded dispatcher e3822a18. Population pursuit remains gated until
shared checked per-creature movement and full world engagement tests pass. Local only.

P3-06 / P3-09 bounded shared navigation scheduling, 13 November 2026:
Added a 16-creature dispatcher around the existing dedicated navigation worker.
Each exact creature identity retains bounded latest-request/reply storage;
round-robin dispatch prevents a changing goal from bypassing queued neighbours.
Retirement, superseded revisions and simulation deadlines discard stale results,
including retirement followed by re-registration of the same key. Route failures
remain available to the existing blocked/return policy; no direct-line fallback.
The owner pump only dispatches/polls; fixed steps need not search or wait.
Six real-worker integration checks pass (four new scheduler contracts), all 501
network library tests pass (10 ignored), and strict workspace/all-target Clippy,
fmt and diff checks pass. The sandbox-only library run could not open local UDP
sockets; the subsequent authorized local-socket run passed. Evidence:
build/navigation-scheduler-checks.log and navigation-scheduler-final-checks.log.
Prerequisite: existing prepared Landmass worker and per-NPC retaliation. This is
transport groundwork, not enabled population pursuit or a capacity claim. Next:
move the existing pursuit runtime onto the shared dispatcher, then connect checked
per-creature movement and qualify multiple live engagements. Local only.

P3-06 / P3-09 shared per-NPC retaliation for spawned populations, 13 November 2026:
Normal population launches with player-life/retaliation profiles now install one
bounded independent engagement per spawned creature. All engagements reuse the
existing retaliation windup/recovery, authoritative reach/LOS, player-life checks,
physical/ordinary outcomes and damage application. Provoking one enemy does not
provoke its neighbours. Departure cancels that player's engagement; NPC death and
fresh-ID respawn reset only the corresponding engagement. No database work or
navigation search was added to the simulation step.
Client and server share an explicit population-retaliation identity domain, separate
from the retained passive-life test fixture. Shared combat vitals now read the
actual player life even without the old singleton NPC authority. Private windup
presentation chooses the earliest pending strike deterministically; multi-NPC public
attack streams/animations remain separate. Stationary retaliation does not add
perception, chase, NPC facing changes, threat transfer, evade or return movement.
Two actual authored-world checks pass for physical and ordinary retaliation at
20/30/60 Hz: two independent provoked enemies among five, idle bystanders, target
disconnect, ordinary weapon kill, fresh-ID respawn and a new engagement. Lifecycle
tests use explicit 1000-health players to decouple identity checks from balance.
All 501 network tests pass (10 ignored), alongside strict workspace/all-target
Clippy, fmt, matching client/server builds and 20 Python launcher tests (one asset
check skipped). The initial link failed from disk exhaustion; clearing only stale
incremental caches restored space and the rebuild passed.
Native execution through the frozen editor account launcher and selected loot
catalogues passes: the attacked enemy deals 28 damage, the player kills it in five
casts, four neighbours remain untouched, two corpse pickups commit, and 25 XP plus
revision-4 inventory survive PostgreSQL restart/character rejoin. Player health is
transient (72 before exit, 100 on rejoin), not persisted. Zero GPU diagnostics on
Linux RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6; no OS input automation.
Evidence: build/population-retaliation-native-v1/qualification.json, lifecycle and
final-check logs, and population-retaliation-rebuild.log. Windows, MMO-scale capacity,
live SQL-backed competing attackers, and population pursuit remain unqualified.
Next: reuse shared pursuit/return and checked movement with per-creature state and
bounded navigation scheduling; do not introduce a different population combat model.
Local only.

P3-11 / P5-17 explicit editor item and loot catalogue selection, 13 November 2026:
Account population Start now exposes item and reward/corpse-loot catalogue paths.
Bounded editor reads use the runtime parsers to reject missing/mismatched items,
invalid catalogues and unsupported quest-only loot before export. The owned wrapper
freezes the selected bytes into its existing per-run profile set; per-creature-life
entries enable the ordinary corpse pickup path automatically. Existing training
catalogues remain the defaults. Selection is explicit and does not migrate saved
inventories or bypass server admission compatibility checks.
242 editor tests pass (11 asset checks ignored), 20 Python launcher/freeze tests
pass (one asset check skipped), along with fmt, strict workspace/all-target Clippy,
editor build and Python compilation. All three private PostgreSQL gameplay-profile
contracts pass: incompatible admission preserves state, and the worker remains
available after refusal/cancellation. This is safe refusal, not content migration.
Actual native editor catalogue entry, export, character creation, six-cast kill,
two corpse pickups, Stop, database restart, matching-character selection and Stop
pass. Inventory revision 4 and 25 XP persist, claims remain consumed at revisions
2/3/4, source zones remain unchanged and owned services/tickets clean up. Native
UI input is restricted to the owned editor; gameplay uses ordinary typed requests.
Zero GPU diagnostics on Linux AMD RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6.
Evidence: build/editor-loot-catalogues-native-v1/qualification.json and fresh/resumed
proofs, build/editor-loot-catalogues-regression.log, editor-loot-compatibility-v2/tests.log.
Prerequisites: shared population corpse loot, frozen owned launch, editor account
picker. No new loot rules, general catalogue authoring, saved-data migration,
Windows native, multi-NPC pursuit/retaliation or MMO-scale claim. Local only.

P3-09 / P3-11 shared corpse loot in normal population launches, 13 November 2026:
Remove the population-only rejection of --creature-loot. Normal launcher/client/
server admission now uses the same checked reward catalogue, first-tag recipient,
per-victim-life corpse source, pickup range and durable inventory transactions as
individual creatures. Other singleton AI/reset and quest-loot gates remain explicit.
No new population loot formula or parallel gameplay implementation was introduced.
The native opt-in qualification driver can select an already nearby enemy, use
ordinary equipment/target/cast requests, collect the training offer and submit two
ordinary private corpse pickups. It checks exact victim life, inventory revision,
pelt/token counts and untouched bystanders; it never moves/rotates the player or
writes server health, inventory or positions. The test world authors its spawn near
one checked enemy, rather than bypassing range validation.
Three configuration checks and 18 Python launcher/frozen-profile checks pass (one
asset check skipped). All 501 network library tests pass (8 ignored); the three
actual authored encrypted population credit/private-loot/quest-loot checks pass
separately at 20/30/60 Hz. Formatting, workspace/all-target denied-warning Clippy,
client/server builds and Python compilation pass.
Native normal account launch passes six casts, one death among five NPCs, 25 XP,
one training claim plus two corpse claims, one pelt/101 tokens at inventory revision
4, database restart and exact inventory rejoin without duplicate claims. SQL checks
confirm both corpse claims belong to one victim life and consumed revisions 2/3/4.
Evidence: build/population-corpse-native-v2/qualification.json and fresh/resumed
proofs; build/population-corpse-final-checks.log and population-corpse-network-suite.log.
Linux AMD RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6, zero GPU diagnostics;
no OS input automation. Owned services and tickets clean up. Native Windows, live
competing-account SQL loot, repeated same-session kills and mouse loot-UI acceptance
are not newly claimed. Encrypted competing-player tests use bounded synthetic ports.
The existing frozen editor preset also passes native create/kill/collect/restart/
select and owner-pipe cleanup unchanged (population-corpse-training-regression-v1).
The editor's existing frozen training preset remains unchanged: switching its item
catalogue would invalidate saved-character inventory hashes without a migration.
Explicit loot catalogue selection in editor playtests and safe profile migration
remain follow-up integration, alongside shared multi-NPC aggro/pursuit/retaliation.
Normal launch example uses --population-config, --creature-loot, --rewards,
--reward-catalog content/gameplay/development-creature-loot-rewards.json,
--items content/gameplay/development-loot-items.json and the existing explicit
account/Classic combat/solo-credit prerequisites. Local changes only.

P3-03 / P5-17 editor account population Start/create/select/Stop, 13 November 2026:
The Zone menu now starts an authenticated population playtest directly from the
current zone snapshot. Export does not require enabled Test Players, preserves the
authored player spawn (with normal ground resolution), and omits scripted test
controllers. The standard Bevy character dialog displays the account's bounded list,
validates creation names, and publishes an explicit private create/select choice.
Cancel and the existing Stop action close the owned launcher; editor exit retains
the owner-pipe cleanup. Opening the name field clears its buffer while preserving
Bevy's initialized text layout, so selection/replacement and editing keys work.
The launcher uses the frozen reviewed development combat/equipment/XP/reward preset;
it requires configured OIDC and PostgreSQL services, not a Test Player identity.
All 241 editor tests pass (11 asset-dependent checks ignored in the general run),
as do separate actual account/network population export checks, cargo fmt --check,
workspace/all-target denied-warning Clippy, editor build and Python compilation.
Native actual menu/export/create/select/Stop passes with owned OIDC/PostgreSQL:
invalid name rejected, corrected name created, six-cast NPC kill, 25 committed XP,
collected training reward, database restart, existing-character selection, identical
revision-2 inventory, no duplicate creation, and clean Stop/ticket cleanup in both
runs. Both source documents remain unchanged. Screenshots inspected; zero GPU
diagnostics on AMD RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6 (Linux).
Evidence: build/editor-account-ui-native-v4/qualification.json, fresh/resumed UI and
game captures, database-proof.txt, and build/editor-account-ui-final-regression.log.
Prerequisites: frozen account launcher plus checked passive population export and
private account handoff. This completes their editor UI integration, not general
quest/item/creature catalogue authoring. Multi-NPC aggro/pursuit/retaliation, normal
per-life corpse/quest loot launch, production player auto-pathing, Windows native
verification and MMO-scale capacity remain separate work. Changes local only.

P3-03 / P5-17 frozen account population launcher, 13 November 2026:
A dedicated owned wrapper now launches an editor-exported population with a frozen
per-run development gameplay setup: sixteen exact files for typed combat, equipment,
shared life, solo first-tag credit, XP and training rewards. It records file hashes,
uses the normal launcher/preflight and authenticated character picker, and reserves
an isolated runtime directory under the owned output. The normal launcher accepts
explicit progression/reward catalogues so those files are also frozen; conflicting
bundle overrides and catalogues without their modes refuse. Test Player controllers
are not used by this one-account launch. NPC pursuit/retaliation remains absent.
Three freeze/isolation tests, fourteen population launcher tests (one asset check
skipped), all 13 authored-launch and 49 bundle checks, Python compilation and diff
checks pass. Native execution through the actual wrapper passes first-character
creation, six-cast kill, committed 25 XP/training collection, database restart,
character picker rejoin, identical revision-2 bag and owner-pipe shutdown. All sixteen
frozen hashes match on both runs. Zero GPU diagnostics on RX 7900 XTX / RADV NAVI31 /
Vulkan / Mesa 26.2.6; owned services/tickets/passwords are cleaned up. No OS input.
Evidence: build/editor-account-package-native-v1/qualification.json and each run's
owned/launch.json/profiles/runtime. This is the tested launcher behind the planned
editor account Start action; picker widgets and menu wiring remain next. Local only.

P3-03 / P5-17 first-character creation through the owned handoff, 13 November 2026:
The private picker now advertises creation availability and accepts an explicit
create_name choice as an alternative to an existing character ID. Empty accounts
wait for a choice instead of refusing. Eight-character accounts refuse creation;
no timeout/cancellation creates a character implicitly. Creation shares the normal
authenticated API and validates the current 3–20 ASCII character name policy and
returned identity. Accepted creation is durable server state, not a client fixture.
Six handoff tests and all 16 real localhost account tests pass, including malformed
names rejected before HTTP, explicit empty-account creation, account limits and
preserved terminal selection. Thirteen population launcher checks pass (one asset
check skipped), Python compilation and diff checks pass.
Native owned OIDC/PostgreSQL creation, equipment, kill, reward collection, database
restart and existing-character picker rejoin pass with no terminal or OS input.
Server counters report exactly one creation initially and none on rejoin; committed
25 XP, consumed reward and inventory revision 2 persist. Both sessions close through
the owner pipe and remove tickets. Zero GPU diagnostics on RX 7900 XTX / RADV NAVI31 /
Vulkan / Mesa 26.2.6; owned services/password files are cleaned up.
Evidence: build/population-account-create-picker-native-v1/qualification.json and
fresh/resumed runtime character/selection files and database proofs. Editor picker
widgets and the account-backed Start action remain subsequent integration. Local only.

P3-03 / P5-17 private account character-picker handoff, 13 November 2026:
Normal play can publish a bounded authenticated character list into its exclusively
reserved session directory, then await a bounded schema-1 selection file instead
of terminal input. Picker mode requires account login, an owner pipe and a new
session directory, and excludes an explicit character-ID override. Only IDs from
that account's validated list are selectable; actual admission still goes through
the ordinary server ownership/profile checks. Display data contains names/IDs only,
is published atomically with private permissions, and excludes provider/session
credentials. Selections must be published atomically by the caller. Empty accounts
currently refuse; creation and the editor picker UI are subsequent work.
The normal terminal menu now uses the same bounded validated list. Fifteen real
localhost account tests, five handoff tests, thirteen population launcher tests
(one asset check skipped), 13 authored-launch and 49 bundle tests pass. Coverage
includes invalid/duplicate IDs, unlisted and oversized selection, cancellation,
timeout, unknown fields and private publication. Python compilation/diff pass.
Native owned OIDC/SQL kill/collection, restart, private picker rejoin and owner-pipe
shutdown pass: 25 XP, consumed reward and identical inventory revision 2 persist.
No OS input is automated. RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6 reports
zero GPU diagnostics; owned services/tickets/password files are cleaned up.
Evidence: build/population-account-picker-native-v1/qualification.json and resumed
runtime/characters.json/selection.json. Local only; no editor picker UI claim yet.

P3-01 / P3-03 / P5-17 isolated account launch sessions, 13 November 2026:
Normal play accepts --session-directory and reserves that directory exclusively
with private permissions for logs and temporary tickets. Existing directories are
refused before certificates/logs can be removed or overwritten. The default CLI
location remains compatible. Population account qualification now uses distinct
fresh/resumed runtime directories instead of the shared development-session log.
Twelve launcher tests pass (one asset check skipped), including existing-file
preservation and cleanup for default/isolated sessions; all 13 authored-launch and
49 gameplay-bundle regressions, Python compilation and diff checks pass. Native
owned OIDC/SQL kill/collection, database restart, explicit-character rejoin and
owner-pipe shutdown pass in isolated directories. XP remains 25, inventory revision
2 and the consumed training reward persist; ticket directories are removed.
RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6, zero GPU diagnostics, no OS input.
Evidence: build/population-isolated-account-native-v1/qualification.json and each
run's runtime/server.log, database proofs and screenshots. This supplies session
isolation for future account-backed editor launches, not an editor account picker.
All changes remain local.

P3-01 / P3-03 / P5-17 owned account launcher shutdown, 13 November 2026:
Normal play now optionally stops on its owner's stdin EOF. Account mode requires
explicit character selection so the lifetime pipe cannot consume an interactive
character prompt. Cancellation is checked before launch, through startup/admission
and while running. Account requests stop on cancellation, but challenge cancellation
and logout remain allowed; a cancelled admission response cannot write a ticket.
The existing managed launcher shares the small pipe watcher without behavior change.
Fourteen real localhost HTTP tests pass, including login challenge cleanup and
cancellation during ticket admission; eleven population launcher tests pass (one
asset check skipped), plus all 19 managed-launch, 13 authored-launch and 49 bundle
tests. Python compilation and diff checks pass. Real native account rejoin with the
owner pipe open succeeds, then pipe closure exits cleanly and removes tickets while
preserving committed 25 XP, consumed training reward and inventory revision 2.
The native run covers joined shutdown; during-login cancellation is covered by the
HTTP tests. No OS input is automated. RX 7900 XTX / RADV NAVI31 / Vulkan /
Mesa 26.2.6, zero GPU diagnostics; owned services/password files are cleaned up.
Evidence: build/population-owned-account-stop-native-v1/qualification.json.
This is launcher lifecycle support, not yet an editor account-picker UI. Local only.

P3-01 / P5-17 explicit account character entry prerequisite, 13 November 2026:
The normal play launcher now accepts --character-id for an existing character.
It validates a nonzero UUID and requires account login before process/file startup,
then uses the ordinary authenticated server admission operation instead of a terminal
selection prompt. No character is implicitly created or selected and the existing
interactive flow remains available. This supplies a launcher prerequisite; the
editor does not yet have an account character picker or account-backed Start action.
Native owned OIDC/PostgreSQL validation passes with stdin closed on explicit rejoin.
The same character keeps its revision-2 equipped bag and consumed training claim;
a second kill after restart raises XP to 50 with two receipts and no duplicate item.
Ten population launcher tests pass (one asset check skipped), along with all 13
authored-launch and 49 gameplay-bundle tests, Python compilation and diff checks.
Native RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6 runs report zero GPU diagnostics;
owned services/tickets/password files are cleaned up. No OS input is automated.
Evidence: build/population-selected-character-native-v1/qualification.json and
fresh/resumed database and combat reports. Changes remain local.

P5-17 native editor resource-cast acceptance, 13 November 2026:
The bounded population command qualifier now also accepts the editor's resource-only
combat context without requiring or inventing persistent equipment or membership.
It selects an already frontal in-range NPC and uses normal target/cast requests,
watching exact actor/NPC lives, resource spending and four untouched bystanders.
After scene readiness and bounded warmup, the real editor-exported package passes:
one accepted cast, zero rejected requests, one damage event, target 100→80 HP and
minimum mana 75 from 100 with normal subsequent regeneration. Player health stays
100, generation 1, and no inventory exists. Screenshot confirms the selected damaged
wolf and 25-mana action. An initial ungated run was rejected once during startup;
the final v2 check requires a clean first request. No OS input or pose changes.
The preserved account-backed typed kill/collection/restart path also passes after
the shared driver refactor: 25 XP, consumed reward and identical revision-2 bag.
Formatting, strict workspace/all-target Clippy, client build, Python compilation
and diff checks pass. Both native modes report zero GPU diagnostics on RX 7900 XTX /
RADV NAVI31 / Vulkan / Mesa 26.2.6. Owned services/tickets are cleaned up.
Evidence: build/editor-population-resource-cast-native-v2/qualification.json and
scene.png; build/population-resource-mode-typed-regression-v1/qualification.json;
build/editor-population-resource-cast-build.log. No membership/typed damage claim
for the resource-only mode. All changes remain local.

P5-17 managed population mana-cost combat, 13 November 2026:
Editor population exports now freeze the existing resource-aware Arcane Bolt
catalogue (25 mana per cast) alongside their already exported character stats and
shared life profiles. Managed playtests therefore advertise the real ability cost
instead of the prior free development spell. This remains basic damage combat with
development tickets; equipment, typed first-tag grants and durable inventory still
require the separate account-backed launch. No new account bypass is introduced.
The actual offline-baker export test, formatting, strict workspace/all-target
Clippy and editor build pass. Fresh headless export plus two actual managed native
clients pass: five wolf rigs, two admissions, completed authored controller, three
public-life frames, zero rejected inputs/recoveries and clean shutdown. A separate
native screenshot confirms the 25-mana action and health/mana UI with five wolves.
No new cast-execution claim is made by these launch/UI checks. No OS input is used;
the source zone is unchanged and owned processes/tickets are cleaned up.
RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6, zero GPU diagnostics; managed
1280x720 clients and 1600x1000 visual check. Evidence:
build/editor-population-mana-build.log,
build/editor-population-mana-native-v1/verification.json and
build/editor-population-mana-visual-v1/qualification.json/scene.png.

P3-10 / P3-11 population repeat kill after restart, 13 November 2026:
The native normal-launch qualifier can now kill again after database/server restart
and fresh account login. Both exact server epochs defeat one NPC in five casts;
four bystanders and the player's 100 HP remain unchanged in each run. The second
life grants another 25 XP: SQL and client agree on 50 total and two XP receipts.
The already consumed training claim stays at revision 2, inventory stays byte-equal
with 100 tokens, and no second reward is offered or consumed. Both combat membership
entry/exit checks pass. This tests repeat kills across server epochs, not timed
respawn within one running server. No OS input is automated. Python compilation
and diff checks pass; native RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6 checks
report zero GPU diagnostics. Owned services/passwords/tickets are cleaned up.
Evidence: build/population-repeat-kill-native-v1/qualification.json and both runs'
typed-combat.json, database-proof.txt and consumed-proof.txt. Changes remain local.

P3-10 / P3-11 / P5-17 native population training reward collection, 13 November 2026:
The opt-in native qualifier now collects the committed once-per-character training
offer through the normal client reward request. Actual account-backed population
kill and collection pass: six casts defeat one NPC without harming four bystanders,
25 XP commits, the offer disappears and training tokens increase from 99 to 100.
Inventory revision 2 and the SQL consumed-claim revision agree. The worker reports
one equipment commit and one reward consumption. PostgreSQL process restart and
fresh login preserve the identical bag, XP and consumed claim with no offered reward.
This verifies ordinary training rewards on the checked editor population package;
it does not enable per-life corpse pickup, competing native players or default
editor account-backed playtests. No OS mouse/keyboard or movement/facing automation.
Formatting, strict workspace/all-target Clippy, client build, Python compilation
and eight launcher tests pass (one actual-assets check skipped). Native fresh/rejoin
runs pass with zero GPU diagnostics on RX 7900 XTX / RADV NAVI31 / Vulkan /
Mesa 26.2.6, 1280x720. Owned services/tickets and password files are cleaned up.
Evidence: build/population-reward-collect-native-v1/qualification.json, fresh and
resumed consumed-proof.txt/database-proof.txt, and
build/population-reward-collect-build.log. Changes remain local.

P3-09 / P3-10 / P3-11 / P5-17 normal population durable grants, 13 November 2026:
Normal client/server and play launcher now accept progression and training rewards
for a checked passive population when explicit solo first-tag credit is configured.
Missing credit refuses before launch; population grant options no longer implicitly
add the singleton patrol. Existing account, inventory and combat prerequisites remain.
The bounded native qualifier can now defeat one NPC using ordinary equipment,
target and cast requests, waiting for mana and checking exact lives and bystanders.
Actual editor-exported five-NPC package passes with owned OIDC and PostgreSQL:
five casts defeat one NPC, four bystanders remain unharmed, player health stays 100,
and membership enters/exits. The client observes committed 25 XP and one unconsumed
training reward offer. SQL confirms one XP receipt and one claim. Database process
restart and fresh login preserve both grants and equipped inventory revision 1.
No OS input or movement/camera manipulation is used. This is one native player and
one defeated life; competing first-tag ownership is covered by separate encrypted
and SQL fixtures, not a native multi-account contest. Loot consumption, population
quest interaction, NPC retaliation and richer default editor playtests remain open.
Configuration/launcher tests, Python compilation, client/server builds, formatting
and denied-warning workspace/all-target Clippy pass. Native screenshots inspected;
zero GPU diagnostics on RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6, 1280x720.
Owned services and tickets are cleaned up. Evidence:
build/population-grants-native-v1/qualification.json, fresh/typed-combat.json and
fresh/resumed database-proof.txt; build/population-grants-launch-build.log and
build/population-grants-authority-final.log. All changes remain local.

P3-09 / P5-17 native typed population cast without OS input, 13 November 2026:
An explicit bounded client qualifier now exercises the normal equipment, target and
cast request paths. It selects a healthy NPC already within the player's frontal
spell range, never moves/turns the player or camera, requests the committed training
wand equip, and permits at most three casts. It watches the exact player/NPC lives,
refuses changed lives/bystander damage and waits for combat membership to clear.
The actual normal account-backed launch passes: one Arcane cast spends mana and
reduces one NPC from 100 to 81 HP; four bystanders remain at full health. Player
health stays 100 and generation 1. Membership enters then exits after hostility grace.
The inventory owner commits exactly one equip at revision 1. PostgreSQL process
restart and fresh OIDC rejoin preserve the identical equipped inventory. Native
screenshots show the selected damaged NPC and equipped wand. No OS pointer, keyboard,
movement or facing automation is used; this is normal network-command execution,
not a new physical input acceptance or NPC kill/reward/retaliation claim.
Formatting, strict workspace/all-target Clippy, Python compilation and client build
pass. Both native runs have zero GPU diagnostics on RX 7900 XTX / RADV NAVI31 /
Vulkan / Mesa 26.2.6, 1280x720. Owned services/processes/tickets are cleaned up.
Evidence: build/population-typed-combat-native-v1/qualification.json and
fresh/typed-combat.json; build/population-typed-combat-build.log.

P3-09 / P5-17 normal typed-population profile admission, 13 November 2026:
Normal population client configuration now binds physical damage to the checked
standalone creature definition and accepts outgoing ordinary/typed ability profiles
without inventing a pursuit controller. The shared population gate permits combat
membership and explicit solo first-tag credit; singleton pursuit/reset/public NPC
attacks and unqualified progression/loot launch combinations remain refused.
Configuration reports expose the installed typed-combat capabilities.
The new owned qualifier uses the actual play launcher, OIDC login, PostgreSQL-backed
inventory and native encrypted client. Fresh admission and database process restart/
rejoin pass with five NPCs, player health 100, out-of-combat membership and identical
inventory at revision 0. Both logins verify; inventory ensures and exact gameplay
profile checks succeed. Native screenshots after bounded graphics warmup show all
five wolf rigs, health/mana, combat state and Arcane Bolt/Sword Slash actions. Initial
captures were too early for GPU pipeline readiness; the final qualifier waits for
asset/readiness observations plus a bounded warmup before capture.
This qualifies real account-backed profile admission and UI, not native typed hits,
first-tag grants or NPC retaliation. No OS mouse/keyboard input was automated.
Private services/processes/tickets are cleaned up; password files are removed.
Configuration tests, launcher regressions, Python compilation, formatting, strict
workspace/all-target Clippy and client/server build pass. RX 7900 XTX / RADV NAVI31 /
Vulkan / Mesa 26.2.6, 1280x720, zero GPU diagnostics. Evidence:
build/population-typed-native-v3/qualification.json and its configuration/fresh/resumed
artifacts; build/population-typed-launch-build.log.

P3-09 / P3-10 / P3-11 population first-tag SQL receipts, 13 November 2026:
The owned PostgreSQL competing-hitter qualifier now also loads the checked five-NPC
population package produced by the editor. Two public World actors contest one NPC;
the first effective tagger retains the XP/reward recipient while another actor lands
the fatal typed Arcane hit. Both players stay at 100 health and the other four NPCs
remain unharmed. This deliberately retains the existing unequipped/no-mana Arcane
fixture and synthetic account/quest acceptance; it is not normal equipment balance.
The original owner receives 25 XP, one legacy SQL quest count and one unconsumed
training claim. The finisher receives none. A held SQL row lock prevents successful
acknowledgement while the fixed simulation advances; duplicate receipts/claims do
not multiply, and a replay under the finisher is refused. Connection and PostgreSQL
process restart preserve both recipients' exact state with fsync, synchronous commit
and full-page writes enabled. Owned password files are removed on cleanup.
This covers one defeated life of the five-actor population, not ten competing lives,
SQL-backed encrypted clients, normal population quest interaction or loot consumption.
The population branch routes trusted World recipient facts to the isolated legacy
SQL lifecycle fixture without installing the singleton lifecycle interaction mode.
The preserved saved-terrain/patrol qualifier also passes after this refactor.
Formatting, strict workspace/all-target Clippy, Python compilation and example build
pass. Evidence: build/population-credit-durable-v1/verification.json,
build/singleton-credit-durable-regression-v1/verification.json and
build/population-credit-durable-build.log.

P5-17 managed population player-life integration, 13 November 2026:
Population Start playtest now freezes character stats, the shared life policy and
player respawn data beside the exported creature/combat files. Export validates the
life binding before publication; the managed launcher preflights both life hashes
before starting children and passes identical files plus public-life replication to
the server and every client. Legacy packages can still launch without the explicit
player-life option. Missing/unvalidated life context refuses before session creation.
The actual offline-baker export test verifies unchanged frozen bytes and authored
players/routes. All 238 editor tests pass (the socket-restricted encrypted regression
was rerun successfully with localhost access); 10 asset checks remain ignored in the
regular suite. Nineteen managed-launcher tests, Python compilation, formatting,
strict workspace/all-target Clippy and editor build pass.
Headless editor export followed by two actual native clients passes using the new
package: five passive wolf rigs, two encrypted admissions, movement controller
completion, three public-life frames, zero input rejections/recoveries/backpressure
disconnects and zero GPU diagnostics. The bounded session ends at its duration;
owned processes/tickets are removed and the source zone is unchanged. No OS input
was automated. This qualifies the shared export/managed-launch path, not a new
mouse-driven Start/Stop menu acceptance or NPC retaliation. Native Linux Vulkan:
RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6, two 1280x720 clients.
Evidence: build/editor-population-life-native-v1/verification.json,
build/editor-population-life-build.log and build/editor-population-life-final.log.

P3-09 / P5-17 normal passive-population player-life launch, 13 November 2026:
The ordinary client/server population path now accepts paired player-respawn and
retaliation-policy profiles to bind shared player life, using the exact checked
standalone creature bytes from the population package. It installs no NPC retaliation
or pursuit. The normal Python launcher selects resources without inventing a patrol
or navigation override. Public player-life replication is supported; richer population
membership/credit/progression/loot launch gates remain in place.
Headless configuration reports expose both life profile hashes and validate the real
schema-3 five-NPC package without starting renderer or transport. An owned native
Linux client/server check passes: five wolf rigs, health 100, mana 100, generation 1,
zero player deaths and one live public actor. Screenshot inspection confirms the
health/mana/alive interface; no OS mouse or keyboard automation was used. The visual
qualifier now serializes its duration as integer seconds, as required by the client,
so fractional CLI parsing cannot silently fall back to the 65-second default.
Configuration tests, all seven launcher tests including actual binary preflight,
Python compilation, formatting, strict workspace/all-target Clippy and client/server
build pass. Native Vulkan validation has zero diagnostics on RX 7900 XTX / RADV
NAVI31 / Mesa 26.2.6, 1600x1000. Owned processes and tickets are cleaned up.
Evidence: build/population-life-configuration.json, build/population-life-native-v2,
build/population-life-launch-build.log and build/population-life-launch-final.log.
Managed editor export of these life profiles is the next integration step.

P3-09 / P3-11 population quest-loot transport, 13 November 2026:
The encrypted authored-population fixture now covers quest-conditioned private loot
and public player-life replication. At 20/30/60 Hz, the original tag owner's exact
fatal-life event routes to the journal queue; a competing finisher gets neither
quest credit nor a pickup entitlement. Existing full-bag/retry, successful private
pickup and spent-claim refusal checks pass. Both clients observe both living players
while one of five passive NPCs dies; the four bystanders retain full health.
Three actual encrypted population checks pass with deterministic UDP impairment.
This uses synthetic inventory/progression owners and tests routing, not durable
quest eligibility, SQL transactions, player death/respawn presentation or normal
launcher adoption. The three preserved single-patrol credit regressions, formatting
and strict workspace/all-target Clippy pass. Evidence:
build/population-quest-loot-check.log and build/population-quest-loot-final.log.

P3-09 / P3-10 / P5-17 population reward routing, 13 November 2026:
Rewards now bind the checked population creature definition and resolve the exact
fatal life and authoritative corpse position, removing the singleton patrol-only
assumption. Existing recipient policy, fixed-step work bounds and durable-owner
responsibilities remain unchanged. Wrong creature profiles and repeated installation
refuse without changing identity. Across five NPCs, eight players and two generations
at 20/30/60 Hz, first-tag reward ownership, duplicate-fact suppression, exact corpse
positions and rejection of retired-life facts pass using actual authored assets.
Two additional encrypted two-client checks use the real checked five-NPC population
at all three rates with delayed/reordered/lost/duplicated UDP. Only the first tagger
receives XP/reward work; the competing finisher cannot pick up the private corpse.
Full-bag retry, successful pickup, stale-claim refusal and unchanged bystander health
pass. The two-hit transport fixture explicitly sets one NPC to 20 HP; it does not
establish normal balance. Inventory/progression owners are synthetic committed ports:
this is no new SQL durability, OIDC, native UI or ordinary-launch adoption claim.
Prerequisites are checked population placement, shared player lives, typed combat,
combat membership and exact-life first-tag credit, qualified in preceding slices.
All seven authored-population checks and 501 regular network tests pass (seven
asset-dependent checks ignored in the regular suite). Formatting and strict
workspace/all-target Clippy pass; rendering is unchanged. Evidence:
build/population-rewards-final.log, build/population-encrypted-credit-check.log
and build/population-rewards-lint.log.

P5-17 standalone creature context native editor qualification, 13 November 2026:
Owned native Linux checks now cover File -> Load creature definition, selecting an
unresolved spawn group, Use loaded creature, undo/reapply and population export.
Loading itself leaves the draft unchanged; assignment is undoable. The loaded file
prepopulates export without manual path replacement. Invalid spacing refuses before
publication; the five-creature package matches the current draft while original zone,
owned source file and creature definition bytes remain unchanged.
A second native check preloads --creature-definition and starts a managed population
playtest from Zone. Two encrypted clients are admitted, the test controller finishes,
state traffic is sent with no rejected inputs/recoveries, and Stop cleans processes
and tickets. The standalone creature and wolf GLB are frozen byte-for-byte in the
package. No new rich-combat/first-tag normal-launch claim is made by this legacy
passive playtest. Inspection found stale guidance claiming population export was
unconnected; the inspector now points to File export and Zone playtest, and generic
export directs spawn-group users to the population workflow.
All 238 editor tests pass (10 asset-dependent checks remain ignored); the focused
export guard, fmt, strict workspace/all-target Clippy and rebuild pass after final
wording. Native checks have zero GPU diagnostics on RX 7900 XTX / RADV NAVI31 /
Vulkan device 29772, editor 3440x1440. Evidence: build/editor-creature-context-native-v1,
build/editor-creature-context-playtest-native-v1, build/editor-creature-context-final.log
and build/editor-population-guidance-final.log. Source files remain unchanged.

P4-03 / P3-11 checked native quest travel, 13 November 2026:
The owned native authored introduction and three-kill follow-up now pass using the
checked client follower for both long-distance legs and ordinary keyboard alignment
near contacts. Outward travel takes 53.664 s and return 52.835 s across 136 points,
with no route failures, no player deaths and no OS mouse camera steering for travel.
All three kills count, four quest actions apply without rejection, and turn-in ends
at revision 11 / 125 XP / two completed quests. Owned PostgreSQL process restart and
fresh OIDC rejoin preserve the exact result without duplicate actions/rewards.
An earlier run exposed death during endpoint alignment: the qualifier now watches
exact player-life generation during walking/turning, releases owned keys immediately,
and bounds restart/return-via-respawn handling. An initial repair used the UI key
helper's 100 ms hold floor and overshot small turns; the final repair adds observations
to the existing precise gameplay hold. Five focused tests cover death, fast respawn,
no key press from a dead start, short-turn timing and release after observation error.
Native success had zero deaths; recovery branches are unit-tested, not newly native-
qualified. Python compilation passes; Rust binaries rebuilt from the previously
fmt/Clippy/network-tested tree. RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6,
1280x720, zero GPU diagnostics. Private services close. Evidence:
build/authored-quests-checked-travel-native-v4/{qualification.json,fresh/,resumed/}.
This qualifies test travel, not player auto-pathing, automatic combat facing,
area-quest travel or population first-tag normal-launch adoption.

P3-09 / P5-17 public avatar life for passive populations, 13 November 2026:
Public player-life admission now binds the shared Lives owner instead of requiring
a single NPC retaliation controller. It retains healthy-authority, exact identity,
pre-admission and duplicate-install checks, including paired life errors.
All five authored population fixtures verify empty initial rosters, five/eight
connected players, and an isolated trusted player death after the NPC lifecycle
checks. Respawn advances only that player's generation, other records stay exact,
disconnect removes only that player, and the existing bounded wire round-trips the
public roster at 20/30/60 Hz. This test damage isolates life projection; it is not
population NPC retaliation. All 501 ordinary network-library tests, five authored
fixtures, fmt and strict workspace/all-target Clippy pass. Evidence:
build/population-avatar-{checks,final}.log. No new wire format or normal client/
launcher adoption; no new rendered avatar claim. Validation was headless. The user
subsequently reauthorized mouse/display use for needed native checks.

P3-09 / P3-10 / P5-17 passive population first-tag ownership, 13 November 2026:
Explicit credit installation now admits checked passive typed/membership populations.
Bounded lifecycle sync retains independent live ledgers, resets only departed owners'
unfinished claims and removes retired NPC identities; fresh lives get empty entries.
Paired pursuit/reset behavior remains unchanged. Completed facts retain their owner;
the existing solo-recipient adapter still checks current owner admission separately.
The authored fixture tests five taggers plus three distinct finishers within the
existing eight-player cap, leaving two uncontested controls. Three competitors cannot
steal tags or solo recipients; each fatal ledger totals exactly 100 health lost with
two contributors for contested NPCs. Both generations pass at 20/30/60 Hz, with exact
actor lives, independent membership grace and no inherited tags. A registry test
covers departure isolation, replacement, invalid identity-set refusal and retained
finished ownership. Initial ten-player fixture correctly hit the admission cap;
the revised fixture preserves the cap. All five authored population tests, all 501
ordinary network-library tests, fmt and strict workspace/all-target Clippy pass.
Evidence: build/population-credit-checks-v2.log and build/population-credit-final.log.
This qualifies opt-in World ownership/recipient facts, not durable competing-player
population rewards, normal launcher flags, NPC pursuit or native/editor adoption.
No display access. Earlier single-player SQL last-hit qualification remains separate.

P3-09 / P5-17 live passive-population combat membership, 13 November 2026:
Explicit membership admission now supports checked passive populations with both
ordinary and typed ability damage installed. A complete bounded encounter snapshot
synchronizes each NPC independently; passive NPCs supply hostility grace but do not
invent pursuit/active-encounter reasons. Death and fresh-ID replacement retire only
that exact opponent. Existing single-pursuit admission and active reasons remain.
Five-player authored special/projectile fixtures verify membership is empty before
resolved attempts, present during each surviving fight and cleared at death and
respawn. Under-capacity and duplicate policies fail without identity changes; no
incoming or active-pursuit reason appears. All four authored population fixtures
pass at 20/30/60 Hz; all 500 ordinary network-library tests, fmt and strict workspace/
all-target Clippy pass. Evidence: build/population-membership-{checks,final}.log.
This installs no population ownership/credit, NPC pursuit/retaliation or normal
launcher/client/editor flags. Headless only; existing membership wire is unchanged.

P3-09 / P5-17 passive population typed ability damage, 13 November 2026:
Explicit typed ability installation and resolution now accept the checked passive
physical/ordinary population context. Both melee specials and arcane projectile
impacts validate exact live creature identities and use population defensive facing;
existing pursuit readiness/return behavior is preserved. Wrong ability-catalogue
bytes and duplicate installation fail without changing World identity.
Four separately selected authored fixtures pass at 20/30/60 Hz: physical swings,
ordinary swings, sword specials and cast-time arcane projectiles. Five players use
required equipment, each resolves one typed ability against its NPC then ordinary
swings finish the life; both generations compare exact resolved draws/results and
health loss with the existing profiles. All ten ability facts per rate preserve
player life identity; stale NPCs are refused, and no incoming strikes occur.
All 500 ordinary network-library tests, fmt and strict workspace/all-target Clippy
pass. Evidence: build/population-abilities-final.log. This does not install passive
population membership/credit, NPC retaliation/pursuit, normal launcher flags or
editor controls. Existing fixture numerical rules remain; no display access.

P3-09 / P5-17 passive population ordinary melee outcomes, 13 November 2026:
Explicit ordinary weapon outcomes now admit checked passive physical populations
with healthy shared player-life authority. Exact live creature-facing lookup keeps
patrol orientation and defines stationary population zero yaw (-Z), matching the
existing stationary presentation without target tracking. Outgoing eligibility and
resolution use exact bound target identity; the incoming pursuit adapter is unchanged.
The five-player/five-NPC authored fixture checks two generations at 20/30/60 Hz,
each server-resolved draw/result/health change against the existing ordinary profile,
no incoming strikes, stable player lives, fresh NPC IDs and retired facing refusal.
Additional supported front/back probe positions confirm defensive orientation. Public
attack telegraphs and typed ability installation remain explicitly pursuit-gated.
All 500 ordinary network-library tests and both separately selected authored physical/
ordinary population tests pass; fmt and strict workspace/all-target Clippy pass.
Evidence: build/population-ordinary-final.log. No new formula coefficients, native
rendering, population pursuit/retaliation, first-tag rewards, launcher flags or editor
controls are claimed. Headless only; authored population headings remain future work.

P3-09 / P5-17 passive population physical melee, 13 November 2026:
Explicit physical-damage installation now accepts checked homogeneous passive
populations with shared player lives and exact equipment bindings. Outgoing weapon
eligibility/resolution checks the exact live creature type instead of requiring a
patrol. Existing paired patrol requirements and incoming retaliation are retained.
The real authored western-grove fixture uses five equipped players and five NPCs:
weapon-timed swings kill two generations at 20/30/60 Hz; each hit matches the existing
physical reference calculation, actual health loss and exact player life. Fresh
NPCs are matched by spawn position rather than death/identity allocation order;
retired target selection fails and player health/lives remain unchanged. Missing
player-life context, mismatched item bytes and repeated installation leave identity
unchanged. All 500 ordinary network-library tests and the separate asset-dependent
population test pass, along with fmt and strict workspace/all-target Clippy.
Evidence: build/population-physical-final.log. This uses existing fixture formulas;
no new Classic coefficients, outgoing avoidance tables, population retaliation,
first-tag rewards, normal launcher flags or editor controls are adopted. No display.

P3-09 / P5-17 bounded combat-membership registry prerequisite, 13 November 2026:
Membership now keeps exact creature keys in a bounded registry instead of one NPC
field. Player observation retires missing NPC lifetimes independently, resolves
only the specified opponent and preserves other active/hostility reasons. Refused
observations leave the original player state unchanged. Empty, duplicate, mixed-
epoch, over-target-budget and over-policy-capacity registries are rejected.
Two new registry tests exercise retirement, replacement without inherited hostility,
independent resolution, grace expiry and stale observations at 20/30/60 Hz. Existing
World pursuit/typed-damage membership and encrypted owner-private tests regress.
All 500 network-library tests, fmt and strict workspace/all-target Clippy pass.
The initial restricted filtered run denied two private UDP sockets; the full
headless rerun passed with localhost access and display variables removed.
Public membership admission still requires the single typed pursuit encounter;
this internal prerequisite does not enable passive-pool membership or first-tag
rewards. Evidence: build/membership-registry-final.log. No display access.

P3-09 / P5-17 passive population player-life admission, 13 November 2026:
An explicit pre-admission World API installs shared player lives for checked passive
populations with matching creature behavior and combat resources. Standalone and
legacy creature inputs retain exact source identities; canonical behavior permits
validated population binding. Missing combat, mismatched stats/behavior and repeated
installation fail without changing World identity. Five players retain their exact
lives and full health across five NPC deaths/fresh-ID respawns at 20/30/60 Hz.
Nine wire tests and all 498 network-library tests pass, along with fmt and strict
workspace/all-target Clippy. The updated owned PostgreSQL fixture records ten distinct
NPC lives, 250 XP and ten quest counts; duplicate replay adds nothing, simulation
advances during a blocked SQL award, and process restart retains receipts. Shared
player life remains healthy throughout. Evidence: build/population-player-lives-
{checks,final}.log and build/population-durable-v3/verification.json.
This opt-in headless API adds no NPC retaliation, pursuit, typed population damage,
membership, first-tag policy or normal launcher/client adoption. The SQL fixture
retains its explicit single-player development last-hit scope. No display access.

P3-09 / P5-17 shared player-life ownership prerequisite, 13 November 2026:
Player profile/admission, health liveness, respawn counters/errors and private
life observations now have a separate Lives owner in combat authority. NPC
retaliation retains its own engagement/pursuit state; the existing paired install
API still requires the matching single patrol and publishes both together.
Existing profile identity and reported aggregate counters remain compatible.
Respawn errors retain the paired adapter's fail-closed encounter behavior.
An internal checked-World fixture removes the NPC controller before player join:
health admission, trusted death and respawn still work at 20/30/60 Hz, with the
same player session, fresh life/input generations and supported home within 0.1 mm.
The public-avatar unavailable-authority regression now removes the actual Lives
owner. All 498 network-library tests, fmt and strict workspace/all-target Clippy
pass, including existing paired retaliation, typed combat and encrypted life flows.
This exposes no population life install, new damage authority, public protocol,
normal reward flag or client behavior. No native/display check was performed.
Evidence: build/shared-player-lives-final.log.

P3-09 / P5-17 indexed creature-credit state prerequisite, 13 November 2026:
Credit authority stores bounded per-creature encounter entries, including ledger,
resolution serial, departure/reset flags and immutable fatal credit. Cast capture,
white-melee capture, damage preflight/commit and recipient lookup address the exact
victim identity. New targeted observation methods reject player/retired identities;
legacy unaddressed observations require one entry. Existing profile identity and
singleton pursuit admission/synchronization remain, including serial continuity
across respawn. No population credit/life/membership admission or reward flags change.
Independent-ledger checks verify separate owners, departure/reset generations,
retained finished credit, stale epochs, duplicate/empty/over-capacity refusal.
Actual World targeted observations and stale respawn queries regress alongside
competing hitters, delayed impacts, resets and encrypted recipient routing.
All 497 network-library tests, fmt and strict workspace/all-target Clippy pass.
The initial restricted test run refused three UDP socket checks; the complete
headless rerun permits private localhost sockets and removes display variables.
Evidence: build/creature-credit-index-final.log. Next integration still needs
per-population player life, typed combat/membership and lifecycle synchronization;
the registry alone is not live multi-creature first-tag adoption.

P3-10 / P3-11 / P5-17 population durable receipt qualification, 13 November 2026:
The new headless population_durable_qualification example loads a real checked
five-actor export and drives one synthetic player through ten fatal lives across
fresh-ID respawns. Actual World XP and quest facts agree and all ten KillIds are
distinct. The existing asynchronous Worker commits 250 XP and ten quest counts;
replaying every fact in reverse and forward order adds nothing. A held progression
row lock prevents award acknowledgement while ten further World ticks advance.
Fresh Store connections verify the ready quest/count and XP. The owned PostgreSQL
runner then restarts the process and verifies 250 XP, journal revision 11 and ten
receipts in each kill path; fsync/synchronous_commit/full_page_writes are on.
Plaintext password files are removed. Initial v1 refused a fixture World origin
that differed from the saved player spawn; v2 uses the authored spawn and passes.
This is explicitly a one-player development last-hit fixture with synthetic
acceptance and a ten-kill quest variant. It neither establishes population first-tag
eligibility nor enables normal reward launch flags, OIDC, encrypted clients, native
UI, quest turn-in or loot. Per-creature ownership remains the next reward prerequisite.
Example build, fmt, strict workspace/all-target Clippy and Python compilation pass.
Evidence: build/population-durable-build.log; build/population-durable-v2/
{verification.json,world-postgres.log,restart-rows.txt}. No display access.

P5-17 creature context startup and headless validation, 13 November 2026:
Editor --creature-definition FILE seeds the same direct-load context. Headless mode
reports its ID/canonical path without creating a window. Context validation runs
before prepare can write an explicit --output; missing/invalid inputs preserve an
existing output file. Export modes refuse this context flag and continue using
--population-creature for their explicit binding. Two real-binary Python checks
pass with display variables removed, covering standalone/legacy files, read-only
source bytes, failure-before-output and unambiguous export refusal. Editor build,
fmt and strict workspace/all-target Clippy pass; native startup remains unverified.
Evidence: build/editor-creature-cli-{checks,runtime}.log.

P5-17 direct creature loading in the editor, headless checkpoint, 13 November 2026:
File → Load creature definition uses the existing bounded JSON browser and accepts
standalone or validated legacy creature files without a quest template. Loading
preserves document/history/saved state; bad files preserve the previous context.
Use loaded creature applies the existing undoable spawn-group ID edit. The inspector
refreshes on context changes and shows the source file; an explicitly loaded type
takes precedence over a gameplay template's creature. Population export/Start seeds
its creature field from that path. Session context is not stored as a zone dependency.
All 238 editor tests pass (ten asset-dependent checks remain opt-in), including the
actual private UDP export regression, read-only browser failure handling, assignment/
undo, inspector cache refresh and export field default. Fmt and strict workspace/
all-target Clippy pass. Initial checks exposed a test fixture already using the
same creature ID and sandbox socket refusal; both were resolved before acceptance.
The final run removed DISPLAY/WAYLAND_DISPLAY. No mouse/display or native rendering
check was performed; new menu layout and native interaction remain unverified.
Evidence: build/editor-creature-context-final.log.

P5-17 standalone population creature definitions, 13 November 2026:
Population loader and editor export accept bounded schema-3 creature type files
without a patrol route. Explicit can_deflect and existing behavior bounds remain;
unknown/route fields, missing required values and unsupported schemas refuse.
Legacy schema-1/2 patrol files still validate before extracting the type. Common
validation is shared, and equivalent legacy/standalone types keep the same canonical
population identity. Patrol and gameplay-manifest inputs retain their old contract.
The new development-creature-passive.json fixture has no aggression; this does not
enable roaming, pursuit, player life, typed pool damage or first-tag rewards.
Seventeen creature tests and two editor export unit tests pass. Actual baked
network snapshot reload with two test players/checked route passes headlessly.
The five-actor western-grove World fixture uses the standalone file at 20/30/60 Hz,
verifying combat, XP/quest facts, fresh-ID respawn and identity compatibility.
Fmt and strict workspace/all-target Clippy pass. No display or rendering change.
Evidence: build/standalone-creature-{checks,integration}.log.

P3-09 / P5-17 damage definition binding prerequisite, 13 November 2026:
Physical damage profile validation now accepts the validated creature Definition
rather than requiring its patrol Profile. Existing client, gameplay bundle and
World callers pass that definition; patrol authority admission guards remain.
A detached-type regression verifies unchanged damage/context identity and foreign
creature refusal. No population typed damage, life, first-tag or reward policy is
enabled by this prerequisite. All 493 network-library tests pass, including real
private UDP checks; the initial sandbox-only run refused socket creation and was
rerun with local socket permission. Fmt and strict workspace/all-target Clippy pass.
All work is headless. Evidence: build/damage-definition-binding-final.log.

P4-03 headless authoritative quest-route qualification, 13 November 2026:
The opt-in follower now has a real authored-World regression: shared Classic
controls emit ordinary queued commands through authority and Avian collision,
with owner-side terrain polling outside fixed simulation. The contact-v3 village/
grove trail completes all 136 grounded points outward and back at 20/30/60 Hz.
Outward takes 52.8–53.2 simulated seconds and return 53.17–53.7; endpoint error
is at most 0.25 m, neutral input stops movement, and recoveries remain zero.
A turn budget derived from route bends rejects repeated waypoint spins. Thirteen
controller unit tests pass, two asset-dependent tests remain opt-in; the new
actual-route test passes separately. Fmt and strict workspace/all-target Clippy
pass. No renderer, display, mouse, authenticated transport, quest UI or durable
reward acceptance is claimed. Full native checked-route quest flow remains pending.
Evidence: build/quest-route-authority-final.log. Reproduce with THEIA_ROUTE_WORLD
and THEIA_ROUTE_FILE, then cargo test -p theia-client
actual_route_walks_out_and_back_through_authority_at_all_rates -- --ignored --nocapture.

P3-11 / P5-17 population quest-reference and occurrence prerequisite, 13 November 2026:
Quest catalogue validation now recognizes the installed passive population's creature
type independently of its live actors. Reward/document/area alias checks use the
same installed-type lookup. Trusted quest kill occurrences resolve exact combat
incarnations after target retirement, deduplicate the batch and reject old lives
after fresh-ID respawn, matching the XP fact path. No quest acceptance, durable
award, first-tag ownership or population CLI/export policy is enabled here.
Actual western-grove navigation/World checks at 20/30/60 Hz install a supported
guide and catalogue alongside five actors, reject a foreign kill objective without
changing identity, and verify five matching XP/quest facts, duplicate handling and
stale-life rejection. All 492 network-library tests, fmt and strict workspace/all-
target Clippy pass. All work and verification are headless under the user's renewed
mouse/display reservation. Evidence: build/population-quest-facts-checks.log.

P4-03 checked quest-travel wiring, headless checkpoint, 13 November 2026:
Shift+F7 explicitly restarts an opt-in checked route outward, including after a
return-leg interruption or player respawn. It clears the old input-incarnation
context, requires focused activation and retains the original checked-start guard;
no position, authority or normal player-control change. F7 retains its existing
next-leg/manual behavior. Completed-route status exposes the new shortcut.
The existing authored quest qualifier accepts --checked-travel to prepare a paused
route and use the follower for long village/grove legs. Short endpoint/combat
alignment uses bounded ordinary A/D holds instead of camera corrections. Ordinary
quest/UI/combat commands remain. Holds release on exceptions and reject foreign
focus/invalid duration. Cursor readiness now uses one telemetry snapshot per poll.
Thirteen controller tests and actual contact-v3 world/navigation route preparation
plus wrong-floor refusal pass. Two Python input-helper safety checks use mocks with
no X connection; Python compilation, fmt and strict workspace/all-target Clippy pass.
The native attempt reached login and the paused 136-point route, but stopped during
cursor readiness before any quest action. User resumed mouse/display use and now
requires headless work; owned test services/processes are closed. Full checked-route
quest/combat/area completion, restart/rejoin and updated native status rendering
remain unverified. Preserve the earlier completed manual-travel quest evidence.
Evidence: build/quest-checked-travel-{build,headless-checks,final-headless}.log;
build/authored-quests-checked-travel-native-v1/ (incomplete attempt, not acceptance).

P3-10 / P5-17 population fatal-hit definition prerequisite, 13 November 2026:
Trusted XP fact collection can resolve each bound passive population incarnation
through combat health after death removes targetability. Respawn replaces that
identity, so replaying the old fatal batch no longer resolves it. Progression
profile admission accepts the installed population definition and still rejects
foreign recipe IDs. Existing patrol recipient policy and profile identity remain.
This creates bounded facts only: population progression CLI remains explicitly
refused, no durable grant/quest integration or first-tag policy expansion is claimed.
The adopted first-tag recipient policy must be connected per population actor before
normal pool rewards are enabled; existing development fatal-hit policy is not a
new production reward decision.
Actual navigation/World tests kill five independent actors at 20/30/60 Hz, verify
five actor/victim/type facts, duplicate-batch deduplication, untargetable dead actors
and stale-fact rejection after fresh respawns. The initial smaller native-combat
fixture passed those cases but could not satisfy the retained over-capacity spatial
fixture; the complete check passes on the established western-grove valley fixture.
All 492 network-library tests, fmt and strict workspace/all-target Clippy pass.
No rendering change or new GPU/SQL/Windows claim.
Evidence: build/population-xp-facts-{checks,valley-check,network-regressions}.log.

P5-17 / P2-13 saved particles in authored population playtests, 13 November 2026:
Population exports now retain saved particle placements through the existing
world exporter. The predicted authored-world store admits manifest-bound particle
catalogues and verifies visual/effect/texture dependencies before simulation.
Client preflight retains existing resource/decoded-image budgets. Standard particle
presentation waits for authored scene/GPU readiness and anchors placements through
checked f64 global coordinates across local render-frame changes. F9 affects only
world effects. No navigation, gameplay collision, authority or fixed-step I/O added.
Eight authored-world tests, eight particle tests and the actual export/dependency
regression pass; fmt and strict workspace/all-target Clippy pass. Headless combined
package/preflight passes with one effect/five actors; missing effect, changed
published effect and source-isolation checks pass. The initial broad network test
command was interrupted while linking unrelated binaries and replaced by focused
library tests. An early export check correctly exposed the old authored-store
particle refusal, now replaced by explicit validated dependency admission.
Native editor assignment/undo, export, two-client Start and Stop pass with copied
appearance and particle dependencies; checked route takes 11.86 s, final authority
error 0.181 m. Native combat repeats approach, five casts, independent death and
fresh-ID respawn, then checked return. Four other actors remain unchanged. One
live effect records 1501 visible emitter frames, zero unready frames; normal F9
hide/restore passes twice with unchanged health. No dropped creature cues, rejected
inputs or movement recoveries. Screenshot review found the effects toggle over
the diagnostic HUD; it now uses the lower right in graybox/authored sessions and
the corrected native screenshots were inspected. Linux RX 7900 XTX / RADV NAVI31,
Vulkan device 29772, validation clean; editor 3440x1440/client 1600x1000. Original
review editor/source assets untouched; owned processes/tickets close. This is a
bounded authored package, not streaming, cosmetic-load/capacity, quest-credit or
Windows qualification.
Evidence: build/authored-particles-{final-checks,layout-checks}.log;
build/population-particles-package-v1/qualification.json;
build/editor-population-particles-native-v1/{qualification.json,finished.png};
build/population-particles-combat-native-v2/{qualification.json,effects-hidden.png,effects-restored.png}.

P5-17 / P4-03 native population combat after checked travel, 13 November 2026:
Owned native client follows 27 checked points, releases controls, selects through
normal Tab and casts five normal Arcane Bolts (100→80→60→40→20→0). Four other
creatures retain full health. The defeated actor respawns after the fixture's
three-second deadline with a fresh entity ID at the same home; retired selection
clears. Six rig attachments, 108 hit frames, 102 dead frames, no dropped cues.
F7 returns along the checked route. Outward 11.56 s, return 12.08 s; no OS mouse
camera/movement steering, rejected inputs or movement recoveries. Read-only opt-in
OS qualification telemetry now includes actor health/identity, visual metrics and
route state; tools/qualify_population_combat.py owns and cleans up its processes.
The harness activates only its still-PID-owned client before normal key actions;
two earlier attempts stopped safely when desktop focus changed during long waits.
Screenshots inspected at 1600x1000; Linux RX 7900 XTX / RADV NAVI31, Vulkan device
29772, validation clean. Fmt, strict workspace/all-target Clippy and client build
pass. This uses the development schema-1 ability and one passive pool, without
quest credit, pursuit or production combat-facing qualification. The exported
snapshot is copied before overriding respawn duration; source package unchanged.
Evidence: build/population-native-combat-build.log and
build/population-native-combat-v3/{qualification.json,selected.png,dead.png,respawned.png,returned.png}.

P5-17 / P4-03 frozen creature appearance in population playtests, 13 November 2026:
Population export/Start dialogs now offer an optional creature GLB. Editor
--creature-asset seeds that field and also works with headless population export.
The worker copies validated bytes as creature.glb into the owned snapshot and
records their SHA-256; managed population clients receive that copy. Source edits
cannot change the published copy. Appearance is client-only and does not change
population authority/identity, spawn bindings, health or model-ID resolution.
The shared current-adapter envelope check accepts bounded embedded GLB 2 assets
(8 MiB file, 2 MiB aligned JSON, one binary buffer/scene/mesh/material/skin) with
exactly Idle/Walk/Run/Attack/Hit/Death named clips. External buffer/image URIs,
truncation, malformed chunks and incompatible adapter shape refuse. Bevy remains
the actual mesh/skin/animation loader and performs runtime data validation.
Client configuration reports expose selected appearance without starting rendering
or networking; managed session reports retain the copied path and selection.
236 ordinary editor tests pass (10 optional ignored), plus the actual navigation/
appearance snapshot and unsafe-floor refusal test. Shared asset tests pass in both
apps, five dialog tests and 17 launcher regressions pass; malformed asset export
refuses before baking/publication. The refusal fixture's remaining-input count was
corrected after adding its second input file. Fmt and strict workspace/all-target
Clippy pass. Actual client preflight accepts the wolf and refuses malformed GLB.
Native editor assignment/undo, optional appearance field, package export, two-client
Start, patrol arrival and Stop pass. First client arrives in 11.85 s with 0.181 m
authority error and manual release; frozen appearance is recorded for both clients.
The first automation attempt hit startup menu timing; the harness now waits for
stable menu geometry. Separate native playback from the exported appearance loads
five animated wolf rigs, with five idle transitions and no dropped cues. Screenshots
inspected at editor 3440x1440 and client 1600x1000. Linux AMD Radeon RX 7900 XTX
(RADV NAVI31), Vulkan device 29772, validation clean. Source files and original
review editor untouched; owned children/tickets close. No generic model catalogue,
pursuit/loot/quest binding or native Windows acceptance claim.
Evidence: build/population-appearance-{build,checks,refusal,launcher-tests,final-checks}.log;
build/population-appearance-preflight-v1/qualification.json;
build/editor-population-appearance-native-v2/{qualification.json,dialog.png,package};
build/population-frozen-appearance-native-v1/{qualification.json,scene.png,client.log}.

P5-17 / P4-03 editor-authored test-player travel, 13 November 2026:
Test Player property actions now cycle next/previous authored patrol paths, using
stable path IDs in the existing controller reference. Assignment preserves feet,
uses ordinary undo/redo and displays the path name. Manual/walk-turn remain choices.
Population playtest export bakes navigation first, prepares a grounded route from
the player's checked landing through the assigned path, then freezes a per-player
route in the snapshot. Runtime uses normal client input with explicit return legs.
Client and editor share grounded-route preparation, strict source/floor/clearance
checks and existing 8192-sample/256-point budgets. No navigation or file loading
moves into the fixed step. Ordinary network export refuses patrol controllers with
an explicit requirement for population-playtest navigation; missing/deleted patrol
references refuse, disabled players do not create route work. No schema bump.
235 ordinary editor tests pass (the local UDP case rerun with socket access), plus
actual navigation snapshot/unsafe-floor refusal checks. Failed route export leaves
no published destination and preserves the previous package. Eleven controller
tests and the actual rolling-valley/wrong-floor regression pass; 17 launcher tests,
fmt and denied-warning workspace/all-target Clippy pass. A fixture schema mistake
was fixed before the actual snapshot acceptance; earlier logs preserve that failure.
Native editor assignment, undo/reassignment, invalid-spacing refusal, Start with
two real clients, route arrival and Stop all pass. First client follows 29 points
in 11.87 s, final authority error 0.182 m, releases input. Second uses its existing
walk-turn controller with 40 ms/way delay and 5 ms jitter. Five passive actors,
zero rejected inputs/recoveries, clean owned-process/ticket shutdown, source files
unchanged. RX 7900 XTX/RADV Vulkan validation clean; 3440x1440 editor screenshot
inspected. This moves test players, not NPC patrol AI or full quest/kill automation.
Evidence: build/editor-test-route-{checks,refusal-checks,launcher-tests}.log;
build/shared-grounded-route-checks.log;
build/editor-patrol-test-native-v1/{qualification.json,dialog.png,finished.png,package}.

P4-03 explicit checked test return leg, 13 November 2026:
Route documents may set return_trip=true. After arrival releases input, F7 starts
the same statically checked points in reverse; after returning, F7 may start outward
again. No automatic transition or interaction/attack is added. Manual cancellation
or failure restarts the current leg, never silently switches its destination, and
the current leg's checked-start proximity guard remains mandatory. Per-leg status
and telemetry identify outward/return and retain elapsed/turn/failure observations.
Eleven controller tests pass (one optional fixture ignored); fmt and strict
workspace/all-target Clippy pass after replacing an unnecessary checked unwrap.
Owned native Linux run starts paused, walks outward in 11.83 s, releases input,
then starts the reverse route through a second normal focused F7 key. Return takes
12.76 s, final authority is 0.182 m from the original start, controls released.
Five population actors remain admitted; RX 7900 XTX/RADV Vulkan validation is clean,
1600x1000 final status screenshot inspected. No camera/movement OS steering.
Evidence: build/test-route-return-checks.log and
build/test-route-return-native-v1/{qualification.json,native/qualification.json}.
This is a bounded static return path, not moving-target replanning or a completed
quest/kill migration. Combat displacement beyond the checked start still refuses.

P4-03 opt-in paused test-route start, 13 November 2026:
A checked route may set start_paused=true. Navigation/collision preparation still
runs before startup, but ordinary input remains available until the focused F7
key starts the route. This permits quest acceptance before test travel. Default
routes still start automatically. Arrival/failure releases ordinary input as before;
F7 restart still requires the original checked start. Configuration reports expose
whether the prepared controller starts active. Ten controller tests pass (one
optional fixture ignored), fmt and denied-warning workspace/all-target Clippy pass.
Owned native qualification observes four stationary inactive reports with zero
travel elapsed, starts through the normal F7 key, then completes in 11.86 s with
0.183 m final authoritative goal error and automatic manual release. Five actors,
RX 7900 XTX/RADV Vulkan, no GPU diagnostics; original editor untouched.
Evidence: build/test-route-paused-checks.log and
build/test-route-paused-native-v1/{qualification.json,native/qualification.json}.
The helper tools/qualify_test_route_paused.py creates a separate route copy and
owned qualifier process group; no source route edits or game-state injection.
Full quest/target/kill orchestration remains separate work.

P4-03 normal-launcher checked test travel, 13 November 2026:
tools/play.py accepts --test-route for one authored Classic client, preflights
that route with the actual client before server/login startup, and forwards it
only to the client. Multiple clients must use per-player managed scenario routes.
No route means unchanged manual play. Thirteen authored launcher tests and
17 managed launcher regressions pass. Actual normal-launcher Linux native run
completes the checked route in 11.83 s with 0.183 m final authoritative error,
releases input, retains five population actors and shuts down owned processes.
RX 7900 XTX/RADV Vulkan validation reports no diagnostics.
Evidence: build/test-route-play-{tests,managed-regressions}.log and
build/test-route-play-native-v1/{qualification.json,launcher.log,server.log,scene.png}.
This makes route travel available to the launcher used by account-backed quest
qualification; the complete quest sequence has not yet been migrated.

P4-03 checked-route input handoff, 13 November 2026:
Arriving or failing a checked route now releases ordinary player input automatically.
The terminal route result remains visible; F7 explicitly restarts with the same
checked-start requirement. Legacy repeating step controllers are unchanged.
Read-only observations continue after release so server corrections can settle:
the first native check exposed a stale pre-arrival authority observation (0.348 m)
and the reporting fix preserves both terminal state and later authority evidence.
Nine controller tests pass (one optional fixture ignored), workspace all-target
Clippy with denied warnings and fmt pass. Native Linux Vulkan repeat arrives in
11.88 s, final authority error 0.183 m, active=false, five admitted creature rigs,
clean GPU validation. Status screenshot inspected on RX 7900 XTX/RADV at1600x1000.
Evidence: build/test-route-handoff-checks.log and
build/test-route-handoff-native-v2/{qualification.json,scene.png,client.log,server.log}.
This enables subsequent ordinary interaction; no full quest-flow migration claim.

P4-03 rolling-ground test-route preparation, 13 November 2026:
Long path segments now project onto their continuous navigation/collision floor
at 0.1 m intervals before retaining roughly metre-spaced follower points. Strict
navigation layer sampling, grounded capsule checks and final segment-clearance
validation are retained. Endpoint floor changes refuse; budgets remain 8192
projection samples and 256 retained points. This changes test-route preparation,
not actor authority or ordinary NPC path behavior. The initial straight shortcut
near the valley cottage still refuses because the capsule has no safe support.
Three waypoint-guided valley approaches now pass preflight instead of failing
because linearly interpolated endpoint heights disagree with rolling terrain.
Actual rolling-route regression checks short retained segments and wrong-floor
refusal; strict workspace/all-target Clippy/fmt pass. Native real server/client
follows the western approach through authored scenery/terrain: 111 grounded
points, 48.93 s, about 0.108 m goal error and 2.575 radians total requested turn
(about 148 degrees). Authority receives 1554 inputs without rejection or movement
recovery. No OS character/camera input. Screenshot inspected at 1600x1000;
RX 7900 XTX/RADV Vulkan validation has zero GPU diagnostics. Native qualifier
now accepts a bounded --seconds 1..300 observation window for longer routes.
Evidence: build/test-route-grounding-{build,tests}.log;
build/test-route-input-v1/valley-*-grounded.log;
build/test-route-valley-direct-grounded.log;
build/test-route-valley-native-v1/{qualification.json,scene.png,client.log,server.log}.
The player can now approach the populated grove through checked test travel;
quest/NPC interaction, explicit target/range binding and kill automation remain
separate follow-ups. No universal route coverage or performance/capacity claim.

P4-03 / P5-17 per-player checked routes in managed playtests, 13 November 2026:
Schema-1 test scenarios may assign a client-only route file, exclusive with a step
controller. Route files resolve relative to the scenario; the existing byte/path
bounds apply. The actual client preflights every assigned route before any owned
process or session output is created. Server spawn profiles remain unchanged.
Seventeen launcher tests pass, including invalid-route preflight and child cleanup.
Actual Linux Vulkan session admits one manual and one route-following client;
the latter has 40 ms each-way delay/5 ms jitter, reaches the checked three-segment
goal within 0.176 m in 11.94 s and requests 3.063 radians of turn. Manual client
receives no controller. Both admit normally, with no rejected inputs or movement
recoveries; managed stop removes tickets/children without cleanup errors. GPU
validation logs are clean on RX 7900 XTX/RADV. Evidence:
build/test-route-managed-tests.log; build/test-route-managed-native-v1/{qualification.json,
launcher.log,session/}. This is external scenario support; editor route authoring
and full NPC/quest/target interaction automation remain follow-ups.

P4-03 / P2-04 checked test-route travel foundation, 13 November 2026:
The native client accepts explicit --test-route FILE, exclusive with the existing
step controller. Startup loads exact-source authored navigation, computes paths,
grounds their points and checks sampled capsule clearance before rendering or
connecting. The bounded follower emits ordinary Classic walk/turn input, uses
predicted feet for steering, and reports authoritative correction feet separately.
No pose teleport, actor facing bypass or OS mouse-driven travel is used.
Configured movement keys, Escape or mouse buttons yield to manual control; F7
also returns to manual. Incarnation/input-stream changes, inability to act, network
errors, route departure, stalls, lost readiness and total travel limit stop input.
Test-only tolerances are 0.25 m arrival, 1 m start/corridor, 8 s no progress,
10 s lost readiness and 600 s total; these do not change gameplay movement rules.
Pure steering tests cover 20/30/60/144 FPS, yaw wrap, bounded turns and failures.
All 255 client tests pass (two optional skipped); nine Classic-control tests and
strict workspace/all-target Clippy/fmt pass. Real headless preflight rejects wrong
floor/source, zero-length routes and conflicting controller flags. Native real
server/client completes the 28 m, three-segment route in about 11.91 s, ends
0.175 m from the goal and requests 3.058 radians total turn (about 175 degrees).
Authority reports 728 inputs with zero rejection/recovery; five passive creature
rigs remain admitted. RX 7900 XTX/RADV Vulkan/KHRONOS at 1600x1000 has no GPU
diagnostics. Screenshot inspected; test status moved clear of the target HUD.
Evidence: build/test-route-{build,final-checks,final-lint}.log;
build/test-route-input-v1/preflight-results.json;
build/test-route-native-v2/{qualification.json,world.json,scene.png,client.log,server.log}.
This completes static-route travel only. Managed per-player route selection,
obstacle-rich native runs and the complete NPC/quest/approach/kill flow with target
invalidation remain follow-ups; normal player manual-facing policy is unchanged.

P3-07 / P5-17 editor-owned population playtests, 13 November 2026:
Zone -> Start population playtest exports one checked passive group plus enabled
test players and starts the managed server/client session. The seven-field dialog
checks profile, spacing and player assets; repeated starts propose fresh snapshot
folders. Zone -> Stop network playtest and editor owner-exit use existing cleanup.
Shared scenario export checks every start against actual static collision, copies
controller scripts, and binds starts to the exported static world identity. The
snapshot retains the full source zone and an immutable development combat catalog.
Plain passive export still refuses test-player presets; explicit CLI combination
--population --network-test produces the same snapshot without launching a server.
Acceptance: 232 editor tests pass (ten optional skipped), actual population/network
snapshot reload passes, combined real CLI export passes, and fmt plus strict
workspace/all-target Clippy pass. Owned native 3440x1440 editor starts two real
1280x720 Classic clients; second peer has 40 ms each-way delay/5 ms jitter and
completes its normal-input walk/turn/strafe/jump controller. Both receive combat
and targeting states. Explicit Stop cleans children/tickets; source bytes and
normalized saved zone agree. RX 7900 XTX/RADV device 29772 Vulkan/KHRONOS logs
have zero GPU diagnostics. Dialog screenshot inspected. Initial harness expected
an info log hidden at default verbosity; its timeout closed the owned editor and
independently verified owner-exit cleanup. Fixed harness uses real server admissions;
the complete rerun passes. No OS-driven character/camera movement was needed.
Evidence: build/population-network-{snapshot-tests,cli}.log;
build/population-editor-playtest-{regressions,final-checks}.log;
build/population-editor-playtest-native-v2/{qualification.json,dialog.png,package/runs/};
v1/owner-exit-check.json. Reusable tool: tools/qualify_editor_population_playtest.py.
This remains passive combat targets/respawns, not pursuit, quest publication,
model-catalog resolution, Windows execution or capacity qualification.

P3-07 / P5-17 managed multi-client population launcher, 13 November 2026:
tools/network_playtest.py now accepts --population-config with optional explicit
--combat-catalog and client-only --creature-asset. First validate the scenario's
static-world identity, then preflight the complete checked population/combat
configuration before creating any owned server, relay or client. Existing local
player positions, per-peer impairment, owner EOF and stop cleanup remain in use.
Fourteen launcher tests pass, including failed population preflight, forwarding
and cleanup. An actual managed Linux Vulkan session admits two native clients to
five passive actors; one peer adds 40 ms each-way delay and 5 ms jitter. Server
reports two test admissions, two combat and targeting states, no refused inputs
or movement recoveries; 25-second duration stop removes tickets and all owned
children without cleanup errors. RX 7900 XTX/RADV Vulkan validation logs are clean.
Evidence: build/population-managed-launcher-tests.log;
build/population-managed-v1/{qualification.json,launcher.log,session/}.
This is launcher support; editor snapshot/start wiring follows. No movement,
combat exercise, capacity or Windows execution claim in this check.

P4-03 / P3-07 population navigation at authored world edges, 13 November 2026:
Route-free area baking now distinguishes absent clipped surfaces and a successful
bake with no walkable polygons from invalid input/backend failures. Only the first
two cases omit a tile; an entirely unwalkable area still refuses. Existing patrol
baking keeps its nonempty-tile requirement. Maximum coverage remains nine tiles.
The regression fixture uses an exported world's corner: four ground tiles and the
real high collision-envelope corner remain; four empty neighbours are omitted.
Checked placement admits five reachable ground actors, preserves source state,
and an out-of-world bake leaves no output. The height patch alone is not the map
boundary: editor exports retain their existing surrounding ground apron.
Eleven ordinary baker tests pass; actual valley tiles remain byte-identical.
Actual edge-package export/reload and real server/headless client with two encrypted
peers pass. Strict workspace/all-target Clippy and fmt pass. Evidence:
build/population-{small-area-tests,small-area-valley,edge-world-export,edge-cli,
edge-connected,edge-clippy}.log; build/population-edge-package-v1.
Also completed native client follow-through for the preceding editor-dialog
package: five visible rigs/animation players, five idle transitions, no dropped
cues or GPU diagnostics, screenshot inspected at 1600x1000 on the RX 7900 XTX
Vulkan device. Evidence: build/population-dialog-playable-v1. No new movement,
pursuit, quest, Windows execution or population-capacity claim.

P3-07 / P5-17 native passive population export controls, 13 November 2026:
File and document menus now offer Export passive population, with explicit
profile, navigation executable and spacing alongside output identity/revision.
The existing asynchronous export worker runs the checked package publisher;
keyboard traversal includes all six fields and invalid spacing/missing profile
refuse before starting work. No source save or server launch is implicit.
Acceptance: 232 editor tests pass (eight optional skipped), all five export-dialog
tests and strict workspace/all-target Clippy/fmt pass. Owned native Linux editor
at 3440x1440 successfully refuses invalid spacing then exports a five-actor package;
source bytes and saved-zone semantics are preserved. Screenshots inspected.
RX 7900 XTX (RADV NAVI31), device 29772, Vulkan with KHRONOS validation: zero
GPU diagnostics. The user's pre-existing review editor is untouched.
Evidence: build/population-dialog-{tests,regressions,final-checks}.log;
build/population-dialog-native-v1/{qualification.json,state.json,dialog.png,
finished.png,editor.log,package/}. Reusable native qualifier:
tools/qualify_editor_population_export.py. Existing passive-only scope and
bounded navigation coverage remain; no Windows native qualification.

P3-07 / P5-17 checked passive population package export, 13 November 2026:
The editor headless export now accepts --population with an explicit creature
profile. It exports the static world, retains the complete spawn-zone document,
runs the external route-free navigation baker, and validates actual placement
through the same startup loader used by server/client before publishing a new
package. Exactly one enabled passive group is supported (runtime cap 16);
quest contacts/areas, particle/test-player presets, model IDs and aggression
refuse explicitly. Failed baking/placement leaves no partial destination;
existing exports and source documents are preserved. No runtime Recast dependency.
Prerequisites: checked population admission and route-free authored baking.
Three exporter tests pass, including actual nine-tile baking/reload and spacing
failure. Actual editor CLI export then admits two encrypted Classic peers through
the real server with matching real headless client configuration. Strict workspace
Clippy and fmt pass. Evidence: build/population-export-{tests,cli,connected,clippy}.log;
build/passive-editor-package-v1. This slice is CLI/package plumbing; interactive
export controls follow. No new rendering, multiplayer pursuit or quest claim.

P4-03 / P3-07 route-free authored navigation bake, 13 November 2026:
Add bake-authored-area SOURCE OUTPUT CENTRE_X CENTRE_Z for static population
navigation without a fabricated patrol. The existing bounded nine-tile collision
bake/publication/verification pipeline is shared; route manifests may already be
empty under schema 1. Existing authored patrol baking still emits its checked route.
Actual valley test produces nine byte-identical collision-derived tiles to the
existing patrol bake, no routes, and the same five accepted spawn points. Existing
output refuses without mutation and invalid centres leave no output. The normal
patrol CLI also rebakes/verifies successfully; nine ordinary baker tests pass.
Fmt and strict workspace/all-target Clippy pass. Evidence:
build/population-area-{bake-test,bake-regressions,bake-clippy,legacy-route}.log.
This is an offline prerequisite for complete population package export, not a new
runtime navigation format, larger coverage or editor export completion.

P3-07 passive population creature presentation, 13 November 2026:
The client now permits its existing explicit --creature-asset override with a
checked single-type passive population. The normal development launcher forwards
art only to clients. Reuse standard Bevy scene/skin/animation/selection-ring code;
no new renderer, actor collision or authoritative model policy. Saved group model
IDs still refuse until catalog resolution is implemented; this override does not
silently satisfy such references. Existing qualified wolf GLB contract is unchanged.
An actual local server/native Classic client admits the checked five-actor group;
five independent rigs and animation players attach, all enter idle, and maximum
visuals is five with zero dropped cues. Screenshot inspected at 1600x1000. Linux
RX 7900 XTX (RADV NAVI31), device 29772, Mesa 26.2.6, Vulkan/KHRONOS validation:
zero GPU diagnostics. No OS movement automation; owned processes/tickets clean up.
This is initial passive presentation evidence, not native damage/death animation,
model catalogs, pursuit, Windows or a performance/capacity qualification.
Three configuration tests, all 55 launcher/bundle tests (including actual headless
preflight), binary builds, fmt and strict workspace/all-target Clippy pass.
Evidence: build/population-native-wolves-v1/{qualification.json,world.json,scene.png,
client.log,server.log}; build/population-visual-{build,config-tests,launcher-tests,
clippy}.log. Reusable qualifier: tools/qualify_population_visual.py.

P3-07 / P5-17 native spawn controls qualification, 13 November 2026:
After renewed user authorization for mouse/display use, an isolated owned editor
session passes boundary vertex movement, midpoint insertion/removal, whole-boundary
return and exact three-step undo, plus loaded-creature assignment/undo. Existing
count/delay/u64 seed/enabled controls, copy/delete/undo, Add, volume preview, offline
play preview hiding and explicit save also pass. Original source bytes stay exact;
only the test save path is written. The existing review editor is not modified.
Linux X11 3440x1440, AMD Radeon RX 7900 XTX (RADV NAVI31), Vulkan, device 29772,
KHRONOS validation enabled: zero GPU validation diagnostics. Captured layout shows
all controls and loaded aggressive-definition feedback; inspected the screenshot.
The two pre-existing missing default asset watcher warnings remain unrelated.
Evidence: build/population-editor-native-extended-v1/{qualification.json,state.json,
editor.log,boundary-and-creature-binding.png,spawn-attributes.png,spawn-volume.png};
owned process exits and input releases. Native evidence is this Linux device/layout,
not Windows, other layouts or live population in editor Play/export. Prior 229
headless editor checks and strict workspace Clippy remain passing. Extended native
qualifier is tools/qualify_editor_spawn_groups.py --extended.

P3-07 real connected population combat and independent respawns, 13 November 2026:
Extend the actual server executable qualification with a checked five-actor group
near the saved player spawn and an explicitly saved 3-second respawn policy. Two
real encrypted Classic peers each cast five normal 20-damage abilities against a
different 100-health actor. Both observe all ten damage events, staggered deaths,
independent respawn clocks, unchanged health/identities for the other three actors,
exact home positions, one fresh incarnation per death, cleared selections and
rejected retired-ID selection. First actor respawns while the second remains dead.
No teleport, actor health override, traversal or rendered input is used. This proves
connected basic ability combat for passive stationary populations, not multi-NPC
pursuit, physical melee, quest credit, model rendering or MMO capacity. Exact all-rate
lifecycle arithmetic remains covered by the separate authored World test.
The actual startup regression and strict workspace/all-target Clippy/fmt pass.
Evidence: build/population-connected-{combat,startup,clippy}.log. Initial test assumed
an expired cooldown stayed in the snapshot; corrected observation uses the current
tick when the cooldown entry has expired. Runtime behavior was unchanged. All owned
processes/tickets clean up. Mouse/display now permitted but unnecessary for this test.

User steering, 13 November 2026: continue active MMO/editor implementation and
local commits. Mouse/display use is now permitted as needed; earlier headless-only
entries retain their historical evidence limits. Navigation-driven quest-test
travel is requested as a future improvement; concrete scope/prerequisites and
acceptance checks are recorded at the top of ROADMAP.md and CODEX_TASKS.md.

P3-07 / P5-17 editor creature assignment and population inspection, 13 November 2026:
Loaded gameplay context now retains its validated creature definition. Spawn-zone
Attributes offers Use loaded creature, assigning only that logical reference via
normal command/history and committing pending inline fields first. Loaded content
never changes source files automatically. Missing selection/context and same-reference controls
remain inert; undo restores the prior reference and other group fields are retained.
Inspection uses the actual runtime binding validator for unknown/mismatched IDs,
aggressive definitions, unresolved model references, disabled groups and the current
16-actor ceiling. A resolved passive definition still explicitly needs placement
checks. Cache by document/revision/selection/gameplay generation so full zone
validation is not repeated every UI frame. Gameplay context changes refresh feedback
without editing the zone. Editor Play/export remains explicitly unconnected.
228 full editor tests pass with seven optional tests skipped; the added actual
headless panel/pending-edit/context-refresh test also passes (229 total). Strict
workspace/all-target Clippy, fmt and diff checks pass. Evidence:
build/population-editor-binding-{tests,panel,clippy}.log. No mouse/display used;
native layout/input/GPU checks remain pending under the user's restriction.

P3-07 development launcher population integration, 13 November 2026:
tools/play.py now accepts --population-config with an authored world, implies
Classic combat, and forwards the exact resolved configuration to server and every
client. It does not imply a patrol or resource profile. Reject singleton gameplay
manifest conflicts and patrol/model overrides. Run the actual client configuration
preflight before creating game processes; invalid placement/configuration refuses
without starting a server or window. Existing process/ticket cleanup is retained.
Five population launcher tests and 49 gameplay-bundle regressions pass, including
one actual built-client headless preflight that stops before the first game process.
Other launcher tests use process doubles only for argument/cleanup assertions;
actual encrypted peer evidence remains in the preceding real-binary qualification.
Fmt, strict workspace/all-target Clippy and diff checks pass. Evidence:
build/population-launcher-{tests,clippy}.log. No mouse/display or native UI run.
Editor export/Play population integration and model/AI support remain pending.

P3-07 real server/client passive population startup, 13 November 2026:
Both binaries now accept --population-config FILE with authored world, prediction,
Classic controls and combat. Server consumes checked actors and the group's respawn
policy before combat/network admission. Client computes the same population/combat
identity and includes count/delay in its renderer-free --configuration-report.
Shared guards reject duplicate configuration, missing prerequisites and conflicting
singleton patrol, pursuit, credit, loot or creature-model presentation options.
The real server executable and actual client configuration report agree exactly;
two encrypted Classic peers admit, receive all five healthy actors and select
independent targets. Test peers retain the required Classic prediction command
channel; no movement, rendered presentation or connected combat is claimed by this
startup test. The existing World test separately qualifies deaths/respawns at all
rates. Eight ordinary encrypted combat regressions and 58 launcher/profile Python
tests pass; five adapter/config tests, binary builds, fmt and strict workspace/
all-target Clippy pass. Evidence: build/population-launch-{build,network,regressions,
tool-tests,tests,clippy}.log. Local loopback tests required sandbox socket escalation.
No mouse/display. Owned server/test peers/tickets are cleaned up; review editor
untouched. tools/play.py convenience wiring and editor export remain pending.

P3-07 bounded population startup configuration, 13 November 2026:
A shared schema-1 loader resolves a saved zone/group, passive creature definition
and navigation directory beneath the configuration directory. Bound configuration
to 4096 bytes and paths to 256 bytes; reject unknown fields, unsupported schemas,
escaping paths, procedural worlds and unresolved group/behavior/model settings.
Preparation completes before simulation/network admission. Loaded state provides
matching population identity, count and respawn timing for server/client setup.
The authored World test now loads all inputs through this actual file path and
proves exact identities, five full-health kills and fresh-ID 30-second respawns at
20/30/60 Hz. Malformed config/group/spacing/profile cases refuse; four ordinary
adapter/loader tests pass. Fmt and strict workspace/all-target Clippy pass.
Evidence: build/population-config-{tests,authority,clippy}.log. CLI consumption is
the next slice; this loader alone does not claim launcher or editor Play integration.
No mouse/display, native rendering or native Windows execution.

P3-07 saved editor population admission adapter, 13 November 2026:
A shared startup adapter now derives placement requests and passive lifecycle
bindings directly from validated zone groups. The read-only spawns-zone CLI uses
that same request conversion. Admission requires a known enabled group, matching
creature definition and at most 16 targets; explicit model references refuse until
model catalog integration exists. Placement-only inspection still accepts those
model-bearing drafts. Other zone geometry is not asserted to match an exported
world, and selecting one group does not implicitly activate other groups.
The actual World qualification now serializes/reloads a schema-14 draft and passes
its saved bounds/count/seed/creature/30-second respawn through the adapter, proving
five deaths and fresh-ID respawns at 20/30/60 Hz. Adapter rejection tests and the
actual CLI source-preservation regression pass. Strict workspace/all-target Clippy
and fmt pass. Evidence: build/saved-population-{tests,authority,cli,clippy}.log.
The network crate depends only on the engine-independent zone data crate for this
startup validation; no editor UI or Recast bake dependency enters the server.
No mouse/display used. Normal launcher and editor export remain pending/gated.

P3-07 passive creature definition and population lifecycle binding, 13 November 2026:
Separate validated creature behavior from its patrol route while retaining legacy
exact-byte patrol identities. Canonical definition identity covers logical ID,
speed/wait, deflection and every pursuit field; route/reformatting do not change it.
A resolved passive population binding requires an exact creature ID and the saved
1..86400-second respawn range. Reject aggressive definitions until multi-creature
pursuit exists. World admits the checked pool atomically, includes definition and
lifecycle in its identity, resolves definitions only for current live incarnations,
and rejects conflicting combat respawn settings before mutation. No new file schema.
The authored valley test proves five full-health deaths and exact 30-second fresh-ID
respawns with retained definitions at 20/30/60 Hz. Fifteen creature tests and all 203
combat tests pass, including actual encrypted loopback regressions; sandbox socket
failures were rerun with local UDP permitted. Fmt and strict workspace/all-target
Clippy pass. Evidence: build/creature-definition-{tests,authority,combat-qualified,
clippy}.log. No mouse/display or native rendering qualification. This is typed
runtime admission, not yet saved group loading, model binding, rewards, pursuit or
normal editor export. Next: connect saved authoring inputs to this admission path.

P3-07 authored respawn deadline range, 13 November 2026:
Combat configuration now accepts the editor's full 24-hour respawn ceiling; retain
100 ms legacy minimum and the existing 5 s development default. This is an explicit
Theia authoring limit, not a Classic formula. Health/death wire messages validate
respawn deadlines separately up to that ceiling at supported 60 Hz. Ability cast,
GCD, cooldown and queue deadline limits stay unchanged, as do frame tags/layouts
and bounded bytes. Existing short-delay configuration identities remain unchanged;
long-delay identities include their exact delay and older config loaders refuse them.
Eleven actual authority tests, 13 wire tests and all eight ordinary encrypted combat
regressions pass (one optional population test skipped). Test 1 s / 61 s / 24 h
exact deadlines at 20/30/60 Hz after normal full-health damage, reject excessive/
past/corrupted deadlines and retain short ability limits. A real encrypted client
receives a 24-hour deadline after five normal 20-damage casts against 100 health
and observes no early respawn. This is deadline arithmetic/transport evidence,
not a full-day soak or saved group-setting adoption. Fmt and strict workspace/
all-target Clippy pass. Evidence: build/population-respawn-{tests,network,
renet-regressions,clippy}.log. No mouse/display; localhost tests use socket escalation.
Next: bind creature definitions and saved group lifecycle settings; export stays gated.

P3-07 / P1-07 prepared population over secure transport, 13 November 2026:
Two actual encrypted Renet peers receive all five checked valley targets and their
100-health states, with coordinates agreeing within 0.1 mm. Independent selection
and clearing, retained selection across normal world ticks, disconnect and a fresh
third session pass under 15 ms delay, 10 ms jitter, every-tenth packet loss and
every-seventeenth duplication. Fresh session has a new player identity, no selected
target and the same five population identities. Maximum connected players is two;
no event backpressure disconnect or relay overflow. Owned relays/server/tickets end.
The first test incorrectly expected unchanged targeting timestamps to advance;
static targeting intentionally suppresses timestamp-only updates. Observe ordinary
world snapshot ticks instead; runtime behavior is unchanged. This is connected
target/health/selection qualification at 30 Hz, not connected multi-creature combat,
Classic movement, rendering, model binding, MMO capacity or native Windows.
The preceding actual World test separately proves Classic death/respawn at all
three rates. Normal source/configuration/export integration remains pending.
Evidence: build/population-network-{tests,clippy}.log; opt-in test
`checked_authored_population_replicates_to_two_encrypted_peers_with_independent_selection`
in crates/network/tests/combat_renet.rs. Fmt and strict workspace/all-target Clippy
pass. Local UDP sockets require sandbox escalation; no mouse/display was used.

P3-07 / P4-03 bounded authoritative population admission, 13 November 2026:
World now atomically consumes one complete prepared spawn pool before player or
combat admission. Validate source identity, empty population/target state, current
16-target protocol ceiling, checked body positions and fresh identity capacity
before publishing any target or static patch. Retain prepared static collision and
home demand across deaths; actors add no collision bodies. Placement identity binds
request, source/navigation and exact checked points. Existing combat applies health
and a shared respawn policy afterward; saved group/creature/model binding is not
connected yet and normal editor world export still refuses population drafts.
Actual valley World/Avian tests at 20/30/60 Hz enable Classic prediction and admit
five collision-validated player restores. Five checked targets each take five
unaltered 20-damage development casts from full 100 health, die, clear selections
and respawn exactly 30 simulation seconds later with fresh IDs at the same checked
positions. Reject stale IDs, duplicate admission, foreign source and a checked
17-point pool; failed admission preserves identity and the next actor ID.
This is headless authority evidence, not five network clients, production wolf AI,
model/reward binding, group-specific lifecycle settings or native presentation.
Ten existing combat and 13 creature-authority regressions, nine baker tests, fmt
and strict workspace/all-target Clippy pass. Evidence:
build/population-admission-{authority,combat-tests,creature-tests,baker-tests,clippy}.log.
No mouse/display use. Next: actual secure replication/selection of the prepared
population, then complete creature/profile and editor export binding.

P3-07 / P4-03 shared spawn preparation, 13 November 2026:
Move authored standing/route-clearance and bounded spawn checking from the baker
binary into the shared network navigation library. The CLI uses that same path.
A private, non-deserializable Prepared value exposes complete checked feet and
source/navigation/request identity; incomplete diagnostics cannot become Prepared.
File loading and navigation queries remain loader/worker operations, never fixed
simulation work. This is a runtime-admission prerequisite, not live population.
Nine baker tests, two navigation unit tests, actual valley CLI/direct-preparation
checks and authored route verification pass. Pre/post-move valley reports match
exactly (five points, 299 access samples); route verification retains 231 samples.
Incomplete capacity refuses promotion; fmt and strict workspace/all-target Clippy
pass. Evidence: build/spawn-preparation-{tests,network-tests,cli,route,clippy}.log.
No renderer, mouse/display, Windows, connected population or respawn claim. Next:
atomic bounded admission to the actual World, then population lifecycle/profile
binding. Saved-group export refusal remains in place.

P5-17 / P3-07 / P4-03 saved spawn-zone placement checks, 13 November 2026:
`theia-nav-bake spawns-zone WORLD NAVIGATION ZONE_JSON GROUP_ID SPACING X Y Z`
reads a validated saved editor group directly. Preserve its custom polygon, exact
seed and alive count; require an enabled known group and explicit spacing/access
anchor. Output includes the group metadata and canonical draft digest alongside
the checked world/navigation identities. It uses the existing bounded support,
standing-clearance, containment, spacing and access-route checks unchanged.
Nine ordinary baker tests and the actual valley CLI integration test pass, plus
fmt and strict workspace/all-target Clippy. The saved-group and explicit-request
paths return the same five points / 299 access samples; repeatability, disabled/
missing groups, wrong heights, invalid access, insufficient capacity and preserved
source/package files pass. Evidence: build/spawn-zone-check-{tests,cli,clippy}.log.
This is read-only placement checking against the supplied world, not proof that
all draft geometry matches that world, resolved creature/model binding, runtime
population or respawn qualification. No mouse/display use or rendering change.
World export still refuses population drafts. Next: reusable checked preparation
for authoritative population admission, retaining the current 16-target runtime
budget until protocol/client capacity is deliberately extended.

P5-17 / P3-07 spawn-boundary reshaping, 13 November 2026:
Autonomous local work resumed under the user's no-mouse/no-display instruction.
Attributes now cycles through numbered polygon vertices, edits exact global X/Z,
splits the selected edge (including the closing edge), removes vertices down to
three, and returns to whole-boundary translation. Pending fields commit before
boundary actions. Shared structured move/insert/remove commands retain validation,
atomic history and explicit Save; invalid geometry, indices and budget overflow
preserve the prior document. Far-origin precision and metadata remain intact.
Prerequisites: schema-14 validated polygon volumes and existing inline/history UI.
Acceptance: 75 zone tests and 226 editor tests pass (seven optional skips), plus
fmt and denied-warning workspace/all-target Clippy. The existing UDP regression
requires sandbox escalation for localhost sockets; it passes there. Build tools
use tools/edit_zone.py's staged native SDK environment. Evidence:
build/spawn-boundary-{zone-tests,editor-qualified,clippy-final,focus-tests}.log.
No display/mouse use or new GPU/Windows qualification; native boundary-control
layout/input remains pending. The open review editor and saved user files are
untouched. World export still refuses population drafts. Next: consume saved
spawn zones in checked placement validation, then authoritative population/respawn
integration. These authoring edits do not create live NPCs. All work stays local.

Safe stop requested, 12 November 2026:
Work stops after cf2a8018 (spawn-zone editor drafts). Final editor suite completed:
223 passed, seven optional skips. No implementation is in progress. Resume with
per-vertex custom-boundary authoring, then connect checked placement candidates
and authoritative multi-creature population/respawn support. Do not remove the
explicit export refusal until that runtime connection is validated.
The user review editor PID 308099 remains open on the saved valley patrol copy;
its running binary predates the spawn-zone UI. Its report/log are under
build/editor-quests-review-v1/. Check unsaved state before any replacement; preserve
user files. Separate spawn-zone qualification windows are closed. All commits are
local; nothing pushed. Current review source and gameplay/scenery paths remain
recorded below and in docs/ZONE_EDITOR.md.

P5-17 / P3-07 spawn-zone editor drafts, 12 November 2026:
Add → NPC spawn zone at spawn creates a bounded rectangular population draft.
Explorer has an NPC spawn zones filter; select a draft to see its custom polygon
prism and permanent inline creature/model IDs, exact u64 seed, enabled checkbox,
centre X/Z, feet height limits, alive count and respawn seconds. Existing rename,
copy, delete, undo and explicit Save apply. Source files stay unchanged until Save.
Preview shares one mesh/material across a bounded 192-edge pool, uses local
coordinates after f64 origin subtraction, has no collision and hides in playtest.
Unsupported gizmo/ground operations are disabled. Imported concave polygons remain
intact; per-vertex boundary editing is next. These drafts do not spawn NPCs and
world export still explicitly refuses them pending runtime population binding.
Acceptance: 223 ordinary editor tests pass (seven optional skips); fmt, strict
workspace/all-target Clippy and build pass. Native creation/filtering, concave
preview, numeric/seed edits, enabled toggle, copy/delete/undo, playtest hiding and
explicit Save pass on RX 7900 XTX / RADV NAVI31 / Vulkan with zero GPU diagnostics.
Captures visually inspected; source preserved. Windows remains unqualified.
Evidence: build/spawn-group-editor-{tests-final,clippy,build}.log and
build/editor-spawn-groups-native-v1/qualification.json; reproduce with
tools/qualify_editor_spawn_groups.py.

P5-17 / P3-07 / P4-03 bounded spawn-candidate validation, 12 November 2026:
A read-only `theia-nav-bake spawns-authored WORLD NAVIGATION REQUEST_JSON` command
now samples seeded points from detailed navigation triangles within a custom
polygon/height volume. Canonical tile order, bounded draws and area weighting are
explicit. Require the exact world/navigation identity and development capsule,
then check grounded standing clearance, final containment, spacing and an access
route with the existing strict 0.2 m sampled navigation/collision checks. Caps are
64 requested points, 4,096 proposals, 256 access queries and 8,192 clearance
samples. Insufficient results are reported and return failure; never force points
or infer a roaming boundary. Preparation remains outside the fixed simulation.
Two focused network unit tests and an opt-in actual CLI test pass; fmt, strict
workspace/all-target Clippy and baker build pass. The saved valley patrol world/nav
pair yields five points in the concave western grove and 299 access samples.
Repeated output is exact; wrong height, invalid access, foreign navigation and
insufficient candidate count refuse, with source files preserved. No rendering
change, actor population, respawn timing, model binding or Windows claim.
Evidence: build/spawn-candidates-{tests,clippy,build}.log and
build/spawn-candidates-valley-v1/{result.json,cli-test.log}; example request is
content/navigation/starter-valley-spawn-candidates.json. This request is a checker
input, not a shipped spawn-group manifest. Schema-14 zone population drafts still
refuse world export. Next: editor authoring/preview, then validated runtime binding.

P5-17 / P5-18 NPC quest inspection, 12 November 2026:
Selecting a bound NPC now shows expandable quest offers/turn-ins in Attributes,
including description, prerequisite quest names, ordered stages/completion rules,
objectives, XP and configured fixed item rewards. Resolve existing stable NPC IDs
against a validated gameplay catalogue; no duplicate NPC quest list or database
editing is introduced. Missing templates, unbound and unmatched NPCs are explicit.
File → Load gameplay and the inspector use the existing file browser; reload is
prefilled, invalid files retain current data and zone/history/files remain intact.
Interactive --gameplay-template now loads this session context and supplies the
world/network export default. It is not persisted by zone Save; load it again on
another launch, or pass the CLI option. This is read-only quest inspection, not
full quest creation or multiple runtime NPC contacts.
Prerequisites: existing quest catalogue/reward validation and NPC binding/export.
Acceptance: all 220 ordinary editor tests pass (seven optional skips), fmt and
strict workspace/all-target Clippy pass. Native actual selection, three quest
expand/collapse flows, manifest reload, browser cancel and unbound NPC clearing
pass on RX 7900 XTX / RADV NAVI31 / Vulkan, zero GPU diagnostics. Captures were
visually inspected; drawn chevrons reuse the existing section style. Source zone
and gameplay bytes remain unchanged. No Windows or new connected gameplay claim.
Evidence: build/quest-inspector-{tests-final,clippy,build}.log and
build/editor-quests-native-v2/{qualification.json,npc-quests.png,quest-details.png,
gameplay-file-browser.png,unbound-npc.png}; reproduce with tools/qualify_editor_quests.py.
Continue custom-bound NPC spawn-group implementation after this user-reported UI gap.

P5-17 / P3-07 spawn-group data foundation, 12 November 2026:
Schema 14 adds bounded custom-polygon population drafts with creature/model IDs,
alive count, respawn delay and seed. Pure f64 polygon validation accepts concave
bounds and both windings, rejects crossings/touches/degenerate edges, and preserves
far-origin precision. Shared commands support atomic history, translation, copy and
rename. World export explicitly refuses population drafts until runtime binding
exists. No editor controls, candidate placement or live population are claimed.
Prerequisites are existing zone history and checked coordinates; the next slices
need supported navigation/collision candidates and multi-creature authority.
Two polygon tests and 73 zone tests pass; strict workspace/all-target Clippy passes.
Evidence: build/spawn-volume-tests.log, spawn-group-zone-tests.log and
spawn-foundation-clippy.log. No rendering change. NPC quest inspection is now the
immediate UI priority after the user's missing-quest-information report.

P5-17 / P3-07 / P4-03 patrol-to-gameplay export binding, 12 November 2026:
With a gameplay template and exactly one Patrol guide, world export now binds the
copied creature profile to that guide's stable route ID. Preserve the creature ID,
combat/behavior values and every unrelated role byte; revalidate the complete copied
bundle. Multiple patrol guides with a single-creature template refuse before staging
instead of choosing arbitrarily. Exports without a gameplay template still support
multiple independent route requests. Baking remains an explicit offline step.
The actual valley review copy adds Western grove patrol #1002 through the shared
headless command API, preserving the original source. Exported profile and request
agree on theia.zone.patrol.1002. Nine tiles bake with 231 clearance samples; 1,200
actual authoritative fixed steps complete a loop, travel 70.011 m and record zero
blocked steps with that exact exported creature profile. Native client configuration
also checks the resulting source/route binding separately from simulation. This is
not a new connected combat/rejoin run; the prior full valley quest proof remains.
Evidence: build/editor-patrol-binding-{tests,export-tests,clippy,build}.log and
build/valley-patrol-binding-v1/{edit.log,export.log,bake.log,runtime-test.log,preflight/}.
No rendering changes in this binding slice. Earlier native patrol authoring captures
remain valid. Review zone: build/valley-patrol-binding-v1/valley.zone.json, with
world/ and navigation/ beside it. This world has a new identity: keep this pair together.
Next requested feature: custom-polygon NPC spawn groups with creature/model, alive
count and respawn delay, collision/navigation placement and separate roam limits.
That population feature is not implemented yet; see DECISIONS.md.

P5-17 / P3-07 / P4-03 editable patrol requests, 12 November 2026:
Schema 13 adds Patrol to the existing guide tools. Create ground points, edit exact
inline coordinates/preview width, rename, insert/remove points and use normal
selection/history/Save. Stable route IDs derive from authoring IDs, independent of
names. Purple previews hide in offline playtest; patrol ribbons never enter exported
world geometry/collision. Terrain grading refuses patrol requests. World export writes
exact navigation-requests/patrol-ID.json requests, explicitly marked not baked;
the existing offline baker resolves/checks them separately. No runtime creature
assignment or automatic editor bake is implied by this first authoring slice.
All 71 zone and 215 ordinary editor tests (seven skips), fmt, strict workspace/all-target
Clippy and editor/nav builds pass. Native creation, inline point edit/undo, source
preservation, explicit Save and preview checks pass; the exported request then bakes
nine tiles and passes existing clearance verification. Zero GPU diagnostics on Linux
RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6; no Windows or runtime patrol claim.
Evidence: build/editor-patrol-{tests,clippy,build}.log and
build/editor-patrol-native-v2/{qualification.json,patrol-editor.png,patrol-offline-preview.png,bake.log}.
Reproduce with tools/qualify_editor_patrol.py. The earlier empty-zone fixture's single
1024 m box floor fails strict grounded-support verification at [8196.454545454546,
0.0057672614401037044,8196]; this remains an explicit coverage limitation. The positive
fixture uses authored height-patch terrain, as the starter valley does. No tolerance
was relaxed. Baker errors now identify the sampled point and local capsule feet.
Next: bind the exported creature profile to its authored patrol and qualify that path.

P5-17 / P3-09 authored valley quest loop, 12 November 2026:
Native connected account play now qualifies all three existing development quests
in the dressed valley: guide introduction, three kills of the single respawning
patrol creature, and reaching the authored western-grove overlook. Normal Classic
mouse/keyboard input walks the checked trail; normal dialogue/buttons equip the
sword and accept/turn in quests. No health/damage/reward profiles are weakened.
Area occupancy changes the objective without XP; the final turn-in reaches 150 XP,
three completed quests and journal revision 14. Database restart/rejoin preserves
all three and the exact observed feet [8192.279571294785,0.008000016212463379,
8192.029574822634]. Completed quests are no longer offered. Six quest UI actions
apply, none reject; server patrol blocking and GPU diagnostics remain zero.
All 73 authored/account/gameplay/identity Python tests, fmt and strict workspace/
all-target Clippy pass (build/authored-quests-python.log and editor-patrol-clippy.log).
The complete run includes one player death/automatic respawn; this is not a
continuous death-free route qualification or a combat-balance/health-recovery claim.
It uses one bound guide and one respawning training creature, not multiple NPCs,
a population of wolves, authored quest editing or item-pickup qualification.
Evidence: build/authored-quests-native-v4/{qualification.json,position-proof.json},
fresh/resumed captures and logs. Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6;
no Windows claim. All owned game/identity/database processes and tickets close.
Earlier attempts exposed test handling of respawn, a partial live-report read and
the intentional five-minute account expiry. The qualifier now retries partial
reports, checks connection state, handles respawn and explicitly uses the existing
1800-second owned issuer/server session policy. Expiry enforcement is unchanged.
Reproduce with tools/qualify_authored_quest_intro.py --quest-trail
content/navigation/starter-valley-quest-trail.json --reach-area plus the contact-v3
world/navigation and original template/player assets. Qualified gameplay copy:
build/starter-valley-playable-gameplay-v1/gameplay.manifest.json. Next: editable
patrol guides/export, then multiple NPC contacts and broader quest/content authoring.

P5-17 / P4-03 authored route inspection, 12 November 2026:
Add read-only `theia-nav-bake route-authored WORLD NAVIGATION ROUTE_JSON`, using the
same bounded route reader, strict floor sampling and collision checks as publication.
It verifies world identity/tile hashes and returns grounded waypoints only after
0.2 m sampled capsule clearance. No package or actor is changed. The western village
trail passes 679 samples / 35 output points; wrong-floor input is rejected and source
bytes are preserved by the actual CLI integration check. Eight ordinary baker tests,
fmt and strict workspace/all-target Clippy pass; no rendering change/GPU claim.
The initial direct route was correctly rejected at a cottage edge; a southward
alternative also failed strict height agreement on a slope. These are unresolved
coverage limitations, not relaxed tolerances or evidence of exhaustive navigation.
Use the checked western trail for the next normal-input quest qualification.
Evidence: build/authored-route-{tests,cli-test,clippy,build}.log and
build/authored-valley-route-v1/west-result.json. Route input is preserved at
content/navigation/starter-valley-quest-trail.json, bound by qualification to the
contact-v3 world/navigation pair. No native traversal claim yet.

P5-17 / P3-01 / P3-09 authored gameplay bundle and introduction quest, 12 November 2026:
The normal launcher now accepts an authored gameplay manifest with explicit matching
navigation and the world's bound guide. Configuration preflight checks the real
client before login/server launch; arbitrary guide-position overrides are refused.
The candidate copies the existing development gameplay profiles, changing only the
creature route to the baked west-grove patrol and the manifest identity. This is a
bounded development fixture, not a new multi-NPC content catalogue.
Actual private OIDC/PostgreSQL/native Vulkan qualification completes the guide's
introduction, its dialogue objectives and turn-in for 25 XP. After database restart,
rejoin preserves journal revision 5, one completed quest and 25 XP; the completed
quest is no longer offered. The authoritative patrol moves with zero blocked steps.
Owned processes/tickets close, checkpoints commit and GPU diagnostics remain zero
on Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6. No Windows qualification.
All 71 authored/account/gameplay-bundle Python tests, fmt, strict workspace/all-target
Clippy and diff checks pass. Evidence: build/authored-gameplay-{python,clippy}.log,
build/authored-quest-intro-native-v2/qualification.json and fresh/resumed captures.
Reproduce with tools/qualify_authored_quest_intro.py. An earlier native attempt
completed/restored the quest but clicked before resumed input/scene readiness;
the qualifier now waits for readiness, focus and cursor position before interaction.
Qualified bundle: build/authored-quest-intro-native-v2/gameplay/gameplay.manifest.json,
paired with starter-valley-contact-world-v3 and starter-valley-contact-navigation-v3.
Next: actual wolf-kill and overlook quest completion, then broader editor NPC/quest
support. This does not qualify all valley quests, multiple contacts or wolf art.

P5-17 / P3-07 / P4-03 authored patrol configuration checks, 12 November 2026:
Client configuration now verifies navigation tile hashes and resolves the creature
profile's route against the selected manifest. Share that route lookup with server
patrol preparation; authoritative physical route preparation remains unchanged.
The normal launcher runs configuration-only preflight before starting an authored
creature session or creating its ticket directory. Keep service credentials out
of that client process. Full authored gameplay-manifest launch remains gated.
All 13 creature simulation checks, 21 authored/account Python tests, fmt, strict
workspace/all-target Clippy and client/server builds pass. Four actual headless
client checks accept the matching west-grove profile and reject missing route,
foreign source identity and a modified tile with its unchanged expected hash.
No renderer, network connection or saved-state admission starts in preflight.
The qualification fixture includes the existing ability item/status/life dependencies;
earlier v1-v3 attempts correctly rejected omitted companion profiles.
Evidence: build/authored-patrol-preflight-{tests,python,clippy,build}.log and
build/authored-patrol-preflight-v4/qualification.json; reproduce with
tools/qualify_authored_patrol_preflight.py. No new GPU/Windows/runtime quest claim.
The valley's exported development template still references the older training
route; the matching candidate changes only that route reference. Next: construct
and qualify a coherent valley gameplay bundle with its bound contact/navigation.

P5-17 / P2-02 / P3-01 account-backed authored-world entry, 12 November 2026:
Normal --account-login now admits validated immutable authored packages as well as
saved terrain. Retain source identity, supported spawn/restore checks, normal
speed/prediction, secure Renet and existing asynchronous persistence. Generated
terrain, development ticket/persistence overrides and tours remain outside this
account path. The movement-test launcher stays separate. Full authored gameplay
bundles remain explicitly gated pending navigation/profile/runtime qualification.
All 25 server library tests, 20 authored/account Python tests, fmt, strict
workspace/all-target Clippy and server build pass. Actual private Keycloak 26.8.1 /
PostgreSQL 16.15 / native Vulkan run passes fresh login/movement/checkpoint,
rejoin at exact committed XYZ, subsequent movement, normal interruption, failed
client cleanup and pending-login cancellation. The persisted runtime position
moves from [8192,0.908,8192] to [8202.733340263367,0.9079999923706055,8192], then
restores exactly and commits [8214.633352279663,0.9079846143722534,8192] under the
same authored identity. These are runtime global positions, not editor pawn feet.
Native client service/SQL credentials are absent, tickets are removed, final
checkpoints finish before database closure, and all owned services/windows stop.
Zero GPU diagnostics on Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6; no Windows
or authored quest-completion claim. Evidence: build/authored-account-*.log and
build/authored-account-native-v1/{verification.json,*-authored-checkpoint.json}.
Reproduce with tools/qualify_account_launcher.py --authored-world and explicit
current binaries/player assets. Next: bind the valley's creature route and quest
profiles to its navigation before enabling the complete authored gameplay bundle.

P5-17 / P2-14 authored-valley minimap terrain, 12 November 2026:
The minimap now samples already admitted authored-world ground after scene readiness,
with checked global XZ bounds and topographic highest-surface semantics. No world,
collision, quest guidance, readiness or loading authority changes. A bounded
four-metre presentation cache holds at most 4,225 samples, warms at most 128 per
frame near the player first, retains overlap during panning and drops old cells.
Unknown/outside-zone pixels remain dark. The existing 64x64/16x16 image pair stays
17,408 bytes and refreshes at most four times per second. CPU sample cache is
additional; this is a coarse topographic map, not actor support or navigation.
All 250 ordinary client tests (two skips), eight authored-world tests, formatting,
strict workspace/all-target Clippy and client/server/editor builds pass. Native
Linux two-client scripted valley checks show 4,096 known pixels, 4,225 cached
samples, 4,451/4,419 total height queries and zero tile loads/retirements. Peak
raster costs are 4.197/4.063 ms, cache warming 3.492/3.997 ms, complete presentation
5.068/6.874 ms. The initial uncached experiment peaked at 37.052/38.665 ms and was
replaced before commit. These are local debug-build measurements, not general FPS
or MMO capacity claims. Both routes finish; owned cleanup and zero GPU diagnostics
pass on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan. No Windows qualification.
Evidence: build/authored-minimap-{network-tests,client-tests,client-suite,clippy,build}.log
and build/authored-minimap-native-v2/{qualification.json,view-1.png,view-2.png}.
Reproduce with tools/qualify_network_windows.py --require-authored-map and the
exported scripted scenario. Next: authored account/quest/NPC launch prerequisites;
the development network-test launcher still exercises movement only.

P5-17 / P1-06 independent network camera windows, 12 November 2026:
Managed playtests pass each client a bounded --test-window INDEX/COUNT. Standard
Bevy monitor/window APIs arrange two views horizontally where space allows,
vertically on tall displays, or as a cascade on small screens; a single enabled
client centres normally regardless of its authenticated slot. Titles use the
server-supplied player ID. Background views keep updating and initial placement
never overrides later user movement/resizing. No embedded framebuffer sharing.
All 248 ordinary client tests (two skips), 21 focused Python checks, formatting,
strict workspace/all-target Clippy and application builds pass. Native Linux
Vulkan qualification verifies actual separate 1280x720 client rectangles inside
the 3440x1440 monitor, distinct authenticated player titles/camera captures, both
movement scripts completing and owned cleanup/ticket removal. Zero GPU diagnostics
on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6. Portrait/negative-origin layouts have
headless checks; native Windows/Wayland/multiple-monitor execution is unqualified.
Evidence: build/test-window-* and build/network-windows-native-v2/; reproduce with
tools/qualify_network_windows.py. Next: authored-world minimap terrain, currently
blank in these new zone views, before more authored quest/NPC integration.

P5-17 / P1-06 managed Network Playtest in the editor, 12 November 2026:
Zone -> Start network playtest exports the current immutable authoring snapshot,
then owns one real server and one encrypted native client per enabled Test Player.
The dialog takes a fresh folder, player assets and optional/required-by-content
gameplay template; hidden content metadata stays automatic. Stop ends the whole
session. Private owner-pipe EOF also cleans up after abrupt editor death. F5 is
explicitly Offline playtest and blocked while the network session runs. Source
zones remain untouched without Save; setup edits affect the next launch only.
All 214 ordinary editor tests (seven opt-in skips), fmt, strict workspace/all-target
Clippy and client/server/editor builds pass. Native Start/Stop/restart and abrupt
owner-kill checks pass with two scripted Synty valley clients, including 80 ms
one-way delay + 0..20 ms jitter on client 2. Both runs admit two players with zero
rejected inputs/readiness, movement recoveries, relay overflow, cleanup errors or
GPU diagnostics. Tickets removed and exact source bytes preserved. All owned
windows/processes closed. Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan;
Windows, embedded views and account-backed quest/combat launch remain unqualified.
Evidence: build/editor-network-{tests,clippy,build}.log and
build/editor-network-native-v4/{qualification.json,state.png,editor.log}.
Native harness uses report barriers between complete key presses; earlier v1-v3
attempts exposed automation timing/repeat issues, not failed network sessions.
Reproduce with tools/qualify_editor_network_playtest.py. Next: arrange/distinguish
the two native client views, then return to authored MMO gameplay integration.

P5-17 / P1-06 editor-owned network session lifetime, 12 November 2026:
The managed launcher accepts --stop-on-stdin-eof: a private editor-owned stdin
pipe closing requests whole-session cleanup, including when the editor exits
unexpectedly. Cancel before/after preflight, during server/client startup and
while running; finish bounded preflight before returning if it is already active.
Partial sessions report which configured clients actually started. Ordinary CLI
stdin is unaffected unless the flag is supplied.
All 20 focused launcher/profile tests, formatting and strict workspace/all-target
Clippy pass. Actual two-client scripted Vulkan run with one delayed connection
ends from owner-pipe EOF in 0.42 seconds, removes tickets, and reports no cleanup
errors or GPU diagnostics. Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 only;
Windows and editor-button integration remain pending. Evidence:
build/network-owner-{tests,clippy}.log and build/network-owner-native-v1/.
Next: use the export dialog and this ownership protocol for editor Start/Stop.

P5-17 / P1-06 native Test Player authoring, 12 November 2026:
Add -> Test player at spawn creates a bounded slot with a nonblocking capsule
badge. Explorer has a Test players filter and normal selection/frame/rename;
shared XYZ handles, permanent position fields, grounding, duplicate/delete and
undo/redo preserve exact global coordinates and explicit Save. Rotation/scale
are disabled because this first network spawn profile specifies feet only.
Attributes includes validated one-way delay, added jitter, loss and seed. The
three-dot menu toggles enabled and selects Manual or Walk and turn; controller
and network settings survive movement/history. Outage schedules remain JSON-only.
The badges hide in offline F5, add no collision and release deleted entities.
All 213 ordinary editor tests pass (seven opt-in skips), as do fmt, strict
workspace/all-target Clippy and client/server/editor builds. Native Linux owned
mouse/keyboard tests pass add/filter, controller/enabled, position/delay edits,
XYZ drag/undo, duplicate/delete/undo and exact source-file preservation. Zero GPU
diagnostics on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan. Tests close their
owned window; no Windows execution claim. Evidence: build/test-player-ui-*
and build/test-player-ui-native-v3/{qualification.json,state.png,editor.log}.
Reproduce with tools/qualify_test_player_ui.py. Current Network Playtest launcher
and authored export work from CLI; managed in-editor Start/Stop remains next.

P5-17 / P1-06 zone-to-network-test export, 12 November 2026:
The editor CLI --export DIR --network-test publishes a fresh world/scenario and
bundled controller snapshot from enabled schema-12 Test Players. Resolve the
curated test.walk-turn controller, retain unique client slots and per-connection
conditions, verify exact world identity and real collision/landing support, and
remove owned partial output on failure. Disabled players remain in the authoring
snapshot but do not launch. Quest areas retain the required gameplay template;
the movement launcher does not activate its quest/account/combat profiles.
All 210 ordinary editor tests pass (seven opt-in skips); focused final export
checks, fmt, strict workspace/all-target Clippy and client/server/editor builds
pass. Actual full Synty valley export preserves 65 scenery models, its quest area
and contact template. Its generated scenario drives two native scripted clients,
including 80 ms each-way delay + 0..20 ms jitter on client 2. Both finish; zero
rejected inputs, recoveries, relay overflow or GPU diagnostics. Owned session
closes and tickets are removed. Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6;
no Windows execution claim. Evidence: build/network-test-export-* and
build/network-export-native-v1/. Next: editor pawn selection/inspector/placement
and managed Start/Stop. CLI export is connected; editor buttons remain pending.

P5-17 / P1-06 saved test-player definitions, 12 November 2026:
Zone schema 12 stores at most two Test Players, separate from NPC markers and
production player spawn. Each has an exact global feet position, shared authoring
ID, unique development client slot, enabled flag, optional stable controller ID
and bounded delay/jitter/loss/seed/outages. Existing schemas serialize unchanged.
Put/delete, translate, rename, terrain grounding, duplicate into the free client
slot and undo/redo use the same transactional document history. Validation checks
all entries including disabled slots; physical support remains a launch-time
server check. Blank-terrain recipes refuse a zone containing test pawns.
All 70 zone tests, formatting and strict workspace/all-target Clippy pass, including
far signed origins, atomic rejection, history, metadata, schema/ID/slot budgets
and impairment limits. No rendering change or new native/Windows claim. Evidence:
build/test-player-authoring-{tests,clippy}.log. UI selection/placement/inspector,
controller catalogue resolution and managed editor launch remain pending; this
slice is the persisted authoring foundation. See ZONE_EDITOR.md. Continue locally.

P5-17 / P1-06 attachable movement controllers, 12 November 2026:
Enabled scenario players may bind a bounded JSON movement script; the launcher
preflights it and the native client drives the shared Classic input/controller,
normal prediction and authenticated server path. Wait for scene/model and actual
acknowledged collision footprint before advancing; support background clients,
finite/repeating routes and F7/button detach/restart with ordinary manual controls.
No arbitrary code, teleports or scripted combat. The checked walk/turn/strafe/jump
example is content/testing/walk-turn-controller.json.
All 246 ordinary client tests (two opt-in skips), 17 focused Python checks,
formatting, strict workspace/all-target Clippy and client/server/editor builds pass.
Actual two-client native Vulkan routes pass on Linux RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6, including 80 ms each-way + 0..20 ms jitter on client 2 and owned F7
manual/restart interaction. Zero rejected inputs, recoveries, relay overflow or
GPU diagnostics; managed cleanup removes tickets. Evidence: build/test-controller-*
and build/network-scripted-native-v1/ (v3 checks the refined status placement).
No Windows/account/combat/editor launcher UI claim. Next: author Test Player
placements with stable client slots/controller references/network settings, then
connect managed Start/Stop. F5 remains offline. See RENET_QUALIFICATION.md.

P5-17 / P1-06 managed native network playtest, 12 November 2026:
New tools/network_playtest.py derives client count from enabled scenario players,
starts one authoritative server plus independent local-ticket clients/relays,
checks authored identity, retains manual Classic controls and handles whole-session
Stop/window-close/timeout/interruption/partial-failure cleanup. Profiles preserve
client IDs (including only slot 2 enabled), separate one-way delay from base added
RTT, and leave real latency measurement distinct. Server still validates spawns.
All 24 focused Python tests pass. Native Linux RX 7900 XTX / RADV NAVI31 / Mesa
26.2.6 Vulkan runs two manual clients for 25 seconds: first no impairment, second
80 ms each-way + 0..20 ms jitter. Both enter at distinct configured positions and
publish nine-chunk readiness. Zero rejected inputs, recoveries, relay overflow or
GPU diagnostics; automatic shutdown removes tickets and reports no cleanup errors.
No native scripted movement, combat, Windows, persistence or editor Start/Stop UI
claim. Evidence: build/network-playtest-tests-final.log and
build/network-playtest-native-v1/{session.json,server.log,client-1.log,client-2.log}.
Owned test windows/server are closed. Next: editor Test Player authoring/control
binding and managed launch; F5 remains offline. See RENET_QUALIFICATION.md.

P5-17 / P1-06 per-client authored test spawns, 12 November 2026:
Headless server --test-players accepts a bounded source-bound profile of one/two
local client IDs and full XYZ feet. Validate prepared capsule collision/support
before issuing tickets; reject stale world identities, unsupported/intersecting
positions, unknown/duplicate IDs and incompatible account/persistence/travel modes.
Distinct positions bind authenticated client IDs rather than arrival/actor order;
normal World admission, replay guard and readiness/prediction remain in use.
Actual encrypted valley peers join in reverse order at their configured positions,
then both move through Classic prediction. An unconfigured third authenticated
peer is refused; two test admissions, one refusal, zero persistent admissions,
zero rejected movement/readiness inputs and zero movement recoveries.
All 482 authority unit tests, three actual CLI tests, formatting, strict workspace/
all-target Clippy and client/server/editor builds pass. No display use, new native
render qualification or Windows claim. Evidence: build/test-player-spawn-
{authority-tests,renet,cli-tests,clippy,build}.log. Local generated profile:
build/starter-valley-test-players-v1.json. See RENET_QUALIFICATION.md.
Next: connect profiles/independent impairment to a managed multi-client launcher,
then add editor Test Player placement/properties and normal-input controllers.
Current F5 remains offline; no complete Network Playtest UI claim. Continue locally.

P1-06 / P5-17 configurable test-connection foundation, 12 November 2026:
The loopback qualification relay accepts a checked per-connection Profile with
one-way delay, bounded added jitter, seeded loss and scheduled outages. Explicit
Profile() adds no impairment; old no-argument callers retain their original
fixture exactly. Outages discard queued traffic, queues remain bounded and close
cancels delayed packets. Real UDP tests verify two independent connections,
round-trip delay/payloads, outage/recovery/loss, queue overflow and legacy behavior.
All 18 focused Python tests, formatting and strict workspace/all-target Clippy
pass. Logs: build/network-profile-{tests-final,clippy}.log. No new renderer use,
Windows execution, authenticated Renet or editor-pawn integration claim. Next:
authored test-player definitions, server-validated per-client spawns and normal
client input scripts, then editor-managed Start/Stop and distinct client views.
See RENET_QUALIFICATION.md. Continue locally across commits.

P5-17 selected model preview, 12 November 2026:
Selecting an asset card now shows its actual verified GLB through a standard Bevy
camera in the browser's right-hand preview. Only that presentation instance is
normalized to fit; original materials, authoring coordinates and source bytes are
preserved. Imported descendants receive isolated render/picking ownership before
visibility. Changing/filtering selection or hiding the browser/playtest/particle
workspace releases the instance and stops the camera; loading failures show a
fallback without changing placement/history. Per-card thumbnails and drag-to-place
remain follow-ups.
All 207 ordinary editor tests pass (seven opt-in skips), formatting and denied-
warning workspace/all-target Clippy pass, and client/server/editor builds pass.
Offscreen Vulkan renders a real hut and verifies layer isolation, filtered-instance
removal and camera shutdown. Native Linux input passes category/search/model preview,
placement/undo, independent resize, close/release and exact source preservation,
with zero GPU diagnostics on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6. The owned test
editor is closed; no Windows execution claim. Evidence: build/asset-preview-
{tests,gpu,clippy,build}.log, asset-preview-offscreen.png and
asset-preview-native-v1/{qualification.json,state.png}. Next: prerequisites for
editor Test Player network playtests; existing multi-client launch support does not
yet bind authored pawns/controllers/per-client spawns or impairment settings.

P5-17 terrain startup framing, 12 November 2026:
Initial view and terrain creation now target the actual patch midpoint and sampled
surface height instead of a fixed 64 m offset. Five camera tests pass, including
all supported spans and positive/negative distant origins without authoring edits.
Formatting, denied-warning workspace/all-target Clippy and client/server/editor
builds pass. A settled owned native Linux capture on RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 Vulkan shows the village and foliage at the correct centre, preserves
the exact source file and reports zero GPU diagnostics. The test editor is closed.
Evidence: build/terrain-start-view-{tests,build,clippy}.log and
build/terrain-start-view-settled.{json,log,png}. No Windows execution claim.
Next: model previews in the asset browser; proposed network-test players remain
unimplemented. Continue local work across commits.

P5-17 scenery browser, 12 November 2026:
Assets / View → Asset browser replaces the flat Scenery menu with a collapsible,
height-resizable lower dock, category dropdown, case-insensitive multi-word search,
selectable model cards and explicit Place at spawn. Existing exact placement,
validation/history/Save remain; selection/filtering does not change authoring data.
Filtered selection and stale deferred requests refuse placement. Browser input
ownership survives leaving the panel, search consumes Escape/Enter before editor
shortcuts, and F5 restores its normal full viewport. The left stack is independent.
The category popup uses native placement and bounded scrolling. Image previews,
drag-to-place and a broader mixed asset library remain follow-ups.
All 205 ordinary editor tests pass (seven opt-in skips), formatting and denied-
warning workspace/all-target Clippy pass, and client/server/editor builds pass.
Actual RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan offscreen layout shows 11
model cards and filters to two huts, with unchanged source and an on-screen popup.
Owned native Linux input passes category selection, typing, placement/undo,
independent resize and collapse, preserving the copied source with zero GPU
validation diagnostics. No native Windows qualification. Evidence:
build/asset-browser-{tests-final,clippy-final,app-build-final,gpu-final}.log and
build/asset-browser-native-v3/qualification.json / state.png. Earlier native
attempts exposed a diagnostic helper Path/text error and a drag that moved before
its press had been sampled; the helper now waits for confirmed resize ownership.
The user currently allows mouse/display. The owned test editor has been closed.
Next observed issue: initial terrain view still adds a hardcoded 64 m offset,
so the 512 m valley opens aimed at its old corner. Correct framing before adding
model previews. Test-player/network-playtest and per-client impairment proposals
are recorded in DECISIONS.md; not implemented yet. Broader authored quest gameplay
and multiple NPCs remain active goals. Continue across local commits.

P5-17 / P3-12 / P3-15 editor NPC contact binding, 12 November 2026:
NPC markers now have a permanent inline Catalog ID with validation/history/Save.
Gameplay-template export binds one matching dialogue contact at authored XYZ feet,
rejecting unknown IDs, multiple contacts and unsafe capsule support/headroom/world
boundary intersections. Anchor schema 2 binds the contact into world identity;
legacy schema 1 remains compatible. Server/client/launcher --guide-position authored
resolve against that verified contact and matching dialogue profile.
All 199 ordinary editor tests (six opt-in skips), 241 client tests (two skips),
481 network authority tests and seven launcher tests pass. A focused final export
regression also covers boundary-wall refusal. Formatting and denied-warning
workspace/all-target Clippy pass. Client/server/editor/baker builds pass before
that final export-only guard; the final editor test binary includes the guard.
Offscreen RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan verifies the actual inline
field alignment and unchanged source (build/editor-contact-gpu.log and
editor-contact-inspector.png). No new native interaction or Windows qualification.
The user confirmed they intentionally closed the editor. Their eight saved v2
scenery placement edits remain untouched and are carried into a new v3 zone/world
copy. Fresh nine-tile navigation and 231 route-clearance samples pass. Two actual
encrypted peers use the exported dialogue/placement, open independent conversations
and close on walkaway while the wolf patrol moves; zero rejected movement/readiness/
dialogue and zero recovery resets. Actual CLI startup/shutdown/unsupported-height
refusal and rebuilt client identity match pass. Logs: build/editor-contact-*.log;
source proof: build/editor-contact-source-proof.json; CLI proof:
build/editor-contact-cli-proof.json. Source: build/starter-valley-contact-v3.zone.json;
world/navigation: build/starter-valley-contact-{world,navigation}-v3.
This is one bound contact, not complete quest/persistence or multiple-NPC support.
Unbound markers/no-template exports remain annotations; marker yaw/radius remain
metadata. Proper spell animation, multiple contacts/patrols, pickups and authored
account gameplay remain outstanding. User approved the supplied Bevy editor SVG as
inspiration (DECISIONS.md), retaining the left dock and Sprinkles behavior, and
currently allows mouse/display if needed. Continue across local commits.

P3-12 / P3-15 explicit XYZ contact placement, 12 November 2026:
Guide startup accepts x,y,z feet through launcher/client/server and preserves the
requested support layer. XYZ hashes bind all three coordinates; legacy XZ APIs,
fixtures and hashes are unchanged. Atomic rejection covers unsupported/intersecting
positions and attempts after admission. Synthetic ground/roof worlds verify actual
3D interaction range and no NPC collision. All 480 authority tests, seven launcher
checks, formatting, strict workspace/all-target Clippy and three app builds pass.
Two actual encrypted peers in the dressed valley see the same guide, independently
open dialogue, then close it on walkaway while the wolf patrol moves. Zero rejected
dialogue/movement/readiness and zero movement recoveries. Rebuilt client configuration
matches that exact world identity; actual CLI startup/shutdown and unsupported-height
refusal pass. Evidence: build/layered-guide-{authority-tests,renet-v3,clippy-final,
build}.log and layered-guide-cli-proof.json. No new rendering/Windows qualification.
Initial synthetic test assumed guide allocation did not consume an entity ID;
it now admits explicit ground/roof poses. Initial live harness added an excessive
snap offset and oversized request queue; final proof uses actual supported feet
and the established four-entry bound. Initial client error conversion was corrected.
Editor marker-to-contact/quest bindings, multiple NPCs/patrols, placed quest pickups,
authored account gameplay and a properly sourced/authored spell clip remain open.
User requested the editor for playtesting: launched target/debug/theia-zone-editor
with build/starter-valley-village-v2.zone.json, PID 199365, log
build/starter-valley-editor-playtest.log. This authorizes opening the display for
their playtest; avoid input interference while they use it. Continue source work.

P3-04 / P3-15 full XYZ respawn support, 12 November 2026:
Admission now retains the landing layer beneath the player's full feet position
when preparing immutable respawn home. The former XZ-only highest-ground lookup
could put a ground-floor player onto an overhead roof. A real Avian regression
covers grounded/airborne admission below and above a walkable roof, lethal damage,
respawn, exact horizontal placement, life/stream advancement, and unsupported/
penetrating refusal. All 478 network authority tests pass, including their encrypted
life/combat regressions. Formatting, denied-warning workspace/all-target Clippy,
and rebuilt client/server/editor pass. No presentation or protocol change, desktop
use, new GPU qualification or Windows claim. Logs: build/layered-respawn-{direct,
authority-tests,clippy,build}.log. See docs/PLAYER_LIFE_AUTHORITY.md.
Next: give stationary NPC/guide placement the same explicit height capability,
then connect authored NPC/quest bindings; existing XZ fixture compatibility must
remain exact. Proper spell animation and multi-NPC gameplay are still outstanding.

P5-17 / P2-09 / P3-08 Synty player in editor F5, 12 November 2026:
The editor launcher now selects the locally qualified 22-clip Synty base mannequin.
Player menu switches model/capsule and automatic locomotion or explicit idle,
walk/run, sword stance/swing and death previews. Optional SpellCast stays disabled
because the local art has no casting clip. The game and editor share eight-way
direction hysteresis and normalized phase preservation; no controller or gameplay
authority change. Art loading uses standard Bevy glTF, skinning and animation.
All 197 ordinary editor tests / five opt-in skips, 241 client tests / two skips,
10 launcher/selection checks, formatting and strict workspace/all-target Clippy
pass. Editor/client/server builds pass. A new real RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 Vulkan offscreen check passes: 22 clips, 52 joints, changing sword pose,
directional run, model/capsule toggle, F5 exit and exact unchanged document/feet.
The captured base mannequin image was inspected. No OS window/input was used;
native desktop interaction and Windows are not newly qualified. Asset-watcher/
unavailable-clipboard warnings occur in the windowless harness; no GPU errors.
Initial test compilation missed one fixture velocity field and first Clippy
flagged the pixel iterator; corrected before acceptance. Evidence:
build/editor-player-{build,editor-tests,client-tests,gpu,clippy-v2,launch-tests}.log,
build/editor-synty-player-sword.png. Instructions: docs/ZONE_EDITOR.md.
This does not equip a weapon or add combat to F5. Properly sourced/authored spell
animation, multi-NPC quest/patrol authoring and connected authored account gameplay
remain follow-ups. Continue with full XYZ respawn/contact support for layered
village geometry before enabling the broader connected gameplay bundle.

P3-07 / P3-15 / P2-09 connected valley patrol, 12 November 2026:
The normal launcher accepts authored-world navigation and creature combat while
retaining the account/bundle gates. content/gameplay/starter-valley-wolf-patrol.json
binds the checked northwest grove route. Full XYZ patrol support now survives both
preparation and target installation under an overhead walkable floor. A 300-step
synthetic regression proves it does not jump to the roof. All 823 ordinary network
tests pass (12 opt-in skips), plus both actual authored-world encrypted two-peer
walking/patrol tests. Moving targets and resource combat state replicate; no new
attack/damage, pursuit, loot or quest acceptance is claimed. Zero rejected movement/
readiness and zero recovery resets in the patrol probe. Initial harness attempts
used an incorrect Classic identity base and then an oversized event queue; both
refused before admission and the harness now uses the production bounds/profile.
Formatting, strict workspace/all-target Clippy, 11 launcher/asset-selection tests,
and rebuilt client/server pass. The actual CLI server starts the patrol/combat,
moves it without blocking, shuts down cleanly and refuses an authored XZ spawn
override. The rebuilt client's windowless configuration report matches the actual
network identity, including the selected local Synty art. No desktop was used.
Evidence: build/starter-valley-patrol-{network-tests,clippy,app-build}.log,
-patrol-renet-v4.log, -patrol-server-smoke.{json,log}, -patrol-client-config-final.json.
Connected launch instructions: docs/ZONE_EDITOR.md (Dressed valley connected patrol).
The user was notified earlier that walking is ready; this adds one nonaggressive
wolf and existing resource combat. Multiple wolves/contacts, editor NPC/patrol
bindings, quest pickups and authored account gameplay remain outstanding.

User's additional request: load the Synty base player with sword/spell animations.
The game launcher now prefers the qualified 22-clip directional/sword player art,
with explicit --players override and older-stage fallbacks. Current prepared Synty
art includes no spell-casting clip. Sword, Modular Hero and AnyRPG package inventory
checks found no named casting clip; no reference-game animation was copied.
Next: add the same optional character loading to the editor F5 playtest, then
provide animation preview and complete a properly sourced/authored spell clip.
Keep the broader village/quest editor objective active. Remain headless until the
user permits desktop use; their new player-art request did not change that limit.

P3-07 / P4-03 dressed-valley navigation repair, 12 November 2026:
The missing contour hole-joining step in Rerecast 0.5 caused InvalidContour near
scenery. A bounded adapter of Recast 1.6.0 now joins holes to their region outline
before the unchanged backend triangulator; it never discards obstacle boundaries.
Three new synthetic tests verify excluded area, rejection and work limits. Eight
ordinary baker checks, formatting and denied-warning workspace/all-target Clippy
pass. Both dense actual hut/grove tiles bake; the full nine-tile publication takes
33.2 s and verifies 231 capsule-clearance samples at 0.2 m spacing. A headless
1,200-step authoritative patrol completes one loop, travels 70.01 m and has zero
blocked steps. Offline input caps increase to 32,768 vertices / 65,536 triangles
because one tile measures 24,272 / 45,146; runtime tile/polygon limits are unchanged.
Artifact: build/starter-valley-navigation-v1; logs: starter-valley-navigation-v1.log,
starter-valley-authoritative-patrol.log, starter-valley-navigation-{tests,clippy}.log.
Reproduction: apps/nav-bake/README.md and content/navigation/starter-valley-west-patrol.json.
This covers the village/groves, not the whole 512 m enclosure. No new renderer,
desktop, multiplayer NPC, combat, quest or Windows qualification. Earlier tuning
and border diagnostics failed and were removed; the isolated contour diagnostic
stalled and was terminated. The production bake retains all admitted collision.
Next: expose an actual authored-world wolf patrol in the normal connected client,
then expand bounded NPC/quest/pickup authoring and runtime beyond a single contact.

P5-17 / P3-15 authored player spawn and valley walking launch, 12 November 2026:
server admission now uses the exported, validated full XYZ player spawn. Ground
and rooftop slots retain their support layer; restored admissions keep their
separately validated global pose. Authored worlds reject missing spawns and explicit
XZ developer overrides. Generated/saved terrain retains its existing controls.
All 822 ordinary network tests pass (11 opt-in checks skipped), plus the actual
new-valley two-peer encrypted Renet test with zero rejected inputs/readiness and
zero movement recovery resets. Four focused source/layer tests and four launcher
checks pass. The broader Python run passes 96 ordinary checks; its four missing-bpy
errors are resolved by running the five affected checks in the Blender environment.
Formatting, denied-warning workspace/all-target Clippy and client/server/editor
builds pass. The rebuilt server starts, shuts down cleanly and refuses authored
XZ overrides. Actual RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan client rendering
loads 114 world primitive entities, gates readiness, survives rebasing, withdraws
readiness when a GPU mesh is removed and restores it when reintroduced. No OS
window/input used. See build/authored-spawn-*, authored-valley-*, and
starter-valley-client-gpu-readiness.log / starter-valley-client-offscreen.png.
The user was notified that the editor and game-client walking/camera/scenery
playtests are ready; exact launch commands are in docs/ZONE_EDITOR.md. Normal
native interaction and Windows are not newly qualified. Questgivers, multiple
wolves, pickups and authored account/combat integration remain outstanding.
Next: the full dressed valley navigation probe fails with Recast InvalidContour
at tile [63,64] (build/starter-valley-navigation-probe.log). Resolve actual imported
geometry navigation before adding patrol gameplay; do not publish a partial bake.

P5-17 / P3-15 editable starter village scenery, 12 November 2026:
a separate command recipe builds six Kingdom cottages, 44 trees across the village
and four outskirt groves, ruins, hollow trunk, fallen timber, meeting furniture,
rocks and an exit arch. All 65 models retain ordinary editor gizmos and history;
58 paint strokes connect the outskirt clearings. The 512 m scene exports 232,062
instantiated triangles with 11 shared visuals and 58 solid model instances.
Actual Classic capsule movement reaches all four clearings and the southern exit,
then returns to the village without jump/teleport; sampled routes, not navigation.
Source-exact full export and playtest spawn checks pass. World comparison now
correctly accounts for f64-to-f32 local translation rather than rejecting valid
rounding at larger distances. Strict workspace Clippy and formatting pass.
RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan offscreen overview and close-up
captures pass with all 110 mesh primitives and visible canopies. The harness now
waits for compiled pipelines: earlier images taken too soon lacked foliage.
Final images: build/starter-valley-village-{overview,closeup}-final.png; scene:
build/starter-valley-village-v2.zone.json; recipe and reproduction are documented
in docs/SYNTY_ASSET_REVIEW.md. No desktop used. This is an editable scenery
blockout, not the complete quest zone. Next: preserve authored spawn in the
connected runtime, navigation coverage, NPC quest contacts and wolf patrols.

P1-10 / P5-17 Kingdom huts in the integrated scenery library, 12 November 2026:
11 verified local assets now include two Kingdom huts with source palette art and
an explicit Standard normal-map adaptation. Bounded legacy prefab/model metadata
support preserves exact source IDs; stale blacksmith references still refuse.
Actual headless Linux Unity confirms inherited texture settings and both hut mesh
frames. Independent geometry, UV, winding, normals and source-image checks pass;
source compression/Unity lighting parity and walkable interiors are not claimed.
244 asset-tool tests, eight launcher tests, formatting and strict workspace Clippy
pass. CPU library load/delete and both-hut source-exact world export pass. Actual
RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan offscreen rendering verifies both
huts with move/rotate/scale handles, ray-picked movement/undo and preview cancel.
No desktop/window/input is used. Final GPU logs are build/kingdom-hut0{1,2}-gpu-final.log;
images are in the corresponding -gpu folders. The first capture attempt lacked
its output directories; those image-write failures were corrected and rerun.
Normal launches prefer build/starter-valley-scenery-v2/catalog.json when present.
Village placement and multiple quest/NPC/patrol runtime integration remain next.
See docs/SYNTY_ASSET_REVIEW.md for reproduction, provenance and limitations.

P5-17 / P1-10 static forest trees in the editor library, 12 November 2026:
reuse the already verified full-foliage GLB adaptation with exact material/image
bytes, explicit source wind/highlight/LOD limitations and trunk-only triangle
collision. A nine-asset local catalog adds the small forest tree and large hollow
trunk to the seven existing props. CPU loading creates ten primitive entities
and removes all descendants on deletion; the checker now counts glTF primitives,
not one entity per mesh. Actual RX 7900 XTX offscreen rendering of the tree passes
all three gizmo modes, ray-picked movement/undo and preview cancellation. A world
export preserves all visual attributes, indices, embedded images, materials,
samplers, placement and admitted collision. Six staging and eight launcher tests,
strict workspace Clippy and editor build pass. Evidence: build/starter-valley-
scenery-v1, -tree-gpu.log, -library-check.log, -tree-source-comparison.json.
Normal launches prefer the expanded local catalog when available. No source
payload is tracked. Kingdom village-building conversion is still underway.

P5-17 / P3-15 expanded starter valley, 12 November 2026:
Add → Starter valley terrain creates an editable 512 × 512 m patch with a village
clearing, curved paths, uneven surrounding ridges and one +Z pass. Schema 11 adds
an optional 1..4-chunk span while preserving old one-chunk documents. At most
129 × 129 samples are admitted: 512 m uses 4 m samples; 2 m remains available up
to 256 m. Sculpt, paint, resampling, framing, flat-ground cutout and world export
use the actual extent. The document/command cap is now 2 MiB; the saved valley is
1,266,785 bytes, so the earlier 1 MiB cap was insufficient. This is a bounded
finite authoring patch, with runtime streaming and navigation coverage unchanged.
66 zone tests and 193 ordinary editor tests pass; final ridge adjustments also
pass the two focused geometry/Classic-containment checks. All 24 sampled radial
walk/jump routes exit only through the +Z pass. Actual RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 Vulkan offscreen rendering verifies terrain/paint without a window.
An additional export regression compares 4,096 surface samples against the
authoring patch within 0.1 mm and preserves the full saved document and origin.
Strict workspace Clippy, formatting, editor build and updated bounded stdin
qualification pass. Evidence: build/starter-valley-*, including the v2 zone,
world export, preflight and terrain image. The tracked base/command recipe is
content/zones/starter-valley.*; older starter-zone/basin examples are preserved.
Village buildings/trees, NPC quest assignment, multiple wolf patrols and combined
authored gameplay remain pending. The current image is a terrain blockout.

P5-17 model gizmos and Classic playtest controls, 12 November 2026:
imported scenery now uses the same viewport XYZ arrows, XZ body drag, yaw ring
and uniform-scale handle as primitives. Shared commands retain the source asset,
name, exact global coordinates, undo/redo and explicit Save. Model bounds and
scale admission reject invalid previews; cancellation restores the existing
scene root without changing the document. The game and editor now share the
actual binding sampler and orbit math, including LMB orbit, RMB steer, both-button
forward, wheel zoom, Q/E strafe and walk/run toggle. Offline playtest uses the
Classic ground solver and collision-aware camera; render-frame jump edges survive
until consumed by a fixed tick. This remains an offline authoring sandbox.
Actual offscreen Synty archway rendering on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6
Vulkan verifies all three handle modes, real ray-picked movement/undo, preview
and cancellation with no OS window/input. Evidence: build/editor-model-gizmo-gpu.log
and build/model-{Move,Rotate,Scale}.png. Native interaction/Windows remain pending.
The 190 passing ordinary editor checks plus the corrected cancellation check,
241 client tests, formatting, strict workspace/all-target Clippy and editor build
pass. Two editor and two client opt-in checks are outside ordinary suites.
Larger village terrain, multiple quest contacts and patrol authoring remain next;
the earlier authored-spawn/launcher/minimap gaps are still open.

Previous safe-stop checkpoint — 12 November 2026:
stopped at the user's request after local commit c9e3ca84. No build/test process
or game/editor window remains from this work. Resume with authored-world launch
integration. Newly identified before stopping: the server/launcher still default
to developer spawn [8000,8000], and procedural spawn resolution takes XZ with a
high ground-search ceiling. Preserve the editor-authored spawn (including its
vertical layer) before treating arbitrary authored worlds as supported; do not
silently place an under-roof spawn onto the roof. No fix for this is implemented
yet. Then qualify authored navigation with creature/gameplay/account launch,
adapt the minimap terrain raster, and implement supported NPC/contact/quest
bindings and placed pickups in the editor. Keep mouse/display reserved unless
explicitly reauthorized. Current Synty starter scene is centred at [8000,0,8000]
with an open central spawn, so existing bounded evidence remains applicable.

P5-17 / P3-16 authored navigation, 12 November 2026:
`theia-nav-bake bake-authored` consumes the admitted Avian static meshes/boxes/
hulls/heightfields, clips their surfaces to each tile plus padding, and preserves
the existing nine-tile Recast/Landmass format and budgets. Unsupported shapes
refuse rather than disappear. The actual Synty starter package bakes to nine
navigation tiles; its patrol passes 158 capsule-clearance samples at 0.2 m spacing.
Four routes to clear exterior ground at radius 70 m leave through the +Z opening;
this is sampled routing evidence, not exhaustive navigation. A real authoritative
patrol completes one loop over 900 fixed steps, travels 47.98 m and reports zero
blocks, without adding actor colliders. Two geometry tests and three existing
backend tests pass, alongside the two explicitly invoked local-asset checks;
blocked endpoints and existing outputs are refused without partial publication.
Formatting and strict workspace Clippy pass. Evidence: `build/authored-nav-*`,
`build/authored-patrol-test.log`, `build/starter-zone-navigation-v1/`.
Creature launch/gameplay-bundle wiring, minimap and NPC authoring remain next.

P5-17 / P3-16 authored encrypted-loopback qualification, 12 November 2026:
two real Renet peers load independent snapshots of the exported Synty starter
world, use the Classic prediction driver, remain still before readiness, then
walk while seeing both players. A third stale-identity probe detects world
mismatch before movement. The four-second headless run reports zero rejected
inputs/readiness and movement recoveries; this is not a load/capacity claim.
`build/authored-renet-test.log` records the explicitly invoked proprietary-asset
test. Formatting and strict workspace Clippy pass. The harness supplies CPU
collision readiness; native-client rendering, quests and persistence are not
combined acceptance. Navigation and minimap terrain support remain next.

P5-17 / P3-16 authored client presentation, 12 November 2026:
client/server accept explicit `--authored-world` through the existing prediction
path. The client loads the immutable standard GLB with Bevy world assets, updates
its root from the global frame and gates chunk readiness on scene instantiation,
CPU/GPU meshes, prepared materials and observed PBR pipelines. Removing a mesh
retracts readiness; restoration and a 128 m frame change pass the actual offscreen
GPU test. The exported eight-placement Synty starter scene instantiates 13 meshes
on AMD RX 7900 XTX / RADV NAVI31, Mesa 26.2.6, Vulkan, without a window or OS input.
Evidence: `build/authored-gpu-test.log`, `build/starter-zone-offscreen-v3.png`.
All 241 ordinary client tests pass (two explicit opt-ins ignored); three launcher
selection/rejection checks, formatting, strict workspace Clippy and client/server
builds pass. Client configuration admission passes without starting a renderer or
connection. No connected authored gameplay is claimed yet.
The launcher keeps gameplay-bundle navigation/account paths restricted pending
qualification; authored minimap terrain raster support also remains next.

P5-17 / P3-16 authored prediction foundation, 12 November 2026:
editor exports now include a hash-bound `authored.world.json` companion carrying
the original f64 origin. A bounded immutable network source preserves exported
collision and visual bytes in the existing asynchronous prediction-patch path;
it does not resample terrain or replace the renderer. Two CPU tests cover frame
changes, ground/wall contact, Classic/guide preparation and changed-origin/content
rejection. The scenery export regression verifies the companion and visual
snapshot. Strict workspace Clippy passes. Client rendering/CLI/launcher support
is not connected yet, so this is not a playable authored quest world. Initial
admission requires zero origin Y, bounds inside the existing 16 km domain and
standard embedded visuals without particle/shared-image/material-profile
extensions. A separate `starter-zone` base/command recipe preserves the earlier
basin example and anchors the starter scene around [8000, 0, 8000].

P5-17 / P3-15 / P3-16 starter basin terrain, 12 November 2026:
Add → Starter basin terrain creates a 128 m editable 2 m height patch with a
painted clearing, 16 m high surrounding slopes and one broad +Z opening. It
requires a blank zone, sets the central spawn, preserves locked borders and is
one shared undo/redo transaction. A tracked starter-basin base/command recipe
adds the eight Synty placements, including the arch at the opening. All 64 zone
tests pass; 48 radial walk/jump routes at each of local/far origins leave only
through the intended opening (`build/starter-basin-routes.log`). This samples
offline movement, not exhaustive navigation or connected Classic containment.
Actual saved-scene spawn and runtime export checks pass; artifacts are
`build/starter-basin.zone.json`, `build/starter-basin-world-v1/`,
`build/starter-basin-preflight.json` and `build/starter-basin-source-comparison.json`.
The exported scene has 16,945 visual triangles and unchanged staged scenery
attributes/materials/images. Formatting, strict workspace Clippy and editor build
pass. Native appearance, navigation, NPC quest assignments and pickups remain next.

P5-17 starter library launcher integration, 12 November 2026: normal
`tools/edit_zone.py` launches automatically use the locally staged v2 scenery
catalog when present. Explicit catalog arguments win; `--no-scenery-library`
disables discovery. Missing catalogs leave ordinary editor launches available.
Eight launcher tests and an actual headless dressing-document launch pass;
no display/input access or automatic document write occurred.

P5-17 / P1-10 scenery world publication, 12 November 2026: CLI and File →
Export world now resolve the loaded immutable scenery library into the existing
GLB/schema-4 package. Repeated placements share visual and collision meshes;
embedded image bytes and standard material/sampler references are retained.
Missing assets, unspecified collision, geometry budgets and existing outputs
refuse publication. The eight-placement study exported to ignored
`build/forest-scenery-world-v1/` (seven visuals, five shared collision meshes,
six solid/two decorative placements). Independent source comparison passes for
all vertex attributes, indices, image bytes, material parameters, transforms and
package hashes: `build/starter-forest-scenery-v2/published-source-comparison.json`.
The actual-asset capsule route and pillar contact pass against exported collision
at local/far origins, within 1 cm of offline editor results (ground-contact
rounding differs by millimetres): `published-route.log` alongside that report.
26 export/dialog regressions, two GLB assembly tests, fmt and strict workspace
Clippy pass. Native rendering and connected Synty-zone gameplay remain untested;
the existing two-client graybox export regression passing is not that evidence.

P5-17 / P1-10 explicit Synty scenery collision, 12 November 2026:
starter catalog v2 stages five source triangle-mesh bindings and two decorative
bindings with bounded immutable hash-checked data. Offline playtest resolves them
through standard Avian static meshes and refuses absent assets or unspecified
policy. The eight-placement dressing study admits its spawn with six solid bodies;
evidence: `build/starter-forest-scenery-v2/playtest-preflight.json`. The local-asset
ignored test was explicitly run: capsule passage through the actual arch and
contact with its pillar pass at ordinary and far global origins. Six portable
scenery tests, four Python staging tests and strict workspace Clippy pass. This
is offline collision evidence; scenery world export and native appearance remain
pending. Legacy visual-only catalogs can open but cannot silently enter playtest.

P5-17 / P1-10 model grounding, 12 November 2026: new scenery is grounded
using its scaled lower bound at the terrain sample under its anchor. Existing
scenery supports the same Ground action and undo. Stale document/revision/selection
requests and missing libraries are refused. Five focused scenery tests pass,
including far-origin hanging pivots and preserved saved-file state; formatting
and strict workspace Clippy pass. This is anchor-sample alignment, not full
footprint fitting, stacking or collision admission.

P5-17 / P1-10 Synty scenery placement UI, 12 November 2026:
loading a scenery catalog adds a Scenery menu with named models. Placements appear
in Explorer with a type filter, resolve viewport mesh clicks to their authoring
object, and expose permanent name/position/yaw/scale fields. Enter or leaving a
field applies the accepted value through shared history; disk Save stays explicit.
Duplicate/delete/undo retain the scenery selection contract. A tracked eight-
placement `content/zones/forest-scenery.zone.json` references only logical IDs;
proprietary payloads remain local. All 169 other editor tests passed in the full
run; the new inline test's incorrect pre-Enter expectation was corrected and all
four scenery tests then passed. Formatting and strict workspace Clippy pass.
Actual headless Bevy loading instantiated seven model meshes and removed all seven
on root deletion: `build/starter-forest-scenery-v1/bevy-scene-lifecycle.json`.
Native appearance remains pending. Scenery grounding, export and offline collision
admission are still next; the existing export/playtest guards remain explicit.

P5-17 / P1-10 scenery loading and scene instances, 12 November 2026:
editor `--scenery-library` verifies a bounded, hash-addressed catalog and registers
its immutable bytes with Bevy's in-memory asset source. Standard glTF world assets
follow document placement revisions, with rotated/scaled bounds checked in the
local frame. `--check-scenery` loaded all seven staged Synty models, materials and
images without a window; evidence: `build/starter-forest-scenery-v1/bevy-load.json`.
Two integrity/precision tests and denied-warning workspace Clippy pass. Native
appearance is pending; placement UI and export/collision admission remain next.

P5-17 / P1-10 model-placement document foundation, 12 November 2026:
zone schema 10 stores bounded scenery references by logical asset ID with exact
global position, yaw and uniform scale. Shared commands provide put/delete,
rename, translation, duplication and undo/redo. All 63 zone tests pass; strict
workspace Clippy passes. Model resolution/rendering is next. Export and offline
playtest explicitly refuse scenery until their asset/collision paths are wired;
there is no silent omission or model-collision claim.

P5-17 / P1-10 / P2-08 starter scenery preparation, 12 November 2026:
the user authorized using local Downloads/Synty assets for the starter-zone work.
The actual folder is `Downloads/Synty Models`. Seven existing verified Enchanted
Forest prototypes (archway, ruins, table, chair, lantern, log, pebbles) are staged
under ignored `build/starter-forest-scenery-v1/` with exact hashes and bounds
(8,731,372 bytes). Three staging regression tests pass. Assets remain local;
editor model placement/export is the next slice. This is staging, not native
rendering or collision qualification. Medieval Camp is supplied as Unreal assets;
cliff projected materials need their supported binding path.

P5-17 / P3-15 selected quest-area preview, 12 November 2026: selecting an
area now builds a standard Bevy mesh outline of its bounds. Twelve fixed edges
share one mesh/material, ignore picking, have no collider and hide during offline
playtest. Headless scene checks cover far-coordinate precision, edits, deletion,
undo and asset counts. Native GPU/appearance checks are explicitly deferred under
the user's no-display instruction; no rendered acceptance is claimed.

P5-17 / P3-15 integrated gameplay export dialog, 12 November 2026:
File → Export world now offers an optional gameplay-template manifest field;
quest-area zones require it. The existing background export route publishes the
same validated combined artifacts as the CLI. Headless dialog tests cover both
ordinary and gameplay exports, particle-save gating, invalid revisions and
existing-output refusal. Chunk export retains its two-field form. Native visual
and keyboard acceptance remain pending; no display or OS input was used.

P5-17 / P3-15 combined world/gameplay export, 12 November 2026:
`--export NEW_FOLDER --gameplay-template MANIFEST` now publishes terrain/collision,
particles when present, full authoring source and a validated gameplay copy with
the authored quest-area catalog. Other gameplay profiles retain exact validated
bytes; source files and existing destinations remain unchanged. All 22 export
checks pass, including the existing connected geometry test; the combined
artifact also passed the actual launcher snapshot path headlessly
(`build/zone-gameplay-headless-20261112-v2/verification.json`). Staged failures
clean up without publishing. This is artifact/launcher validation, not connected
quest-credit or native rendering evidence. GUI template selection is next; NPC
markers and world pickups are not made executable by this slice.

P5-17 / P3-15 gameplay-publication prerequisite, 12 November 2026: offline
manifest validation now lives in `theia_network::gameplay_bundle`; the existing
validator CLI uses the same implementation. A validated snapshot retains the
exact manifest/profile bytes from that pass for editor publication. All 62
existing validator tests and a source-change snapshot regression pass. This
changes no admitted gameplay schema and does not itself publish or activate a
world; combined editor export is next.

P5-17 / P3-15 quest-area creation and binding fields, 12 November 2026:
Add → Quest area at spawn creates an undoable 8 m volume with a unique definition
ID, including near the authoring boundary. Attributes now edits Area ID and
comma-separated minimap objective IDs through the permanent text-field route.
Invalid IDs/duplicate links preserve the document; names and bounds remain
editable, and Save remains explicit. Headless edge-spawn, selection/history and
real text-event tests pass; native UI acceptance and integrated publication remain
pending. Objective IDs are format-checked on edit and resolved against the chosen
quest catalog on export.

P5-17 / P3-15 quest-area inspector, 12 November 2026: authored areas appear
in the explorer and its type filter, with permanent exact-f64 minimum/maximum
fields and the shared inline name editor. Selection survives edits; inverted
bounds are rejected atomically. Frame, context actions, delete and undo work;
unsupported duplicate/ground/gizmo operations are disabled. Headless CPU-layout
and edit-event tests pass. No native visual acceptance is claimed while the user
reserves the display; area creation and gameplay-link editing remain next.

P5-17 / P3-15 / P3-11 quest-area catalog export, 12 November 2026:
`--export-quest-areas NEW_FILE --quests QUEST_CATALOG --content-id ID --revision N`
exports authored schema-9 volumes through the existing area and quest-reference
validators. Complete bytes are published without replacing an existing file;
source catalogs are preserved. Four actual headless editor cases pass, including
unknown objective rejection and incompatible CLI modes
(`build/zone-quest-areas-headless-20261112/verification.json`). This produces a
standalone catalog; server bundle activation and visual area controls remain open.

P5-17 / P3-15 / P3-11 quest-area authoring foundation, 12 November 2026:
zone schema 9 adds explicit bounded global area volumes, stable definition IDs
and minimap objective links. Shared commands provide transactional put/delete,
rename, translation and undo/redo; draft bytes pass the existing runtime area
parser. Far-coordinate, malformed-data, duplicate-ID and redo-preservation checks
pass. Ordinary world export refuses these drafts until gameplay export is wired.
No visual area editing, resolved quest binding or server publication is claimed
by this slice; older quest markers remain annotations.

P5-17 / P2-13 headless cache closeout, 12 November 2026: a windowless probe
ran 1,000 two-owner effect/cache open-close cycles and 120 seconds of idle
updates. Image and effect assets returned to zero after every close. Sampled
Linux RSS ranged from 59,160 to 59,244 KiB. Evidence:
`build/particle-headless-cache-soak-20261112/report.json`. This covers CPU
baked gradient/curve caches only; full-editor and GPU memory stability remain
unqualified. Native saved-preference interaction also remains pending while the
user reserves mouse/display. Work now moves to the adopted starter-zone editor
milestone; those display-dependent checks are explicitly deferred.

P2-13 particle preference startup checks, 12 November 2026: the actual client
configuration path accepts saved world-effects visibility, respects the temporary
hide override, rejects malformed values and leaves the preference file unchanged.
All 13 CLI cases pass (`build/particle-preferences-cli-20261112/verification.json`).
Preference save feedback now refers to preferences rather than UI size alone.
This is startup and headless persistence evidence; native save/relaunch and the
editor memory soak remain the bounded closeout tasks in SPRINKLES_INTEGRATION.

P2-13 world-effects preference, 11 November 2026: when fantasy UI and the
explicit `--ui-preferences FILE` store are enabled, changing the World effects
control persists its visibility alongside UI size. Older preference files remain
valid; `--hide-world-particles` overrides the saved value for that launch without
rewriting it unless the user changes the control. Headless tests cover reload,
rapid changes during a pending save and the override. Two connected Vulkan
clients also passed the hidden-effects scenario on RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 (`build/particle-preferences-native-20261111/verification.json`);
the saved-preference path itself has not been tested with native input. Writes
remain asynchronous and failures follow the existing preference-store behavior.

P2-13 combined projected-material and particle package foundation, 11 November
2026: schema 5 binds both requirements under a new identity domain; Rust and
Python agree on a golden fingerprint, missing dependencies are rejected, and
visual-only loading remains closed. The overlay publisher preserves material
requirements when adding, replacing or removing particles. An editor-produced
effect was published into a far-origin projected-material chunk world; two
connected Vulkan clients verified actual projected-material bindings and visible
particle emitters with no validation diagnostics on RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 (`build/projected-particle-native-20261111-v4/verification.json`). The editor's actual File-menu export was also exercised at the same far origin; it
preserved the source files, and its published output passed the same two-client
checks (`build/projected-particle-editor-native-20261111/qualification.json`,
`build/projected-particle-editor-world-native-20261111/verification.json`). The editor's explicit empty replacement
also removed the catalog while preserving schema-3 projected bindings, verified
on two Vulkan clients (`build/projected-particle-editor-clear-native-20261111/`,
`build/projected-particle-editor-clear-world-native-20261111/verification.json`). A separate bounded fixture combined projected visuals with paired-world collision; two Vulkan clients verified material bindings, Tube Trails particle retirement and render-origin translation during a chunk transition (`build/projected-particle-stream-native-20261111-v2/verification.json`). Visual and collision shape equivalence was not tested. Corrupting the incoming particle effect rejected the transition while preserving active roots and material bindings on both clients (`build/projected-particle-stream-corruption-20261111/tampered/verification.json`).

P2-13 world-space particle translation foundation, 11 November 2026: the
Sprinkles GPU runtime can queue a render-frame offset without advancing or
restarting the emitter, including while paused and between fixed steps. A native
GPU readback on RX 7900 XTX / RADV verified the active head and all 120 trail
history entries moved by the exact requested offset with no Vulkan validation
diagnostics (`build/particle-gpu-rebase-20261111/result.json`). This capability
is now connected to ordinary client render-origin shifts; local-space emitters
are untouched. Content-frame changes or out-of-bounds offsets still clear without
rewinding the emitter. Two connected Vulkan clients completed the one-shot 3D
Explosion under forced rebases with translations and no fallback clears or GPU
validation diagnostics (`build/chunk-explosion-translate-native-20261111/`).
The upstream Tube Trails also passed a two-client chunk transition
(`build/chunk-trails-translate-native-20261111/`). A second paused GPU readback
verified all three rendered trail segments and 120 history entries moved exactly
(`build/particle-gpu-rebase-segments-20261111/result.json`). These are scenario
and buffer checks, not frame-synchronized visual equivalence tests.

P2-13 / P5-17 fixed-step Sprinkles comparison, 11 November 2026: all seven
bundled effects were captured at five matching update counts in the pinned
upstream preview and Theia port with fixed simulation steps and in-memory seeds,
plus a Theia repeat. Representative frames look close; selected-emitter outlines
differ. The original debug host emitted repeated Vulkan validation diagnostics,
while Theia's runs had none. GPU scheduling, shared background and in-world
differences limit the conclusion. See [comparison](SPRINKLES_PARITY.md).

P2-13 particle chunk authoring foundation, 11 November 2026: verified particle
catalogs can now be drafted per half-open chunk owner, with placements projected
through global coordinates and only the effects and role-specific textures used
by that chunk. Tests cover far origins, negative and exact-boundary placement,
dependency selection and missing output chunks.
The draft also retains exact verified effect and decoded-validated texture file
bytes, enforces an aggregate output-copy limit and rejects tampered inputs before
any publication. `draft_particle_chunks SOURCE_WORLD CHUNK_WORLD NEW_OUTPUT
ORIGIN_X ORIGIN_Y ORIGIN_Z` now writes these unpublished overlays for an existing
chunk index, recording both source identities and the explicit global origin. It
was exercised against the editor-produced export
(`build/editor-particle-overlay-draft-20261111/`) and a textured fixture.
`publish_particle_chunks CHUNK_WORLD OVERLAY_DRAFT NEW_OUTPUT` copies and verifies
indexed geometry and collision, binds overlays to incremented chunk revisions,
and publishes the new index last. It refuses existing output, altered catalogs,
particle-bearing input unless replacement is explicitly selected; projected-
material chunks now use schema 5 to retain their material requirements. A headless test covers publication and
failure cleanup; the editor export rendered from the resulting chunk world on
two connected Vulkan clients on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 with zero
validation diagnostics (`build/editor-particle-chunk-world-validated-native-20261111/verification.json`).
This combines effects with existing chunks; it does not partition editor terrain.
A textured particle overlay on a schema-2 shared-image chunk world also retained
the actual terrain image bindings on two Vulkan clients
(`build/shared-particle-world-native-20261111-v3/verification.json`).

P5-17 / P2-13 integrated chunk export, 11 November 2026: **File → Export
particles to chunks…** takes a new output folder and existing chunk world, uses
the current zone origin and saved effects, and runs the verified draft/publication
pipeline in the background. It preserves the source zone and chunk world; normal
Save remains explicit. A native mouse/keyboard test covered the menu, text entry,
Tab/Shift-Tab and export (`build/chunk-export-dialog-native-20261111/`); the
resulting world rendered on two connected Vulkan clients
(`build/chunk-export-dialog-world-native-20261111/`) on RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.6 with zero validation diagnostics. The original world-export dialog
also passed its native regression (`build/world-export-dialog-regression-20261111/`).
This requires an existing particle-free schema-1/2/3 chunk world; it does not
partition the editor's terrain. Projected-material schema-3 packages are now
supported as described above.
Particle chunk export now drafts directly from saved particle files instead of
constructing a temporary terrain world. A native export from a zone without
objects passed (`build/chunk-export-empty-zone-native-20261111/`).
**File → Replace all world particles…** explicitly creates a new copy in which
all existing world particle placements are replaced by the current zone's; an
empty zone clears them. It verifies the previous packages and increments affected
revisions. Native menu tests covered replacement and clearing while preserving
the original (`build/particle-replace-dialog-native-20261111/` and
`build/particle-clear-dialog-native-20261111/`); connected Vulkan clients tested
both resulting worlds. This is a whole-world operation, not a per-zone merge.

P2-13 particle image admission, 11 November 2026: monolithic and chunk clients
now apply their decoded-image allowance to verified PNG headers before invoking
the image decoder, and recheck actual decoded bytes afterwards. Chunk preparation
passes the remaining allowance across catalogs. This bounds admitted base-level
RGBA bytes; it is not a complete GPU-memory budget. A textured two-client Vulkan
transition and corruption checks pass with this admission path
(`build/chunk-particle-image-budget-20261111/`).
For staged chunks, the allowance is reduced by active image bytes before the
worker reads and decodes the candidate, including retained snapshots twice.
A valid, hash-bound 4096×2049 incoming PNG exceeds the 32 MiB base-level limit
and was rejected on both Vulkan clients without retiring their active roots
(`build/chunk-particle-overbudget-20261111/oversized/verification.json`).

P2-13 chunk particles, 11 November 2026: chunk clients prepare verified
particle catalogs off-thread, check conservative active-plus-candidate budgets,
and publish and retire their particle roots with the corresponding chunk scene.
Retained chunks keep their existing roots; startup waits for scene, network and
render readiness. A headless failure test and a two-client encrypted Renet
transition with Vulkan validation cover retention and retirement on RX 7900 XTX
(`build/chunk-particle-native-20261111/retirement/verification.json`). This is
a small local fixture, not an MMO-scale or performance qualification. A second
two-client run corrupts the incoming effect and confirms both clients retain the
old roots (`build/chunk-particle-native-20261111/tampered-verified/verification.json`).
The same two-client transition now also passes with one cataloged sRGB texture;
corrupting either the effect or its texture rejects the candidate while retaining
the existing roots (`build/chunk-particle-textured-20261111/`). This does not
establish texture fidelity or texture-heavy performance.

P2-13 chunk particle preparation, 11 November 2026: a worker-suitable helper
verifies each already-admitted chunk manifest, visual and particle catalog,
prepares effects and verified images, projects placements from each chunk's f64
origin to the index frame, and enforces aggregate candidate counts and decoded
image limits. A far-origin mixed-residency test covers repeated effects, aggregate
limits, tampering and unchanged active residency. The staging check counts old
runtime particles plus only newly bound chunks; it still counts decoded images
from both snapshots. Larger-world budgets and texture-heavy transitions remain
unqualified. Retention and corruption were rerun against this accounting with
two Vulkan clients (`build/chunk-particle-reuse-20261111/`).
The upstream world-space Tube Trails effect also exercised floating-origin
resets during a two-client chunk transition with Vulkan validation
(`build/chunk-particle-rebase-suite-20261111/retirement/verification.json`).
This checks the reset path, not frame-synchronized visual parity.
Before the translation change above, the upstream 3D Explosion finished after
six forced clears on each of two Vulkan clients, with zero pre-ready emitter
frames (`build/chunk-explosion-rebase-native-20261111/`); the clear caused a
visual discontinuity for already-live world-space particles.
The same one-shot was also tested when its chunk arrived after startup: both
clients released and finished it with zero pre-ready emitter frames
(`build/chunk-explosion-arrival-native-20261111-v2/`). Captures are not
synchronized; one visibly shows the burst while the other shows little of it
under the forced rebase stress.

P5-17 / P2-13 editor world export, 11 November 2026: **File → Export world…**
opens a Sprinkles-styled dialog for a new output folder, content ID and revision.
It exports the current zone snapshot on a background worker, refuses unsaved
particle edits and existing destinations, and leaves source files unchanged.
A native Linux mouse/keyboard test covered menu access, field focus, Tab/Shift-Tab,
revision editing and particle-bearing publication with zero Vulkan diagnostics
(`build/editor-export-dialog-native-20261111-v6/qualification.json`). The exact
GUI-produced package then rendered its particle effect on two connected clients
(`build/editor-export-world-native-20261111-v2/verification.json`) on RX 7900
XTX / RADV NAVI31 / Mesa 26.2.6. Save still explicitly saves the source zone;
export does not do so. The exported world remains a bounded monolithic slice.

P2-13 world-particle startup readiness, 11 November 2026: world effects are
constructed in advance but held paused before their first simulation step until
the terrain scene, render pipelines and a connected snapshot are ready. This
fixes a measured case where a short upstream explosion advanced hundreds of
emitter-frames while the scene was still loading. Two connected native Linux
clients captured the upstream 3D Explosion and Magic Puff after readiness with
zero pre-ready emitter-frames and zero Vulkan validation diagnostics on RX 7900
XTX / RADV NAVI31 / Mesa 26.2.6
(`build/upstream-explosion-world-native-20261111-v5/` and
`build/upstream-magic-world-native-20261111-v1/`, `particle-first.png`).
Explosion completion/despawn was observed; a separate Tube Trails run retained
world-space rebase behavior (`build/particle-world-trails-rebase-native-20261111-v2/`).
These captures are not synchronized upstream parity or an MMO load test.

P2-13 world-effect visibility control, 11 November 2026: worlds with placed
particle effects expose a clickable World effects control and F9 shortcut, plus
`--hide-world-particles` for starting hidden. It changes only world-particle
visibility; gameplay cues remain separate. A native two-client X11 test used one
actual click and two F9 presses on one client while the other retained effects
(`build/particle-control-os-native-20261111-v2/verification.json`). Starting
Windy Snow hidden restored the pawn-pixel checks on both clients
(`build/upstream-snow-hidden-world-native-20261111-v1/verification.json`);
both runs recorded zero Vulkan validation diagnostics on RX 7900 XTX / RADV
NAVI31 / Mesa 26.2.6. Unit tests cover focus and cue isolation. The switch is a
manual readability aid, not automatic critical-telegraph protection, a
persistence setting, or a simulation/GPU workload reduction.
The control now hides when the active chunk set has no particle placements and
returns when placements load. Two-client Vulkan checks cover both an empty world
(`build/particle-control-empty-native-20261111/`) and a populated world with real
mouse/keyboard toggles (`build/particle-control-present-native-20261111/`).
Both clients also started in a ready particle-free scene and showed the control
after a particle-bearing chunk arrived (`build/chunk-particle-arrival-native-20261111/`).
The reverse transition also passed on both clients: after the last particle
chunk retired, the control disappeared and no particle roots remained
(`build/chunk-particle-departure-native-20261111-v2/`).

P2-13 upstream example package check, 11 November 2026: all seven bundled
Sprinkles example RONs exported and passed actual-client pre-launch admission
(`build/upstream-particle-package-20261111-v3/qualification.json`). Two-client
native Vulkan tests passed for Acid Pool, Loot, Rain and Tube Trails with zero
GPU validation diagnostics (`build/upstream-{acid,loot,rain}-world-native-20261111-v1/`
and `build/particle-world-trails-rebase-native-20261111-v1/`). Windy Snow also
rendered on both clients with zero diagnostics, but the automated pawn-pixel
check failed because its dense streaks changed player colors
(`build/upstream-snow-world-native-20261111-v1/`). The upstream editor capture
shows similarly dense authored streaks. This demonstrates a gameplay visibility
risk for large environmental effects; these checks do not establish visual
parity or protection of combat indicators. One-shot examples have only been
checked through export and admission in this slice.

P2-13 world-space particle rebases, 11 November 2026: bounded monolithic
worlds now admit both local- and world-coordinate Sprinkles emitters. On a
render/content origin change, the original implementation cleared and restarted
world-space emitters before simulation. It has since been replaced as described
at the top of this file. Local-coordinate emitters are left running. Two connected
native Linux clients rendered a textured world-space effect and upstream
Tube Trails during repeated forced render rebases with zero GPU validation
diagnostics on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan
(`build/particle-world-rebase-native-20261111-v1/verification.json` and
`build/particle-world-trails-rebase-native-20261111-v1/verification.json`).
Each client recorded 3–4 world-space resets; screenshots show the effects after
movement. This is not a per-frame visual equivalence or MMO-load result.

P2-13 bounded zone particle playback, 11 November 2026: the monolithic
`--terrain-world` client now admits manifest-bound particle packages, validates
simulation data and explicit resource counts, binds verified decoded images and
spawns anchored Sprinkles effects. This initial slice required local-coordinate emitters; the subsequent rebase
slice above adds world-space support. Chunk-world playback remains unsupported. A
real editor-exported textured effect was visible on both native Linux clients
connected to an authoritative server over encrypted Renet; the test recorded
one running visible emitter per client and zero GPU validation diagnostics on
AMD Radeon RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan
(`build/particle-world-native-20261111-v5/verification.json`, captures in `a/`
and `b/`). Actual-client admission tests rejected tampered catalog/effect/image
files, excessive particle counts and (at that checkpoint) world-space emitters. Frame-rebase unit
tests preserve the local effect's anchor, rotation and scale; this does not yet
qualify live GPU rebases, total GPU memory or MMO-scale cosmetic load.

P2-13 particle simulation preparation, 11 November 2026: package preparation
now applies a conservative 3D/emitter/collider, timing, reflected numeric and
curve/gradient complexity check before resource counting. Sprinkles' documented
infinite material attenuation default is retained. Tests reject non-finite,
excessive, unordered or invalid values; all seven upstream example files pass
this numeric subset check, which does not mean they fit runtime budgets or have
been rendered. Complete semantic validation, GPU memory admission and client
package playback remain open.

P2-13 particle world export, 11 November 2026: the zone exporter can now
stage a particle-bearing schema-4 world package with a manifest-bound catalog,
portable effect snapshots and content-hashed, role-specific PNG variants.
Missing effects fail before destination creation; staged files are verified
before publication and the manifest is moved last. Export tests cover real
texture dependencies, source-file changes and tampering. The report explicitly
states that client playback is unavailable: clients still reject schema 4 until
particle admission and rendering are integrated. This is a local packaging step,
not playable zone effects or an MMO-scale performance claim.

P2-13 verified particle texture handles, 11 November 2026: prepared catalog
images can be installed in Bevy and bound to Sprinkles materials by strong,
runtime-only handles, avoiding a second path-based disk read. Binding checks
all role-specific references before inserting assets. Tests cover mismatched
packages, changed source files and distinct sRGB/linear variants; runtime handles
cannot be serialized into editor files. A windowless native Vulkan smoke test
confirmed that the Sprinkles material receives the supplied image handle on AMD
Radeon RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6, with zero GPU validation
diagnostics (`build/particle-verified-texture-gpu-20261111/gpu.log`); injected
invalid-material cases deliberately log errors. No package is played by the
client yet, and complete simulation and GPU memory admission remain open.

P2-13 bounded projectile cosmetic, 11 November 2026: projectile-enabled clients
run a bundled, editable Sprinkles RON as a child of the existing server-driven
projectile indicator. The indicator and authoritative impact remain separate;
parent cleanup removes the effect, local particle coordinates follow the render
frame, and failed asset parsing leaves the indicator intact. The fixed effect
uses 12 particle slots per cue, at most 32 active cues. A two-client native
qualification covers three launches/impacts, synthetic action-bar pointer and
keyboard inputs, visible running emitters on both clients and zero GPU validation
diagnostics on AMD Radeon RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan:
`build/sprinkles-projectile-native-20261111-v4/verification.json` and captures.
The test focuses the caster window and accommodates the current action-bar slots.
This is a fixed projectile prototype, not zone catalog playback, complete cosmetic
memory budgeting, Windows execution or an MMO-scale load result.

P2-13 manifest-bound particle preparation, 11 November 2026: a CPU-only helper
combines cataloged effect loading, explicit count-limit checks, checked frame
projection and verified PNG decoding into an all-or-nothing snapshot. A test
binds a catalog to a schema-4 manifest and rejects changed effects/textures,
out-of-frame placements and insufficient limits. Six VFX tests and strict
workspace Clippy pass. Client rendering, texture registration, complete effect
admission and export remain blocked. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle catalog allocation counts, 11 November 2026: the VFX helper
conservatively totals emitters, mesh vertices/indices, particle slots, trail
history and sub-emitter buffer words across each placement, including disabled
emitters and repeated effects. Callers can compare the checked totals to
explicit limits. Five VFX tests and strict workspace Clippy pass. These counts
do not include textures or other allocations and are not a complete memory
budget, playback admission or MMO capacity claim. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle placement projection, 11 November 2026: the VFX helper projects
catalog placements from checked content coordinates through f64 global space
into a bounded render frame, then composes the effect's initial transform.
Tests cover a far origin, shifted and restored frames, missing effects and
non-finite/out-of-frame transforms. This is a CPU transform helper; no particle
rendering, origin-shifted trails, client playback or gameplay admission is
claimed. Four VFX tests pass; see [integration](SPRINKLES_INTEGRATION.md).

P5-17 selection-aware properties heading, 11 November 2026: the dock heading
now identifies the selected particle, object, guide, marker, plant, spawn or
terrain instead of only the active tool mode. A native particle gradient
Save/reopen run and screenshot show “Properties · Particle”; Linux RX 7900 XTX /
RADV NAVI31 / Mesa 26.2.6 Vulkan reported zero GPU diagnostics. Fmt and strict
workspace Clippy pass. Evidence: `build/particle-gradient-native-20261111-v4/`.

P5-17 particle gradient Save/reopen, 11 November 2026: native input edits a
gradient while its zone and effect files stay unchanged, invokes the editor's
Save button, then opens a fresh editor and verifies the exact saved stop values.
Both Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan runs reported zero
GPU diagnostics. Evidence: `build/particle-gradient-native-20261111-v3/`.

P5-17 initial-color variant qualification, 11 November 2026: native input
switches the original particle initial-color control from Solid to Gradient and
confirms its nested gradient control appears in the open popup without losing
selection. The same run repeats shape/radius editing across the dock divider.
Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan reported zero GPU
diagnostics. Evidence: `build/particle-shape-native-20261111-v2/`.

P2-13 catalog texture decoding, 11 November 2026: read bounded catalog PNGs
into owned buffers and verify their hashes, headers and aggregate decoded-size
estimate before Bevy decodes those same bytes in the cataloged color space.
Focused catalog/VFX tests cover tampering, malformed headers, bad image payloads
and sRGB/linear variants. This is not yet connected to package-path asset
registration, client rendering, resource budgets or playback. See
[integration](SPRINKLES_INTEGRATION.md).

P2-13 catalog effect loading, 11 November 2026: the network catalog can read
bounded effect bytes and verify the exact returned buffers against their hashes.
The VFX helper migrates those buffers, checks allocation arithmetic and
cataloged texture roles, and rebases cloned references. Three focused catalog
and three VFX tests plus strict workspace Clippy pass. This performs blocking
I/O and is not wired to client playback; texture bytes/decoding, complete
parameter validation, allocation budgets, export and playback remain pending.
See [integration](SPRINKLES_INTEGRATION.md).

P5-17 particle variant popup repair, 11 November 2026: native reproduction
showed Point → Sphere changed the effect but did not create the radius field in
the already-open popup. Keep a hidden field container for fieldless variants and
rebuild its rows and scrollbar when selecting another variant. Native input
now changes Point → Sphere, edits Radius, then changes Sphere → Point → Ring,
including dropdown clicks over the dock divider; selection and source files are
preserved. Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan reported zero
GPU diagnostics. Evidence: `build/particle-shape-native-20261111-v1/`.

P5-17 top-menu/divider qualification, 11 November 2026: resized the dock so
the divider crossed each View and Zone popup, then pressed menu commands
directly over it. Top view and particle deletion dispatched without panel
resizing; the saved source remained unchanged. Screenshot confirms the View
menu draws over the divider. Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6
Vulkan reported zero GPU diagnostics. Evidence:
`build/menu-divider-native-20261111-v1/qualification.json`.

P5-17 inline particle gradient qualification, 11 November 2026: native input
opens the original gradient in the zone inspector, drags its first stop across
the dock divider without resizing or opening a picker, adds and deletes a stop
with its row, and edits a stop color while retaining the selected particle and
Attributes tab. Zone and effect files remain unchanged until Save. On Linux
RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan there were zero GPU diagnostics;
one run observed gradient opening in 0.53 s and nested color opening in 0.47 s,
including test input delays. This single observation is not a latency guarantee
or proof that the user's original slow first-open case is resolved. Evidence:
`build/particle-gradient-native-20261111-v2/qualification.json`.

P5-17 particle deletion and Exit interaction, 11 November 2026: native input
deletes an unassigned placement through Properties and Zone menus, undoes it,
and verifies dirty Exit cancellation and keyboard-confirmed discard while the
original zone file remains unchanged. Fix Tab focus movement between Exit
confirmation buttons. Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan,
zero GPU diagnostics; evidence `build/particle-delete-exit-native-20261111-v3/`.
This controlled fixture passes; the original reported deletion failure's exact
trigger has not been reproduced. Gradient/popup native retesting remains open.

P5-17 editor startup schedule repair, 11 November 2026: the initial native
check found an ordering cycle introduced by the Exit handler. Move it outside
the zone system set while retaining the zone activity guard; a headless exit
regression now includes the inline inspector's real scheduling relationship.
The editor starts on Linux Vulkan without the cycle. The older end-to-end
qualifier then timed out because it expects the retired separate property form;
current inline and Exit menu native checks remain pending. Fmt, focused tests
and strict workspace Clippy pass. Evidence: `build/zone-native-20261111-current/`
and `build/zone-native-20261111-schedule-fix/`.

P5-17 exploratory Sprinkles visual comparison, 11 November 2026: captured all
seven original examples in the pinned upstream editor and the ported standalone
host; representative stills appeared broadly similar. Short clips cover two
effects. Runs used Vulkan on RX 7900 XTX / RADV Mesa 26.2.6, with validation
errors in the original debug build. Unsynchronized seeds/timing, the embedded
zone and full animation parity remain unqualified. See [comparison](SPRINKLES_PARITY.md).

P5-17 editor exit and particle deletion access, 11 November 2026: File → Exit
closes a clean editor and asks before discarding unsaved zone or particle work;
Zone → Delete selected exposes the shared placement deletion action. The real
menu activation and exit guards pass headless tests; the reported native
properties-menu deletion failure remains unreproduced. Native interaction and
the confirmation's appearance still require testing. See
[integration](SPRINKLES_INTEGRATION.md).

P5-17 gradient point and particle-delete investigation, 11 November 2026:
a deliberate gradient-point click can open its nested color picker, while drag
and post-drag clicks do not; a new press clears a stale drag marker. Headless
tests also cover deleting an unassigned particle through the command, key,
explorer and properties menus. A user-reported native deletion failure remains
unreproduced, and native gradient retesting remains open. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 menu/divider ownership, 11 November 2026: both native menus and
Sprinkles popovers own pointer gestures above dock dividers, and top-bar menus
render above the divider. Focused headless menu and divider tests pass; native
visual retesting remains pending. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 popup/divider input repair, 11 November 2026: floating Sprinkles
popups take precedence over dock divider gestures, preventing gradient-point
drags from resizing either panel split. Five dock tests, formatting and strict
workspace Clippy pass. Native interaction retesting remains pending. See
[integration](SPRINKLES_INTEGRATION.md).

P5-17 live gradient editing, 11 November 2026: original Sprinkles gradient
change events update the live effect before final commit; unchanged follow-up
events do not restart it again. Sixty-three particle-editor tests, formatting
and strict workspace Clippy pass. Native drag behavior needs retesting. See
[integration](SPRINKLES_INTEGRATION.md).

P5-17 curve drag guard, 11 November 2026: Sprinkles curve-point dragging
refuses invalid neighbor or value ranges instead of panicking when points overlap
at an endpoint. A focused headless regression covers overlapping and reversed
ranges; native curve interaction remains to be retested. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 gradient point deletion repair, 11 November 2026: committing a
Sprinkles gradient replaces its full live value so removed points stay removed;
endpoint drags and stale delete indices are guarded. Focused tests exercise
actual effect binding and menu-driven row removal. All 61 particle-editor
tests, formatting and strict workspace Clippy pass. First-open latency and
native interactive retesting remain open. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 inline popup input repair, 11 November 2026: opening presses no longer
immediately dismiss Sprinkles popups, and floating popup gestures block world
selection through release. A second color-swatch lifetime race is guarded.
Fifty-seven particle-editor and 143 zone-editor tests and strict workspace
Clippy pass; user interaction retesting is underway.
See [integration](SPRINKLES_INTEGRATION.md).

P5-17 inline popup repair, 11 November 2026: embedded Sprinkles popovers
are no longer hidden behind the workspace gate and receive global stacking;
a gradient color-picker removal race is guarded. Headless regression tests and
strict workspace Clippy pass. Native interaction retesting is in progress.
See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle texture-reference foundation, 11 November 2026: the VFX helper
checks original Sprinkles standard-material references against cataloged
role-specific texture names, rejects editor/local paths and unused descriptors,
and rebases cloned effects to the package asset root. Both VFX tests pass;
this does not verify file bytes or admit, export or play particle packages.
See [integration](SPRINKLES_INTEGRATION.md).

P5-17 selected particle auto-open, 11 November 2026: selecting a local
particle opens its original Sprinkles project into the shared inspector after
pending field edits and project operations settle. Unsaved projects block the
switch; local files are checked for presence and size. Focused headless tests
cover dirty and focused-field guards, retry suppression and non-UTF-8 paths.
All 53 particle-editor and 143 zone-editor tests, formatting and strict
workspace/all-target Clippy pass. No display or mouse was used; native interaction and GPU qualification remain
pending, as do world export and client playback. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 shared inline particle inspector, 11 November 2026: a selected
reference whose local effect is already open moves the original Sprinkles
inspector into the zone dock, starts its preview, supports emitter stepping and
explicit Save. Fifty-two particle-editor and 142 zone-editor tests pass, plus
focused tests after the final scheduling guard; fmt and strict workspace/all-
target Clippy pass. No display or mouse was used; native layout, popups and GPU
behavior remain unqualified. Automatic opening and client playback remain open.
See [integration](SPRINKLES_INTEGRATION.md).

P5-17 original particle binding test, 11 November 2026: a headless test
confirms that an original Sprinkles field binding updates its live asset, dirty
state and respawn event while the full workspace is inactive. Fmt and strict
workspace/all-target Clippy pass. Inline layout, focus and save routing remain
unfinished. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 selected effect identity, 11 November 2026: match the selected zone
particle to the open Sprinkles project by its existing validated local library
path, and report that match in preview status. Seven focused preview tests, fmt
and strict workspace/all-target Clippy pass. Project switching and the inline
inspector remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 scoped live particle editing, 11 November 2026: original Sprinkles
inspector edits restart only systems using the edited asset, with per-emitter
seeds; structural edit events rebuild matching zone previews. Forty-nine particle
editor tests and six focused zone-preview tests, fmt and strict workspace/all-
target Clippy pass. No native display or mouse was used; visual qualification and
the actual inline inspector remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 reusable Sprinkles inspector schedule, 11 November 2026: the original
inspector and widgets can update independently of the full particle workspace;
inline mode is not yet wired into the zone UI. Forty-seven particle-editor and
139 zone-editor tests pass, plus focused real-checkbox and workspace-transition
tests; fmt and strict workspace/all-target Clippy pass. Native interactive GPU
qualification was interrupted when the user requested no display or mouse. See
[integration](SPRINKLES_INTEGRATION.md).

P2-13 particle texture loading foundation, 11 November 2026: a client-side
helper requests verified, package-relative PNG variants through Bevy's standard
image loader with explicit sRGB/linear settings. A real-file Bevy test confirms
format and handle reuse; fmt and strict workspace/all-target Clippy pass. World
loading and Sprinkles playback do not yet use this helper. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 external particle texture drafts, 11 November 2026: export preflight
rewrites PNG references in cloned effect snapshots to bounded hash-derived
package-relative sRGB/linear variants. Eighteen particle-focused editor tests,
fmt and strict workspace/all-target Clippy pass; the snapshot regression was
repeated after its final assertions. Drafts are discarded: client path
resolution, role-aware loading, image decoding and export remain pending. See
[integration](SPRINKLES_INTEGRATION.md).

P2-13 particle texture descriptors, 11 November 2026: the preliminary
catalog can bind bounded PNG files by hash and sRGB/linear role. Three focused
catalog tests and sixteen package/shared-image regressions pass, with one
private staged-world test ignored; fmt and strict workspace/all-target Clippy
pass. Effect-reference matching, image decoding, role-aware loading and client
playback remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle mesh preflight, 11 November 2026: reject non-finite mesh
dimensions in local export preflight and preset snapshots. Six focused editor
tests, fmt and strict workspace/all-target Clippy pass. This does not validate
all effect parameters or establish an allocation budget. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 preset-only particle snapshots, 11 November 2026: draft catalogs now
hash deterministic RON snapshots with editor asset-folder metadata removed.
External textures and custom shaders cannot enter this snapshot; their package
handling remains pending. Five focused editor tests, fmt and strict workspace/
all-target Clippy pass. No particle package is published or played by clients
yet. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle manifest identity, 11 November 2026: schema 4 binds the
particle catalog length and hash into a new world identity. Headless collision
loading recognizes the identity; both client visual verification paths reject
it until particle playback exists. Nineteen package/shared-image/material tests,
fmt and strict workspace/all-target Clippy pass; one private staged-world test
was ignored. No exporter publishes schema 4 yet. See [contract](CONTENT_PACKAGES.md).

P2-13 particle placement allocation estimates, 11 November 2026: export
preflight aggregates checked counts across placed effect instances and reports
invalid arithmetic. Four focused editor tests, fmt and strict workspace/all-
target Clippy pass. Mesh sharing and additional renderer/texture costs mean these
are not a memory budget or capacity claim. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle allocation counts, 11 November 2026: the Sprinkles runtime
exposes checked per-emitter mesh, particle-slot and trail-history counts for
future admission. All 18 particle-runtime tests, fmt and strict workspace/all-
target Clippy pass. A 3440×1440 RX 7900 XTX / RADV / Mesa 26.2.6 Vulkan run
created the local preview's emitters with Khronos validation and zero GPU
diagnostics. Counts are not a complete memory budget or capacity claim. See
[integration](SPRINKLES_INTEGRATION.md).

P2-13 draft particle catalog, 11 November 2026: export preflight hashes
unique local effects and validates a deterministic in-memory catalog using
checked global-to-local placement conversion. Three focused editor tests, fmt
and strict workspace/all-target Clippy pass. The draft is discarded; world
packaging, textures and client playback remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle catalog foundation, 11 November 2026: a separate headless
decoder validates bounded local placements and effect descriptors, and checks
effect file lengths and hashes. Two focused tests, fmt and strict workspace/
all-target Clippy pass. This is not yet a world-manifest format or client
feature; asset parsing, texture dependencies, resource budgets, export and
playback remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 local particle export preflight, 11 November 2026: before refusing
particle-bearing world exports, inspect bounded local effect files and all five
standard texture slots, reporting missing assets or unsupported custom shaders.
Texture existence alone does not establish portability or image validity. All
134 zone-editor tests, fmt and strict workspace/all-target Clippy pass; the
preflight tests were repeated after improving diagnostics. Packaging and client
playback remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 local particle ID preview, 11 November 2026: a selected placement can
load its saved local Effect ID independently of the open project, with reload,
missing-file and malformed-file handling. The short-window action menu stays
within bounds. All 132 zone-editor tests, fmt and strict workspace/all-target
Clippy pass. A 3440×1440 native RX 7900 XTX / RADV / Mesa 26.2.6 Vulkan run
created the saved asset's emitters with a different project open and zero GPU
diagnostics; its captured frame was between bursts. Packaging, client playback,
physical menu activation and Windows qualification remain open. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 bind local particle effect ID, 11 November 2026: the selected reference
menu explicitly saves the open project to its local effect ID and relays save
results to the zone status bar. 131 zone-editor tests, fmt and strict workspace/
all-target Clippy pass. A fresh-data native RX 7900 XTX / RADV / Mesa 26.2.6
Vulkan run created the intended library file, preserved its bundled source, and
reported zero GPU diagnostics. Library loading, packaging and client playback
remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 local particle library save foundation, 11 November 2026: an explicit
save event writes the open project under a validated content ID using the existing
asynchronous atomic save flow. New bindings cannot overwrite another project;
the project can update its own library file. Forty-six particle-editor tests, fmt
and strict workspace/all-target Clippy pass. Inspector wiring, ID resolution,
portable texture packaging and client playback remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 first-run particle demo, 11 November 2026: load the bundled explosion
from the configured examples directory rather than the process working directory,
and cache its actual path. Forty-four particle-editor tests, fmt and strict
workspace/all-target Clippy pass. A fresh-data 3440×1440 native RX 7900 XTX /
RADV / Mesa 26.2.6 Vulkan run renders the demo at a zone anchor with Khronos
validation and zero unexpected GPU diagnostics. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 temporary particle placement preview, 11 November 2026: the inspector
can preview the currently open Sprinkles project at a selected zone anchor,
without linking its effect ID or changing the zone. The preview uses the original
asset and shared one-shot loop logic, follows transforms, and suspends for the
particle workspace and playtest. Forty-three particle-editor and 130 zone-editor
tests, fmt and strict workspace/all-target Clippy pass; final preview regressions
were repeated. Native 3440×1440 RX 7900 XTX / RADV / Mesa 26.2.6 Vulkan
captures visibly render rain and a looping burst with Khronos validation and no
unexpected GPU diagnostics. Persistent ID resolution, packaging, client playback
and MMO budgets remain open. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle viewport anchors, 11 November 2026: unresolved references use
a selectable editor-only badge at their checked local position, update selection
colour and disappear in playtest. 127 zone-editor tests, fmt and strict
workspace/all-target Clippy pass. A 3440×1440 RX 7900 XTX / RADV / Mesa 26.2.6
Vulkan capture with Khronos validation reports zero GPU diagnostics. Native
mouse picking, effect resolution, packaging and client playback remain pending
qualification. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle reference controls, 11 November 2026: Add creates an unresolved
reference at spawn; the left explorer filters/selects it and the inspector edits
its content ID, name and exact transforms inline. Shared duplicate, ground, frame,
delete and undo work. 126 zone-editor tests, fmt and strict workspace/all-target
Clippy pass. Native 3440×1440 RX 7900 XTX / RADV / Mesa 26.2.6 Vulkan capture
with Khronos validation reports zero GPU diagnostics. A viewport anchor, effect
resolution, packaging and client playback remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle shared authoring commands, 11 November 2026: particle
placements support the existing rename, translate, duplicate and ground
transactions, including checked IDs and undo/redo. Fifty-nine zone tests, fmt
and strict workspace/all-target Clippy pass. Editor controls, packaging and
client rendering remain pending. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle placement data, 11 November 2026: zone schema 8 stores bounded
renderer-independent effect references, checked f64 global anchors and
transactional edits with undo/redo. The existing exporter rejects placements
until packaging is implemented, preventing silent loss. Fifty-eight zone tests,
123 zone-editor tests, fmt and strict workspace/all-target Clippy pass. Editor
placement controls, actual effect packaging and client rendering remain pending.
See [integration](SPRINKLES_INTEGRATION.md).

P5-17 particle example switching, 11 November 2026: unchanged bundled
examples no longer trigger the discard prompt even though Save still requires a
new path; edited examples remain protected. Forty-three particle-editor tests,
fmt and strict workspace/all-target Clippy pass headlessly. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 particle New dialog routing, 11 November 2026: Create responds only to
its own dialog and cannot start a form over an existing modal. Forty-two particle
editor tests, fmt and strict workspace/all-target Clippy pass headlessly. See
[integration](SPRINKLES_INTEGRATION.md).

P5-17 particle project switching, 11 November 2026: Open and New commit a
valid focused field and request confirmation before replacing unsaved work;
cancel and stale-project checks preserve the current asset. An active save blocks
switching, and dialog actions are scoped to their operation. Forty-one particle
editor and 122 zone-editor tests, fmt and strict workspace/all-target Clippy pass.
Offscreen 512×384 confirmation layout and particle runtime checks on RX 7900 XTX /
RADV / Mesa 26.2.6 Vulkan with Khronos validation report zero unexpected GPU
diagnostics. No physical display or mouse; visual assessment and Windows remain
unqualified. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle prism and cuboid arithmetic, 11 November 2026: runtime mesh
admission checks prism geometry and cuboid replication for integer overflow.
Sixteen runtime and 14 loader tests, fmt and strict workspace/all-target Clippy
pass; windowless RX 7900 XTX / RADV / Mesa 26.2.6 Vulkan with Khronos
validation reports zero unexpected GPU diagnostics through rejection and recovery.
Checks are not memory limits or general input validation. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle trail mesh arithmetic, 11 November 2026: tube and ribbon
geometry and replication reject integer overflow before construction. Fifteen
runtime and 14 loader tests, fmt and strict workspace/all-target Clippy pass;
windowless RX 7900 XTX / RADV / Mesa 26.2.6 Vulkan with Khronos validation
reports zero unexpected GPU diagnostics through rejection and recovery. Prism,
cuboid and memory budgets remain open. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle cylinder arithmetic, 11 November 2026: checked side and
optional cap geometry and replicated counts reject overflow before construction.
Fourteen runtime and 14 loader tests, fmt and strict workspace/all-target Clippy
pass; windowless RX 7900 XTX / RADV / Mesa 26.2.6 Vulkan with Khronos
validation reports zero unexpected GPU diagnostics through rejection and recovery.
Other mesh types and memory budgets remain open. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle sphere arithmetic, 11 November 2026: checked Bevy UV-sphere
counts and particle replication reject integer overflow before construction.
Thirteen runtime and 14 loader tests, fmt and strict workspace/all-target Clippy
pass; windowless RX 7900 XTX / RADV / Mesa 26.2.6 Vulkan with Khronos
validation reports zero unexpected GPU diagnostics through rejection and recovery.
Other mesh types and memory budgets remain open. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle quad arithmetic, 11 November 2026: overflowing subdivision,
index-count and replicated-vertex calculations are rejected before mesh creation;
invalid live edits suppress rendering and recover after repair. Twelve runtime and
14 loader tests, fmt and strict workspace/all-target Clippy pass. Windowless RX
7900 XTX / RADV / Mesa 26.2.6 Vulkan with Khronos validation reports zero
unexpected GPU diagnostics. Other mesh types and memory budgets remain open. See
[integration](SPRINKLES_INTEGRATION.md).

P2-13 particle sphere geometry, 11 November 2026: zero-radius spheres retain
finite normals and zero resolution uses at least three segments and two rings.
Eleven runtime, 14 loader and 36 particle-editor tests, fmt and strict workspace/
all-target Clippy pass. Windowless RX 7900 XTX / RADV / Mesa 26.2.6 Vulkan
with Khronos validation reports zero unexpected GPU diagnostics. Arbitrary mesh
parameters and VFX memory budgets remain open. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 particle mesh normals, 11 November 2026: flat or very thin cylinders
and degenerate prisms retain finite normals; subdivided-prism index integrity is
regressed. Ten particle-runtime, 14 loader and 36 particle-editor tests, fmt and
strict workspace/all-target Clippy pass. Windowless RX 7900 XTX / RADV / Mesa
26.2.6 Vulkan with Khronos validation reports zero unexpected GPU diagnostics.
Arbitrary mesh-parameter validation and memory budgets remain open. See
[integration](SPRINKLES_INTEGRATION.md).

P2-13 particle allocation arithmetic, 11 November 2026: checked slot,
trail-history and platform-sized sub-emitter counts reject arithmetic overflow
before allocation or render extraction. Invalid live emitters are suppressed and
recover after repair. Eight runtime, 14 loader, 36 particle-editor and 122 zone
editor tests, fmt and strict workspace/all-target Clippy pass. A windowless RX
7900 XTX / RADV / Mesa 26.2.6 Vulkan run with Khronos validation reports zero
unexpected GPU diagnostics. This is not a memory cap, mesh validation or MMO VFX
budget. See [integration](SPRINKLES_INTEGRATION.md).

P2-13 / P5-17 runtime material admission, 11 November 2026: unsupported
custom shaders are rejected by the Bevy loader and cannot panic a programmatic
system; live emitters stop rendering and recover after valid material/buffer
updates. Actual loader and multi-emitter regressions, 6 runtime, 14 loader,
36 particle-editor and 122 zone-editor tests, fmt and strict workspace/all-target
Clippy pass. Windowless RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan with
Khronos validation reports no unexpected GPU diagnostics; no display or mouse
was used. Windows, visual parity, zone placement and gameplay VFX remain pending.
See [integration](SPRINKLES_INTEGRATION.md).

P5-17 particle exit recovery, 11 November 2026: when a project switch and
normal exit happen in the same frame, preserve both committed project snapshots.
A failed outgoing write is retried if storage recovers before exit. Headless
schedule and failure regressions, particle-editor tests, fmt and strict
workspace/all-target Clippy pass. Exit flushing remains bounded; uncommitted text,
forced termination, storage failure and Windows remain unqualified. See
[particle integration](SPRINKLES_INTEGRATION.md).

P5-17 particle draft recovery, 11 November 2026: separate periodic and
transition snapshots, a bounded exit flush and guarded restore/discard are in
place. Explicit Save still owns project-file writes. 34 particle-editor tests,
122 zone-editor tests, fmt and strict workspace/all-target Clippy pass. Input-free
far-origin llvmpipe Vulkan startup has zero GPU diagnostics. The recovery dialog
has not been visually or interactively qualified on native hardware; Windows is
unverified. Recovery remains best-effort for uncommitted fields, abrupt process
termination and storage failures. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 particle transition drafts, 11 November 2026: leaving the particle
workspace or switching projects queues the last committed snapshot, including
edits since the periodic capture; headless transition, particle-editor, fmt and
strict workspace/all-target Clippy checks pass. Shutdown and user-facing draft
disposal remain pending; queued work is not durable until its write completes.
See [integration](SPRINKLES_INTEGRATION.md).

P5-17 periodic particle drafts, 11 November 2026: committed changes in an
active particle workspace are captured to separate draft files every five seconds
using the bounded background writer. Explicit Save still writes the project.
33 particle-editor tests, 122 zone-editor tests, fmt and strict workspace/all-target
Clippy pass; input-free far-origin llvmpipe Vulkan startup has zero GPU diagnostics.
Quick switches, shutdown and visible draft removal remain pending; native dialog
interaction and Windows are not qualified. See [integration](SPRINKLES_INTEGRATION.md).

P5-17 particle recovery list, 11 November 2026: the Sprinkles project menu
lists separately stored drafts with background loading and guarded restore as a
dirty, pathless project. Storage and restore tests, 30 particle-editor tests,
122 zone-editor tests, fmt and strict workspace/all-target Clippy pass. A far
origin input-free llvmpipe Vulkan startup capture has zero GPU diagnostics; it
does not exercise the recovery dialog. Native interaction remains unqualified.
Automatic snapshots and visible discard remain pending, so crash recovery is not
complete. See [particle integration](SPRINKLES_INTEGRATION.md).

P5-17 particle Save integrity, 11 November 2026: explicit Save commits a valid
focused field, serializes writes and keeps the project dirty until a successful
replacement; intervening edits, failed writes, migration and existing new-project
targets are protected. Headless save regressions, fmt, strict workspace/all-target
Clippy and the runtime/editor suites pass. An input-free far-origin virtual-X
startup capture on llvmpipe Vulkan reports zero GPU diagnostics; native interaction
and Windows remain unqualified. Separate recoverable drafts remain pending. See
[particle integration](SPRINKLES_INTEGRATION.md).

P5-17 property action popover, 11 November 2026: a Sprinkles more icon opens
one unclipped native popover over shared selection actions; guide point controls
appear only for guides. Guard document/revision/selection/point, including the
explorer row menu, and commit a focused field before opening. All 122 editor
tests, fmt and strict workspace/all-target Clippy pass. Input-free virtual-X
captures of the far spawn and open popup pass on llvmpipe Vulkan with zero GPU
diagnostics. Actual RX 7900 XTX/native input checks remain pending while the user
reserves mouse and keyboard. See [record](EDITOR_UI_REFINEMENT.md).

P5-17 inline player spawn, 11 November 2026: explorer selection and Zone → Edit
spawn show permanent exact X/Y/Z fields in the left inspector; shared validation,
undo and explicit Save apply. All 114 editor tests, fmt and strict workspace/
all-target Clippy pass. Isolated virtual-X startup renders on llvmpipe Vulkan pass
at local/far origins with zero GPU diagnostics and no OS input. RX 7900 XTX could
not present to virtual X without DRI3; selected-spawn native/hardware interaction
checks remain pending while the user reserves mouse and keyboard. Action-popup
refinement remains. See [record](EDITOR_UI_REFINEMENT.md).

P5-17 permanent content names, 11 November 2026: Name joins numeric attributes
in the left inspector. F8/menu Rename focuses it; Enter/blur/Tab use one validated
shared command, Escape restores source and explicit Save commits valid focused
text first. Keep exact f64 values/history. All 113 editor tests plus the added
name-to-numeric Tab check, fmt and strict workspace/all-target Clippy pass;
native local/far naming/numeric/category checks pass on RX 7900 XTX / RADV
NAVI31 / Vulkan, zero GPU diagnostics. Spawn/action-popup refinement remains.
See [record](EDITOR_UI_REFINEMENT.md).

P5-17 selected-content viewport toolbar, 11 November 2026: shared Move/Rotate/
Scale/Frame/Ground actions use Sprinkles icons beside the left dock. Guard the
whole strip and retain left/middle/right gesture ownership through release,
including when the pointer leaves it. All 112 editor tests, fmt and strict
workspace/all-target Clippy pass; native local/far tool and camera/source/history
isolation checks pass on RX 7900 XTX / RADV NAVI31 / Vulkan with zero diagnostics.
Inline name/spawn workflows and popup refinement remain. See [record](EDITOR_UI_REFINEMENT.md).

P5-17 property categories, 11 November 2026: the stacked left inspector now
uses Sprinkles icon tabs for Attributes, Tools and View. Hide inactive controls
from layout/focus, retain manual tab choice across edits, preserve explorer scroll
and open/focus existing attributes with F9. All 111 editor tests, fmt and strict
workspace/all-target Clippy pass; native local/far checks pass category switching,
pending numeric edits and exact source/history preservation on RX 7900 XTX /
RADV NAVI31 / Vulkan, zero GPU diagnostics. Toolbar, inline name/spawn and popup
refinement remain follow-on work. See [record](EDITOR_UI_REFINEMENT.md).

P5-17 permanent numeric attributes, 10 November 2026: selected objects/plants/
markers/guides use always-visible textboxes with one aligned frame/background.
No click-to-reveal or Apply panel; Enter/blur applies one validated exact edit,
Escape cancels, Tab advances and explicit Save commits valid focused text first.
Keep f64/history and reject stale/replaced-document drafts. All 110 zone-editor
tests, fmt and strict workspace/all-target Clippy pass; local/far native checks
pass on RX 7900 XTX / RADV NAVI31 / Vulkan with zero diagnostics. Further P5-17
refinement remains active. See [record](EDITOR_UI_REFINEMENT.md).

P5-17 dock/status checkpoint, 10 November 2026: implement the stacked left
explorer/properties dock, independent folding and width/split resizing, type
filter dropdown and compact full-width status bar. Remove repeated explorer/help
text and the companion launcher. All 107 zone-editor tests, fmt and strict
workspace/all-target Clippy pass; native local/far checks pass with zero GPU
diagnostics on RX 7900 XTX / RADV NAVI31 / Vulkan. Inline edits, category tabs,
viewport toolbar and popup refinement remain active work, not completed behavior.
See [refinement record](EDITOR_UI_REFINEMENT.md).

P5-17 current execution, 10 November 2026: refine the integrated editor again
under the user's direct feedback. Target inline auto-applied values (explicit
Save writes disk), explorer/attributes stacked with a horizontal divider and
independent collapsible/resizable panels, dropdown filtering, meaningful property
sections, Sprinkles-style popovers and correctly aligned input text. Remove the
separate companion launch option. These are current requirements, not completed
behavior; earlier close-enough acceptance remains historical. Preserve exact
f64/validation/history. See [adoption](DECISIONS.md).

P5-17 upstream Sprinkles audit, 10 November 2026: reviewed all 28 issues
against the imported source. Fix obsolete curve/gradient/mesh cache retention,
apply authored bounds to every emitter and move the preview culling toggle onto
meshes. Guard unsupported custom-shader project loads/menu choices to prevent
shared-host panics. Regressions cover reclamation, sharing, bounds, rejected-load
preservation and 0.2 save/reload stamping. Validation and remaining priorities (draft recovery,
long idle memory investigation, per-instance
controls and gameplay integration) are recorded in the
[issue review](SPRINKLES_UPSTREAM_REVIEW.md). No full idle-leak/Windows/MMO qualification claim.

P5-17 same-window Sprinkles authoring accepted on Linux, 10 November 2026:
File → Particles workspace opens the actual upstream UI inside Theia; Zone in
its top bar returns. Original editing/playback/save, project/pause retention and
exclusive input/camera/render-layer ownership pass native local/far checks with
exact zone/source/revision/history preservation and zero Vulkan diagnostics on
RX 7900 XTX / RADV NAVI31. All 177 focused tests, three doctests, fmt and strict
workspace/all-target Clippy pass. The original standalone host remains qualified.
The direct editor-integration slice is complete at this scope. Asset placement/
world export/client VFX, full P2-13, Windows and MMO budgets remain separate.
See [SPRINKLES_INTEGRATION](SPRINKLES_INTEGRATION.md) for use and exact evidence.

P5-17 same-window preparation, 10 November 2026: all original Sprinkles Update
systems share an activity gate, defaulting active for the qualified companion.
An actual save-shortcut regression proves inactive Sprinkles cannot save its
project on Ctrl S. Four editor tests, fmt and strict workspace Clippy pass, as
does the full native widget-edit/save flow with zero Vulkan diagnostics. UI,
focus, camera/layer isolation and Theia input suspension remain to be connected;
this does not yet claim a same-window workspace. See SPRINKLES_INTEGRATION.

P5-17 original Sprinkles widgets, 10 November 2026: native curve/gradient/colour
popovers render and commit actual values through the reused UI, preserving exact
expected effect data and the source file until Save. Fix the range handler's
incorrect hierarchy lookup; add finite/inverted-range rejection and a regression
for one-time commits and clamping. Native GPU validation remains clean on
RX 7900 XTX / RADV NAVI31. All 16 focused runtime/editor tests and three doctests pass, as do formatting
and strict workspace Clippy; see SPRINKLES_INTEGRATION for evidence.
Same-window hosting remains next; no gameplay VFX/Windows/MMO capacity claim.

P5-17 direct Sprinkles source integration, 10 November 2026: the original
runtime and editor are imported at the reviewed pin and ported to exact Bevy
0.20.0. File → Particle editor opens the retained UI/preview/RON authoring tool
in one companion window. Native editing/save and host launch/return checks pass
on RX 7900 XTX / RADV NAVI31 / Vulkan with zero diagnostics and exact zone/source/
history preservation. All 172 focused tests plus three runtime doctests, fmt and
denied-warning workspace/all-target Clippy pass. Source and asset licenses/hashes
are retained. This completes the first direct-reuse slice; same-window hosting,
asset placement/client integration, Windows and full P2-13 remain pending.
See [SPRINKLES_INTEGRATION.md](SPRINKLES_INTEGRATION.md) for exact evidence.

P5-17 Sprinkles-style UI migration accepted on Linux, 10 November 2026:
the user accepts a close, usable match without pixel-perfect comparison. Native
local/far runs pass menus/selectors/context actions, direct attribute entry,
exact-edit/undo preservation, panel/section behavior, resize, dialog and attribute
focus scrolling, real wheel/thumb input in short windows and playtest return.
Both runs restore exact source content and preserve the source file, with zero
Vulkan diagnostics on AMD Radeon RX 7900 XTX (RADV NAVI31). Native sizes include
3440x1440, 3440x600 and 3440x420. The qualified binary SHA-256 is
8bbfefa2104ec5f0167abc964febf61c8fed24e6480518a81bd4085d6279e550.

Evidence: `build/zone-sprinkles-close-enough-20261110-v2/qualification.json` and
`build/zone-sprinkles-close-enough-far-20261110-v2/qualification.json`. Harness
fixes keep wheel batches within the desktop helper's bound and wait for applied
focus-reveal geometry after resize. Earlier failed runs were harness failures,
not acceptance. All 156 focused Rust tests, formatting, denied-warning workspace/
all-target Clippy and the editor build previously pass for this production code.
Windows cross/native execution remains unverified; no multiplayer claim is made.

The UI migration slice is complete at this Linux acceptance scope. Begin the
next Sprinkles integration slice by porting reviewed runtime/editor source to
Bevy 0.20. The compatibility probe is under ignored build/sprinkles-port; no
particle runtime or authoring workspace has yet been added to the Theia editor.
Full authoritative spell VFX/audio P2-13 remains separate. Desktop use is renewed;
keep working locally without pushing.

Consistent resize affordances, 10 November 2026 (P5-17): divider
highlight/cursor feedback now shares the drag controller's window/menu/modal/
playtest/preview availability checks. A hovered scrollbar suppresses idle resize
feedback as well as new resize acquisition. Active permitted resizing keeps
feedback while crossing a scrollbar. The cursor uses Sprinkles' native column-
resize icon. Width limits, retained drag/cancel handling and source/history stay.

All 156 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. A CPU regression covers cursor/highlight removal
and blocked acquisition while menus/scrollbars own input, restoration when
available, focus-loss cleanup and source/width/history preservation. Evidence:
`build/sprinkles-review/resize-feedback-tests.log`, `resize-feedback-clippy.log`
and `resize-feedback-build.log`. No OS input/window/renderer was used; native
visual/input/GPU acceptance remains pending while desktop use is stopped.

Native Bevy slider-event headless checks, 10 November 2026 (P5-17):
a new CPU font/layout case sends actual PointerPress/PointerRelease events
through Bevy's native slider and pointer-focus observers. At local/far origins,
yaw, plant scale, marker radius and point/whole-guide width reach both authored
range ends, acquire focus and clear Pressed on release. Only the selected draft
field changes; all other original strings, source document, revision, dirty
state and pending redo stay unchanged. No production code changed.

All 155 focused Rust tests, formatting and denied-warning workspace/all-target
Clippy pass. Evidence: `build/sprinkles-review/native-track-focused.log`,
`native-track-tests.log` and `native-track-clippy.log`. This is Linux CPU event/
layout evidence without an OS input device, Winit window, renderer or GPU.
Native visual/input/GPU acceptance remains pending while desktop use is stopped.

Naming/property native qualifier preparation, 10 November 2026 (P5-17):
`tools/qualify_zone_names.py` now checks exact fields against the current
inspector rectangle and chooses the visible inspector Transform action rather
than a hidden menu item with the same command. It records layout geometry,
filters native client environment and verifies a stable binary hash. Python
syntax and --help checks pass. No native run, window, capture or desktop input
was performed; production code and native acceptance are unchanged.

Grid/pan native qualifier preparation, 10 November 2026 (P5-17):
`tools/qualify_zone_grids.py` now uses current canvas height for projected pan
expectations, actual browser/inspector rectangles for gesture-ownership checks,
and View/Zone menus for camera and spawn commands. It records layout geometry,
uses the shared client environment filter and guards the binary hash. Python
syntax and --help checks pass. No native run, window, capture or desktop input
was performed. Existing recorded GPU evidence remains limited to its binaries;
this script preparation adds no visual/input/GPU acceptance or production change.

Content-browser native qualifier preparation, 10 November 2026 (P5-17):
`tools/qualify_zone_content.py` now reads current browser/inspector rectangles
instead of the retired right-hand coordinates, routes wheel input into that
browser and opens Edit spawn through the Zone menu. It records layout geometry,
uses the shared client environment filter and verifies the binary hash remains
unchanged through a run. Python syntax and --help checks pass. No native run,
window, screenshot or desktop input was performed; this is prepared coverage,
not new visual/input/GPU acceptance. Production code did not change.

Aligned tool-setting values, 10 November 2026 (P5-17): tool settings
now present 10 px medium muted names beside right-aligned 12 px tabular values
and units, above the retained native sliders. Exact-form slider labels share
the same field typography. Existing setting captions still supply status text,
units and precision; no range, keyboard mapping, source edit or transaction changed.

All 154 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence: `build/sprinkles-review/setting-labels-tests.log`,
`setting-labels-clippy.log` and `setting-labels-build.log`. CPU layout includes all
selected-inspector widths/heights and slider/exact-draft checks. Native visual/
input/GPU acceptance remains pending while desktop use is stopped.

Shared Sprinkles-style native sliders, 10 November 2026 (P5-17):
tool settings and exact-attribute forms now share a thin 4 px Zinc track and
compact rounded blue thumb. Hover, keyboard focus and disabled track states use
the adopted palette. The full 24 px control area, 14 px horizontal thumb width,
travel bounds, native Bevy input, step/range/precision and validated draft/source
controllers are retained. This is a Theia-specific matching component, not an
imported upstream slider; the reviewed upstream widgets have no slider.

All 154 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence: `build/sprinkles-review/shared-sliders-tests.log`,
`shared-sliders-clippy.log` and `shared-sliders-build.log`. CPU checks include
slider keyboard behavior, exact draft/source preservation and the complete
selected-inspector layout matrix. Native pointer/visual/GPU acceptance remains
pending while desktop use is stopped.

Sprinkles-style tool shortcut columns, 10 November 2026 (P5-17):
Tools rows now separate action names from right-aligned 10 px muted keyboard
hints, using the same secondary typography as native menus. Each workspace keeps
its existing action bindings, visibility/active states and live brush-value
labels. The retained action captions supply shortcuts; no keyboard mapping or
source transaction changed.

All 154 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence: `build/sprinkles-review/tool-captions-tests.log`,
`tool-captions-clippy.log` and `tool-captions-build.log`. The complete selected-
inspector CPU layout matrix passes. Native visual/input/GPU acceptance remains
pending while desktop use is stopped.

Sprinkles browser controls and file rows, 10 November 2026 (P5-17):
content search/filter/paging actions now use shared 28 px MD button geometry and
12 px labels. File rows pair original pinned Sprinkles/Remix 16 px folder/file
icons with bare filenames and shared Ghost/Active states. Bounded scan/paging,
selection, filename validation and explicit overwrite confirmation are retained.
Menu keyboard hints keep muted 10 px secondary typography through state updates.
Icon hashes, upstream Git blobs and licensing are recorded under third_party.

All 154 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence: `build/sprinkles-review/browser-polish-tests.log`,
`browser-polish-clippy.log` and `browser-polish-build.log`. Native visual/input/GPU acceptance remains pending;
no windows or desktop input were used while the user needs the display.

Complete inspector layout checks, 10 November 2026 (P5-17): a new CPU
font/layout case assembles the production inspector, theme, selectors, menus,
scrollbars, summaries, footer and exact-form controllers with real object/plant/
marker/point-guide/whole-guide selections at a distant origin. All 20 combinations
of selection, 300/600 px inspector widths and 420/1440 px window heights reveal
each visible inspector button/slider/info block through native ScrollIntoView.
Bounds stay within the panel/window and the complete document, revision, dirty
state and pending redo remain unchanged. No production behavior changed here.

All 154 focused Rust tests, formatting and denied-warning workspace/all-target
Clippy pass. Evidence: `build/sprinkles-review/inspector-matrix-tests-final.log`
and `inspector-matrix-clippy.log`. Native visual/input/GPU acceptance remains
pending while desktop use is stopped. Windows cross/native execution is unverified:
only x86_64-unknown-linux-gnu is installed and no MinGW cross compiler is present.
This is headless Linux CPU layout evidence, without a renderer or OS input.

Direct attribute-value editing, 10 November 2026 (P5-17): selecting an
inspector value now opens the retained exact editor at that field. Ghost value
buttons use the shared 28 px interaction style and a short edit hint. Native
ActivateOnPress opens the draft before release, preserving text-input ownership.
The entry point binds to rendered target/type/revision/guide-point identity and
refuses stale/hidden/unfocused/busy values. Normal F9/menu transform keeps its
first-field entry. All values stage the existing full validated transaction;
Apply/Cancel, no-op and undo/redo semantics remain.

All 153 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Tests enter every field on all five selected-target
fixtures at local/far origins, verify exact focus/draft strings and real Cancel
redo preservation, reject stale/hidden input, and exercise held press/release
reflow plus repeated reopening without source edits. Evidence:
`build/sprinkles-review/attribute-actions-tests-final.log`,
`attribute-actions-clippy.log` and `attribute-actions-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped.
The prepared native qualifier now clicks all seven object values and checks the
focused field, exact draft and cancellation; it has only been syntax-checked.

Sprinkles-style aligned attribute summaries, 10 November 2026 (P5-17):
selected objects, plants, markers and guide points/widths now show aligned label/
value rows in the default inspector. Shared capture supplies the same original
f64 values as the exact editor: 10 px medium muted labels, 12 px tabular values,
20 px minimum rows and bounded label/value columns. Full global coordinates are
shown without formatting them through local f32 positions or three-decimal
rounding. Names/kinds/IDs, marker definition/annotation and guide-point metadata
stay in the selection description. Object drags show the candidate pose with an
explicit preview label; source data remains untouched. Summaries hide during
offline playtest. Attribute edits retain the existing Apply/Cancel form.

All 150 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. New CPU layout checks cover all six exact-editor
fixtures at local/far origins, correct visible fields/labels, readable bounded
columns, exact draft/summary consistency, preview/source transitions, playtest
hiding and complete source/revision/history/dirty preservation. Evidence:
`build/sprinkles-review/attribute-summary-tests-final-v3.log`,
`attribute-summary-clippy-final.log` and `attribute-summary-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped.
The prepared qualifier reads actual rendered-value Text components via telemetry
and compares all seven object values with source; it is syntax-checked only.

Sprinkles panel heading/summary cleanup, 10 November 2026 (P5-17):
browser and property headings now use compact 12 px semibold display text and
title case, matching the shared panel-section typography. Inspector summaries
lead with selected content/tool information; document name/save state stay in
the document selector, window title and footer. Offline-playtest controls,
placement refusal reasons, global-position summaries and tool metadata remain.
The exact-attribute form uses the same property heading style.

All 148 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence:
`build/sprinkles-review/panel-headings-tests-final.log`,
`panel-headings-clippy.log` and `panel-headings-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped;
no native window, OS input or image inspection was used for this slice.

Sprinkles document selector, 10 November 2026 (P5-17): the top-left
header now shows the current zone name, upstream unsaved '* ' prefix and chevron
through the shared native dropdown. Its ghost trigger follows pinned
project_selector.rs and is bounded to 320 px; a clipped label wrapper protects
long Unicode names and the arrow. The menu reuses New/Open/Save as/Reload and
adds visual access to the retained zone-rename form. Explicit Rename zone targets
the zone even while an object is selected; ordinary selection rename and Shift F8
remain. File actions retain their existing guarded/undoable behavior.

All 148 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Tests cover local/far zone rename with a selected
object, exact document equality, dirty marker / rename undo, undoable New and
restored unsaved content, and long Unicode header geometry with actual theme.
New/Undo retains the existing file-binding behavior: a restored document remains
unsaved after New cleared its binding. Evidence:
`build/sprinkles-review/document-selector-tests-final.log`,
`document-selector-clippy.log` and `document-selector-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped.
The prepared native qualifier now checks document-menu rename/cancel and marker
transitions around apply/undo; it has only been syntax-checked for this binary.

Sprinkles viewport dropdowns, 10 November 2026 (P5-17): Camera and Gizmo
mode now show current values in full-width combobox-style fields instead of
button clusters. The pinned combobox.rs MD trigger presentation supplies 28 px
height, 12 px text/padding, current-value label, right chevron and active-open
feedback; shared field labels use 10 px medium text and 3 px spacing. Native
Bevy menus reuse the existing four camera/three gizmo choices and guarded handlers.
Popup placement flips above near the window bottom and escapes panel clipping.
Gizmo mode reflects its existing Objects-only availability; busy/unfocused popups
close while preserving a newly opened text form's focus.

All 146 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Tests cover all choices at local/far origins,
current-value labels, exact source/history/revision preservation, disabled/focus/
modal guards, clipped short-window placement, and actual theme/menu/selector
geometry across modes and supported panel widths. Evidence:
`build/sprinkles-review/select-tests-final.log`, `select-clippy.log` and
`select-build.log`. Native rendering/input/GPU acceptance remains pending while
desktop use is stopped. The prepared native qualifier now covers all seven
choices and gizmo availability; it has only been syntax-checked for this binary.

Sprinkles content-row action menus, 10 November 2026 (P5-17): objects,
guides, markers and plants now expose a compact ellipsis menu and secondary-click
access following pinned data_panel.rs. Actions reuse the native application-menu
items and guarded Rename/attributes/Duplicate/Ground/Frame/Delete commands.
Opening selects the row through the existing validated selection path, then binds
the context to its exact target, source revision and row mapping. Stale selection,
revision, paging or filtering cannot redirect an action to a different target.
Busy/unfocused contexts close; native focus returns to the trigger on dismissal.

Native Bevy Popover placements keep the popup within the window; OverrideClip
lets it escape the scrolling browser. Open menus own wheel/panel-edge gestures,
and focus reveal treats popup items independently of the underlying scroll pane.
Menu-item text state now respects its own active/disabled state. Row labels use
exact 12 px text and reserve space for the 24 px trigger. The row pool stays bounded.

All 143 focused Rust tests pass, including native menu event dispatch, local/far
complete-document duplicate oracles, replay/stale guards, exact undo, short-window
popup bounds/unclipping and raw panel-edge ownership. Formatting, denied-warning
workspace/all-target Clippy and editor build pass. Evidence:
`build/sprinkles-review/context-tests-final-v3.log`, `context-clippy-final-v2.log`
and `context-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped.
The prepared Sprinkles qualifier now covers ellipsis/secondary-click, Frame and
rename cancellation using explicit menu-item telemetry; it is syntax-checked only.

Sprinkles 52 px top bar and viewport section, 10 November 2026 (P5-17):
the editor now uses pinned topbar.rs height, 12 px inset, panel background and
bottom border. Menus/brand sit left; Playtest and primary-blue Save sit right.
The old second header row is removed. Camera presets, grid, snapping, handles
and gizmo mode move into their own independently collapsible Viewport inspector
section. Grid/Handles share a wrapping row. Existing View-menu camera access and
all guarded commands/preferences remain. Browser, rail, inspector and embedded
viewport start at the shared 52 px boundary, recovering 40 px of working height.

All 140 focused tests pass. The actual-theme CPU layout case now checks all ten
relocated viewport controls inside the inspector, both top-bar actions inside
52 px, rail/browser/tool geometry across modes/widths, and exact source/history
preservation. Source pick/window/image conversions use the shared header boundary.
Formatting, denied-warning workspace/all-target Clippy and editor build pass.
Evidence: `build/sprinkles-review/topbar-tests-final.log`, `topbar-clippy.log`
and `topbar-build.log`. Native rendering/input/GPU acceptance remains pending while desktop
use is stopped. The modern Sprinkles qualifier now uses measured inspector top
bounds and checks independent Viewport collapse; it has only been syntax-checked.

Sprinkles shared action buttons, 10 November 2026 (P5-17): workspace,
exact-attribute, naming and file actions share upstream MD sizing (28 px height,
12 px horizontal padding, 4 px radius and 12 px text). Full-width tool actions
retain automatic height for wrapping and left alignment; file rows remain left
aligned. Live variant borders now match upstream button.rs: default outline,
borderless primary/destructive/active/ghost/disabled states and corresponding
border/foreground colors. Bounded content rows, rail and section header sizing
remain specific to those controls. Native Bevy activation and command guards stay.

All 140 focused tests, formatting, denied-warning workspace/all-target Clippy and
editor build pass. The CPU font/layout test now includes actual theme/checkbox/
border systems across all modes and supported browser/inspector widths, while
checking exact source/history preservation. Evidence:
`build/sprinkles-review/button-style-tests-final.log`,
`button-style-clippy-final.log` and `button-style-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped;
this slice used no native window, display capture or OS input.

Sprinkles inspector sections, 10 November 2026 (P5-17): selection actions
now have their own independently collapsible Selection section beside the retained
Tools and Tool settings sections. Shared headers use the pinned panel_section.rs
12 px semibold display text, 12 px vertical padding/body spacing, compact 20 px
header and bottom divider. Inspector status stays visible outside the sections,
including during offline playtest. Existing commands and collapse/focus guards
remain unchanged; section state is a session UI preference, not a source edit.

All 140 focused tests and denied-warning workspace/all-target Clippy pass.
Evidence: `build/sprinkles-review/inspector-sections-tests.log` and
`inspector-sections-clippy.log`; editor build also passes
(`inspector-sections-build.log`). Layout checks retain supported panel widths,
all modes, placement settings and playtest return. Native rendering/input/GPU
acceptance remains pending while desktop use is stopped. The prepared native
qualifier now checks independent Selection/Tools collapse as well as Tool settings;
it has only been syntax-checked, not run for this binary.

Sprinkles dialogs, 10 November 2026 (P5-17): naming/search and zone-file
forms now share upstream dialog.rs framing: 6 px radius, 1 px border, 80% black
backdrop, separated 24 px header/body/footer padding, exact 18 px semibold title,
14 px regular status description and right-aligned 12 px action labels. Naming
uses the upstream 448 px maximum width; the file browser retains 900 px for paths.
Dialogs cap their height, scroll only the body and keep Cancel/Apply fixed.
Native focus reveal handles viewport/pane size changes as well as focus changes,
while unchanged bounds/focus preserve manual scrolling. Find has its own primary
button label; file Open/Save use primary blue and overwrite uses upstream red.
Existing draft validation, cancellation, guarded commands and file APIs remain.
Checkbox labels also now request exactly 12 px without legacy size remapping.

Headless checks include 140 focused tests; the short-window case opens at full
height then resizes to 420 px with unchanged focus, checks filename and actions,
manual scrolling, naming/search and exact source/history/bytes/file-count safety
at local/far origins. Formatting, denied-warning workspace/all-target Clippy and
editor build pass. Evidence: `build/sprinkles-review/dialog-tests-final-v3.log`,
`dialog-clippy-final-v2.log` and `dialog-build-final.log`. Native rendering/input/
GPU acceptance remains pending while desktop use is stopped. --dialogs native
qualification is prepared and syntax-checked; it has not been executed.

Sprinkles checkboxes, 10 November 2026 (P5-17): Snap, Handles and Grid
adopt the pinned checkbox.rs 16 px outlined box, 2 px radius, 6 px label gap,
12 px text and hover border feedback. A native UI-border check mark replaces
textual [x]/[ ] prefixes. Keyboard focus adds a blue border and disabled state
uses muted colors. The existing native Bevy Checkbox, guarded ValueChange
handler and Session/View state synchronization remain in use.

All 139 focused tests and denied-warning workspace/all-target Clippy pass.
Evidence: `build/sprinkles-review/checkbox-tests-final.log` and
`checkbox-clippy-final.log`. Native visual/input/GPU acceptance remains pending
while desktop use is stopped; this is presentation work checked headlessly.

Sprinkles browser scrolling, 10 November 2026 (P5-17): the content browser
shares the 24 px wheel-step, hover-only thin scrollbar and bounded scroll pane
used by the inspector. Short windows scroll within the current bounded page;
full-height pages retain existing wheel paging. One wheel gesture never both
scrolls and pages. Explicit page/filter changes reset the pane; selection changes
use Bevy ScrollIntoView to reveal the selected row after layout without changing
keyboard focus. Manual scrolling stays put until selection/focus changes.

All 139 focused Rust tests and denied-warning workspace/all-target Clippy pass.
CPU layout/event tests cover short/local/far selection, full-height page routing,
modal/focus/gesture guards, filter reset and exact source/revision/history/dirty
preservation. Evidence: `build/sprinkles-review/browser-scroll-tests-final.log`
and `browser-scroll-clippy.log`; editor build passes (`browser-scroll-build.log`).
Native rendering/input/GPU
acceptance remains pending while desktop use is stopped. The existing --scroll
qualifier now prepares browser wheel/thumb and width/source guards as well as
inspector checks; it has only been syntax-checked, not run for this binary.

Sprinkles numeric field frames, 10 November 2026 (P5-17): exact attributes
now use the upstream 28 px transparent frame, 1 px border, 2 px radius, compact
drag prefixes, suffixes and tabular digits. Object size is grouped as an X/Y/Z
row with 12 px spacing. Position remains three full-width world-f64 fields so
large coordinates stay readable; scalar yaw/scale/radius/width retain explicit
labels/units. Parser/error labels, values, command validation and Apply/Cancel
remain unchanged. Focus colors belong to the shared frame.

All 137 focused Rust tests and denied-warning workspace/all-target Clippy pass.
CPU layout checks include grouped size, sufficient full-precision position width,
short-window focus reveal and all retained exact edit/undo/cancel checks. Native
qualification now reads real drag-label rectangles rather than hardcoded label
geometry. Native rendering/input/GPU acceptance remains pending: desktop use is
stopped and no native window/input/capture inspection was used for this slice.
Evidence: `build/sprinkles-review/shared-inputs-tests.log`,
`shared-inputs-clippy-final.log` and `shared-inputs-build.log`. Editor build passes.
Rename, search and file-path inputs share the same compact transparent input
frame, Inter/tabular font and source cursor/selection colors.

Headless validation passes: 137 focused Rust tests, formatting,
denied-warning workspace/all-target Clippy and editor build. Evidence:
`build/sprinkles-review/attributes-scroll-tests-final-v2.log`,
`attributes-scroll-clippy-final.log` and `attributes-scroll-build-final.log`.
Native visual/input/GPU acceptance remains pending; desktop use is stopped.

Sprinkles exact-attribute scrolling, 10 November 2026 (P5-17): the exact
attribute editor now reuses the inspector scroll pane with neutral panel framing.
Pane ownership distinguishes an active attribute draft from ordinary workspace
controls, so its wheel/scrollbar stays usable while underlying authoring remains
guarded. Newly focused controls scroll into view; manual scrolling is retained
until focus changes. Focus pending before visible layout is handled explicitly.

Headless InputPlugin/CPU-layout checks cover all six forms at local/far origins
in a 420 px-high layout: every field and slider is visible when Tab focuses it;
wheel input inside the draft is bounded and does not fight focus; exact strings,
source, revision and redo stay unchanged; Escape restores the prior saved state.
Native qualification now has a prepared `--attributes-scroll` path using actual
reported focus/entity rectangles. No desktop access was used for this slice.
Native rendering/input/GPU acceptance remains pending while the user needs the
mouse and display. Earlier native evidence belongs to its recorded binaries.

Sprinkles inspector scrolling and button reflow, 10 November 2026 (P5-17):
implements a scrolling inspector with Sprinkles' wheel units and slim hover/drag
thumb, native Bevy scrollbar behavior, bounds/shrink clamping and exclusive UI
gesture ownership beside panel resize grips. Fixes a reproduced stale Pressed
state when tool activation reflows the inspector before pointer release. Cleanup
never replays commands and preserves held mouse/touch gestures.

All 136 focused tests, formatting, denied-warning workspace/all-target Clippy
and build pass. Linux native local/far runs on AMD Radeon RX 7900 XTX (RADV NAVI31),
Vulkan validation enabled, pass six placement command oracles, exact saves,
keyboard sliders, preview/modal/panel/refusal/playtest isolation and return-to-edit
preferences. The final UI binary separately passes menus, sections, resizing,
numeric dragging and undo, real 24 px wheel input, scrollbar dragging in a
3440 x 600 owned window, unchanged panel widths/documents/history, and restored
3440 x 1440 playtest entry/return. Zero GPU diagnostics in both runs.

Evidence: `build/zone-rearm-probe-20261110-v2/`,
`build/zone-sprinkles-placement-native-20261110-v3/`,
`build/zone-sprinkles-scroll-native-20261110-v2/` and
`build/sprinkles-review/scrolling-*.log`. Native evidence records exact binary
hashes. No Windows or multiplayer claim. Desktop use remains authorized; work
continues across local commits. Next: reuse scrolling for exact attributes,
including keyboard focus reveal and neutral inspector framing.

Desktop use stopped again at the user's request, 10 November 2026.
No editor or desktop-input process remains. Continue headlessly. The completed
sidebar/menu/resize/numeric native run remains valid. A separate placement run
passed local box/sphere/cylinder dimensions, keyboard sliders, preview/placement
command oracles, undo/redo and exact save, then timed out while re-arming a tool
before playtest. This does not qualify playtest or the far placement case.
The qualifier now waits for the intended tool's actual hover before pressing,
as it already did for other controls; this harness change needs a native rerun.
No application fix is inferred from the timeout. Local log GPU diagnostics: zero.
Evidence: `build/zone-sprinkles-placement-native-20261110-v1/` and
`build/sprinkles-review/sidebar-placement-native-v1.log`.

Sprinkles-style workspace sidebar, 10 November 2026 (P5-17): replaces
the top workspace row with a 72 px vertical rail using Sprinkles' icon plates,
labels, spacing and active/hover feedback. Six unmodified Remix icons are embedded
through Bevy's standard asset system with license and exact hashes retained.
Existing guarded workspace commands remain authoritative for mode changes.
Browser, inspector, canvas projection/input, wheel routing, grips and telemetry
include the rail offset; the future native qualifier reads that offset.

All 134 focused Rust tests pass, including actual Bevy CPU layout across all six
workspaces and three panel-width pairs, viewport/ray conversion across DPI scales,
resize cancellation and browser wheel exclusion over the rail. Formatting,
denied-warning workspace/all-target Clippy and build pass. With desktop use
renewed, Linux native 3440 x 1440 checks pass at local/far fractional origins on
AMD Radeon RX 7900 XTX (RADV NAVI31), Vulkan with validation enabled: six workspace
rail buttons, resize/cancel, sections, mouse/keyboard menus, file browser,
stationary exact attributes, numeric drag/Cancel/Apply/undo and New/undo preserve
full documents. Zero GPU diagnostics. Visual review found unsupported font
chevrons; standard Bevy UI border geometry now renders them consistently.
Evidence: `build/sprinkles-review/sidebar-*.log` and
`build/zone-sprinkles-native-20261110-v3/qualification.json`.
Placement/preview/playtest requalification remains incomplete; desktop use is
stopped again. See STATUS for partial local placement evidence. No Windows claim.

Sprinkles-style numeric attribute dragging, 10 November 2026 (P5-17):
visible ↔ labels scrub their exact field values while ordinary text input remains
available. Normal drag steps by 0.1 per 5 logical pixels; Alt/Super uses 1 per 10.
The adapter preserves the starting f64 and exact original string at zero steps,
uses a symmetric dead zone, and refuses invalid/composing input. It deliberately
omits upstream absolute decimal rounding so distant global coordinates cannot
lose precision. Changes remain pending until the retained Apply transaction;
Escape/focus loss cancels without changing source or redo.

All 134 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and build pass. Headless real field-controller checks cover stationary
precision, local/far object/plant/marker/guide-point/spawn/guide-width drags,
command-oracle equality, one-entry Apply, malformed fields and cancellation.
Native mouse/UI/GPU acceptance remains pending while desktop use is stopped.
Evidence: `build/sprinkles-review/scrub-*.log`.

Standard Bevy embedded viewport, 10 November 2026 (P5-17): adopts the same
ViewportNode mechanism used by Sprinkles. A separate window UI camera presents
the panels and the scene camera renders into the widget's standard image target;
Bevy resizes it to the canvas rectangle. Shared input conversion maps logical
window coordinates into image pixels, and report projection maps back to window
coordinates. Panning uses the actual canvas height. No custom renderer or new
engine/plugin version is introduced.

All 132 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and build pass. Real Bevy camera projection/ray round trips cover all four
views, three panel-width pairs and 1x/1.5x/2x scale factors, including panel input
exclusion. The scene/ghost data-only test initializes the real viewport and its
pointer map. Native rendering, GPU/input, resize and playtest acceptance remain
pending while mouse/display use are stopped. Earlier partial native evidence is
for the pre-ViewportNode binary only. Evidence: `build/sprinkles-review/viewport-*.log`.

Sprinkles-style panel resizing, 10 November 2026 (P5-17): browser and
inspector boundaries expose bounded 12 px grips, hover/drag feedback and standard
resize cursors. Browser width is 300..480 px and inspector width 300..600 px.
Settings remain session-local; cancellation/focus loss restores the original
widths, and gesture ownership shields world/camera/keyboard edits through release.
Shared canvas bounds, wheel routing, exact form placement and footer track the
widths. All 131 focused Rust tests, CPU layouts at narrow/wide/intermediate widths,
formatting, denied-warning workspace/all-target Clippy and build pass.

A partial Linux RX 7900 XTX/Vulkan 3440 x 1440 run exercised resizing, Escape
cancellation, source/history preservation and section collapse/expansion with zero
GPU diagnostics. It stopped safely when focus left the owned window before File
menu input. Full native menu/form/local/far acceptance remains pending; its two
captures have not been visually inspected. Mouse/display use is stopped again.
Evidence: `build/sprinkles-review/panels-*.log` and
`build/zone-sprinkles-native-20261110-v1/`. New native harness:
`tools/qualify_zone_sprinkles.py`. Keep working headlessly; do not push.

Desktop restriction renewed, 10 November 2026: the user needs the mouse and
display and explicitly resumes work headlessly. Do not use mouse/display or
launch native UI checks. The native qualifier stopped on its owned-focus guard
before further input, and its editor process was terminated. Continue local code,
CPU tests and documentation; native checks remain pending until reauthorized.

Desktop authorization renewed, 10 November 2026: the user permits mouse and
display use as needed again. Keyboard use remains authorized. Continue local
P5-17 Sprinkles layout/interaction implementation and native 3440 x 1440 checks;
continue after local commits and do not push. Earlier headless qualification below
retains its stated native limitations until the new binary is actually checked.

Sprinkles-style application menus, 10 November 2026 (P5-17): compact File,
Edit, View and Zone menus replace the long document-command button row. Save and
Playtest remain directly accessible. Native Bevy MenuButton/MenuPopup/MenuItem
provide the menu interaction path; command activation shares existing validation,
undo/history and modal/playtest guards. Headless events exercise open/close/focus,
one-entry undo/redo, disabled/unfocused refusal and far-coordinate preservation.
Raw menu focus blocks authoring navigation, camera movement and browser wheel
paging; closed menu buttons retain global shortcuts. All 130 focused Rust tests
pass. Native visual/input/GPU acceptance remains pending while desktop use is
stopped. Evidence: `build/sprinkles-review/menu-*.log`.

Sprinkles browser/inspector layout, 10 November 2026 (P5-17): replaces the
old opposing/stacked side panels with adjacent full-height left browser and
inspector columns, followed by the world canvas. The inspector contains the
contextual palette in a collapsible Tools group and a separate Tool settings
group. Exact transform editing occupies the same inspector column. Wheel routing,
canvas input exclusion and footer bounds use the new regions. Actual Bevy CPU
layout checks cover all six workspaces, reachable tools, long properties and
local/far exact coordinates; all 129 focused Rust tests pass. Native UI/input/GPU
acceptance is pending while mouse and display use remain stopped. Existing native
evidence applies only to the earlier theme binary, retained under
`build/sprinkles-theme-qualified/`. Menus, richer direct attribute editing and
resizing remain subsequent migration work. Evidence: `build/sprinkles-review/`.

Sprinkles collapsible tool settings, 10 November 2026 (P5-17): a native Bevy
section header now collapses/expands the retained tool settings with Sprinkles'
chevron, spacing, ghost hover and separator styling. Activation moves focus to
the header so hidden sliders cannot own keys. Actual observer checks at local/far
fractional origins preserve exact source, redo, preferences and saved state;
unfocused/playtest activation is refused. Section telemetry supports later native
qualification. CPU layout and all 128 focused Rust tests pass; denied-warning
workspace/all-target Clippy passes. Native UI/input/GPU acceptance is pending:
mouse and display use are stopped. The previously qualified theme binary remains
available until the new presentation passes native checks. Evidence:
`build/sprinkles-review/sections-tests.log` and `sections-clippy.log`.

Desktop restriction, 10 November 2026: the user stopped mouse and display
use. Continue headlessly; do not launch native windows, inspect captures or send
OS desktop input until authorization changes. No qualifier/editor process was
running when the restriction was applied. Existing native evidence below belongs
only to the binaries already checked; subsequent UI changes need new acceptance.

Sprinkles theme foundation, 10 November 2026 (P5-17): shared upstream
zinc/blue tokens, adapted button states and embedded licensed Inter now style the
palette, command bar, workspaces, content list, properties and dialogs. Focused
inputs use 1 px blue borders; inactive inputs use zinc borders. Rename uses a
compact matching dialog. The retained playtest context/settings restoration is
included. All 127 focused Rust tests, formatting and denied-warning workspace/
all-target Clippy pass. Linux RX 7900 XTX / Vulkan at 3440 x 1440 passes local/far
placement, focused keys, preview hiding, undo/redo, exact document command oracles,
Save and playtest/settings restoration, with zero GPU diagnostics. Native captures
were inspected. Native local/far invalid/corrected naming and numeric-form input checks also pass;
those dialog captures have not been visually inspected. This is a styling
foundation; collapsible sections and panel resizing follow. No particle runtime
or Bevy version change. Evidence: `build/sprinkles-review/` and
`build/zone-sprinkles-theme-native-20261110/`; provenance:
`third_party/sprinkles/PROVENANCE.md`. Keep working locally; do not push.

## 10 November 2026 — adopted Sprinkles editor UI direction (P5-17)

The user adopts Sprinkles as the concrete look, feel and interaction reference
for the full Theia editor. Adapt its native Bevy UI components where practical;
create matching controls for Theia-specific functions. Keep the existing validated
command/history, exact f64 fields, source schemas, agent interface and offline
playtest underneath the presentation. Continue local implementation and commits
until stopped; mouse, keyboard and display use are authorized. Do not push.

Reviewed upstream: `doceazedo/sprinkles` commit
`0e978f5a9afc6b456b3693c8761e73a6c7e155bd` (Bevy 0.19). Adapt to Theia's
existing Bevy 0.20 pin; no engine downgrade. MIT-selected UI provenance is in
`third_party/sprinkles/`. Particle runtime remains a candidate requiring its own
P2-13 compatibility, rebasing, budget and native GPU qualification.

Initial UI prerequisites are the retained editor command/history, standard Bevy
widgets and existing 3440 x 1440 layout. First slice: shared Sprinkles tokens,
button states and Inter typography across workspaces, palette, outliner,
properties and dialogs. Follow with resizable panels and section/inspector
interaction. Acceptance: formatting, denied-warning Clippy, retained focused
Rust tests, CPU layout and actual Linux 3440 x 1440 input/GPU checks. Planned
migration is not a claim of completed UI or particle adoption.

Stopped at the user's request, 9 November 2026. Latest local commit is
c2e0e3e; nothing was pushed. No editor, native qualifier or Cargo process remains
running from this work. The next playtest-panel slice is intentionally uncommitted:
Properties shows playtest controls, authoring sliders hide during play and restore
unchanged settings on exit. Changes are in main.rs, workspace.rs, placement_tests.rs
and tools/qualify_zone_placement_size.py. All 127 focused Rust tests, strict
workspace/all-target Clippy and build pass (build/zone-playtest-context-*.log).
CPU layout includes the play panel and settings restoration. Native 3440 x 1440
acceptance of this latest presentation change is still pending; do not attribute
previous native evidence to its new binary. Resume with that native check before
committing this slice. Desktop access remains authorized when work resumes.

Desktop authorization renewed, 9 November 2026: the user permits display and
mouse use as needed again; keyboard use remains authorized. Continue after local
commits and do not push. Native authoring checks target 3440 x 1440 only. Earlier
headless records below retain their original execution/acceptance limitations.

Placement hints and renderer guards, 9 November 2026 (P5-17 / P3-15 / P1-10):
Properties explains refused candidates. The renderer hides cached ghosts during
forms/playtest or when the tool no longer matches, independently of pointer order.
All 127 focused Rust tests, formatting, strict Clippy and build pass; long-name/
refusal layout fits. Native local/far Find/playtest/preview checks pass at
3440 x 1440 with zero Vulkan diagnostics. See [the record](ZONE_EDITOR.md#placement-refusal-hints-and-renderer-guards--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Read-only primitive placement preview, 9 November 2026 (P5-17 / P3-15 / P1-10):
A standard Bevy ghost shows the prospective stamp: blue for valid, red for a
refused candidate. It shares pose/object construction and command validation
with actual placement while preserving source, history, redo and IDs. All 127
focused Rust tests, formatting, strict Clippy and build pass. Native local/far
3440 x 1440 preview/stamp equality, panel hiding and refused-click isolation
pass with zero Vulkan diagnostics. See [the record](ZONE_EDITOR.md#read-only-primitive-placement-preview--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Workspace slider focus cue, 9 November 2026 (P5-17 / P3-15 / P1-10):
A pale thumb identifies the focused slider; the footer explains its keys.
All 124 focused Rust tests, strict Clippy, formatting and build pass. Native
local-origin 3440 x 1440 rendering/keys/placement pass with zero Vulkan diagnostics;
app captures were inspected. See [the record](ZONE_EDITOR.md#workspace-slider-focus-cue--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Workspace slider keyboard ownership, 9 November 2026 (P5-17 / P3-15 / P1-10):
Workspace sliders now take focus on click. Left/Right/Home/End adjust that widget
without also editing the selection through raw keyboard polling; ordinary nudges
resume when focus clears. All 124 focused Rust tests, formatting, strict Clippy
and build pass. Native grid/size keys preserve selected source and redo at local/
far origins, 3440 x 1440, with zero Vulkan diagnostics. See [the record](ZONE_EDITOR.md#workspace-slider-keyboard-ownership--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Primitive placement dimensions, 9 November 2026 (P5-17 / P3-15 / P1-10):
Properties has quarter-step size sliders for new box X/Y/Z, cylinder diameter/
height and sphere diameter. Preferences preserve source/history; stamps use
checked snapping/surface support and one shared edit. All 123 focused Rust tests,
formatting, strict Clippy and build pass. Native local/far shape controls,
placement, full-document command oracles, undo/redo and Save pass at 3440 x 1440
with zero Vulkan diagnostics. See [the record](ZONE_EDITOR.md#primitive-placement-dimensions--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Application shortcuts, 9 November 2026 (P5-17 / P3-15 / P1-10):
Ctrl+N invokes shared New; Ctrl+F opens content search without framing. Modal,
preview, playtest and focus guards remain. All 119 focused Rust tests, strict
Clippy, formatting and build pass; native key delivery and full New/save/play
workflow pass at 3440 x 1440 from local/far origins with zero Vulkan diagnostics.
See [the record](ZONE_EDITOR.md#application-document-and-search-shortcuts--9-november-2026).
Continue after local commits; desktop access authorized; do not push.

Native application controls accepted, 9 November 2026 (P5-17 / P3-15 / P1-10):
Content paging/wheel, filters, search, double-click framing and yaw/scale/radius/
guide-width sliders pass native 3440 x 1440 checks at local/far origins, including
Reset, Home/End, exact source/oracle equality and undo/redo. Vulkan validation is
active with zero diagnostics. The earlier wheel-delivery gap is closed for these
accepted runs after pre-launch synthetic-device creation. See [the record](ZONE_EDITOR.md#native-application-controls-acceptance--9-november-2026).
Continue after local commits; desktop access authorized; no push. Historical
headless records below retain their original execution limitations.

New untitled zones, 9 November 2026 (P5-17 / P3-15 / P1-10):
New zone creates an undoable empty document. Its first Save chooses a filename;
cancel/failed writes preserve untitled state and the prior file is never written.
All 118 focused Rust tests, formatting, strict Clippy and the editor build pass.
Native New/save/playtest checks pass at 3440 x 1440 from local/far origins,
with active Vulkan validation and zero GPU diagnostics.
See [the zone record](ZONE_EDITOR.md#new-untitled-zones--9-november-2026).
Desktop access is authorized again for native 3440 x 1440 checks; continue after
local commits and do not push. Historical headless limits below remain historical.

Reset transform fields, 9 November 2026 (P5-17 / P3-15 / P1-10):
A Reset fields button restores the open form’s exact original numbers and
sliders, retaining the form, original revision and pending redo. Apply/Cancel
remain the transaction boundary. Headless behavior/layout checks pass; native
UI/input/GPU acceptance remains pending. See [the zone record](ZONE_EDITOR.md#reset-transform-fields--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Property keyboard routing, 9 November 2026 (P5-17 / P3-15 / P1-10):
Headless real input/focus/visibility tests cover Tab/Shift-Tab and slider keys
without OS input. They caught and fixed a yaw +180/-180 thumb jump. All 115
focused Rust tests pass; native key delivery/UI/GPU acceptance remains pending.
See [the zone record](ZONE_EDITOR.md#property-keyboard-routing-and-yaw-endpoints--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Bounded property sliders, 9 November 2026 (P5-17 / P3-15 / P1-10):
Plant scale, marker radius and whole/point-guide width now use quarter-step
sliders in the retained exact form. Shared controlled-widget handling preserves
precision, validation and one-entry undo. All 113 focused Rust tests and CPU
layout pass; native UI/input/GPU acceptance remains pending. See [the zone record](ZONE_EDITOR.md#bounded-property-sliders--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Transform yaw slider, 9 November 2026 (P5-17 / P3-15 / P1-10):
Object/plant/marker properties now expose a standard Bevy yaw slider alongside
exact fields. It edits the pending form; Apply remains one shared validated undo
entry. Precision, no-op/redo, cancellation and 3440 x 1440 CPU layout pass;
native UI/input/GPU acceptance remains pending. See [the zone record](ZONE_EDITOR.md#transform-yaw-slider--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content double-click framing, 9 November 2026 (P5-17 / P3-15 / P1-10):
Double-clicking an entry selects and frames it through the existing checked
Frame helpers. Single-click behavior, view preset, source and redo are preserved.
Actual headless Bevy event checks cover every content kind at local/far origins;
native input/GPU acceptance remains pending. See [the zone record](ZONE_EDITOR.md#content-double-click-framing--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content search, 9 November 2026 (P5-17 / P3-15 / P1-10):
Find/Clear search use the retained Bevy text form for bounded name/kind/#ID
matching within the chosen type filter. Search, including a pending form, leaves
source/save state/history unchanged. Headless behavior/layout checks pass;
native UI/input/GPU acceptance remains pending. See [the zone record](ZONE_EDITOR.md#content-search--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content-type filters, 9 November 2026 (P5-17 / P3-15 / P1-10):
All/Objects/Guides/Markers/Plants/Terrain buttons narrow the content list while
preserving source, selection and pending redo. Actual Bevy observer/sync and
CPU font/layout checks pass; native UI/input/GPU acceptance remains pending.
See [the zone record](ZONE_EDITOR.md#content-type-filter-buttons--9-november-2026).
Continue headlessly after local commits; no desktop input/display use; no push.

Content-list paging, native acceptance pending, 9 November 2026
(P5-17 / P3-15 / P1-10): the row pool reserves space above Properties; wheel
paging and disabled boundary buttons have headless Bevy-system coverage.
Standard Bevy CPU font/layout checks also verify all row/footer bounds at 3440 x 1440.
The earlier native wheel check timed out; do not claim GPU/input acceptance.
See [the zone record](ZONE_EDITOR.md#content-list-paging-headless-coverage--9-november-2026).
Continue headlessly after local commits; no mouse, keyboard/display use or push.

Piped structured commands, 9 November 2026 (P5-17 / P3-15 / P1-10):
--commands - reads bounded JSONL from stdin through EOF using the same command
validation/history path as files and the viewport. Headless output is written
only after all commands succeed; read-only playtest preflight accepts the same
stream. See [the zone record](ZONE_EDITOR.md#piped-structured-commands--9-november-2026).
Continue headless after local commits; no desktop input/display use; no push.

Structured entry translation, 9 November 2026 (P5-17 / P3-15 / P1-10):
The shared translate edit offsets primitive/plant/marker anchors or every point
of a whole guide, preserving identity and metadata. Mixed Apply batches retain
whole-candidate validation and one-entry undo. CLI/preflight and mailbox paths
are qualified headlessly; see [the zone record](ZONE_EDITOR.md#structured-entry-translation--9-november-2026).
Continue headless after local commits; no desktop input/display use; no push.

Headless playtest preflight, 9 November 2026 (P5-17 / P3-15 / P1-10):
--check-playtest builds the actual offline collision scene and resolves the
requested capsule spawn, returning JSON/readiness and a nonzero refusal status.
It opens no window, writes no source, and supports in-memory command validation.
See [the zone record](ZONE_EDITOR.md#headless-playtest-preflight--9-november-2026).
Continue headless after local commits; no desktop input/display use; no push.

Placement/move grid presets, 9 November 2026 (P5-17 / P3-15 / P1-10):
Properties now exposes a discrete 0.125/0.25/0.5/1/2/4/8 m step slider. Shared
zone-origin snapping covers placement and primitive move/body gestures; nudges
use the chosen increment. Settings preserve history/redo; rotation and scale
snapping retain their contracts. See [the zone record](ZONE_EDITOR.md#placement-and-move-grid-presets--9-november-2026).
Continue after each local commit; no push.

Pointer viewport panning, 9 November 2026 (P5-17 / P3-15 / P1-10):
Middle-mouse drags now pan in the current view plane with projection/zoom-aware
logical cursor tracking. Pans must begin inside the viewport; panels, Escape,
focus loss, editing gestures, modals and playtest cancel/refuse them. Source and
history stay unchanged. See [the zone record](ZONE_EDITOR.md#pointer-viewport-panning--9-november-2026).
Continue after each local commit; no push.

Exact spawn properties, 9 November 2026 (P5-17 / P3-15 / P1-10):
Edit spawn opens three exact world-coordinate fields in the existing docked
properties form. Apply uses shared SetSpawn validation and one-entry undo;
unchanged values preserve redo. Existing placement actions remain available.
See [the zone record](ZONE_EDITOR.md#exact-spawn-properties--9-november-2026).
Continue after each local commit; no push.

Active document context, 9 November 2026 (P5-17 / P3-15 / P1-10):
The footer now persistently identifies the active save path and saved/unsaved
state; the window title identifies the zone/file and marks dirty content. Older
terrain/vegetation shortcut hints now distinguish Open from Reload. See
[the zone record](ZONE_EDITOR.md#active-document-context--9-november-2026).
Continue after each local commit; no push.

View-plane reference grids, 9 November 2026 (P5-17 / P3-15 / P1-10):
Front/Right now expose adaptive vertical reference grids; zones without terrain
also use an adaptive ground grid. The standard Grid checkbox controls references
and retained terrain surface lines, shows metre spacing, and preserves history.
See [the zone record](ZONE_EDITOR.md#view-plane-reference-grids--9-november-2026)
for acceptance and limitations. Continue after each local commit; no push.

Zone file browser, 9 November 2026 (P5-17 / P3-15 / P1-10):
The command bar now exposes Open and Save As with a standard Bevy folder/file
browser, filename input, paging and explicit overwrite confirmation. Open uses
validated document replacement with undo; Save As changes the active path only
after success and preserves history. Ctrl+O opens, Ctrl+Shift+S saves as and
Ctrl+Shift+O reloads the active file. See [the zone record](ZONE_EDITOR.md#zone-file-browser--9-november-2026)
for acceptance and limitations. Continue after each local commit; no push.

Shared body-drag previews, 9 November 2026 (P5-17 / P3-15 / P1-10):
Selected-body XZ dragging now shares the same validated gesture path as axis,
yaw and scale handles. Relative local deltas preserve original f64 values on a
stationary click, with unchanged Y/yaw/size. Invalid candidates never replace the
last valid preview, live source or undo/redo. See [the zone record](ZONE_EDITOR.md#validated-body-plane-drag--9-november-2026) for acceptance.
Continue after each local commit until stopped; no push.

Canonical authoring views, 9 November 2026 (P5-17 / P3-15 / P1-10):
Standard Bevy orthographic Top/Front/Right views and a Perspective control now
support terrain and greybox authoring. Canonical pan/zoom, stable orthographic
camera distance, projection-aware handle sizing and camera locking during drags
are implemented; playtest retains its perspective camera and restores the editor
view. See [the zone record](ZONE_EDITOR.md#canonical-authoring-views--9-november-2026) for acceptance.
Continue after each local commit until stopped; no push.

Primitive uniform scale handle, 9 November 2026 (P5-17 / P3-15 / P1-10):
Scale uniformly joins Move/Rotate Y in the application toolbar. A standard Bevy
cube/stem scales all dimensions by one factor, with optional 10% snapping, keeping
position/yaw exact and preserving sphere/cylinder constraints. Changed previews and
release share existing whole-candidate Transform validation and one-entry undo.
See [the zone record](ZONE_EDITOR.md#primitive-uniform-scale-handle--9-november-2026) for acceptance.
Continue after local commits until stopped; no push.

Primitive yaw rotation handles, 9 November 2026 (P5-17 / P3-15 / P1-10):
The application toolbar now chooses Move or Rotate Y. A standard Bevy torus
provides a yaw ring, with 15-degree snapping and unsnapped editing; angle unwrapping
supports crossing the ±180-degree seam and full turns. Position and size remain
exact. Both tools share validated previews and one-entry Transform history.
See [the zone record](ZONE_EDITOR.md#primitive-yaw-rotation-ring--9-november-2026) for acceptance.
Continue after local commits until stopped; no push.

Primitive move handles, 9 November 2026 (P5-17 / P3-15 / P1-10):
Selected primitives now expose standard Bevy red X / green Y / blue Z move arrows.
Axis drags preview through shared Transform validation and commit once on release;
Escape, focus loss and release over panels cancel. A Move XYZ checkbox controls
visibility. Exact f64 coordinates on the other axes remain untouched. See [the zone record](ZONE_EDITOR.md#primitive-world-axis-move-handles--9-november-2026)
for constraints and acceptance. Continue after each local commit; no push.

Guide properties and point controls, 9 November 2026 (P5-17 / P3-15 / P1-10):
The docked properties form now exposes whole-guide width or selected-point XYZ
and width, through existing validated PutPath commands. Mouse Previous/Next point,
Whole guide, Insert and Remove controls expose point selection/topology directly.
Undo/redo and pending/modal/playtest isolation remain shared. See [the zone record](ZONE_EDITOR.md#guide-properties-and-point-controls--9-november-2026)
for prerequisites and acceptance. Continue after local commits until stopped; no push.

Plant and marker properties, 9 November 2026 (P5-17 / P3-15 / P1-10):
The docked Transform form now edits plants and NPC/quest authoring markers as
well as primitives. Five relevant fields expose position/yaw and scale or radius;
shared candidate commands preserve names, kinds and marker definition keys.
Apply/Cancel, exact f64 values, modal input isolation and one-entry history remain.
The primitive form still exposes all seven fields. See [the zone record](ZONE_EDITOR.md#plant-and-marker-transform-properties--9-november-2026)
for prerequisites, acceptance and evidence. Continue locally; no push.

Vegetation properties, 9 November 2026 (P5-17 / P3-15 / P1-10):
The application properties area now exposes contextual scatter radius (1..32 m)
and integer plants-per-click (1..64) sliders for tree/shrub scatter tools.
Presets remain editor-local; a ground click uses the existing seeded shared
ScatterVegetation edit, with whole-candidate validation and one-entry undo.
Source schema, gameplay and engine/plugin pins are unchanged. Native local/far
3440x1440 acceptance and exact evidence are in [the zone record](ZONE_EDITOR.md#vegetation-scatter-properties--9-november-2026).
Continue local implementation and commits; no push.

Application-style editor work resumed, 9 November 2026 (P5-17 / P3-15 / P1-10):
The user chooses to continue the in-repository editor and prioritizes Blender-like
application workflows: direct sliders, buttons, toggles and a properties area,
rather than larger controls or additional shortcut-driven features. Standard Bevy
widgets now provide workspace buttons, a command bar, a snap checkbox and
contextual terrain sliders. The right column splits a 24-row outliner from a
properties area; exact transform editing docks there with existing transactional
Apply/Cancel validation. Pending previews and modal/playtest isolation remain
protected. The first shell slice has 77 focused Rust checks and native actual
3440x1440 local/far evidence; see the zone record for exact scope. Keep changes
local and do not push. The earlier safe-stop record below is historical.

Safe stop, 8 November 2026 (P5-17 / P3-15 / P1-10; P2-06/07):
The user requested stopping after the current slice. Terrain-conforming brush
feedback, accepted stroke counts/cap feedback, 2 m spatial drag sampling,
surface-supported cursor spawn placement and a clearer spawn marker are complete.
Unrelated document edits also preserve resident terrain meshes. The current
77 focused Rust cases, formatting, strict Clippy and native 3440x1440 checks pass;
exact contracts/evidence are in the zone record. Work is committed locally; no
push. No further autonomous slices begin until the user resumes work.

Zone authoring workspace, 8 November 2026 (P5-17 / P3-15 / P1-10; P2-06/07):
H and structured `project_to_surface` now ground selected objects, plants,
markers and guide points through shared candidate validation and undo. Primitive
centres account for half-height; foot anchors use the exact terrain support.
Whole guides are one undo entry. The summary also distinguishes saved content
from unsaved visual/agent edits and pending drafts. This grounds anchors rather than prop stacks
or full slope footprints. Tab/Shift-Tab also cycle hidden/distant objects; F
frames objects, plants, markers and the existing guide/point selection without
changing history. Committed guides also insert/remove points through Insert /
Backspace and narrow shared commands; structural changes clear stale point
selection while visual edits explicitly select the new/next point. A standard-Bevy
right-side content browser also selects named entries across modes through a
fixed 40-row pool and paging, without changing document/history. Both side
panels are excluded from world mouse actions; the list hides in playtest. F8 /
Shift-F8 also rename selected content / the zone through standard Bevy TextInput
and narrow shared commands, with modal input isolation and one-entry undo. Names
retain existing UTF-8 byte validation; geometry/definitions are unchanged.
Local/far/reopen name-dialog native checks pass with zero GPU diagnostics.
F9 also edits exact f64 primitive position/yaw/size through standard Bevy fields
and modal tab navigation, applying one existing validated Transform transaction.
All six local/far primitive edits match headless commands exactly; native
save/reopen and zero-GPU-diagnostic checks also pass. Both forms clear stale
error messages on text edits while retaining validation on Apply.
Terrain brush feedback also conforms to the exact surface triangles and temporary
stroke preview, sharing sculpt/paint falloff and locked-border rules. One resident
standard Bevy mesh shows potential sample influence without source/history changes.
Held strokes also show accepted stamp counts and an explicit 128-stamp limit
message, preserving the existing bounded commit/cancel behavior. Fast drags
also interpolate 2 m stamps, breaking continuity across panels/outside terrain.
Both grids pass native sculpt/paint command equivalence, undo/redo and save.
P also places a surface-supported playtest spawn from Sculpt/Paint, respecting
snapping and pending-stroke/modal isolation through the existing SetSpawn edit.
The view-centre button shares checked conversion and surface sampling.
An unlit downward spawn marker also replaces the small sphere and hides during
playtest. Unrelated edits retain the unchanged terrain mesh instead of rebuilding it.
Flatten also samples the surface with I and adjusts its local
level in 1 m / 0.1 m steps, sharing exact visual/headless stamp output.
Ctrl-D and structured `duplicate` commands copy any authoring kind with fresh
shared IDs, bounded names/offsets and one-entry history. Local/far/empty native
copy/framing and positive/negative flatten cases pass. Raise/lower strength also
halves/doubles within 0.05..8 m, with exact visual/headless 0.125 m stamp checks
on both grids. Brackets also adjust independent sculpt/paint blends within
5..100%, retaining 50% / 100% defaults and exact 12.5% command equivalence.
`python3 tools/edit_zone.py --example` now opens/reopens a separate
Wayfarer Hollow draft from an original four-transaction recipe. F also frames
the terrain patch. Its graded walking route and shelter pass actual Avian checks;
local/far 3440x1440 editor runs and two real Renet client pairs pass with zero GPU
diagnostics, including far origin Y=20. Markers remain authoring notes and the
stream remains a visual guide. 77 focused Rust cases, seven launcher cases, formatting
and strict workspace/all-target Clippy pass. The full native 3440x1440 walkthrough also passes
with Vulkan validation active and zero GPU diagnostics; [contract and evidence](ZONE_EDITOR.md#wayfarer-hollow-example-and-terrain-framing--8-november-2026).
Local commits only; no push. Chat composer/bubbles remain deferred.

Zone-authoring focus, 8 November 2026 (P5-17 / P3-15 / P1-10):
The user authorizes ongoing terrain/greybox/zone-editor work, recommended local
implementation decisions, desktop use and local commits; no push. Chat composer
and speech bubbles are deferred. A bounded standard-Bevy primitive editor and shared
validated command/undo foundation now pass eight focused Rust checks, strict
workspace Clippy, headless CLI and four native Linux walkthroughs through actual
3440x1440, with zero GPU diagnostics. The opt-in live agent mailbox now also
passes mixed agent/visual undo/redo and playtest refusal natively at 1280x720,
with 12 focused Rust cases and strict Clippy. Schema-2 terrain brushes now add
shared validated stroke previews/undo, actual Avian surface queries and three-size
native sculpt/save/load/playtest checks: 17 focused Rust cases, strict Clippy and
zero GPU diagnostics. Standard GLB/schema-4 export now also passes 22 focused
Rust cases, Khronos validation, actual encrypted admission and four native
two-client cases (including hillside and far-frame traversal). Default gameplay
content is unchanged. Schema-3 road/stream guides now share visual/agent validation,
undo, save/load and render-only export (27 focused Rust cases). The current user
requires the editor at 3440x1440 only: primary-display fullscreen is now the
default and future editor qualification targets that viewport. Earlier smaller
checks below retain their historical scope. Schema-4 NPC/quest authoring markers
now add shared validated placement/transforms/history and source-only export
preservation (31 focused Rust cases); live catalogue publication remains separate.
Schema-5 tree/shrub proxies now add bounded seeded scatter, shared history and
render-only export (35 focused Rust cases and the full 3440x1440 native editor
walkthrough). General vegetation assets/behavior and broader terrain tools remain
future work. Guide-driven terrain grading now shares visual/agent history and
passes 39 focused Rust cases, including actual Avian graded-surface traversal.
Individual guide-point commands/picking/focus now add two domain checks (41 total);
visual and agent point moves share whole-candidate validation and history.
Schema-6 terrain colour painting adds shared stroke previews/history and standard
Bevy/glTF vertex colours, with normalized weights and unchanged collision
(45 focused Rust cases). The one-command editor launcher now reopens saved
drafts, prepares the local build SDK and defers new-file writes until Save;
46 Rust / four Python cases and actual 3440x1440 save/reopen checks pass.
Schema-7 optional 2 m terrain refinement/coarsening now shares validated history,
sculpt/paint/grading and existing export: 50 focused Rust plus sixteen retained
terrain/procedural/saved-terrain regression cases pass.
Runtime multiplayer remains
Rust/Bevy/Avian/Renet; editor playtest is an offline sandbox. Actual prerequisites,
acceptance checks and implementation evidence are tracked in [the zone record](ZONE_EDITOR.md).
Earlier stop/chat-first queue entries are historical; P3-17/18 stay unspecified.

Safe stop after chat client-worker slice, 8 November 2026 (P4-12 / P2-14 / P5-13):
The user requested stopping after the current slice. Finished the opt-in real
Renet worker's bounded chat producer/receipt view, reply pacing and local timeout
without gameplay disconnect. 747 full network cases pass (10 explicit skips),
with18 final focused chat checks,231 ordinary client cases and strict Clippy/
formatting. Actual baseline/Classic encrypted worker fixtures pass. Normal chat
activation, composer and speech bubbles remain pending; no new UI work begins.
The action bar is separately native-qualified at three sizes plus SQL rejoin.
Local commits only; no push. Desktop permission remains available as needed on
future authorized work; it does not override this stop.
[Scope and resume boundary](LOCAL_CHAT.md#real-client-worker-integration--8-november-2026).

Chat inbox prerequisite, 8 November 2026 (P4-12 / P2-14 / P5-13):
A64-message client inbox and checked world delivery serial bound history/replay
state across actor churn. Fourteen focused tests pass, including actual encrypted
frames through the inbox and10,000 actor IDs; strict Clippy/formatting pass.
The chat candidate profile is v3/CD02; default worlds/protocol13 remain unchanged.
Normal client worker/UI and bubbles remain next. Desktop access is allowed as
needed; continue local implementation/commits, no push.
[Scope and evidence](LOCAL_CHAT.md#bounded-client-inbox-and-world-delivery-serial--8-november-2026).

Chat replies prerequisite, 8 November 2026 (P4-12 / P2-14 / P5-13):
The opt-in server candidate has private bounded submission replies and reserves
half the reliable queue against chat use. Ten focused tests pass, including
actual encrypted replies/refusals/forged uploads and queue-pressure recovery;
strict Clippy/formatting pass. Default protocol13 and normal chat activation are
unchanged. Receipt/history/composer/bubbles remain pending. Native action-bar
editing is separately qualified at three sizes plus SQL rejoin.
Continue implementation/local commits; mouse, keyboard and display may be used
as needed per latest permission. No push. [Scope and evidence](LOCAL_CHAT.md#private-replies-and-gameplay-queue-reserve--8-november-2026).

Native action-bar qualification, 8 November 2026 (P2-14 / P3-06 / P5-13):
Four Linux starts pass at1280×720,1920×1200,3440×1440 and SQL restart/rejoin.
Actual book-copy, empty/occupied swaps, invalid drop, focus cancellation and
copied '=' binding casts pass;119 positive duplicate cooldown samples match.
Normal quest/combat/progression remains intact with zero RX7900XTX/RADV Vulkan
GPU diagnostics. Session-local bindings reset on login; persistence remains open.
[Evidence and limits](ABILITY_BOOK_ACTION_BAR.md#native-action-bar-qualification--8-november-2026).
Continue implementation/local commits; desktop access is allowed as needed. No push.

Opt-in chat runtime candidate, 8 November 2026 (P4-12 / P2-14 / P5-13):
Authorized named leases and checked server poses now feed bounded Say/Area routing
in the Renet owner loop under an explicit world compatibility profile. Three
actual encrypted clients pass; disabled/malformed/forged uploads fail closed.
738 network regression cases (ten explicit skips),231 client cases and strict
Clippy pass, with three final focused runtime cases. Normal chat UI/launcher
activation, submission replies and bubbles remain pending.
**Latest permission: mouse, keyboard and display may be used as needed. Continue
implementation and local commits; no push.** Earlier desktop bans are historical.
[Evidence and limits](LOCAL_CHAT.md#opt-in-encrypted-owner-loop-candidate--8-november-2026).

Local-chat routing prerequisite, 8 November 2026 (P4-12):
The host-side router bounds eight admissions and sixteen queued messages per
recipient; current trusted name/pose/area context determines delivery. Three
focused codec/routing/backpressure tests and strict Clippy/formatting pass.
This library is not installed in the owner loop; encrypted chat, composer and
bubbles remain pending. [Evidence and next acceptance](LOCAL_CHAT.md#bounded-host-router-prerequisite).
Continue headlessly; local commits only, no push or desktop access.

Local-chat prerequisite, 8 November 2026 (P4-12 / P2-14 / P5-13):
Bounded Unicode text, server-context local/area delivery policy, replay/mute/flood
gate and exact CQ01/CD01 codecs pass 378 core and two focused wire cases plus
strict Clippy/formatting. This standalone foundation does not enable runtime
chat, composer or speech bubbles; protocol13 remains unchanged. Continue
headlessly with local commits only, no mouse/keyboard/display or push.
[Policy, actual prerequisites and acceptance](LOCAL_CHAT.md).

Action-bar candidate, 8 November 2026 (P2-14 / P3-06 / P5-13):
Session-local ability-ID bindings, book-copy/bar-swap drag handlers and bound
cooldown/tooltips are implemented headlessly. 231 client tests and strict
Clippy pass, including encrypted loopback cast/damage/cooldown refusal and
synthetic widget/drag/focus checks. New rendering and actual pointer interaction
remain unqualified: no mouse, keyboard or display access per current instruction.
Persistent layouts and learned-skill policy remain pending. Local commits only;
no push. [Evidence and limits](ABILITY_BOOK_ACTION_BAR.md#action-bar-binding-candidate--8-november-2026).

Ability book checkpoint, 8 November 2026 (P2-14 / P3-06 / P5-13):
The opt-in P-key/Interface ability book shows configured icons and metric details
with bounded paging. 225 ordinary client tests, strict Clippy, formatting and
four native Linux starts pass; no casts or learning requests are sent. Dragging
and action-bar rearrangement remain pending. [Evidence and scope](ABILITY_BOOK_ACTION_BAR.md).
**Latest instruction: continue implementation and local commits only, with no
mouse, keyboard or display access. No push.** Earlier permissions are historical.

Optimized identity checkpoint, 8 November 2026:
Preserved `build/local-play-release-20261108-v1`, runtime `2930fa4`, paired
protocol13 client/server/validator and copied private qualified art. Build hashes
and six final native starts are recorded in [the identity record](PLAYER_DISPLAY_IDENTITY.md#integrated-qualification--8-november-2026).
Next bounded UI work is the ability book/action-bar workflow; current configured
development abilities are not a persistent learned-skill/class-unlock system.

Implementation resumed, 8 November 2026 (P2-03 / P3-01 / P2-14 / P5-13):
The user restored ongoing implementation, local commits and desktop access.
SQL-selected player names now cross the authorized lease and reliable Renet view;
client admission/snapshot validation gates the owner frame. Protocol13 is the
qualified path; frozen protocol12/v5 remains preserved. Actual OIDC/SQL lease
checks, 731 network cases (ten explicit skips), 224 client cases including local
art, formatting and strict Clippy pass. Six final native starts cover three
resolutions and optimized rejoin with zero RX7900XTX/RADV Vulkan diagnostics.
Long-name wrapping, committed levels and disconnect clearing pass. Continue
bounded UI work and local commits only; no push.
[Scope and evidence](PLAYER_DISPLAY_IDENTITY.md). Earlier stop entries are historical; no push.

Safe stop, 7 November 2026 (P2-03 / P3-01 / P2-14):
Stopped at the user's request after saving the standalone bounded player-name
codec prerequisite. Its six focused tests, formatting and strict workspace/
all-target Clippy pass. Runtime publishing, receiving and name UI remain pending;
gameplay protocol12 is unchanged. The qualified optimized v5 playtest pack is
preserved. Changes are committed locally only; no push. Earlier continuation
instructions below are historical. [Scope and acceptance](PLAYER_DISPLAY_IDENTITY.md).

Player-identity prerequisite, 7 November 2026 (P2-03 / P3-01 / P2-14):
The existing player-name validator is now a pure core value re-exported by
persistence. 385 core/persistence cases pass (73 service skips); the actual
PostgreSQL name/quota/ownership/selection contract additionally passes. Formatting
and strict workspace/all-target Clippy pass. No gameplay name metadata or protocol
change is implemented yet. [Scope and next acceptance](PLAYER_DISPLAY_IDENTITY.md).
The qualified v5 UI pack remains available. Continue locally; no push.

General UI checkpoint, 7 November 2026 (P2-02/05/11/14 / P3-06 / P5-13/19):
Saved Interface size and failed-save retry, configured metric ability tooltips,
bounded combat history, focused fresh input and live committed XP cues are
implemented. Current-source reader/loot/backpack/32-offer/reward flows pass at
1280×720, 1920×1200 and 3440×1440 with PostgreSQL restart/fresh login. The optimized
1920×1200 replay/rejoin also passes. Twenty-eight accepted native starts across
these consecutive slices have zero RX 7900 XTX / RADV Vulkan GPU diagnostics.
The preserved optimized `build/local-play-release-20261107-v5` contains client
runtime source `b412e74`, copied qualified local model art and binary hashes.
222 full client cases, 53 launcher checks, ten core ability cases, formatting,
strict Clippy and builds are the recorded source prerequisites; later focused
native/XP checks retain their exact scope. [Evidence and limits](UI_FANTASY_WARRIOR.md#optimized-playtest-checkpoint--7-november-2026).
Windows, non-unit monitor DPI, capacity/crowd/FPS qualification and complete theme
adoption remain open; the skin stays opt-in. A visible Interface Exit control now
passes native three-size shutdown and durable rejoin checks, and optimized
1280×720 saved-size, real-timeout and Exit checks. It is included in frozen v5.
The owner-level heading passes gain/rejoin/disconnect in debug 1280×720 and
optimized 3440×1440 sessions and is included in v5. Server-owned gameplay name
replication is the next bounded follow-up; it remains unimplemented.
**Continue implementation and local commits only; no push. Mouse, keyboard and
display access are authorized by the latest user instruction.**


Connected general UI checks, 7 November 2026 (P2-05/11/14 / P3-06 / P5-13):
Desktop access was restored by the user. Saved Interface size, ability tooltip,
resize fitting, combat/quest flow and PostgreSQL restart/fresh login pass at
1920×1200 and 3440×1440, with three native starts and zero GPU diagnostics.
This qualifies the copied `9957e7d` debug client in a controlled journal fixture;
optimized contrast and background-input checks are in progress. Windows and DPI
remain pending. [Evidence and scope](UI_FANTASY_WARRIOR.md#connected-ui-qualification--7-november-2026).
Continue implementation and local commits only; no push.

Headless general UI continuation, 7 November 2026 (P2-05/11/14 / P3-06 / P5-13):
Implemented optional saved interface size, configured metric ability tooltips,
a six-message received combat log and focused-window guards for ability, world-
click, backpack, corpse-loot, reader and Interface input. The actual Interface buttons/painter/I/O save-and-fit flow passes
headlessly. 218 client cases, 53 launcher checks, ten core ability cases,
formatting, strict workspace/all-target Clippy and builds pass. Nine actual
client configuration preflights pass with unchanged gameplay identity and no
App/network/renderer startup.
An optimized Linux candidate is frozen at `build/local-play-release-20261107-v1`
(source `d9df562`), including copied qualified local model art and binary hashes.
That frozen pack predates later panel-focus guards; its source is retained.
Its nine real CLI preflights also pass; the previous pack remains unchanged.
This is a prepared build, not native UI/playability or FPS evidence. Native
expanded tooltip/combat-log layout, input/focus/DPI/resize and Windows checks
remain pending. The skin stays opt-in.
**Current permission, updated 7 November 2026: continue implementation and commit
locally only; no push. The user restored mouse, keyboard and display access.**
Connected native UI qualification is in progress; the headless checkpoint above
did not launch a game or services. Earlier restrictions below are historical.

Headless development resumed, 7 November 2026 (P2-14):
The user has resumed implementation with local commits only and explicitly
forbidden mouse, keyboard and display use. This supersedes earlier desktop
permission/stop entries. Optional `--ui-preferences FILE` now loads a bounded,
versioned local scale preference. In-game changes save off-thread; explicit
`--ui-scale` overrides the saved value on launch. Five persistence/error cases
pass in the 198-case client suite; 53 launcher checks and strict all-target
workspace Clippy pass. UI/native visual checks remain deferred. No game launched.

Safe stop checkpoint, 6 November 2026 (P2-14 / P5-13 / P5-19):
Stopped at the user's request after completing floating XP feedback. One reusable
standard Bevy label presents positive committed owner XP deltas, coalesces nearby
gains and fades after 1.5 seconds. Login/rejoin progress is a baseline, not an
award. Invalid/replayed snapshots cannot create duplicate feedback. No gameplay
or protocol changes. 193 client cases, formatting, strict workspace/all-target
Clippy and client build pass. Six windowless Linux GPU fixture layouts through
3840×2160 pass; live connected XP presentation remains unverified. No desktop
input or new play session. Local commit only; no push. No play session running.

Longer local playtest sessions, 6 November 2026 (P2-02 / P3-01 / P3-03):
An explicit bounded server lifetime can now extend an intentionally configured
local session beyond five minutes, capped by signed identity expiry. The default
remains 300 seconds; renewal is separate. 40 auth/server Rust cases, 14 focused
Python checks and actual Keycloak/PostgreSQL HTTP login/logout pass; strict
workspace/all-target Clippy and builds pass. No desktop input was sent.
[Policy and exact evidence](AUTHENTICATED_ADMISSION.md).
Local commits only, no push. The user's earlier frozen play session has closed.

General UI reference candidate, 6 November 2026 (P2-14 / P5-13 / P5-19):
The optional Fantasy Warrior skin now follows the user's AntonilzUI reference:
compact lower-centre player/target frames, twelve 44-pixel primary slots,
cast feedback, a centred XP strip, circular minimap and shared pane headers.
It uses standard Bevy UI with nine local PNG icons and two fonts; source assets
remain outside Git. Preferred UI scale fits/restores the 1280×720 logical layout.
193 client cases, formatting, strict workspace/all-target Clippy and build pass.
Windowless cases through 3840×2160 check combat/XP/dock separation and
projected anchors; earlier circle-corner pixel checks remain valid. Earlier 51 launcher cases remain valid.
Connected OIDC/Renet/PostgreSQL quest/minimap walkthroughs now pass at
1280×720, 1920×1200 and 3440×1440 plus genuine restart/rejoin. They confirm the
fixed panel layering, bag dock and hidden themed terrain diagnostics in a
controlled fixture. Passive enemy nameplates now pass six GPU layouts and a
1280×720 connected health/death/rejoin check, including separated damage text.
Left-side parchment dialogue and full-height reward rows now also pass the
32-offer/note/item-reward walkthrough at all three sizes plus restart/rejoin.
Themed target/status/quest-marker hints are now contextual and compact, with
an additional 1280×720 connected health/death/rejoin check.
Backpack slots now show item icons, confirmed stack counts and real hover
tooltips. The inventory/document/reward flow passes all three native sizes plus
restart; final tooltip placement has six GPU cases and a 1280×720 native replay.
Read-only UI coordinate exports now preserve UI enlargement while removing
window DPI; headless geometry/scroll checks pass. Native enlarged-UI input
now passes the 1920×1200 / 125% document/reward/32-offer flow and restart.
Backpack action colours now match the dark red theme. Six windowless cases
include the five shared inventory actions; the 125% native panel flow now passes.
Fixed quest offers now include decorative icons/counts from their bound reward
metadata. Headless binding/legacy checks and six narrow-cell GPU layouts pass;
normal offer-list interaction with the taller rows now passes at 125% UI scale.
The reader now uses a compact left parchment panel with top page controls;
six windowless cases verify text, controls and clear lower combat frames;
the current reader also passes the 125% native panel/rejoin flow.
Minimap shading follows validated active goals, with explicit objective-to-area
metadata for kill/collection and related goals. 385 core/persistence and 27 focused
network cases pass; the prior six overlay GPU layouts remain raster evidence.
Existing content is unchanged; authored spawn/collection regions and native
replay remain separate. Minimap hover descriptions now have six windowless
GPU layouts and 1280×720 connected contact and reach-area hover/rejoin passes.
Walking into the region clears shading; turn-in/rejoin retains revision 14 /
XP 150. Enlarged panels pass at 1920×1200 / 125%.
Combat frames now spread on wide logical viewports, preserving small layouts;
six GPU layouts and a 3440×1440 / 125% native contact/completion/rejoin pass.
The skin now includes an Interface [F10] panel with session-only size presets,
preferred/fitted scale feedback and top-priority Escape. 189 headless cases and
six windowless settings layouts pass; native pointer/resize checks remain pending.
The skin remains opt-in; clickable plates, portrait art, chat/game-menu systems,
broader scale/DPI/resize traces and complete-theme qualification
remain separate work. [Scope, reference, assets and evidence](UI_FANTASY_WARRIOR.md).
**Historical permission (6 November): desktop/mouse tests were authorized.**
Commit locally only; no push. Earlier permission restrictions below are historical.

Bounded quest offers/wheel ownership qualified, 6 November 2026:
P3-11 / P3-12 / P3-15: the last of 32 guide offers is reachable through a bounded
standard Bevy scroll area. Wheel input belongs to the hovered panel. All three
Linux sizes pass last-offer acceptance, normal note/fixed rewards and SQL restart/
rejoin; the older two-quest dialogue flow also passes at 1280×720 and after restart.
154 client tests, formatting, strict workspace/all-target Clippy and optimized
build pass. [Fixtures and exact scope](QUEST_ENGINE.md#bounded-quest-offers-and-wheel-ownership--6-november-2026).
Catalogue limits and canonical content remain unchanged. Local commits, no push;
desktop use remains authorized while the user is away.

Single-click reader/shared UI recovery qualified, 6 November 2026:
P2-14 / P3-05 / P3-11 / P3-12: the reader's ignored second-open click is fixed
by one shared standard-Bevy button release recovery. Two opens now require two
pointer clicks at all three Linux sizes; completion survives SQL restart/rejoin.
The small-window full-bag refusal/free-space retry remains qualified. 154 client tests,
formatting, strict workspace/all-target Clippy and optimized build pass.
[Measured defect, fixtures and limits](QUEST_DOCUMENTS.md#single-click-reader-and-shared-button-release--6-november-2026).
Local commits only, no push; desktop remains authorized while the user is away.

Fixed quest reward UI qualified locally, 6 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: offers and unfinished journals show fixed item
names/counts; full bags retain Ready/holdings/XP and give clear retry guidance.
Native refusal, free-space retry, atomic whole bundles and SQL restart/rejoin
pass at 1280×720, 1920×1200 and 3440×1440. A discovered stale backpack press after
panel reflow is fixed with a ButtonPlugin event regression. 153 client tests,
formatting, strict workspace/all-target Clippy and optimized builds pass.
[Exact fixtures/evidence/limits](QUEST_ITEM_REWARDS.md#fixed-reward-ui-qualified--6-november-2026).
The user restored desktop/mouse authorization while away. Local commits, no push;
earlier headless-only entries describe historical permission/checkpoints.

Actual client/server reward configuration identities match, 6 November 2026:
The client now has an explicit CLI configuration report that returns before
networking or renderer/window creation. Its content identity matches the real
headless server for both sealed schema-21 bundles, with real OIDC/SQL HTTP ticket
preflight. 150 client tests, four Linux CLI groups (twelve cases), formatting and
strict Clippy pass. [Scope/evidence](QUEST_ITEM_REWARDS.md#actual-clientserver-configuration-identities-agree-headlessly). No game ticket is consumed and no
reward UI/native proof is claimed. Continue headless only; local commits, no push.

Positive normal reward server configuration qualified, 6 November 2026:
Both schema-21 bundles start the actual headless server with sealed 25-role
content, real OIDC/PostgreSQL and HTTP admission preflight. Unauthenticated and
duplicate ticket requests refuse; a fresh authorised character receives a private
ticket. No game client consumes it or creates economy/quest state. Owned services
and snapshots clean up. [Exact scope/evidence](QUEST_ITEM_REWARDS.md#normal-server-startup-and-ticket-preflight-qualified-headlessly). UI/native work remains
pending; continue headless only, local commits and no push.

Fixed reward authoring handoff, 6 November 2026 (P3-11 / P3-15):
The read-only document/quest checker now accepts optional fixed reward references,
with known quest/item, bounded file and area/document namespace checks. Nine Linux
tests plus actual CLI ordering/refusal/unchanged-source checks pass; omitting the
option preserves the earlier report exactly. Formatting and strict Clippy pass.
[Command and limits](QUEST_PACK_AUTHORING.md#optional-fixed-turn-in-reward-references). Normal reward UI/native proof remains pending.
Continue headless only; user is using desktop/mouse. Local commits, no push.

Fixed quest reward normal configuration prerequisite, 6 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: opt-in schema 21 seals 25 exact roles, adding
`quest_item_rewards` to the document bundle. Both binaries accept the bounded
`--quest-item-rewards` catalogue; server World, stored policy and retained owner
bind the same bytes before quest-only loot. 62 validator, 45 bundle, three Linux
CLI tests (eight cases), 24 server and 150 client tests, formatting and strict Clippy pass.
[Scope/evidence](QUEST_ITEM_REWARDS.md#implemented-normal-configuration-prerequisite). Four legacy manifest reports exactly match the preserved
reader validator. Reward/capacity UI and combined native qualification remain
pending; the qualified note-reader binary remains available. No desktop/mouse
tests while the user works. Continue headless and commit locally only; no push.

Fixed quest reward World/encrypted bridge qualified locally, 6 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit reward World installation now checks
exact inventory/XP context, inactive exclusive setup and installed namespaces.
Real encrypted two-peer game commands at 20/30/60 Hz pass under packet impairment:
capacity refusal, held success, deletion after completion, original retry and
private projections. [Scope/evidence](QUEST_ITEM_REWARDS.md#implemented-explicit-world-and-encrypted-bridge). 379 network library, nine wire and
22 server library tests, formatting and strict Clippy pass. Transport commit
publications are synthetic; real SQL/retained ownership is qualified separately.
Normal manifest/CLI and reward UI remain next. Continue headless only while the
user uses the desktop; local commits, no push.

Fixed quest reward retained owner qualified locally, 6 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit exact stored/wire reward context now
connects the retained item/progression owners to atomic turn-in. Real OIDC,
encrypted ticket and PostgreSQL prove nonfatal full-bag refusal, same-intent retry,
COMMIT-before-publication, queued item movement during retirement and rejoin
without another grant. Existing document-owner regression also passes.
[Scope/evidence](QUEST_ITEM_REWARDS.md#implemented-retained-owner-adoption). 377 network library, nine wire and 22 server library tests,
formatting and strict Clippy pass. World/client/normal manifest adoption is next;
this direct-lease fixture does not exercise game command transport or UI.
Continue headless only; desktop/mouse tests paused by user, local commits, no push.

Fixed quest item reward storage qualified locally, 6 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit reward-bound Store/worker turn-in now
commits delivery consumption, the whole fixed bundle, final-holdings quest state,
XP and the original receipt together. Full bags refuse without partial effects;
old successful retries retain later item moves without granting again. 73 actual
PostgreSQL contracts and process/database restart pass, plus eight persistence
and 22 server library tests. [Scope and next retained-owner slice](QUEST_ITEM_REWARDS.md#implemented-atomic-storeworker-turn-in).
World/client/server-owner admission remains closed. Continue headless work;
no desktop/mouse tests while the user works, and no push. Earlier entries retain
their checkpoint scope.

Fixed quest reward exact binding prerequisite qualified, 6 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: wire/storage profiles agree on exact reward,
inventory, quest, XP and optional area/document bytes; reward-only quests still
bind items. Legacy hashes remain unchanged. Existing runtime/owner/Store/client
admission stays closed for reward-bearing profiles. 371 core, 377 network,
eight persistence-library and 22 server tests, formatting and strict Clippy pass.
[Scope and next storage slice](QUEST_ITEM_REWARDS.md#implemented-exact-profile-binding-runtime-still-closed).
Continue authorized atomic reward storage locally; no push. No SQL reward proof.

Fixed quest reward pure proposal qualified, 6 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: completion, delivery consumption and fixed grants
now form one pure candidate with one final bag revision and final-holdings quest
reconciliation. Full bags/stale/missing inputs refuse atomically. Eight new cases,
371 core and 376 network regression tests, formatting and strict Clippy pass.
[Scope and next prerequisites](QUEST_ITEM_REWARDS.md#implemented-pure-turn-in-proposal).
No durable item reward or runtime admission yet; continue exact content binding
and authorized storage locally. No push. Earlier entries retain their scope.

Fixed quest reward metadata prerequisite, 6 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: a bounded separate catalogue validates known
quest/item fixed grants and rejects bundles impossible even in an empty bag.
Six new checks / 363 core tests, formatting and strict Clippy pass.
[Implemented scope and next steps](QUEST_ITEM_REWARDS.md#implemented-metadata-prerequisite).
No item grant proposal, durable turn-in or live admission yet. Continue locally;
no push. Earlier checkpoints retain their stated scope.

Readable quest documents qualified locally, 6 November 2026:
P3-05 / P3-11 / P3-15: opt-in manifest 20 now has a standard Bevy note reader
with exact server approval, local page navigation and contextual Escape. Native
1280×720, 1920×1200 and 3440×1440 pass pickup, pre-accept read exclusion,
read/possession readiness, destruction reversal, replacement pickup, consumed
turn-in and actual PostgreSQL restart/rejoin. RX 7900 XTX / RADV Vulkan reports
zero GPU diagnostics. [Scope and evidence](QUEST_DOCUMENTS.md#reader-native-qualification-complete--6-november-2026).
The fixture guarantees loot and reduces wolf danger; normal note loot remains
35%. Campaign loading, save migration and broader document interactions remain
separate. Work resumed by the user; commit locally only, no push. Earlier pause
and prerequisite entries below describe historical checkpoints.

Reader UI checkpoint, paused at user request, 5 November 2026:
P3-05 / P3-11 / P3-15: the opt-in standard Bevy note reader now opens exact local
pages after a matching server approval. 150 client and 376 network tests,
formatting and strict workspace/all-target Clippy pass. A complete native
1920×1200 login/pickup/read/loss/reacquisition/turn-in/PostgreSQL restart/rejoin
passes on RX 7900 XTX Vulkan, with zero GPU diagnostics. The additional size run
failed at launcher startup before gameplay; 1280×720 and 3440×1440 remain
unqualified. Work is a local checkpoint, not a completed reader release.
[Evidence and remaining checks](QUEST_DOCUMENTS.md#reader-ui-checkpoint-paused).
No push. Development stopped; resume only on user instruction.

Document manifest/CLI prerequisite qualified locally, 5 November 2026:
P3-05 / P3-11 / P3-15: opt-in schema 20 snapshots 24 exact roles and binds notes
with inventory/journal/XP/areas on both client and server. Five development quests
include reading and delivering a note; ordinary note loot is 35%, fixture 100%.
60 validator, 43 launcher and seventeen server tests, formatting and strict Clippy
pass. [Scope/evidence](QUEST_DOCUMENTS.md#implemented-prerequisite-normal-manifest-and-cli-adoption).
Reader action/panel and combined native proof remain pending. Continue those;
no push. Qualified playable remains `65977bd`.

Encrypted document bridge qualified locally, 5 November 2026:
P3-05 / P3-11 / P3-15: two encrypted peers at 20/30/60 Hz pass held read approval,
shared bag ordering, fresh missing-slot refusal and original retry under packet
delay/loss/duplication. Private replies stay with their owner. Formatting and
strict Clippy pass. [Scope/evidence](QUEST_DOCUMENTS.md#encrypted-document-bridge-qualified).
This uses synthetic transport publications; real OIDC/SQL ownership is qualified
separately. Continue normal manifest and Bevy reader, then combined native proof.
No push; qualified normal playable remains `65977bd`.

Retained document owner and World admission qualified locally, 5 November 2026:
P3-05 / P3-11 / P3-15: explicit exact document context now connects the retained
item owner to durable opening. Real login, encrypted ticket and PostgreSQL prove
original-stage credit, queued retirement/deletion, rejoin and one-time turn-in XP.
375 network and sixteen server tests, formatting and strict Clippy pass.
[Scope/evidence](QUEST_DOCUMENTS.md#implemented-retained-document-owner-and-explicit-world-admission).
The owner fixture drives leases directly; document-wire/native combined proof
and normal manifest/Bevy reader remain next. No push; playable remains `65977bd`.
Earlier entries describe prior checkpoints.

Bounded document transport bridge qualified locally, 5 November 2026:
P3-05 / P3-11 / P3-15: exact-profile document intents share bounded inventory
ordering. Matching read approval permits an unchanged bag; ordinary mutation
revision checks and legacy codecs remain strict. Network, core and server
regressions, formatting and strict Clippy pass. [Scope/evidence](QUEST_DOCUMENTS.md#implemented-prerequisite-bounded-document-transport-bridge).
Owner publications in these bridge tests are synthetic. Continue retained owner
and explicit World adoption, then normal manifest/Bevy reader. No push.
Qualified playable remains `65977bd`; earlier entries are prior checkpoints.

Durable document opening qualified locally, 5 November 2026:
P3-05 / P3-11 / P3-15: bound Store/worker opens authorize the saved item and
record original-stage read credit before success. Exact retries, item loss,
concurrent mutation, rollback, lost replies and shutdown drain pass. 66 actual
PostgreSQL contracts/restart, seven library and fifteen server tests, formatting
and strict Clippy pass. [Scope/evidence](QUEST_DOCUMENTS.md#implemented-durable-authorized-document-opening).
Retained owner, encrypted commands, normal manifest and Bevy reader remain pending;
continue those locally. No push. Qualified playable remains `65977bd`.
Earlier entries describe prior checkpoints.

Document content binding prerequisite qualified, 5 November 2026:
P3-05 / P3-11 / P3-15: explicit wire/storage profiles now bind exact document,
inventory, journal, XP and optional area bytes; read-only quests still bind items.
Legacy Store/Publisher/owner/World/client execution stays closed for documents.
Fifteen server, seven persistence-library and 60 actual PostgreSQL regression
contracts/restart, formatting and strict Clippy pass. [Scope/evidence](QUEST_DOCUMENTS.md#implemented-prerequisite-exact-documentitemjournal-binding).
Continue durable authorized opening locally; no push. No document UI/runtime yet.
Qualified normal quest-loot playable remains `65977bd`.

Read-objective core prerequisite qualified, 5 November 2026:
P3-05 / P3-11 / P3-15: schema-4 `read_document` objectives and opaque original-
stage observations now preserve historical read credit separately from possession.
Registry compilation/version review support the selector; legacy runtime, World
and normal manifest paths refuse it. 357 core, 368 network, twelve server and
79 authoring/validation tests, formatting and strict Clippy pass.
[Scope and next slices](QUEST_DOCUMENTS.md#implemented-prerequisite-schema-4-logical-read-objectives).
Continue exact document/item/journal binding and durable opening locally; no push.
No document UI/runtime adoption yet. Qualified playable remains `65977bd`.

Readable-document catalogue prerequisite qualified, 5 November 2026:
P3-05 / P3-11 / P3-15: bounded plain-text document definitions now map to known
items. Pure bag-slot/revision checks reject missing, moved, deleted or wrong
items; they grant no authorization or read credit. All 351 core tests, formatting
and strict workspace/all-target Clippy pass. [Scope and next slices](QUEST_DOCUMENTS.md).
Read objective, exact runtime binding, durable open receipts and UI remain pending.
Continue those locally; no push. Qualified quest-loot playable: `65977bd`.

Normal quest-only loot qualified locally, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: opt-in manifest schema 19 binds the exact item
journal before conditional loot on client/server and selects the retained owner.
Linux native pickup, split/delete reversal, replacement pickup, consumption and
PostgreSQL restart/rejoin pass at 1280x720, 1920x1200 and 3440x1440. First three
unaccepted lives stay denied; four later lives retain their original rights.
57 validator, 41 bundle, 147 client and eleven server tests, formatting, strict
Clippy and optimized builds pass. [Evidence and limits](LOOT_TABLES.md#normal-quest-only-loot-qualified--5-november-2026).
The native fixture guarantees loot and uses low damage; ordinary random gameplay
content remains 60% pelt / 25% token. Continue readable quest documents next;
commit locally only, no push. Earlier gate/stop entries describe prior checkpoints.

Quest-only client/transport prerequisite qualified, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit client loot profiles retain item/vendor
identity and require the exact configured item journal before connection.
Conditional encrypted two-peer tag/life/private pickup passes at 20/30/60 Hz with
UDP impairment; its owner publications are synthetic. 368 network tests,
formatting and strict Clippy pass. [Scope/evidence](LOOT_TABLES.md#quest-only-client-binding-and-encrypted-bridge--5-november-2026).
Normal CLI/manifest/native conditional adoption remains pending. Continue that
locally; no push. Qualified ordinary item-quest playable: `85fd967`.

Quest-only World prerequisite qualified, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit simulation admission now requires the
exact installed item journal and inactive matching combat/creature. Real fatal
casts at 20/30/60 Hz preserve original loot facts; durable need stays in the owner.
365 network-library tests, formatting and strict Clippy pass.
[Scope/evidence](LOOT_TABLES.md#quest-only-world-admission--5-november-2026).
Normal client/manifest adoption remains pending. Continue client binding and
conditional encrypted runtime checks locally; no push. Playable: `85fd967`.

Quest-only retained owner qualified, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit exact-journal owner admission now uses
conditional generation/view/pickup; stationary bag/journal changes refresh loot.
Actual OIDC/encrypted-ticket/PostgreSQL checks cover denial replay, deletion,
retained pickup/queued fatal facts, rejoin and atomic turn-in. Ordinary loot and
item-journal regressions, eleven server tests, formatting and strict Clippy pass.
[Scope/evidence](LOOT_TABLES.md#quest-only-retained-owner--5-november-2026).
World and normal manifest/client adoption remain pending. Continue locally with
World validation and encrypted runtime coverage; no push. Playable: `85fd967`.

Quest-only worker prerequisite qualified, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit conditional generation/list/pickup now
use the existing bounded critical FIFO. Accepted writes survive queued reply loss
and drain on shutdown; async producers wait for capacity and stale need rejects
without stopping the worker. 60 actual PostgreSQL contracts plus restart, eleven
server-library tests, formatting and strict Clippy pass. [Scope/evidence](LOOT_TABLES.md#quest-only-worker-prerequisite--5-november-2026).
Retained owner, World and normal manifest/client adoption remain closed. Continue
owner refresh and authorization locally; no push. Playable remains `85fd967`.

Quest-only paging/pickup storage prerequisite qualified, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit Store views filter current item need
before the nearest-16 page; pickup validates original generation and fresh locked
need. Ready replay, deletion/reacquisition, completion, ordinary rewards and full-
bag rollback pass 59 actual PostgreSQL contracts plus restart. [Evidence/limits](LOOT_TABLES.md#quest-only-paging-and-pickup-storage-prerequisite--5-november-2026).
Formatting and strict all-target Clippy pass. Generic APIs, worker/owner/World
and normal manifests remain closed for quest-only content. Continue worker and
retained-owner adoption locally; no push. Normal playable remains `85fd967`.

Quest-only generation storage prerequisite qualified, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit trusted Store generation now persists
first-transaction eligibility, the single ticket and observed bag/journal revisions.
Later acceptance/removal cannot create old rights; a failed winning claim rolls
back the whole generation. 57 actual PostgreSQL contracts plus process restart,
version-13 ordinary-roll upgrade, formatting and strict all-target Clippy pass.
[Ordering policy, evidence and limits](LOOT_TABLES.md#quest-only-generation-storage-prerequisite--5-november-2026). Owner/World/manifest admission and
conditional paging/pickup remain closed. Continue those locally; no push.
Normal item-quest playable remains the separately qualified `85fd967`.

Quest-only profile binding prerequisite, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit headless constructors now bind quest-only
loot to the exact inventory-bound journal, including its XP/area/source identity.
Legacy constructors, owner/World admission, ordinary Store paths and normal
manifests stay closed for this content. Eleven server, five non-SQL persistence,
ten reward-wire, 56 validator and 13 focused core tests, formatting and strict
all-target Clippy pass. [Scope and remaining work](LOOT_TABLES.md#quest-only-profile-binding-prerequisite--5-november-2026).
No new SQL/native/runtime-adoption claim. Normal item quests remain qualified at
`85fd967`. Continue durable generation rights and filtered paging/pickup locally;
no push. Earlier metadata-only gate statements describe their checkpoints.

Quest-only loot metadata prerequisite, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: reward schema 4 now describes quest-required
independent single-item per-life entries. Ordinary chances stay separate from
eligibility. Wire profiles, storage profiles and normal manifests refuse these
entries until generation rights, exact journal binding and pickup checks exist.
345 core, ten reward-wire, four persistence-library and 56 validator tests,
formatting and strict all-target Clippy pass. [Scope and remaining work](LOOT_TABLES.md#quest-required-metadata-prerequisite--5-november-2026).
No new SQL/native/runtime claim; normal item quests remain qualified at `85fd967`.
Continue implementation locally; no push.

Normal item quests qualified locally, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: opt-in schema-18 inventory-bound quests now pass
native Linux pickup, split/delete reversal, replacement pickup, atomic turn-in
consumption and actual PostgreSQL restart/fresh login. 1280x720, 1920x1200 and
3440x1440 pass on RX 7900 XTX / RADV Vulkan with zero GPU diagnostics. Guaranteed
loot and low-damage combat are explicit native fixtures; the ordinary random-loot
profile is separate. 147 client, 55 validator and 40 bundle tests, formatting,
strict all-target Clippy and optimized builds pass. [Evidence and limits](QUEST_ITEM_OBJECTIVES.md#normal-item-quest-qualification-complete--5-november-2026).
Continue with quest-only generation rights and current-need checks before paging
and pickup. Commit locally only; no push. Earlier stop/gate entries are historical.

Explicit headless item-journal admission prerequisite, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: opt-in World installation/preparation now requires
the exact bound inventory profile; legacy/raw paths remain closed. 363 network
library tests, nine wire checks, formatting and strict all-target Clippy pass.
[Exact scope and remaining normal adoption](QUEST_ITEM_OBJECTIVES.md#explicit-headless-item-journal-admission--5-november-2026).
No new saved-player/native claim. Normal CLI/manifest item-quest admission and
quest-only loot filtering remain pending. Continue implementation with local
commits only; no push. Ordinary loot checkpoint remains `4dbeefd`.

Quest-only loot eligibility core prerequisite, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: current checked backpack holdings now derive the
items needed by accepted unfinished quests, independently of cached Ready counts.
343 core tests, formatting and strict all-target Clippy pass. [Exact scope and
remaining adoption](LOOT_TABLES.md#quest-only-eligibility-core-prerequisite--5-november-2026).
No new runtime/SQL/native claim; quest-only and live possession gates remain closed.
Ordinary corpse pickup/random loot remain qualified at `4dbeefd`. Continue with
joint profile binding, bounded eligible pages and pickup checks; local commits only.

Corpse pickup and authored loot rolls qualified locally, 5 November 2026:
P3-02 / P3-05 / P3-09 / P3-11 / P3-15: opt-in schema-17 corpse UI passes three
native Linux sizes and database restart/rejoin. Reward schema 3 adds authored
independent chances and exclusive groups with durable winning/missing rolls.
1,295 workspace/all-target tests pass (65 service skips); 55 actual PostgreSQL
contracts/restart, actual OIDC/SQL retained-owner rolls, formatting and strict
all-target Clippy pass. [Pickup evidence](QUEST_ITEM_OBJECTIVES.md#explicit-corpse-pickup-ui--5-november-2026)
and [drop-table semantics, profiles and limits](LOOT_TABLES.md).
Guaranteed content remains the native UI fixture; random ordinary loot has a
separate development manifest. Quest-only eligibility and live possession quests
remain gated; continue those next. Local commits only, no push. Earlier pending/
stop entries below are historical.

Explicit corpse pickup UI qualified, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: normal opt-in corpse selection, read-only opening,
individual pickup, serial Loot all and partial-loot recovery pass native Linux
at 1280x720, 1920x1200 and 3440x1440 plus actual database restart/rejoin.
[Exact evidence and limits](QUEST_ITEM_OBJECTIVES.md#explicit-corpse-pickup-ui--5-november-2026).
Guaranteed drops remain the repeatable UI fixture; [authored random rolls](LOOT_TABLES.md)
have separate storage evidence and combined checks in progress. Live item quests
remain gated. Continue implementation, commit locally, do not push.

Nearby durable loot interest qualified, 5 November 2026:
P3-02 / P3-05 / P3-09 / P3-11 / P3-15: per-life owner pages follow the retained
server pose instead of distant oldest drops. 54 actual PostgreSQL contracts and
restart, new per-life and legacy OIDC/SQL owner qualifications, 394 selected
library tests, formatting and strict all-target Clippy pass.
[Bounds, evidence and remaining client work](QUEST_ITEM_OBJECTIVES.md#nearby-durable-loot-interest--5-november-2026).
Continue implementation with local commits only. Corpse UI/normal manifest and
live item-quest native adoption remain pending. No new rendering/capacity claim.

Implementation resumed, 5 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: explicit headless per-life pickup now shares the
backpack acknowledgement stream and passes impaired encrypted two-peer tests
at 20/30/60 Hz. 381 selected tests, formatting and strict all-target Clippy pass.
[Scope and next slice](QUEST_ITEM_OBJECTIVES.md#explicit-headless-pickup-integration--5-november-2026).
Default/normal manifest adoption stays gated; corpse UI and native item-quest
flow remain pending. Continue until user stops, commit locally, do not push.
Earlier stop entries below are historical.

Stopped safely at user request, 4 November 2026:
Explicit corpse-life pickup validation prerequisite passes 367 selected tests,
formatting and strict workspace/all-target Clippy. [Exact scope and resume work](QUEST_ITEM_OBJECTIVES.md#safe-stop-explicit-pickup-authority-prerequisite--4-november-2026).
Per-life content and live item-quest admission remain closed: corpse UI, client
pickup and encrypted/native integration are unfinished. Source/proof preserved at
`build/quest-loot-pickup-stopped-v1-20261104`; last qualified runnable UI remains
`build/quest-item-ui-qualified-v1-20261104`. No active owned services or desktop
input. Changes committed locally only; no push. Resume only when requested.

Original fatal-life reward forwarding checked locally, 4 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: reward facts retain the exact victim life and death
position through the bounded queue to the async owner. 357 network tests and
actual OIDC/encrypted-ticket/PostgreSQL reward retirement qualification pass;
formatting and strict all-target Clippy pass. [Scope and remaining pickup work](QUEST_ITEM_OBJECTIVES.md#original-fatal-life-forwarding--4-november-2026).
Per-life runtime admission stays closed. Continue with local commits only.

Loot availability codec prerequisite checked locally, 4 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: opt-in RW02 carries bounded victim/corpse metadata;
legacy RW01/RC01 remain byte-compatible. 366 selected checks, formatting and
strict workspace/all-target Clippy pass. [Exact bounds and remaining work](QUEST_ITEM_OBJECTIVES.md#bounded-loot-availability-codec-prerequisite--4-november-2026).
Default profile and World admission still refuse per-life loot. Continue with
corpse interaction/explicit pickup; no new encrypted/native/SQL claim or push.

Per-life loot storage prerequisite qualified locally, 4 November 2026:
P3-02 / P3-05 / P3-09 / P3-11 / P3-15: migration 12 preserves old once-character
claims and binds repeatable loot to victim life and corpse position. Death/offer
alone changes no bag or possession credit; explicit pickup commits both together.
53 actual PostgreSQL contracts and restart hashes pass; 377 selected regular
tests, formatting and strict workspace/all-target Clippy pass. No full-workspace
execution claim. [Exact scope and pending corpse interaction](QUEST_ITEM_OBJECTIVES.md#per-life-loot-storage-prerequisite-qualified--4-november-2026).
Preserve `build/quest-loot-storage-qualified-v1-20261104`. Runtime per-life content
and live item quests stay gated. Continue explicit corpse pickup/UI with local
commits only; no new native/Windows/capacity claim for this storage slice.

Per-life loot core prerequisite checked locally, 4 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: schema 2 and trusted victim-life selector added;
339 core tests, three reward-wire checks and one non-SQL storage gate pass.
Formatting and strict workspace/all-target Clippy pass. Runtime/storage still
refuse per-life content; no live repeatable-loot or new SQL/native claim.
[Current boundary and next work](QUEST_ITEM_OBJECTIVES.md#per-life-loot-core-prerequisite--4-november-2026).
User-authorized work continues with local commits only. Durable per-life claims
and bounded pending availability are next; item-quest admission stays closed.

Confirmed destruction UI qualified locally, 4 November 2026:
P3-05 / P3-09 / P3-11 / P3-15: normal split/confirm/cancel/close/reopen/destroy
and database restart/rejoin pass at 1920x1200, 3440x1440 and 1280x720.
All four native starts have zero GPU diagnostics on Linux RX 7900 XTX / RADV.
144 client tests, 11 inventory bridge tests, formatting and strict workspace/
all-target Clippy pass; optimized current-checkout client/server/validator built.
[Exact scope and evidence](QUEST_ITEM_OBJECTIVES.md#confirmed-destruction-ui-qualified--4-november-2026).
Preserve `build/quest-item-ui-qualified-v1-20261104`. User resumed implementation
and authorized local commits only. Continue with actual per-creature-life loot;
item-quest live admission and document-reading objectives remain pending.
Earlier stop/local-only records below are historical. No new native Windows claim.

Repository alignment completed at user request, 4 November 2026:
Approved story reconciliation [PR #52](https://github.com/code-studio-au/theia-mmo/pull/52)
and clean-checkout CI repairs [PR #53](https://github.com/code-studio-au/theia-mmo/pull/53)
are merged into `main`. Its tree matches the qualified story head exactly.
The complete Linux/Windows matrix passed on
[run 41801271067](https://github.com/code-studio-au/theia-mmo/actions/runs/41801271067):
1,046 Rust tests on Linux and 1,044 on Windows, 46 service-dependent skips each;
formatting, denied-warning Clippy and asset-tool suites pass on both.
Q169's authoring cutoff binds to Oth'varen's actual death objective; runtime expiry
and the unresolved custody/turn-in cases remain pending, not executable behavior.

Unfinished MMO work is committed and pushed on `dev/quest-inventory-integration`,
with current `main` merged back in. [PR #54](https://github.com/code-studio-au/theia-mmo/pull/54)
remains a draft and is not merged. The aligned development branch passes all eight
encrypted account tests, formatting and strict network/all-target Clippy locally.
These checks do not qualify the unfinished destruction UI or native item-quest flow.
Implementation remains paused; resume scope is [item quest work](QUEST_ITEM_OBJECTIVES.md).
Earlier local-only/stop records below describe their historical checkpoints.

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

Checkbox slice editor build passes: `build/sprinkles-review/checkbox-build.log`.
