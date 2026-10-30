Vendor continuation, 2026-10-30: the opt-in guide barter now passes actual
OIDC/native purchasing at 1280×720, 1920×1200 and 3440×1440. It uses the shared
item response rather than treating a dialogue-node reply as purchase success.
The informational-only profile and its frozen evidence below remain preserved.
[Qualified barter behavior and proof](VENDOR_BARTER_FOUNDATION.md).

# NPC dialogue and vendors — initial P3-12 informational slice

Status, 2026-10-30: the initial live informational guide/dialogue slice is
qualified. 418 workspace tests pass with 27 explicit skips; all seventeen SQL
contracts/twenty-three commands separately pass. Two actual OIDC accounts and
native encrypted clients pass keyboard/picking, private branches, closure,
reconnect, database/server restart and backend loss at 1280×720, 1920×1200 and
3440×1440 fullscreen. Exact source, executables, documentation and proof are frozen
at `build/dialogue-qualified-v1-20261030`. Vendors, quest acceptance/turn-in,
production NPC art, wall/occlusion checks and real player-death closure remain open.
Names/text are development fixtures, separate from story edits; P3-17/18 remain
unspecified. All changes remain local/uncommitted.

## Initial scope and actual prerequisites

Start with one stationary, physically nonblocking development guide and a short
informational dialogue. Reuse the authoritative world/entity identities, checked
f64 global positions, bounded encrypted commands, independent client rendering and
standard Bevy UI. Keep the guide separate from attackable creature health and
kill credit. Dialogue does not accept, reset, complete or reward the automatically
initialized development objective.

First qualify a pure bounded dialogue profile, then server interaction/range
checks and private observations, then the native Bevy panel. A text conversation
has no persistent mutation and needs no database worker or new service. Vendors
remain part of P3-12's broader scope, with durable exchange rules reviewed after
this informational interaction works. Neither a read-only catalogue nor an open
panel establishes working purchasing.

## Proposed contracts

- One immutable schema-1 profile, at most 16 KiB, with stable NPC/dialogue/node/
  choice IDs. NPC/display names are at most 64 UTF-8 bytes, node text 512 bytes and
  choice labels 96 bytes; reject blank/control-bearing strings. Bound the graph
  to eight nodes and four choices per node. Reject duplicate definitions, unknown
  destinations, unreachable nodes and cycles for this initial finite dialogue.
  Terminal nodes have no choices; the UI always permits closing.
- Pure choice resolution validates the current node and its offered choice. It
  returns a new node without modifying the source profile or granting gameplay
  effects. Bind exact profile bytes at the runtime content boundary.
- Placement is authoritative and uses checked world coordinates. A request names
  the current NPC entity key, never an arbitrary position, text or character UUID.
  Opening requires the authenticated own live actor within a three-metre 3D range.
  Range checking alone is not evidence of wall/occlusion checking.
- Keep one ephemeral dialogue session per player, bound to current world/NPC,
  actor, profile and monotonically checked session/step identities. Reject foreign,
  stale, replayed or unoffered choices. Death, departure, range loss, disconnect
  and world replacement close it. No saved dialogue state is implied.
- Bound incoming work and replies using the existing runtime budgets. Never await
  transport or database work in the simulation step. Publish accepted current-node
  state privately; a client request does not establish authoritative acceptance.
- Use a standard Bevy proximity prompt and panel with actual buttons plus keyboard
  interaction. Render the observed node/choices, not client-chosen script state.
  Show closure or loading consistently and prevent an obsolete session's reply
  from reopening a closed/replaced panel. Preserve 1280×720 regression usability,
  the 1920×1200 development window and 3440×1440 fullscreen anchors.

## Concrete acceptance before qualification

1. Core definition limits, invalid IDs/graphs/text, missing destinations, immutable
   branching/terminal resolution and unknown/unoffered choice rejection pass.
2. Headless authority proves exact/in/out-of-range requests, stale incarnation,
   foreign actor, dead actor, repeated/out-of-order choices, closure and bounded
   work. NPCs have no collision body and cannot accidentally earn kill credit.
3. Encrypted integration proves each client's independent dialogue, malformed/
   oversized request rejection, private observations and no effect on XP,
   objectives, item revisions or fixed-step I/O.
