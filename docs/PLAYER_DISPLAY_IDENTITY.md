# Player display identity

7 November 2026. P2-03 / P3-01 / P2-14 / P5-13.

**Qualified locally, 8 November 2026: gameplay name replication and owner-frame
display are implemented.** The 7 November checkpoint below predates runtime wiring.

**Previous checkpoint:** The player frame currently shows a generic Player label and its
validated committed progression level. The qualified protocol-12 client/server
checkpoint is preserved in `build/local-play-release-20261107-v5`.

## Existing boundary

The account service selects a named character through the asynchronous SQL
publisher. `accounts/routes.rs::enter` currently passes only `selected.saved` to
the admission reservation. The name stays in the account/persistence layer.
Gameplay snapshots and public equipment observations do not carry it.

Names are display data, never ownership. Admission remains bound to authenticated
owner, character, realm, world, reservation generation and private ticket nonce.
Neither launcher arguments nor client gameplay commands may supply another
actor's displayed name.

## Proposed small slices and prerequisites

1. Extract the existing pure `CharacterName` value/parser from persistence into
   core, preserving the persistence re-export and temporary 3–20 ASCII policy.
   No SQL schema or ownership-policy change. Verify parser and persistence
   contracts before using it in network data.
2. Add a bounded player-identity codec with actor IDs scoped to epoch/world and
   optional validated names. Unknown/anonymous names retain the generic label.
   Codec qualification is a prerequisite, not evidence of working replication.
3. Capture the SQL-selected name in the reservation/lease outside simulation.
   Snapshot that bounded display value at admission; publishing performs no SQL,
   provider calls or awaited I/O in a simulation step.
4. Publish names for the same replicated-player view as movement, over secure
   reliable Renet. Cache the last sent view so movement alone does not resend
   static names. Defer metadata when its bounded queue cannot accept it; retain
   priority for authoritative replies and critical presentation.
5. Validate/store received metadata and bind the player-frame name to the current
   admitted actor. Remove unavailable/context-invalid display data. Preserve the
   independent committed-level display and generic fallback.

The current limit is eight actors, matching `wire::MAX_PLAYERS`. The candidate
frame budget is 53 header bytes plus at most eight 29-byte rows (285 bytes):
four-byte tag, epoch, tick, world hash, count; each row has actor ID, name length
and at most 20 bytes. Zero length represents an unknown name. Final encoding
and compatibility details must be checked against the actual transport path.

Bump the gameplay wire version from 12 to 13 only when the runtime is integrated;
Renet's protocol ID already incorporates that version. Client/server/tickets
must match. Preserve the qualified older binaries and verify their launcher
path remains usable. No new distributed service, content renderer, collision,
combat-stat calculation or player-authoritative identity is required.

## Concrete acceptance

- Preserve name-parser behavior, spelling and SQL constraints, including maximum
  length, interior punctuation and rejection of unsupported input.
- Codec round-trip empty/maximum views; reject truncation, excess length/count,
  duplicate/unordered actors, malformed names and wrong world/epoch. Bound all
  decoding work and allocation before accepting payloads.
- Admission returns the selected saved name for the exact lease generation;
  expired/replayed/replaced reservations cannot expose a replacement identity.
- Secure UDP peers observe authoritative names for their current player view;
  stale metadata cannot resurrect a removed actor. No account IDs, database
  character IDs, session credentials or private inventories enter this payload.
- Idle/moving unchanged rosters do not publish names every tick. Queue pressure
  remains bounded and cannot block the fixed simulation or critical replies.
- Actual OIDC/PostgreSQL admission and native UI show the chosen character name,
  retain level behavior, handle long allowed names, and clear stale presentation
  on disconnect/rejoin. Distinguish headless transport tests from native proof.
- Run formatting, denied-warning Clippy, appropriate tests and compatible builds;
  record exact sources/devices/limitations before freezing a replacement.

Unicode/name-policy expansion, renaming, character-selection UI, public class or
level metadata, party/chat systems and NPC naming remain separate scopes. NPC
authored display names must not be constrained by the player-name parser.


## Shared name value qualified — 7 November 2026

Slice1 is implemented. `theia_core::character_name::CharacterName` now owns the
existing parser and its original accepted/rejected corpus. Persistence re-exports
the value and converts parse failures into its existing Invalid error category
for database operations. Spelling, byte limits, punctuation, SQL schema and
owner/realm checks are unchanged. This direction lets a future codec use the
same validated value without a network-to-database dependency.

`cargo test --locked -p theia-core -p theia-persistence -j2` passes 385 cases with
73 service cases explicitly skipped. The actual PostgreSQL
`owned_characters_names_quotas_selection_and_rejections_are_transactional`
contract additionally passes in an owned temporary cluster. Formatting and
strict workspace/all-target Clippy pass. Evidence:
`build/player-identity-20261107/{name-value-tests-v1.log,
clippy-name-value-v1.log,fmt-name-value-v1.log}` and
`name-sql-v1/characters-contract.log`. The SQL contract is one executed test,
not qualification of the other skipped service suites.

No name frame is sent or received yet. Gameplay wire version remains12 and the
frozen v5 UI checkpoint remains available. Codec and runtime integration are the
next actual prerequisites.


## Standalone codec qualified; stopped safely — 7 November 2026

