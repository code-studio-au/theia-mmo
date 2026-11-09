# Zone authoring — 8 November 2026

The current user direction prioritizes terrain, greyboxing and zone authoring.
Chat composer and speech bubbles are deferred. Rust/Bevy/Avian/Renet remain the
adopted stack. The editor is a separate standard Bevy app, independent of normal
multiplayer startup, with a renderer-free `theia-zone` document/command crate.
This is P5-17 / P3-15 / P1-10; later terrain integration also uses P2-06/07.
P3-17/18 remain unspecified. The current tool includes primitive authoring,
shared visual/live-agent history, bounded terrain brushes, road/stream guides,
NPC/quest annotations, greybox vegetation and validated export
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
G toggles one-metre snap; H grounds the selection; Ctrl-Z/Y undo/redo; Ctrl-S/O save/load. WASD pan, RMB
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

## NPC/quest authoring markers — 8 November 2026

P5-17 / P3-15 continue from the shared bounded document/history, checked terrain
placement and standard Bevy editor. Zone schema 4 adds at most 256 markers, each
with a shared positive authoring ID, name, kind (`npc` / `quest`), global f64
anchor, yaw, 0.25..64 m footprint and optional 1..96-byte ASCII catalogue key.
NPC anchors represent feet; quest anchors locate an authoring footprint. Keys
allow letters, digits, dots, hyphens and underscores. They remain unresolved
annotations; no arbitrary file path, live entity key or executable script is
accepted. Whole-document IDs, checked position/footprint margins, byte budget,
finite yaw and revision rules apply before any mutation. Schema-1/2/3 files remain
readable; subsequent terrain/path edits preserve schema 4.

`put_marker` upserts and `delete_marker` removes through shared command history.
The fixture `content/zones/markers.commands.jsonl` starts with revision 0; it can
be applied headlessly or adapted to a live mailbox's current revision/session.
F4 exposes NPC/quest placement, pointer selection, Select Next, Delete, footprint
controls, snapping and normal history/save/playtest actions. Click ground to
place, Escape returns to selection. Arrows/PgUp/PgDn translate; R/Shift-R change
yaw; +/- changes the selected footprint by 0.5 m. Ctrl-O loads the saved zone.
Visual placement samples the same terrain surface after X/Z snapping. Catalogue
keys/names can be set through structured commands and appear in the inspector.

Cyan NPC badges and pink quest badges have visible footprints/headings; selection
is amber. Three standard shared Bevy meshes/materials serve all markers. Deleted
entities are removed without allocating per-marker meshes. Markers hide during
offline playtest and return afterward. No creature, dialogue, quest objective,
trigger volume, navigation or collision is activated by these annotations.

Export preserves them in `authoring.zone.json` and counts them in `export.json`.
Runtime GLB/collision bytes and world identity stay identical when only markers
change. Marker metadata is not bound as runtime gameplay by the current manifest;
a future explicit catalogue compiler/publisher must validate/bind references
before a server can use it. This preserves the existing authoritative NPC/quest
contracts rather than treating placement notes as published gameplay.

Three new domain checks cover signed far-frame centimetres, round-trip data,
shared schema/history/allocator behavior, invalid key/geometry transactions,
redo preservation, duplicate IDs and the 256-marker cap. The export check proves
exactly unchanged runtime geometry/collision and preserved source metadata.
Together with retained cases, 31 focused Rust tests pass (22 domain / nine editor).
Strict workspace/all-target Clippy and formatting pass. Evidence: ignored
`build/zone-markers-tests-v1.log`, `zone-markers-clippy-v1.log`.

The full native OS-input walkthrough passes at default fullscreen 3440x1440 on
the same RX 7900 XTX / RADV Vulkan device. Marker placement, pointer selection,
translation/elevation/yaw/radius, delete/undo/redo, mixed agent IDs, save/load and
playtest pass alongside retained primitive/terrain/guide workflows. Six marker
entities hide during playtest and return afterward; the shared mesh count stays
constant through placement/delete/undo/load. Validation is observed active with
zero GPU diagnostics/resize exceptions. Evidence: ignored
`build/zone-markers-native-3440-v1/{qualification.json,state.png}` and
`zone-markers-build-v1.log`. Headless fixture save and real package admission also
pass without DISPLAY/WAYLAND_DISPLAY: `zone-markers-headless-v1.json` and
`zone-package-markers-v1/`. This does not qualify live NPC/quest publication or
Windows execution.

## Greybox vegetation placement/scatter — 8 November 2026

P5-17 / P3-15 continue from checked coordinates, shared commands/history and the
existing exporter. Zone schema 5 optionally contains at most 512 plants. Tree and
shrub proxies have shared positive IDs, bounded names, global f64 foot anchors,
finite yaw and uniform dimensionless scale 0.25..8. Conservative margins account
for the fixed metre-sized proxy geometry. Earlier schemas remain readable;
terrain/path/marker edits preserve schema 5. The overall byte budget still applies.

`put_plant` upserts, `delete_plant` removes and `scatter_vegetation` generates a
bounded stamp. Scatter names first ID, seed, kind, global X/Z centre, 1..32 m
radius and 1..64 plants. Explicit SplitMix64 arithmetic and bounded unit-disk
rejection sampling determine positions, yaw and scales 0.8..1.2 without global
RNG or frame dependence. Each point samples the exact triangular terrain support
through `Zone::surface_position`, falling back to flat zone-origin Y. ID overflow,
collisions, area/count/byte limits or invalid geometry refuse the entire command
without altering revision/history. One scatter stamp is one undo entry; redo
restores stored plants rather than rerolling. The sampler does not impose spacing,
biomes or slope exclusion, and existing vegetation does not move after sculpting.

F6 opens Plants. Place tree/shrub proxies individually, or choose scatter and click
once per stamp. The standard Bevy transparent footprint previews the stamp area.
`[`/`]` changes radius by 1 m and `,`/`.` changes count by four, within domain
bounds. Default: eight plants / 8 m. Escape selects; pointer/Select Next selects
plants; Delete removes. Arrows/PgUp/PgDn move, R/Shift-R yaws and +/- changes scale.
Snapping, save/load and visual/agent undo share the existing pipeline. Agent IDs
advance the visual allocator even after visual undo. Three shared meshes serve
the cursor, trunk and low-poly foliage; removed plants release their entities.

Tree trunks and spherical crowns, and ellipsoid shrubs, are static visual
proxies using normal Bevy meshes/materials. They remain visible during playtest
and export through the normal self-contained GLB pipeline. They add no collision,
navigation or harvesting/gameplay behavior. Source metadata is retained and the
visual digest changes. World collision remains exactly the authored primitives
and terrain. General art assets, vegetation collision policy, wind, biomes,
spacing, LOD/streaming and population capacity are separate work. The exporter
now applies the inverse-transpose normal transform for nonuniform mesh scale,
which the ellipsoid shrubs require; an emitted-data ellipsoid-gradient test
checks it independently.

Acceptance: 25 domain and ten editor tests pass. New cases cover deterministic
terrain sampling and grouped history, rollback/redo/ID overflow, far signed
centimetres, the 512-plant cap, source/export binding, unchanged collision and
ellipsoid normals. Formatting, strict workspace/all-target Clippy and Khronos
GLB validation pass (zero errors/warnings). Native OS-input qualification passes
at default fullscreen 3440x1440 on the same RX 7900 XTX / RADV Vulkan device,
with observed validation and zero GPU diagnostics/resize exceptions. It repeats
all primitive/terrain/guide/marker workflows, then exercises plant placement,
selection/transforms/delete, a twelve-tree scatter as one undo entry, exact redo,
agent retry/IDs, save/load and playtest. Shared mesh count stays constant.

Evidence: ignored `build/zone-vegetation-tests-final-v2.log`,
`zone-vegetation-clippy-v2.log`, `zone-vegetation-build-v1.log`,
`zone-vegetation-native-3440-v1/{qualification.json,state.png}`,
`zone-package-vegetation-v1/`, `zone-vegetation-export-v1.log` and
`zone-vegetation-gltf-validation-v1.json`. The two initial strict-Clippy findings
were test style issues and are fixed in the passing v2 checks. The initial domain
compile's integer/float range typo is corrected; failed candidates are not proof.
The fixture `content/zones/vegetation.commands.jsonl` starts at revision 0 and
shows individual placement plus seeded scatter. All changes remain local.


The vegetation package also passes normal server/two-rendered-client encrypted
Renet qualification, both clients at actual 3440x1440, with observed Vulkan
validation and zero GPU diagnostics. The independent checks confirm admitted
content identity, distinct players, moving poses, wall clearance and both avatar
colours. Evidence: `build/zone-package-native-vegetation-3440-v1/verification.json`
and both `scene.png` captures. The current editor binary SHA-256 is
`2a961f0eecb14dadd1318a916ed9047fef8c03067f0c0f04ba026d34854afa1d`;
normal client/server hashes remain those in the export milestone. This is Linux
loopback presentation/transport evidence, not native Windows, production admission
or capacity qualification.

## Guide-driven terrain grading — 8 November 2026

P5-17 / P3-15 / P2-06/07 continue from committed guide geometry, the bounded
height patch, shared command history and the retained Avian/export pipeline.
`grade_path` accepts a `grade` with `path_id`, `surface_offset_metres` (-16..16),
`falloff_metres` (0..32) and `blend` (0.05..1). It resolves the committed guide and
changes the existing terrain samples in one validated transaction. Persistent
zone schemas/data remain compatible: the resulting heights and guide are saved.

For each interior 4 m sample, choose the nearest horizontal guide segment,
interpolate its explicit endpoint heights, and add the requested surface offset.
The full guide width receives the target blend; an optional linear falloff blends
outward from that band. Endpoint regions are round. At exact crossings the first
authored segment wins. Global anchors are subtracted before interpolation/local
arithmetic, including signed far frames and nonzero origin Y. Targets/results
must stay within the existing +/-64 m local height budget. The locked zero border
is preserved. Only this patch's interior samples change; paths outside it may
produce no change and refuse. Guide ribbons, objects, plants and markers retain
their authored positions. There is no water simulation or navigation operation.

In F3 Guides, select a committed guide and choose **Grade terrain to guide**.
Defaults are -0.03 m surface offset, 4 m falloff, full blend, putting the terrain
under the existing visual ribbon offset. Backspace still removes draft points.
A pending guide draft refuses grading and remains available to finish/cancel.
Changing a selected guide's height with PgUp/PgDn before grading supports cuts
and embankments. Explicit agent parameters support stream-bed lowering and softer
blends. Repeating falloff/partial blends continues toward the target; retrying the
same mailbox request is still cached and cannot accidentally apply it twice.

Missing terrain/guide, invalid parameters, over-height targets and no-change
operations refuse atomically. Undo restores the entire previous patch as one
entry; redo restores exact saved samples. Mixed agent/visual history is shared.
The fixture `content/zones/grading.commands.jsonl` is revision-0 input for the
retained terrain authoring fixture: it adds a sloped road, grades the patch and
sets a valid elevated spawn in the same transaction.

Acceptance: 28 domain and eleven editor cases pass, including numerical width/
falloff/interpolation, grouped history, rollback with preserved redo, signed
far-frame stream lowering, 4,096 new actual Avian triangle-centroid queries and
capsule walks up/down the resulting slopes. All graded visual triangle heights
agree with actual Avian within 0.1 mm; capsule support stays within 8 cm. Strict
workspace/all-target Clippy and formatting pass. The graded world package exports
without DISPLAY/WAYLAND_DISPLAY, passes actual runtime admission and Khronos GLB
validation with zero errors/warnings. Evidence: ignored
`build/zone-grading-tests-v2.log`, `zone-grading-clippy-v1.log`,
`zone-grading-build-v1.log`, `zone-grading-export-v1.log`,
`zone-package-grading-v1/`, `zone-grading-gltf-validation-v1.json`.
The initial test incorrectly expected a repeated falloff blend to be a no-op;
the final check verifies no-change refusal for a full-width, zero-falloff pass.
This is bounded terrain authoring, not terrain streaming or MMO-scale evidence.

