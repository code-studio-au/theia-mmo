Update: Native admission, presentation and full-source regression qualification now pass. [Latest playable proof and command](CREATURE_PURSUIT_NATIVE.md).
The dated checkpoint below remains preserved.

# Perception, pursuit and return — P3-07 / P4-03

Status: headless authority and encrypted two-peer pursuit qualified, 2026-10-31.
The new opt-in `development-gameplay-creature-pursuit.json` bundle enables these
settings. Native OIDC/PostgreSQL/wolf qualification remains in progress; the prior
runnable grip checkpoint stays preserved. This is the next slice in
[the requested feature register](COMBAT_WORLD_FOLLOWUPS.md).

## Policy implemented in the core crate

`pursuit::Settings` validates finite perception/leash distances, chase/return
speeds and bounded sight-grace, engagement and reacquisition times. Pure fixtures
use 12 m perception, a 35 m leash, 5 m/s movement, 2 s sight grace, a 30 s engagement
limit and 2 s after return before reacquisition. These settings also appear in the new opt-in creature profile; they are not
finalized game balance. Existing profile bytes and default behavior are preserved.

Perception selects the nearest visible live player in the same world, with actor
ID breaking distance ties. Candidate input is capped at 256; the current network
world supports eight players, and no larger population is qualified. The exact
first attacker can provoke engagement outside passive perception, within the
leash. An already held target is not replaced by a closer player or another hit.
The server adapter must supply current life/position/visibility observations;
clients cannot grant perception or create aggro.

On acquisition, the current supported patrol position becomes a fixed anchor.
Chase and Attack retain the exact target life. Attack is permission for the
existing melee authority only when its current reach/LOS observation allows it;
the pure policy does not calculate damage, execute windups or move bodies.
On losing sight, chase uses the last seen position for the configured grace.
Restoring sight updates that position and resets grace. Expiry, a confirmed
unusable route, an invalid/dead/disconnected/replaced target life, or the player
or creature exceeding the leash from the original anchor causes Return.

Return clears the target and attack permission, ignores new aggro, and points to
the saved patrol anchor. Arrival resumes Patrol with a bounded reacquisition
delay. A return that exceeds the bounded time becomes explicitly Blocked, with
no movement goal; it does not teleport. A dead creature stays Dead until a new
state is constructed for its fresh actor key. Tick ordering and time overflow
are checked; refusal preserves the complete prior state. Repeating a tick returns
the existing directive without another transition.

## Concrete checks

Eight policy tests cover nearest/visible/live/current-world selection and ties,
perception boundaries and candidate budget, fixed-anchor chase/attack/leash and
return/reacquisition at 20/30/60 Hz, exact LOS-grace boundaries and recovery,
first-attacker provocation, unchanged held targets, NPC leash crossings,
dead/disconnected/replaced-life targets, blocked paths, engagement/return limits,
invalid settings, stale/duplicate ticks and atomic clock-exhaustion refusal.
Core all-target qualification passes **108 tests in 17 groups**, denied-warning
core/all-target Clippy and workspace formatting. Exact source/proof is preserved
in `build/pursuit-policy-qualified-v1-20261031`; logs are
`build/pursuit-core-*-v2.log`. The earlier full-workspace result remains 539 passes /
40 explicit skips for the preceding navigation-worker source. No new whole-workspace,
native, performance or Windows qualification is claimed for this pure-policy step.

## Implemented headless integration and qualification

Optional strict `aggro` settings are bound into the existing exact-byte creature /
world / life identities. A worker is prepared only for the enabled profile.
The owner pump polls replies; fixed ticks perform no path search or file/SQL I/O.
Perception scans at most the existing eight players every 100 ms while patrolling,
using current lives and static LOS. A held target is checked each tick.
Chase/return requests are limited to one outstanding job and no more often than
250 ms, with one-second validity. Publication also checks current actor/life,
phase/destination revision, start drift (1 m) and goal drift (2 m).

Accepted waypoints drive the existing collision/grounding controller. Confirmed
unusable paths or collision stalls end pursuit; no direct-line fallback is used.
The saved patrol waypoint and remaining wait resume after return. Melee retains
windup/completion range, LOS and life validation. Return/death/despawn clear
attack permission, and respawn retires the previous creature/job identity. No
health reset or damage immunity is added on return; production evade balance is
separate work. The NPC can still be damaged and killed while returning.

All **89 network unit tests** pass, including six new actual controller/worker /
retaliation integration tests and the strict optional-profile contract. Full
workspace/all-target denied-warning Clippy and formatting pass. The six tests
cover real pursuit/melee/leash/return and saved route progress at 20/30/60 Hz,
departure, static occlusion and restored sight, unavailable navigation without
movement fallback, NPC death/respawn/despawn, and player death/replacement life.
Their navigation is an explicit gentle-terrain triangulation fixture; Recast
obstacle clearance and seams retain their separate baker/backend proof.

A new actual encrypted two-peer 30 Hz test passively acquires A, applies private
in-range damage, moves both players through the shared prediction driver, exceeds
a **20 m fixture leash**, replicates more than 10 m of NPC movement to both peers,
returns to patrol and keeps B's health/windup private. It records one acquisition,
one return/arrival, 17 published requests and no failed routes in this run.
These are real encrypted development tickets, not OIDC or native clients.
Existing impaired life/respawn tests continue at 20/30/60 Hz.

The actual release validator accepts both the new thirteen-profile pursuit bundle
and the preserved player-life bundle. Matched headless source/proof and validator:
`build/pursuit-authority-qualified-v1-20261031`. Logs:
`build/pursuit-authority-*-v4.log`, `build/pursuit-content-*-v1.*`.
The previous 108 core passes remain applicable to unchanged core code; the earlier
539-pass / 40-skip whole-workspace result belongs to the navigation prerequisite,
not this newer source. No new complete workspace total, native, FPS, population
capacity or Windows qualification is claimed here.

## Native acceptance still required

Use actual OIDC accounts and owned PostgreSQL with the new profile and animated
wolf. Observe first-player perception, run-in and in-range melee, movement beyond
the configured 35 m leash, attack cancellation, return and resumed patrol from a
second native observer. Check private health, model Run/Attack presentation,
standard camera/UI captures, cleanup and Vulkan diagnostics. Preserve current
and legacy gameplay regressions and normal launcher cleanup before promoting the
replacement playable checkpoint. Public NPC attack timing, player sword attacks,
ranged enemies, projectiles and hit outcomes remain in the follow-on register.

The [path worker contract](CREATURE_PATH_WORKER.md) remains a separate prerequisite.
These local-package results do not establish streamed/all-world navigation or MMO
population capacity.
