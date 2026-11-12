# Classic ground movement v3 — qualified local-player slice, 3 November 2026

P2-04 / P2-05 / P2-07 / P2-09 / P2-11. Prerequisites: qualified manual
Classic facing, bounded movement commands, prepared Avian terrain, shared
prediction, encrypted admission and the preserved minimap traversal playable.
This slice replaces the prototype travel speeds in the explicit Classic profile.

## Behavior and compatibility

The embedded strict data file `content/movement/classic-ground.json` supplies
walk 2.5 m/s, forward/sideways run 7 m/s and backward run 4.5 m/s. Walking uses
2.5 m/s in each direction. Any backward component selects backward running,
including backward diagonals; diagonals normalize through the existing controller.
Movement uses accepted character facing. Camera rotation and target selection
cannot change speed. Independent manual heading remains unchanged.

The retained bounded wire field named `sprint` carries **run=true / walk=false**
in this profile. It cannot supply an arbitrary speed. Its interpretation is
bound to the new `theia-classic-ground-movement-v3` identity, which includes
the exact embedded movement-data bytes and complete prior source-world identity.
Old Classic v2 clients cannot join this identity; rebuild both ends and obtain
fresh tickets. Editing the embedded speed file requires a rebuild, not runtime
hot reload. Datagram layouts, Renet transport revision 1 and Quinn wire 12
are unchanged. Legacy non-Classic 5/8 m/s development movement remains available
in its separate profile; immutable earlier qualified binaries retain their values.

Controls schema 2 replaces `run_modifier` with `walk_run_toggle`.
Running defaults on; NumpadDivide toggles walking/running. The key is configurable
through the existing strict bounded controls file. Shift no longer boosts Classic
travel; reverse Tab remains independent. Old schema-1 Shift-boost configuration
is refused rather than silently reinterpreted. Dialogue/focus ownership blocks the
toggle, just as it blocks other movement input. An open backpack retains its
existing keyboard movement behavior; pointer ownership is separate. Gait preference stays client-side;
the authority accepts only the bounded input and owns actual displacement.
No autorun, mount, swimming, acceleration, aura speed, root/slow or airborne
momentum rules are introduced by this slice.

Authority and client prediction call the same capsule solver and speed selector.
An axis-unit tolerance prevents quantized pure strafing from being classified as
backward at arbitrary headings. This is a documented Theia quantization rule:
world-XZ inputs are integral thousandths; the maximum rounding projection of an
exact perpendicular vector is below one unit. Very small backward projections
within that unit use the sideways rate. The unchanged axis quantization can
shorten diagonal displacement by less than 0.1%; it never adds a diagonal boost.
Gravity, jump speed, capsule clearance, slope/step rules, world bounds, readiness
and nonblocking actors retain their existing Theia fixture values.
NPC patrol/chase data is unchanged.

## Source evidence and metric mapping

Checked 3 November 2026 against pinned source versions:

- Extracted Blizzard Vanilla character UI at revision
  `4192ac60067fc55566182291bc67c85dd9b6794d` declares base movement 7:
  [PaperDollFrame.lua, line 7](https://github.com/Gethe/wow-ui-source/blob/4192ac60067fc55566182291bc67c85dd9b6794d/Interface/AddOns/Blizzard_CharacterFrame/Vanilla/PaperDollFrame.lua#L7).
  This is UI reference evidence, not Blizzard's server algorithm; the Era mirror
  also contains later additions.
- The same revision explicitly binds the Vanilla TOGGLERUN action to ToggleRun():
  [Bindings_Vanilla.xml, line 93](https://github.com/Gethe/wow-ui-source/blob/4192ac60067fc55566182291bc67c85dd9b6794d/Interface/AddOns/Blizzard_FrameXML/Bindings_Vanilla.xml#L93).
  This supports a toggle. NumpadDivide is Theia's selected configurable default;
  this source does not establish the original client default key.
- CMaNGOS Classic revision `ff13624a4be74816796bc0f8555f03011438144e`
  defines walk 2.5, run 7, backward run 4.5:
  [Unit.cpp, line 63](https://github.com/cmangos/mangos-classic/blob/ff13624a4be74816796bc0f8555f03011438144e/src/game/Entities/Unit.cpp#L63).
  Its flag-based horizontal speed selector applies walk first, then backward run,
  then ordinary run:
  [Unit.cpp, line 8455](https://github.com/cmangos/mangos-classic/blob/ff13624a4be74816796bc0f8555f03011438144e/src/game/Entities/Unit.cpp#L8455).
  Its own walking-backward comment is tentative. Emulator implementation is
  supporting evidence, not proof of original Blizzard behavior.

**Explicit Theia mapping: one referenced gameplay movement unit per second maps
to one metre per second.** These are gameplay-unit rates, not a claim about
physical yards. Theia's existing one-world-unit = one-metre convention and world
dimensions remain. No automatic 0.9144 conversion or terrain rescaling occurs.
This is Classic-oriented calibration with the above uncertainty, not complete
WoW movement parity. Backward/strafe directional animation blending and further
subjective camera/turn feel remain separate follow-ups.

## Acceptance

Headless acceptance passes: all 1,139 workspace tests / 50 explicit skips,
including both gaits through World/prediction and impaired encrypted Renet at
20/30/60 Hz, pure strafing/backward selection at all 65,536 headings, actual
capsule distances/diagonals/jumps at all rates, strict data bounds and toggle
ownership. All 138 client tests, formatting and workspace/all-target
denied-warning Clippy pass. Optimized client/server and validator builds pass.

Optimized native OS checks pass at 1920×1200, 3440×1440 fullscreen and
1280×720; the last uses real alternate A/D-strafe bindings. All 27 authority-tick
speed measurements pass without loosening the 0.07 m/s tolerance:

| Travel | Observed horizontal rate across three sizes |
| --- | --- |
| Forward / sideways run | 6.9982–6.9983 m/s |
| Shift + forward | 6.9983 m/s; no extra boost |
| Forward diagonal run | 6.9967–7.0001 m/s |
| Backward run | 4.4988 m/s |
| Backward diagonal run | 4.4978–4.5001 m/s |
| Walk forward / backward / sideways | 2.4993 m/s |

The native controls fixture disables wolf perception to isolate input. It proves
manual turning, independent LMB orbit, stationary RMB steering, both-button
forward, selection retention, actual equipment-widget mouse ownership,
panel-before-target Escape, disabled dodge/action camera, toggle and reverse Tab.
The 720p capture also shows the updated key hint fitting beside vitals/minimap.
Authority tick measurements wait for the full accepted direction, excluding
both command-start latency and the transition between a diagonal's two keys.

A separate normal schema-15 run uses the **unmodified wolf and quest data**,
actual local OIDC and PostgreSQL at 1920×1200. It completes two quests / three
kills / four durable actions, then restarts the database and rejoins with fresh
authentication: journal head 13 / 125 XP remains unchanged. Normal W running
after rejoin traverses 207.655 m in a 29.942-second check, through three sampled
chunk owners and a render-origin revision (1→2). Terrain loaded/retired reaches
115/51, four collision patches and 152 saved-tile reads; the distant guide icon
is filtered and quest rewards remain unchanged. Maximum numerical map-to-rendered
position error is 9.715e-6 m. This tests coordinate projection, not terrain,
physical or pixel accuracy; the map raster still has 4 m texels.

All **five final native starts** have **zero GPU diagnostics** on Linux AMD
RX 7900 XTX / RADV NAVI31, Vulkan, Mesa 26.2.6 kisak. Owned services stop and test
password files are removed. No new sustained FPS, native Windows, population or
exhaustive continent-coverage claim. The earlier accelerated 32× traversal remains
its separate retained-profile evidence; this slice does not repeat it.

Proof: `build/classic-ground-{tests-v2,workspace-v2,clippy-v1,release-v1,validator-v1}-20261103.log`,
`build/classic-ground-controls-native-v2-20261103/verification.json` and
`build/classic-ground-journal-native-v2-20261103/verification.json`.
The final launcher text-only correction removes its stale Shift-run/Escape-quit
message for Classic; 35 existing launcher/bundle checks and Python compilation
pass after that correction. Rust binaries are identical to native qualification.

Preserved source/binaries/art/proof:
`build/classic-ground-client-qualified-v1-20261103`.
Earlier checkpoints remain unchanged. Changes local, uncommitted and unpushed.

Earlier failed attempts remain evidence: focused v1 requested a test spawn 1 m
above the floor despite the near-support spawn contract; v2 uses 0.01 m. Full
workspace v1 exposed two old combat fixtures assuming implicit 5 m/s; they now
explicitly run to reach melee / escape, with original assertions intact.
Workspace v2 passes. Native controls v1 passed the existing controls and first
four speeds, then its diagonal checker captured a first-key-only start command.
The corrected checker waits for the full accepted direction; the 0.07 m/s native
speed tolerance remains unchanged. This was a measurement transition, not an
engine speed correction. Normal journal native v1 was interrupted by SIGTERM
after completing the quest flow, without an assertion traceback. It is not a
successful qualification: its remaining owned provider was stopped, test inputs
released and password files removed. Complete native v2 supplies the acceptance
above; the signal's external cause was not established.
The prior `build/minimap-traversal-client-qualified-v1-20261103` playable remains
preserved. Changes local; no commit or push.