4. Actual native Linux clients open, choose and close the observed dialogue;
   walking away closes it. Inspect the real widgets and input paths at the
   development, small regression and ultrawide sizes with exact GPU diagnostics.
5. Formatting, denied-warning Clippy and appropriate regressions pass. Preserve
   the prior qualified game until a replacement source/binary/proof checkpoint
   exists. Report native Windows separately and keep changes local.

## Later vendor slice

Review item/cost catalogue binding, available funds or a specifically defined
barter policy, bounded quantities, overflow/full-bag behavior and idempotent
transaction identity before implementation. The existing durable backpack owner
and retirement fence are prerequisites. Commit cost removal and purchased items
atomically before success acknowledgement; stale quotes, repeated requests,
dropped answers, crashes and restart must not duplicate value. Do not invent a
production currency balance, unlimited free grants or quest rewards as a shortcut.

## Qualified pure-content prerequisite

`theia_core::dialogue` checks a schema-1 profile before exposing read-only nodes
and choices. All profile/NPC/node/choice definition IDs are globally distinct
within the profile. Branch references may share a terminal node. Every node must
be reachable from the start, and cycles reject; this yields at most eight nodes
on a path. Text limits count UTF-8 bytes without truncation, and parsing rejects
unknown fields and duplicate object keys. Pure branch lookup accepts only a choice
offered by the supplied current node; it changes no source content and grants no
interaction authority or effect. There is no mutable session or counter here.

`content/gameplay/development-dialogue.json` is a four-node informational fixture
about existing movement and training controls. It accepts/grants no quest, item,
XP or currency. A terminal node simply has no choices; closing remains a runtime
UI responsibility. Graph limits and finite navigation do not establish network
request rate limits or server session validation.

`build/dialogue-core-tests-v1.log` records 36 passing focused tests: six dialogue
checks and thirty objective/XP/equipment/backpack/reward regressions.
`build/dialogue-core-fmt-v1.log` and
`build/dialogue-core-workspace-clippy-v1.log` record successful formatting and
denied-warning workspace/all-target Clippy. No rendering changed and no new
full-workspace/native dialogue result is claimed. Exact prerequisite source,
non-story documentation, the executed dialogue test binary and proof hashes are
frozen in `build/dialogue-core-qualified-v1-20261030`. All changes remain local.

## Qualified headless interaction prerequisite

`theia_network::dialogue` binds exact profile bytes and one trusted supported
placement into the opt-in world's identity. The guide has its own entity key,
is not an attackable target or fake player, and constructs no physics body.
Preparation is allowed before admission/time advance only; invalid/out-of-bounds/
unprepared placement rejects. Retirement closes sessions while retaining the
bound world identity. World replacement gets a new epoch.

The owner-side `World::interact` API accepts the actor from its caller, never from
the request. This API is not a network endpoint. Existing admitted players can
open the current guide within an inclusive three-metre 3D radius, choose only an
offered current-node choice, and close. One ephemeral state per player records
strictly increasing positive command sequence and current view revision. Invalid
commands consume the sequence/budget; stale/unoffered choices cannot advance the
view. Eight attempts per fixed one-second simulation bucket are allowed; this is
not a sliding-window limiter. No persistent gameplay value changes.

Walkaway closes on the fixed step; guide retirement closes immediately; departure
removes the session. Closure advances the view revision so a delayed old node
can be rejected by a future client. The final u64 revision is reserved for closure;
positive mutations fail rather than wrapping or changing contents at one revision.
Views carry each own actor/profile; encrypted publication, framing, backpressure,
client succession validation and loading/closure presentation remain open.

Current player health presentation reports maximum health; authoritative player
damage/death is not implemented. Presence in the admitted actor set is the current
live-actor condition. Death-driven closure must be integrated and tested when real
player death authority exists; no mocked health flag or death claim was introduced.
Range is geometric only, with no wall/occlusion guarantee.

`build/dialogue-authority-regressions-v1.log` records 57 passes: 49 network unit
tests (six new dialogue checks), four targeting tests and four XP/objective wire
tests. Actual headless static-controller movement crosses the guide at 20/30/60 Hz
without collision. Tests also cover exact/vertical/diagonal range, stale NPC epoch,
invalid actor, independent branches, repeated/out-of-order/unoffered choices,
walkaway/retirement/disconnect, budgets and revision exhaustion. The guide stays
out of enabled combat's health targets. `build/dialogue-authority-fmt-v1.log` and
`build/dialogue-authority-clippy-v1.log` pass. Exact source, non-story docs, the
executed network test binary and proofs are frozen at
`build/dialogue-authority-qualified-v1-20261030`. No full-workspace/native/encrypted
dialogue qualification is claimed. The previous playable checkpoint stays intact.

