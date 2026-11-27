# Renet connection confirmation — 3 November 2026

P1-06 / P1-08 / P3-04 / P3-16. This fixes an observed connection failure; it
does not change movement, terrain, quest rules or combat authority. Dependencies
remain pinned. The earlier normal schema-15 playable is preserved separately.

## Reproduction and cause

The public-avatar lifecycle fixture intermittently timed out at Welcome when a
new session reused the same relay backend endpoint. A standalone plain World
matrix passed all 12 baseline/impairment, fresh/reused endpoint, 20/30/60 Hz cases.
That alone did not explain the failure. The matched death/respawn fixture then
reproduced it in isolation at 30 Hz.

The pinned renetcode 2.0.0 server sends its initial KeepAlive on accepting a
connection response. Its client accepts game payloads only after receiving a
KeepAlive. The server's ordinary KeepAlive resend condition uses the same
last-send timestamp that outgoing payloads update. Continuous game traffic can
therefore prevent KeepAlive retransmission while the client remains connecting.

The bounded relay can now deliberately drop a chosen number of server KeepAlive
packets, identified only by the pinned public packet-type nibble. No decrypted
payload, token or raw packet is written to a report. Dropping only the first
KeepAlive failed all 12 standalone cases before the fix, including initial
connections on fresh endpoints. Endpoint reuse affects the deterministic loss
schedule; it is not a necessary condition for the handshake failure.

Primary implementation references: pinned local renetcode 2.0.0
`server.rs::update_client`, `generate_payload_packet` and
`client.rs::process_packet`. The upstream
[server source](https://github.com/lucaspoffo/renet/blob/master/renetcode/src/server.rs)
and [client source](https://github.com/lucaspoffo/renet/blob/master/renetcode/src/client.rs)
are context; qualification is against the repository's locked versions.

## Implemented boundary

Renet transport revision 1 adds an exact four-byte NC01 encrypted reliable connection
confirmation before World admission. The client sends it once after netcode
reports connected. The server waits without publishing game payloads to that
connection, allowing the normal library KeepAlive retry to complete. Confirmation
does not contain an account, character, position, input or gameplay entitlement.

The gate has at most eight waiters, each with a five-second deadline. Invalid
first event messages are refused; disconnects remove waiters. Only a confirmed
connection proceeds to the existing ticket/account authority and prepared-terrain
admission. Existing account lease, one-use ticket, World readiness and critical
queue rules remain in force. Other admitted peers continue normally.

The Renet protocol ID encodes transport revision 1 independently of movement
wire/Quinn ALPN 12. Rebuild both Renet ends and issue fresh tickets. Older Renet
protocol tokens are rejected explicitly; no silent mixed-version compatibility
claim. No dependency fork, key/crypto change, timeout increase, server-authored
movement bypass or simulation-step transport I/O is introduced.

## Qualification

The first fixed build passes all 12 deliberately lost-initial-KeepAlive cases,
and the matched same-endpoint death/respawn/rejoin scenario passes all three tick
rates. The ordinary fresh-endpoint test remains, and the previously failing
same-endpoint scenario is retained as ordinary regression.

The first full network regression passes all 674 tests (ten explicit service
skips). The deterministic lost-first-KeepAlive check is also retained for all
three rates. Final workspace, strict Clippy and normal account/native
rejoin acceptance are recorded below. No new production, native Windows, population-capacity or
unbounded loss/reconnect guarantee follows from these controlled loopback tests.
The earlier public-avatar endpoint limitation remains historical evidence.

## Snapshot loss is a separate boundary

Repeated periodic-loss diagnosis passed 46 of 48 cases and retained two failures
after Welcome, not handshake timeouts. In one 20-Hz failure, both connections
were confirmed and both clients had Welcome. The affected relay received 122
larger server packets, deliberately dropped 121, and forwarded only one; its
client had no snapshot. The other peer continued through tick 126. The periodic
every-eleventh-packet loss had phase-locked to the snapshot cadence. This is a
valid adversarial limitation of the unreliable snapshot path, not a reason to
raise admission deadlines or claim recovery when all useful snapshots are erased.

The standalone checker retains that pattern through `--periodic-loss`.
Its default comparison now explicitly names `seeded-block-loss`: fixed
non-secret seed, one dropped packet per eleven-packet block, with the loss
position mixed by block/direction. The baseline has no loss. Existing regression
fixtures retain their prior every-Nth schedule unless they explicitly select the
new seed. This does not model measured Wi-Fi quality or guarantee delivery under
all loss patterns. A focused test verifies exact block budgets, at most two
adjacent losses for this eleven-packet configuration, determinism and preservation
of the original periodic policy.

```sh
cargo run --locked -p theia-network --example qualify_session_rejoin -- --rounds 2
cargo run --locked -p theia-network --example qualify_session_rejoin -- --rounds 2 --drop-first-keepalive
# Explicit adversarial diagnostic; a missing snapshot can correctly fail this:
cargo run --locked -p theia-network --example qualify_session_rejoin -- --rounds 4 --periodic-loss
```

Reports distinguish handshake confirmation, Welcome, latest snapshot tick/player
IDs and coarse packet-size counters. No tickets, private gameplay values,
decrypted bytes or captured packets are printed. All captures stay bounded and
in memory. The checker is a diagnostic executable, separate from the normal
client and server.

## Workspace and normal account acceptance

The production fix passes the complete workspace run: 1,128 tests, 50 explicit
service skips. The subsequent test-only seeded-loss extension passes its two
focused checks; this is not a claimed fresh 1,129-test whole-workspace run.
Formatting and workspace/all-target Clippy with warnings denied pass.

The optimized normal schema-15 game passes actual OIDC/PostgreSQL admission,
two quests, three unmodified-wolf kills, four committed actions, two pre-accept
no-op conversations, journal head 13 and 125 XP. Database process restart and
fresh-ticket rejoin preserve exactly that completion/reward state, with no new
damage facts. Two native starts at 1920x1200 have zero GPU diagnostics on Linux
AMD RX 7900 XTX / RADV Vulkan / Mesa 26.2.6. Owned services and plaintext password
files are cleaned. Proof: `build/session-confirmation-journal-native-v1-20261103`.
This is PID-bound synthetic X11 input; no native Windows, physical input,
population-capacity or new death-animation claim.

## Final repeated comparison

The final rebuilt checker passes 48 cases: two rounds each of baseline/seeded
loss, 20/30/60 Hz and fresh/reused backend endpoint, repeated with the first
KeepAlive deliberately dropped. Twenty-four cases reuse the endpoint. Across
these cases, 60 initial KeepAlives are dropped and 3,607 old encrypted packets
are injected. Replacement admission is at most 926.77 ms in this run; relay
overflows and cleanup errors are zero. These are measured controlled runs, not
universal latency bounds. The 46/48 periodic-loss diagnostic above remains
recorded and is not included as a fully passing matrix.

Preserve `build/connection-confirmation-client-qualified-v1-20261103`, including
exact source/binary hashes and selected proof. Earlier normal public-avatar and
authoring-review checkpoints remain unchanged. All changes remain local.
