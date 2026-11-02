Current opt-in executable and rendered adoption is qualified in
[player-life native integration](PLAYER_LIFE_NATIVE.md). The checkpoint below
records its own prerequisite scope and historical remaining gates.

# Creature retaliation — next P3-07 / P3-04 slice, 2026-10-31

Prerequisites: retained authoritative patrol/navigation, player mana/stats and
combat events, plus the complete training and character-menu checkpoints. Current
creatures patrol and take damage; player health observations still report the
configured maximum. No creature attack/player-death gameplay is implemented yet.

Build one opt-in, data-driven melee retaliation loop on the saved graybox. A
positive authoritative player hit can engage that exact runtime player life. Keep
the first attacker until it disappears, dies or the bounded memory expires; later
foreign hits neither steal nor extend the engagement. Existing last-hit kill/loot
credit remains independent. This first slice has no chase, threat table, groups or
PvP. A distant attacker is not pursued; normal patrol remains the fallback. Players
and creatures remain physically nonblocking.

Use checked fixed-tick windup/recovery/memory, one target and one pending attack
per creature. The authority must validate exact identity, player/NPC life, reach
and static line of sight both when starting and completing a swing. Losing reach
cancels the swing and incurs recovery; losing the target clears engagement. A
repeated tick cannot strike twice; a late step cannot catch up multiple swings.
Hits during a windup do not restart it. Time overflow refuses without partial
state changes. No pathfinding, SQL or awaited transport I/O enters World::step.

First qualify a pure bounded recipe/state machine. This prerequisite emits attack
intents only: it establishes neither player damage nor working PvE. Next add checked
player health/death/respawn behavior, fixed-step authority adoption and matching
private replication. Dead players must not cast or move; respawn must reset input/
cast state safely and reconcile through existing movement/origin conventions.
Define transient versus durable vitals explicitly before runtime adoption.

Standard Bevy health/target presentation and a protected readable enemy windup
must accompany native adoption. Keep the current patrol/training mode unchanged
until the new opt-in path passes. Acceptance includes 20/30/60 Hz arithmetic,
foreign/stale/dead/range/LOS and exhaustion checks; actual encrypted two-client
health/telegraph privacy and lifetime behavior; actual native death/respawn and
reconnect checks with bounded queues and no capacity/performance overclaim.
P3-17/18 remain unspecified. Item grants, general threat/pursuit and production
balance remain outside this slice.


## Pure policy prerequisite — qualified 2026-10-31

The bounded recipe and state machine are implemented in core. Each state owns an
immutable checked profile and simulation rate. It retains at most one exact
runtime target and one windup; reach/LOS/life observations are supplied by the
future authority. Eight new tests cover 20/30/60 Hz ceiling deadlines, recovery,
foreign hit/identity refusal, retained windups, expiration, duplicate/late ticks
and unchanged state on time failure. All 92 core tests, formatting and denied-
warning workspace/all-target Clippy pass. Exact source/docs/proof are preserved at
`build/retaliation-core-qualified-v1-20261031`. This is an intent policy only.
No player damage, native telegraph, transport/CLI/manifest adoption or new complete
workspace/native result is claimed. The prior playable menu/training checkpoints
remain preserved.

The next health/death prerequisite will use transient development-session vitals,
matching the current transient mana boundary: fresh admission starts full; this is
not durable health or a production logout policy. Death must prevent new casts and
movement, cancel unfinished/queued casts, and reset safely on server-scheduled
respawn. Already applied effects and reward credit require an explicit runtime
ordering decision before adoption. Persistent vitals and logout exploitation
protection remain open. The new mode stays opt-in until complete native checks pass.


## Health/life identity prerequisite — qualified 2026-10-31

[Checked transient health and respawn](PLAYER_LIFE.md) now owns each player's exact
runtime actor plus a nonzero respawn generation. Retaliation's target observations
and intents bind both. Its generation test cancels an old windup even if no creature
step observed the dead interval; the health API also refuses an already emitted
previous-life intent. All 100 core tests, formatting and denied-warning workspace/
all-target Clippy pass at `build/player-life-core-qualified-v1-20261031`. This
supersedes the earlier policy's EntityKey-only target representation, while its
checkpoint remains immutable. World/transport/Bevy adoption is still open.


## Later headless World adoption

[Authority behavior and exact proof](PLAYER_LIFE_AUTHORITY.md) now cover one
retaliating creature, health/death/streamed respawn, life/movement stream changes,
world-interaction closure and old-source effect cleanup at respawn. Applied periodic
effects may finish while dead; remaining instances are removed before the new life.
Thirteen new World tests / 100 core / 75 network-library passes, formatting and
denied-warning Clippy pass. This is headless authority; new-mode private transport,
normal executable/bundle selection and Bevy/native presentation remain open.