## Qualified encrypted runtime prerequisite

`DI01` requests are at most 126 bytes and bind world epoch, positive request
sequence, expected view revision and a bounded action. They contain no actor UUID,
position, text or reward command. `DV01` views are at most 174 bytes; node IDs must
exist in the bound profile. `DG01` presence is at most 93 bytes and carries checked
f64 global coordinates. Drawing uses the existing wider XZ interest area, while
interaction permission remains three-dimensional/three metres. All observations
are private to an admitted actor and exact profile. Presence relies on the existing
reliable ordered channel, not an invented movement prediction or persisted state.

`DR01` is a 69-byte independent acknowledgement: own actor/profile, request sequence,
required view revision and accepted/rejected outcome. Refusing a choice can retain
the old node/revision while still acknowledging that request. The server queues
the current view before its reply on the same ordered channel. The client clears
a pending action only after both its matching reply and at least that view revision
arrive; acknowledgement never predicts a node change. Regressed/aliased/foreign
views or replies, unsolicited replies, malformed frames and live guide identity/
position changes fail closed.

The optional runtime client uses a four-action channel, one outstanding command,
150 ms minimum send spacing and an eight-second pending deadline. Initial view
and presence have an eight-second deadline too. Peer output is coalesced and uses
the existing bounded event channel; unavailable capacity disconnects the slow
peer. Malformed requests or the authoritative fixed-time request budget also fail
the connection closed. Dialogue fields survive movement snapshot replacement in
one watch publication; ordinary movement does not briefly clear the journal.

`build/dialogue-encrypted-tests-v1.log` passes one real secure Renet two-client
loopback flow: independent branches, rejected unoffered/stale requests with
unchanged-node acknowledgement, explicit close/reopen and actual fixed-step
walkaway closure. It uses trusted locally issued tickets, without OIDC, database,
mocked connections or rendering. `build/dialogue-wire-tests-v2.log` records 61
network/targeting/wire passes; final four framing tests after an allocation-only
Clippy correction pass in `build/dialogue-wire-tests-v3.log`.
`build/dialogue-wire-clippy-v2.log` passes workspace denied-warning Clippy. Expanded
qualification passes below; no native dialogue checkpoint is claimed yet. No new gameplay success/value is acknowledged by dialogue.

`build/dialogue-runtime-tests-v2.log` records 75 passes/zero failures/zero skips:
51 network unit tests, three admitted-account boundary tests, seven actual
encrypted combat/status tests, two actual encrypted dialogue tests, four dialogue
framing tests, four XP/objective wire tests and four targeting tests. Dialogue
uses real secure Renet with trusted local tickets; this is not OIDC or database
qualification. One actual peer selects while another stays on a different branch;
unoffered/stale choices acknowledge refusal with unchanged node, close/reopen is
checked, and actual movement closes on walkaway while the guide remains visible.
Truncated and oversized requests from two actual encrypted raw peers disconnect
without advancing dialogue. Unit checks separately prove four-action capacity,
missing-initial/pending deadlines and receipt-before-view waiting.

`build/dialogue-runtime-fmt-v1.log` and
`build/dialogue-runtime-clippy-v2.log` pass formatting and denied-warning
workspace/all-target Clippy. No rendering changed, and no new full-workspace or
native dialogue result is claimed. `build/dialogue-encrypted-qualified-v1-20261030`
preserves exact source, non-story Markdown, three executed test binaries and proof
hashes. The earlier expanded run failed because its negative-test driver unwrapped
the expected netcode disconnect while sending; the corrected driver accepts that
disconnect and still requires the server's exact malformed-rejection counters.
Only the final passing run is used for this checkpoint. All services/ticket files
belong to their tests and are stopped/removed. All changes remain local.

## Qualified standard Bevy client

The normal launcher opts in with `python3 tools/play.py --dialogue`. The default
fixed guide placement is `8004,8000` metres, independent of the player default
spawn; `--guide-position x,z` selects an explicit fixture placement. Server/client
load the same bounded `--dialogue-catalog`. Startup prepares its supporting
terrain using the existing bounded loader before opening endpoints or advancing
time; no I/O was added to the simulation. A far-guide startup test proves this
works independently of the player spawn.

