# Local chat — bounded foundation

8 November 2026. P4-12 / P2-14 / P5-13. The user explicitly identified in-game
chat, local-area chat and overhead speech bubbles in the supplied references.
Implement this incrementally without copying captured player messages into
fixtures. The user restored mouse, keyboard and display access later on8 November.
Continue implementation and local commits; no push. Earlier headless-only
checkpoints below retain their original evidence scope.

## Initial policy

Use server-owned admitted actor identity and the selected saved name. A request
contains only its sequence, channel and validated text. It cannot choose a sender,
position, area or recipient list. Bind delivery to the current world incarnation;
keep messages out of movement snapshots and do not require a nearby actor to be
in the recipient's movement roster just to receive chat.

The initial channels are Say and Area. Say reaches listeners within 25 metres
inclusive in 3-D global f64 coordinates. Area reaches listeners with the same
nonzero server-authored chat-area ID. This ID is separate from rendering chunks
and simulation regions. Missing area membership refuses an Area submission.
The 25-metre range and three accepted messages per sliding five-second window
are explicit provisional Theia choices, not evidenced Classic formula claims.
Both channels share one bounded per-incarnation budget. Server ticks at 20/30/60 Hz
produce the same real-time interval; client timestamps never refill the budget.

Text is one plain Unicode line, 1–256 UTF-8 bytes with non-whitespace content (zero-width space alone is empty). Preserve
spelling/spacing and do not truncate or interpret markup. Reject control/line
characters, directional override/isolate marks and a byte-order mark. Emoji
joiners remain supported. This is a temporary input policy, not complete Unicode
confusable filtering or content moderation. Debug output reports text byte length
without copying message content into diagnostics.

Consume valid new request sequences even when muted, rate limited or without
area membership, so those refusals cannot be replayed later. Reject stale/zero/
more-than-128-ahead sequences, wrong actor incarnation and regressing/zero server
time. Mute is a server-owned submission decision; listener block results suppress
delivery. Account-wide moderation and reconnect flood protection must be supplied
outside the per-incarnation gate before a public release.

## Implemented prerequisite

`theia_core::chat` now contains the validated text, intent, trusted context,
delivery policy and fixed-memory submission gate. It has no engine, transport,
SQL or external-service dependency. Tests cover multibyte bounds/controls,
plain Unicode and emoji, far-origin metric radius including vertical/boundary
cases, cross-epoch/blocked recipients, Area isolation, shared-channel budget at
all supported tick rates, mute/replay/time/identity refusal and u64 exhaustion.
This standalone policy is not working multiplayer chat or moderation tooling.

Actual prerequisites for the next slice are authorized selected-name admission,
checked server positions and bounded reliable Renet events; parties and group
encounters are not prerequisites for Say. The standalone CQ01/CD01 request/delivery codec is implemented: requests are
at most 279 bytes; deliveries at most 348 bytes with a validated saved name and
world/incarnation context. Exact lengths, tags, channels, UTF-8, text bounds,
identity fields and admitted world are checked. No runtime publisher, receiver,
protocol-version change or gameplay compatibility change is enabled yet.
Headless encrypted routing checks must pass before enabling that path.
Use bounded independent queues/history, no awaited I/O in simulation, no message
body logging, and an explicit opt-in compatibility profile until the path passes.

Client acceptance remains pending: a focused plain-text composer must consume
movement/ability/menu input while typing; Enter submits, Escape cancels, and
focus loss discards unsubmitted input. Show server echo only after acceptance.
Keep channel labels and sender names separate from message text. Bound history,
scrolling and speech-bubble count/lifetime; bubbles apply to Say only and never
create gameplay targets or reveal hidden actors. Native qualification is deferred
under the user's desktop prohibition. Do not add inert chat controls meanwhile.


Headless evidence: 378 complete core cases and two focused wire cases pass;
strict workspace/all-target Clippy and formatting pass. Artifacts:
`build/local-chat-20261108/core-v1.log`, `wire-v1.log`, `clippy-v2.log` and
`fmt-v1.log`. No desktop, OS input or renderer was used. Core policy and codec
checks establish no live chat, speech bubbles, moderation tools or scale claim.


## Bounded host router prerequisite

`theia_network::chat::Router` now owns up to eight admitted speakers, separate
server-updated checked positions/Area membership, submission gates and sixteen
queued deliveries per recipient. Admission never accepts a client display name
or position; the host must bind its caller key to a live authorized connection.
Duplicate admission refuses instead of resetting flood state. Submission uses
the current server contexts and stamps the saved sender name. Recipient blocking
also purges that sender's unsent messages. Retiring a recipient drops its queue.