The full OS-input walkthrough passes at default 3440x1440 on AMD Radeon RX 7900
XTX (RADV NAVI31), Vulkan device 29772. It verifies refusal with a pending draft,
selected-guide height edits and grading, one-entry undo/redo, mixed agent grading,
exact cached retry after visual undo, save/load and offline playtest, alongside
all earlier tools. Validation is observed active; zero GPU diagnostics or resize
exceptions. Evidence: `build/zone-grading-native-3440-v1/{qualification.json,state.png}`.
The normal headless server and two actual rendered Renet clients also pass the
graded-slope package at 3440x1440, including bound content identity, movement/
terrain support, distinct identities and avatar pixels. Both clients observe
active Vulkan validation with zero GPU diagnostics. Evidence:
`build/zone-package-native-grading-3440-v1/verification.json` and both captures.
These are Linux loopback checks; native Windows, production admission and capacity
remain separate. All artifacts remain ignored; commits stay local.

## Individual guide-point editing — 8 November 2026

P5-17 / P3-15 continue from guide validation/history and terrain grading. A new
`move_path_point` edit names guide `id`, zero-based `index` and global f64
`position`. It changes only that point, then validates the whole candidate with
the retained bounds, horizontal separation and segment-length rules. Unknown
IDs/indices, collapsed segments, out-of-area coordinates and stale revisions
preserve the document and both history branches. Existing source schema/data is
unchanged; saved guides retain their explicit coordinates.

In F3, click a ribbon to select the whole guide or click a control-point handle
to select one point. Tab/Shift-Tab cycles whole-guide selection and points in
both directions. Arrows/PgUp/PgDn move the selected point, or the whole guide if
no point is selected, using the existing 1 m / 0.1 m increments. F focuses the
camera on the selected guide/point. The inspector names the active point; the
selected handle is larger/amber. Width controls still change guide width. Use
Grade terrain to guide after adjusting heights to construct an authored slope.
Select Next returns to whole-guide selection. Load, deletion and agent changes
reconcile invalid point selection before subsequent edits.

The editor uses normal Bevy meshes, materials and mesh ray casting. A pool of
64 handles shares one mesh; only the active draft or selected guide's handles
are visible, and playtest hides them. These are editor presentation entities,
excluded from collision, source data and GLB export. There is no new engine
infrastructure. Point dragging and point insertion/deletion on committed guides
remain separate usability work; replacement remains available through `put_path`.

Two new domain checks exercise narrow edits, full-candidate rollback/redo,
unknown indices and signed far-frame centimetres with command JSON round-trip.
Together with retained cases, 41 focused Rust cases pass (30 domain / eleven
editor). Strict workspace/all-target Clippy and formatting pass. Evidence:
ignored `build/zone-guide-points-tests-v1.log`, `zone-guide-points-clippy-final-v4.log`.


Camera pan/focus now uses the full +/-512 m authoring extent. This allows focus
on valid control points beyond the previous +/-480 m camera limit. The optional
qualification report exposes actual Bevy hover and left-button state; the native
harness waits for those observations before releasing presses. It never retries
mutating button actions. The first basic point walkthrough passed; the first
edge-expanded run missed an earlier terrain toolbar press and is retained as a
failed candidate, not counted as a final pass.

The final full native OS-input walkthrough passes at default 3440x1440 on the
same RX 7900 XTX / RADV Vulkan device, including ribbon/point mesh picking,
isolated endpoint movement, both directions of point cycling, agent point edits,
collapsed-segment refusal, shared undo, grading and save/load. It also places an
edge guide at x=500..504 m, focuses its first point to the viewport centre, then
undoes it and verifies stale selection is cleared. The final capture focuses the
selected road point at close range. Vulkan validation is observed active with
zero GPU diagnostics/resize exceptions. Evidence: ignored
`build/zone-guide-points-native-3440-final-v3/{qualification.json,state.png}`,
`zone-guide-points-build-final-v4.log`. The headless
`content/zones/guide-points.commands.jsonl` fixture moves a point in the grading
package's source, grades it and relocates spawn in one transaction; it passes
without a display (`build/zone-guide-points-headless-v1.{json,log}`). No runtime
format, normal game controls or network behavior changed in this slice.

## Terrain colour painting — 8 November 2026

P5-17 / P3-15 / P2-06/07 continue from the shared sculpt-stroke previews, bounded
patch and standard Bevy/GLB renderer. Zone schema 6 adds optional per-vertex colour
weights to the terrain patch. Omitted weights retain the existing grass colour;
painted patches carry exactly 33x33 four-channel u8 rows in grass/dirt/rock/sand
order, each summing to 255. The outer border remains all grass to meet the flat
surroundings. Height/sculpt/grade edits retain those weights. Schemas 1..5 remain
readable; data with paint requires schema 6. The combined document-byte limit
continues to apply.

`paint_terrain` accepts `paint` with global X/Z `centre`, `radius_metres` (4..32),
`surface` (`grass`, `dirt`, `rock`, `sand`) and `blend` (0.05..1). Radial linear
falloff reduces other channels with bounded integer quantization; the target
receives the remaining weight, keeping every row's sum exact. All input weights
are checked before arithmetic, including malformed intermediate patches inside
multi-edit transactions. Invalid/no-change/outside-patch operations preserve the
whole document, revision and redo. Paint affects colours while heights, shapes,
spawns and other placements retain their data.

F7 opens terrain paint; F2 returns to sculpting. Choose a surface, click/drag a
stroke and release for one history entry. Existing radius controls, Escape/focus
cancellation, draft busy refusal, shared agent history and save/load apply. The
same disposable validated preview editor drives sculpt and paint. Four standard
Bevy colour factors blend in linear space through `Mesh::ATTRIBUTE_COLOR`, with
a white standard PBR material for painted terrain. Border grass matches the
retained uniform ground. This is colour authoring; texture sets, UV/material
catalogues, biomes, gameplay surface/friction and terrain LOD remain separate.

Export reuses the exact editor mesh factory and emits standard glTF `COLOR_0`
float RGBA attributes, a white PBR material and the existing world package. The
visual digest/identity changes and authoring weights are retained. Existing
collision triangles/shapes stay identical. No custom shader, texture generator,
renderer or terrain runtime format is introduced. The structured fixture
`content/zones/painting.commands.jsonl` applies three coloured regions to the
retained terrain source at revision 0.

Acceptance: 33 domain and twelve editor tests pass. New cases cover grouped
paint history, exact normalization/borders, all channels at signed far origins,
invalid serialized/intermediate data and rollback/redo. Export checks exact
linear centre colours, the standard colour accessor/material, source round-trip,
bound visual bytes and unchanged collision. Strict workspace/all-target Clippy
and formatting pass. Headless export and actual runtime admission pass without a
display; Khronos GLB validation reports zero errors/warnings. Evidence: ignored
`build/zone-paint-tests-v1.log`, `zone-paint-clippy-v1.log`,
`zone-paint-build-v1.log`, `zone-paint-export-v1.log`,
`zone-package-paint-v1/`, `zone-paint-gltf-validation-v1.json`.

The full native OS-input walkthrough passes at default 3440x1440 on the same
RX 7900 XTX / RADV Vulkan device. It verifies uncommitted colour stroke preview,
one-entry commit/undo/redo, exact normalized weights/grass border and unchanged
heights, all four surfaces, live agent paint/visual undo/cached retry, save/load,
playtest and stable mesh asset count. It then repeats point picking/focus and
edge-selection cleanup. Vulkan validation is observed active with zero GPU
diagnostics/resize exceptions. Evidence:
`build/zone-paint-native-3440-v1/{qualification.json,paint.png,state.png}`.
The normal headless server and two rendered Renet clients also pass at 3440x1440
with the exported painted hillside, active validation and zero GPU diagnostics.
Actual package binding, player identities, poses/terrain support and both avatars
pass the retained independent checks; colour regions are visible in both native
captures. Evidence: `build/zone-package-native-paint-3440-v1/verification.json`
and both `scene.png` files. This remains Linux loopback evidence; no Windows,
production admission or terrain/population capacity qualification is claimed.

## One-command editor launcher and deferred save destination — 8 November 2026

P5-17 / P3-15 usability continues from the qualified editor and local build SDK.
Run `python3 tools/edit_zone.py` from the repository. It builds the locked debug
editor with one Cargo job and opens the existing 3440x1440 fullscreen app. When
the local Linux native SDK exists it prefixes its pkg-config/link paths for the
build, preserving caller settings. Runtime uses the retained client environment
helper. No server, service or transport is started.

Default draft: `build/zone-draft.json`; `--draft FILE` chooses another default.
An existing draft is loaded without rewriting it. A new draft starts in memory
and is written on Save. The new editor `--save-path FILE` selects that later save/
load destination without startup output. Existing `--output FILE` retains its
validated startup-write semantics. Explicit editor authoring flags (`--zone`,
`--commands`, `--output`, `--save-path`, `--export`) bypass automatic draft
selection; caller-relative paths retain their meaning. The launcher supports
`--build-only`, `--no-build`, and the caller's `CARGO_TARGET_DIR`. All other editor
arguments pass through. Build failure prevents launching. Invalid existing drafts
refuse through the normal Rust validator and retain their bytes.

Acceptance: 46 focused Rust cases pass (33 domain / thirteen editor), plus four
Python launcher tests for default-draft preservation, explicit arguments and
nonmutating build-environment preparation. Strict workspace/all-target Clippy,
formatting and Python compilation pass. The launcher builds successfully without
manual SDK environment setup. Actual native OS input at 3440x1440 verifies a new
save destination stays absent through startup and primitive placement, Ctrl-S
writes the selected destination, and the subsequent headless reopen preserves
saved data/bytes. Invalid-draft refusal also preserves source bytes. The native
capture uses RX 7900 XTX / RADV Vulkan with validation observed active and zero
GPU diagnostics. Evidence: ignored `build/zone-launch-tests-v1.log`,
`zone-launch-python-tests-v1.log`, `zone-launch-clippy-v1.log`,
`zone-launch-build-v1.log`, `zone-launch-native-3440-v1/{qualification.json,state.png}`
and the retained `build/check_zone_launcher_v1.py` walkthrough. This is Linux
execution evidence; the portable executable-suffix logic is not Windows execution.

## Optional 2 m terrain samples — 8 November 2026

P5-17 / P3-15 / P2-06/07 continue from the finite height/colour patch, grading and
retained mesh/query/export pipeline. Zone schema 7 adds optional terrain
`resolution: "2m"`; omitted/default `"4m"` retains the original 33x33 samples.
The fine grid is 65x65 (4,225 vertices / 8,192 triangles) over the same 128 m patch.
Only these two typed resolutions are accepted. Zero-height/grass border, finite
height/colour validation, area bounds and the combined byte budget remain active.
Older source files retain their format when loaded/saved at default resolution.

`resample_terrain` names resolution `"2m"` or `"4m"`. Refinement samples the exact
retained triangular surface. Colour channels interpolate over the same triangles
with deterministic largest-remainder u8 quantization to preserve sum 255.
Coarsening selects the corresponding coarse samples and may remove intervening
fine detail; undo restores it. Refinement followed by coarsening without edits
restores the original height/colour arrays exactly. Source schema stays at its
highest adopted version until undo/load restores an older document. Same-resolution,
missing-patch and malformed operations refuse without altering state/history.

