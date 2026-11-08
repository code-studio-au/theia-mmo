# Local chat — bounded foundation

8 November 2026. P4-12 / P2-14 / P5-13. The user explicitly identified in-game
chat, local-area chat and overhead speech bubbles in the supplied references.
Implement this incrementally without copying captured player messages into
fixtures. Current instruction permits headless development and local commits;
mouse, keyboard and display access remain prohibited.

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
