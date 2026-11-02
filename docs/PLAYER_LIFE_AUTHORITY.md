Current opt-in executable and rendered adoption is qualified in
[player-life native integration](PLAYER_LIFE_NATIVE.md). The checkpoint below
records its own prerequisite scope and historical remaining gates.

# Player life and creature retaliation authority — qualified 2026-10-31

Initial headless P3-04 / P3-07 continuation. Prerequisites:
[core health/life](PLAYER_LIFE.md), [retaliation policy](CREATURE_RETALIATION.md),
existing static collision/patrol and the retained playable character-menu checkpoint.

The new opt-in World API requires inactive predicted terrain, resource combat,
exact stats/creature byte identities and one prepared patrol/combat creature.
Its checked profile binds stats, creature, retaliation and respawn bytes with
length-delimited hashing. Failed activation does not change world identity. The
mode is not yet selectable by normal executable/manifest launching; normal play
retains the previously qualified behavior and configured maximum player health.

A positive hit on the live creature can engage the first exact attacker life.
The creature pauses its existing patrol while that player is reachable, winds up,
then rechecks life, reach and actual static-scene LOS before applying checked damage.
There is no pursuit or threat table. Losing reach/LOS cancels; NPC death/respawn
and player death/departure clear the corresponding engagement. A different player
can then arm a fresh windup. One NPC keeps the existing 64-event budget unchanged;
this does not establish population capacity.

Each player starts with full transient health. Lethal damage schedules one respawn,
clears buffered movement by changing its stream epoch and cancels current/queued
casts. Dead input cannot control movement/jumping; gravity and world collision
still apply. Mana regeneration stops while dead. Guide access closes, so new
vendor/quest interactions cannot use the old conversation revision. Previously
accepted durable operations retain their owners and are not rolled back by death.

Respawn uses an immutable supported global home prepared at admission. The existing
terrain owner pump retains dead-position/home demand and loads missing collision
off-tick. A due respawn stays dead until that home is prepared. It then restores
full health/mana, increments life generation and movement stream, and resets
controls/body. Ability cooldown deadlines remain. The last validated client chunk
set is retained: same-patch respawn works without a redundant report; a far home
still holds horizontal movement until its footprint is reported ready. Old-stream
inputs cannot move the new life. Private publication/client prediction correlation
is still an open adoption gate.

Already applied periodic effects may finish while the caster is dead and retain
existing authoritative credit. Any remaining old-source instances are removed at
the respawn barrier before advertising the new generation. They cannot re-engage
or damage through the player's new life. Fresh admitted sessions start full with
a fresh runtime actor; durable vitals/production logout policy remain open.

## Evidence and limits

Thirteen new headless World tests pass, including 20/30/60 Hz damage/death, actual
static-obstruction checks, attacker-specific observation projections, two-stage
collision/client-readiness respawn, old stream refusal, guide revisions, cast/queue
cancellation, retained cooldowns, periodic-effect cleanup and fresh NPC/player
incarnations. These drive the actual World/Avian collision path and existing terrain
worker; projection tests are not encrypted publication evidence.

All **100 core tests / 75 network library tests**, formatting and denied-warning
workspace/all-target Clippy pass with unchanged source throughout the final checks.
The network suite includes the existing encrypted journal-only regression; it does
not qualify new-mode transport. Exact source/docs/public logs:
`build/player-life-authority-qualified-v1-20261031`. No new complete-workspace or
native PvE result is claimed. Prior playable source/binaries/proof are preserved
at `build/character-menu-qualified-v1-20261031`.

Next: bounded owner-only wire frames, matching client/profile handling, movement-
stream/life correlation, critical readable windup and standard Bevy death/respawn
presentation, opt-in executable/bundle adoption, then actual encrypted/native
qualification. No client command may set health, damage, respawn time or position.
Physical input, native Windows, physical 4K and production balance remain separate.


## Later codec/receive prerequisite

[PL01 and its movement barrier](PLAYER_LIFE_PROTOCOL.md) now pass eight wire tests
plus 100 core / 76 network-library tests, formatting and denied-warning Clippy.
The fourteenth life/authority test round-trips real World observations through the
codec. Actual new-mode Renet publication and native adoption remain open. The
headless-only checkpoint above keeps its original source/counts.
