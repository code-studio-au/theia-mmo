# Classic basic-melee transport

P2-12 / P3-06 / P3-08 / P2-14: the authoritative
[weapon-timed melee implementation](CLASSIC_AUTOATTACK_AUTHORITY.md) now has an
explicit encrypted Renet path. Regression qualification passes.
Normal application content/launcher adoption and rendered autoattack remain
pending. The playable checkpoint is still
`build/classic-targeting-qualified-v2-20261101`; the strict combat milestone
remains five of thirteen.

## Contract and implementation

A World enables the immutable melee/item profiles before admitting players.
These bytes already participate in its gameplay identity, inside the outer
classic movement identity. `CombatActions.melee` explicitly configures the
paired client codec and weapon catalog; classic prediction and targeting are
required. Existing callers choose `None` and retain their earlier wire format.
The server's previous blanket refusal of an enabled melee World is replaced by
this explicit path. No untrusted equipment setter is introduced.

`MA01` is exactly 36 bytes: world epoch, movement stream, monotonic request
sequence and optional target ID, after its tag. Zero target means stop. The actor
comes from the authenticated peer. World checks reject stale/foreign contexts
without poisoning live sequences. The client stamps stream identity at input
time, discards stale local choices and keeps request sequences across movement
stream changes. The latest-intent watch mailbox is bounded to one value;
intermediate unsent choices supersede each other. This is engagement intent,
not a queue of attacks or ability casts.

`MS01` is at most 72 bytes and carries owner incarnation, stream, completed tick,
engagement phase/pause reason, optional catalog-bound weapon and ready tick,
swing sequence and historical request acknowledgement. A historical engage ack
can coexist with a currently idle state after death or explicit clearing.
Decoders reject invalid flags, weapon ordinals, unbounded deadlines, self/foreign
targets, ignored acknowledgements, truncation and trailing bytes. Runtime checks
also reject another owner, unsent acknowledgements and regressing observations.
State publication suppresses unchanged views rather than sending every tick.

`CE05` preserves the existing combat event sequence and mixed ability/basic-hit/
death ordering. It adds an explicit weapon-bound melee damage variant to the
bounded event batch. Earlier event codecs reject it; the explicit codec also
rejects earlier tags. Existing catalog-dependent batch counts and the 1,200-byte
frame cap remain. Weapon ordinals use stable content-ID ordering, independent of
JSON array order. Current fixture damage is bounded by the selected weapon's
base damage; modifiers and rich melee outcomes require later versioned work.

Engage/cancel shares the existing per-peer limit of ten combat requests per
control period, plus the overall control budget. Simulation does not await
transport or inventory I/O. Gear comes from the existing committed inventory
observation. Reliable send-capacity exhaustion retires a slow peer. A full or
closed client presentation queue disconnects instead of silently dropping
melee damage/death events. Dodge remains deferred and distances remain metric.

## Acceptance and limits

Five codec tests cover canonical requests/state, historical acknowledgement
independence, malformed/bounded frames, mixed-event ordering, incompatible
profiles, weapon damage and foreign incarnations.

Two actual encrypted UDP tests use the production server/client loops and real
World/Avian health. One runs 20/30/60 Hz through configured delay, jitter, packet
loss and duplication, using the normal prediction driver. It checks engage,
weapon damage, manual turn-away pause, turn-back resume, cancel without losing
selection, and discarding a mismatched local stream intent. The other deliberately
fills a one-entry presentation queue and verifies explicit disconnection with
the first event retained. Equipment is supplied by a synthetic server-owned
admission/inventory observation, not PostgreSQL or a client packet.

Focused proof: `build/melee-udp-v4.log`. Workspace formatting, denied-warning
Clippy, 153 core tests, 464 network passes / ten explicit skips / 69 groups, and
113 client tests pass. Results are in
`build/classic-melee-transport-results-v1.json` and matching logs.

These checks do not establish normal OIDC/PostgreSQL melee play, new GPU/native
presentation, Windows execution, MMO capacity, or every forged packet and queue
failure over UDP. The existing World tests separately cover foreign epochs,
replays, gear/readiness/range/LOS, death/respawn and authority work budgets.
Next: versioned gameplay content/launcher and normal-client right-click
engagement, swing feedback and native one-hostile acceptance. Per-ability facing
and timer interactions remain further work.

Exact source and proof: `build/classic-melee-transport-qualified-v1-20261101`.
All changes remain local and unpushed.
