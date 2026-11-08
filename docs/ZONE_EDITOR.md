# Zone authoring — 8 November 2026

The current user direction prioritizes terrain, greyboxing and zone authoring.
Chat composer and speech bubbles are deferred. Rust/Bevy/Avian/Renet remain the
adopted stack. The editor is a separate standard Bevy app, independent of normal
multiplayer startup, with a renderer-free `theia-zone` document/command crate.
This is P5-17 / P3-15 / P1-10; later terrain integration also uses P2-06/07.
P3-17/18 remain unspecified. The current tool includes primitive authoring,
shared visual/live-agent history, bounded terrain brushes, road/stream guides and validated export
into the existing world-package pipeline. Milestone records below retain their
exact source/evidence scope. The current tool targets **3440x1440 only**, opening
borderless fullscreen on the primary display. Smaller-window checks below are
historical evidence; they are no longer supported qualification targets.

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

Next: explicit playtest export into normal server content admission, stronger
transform handles/property widgets, multi-tile terrain, paths/streams, deterministic vegetation overrides, NPC/quest markers.
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

Limits at the first-slice checkpoint: this was a finite local primitive editor
and offline playtest. Arbitrary assets, draggable transform handles/property fields, multi-tile terrain, paths/streams, vegetation/markers, normal server content export/admission,
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

## Bounded terrain brushes (P5-17 / P3-15 / P1-10 / P2-06/07)

The bounded terrain continuation is implemented and qualified. Zone schema 2 optionally contains
one `terrain` patch: integer `chunk: [x,z]`, 33x33 Z-major f32 `heights`, 128 m
square / 4 m sample spacing. Chunk lower corners use the existing integer
`ChunkId::origin(128)` convention. Heights are relative to the zone's global f64
origin Y and bounded to +/-64 m. The patch must fit the finite authoring frame.
All border samples remain zero to meet the implicit flat surroundings. This is
one editable height patch, not multi-tile authoring, streaming or a normal server
terrain package. Schema-1 primitive zones retain their format and remain readable.

`set_terrain` creates/replaces/removes a patch and upgrades the document to schema
2. `brush` carries a global f64 `centre: [x,z]`, `radius_metres` in 4..32 and an
`operation`: `raise`/`lower` with `metres` in 0.05..8, `flatten` with local `height`
and `blend` in 0.05..1, or `smooth` with `blend`. Radial linear falloff and
four-neighbour smoothing are explicit Theia authoring choices. Smoothing samples
one immutable prior grid per stamp. Invalid/no-op/out-of-patch/over-height strokes
refuse atomically. The fixed border is excluded from every brush.

F1/F2 switch between object and terrain palettes. Creating terrain focuses its
128 m patch. The terrain palette provides create/remove, raise/lower (2 m per
stamp), flatten to local zero, smooth, radius +/-4 m, undo/redo, save/load and
playtest. A drag stamps on pointer movement of at least 2 m; stationary holds do
not accumulate changes at a render-dependent rate. At most 128 stamps fit one
stroke. Each preview stamp is validated in a disposable `Editor`; releasing
replays the same command transaction once into live history. Escape, focus loss
or release over the sidebar cancels the preview. Agent requests refuse during it.

Rendering and Avian use the existing 32-cell procedural grid topology with every
authored sample. Standard Bevy mesh normals/PBR and ordinary static Avian triangle
meshes present/query it. The surroundings are four ordinary cuboids with a true
patch cutout, so lowered terrain has no hidden flat collider or render plane.
Reusable terrain surface/grid mesh handles update for previews/undo/reload; no per-stamp
asset accumulation is introduced. Flat grid lines hide while terrain is active. A standard line-list mesh follows
the authored grid in edit mode, and active palette tools are highlighted.
Primitive placement and the spawn-at-view action sample the same triangle surface.
F5 validates a capsule spawn and uses the same existing shared movement solver.
No file I/O or terrain authoring enters the normal authoritative simulation.

Example structured terrain fixture:

```bash
cargo run -p theia-zone-editor -- \
  --commands content/zones/terrain.commands.jsonl --output build/terrain-zone.json
```

The fixture creates a hill and depression. Terrain file values are f32; JSON may
print shorter decimal representations than expanded telemetry. Comparisons must
use the declared sample precision, not require equal f64 decimal spellings.

