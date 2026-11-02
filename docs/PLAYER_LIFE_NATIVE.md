# Player-life executable and native adoption — qualified 2026-10-31

P3-04 / P3-07 build on [encrypted life state](PLAYER_LIFE_RUNTIME.md) and the
[life-aware prediction driver](PLAYER_LIFE_PREDICTION.md). The paired optional
`--creature-retaliation FILE --player-respawn FILE` flags require predicted
creature combat, prepared navigation and character stats. They enable transient
health/life on the headless authority and bind all four exact profile inputs into
the matched world identity before admission. Legacy flags retain their behavior.

The client uses the shared life-aware driver, drains the bounded 16-event life
queue in ordinary combat presentation, and derives its health bar from PL01.
Legacy combat vitals cannot overwrite this health. Standard Bevy text shows the
incoming attack or dead/respawn countdown; action/cast text also reflects the life
hold. New casts and guide/quest requests wait for matched live movement authority.
Input attempted while dead produces no predicted movement or cast. Critical life
presentation is independent of cosmetic spell-feedback quality. The character
currently retains its standing locomotion presentation while dead: a death
animation is separate work.

`development-gameplay-player-life.json` uses schema 3: all thirteen rewarded-life
roles are required, including both new recipes and completion XP. The Rust
validator checks budgets, portable confined paths, exact-byte hashes and stats /
creature references. The launcher seals and revalidates the same thirteen files
for both children and removes them after cleanup. Schemas 1/2 keep exactly their
ten/eleven-role contracts. The new manifest is opt-in; existing manifests remain.
Default balance is unchanged; the native fixture explicitly uses a 0.1 m/s patrol,
25-damage instant ungated spell, 2 s windup / 100-damage strike and 600 ms respawn.

## Completed native evidence

`build/player-life-app-native-1280-v3-20261031` passes two actual OIDC accounts,
PostgreSQL-backed independent quests/XP/loot/trade/equipment and encrypted native
Bevy at 1280×720 windowed. The same checks also pass at 1920×1200 windowed
(`build/player-life-app-native-1920-v1-20261031`) and 3440×1440 fullscreen
(`build/player-life-app-native-3440-v1-20261031`). Each fresh player receives an incoming windup, dies,
attempts blocked movement/casting, respawns and completes its own 12-cast / three-
kill quest with the 75-to-100 XP turn-in. The passive peer remains at full health
before its own inputs. Both rejoin and server/database restart preserve durable
rewards without repeating input, while transient life starts fresh at full health
and generation 1. Database loss fails closed. Owned services stop and private
ticket files are removed. Each windowed run's seven client starts report zero
Vulkan diagnostics. Fullscreen starts report one or two occurrences of the known
`VUID-VkSwapchainCreateInfoKHR-pNext-07781` surface-resize diagnostic, with no
other Vulkan errors; the existing strict checker rejects additional diagnostics.

Linux device: AMD Radeon RX 7900 XTX / RADV NAVI31, Mesa 26.2.6, Vulkan, on Ryzen
7800X3D / 128 GiB. Native Vulkan validation and workspace compilation ran during
functional qualification; this is not a standalone frame-rate or capacity result.
Real Bevy keyboard/picking events are exercised; physical OS input and native
Windows execution are not qualified here.

The first 1280 run completed both gameplay loops but exceeded its owned provider's
60 s token during the 75 s test. The rerun uses an explicit bounded 180 s fixture
token; ordinary fixture defaults and production session policy remain unchanged.
That failed run is retained separately, not counted as a passing qualification.
Visual inspection also corrected a stale "Ready" label while dead before v3.

The matched qualified source / binaries / proof are frozen under
`build/player-life-qualified-v1-20261031`; the unchanged native candidate remains
preserved at `build/player-life-app-candidate-v3-20261031`. Normal thirteen-profile launching
passes interruption, child-failure and pending-login interruption with identical
sealed profiles in both executables and complete ticket/snapshot/service cleanup
(`build/player-life-app-launcher-v1-20261031`). Final checks on unchanged 562-file
source pass **525 workspace tests / 40 explicit fixture skips in 111 groups**,
denied-warning workspace/all-target Clippy, formatting and 28 Python tests. The
previous 30 actual SQL contracts are retained against unchanged persistence source;
no new standalone SQL-contract run is claimed. Actual SQL gameplay/restart checks
were rerun in each native size. The prior playable checkpoint remains preserved
at `build/character-menu-qualified-v1-20261031`.
Health persistence, production combat logout policy, chasing/general threat and
broader creature AI remain open. Changes stay local.