F2 Sculpt exposes **Toggle 2 m / 4 m samples**; Ctrl-O retains load access.
The inspector reports actual sample spacing. Sculpting, painting, guide grading,
terrain sampling, grid presentation, offline Avian and export all use the active
resolution. The shared bounded regular triangulation now also accepts 64 cells;
existing game procedural/saved-terrain callers retain their 8/32-cell policies.
This adds finite authoring detail, without streaming, runtime LOD or a new world
package format. Existing placed objects/plants/markers/guides keep their positions.

Acceptance: 35 domain and fifteen editor tests pass. New cases cover exact
refinement surface/coarse round-trip, normalized colour resampling, atomic
refusals/source schema, both signed far frames, fine sculpt/paint and 16,384
actual Avian triangle-centroid queries with capsule walks. Fine mesh/query heights
agree within 0.1 mm and capsule support within 10 cm. Fine export verifies 8,192
collision triangles, source round-trip and actual existing runtime admission.
Seven retained terrain-library cases plus six procedural-world and three saved-
terrain integration cases also pass. Strict workspace/all-target Clippy and
formatting pass. Evidence: ignored `build/zone-resolution-tests-final-v3.log`,
`zone-resolution-network-terrain-v1.log`,
`zone-resolution-network-regressions-v3.log`, `zone-resolution-clippy-v1.log`.
The first fine-mesh test exposed the old generator's 8/32-only assertion; the
bounded 64-cell case now passes. A broad network-test build was stopped because
concurrent unnecessary links saturated swap; the final regressions select only
the two relevant integration targets with one build job. The initial library
saved-terrain name filter matched zero cases and is not counted as proof.

The full native OS-input walkthrough passes at default 3440x1440 on RX 7900 XTX
(RADV NAVI31), Vulkan device 29772. It verifies exact retained coarse vertex/
colour samples after refinement, one-entry undo/redo, exact coarse round-trip,
agent coarsening/visual undo/cached retry, fine sculpt/paint, save/load/playtest and
stable mesh handles. Subsequent point edits/guide grading also execute on the
fine grid. Vulkan validation is observed active with zero GPU diagnostics/resize
exceptions. Evidence: ignored
`build/zone-resolution-native-3440-v1/{qualification.json,resolution.png,state.png}`.

Headless export of `content/zones/resolution.commands.jsonl` against the painted
source adds a small 2 m-grid sculpt detail, passes existing runtime admission and
Khronos GLB validation (zero errors/warnings). Both normal server/two-rendered-
client Renet checks pass at actual 3440x1440: local and instanced at
`[128000000000, 0, -128000000000]`. All four clients observe active Vulkan
validation with zero GPU diagnostics. Exact content binding, distinct identities,
movement/terrain clearance and avatar pixels pass the retained checks. Evidence:
`build/zone-resolution-export-v1.log`, `zone-package-resolution-v1/`,
`zone-resolution-gltf-validation-v1.json`,
`zone-package-native-resolution-3440-v1/verification.json`,
`zone-package-native-resolution-far-3440-v1/verification.json` and captures.
Paired binaries are freshly built from the shared generator change with one job:
`zone-resolution-paired-build-v1.log`. Final strict Clippy is
`zone-resolution-clippy-final-v2.log`. Native Windows, production admission,
streaming/LOD and population capacity remain separate qualifications.

The final paint-help text is grid-neutral. A fresh 2 m source opens directly in
paint mode at 3440x1440 with validation active and zero GPU diagnostics:
`build/zone-resolution-ui-final-v2/{qualification.json,state.png}`. Final paired
build and exact hashes: `zone-resolution-paired-build-final-v2.log` and
`zone-resolution-paired-sha256-v2.txt`. The normal client/server match the two
successful native pairs; the final editor changes only that help line after the
full walkthrough.

| Final binary | SHA-256 |
| --- | --- |
| editor | `1bd17cc1dc5b004a7c4d444a65ec626ce0749cd7cb865036de401f996b175d51` |
| client | `7f0e002e0bc400a0d939033a7d1b87c5da7a6c58cb6d57e88a9f53a7f42c9f9b` |
| server | `0ddeef5da03401b35faa4d9c2a812f974d175d038961564c04fbb275dedc83a6` |

## Explicit terrain grounding — 8 November 2026

P5-17 / P3-15 / P1-10, with P2-06/07 terrain: sculpting and grading preserve
saved object/guide/marker/plant heights. H now grounds the current selection
explicitly through the same validated command and undo path as agent requests.
Objects use their centre anchor plus half their Y size; vegetation and markers
use feet. A selected guide point grounds individually; a whole selected guide
grounds all its control points in one history entry, with a 0.03 m ribbon offset.
X/Z, yaw, size, metadata, terrain and unselected placements remain unchanged.

```json
{"command":"apply","expected_revision":0,"edits":[{"op":"project_to_surface","projection":{"targets":[{"kind":"placement","id":1},{"kind":"guide_point","id":100,"index":0}],"offset_metres":0}}]}
```

IDs share the existing authoring namespace. Placement targets resolve objects,
plants or markers; guides require explicit point indices. Each projection has
1..128 unique typed targets and a finite offset within ±16 m. The surface is
sampled over the active terrain's exact triangles in global f64 coordinates;
outside the patch or without terrain it uses zone-origin Y. This grounds an
anchor, not the full footprint on a slope, and does not stack objects on props.
Spawn clearance remains the playtest/export validator's responsibility. No
schema change is needed for these existing positions.

Unknown IDs/indices, duplicates, malformed candidates, invalid offsets and an
entirely unchanged group refuse without advancing revision or changing either
history stack. Transactions still commit only after whole-document validation.
Draft guides must be finished/cancelled before H; playtest remains read-only.
Mailbox busy/refusal/retry rules are retained. One grouped command is one undo
entry even when it combines several placement types and guide points.

Qualification: 54 focused Rust cases (39 domain / 15 editor), four launcher
cases, formatting, diff checks and denied-warning workspace/all-target Clippy
pass. Grounding tests cover exact off-grid triangle support, flat fallback,
primitive centre/foot offsets, mixed groups, positive/negative far frames,
no-op refusal, stale history protection and malformed intermediate terrain.
The existing actual Avian/export/encrypted-admission tests remain included.
Evidence: ignored `build/zone-ground-tests-final-v2.log`,
`zone-ground-clippy-final-v2.log` and `zone-ground-build-v1.log`.

The full native 3440x1440 walkthrough passes on AMD Radeon RX 7900 XTX
(RADV NAVI31), Vulkan device 29772, with validation observed active and zero GPU
or resize diagnostics. Real pointer/keyboard checks cover object/plant/marker/
whole-guide grounding, individual guide-point H, exact undo/redo, repeated no-op
refusal and agent object grounding/cached retry. Retained sculpt/paint/refine/
grading/save/load/playtest checks also pass. Evidence:
`build/zone-ground-native-3440-v4/{qualification.json,state.png}`, final revision
138, history 54/1. Editor SHA-256:
`837961d8e6cad4fc1587b73d34d3b1c245b63afc5bdf775dd63d79b30b7df372`.

An initial native fixture's +1 m marker move landed exactly on terrain; the
correct no-op refusal invalidated its expected revision. Explicit fixture
movement directions now avoid that coincidence. A rerun also exposed initial
cursor motion being sent before the first rendered report; the harness now
moves within the owned window after that report before targeting a button.
The intermediate partial runs are not counted as acceptance. Native Windows
and new normal-game pairs were not run for this position-only command slice;
the earlier exported-package pair evidence retains its recorded source scope.


## Saved and unsaved authoring state — 8 November 2026

P5-17 / P3-15 / P1-10: the standard Bevy summary now shows Saved or Unsaved.
Compare the current validated document with a bounded snapshot of the selected
save destination at startup or its last successful Save/Load. A new destination
is unsaved until Save succeeds. Startup commands that change a loaded source
also remain unsaved unless `--output` already wrote that exact candidate.

Content equality, rather than revision number, makes undo back to saved content
clean and redo away from it dirty. Visual and accepted agent edits use the same
comparison. Pending drags, strokes and guide drafts count as unsaved work;
saving the committed document while a guide draft is open retains Unsaved.
Cancelling the draft can restore Saved. Offline playtest movement does not
change saved authoring content. Save failure retains the baseline and document;
load failure does not clear unsaved state. Existing files are not watched for
external changes. No autosave, close interception or save-format change is added.
The optional qualification report exposes `unsaved_changes`.

Qualification: all 54 focused Rust cases, formatting and denied-warning workspace/
all-target Clippy pass. Three native Linux runs at actual 3440x1440 cover a new
draft, saved-source reopen and failed Save to a directory. Real keyboard/pointer
checks also cover exact undo/redo/load baseline changes, an accepted mailbox edit,
saving/cancelling pending guide drafts and unchanged playtest content. All three
runs observe active Vulkan validation on RX 7900 XTX (RADV NAVI31), device 29772,
with zero GPU/resize diagnostics. Captured pixels show Saved/Unsaved in the
standard Bevy summary. Evidence: ignored `build/zone-save-state-tests-v1.log`,
`zone-save-state-clippy-v1.log`, `zone-save-state-build-v2.log` and
`zone-save-state-native-3440-v2/{qualification.json,*.state.png}`.

Final screenshot review found the mode-specific help replacing the initial H
hint. Object, guide, marker and plant help now retain that shortcut. The final
three native cases pass again with zero GPU diagnostics:
`build/zone-save-state-native-3440-final-v3/qualification.json` and captures.
Final build/Clippy: `zone-save-state-build-final-v3.log`,
`zone-save-state-clippy-final-v2.log`. Final editor SHA-256:
`c6bc2ace0840765d20a1ff206ace7a804dd9f1190c0cb2f3d639d90380a3a5bd`.


## Keyboard selection and placement framing — 8 November 2026

P5-17 / P3-15 / P1-10: Objects now support Tab / Shift-Tab selection in document
order, with forward/backward wrapping and safe empty selection. Cycling leaves
placement mode and can reach objects hidden by terrain or outside the view.
H can then ground the selected object. Delete/agent changes retain the existing
selection reconciliation; no visibility-dependent selection list is stored.

F now frames selected objects, plants and marker footprints as well as the
existing whole-guide/control-point selection. Object framing uses a conservative
bounding sphere. Fixed greybox plant bounds include the canopy; marker framing
includes its footprint. Convert f64 anchors through the existing checked zone
frame, then move the normal Bevy camera target and distance. A three-radius plus
8 m framing distance gives margin at the retained default 45-degree vertical
field of view. The bounded orbit maximum increases from 240 m to 512 m so the
largest allowed 128 m primitive fits. View positions remain within the retained
4,096 m local coordinate bound. Long guides can still exceed a single view.

Selection/framing never creates a document edit, changes either history stack
or clears unsaved state. F and object cycling refuse during playtest or pending
stroke/drag/guide draft; Ctrl chords retain existing behavior. Guide Tab cycling
continues to select whole guides/control points. The optional native report adds
local camera `view.target` / `view.distance` fields for verification.

All 54 retained focused Rust cases, formatting and denied-warning workspace/
all-target Clippy pass. Three native 3440x1440 cases cover local coordinates,
origin `[128000000000, 0, -128000000000]` and an empty document. Actual keyboard/
pointer checks cover forward/reverse wrapping, a buried-step H/undo recovery,
large primitive framing beyond 300 m, plants, marker footprints, whole guides,
individual points and playtest refusal. Captures show the entire selected large
cube and tree proxy. Selection and framing preserve document/history/dirty state.
All three observe active Vulkan validation on AMD RX 7900 XTX (RADV NAVI31),
device 29772, with zero GPU/resize diagnostics. This is Linux editor evidence;
normal-game networking and native Windows retain their earlier qualified scope.