Four new domain tests pass for all brush operations/grouped undo/schema round
trip, complete rollback after a stroke exceeds the height budget, exact triangle
interpolation/far signed chunk binding, and malformed borders/topology/extents.
An actual Avian test checks every triangle centroid of two opposite-height patches
at signed far origins: 4,096 rays agree within 0.1 mm, including negative samples
with no hidden floor, plus shared capsule spawn checks. Combined focused suite:
15 zone + two editor cases. Fresh locked focused tests, denied-warning workspace /
all-target Clippy, formatting/diff checks and editor build pass.

Final actual Linux/RADV/Vulkan walkthroughs pass at 1280x720, 1920x1200 and
fullscreen 3440x1440 with zero GPU diagnostics. They repeat the primitive and live
agent checks and prove actual Escape-cancelled previews, one-entry brush drags,
live agent refusal during a brush, all four brush modes, radius changes, exact
f32 terrain save/load, spawn sampling, shared-capsule walking on the sculpted
surface and a live agent brush undone visually. Each final editor reaches
revision 32 and closes normally. Actual screenshots verify visible terrain grid,
palette controls and highlighted brush; these are synthetic OS-input checks,
not physical-device/Windows or capacity evidence.

Evidence: ignored `build/zone-terrain-tests-final-v2.log`,
`zone-terrain-clippy-final-v1.log`, `zone-terrain-build-v4.log`, and
`zone-terrain-final-{1280x720,1920x1200,3440x1440}/`. The new fixture also passes
actual `--headless` command execution (revision 2 / two grouped entries).
`tools/qualify_zone_editor.py --terrain` repeats the complete native workflow.
The first terrain native attempt is retained in `zone-terrain-native-v1/`: its
file check incorrectly required f64 decimal equality for saved f32 heights. The
corrected driver compares the declared f32 sample values; the file format was
not changed to accommodate that assertion. Native v2 passes before the final
grid/highlight continuation. Existing source/screenshot evidence remains scoped
to its exact version.

Follow-ups at the terrain checkpoint included multi-tile/seam authoring and
import/export into admitted game packages (the latter is implemented below),
terrain materials/LOD, configurable flatten heights/strength, path/stream tools,
vegetation and nonblocking NPC/quest markers. This patch does not modify the
existing 125x125 saved-terrain world or enable authoritative multiplayer zones.

## Export into admitted world packages (P5-17 / P3-15 / P1-10 / P2-07/08)

`--export NEW_DIRECTORY --content-id LOGICAL_ID --revision N` prepares an offline
world package and exits without a window. It applies optional startup commands
first, validates the zone, generates standard Bevy primitive/terrain meshes and
writes an ordinary self-contained GLB plus existing schema-4 collision data.
No gameplay format, transport, server/client runtime or default world is changed.
The source authoring document is copied as `authoring.zone.json`; `export.json`
records logical identity, source origin, counts and approximation tolerances.

```bash
cargo run -p theia-zone-editor -- --zone build/zone-draft.json \
  --export build/my-zone-playtest-r1 --content-id theia.zone.my-zone --revision 1
```

Output requires a new directory. Geometry and spawn checks run before publication
through the actual headless `Terrain::decode`/Avian/shared capsule path, then the
staged package passes `Terrain::load` and visual digest verification. An exclusive
destination reservation refuses races/existing output; verified artifacts move
into it with the required manifest last. Failure cleans only directories created
by this export. There is no registry enforcing revision uniqueness and no crash-
durable publication claim. Keep admitted files immutable and export a fresh
revision/directory after editing.

- Boxes retain analytic oriented collision. Spheres use the existing zero-length
  capsule representation; an actual decoder/query test confirms it works.
- Cylinders use a Bevy mesh with adaptive radial resolution targeting <=1 mm
  polygon error, then the existing convex-mesh collider representation. A collision
  copy welds split cap/UV/normal seams within 0.1 mm, retaining source positions
  rather than averaging them. The GLB keeps the original split render vertices.
  This is a bounded approximation, with additional f32 precision error; it does
  not change the editor's analytic cylinder controller or extend the runtime format.
- Terrain uses every authored sample and the same existing triangulation. Flat
  surrounding boxes retain the true cutout for depressions. Spawns remain explicit
  validated capsule-foot guesses. No navigation, object interaction or NPC physics
  is inferred from visual geometry.
