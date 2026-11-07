# Player display identity

7 November 2026. P2-03 / P3-01 / P2-14 / P5-13.

**Planned follow-up. Gameplay name replication is not implemented at this
checkpoint.** The player frame currently shows a generic Player label and its
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
