# Authenticated native admission — P3-01/P3-03, 2026-10-29

The initial account-to-game path is implemented and passes live checks against
owned Keycloak 26.8.1, PostgreSQL 16.15 and two native encrypted Linux clients.
`build/account-native-live-v4-20261029/verification.json` records exact tested
binary hashes, private ticket permissions and the ordinary launcher account helper
using actual provider authorization pages/code exchange. Browser opening is an
injected fixture callback in this test; physical system-browser interaction has
not been exercised. No story/private account exports or fixture secrets are
required by the runtime or committed as evidence.

Implemented behavior:

- `/characters/enter` validates an opaque account session, reserves its owner and
  character **before** asynchronous SELECT, verifies the selected owner/world and
  issues a twenty-second encrypted Renet ticket with memory-only signing key,
  random one-use nonce and reservation generation. Eight total reserved/consumed/
  active/retiring slots are a tested development bound, not MMO capacity.
- The authority consumes only transport-authenticated user data, waits up to ten
  seconds for saved-pose collision publication, validates support and creates a
  fresh actor/input scope. Session expiry, logout or supersession prevents further
  controls at the I/O/fixed-step boundary. Default development-ticket replay
  behavior remains available and unchanged.
- Every joined lease records checked f64 body centres after each authoritative
  step, before backpressure/disconnect removal; shutdown drops leases before the
  async owner flushes. Dropped never-joined reservations are released; joined
  reservations retire. Final poses stay in superseding batches until the matching
  pose version's mailbox generation is acknowledged committed. A queued snapshot
  or an older commit cannot free a changed final pose. Database failure closes
  account API and game authority together.
- The opt-in `--account-login` server and `python3 tools/play.py --account-login`
  use the existing saved world and a configured local provider/database. The
  launcher opens the system browser, proof-polls for the session, creates/selects
  an owned character, writes an exclusive private ticket file, then starts the
  unchanged native client. One account/client per launcher initially; clients do
  not inherit the database connection environment. Logout and owned-process/file
  cleanup run when leaving. Ordinary launch still uses development tickets.

Native evidence: two actual accounts/two database characters/two simultaneous
players; foreign selection and competing reservations denied; logout disconnect;
exact committed-pose rejoin with fresh actor/input IDs; rejected old ticket;
actual database/authority restart with stable character IDs, invalid old process
sessions and changed 15 km→8 km default spawn; exact distant restore; actual sole
owned database backend termination fails authority closed. RX 7900 XTX/RADV NAVI31,
Mesa 26.2.5 kisak, Vulkan, 1280×720, Ryzen 7800X3D. This is correctness evidence,
not a population or frame-rate capacity benchmark.

`build/admission-leases-live-v2-20261029/verification.json` separately drives the
production lease source with actual verified OIDC, encrypted ticket user data and
PostgreSQL: valid/unexpired nonce replay, tampered nonce/wrong slot, cancelled
SELECT, queued-versus-committed final pose, old acknowledgement versus newer pose,
actual final commit before reuse, stale generation, revoked lease and expired
reservation reclamation all pass. This direct lease fixture is not native-player
evidence; the native run above supplies that separately.

`build/account-launcher-live-v2-20261029/verification.json` additionally passes
ordinary `play.py --account-login` on actual Keycloak/PostgreSQL and one native
client for both interruption and child failure: private ticket removal, owned
children stopped, final pose committed, database ownership closed, client SQL
environment removed. The browser command is intercepted; no physical browser UI
claim follows. Ordinary two-client development launcher cleanup also passes in
`build/admission-launcher-legacy-v1-20261029`. Five launcher HTTP/security tests
pass actual loopback redirect refusal, bounded response reads, failed-browser
cancellation, invalid ticket rejection and endpoint restrictions. The final
network lifecycle regression also injects old encrypted UDP captures through the
same relay endpoint and confirms a replacement actor receives no old command.

The account-only HTTP regression passes on the new server/session code in
`build/account-http-admission-regression-v1-20261029`: exact prior ownership,
proof/cancellation, bounds/deadlines, revocation, outage/shutdown/restart checks.
The full workspace run passes 297 tests with 13 explicit skips (ten private asset
fixtures plus three separately executed PostgreSQL contracts on unchanged source).
Workspace all-target denied-warning Clippy, formatting, twenty-one focused network
tests and five Python launcher HTTP/security tests pass. The native run checks
each client's first rendered pose against its own saved character within 5 cm,
both on rejoin and after restart. The pending-login launcher check in
`build/account-launcher-pending-v1-20261029` cancels its proof-bound challenge,
mints no session/player/ticket and closes the database owner. It uses the frozen
main executable and no GPU client; it is not a native-player count.

