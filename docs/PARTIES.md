# Party membership — P4-11

## UI layout reference — 16 November 2026

Additional screenshots on 16 November at 15:38:52, 15:38:58 and 15:39:21
explicitly extend the requested direction to invitations, member stats and
party targeting/marking. Originals use the same Pictures path below with date
2026-11-16. The invitation prompt and compact name/health/mana frames are qualified in STATUS;
portraits, member targeting, level/status icons and shared marks remain planned:

- Incoming invite: compact top-centre prompt naming the inviter, with Accept
  and Decline side by side while the world stays visible. Reuse the existing
  invitation identity, expiry and pending-action guards.
- Member frames: portrait, name, health/resource bars and leader crown;
  selecting a member shows their target frame, level and available status icons.
  Display server-observed values with explicit unavailable/out-of-range states.
- Shared unit marks: a compact palette and clear action; show the assigned
  symbol above the unit and on its target frame. Skull, cross, square, moon,
  triangle, diamond, circle and star are visual references. Mark assignment
  needs party-scoped authoritative state, permissions, epoch-qualified targets
  and leave/despawn cleanup before it can be presented as working. Selecting
  or marking must preserve manual facing and must not itself start an attack.
- The pictured Ground tab is a separate ground-marker idea; it does not make
  ground placement, pings or ready checks part of the unit-marker implementation.

Acceptance for the remaining UI includes readable layouts at 1720x1400 paired
and 1720x700 four-player tiles, invite expiry/decline, stale member guards,
cross-client mark agreement and party isolation. The current membership slice
now replicates and displays authoritative health/mana. Level/status replication,
member targeting and shared marks remain later slices.
Title/selection/loading references are in [CHARACTER_LAUNCHER_MENU.md](CHARACTER_LAUNCHER_MENU.md).

The user supplied three WoW screenshots captured on 15 November at 21:14:46,
21:15:01 and 21:15:09 as party UI/menu layout references. Local originals are
`/home/steven/Pictures/Screenshot from 2026-11-15 <time>.png` (hyphenated times).
They show compact member portrait/name/health/resource frames near the upper
left, a collapsible group-management panel beside those frames, and a member
context menu anchored beside the selected frame. The panel has a group heading,
member count and a prominent Leave Party action. The menu groups member actions
under the character name, including Promote to Leader and Uninvite.

Use this as the next presentation direction with Theia's existing Bevy UI and
licensed UI assets. The current Party [O] panel remains the qualified implementation;
compact member frames and member context menus are adopted below. Context-menu
promotion/removal reuse the existing authority.
Preserve painted revisions, pending-command reservation, stale-member guards,
Escape handling and pointer ownership; right-clicking a member frame must not
also steer the character. Persistent member bars require matching authoritative
member data, including an explicit unavailable state when data is absent.

The screenshots also show markers, pings, ready checks, roles, whisper, inspect,
trade, follow and other features. Their presence is reference material, not
evidence of implemented Theia functionality or a decision to add every pictured
system in this UI pass. Do not present inert controls as working actions.

## Reference and scope

