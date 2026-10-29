# Combat UI — initial P2-14 slice qualified, 2026-10-29

Actual prerequisites: the validated ability catalog and authoritative targeting,
health, casts, queue/cooldowns and reliable outcomes (P1-11/P2-11/P2-12), plus the
qualified spell presentation hooks (P2-13). No new simulation, transport profile or
server damage logic is required for this presentation slice. The prior binaries
in `build/spell-feedback-qualified-v1-20261028` remain preserved.

Implement one development action slot for the first catalog ability with standard
Bevy 0.20 widgets. Click/press and the existing Digit 1 shortcut enter the same
bounded cast-request path; widget and keyboard activation in one frame coalesce.
The slot remains pressable during cooldown so the server can apply its existing
queue/rejection rules. UI progress never grants readiness or predicts damage.

Health, cast progress and cooldown bars are bounded views of the latest validated
server health/tick deadlines, with numeric/state text as an alternative to colour.
Keep target controls, critical notices, locomotion and spell cues available.
Clearing/losing a target and disconnects must not leave a ready-looking action.
Use Bevy's current `ui_widgets::Button`/`Activate` API rather than its deprecated
Interaction button alias. The matching `bevy_ui_widgets` 0.20.0 package is the only
new dependency; existing Bevy/Avian pins stay unchanged.

Acceptance: focused activation/coalescing and deadline edge tests, denied-warning
workspace/all-target Clippy/fmt and relevant regressions; two actual Linux clients
on saved terrain under impaired encrypted UDP, alternating production keyboard
handling with native Bevy picking/hit-testing/widget activation. Verify observed
layout widths against server-derived fractions, real casts/damage/death/respawn,
critical spell feedback and continued movement/rebasing. Capture only game
windows; isolate any PCM capture as documented in [SPELL_FEEDBACK](SPELL_FEEDBACK.md).
Baseline and launcher cleanup must continue to pass. Record native GPU and input
scope honestly: synthetic Bevy pointer messages are not physical OS mouse events.

This is one development slot and target health, not a production multi-bar editor,
player HP/stats, full keyboard-focus/accessibility support, responsive UI or a
Windows execution qualification. Those remain separate work. Native and full workspace regression acceptance are recorded below. Changes are local/uncommitted; story edits remain untouched.

The first native run (`build/combat-ui-native-wifi-v1-20261029`) failed overall
acceptance: its bars measured correctly (maximum width error 0.467 px), keyboard
casting/real damage worked, and integrated critical cues survived cosmetic load,
but six injected pointer cycles produced zero widget requests. Source inspection
confirmed that mesh picking did not enable Bevy's separate `ui_picking` feature.
The current continuation enables that standard backend and retains the exact
pointer-cycle/widget-request equality gate. No synthetic Activate event replaces
native UI hit testing. The first attempt's screenshots/logs remain available;
this failed run does not establish working pointer activation.


## Qualified native evidence

`build/combat-ui-native-wifi-v2-20261029/verification.json` passes the unchanged
pointer equality gate after enabling standard UI picking. Two actual secure Renet
clients ran for 35 seconds on saved terrain, with 40–100 ms per direction, 10%
loss plus duplicates/reordering, active Khronos Vulkan validation and 1280×720
windows. Linux RX 7900 XTX (RADV NAVI31), Vulkan, Mesa 26.2.5 kisak.

Each client reports five pointer presses/five widget requests plus five keyboard
requests, 15 damage outcomes, three deaths/two respawns, and zero gaps, rejection,
backpressure or large movement displacements. Cast progress has 612/614 observed
samples, cooldown 1,422/1,438, damaged health 946/953. Actual computed fill widths
have maximum error 0.467 px, no invalid layouts. Screenshots were inspected; the
action slot/cast bar sit above the development terrain readout. The widget carries
a semantic Button role and an explicit accessible label; that is not a full
accessibility qualification. Startup asserts the standard UI picking plugin exists.

Both clients inject 10,240 cosmetic requests and retain all 18 critical visual
and spatial audio cues under concurrent cosmetic load, peak 50 visuals/26 voices,
no critical overflow. Actual live rebase samples remain and checked visual
round-trip error is ≤0.36 µm. Terrain loads/retires 385/304 with ≤81 resident; actual
moving-cast authority samples 156/160. Instrumented two-client rates 77.89/78.19 FPS
are not a hardware ceiling or population capacity. Private-sink PCM RMS 0.004390,
peak 0.05402, 421,347 nonzero samples; no foreign streams or default-sink changes.
The server accepts 20 casts, delivers 90 events to peers, reliable queue
peak 518 bytes, no simulation/I/O deadline misses; maximum simulation work 0.533 ms.

`build/combat-ui-critical-only-v1-20261029/verification.json` also passes, including
five actual pointer cycles/five widget requests, six keyboard requests, actual
critical audio/visuals and <0.47 px fill error with cosmetics disabled.
`build/combat-ui-baseline-v1-20261029/verification.json` passes ordinary no-combat,
audio-disabled prediction at 832.17 FPS uncapped, one client/35 seconds with Vulkan
validation off. This empty-world number is not comparable to instrumented two-client
saturation as a hardware ceiling. Actual two-client feedback and UI-only combat
launcher interruption/child-failure cleanup pass in
`build/combat-ui-launcher-{feedback,combat}-v1-20261029/verification.json`.

The native messages exercise Bevy picking, its UI backend, standard widget
observers and the real encrypted request path. They do not establish physical
mouse/keyboard delivery, Windows execution or arbitrary-resolution UI behavior.


## Regression closure and preserved artifacts

`build/combat-ui-workspace-v2.log`: **280 passed, zero failed, ten private-fixture
skips**. This includes all 59 client tests and the new activation/coalescing and
bounded deadline cases. `build/combat-ui-clippy-final.log` and
`build/combat-ui-fmt-final.log` pass workspace/all-target denied-warning Clippy and
formatting. The first full-build attempt (`build/combat-ui-workspace-v1.log`)
crashed inside Rust 1.99's rust-lld while linking the targeting test executable
(signal 11), before that test ran; `-j 1` completed the retry. No gameplay gate was
relaxed. SDK/cache preparation is independent of runtime acceptance. Obsolete
generated debug executable variants were removed to make room; newest two per
family and all `build/` qualification artifacts/source/assets were retained.

`build/combat-ui-qualified-v1-20261029/sha256.json` preserves exact Linux binaries,
original data and external character identity; `source-sha256.json` records the
relevant UI sources/configuration at this checkpoint.

| Artifact | SHA-256 |
| --- | --- |
| Client | `852349693c91ced7779d8ab5871386b467bec97fb8bee4a3bbacdff7f999aa63` |
| Server | `6fed4feb62c9b572800f98ad32b43b8cd17aabb759f66b3b8574015882304c3a` |
| Ability data | `c5c53ea53fc01df78e9be45a733731257bccc76f31853e12c5b09952b54d377c` |
| Feedback data | `ffdfaa7d096d06c01abf3a0f69b46dfdee4ecac64bdbe3509aa2620fb5abc366` |
| External character | `e0711a9603665cf25ec19e5b12539cc9a6f9db70a4658bdc87a74064fc0a0dfe` |

Next planned foundation is P3-02 asynchronous PostgreSQL persistence/migrations,
with actual server save/reload evidence and no awaited database work in the fixed
simulation step. The SDK-only smoke in `build/postgres-sdk-smoke-v1-20261029` is
preparation, not implemented MMO persistence. External account-provider selection
remains unresolved. No changes were committed/pushed; private story files remain
outside this engineering work.