The audience is captured at acceptance time. Queued Say does not get rerouted
when either actor later moves. A full outbox drops the newest message; a transport
capacity refusal retains the oldest. Flush attempts at most one bounded delivery
per peer and uses a synchronous capacity/send callback after gameplay-priority
publishing. Counters saturate and carry no message text. Eight full outboxes hold
at most128 deliveries (44,544 encoded bytes, excluding container/allocator overhead).
This policy does not guarantee delivery or acceptance echo when a recipient is
slow. A future composer requires explicit submission replies to report refusals.

Three focused routing tests pass: real codec frames, proximity/area/authoritative
name stamping, pose updates, mute/replay/flood behavior, block queue purge,
retirement, bounded admission, full-queue drops and deferred ordered flushing.
Strict workspace/all-target Clippy and formatting pass. Artifacts:
`build/local-chat-20261108/router-v1.log`, `router-clippy-v1.log` and
`router-fmt-v1.log`. This router is not installed in the Renet owner loop and has
no authenticated encrypted routing evidence yet. No normal executable, protocol,
rendering or input path changed. Next implement explicit opt-in compatibility,
lease/pose synchronization and encrypted loopback acceptance before normal use.


## Opt-in encrypted owner-loop candidate — 8 November 2026

`World::enable_local_chat` now enables a candidate one-area local world before
actors join. Its compatibility identity hashes the prior complete world identity,
area ID and initial policy/codec version. Default worlds retain their exact prior
identity; this adds no protocol13 change. The normal executable and launcher do
not enable this mode. An anonymous owner loop refuses chat-enabled startup.

The existing Renet owner loop synchronizes live authorized named leases and
checked world poses into the router before requests and after peer retirement.
Lease errors fail authority; revoked/unnamed/retired peers cannot remain chat
recipients. The initial world supplies one configured area for everyone, rather
than deriving zones from chunks. Routing stays outside World::step and performs
no SQL, provider calls or awaited transport work. Flush follows gameplay and
identity publishing. Existing eight-event/poll and180-event/second limits bound
chat parsing too. Malformed uploads, disabled-mode requests and forged server
deliveries disconnect the uploading peer; valid policy refusals do not.

Actual encrypted loopback with three named synthetic authorized leases passes:
nearby Say, distant Area-only delivery, saved-name stamping, three accepted
messages/seven recipient deliveries and two flood/replay refusals. Separate
actual encrypted scenarios reject disabled, truncated and forged uploads.
The compatibility/revocation test purges queued text on lease revocation.
These fixtures use in-memory authorization, not OIDC/SQL or native rendering.
Normal login/SQL name authority was qualified separately by the identity slice.

Full network regression `build/local-chat-20261108/network-v1.log` passes738 cases
with ten explicit skips across70 result groups, including the two original runtime
cases. The added three-scenario negative test and final event fixture pass in
`runtime-v6.log` (three focused tests total). Client regression passes231 cases
with one explicit art skip (`client-v1.log`). Strict workspace/all-target Clippy
and formatting pass (`runtime-clippy-v3.log`, `runtime-fmt-v1.log`). Earlier runtime
logs retain diagnostic compile/assertion failures and are not accepted evidence.

This is an opt-in server candidate with raw qualified test clients. The normal
client has no chat receiver/composer yet. Explicit bounded submission replies,
client history/deduplication, typing capture and Say bubbles remain next work.
Mute/block policy functions exist in the router, but no live moderation UI or
persistent account-wide moderation/reconnect flood controls are implemented.
Native drag work can resume after the user's renewed desktop permission; it does
not constitute native chat qualification.


## Private replies and gameplay queue reserve — 8 November 2026

CR01 adds an exact69-byte private submission reply with admitted owner/world,
request sequence, server tick and bounded outcome (accepted, unavailable,
sequence/time refusal, muted, no area or rate limited). No body or public sender
metadata is copied into the reply. The opt-in compatibility identity advances to
candidate v2 and includes the reply tag; default worlds/protocol13 are unchanged.

The owner loop keeps at most four pending replies per named peer, purges them
with retirement/revocation and attempts one reply plus one delivery per peer per
I/O pass after gameplay publishing. Overflow drops new replies with a counter;
it does not widen queues or wait. Normal future UI must permit one outstanding
submission and handle timeout/disconnect rather than assume reply delivery.
Replies go only to their admitted sender. Uploading CR01/CD01 is invalid.