Reproducer fixture: `content/zones/selection.commands.jsonl`. Evidence: ignored
`build/zone-selection-native-3440-v1/{qualification.json,*.png}`,
`zone-selection-tests-v1.log`, `zone-selection-build-v1.log` and
`zone-selection-clippy-v1.log`. Editor SHA-256:
`d934da475b0947a812bf0c0d1d7ba08eb8974feccc2f4514249e1d32bff234b4`.


## Sampled and adjustable flatten level — 8 November 2026

P5-17 / P3-15 / P1-10; P2-06/07 terrain: in Sculpt (F2), place the pointer over
the surface and press I to select Flatten at that sampled height. Sampling uses
the existing checked conversion and exact domain triangles at the cursor X/Z,
rather than persisting the approximate render-ray Y. Heights are relative to
zone-origin Y, including negative terrain and nonzero global origin Y.

With Flatten selected, PageUp/Down adjusts the chosen level by 1 m; G snap off
uses 0.1 m steps. Clamp the level to ±64 m, matching the retained terrain domain.
The Flatten button displays the current level. The default is still zero.
Settings/sampling change no document or history; paint mode, held/pending
strokes and playtest do not sample. Level adjustments during a preview cancel
it through the existing action path. Existing focus/panel/Esc cancellation
and one-release/one-undo stroke behavior remain intact.

Visual flatten stamps now supply this level to the existing `brush` operation
(`kind: flatten`, `height`, `blend: 0.5`), using the same full-candidate validation,
command bounds and history as agent/headless edits. There is no new schema or
persisted brush-setting format. The optional report includes local
`flatten_height` and render-local `brush_point` for qualification.

Two native Linux 3440x1440 cases pass on RX 7900 XTX (RADV NAVI31), Vulkan device
29772, with validation active and zero GPU/resize diagnostics: a 4 m positive
plateau and a 2 m negative plateau at origin `[128000000000,20,-128000000000]`.
Actual input samples levels +4/-4, adjusts coarse/fine increments, isolates paint
mode, and stamps +5/-5. Independently running the same structured command through
the headless path produces exact matching f32 sample arrays. One-entry undo,
redo, save and actual Avian offline playtest pass. Captured pixels show the
selected level and appropriate terrain. No normal-game or Windows qualification
is added by these editor checks.

Evidence: ignored `build/zone-flatten-native-3440-v1/{qualification.json,*.png}`,
including generated source documents and exact reference command/output files;
`zone-flatten-tests-v1.log`, `zone-flatten-build-v2.log` and
`zone-flatten-clippy-v1.log`. Editor SHA-256:
`fd7196b36d3a8834fe9910de9a27aa463e9396a01f483ca65fd859c4db221ea6`.
All 54 retained focused Rust cases, formatting, diff checks and denied-warning
workspace/all-target Clippy pass for this slice; launcher Python checks remain
four passing cases.


## Shared duplication for greyboxing — 8 November 2026

P5-17 / P3-15 / P1-10: Ctrl-D copies the selected object, plant, marker or whole
guide, selects the new copy and records one undo entry. Even when a guide point
is selected, duplication copies its whole guide and returns to whole-guide
selection. UI copies offset X/Z by 1 m, or 0.1 m with grid snap off, preserving Y.
They leave placement mode so subsequent transforms act on the copy. Use H when
the copy should be grounded on differing terrain. There is no implicit fitting,
prop stacking or publication of authoring annotations.

The same renderer-free edit is available to agents/headless commands:

```json
{"command":"apply","expected_revision":0,"edits":[{"op":"duplicate","id":1,"new_id":2001,"offset":[8,0,8]}]}
```

`id` can resolve any of the four authoring kinds. `new_id` must be positive and
unused throughout the shared namespace. Offsets are finite f64 metres within
±1024 per axis; final positions still obey all existing zone/global limits.
Preserve source geometry, transforms, kind, radius/scale, explicit guide points
and unresolved marker definitions. Append ` copy` to the name after truncating
its prefix at a UTF-8 boundary to keep the existing 80-byte label bound. Source
records remain unchanged. No schema/format change is needed.

Every duplicate passes the existing per-kind count, document-byte and spatial
validation before committing. Unknown sources, occupied IDs, invalid offsets,
out-of-zone results and exhausted budgets leave revision/history/redo intact.
Mixed copies can share one transaction and undo. The visual allocator stays
monotonic across undo and agent requests, so a copied entity's removed ID is
not silently reused. Guide drafts and pending drag/stroke previews must be
finished/cancelled first; playtest is read-only. Agent mailbox busy/refusal/cache
semantics are retained. The context-specific help is now initialized from its
single mode-aware source, keeping Ctrl-D and H hints visible.

58 focused Rust cases (43 domain / 15 editor) pass, including mixed copy groups
at positive/negative far frames, exact undo/redo and metadata, bounded UTF-8 copy
names, stale/invalid transaction rollback and every per-kind count limit.
Denied-warning workspace/all-target Clippy and formatting pass. Evidence:
ignored `build/zone-duplicate-tests-v1.log`, `zone-duplicate-clippy-v1.log` and
`zone-duplicate-build-v1.log`.

Local/far/empty native 3440x1440 cases pass actual Ctrl-D, one-entry undo/redo,
all four copy types, whole-guide behavior from a selected point, agent copy and
visual undo, and subsequent UI ID 5001 after agent ID 5000 was undone. Pending
guide drafts and playtest refuse copying without losing state. All three observe
active Vulkan validation on RX 7900 XTX (RADV NAVI31), device 29772, with zero GPU
or resize diagnostics. Native Windows and new normal-game pairs remain separate.
Evidence: `build/zone-duplicate-native-3440-v1/qualification.json`.

The mixed fixture `content/zones/duplication.commands.jsonl`, applied to the
selection source, also exports through the existing runtime's collision/content
verification. Its schema-4 package has four primitives, two guides, two plants
and two source-only markers, with 280 visual triangles. This is actual package
validation rather than a new runtime format or gameplay marker activation.
Evidence: `build/zone-copy-export-v1.log`, `zone-copy-export-source-v1.json` and
`zone-package-copy-v1/`. Editor SHA-256:
`3eb4d1f0b3d37e8389e2a3aa61d48430cb1616c0cfc334824162132c2f72c506`.
A final rerun captures each selected copied type before undo; all local/far/empty
checks still pass with zero GPU diagnostics:
`build/zone-duplicate-native-3440-v2/{qualification.json,*copy*.png}`.


## Adjustable raise/lower strength — 8 November 2026

P5-17 / P3-15 / P1-10; P2-06/07 terrain: with Raise or Lower selected in Sculpt,
`[` halves the stamp amount and `]` doubles it, clamped to the existing domain's
0.05..8 m range. The default remains 2 m. Both buttons show the current amount
to three decimal places, including the exact 0.125 m preset used for detail work.
Amounts are per existing distance-spaced stamp, not per frame or per second.

Strength is an editor-local preset; changing it creates no document/history edit
and does not mark clean terrain unsaved. Existing action behavior cancels pending
stroke previews before changing the amount. Paint/flatten/smooth and nonterrain
modes do not change height strength with those keys; vegetation scatter retains
its own bracket controls. Playtest is read-only. No schema/preferences file or
runtime format is added.

Stamps pass the selected finite f32 amount to the retained `raise` / `lower`
brush operations, preserving candidate validation, border/height bounds,
preview cancellation and one-release/one-undo behavior. Agents can send the same
amount directly using existing structured brush commands.

All 58 focused Rust cases, formatting and denied-warning workspace/all-target
Clippy pass. Native Linux 3440x1440 checks cover the 0.05/8 m endpoints and exact
0.125 m raise/lower command equivalence on a positive 4 m-grid plateau and a
negative 2 m-grid plateau at `[128000000000,20,-128000000000]`. Exact f32 arrays,
one-entry undo/redo, save and playtest refusal pass alongside the retained
sampled-flatten cases. Both observe active Vulkan validation on RX 7900 XTX
(RADV NAVI31), device 29772, with zero GPU/resize diagnostics. This is editor
qualification, without new Windows or normal-game claims.

Evidence: ignored `build/zone-strength-native-3440-v1/{qualification.json,*.png}`,
generated reference commands/documents, `zone-strength-tests-v1.log`,
`zone-strength-build-v1.log` and `zone-strength-clippy-v1.log`. Editor SHA-256:
`ea41f02024972c98f56c1162f97355e9dd362e37624aa006c6f458d7afb9d463`.


## Wayfarer Hollow example and terrain framing — 8 November 2026

P5-17 / P3-15 / P1-10; P2-06/07 terrain: an original primitive-only example now
connects the existing authoring tools into one playable bounded slice. Open it:

```sh
python3 tools/edit_zone.py --example
```

This seeds `build/wayfarer-hollow.zone.json` in memory on first launch and writes
it only on Save. Subsequent launches reopen that edited draft without reapplying
the recipe. Use `--draft FILE` for a different destination. The original base and
recipe remain unchanged. `--example` refuses explicit `--zone`, `--commands`,
`--output`, `--save-path` or `--export` combinations; use the existing explicit
headless pipeline below for those operations. Invalid existing drafts still
fail instead of falling back to a fresh seed. The ordinary blank-draft launcher
and caller-relative explicit paths retain their behavior.

The source is `content/zones/wayfarer-hollow.base.json` (named schema-1 origin /
spawn) plus `wayfarer-hollow.commands.jsonl`: four validated transactions with
10/4/37/17 edits. They create hills, a graded approximately 92 m approach route,
a level shelter courtyard and stream trench; refine to 2 m samples; paint dirt,
sand and rock; then place eleven primitives, two authoring notes and 48 seeded
plant proxies. The resulting schema-7 source is 337,763 pretty-serialized bytes,
within the retained 1 MiB budget. Only original source/recipe data is tracked;
generated documents and runtime exports stay under ignored build directories.
No new engine/collision special cases or production assets are introduced.

The route starts at `[64, 0.1, 116]`, faces the initial forward direction and
climbs to a 6 m summit between static posts. The shelter has a raised floor,
three walls, an open front and roof built from ordinary generic boxes. All
vegetation remains nonblocking greybox presentation. NPC/quest notes are not
live entities; their catalogue keys are unresolved example identifiers. The
stream ribbon is a render-only authoring guide, without water/swimming/flow
behavior. Existing default gameplay content is unchanged.

In Sculpt (F2) or Paint (F7), F now frames the terrain patch using its horizontal
extent and actual min/max heights, checked global/local conversion and the
normal bounded Bevy camera. It changes neither document nor history. Existing
object/plant/marker/guide/point framing remains intact. Suggested walkthrough:
F2, F to frame, wheel to zoom, F5 to play, W to walk through the approach posts,
then F5 to return. Save before exporting an edited draft. To reproduce directly:

```sh
target/debug/theia-zone-editor --headless \
  --zone content/zones/wayfarer-hollow.base.json \
  --commands content/zones/wayfarer-hollow.commands.jsonl \
  --output build/wayfarer-hollow.zone.json \
  --export build/wayfarer-hollow-package \
  --content-id theia.zone.wayfarer-hollow --revision 1
```