- Geometry rebases through `Zone::local` before f32 serialization. The runtime
  profile is already local (`source_origin` zero); the exact f64 authoring origin
  remains in the source/report. Export does not automatically place or own an MMO
  region at that origin. The existing server `--world-origin` can instance the
  local package separately; native far-frame evidence uses that explicit setting.
- Visual preparation is capped at 262,144 triangles / 64 MiB. Existing schema-4
  vertex/triangle/body/coverage/profile bounds still apply and refuse oversized
  exports. It is valid for a document within the editor's object budget to exceed
  an export budget; nothing is silently decimated or dropped.

Run the ordinary imported-world development path with simple capsule players,
using fresh ticket directories (build the paired binaries first):

```bash
target/debug/theia-server 127.0.0.1:5001 build/zone-server-cert.der \
  --terrain-world build/my-zone-playtest-r1 --renet-tickets build/zone-tickets-r1
# Separate terminals; tickets are private, single-use development credentials.
target/debug/theia-client --terrain-world build/my-zone-playtest-r1 \
  --renet-connect 127.0.0.1:5001 build/zone-tickets-r1/client-1.token
target/debug/theia-client --terrain-world build/my-zone-playtest-r1 \
  --renet-connect 127.0.0.1:5001 build/zone-tickets-r1/client-2.token
```

This is the retained raw imported-world movement profile and development admission,
not the full Classic account/combat/quest bundle. The F5 editor sandbox remains
separate; export does not start services or change the normal game launcher.

Five export cases pass, in addition to the 17 prior zone/editor cases:
actual runtime mixed-shape admission/source round trip/identity without visual I/O;
invalid-spawn/metadata/visual-budget refusal with no output; far signed centimetre
preservation and negative terrain; closed cylinder collision topology plus byte-
deterministic repeated export; and two actual encrypted Renet clients moving in
the authored world, stopping at its wall and rejecting a stale package revision.
Fresh focused tests total **22**, with locked workspace/all-target denied-warning
Clippy, formatting/diff checks and builds. Actual CLI export with DISPLAY and
WAYLAND_DISPLAY unset passes; exporting over an existing package refuses and
preserves every artifact hash.

Both primitive and terrain GLBs pass Khronos validation with **zero errors and
zero warnings**. Independent float64 closed-convex source checks pass on the
welded cylinder collision copies. Four final native pairs (eight normal rendered
clients) pass on Linux RX 7900 XTX / RADV NAVI31 / Vulkan with validation-layer
activation proved in logs and no GPU diagnostics: primitive world, terrain world,
explicit 128-billion-metre instance, and direct hillside traversal. The hillside
fixture starts on actual nonzero terrain: one owner gains **1.4444 m** and the
other descends **0.1654 m** while both clients agree on committed poses; maximum
reported ground discrepancy is 0.0000001193 m. That is this short controlled
fixture, not a general numerical error/performance/capacity guarantee.

Evidence: ignored `build/zone-export-tests-final-v1.log`,
`zone-export-clippy-final-v2.log`, `zone-export-paired-build-v1.log`,
`zone-export-build-v3.log`, `zone-export-no-display-v1.log`,
`zone-package-{primitives,terrain}-v3/`, `zone-package-terrain-v4/`, corresponding
`zone-package-*-gltf-validation-v3/v4.json`, and
`zone-package-native-{primitives,terrain,far}-v3/` /
`zone-package-native-hillside-v4/`. The first native attempt's independent oracle
rejected un-welded split render vertices as an open collision source; it is
retained as a failed candidate and not counted. The final collision copy is
independently closed/convex. Initial flat-terrain traversal is retained separately
from the final hillside proof. `tools/verify_multiplayer.py --terrain-world DIR`
repeats the actual normal server/two-rendered-client check.

Exact paired debug SHA-256:

| Binary | SHA-256 |
| --- | --- |
| zone editor | `8f0c964c911b77b20053393a37ea42b625d0839e9d19fc39b2bd927d2f53bf72` |
| normal client | `3ee30b087260ca8cbfb4ba176ca969540eeb327a278159d9446470bfba7106b7` |
| headless server | `0e79e534335cd1718ba3f7d06b2622c537a90183e20f7a5809b6566b872dde89` |

Windows execution, production authentication for imported zones, persistence,
Classic gameplay-bundle integration, terrain streaming/LOD/materials, navigation,
NPC/quest publication and population/resource qualification remain open.