Both reply and delivery capacity checks reserve4096 bytes (half the shared8192
reliable queue) against chat use. Publishing order alone would allow chat to fill
space needed by gameplay on the next simulation step. The reserve supplements
existing critical channels; it is not a guarantee against gameplay's own bounded
queue exhaustion. A synthetic Renet queue-pressure/recovery check proves chat
defers while a gameplay frame can still enqueue and later resumes without peer
retirement. The server does not log text or reply bodies.

Ten focused policy/codec/runtime tests pass in `replies-v3.log`, including actual
encrypted accepted/rate-limited private replies, a deliberate five-request/four-
reply-capacity overflow, replay refusal, disabled/truncated requests and forged
CD01/CR01 uploads. Strict workspace/all-target Clippy and formatting pass in
`replies-clippy-v2.log` and `replies-fmt-v1.log` under
`build/local-chat-20261108`. Earlier full regression/native evidence retains its
original source scope. Client receipt/history/composer and bubbles remain pending.


## Bounded client inbox and world delivery serial — 8 November 2026

`chat_client::Inbox` validates admitted world/incarnation context, retains64
messages and one owner-scoped latest reply, and ignores stale/duplicate delivery
serials. A conflicting latest serial or regressing delivery tick fails before
mutation. Replies have independent ordering, so an older queued chat delivery
can legitimately arrive after a private reply. A stale/replayed reply cannot
overwrite the latest request outcome. Area senders need no movement-roster entry.

The router now assigns a checked monotonic world delivery serial across speakers
and channels. This permits bounded duplicate protection without accumulating a
per-speaker map over a long world lifetime. Accepted messages consume one serial;
refusals do not, and exhaustion refuses further acceptance rather than wrapping.
Global accepted chat time cannot regress. CD02 carries the additional serial;
maximum delivery size is now356 bytes. The opt-in compatibility identity advances
to candidate v3. CQ01 requests and69-byte CR01 replies retain their shapes; normal
protocol13/default worlds are unchanged. Prior CD01/v1/v2 artifacts are historical.

A full64-message inbox holds at most22,784 encoded delivery bytes; eight full
sixteen-message server outboxes hold at most45,568 encoded delivery bytes. These
exclude allocator/container overhead and the independently bounded reply queues.
Text limits and delivery rate continue to bound content allocation.

Fourteen focused tests pass in `build/local-chat-20261108/inbox-v2.log`. New cases
exercise10,000 changing actor IDs, retained history bounds, serial gaps, exact/
stale/conflicting delivery, context rejection, independent reply ordering and
serial/time exhaustion. The actual encrypted three-peer fixture now feeds its
real received frames through Inbox as well as the decoder; nearby/Area delivery
and private rate-limited reply results remain correct. Strict workspace/all-target
Clippy and formatting pass (`inbox-clippy-v1.log`, `inbox-fmt-v1.log`).

This is the receipt/cache prerequisite exercised by raw encrypted fixture clients.
The normal Renet client worker and gameplay State do not use it yet; no composer,
chat window, native chat input or bubbles are claimed. Next wire the bounded
producer/receipt mode into the real client worker with timeout/reply behavior
that cannot disconnect or pause gameplay just because chat is delayed.


## Real client-worker integration — 8 November 2026

`GameplayClient.chat` now explicitly opts the real Renet worker into the source
world/area profile, bounded composition queue and client inbox. Existing callers,
including the normal application and server-side fixture clients, explicitly
retain `None`. Admission validates the combined world identity; Classic movement
validation uses the complete gameplay identity before the chat suffix. Unknown
chat observations remain invalid for clients that did not opt in.

The worker stamps request sequences and permits one outstanding request. Its
maximum-four input queue and reliable-capacity reserve bound queued work. It
discards queued text before admission and after a five-second local reply timeout.
Timeout sets a local view flag without disconnecting, pausing or mutating gameplay.
A late reply cannot clear a newer pending request; sequence exhaustion disables
further chat submissions without wrapping or disconnecting. An explicit fresh
submission can follow timeout; text is never silently resubmitted.

Validated deliveries/private replies publish a shared immutable inbox through
`client::State.chat`. Movement snapshots preserve this view. Transport diagnostics
count requests, deliveries, replies and timeouts without body logging. Rendering,
keyboard handling and the normal executable's profile selection remain unchanged;
no chat controls are exposed yet.

