# Account HTTP boundary — P3-01/P3-03 next slice, 2026-10-29

Status: in-process HTTP implementation passes focused checks and initial actual
provider/database tests. Actual restart/shutdown checks pass; full workspace validation passes; native launcher
login UX and game admission are not qualified yet. Actual prerequisites are the real-provider OIDC boundary,
committed owner mapping, bounded account sessions and owner-checked character APIs.
Preserve those source/binary checkpoints and the working development launcher.

Build an in-process headless server module with standard Axum/Hyper HTTP handling.
The initial development interface binds numeric loopback only, with pinned Host/
Origin, no CORS relaxation, strict bounded JSON, no credential/code/query logging,
no-store responses and owned connection/request deadlines. Bound accepted
connections, concurrent handlers, pending login challenges and start rate; do not
put identity-provider/database I/O or HTTP processing in World::step.

A native caller starts an OIDC challenge and keeps a separate random poll proof.
The system browser follows the generated provider authorization URL. The callback
consumes its state/nonce/PKCE attempt and resolves the verified issuer/subject to a
committed owner. A valid challenge proof creates and collects its session once;
knowing the browser's callback URL alone must not retrieve that bearer credential.
Expired/cancelled/failed flows cannot issue sessions. Passwords stay with the
identity provider. Tests use only the owned provider's synthetic users/forms.

Character routes derive owner from a validated server session; client JSON never
sets owner, world identity, spawn or checkpoint sequence. Revalidate the opaque
server session context after awaited database work. List/create/select and logout
must preserve name/quota/ownership rejection and rollback semantics. A validated
character lookup still does not mint a transport ticket or claim a live actor.

Acceptance: actual HTTP + local Keycloak + PostgreSQL, two distinct synthetic
users, wrong poll proof/state/replay rejection, revoked/superseded credentials,
foreign-character denial, strict name/body/Host/Origin checks and measured bounded
connection/request behavior. Verify database failure shuts the service down and
all owned processes/connections/private fixture files clean up. Protocol/provider
automation is not physical browser input or Windows execution qualification.

Then connect selected ownership to short-lived single-use encrypted Renet tickets
and one live character session, with prepared saved-pose collision and final
checkpoint ownership before admission-slot reuse. Prove actual native-client
login, reconnect, replay rejection, revocation and database failure separately.
Do not equate this HTTP/account test with working MMO login before those pass.
Production remote HTTPS, provider choice, renewal/key rotation and full character
selection UI remain explicit subsequent requirements.

Implementation: Axum =0.8.9 / Hyper =1.12.0 / hyper-util =0.1.21 in the headless
server library; default game launch does not start this API yet. The module binds
numeric loopback, pins Host, rejects foreign API Origin/absolute request URIs,
limits JSON to 4 KiB/URI to 8 KiB, sets no-store/referrer protections and emits only
static redacted errors. A bounded accept loop permits 64 connections, 16 handlers,
16 pending challenges and 32 starts/minute. HTTP/1 connections handle one request,
with a two-second header and 15-second whole-connection deadline. All connection
tasks are owned/drained on shutdown; identity-provider/SQL waits stay outside any
world simulation. No HTTP/2, remote HTTPS or production ingress claim is made.

Routes: POST `/auth/start`, `/auth/poll`, `/auth/cancel`, `/auth/logout`; GET
`/callback`; authenticated GET/POST `/characters` and POST `/characters/select`.
The callback retains a verified identity/committed owner; proof-bearing collection
issues the account session once. Callback completion alone cannot supersede an
existing account credential. Cancellation needs the same private proof and removes
waiting/verified/in-flight challenges; a late callback cannot issue a session.
Character requests use opaque server-created session contexts, revalidated after
awaited database operations. Selected UUIDs alone do not authorize ownership.

`build/account-http-live-v2-20261029/verification.json` passes actual HTTP with two
synthetic Keycloak users and PostgreSQL: two accounts/nine database characters,
owned lists and selection, foreign-character rejection, quota/name/unknown-field/
oversized-body/Host/Origin rejection, wrong poll proof/nonce/callback replay,
revocation/supersession and cancellation before/after verified callback. The latter
leaves the existing account session valid. Actual pending/handler/connection
pressure exercises 16/16/64 limits; the measured idle-header/partial-body deadlines
match two/15 seconds. Terminating the sole owned database backend shuts the idle
service down with nonzero status. Owned processes stop, owner connections close,
private import/password files are removed and generated secrets do not appear in
diagnostic logs. This remains account API qualification, with no native game
admission or physical browser/Windows execution claim. Version-3 checks pass normal shutdown and same-endpoint database/API restart:
`build/account-http-live-v3-20261029/verification.json`. Old process credentials
reject, and a fresh verified login still lists the original character IDs.
Measured header/body deadlines were 2.001/15.002 seconds. Final workspace validation passes: 294 tests, 13 explicit skips (ten private
fixtures/three PostgreSQL contracts executed separately), formatting and denied-
warning workspace/all-target Clippy. Exact source copies/hashes and the final
headless qualification executable are frozen in
`build/account-http-qualified-v1-20261029`. Final real-HTTP evidence is
`build/account-http-live-v4-20261029/verification.json`; its executable SHA-256 is
`5fd3135faceefabce4ec46f661964dea5194c4e8964563a8528185115c71ed04`.
The standalone server dependency graph contains no renderer/UI/audio/wgpu.

Primary API references: [Axum routing](https://docs.rs/axum/0.8.9/axum/struct.Router.html),
[body limits](https://docs.rs/axum/0.8.9/axum/extract/struct.DefaultBodyLimit.html),
[Hyper HTTP/1 connection bounds](https://docs.rs/hyper/1.12.0/hyper/server/conn/http1/struct.Builder.html).

Next native-ticket requirements and actual prerequisites are recorded in
[AUTHENTICATED_ADMISSION.md](AUTHENTICATED_ADMISSION.md). No native admission code
has been changed for this HTTP slice. Existing engine/network pins and all
previously locked package versions remain present.
