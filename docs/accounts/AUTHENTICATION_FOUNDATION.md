# Authentication and sessions — P3-01 scope, 2026-10-29

Status: renderer-free Rust OIDC, bounded process-local sessions and durable owner
mapping pass focused and actual provider/database checks. The workspace run passes 291 tests (12 explicit skips); final eight auth tests
and denied-warning Clippy pass after issuer/deadline hardening. The updated native
server regression passes. HTTP/browser UX, character selection and authorized native
game admission remain unimplemented. The user selected provider-neutral OIDC with
local Keycloak qualification; the production identity provider remains undecided.
Keycloak 26.8.1 here is an owned test fixture.

Actual prerequisites are the encrypted Renet admission/replay boundary, distinct
persistent owner/character IDs, bounded asynchronous owner management and the
qualified PostgreSQL position slice. The current trusted development issuer must
remain available until its replacement passes. Its numbered ticket slots are not
accounts and must never become a client-controlled ownership API.

The first slice will use provider-neutral OpenID Connect authorization code flow
with PKCE, state and nonce, through an established Rust implementation. The native
client uses the system browser. Test automation may submit credentials to the
owned identity provider's actual authorization page; it must not substitute the
password grant, a mocked signing-key service or a service principal for a human
login. Pin the configured issuer/client/redirect, validate issuer, audience,
signature, expiry and nonce, disable automatic HTTP redirects in discovery/token
fetching, and keep credentials/codes/tokens out of logs and command arguments.
Development HTTP is restricted to explicit numeric loopback endpoints; a remote
production deployment needs a separately reviewed HTTPS configuration.

A verified (issuer, subject) maps to an internal owner ID; email/display name are
not ownership identifiers. Internal sessions require random opaque credentials,
bounded storage, expiry/revocation and rate limits. Character selection must check
owner and realm before minting a short-lived single-use Renet ticket. Admission
must enforce one character session and preserve the transport's existing replay
checks. Database, identity-provider and HTTP waits remain outside World::step.
A verified ID token alone does not establish a game session or authorize an
arbitrary character ID.

Concrete initial acceptance: a real local identity-provider authorization-code
exchange, wrong state/nonce/issuer/audience/signature and expired-token rejection,
PKCE/code-reuse rejection, bounded cancellation/timeout behavior and redacted error
paths. Further integration acceptance must prove persistent owner mapping,
authorized selection, denied foreign-character access, session expiry/revocation,
single-character concurrency and actual native-client secure admission. Qualify
these separately; do not label the verifier-only slice working MMO login.

The ignored `build/identity-sdk` contains official Keycloak and Ubuntu OpenJDK 25
archives extracted without installing a system service. Its provenance records
SHA-256 verification against the official GitHub release digest and signed Ubuntu
apt metadata. Any running test instance must bind loopback, own a unique private
working directory/test realm and synthetic user, and remove private credentials
and stop its owned process in cleanup. No existing user identity service is used.