Eighteen focused checks pass in `build/local-chat-20261108/worker-v3.log`. Actual
encrypted production client-worker fixtures exercise both baseline predicted and
Classic movement: nearby peer traffic, Say/Area sending, serialized submissions,
private rate-limit replies and inbox survival across subsequent snapshots. The
server uses synthetic authorized named leases. Deterministic driver checks cover
pre-admission input discard, queue pressure, timeout, stale queued work, late
reply isolation, world binding and exhaustion without gameplay disconnect.
Full client regression passes231 ordinary cases with one explicit licensed-art
skip (`worker-client-v1.log`); strict workspace/all-target denied-warning Clippy
passes (`worker-clippy-v2.log`), as does formatting (`worker-fmt-v2.log`). Full
network regression is recorded separately with its exact final counts.

The user asked to stop after this slice. The safe boundary is this client-worker
checkpoint with local commits only. Next work, after an explicit resume: normal
launcher/profile opt-in, focused composer with gameplay-key consumption, bounded
chat HUD/history controls and Say speech bubbles with visibility/lifetime caps,
then normal-account/SQL and native qualification. Those parts are not implemented
by the worker tests or by the earlier native action-bar proof.

Final full network regression: 747 passing cases, 10 explicit skips across
70 result groups in `worker-network-v1.log`. Its library run contains404
cases; the final Classic-worker and extra queue/exhaustion additions are covered
by the subsequent18-test `worker-v3.log`. No production behavior changed after
that full run's source except formatting. All checks pass; changes are local.

## Native composer and normal account profile — 23 November 2026

P4-12 / P2-14. The user resumed autonomous implementation and authorized desktop
input, local commits and continued slices. The current change enables explicit
`--chat-area <nonzero u32>` selection in the client, server, `tools/play.py` and
`tools/account_population_playtest.py`. The launcher requires Classic account
admission. Both endpoints wrap the complete Classic gameplay identity with the
chat profile before the optional party profile; prediction uses the same ordering.
This remains one server-authored area for the whole local world. It does not infer
membership from render chunks or implement cross-zone area changes. Defaults
without the flag retain their previous compatibility identity.

The General tab now has a standard Bevy editable field, Say/Area switch, Send
button and private submission status when chat is enabled. Enter opens/submits,
Escape cancels, and losing focus discards the unsubmitted draft. Gameplay keys
are consumed while composing, including menu/ability shortcuts; held keys stay
suppressed until released. Classic movement is also disabled during composition,
including mouse-button forward movement. Input is bounded to 256 characters;
the existing stricter 256 UTF-8 byte/plain-line policy validates before queueing,
without truncation or markup parsing. Multibyte text may therefore hit the byte
limit before the character limit.

A capacity-one producer queue and local submission latch bridge the asynchronous
worker view. Only the existing worker assigns sequences. The HUD never appends an
optimistic message: its at-most-64 messages come from the validated server inbox.
Refusals and unknown-delivery timeouts are separate from history, and nothing
resubmits automatically. The scroll area follows the latest line until the player
scrolls upward; scrolling back to the bottom resumes following. Disabled profiles
retain the prior unavailable composer. Diagnostics expose counts, channel, saved
name and text length, never draft/message bodies.

Validation: 355 client tests pass (five existing skips), 38 Python launcher tests,
five actual server CLI refusal cases, formatting, denied-warning all-target Clippy
and matched release client/server builds. Evidence is under
`build/chat-composer-20261123`: `client-tests-entry-fix.log`, `launcher-tests.log`,
`server-cli.json`, `clippy-bounded-input.log` and `build-bounded-input.log`.

The final focused native run `native-v5/qualification.json` passes normal password
login/SQL character admission, real encrypted server echo for Say/Area, saved-name
stamping, rate refusal without optimistic echo, typing/input ownership, held-key
release, Escape, window-focus loss and long-draft clipping. Captured screenshots
were inspected. Zero GPU diagnostics on Linux AMD Radeon RX 7900 XTX / RADV NAVI31 /
Mesa 26.2.7 / Vulkan at 1920×1080. This is one normal account against the actual
local server; earlier multi-peer routing evidence retains its synthetic-lease
scope. No public moderation, account-wide reconnect flood control, speech bubbles,
Windows execution or MMO capacity is claimed.

