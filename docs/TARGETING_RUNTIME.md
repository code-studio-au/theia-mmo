# Opt-in targeting runtime — 2026-10-28

P1-08/P2-03/P2-11 continuation. Prerequisites are the server targeting foundation,
checked entity/global coordinates, authenticated Renet sessions, saved terrain
readiness and the standard Bevy player/camera. This slice adds one static,
nonblocking training target and authoritative per-player selection. Casts,
health, damage, creature AI, navigation and general interactions remain separate.

## Current verification

The slice passes on Linux. Full workspace regression: **253 tests**, ten
explicit private-fixture skips (`build/targeting-resume-workspace-v2.log`).
Workspace/all-target denied-warning Clippy and formatting pass
(`build/targeting-resume-clippy-v1.log`, `build/targeting-resume-fmt-v2.log`).
The first unrestricted parallel relink was safely terminated under memory pressure;
the completed run used `-j 2`. No result from that interrupted attempt is claimed.
A pre-existing argparse help-format error from literal `10%` was fixed to `10%%`;
all three launcher/qualifier help paths and Python syntax checks pass.

Native evidence: `build/targeting-native-wifi-v1-20261028/verification.json` and
`transport-acceptance.json`. Linux Ryzen 7 7800X3D, AMD Radeon RX 7900 XTX
(RADV NAVI31), Mesa 26.2.5 kisak, Vulkan, 1280×720, optimized binaries,
KHRONOS validation enabled. Two actual encrypted clients run for 65 seconds with
40–100 ms delay per direction, 10% configured loss, duplication and reordering.
Expected duplicate netcode replay rejections remain counted separately; no other
errors or Vulkan validation failures occur.

- Six accepted selections, zero rejected selections; ten changed target states
  sent, rather than per-tick static retransmission. Reliable event queue peak
  548 bytes; no movement recovery, skipped interval, simulation or I/O deadline miss.
- Each client completes selected/cleared/reselected screenshots through the
  production input handler and authoritative acknowledgement. Native screenshots
  were inspected: gold selected capsule, purple unselected capsule, animated
  POLYGON players and readable separate terrain/target HUD panels.
- Actual target transform error against the current render frame is 0 m in
  1,922 / 3,752 sampled frames, with two / three observed origin revisions.
  At most one target visual per client and zero target collider samples.
- The travelling client records 3,460 frames with an empty target set and no
  visual, then 1,130 frames with the target restored after return. The other client
  retains its independent selection. Both retain bounded terrain (maximum 81
  roots), actual mesh retirement, all four player animations and predicted travel.
- Target-run rates are 84.32 / 86.36 FPS with two active clients, validation,
  impairment and qualification instrumentation. These are not ordinary-play or
  populated-world performance ceilings.

The same binary baseline without targeting passes native predicted movement,
animation, ground probes and bounded loading/unloading for 35 seconds:
`build/targeting-baseline-native-v1-20261028/verification.json`. Validation is off;
828.28 FPS uncapped, 100 meshes loaded / 19 retired, maximum 81 roots. Empty-world
single-client timing does not establish MMO capacity.

Actual default and opt-in launcher admission/command traffic, SIGINT cleanup and
injected child-failure cleanup pass, with no owned children or temporary tickets
left behind: `build/targeting-launcher-baseline-v1-20261028/verification.json` and
`build/targeting-launcher-optin-v1-20261028/verification.json`.

Preserved optimized binaries: `build/targeting-qualified-v1-20261028`:

- Client SHA-256: `48b839d07cae27decdeee2be13f7741c9313bcdface3f9e041a0d47f97a524ce`.
- Server SHA-256: `aeb129a976e79da703fd3ca17853c37c95d323eaeb3bc50320dec5da9aba8fb0`.
- Existing private character SHA-256:
  `e0711a9603665cf25ec19e5b12539cc9a6f9db70a4658bdc87a74064fc0a0dfe`.

No native Windows, physical keyboard/OS input automation, trusted-owner deletion
on a native GPU, combat death, pathfinding or population capacity is claimed.
Actual encrypted owner deletion is covered by the full regression's transport
checks. The dated plan/stop is preserved in [the original contract](TARGETING_PROTOCOL_PLAN.md).

The native fixture is explicitly enabled by `--targeting` in
`tools/qualify_graybox.py`, with predicted normal-speed movement. Only that fixture
sets `THEIA_TARGETING_QUALIFY=1`. It injects Tab/Backspace into Bevy ButtonInput,
then observes real encrypted server acknowledgements, material selection,
ViewVisibility and renderer pipeline readiness before requesting game-window-only
screenshots. This exercises the production input handler, not physical keyboard
or operating-system input delivery. Movement pauses for those initial screenshots,
then the existing traversal/teleport circuit continues.

The fixture records projected target transforms against the current checked render
frame, observed origin revisions, bounded visual counts, absence of target Avian
colliders, selected/cleared visible samples and actual relevance departure/return.
It has no authority bypass, damage command or movement coordinate submission.
Qualification screenshots and per-second counters are opt-in and remain in ignored
`build/` output. Timing from this instrumented run is not an ordinary-play benchmark.
The targeting panel sits above the terrain panel so both remain readable.

## Acceptance

- Full workspace tests after the transport/world-control changes, denied-warning
  workspace/all-target Clippy, formatting and diff checks.
- Two native Linux Bevy/Vulkan clients on saved terrain over bounded impaired UDP:
  select, clear and reselect through the actual production controls; target colour
  and target HUD; origin changes; visibility retirement on departure and return;
  continued predicted movement, animation and bounded terrain publication.
- Actual encrypted owner removal and malicious/foreign/stale request handling in
  focused transport tests; the native tour covers relevance retirement, not
  trusted-owner deletion or combat death.
- Exact native GPU/driver/validation evidence and game-window screenshots. Native
  Windows remains independently unqualified.
- Retain an explicit baseline launch without targeting. Launcher opt-in now passes; no targeting default is selected.

The baseline wire stays version 12. The profile digest binds targeting version 1
separately from the immutable source terrain identity. TS01 selections carry no
actor identity or position; the admitted transport actor supplies authority.
TG01 target state has at most 16 targets and 550 bytes. Static data retransmits on
view/ack changes, with existing reliable-channel and pump budgets retained.

## Run and continue

```sh
python3 tools/play.py --targeting
python3 tools/play.py --targeting --clients 2
# Reproduce the native fixture with optimized binaries:
python3 tools/qualify_graybox.py build/targeting-native-new \
  --saved-terrain build/graybox-terrain-v1 --prediction --targeting \
  --speed 1 --clients 2 --duration 65 --impairment
```

Tab selects/cycles known loaded candidates; Backspace clears. Escape still exits.
The normal launcher remains unchanged without the flag. The server and client
both reject targeting with imported worlds or unauthenticated/Quinn modes.
P1-11/P2-12 now has a qualified optional [ability slice](COMBAT_AUTHORITY.md).
This selection-only profile still contains no cast/health messages; preserve it
and the normal baseline as regressions.