Primary references: [native OAuth applications (RFC 8252)](https://www.rfc-editor.org/info/rfc8252/),
[OpenID Connect Core](https://openid.net/specs/openid-connect-core-1_0.html),
[openidconnect Rust implementation](https://docs.rs/openidconnect/4.0.1/openidconnect/),
and [Keycloak local setup](https://www.keycloak.org/getting-started/getting-started-zip).

`crates/auth` uses openidconnect =4.0.1 and reqwest =0.12.28. Existing workspace
package versions remain present and Bevy/Avian/Renet pins are unchanged. The
initial factory accepts only explicit numeric-loopback HTTP issuer/callback URLs,
checks all discovered fetch/browser endpoints against the pinned origin, disables
proxy/redirect behavior and bounds request/response bodies to 16 KiB/256 KiB with
1-second connect/5-second total HTTP deadlines. A consumed login attempt owns
state/nonce/PKCE and expires after 120 seconds; callback bodies are bounded and
ambiguous code/state parameters rejected. Verified identities have private fields
and no unchecked deserializer. Tokens and credential bodies never reach errors.

Five focused tests pass, including signed negative issuer/audience/nonce/key/
expiry/access-hash cases, callback/expiry rejection, actual HTTP oversized declared
and streamed bodies, and redirect/origin rejection. Focused all-target denied-
warning Clippy passes. The actual Keycloak proof is
`build/auth-oidc-native-provider-v3-20261029/verification.json`: two Rust-verified
logins resolve the same subject; changed state/nonce, wrong PKCE and replayed
callback fail. Only synthetic credentials are submitted to the owned provider's
real authorization page. No password grant or fake key service substitutes for
this positive proof. Private authorization URLs/callbacks travel over owned pipes,
not diagnostic artifacts. The provider stops and plaintext import is removed.
This is not yet an account session, browser UX or MMO admission qualification.

The account/session extension passes seven focused auth tests and actual
PostgreSQL checks in `build/auth-accounts-postgres-v1-20261029`. Immutable migration
3 adds exact case-sensitive (issuer, subject) -> nonzero owner UUID mapping;
versions 1/2 stay unchanged. Version-1/2 upgrades retain characters/bindings,
unknown future versions and tampered checksums still reject. Mapping survives an
actual clean database restart. The async worker adds a fixed 16-request account
queue with one SQL operation in flight, fair service alongside latest-only pose
snapshots, five-second caller deadline, cancelled-reply skipping and committed
acknowledgement. A real row lock proves queue overflow rejection and 15 completed
lookups after cancelling one of 16 queued requests, while 10,001 pose publications
still coalesce to two committed snapshots. No SQL enters World::step.

`Sessions<Owner>` holds at most 32 entries and stores only SHA-256 credential keys.
It issues cryptographically random 256-bit opaque bearer credentials with explicit
secret access/redacted Debug; expiry uses a monotonic deadline conservatively
bounded by five minutes and the verified ID-token expiry. A new login supersedes
that owner's prior account credential. Logout/revocation, expiry, bounded malformed
input and capacity rejection retaining existing live sessions have focused tests.
Sessions are process-local and intentionally invalid after restart; neither an
account session nor its owner ID alone authorizes a character or transport ticket.
Production renewal, multi-process session ownership and recovery remain open.

The server-package qualification example combines actual provider verification,
the committed database-worker owner result and account sessions. Evidence:
`build/auth-accounts-integrated-v2-20261029/verification.json`. Database and example
process restart preserve the owner while rejecting the previous process's bearer
credential; same-process supersession/logout also reject old credentials. The
example checks the actual database directory against its owned fixture marker
before mutation. Auth URLs/callbacks/session credentials travel only through owned
pipes; no password/session credential is written to result/error artifacts. Owned
provider/database processes and owner connections close and plaintext fixture files
are removed. This is library/account-boundary integration, not an HTTP login UX or
working native MMO admission. Existing development tickets remain explicit.

A final auth review adds canonical configured-issuer spelling and exact discovered
issuer matching, preventing URL aliases from creating alternate owner keys. Eight
focused auth tests now pass (`build/auth-unit-v7.log`), including an actual stalled
HTTP endpoint: explicit task cancellation completes and an unanswered request
fails at the configured five-second total deadline. Final focused all-target
Clippy passes (`build/auth-clippy-v6.log`). This focused pass uses an isolated
ignored build directory so it can run while the full workspace links; it does not
replace the full workspace record. The updated real-provider/account proof passes in the version-2 evidence path
above after that workspace run. The full workspace count is from before the final
issuer spelling/deadline test follow-up; the final focused auth pass covers those
changes. No aggregate test count or native admission claim is inferred.

The initial OIDC/owner/session boundary is now qualified and frozen in
`build/auth-accounts-qualified-v1-20261029`: source copies/hashes, exact server and
qualification binaries, evidence manifest and retained client hash. Updated native
regression passes in `build/auth-accounts-native-regression-v1-20261029`: two real
clients, server/database restart with reversed admission, exact saved-pose restore,
wrong-world rejection and actual backend-loss fail-closed behavior. This validates
legacy development-issued game admission with the extended worker, not OIDC-issued
game tickets. The updated server remains headless. Exact release server SHA-256:
`b165e2f618ae701a20ffc9c4f335fefe49b076e248fee697d9be36ae59c80706`.
All owned provider/database/native processes stopped and plaintext fixture files
were removed. Subsequent character work does not replace these preserved artifacts.