The original Blizzard World of Warcraft manual (copyright 2004, layout dated
9/11/04), printed pages 140–141, supports five members, invitation acceptance or
decline, one group per player, leader-only invitations/removal/promotion and
leadership passing to the next member when the leader leaves. A group starts
when the first invitation is accepted. Source: [Blizzard CDN manual](https://bnetcmsus-a.akamaihd.net/cms/template_resource/LO0VQ46XB1281555957773363.pdf),
retrieved 15 November 2026, SHA256
`897a39e8c8119589de46df1af339b84bab548ba2b2ba41746d8bfa98e90291e3`.
This is a 2004 manual, not proof of the exact 1.12 server implementation. Counts
are dimensionless; this membership slice introduces no distance formula.

## Initial Theia policies

These engineering choices are not claimed as verified Classic rules:

- Session-local membership for the current eight-player world; no database work
  or awaited I/O in membership operations. Death does not leave a party.
- Invitations expire after 60 simulation seconds. One incoming invitation per
  player; pending invitations reserve the inviter's remaining party slots.
- Accepting another invitation cancels one's outgoing invitations. Promotion,
  leader departure and dissolution cancel the old leader's invitations. A party
  with fewer than two members dissolves; successor selection uses join order.
- Disconnect retires the incarnation, cancels its invitations and leaves its
  party. Reconnection creates a new solo incarnation. No offline membership.
- Caller identity comes from admitted transport ownership. Epoch-qualified
  target identities, monotonic request sequences and expected private-view
  revisions reject forged, duplicate and stale commands. Valid-sequence refusals
  consume the sequence. Each player is limited to four requests per two simulation
  seconds, including policy refusals. Replays do not consume additional slots.
- Only the invited player can accept/decline; only the inviter can cancel. Leader
  removal cannot target oneself (use Leave). Promotion retains original join
  order. No self-invitation or membership in multiple parties.

## Delivery and acceptance

The bounded codec and encrypted worker integration now pass. Party mode binds an
explicit compatibility suffix after Classic movement and optional chat. Requests
contain no caller ID. Each private state includes the owner's membership/revision,
last processed request outcome and a bounded roster of authenticated same-world
display names, without account IDs or positions. The name roster carries no
stats; PS02 separately includes exact current party members and their optional
authoritative health/mana pools in join order. Unavailable profiles remain
unavailable, and solo recipients receive no member stats. Membership, vitals and roster
updates coalesce under reliable-channel pressure, reserving half its budget for
gameplay. Invalid frames, foreign epochs, forged state uploads and invalid request
sequences disconnect; ordinary policy refusals acknowledge without disconnect.
The client retains the painted revision, allows one outstanding command, bounds
initial state to eight seconds and request waits to five seconds, and discards
queued clicks on timeout/admission. A late older acknowledgement cannot clear a
newer pending command. The 60-second invitation expiry uses simulation ticks;
client wait deadlines use elapsed wall time.

Three actual encrypted Classic client workers verify private invitations,
acceptance, non-leader refusal, promotion, disconnect succession and leave. The
same flow passes with chat enabled. Admissions in this headless test use named
trusted leases; native OIDC-account/UI evidence is recorded below. Standalone
launches remain opt-in.

## Playable opt-in

`tools/play.py --parties` requires Classic account admission. Exported account
packages can use `tools/account_population_playtest.py --parties`; source gameplay
files are unchanged. Matching client/server preflight binds the party suffix.
Party [O] opens the standard Bevy panel. Its named rows offer Invite, Accept,
Decline, Cancel, Promote and Remove when appropriate, plus Leave party and Close.
One submitted click is reserved until acknowledgement; controls keep their painted
revision. Interface Escape precedes Party, then the existing panels and target
handling. The world keeps running while the panel is open.

Stationary membership qualification `build/party-ui-native-v1/qualification.json` passes on two real OIDC
accounts at 1920x1440 each, then the same characters after PostgreSQL/server
restart: 13 accepted UI commands per phase, zero refusals/invalid uploads, exact
unchanged journal/bag/XP and fresh solo membership on rejoin. Source hashes and
SQL checks pass; 140.52 seconds total. Invitation/member screenshots were visually
inspected. Linux RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6, zero GPU
diagnostics; no Windows execution. 281 client tests (four explicit skips), build,
format, strict client/server Clippy and 53 launcher tests (one explicit skip) pass.
That first qualification did not exercise movement. The editor quest probe
`build/party-editor-native-v1` subsequently exposed a local predictor still bound
to the Classic-only identity: party-enabled corrections were refused, preventing
movement. Its failure is not quest-flow acceptance. The predictor now derives the
complete Classic → optional chat → optional party identity; foreign identities
still fail. Encrypted worker regressions exercise real predicted facing/movement
with parties and parties plus chat, and reject the old binding. Native qualifiers
now require actual movement and reject prediction-binding warnings.

Shared XP, loot, tags and general quest credit remain separate slices. The
shared-escort implementation and its acceptance are described below; earlier
private-solo acceptance is retained as regression evidence.

Movement regression acceptance: `build/party-ui-native-v3/qualification.json`
passes both real accounts before and after database/server restart in 144.14
seconds. Ordinary W-key holds move each actor 0.93–1.63 metres; all membership
controls and unchanged quest/bag/XP checks pass. Zero prediction-binding warnings
and GPU diagnostics on the same Linux RX 7900 XTX/Vulkan device; no Windows run.
535 network tests, 281 client tests, format and strict network/client/server/editor
Clippy pass. See STATUS for log paths and explicit skips.

Editor account Start now enables this profile. Native `build/party-editor-native-v2/qualification.json`
passes export, account create/select, the Party panel at 3440x1440, all three
travel quests, editor Stop and durable PostgreSQL/server restart with the same
journal revision 9, XP 75 and bag revision 2. Source bytes remain unchanged,
owned services close and GPU/prediction-binding diagnostics are zero. The party
screenshot was visually inspected. Same Linux GPU as above; no Windows run.
All 263 editor tests (16 skips), build, format and strict Clippy pass; see STATUS.

## Shared active escorts — accepted 16 November 2026

The authoritative fixed tick now groups committed Active escort journeys by
party and NPC, advances one checked copy, then publishes that position through
each participant's existing contact view. Any living participant within the
existing 10-metre range permits travel at the authored 7 m/s; none in range stops
it. Prepared navmesh corners and ordinary terrain collision remain unchanged.
Unaccepted/Ready/Completed characters keep their private contact state. Pending
acceptance does not start or join travel. Every character still requires a local
arrival fact, durable Ready journal and their own conversation to complete.

Late acceptance or merging ongoing journeys adopts the furthest checked progress
on that route. Leaving/dissolution forks at the current position without rewind;
disconnect removes its journey. Death preserves the observed shared position but
cannot drive movement or receive arrival credit. Restart retains the existing
Active-at-home and Ready/Completed-at-destination policy. These explicit Theia
rules are bound in the party v3 compatibility identity; no shared XP or general
quest-credit policy is inferred.

Authority regressions at 20/30/60 Hz cover solo/unaccepted isolation, range handoff,
all-away waiting, late acceptance, leave/merge, disconnect and independent arrival.
An actual life-authority death proves a dead member cannot drive or receive the
arrival fact while the living partner can.

Native `build/party-escort-native-v2/qualification.json` passes two actual OIDC
accounts at 1920x1440, sharing the Active escort with range handoff, all-away wait,
unaccepted contact isolation, individual arrival/turn-in and PostgreSQL/server
restart. Both restore journal revision 9, XP 75 and bag revision 2. Source hashes
and SQL pass; the fresh phase accepts exactly invite/accept and the resumed phase
rejoins solo. Zero refusals, invalid uploads, prediction-binding or GPU diagnostics;
295.44 seconds total. Linux RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6;
no Windows execution. Waiting and Ready captures were visually inspected.

The 135 movement samples contain actual rendered ranger positions in both clients.
Across the two driving legs, the background client records 23/57 distinct rendered
positions and 28.00/53.40 m displacement. Maximum sampled authority separation
between clients is 1.172 m; maximum presentation lag is 0.749 m. This replaces the
inconclusive late stationary render sample from v1. Fresh 538 network tests pass
(18 explicit skips), fmt, denied-warning all-target network/client/server/editor
Clippy and Python compilation pass. Logs: `build/party-escort-resume-*.log`.
The earlier `party-escort-build.log` and `party-escort-death-tests.log` remain
applicable; no Rust implementation changed during the 16 November rerun.


16 November presentation continuity correction: earlier quest-completion evidence
missed brief NPC disappearance while new world shaders prepared. Complete authored
scene pipeline preparation now passes `build/scene-pipeline-native-v1/qualification.json`
(283.92 seconds): both first-quest approaches, ranger handoff and durable restart,
116 prepared pipelines and zero readiness retractions for both clients in both
phases. Party authority is unchanged. See STATUS for reproduction, GPU evidence
and limits; a startup loading screen is not implemented.


## Private member vitals — 16 November 2026

P4-11 prerequisite for member frames: party profile v4 / PS02 carries current
health, maximum health, mana and maximum mana from the same authoritative pools
used by private combat observations. Resource changes do not change membership
revisions or consume request acknowledgements. Exact membership/order and checked
resource bounds reject extra, missing, foreign, malformed and legacy entries.
Death retains zero health in the party; respawn updates the same member; departure
and dissolution remove observations. Stats span the current bounded world even
outside spatial interest; this is explicit Theia policy, not a Classic range rule.

541 network tests pass (18 skips), including real encrypted three-peer delivery
of a spell's mana cost, outsider isolation, leave/disconnect cleanup, full codec
round trips/rejections, bounded-channel coalescing, and damage/mana/death/respawn
at 20/30/60 Hz. The unavailable-resource encrypted fixture still passes. Client
suite: 284 pass / four skips. Exact logs are `build/party-vitals-*.log` and
`build/party-vitals-checks.json`. No rendering changes in this prerequisite; the
next slice will qualify actual frames at both tile heights. Existing editor
account playtests adopt the new protocol through matching rebuilt binaries.


## Compact member frames — accepted 16 November 2026

Up to four other members appear at the upper left with their trusted names,
leader label and numeric health/mana bars. Initial badges stand in for portraits
until remote appearance data is available. The header opens the existing Party
management panel. Frames move beside open dialogue, retain mouse ownership, and
disappear on membership/connection loss. Resource updates repaint existing widgets;
identity/name/leader changes rebuild their bounded layout. Zero and unavailable
resources remain distinct. This initial frame slice did not yet select a player
or open a context menu; the latter is qualified below.

286 client tests pass (four skips), including five-member formation/promotion/
removal/dissolution and partial, zero and unavailable bar updates without widget
replacement. Native two-account checks at 1720x1400 and 1720x700 pass all thirteen
membership actions, rendered name/leader glyphs, rendered bar widths against the
other client's HUD, frame removal, Manage shortcut, drag ownership, unchanged
quest/bag/XP state and server/PostgreSQL restart. Evidence:
`build/party-frames-paired-v2/qualification.json` (147.80 seconds) and
`build/party-frames-quadrants-v2/qualification.json` (149.06 seconds). Captures were
visually inspected. Paired frames measure 268x73; quadrant frames 260x72 after
interface scaling. No scene-readiness retractions or GPU diagnostics.

Linux RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6; no Windows execution.
The smaller-window native run uses two accounts in the first two quadrants,
not four-account gameplay. Partial/zero/unavailable painting and maximum member
count have headless coverage; native resource checks here use full starting pools.
The prior encrypted network slice separately proves spell-cost delivery and
life transitions. Matching client build, fmt, strict all-target client Clippy and
Python compilation pass (`build/party-frames-checks.json`). The editor's existing
account launch path inherits these frames; no extra editor options are needed.
The v1 visual run exposed a collapsed name column and is not acceptance; explicit
text-column widths and rendered-name/height checks fixed it before the v2 reruns.


## Member context menus — accepted 16 November 2026

Right-click a compact member frame for its trusted name, role, Manage and Close.
Only the current leader sees Promote/Remove; these reuse existing server commands,
painted owner/revision/member guards and the shared pending reservation. Resource
updates keep the menu; membership/revision changes, disconnection and focus loss
retire it. Escape closes context before Party. Outside dismissal and frame drags
retain UI ownership without steering, facing changes or world actions.
289 client tests pass (four skips), including authority-derived stale/role guards,
keyboard ordering and both viewport clamps. Build, fmt, denied-warning all-target
client Clippy and Python compilation pass; build/party-menu-checks.json.
Native two-account controls at 1720x1400 and 1720x700 pass all 13 membership actions
through fresh and PostgreSQL/server-restored accounts, with actual context-menu
promotion/removal, nonleader permissions, Manage/Close, Escape/outside/focus
dismissal, unchanged progression and zero readiness retractions/GPU diagnostics.
Evidence: build/party-menu-paired-v2/qualification.json (160.22 seconds) and
build/party-menu-quadrants-v2/qualification.json (156.95 seconds).
Both menu captures were visually inspected. Linux RX 7900 XTX / RADV NAVI31 /
Vulkan / Mesa 26.2.6; no Windows run. Quadrant checks use two accounts in the
first two tiles, not four-account gameplay. V1 stopped at a test-harness variable
collision after promotion; v2 fixes the harness and is the acceptance run.
Editor account playtests inherit the client controls with no extra options.
Next P4-11 prerequisite: distinguish authoritative friendly party selection from
hostile targets before enabling frame selection and shared unit marks. The
adopted title/selection/loading flow remains subsequent work. Continue locally;
no push authorized.
