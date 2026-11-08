# Zone authoring — 8 November 2026

The current user direction prioritizes terrain, greyboxing and zone authoring.
Chat composer and speech bubbles are deferred. Rust/Bevy/Avian/Renet remain the
adopted stack. The editor is a separate standard Bevy app, independent of normal
multiplayer startup, with a renderer-free `theia-zone` document/command crate.
This is P5-17 / P3-15 / P1-10; later terrain integration also uses P2-06/07.
P3-17/18 remain unspecified.

Actual prerequisites for this slice are the existing pinned Bevy/Avian stack,
checked coordinate conversion, static query scene and shared capsule solver.
P5-16 and population/region work do not block a bounded offline authoring tool.
Normal game networking remains Renet; editor playtesting is an offline sandbox
and does not establish server authority, multiplayer or deployable zone content.

## First slice contract

- Schema 1: named zone, immutable global f64 origin, global f64 player-foot spawn,
  stable positive object IDs and named box/sphere/cylinder placements. Y-up metres,
  yaw in degrees, positive dimensions in metres. Spheres are uniform; cylinders
  have equal X/Z diameters. No arbitrary scripts, mesh paths or network uploads.
- Bound 1,024 objects, 128 edits per transaction, 64 full-document undo entries and
  64 redo entries, 1 MiB document/command file, 1,024 startup commands. The bounded
  authoring area extends ±512 m from the origin with conservative object margins.
  It is a finite editing area, not a streamed world or a performance claim.
- Commands validate a candidate before replacing the document. Failures preserve
  document, monotonic revision and both history branches. Undo/redo require the
  current revision. A successful branch clears redo. Reload is an undoable edit.
  Unknown fields/schema, IDs, dimensions, nonfinite/out-of-bounds positions refuse.
- Snapping is zone-origin-relative f64 arithmetic. Rendering/physics subtract the
  global origin through existing `LocalFrame` conversion before narrowing to f32.
- Visual drag previews do not alter the document. Release commits one validated
  transform; focus loss or release over the panel cancels it. UI palette/keyboard
  and structured agent commands use the same `Editor::execute` entry point.
- Save validates and syncs a same-directory temporary file, then renames. Refusal
  preserves the existing destination. Load reads bounded bytes and validates before
  replacing. Filesystem operations are explicit local editor actions; no I/O is
  added to the authoritative game simulation.
- F5 freezes editing and builds static Avian shapes from the current document,
  checks capsule spawn clearance and uses the existing shared controller at 30 Hz.
  The implicit ground is a 1024 m square at local Y=0. Returning retains document
  and history. Moving objects/characters are never added as physical blockers.

## Run and command interface

```bash
cargo run -p theia-zone-editor -- --commands content/zones/greybox.commands.jsonl
cargo run -p theia-zone-editor -- --zone build/zone-draft.json
cargo run -p theia-zone-editor -- --headless \
  --commands content/zones/greybox.commands.jsonl --output build/my-zone.json
```

The existing Linux SDK environment may be required; see `docs/VALIDATION.md`.
Default save/load path is `build/zone-draft.json`; an explicit `--output` or
`--zone` supplies the path instead. Failed command scripts exit before writing
output or opening a window. `--capture build/editor.png` captures actual GPU
pixels after 90 frames and exits; it is a render smoke path, not interaction proof.

Each JSONL line is a command, e.g.:

```json
{"command":"apply","expected_revision":0,"edits":[{"op":"place","object":{"id":1,"name":"Block","primitive":"box","pose":{"position":[0,0.5,0],"yaw_degrees":0,"size":[1,1,1]}}}]}
{"command":"undo","expected_revision":1}
{"command":"redo","expected_revision":2}
```

Other edits are `transform` (`id`, full `pose`), `delete` (`id`) and `set_spawn`
(`position`). Startup command history is retained for visual undo/redo. Revision
starts at zero for each freshly opened document and is not persisted identity.

