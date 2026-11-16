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