The guide is a cyan greybox mesh using Bevy's existing pawn mesh/material path,
without an Avian body. It uses checked global positions and the existing render
frame conversion, and hides when its terrain is not client-ready. The POLYGON
animated development player is unchanged. The panel uses standard Bevy nodes,
buttons, wrapping text and a scrollable body with an always-accessible close
button. Keyboard controls are E to talk, F1–F4 for offered choices and Esc to close.
Esc closes the conversation without exiting; the camera wheel does not zoom while
the panel is open. Movement remains available and walking away closes the session.

A widget activation captures the choice ID and revision from its displayed model.
It is not reinterpreted against a newer network node when consumed. A focused
Bevy observer test covers that stale-choice/close binding and ignored activation
while pending or hidden. The server still validates every request. The UI shows
only observed node state and pending/refusal feedback; disconnect hides the guide/
panel, and cached dialogue cannot permanently suppress the ordinary exit shortcut.
No dialogue action changes items, XP, quests or currency.

## Final native and regression evidence

- `build/dialogue-native-1920-v1-20261030/verification.json`,
  `build/dialogue-native-ultrawide-v1-20261030/verification.json` and
  `build/dialogue-native-small-v1-20261030/verification.json` pass with the same
  client and standalone qualification server. Each uses two actual local OIDC
  accounts, PostgreSQL and native encrypted clients. Synthetic native Bevy keyboard
  messages and picking exercise E, actual choice-button activation, F1, Esc, reopen
  and actual walkaway. Each character takes its own distinct movement/training
  branch. Fresh actor sessions start closed at revision zero after reconnect and
  actual database/server restart, including a changed default spawn about 15 km
  away. Committed character poses restore within the existing native tolerance.
  Backend loss fails closed. All item/reward/XP/objective tables remain empty.
- Actual welcome/branch screenshots were inspected at all three sizes. Panels stay
  within the physical window, stable text matches observed nodes, and no guide
  physics body appears. This is the small development fixture; long production
  copy, accessibility, non-default DPI and production NPC art are not qualified.
- Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 / Vulkan was used. Windowed runs
  have no GPU diagnostics. Fullscreen records one or two exact upstream wgpu
  surface-resize diagnostics per startup under the existing narrow policy; other
  errors reject. Physical 4K and native Windows remain unqualified. Physical OS
  mouse/keyboard/browser input is not established by synthetic native delivery.
- `build/dialogue-mixed-launcher-v1-20261030/verification.json` passes the ordinary
  1280×720 CLI with dialogue, Scorch, resources, statuses, creatures, spatial
  feedback, equipment, rewards, XP and objectives. Interrupt, child failure and
  pending-login cancellation retain credential removal, owned-process cleanup
  and accepted final persistence work. This mixed run leaves dialogue closed;
  opening/choosing is checked separately in the focused native fixtures.
- `build/dialogue-workspace-tests-v2.log` and summary record 418 passes/zero failures/
  27 explicit skips across 95 groups, with all 503 captured source files unchanged.
  Seventeen skipped SQL contracts separately execute in
  `build/dialogue-postgres-v1-20261030/verification.json` (twenty-three commands).
  Ten private-asset cases remain skips. The first workspace run also passed, but
  its starting map predates the final UI binding/qualification changes.
- `build/dialogue-final-fmt-v1.log`, `build/dialogue-client-clippy-v4.log` and
  `build/dialogue-python-tests-v1.log` record successful formatting, denied-warning
  workspace/all-target Clippy and nine helper tests. The separately built server
  has no normal renderer/UI/audio/wgpu dependency
  (`build/dialogue-server-dependencies-v1.txt`). The joined preliminary release
  build is not used to establish the standalone headless server.

`build/dialogue-qualified-v1-20261030` preserves exact source, non-story Markdown,
three executed release binaries and proof digests. Earlier content/headless/
encrypted prerequisites and the playable quest checkpoint remain preserved. No
new frame-rate or MMO-population benchmark is claimed. The prior 808.60 FPS result
belongs to the frozen XP/display binary and retains its stated empty-world limits.
Nothing was committed/pushed and concurrent story files were untouched.