Palette buttons / 1–3 choose placement; click the ground to place. Escape returns
selection mode. Click an object to select; drag it on a second press to move X/Z.
Arrows move X/Z, PageUp/Down move Y, R/Shift-R rotate ±15°, +/- scale uniformly.
G toggles one-metre snap; Ctrl-Z/Y undo/redo; Ctrl-S/O save/load. WASD pan, RMB
orbits and wheel zooms. F5 enters/leaves playtest; W/S forward/back, A/D turn,
Q/E strafe and Space jumps. This sandbox uses the retained development controller
speed policy; it is not a new Classic gameplay movement configuration.

## Acceptance and next work

Required checks: transaction rollback/stale revisions/history branching and bounds,
far signed coordinate precision, save/load/refusal, actual Avian rotated shape /
shared capsule contact and blocked spawn, formatting and strict workspace Clippy.
Native Linux checks must include actual rendered pixels and pointer/keyboard
placement, transform, undo/redo, save/load and playtest transitions. Record executed
results below; Windows cross-builds and native execution remain separate evidence.

Next: stronger transform handles/property widgets,
explicit playtest export into normal server content admission, terrain samples and
brush undo, paths/streams, deterministic vegetation overrides, NPC/quest markers.
Do not call the offline sandbox authoritative multiplayer. Terrain brushes should
reuse the shared validation/history boundary; avoid a second editing engine.

## Implemented and qualified first slice

The first source slice implements the contract above with standard Bevy meshes,
PBR, UI widgets, mesh raycasts and camera/input infrastructure. An original
five-object JSONL greybox fixture is tracked under `content/zones/`. There are no
new third-party pins, game-protocol changes or imported/private art assets.

Executed checks:

- Seven `theia-zone` tests pass: atomic rejection preserving redo/revision,
  serialized grouped commands/history branching, undoable validated reload,
  history/transaction budgets, signed billion-metre centimetre precision and
  snapping, invalid geometry/reference rejection, save/replace/error preservation.
- One editor integration test passes using actual Avian: a shared moving capsule
  slides against a 15-degree authored wall at expected wall-local clearance;
  sphere/cylinder sight blocking and penetrating-spawn refusal also pass.
- Actual headless CLI passes grouped placement/undo/redo, stale command rejection,
  strict unknown-field refusal and preservation of existing output on failure.
- `cargo fmt --check`, `git diff --check` and locked denied-warning workspace /
  all-target Clippy pass, using the existing local Linux SDK environment.
- The native OS-input walkthrough passes at 1280x800, 1280x720, 1920x1200 and
  actual fullscreen 3440x1440. Each successful run checks actual palette
  placement, drag preview/release as one transaction, translation/elevation/yaw/
  uniform scale, keyboard undo with widget redo, delete/restore, exact save/load
  with undoable reload, snap toggles and F5 shared-Avian walking/return. Actual
  Vulkan screenshots are captured and the owned process closes normally.
  Successful logs contain zero GPU diagnostics (including resize exceptions).

Native device: Linux, AMD Radeon RX 7900 XTX, RADV NAVI31, Vulkan, device ID 29772.
Evidence: ignored `build/zone-tests-final-v1.log`, `zone-clippy-final-v1.log`,
`zone-cli-check.py`, `zone-native-v3/` and `zone-native-{1280x720,1920x1200}/` and `zone-native-3440x1440-v2/`.
`tools/qualify_zone_editor.py` repeats the actual OS-input walkthrough and uses
read-only opt-in `--report FILE` telemetry to obtain projected object/button
positions. In that mode F12 captures `FILE` with its extension replaced by `.png`.
Report output is explicit development tooling, outside the fixed-step controller.

Retained failed attempts: the first shell lacked the documented native SDK env;
initial compile attempts caught Bevy API/visibility mismatches; the first collision
assertion incorrectly used world Z for a rotated wall and was corrected to
wall-local clearance. Native v1 exposed the LUT-dependent default tonemapper and
release-activated widget setup; the app now explicitly uses standard
ReinhardLuminance, continuous Winit updates and ActivateOnPress. Native v2 reached
save but its driver incorrectly assumed deleting/undoing retained selection;
the final driver explicitly reselects the restored object. None of those failed
attempts is counted as qualification. The first ultrawide window resize was
limited by desktop decorations to 3440x1368; the final fullscreen run verifies
actual 3440x1440 pixels.