Export destinations must be new directories. This uses the existing standard
GLB / collision-profile schema 4 / bound manifest path. The qualified package
has 15,242 visual triangles, terrain vertex colours, three guides, 48 render-only
plants and two preserved source-only notes. Existing runtime collision/content
verification passes; Khronos validator `2.0.0-dev.3.10` reports zero errors,
warnings, infos or hints. Evidence: ignored `build/wayfarer-hollow-export-v1.log`,
`zone-package-wayfarer-hollow-v1/`, `wayfarer-hollow-export-source-v1.json` and
`wayfarer-hollow-gltf-validation-v1.json`.

All 59 focused Rust cases (43 domain / 16 editor) and seven launcher Python cases
pass, as do formatting and denied-warning workspace/all-target Clippy. A new
fixture test uses actual Avian queries/shared capsule movement to traverse every
route waypoint with terrain contact, stand on the shelter floor, stop at its
back wall and refuse a spawn inside a wall. This is source-content collision
proof, not a navigation, gameplay AI or Classic movement-policy change.
Evidence: `build/wayfarer-hollow-tests-v1.log`, `wayfarer-hollow-clippy-v1.log` and
fresh paired `wayfarer-hollow-build-v1.log`.

Three actual native 3440x1440 editor cases cover first-launch seed/save, a user
marker edit surviving reopen, and origin `[128000000000,20,-128000000000]`.
They verify source hashes unchanged, terrain framing without history changes,
actual forward walking past approach posts and unchanged authoring content
after playtest. Every accepted PNG checks visible road, blue guide/avatar and
foliage pixels in the scene region. Initial captures can precede Bevy's pipeline
compilation; the read-only capture helper retries until scene pixels exist.
The first run's early blank overview is not counted as final visual acceptance.
Final evidence: `build/wayfarer-hollow-native-3440-v2/{qualification.json,*.png}`.
All three observe active Vulkan validation on RX 7900 XTX (RADV NAVI31), device
29772, with zero GPU/resize diagnostics.

The exported package also passes two actual normal server/two-rendered-client
Renet cases, all four clients at 3440x1440: local and instanced at
`[128000000000,20,-128000000000]`. Bound content identities, separate player
identities, authoritative movement/contact, global/local coordinates and blue/
red avatar pixels pass. Capsule clearance is approximately 0.008 m in both
frames. All four clients observe active Vulkan validation with zero GPU
diagnostics. Evidence: `build/wayfarer-hollow-renet-local-3440-v1/verification.json`,
`wayfarer-hollow-renet-far-3440-v1/verification.json` and captures. This is loopback
encrypted development admission with real Bevy clients; production account
admission, native Windows, population capacity, streaming and latency benchmarks
remain separate. The normal client/server source is unchanged by this authoring
slice; exact fresh paired hashes are recorded below and in
`build/wayfarer-hollow-paired-sha256-v1.txt`.

| Binary | SHA-256 |
| --- | --- |
| editor | `7d4090274302f02a6664d703e85fc0c7ef464cc52bdc46cc4a4cb099dc49d879` |
| client | `7f0e002e0bc400a0d939033a7d1b87c5da7a6c58cb6d57e88a9f53a7f42c9f9b` |
| server | `0ddeef5da03401b35faa4d9c2a812f974d175d038961564c04fbb275dedc83a6` |


## Inserting and removing guide points — 8 November 2026

P5-17 / P3-15 / P1-10: a committed road or stream can now be reshaped without
rebuilding its entire guide. Insert adds the midpoint of the segment after the
selected point; at the last point it uses the preceding segment. Whole-guide
selection uses its first segment. The new point becomes selected. Backspace
removes a selected committed point and selects the next available index, or the
previous last point. A guide must keep at least two points. Delete still removes
the whole guide, and Backspace during a pending guide draft still pops its last
point. Insert during a draft refuses without discarding it.

The shared renderer-free edits are `insert_path_point` (`id`, insertion `index`,
global f64 `position`) and `remove_path_point` (`id`, `index`). An insertion index
can be 0..=length, allowing structured commands to prepend/append explicitly.
Removal indices must exist. The complete candidate still enforces 2..64 points,
at least 0.25 m horizontal separation, at most 256 m segment length, width/margin,
coordinate and serialized-byte bounds. Midpoint insertion into a segment too
short to preserve minimum spacing refuses. Schema and guide IDs remain unchanged.

For a freshly loaded original example, these two separate commands add/remove
one midpoint while preserving all other guide metadata:

```json
{"command":"apply","expected_revision":0,"edits":[{"op":"insert_path_point","id":100,"index":2,"position":[56,2.03,77]}]}
{"command":"apply","expected_revision":1,"edits":[{"op":"remove_path_point","id":100,"index":2}]}
```

Each operation is one undo entry, or part of a bounded grouped transaction.
Invalid indices, unknown guides, invalid spacing/coordinates, stale revisions
and exhausted point budgets preserve the prior document, revision and redo.
No terrain grading is implicit; use Grade when changed guides should reshape
terrain. No navigation/water/runtime actor behavior is added.

Point indices are positional. When a successful edit changes the selected
guide's point count, the common visual/agent reconciliation clears the old point
selection; undo/redo do this too. Successful visual insertion/removal then sets
a valid new/next selection explicitly. This prevents an agent insertion from
silently retargeting a later visual point move. Nonstructural point transforms
retain their selection. A selected point now uses one shared opaque cyan standard
Bevy material, distinguishing it from the amber selected ribbon; unselected
handles remain pale and the existing 64-handle/single-mesh pool is retained.
Earlier amber-point capture records retain their historical source scope.

All 62 focused Rust cases (46 domain / 16 editor), seven launcher cases,
formatting and denied-warning workspace/all-target Clippy pass. Three new domain
cases cover road/stream insert/remove exact undo/redo at positive/negative far
frames, metadata/schema/ID retention, safe endpoints, 64-point limits, invalid
index/spacing/global position rollback, stale revision and preserved redo.
Evidence: ignored `build/zone-point-structure-tests-v1.log`,
`zone-point-structure-build-final-v2.log`, `zone-point-structure-clippy-final-v2.log`.

The final local/far/empty native cases pass at actual 3440x1440 on RX 7900 XTX
(RADV NAVI31), Vulkan device 29772, with validation observed active and zero GPU
or resize diagnostics. Actual input selects the inserted midpoint mesh, removes
it, then selects the ribbon at its old location. Handle projections and mesh
asset counts remain consistent. Exact undo/redo, two-point refusal, agent
structural selection clearing and draft refusal pass alongside all retained
selection/framing/duplication cases. Final evidence:
`build/zone-point-structure-native-3440-v3/{qualification.json,*.png}`. Normal-game
package formats/client/server code are unchanged; earlier example client-pair
and Windows qualification records retain their stated scope. Editor SHA-256:
`a4231af95f4ce17f184e30e010e09acdcf8c8150dcbcc4d772c23715561db83f`.


## Paged content selection at 3440x1440 — 8 November 2026

P5-17 / P3-15 / P1-10: the right-side Zone Content list now shows named terrain,
spawn, primitive, guide, marker and plant entries. It uses ordinary Bevy UI
buttons/text and primary-pointer picking, with 40 fixed row slots and Previous /
Next controls. The current document limits allow at most 1,922 entries including
terrain and spawn; paging does not allocate a UI row per authored entity.

Click an authored entry to select it and enter its existing editing mode. F
frames it through the retained camera path. Guide rows select the whole guide;
point selection remains in the viewport. The terrain row selects Sculpt, or
retains Paint when already painting. The spawn row frames the existing spawn
without moving it. Full selected names appear below the list when a row label
is clipped. The existing F-key mode switches and list selection now use the same
mode-reset helper. This is a flat content browser, without hierarchy/group data.

Selection, paging and framing change no document or history and do not clear
unsaved work. The list reveals a changed keyboard/viewport selection's page and
clamps its page when content shrinks. Agent edits, duplicate/undo and loaded
content refresh labels and rows from the current validated document. A stale
clicked entry is checked against current content before selection. Pending
strokes, drags and guide drafts must be finished/cancelled before row selection;
paging itself remains available. The list hides during playtest.

Both side panels are excluded from all object/guide/marker/plant/terrain mouse
picking and from editor orbit/wheel zoom. Keyboard transforms keep their existing
behavior. In particular, paging while a placement palette is active cannot place
behind the panel, and clicking a content row cannot stamp terrain underneath it.
The editor still targets 3440x1440 only. It uses the normal full-window Bevy camera
and symmetric 300 px UI panels; no custom render target or viewport renderer is
introduced. The optional report includes list controls/bounds plus cursor/focus
and camera yaw/pitch for native qualification. Zone/runtime formats and normal
client/server code remain unchanged.

The 62 retained focused Rust cases, seven launcher cases, formatting and denied-
warning workspace/all-target Clippy pass. Initial native local/far/reopen cases
pass 40-row bounds, cross-mode selection/framing, click/wheel/right-drag isolation,
draft protection, duplicate/undo and agent name updates. Keyboard selection also reveals its page; a grouped 48-plant removal clamps the
browser from two pages to one and visual undo restores all rows. Final evidence:
ignored `build/zone-outliner-tests-v1.log`, `zone-outliner-clippy-final-v2.log`,
`zone-outliner-build-v5.log`, `zone-outliner-layout-v1.png` and
`zone-outliner-native-3440-v5/{qualification.json,*.png}`. All three fresh/reopen/
far cases run at actual 3440x1440 on RX 7900 XTX (RADV NAVI31), Vulkan device
29772, with validation active and zero GPU/resize diagnostics.

The full retained native walkthrough also passes with the new panels and common
mode switch: primitive placement/drag/transforms, live mailbox, sculpt/paint,
refinement/coarsening, guide grading/points, vegetation, markers, grounding,
undo/redo/save/load and offline playtest. Final revision/history remain 138 and
54/1. Evidence: `build/zone-outliner-regression-3440-v2/qualification.json` and
captures. No new normal-game or Windows qualification is inferred from these
editor UI checks. Editor SHA-256:
`5d248972af2e36c15c44744a730db74c46dad6ec61f82f0f84ad74110cd7994a`.

Qualification refinements were confined to the native harness: initialize the
owned synthetic wheel over UI before measuring viewport zoom, wait for toolbar
bindings after mode switches, and read the allocated copy ID instead of assuming
a cancelled guide draft releases its reserved ID. Earlier partial runs are not
counted as final acceptance. Product input uses Bevy's platform abstractions;
primary pointer selection does not let secondary-button gestures select rows.


## Adjustable sculpt and paint blends — 8 November 2026

P5-17 / P3-15 / P1-10; P2-06/07 terrain: bracket keys now adjust the current
terrain brush amount consistently. Raise/Lower still halve/double metres per
stamp. With Flatten or Smooth selected, they halve/double the shared sculpt
blend; in Paint they adjust a separate paint blend. Both blends clamp to the
existing domain's 5..100% range. Defaults remain 50% sculpt and 100% paint.
Buttons show two-decimal percentages, including the exact 12.50% preset.

Presets persist across tool/mode changes within the session but are not saved
in zone data. Adjusting them changes neither document/history nor clean/dirty
state, cancels pending preview through the existing action path and refuses
while playtesting. Vegetation bracket controls retain their scatter meaning.
Paint changes normalized colour weights only; it does not alter collision or
introduce material-alpha/gameplay-surface semantics.

Stamps feed the chosen blend into the existing validated `flatten`, `smooth`
and `paint_terrain` commands. Shared candidate/border/height/weight budgets,
one-release/one-undo behavior and normal standard-Bevy vertex-colour rendering
remain intact. No source or runtime schema is added. The optional qualification
report exposes `sculpt_blend` and `paint_blend` separately.

