# Character gameplay-profile preflight — qualified 2026-10-31

P3-03 / P3-02 continuation, supporting P3-10 / P3-11 development content changes.
Prerequisite: the complete rewarded training checkpoint is preserved at
`build/quest-turn-in-xp-qualified-v1-20261031` (475 workspace passes / 37 explicit
skips, actual two-account native/SQL at three sizes).

Selecting an old journal-only character with the new rewarded lifecycle currently
reaches the retained owner's initialization before its exact-profile mismatch is
found. Expected development content incompatibility should instead produce a clear
HTTP refusal before any game ticket is issued, without failing the realm's SQL
worker. Saved rows must remain unchanged.

Implement a bounded read-only preflight through the existing asynchronous worker.
Keep the generation-bound Selecting reservation across selection and preflight.
Only enabled inventory, progression, automatic-objective and explicit-lifecycle
bindings are checked. Missing optional state is compatible; existing hashes must
match exactly. The primary saved character must still belong to the authorized
owner/realm/world. Return a typed compatible/incompatible result. Unexpected SQL
or corrupt-data failures remain fail closed; active-operation mismatch checks are
unchanged. No state initialization, migration, reseeding, receipt mutation or
retroactive reward belongs to this operation. This is not validation of every
saved gameplay value or a general content migration service.

Acceptance: actual PostgreSQL missing/matching/changed/disabled-profile cases;
wrong owner/world; read-only proof; cancelled answer and bounded queue handling;
expected incompatibility leaves the worker usable. Actual OIDC HTTP must refuse
an incompatible owned character, issue no ticket, then admit a compatible or fresh
character. Preserve retirement exclusion, cancellation and session revocation.
Run format, denied-warning Clippy, focused contracts and an account/native entry
regression. Native Windows and production migrations remain separate.


## Current evidence

The read-only repeatable-read SQL method, bounded account-worker request and
reservation-held HTTP entry check are implemented. Expected incompatibility
returns HTTP 409 before any ticket; it does not relax active-owner data checks.
The launcher recognizes only the exact public content-conflict error, displays it
without retrying and writes no bearer file. Other admission conflicts retain the
existing bounded retry behavior. Unknown response text is never echoed.

One pure construction test and three actual SQL contracts pass at
`build/gameplay-profiles-postgres-v2-20261031`. They cover missing/matching/changed/
disabled profiles, ownership/world refusal, unchanged legacy rows, a usable worker
after incompatibility, an executing query whose answer is cancelled, and exactly
16 queued versus 24 refused requests under a blocked-reader flood. The cancelled
query finishes without mutating state. Owned PostgreSQL stopped and private
credentials were removed. Six account-helper / 24 focused Python tests, formatting and denied-warning
workspace/all-target Clippy pass. The complete SQL regression also passes 30
actual contracts / 33 commands, including existing reward and process/database
restart cases: `build/gameplay-profiles-full-postgres-v1-20261031`.
Actual OIDC/native preflight now passes at
`build/gameplay-profiles-native-1920-v1-20261031`: an owned legacy journal row
returns the exact nonretryable HTTP 409 with no bearer file or native actor, and
is byte-for-byte unchanged after refusal and database/server restart. Fresh
compatible characters still complete private rewarded training, loot, barter and
equipment, reconnect and restart. A foreign owner receives ordinary unavailability.
The deliberate legacy row is seeded only by the owned qualification fixture;
production entry never rewrites it. All seven native processes report zero Vulkan
diagnostics at 1920×1200 windowed on Linux RX 7900 XTX/RADV/Vulkan. Providers and
database stopped; private tickets were removed. Client/core/network/content source
is unchanged from the prior three-size reward checkpoint. Final unchanged-source workspace passes 476 tests / 40 explicit
skips in 108 groups. Formatting, denied-warning workspace/all-target Clippy,
24 focused Python checks and the complete 30-contract/33-command SQL regression
pass. Exact source, matched optimized binaries, validator and public proof are
preserved at `build/gameplay-profiles-qualified-v1-20261031`. Native Windows,
physical input and a new three-size server qualification are not claimed.
