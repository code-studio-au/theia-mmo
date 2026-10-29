# Authenticated native admission — P3-01/P3-03 next-slice contract, 2026-10-29

Status: character/session admission is planned; the game still uses the qualified
trusted development issuer. The reusable memory-only ticket signer is implemented
and its two real-UDP tests pass, together with fourteen secure-runtime/saved-pose
regressions. It does not register one-use admissions or authorize characters.
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