## Road/stream guides and 3440x1440 tool — 8 November 2026

P5-17 / P3-15 continue from the shared command history, terrain sampling and world
exporter. Schema 3 adds optional road/stream guides: positive stable IDs shared
with primitive objects, names, width 0.25..32 m and 2..64 explicit global f64
control points per guide, at most 128 guides. Adjacent points need at least
0.25 m horizontal separation and at most 256 m spatial distance. Width margins
must fit the existing bounded authoring area. Older schema-1/2 documents remain
readable. Whole-document validation also checks serialized size, preventing a
combined valid-count document from exceeding the 1 MiB save/load budget.

`put_path` upserts and `delete_path` removes through the same atomic command and
undo pipeline as objects/terrain. Failed geometry, cross-kind ID collisions,
old-schema guide data and oversized documents preserve state/history. The
allocator advances across both kinds, including agent-authored IDs.

F3 opens Guides; F1/F2 return to Objects/Terrain. Start a road or stream, click
control points on the sampled ground, Backspace removes the last point, Enter
commits one history entry, Escape cancels. Drafts block live agent mutation.
Ctrl-S saves committed data and explicitly leaves the pending draft unsaved.
Select Next cycles guides; arrows/PgUp/PgDn translate the whole selected guide;
width controls, Delete, undo/redo and load share normal history. Snapping uses
global f64 positions before bounded local conversion. The editor caches committed
and draft meshes and releases removed guide mesh assets.

These are straight nonblocking visual ribbons with explicit point heights. Roads
use standard PBR; streams use standard transparent unlit materials. They export
through the existing GLB pipeline and remain in the source-zone copy. They do not
carve/grade terrain, conform between points, establish navigation, simulate water
or create collision. Those remain separately scoped work.

Acceptance evidence: 19 domain and eight editor tests pass, including far signed
centimetre coordinates, schema/history/ID failures, combined byte limits and
render-only export. The export case checks unchanged collision and actual Avian
capsule movement across ribbons while package identity changes. Strict workspace
all-target Clippy and formatting pass. The headless mixed-guide package passes
Khronos GLB validation with zero errors/warnings.

The current user explicitly restricted tool development to 3440x1440. Startup
uses standard Bevy primary-monitor borderless fullscreen, with a 3440x1440 window
resolution. The OS-input qualifier now accepts only that viewport and verifies
the application's own startup without forcing a resize. This policy does not
change normal game-client display support. Native qualification uses Linux,
AMD Radeon RX 7900 XTX (RADV NAVI31), Vulkan device 29772. This is not Windows or
MMO-scale evidence.

Evidence: ignored `build/zone-path-tests-final-v1.log`,
`zone-guide-3440-clippy.log`, `zone-guide-3440-build.log`,
`zone-package-guides-v1/`, `zone-guide-gltf-validation-v1.json`.

The complete OS-input walkthrough passes at actual 3440x1440 with the fullscreen
default: primitive transforms/drag/snapping, shared agent history/retry/busy
refusals, terrain strokes, road/stream drafts/cancel/commit, whole-guide
translation/width/delete, shared ID allocation, save/load and offline playtest.
The Vulkan validation layer is observed active; zero GPU diagnostics or resize
exceptions were needed. Screenshot: ignored
`build/zone-guide-3440-default-v1/state.png`; structured evidence beside it.
Two retained guide meshes raise the CPU asset count from 17 to 19; delete/undo
and load return the expected counts. This is asset-lifetime evidence, not a GPU
memory/capacity claim. Earlier mixed-size guide runs predate the viewport policy.

The normal headless server and two actual rendered clients also pass encrypted
Renet traversal for the exported guide package, both at 3440x1440 with observed
Vulkan validation and zero GPU diagnostics. Actual package binding, distinct
player IDs, wall clearance and both avatar colours pass existing independent
checks. Evidence: `build/zone-package-native-guides-3440-v1/verification.json`
and both `scene.png` captures. `tools/verify_multiplayer.py` adds an optional
`--window-size 3440x1440` fullscreen mode while preserving its ordinary default.
The current editor SHA-256 is
`a816f15a1e4b57804ce1e9e17d8facfef99db569906d317b8072f948962fa14f`;
the normal client/server hashes remain those in the export milestone above.