The source/binary/evidence checkpoint is
`build/account-admission-qualified-v1-20261029`, with 374 engineering source copies
and SHA256 hashes, exact native/lease/HTTP/main/client executables, asset hashes and
linked live evidence. No private exports or generated credentials are included.
Physical browser interaction, native Windows, production HTTPS/provider/key refresh/session renewal, crash/power-loss
recovery and the full persistent-zone login UI remain open. New engineering
changes are local, separate from concurrent story edits. The original acceptance
contract below remains the target; no entire phase closes here.

## Running the initial account mode

Configure `THEIA_DATABASE_URL` (numeric loopback PostgreSQL), `THEIA_OIDC_ISSUER`
and `THEIA_OIDC_CLIENT_ID`. Optional `THEIA_ACCOUNT_BIND` defaults to
`127.0.0.1:5050`; the provider's public PKCE client must allow the matching exact
`http://127.0.0.1:5050/callback`. `THEIA_DEV_REALM_ID` can select a nonnil realm UUID;
the account default is `74686569-6100-0000-0000-000000000007`, separate from legacy
fixture slots. The local provider needs an existing user. This development-only
HTTP loopback mode does not provision a production provider or database.

Build with the documented native SDK environment and
`cargo build --release --locked --workspace`, then run
`python3 tools/play.py --account-login` (optionally `--spell-feedback`). Signing
keys remain in memory; session/token/ticket data must not be printed or committed.
The initial session expires within five minutes or the earlier verified identity
expiry; renewal is not implemented yet. Without these variables, ordinary
`python3 tools/play.py` continues to work as before.

## Original slice contract

Actual prerequisites are the real HTTP/OIDC/session boundary, committed owner and
owned-character selection, encrypted Renet transport, validated saved-pose restore
and bounded asynchronous checkpoint worker. Preserve the current development
launcher and qualified source/binaries until replacement admission passes.

The server selects the character only after validating its account session and
owner/realm/world binding. Mint a short-lived encrypted Renet ticket using a key
held only in server memory. Bind its opaque authenticated user data to the selected
character/session admission and a random one-use nonce/generation; neither actor
IDs, transport client IDs nor supplied UUIDs establish persistent ownership.
Unknown/expired/replayed/revoked tickets fail before admitting a player. Keep the
legacy issuer's existing replay behavior when authenticated mode is disabled.

Use at most eight reserved/active/retiring admissions initially, matching the
existing authority's tested peer bound, not MMO-scale capacity. Reuse an admission
slot only after its previous actor has left and its final authoritative pose is
acknowledged committed. New connections get fresh runtime actor/input scope.
A character/account cannot have competing live/pending admissions. Rejoin must
load the correct owner/character pose instead of trusting reconnect coordinates.
Session revocation/expiry must prevent later control/ability processing and cause
bounded disconnection; cached authorization alone is insufficient.

The authority needs only bounded synchronous admission/session decisions and pose
publication. Identity-provider/database/awaited transport I/O remains outside
World::step. Prepare missing saved-world collision on the existing worker/I/O
boundary before join; retain bounded pending/retiring terrain demand and validate
saved body centres. Capture final poses on every removal path, including malformed
controls, event backpressure, explicit disconnect and shutdown. Database failure
must close authority/admission together; queued snapshots are not saved status.

Concrete acceptance: two actual OIDC users and native encrypted clients; denied
foreign-character selection; one-use/replay/expiry/revocation and concurrent
admission rejection; supported distant saved-pose restore; reconnect with a fresh
actor/input identity; delayed old packets cannot control the replacement; correct
final checkpoint before reuse; actual backend-loss fail-closed behavior; bounded
queues/terrain/simulation deadlines; ordinary development mode and launcher
cleanup regressions. Run native Linux GPU checks with exact device/limits. Physical
browser interaction and native Windows remain separate evidence.

Then connect the ordinary launcher to the system browser, proof-bearing session
collection and owned character choice, writing only authorized private bearer
files for the game client. Keep raw login/ticket/session data out of diagnostics
and Git. No production provider, remote HTTPS, renewal/key-rotation, sharding or
population-capacity claim follows from this initial slice.
