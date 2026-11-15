# Party membership — P4-11

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
display names, without account IDs, positions or stats. Membership and roster
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

Native `build/party-ui-native-v1/qualification.json` passes on two real OIDC
accounts at 1920x1440 each, then the same characters after PostgreSQL/server
restart: 13 accepted UI commands per phase, zero refusals/invalid uploads, exact
unchanged journal/bag/XP and fresh solo membership on rejoin. Source hashes and
SQL checks pass; 140.52 seconds total. Invitation/member screenshots were visually
inspected. Linux RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6, zero GPU
diagnostics; no Windows execution. 281 client tests (four explicit skips), build,
format, strict client/server Clippy and 53 launcher tests (one explicit skip) pass.
Editor account-launch adoption follows this opt-in acceptance.

Shared XP, loot, tags, quest credit and escort journeys remain separate slices.
The user's later party-escort rule is any participating member in range keeps
the shared ranger moving. Existing private solo quests remain unchanged until
membership is integrated and that explicit eligibility/credit slice passes.