Limits: this is a finite local primitive editor and offline playtest. Arbitrary assets, draggable transform handles/property fields, terrain
brushes, paths/streams, vegetation/markers, normal server content export/admission,
multiplayer, performance/capacity, non-unit monitor DPI and Windows execution
remain open. Startup commands and visual edits already share one history.

Qualified debug editor SHA-256: `e4a6f6f967543dfc0514645a7ac980b55e455c0f6e56b1c6ff92d99b61b53a9f`.

## Live local agent mailbox (P5-17 / P3-15)

`--mailbox build/zone-mailbox` opts an open editor into a bounded single-slot
filesystem handoff. Use one editor and one command producer per directory.
No listener, service, socket, scripting runtime or game-protocol change is added.
Session matching prevents stale files from applying to a later opening; the
session string is not authentication. Access is governed by local filesystem
permissions and explicit opt-in. This is local development tooling.

The editor publishes `state.json` with schema 1, a fresh `session`, current
`revision`, undo/redo counts, `busy` and the full validated `document`. Publish a
request to a temporary file in the same directory and rename it to `request.json`:

```json
{"session":"COPY_FROM_STATE","request_id":1,"command":{"command":"apply","expected_revision":15,"edits":[{"op":"place","object":{"id":100,"name":"Agent block","primitive":"box","pose":{"position":[3,0.5,4],"yaw_degrees":0,"size":[1,1,1]}}}]}}
```

Wait for `receipt.json` with the matching session/request ID before submitting
another request. Receipt fields are `session`, `request_id`, `accepted`,
`revision` and nullable `error`. Success is an editor-history entry; save and
normal authoritative server content publication are separate operations.
State/receipt files can remain after exit and are not proof of a live process.
A fresh app opening publishes a new session and starts a new in-memory history.

Requests require positive increasing IDs and exact current editor revisions.
The most recent exact parsed request is cached with its original receipt, so a
retry cannot reapply after a visual undo. Reused/older IDs with other content
refuse. Only the latest receipt is cached; older receipts are not a replay log.
Semantic refusals also consume their request ID; use a new ID after correcting
stale state or waiting for a drag/playtest to finish. Malformed or old-session
requests do not consume the current session's request sequence.

The consumer claims at most one request per 200 ms by renaming it to its own
session-specific processing file, reads at most 1 MiB plus a sentinel, validates
through `Editor::execute` and publishes the decision/current document. A producer
must not replace a pending request. File I/O runs in the editor update, outside
fixed-step playtest and the game simulation. Slow local storage can stall this
development editor; no asynchronous throughput/performance promise is made.
If receipt publication fails after an accepted edit, the pending receipt is
retained and publication is retried before consuming another request. The edit
is not reapplied. This is in-process recovery, not crash-durable command history.
Crash processing remnants are not automatically scanned or replayed.

Requests refuse during visual drag previews or playtest. New agent object IDs
advance the visual allocator; selection clears when a command removes its object.
An agent placement can be visually undone, agent-redone and visually transformed
using the same validated history. No second mutation path is introduced.

Four additional core cases pass: exact retry across intervening visual undo,
wrong-session/stale/busy/malformed refusal without mutation, actual file claim /
state/receipt publication, and publication-failure recovery without reapplying.
The focused Rust suite is now 11 zone + one actual Avian editor case. Locked
workspace/all-target denied-warning Clippy, formatting, diff checks and build pass.

The actual 1280x720 Linux/RADV/Vulkan mixed-agent walkthrough passes with zero
GPU diagnostics. It repeats the first-slice visual/file/playtest checks, then
verifies live agent placement, keyboard undo, exact retry after undo without
reapplication, agent redo, widget undo, stale revision refusal, visual allocation
above the agent's object ID 100 and live refusal during F5 playtest. The resulting
revision is 21, with the intended six-object document restored. Owned app/window
closes normally. Evidence: `build/zone-mailbox-tests-v2.log`,
`zone-mailbox-clippy-v1.log`, `zone-mailbox-build-v1.log` and
`zone-mailbox-native-v1/{qualification.json,state.png,client.log}`.
Repeat with `tools/qualify_zone_editor.py --output build/zone-agent-proof
--size 1280x720 --agent` using the existing asset-tools Python environment.
No new rendering code was added in this continuation; the original three-size
visual qualification remains scoped to its recorded source/binary.