All 62 focused Rust cases, seven launcher cases, formatting and denied-warning
workspace/all-target Clippy pass. Two native 3440x1440 Linux cases exercise the
5/100% endpoints and independent presets on positive 4 m and negative 2 m grids,
the latter at `[128000000000,20,-128000000000]`. Actual 12.5% Flatten, Smooth and
Paint stamps produce exact matching headless f32 samples and normalized u8
weights. Undo/redo/save and playtest refusal pass alongside retained sampled
flatten/raise-lower strength checks. Both observe active Vulkan validation on
RX 7900 XTX (RADV NAVI31), device 29772, with zero GPU/resize diagnostics.
Normal-game and Windows evidence retains its earlier scope.

Evidence: ignored `build/zone-blend-native-3440-v1/{qualification.json,*.png}`,
generated per-operation source/command/reference files, `zone-blend-tests-v1.log`,
`zone-blend-build-v1.log` and `zone-blend-clippy-v1.log`. Editor SHA-256:
`adabb4ac4aac24fce0def06e4d843368756b4368373c83b215fb241352d4ea37`.


## Shared names and standard Bevy text input — 8 November 2026

P5-17 / P3-15 / P1-10: F8 opens a name dialog for the selected object, whole guide,
marker or plant. With no selection it edits the zone name; Shift-F8 always edits
the zone name. Selecting an individual guide point still renames its whole guide.
The current name starts selected in Bevy's built-in TextInput/EditableText. Enter
or the primary-pointer Apply button submits; Escape, Cancel or window focus loss
cancels. Empty/whitespace/control-containing names and names exceeding 80 UTF-8
bytes are invalid through the existing shared validators. Invalid submissions
retain the text for correction. The widget additionally caps input at 80 Unicode
characters; domain byte validation remains authoritative. Unicode storage is
supported; visible glyph coverage depends on the retained font. Bevy's default
in-app clipboard is retained; the optional system-clipboard feature is not enabled.

Visual submissions and agent commands use the same narrow operations:

```json
{"command":"apply","expected_revision":0,"edits":[{"op":"rename","id":1,"name":"West wall"},{"op":"rename_zone","name":"Wayfarer Hollow"}]}
```

IDs share the existing authoring namespace. Missing IDs, stale revisions or any
invalid candidate reject the entire transaction and preserve revision/history/
redo. A transaction is one undo entry; visual submissions contain one name edit.
Names preserve all other source fields, including geometry, coordinates, schema
and annotation catalogue keys. Like retained Apply commands, an agent's identical
valid name still counts as a transaction. The dialog closes an unchanged visual
submission without issuing a command. Successful renames refresh content labels
and save/load normally; marker naming does not create live NPC/quest behavior.

Opening refuses during playtest, a drag/stroke or a pending guide draft and keeps
that work intact. While open, editing/mode/load/save/playtest shortcuts, content
selection, world pointer operations and camera input are blocked. The mailbox
uses its existing busy refusal. Text editing remains in the standard field and
an open draft contributes to the existing pending/unsaved indicator. Cancelling
restores the content-based saved state. Bevy owns selection, cursor, glyph layout,
scrolling and ordinary text shortcuts. The reusable field clears/sets only its
text, preserving the font/layout styles Bevy prepared for that entity; replacing
EditableText with a fresh internal editor would discard those styles without
changing TextFont. Submission reads after Bevy applies queued edits.

The editor remains 3440x1440 only. No source/runtime schema, custom widget engine,
renderer, plugin or game-stack pin changes. Chat composer/bubbles remain deferred.

All 65 focused Rust cases (49 domain / 16 editor), seven launcher cases,
formatting and denied-warning workspace/all-target Clippy pass. The new domain
cases check grouped names for all authoring kinds/zone, unchanged other metadata,
UTF-8 byte boundaries, unknown fields/IDs, stale revisions and atomic refusal
preserving redo. Native Linux local, far `[128000000000,20,-128000000000]` and
saved-reopen cases pass at actual 3440x1440. They exercise prepared initial
selection and real text replacement, invalid empty-name correction, primary Apply/
Cancel, unchanged submit, Escape, shared undo/redo, saved names, all-mode/shortcut/
camera/pointer isolation, busy agent refusal, accepted agent updates, guide-draft
and playtest refusal. Opening without a selection edits the zone on reopen.
All three observe active Vulkan validation on AMD RX 7900 XTX (RADV NAVI31),
device 29772, with zero GPU/resize diagnostics. Normal-game/Windows evidence
retains its earlier scope.

The first desktop run caught discarded font/layout styles when replacing the
field's internal editor; the retained-buffer fix passes subsequent native checks.
An early qualification helper also missed a synthetic Control press after pointer
and text-focus transitions. The final helper waits for reported modifier state,
fresh input processing and actual selection before typing. A bounded harmless
modifier re-press is available before commands; the accepted full run required
zero retries. Mutating clicks/edits are never retried. Earlier partial runs are
not acceptance evidence.

Evidence: ignored `build/zone-names-native-3440-v9/{qualification.json,*.png}`,
`zone-names-tests-v5.log`, `zone-names-build-v8.log`, `zone-names-clippy-v4.log` and
`zone-names-launcher-tests-v1.log`. Editor SHA-256:
`c2d9bd13142680d59ac80b3469df30d726a2f6f8e030b00749ac6d28ce3d9046`.


## Numeric primitive transform fields — 8 November 2026

P5-17 / P3-15 / P1-10: F9 opens properties for a selected Box, Sphere or Cylinder.
The seven standard Bevy TextInput fields hold global X/Y/Z metres, yaw around Y
in degrees and XYZ dimensions in metres. Global values stay f64 from the source
Pose through exact round-trip strings and finite-number parsing. They never pass
through f32 display coordinates or a new origin-conversion path. Text accepts
ordinary signed/decimal/scientific numbers and surrounding whitespace; each field
is bounded to 64 characters. Existing whole-candidate validators still own global/
local-frame/authoring bounds, dimensions 0.1..128 m, yaw bounds, uniform sphere
size and equal cylinder X/Z diameter. Numeric entries apply exactly, independently
of the grid-snap preset.

Tab/Shift-Tab cycle within Bevy's modal TabGroup using its TabNavigation system
parameter; primary-pointer field selection uses standard focus infrastructure.
This does not install a global tab observer or change ordinary object/guide
cycling. Text buffers retain Bevy's prepared font/layout styles when reset.
Enter or primary Apply submits all seven fields in one existing Transform edit.
Nothing changes in the document until Apply succeeds. Invalid numeric/shape/
coordinate input leaves the dialog and values intact, with no change to revision,
history or redo. Escape, primary Cancel or window focus loss cancels. Unchanged
submission closes without an edit. F9 without a selected primitive explains the
required selection; this first property form does not edit guide/marker/plant
properties.

Both dialogs share a modal guard: world/mode/load/save/playtest/undo/duplicate
input, content selection and camera movement remain isolated during typing;
mailbox mutations use the retained busy refusal. Each refuses opening during
playtest or a pending drag/stroke/guide draft, preserving that work, and either
open form contributes to the pending/unsaved indicator. The report exposes field
values, active selection and UI bounds for native qualification. No source/runtime
schema, engine/plugin pin or server-authority change is introduced.

All 68 focused Rust cases (49 domain / 19 editor), seven launcher cases,
formatting and denied-warning workspace/all-target Clippy pass. New parser cases
exercise exact far f64 strings, scientific notation and malformed/nonfinite fields;
all three primitive kinds share full-candidate refusal, redo preservation and
one-entry undo at local/far origins. Three native Linux cases pass at actual
3440x1440: local, far `[128000000000,20,-128000000000]` and saved reopen. Real
Tab/Shift-Tab wrapping, primary field focus and Apply/Cancel, malformed-number and
nonuniform-sphere refusal, pending-input preservation, unchanged submit, all-field
atomic Apply and visual undo/redo pass. Six resulting complete documents match
independently executed headless Transform commands exactly. Saved far fields
reopen exactly without an unchanged-submit history entry. Missing selection,
modal isolation, busy mailbox and draft/playtest refusal pass alongside the full
retained naming walkthrough. All three observe active Vulkan validation on AMD
RX 7900 XTX (RADV NAVI31), device 29772, with zero GPU/resize diagnostics. The
owned input driver recorded two harmless missed-Control re-presses before sending
select-all; it never retried mutating clicks or commands. Normal-game and Windows
evidence retain their earlier scope.

Evidence: ignored `build/zone-properties-native-3440-v1/{qualification.json,*.png}`,
six `.before.json` / `.commands.jsonl` / `.expected.json` pairs in that directory,
`zone-properties-tests-v1.log`, `zone-properties-build-v1.log` and
`zone-properties-clippy-v1.log`. Editor SHA-256:
`19a8276ec6158df6ca194ab34c895331453885f4c69fe7e2afc052c57d238fcf`.

## Form error feedback on edit — 8 November 2026

P5-17 / P3-15 / P1-10: actual name/property text changes clear the previous
attempt's status and restore the editing instructions. Caret/selection changes
do not do so. This avoids showing a stale invalid-number/name result after a
correction; it does not claim the pending values are valid or run a second
validator. Apply still validates the current complete candidate and preserves
invalid buffers/history exactly as before. Cancel/unchanged-submit semantics and
all source/runtime formats remain unchanged.

All 68 focused Rust cases, formatting and denied-warning workspace/all-target
Clippy pass. A focused native far-frame case at actual 3440x1440 observes both
errors, corrects both buffers through real text input, verifies refreshed messages
and unchanged document/revision/history, then cancels the name and submits the
unchanged pose to restore clean state. Vulkan validation is active on RX 7900 XTX
(RADV NAVI31), device 29772, with zero GPU/resize diagnostics and no input retries.
Earlier full naming/property, normal-game and Windows evidence retains its scope.

Evidence: ignored `build/zone-form-feedback-native-3440-v1/{qualification.json,*.png}`,
`zone-form-feedback-tests-v1.log`, `zone-form-feedback-build-v1.log` and
`zone-form-feedback-clippy-v1.log`. Editor SHA-256:
`aa913989f4c4b9dbc4184da82959eaef6a36cc986f76b57645b95030b8edeb06`.

## Terrain-conforming sample influence — 8 November 2026

P5-17 / P3-15 / P1-10; P2-06/07. Prerequisites are the retained 4 m / 2 m height
patch, shared sculpt/paint commands and checked local/global conversion. The
amber brush now follows exact terrain triangles rather than a flat cylinder.
Sculpt and paint share `HeightPatch::brush_influence`: finite in-patch global
centre, 4..32 m radius, existing linear f64-distance/f32-weight falloff and zero
influence on the locked border. Their operation/blend/quantization and complete
candidate validation remain in the existing commands.

The standard Bevy mesh uses the terrain's exact indices and local positions,
lifted 0.04 m; unlit amber vertex alpha is half the potential sample influence.
Interpolation follows the grid triangles, so its visible edge can extend to
adjacent zero-weight samples rather than tracing an analytic circle. It does not
predict final height, blend strength, paint rounding or an operation's no-change
refusal. The overlay follows temporary sculpt heights while a stroke is held.
It reuses one resident asset, caches source/draft revision, centre and radius,
and never participates in ray picking, shadows or collision. Its bounded single
entity skips frustum culling to avoid stale mesh bounds after replacement. It
hides outside terrain, over panels, during modal input/playtest, in other modes
or when no unlocked samples have influence. Cursor/settings alone never mutate
source/history; release retains the existing one-entry stroke commit. There is
no new source/runtime format, engine feature, renderer or server-authority path.

