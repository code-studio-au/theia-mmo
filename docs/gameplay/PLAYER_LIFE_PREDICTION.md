Current opt-in executable and rendered adoption is qualified in
[player-life native integration](PLAYER_LIFE_NATIVE.md). The checkpoint below
records its own prerequisite scope and historical remaining gates.

# Player-life prediction gate — qualified 2026-10-31

P3-04 / P3-07 follow [encrypted publication](PLAYER_LIFE_RUNTIME.md). The shared
prediction driver now has explicit `with_player_life()` construction: it waits
for the first validated own life frame even if movement corrections arrive first.
A dead or unmatched life/stream/tick barrier emits no commands and clears old
prediction history, retransmission commands and clock debt. Completed authority
corrections still provide the displayed pose, including corpse gravity. Resuming
rebuilds prediction from the matched new-stream correction; it fabricates no input
for time spent dead. Ordinary same-life damage/windup keeps the existing clock.
The default legacy driver remains available.

Three new driver tests cover initial waiting, both message orders and a ten-minute
wall-clock pause across death/respawn at 20/30/60 Hz, plus continued movement after
ordinary damage. The existing impaired encrypted two-peer life test now runs this
driver: dead input produces no packet, then current-stream commands actually move
the authoritative player after respawn at each rate. These are actual headless
UDP/World/Avian executions, not native rendered input.

Final unchanged-source checks: **108 relevant network tests** (81 library, eight
life-wire, nine prediction-driver, seven adverse-network and three targeting),
formatting and denied-warning workspace/all-target Clippy. The previous full
workspace result remains 519 passes / 40 explicit skips; it was not rerun for
these three source-file changes. Core, application executables, persistence,
content and Python helper source are unchanged from that checkpoint.

Exact source/proof: `build/player-life-prediction-qualified-v1-20261031`.
Normal CLI/bundle selection, rendered movement gating and health/death/windup UI
still need adoption and native GPU qualification. The latest preserved playable
checkpoint remains `build/character-menu-qualified-v1-20261031`. No new native,
OIDC or SQL-health result is claimed. Health remains transient. Changes stay local.
