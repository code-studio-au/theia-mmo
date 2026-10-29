# Resumed targeting verification — 2026-10-28

The post-transport full workspace regression passes 253 tests (ten private-fixture
skips) and workspace/all-target denied-warning Clippy. Native Linux verification and optional launcher adoption now pass. [Current runtime checkpoint](TARGETING_RUNTIME.md). The dated stop and
original contract below remain provenance.

# Targeting transport/client continuation — safe-stop checkpoint

P1-08/P2-03/11. Stopped at the user's request on 2026-10-26. The server
foundation passed the combined 245-test workspace checkpoint before the later
transport and client changes. The transport now has focused runtime evidence;
the client/server opt-in application integration is drafted and compiles with
denied-warning Clippy, but has no native GPU qualification. No creature AI,
navigation, damage or full combat loop is implied.

## Implemented and checked

- Opt-in profile identity binding with unchanged baseline wire version 12 and
  77-byte welcome; tagged bounded TS01 selection and TG01 state frames, at most
  16 targets and 550 bytes per target state.
- Actor-bound authoritative selection, bounded monotonic request sequences,
  negative acknowledgements, independent per-player selection and explicit clear.
  Repeating a choice after a server clear produces a fresh request.
- A bounded trusted local world-control request can remove a target; the actual
  server loop clears selections and publishes removal. This reuses the existing
  chunk-management queue, with the old chunk-control API retained as aliases.
- Four World and four codec tests pass. Three actual encrypted UDP tests pass,
  covering impaired two-client selection/clear/reselect and owner removal,
  mismatched-profile rejection, fresh-session isolation, malicious request cases,
  and saved-terrain predicted traversal through a nonblocking target.
- Latest transport evidence: `build/targeting-live-tests-v5.log` (3 passed),
  `build/targeting-live-v5/targeting-wifi.json`, and workspace/all-target
  denied-warning Clippy in `build/targeting-live-clippy-v5.log`.
  The earlier v4 focused run passed all 11 World/codec/UDP tests.
- Draft `--targeting` client/server integration uses standard Bevy meshes and
  authoritative selection colour/HUD, central global/local conversion and
  loaded-chunk visibility. Tab cycles candidates; Backspace clears; Escape
  retains quit behavior. Targets add no physics bodies. These application
  behaviors are code reviewed/compiled, not native runtime evidence.

## Pending at stop

Run the full workspace regression after the profile/control-loop changes,
including the new client cycling unit test. Then qualify actual native Linux
visibility, keyboard selection, rebases, removal and nonblocking movement with
exact GPU/validation evidence. Actual network relevance departure/return remains
open beyond the World-level checks. Native Windows is a separate gate. Keep the
normal launcher unchanged until the visual slice passes. No qualification or
feature process remains running; changes are local and uncommitted.

The following contract and acceptance describe the intended completed slice.

## Contract

- Explicit opt-in targeting profile. Bind its version into the welcome's expected
  world identity, separately from immutable terrain identity. Preserve baseline
  prediction, saved/generated worlds and existing imported-map regressions.
- Keep common movement/welcome framing compatible if the opt-in profile can be
  safely distinguished. Use explicit tagged bounded selection/state messages;
  mismatched-profile clients must reject the welcome before gameplay admission.
  Retain Renet's authenticated reliable ordered event channel and bounded pump.
- The transport binds the requester to its admitted actor; selection messages
  supply no player position, damage or authority actor ID. Validate incarnation,
  monotonic bounded sequences, target existence and current relevance. Clearing
  selection is explicit. Unknown/malformed/flooded traffic must fail safely.
- Target state has at most 16 IDs with checked f64 global body positions and one
  selected ID. Encode within the existing datagram/event budgets, reject duplicate
  IDs/nonfinite values and bind selection to published candidates. Send on actual
  state/ack changes rather than rewriting static target data every movement tick.
- Bevy presents a standard development mesh and target indicator, using central
  global-to-local conversion and live rebases. No Avian body or world occluder is
  added for a target. TAB cycles known candidates; Backspace clears selection. Preserve existing
  Escape-to-quit and camera controls.
- Death/respawn generations, ability catalog binding, cast commands, health and
  reliable combat effects require the following P2-12 integration contract. Do not
  reserve unspecified features or claim their behavior from a static target.

## Acceptance

Codec/fault tests; real encrypted two-client selection/clear, separate actor
selection, invalid/foreign/stale traffic, actual removal and relevance departure/
return; native Linux Bevy/Vulkan visibility, target input and rebase/nonblocking
movement; native GPU identity/validation evidence. Run formatting, denied-warning
Clippy and appropriate regressions. Native Windows remains a separate gate.
Do not adopt a targeting launcher default until this bounded slice passes.