The longer frontend run exposed an authored-scene readiness timer counting time
spent in character menus before world admission. Its deadline now starts only
when the scene is required; real preparation remains bounded to 120 seconds.
The deadline regression and subsequent native entry after extended menu use pass.
Native v1–v4 are incomplete diagnostic attempts, not final qualification: they
exposed that timer, a fixture name-case expectation, moving composer controls and
an X11 focus-loss fixture issue. The final composer reserves status space and
constrains its editable width, so acknowledgements and long drafts cannot move
its controls.

User-directed next slice: return to character customisation and closely follow
the supplied reference layout, with one category navigation (no duplicate tabs),
a central live character, adjacent appearance controls and bottom actions. Keep
presets omitted, retain drag rotation/foot arrows/zoom, and disable unsupported
choices. This supersedes the earlier request to retain only the compact layout.

## Say speech bubbles — implemented and locally qualified

P4-12/P2-14, 23 November 2026. The existing verified Say inbox now drives a
bounded native Bevy overlay. Area messages never produce bubbles. One bubble per
speaker replaces their previous message; at most eight remain, each for six real
seconds. These count/lifetime choices are provisional presentation policy, not
Classic formula claims. Duplicate delivery observation cannot extend a lifetime.

Bubbles require an already instantiated, visible player presentation, membership
in the current authoritative movement snapshot, an on-screen head anchor and an
unobstructed camera ray using the existing solid/terrain occlusion query. Missing,
hidden, off-screen, occluded or retired actors produce no bubble. Losing that
visibility drops the bubble; returning cannot replay it from history. Disconnect,
world/owner changes and first attachment discard prior history. Chat history and
server delivery are unaffected by this cosmetic filtering.

The overlay uses the shared navy/gold palette, plain wrapped white text and a
small pointer. It has no clickable target or gameplay component. Text remains the
validated original Unicode line; no markup interpretation or message-body logging
is added. Opt-in evidence reports only count, actor ID, serial and text byte length.
The native chat qualifier accepts an explicitly chosen display and software Vulkan
for isolated tests without taking over the user's open game.

Acceptance: 361 client tests pass (five ignored), including initial-history
suppression, duplicate/expiry handling, per-speaker replacement, capacity,
visibility and world/epoch rejection. Release build, formatting and denied-warning
Clippy pass. `build/bubbles-native-v2/qualification.json` verifies normal account
login/character creation/world admission, authenticated Say overhead text, Area
exclusion, six-second expiry and existing composer/input/rate-limit checks.
Screenshots inspected. Linux Vulkan on llvmpipe LLVM 21.1.8 / Mesa 26.2.7, zero GPU
diagnostics. V1 reached bubble checks but failed a focus-loss fixture; V2 uses an
owned peer window instead of requesting no focus under a window manager.

This is one native account and local server, not proof of remote-player bubble
presentation, discrete-GPU performance, Windows or MMO capacity. Two native
accounts/remote presentation are the next bounded qualification slice.


## Two native accounts and late model attachment

P4-12/P2-08/P2-09, 23 November 2026. The `--peer-check` extension starts a
second native frontend against the same owned server and authenticates a second
account through the in-game form. Each saves a distinct body choice, enters via
normal account admission and moves through ordinary authoritative input. It
checks Say delivery/presentation in both directions, Area delivery without a
bubble, long unbroken text and removal on actor retirement before normal expiry.
Message bodies and credentials are not added to result telemetry.

The first diagnostic attempt lacked the second bridge's provider setting. The
second reached admission and exposed a real late-join failure: reference visuals
used the asset catalogue's startup timer for scene instantiation, immediately
failing a new remote model after the client had run for over a minute. Visuals now
start their own bounded timer; see REFERENCE_SCENE.md. The regression also proves
a genuinely stalled instance retains a timeout.

Final `build/bubbles-peers-v3/qualification.json` passes with the first client
already 64 seconds old before peer launch. Both saved body choices appear in both
clients; Say works in both directions, Area remains history-only, a 200-byte
unbroken message wraps inside its bubble, and retiring the sender removes that
bubble 2.30 seconds after submission (before its six-second expiry) while history
remains. 362 client tests (five skips), formatting, strict Clippy and release build
pass. Inspected native screenshots use Linux Vulkan, llvmpipe LLVM 21.1.8 /
Mesa 26.2.7; zero GPU diagnostics in either client. No discrete-GPU performance,
Windows, public deployment or MMO-capacity claim is made.

Visual follow-up: that long message still escapes the separate history panel,
and an unfocused connected client temporarily displays “Chat unavailable”. Those
are the next focused UI corrections; the peer qualification does not claim they
are fixed.
