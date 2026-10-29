# Character creation/selection — P3-03 next-slice contract, 2026-10-29

Status: the bounded account-owned create/list/select database boundary is
qualified. HTTP/browser UI and authorized native game admission remain open. Actual prerequisites are a verified OIDC
identity, durable internal owner mapping, bounded account sessions, checked saved
world positions and the existing asynchronous persistence worker. P3-01's HTTP/
browser/game-admission integration remains open. The imported list-order is not a
rigid gate: character authorization is needed to complete that admission path.

The initial slice will create and list at most eight characters per owner/realm,
with server-generated nonzero character UUIDs and server-selected supported spawn
positions in the saved world. The owner comes only from a valid server session;
client requests must not choose or replace ownership, world content identity,
checkpoint sequence or initial position. Keep existing trusted development
fixtures available until authenticated entry passes.

Add immutable migrations retaining existing characters and development bindings.
Use an explicit temporary development naming policy and atomic realm-scoped name
uniqueness. Validate lengths/characters and return bounded errors; rejected or
conflicting creation must leave no orphan character row. Creation/list/selection
are awaited outside World::step, using bounded owner-worker requests. Existing
account/pose backpressure, cancellation, ownership failure and shutdown behavior
remain acceptance requirements. Do not add a separate distributed character
service or put character creation in a simulation step.

List only characters owned by the authenticated owner. Select by server-checked
owner/realm/character/world binding; a supplied UUID is a lookup key, never proof
of ownership. Missing, foreign or changed-world records fail closed. Saved body
centres still require prepared finite-world collision before native admission.
Transient entity IDs, held inputs, casts and targets never become persistent IDs
or restored gameplay state. Inventory, levels/stats, appearance customization and
production naming/moderation/deletion policy are separate tasks.

Acceptance: actual PostgreSQL create/list/select across process/database restart;
foreign owner/realm rejection; two competing name requests; per-owner character
cap; malformed/oversized names/UUIDs; transaction rollback with no orphan rows;
valid saved-world/spawn identity and existing migration/pose/worker regressions.
Then integrate the selected character with short-lived single-use encrypted
Renet tickets and one active character session, proving actual native-client
admission/replay/revocation behavior. Database-only selection is not working MMO
login. A production character-selection UI is not established by these API tests.

Initial development naming policy: 3–20 ASCII characters; start/end with a letter;
interior apostrophe or hyphen allowed, with no adjacent punctuation. Names are
unique ignoring ASCII case within a realm, across owners. Another realm may use
the same name. Store/display the entered spelling; never use the name as an
ownership identity. This temporary policy is not production Unicode/localization
or moderation policy.

Creation requires an existing account owner and a nonzero saved-world identity.
The database boundary enforces checked coordinate bounds; its trusted server
caller must choose/validate the actual supported spawn. Creation starts checkpoint
sequence zero and returns only after COMMIT. Existing development fixtures and
unnamed legacy records stay intact; they are not automatically assigned to the
first external login. The eight-character cap counts the owner's stored rows in
that realm, not connected actors. Lists and selection replies remain bounded.

Expected name/cap/authorization rejections must return to the caller without
killing healthy position persistence. Database connection/durability/internal
integrity failures still fail closed. Creation/list/selection share the bounded
account request queue, not the superseding pose mailbox. A returned character ID
or a successful database lookup does not mint a ticket; valid session, matching
owner/realm/world and prepared collision remain required at native admission.

Implemented: immutable migration 4; server-generated nonzero character IDs;
transactional character/name creation; eight-character owner/realm cap; bounded
owner lists and strict owner/realm/world selection. The 16-command worker queue
handles owner/create/list/select requests fairly alongside latest-only positions.
Expected name/cap/ownership/missing/world rejections are nonfatal; database or
internal-integrity failures still fail the owner closed. Body positions remain
trusted checked server inputs; these database APIs do not validate terrain support.

`build/characters-postgres-v2-20261029/verification.json` passes three actual
PostgreSQL contracts, including competing names with one winner, rollback with no
orphan rows, ASCII-case uniqueness per realm, apostrophe/hyphen support, direct SQL
name constraints, owner cap, foreign owner/realm/world rejection, retained healthy
persistence after expected errors and version-1/2/3 upgrade behavior. Named
character ID, owner, spelling, position and checkpoint sequence survive an actual
clean PostgreSQL restart. Existing owner-queue/10,001-position publication and
backend-loss/cancelled-shutdown regressions pass too.

`build/characters-account-integration-v1-20261029/verification.json` passes actual
Keycloak OIDC -> committed owner -> valid account session -> owned creation/list/
selection. Database/process restart preserves the same owner and character; old
process/superseded/revoked credentials reject. This has one synthetic account and
one database character, not a connected native player. No client chooses owner,
world, spawn or checkpoint sequence.

Formatting and workspace/all-target denied-warning Clippy pass; focused auth +
persistence tests pass (11 tests; three live PostgreSQL contracts separately
executed). Updated legacy native regression also passes:
`build/characters-native-regression-v1-20261029/verification.json` has two actual
clients, impaired encrypted UDP, reversed server/database restart restore,
wrong-world rejection and actual backend-loss fail-closed behavior. This uses
trusted development tickets, not OIDC game admission. The server remains headless;
Linux hardware/limits match the persistence checkpoint. Native Windows and physical
browser input are not qualified.

Exact source copies/hashes, migration sources, binaries and evidence are retained
in `build/characters-qualified-v1-20261029`, before subsequent HTTP/session-context
work. Server SHA-256:
`ee75c28c149073619b83a0d8ed5e147d720234f9fe3026ebe08adabd0e64fbf5`.
All owned provider/database/native processes stopped and plaintext fixture files
were removed. HTTP login and character routes are the next slice in
[ACCOUNT_HTTP_FOUNDATION.md](ACCOUNT_HTTP_FOUNDATION.md); ticket issuance and one
active native character session remain separate acceptance requirements.
