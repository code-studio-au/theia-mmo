Current opt-in executable and rendered adoption is qualified in
[player-life native integration](PLAYER_LIFE_NATIVE.md). The checkpoint below
records its own prerequisite scope and historical remaining gates.

# Transient player health/respawn prerequisite — qualified 2026-10-31

P3-04 / P3-07 prerequisite for [creature retaliation](CREATURE_RETALIATION.md).
This pure core contract applies checked damage and scheduled health reset only.
The server still reports configured maximum player health until runtime adoption;
no current executable gains damage/death gameplay from this prerequisite.

The checked respawn recipe references the configured stats profile and binds its
health capacity. Each state owns an immutable profile, tick rate, exact runtime
actor and nonzero life generation. Positive damage clips to remaining health;
lethal damage schedules one respawn. Further corpse hits do not extend it. There
is no implicit health regeneration or respawn during damage application. Explicit
advance restores full health once when due, increments generation, and never
catches up multiple lives after a late step. Foreign/previous-life damage cannot
change health or the clock. Stale ticks and deadline overflow leave state unchanged.

Retaliation targets now include the player's respawn generation. An old pending
swing cancels when that generation changes, even if the creature was not evaluated
during the whole dead interval. The damage API independently checks the expected
life, so a previously emitted stale intent cannot injure a new life either.

Eight new player-life tests and the strengthened eight retaliation tests pass;
all **100 core tests**, formatting and denied-warning workspace/all-target Clippy
pass. Exact current source/docs/proof: `build/player-life-core-qualified-v1-20261031`.
The earlier EntityKey-only policy checkpoint remains preserved with its narrower
scope. No new full-workspace, encrypted transport or native PvE result is claimed.
The complete playable checkpoint is still `build/character-menu-qualified-v1-20261031`.

## Runtime acceptance still open

Use transient development-session vitals initially, matching current transient
mana: a fresh admitted session starts full and receives a fresh runtime actor.
This does not establish durable vitals or a production logout policy. Keep the
existing patrol/training mode available and require explicit matching profiles
for the new development mode.

Bind the authority's creature hit intents to exact player lives. Check static LOS,
reach and both lives at windup start/completion; no chasing or new path service.
A dead player's input cannot control movement or start/finish queued casts.
Cancel unfinished/queued casts; retain cooldown deadlines. Already applied periodic
effects may complete with existing authoritative credit; document and test that
ordering. Committed inventory/progression operations accepted before death still
finish under their retained owners.

Reuse the existing movement-stream epoch and prediction reset for death/respawn.
Old-stream inputs must never move the new life. Complete the supported server-owned
respawn pose and input/cast barrier before advertising the new alive generation.
The client must correlate life and movement corrections before resuming prediction.
Keep required collision demand bounded/off-tick and hold dead state if a respawn
pose is not ready. No SQL or awaited transport I/O enters the fixed step.

Add matching bounded private life observations, protected readable windup feedback
and standard Bevy health/death/respawn presentation. Test actual encrypted two-peer
identity/privacy/stale stream handling and native damage/death/respawn, terrain
readiness, reconnect and default-mode regression before claiming working PvE.
Durable health, production respawn points, general threat/pursuit, groups, native
Windows and production balance remain open.


## Later headless authority adoption

[World adoption](PLAYER_LIFE_AUTHORITY.md) now passes thirteen new World tests,
all 100 core / 75 network-library tests, formatting and denied-warning Clippy.
The earlier pure checkpoint above remains immutable. Private wire/client/native
adoption and durable vitals remain open; normal launching has not enabled this mode.
