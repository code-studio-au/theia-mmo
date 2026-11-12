# Public player visual lifecycle — 3 November 2026

P3-04 / P3-14 / P3-16. Prerequisites: existing server-owned player life,
death/respawn movement barrier, exact actor identity and standard Bevy character
Death/locomotion clips. Latest qualified playable is
`build/avatar-life-client-qualified-v1-20261103`. World/codec, reliable transport,
explicit normal schema-15 selection and standard Bevy adoption are qualified.
Earlier component/playable checkpoints retain their recorded scopes.

## Public boundary

Previously the observer saw another player's moving body without its private
death state. The implemented separate public projection contains only
World epoch, completed tick, exact actor/life identity and alive/dead status.
Keep current/max health, damage/results, incoming windups, respawn deadlines and
movement commands on their existing private authority paths. No gameplay formula,
respawn entitlement, facing, target or collision behavior changes.

Life generation is deliberately public **visual identity** in this new contract.
It is the existing nonzero authoritative generation, not a new client-selected
nonce. It lets an observer reset its animation even if a render stall spans both
death and respawn. It grants no input or damage authority. NA01 remains unchanged
and still contains no player-life generation. Movement uses its existing separate
authoritative input-stream/life barrier; AV01 cannot replace that barrier.

`World::enable_public_avatar_life` explicitly adds a pre-admission Classic World
identity wrapper after public NPC timing when enabled. It binds the exact existing
life profile and a separate public visual contract domain. Inactive modes retain
their identities. A read-only query projects only the bounded current connected
player set and refuses unavailable/inconsistent life authority.

## Codec prerequisite

AV01 has a 54-byte header (tag, supported tick rate, profile hash, epoch, tick,
count) and 17 bytes per actor (entity ID, life generation, dead flag). The current
development connection cap is eight, so the maximum is 190 bytes. Full rosters
are ordered and unique, with nonzero exact identities and strict flags, lengths,
profile, epoch and rate checks. This is the current one-server development roster,
not region interest management or an MMO population-capacity claim.

Receive state binds the admitted owner/world/profile/rate and requires that owner
in live rosters. It atomically refuses contradictory same-tick observations,
generation regression and revival without a fresh life. Duplicates and stale
frames cannot replay an old corpse/life. Join/leave removes absent remote actors;
current-generation data remains sufficient after presentation coalescing. A new
world/admission constructs fresh receive state. No unbounded tombstone history.

Five wire and four actual World checks pass, including 20/30/60 Hz. All 664
network tests pass (ten explicit service skips), along with formatting and
workspace/all-target denied-warning Clippy. Preserve
`build/avatar-life-codec-qualified-v1-20261103`. World tests use
trusted direct life damage to isolate the projection and actual World respawn
barrier; they do not qualify client attacks, encrypted replication or rendering.

## Normal acceptance (qualified below)

Use the bounded reliable critical presentation channel with validated readiness,
initial full roster and changed lifecycle/membership publication. Preserve legacy
channel modes. Reject uploads, retire slow peers and fail full/closed bounded
presentation queues without silently dropping lifecycle transitions. Qualify
actual encrypted two-peer death/respawn/leave/rejoin under impairment at
20/30/60 Hz, with unchanged observer vitals and stale frames unable to restore
an old life.

Then adopt the existing standard Bevy Death and fresh-life animation reset for
remote actors, using exact actor/life identity, with no camera/target/facing aid.
Version normal bundle/CLI identities explicitly. Native acceptance needs two
actual normal accounts: the observer sees the victim die and recover while its
own health stays private/intact. Recheck owner movement/cast suppression, normal
quest completion and durable rejoin. No adoption claim before those gates pass.

## Qualified optional transport

The avatar and NPC contracts share bounded reliable ordered channel 2 ahead of
ordinary events/poses. Legacy two-channel modes and the original NPC-only entry
point remain available. Both publishers wait for accepted readiness after Welcome;
AV01 sends an initial full roster, then changed membership/lifecycle only. Each
16-event client queue is explicit, with profile/rate/owner checks before state
publication and preserved state through movement snapshots. A full server queue
retires the affected peer; a full/closed client queue fails before receive-state
commit. Client AV01 uploads on either critical or event channels are invalid.
No awaited socket/database I/O is introduced into the simulation step.

Actual encrypted death/respawn/leave/rejoin at 20/30/60 Hz passes under 20 ms delay,
30 ms jitter, packet loss, duplication and observed reordering. Observer vitals
and incoming windups stay private/intact. A separate actual two-peer scenario
passes avatar rosters and NPC victim-transfer windups together on the shared
channel at all three rates. Boundary cases cover readiness, server/client full
queues, forged uploads on both channels and mismatched public profile refusal
before ticket parsing/socket I/O. Existing NPC-only modes remain in regression.

The first unrestricted library regression had one Welcome timeout when replacing
a session immediately through the same relay backend endpoint. The fixture now
uses a distinct endpoint per session, as the existing reconnect qualification
does and a newly bound native UDP socket normally does, then injects captured old
encrypted packets into the replacement path. Raw packets remain bounded/in-memory
and are neither logged nor saved. The unchanged five-second Welcome deadline is
retained. The updated full 340-test library regression passes; the complete 670-test network regression (ten service-dependent skips), formatting
and workspace/all-target denied-warning Clippy pass. Preserve
`build/avatar-life-runtime-qualified-v1-20261103`. This does not qualify immediate reuse of
the same backend endpoint at that checkpoint. The subsequent
[connection-confirmation fix](RENET_CONNECTION_CONFIRMATION.md) reproduces the
underlying lost-initial-KeepAlive failure independently of endpoint reuse and
records its separate qualification; earlier checkpoints remain unchanged.

