# PostgreSQL persistence — qualified initial P3-02 slice, 2026-10-29

Prerequisites: checked global coordinates and world/content identity; authoritative
poses; bounded owner-side management; the qualified movement/ability/UI slices.
P3-01's external provider is still unspecified. This development proof may bind
server-issued loopback ticket slots to explicit persistent character/owner IDs;
client-supplied IDs or positions never establish ownership. It does not implement
external login or character creation/selection (P3-03). Imported dependency order
is not a rigid gate; the trusted development issuer must remain explicit.

Implement a small renderer-free asynchronous PostgreSQL boundary with immutable,
checksummed transactional migrations, distinct nonzero UUID identity domains,
owner/world validation, checked f64 body-centre positions and monotonic checkpoint
sequences. A standard PostgreSQL session advisory lock enforces one initial
persistent realm owner; database connection loss must fail closed. This is an
initial modular-monolith ownership guard, not a sharding/process-handoff design.

A bounded owner worker receives at most the latest fixed-capacity character batch,
coalesces ordinary position checkpoints, and acknowledges saved status only after
COMMIT. No database/awaited transport I/O is introduced into World::step. Critical
economy transactions are outside this slice and must follow the architecture's
durable-before-success rule when implemented. Do not reinterpret an ordinary
periodic position checkpoint as a per-tick durability promise.

Acceptance: actual PostgreSQL migration idempotency/tamper rejection, wrong owner/
world and stale writes, UUID/coordinate/SQL-boundary rejection, competing realm
owners, actual restart reload and bounded stalled-worker behavior; live secure
server save/reload from authoritative positions with fresh runtime entity/world
identity; tick/render/input regressions and exact evidence. Default development
launch remains available. Stored poses require prepared collision validation
before admission; missing/changed world data fails closed.

The local SDK is official Ubuntu PostgreSQL 16.15 extracted into ignored
`build/postgres-sdk`, with package SHA-256 checked against signed apt metadata.
`build/postgres-sdk-smoke-v1-20261029/verification.json` passes wrong-password
SCRAM rejection, enabled fsync/synchronous_commit/full_page_writes and a row after
a clean restart. The owned server stopped and temporary plaintext password files
were removed. This proves the test setup, not MMO persistence or power-loss
recovery. No system PostgreSQL service/package installation was performed.

Primary references: [PostgreSQL initdb](https://www.postgresql.org/docs/16/app-initdb.html),
[WAL/durability](https://www.postgresql.org/docs/16/runtime-config-wal.html),
[advisory locks](https://www.postgresql.org/docs/16/functions-admin.html#FUNCTIONS-ADVISORY-LOCKS)
and [tokio-postgres](https://docs.rs/tokio-postgres/0.7.18/tokio_postgres/).
Formatting, denied-warning workspace/all-target Clippy and 284 workspace tests
pass (12 explicit skips: ten private fixtures and two PostgreSQL tests executed
separately against the owned database). Prior qualified binaries remain preserved.

Store/worker qualification now passes in
`build/persistence-store-final-v1-20261029/verification.json`: actual migrations,
version-1 to version-2 upgrade retaining character data, competing realm owners,
wrong owner/world, stale writes across writer restarts, atomic rollback and SQL
constraints; 10,001 publications under a real row lock coalesce to two committed
batches, and backend loss/cancelled shutdown close ownership correctly. The runner
verifies its actual PostgreSQL data directory against an owned fixture marker
before destructive tests. No user database is used.

The opt-in server adapter and `tools/play.py --persistence` pass live Linux
acceptance and final workspace regressions. `THEIA_DATABASE_URL` stays in the private environment and must
identify a numeric loopback endpoint; the current NoTls boundary is development
only. `THEIA_DEV_REALM_ID` optionally chooses a stable development namespace. Slot
bindings are created once by the trusted issuer, distinct from transient actor IDs.
Saved f64 body centres require prepared finite-world collision and bounded landing
support before admission; transient velocity/input/cast/target state resets.
Pending restored slots retain bounded terrain demand. Existing launch behavior
stays available with persistence off.

The native restart proof passes in
`build/persistence-native-restart-v2-20261029/verification.json`. Two actual clients
run for 35 seconds per phase under encrypted UDP with 40–100 ms per-direction delay,
10% loss, duplication/reordering and Khronos Vulkan validation. The first phase
starts around 15 km; PostgreSQL and the server then restart with the configured
spawn near 8 km and reversed connection order. SQL rows equal the final authority
positions; persistent owner/character/world IDs remain stable, writer/world runtime
incarnations change, and checkpoint sequence advances from 34 to 62. Admission
restores the exact saved f64 position; the first client pose is within 0.05 m.
The two phases have two connected actors each, 34/28 committed batches and 68/56
committed records, zero stale/error batches and no simulation deadline misses. Eight seeded records
are explicitly fixture provisioning, not eight connected players. A mismatched
stored world rejects startup before listening/tickets; terminating the actual
owned database backend fails the server closed and disconnects its native client.

`build/persistence-integrated-native-v3-20261029/verification.json` also passes two
native clients with persistence, clickable UI, combat and spell feedback together:
15 damage events, three deaths, 18 critical cues per client under 9,728 cosmetic
requests each, with actual UI-width error below 0.47 px. Launcher interruption and
child failure clean up both game processes and the database owner connection.
The same-binary default, persistence-disabled baseline passes in
`build/persistence-default-baseline-v2-20261029/verification.json` at 821.84 FPS
uncapped for one empty-world client with validation off. Invalid database settings
are ignored when persistence is disabled. This single measurement is not a
performance-regression study or population-capacity claim.

Native hardware: Linux, Ryzen 7800X3D, Radeon RX 7900 XTX, RADV NAVI31/Mesa 26.2.5,
Vulkan, 1280×720 client windows. Captures cover owned game windows only. Owned
PostgreSQL clusters stop and temporary plaintext password files are removed;
no system service or user database is involved. Native Windows, external account
ownership, character selection, inventory/economy durability, process handoff,
unclean power-loss recovery and production database TLS remain unqualified.

Exact binaries, migrations, gameplay data hashes and source hashes are preserved in
`build/persistence-qualified-v1-20261029`. Client SHA-256:
`b53d9c5f1d064456a0eba86b3cb16b51b095eabdcaa1541248ecd897cdab4a2b`;
server: `1946e3c646f8c74af8f974ceb750c89490ee675e7949516b578bc2619925edac`.
The loopback configuration regression also rejects PostgreSQL `hostaddr` overrides
that could otherwise select a remote address despite a loopback `host`.