All 72 focused Rust cases (51 domain / 21 editor), formatting and denied-warning
workspace/all-target Clippy pass. New cases cover known weights, invalid geometry,
locked edges, actual raise/paint agreement, both resolutions and both far-frame
signs, exact mesh indices/positions/alpha and empty/outside targets. Ten complete
historical sculpt/paint documents match the new headless output exactly, including
0.125 m strength and 12.5% flatten/smooth/paint on both grids.

Native Linux qualification at actual 3440x1440 covers a 4 m hill and 2 m valley
at `[128000000000,20,-128000000000]`: observed resident mesh samples match the
independently calculated falloff and exact surface heights on peaks, slopes,
locked edges and a held temporary sculpt preview. Radius changes, outside-patch and UI/mode/name-modal hiding,
clean source/history before editing and one-entry release pass. Screenshots were
captured and the slope visually inspected. Vulkan validation is active on AMD
RX 7900 XTX (RADV NAVI31), device 29772, with zero GPU/resize diagnostics. The retained native
strength/blend walkthrough also passes on both grids: exact headless equivalence,
one-entry strokes, undo/redo/save and actual offline Avian playtest, with zero
GPU/resize diagnostics. Earlier normal-game, multiplayer and Windows evidence
retains its scope.

Evidence: ignored `build/zone-influence-native-3440-v3/{qualification.json,*.png}`,
`build/zone-influence-compat-v1/qualification.json`,
`build/zone-influence-blend-regression-3440-v1/qualification.json`, `zone-influence-tests-v2.log`,
`zone-influence-build-v2.log` and `zone-influence-clippy-v2.log`. Editor SHA-256:
`bfd9035848c822d74e6b83a1359da59df3612d1eed3f3878bc175b786a51d58f`.

## Bounded stroke feedback — 8 November 2026

P5-17 / P3-15 / P1-10; P2-06/07. The retained shared transaction limit remains
128 edits. Sculpt/paint previews now display accepted stamp counts in the
summary and status; both help panels explain the limit. The 128th accepted
stamp explicitly reports that the limit is reached and asks for release to
commit or Escape to cancel. Further motion leaves the preview unchanged under
the existing cap. Rejected/no-change stamps do not consume capacity. No timing,
spacing, command validation, history, source schema or stroke lifecycle changes.
The optional report exposes accepted count and limit for a held stroke.

All 72 focused Rust cases, formatting and denied-warning workspace/all-target
Clippy pass. Native 3440x1440 qualification performs all 128 accepted stamps,
observes counts while committed source/revision/history remain unchanged,
confirms further motion freezes the capped preview and sees the explicit message.
Release commits exactly that preview as one undo entry; undo/redo and cancellation
of a fresh stroke preserve the expected source/history. Vulkan validation is
active on AMD RX 7900 XTX (RADV NAVI31), device 29772, with zero GPU/resize
diagnostics. This is Linux editor evidence; previous Windows/multiplayer scope
is unchanged.

Evidence: ignored `build/zone-stroke-limit-native-3440-v1/{qualification.json,capped.png}`,
`zone-stroke-feedback-tests-v1.log`, `zone-stroke-feedback-build-v1.log` and
`zone-stroke-feedback-clippy-v1.log`. Editor SHA-256:
`47a97fb97d6e24ea33f3801473680ea7b09686a428573064df5cee5117cb57e9`.

## Spatially sampled terrain drags — 8 November 2026

P5-17 / P3-15 / P1-10; P2-06/07. Fast pointer jumps previously placed only an
endpoint stamp. Sculpt/paint drags now sample the connecting segment every 2 m
in X/Z from the last attempted stamp. The first hit stamps once; shorter motion
and stationary holds add no work. Unused distance carries through the retained
last stamp; an endpoint less than 2 m beyond it is not forced on release. Height
is linearly interpolated only for checked local/global conversion; the existing
commands determine actual height/colour changes from X/Z and source data. This
connects observed input samples with straight segments, not unobserved mouse
motion. It is not time-based brush accumulation.

Each event allocates at most the remaining accepted-edit capacity (128 total).
Every generated stamp uses the same validated disposable Editor and existing
Brush/Paint edits. Failed/no-change attempts advance the spacing anchor but do
not consume accepted capacity. Leaving the canvas, missing a ground hit or
moving outside the patch clears the anchor; reentry starts one fresh stamp
instead of bridging the gap. Source/history stay committed until release, which
applies the accepted edits as one transaction. Escape/focus-loss cancellation
and releasing over a panel retain their earlier behavior. The optional report
also exposes bounded accepted preview edits for headless equivalence checks.
There is no new shared command, source schema, authority or renderer path.

All 75 focused Rust cases (51 domain / 24 editor), formatting and denied-warning
workspace/all-target Clippy pass. New tests cover diagonal/height interpolation,
first/zero/short/stationary input, capacity bounds and equal straight-path samples
across coarse/fine input events. Native actual 3440x1440 qualification passes on
4 m local and 2 m far `[128000000000,20,-128000000000]` terrain. About-64 m OS
jumps produce 32 evenly spaced sculpt and paint edits on each grid; stationary
holds add none. The six complete resulting documents (including interrupted
strokes) match independently executed headless commands exactly. Preview/history
isolation, one-entry release, undo/redo/save and panel/outside-patch discontinuity
checks pass. The cap walkthrough also passes with interpolated multi-stamp
frames, frozen capped preview, exact one-entry commit and cancellation. Vulkan
validation is active on RX 7900 XTX (RADV NAVI31), device 29772, with zero GPU/
resize diagnostics. Linux editor evidence adds no Windows or multiplayer claim.

Evidence: ignored `build/zone-stroke-spacing-native-3440-v1/{qualification.json,*.png}`,
six `.before.json` / `.commands.jsonl` / `.expected.json` pairs in that directory,
`build/zone-stroke-spacing-limit-native-3440-v1/qualification.json`,
`zone-stroke-spacing-tests-v1.log`, `zone-stroke-spacing-build-v1.log` and
`zone-stroke-spacing-clippy-v1.log`. Editor SHA-256:
`48b016b1327b818172d7bb57f0127bc9b567ed309a4fe9ddbb1807485cd2ab0e`.

## Surface-supported playtest spawn — 8 November 2026

P5-17 / P3-15 / P1-10; P2-06/07. P over ground in Sculpt/Paint places the playtest
spawn using the retained SetSpawn edit and full candidate validation. Cursor X/Z
uses the centralized checked f32-local/f64-global conversion, with 1 m snapping
relative to zone origin when G is enabled; disabled snapping preserves the ray
hit's X/Z through that conversion. Exact triangular terrain support (or implicit
flat zone-origin Y outside the patch) supplies height, plus the existing 0.1 m
clearance. Cursor-hit Y is not trusted as authored support. The existing object
palette's view-centre spawn button now shares this checked conversion/sampler
and retains its unsnapped X/Z and transaction behavior. Failed conversion/sample
attempts report errors instead of silently substituting a height.

Cursor placement equal to the current spawn reports that fact without history.
Pending strokes/held mouse refuse placement while retaining the preview. Both
side panels, modal fields, unfocused windows, playtest and other modes isolate P;
text inputs retain their ordinary P key. Help explains the shortcut. Successful
placement is one existing undo entry; save/load/export reuse the same spawn data.
This supports terrain/flat ground, not prop-stack support or navigation checks.
Actual offline Avian entry still checks physical occupancy and may refuse a
blocked location. No new command/schema, renderer or server-authority claim.

All 77 focused Rust cases (51 domain / 26 editor), formatting and denied-warning
workspace/all-target Clippy pass. New cases cover snapped/precise surface heights
on both resolutions and both signed 128-billion-metre frames with Y=20, shared
SetSpawn undo, implicit flat support and invalid/bounded-coordinate refusal.
Native actual 3440x1440 checks pass on 4 m positive local and 2 m negative far
terrain: snapped/precise cursor placement in both modes, unchanged cursor no-op,
view-centre placement, undo/redo/save, panel/name-modal/playtest isolation and
pending-stroke refusal without preview loss. Six complete documents match
independent headless SetSpawn runs exactly. Actual Avian playtest starts on both
surfaces. Vulkan validation is active on RX 7900 XTX (RADV NAVI31), device 29772,
with zero GPU/resize diagnostics. Earlier Windows/multiplayer scope is unchanged.

Evidence: ignored `build/zone-surface-spawn-native-3440-v1/{qualification.json,*.png}`,
six `.before.json` / `.commands.jsonl` / `.expected.json` pairs in that directory,
`zone-surface-spawn-tests-v2.log`, `zone-surface-spawn-build-v2.log` and
`zone-surface-spawn-clippy-v1.log`. Editor SHA-256:
`8fd58f5e084b20e56025d2783417426ecfdbd6938eb481c8ee595a787deef946`.

## Readable spawn marker and retained terrain meshes — 8 November 2026

P5-17 / P3-15 / P1-10; P2-06/07. The small spawn sphere is replaced by one
standard Bevy cone, 1.3 m wide and 2.4 m tall, pointing down at the authored
spawn. Its unlit green material stays legible in terrain shade. It uses the
existing marker entity/mesh/material allocation, ignores picking, casts no
shadow and adds no collision/export/source data. It hides while the real
playtest pawn is visible and returns on leaving playtest. Position still uses
checked origin conversion. The optional report exposes actual mesh bounds,
visibility and projected centre for native checks.

Native review also caught a transient blank world capture immediately after
spawn placement: every document revision replaced unchanged terrain/grid assets.
Terrain sync now compares retained origin/patch inputs after revision changes;
spawn/name/prop-only edits preserve resident terrain meshes. Actual origin,
height, weight, resolution, chunk or terrain-presence changes still refresh the
surface/grid/cutout, including temporary previews and undo. The bounded cache
owns one patch snapshot and adds no document history or authority path.

All 77 focused Rust cases, formatting and denied-warning workspace/all-target
Clippy pass. Native 3440x1440 local/far surface-spawn checks retain all six exact
headless documents, history/save/input isolation and actual Avian entry. They
also verify one resident marker mesh, exact 2.4 m height anchored at spawn,
rendered green pixels at whole-zone framing and playtest hide/restore. Vulkan
validation is active on RX 7900 XTX (RADV NAVI31), device 29772, with zero GPU/
resize diagnostics. The retained local/far flatten walkthrough also passes after
the cache change: actual terrain editing, exact headless equivalence, undo/redo,
save and offline Avian playtest. Linux evidence adds no Windows or multiplayer claim.

Evidence: ignored `build/zone-spawn-marker-native-3440-v2/{qualification.json,*.png}`,
`build/zone-spawn-marker-terrain-regression-3440-v1/qualification.json`,
`zone-spawn-marker-tests-v2.log`, `zone-spawn-marker-build-v2.log` and
`zone-spawn-marker-clippy-v2.log`. Editor SHA-256:
`858fb83c1def1f89a4984c0a855188cfaa480fbc47a0590bfb81dce870af687e`.

## Application-style workspaces and properties — 9 November 2026

P5-17 / P3-15 / P1-10; terrain P2-06/07. The user explicitly retains our editor,
with Blender-like application workflows rather than larger controls. Actual
prerequisites are the existing shared commands/presets, Bevy UI widgets, standard
focus bridge and checked coordinate pipeline. No external editor/UI framework,
engine/plugin pin, source schema or gameplay-authority change is introduced.

A standard Bevy command bar exposes Save, Open saved (the existing draft path),
Undo, Redo, Frame selection, Spawn at view centre, Playtest/Return and a Shortcuts
reference toggle. Workspace buttons select Objects, Sculpt, Paint, Paths/streams,
NPC/quest markers and Vegetation. Active modes are highlighted. A built-in Checkbox
controls 1 m snapping and reflects keyboard changes. Relevant command buttons
are disabled for absent selections/history and during modal/pending/play states;
returning from playtest remains available. Workspace controls refuse pending
previews instead of discarding them. Shared shortcuts remain available.