These are actual encrypted sockets/World life with trusted development damage/
timing profiles and synthetic ports, not normal OIDC/SQL/native presentation.
No CLI/bundle or renderer adoption is claimed for AV01 yet. Preserve the qualified
normal schema-14 public-NPC playable and the AV01 codec checkpoint.

## Qualified normal adoption

The explicit schema-15 `development-gameplay-classic-public-life.json` bundle
retains the same 22 exact inputs and enables both public NPC timing and public
avatar life on the normal binaries. The derived avatar identity follows NPC
timing before outer reward/quest/dialogue identities. Earlier bundle versions
retain their meanings; the default bare launcher does not implicitly enable it.

Remote standard Bevy visuals now use public life identity/dead status, reuse the
existing Death clip and fresh-life reset, and invalidate prior sword cues. Local
visuals continue using owner-private state. Remote pose interpolation resets only
for an initial/fresh public life, waiting for a movement snapshot at/after the
roster tick; normal same-life samples retain their blending. No new movement
protocol, player targeting, camera behavior or facing assistance is introduced.
Bounded lifecycle events are drained with critical combat presentation. Native
instrumentation distinguishes remote death starts, actual changing Death bones,
fresh-life restarts and pose resets without exposing another player's health.

All 131 client, 47 gameplay-validator and 35 Python launch checks pass, as do
formatting and workspace/all-target denied-warning Clippy. Optimized builds and owned normal two-account native death/fresh-life
qualification pass below. Preserve the qualified schema-14 playable and AV01
runtime checkpoint as well as the new normal checkpoint; changes remain local.

## Qualified native and final regression

`build/avatar-life-native-v5-20261103` uses the actual normal schema-15 launcher,
two owned OIDC accounts/PostgreSQL and original wolf settings. The observer is
32.64 m away outside perception, stays at 100 HP with zero private incoming
windups and no selected target. A valid equipped-wand cast is verified while the
owner is alive. The wolf then kills it naturally; dead W/ability input produces
no horizontal movement/new cast. The observer plays the actual standard Death
clip (52 changing bone transforms), holds the completed corpse pose in view,
then sees generation 1 -> 2, fresh-life animation and pose reset. Screenshots
and projected remote-player viewport checks confirm framing; this is not merely
an off-screen animation counter.

Native 1920x1200 on Linux AMD RX 7900 XTX / RADV Vulkan / Mesa 26.2.6, on the
connected 3440x1440 display; synthetic PID-bound X11 input. GPU diagnostics: zero.
Owned services/password files are cleaned. No Windows, population or Classic
corpse-run/resurrection-system claim; the existing three-second development
respawn policy is unchanged.

Earlier harness attempts remain recorded: v1/v2 stopped at the ownership guard
during overlapping-window activation, corrected by raising the exact owned
frame and requesting WM activation with wmctrl. Pointer/focus guards remain.
v3 validated animation state while off screen; v4 added projected visibility;
v5 captures the finished corpse pose as well. No gameplay rule was relaxed to
pass. Normal quest/database-restart regression now also passes; all 1,123 workspace tests pass (50 explicit service skips), alongside formatting
and workspace/all-target denied-warning Clippy. Preserve
`build/avatar-life-client-qualified-v1-20261103`.

`build/avatar-life-journal-native-v1-20261103` passes normal unmodified-wolf
semantic quest interactions, two completed quests, three kills, four durable
actions, head 13 and 125 XP; database process restart/rejoin preserves exactly
that state without duplicated progress/reward. Fresh damage threat accounts for
26 resolved facts, 300 actual HP lost and three clears; rejoin adds none. Both
runs have zero GPU diagnostics. The strengthened two-account v5 check requires
the Death clip to finish while the corpse remains in view, and also passes with
zero GPU diagnostics. Owned services/password files are cleaned after each run.
Final workspace qualification passes all 1,123 tests (50 skips). No native
Windows or population-capacity claim.

The normal checkpoint freezes exact source/binary hashes, optimized client/server/
validator, existing player/wolf art, full workspace and component logs, and only
whitelisted native screenshots/state/proof. Provider/database/password directories
are excluded. Three final native runs are v5's two-account lifecycle scenario and
normal quest fresh/rejoin. Earlier v3/v4 diagnostic runs are not substituted for
the finished-corpse/in-view acceptance. All work is local, uncommitted/unpushed.

With normal account/PostgreSQL configuration and prepared content, select the
qualified version explicitly:

```sh
python3 tools/play.py --classic --account-login \
  --gameplay-manifest content/development-gameplay-classic-public-life.json \
  --binaries build/avatar-life-client-qualified-v1-20261103 \
  --players build/avatar-life-client-qualified-v1-20261103/art/player \
  --creature-asset build/avatar-life-client-qualified-v1-20261103/art/wolf/wolf.glb
```

Remaining: immediate same-backend-endpoint reconnect investigation, broader NPC/
region interest management, larger actor populations, class/equipment animation
coverage and production death/resurrection rules. This slice changes visual life
replication, not those systems or durable player health.