Slice2 adds `network::player_identity_wire` as an exported, standalone codec.
PN01 holds at most eight actor rows and 285 bytes, with optional shared validated
names, strict actor ordering and nonzero world/epoch/tick context. Admission
validation requires the owner row and rejects stale or conflicting updates;
unchanged rosters can be compared independently of movement ticks.

Six focused codec tests pass: bounded round trips, all truncations/trailing data,
malformed names/header/counts, duplicate or unordered actors, admitted context and
monotonicity, and roster change detection. Formatting and denied-warning
workspace/all-target Clippy pass after correcting a constant assertion in the
test. Evidence: `build/player-identity-20261107/codec-tests-v2.log`,
`clippy-codec-v2.log` and `fmt-codec-v1.log`.

This module is not connected to admissions, transport, client state or UI. No
name packets are sent or received; gameplay protocol12 and the frozen v5 build
remain unchanged. The user requested a safe stop here. Resume with slice3 only
when requested; these checks do not establish working name replication.


## Runtime integration — 8 November 2026

Slices3–5 now carry the complete SQL-selected character record into the account
reservation. The lease checks authorization, phase and exact slot generation
before exposing a clone of the validated public name. Ticket userdata stays an
opaque nonce/generation and never carries character names or account identifiers.
Anonymous/direct test admissions retain an unknown name.

The Renet peer captures the label once at admission. PN01 publishes only actors
in the recipient's movement view, shares its epoch/world, and suppresses unchanged
rosters regardless of movement ticks. Publication follows gameplay replies and
critical presentation, defers under reliable queue pressure and never awaits I/O.
A membership change between transport polls waits for the next completed tick if
a different roster was already sent for that tick. This preserves strict client
monotonicity without disconnecting valid joining players.

The client validates owner presence, epoch/world and ordered monotonic metadata,
retains the observation across movement snapshots, and displays a name only when
the current snapshot still contains that admitted owner. Disconnect/context loss
returns the generic label. The opt-in player frame uses a separate level line
for named characters and allows maximum-length names to wrap within its width;
committed-level validation remains independent. Other-player and NPC nameplates,
public level/class metadata, renaming and Unicode policy remain separate work.

Gameplay version is now13, including the derived Renet protocol ID. Protocol12
frozen packs remain preserved and require their own paired binaries. Do not mix
an old client with a new server or silently change transport on mismatch.

Qualification is in progress. The actual local OIDC/encrypted-ticket/PostgreSQL
lease contract passes with selected-name rejoin and revocation checks in
`build/player-identity-20261108/admission-v1/verification.json`. The first native
1280×720 long-name flow passes with committed levels, restart/rejoin and real
timeout clearing; its inline name/level layout required refinement. Native-v2
was rejected by its heading assertion; only a final rebuilt run can qualify the
new separate level line. These diagnostics remain distinct from final evidence.


## Integrated qualification — 8 November 2026

The final network suite passes 731 tests in 70 groups with ten explicitly ignored
fixture/service cases (`network-v6.log`). It includes encrypted two-peer names
under delay/jitter/loss/duplication, generation replacement without old names,
unchanged-movement suppression, strict codec/context checks and deterministic
same-tick/queue-pressure deferral. Three low-level test peers now explicitly
decode PN01 rather than treating every reliable frame as a combat/target/chunk
message. Earlier failed runs identified the same-tick producer issue and outdated
test decoders; those failures are not accepted evidence.

The client suite passes 223 cases plus the separately executed licensed-art PNG
preflight (224 total; `client-v2.log`, `client-art-v1.log`). Strict workspace/
all-target denied-warning Clippy and formatting pass (`clippy-v4.log`,
`fmt-v2.log`). Debug and optimized client/server/validator builds pass. Python
multiplayer qualification assertions now expect protocol13; syntax compilation
passes, but that older two-rendered-client recipe was not rerun in this slice.

`native-v3/verification.json` qualifies four actual Linux starts: 1280×720,
1920×1200, 3440×1440 and PostgreSQL restart/rejoin at1280. `native-v4` qualifies
two optimized starts at1920×1200 with restart/rejoin. Both normal account flows
create real SQL-selected maximum-length names, display the name and committed
level1→2, exercise the existing quest/combat/XP/minimap flow and saved UI size,
and close through Interface Exit. The rejoined run freezes only its owned server
to observe real timeout, generic-name/level clearing and disconnected Exit.
All owned services stop and temporary wheel devices are removed.

All six final native starts report zero GPU diagnostics on AMD Radeon RX7900XTX /
RADV NAVI31 / Vulkan / Mesa26.2.6 kisak. Captures were visually inspected for the
long-name wrapping and separate level line. Evidence is under
`build/player-identity-20261108/`; binaries-v3/v4 contain exact copied binaries.
The small fixtures and debug/optimized starts are functional UI evidence, not
MMO population/FPS, native Windows or non-unit monitor DPI qualification. Name
policy remains 3–20 ASCII characters, and gameplay names are not yet shown as
other-player nameplates or target labels.


The optimized paired protocol13 build is preserved at
`build/local-play-release-20261108-v1` (runtime source `2930fa4`). Its build.json
records SHA256 hashes for client/server/validator, copied qualified local art and
the exact debug/optimized native evidence. Launch with an explicit `--binaries`
path through the existing local launcher. Repository-selected content is still
read at launch; this freezes binaries/art, not an entire world/content release.
The previous v5 pack is unchanged.
