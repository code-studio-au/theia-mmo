# Wednesday, 25 November 2026 — Sydney development realm

**Pick up P5-23 with the launcher wake flow.** The Sydney realm is live with
Cognito managed sign-in and two enrolled testers. Launcher 0.1.6 and client
0.1.0.12 passed browser sign-in, native character selection, and movement in
Millhaven for one tester over the public encrypted connection.

1. Build the API Gateway HTTP API, scoped Rust Lambda, and DynamoDB lifecycle
   state described in the [hosting requirements](engineering/HOSTING_AND_OPERATIONS_REQUIREMENTS.md#initial-hosting-scope-and-cost-priority-creator-approved-supersedes-earlier-phase-scheduling).
   Connect launcher **Play** to authenticated wake and poll until the game and
   database are ready.
2. Test two remote players together. Verify that concurrent Play requests
   start the instance once and that pending joins prevent idle shutdown.
   Check that the five-minute idle stop runs after a successful persistence
   flush and that progress survives restart.

Launch locally with `./build/hosted-development-launch/open-launcher`. The
launcher and client are staged in `build/launcher-client-exports/`. Live
changes are recorded in [STATUS.md](STATUS.md). The working tree contains
local changes for the Cognito and launcher work. Review that diff before
committing. The new builds are staged for private testing.

# Current checkpoint — 16 November 2026: shared-marker UI accepted

16 November 2026 — P4-11 shared-marker presentation accepted:
A compact Unit marks palette beside the member frames assigns the eight original
symbols through cda0608a's authoritative private-party protocol. Leaders can mark
the selected member, NPC or enemy, clear that unit or clear all. Nonleaders see
read-only controls. Exact painted owner/revision/selection, pending requests and
latest target intent protect actions; no optimistic assignment or implicit
selection, movement, facing or attack. Escape closes the palette before clearing
the target; focus loss closes it. Names come from admitted/authored content.
Ordinary Bevy ImageNode widgets display marks above current visible units and on
the selected target frame. Original code-defined symbols avoid font-glyph or
external-art dependencies. Overhead rendering requires current loaded identity,
visible rendered transforms, static-world line of sight, a 60-metre camera range
and viewport bounds; it publishes no extra positions. Distant member marks still
appear in the selected-target frame. Promotion preserves marks; departure and
party dissolution remove them through the accepted authority lifecycle.
295 client tests pass (four skips), including stale identity/selection/leader,
loaded-observation and distinct symbol-image checks. Strict all-target client
Clippy, fmt, client build and Python compilation pass:
build/party-marks-ui-checks.json. Server/editor compatibility is unchanged from
the matched cda0608a builds and protocol acceptance.
Actual two-account native mouse/keyboard runs pass at 1720x1400 and 1720x700,
fresh and after PostgreSQL/server restart. Each flow covers 27 actions (14 mark
commands plus 13 membership actions), all eight shared NPC symbols, member marks,
reassignment, clear-unit/all, disabled nonleader commands, target-frame symbols,
unchanged movement/facing/combat/quests/bags, promotion retention and removal.
The final 244px palette is measured not to overlap the player frame. Evidence:
build/party-marks-ui-paired-v4/qualification.json (213.31 seconds) and
build/party-marks-ui-quadrants-v4/qualification.json (205.28 seconds).
Final palette captures at both heights were visually inspected. Linux RX 7900
XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6; zero GPU diagnostics and scene-readiness
retractions. The quadrant run uses two accounts in two quarter-screen tiles;
no four-account gameplay or native Windows claim. Early quadrant attempts picked
an offscreen marker anchor or had incorrect wrapper fixture paths. The first
passing UI exposed an eight-pixel player-frame overlap; narrowed layout v4 is the
acceptance run. Editor account playtests inherit the controls; native evidence
uses an editor-exported package. Next: the adopted loading/title/character-
selection flow, starting with genuine world readiness. Continue locally until
Stop; no push.

Next recommended slice: P2-02/P2-08 client loading overlay tied to actual world
and pipeline readiness before the larger P3-01/P3-03 title/character-selection
flow. Read docs/CHARACTER_LAUNCHER_MENU.md. Existing sources include
apps/client/src/capture_ready.rs, authored_scene.rs and network.rs SceneState.
Do not mask recurring mid-play readiness loss or invent percentage progress.
No native processes remain from this qualification. The until-Stop goal is active.

---

# Current checkpoint — 16 November 2026: shared unit-mark authority accepted

16 November 2026 — P4-11 shared unit-mark authority accepted:
Party profile v5 / PQ02 / PS03 carries eight bounded, party-private symbols.
Leaders can assign, clear a unit or clear all; reassignment preserves one symbol
per unit and one unit per symbol. Requests validate exact current members or
relevant living enemies/available NPCs, with existing revision, rate and replay
guards. Marking changes no selection, movement, facing or attack. Marks survive
interest loss, promotion and member death/respawn; creature death, NPC retirement,
member departure and party dissolution clear the affected marks. Creature respawn
uses a fresh identity. Leader-only permissions and range policy are explicit
Theia decisions, not claimed Classic rules. No extra positions or stats replicate.
407 core, 905 network (36 skips) and 292 client tests (four skips) pass. Actual
three-peer encrypted flows prove sharing, outsider isolation, permissions,
reassignment/clear and unchanged poses/selection. World tests cover lifecycle at
20/30/60 Hz. Strict all-target core/network/client/server/editor Clippy, fmt and
matching client/server/editor builds pass: build/party-marks-checks.json and
party-marks-*-tests.log. The inventory admission test now stops its synthetic
owners after the server, fixing a fixture-only shutdown race exposed by the suite.
This is the protocol prerequisite; no palette or rendered symbols yet, and no new
native GPU or Windows claim. Next: marker controls, overhead and target-frame
symbols, with native two-account checks at both tile heights. Editor playtests
inherit the matched protocol. Continue locally until Stop; no push.

---

# Current checkpoint — 16 November 2026: member selection UI accepted

16 November 2026 — P4-11 selected-member presentation accepted:
Left-click an exact current member frame to select that member through TG02.
The selected frame shows the trusted name and authoritative health/mana, and the
compact frame gains a white selection border. Member widgets stay separate from
hostile ability eligibility and enemy health; NPC selection replaces the member
presentation. Latest requested target identity blocks an old hostile observation
from receiving an action-bar spell while a newer selection/clear is in flight.
Right-click menus and UI drag ownership remain intact. Promotion retains the
selection; Escape/Backspace and membership departure clear it. No invented member
portrait, level or status, no remote-player mesh-click targeting, no shared marks.
292 client tests pass (four skips), including epoch/membership/identity guards,
partial/zero/unavailable resource repaint and the old-hostile intent race. Strict
all-target client Clippy, fmt, matching build and Python compilation pass:
build/party-target-ui-checks.json. Network authority remains qualified by 77052381.
Native two-account mouse/keyboard controls pass at 1720x1400 and 1720x700, both
fresh and after PostgreSQL/server restart: eight member selections and all 13
membership actions per flow, real rendered names/bars, NPC replacement without
dialogue, no implicit movement/facing/cast, clear/promotion/removal lifecycle,
unchanged quests/bags and zero scene-readiness retractions/GPU diagnostics.
Evidence: build/party-target-ui-paired-v3/qualification.json (178.84s),
build/party-target-ui-quadrants-v4/qualification.json (171.95s).
Both accepted selected-member captures were visually inspected. Linux RX 7900
XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6; no native Windows run. Quadrant evidence
uses two accounts in two quarter-screen tiles, not four-account gameplay.
Earlier attempts stopped at harness focus/fixture/output mistakes or an NPC click
covered by the target HUD. The final harness chooses a visible unobscured NPC.
Editor account playtests inherit these controls without additional options.
Next: party-scoped authoritative shared unit marks, then the adopted title,
character-selection and loading flow. Continue locally until Stop; no push.

All validation/native supervisors completed and cleaned up. Use `git log -1`
for this local commit; nothing pushed. Continue the user's until-Stop instruction.
For shared marks, read the current PARTIES reference and engineering records;
identify explicit Theia permissions, bounded party-private state, exact target
incarnations, unique marker reassignment, clear/despawn/departure/dissolution
cleanup and actual encrypted party/outsider isolation before adding the palette.
Keep marker requests separate from selecting or attacking. Avoid exposing extra
positions or silently making players valid hostile targets. Qualify the protocol
and actual two-client rendering/controls before claiming shared markers work.
The title/character-selection/loading flow remains requested following work.

---

# Current checkpoint — 16 November 2026: friendly selection authority accepted

P4-11 targeting profile v2 / TG02 adds `selected_member` separately from hostile
`selected`/`targets`. The server accepts only exact current other party members,
across the bounded world, without new position/stat disclosure. Selection stops
prior autoattack and leaves pose/facing unchanged. Existing cast/melee validation
rejects member targets. Death/respawn/promotion preserve selection; leave/kick/
dissolution/disconnect clear it. See STATUS/PARTIES/DECISIONS for explicit policy.
903 network tests pass (36 skips, including 544 library passes), 289 client tests
pass (four skips). Strict network/client/server/editor all-target Clippy, fmt and
matching builds pass. `build/party-selection-checks.json` and associated logs are
complete; the strengthened distant-request/kick lifecycle test passed separately
in `build/party-selection-distance-test.log`. No rendering changes or new GPU/
Windows evidence in this protocol prerequisite. Supervisor completed. Use
`git log -1` for this local commit; nothing pushed.

Continue until Stop. Next: left-click a compact member frame to submit the
existing target-choice command with its exact current member identity; consume
`targeting.selected_member` only with matching current party membership/epoch.
Show trusted name and actual private health/mana in the selected target frame;
keep hostile ability controls/Tab lists distinct. Use `world_input::Selection`
carefully: its current `friendly` is NPC-only and local, while this new member
selection is server-confirmed. Enemy/NPC selection, Escape/Backspace and departure
must cleanly replace/clear it. Preserve mouse drag ownership and manual facing;
never submit a damaging ability to the previous hostile while selecting a friend.
Inspect `targeting.rs`, `combat.rs`, `combat/ui.rs` and `world_input.rs` before
adoption. TargetVitals and the current target HUD assume hostile health or an NPC
name; do not fabricate member level/status/portraits. Qualify actual two-account
click/clear/switch/lifecycle, rendered real bars and no steering/attack at both
1720x1400 and 1720x700, then local commit. Shared marks and title/selection/loading
remain following work. Mouse/display and recommended decisions are authorized.

---

# Current checkpoint — 16 November 2026: member context menus accepted

P4-11 right-click member menus now expose trusted name/role, leader-only
Promote/Remove and Manage/Close. Shared server command/pending/revision guards
apply. Escape/outside/focus dismissal and frame drag ownership are qualified.
289 client tests / four skips, strict Clippy, fmt, build and Python compilation
pass. Actual native two-account controls and PostgreSQL/server restart pass at
both tile heights: `build/party-menu-{paired,quadrants}-v2/qualification.json`,
160.22 / 156.95 seconds, zero GPU diagnostics/readiness retractions. Both menu
captures were visually inspected. V1 failed in the harness after a variable
collision; v2 is acceptance. Use `git log -1` for this local commit; nothing pushed.
All owned native supervisors completed and cleaned up.

Continue the user's until-Stop implementation/editor-integration authorization.
Next P4-11 slice: authoritative friendly party-member selection, distinct from
hostile target search/rendering/combat. Existing `targeting::Target` is hostile
only; do not insert players into it without classification. The current TG01
codec requires selected identity in the hostile list. Review a bounded selected
member field/profile update without leaking remote positions; preserve exact
incarnations, manual facing, no implicit attack, departure cleanup and explicit
out-of-interest policy. Existing autoattack only accepts live creatures; spell
validation and every client target consumer need review. Require actual encrypted
selection/lifecycle/isolation plus existing hostile regressions before UI adoption.
Then qualify frame selection with real private stats, shared unit marks, and the
adopted title/selection/loading flow. Recommended choices and mouse/display are
authorized; commit qualified slices locally and continue. Do not push.

---

# Current checkpoint — 16 November 2026: compact member frames accepted

P4-11 member frames now display trusted names, leader labels and authoritative
health/mana. Local prerequisite commit is `2401c48f`; use `git log -1` for this
frame commit. 286 client tests / four skips, fmt, strict Clippy and build pass.
Native two-account paired and quadrant-sized controls and database restart pass:
`build/party-frames-{paired,quadrants}-v2/qualification.json`, 147.80 / 149.06
seconds, zero GPU diagnostics/readiness retractions. Captures visually inspected.
V1 exposed a collapsed name column; explicit column widths plus actual glyph and
height checks fix it. V1 is not acceptance. See STATUS/PARTIES for proof limits.
All owned native supervisors have completed and cleaned up. Nothing pushed.

Continue the user's until-Stop implementation/editor-integration authorization.
Next P4-11 slice: member context menu from right-click on a compact frame, with
trusted name, existing leader Promote/Remove actions, shared pending reservation,
painted owner/revision/member guards, Escape/dismissal and pointer ownership.
Qualify both tile heights and membership changes before the next local commit.
Then continue member targeting/marks and the adopted title/selection/loading
references. Use recommended decisions and mouse/display without reasking.

---

# Current checkpoint — 16 November 2026: private member vitals foundation

P4-11 party profile v4 / PS02 adds authoritative private member health/mana.
541 network tests (18 skips), 284 client tests (four skips), matching client/server/
zone-editor build, fmt and denied-warning Clippy pass. See STATUS and
`build/party-vitals-checks.json`. No rendering changes in this prerequisite.
Use `git log -1` for its local commit; nothing pushed.

Next: compact upper-left other-member frames with real name/leader/health/mana,
explicit unavailable state and lifecycle cleanup, preserving the existing Party
management controls. Qualify readable native 1720x1400 and 1720x700 presentation
and pointer ownership before committing; then continue targeting/marks and the
adopted front-end references. Until-Stop authorization remains active, including
recommended choices, editor integration, mouse/display and local commits.

---

# Current checkpoint — 16 November 2026: invitation prompt accepted

The compact incoming invitation is implemented and inherited by editor account
playtests. Native two-account controls/restart pass at both tile heights; all 284
client tests (four skips), build/fmt/strict Clippy and Python compilation pass.
See STATUS and `build/party-prompt-{paired,quadrants}-v1/qualification.json`.
The native supervisor exited cleanly. Use `git log -1` for this local commit.
Earlier tiling/editor support is local commit `fab0db0b`.

Continue under the user's explicit until-Stop authorization. Next P4-11 slice is
bounded private party-member health/mana replication as a prerequisite for real
member bars; reuse authoritative combat vitals, qualify lifecycle/isolation/codec
and encrypted delivery, then adopt compact member frames. Recommended decisions,
mouse/display use and local commits are authorized; pushing is not.

---

# Current checkpoint — 16 November 2026: tiled playtests accepted

Two 1720x1400 clients or four 1720x700 quadrants fit above the desktop panel.
Four editor slots, checked admission and native resize-before-placement now pass.
Evidence: `build/tiled-windows-native-v5/qualification.json`,
`build/tiled-windows-four-native-v3/qualification.json`,
`build/tiled-windows-party-v2/qualification.json` (282.58 seconds, actual OIDC
shared escort and durable restart, zero scene-readiness drops). See STATUS for
all tests/platform limits. Native harnesses completed and cleaned up.

The user explicitly authorized continuing implementation and editor integration,
using recommended choices and the mouse/display, making local commits after each
passing slice and continuing until they say Stop. Do not push. This supersedes
the historical paused instructions below. Next P4-11 slice: compact top-centre
incoming-party invitation with Accept/Decline while the Party panel is closed,
using the existing invitation identity/expiry/revision/pending guards. Native
checks must cover decline/cancel/accept, panel ownership and both tile heights.
Then continue the adopted party/front-end references in small qualified slices.
Use `git log -1` for the local tiling commit; no new title/loading/member-stat/
marking implementation is claimed yet.

---

# Current checkpoint — 16 November 2026

The reported NPC disappearance during the first quest approach and ranger handoff
is fixed. Startup had accepted only camera-visible world pipelines; new shader
preparation then revoked all ready chunks and hid contacts. The bounded authored
scene now prepares every mesh using temporary standard Bevy culling overrides,
restores ordinary culling, and retains genuine missing-asset safety.

Acceptance: `build/scene-pipeline-native-v1/qualification.json`, 283.92 seconds,
both native accounts through all quests/shared escort and PostgreSQL/server
restart. Both clients have 116 prepared pipelines and zero readiness retractions
in fresh and resumed phases. Revision 9 / XP 75 / bag revision 2 restore exactly.
All 281 client tests pass (four skips); explicit native Vulkan camera-sweep,
culling-restoration, mesh-loss/recovery and rebase test passes. Build, strict
all-target client Clippy, fmt and Python compilation pass. Logs:
`build/scene-pipeline-{tests,gpu,build,clippy,fmt}.log`. Linux RX 7900 XTX / RADV
NAVI31 / Vulkan / Mesa 26.2.6; no Windows execution. No push authorized.

The short diagnostic reproductions are NOT full-flow acceptance. The baseline
video shows the disappearance; `build/scene-pipeline-reproduction.json` records
the 0.505-second contact/readiness withdrawal with all CPU/GPU assets retained.
The user asked whether a loading screen would help; answered yes for initial
startup, but none was requested for implementation or added. Continue taking the
user's issues before the recorded party UI layout pass. Use `git log -1` for the
local fix commit; earlier handoff context follows.

---

# Earlier shared-escort checkpoint — 16 November 2026

The 15 November remaining shared-escort rerun is complete. Native
`build/party-escort-native-v2/qualification.json` passes in 295.44 seconds,
including both clients' actual background rendered movement, range handoff,
all-away waiting, individual arrival/turn-in and server/PostgreSQL restart.
Both characters restore journal revision 9, XP 75 and bag revision 2. There are
135 motion samples, with 23/57 distinct background rendered positions across
the two driving legs. Waiting/Ready captures were inspected. Linux RX 7900 XTX /
RADV NAVI31 / Vulkan / Mesa 26.2.6, zero GPU or prediction-binding diagnostics;
no Windows execution.

Fresh 538 network library tests pass (18 explicit skips), format and denied-warning
all-target network/client/server/editor Clippy pass, and all four changed Python
helpers compile. Logs: `build/party-escort-resume-{network-tests,fmt,clippy}.log`.
The earlier build and all-rate/death evidence below remain applicable; no Rust
implementation fix was needed during this resume. The first foreground test
rerun was interrupted; the tracked detached rerun completed with exit 0.
Both tracked supervisors (62609 native, 64965 network) have exited. No harness
is left running. All build artifacts stay ignored and no push is authorized.

STATUS, ROADMAP, ALIGNMENT, CODEX_TASKS and PARTIES record final acceptance.
This checkpoint accompanies the local shared-escort commit; use `git log -1`
for its hash. Previous committed work and original implementation notes follow.

The user supplied three party UI/menu screenshots and then said there are issues
to detail. Ready to receive those issues; hold further feature work for that input.
The screenshots' compact upper-left member frames, collapsible group panel and
member context menu are recorded in PARTIES as reference, not implemented UI.

---

# Historical handoff — 15 November 2026

Work stopped at the user's request. No push occurred. All changes below are saved
on disk. Owned native test harnesses exited. Resume implementation only when the
user asks; logging off does not authorize unattended work.

## Last committed work

Branch: `dev/quest-inventory-integration`.

- `96d0f9cf`: playable Party [O] controls and authenticated party membership.
- `1b840e53`: fix movement prediction to bind the complete Classic / optional
  chat / party identity. Earlier stationary party tests missed this bug.
- `3d2461bc`: enable parties in the editor's owned account playtests. Native
  export/create/select, all three travel quests, Stop and durable restart passed
  in `build/party-editor-native-v2/qualification.json`.

## Saved but uncommitted

Shared active party escorts: same party + same committed Active quest shares one
ranger position. Any living participant within the authored range keeps her
moving; none in range stops her. Each character must arrive and speak to her for
their own durable completion/reward. Unaccepted/Ready/Completed contacts remain
private. Late acceptance/merging adopts furthest checked route progress;
leave/dissolution forks in place; disconnect removes the incarnation. Dead
members observe but cannot drive or receive arrival credit. Party profile v3
binds this behavior. Policy is recorded in DECISIONS/PARTIES; shared XP/loot and
general quest credit are not implemented by this slice.

Changed files: `crates/network/src/party_wire.rs`,
`crates/network/src/simulation/follow.rs`,
`crates/network/src/simulation/combat/player_life/tests.rs`,
`docs/DECISIONS.md`, `docs/PARTIES.md`, `tools/party_os.py`,
`tools/qualify_quest_travel_two_players.py`, `tools/quest_travel_os.py`;
new `tools/party_escort_os.py`. Preserve these changes.

## Evidence and exact remaining check

- 538 network library tests pass, 18 explicit skips:
  `build/party-escort-network-tests.log`.
- Shared movement/range/arrival/lifecycle tests pass at 20/30/60 Hz; actual
  player-death authority test passes (`party-escort-death-tests.log`).
- Build, formatting and denied-warning all-target network/client/server/editor
  Clippy pass: `build/party-escort-{build,fmt,clippy}.log`.
- Native two-account shared escort, range handoff, everyone-away waiting,
  independent arrival/turn-in and PostgreSQL/server restart pass:
  `build/party-escort-native-v1/qualification.json`, 313.57 seconds.
  Both characters restore journal revision 9, XP 75, bag revision 2.
  Linux RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6, 1920x1440 each;
  no Windows execution claim.
- AFTER that native run loaded its helpers, the helpers were refined to record
  actual rendered ranger positions in both clients and assert background render
  movement. Shared-escort setup now uses just invite/accept (two actions), rather
  than repeating all 13 membership-control actions. Python compilation passes,
  but these latest helper refinements HAVE NOT been run natively.
- `fresh/party-rendered-observations.json` was sampled too late, after the ranger
  stopped at the destination. It does NOT prove moving presentation.

Next: rerun the native qualifier with `--party-escort` into a fresh v2 output
folder, inspect captures and rendered-motion samples, fix any failures, update
STATUS/ROADMAP/ALIGNMENT/CODEX_TASKS/PARTIES with final acceptance, then make a
local commit. Do not claim the latest helper assertions have already passed.

## Commands / environment

Use `cargo ... --offline -j 2`; network tests must use `--lib` to avoid linking
all examples. Native build environment:

```sh
export PKG_CONFIG_PATH="$PWD/build/native-sdk/usr/lib/x86_64-linux-gnu/pkgconfig"
export PKG_CONFIG_SYSROOT_DIR="$PWD/build/native-sdk"
export LD_LIBRARY_PATH="$PWD/build/native-sdk/usr/lib/x86_64-linux-gnu"
```

Native rerun:

```sh
python3 tools/qualify_quest_travel_two_players.py \
  --package build/editor-named-road-account-v1/fresh/package \
  --players build/classic-directional-client-qualified-v1-20261103/art/player \
  --fantasy-ui '/home/steven/Downloads/Synty Models/HUDs' \
  --output build/party-escort-native-v2 --party-escort
```

Use escalated native desktop/owned loopback execution. Launch long qualifiers via
tracked detached Popen with start_new_session=True, stdin DEVNULL and external
log/PID files; foreground exec may be killed around 300 seconds. Do NOT precreate
the output directory: the qualifier owns its creation. Run only one native UI
harness at a time. Diagnostics JSON is written nonatomically; retry partial reads.
Never weaken Desktop focus/pointer ownership checks. Sandbox /proc may not expose
escalated processes; use an escalated read to check tracked PIDs. Stop only owned
processes. All referenced artifacts are local ignored build outputs.

Keep changes local. Git metadata writes require escalation. Do not indiscriminately
clean build outputs (approximately 38 GB free at stop). Read AGENTS and required
architecture/status/alignment documents before resuming feature edits.