The left palette now contains tools rather than duplicated global commands or
radius increment buttons. The right column splits its existing bounded content
browser (24 rows per page, reduced from 40) above a 600 px properties region.
Readouts show authored values and live object-drag previews, with explicit axes
and units rather than revision/debug counters. Numeric transforms open in this
region rather than a full-window scrim. They retain standard TextInput/Tab focus,
f64 round-trip strings, grouped Apply/Cancel validation, modal input isolation
while editing, and one existing Transform undo entry. Rename remains a conventional
modal dialog. The inspector also exposes Rename, Duplicate, Ground and Frame.

Built-in Bevy Slider/SliderThumb/SliderRange widgets edit terrain radius 4..32 m,
raise/lower strength 0.05..8 m, independent sculpt/paint blend 5..100%, and flatten
level -64..64 m relative to origin. Only relevant settings appear. Track clicks,
thumb dragging and labels are synchronized with existing keyboard presets.
These settings are editor-local and never create source/history entries; actual
strokes retain shared command validation/undo. Sliders and the snap checkbox
refuse modal/pending/playtest changes. World pointer actions and camera mouse
input exclude the header, side panels and bottom status region. The optional
report adds actual workspace/slider bounds, values and ranges without changing
the mailbox protocol.

All 77 focused Rust cases (51 domain / 26 editor), formatting and denied-warning
workspace/all-target Clippy pass. Native local 4 m / far 2 m qualification uses
actual 3440x1440 on RX 7900 XTX (RADV NAVI31), device 29772, with Vulkan validation
active and zero GPU/resize diagnostics. Mouse workspace selection, snap toggles,
slider track clicks/thumb drags, independent blends and clean history pass.
Actual paint stamps from slider presets match full headless command documents;
one-entry undo/redo/save, protected guide drafts, header/world isolation,
content selection/paging, docked exact transform editing and playtest isolation
also pass. The docked transform's numeric edit matches the shared headless
Transform document exactly, with one-entry undo/redo. Screenshots cover both
properties modes. Earlier fixed-layout UI qualification retains historical scope;
this does not claim Windows execution, multiplayer or final human usability.
The retained full naming/properties walkthrough also passes at local/far origins
and saved reopen, with actual docked fields for boxes/spheres/cylinders, exact
six-command equivalence, parse/shape refusal, correction, mutual modal/agent
isolation and unchanged submission. Both local and far dock screenshots were
visually inspected. All three runs observe active Vulkan validation and zero GPU/
resize diagnostics. One missed Control modifier was re-pressed before select-all;
no mutating clicks or commands were retried.

The first slice does not yet provide resizable docking, file-picker/Save As,
automatic inline transform commits, viewport move/rotate/scale handles, or direct
properties for every guide/marker/plant field. Those are subsequent application
workflow work; this is not a complete Blender-style editor implementation.

Evidence: ignored `build/zone-workspace-native-3440-v4/{qualification.json,*.png}`,
paint/transform `.before.json` / `.commands.jsonl` / `.expected.json` pairs,
`build/zone-workspace-properties-native-3440-v1/qualification.json` and dock screenshots,
`zone-workspace-tests-v2.log`, `zone-workspace-build-v5.log` and
`zone-workspace-clippy-v3.log`. Editor SHA-256:
`1ebf50076ee77c9418bb2a33a20c7a5633863269ce3c77f9cc97c46ea6dfa00b`.

## Vegetation scatter properties — 9 November 2026

P5-17 / P3-15 / P1-10 follow-up to the application shell. Prerequisites are
its standard Bevy sliders and the existing bounded seeded ScatterVegetation
command. No source-schema, renderer, engine/plugin-pin or gameplay change.
Choosing Scatter trees or Scatter shrubs reveals radius 1..32 m and plants per
click 1..64 in Properties. Count uses integer step/precision and rounds accepted
values; radius uses a 0.1 m widget step. Single-plant placement and other
workspaces hide these settings. Existing bracket/comma/period presets remain
synchronized. Editor-local settings preserve document, revision and undo/redo;
actual placement retains candidate validation and one scatter undo entry. Existing
focus, modal, pending-preview and playtest guards apply to both new settings.
The vegetation readout rounds radius to a useful one decimal place rather than
showing binary floating-point noise. The optional report adds the checked global
plant cursor point for native command-equivalence checks.

Acceptance: retained 77 focused Rust cases, formatting, strict workspace/all-target
Clippy, actual local 4 m and far-origin 2 m native 3440x1440 walkthroughs. Mouse
track click and thumb drag, 1/64 count bounds, integer 17-plant setting, contextual
visibility, unchanged history/document and independent terrain presets are checked.
Actual tree and shrub groups match complete headless shared-command documents,
with one-entry undo/redo and exact saved output. The existing application-shell,
paint, docked transform and playtest walkthrough is retained in the same run.
This qualifies scatter proxy authoring; production vegetation assets/collision,
Windows execution and a complete Blender-style editor remain separate work.

All acceptance checks above passed on the final build. Both runs used RX 7900 XTX
(RADV NAVI31), Vulkan device 29772, with active Khronos validation and zero GPU/
resize diagnostics. The final properties screenshot was visually inspected.
Evidence: ignored `build/zone-scatter-controls-native-3440-v2/qualification.json`,
`*.scatter-properties.png`, `*.scatter-result.png` and matching
`*.scatter.before.json` / `*.scatter.commands.jsonl` / `*.scatter.expected.json`;
`zone-scatter-controls-tests-v2.log`, `zone-scatter-controls-clippy-v2.log` and
`zone-scatter-controls-build-v4.log`. Editor SHA-256:
`1afef7ed59db50b90be09b981277589c460c2f433db7a01d17fbc4687b2201a7`.

## Plant and marker transform properties — 9 November 2026

P5-17 / P3-15 / P1-10 follow-up. Actual prerequisites are the retained standard
Bevy docked TextInput form and shared PutPlant/PutMarker commands. The Transform
button and F9 now open relevant fields for selected tree/shrub proxies or NPC/
quest annotations. These show exact global f64 XYZ metres, yaw degrees and either
dimensionless uniform scale or authoring radius in metres. Unused size rows hide
through both Display::None and Visibility::Hidden, since standard TabNavigation
follows inherited visibility. Tab/Shift-Tab navigation skips them. Reopening a primitive restores
all seven position/yaw/size fields and their labels. Field buffers retain round-trip
numeric strings. All values parse as finite f64 numbers; shared whole-candidate
validation still owns bounds and ranges. Invalid Apply keeps the typed buffer and
history, and actual edits clear stale errors. Apply commits one existing command;
unchanged Apply and Cancel preserve source/revision/history, including pending
redo. Name, kind, ID and optional marker definition are retained by cloning the
selected source record. Marker radius remains an authoring footprint, and plant
scale changes fixed visual proxies; neither creates gameplay behavior/collision.

Existing modal focus/window-loss cancellation, pending-preview/playtest refusal,
agent-mailbox busy refusal and atomic undo/redo are retained. This adds no source
schema, transport or engine/plugin dependency change. Path/point numeric properties,
catalogue pickers, resizable docking and viewport transform handles remain future
application work.

Acceptance covers all four plant/marker kinds at local and far origins, invalid
and nonfinite fields, exact unchanged round trips, preserved metadata, shared
candidate range refusal with pending redo, and one-entry undo/redo. Retained
primitive and naming checks plus native actual 3440x1440 properties interaction
qualify the reusable row/focus layout, Apply/Cancel, save and reopen. All these acceptance checks passed.

The final source passes 78 focused Rust cases (51 domain / 27 editor), formatting
and denied-warning workspace/all-target Clippy. Native local/far/reopen Linux
runs use actual 3440x1440, RX 7900 XTX (RADV NAVI31), Vulkan device 29772 and active
Khronos validation, with zero GPU/resize diagnostics in all three. Eight new
plant/marker edits and six retained primitive edits match complete headless
command documents. Saved reopen is exact with fresh zero history. Standard
five-field Tab/Shift-Tab wrapping and restoration of all seven primitive fields
pass; local/far plant and far NPC screenshots were visually inspected. One missed
Control modifier was re-pressed before select-all; no mutating action was retried.
Windows execution and gameplay multiplayer were not requalified by this UI slice.

Evidence: ignored `build/zone-plant-marker-properties-native-3440-v2/qualification.json`,
`*.properties-100{0,1,3,4}-ready.png` and all matching transform
`.before.json` / `.commands.jsonl` / `.expected.json` pairs;
`zone-plant-marker-properties-tests-v2.log`,
`zone-plant-marker-properties-clippy-v2.log` and
`zone-plant-marker-properties-build-v2.log`. Editor SHA-256:
`68a4a728c39a0f0685ae778d92218d378d54641d9428c5405db00c5b75f5a5df`.

## Guide properties and point controls — 9 November 2026

P5-17 / P3-15 / P1-10 follow-up, using the retained standard Bevy docked form,
shared PutPath/InsertPathPoint/RemovePathPoint commands and whole-candidate history.
Whole-guide properties show width (0.25..32 m); selected-point properties show
exact global f64 XYZ metres plus width. One Apply groups both changes into one
existing PutPath edit and undo entry, preserving ID/name/kind and every other
point. Unchanged submissions and Cancel preserve history. Invalid width, position
or segment geometry retains the buffer and source/history. Fields use existing
finite-number parsing, explicit visibility for Tab navigation, revision checks and
modal isolation. Primitive/plant/marker forms retain their existing layouts.

Contextual properties buttons select Previous/Next point (cycling through points
and the whole guide), Whole guide, Insert point and Remove point. Selection is
editor-local and does not edit history; topology uses the existing shared commands.
Buttons hide outside the guide workspace and refuse modal/pending/play changes.
Insert requires a selected point and fewer than 64 points; Remove requires a
selected point and more than two. Geometric constraints still use shared candidate
validation. Width/point readouts are not new terrain carving or gameplay routes.
No source schema, engine/plugin pin, export or gameplay-authority change.

Acceptance covers roads/streams at local/far origins, exact width-only and grouped
point/width command equivalence, unchanged round trips, preserved other points,
segment/range refusal with pending redo and one-entry undo/redo. Native checks
exercise the actual 3440x1440 controls, one/four/seven-field reuse, Tab wrapping,
outliner paging, invalid correction, topology selection, two-point minimum and
save; retained workspace/paint/primitive/playtest checks run in the same fixture.

All acceptance checks above passed. The final source passes 79 focused Rust cases
(51 domain / 28 editor), formatting and strict workspace/all-target Clippy.
Native local 4 m road / far 2 m stream runs use RX 7900 XTX (RADV NAVI31), Vulkan
device 29772, actual 3440x1440 and active Khronos validation, with zero GPU/resize
diagnostics and no input retries. The local point-properties screenshot was
visually inspected. Four guide edits match complete headless command documents;
shared one-entry history, exact saved output, invalid correction, mouse topology
and one/four/seven-field restoration pass. Windows/gameplay multiplayer were not
requalified by this UI slice.

Evidence: ignored `build/zone-guide-properties-native-3440-v2/qualification.json`,
`*.guide-width-ready.png`, `*.guide-point-ready.png` and matching `.before.json` /
`.commands.jsonl` / `.expected.json` pairs; `zone-guide-properties-tests-v2.log`,
`zone-guide-properties-clippy-v2.log` and `zone-guide-properties-build-v2.log`.
Editor SHA-256: `7b34a22aaf3c5ce4926f168f2cdc09fd962ae067512c8b2ce82f5605986d2394`.
