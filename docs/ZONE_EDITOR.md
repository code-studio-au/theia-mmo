NPC spawn-group groundwork (12 November 2026): schema-14 zone drafts can store
validated custom polygon/height bounds, creature/model IDs, alive count, respawn
delay and seed through the shared command API. Add → NPC spawn zone at spawn
creates a rectangle; use the NPC spawn zones explorer filter and Attributes for
inline IDs, seed, enabled, centre X/Z, height limits, alive count and respawn delay.
Select a group and press F to frame its polygon; rename, copy, delete, undo and
explicit Save use the normal controls. In Attributes, Previous/Next vertex cycles
between the whole boundary and numbered vertices. A selected vertex exposes exact
X/Z coordinates; Whole boundary restores centre translation. Split edge inserts
the midpoint after the selected vertex (including the closing edge), and Remove
vertex retains at least three vertices. Invalid crossings, short edges, invalid
coordinates and out-of-zone shapes refuse without changing document/history.
Pending inline edits commit before a boundary action. Changes remain in memory
until explicit Save. The shared JSONL command is `edit_spawn_boundary`, with
`id` and an `edit` carrying `operation: move|insert|remove`; move uses `index`
and `position: [x,z]`, insert uses `after` and `position`, remove uses `index`.
These boundary controls are headlessly tested; native layout/input/GPU checks
remain pending under the user's no-display/no-mouse instruction (13 November).
Drafts remain explicitly refused by world export until population integration exists. An offline placement checker can now test a separate request against an
existing exported world/navigation pair:

```sh
target/debug/theia-nav-bake spawns-authored \
  build/valley-patrol-binding-v1/world \
  build/valley-patrol-binding-v1/navigation \
  content/navigation/starter-valley-spawn-candidates.json
```

To check a saved editor spawn group without copying its bounds into a request:

```sh
target/debug/theia-nav-bake spawns-zone WORLD NAVIGATION ZONE_JSON GROUP_ID 3 ACCESS_X ACCESS_Y ACCESS_Z
```

`3` is an explicit minimum spacing in metres; the final XYZ is a known accessible
feet anchor. Disabled/unknown groups refuse. The checker uses saved bounds/count/
seed and reports group metadata plus a canonical draft digest. It does not save
the zone or prove that its other draft geometry matches the supplied world.
Creature/model resolution and live spawning remain separate.

It verifies development-capsule standing space, custom bounds, spacing and sampled
access routes; reports rejection counts and insufficient results; and creates no
NPCs or files. Its vertical limits apply to feet, and the access anchor does not
set roaming/chase limits. This is bounded offline placement evidence, not live
population capacity or support for arbitrary creature collision sizes.

NPC quest inspection (P5-17 / P5-18, 12 November 2026): select an NPC in the
explorer, choose Attributes and expand an entry under Quests. Entries show whether
that NPC offers the quest, accepts turn-in or both; requirements, stages/objectives
and rewards come from the existing catalogue. Use File → Load gameplay… or
Change / reload gameplay… to select a validated gameplay manifest. The browser
starts beside the zone if no template is loaded, otherwise beside that template.
`--gameplay-template MANIFEST` also works when opening the interactive editor.
This selection supplies the export default for the session; zone Save does not
write the template or persist that selection. Quest definitions remain content
files; character progress and durable rewards remain authoritative database state.
Quest authoring and general multi-contact gameplay are not implemented by this UI.
The native test covers expansion, reload/cancel, unbound selection and unchanged
source files; exact scope and GPU evidence are in STATUS.md.

P5-17 patrol gameplay binding, 12 November 2026:
Exporting with a gameplay template now connects its creature to the single authored
Patrol guide automatically. Keep one Patrol guide for this current single-creature
runtime; multiple guides are allowed as standalone requests, but a gameplay export
refuses an ambiguous assignment. Names can change without changing the route ID.
Combat/behavior values and creature/quest identities remain those of the template.
After exporting, bake the emitted navigation-requests/patrol-ID.json with the offline
baker and launch with that matching world/navigation pair. An editor Bake button and
multiple-creature spawn groups remain future work.

The current review copy is build/valley-patrol-binding-v1/valley.zone.json, with
Western grove patrol #1002. Its world/ and navigation/ exports pass an actual
1,200-step authoritative patrol loop. Use its world/gameplay/gameplay.manifest.json
as the template for subsequent exports. Original village/contact copies are preserved.
See STATUS.md for evidence and the distinction from the earlier connected quest run.

P5-17 patrol route requests, 12 November 2026:
Choose **Guides -> Tools -> New patrol route**, click ground points and press Enter.
Select its Explorer row or point handles; Attributes edits coordinates and Preview
width directly. The width changes only the authoring ribbon, not NPC clearance.
Normal rename, point insert/remove, movement, undo and explicit Save apply. Patrols
are purple (gold when selected), hidden during F5 offline playtest, and omitted from
exported visual/collision geometry. Grade terrain refuses this guide kind.

World export now writes `navigation-requests/patrol-ID.json`. Its stable logical ID
is `theia.zone.patrol.ID`; names/point edits retain it. These files are requests, not
validated navigation or spawned NPCs. Bake/check one against its matching world:

```sh
target/debug/theia-nav-bake bake-authored WORLD NEW_NAVIGATION WORLD/navigation-requests/patrol-ID.json
```

The baker keeps its nine-tile/256-output-point and sampled collision checks. Use
actual authored terrain: strict support checks currently reject some points on the
empty-zone fallback's giant single box floor. Existing terrain valley checks remain
qualified. Creature-profile assignment and a native Bake button are separate work.
See STATUS.md and tools/qualify_editor_patrol.py for exact scope and evidence.

P5-17 authored valley gameplay acceptance, 12 November 2026:
The introduction, three-kill follow-up and western-grove area quest now all complete
through ordinary connected play and survive database restart/rejoin (150 XP,
revision 14, three completions). Use the stable local gameplay copy below with the
matching contact-v3 world/navigation. This remains one guide and one respawning
creature; multiple-NPC authoring and quest item pickups are separate. One death /
respawn occurred in the complete native run; balance/health recovery is not qualified.

```sh
python3 tools/play.py --account-login --classic \
  --authored-world build/starter-valley-contact-world-v3 \
  --navigation build/starter-valley-contact-navigation-v3 \
  --gameplay-manifest build/starter-valley-playable-gameplay-v1/gameplay.manifest.json \
  --players build/classic-directional-client-qualified-v1-20261103/art/player
```

This normal command requires configured local account/database services. Longer
playtests need both the issuer token lifetime and THEIA_ACCOUNT_SESSION_SECONDS
set explicitly; see AUTHENTICATED_ADMISSION.md. The reproducible owned-service
qualification uses 1800 seconds, closes its private services afterwards, and is not
a persistent user account. See STATUS.md for exact evidence and current limitations.

P5-17 authored account quest playtest, 12 November 2026:
The normal launcher now supports the complete authored gameplay bundle, explicit
matching navigation and the exported guide contact. The earlier dated restrictions
below are historical. The first introduction quest, 25 XP reward, durable database
restart and rejoin are qualified; wolf-kill/overlook completion and multiple contacts
remain next. The editor's Network Playtest still tests movement independently.
With the existing local account/identity/database services configured:

```sh
python3 tools/play.py --account-login --classic \
  --authored-world build/starter-valley-contact-world-v3 \
  --navigation build/starter-valley-contact-navigation-v3 \
  --gameplay-manifest build/authored-quest-intro-native-v2/gameplay/gameplay.manifest.json \
  --players build/classic-directional-client-qualified-v1-20261103/art/player
```

Use this exact matching export/navigation pair. The newer network-test snapshot has
a different source identity. No zone source is changed by this launch; the original
village file is preserved. See STATUS.md for qualification scope and devices.

P5-17 authored creature preflight, 12 November 2026:
The normal launcher checks authored creature sessions with the actual client's
--configuration-report before starting the server/login. Navigation must bind the
same world, pass tile hashes and contain the creature profile's route ID. Missing
route or mismatched navigation now produces an early configuration error. This
is not a runtime patrol/quest completion check, and full authored gameplay bundles
remain gated. See STATUS.md for the matching west-grove candidate/evidence.

P5-17 account playtest prerequisite, 12 November 2026:
Exported authored worlds can now use the normal account-backed walking launcher:
`python3 tools/play.py --authored-world WORLD --classic --account-login --players PLAYER_ASSETS`.
This requires the existing configured local account/identity/database services.
Fresh movement/checkpoint and exact XYZ rejoin are qualified on Linux; see
AUTHENTICATED_ADMISSION.md. The editor's Network Playtest remains a separate
movement-testing session with development tickets. Authored full quest bundles
and an editor account-playtest button are not yet enabled.

P5-17 / P1-06 managed editor Network Playtest, 12 November 2026:
Add and enable one or two Test Players, then choose **Zone -> Start network
playtest**. Choose a fresh output folder, the player model/animation directory
(containing character.glb), and a matching gameplay template when the zone has
quest areas or bound NPC contacts. The player path defaults from --player-assets;
the current editor process remembers the last template/player directory.
The dialog exports the current immutable zone/scenery/controller snapshot and
launches one local authoritative server plus one independent encrypted client per
enabled pawn. Their native windows arrange side by side (or vertically on tall
displays), show the actual server-assigned player IDs, and keep rendering while
unfocused. You can move/resize them afterwards. Small displays use a cascade;
these remain separate processes/windows, not embedded editor camera panels. Test-player attributes determine spawn, controller and network
conditions. **Zone -> Stop network playtest** ends the whole owned session.
Closing or losing the editor also closes its private launcher pipe and cleans up
its server/clients. F5 / Offline playtest is separate and disabled during a
network session. Setup edits affect the next launch; only Save writes the zone.

This development workflow requires Python 3, tools/network_playtest.py in the
checkout, and the client/server binaries beside the editor. Build all three
applications together. THEIA_NETWORK_TEST_PYTHON can select the interpreter.
Each snapshot keeps runs/session-*/ logs and a completion summary; failures name
the launcher log in the status bar. Exports remain for inspection, temporary
connection tickets are removed on cleanup. No account-backed quests/combat or
embedded split-camera view is enabled by this movement-testing launcher.
Native Start/Stop/restart and abrupt owner-exit cleanup pass on the current Linux
Vulkan machine; tools/qualify_editor_network_playtest.py reproduces the check.
See STATUS.md for exact evidence and limitations.

P5-17 / P1-06 Test Player setup in the editor, 12 November 2026:
Use Add -> Test player at spawn (maximum two), then Explorer -> Test players to
filter/select the setup pawns. These are nonblocking capsule badges. Move them
with XYZ arrows or the permanent Position fields; Frame, Ground, Rename,
Duplicate, Delete and Undo/Redo share normal authoring history. Duplicate uses
the other client slot. Rotation and scale are disabled until the runtime spawn
profile supports them. F5 remains the offline preview and hides test-pawn badges.

Attributes -> Network conditions edits delay in milliseconds **each way**, added
jitter, packet-loss percentage and the random seed. Thus 80 ms delay adds 160 ms
base round-trip latency before jitter and actual transport latency. The three-dot
properties menu toggles whether the client launches, or assigns Manual / Walk and
turn controller. Changes apply immediately to setup; Save writes the zone file.
Export/launch takes a snapshot; editing setup does not change an already running
session. Scheduled outages remain supported by JSON, without an inspector yet.

The native Linux qualification uses real owned UI input through add/filter,
controller/enabled selection, inline position/delay edits, axis dragging and
undo/duplicate/delete. Source bytes remain unchanged without Save; all 213
ordinary editor tests pass. `tools/qualify_test_player_ui.py --output NEW_DIR`
repeats the native check; see STATUS.md for device/evidence. Network test export
and launch are also available through the CLI paths below.

P5-17 / P1-06 network-test export bridge, 12 November 2026:
Use `--export NEW_DIRECTORY --network-test --content-id ID --revision N` to turn
enabled saved Test Players into a managed-launcher scenario. Supply the usual
`--scenery-library` and, for zones containing quest areas, `--gameplay-template`.
The new directory contains `world/`, `scenario.json`, assigned controller copies
under `controllers/`, and `network-test.json`. It refuses an existing output,
missing enabled pawns, unknown enabled controller IDs, unsupported network worlds
and blocked/unsupported spawns; owned partial output is removed. The source zone
is unchanged. Currently `test.walk-turn` resolves to the bundled movement example;
null means manual input. Disabled pawns retain their setup but do not launch.

For the current local test-pawn copy (the user's original village is preserved):

```sh
target/debug/theia-zone-editor \
  --zone build/starter-valley-test-pawns-v1.zone.json \
  --scenery-library build/starter-valley-scenery-v2/catalog.json \
  --gameplay-template build/starter-valley-contact-world-v3/gameplay/gameplay.manifest.json \
  --export build/my-new-network-test --network-test \
  --content-id theia.starter.valley.network-test --revision 1
python3 tools/network_playtest.py \
  --scenario build/my-new-network-test/scenario.json \
  --world build/my-new-network-test/world \
  --players build/classic-directional-client-qualified-v1-20261103/art/player \
  --output build/my-new-network-session
```

World export retains authored gameplay data, but this launcher currently exercises
movement with local development tickets; it does not activate account/quest/combat
profiles. Per-pawn script paths are relative to the exported scenario, so later
edits to the repository example cannot silently change this snapshot. Runtime
preflight and authoritative spawn admission still run on every launch. Native
Linux two-client scripted movement, impairment and cleanup pass; see STATUS.md.
The in-editor pawn UI and managed Start/Stop controls remain the next slice.

P5-17 / P1-06 test-player authoring foundation, 12 November 2026:
Zone schema 12 adds optional `test_players` (at most two). Entries contain `id`,
`name`, `client_id` (unique 1/2), `enabled` (defaults true), global f64 feet
`position`, optional stable catalogue `controller` ID (null/absent means manual),
and `network` (defaults to no impairment). Network fields match the local relay:
`delay_ms` and `jitter_ms` 0..5000, `loss_percent` 0..100, u32 `seed`, and at most
16 ordered nonoverlapping `[start_ms,duration_ms]` outages within one hour, each
lasting at most 60 seconds. Delay is one-way; twice it is only the added base RTT.

`PutTestPlayer`/`DeleteTestPlayer`, shared rename/translate/duplicate and terrain
grounding commands are transactional and undoable. Duplication assigns the free
client slot and preserves settings; it refuses when both slots are occupied,
including disabled pawns. IDs share the authoring namespace. Physical spawn
support/capsule clearance is checked by the server at launch; document bounds
alone do not establish a valid gameplay spawn. Test pawns do not become NPCs or
production spawns. Current editor visibility/inspector/launch integration and
controller-ID resolution are still pending. Explicit Save remains the only disk
write; all 70 zone tests and strict workspace checks pass. See STATUS.md.

# Scenery asset browser — 12 November 2026

Open **Assets** in the top bar (or **View → Asset browser**). This lower dock
lists available scenery separately from the left-hand explorer's placed objects.
Use the category dropdown and case-insensitive, multi-word search over names,
categories and IDs. Select a card, then **Place at spawn**; the placed model uses
existing XYZ/yaw/scale controls and undo/redo. Browsing alone does not edit the zone.
Filtering clears an invisible selection, and stale placement requests refuse.
The card identifies solid scenery versus decoration. Selecting a card renders the
actual model in a separate preview at the right of the dock. The preview fits its
verified bounds and uses its original materials, with separate presentation lighting.
Closing the browser, filtering out the selection or entering playtest/particle
workspace releases that instance and disables its camera. Preview errors do not
change placement/history. Per-card thumbnails and drag-to-place remain follow-ups.
The flat Scenery menu is replaced by this browser.

Drag the browser's top divider to resize it, or Close to restore viewport space.
The left explorer/properties divider remains independent. F5 temporarily hides
the browser, and returning to editing restores it. Search owns editor shortcut
keys; Enter/Escape releases search focus without sending that press to the world.
Mouse gestures remain owned through release, including outside the panel. Category
popups use bounded scrolling and native placement above/below their anchor.

Native input reproduction (uses only its owned test editor window and a source copy):

```sh
python3 tools/qualify_asset_browser.py \
  --zone build/starter-valley-contact-v3.zone.json \
  --library build/starter-valley-scenery-v2/catalog.json \
  --output build/asset-browser-native-new
```

# Editor-authored village contact — 12 November 2026

Select an NPC marker and edit **Gameplay reference → Catalog ID**. A blank ID
leaves an annotation. With a gameplay template, one NPC ID matching its dialogue
profile becomes the runtime contact at the marker's exact requested XYZ feet.
Unsupported/intersecting positions (including the world envelope), mismatched IDs
and multiple bound NPCs refuse export. Valid changes use existing undo/redo and
explicit Save; they do not automatically write the zone file.

`content/zones/starter-valley-contact.commands.jsonl` adds a village guide and
western-grove quest area to a copy of the dressed valley. Use the existing
`content/development-gameplay-classic-quest-areas.json` template. The prepared
`build/starter-valley-contact-v3.zone.json` and exported
`build/starter-valley-contact-world-v3` preserve the user's saved v2 scenery edits;
the source v2 file is untouched. Navigation was rebaked separately into
`build/starter-valley-contact-navigation-v3` for the new world identity.

For the connected valley launch, select the v3 world/navigation and add:

```sh
--dialogue --guide-position authored \
--dialogue-catalog build/starter-valley-contact-world-v3/gameplay/quests/development-journal-dialogue.json
```

Both client and server resolve the verified contact instead of a separately typed
position. Actual encrypted two-peer dialogue/walkaway and CLI identity checks pass.
The exported quest catalogs/area are validated content; this test does not enable
account-backed quest progress. Multiple NPCs, quest pickups and full authored
account gameplay remain follow-ups. F5 remains the movement/animation sandbox.

# Height-aware village contact — 12 November 2026

The connected village can include one informational guide using explicit global
feet coordinates. For example, add `--dialogue --guide-position 8194,0.03,8192`
to the connected valley command below. Two-coordinate `x,z` fixture positions
remain supported; use three coordinates to choose a floor beneath a roof.
Unsupported positions refuse before the server opens its endpoint. This is a
connected-client contact, not an editor F5 NPC or a completed quest binding.
Editor NPC markers and multiple quest contacts still need authoring/export work.

# Synty player in editor playtest — 12 November 2026

The launcher automatically uses the local qualified Synty base character when
available. Open the village, then press **F5**:

```sh
python3 tools/edit_zone.py --no-build --zone build/starter-valley-village-v2.zone.json
```

The **Player** menu switches between the character and capsule, automatic movement
and individual animation previews. Walking, running in eight directions and
jumping follow the existing Classic movement solver; sword stance/swing and death
can be previewed without changing the zone or issuing a gameplay action. Select
**Automatic movement** to return to movement-driven animation. Loading failure
retains the capsule. No animation preview saves the zone or changes collision.

Use `--player-assets DIRECTORY` to select a local directory containing
`character.glb`, or launcher option `--no-player-assets` for the capsule alone.
The prepared 22-clip character includes sword animations but **no SpellCast clip**;
that menu entry remains disabled. Loading art does not equip a sword or grant
abilities. Gameplay combat remains in the connected game client; editor F5 is
the offline movement/animation sandbox. Proprietary art stays outside Git.

P5-17 / P2-09 / P3-08 prerequisites: the qualified directional Synty conversion,
standard Bevy glTF/animation and shared Classic editor controls. Acceptance covers
menu activation during F5, availability gates, unchanged document/feet, directional
selection, animated skin rendering, capsule fallback and leaving playtest. Native
desktop interaction and Windows qualification remain separate from offscreen tests.

# Expanded terrain and model controls — 12 November 2026

Imported scenery uses the existing XYZ move arrows, XZ body drag, yaw ring and
uniform scale handle. Release applies one undoable edit; Escape cancels the live
preview. Numeric fields, duplicate/delete and explicit Save share the same data.
F5 playtest uses the game's Classic binding sampler and camera orbit controls.

Add → Starter valley terrain starts a blank zone with a 512 m editable valley.
For the prepared origin, use `content/zones/starter-valley.base.json` with
`content/zones/starter-valley.commands.jsonl`. This terrain blockout still needs
village dressing and working quest/NPC/patrol bindings. Old examples stay valid.

Current terrain format: schema 11 adds optional `span_chunks` (default 1), up to
4 logical 128 m chunks per side and 129 samples per axis. A 512 m patch uses 4 m
spacing; selecting 2 m on patches wider than 256 m refuses atomically. Locked
outer height/paint edges remain. Document and command input limits are now 2 MiB;
older qualification notes below accurately describe their former 1 MiB cap.
World/asset export budgets and runtime navigation coverage are unchanged.

# Editor export update — 11 November 2026

## Starter-zone implementation target — 12 November 2026

| Capability | Current boundary | Next acceptance |
| --- | --- | --- |
| Terrain and boundary geometry | Blank-zone starter basin preset, sculpt/paint, Synty scenery and static collision export; local/far offline walk/jump routes use one opening | Connected Classic movement/navigation and native appearance qualification |
| Quest areas | Explicit bounds/IDs/objective links, Explorer/inline editing, undo/redo and validated combined gameplay export | Connected area credit/minimap acceptance for the authored starter scene |
| NPCs and quest assignment | Editor NPC markers are unresolved annotations; runtime dialogue uses a guide binding | Resolve contact/quest catalogs, place supported contacts and validate giver/turn-in assignments; expand runtime placements where required |
| Quest pickups | Inventory/quest-item and corpse-loot runtime paths exist | Audit and implement placed world-item pickup authority, then authoring and persistence; do not equate corpse loot with placed pickups |
| Playable publication | Geometry, particles, scenery and validated gameplay bundle export; editor F5 is an offline sandbox | Export a coherent gameplay bundle and launch authoritative playtest with quests, minimap and persistence |

Execute through P5-17 / P3-15, reusing P3-11/12 and targeting the bounded P3-16
starter-zone milestone. Each slice needs positive and rejection-path tests; native
UI/GPU acceptance is deferred while the user reserves mouse/display. Existing
primitive/terrain/particle authoring must remain usable throughout.

**File → Export world…** publishes the current zone snapshot, including saved
particle effects, to a new folder. Enter the destination, content ID and positive
revision; export runs in the background and does not save the zone source.
Unsaved particle edits and existing output folders are rejected. Native
mouse/keyboard qualification, including Tab navigation and a subsequent
connected two-client playback, is recorded in
`build/editor-export-dialog-native-20261111-v6/qualification.json` and
`build/editor-export-world-native-20261111-v2/verification.json` (RX 7900 XTX,
RADV NAVI31, Mesa 26.2.6 Vulkan; zero GPU diagnostics).

P5-17 same-window Sprinkles authoring accepted on Linux, 10 November 2026:
File → Particles workspace opens the actual upstream UI inside Theia; Zone in
its top bar returns. Original editing/playback/save, project/pause retention and
exclusive input/camera/render-layer ownership pass native local/far checks with
exact zone/source/revision/history preservation and zero Vulkan diagnostics on
RX 7900 XTX / RADV NAVI31. All 177 focused tests, three doctests, fmt and strict
workspace/all-target Clippy pass. The original standalone host remains qualified.
The direct editor-integration slice is complete at this scope. Asset placement/
world export/client VFX, full P2-13, Windows and MMO budgets remain separate.
See [SPRINKLES_INTEGRATION](SPRINKLES_INTEGRATION.md) for use and exact evidence.

P5-17 same-window preparation, 10 November 2026: all original Sprinkles Update
systems share an activity gate, defaulting active for the qualified companion.
An actual save-shortcut regression proves inactive Sprinkles cannot save its
project on Ctrl S. Four editor tests, fmt and strict workspace Clippy pass, as
does the full native widget-edit/save flow with zero Vulkan diagnostics. UI,
focus, camera/layer isolation and Theia input suspension remain to be connected;
this does not yet claim a same-window workspace. See SPRINKLES_INTEGRATION.

P5-17 original Sprinkles widgets, 10 November 2026: native curve/gradient/colour
popovers render and commit actual values through the reused UI, preserving exact
expected effect data and the source file until Save. Fix the range handler's
incorrect hierarchy lookup; add finite/inverted-range rejection and a regression
for one-time commits and clamping. Native GPU validation remains clean on
RX 7900 XTX / RADV NAVI31. All 16 focused runtime/editor tests and three doctests pass, as do formatting
and strict workspace Clippy; see SPRINKLES_INTEGRATION for evidence.
Same-window hosting remains next; no gameplay VFX/Windows/MMO capacity claim.

P5-17 direct Sprinkles source integration, 10 November 2026: the original
runtime and editor are imported at the reviewed pin and ported to exact Bevy
0.20.0. File → Particle editor opens the retained UI/preview/RON authoring tool
in one companion window. Native editing/save and host launch/return checks pass
on RX 7900 XTX / RADV NAVI31 / Vulkan with zero diagnostics and exact zone/source/
history preservation. All 172 focused tests plus three runtime doctests, fmt and
denied-warning workspace/all-target Clippy pass. Source and asset licenses/hashes
are retained. This completes the first direct-reuse slice; same-window hosting,
asset placement/client integration, Windows and full P2-13 remain pending.
See [SPRINKLES_INTEGRATION.md](SPRINKLES_INTEGRATION.md) for exact evidence.

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

Basic edits include `transform` (`id`, full `pose`), `translate` (`id`, XYZ metre
`offset` for an object/plant/marker/whole guide), `delete` (`id`) and `set_spawn`
(`position`). Startup command history is retained for visual undo/redo. Revision
starts at zero for each freshly opened document and is not persisted identity.

Palette buttons / 1–3 choose placement; click the ground to place. Escape returns
selection mode. Click an object to select; drag it on a second press to move X/Z.
Arrows move X/Z, PageUp/Down move Y, R/Shift-R rotate ±15°, +/- scale uniformly.
G toggles the selected move/placement grid; H grounds the selection; Ctrl-Z/Y
undo/redo. Ctrl-S saves, Ctrl-Shift-S opens Save As, Ctrl-O opens the zone browser
and Ctrl-Shift-O reloads the active file. WASD/MMB pan, RMB orbits and the wheel zooms. F5 enters/leaves playtest; W/S forward/back, A/D turn,
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

## Primitive world-axis move handles — 9 November 2026

P5-17 / P3-15 / P1-10 follow-up. Prerequisites are retained primitive picking,
transactional Drag/Transform, checked local/global coordinates and standard Bevy
PBR meshes. Two resident cylinder/cone meshes, four unlit materials and six
noncolliding, nonshadowing entities render red X, green Y and blue Z arrows around
the selected primitive. Handles use normal world depth/picking rather than an
overlay renderer. Size follows bounded camera distance and object dimensions;
near-parallel view rays refuse a poorly constrained gesture. Rotate/scale and
non-primitive handles remain subsequent work.

A primary drag solves the closest view-ray/world-axis parameter in the bounded
local frame, adding its delta only to the original global f64 coordinate on that
axis. Other axes retain their exact original values. With snapping, only the
active global coordinate rounds to 1 m. A stationary click preserves fractional
values/history and reports Position unchanged. Every changed preview passes the
existing Transform command in a disposable Editor; invalid candidates retain the
last valid preview and cannot reach rendering as unchecked positions. The live
source/revision/history remains unchanged while held. Release in the viewport
commits one revision-checked Transform entry; Escape, focus loss or release outside
the viewport cancels using the existing drag boundary. The active axis renders
white. The existing selected-body XZ-plane drag remains available.

A standard Bevy Move XYZ checkbox stores editor-local visibility; modal, pending
and playtest guards match the snap control. Handles hide for other workspaces,
armed placement, modal forms and playtest. They are neither exported content nor
physics colliders. The optional qualification report exposes actual head entities,
mesh identities, projected centres and test targets plus the active preview.
No source schema, plugin pin, authority or networking change.

Acceptance covers closest-axis mathematics, near-parallel/behind-ray refusal,
f64 preservation and active-axis-only snapping; actual native local/far X/Y/Z
mouse previews and exact shared headless Transform equality; unchanged click,
one-entry undo/redo, Escape/panel cancellation, checkbox and modal/play isolation,
save, standard rendering and GPU validation at actual 3440x1440.

All acceptance checks above passed on the final build: 81 focused Rust cases
(51 domain / 30 editor), formatting, strict workspace/all-target Clippy and actual
3440x1440 native local/far runs on RX 7900 XTX (RADV NAVI31), Vulkan device 29772.
Khronos validation is active with zero GPU/resize diagnostics. Six real axis edits
match complete headless Transform documents. Stationary clicks preserve fractional
coordinates/revision and report Position unchanged. Both cancel paths, the standard
visibility checkbox, modal/play isolation, grouped undo/redo and exact saves pass.
The local coloured-handle view was visually inspected. Windows, rotate/scale and
non-primitive handles, and editor performance at maximum document budgets remain
unqualified; no new gameplay multiplayer claim is made.

Evidence: ignored `build/zone-move-handles-native-3440-v2/qualification.json`,
`*.handles.png`, `*.axis-*-preview.png` and `.before.json` / `.commands.jsonl` /
`.expected.json` pairs; `zone-move-handles-tests-v3.log`,
`zone-move-handles-clippy-v2.log` and `zone-move-handles-build-v2.log`.
Editor SHA-256: `184018297dd68b19c5cea3fb4a5ebdfb7adef3165d02b5b172602f878251c212`.

## Primitive yaw rotation ring — 9 November 2026

P5-17 / P3-15 / P1-10 continuation. Prerequisites are the validated move-handle
preview/commit path, standard Bevy Torus geometry and the existing Pose yaw field.
Move/Rotate Y toolbar buttons now select an editor-local tool and disarm pending
primitive placement while retaining selection. Selected-body XZ dragging remains
exclusive to Move; body clicks in Rotate only select. Handles toggles
visibility for either tool; the snap label identifies 1 m / 15 degree steps.
Rotate Y shows one green ring at the selected primitive centre, with normal world
depth, shared unlit/active-white materials and one resident torus mesh/entity.
This is yaw rotation, matching the existing source pose; pitch/roll, scale and
non-primitive transform handles remain future work.

Horizontal-plane ray intersections produce angles around world Y. Incremental
wrapped deltas accumulate across the ±180-degree seam and full turns. A stationary
click preserves fractional yaw; snapping rounds changed yaw to 15 degrees, while
unsnapped drags retain the calculated value. Position and XYZ size are copied
unchanged from the original f64 pose. Parallel/centre rays refuse a poorly
constrained gesture, with an orbit-view hint when beginning fails. Shared candidate
Transform validation checks each changed preview and the final revision-checked
commit. The existing cancel, UI/pending/modal/playtest isolation and one-entry
undo/redo boundaries apply. The ring does not enter source/export/collision, and
neither tool choice nor snapping changes source/history. No engine/plugin pin,
source-schema, network or authority change.

Acceptance covers yaw seam/full-turn mathematics, stationary fractional values,
unchanged position/size, parallel/centre refusal, snapped and precise native yaw
edits for all three primitives at local/far origins, exact headless Transform
equivalence, cancel, tool/modal isolation, undo/redo and save. Retained move-handle
interaction and standard native GPU checks run in the same fixture.

Final acceptance passes: 82 focused Rust cases (51 domain / 31 editor), formatting,
strict workspace/all-target Clippy and actual 3440x1440 native local/far runs on
RX 7900 XTX (RADV NAVI31), Vulkan device 29772. Khronos validation is active with
zero GPU/resize diagnostics. Twelve new yaw edits plus six retained axis edits
match complete headless documents. Snapped/unsnapped values, stationary fractional
yaw, exact other fields, cancel, one-entry undo/redo, save, placement disarming,
body-selection isolation and modal/tool switching pass. The local active ring
view was visually inspected. Unsnapped 45-degree nominal targets allow one degree
for integer screen-pixel rounding; document equality remains exact. Windows,
non-primitive handles, scale and maximum-document editing performance remain
separate qualification; this UI slice adds no multiplayer claim.

Evidence: ignored `build/zone-rotation-handles-native-3440-v3/qualification.json`,
`*.rotate-*-preview.png` and matching `.before.json` / `.commands.jsonl` /
`.expected.json` pairs; `zone-rotation-handles-tests-v3.log`,
`zone-rotation-handles-clippy-v3.log` and `zone-rotation-handles-build-v4.log`.
Editor SHA-256: `4643118e4a96023f71173e5f0844151b3696de23c7ce46941a18e4164b007679`.

## Primitive uniform scale handle — 9 November 2026

P5-17 / P3-15 / P1-10 continuation using the retained standard transform gesture,
whole-candidate Transform previews and validation. Scale uniformly selects an
editor-local tool with a pale-blue cube/stem above the primitive; drag up/down
along world Y to multiply all dimensions by one factor. One resident Cuboid mesh,
one shared standard unlit material and two noncolliding/nonshadowing entities
reuse existing picking. The stem and cube share one initial head-distance reference.
The active handle tracks the accepted relative scale and has a bounded 2048 m
visual extension; its shaft retains positive render scale when shrinking.

Snapping uses 10% steps relative to the gesture's initial dimensions; unsnapped
values retain the calculated factor. Position and yaw are copied exactly from
the original f64 pose, as are dimension ratios. This respects existing sphere
uniform-size and cylinder X/Z constraints while allowing proportional box edits.
Stationary clicks preserve values/history and report Size unchanged. Negative,
too-small/large or out-of-zone candidates fail shared validation before rendering,
retaining the last valid preview. A held gesture never changes the live document;
release commits one revision-checked Transform entry. Escape, panel release,
focus loss, modal/playtest and tool-selection boundaries remain shared. The scale
tool disarms placement; body clicks only select, and legacy body XZ dragging remains
Move-only. Numeric box fields still allow existing validated nonuniform dimensions.

The snap control now identifies move / rotate / scale steps. Tool/visibility/snap
settings do not create source/history entries. Scale widgets do not enter export
or physics, and source schema/engine/plugin pins/network/gameplay are unchanged.
Non-primitive handles, individual-axis box scale and broader application views/
docking remain follow-ups.

Acceptance covers proportional box/sphere/cylinder scaling, exact position/yaw,
stationary values, snapped/precise factors, invalid scale previews, whole-command
equality, cancel, modal isolation, undo/redo and save at local/far origins. The
same actual 3440x1440 native fixture also retains all move/yaw checks.

Final acceptance passes: 83 focused Rust cases (51 domain / 32 editor), formatting,
strict workspace/all-target Clippy and actual 3440x1440 native local/far runs on
RX 7900 XTX (RADV NAVI31), Vulkan device 29772, with active Khronos validation and
zero GPU/resize diagnostics. Twelve new scale edits plus six retained move and
twelve yaw edits match complete headless documents. Proportions, unchanged pose
fields, snapped/precise values, invalid negative-scale preview refusal, stationary
click, cancellation, one-entry undo/redo, modal isolation and exact save pass.
The local active scale-cube view was visually inspected. Windows, non-primitive
handles, independent-axis scale and maximum-document editing performance remain
unqualified; this adds no gameplay multiplayer claim.

Evidence: ignored `build/zone-scale-handles-native-3440-v1/qualification.json`,
`*.scale-*-preview.png`, `*.invalid-scale.png` and matching `.before.json` /
`.commands.jsonl` / `.expected.json` pairs; `zone-scale-handles-tests-v2.log`,
`zone-scale-handles-clippy.log` and `zone-scale-handles-build.log`.
Editor SHA-256: `687798a4124d6e84927f1091cb5b9705fda8497906f6c0aa9ea77fdc198877f9`.

## Canonical authoring views — 9 November 2026

P5-17 / P3-15 / P1-10 follow-up using standard Bevy Projection,
OrthographicProjection and the existing editor View/picking/frame pipeline.
Perspective, Top, Front and Right buttons share workspace pending/modal/play
refusal. Canonical views use exact camera directions/up axes and orthographic
fixed-vertical span. Wheel zoom changes the span, while a 1024 m camera stand-off
and 4096 m far plane keep the camera outside the authored zone. Ray-gesture limits
also cover the bounded 4096 m local frame. Perspective retains the prior orbit.
Handles derive orthographic size from view span rather than physical stand-off.

WASD pans along each canonical view's screen axes (Top: X/-Z, Front: X/Y,
Right: -Z/Y); existing perspective XZ panning remains. RMB motion returns to
perspective, and Perspective resets the usual isometric orientation. Framing
works with either projection. Pointer processing precedes camera controls so an
active transform freezes keyboard pan, orbit and wheel zoom from its first frame.
Projection/target/tool settings do not change source/revision/history. Offline
Avian playtest always uses the retained perspective camera and restores the stored
editor projection on return. Normal gameplay controls/facing are unchanged.

All acceptance checks pass: 85 focused Rust cases (51 domain / 34 editor),
formatting, strict workspace/all-target Clippy and actual 3440x1440 native local
4 m / far 2 m runs. These use RX 7900 XTX (RADV NAVI31), Vulkan device 29772, active
Khronos validation and zero GPU/resize diagnostics, with no input retries. Exact
axis/up/projection and zoom/restore math, all view controls, top sculpt and front
Y movement match the shared headless commands (four new complete document pairs).
The held transform also refuses real W/RMB/wheel camera motion. Pan/zoom, fixed
camera position during ortho zoom, reachable handles, orbit-to-perspective,
modal/draft/play isolation, return projection and exact save pass. Retained
workspace/paint/primitive editing also passes. The front active-move screenshot
was visually inspected. Windows, adaptive vertical grids, arbitrary projection
controls and maximum-budget performance remain unqualified; no multiplayer claim.

Evidence: ignored `build/zone-camera-views-native-3440-v3/qualification.json`,
`*.top-sculpt.png`, `*.front-move.png`, `*.right-view.png` and matching
`.before.json` / `.commands.jsonl` / `.expected.json` pairs;
`zone-camera-views-tests-v3.log`, `zone-camera-views-clippy-v3.log` and
`zone-camera-views-build-v3.log`. Editor SHA-256: `2af9a103d87e0540be26a91df325218954c7ed837d43bb7bda59a75de637f497`.

## Validated body-plane drag — 9 November 2026

P5-17 / P3-15 / P1-10 quality follow-up to transform gestures. The selected-body
XZ drag now uses the same Gesture preview/validation path as axis, yaw and scale,
instead of directly writing an unchecked preview. Its initial local ray-plane hit
anchors a delta added to the original f64 X/Z values; Y/yaw/size are copied exactly.
A stationary click skips grid snapping and preserves all original values/history.
Changed plane drags retain existing optional absolute 1 m XZ snapping.

All changed previews run the existing Transform command in a disposable Editor.
Invalid positions retain the last valid preview and typed status, leaving the
live document/revision/history untouched. Release commits once if changed;
unchanged/invalid-only release skips commands, preserving pending redo. The shared
plane intersection also serves yaw gestures, with parallel/forward-ray bounds.
Body dragging remains Move-only, and existing camera freeze, modal/play/pending
isolation, cancellation and final revision checks remain. Source schema, export,
engine/plugin pins and gameplay are unchanged.

Acceptance covers stationary snapped/precise f64 fields, relative plane deltas,
exact unaffected fields, native body click/drag and shared command equality,
one-entry undo/redo, out-of-zone preview refusal with pending redo, save and
retained move/yaw/scale checks at local/far origins and actual 3440x1440.

Final acceptance passes: 86 focused Rust cases (51 domain / 35 editor), formatting,
strict workspace/all-target Clippy and native actual 3440x1440 local/far runs on
RX 7900 XTX (RADV NAVI31), Vulkan device 29772, with active Khronos validation and
zero GPU/resize diagnostics. Two new body-plane edits and thirty retained
move/yaw/scale edits match complete headless documents. Fractional stationary
clicks, preserved fields, source-free previews, one-entry undo/redo, bounded-zone
refusal with pending redo and exact save pass. The local refused-preview capture
was visually inspected. Windows and maximum-budget interactive performance remain
unqualified; no gameplay multiplayer change is claimed.

Evidence: ignored `build/zone-body-drag-native-3440-v1/qualification.json`,
`*.body-plane-preview.png`, `*.body-out-of-bounds.png`, `*.body.before.json` /
`*.body.commands.jsonl` / `*.body.expected.json`; `zone-body-drag-tests-v2.log`,
`zone-body-drag-clippy-v2.log` and `zone-body-drag-build.log`.
Editor SHA-256: `c61007ad9e3b43c8228508596918cd8a94d354ab9d3895af095be2a7251a1fc1`.

## Zone file browser — 9 November 2026

P5-17 / P3-15 / P1-10 application workflow follow-up. Prerequisites are the
existing validated zone load/save and Editor replacement/history. Standard Bevy
buttons, TextInput and a modal scrim provide Open and Save As with folder
navigation, sorted directory/JSON rows, twelve rows per page and a literal path
field. Ctrl+O opens; Ctrl+Shift+S saves as; Ctrl+Shift+O retains active-file Reload.
A filename without an extension gains .zone.json on Save As. Other non-JSON
extensions are refused. Existing destinations require a second explicit Apply;
typing/navigation resets this review, and Cancel does not write.

Open loads and validates before one undoable Editor replacement, reconciles IDs
and frames the loaded origin. Undo restores the previous document while the
opened path remains active; dirty state compares content with the saved snapshot.
Save As changes path/saved metadata only after a successful write and preserves
revision, undo and redo. Failed operations preserve the document, active path and
history. Modal input blocks world/camera/agent edits; playtest and pending
gestures/drafts refuse browser shortcuts. IME composition blocks Apply. Directory
scanning is synchronous and bounded to 4096 entries, with a visible truncation
notice; non-UTF8 names are omitted. No recursive asset index, shell expansion,
folder creation or native OS picker is claimed.

Acceptance passes: 88 focused Rust tests (51 domain / 37 editor), formatting,
strict workspace/all-target Clippy and actual 3440x1440 native local/far browser
walkthroughs on AMD RX 7900 XTX (RADV NAVI31), Vulkan device 29772, with active
Khronos validation and zero GPU/resize diagnostics. Checks cover folder/paging,
filename correction, overwrite review/cancel/confirmation, malformed load,
cross-origin Open/undo/redo, fresh placement IDs, Save As preserving redo, failed
write metadata preservation and busy shortcuts without accidental Save. The
retained native save-state suite also passes fresh/reopen/failed-write cases.
The open browser capture was visually inspected. Windows native file replacement
and maximum-budget directory performance remain unqualified. No multiplayer
authority, gameplay, schema or dependency changes.

Evidence: ignored build/zone-file-picker-native-3440-v2/qualification.json and
local/open-ready.png; build/zone-file-picker-save-state-native-3440-v1/qualification.json;
zone-file-picker-tests-v2.log, zone-file-picker-clippy-v2.log and
zone-file-picker-build-v2.log. Editor SHA-256:
`b100d0279a4cc590bafba89f4d2edd098c90492315632bcd36cc0a310a36c84c`.

## View-plane reference grids — 9 November 2026

P5-17 / P3-15 / P1-10 follow-up; prerequisites are canonical authoring views and
standard resident Bevy mesh presentation. Front uses local XY references and
Right uses ZY; Top/Perspective retain the surface-following sample grid when a
terrain patch exists, otherwise they use XZ references. A standard Grid checkbox
shows/hides both kinds, displays spacing in metres and distinguishes surface
samples. Movement snapping remains 1 m independently of visual spacing.

Adaptive spacing chooses 1/2/5 decade steps from the view span. A fixed 130
resident unlit cuboids follow the pan target on zone-origin-aligned multiples,
with stronger fifth lines and coloured X/Y/Z axes. Canonical grids remain on the
zone-origin plane, depth tested against geometry, rather than following object
depth. Width follows zoom; no per-frame mesh allocation or custom renderer. The
references ignore picking, receive/cast no shadows and never enter authored
source, collision or export. Playtest hides them and restores the setting on
return. Existing modal/pending-draft/play guards prevent changing the toggle
while busy; grid and projection choices never create source history.

Acceptance: 90 focused Rust tests (51 domain / 39 editor), formatting and strict
workspace/all-target Clippy pass. Native terrain view/transform walkthroughs at
actual 3440x1440 pass at local and far origins, including adaptive zoom/plane
alignment, surface/reference visibility, toggle/history preservation, modal and
pending-draft refusal, playtest hiding/restoration and retained exact brush and
move-command oracles. The Front reference capture was visually inspected.
Separate no-terrain native walkthroughs also pass all four view planes, toggles,
pan and playtest hiding/restoration at both origins; the Perspective capture was
visually inspected. All four runs use AMD RX 7900 XTX (RADV NAVI31), Vulkan
device 29772, active Khronos validation and zero GPU/resize diagnostics. Windows,
user-selectable grid origins and arbitrary clipping/slice planes remain deferred.
Engine pins, source schema, gameplay and multiplayer authority are unchanged.

Evidence: ignored build/zone-reference-grid-native-3440-v1/qualification.json,
coarse.front-grid.png, build/zone-reference-grid-empty-native-3440-v1/qualification.json,
local/perspective.png, zone-reference-grid-tests-v2.log,
zone-reference-grid-clippy.log and zone-reference-grid-build.log. Editor SHA-256:
`f4c84fd2bfc132dc6b5456544f4925e7c541ad49280ed3c8d9364f253562a28e`.

## Active document context — 9 November 2026

P5-17 / P3-15 / P1-10 follow-up to multi-file authoring. The footer persistently
shows saved/unsaved state and the active save path above the transient status
message. The window title shows zone name, filename and an asterisk when dirty.
This makes Open/Save As destinations visible even after another operation
replaces status text. Undo after Open intentionally keeps the opened path and
compares the restored content with that path's saved baseline; the visible
context reflects both. Paths are presentation-bounded to the final 180 Unicode
characters with a leading ellipsis, control characters replaced, and both footer
lines clipped within the reserved 52-pixel bottom area. Full paths remain intact
for actual operations. Older terrain/vegetation reload hints now use
Ctrl+Shift+O; Ctrl+O opens the browser. No source/history behavior changes.

Acceptance: all 90 focused Rust tests, formatting and strict workspace/all-target
Clippy pass. Native file-context walkthroughs pass at both origins on actual
3440x1440, AMD RX 7900 XTX (RADV NAVI31), Vulkan device 29772, active Khronos
validation and zero GPU/resize diagnostics. Actual title assertions cover Save As,
Open, Undo/Redo and failed writes; retained browser overwrite/paging, cross-origin
replacement, history/ID reconciliation and modal/play/draft isolation also pass.
The local unsaved-file-context capture was visually inspected. Local v2 completed
successfully before its subsequent far window lost focus and cancelled the dialog;
the far case passed independently in v3. No interrupted case is counted as passing.
Windows native execution and very long-path native presentation remain unqualified.

Evidence: ignored build/zone-file-context-qualification.json;
zone-file-context-native-3440-v2/local/unsaved-file-context.png and state.json;
zone-file-context-far-native-3440-v3/qualification.json; zone-file-context-tests.log,
zone-file-context-clippy-v2.log and zone-file-context-build.log. Editor SHA-256:
`7487ad72ca6d873c605562e900bd3180fd198697bf8a313c50b240eefe69f691`.

## Exact spawn properties — 9 November 2026

P5-17 / P3-15 / P1-10 application workflow follow-up, depending on the retained
SetSpawn edit, checked global coordinates and standard docked numeric form.
Edit spawn in the command bar opens three exact f64 world X/Y/Z fields, retaining
existing entry selection. Spawn has no entry ID; the form/report represents it
as absent rather than allocating an object ID. Unused rows are hidden from layout
and focus navigation. Shared field parsing rejects non-finite values, and Apply
runs SetSpawn through the existing whole-candidate validation/revision guard,
creating one undo entry. Unchanged Apply skips commands and preserves redo.
Cancel/Escape/focus loss leave source unchanged, with retained modal input/agent
isolation. Busy playtest, transform, stroke and guide drafts refuse opening.
Existing view-centre and terrain-cursor placement are unchanged. Clearance is
checked on entering the offline Avian playtest, separately from source bounds.

Acceptance: 91 focused Rust tests (51 domain / 40 editor), formatting and strict
workspace/all-target Clippy pass, covering exact local/far values, finite/field
count refusal, candidate bounds, one-entry undo/redo and rejected/no-op edits
preserving redo. Native actual 3440x1440 walkthroughs pass at local/far origins on
AMD RX 7900 XTX (RADV NAVI31), Vulkan device 29772, with active Khronos validation
and zero GPU/resize diagnostics. Four complete native documents match the shared
headless SetSpawn command. Three fields/hidden unused rows, finite/range refusal,
unchanged Apply preserving redo, Apply/Cancel, one-entry undo/redo, exact save and
busy playtest/guide-draft refusal pass. Native checks explicitly refuse a source-
valid spawn 12.5 cm above the floor (outside the retained 10 cm support probe),
then correct Y to 6.25 cm and enter actual Avian playtest at the authored X/Z.
The far-origin field capture was visually inspected. Windows and maximum-budget
interactive performance remain unqualified; gameplay controls/authority, engine
pins and source schema are unchanged.

Evidence: ignored build/zone-spawn-properties-native-3440-v3/qualification.json,
far/spawn-fields.png, local/unsupported-spawn.png and *.commands.jsonl /
*.expected.json complete-document pairs; zone-spawn-properties-tests.log,
zone-spawn-properties-clippy-v2.log and zone-spawn-properties-build.log.
Editor SHA-256:
`a073beaed76c132ce234bf6780e5b87e9c00a8a49616a62bb8dca8e240a5735e`.

## Pointer viewport panning — 9 November 2026

P5-17 / P3-15 / P1-10 follow-up; prerequisites are canonical view projections and
standard Bevy window/input. Middle-mouse drags pan the editor camera in its screen
plane through the target, with exact canonical axes in orthographic views and
projection FOV/span scaling. Logical window cursor deltas and viewport height
keep motion tied to the visible pointer rather than raw-device scaling. The
target remains within the existing bounded local authoring frame. Source/global
positions, selection, revision/history and gameplay controls remain unchanged.

A pan must start inside the viewport. Crossing into a panel, loss of focus,
Escape, primary/secondary-button input, a transform gesture, modal form or
playtest cancels/refuses it; re-entering while held does not reacquire the pan.
RMB orbit, wheel zoom and keyboard pan retain their existing controls. Pan uses
standard Bevy middle-button/window input without cursor capture or custom engine
infrastructure. Existing normal-client Classic controls and offline player
movement are unchanged.

Acceptance: 93 focused Rust tests (51 domain / 42 editor), formatting and strict
workspace/all-target Clippy pass. Tests cover canonical/perspective plane math,
zoom scaling, exact unaffected orthographic coordinate, input rejection and local
bounds. Native actual 3440x1440 local/far runs pass on AMD RX 7900 XTX
(RADV NAVI31), Vulkan device 29772, with active Khronos validation and zero
GPU/resize diagnostics. Sixteen positive drags (four views, two zooms, two origins)
match projected plane math; twelve orthographic drags additionally preserve exact
screen displacement within 0.05 pixels. Panel-origin presses, panel exit/re-entry,
held-transform cancellation, modal and playtest refusal preserve camera/source
as required. Retained grid/keyboard-pan/playtest restore checks pass. The local
Front capture was visually inspected. Windows, physical device input and display
scaling beyond the actual 3440x1440 Linux setup remain unqualified.

Evidence: ignored build/zone-pointer-pan-native-3440-v1/qualification.json,
local/front.png; zone-pointer-pan-tests-v2.log, zone-pointer-pan-clippy-v2.log and
zone-pointer-pan-build.log. Editor SHA-256:
`7311b34ea6ae23f03967d04f20160dffa22e1b2ea893a81a660c42d5157895e7`.

## Placement and move grid presets — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the shared checked zone snap helper,
standard properties sliders, validated move/body gestures and grouped history.
A discrete standard Bevy slider chooses 0.125/0.25/0.5/1/2/4/8 metres, with 1 m
retained as the default. Integer preset selection avoids storing a promoted f32
approximation of the metre step. The checkbox caption and properties label show
actual metres, while the visual reference-grid spacing remains independent.

Object, plant, marker and guide placement share zone-origin-relative quantization.
Primitive world-axis/body previews capture the same grid at gesture start; changed
axes quantize through the existing checked helper. Stationary clicks preserve
all original f64 values and unaffected fields remain exact. This also reconciles
the previous absolute-zero handle rounding with the placement helper at fractional
origins and half-step ties. Ties round away from the zone anchor. Rotation remains
15 degrees and uniform scale remains 10%; numeric properties remain exact and
unsnapped. Shared whole-candidate commands still validate every changed preview
and commit, including zone bounds and primitive constraints.

Movement keys for primitives/plants/markers/guides use the selected relative
increment when snapped, retaining 0.1 m precise nudges otherwise. Flatten-level
key increments use the selected linear step too. Terrain-cursor spawn placement
quantizes X/Z but retains surface height plus its existing 0.1 m clearance offset.
Brush stamp centres/resolution and numeric brush sliders retain their contracts.
Settings are editor-local and do not enter source/export, revision/history or
undo/redo. Existing modal/playtest/gesture/draft guards refuse slider changes
while busy. Explicit structured commands remain exact values, sharing validation
and undo with the resolved visual edits. Engine pins and source schema unchanged.

Acceptance passes: 96 focused Rust cases (51 domain / 45 editor), final focused
custom-gesture and half-step checks, formatting and strict workspace/all-target
Clippy. Native actual 3440x1440 local/far fractional-origin walkthroughs pass
slider endpoints/presets, source/history/redo isolation, a 0.125 m axis drag,
keyboard nudges and primitive/plant/marker/guide placement, with eighteen complete
headless document pairs and exact save. Retained default 1 m axis/body-drag
walkthroughs pass stationary/precise fields, bounds refusal, undo/redo and save.
Terrain paint and vegetation scatter/property walkthroughs also pass at 4 m local
and 2 m far resolution. All six runs use AMD RX 7900 XTX (RADV NAVI31), Vulkan
device 29772, active Khronos validation and zero GPU/resize diagnostics. The paint
properties capture was visually inspected before desktop access was withdrawn.
Windows and physical-device input remain unqualified. Further work is headless
while the desktop is unavailable.

Evidence: ignored build/zone-move-grid-native-3440-v2/qualification.json and
*.commands.jsonl / *.expected.json pairs;
zone-move-grid-default-native-3440-v1/qualification.json;
zone-move-grid-workspace-native-3440-v1/qualification.json and
coarse.paint-properties.png; zone-move-grid-tests-v2.log,
zone-move-grid-focused.log, zone-move-grid-ties.log,
zone-move-grid-clippy-v4.log and zone-move-grid-build-v2.log. Editor SHA-256:
`5489d3d7231216e7286f9a6e2334f6ef5cecb2b7b36be6f4c56771290f23102d`.

## Headless playtest preflight — 9 November 2026

P5-17 / P3-15 / P1-10 quality follow-up while desktop access is unavailable.
Prerequisites are retained CLI candidate validation, checked origin conversion
and the actual offline Playtest/Avian static scene and capsule resolver.
`--check-playtest` implies headless execution before any window/app creation.
It reports the existing source counts/revision/history/validity plus
`playtest.ready`, requested global spawn, resolved global feet or an error.
Successful resolution exits 0; a source-valid but unsupported/blocked spawn prints
structured readiness=false and exits 1 with the existing refusal reason. Malformed
source/commands/arguments retain normal nonzero errors. Normal --headless source
validation remains independent of physical readiness and keeps its old output.

The check is read-only and rejects --output/--export. Window capture/report/mailbox
options remain incompatible with headless operation. Optional --commands are
validated/applied only in memory through the same transaction/history engine;
input files are unchanged. The launcher reopens a default/specified draft or
checks the in-memory editable example as usual, but does not create missing draft
parents in this mode. A build may still produce normal build outputs unless
--no-build is used.

Acceptance passes: all 96 focused Rust tests, seven launcher unit tests, Python
compilation, formatting and strict workspace/all-target Clippy. Fifteen actual
Linux CPU/headless cases run with X11/Wayland connections disabled. Safe,
unsupported and blocked spawns at fractional local/far origins, in-memory
SetSpawn repair with revision/history and unchanged source bytes, raised terrain
at both 4 m/2 m resolutions and local/far origins, and empty/reopened/example
launcher drafts pass. Six forbidden writer/window-mode combinations preserve
existing destinations and create no output/export/report/mailbox paths.

This checks the initial offline collision scene and player spawn, rather than
certifying navigation routes, gameplay or multiplayer capacity. No visual changes
or new native GPU checks were made while the desktop is reserved by the user.
Windows execution remains unqualified. Engine pins and source schema unchanged.

Evidence: ignored build/zone-preflight-headless-v1/qualification.json with per-case
stdout/stderr, zone-preflight-tests.log, zone-preflight-launcher-tests.log,
zone-preflight-clippy.log and zone-preflight-build.log. Editor SHA-256:
`45ef8bf8c7fc0d3137ccc2e691b2e6574754accdb908012ac5f292ed640c820e`.

## Structured entry translation — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the shared authoring ID namespace,
checked f64 source positions and candidate/history engine. New `translate` edits
accept an existing ID and `[x,y,z]` offset in metres. They move primitive poses,
plant/marker anchors or every point of a whole guide while preserving names,
IDs, primitive/kind, yaw, dimensions/scale/radius, guide width and marker
definition keys. Origin, terrain and spawn remain unchanged. Explicit deltas
are unsnapped and never narrow through f32.

```json
{"command":"apply","expected_revision":0,"edits":[{"op":"translate","id":1,"offset":[0.25,0,-0.5]},{"op":"translate","id":1002,"offset":[0.25,0,-0.5]}]}
```

Offsets must be finite and within ±1024 metres per axis, consistent with bounded
copy offsets and the ±512 m authoring area. Existing whole-candidate validation
then checks all source bounds and type constraints. Unknown IDs, malformed
payloads, stale revisions and budget/range failures preserve source/history/redo.
Batch up to the existing 128 edits for one undo entry; ordinary explicit Apply
semantics remain, including zero-delta transactions. The document schema and
existing lower-level edits are unchanged. This is an additive command API, with
no new visual controls or gameplay movement behavior.

Acceptance passes: 100 focused Rust tests (55 domain / 45 editor), formatting,
strict workspace/all-target Clippy and display-disabled Linux process checks at
fractional local/far origins. Mixed translations equal complete documents built
with the existing Transform/PutPath/PutPlant/PutMarker edits. Metadata, all three
primitive shapes, unchanged origin/spawn/terrain, one-entry undo/redo and exact
JSONL output pass. Fourteen failed command-file cases preserve existing output
and source bytes, including a later line failing after an earlier valid edit.
Read-only actual Avian preflight also accepts the translated candidates. An
actual filesystem mailbox test proves new-op delivery, exact retry after shared
Undo without double application, Redo, busy refusal and published source state.

No desktop input, display or GPU checks were used for this command-only slice.
Windows cross-build/native execution remain unqualified; the Windows target and
cross compiler are not installed in this environment. No multiplayer authority
or capacity claim.

Evidence: ignored build/zone-translation-headless-v1/qualification.json, source/
command/output pairs and refusal stderr; zone-translation-tests-v2.log,
zone-translation-clippy-v2.log and zone-translation-build-v2.log. Editor SHA-256:
`e080ab4d8e7125314a9b66e99798db9251d9d76e58c7549f5956e4e4793ab5c5`.

## Piped structured commands — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the shared command/validation/history
engine and read-only playtest preflight. `--commands -` now reads JSONL from stdin
through EOF. Command files retain their existing behavior, including addressing
a literal dash filename using `./-`. The launcher forwards stdin directly.

```sh
python3 tools/edit_zone.py --no-build --zone SOURCE --headless --commands - --output DESTINATION < COMMANDS.jsonl
python3 tools/edit_zone.py --no-build --zone SOURCE --check-playtest --commands - < COMMANDS.jsonl
```

Both readers use the same bounded line parser: at most 1 MiB and 1024 commands,
with the existing 128-edit Apply bound and 64-entry history. UTF-8, JSON, stale
revision, unknown-ID and candidate failures abort preparation. Explicit output
is written only after every command succeeds; read-only preflight changes no
files. Empty EOF is accepted, as is a final command without a newline. As with
the existing file path, budget accounting conservatively charges one newline
byte per parsed line, including a final unterminated line. An open producer must
close stdin before processing can finish. No asynchronous live stream or GUI
control is introduced; the existing mailbox remains the live editor interface.

Acceptance passes: all 100 focused Rust tests, seven launcher unit tests,
formatting, Python compilation and strict workspace/all-target Clippy. Twenty-four
Linux process cases with X11/Wayland connections disabled cover equivalent
complete file/piped documents and revision/history at fractional local/far
origins, one-entry history roundtrips, actual Avian preflight and launcher
stdin inheritance. Unicode, empty EOF, unterminated final commands, exactly
1024 commands, exactly 1 MiB and literal dash filenames pass. Malformed JSON,
invalid UTF-8, late stale revisions, unknown IDs, excessive Apply edits and byte/
command overflow (including CRLF accounting and a truncated valid prefix)
preserve original source and existing output byte-for-byte.

Evidence: ignored build/zone-stdin-headless-v2/qualification.json and per-case
stdout/stderr, zone-stdin-tests.log, zone-stdin-clippy.log, zone-stdin-build.log
and zone-stdin-launcher-tests.log. Editor SHA-256:
`29ea933d7d2d749d61b3d615a26c51d3a12b9014d559ab30d9d2be22ab34ebb1`.
No desktop input, display or native GPU checks were used for this CLI-only slice.
Windows cross-build/native execution and multiplayer remain unqualified. Source
schema, standard Bevy infrastructure and engine pins are unchanged.

## Content-list paging: headless coverage — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the existing application shell,
shared bounded content list, selection reconciliation and draft isolation.
Implementation now reserves 18 rows per page above the 600 px Properties area,
with a bounded two-line no-wrap footer. Previous/Next and row controls use standard
Bevy InteractionDisabled state during previews/forms/playtest; Previous and Next
also disable at their respective boundaries. Scrolling vertically over the
content panel requests one page per nonzero accumulated wheel frame, clamped to
existing pages. Other panels and the viewport do not route wheel input here.
This is paging, rather than continuous row scrolling or a searchable outliner.

Four new headless tests run the actual Bevy setup, scroll, rename-opening and
sync systems with a data-only Window at 3440 x 1440. No Winit, renderer, display
connection or OS input is involved. Shared Session construction keeps the test
and normal editor defaults identical. All 44 targets are visited exactly once
across three pages at fractional local/far origins. Wheel browsing preserves
source, revision, clean state, selection and pending redo. Tests cover panel
edges, missing cursor, focus loss, nonfinite/zero deltas, active guide drafts,
a real rename draft, held transforms, actual Avian playtest hide/restore,
disabled page boundaries, content shrink/clamping and Undo
restoration. Opt-in reports retain observed wheel-event counts and the last
delta, cursor, focus, gating and before/after page to diagnose later native runs.

All 104 focused Rust tests (55 domain / 49 editor), strict workspace/all-target
Clippy, formatting, Python compilation and an editor build pass. Evidence is
ignored build/zone-content-headless-{tests,all-tests,clippy,build}.log.

**Native acceptance remains pending.** Before the user stopped desktop use,
build/zone-content-paging-native-3440-v1 selected the first page’s last object,
but its first Linux wheel-paging wait timed out. The final saved report was
unfocused; this does not establish whether input delivery or focus caused the
failure. The qualifier released input and closed its owned editor. No successful
native wheel or complete local/far/GPU acceptance is claimed. The qualifier now
retains explicit delivery/routing diagnostics without retrying a mutating input.
It must only be run after renewed desktop authorization. No display, mouse or
keyboard input was used while adding the headless tests. Windows execution and
multiplayer capacity remain unqualified; engine pins and source schema unchanged.

Headless layout addendum, 9 November 2026: an additional regression uses Bevy’s
public CPU layout, target propagation and text-measurement systems, following
its own pinned 0.20.0 layout test harness. It loads Bevy’s embedded default font
into an in-memory Assets collection and uses a dummy camera render-target
metadata record, without a window backend, UI/input plugins, clipboard plugin,
renderer or GPU. All three pages’ actual ComputedNode and UiGlobalTransform
bounds fit inside the content area at 3440 x 1440; the long-name footer remains
44 px high and wholly above Properties. Text title height is also measured.
This verifies layout geometry rather than pixels, OS wheel delivery or hardware.
All 105 focused Rust tests (55 domain / 50 editor), formatting and strict
workspace/all-target Clippy pass. Evidence: ignored
build/zone-content-layout-{tests,all-tests,clippy}.log. Native acceptance remains
pending as described above. No desktop input or display use.

## Content-type filter buttons — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the bounded content browser, shared
selection and the CPU layout harness. Six standard Bevy buttons expose All,
Objects, Guides, Markers, Plants and Terrain. All includes the playtest-spawn row;
Objects includes every primitive shape, and Markers includes NPC and quest
annotations. These filter only the browser. World presentation, picking,
source, current selection, save state and undo/redo are unchanged. Filters remain
editor-local and retain their chosen type across document edits/reloads.

Choosing a type starts on its first page and shows matching/total counts. Later
frames retain that page even when the old selected entry lies on another page;
a genuinely changed visible selection still reveals itself. A hidden selection
is identified in the footer, while an empty category gives an explicit empty
state and disables Previous/Next. Selecting a filtered row uses the existing
mode/selection path. Preview/form/playtest/focus guards apply to filter buttons.

Headless acceptance: 106 focused Rust tests (55 domain / 51 editor), formatting,
strict workspace/all-target Clippy and an editor build. A real Bevy PointerPress
observer test covers all categories against mixed source with terrain and spawn,
correct counts, unchanged source/revision/clean state/pending redo/selection,
retained first-page browsing across frames, marker-row selection, rename-modal
refusal and filtered deletion/Undo restoration. PointerPress events are injected
only into the in-memory ECS test app; no OS input is sent. The existing actual
font/CPU layout regression now also checks all six button bounds and all rows/
footer at 3440 x 1440. No new source schema, engine pins or dependency changes.

Evidence: ignored build/zone-content-filter-{tests,all-tests,clippy,build}.log.
The owned native qualifier has corresponding filter/empty-category checks for a
future authorized run. It was not run for this slice. Native UI/input/GPU and
Windows acceptance remain pending; no display, mouse or keyboard input was used.
The earlier wheel-delivery gap remains open and is not claimed fixed by these
CPU checks. No multiplayer authority or capacity claim.

## Content search — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the shared standard Bevy text form,
browser filtering and modal input isolation. Find... opens the retained text
form with the current query selected. Apply commits an editor-local browser
preference; Cancel/Escape/focus loss preserve the previous query. Clear search
removes the query while retaining the type filter. The Find button shows an
active query, with clipped single-line text inside its fixed bounds. Opt-in
reports identify the shared text form as search or rename, retaining existing
text/selection fields for compatibility.

Matching is one case-insensitive Unicode-lowercased substring over the existing
row label: name, kind, #ID and existing guide point count. Outer whitespace is
trimmed; there is no regex or query language. It combines with the type filter.
Queries are bounded to 128 UTF-8 bytes without control characters; invalid Apply
keeps the form open and the previous query/list intact. Empty results remain
explicit. Filter/query changes start at page one; subsequent unchanged frames
retain that page. Query changes, clearing and even an open search form preserve
source, selection, saved-state indicator, revision, history and pending redo.
Real rename/transform/file forms retain their authoring dirty behavior. The
shared modal guard still blocks unrelated editing/agent requests during search.

Acceptance: 107 focused Rust tests (55 domain / 52 editor), formatting, strict
workspace/all-target Clippy, Python compilation and an editor build pass.
A headless actual Bevy observer/form/sync test proves Unicode matching, trimmed
#ID search, type-filter combination, result selection, control/oversize refusal,
exact 128-byte multibyte acceptance, empty results, clearing, Escape/focus-loss
cancellation and unchanged complete source/revision/clean state/redo. The test
also verifies that reopening a real rename still marks pending authoring work.
Input is modified only inside the ECS test app; no OS keyboard/mouse events are
sent. The existing embedded-font CPU layout test covers the new buttons, type
filters, all content pages and footer at 3440 x 1440.

Evidence: ignored build/zone-content-search-{tests,all-tests,clippy,build}.log.
The native qualifier includes a future Find/Apply/select/cancel/clear workflow,
but was not run while the user reserves mouse/display access. Native UI/input/
GPU and Windows acceptance remain pending, including the earlier wheel-delivery
gap. Source schema, renderer, engine pins and shared authoring commands unchanged.
Search is a preference and deliberately creates no document undo entry. No
multiplayer authority or capacity claim.

## Content double-click framing — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the standard Bevy content row events,
validated selection and retained Frame helpers. A primary row press with Bevy’s
consecutive-press count of two or higher selects the entry and calls the same
checked object/plant/marker/terrain framing or whole-guide focus used by the
existing Frame button. Single clicks retain selection-only behavior. The spawn
row retains immediate framing on both single and repeated clicks, even when a
previous terrain/other editing mode remains selected. Footer hints expose the
direct action. No custom double-click timer or input backend is introduced.

Headless acceptance: all 108 focused Rust tests (55 domain / 53 editor), strict
workspace/all-target Clippy, formatting, Python compilation and an editor build.
Actual Bevy PointerPress observer events exercise every row kind at fractional
local/far origins. Exact checked local targets and bounded view distance pass,
including guide bounds, tree proxy centre and zone-origin terrain/spawn targets.
Single-click view preservation, retained canonical view preset, pending-guide
and unfocused-window refusal, complete source/revision/clean-state preservation
and pending redo pass. Events remain inside the ECS app, without OS input or a
window backend. The existing real-font CPU layout regression also passes with
the new footer hints.

Evidence: ignored build/zone-content-frame-{tests,all-tests,clippy,build}.log.
The owned native qualifier now includes a future double-click target/capture
check after its paging/search checks. It was not run while desktop access is
reserved. Native repeated-click delivery, UI/GPU and Windows acceptance remain
pending, including the earlier wheel-delivery gap. No display, mouse or keyboard
input was used. Source schema, engine pins and world presentation are unchanged;
this adds an input route to existing editor camera behavior. No multiplayer claim.

## Transform yaw slider — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the retained exact properties form,
standard Bevy controlled Slider infrastructure and shared candidate Apply.
Object, plant and NPC/quest marker forms expose a yaw track/thumb. It displays
one wrapped turn (-180..180 degrees); exact numeric fields retain the original
full-turn f64 value. Snap uses 15-degree steps; disabling it uses one-degree
steps. Spawn and guide/point forms hide and disable this control. Its keyboard
focus is retained inside the modal form alongside existing field navigation.

Intermediate and final widget notifications update only the pending yaw field,
leaving other field strings exact. Apply continues through existing Transform/
PutPlant/PutMarker candidate validation and one undo entry. Cancel/focus loss
preserve source/history. Unchanged slider values retain the exact original
numeric string; dragging back to the original displayed angle also restores it.
Programmatic slider initialization does not quantize authored yaw. Invalid or
foreign events are ignored, and invalid unrelated fields still refuse Apply.
There is no new live scene preview or implicit commit on slider release; the
retained Apply/Cancel form remains the editing boundary.

Acceptance: all 111 focused Rust tests (55 domain / 56 editor), formatting,
strict workspace/all-target Clippy, Python compilation and an editor build.
Three headless integration regressions use the real Bevy SliderPlugin and
SetSliderValue/ValueChange route plus actual field/finish systems. Object/plant/
marker cases at fractional local/far origins preserve complete source metadata,
untouched f64 fields, pending redo during edits and one-entry Apply/Undo/Redo.
No-op/return-to-origin, invalid event/range/field, foreign source, cancellation,
focus loss and hidden spawn/guide controls pass. A shared test-only CPU harness
now supplies embedded-font, editable-text sizing, target propagation and layout
systems to both browser and properties regressions, without a window backend,
renderer, GPU or clipboard plugin. All exact field, yaw track and Apply/Cancel
bounds fit inside the 300 x 600 Properties region at 3440 x 1440.

Evidence: ignored build/zone-yaw-slider-{tests,integration-tests,all-tests,clippy,
build}.log. Opt-in transform-control reports now include slider value/range.
tools/qualify_zone_yaw.py prepares owned local/far native drag, complete headless
command-oracle, no-op/redo and save checks for a future authorized run. It was
compiled but not run while the user reserves mouse/display access. Native UI/
input/GPU and Windows acceptance remain pending; no desktop input or display
use occurred. Source schema, Bevy/Avian/Renet pins and gameplay are unchanged.

## Bounded property sliders — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the exact plant/marker/guide forms,
shared candidate Apply and the standard Bevy controlled-widget yaw slice.
The properties form now also exposes plant scale (0.25..8), marker authoring
radius (0.25..64 metres) and guide width (0.25..32 metres), with 0.25 steps.
Guide width works in both whole-guide and selected-point forms; moving it changes
only width. These fixed steps are independent of the movement/rotation Snap
preference. Labels identify scale versus metres; exact numeric fields remain
available. Object and spawn forms hide/disable this second slider; objects retain
their existing uniform scale gizmo and exact size fields.

Both tracks share the same internal controlled-widget/form path. Range, step,
field index and label derive from the original target, while the retained
Transform/PutPlant/PutMarker/PutPath commands remain authoritative. Slider
notifications edit only pending fields; Apply validates the complete candidate
and makes one undo entry. Unchanged/returned values preserve their original f64
strings; bad typed numbers can be repaired with a valid slider edit. Foreign,
nonfinite, out-of-range and unsupported-target notifications are ignored. Neither
slider release nor initialization commits source. Existing yaw snapping and
modal focus behavior are retained, including support for keyboard slider focus.

Acceptance: 113 focused Rust tests (55 domain / 58 editor), formatting, strict
workspace/all-target Clippy, Python compilation and an editor build pass. Actual
Bevy SliderPlugin event/form tests cover all three values and both guide forms
at fractional local/far origins. Complete documents preserve positions, yaw,
names, kinds, definition keys and every guide point; one-entry Apply/Undo/Redo,
exact no-ops/return-to-original, pending redo, hidden targets, invalid events and
invalid-number recovery pass. Existing yaw regressions also pass with precise
scale/radius/width fixtures. Embedded-font/editable-text CPU layout now checks
all six form targets (object, plant, marker, guide point, spawn, whole guide),
including every visible numeric field and both relevant tracks plus Apply/Cancel
inside the 300 x 600 Properties region at 3440 x 1440.

Evidence: ignored build/zone-value-slider-{yaw-regression-tests,integration-tests,
all-tests,clippy,build}.log; the first intermediate yaw layout run failed because
its test did not yet exclude the newly hidden value control, and is not final
acceptance. The final all-tests run includes the updated visible/hidden geometry
checks. tools/qualify_zone_yaw.py --values prepares a future owned local/far native
scale/radius/whole/point width workflow with full headless command oracles and
no-op/redo/save checks. It was compiled but not run under the user’s desktop
restriction. Native UI/input/GPU and Windows acceptance remain pending; the
earlier wheel-delivery gap remains open. No display, mouse or keyboard input was
used. Source schema, engine pins and gameplay behavior are unchanged.

## Property keyboard routing and yaw endpoints — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the controlled property sliders and
modal field navigation. A real in-memory Bevy InputPlugin/InputFocusPlugin/
InputDispatchPlugin route now qualifies property Tab/Shift-Tab, arrows and
Home/End. Standard CPU asset/mesh/transform/visibility plugins supply inherited
visibility, so hidden rows and controls are excluded by actual TabNavigation.
No Winit, renderer, GPU, clipboard plugin or OS input backend is added. Keyboard
messages are written into the ECS test app rather than sent to a desktop.

The new key-route regression exposed a presentation bug: End set the yaw field
to +180, but wrapped display immediately moved the thumb to -180. Wrapped yaw
now retains positive authored half turns at +180 and negative half turns at
-180, keeping both inclusive endpoints stable. Original full-turn strings and
no-op precision remain intact; source and existing shared Apply semantics are
unchanged. Native physical key delivery is a separate pending check.

Acceptance: all 115 focused Rust tests (55 domain / 60 editor), strict workspace/
all-target Clippy, formatting, Python compilation and an editor build pass. Tab
and Shift-Tab visit exactly the visible fields and relevant sliders, wrap within
the form and skip hidden controls for all six form targets. Actual dispatched
ArrowRight/Home/End messages change the expected pending yaw/scale/radius/whole-
and point-guide width, retain slider focus and keep widget/field endpoints equal.
Complete source, revision and pending redo stay unchanged; Escape cancels.

Evidence: ignored build/zone-properties-keyboard-{tests,all-tests,clippy,build}.log.
The initial failing +180 expectation is retained separately in
build/zone-properties-keyboard-before-fix.log. Final all-tests acceptance includes
the fixed endpoint behavior and all earlier UI/domain regressions. The owned
native qualifier accepts --keyboard (also with --values) for future Home/End and
restored-drag checks. It was compiled but not run under the desktop restriction.
Native UI/input/GPU and Windows acceptance remain pending; the earlier wheel-
delivery gap remains open. No display, mouse or keyboard input was used. Engine
pins, schema, simulation and gameplay-facing rules unchanged.

## Reset transform fields — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the retained exact form, original target
snapshot and controlled sliders. Apply / Reset fields / Cancel now fit in one
button row. Reset restores all field strings from the values captured when the
form opened, returns input focus to its first field and keeps the form open.
The tracks synchronize to those exact originals without quantizing them. It also
repairs invalid pending text and discards pending slider changes. It neither
reloads a document nor refreshes the captured revision; subsequent Apply retains
the existing validation/stale-revision/undo boundary. It does not alter source,
selection, editor settings, history or pending redo. Enter received with Reset
cannot accidentally submit/close the form. Window-focus loss still cancels.

Acceptance: all 115 focused Rust tests (55 domain / 60 editor), formatting,
strict workspace/all-target Clippy, Python compilation and an editor build pass.
Existing actual Bevy form/slider integration flows now exercise the Reset button
observer at local/far origins before reapplying their pending edits. Exact yaw/
scale/radius/width and all untouched numeric strings return, complete source
and pending redo remain, and later Apply still makes one validated undo entry.
The cancellation/refusal flows cover resetting invalid coordinates and all six
target forms, including spawn and whole/point guides. A simultaneous Enter is
suppressed. Existing embedded-font/editable-text CPU layout also covers all
three buttons inside the Properties region at 3440 x 1440.

Evidence: ignored build/zone-property-reset-{regression-tests,all-tests,clippy,
build}.log. The owned native qualifier accepts --reset (with --keyboard and/or
--values) for a future reset/reapply drag workflow. It was compiled but not run
under the user’s desktop restriction. Native UI/input/GPU and Windows acceptance
remain pending, including the earlier wheel-delivery gap. No display, mouse or
keyboard input was used. Schema, shared commands, engine pins and gameplay remain
unchanged. Reset is a form-local action and creates no document undo entry.

## New untitled zones — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are shared validated document replacement,
undo/redo and the retained file browser. The command bar now has New zone. It
creates the default empty zone as one shared undo entry, clears selection and
browser filters/search, and recentres the current view without changing its
preset or grid/editor settings. The identifier allocator remains monotonic.
Prior unsaved source stays undoable; the original file is never written. File
identity is separate from source history: Undo restores prior content into the
untitled document. Save first chooses a filename through Save As, using the
previous directory; only a successful write assigns the new path and saved
snapshot. Cancellation, validation failures and failed writes retain untitled
state. Existing overwrite review remains in force. Reload is disabled until a
path exists. Playtest works before saving; New is disabled during playtest,
forms and previews. Repeating New on an already empty untitled zone is a no-op
that preserves revision and pending redo.

CPU acceptance: all 118 focused Rust tests (55 domain / 63 editor), formatting,
strict workspace/all-target Clippy and the editor build pass. Real Bevy observers
and file controllers cover local/far origins, exact prior unsaved source,
undo/redo, safe first Save, cancellation, failed writes, overwrite confirmation,
repeated-New no-ops and actual Avian playtest construction. Evidence is ignored
build/zone-new-document-{tests,all-tests,clippy,build}.log. Native acceptance is
recorded separately below after execution; Windows execution remains pending.

Native acceptance: tools/qualify_zone_new.py passes both local/far starting
origins at 3440 x 1440. It checks the command header bounds, untitled footer/title,
Reload refusal, New no-op, source undo/redo with retained file identity, Save As
cancellation, failed-write correction, exact saved bytes and clean/dirty state.
The actual Avian playtest runs in the unsaved default zone and New stays blocked
until leaving playtest. App-only screenshots were inspected. Evidence: ignored
build/zone-new-native-3440-v2/{local,far}/ with editor.log, state.json, screenshots
and qualification.json. AMD Radeon RX 7900 XTX (RADV NAVI31), Vulkan device 29772;
VK_LAYER_KHRONOS_validation is confirmed inserted in each log, with zero GPU
diagnostics under the strict checker. Editor SHA-256:
1af86253478939ecbe80e9eae259e4d77f0f9617176387d3e8db4e785f7d0bcb.
The initial v1 run stopped on a qualifier-only missing report key before the
remaining interactions; it is not acceptance evidence. No Windows or multiplayer
claim. Engine pins, schemas and gameplay authority are unchanged.

## Native application controls acceptance — 9 November 2026

P5-17 / P3-15 / P1-10. Following renewed desktop authorization, the current
editor was exercised through owned-window mouse/keyboard input at 3440 x 1440.
Earlier headless-only records retain their original limits; this is new native
evidence. Content acceptance covers all 44 targets across three pages, wholly
bounded rows, last-row selection, disabled boundary buttons, Linux wheel paging,
type filters, Find/Clear search, search cancellation, property-modal isolation
and double-click framing. Source, saved-state indicators and history remain
unchanged by browsing. App-only screenshots were inspected.

The initial renewed run passed local-origin checks but the far-origin run
received no first wheel event, with focus/cursor still present. Creating the
synthetic uinput device before launching Winit removes that startup dependency;
no mutating event is retried. The new driver separates device creation from
scroll delivery and retains focus/owned-pointer checks. The far-origin rerun
passes. This is qualification infrastructure; no production wheel behavior was
changed. Evidence is ignored build/zone-content-native-3440-v2/local/ and
build/zone-content-native-3440-v3/far/, aggregated in
build/zone-content-native-qualification.json. Failed v1/v2 far evidence is
retained and excluded from acceptance.

Native yaw acceptance passes object/plant/marker forms at both local/far
fractional origins: held slider drags leave the full source/history untouched,
Reset restores exact original fields, Home/End keep widget/field endpoints equal,
and Apply makes one shared undo entry. Complete documents equal the structured
headless-command oracle, Undo restores the exact prior source, untouched Apply
preserves original f64 strings and pending redo, and Redo/Save reproduce the
expected document. Evidence: ignored build/zone-yaw-native-3440-v1/ with
--keyboard --reset, logs, reports, pending-property screenshots and manifests.

Native scalar acceptance also passes plant scale, marker radius, whole-guide
width and point-guide width at both local/far fractional origins. The same
held-drag, Reset, Home/End, exact untouched-field, one-Apply, full-document oracle,
undo/no-op/redo and exact-save checks pass. Evidence: ignored
build/zone-values-native-3440-v1/ with --values --keyboard --reset, reports,
logs, screenshots and qualification.json. All accepted content/yaw/scalar cases
used AMD Radeon RX 7900 XTX (RADV NAVI31), Vulkan device 29772, with the validation
layer explicitly confirmed inserted and zero strict GPU diagnostics. Their
common editor SHA-256 is
1af86253478939ecbe80e9eae259e4d77f0f9617176387d3e8db4e785f7d0bcb.
The current 118 focused Rust tests, denied-warning workspace/all-target Clippy,
formatting and build pass; Python qualification files compile. Native Tab routing
is not claimed here (headless real-dispatch coverage remains); native Home/End
and pointer routing are evidenced. Windows execution remains pending. Schema,
engine pins, gameplay authority and networking are unchanged. No push.

## Application document and search shortcuts — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are New zone and the retained Find form.
Ctrl+N invokes the same New helper as the command-bar button, including browser
reset on an actual replacement and preservation on a no-op. Ctrl+F opens Find
without invoking selection framing. Both require the focused editor and respect
forms, playtest and authoring-preview guards. They neither write files nor bypass
shared validation/history. Shifted chords retain their previous behavior.

CPU acceptance: 119 focused Rust tests (55 domain / 64 editor), strict workspace/
all-target Clippy, formatting and build pass. Real Bevy controller flows at
local/far origins check search's unchanged view/source/save state, Ctrl+N while
search is open, New and repeated-New history, playtest/guide-draft refusal and
unfocused input with pending redo. Evidence: ignored
build/zone-application-shortcuts-{tests,all-tests,clippy,build}.log.
The native New qualifier accepts --shortcuts for owned-key delivery and modal/
play guards; its results are recorded below after execution.

Native acceptance passes at both local/far starting origins in ignored
build/zone-shortcuts-native-3440-v1/ (--shortcuts): Ctrl+F opens the search form
without moving the view, Ctrl+N is isolated while it is open, Escape cancels,
and Ctrl+N creates the untitled zone through the shared helper. Both chords are
refused during the running Avian playtest. The subsequent first-save, failed-write
recovery and undo/redo workflow also passes. Each manifest confirms 3440 x 1440,
AMD Radeon RX 7900 XTX (RADV NAVI31), Vulkan device 29772, active validation-layer
insertion and zero strict GPU diagnostics. Editor SHA-256:
af927d2a9df618c898dca0104dc1dd3587519b41dc4748e87e4c9bb7898bfd5b.
Windows execution remains pending. Schema, engine pins, networking and gameplay
controls are unchanged. No push.

## Primitive placement dimensions — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are shared primitive placement, standard
workspace sliders and checked origin-relative surface sampling. Properties now
exposes box X/Y/Z sizes, cylinder diameter/height or sphere diameter while a
placement tool is active. All use 0.25 m steps from 0.25 to 32 m. Sphere dimensions
remain uniform and cylinder X/Z diameters remain equal. Hidden shape dimensions
retain their prior box values when switching tools. These session preferences
do not edit source, dirty state, history or pending redo. They persist across
New/Open within the session and return to 1 m on a fresh launch. Existing objects
retain the exact Transform form and gizmos. The placement summary is concise so
the controls fit after a stamp selects its new object.

Actual placement uses the selected dimensions, origin-relative X/Z snapping and
surface support plus half the chosen height. One stamp is one existing shared
Edit::Place transaction. Invalid dimensions/extent or exhausted IDs refuse a
stamp without changing source/history. Ground ray conversion now reports errors
instead of assuming every hit converts. No schema or engine changes.

CPU acceptance: 123 focused Rust tests (55 domain / 68 editor), formatting,
denied-warning workspace/all-target Clippy and build pass. Actual ValueChange
observers cover shape-dependent visibility, focus/preview refusal, finite bounds,
quarter steps and source/save/history isolation. Complete box/sphere/cylinder
placements at local/far fractional origins equal shared command results; support,
undo/redo and settings changes with pending redo pass. Additional tests cover
raised terrain at local/far origins, invalid/bounded/exhausted-ID refusal and
embedded-font CPU layout for all shape controls inside Properties at 3440 x 1440.
Evidence: ignored build/zone-placement-size-{tests,all-tests,clippy,build}.log.
Native acceptance follows after owned mouse input execution.

Native acceptance passes for all three primitives at local/far fractional
origins, at 3440 x 1440 only. Actual held pointer drags preserve source/history;
shape-specific controls and their bounds are checked. Ground clicks produce the
chosen dimensions, origin-relative snapped X/Z and half-height support. All six
complete documents equal the headless shared-command oracle. Undo/redo retains
the exact stamp while changing defaults preserves pending redo. Escape hides
placement controls, and Save writes the exact final source. App-only screenshots
were inspected. Evidence: ignored build/zone-placement-size-native-3440-v1/
with per-case reports/logs/screenshots/oracles and qualification.json. AMD Radeon
RX 7900 XTX (RADV NAVI31), Vulkan device 29772; active validation-layer insertion
is confirmed in each log with zero strict GPU diagnostics. Editor SHA-256:
06612d754a1e945e198135a4a0f5ddacd19e369a00fa77e621fad8ad5d33f691.
Raised-terrain support has CPU/shared-validation coverage here; native terrain
brush rendering and Windows execution are not newly claimed. Engine pins,
source schema, networking and gameplay authority remain unchanged. No push.

## Workspace slider keyboard ownership — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are standard Bevy focus/input dispatch,
workspace sliders and the existing keyboard authoring controls. A real-dispatch
regression exposed that Right changed the focused movement-grid slider and
nudged the selected box from X=8 to X=10, creating a document edit and losing redo.
Bevy's focused event stops propagation, but the editor also polls ButtonInput.
The editor now reserves Left/Right/Home/End for the focused enabled slider before
selection/authoring controls run. Global commands retain their existing priority.
Workspace sliders now opt into click-to-focus with standard TabIndex(-1);
they remain outside Tab traversal. The first native check exposed that missing
focus opt-in when its End input left the slider unchanged. Pointer focus and
key ownership are both needed for this path. Disabled/hidden sliders do not
capture authoring keys; clearing focus restores
normal selection nudges. Up/Down and PageUp/Down remain existing authoring keys;
standard horizontal Bevy sliders do not consume those keys.

CPU acceptance: all 124 focused Rust tests (55 domain / 69 editor), formatting,
strict workspace/all-target Clippy and build pass. The new regression uses the standard AcquireFocus resolver (including its
TabIndex opt-in), actual InputPlugin KeyboardInput messages, InputDispatch and
SliderPlugin, with a
selected object and pending redo, for both grid and placement-size sliders. Right
changes the expected setting; End/Home/Left retain full source/revision/redo.
Clearing focus restores one normal shared nudge. The original failing evidence is
ignored build/zone-workspace-keyboard-before-fix.log; final acceptance is
build/zone-workspace-keyboard-{all-tests,clippy,build}.log. Native evidence is
recorded after owned key execution below.

Native acceptance passes at both local/far fractional origins in ignored
build/zone-workspace-keyboard-native-3440-v2/ with --keyboard. A real object is
selected before focusing the grid and each shape's placement-size sliders.
Click-to-focus followed by End/Home/Right/Left reaches the expected widget
values while complete source, revision and history stay unchanged, including
pending redo. The subsequent six shared placement oracles, undo/redo and exact
Save checks pass as well. These runs use 3440 x 1440, AMD Radeon RX 7900 XTX
(RADV NAVI31), Vulkan device 29772, explicitly inserted validation layer and zero
strict GPU diagnostics. Editor SHA-256:
fb7187f0ad3cb79851af64d3e0959cc20c9d192fb7f8a65528c0507e0246662c.
The v1 native run stopped on missing pointer-focus opt-in and is excluded from
acceptance. Windows, terrain-slider native keys and workspace Tab traversal are
not newly claimed. Schema, engine pins, networking and gameplay remain unchanged.
No push.

## Workspace slider focus cue — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are click-to-focus and slider key ownership.
The focused workspace slider now has a pale blue thumb; other enabled sliders
keep their cyan thumbs and disabled sliders remain grey. The Properties footer
explains click-to-focus and Left/Right/Home/End. This is presentation only; it
uses Bevy InputFocus and retains the source/undo/input boundaries above.

All 124 focused Rust tests, formatting, strict workspace/all-target Clippy and
build pass. Existing CPU layout checks retain the shape-control bounds. Evidence:
ignored build/zone-slider-focus-cue-{all-tests,clippy,build}.log. Native rendering
and key checks are recorded below after execution.

Native acceptance: the local-origin 3440 x 1440 --keyboard placement workflow
passes for the grid and all shape sizes, retaining selected source/redo and the
three shared command oracles. App-only captures were inspected: the focused Z
thumb is visibly pale while other thumbs stay cyan, and the four-line key hint
fits below the controls. Evidence: ignored build/zone-slider-focus-cue-native-
3440-v1/local/ and its qualification.json. AMD Radeon RX 7900 XTX (RADV NAVI31),
Vulkan device 29772, confirmed active validation-layer insertion and zero strict
GPU diagnostics. Editor SHA-256:
bea3000222275ca90ccef1267439fad1faaf27a0955963c30207bb74cd576a19.
This presentation change has native local-origin coverage; the earlier far-origin
keyboard evidence remains tied to its own binary. No new Windows acceptance.
Schema, engine pins and gameplay unchanged. No push.

## Read-only primitive placement preview — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are primitive dimensions, checked surface/
grid conversion and the existing shared Editor command path. Moving a placement
tool over ground now shows its prospective stamp with a translucent blue mesh.
Refused candidates use red. The ghost hides over panels or when no candidate
exists. It neither changes source/history/dirty state nor reserves an ID. A click
still performs one validated shared Edit::Place. Preview and actual placement
share pose and object construction, including dimensions, snapped X/Z and
half-height terrain support. Candidate validation uses the same Editor::execute
path as structured commands. The cached candidate is reused only while revision,
ID and object/pose remain equal. Source changes, defaults or tool changes recheck
it. A refused click retains the existing status error behavior.

The renderer uses one nonpickable, nonshadowing Bevy mesh entity, the three
existing palette meshes and two fixed standard alpha-blended materials. It adds
no physics collider and no streaming/custom renderer. Checked f64 global poses
convert through the zone frame for f32 rendering. Pure hover is not an authoring
draft and does not block New, Save, agent commands or playtest. Switching workspace
clears the candidate. A placement ray's ground filter excludes the ghost itself.

CPU acceptance: 127 focused Rust tests (55 domain / 72 editor), formatting,
strict workspace/all-target Clippy and build pass. Tests cover all shapes and
snap settings at local/far fractional origins, complete candidate/stamp equality,
unchanged saved/source/history/redo/allocator under hover, rechecking after a
stamp, invalid coordinate/bounds, ID exhaustion and the actual 1024-object budget.
The standard ECS renderer is checked for one hidden/visible ghost, correct shape
mesh/material/local pose, no picking/shadow casting and fixed mesh/material
counts while shapes and valid/refused candidates change. Evidence: ignored
build/zone-placement-preview-{tests,all-tests,clippy,build}.log. Native acceptance
is recorded after the owned-input runs below; these CPU checks do not claim GPU
pixels or native device execution.

Native acceptance passes both local/far fractional origins at 3440 x 1440 in
ignored build/zone-placement-preview-native-3440-v1/ (--keyboard --preview).
All three blue ghost poses/scales match the eventual exact stamped object;
hover and panel entry leave complete source/revision/history unchanged, and
panel entry hides the ghost. The six command oracles, undo/redo, settings keys
and exact Save checks pass. An owned mailbox request adds a high-ID fixture to
exhaust the allocator: its red candidate renders, clicking refuses without
source/history change, and Undo removes the fixture and restores the saved
source. This is authoring validation evidence, not multiplayer or capacity.
Blue local and red far app-only captures were inspected. Each accepted case uses
AMD Radeon RX 7900 XTX (RADV NAVI31), Vulkan device 29772, confirmed active
validation-layer insertion and zero strict GPU diagnostics. Editor SHA-256:
2ed2ab4ed0521edd77742cb2b05608f694ceb73d70093fa2dd00be49ebcfa0c4.
Native terrain support and Windows execution are not newly claimed. The 1024-
object refusal and resident asset checks are CPU evidence, not a scale benchmark.
Schemas, engine pins, physics/gameplay authority and networking are unchanged.
No push.

## Placement refusal hints and renderer guards — 9 November 2026

P5-17 / P3-15 / P1-10; prerequisites are the read-only candidate and existing
forms/playtest. Properties now explains a refused candidate before clicking;
valid candidates retain a concise preview/click hint. The renderer also checks
forms, playtest, active drags/strokes/path drafts and matching placement tool
before showing a cached candidate. This avoids depending on pointer-system order
to hide a stale ghost. These checks do not change validation, source or history.

All 127 focused Rust tests, formatting, strict workspace/all-target Clippy and
build pass. The existing real ECS renderer test now opens/cancels the actual
Find form and enters/leaves actual Avian playtest while deliberately retaining
its cached candidate and not running the pointer system. The ghost hides during
the form/playtest, returns when appropriate, and hides when the placement tool
is cleared; source/history remain exact. Evidence: ignored
build/zone-placement-preview-guards-{tests,all-tests,clippy,build}.log.
Native tests additionally exercise the Find modal and playtest with a placement
tool armed, using a fixture spawn away from geometry at 6.25 cm support offset.
Their final acceptance is recorded below after execution.

Native acceptance passes local/far fractional origins at 3440 x 1440 in ignored
build/zone-placement-preview-guards-native-3440-v1/ (--keyboard --preview).
Find hides the ghost even with the pointer back over the ground; cancelling
restores normal preview. The actual Avian playtest runs with a placement tool
armed and keeps the ghost hidden. Returning to editing retains source/history.
The refused-ID preview shows its reason in Properties, and the existing six
command oracles, panel hiding, exact stamped poses, refused-click isolation,
undo/redo and Save still pass. App-only far refusal and local playtest captures
were inspected. AMD Radeon RX 7900 XTX (RADV NAVI31), Vulkan device 29772;
validation-layer insertion is confirmed, with zero strict GPU diagnostics.
Editor SHA-256:
e414864415581f3f96c0add7590108b92b6bd2b50c718b9d74c690a4cb51bddc.
The CPU layout flow additionally uses a 75-character spaced zone name, maximum
size values and a wrapped bounded-zone refusal; every shape slider and the
four-line key footer remain inside Properties. Evidence is in
build/zone-placement-preview-hint-layout.log and the final all-tests log.
No new Windows or multiplayer acceptance. Schema, engine pins and gameplay
remain unchanged. No push.

## Sprinkles UI foundation — 10 November 2026

Sprinkles theme foundation, 10 November 2026 (P5-17): shared upstream
zinc/blue tokens, adapted button states and embedded licensed Inter now style the
palette, command bar, workspaces, content list, properties and dialogs. Focused
inputs use 1 px blue borders; inactive inputs use zinc borders. Rename uses a
compact matching dialog. The retained playtest context/settings restoration is
included. All 127 focused Rust tests, formatting and denied-warning workspace/
all-target Clippy pass. Linux RX 7900 XTX / Vulkan at 3440 x 1440 passes local/far
placement, focused keys, preview hiding, undo/redo, exact document command oracles,
Save and playtest/settings restoration, with zero GPU diagnostics. Native captures
were inspected. Native local/far invalid/corrected naming and numeric-form input checks also pass;
those dialog captures have not been visually inspected. This is a styling
foundation; collapsible sections and panel resizing follow. No particle runtime
or Bevy version change. Evidence: `build/sprinkles-review/` and
`build/zone-sprinkles-theme-native-20261110/`; provenance:
`third_party/sprinkles/PROVENANCE.md`. Keep working locally; do not push.

## Sprinkles collapsible tool settings — 10 November 2026

Sprinkles collapsible tool settings, 10 November 2026 (P5-17): a native Bevy
section header now collapses/expands the retained tool settings with Sprinkles'
chevron, spacing, ghost hover and separator styling. Activation moves focus to
the header so hidden sliders cannot own keys. Actual observer checks at local/far
fractional origins preserve exact source, redo, preferences and saved state;
unfocused/playtest activation is refused. Section telemetry supports later native
qualification. CPU layout and all 128 focused Rust tests pass; denied-warning
workspace/all-target Clippy passes. Native UI/input/GPU acceptance is pending:
mouse and display use are stopped. The previously qualified theme binary remains
available until the new presentation passes native checks. Evidence:
`build/sprinkles-review/sections-tests.log` and `sections-clippy.log`.

## Sprinkles browser and inspector layout — 10 November 2026

Sprinkles browser/inspector layout, 10 November 2026 (P5-17): replaces the
old opposing/stacked side panels with adjacent full-height left browser and
inspector columns, followed by the world canvas. The inspector contains the
contextual palette in a collapsible Tools group and a separate Tool settings
group. Exact transform editing occupies the same inspector column. Wheel routing,
canvas input exclusion and footer bounds use the new regions. Actual Bevy CPU
layout checks cover all six workspaces, reachable tools, long properties and
local/far exact coordinates; all 129 focused Rust tests pass. Native UI/input/GPU
acceptance is pending while mouse and display use remain stopped. Existing native
evidence applies only to the earlier theme binary, retained under
`build/sprinkles-theme-qualified/`. Menus, richer direct attribute editing and
resizing remain subsequent migration work. Evidence: `build/sprinkles-review/`.

## Sprinkles-style application menus — 10 November 2026

Sprinkles-style application menus, 10 November 2026 (P5-17): compact File,
Edit, View and Zone menus replace the long document-command button row. Save and
Playtest remain directly accessible. Native Bevy MenuButton/MenuPopup/MenuItem
provide the menu interaction path; command activation shares existing validation,
undo/history and modal/playtest guards. Headless events exercise open/close/focus,
one-entry undo/redo, disabled/unfocused refusal and far-coordinate preservation.
Raw menu focus blocks authoring navigation, camera movement and browser wheel
paging; closed menu buttons retain global shortcuts. All 130 focused Rust tests
pass. Native visual/input/GPU acceptance remains pending while desktop use is
stopped. Evidence: `build/sprinkles-review/menu-*.log`.

## Sprinkles-style panel resizing — 10 November 2026

Sprinkles-style panel resizing, 10 November 2026 (P5-17): browser and
inspector boundaries expose bounded 12 px grips, hover/drag feedback and standard
resize cursors. Browser width is 300..480 px and inspector width 300..600 px.
Settings remain session-local; cancellation/focus loss restores the original
widths, and gesture ownership shields world/camera/keyboard edits through release.
Shared canvas bounds, wheel routing, exact form placement and footer track the
widths. All 131 focused Rust tests, CPU layouts at narrow/wide/intermediate widths,
formatting, denied-warning workspace/all-target Clippy and build pass.

A partial Linux RX 7900 XTX/Vulkan 3440 x 1440 run exercised resizing, Escape
cancellation, source/history preservation and section collapse/expansion with zero
GPU diagnostics. It stopped safely when focus left the owned window before File
menu input. Full native menu/form/local/far acceptance remains pending; its two
captures have not been visually inspected. Mouse/display use is stopped again.
Evidence: `build/sprinkles-review/panels-*.log` and
`build/zone-sprinkles-native-20261110-v1/`. New native harness:
`tools/qualify_zone_sprinkles.py`. Keep working headlessly; do not push.

## Standard Bevy embedded viewport — 10 November 2026

Standard Bevy embedded viewport, 10 November 2026 (P5-17): adopts the same
ViewportNode mechanism used by Sprinkles. A separate window UI camera presents
the panels and the scene camera renders into the widget's standard image target;
Bevy resizes it to the canvas rectangle. Shared input conversion maps logical
window coordinates into image pixels, and report projection maps back to window
coordinates. Panning uses the actual canvas height. No custom renderer or new
engine/plugin version is introduced.

All 132 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and build pass. Real Bevy camera projection/ray round trips cover all four
views, three panel-width pairs and 1x/1.5x/2x scale factors, including panel input
exclusion. The scene/ghost data-only test initializes the real viewport and its
pointer map. Native rendering, GPU/input, resize and playtest acceptance remain
pending while mouse/display use are stopped. Earlier partial native evidence is
for the pre-ViewportNode binary only. Evidence: `build/sprinkles-review/viewport-*.log`.

## Sprinkles-style numeric attribute dragging — 10 November 2026

Sprinkles-style numeric attribute dragging, 10 November 2026 (P5-17):
visible ↔ labels scrub their exact field values while ordinary text input remains
available. Normal drag steps by 0.1 per 5 logical pixels; Alt/Super uses 1 per 10.
The adapter preserves the starting f64 and exact original string at zero steps,
uses a symmetric dead zone, and refuses invalid/composing input. It deliberately
omits upstream absolute decimal rounding so distant global coordinates cannot
lose precision. Changes remain pending until the retained Apply transaction;
Escape/focus loss cancels without changing source or redo.

All 134 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and build pass. Headless real field-controller checks cover stationary
precision, local/far object/plant/marker/guide-point/spawn/guide-width drags,
command-oracle equality, one-entry Apply, malformed fields and cancellation.
Native mouse/UI/GPU acceptance remains pending while desktop use is stopped.
Evidence: `build/sprinkles-review/scrub-*.log`.

## Sprinkles workspace rail — 10 November 2026

P5-17 uses the upstream 72 px sidebar with 12 px padding/gap, 28 px rounded icon
plates, 16 px artwork and 10 px labels. Objects, Sculpt, Paint, Guides, Markers
and Plants use the retained workspace controls; mode switching keeps existing
modal, focus, preview and playtest guards. Only icon plates receive the blue
active/hover fill. Six original Remix icons are embedded into the editor, with
license and hashes under `third_party/sprinkles/`.

The browser starts after the rail; inspector and canvas follow the two resizable
columns. Shared panel/input conversion, initial layout, wheel bounds, grips,
footer and report coordinates account for the rail. A pointer over the rail
cannot page the browser or edit the world. The native qualifier derives its
resize positions from the reported browser origin rather than old constants.

Acceptance: all 134 focused Rust tests, actual CPU layout for six workspaces and
three width pairs, DPI/projection/picking round trips, resize cancellation,
wheel exclusion, formatting, denied-warning workspace/all-target Clippy and
build. Evidence: `build/sprinkles-review/sidebar-*.log`. No native windows,
desktop input or capture inspection were used. Visual/input/GPU checks remain
pending; previously qualified theme evidence does not qualify this binary.

Desktop use renewed after this slice's headless checks. Native qualification
`build/zone-sprinkles-native-20261110-v3/qualification.json` now covers both
local and far fractional origins at 3440 x 1440 on AMD Radeon RX 7900 XTX
(RADV NAVI31), Vulkan validation enabled, zero GPU diagnostics. It exercises
all six workspace buttons, bounded resizing/cancellation, collapse/expand,
File/Edit menus including keyboard Open, file browsing, exact stationary
attribute clicks, numeric dragging with Cancel and one-entry Apply/undo, and
New/undo. Full documents and source files are checked, not screenshots alone.
Reviewed native captures confirm sidebar icons, panel bounds, menus, file
browser, embedded scene, fields and distant-coordinate numeric values.

Visual review found the Inter font lacks the section chevron glyphs. Headers
now draw chevrons using standard Bevy UI borders and rotation, without another
font dependency. A later local placement run passed three shape placements, keyboard sliders,
preview/command equality, undo/redo and exact save, then timed out while
re-arming the placement tool before playtest. The qualifier now waits for actual
tool hover before pressing; this is a harness change awaiting rerun. Desktop
use was stopped again at the user's request. No native processes remain;
preview/playtest requalification and far placement remain incomplete. No Windows execution or multiplayer claim.

## Sprinkles inspector scroll and button reflow — 10 November 2026

P5-17 replaces inspector clipping with a reusable scrolling pane. It follows the
pinned Sprinkles `scroll.rs`: 24 logical pixels per wheel line, unscaled pixel
deltas for trackpads, a hover-only 3 px Zinc-600 thumb with 3 px edge margin and
24 px minimum length. Standard Bevy ScrollPosition, Scrollbar and ScrollbarThumb
own layout and drag behavior. Finite-input, focus, modal and authoring-gesture
guards retain Theia's command/history boundaries. Pane offsets clamp when content
shrinks. The scrollbar remains visible during hover/drag, takes precedence over
the adjacent resize grip, and owns input through release/focus loss/Escape.

The independent re-arm probe reproduced a pinned Bevy 0.20 interaction: a tool
button can move after activation, before release is delivered to the previously
hovered entity. The original button can remain Pressed, suppressing its next
activation. Presentation cleanup after physical mouse/touch release or focus
loss clears residual Button state without emitting another Activate. Read-only
qualification telemetry includes the pressed flag.

CPU acceptance: 136 focused tests including the actual ButtonPlugin event path,
held/released/focus-loss states, shorter inspector layout, real PointerScroll
unit conversion, bounds, shrink clamping and exact read-only source/history.
Formatting, denied-warning workspace/all-target Clippy and build pass. Native
probe: `build/zone-rearm-probe-20261110-v2/records.json`. The placement run at
`build/zone-sprinkles-placement-native-20261110-v3/` passes six full-document
command oracles at local/far origins, shape-specific keyboard sliders,
preview/modal/panel/playtest isolation, return-to-editor preference restoration,
refused ID-exhaustion preview and exact save/undo/redo on AMD RX 7900 XTX Vulkan,
validation enabled, zero diagnostics. The final inspector-scroll UI run
`build/zone-sprinkles-scroll-native-20261110-v2/qualification.json` also passes
at local/far origins on binary `9a585151e5d17f5840e1c3ae1669f4080ffb25a271096ba1c6caa68d8a519abf`:
24 px wheel steps, bounded scrolling, thumb drag beside the panel grip with
unchanged width/document/history in a 3440 x 600 owned window, restored
3440 x 1440, playtest entry/return and all prior shell checks. Zero GPU
diagnostics including the forced resizes. Native captures were reviewed. No Windows native execution or multiplayer claim.

## Exact attributes in the shared scroll pane — 10 November 2026

P5-17 reuses inspector scrolling in the retained exact-attribute transaction.
Workspace and Attributes pane scopes keep background controls blocked while
allowing the active draft's wheel and scrollbar. The attribute frame uses the
shared neutral border; focused fields retain the blue indication. On focus
changes, the nearest pane scrolls only enough to show the control. A field
focused before its first visible layout remains pending until geometry exists;
manual scrolling does not repeatedly snap back to an unchanged focus. Existing
Apply/Cancel, numeric scrubbing and exact world-f64 interpretation are retained.

A real headless InputPlugin/CPU font-layout test Tabs through all fields and
sliders for six forms (object, plant, marker, guide point, spawn and guide width),
at both local/far fractional origins, in a 420 px-high window. It checks visible
focus rectangles, bounded wheel scrolling within a modal draft, stable exact
strings, unchanged source/revision/redo, and cancellation back to the prior saved
state. The native qualifier's `--attributes-scroll` path uses actual focus and
control entity IDs from read-only report telemetry. That path is prepared and
syntax-checked, not executed: desktop use was stopped again by the user.

Native rendering/input/GPU acceptance remains pending for this presentation.
Completed earlier native runs qualify their recorded hashes, not this binary.

Validation for this slice: all 137 focused Rust tests, formatting, denied-warning
workspace/all-target Clippy and editor build pass. Evidence:
`build/sprinkles-review/attributes-scroll-tests-final-v2.log`,
`attributes-scroll-clippy-final.log` and `attributes-scroll-build-final.log`.
Native acceptance remains pending while desktop use is stopped.

## Sprinkles numeric field frames and vectors — 10 November 2026

P5-17 adapts the pinned `widgets/text_edit.rs` and `widgets/vector_edit.rs`
presentation: 28 px transparent frames, 1 px border, 2 px corner radius,
6 px internal spacing, compact drag prefixes, units, tabular figures and a
12 px gap in the size-vector row. Native TextInput and the retained exact field
values remain underneath. The frame, including its drag prefix, shows focus.
Object size X/Y/Z shares a row; global position X/Y/Z stays in full-width rows
to expose distant f64 values without replacing them with local f32 numbers.
Scalar yaw, scale, radius and guide width retain explicit Theia labels/units.
Numeric parsing, errors, no-op behavior, validation and one-entry Apply/Cancel
remain unchanged. Report telemetry exposes actual drag-label rectangles so the
native qualifier no longer assumes a particular label width.

All 137 focused Rust tests and denied-warning workspace/all-target Clippy pass.
CPU font/layout checks verify positive readable size components in one row,
full-width world-position fields and reachable fields/actions across all six
forms. Existing local/far exact values, malformed input, numeric scrubbing,
keyboard navigation, undo/redo and cancellation tests still pass. Native
rendering/input/GPU acceptance remains pending while desktop use is stopped;
no native window, input or capture inspection was used for this slice.

Rename, search and file-path inputs also use the shared 28 px transparent
frame, 12 px Inter/tabular font and upstream cursor/selection colors; retained
length limits and validation are unchanged. Final headless evidence:
`build/sprinkles-review/shared-inputs-tests.log` (137 tests),
`shared-inputs-clippy-final.log` and `shared-inputs-build.log` all pass.

## Sprinkles content browser scrolling — 10 November 2026

P5-17 shares the existing Sprinkles-style scroll pane with the content browser:
24 logical pixels per wheel line, native pixel deltas, a 3 px hover-only native
Scrollbar, finite bounded positions and the existing authoring/focus/gesture
guards. The bounded 18-row pool and explicit Previous/Next controls remain.
Short windows scroll the current page; pages that fit retain existing wheel
paging. Raw page routing detects actual computed overflow to prevent a single
wheel gesture from scrolling and paging together. Page/filter changes reset the
pane. Changes in source selection request standard Bevy ScrollIntoView; reveal
runs after visible layout, moves only as far as needed and preserves InputFocus.
Unchanged selection does not pull the pane back after a manual scroll.

All 139 focused Rust tests and workspace/all-target denied-warning Clippy pass.
New CPU font/layout and pointer-dispatch cases cover local/far world coordinates,
short and restored full-height windows, selected rows across all three fixture
pages, modal/focus/gesture guards, filter reset and exact source/history/revision/
dirty preservation. Existing full-height paging and long-label layout tests pass.
Evidence: `build/sprinkles-review/browser-scroll-tests-final.log` and
`browser-scroll-clippy.log`. Native visual/input/GPU checks remain pending while
desktop use is stopped. `tools/qualify_zone_sprinkles.py --scroll` now prepares
browser wheel/thumb, width and source/history checks alongside inspector checks;
this extension has only been syntax-checked, not executed for the changed editor.

Editor build and formatting pass (`build/sprinkles-review/browser-scroll-build.log`).
No native process or desktop input was started for this slice.

## Sprinkles checkboxes — 10 November 2026

Sprinkles checkboxes, 10 November 2026 (P5-17): Snap, Handles and Grid
adopt the pinned checkbox.rs 16 px outlined box, 2 px radius, 6 px label gap,
12 px text and hover border feedback. A native UI-border check mark replaces
textual [x]/[ ] prefixes. Keyboard focus adds a blue border and disabled state
uses muted colors. The existing native Bevy Checkbox, guarded ValueChange
handler and Session/View state synchronization remain in use.

All 139 focused tests and denied-warning workspace/all-target Clippy pass.
Evidence: `build/sprinkles-review/checkbox-tests-final.log` and
`checkbox-clippy-final.log`. Native visual/input/GPU acceptance remains pending
while desktop use is stopped; this is presentation work checked headlessly.

Checkbox slice editor build passes: `build/sprinkles-review/checkbox-build.log`.

## Sprinkles dialogs — 10 November 2026

Sprinkles dialogs, 10 November 2026 (P5-17): naming/search and zone-file
forms now share upstream dialog.rs framing: 6 px radius, 1 px border, 80% black
backdrop, separated 24 px header/body/footer padding, exact 18 px semibold title,
14 px regular status description and right-aligned 12 px action labels. Naming
uses the upstream 448 px maximum width; the file browser retains 900 px for paths.
Dialogs cap their height, scroll only the body and keep Cancel/Apply fixed.
Native focus reveal handles viewport/pane size changes as well as focus changes,
while unchanged bounds/focus preserve manual scrolling. Find has its own primary
button label; file Open/Save use primary blue and overwrite uses upstream red.
Existing draft validation, cancellation, guarded commands and file APIs remain.
Checkbox labels also now request exactly 12 px without legacy size remapping.

Headless checks include 140 focused tests; the short-window case opens at full
height then resizes to 420 px with unchanged focus, checks filename and actions,
manual scrolling, naming/search and exact source/history/bytes/file-count safety
at local/far origins. Formatting, denied-warning workspace/all-target Clippy and
editor build pass. Evidence: `build/sprinkles-review/dialog-tests-final-v3.log`,
`dialog-clippy-final-v2.log` and `dialog-build-final.log`. Native rendering/input/
GPU acceptance remains pending while desktop use is stopped. --dialogs native
qualification is prepared and syntax-checked; it has not been executed.

The native scrollbar is confined to a relative body wrapper, so neither the
thumb nor its hitbox overlays the fixed header/actions. Dialog titles and live
status/limits use separate text nodes; validation messages remain available.
Static presentation is adapted, without importing upstream easing or changing
Theia's backdrop/cancel/file overwrite behavior. Native acceptance of this binary
remains pending and earlier qualified screenshots do not cover these changes.

## Sprinkles inspector section organization — 10 November 2026

Sprinkles inspector sections, 10 November 2026 (P5-17): selection actions
now have their own independently collapsible Selection section beside the retained
Tools and Tool settings sections. Shared headers use the pinned panel_section.rs
12 px semibold display text, 12 px vertical padding/body spacing, compact 20 px
header and bottom divider. Inspector status stays visible outside the sections,
including during offline playtest. Existing commands and collapse/focus guards
remain unchanged; section state is a session UI preference, not a source edit.

All 140 focused tests and denied-warning workspace/all-target Clippy pass.
Evidence: `build/sprinkles-review/inspector-sections-tests.log` and
`inspector-sections-clippy.log`; editor build also passes
(`inspector-sections-build.log`). Layout checks retain supported panel widths,
all modes, placement settings and playtest return. Native rendering/input/GPU
acceptance remains pending while desktop use is stopped. The prepared native
qualifier now checks independent Selection/Tools collapse as well as Tool settings;
it has only been syntax-checked, not run for this binary.

## Sprinkles shared action button sizing and borders — 10 November 2026

Sprinkles shared action buttons, 10 November 2026 (P5-17): workspace,
exact-attribute, naming and file actions share upstream MD sizing (28 px height,
12 px horizontal padding, 4 px radius and 12 px text). Full-width tool actions
retain automatic height for wrapping and left alignment; file rows remain left
aligned. Live variant borders now match upstream button.rs: default outline,
borderless primary/destructive/active/ghost/disabled states and corresponding
border/foreground colors. Bounded content rows, rail and section header sizing
remain specific to those controls. Native Bevy activation and command guards stay.

All 140 focused tests, formatting, denied-warning workspace/all-target Clippy and
editor build pass. The CPU font/layout test now includes actual theme/checkbox/
border systems across all modes and supported browser/inspector widths, while
checking exact source/history preservation. Evidence:
`build/sprinkles-review/button-style-tests-final.log`,
`button-style-clippy-final.log` and `button-style-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped;
this slice used no native window, display capture or OS input.

## Sprinkles 52 px top bar and viewport preferences — 10 November 2026

Sprinkles 52 px top bar and viewport section, 10 November 2026 (P5-17):
the editor now uses pinned topbar.rs height, 12 px inset, panel background and
bottom border. Menus/brand sit left; Playtest and primary-blue Save sit right.
The old second header row is removed. Camera presets, grid, snapping, handles
and gizmo mode move into their own independently collapsible Viewport inspector
section. Grid/Handles share a wrapping row. Existing View-menu camera access and
all guarded commands/preferences remain. Browser, rail, inspector and embedded
viewport start at the shared 52 px boundary, recovering 40 px of working height.

All 140 focused tests pass. The actual-theme CPU layout case now checks all ten
relocated viewport controls inside the inspector, both top-bar actions inside
52 px, rail/browser/tool geometry across modes/widths, and exact source/history
preservation. Source pick/window/image conversions use the shared header boundary.
Formatting, denied-warning workspace/all-target Clippy and editor build pass.
Evidence: `build/sprinkles-review/topbar-tests-final.log`, `topbar-clippy.log`
and `topbar-build.log`. Native rendering/input/GPU acceptance remains pending while desktop
use is stopped. The modern Sprinkles qualifier now uses measured inspector top
bounds and checks independent Viewport collapse; it has only been syntax-checked.

## Sprinkles content-row action menus — 10 November 2026

Sprinkles content-row action menus, 10 November 2026 (P5-17): objects,
guides, markers and plants now expose a compact ellipsis menu and secondary-click
access following pinned data_panel.rs. Actions reuse the native application-menu
items and guarded Rename/attributes/Duplicate/Ground/Frame/Delete commands.
Opening selects the row through the existing validated selection path, then binds
the context to its exact target, source revision and row mapping. Stale selection,
revision, paging or filtering cannot redirect an action to a different target.
Busy/unfocused contexts close; native focus returns to the trigger on dismissal.

Native Bevy Popover placements keep the popup within the window; OverrideClip
lets it escape the scrolling browser. Open menus own wheel/panel-edge gestures,
and focus reveal treats popup items independently of the underlying scroll pane.
Menu-item text state now respects its own active/disabled state. Row labels use
exact 12 px text and reserve space for the 24 px trigger. The row pool stays bounded.

All 143 focused Rust tests pass, including native menu event dispatch, local/far
complete-document duplicate oracles, replay/stale guards, exact undo, short-window
popup bounds/unclipping and raw panel-edge ownership. Formatting, denied-warning
workspace/all-target Clippy and editor build pass. Evidence:
`build/sprinkles-review/context-tests-final-v3.log`, `context-clippy-final-v2.log`
and `context-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped.
The prepared Sprinkles qualifier now covers ellipsis/secondary-click, Frame and
rename cancellation using explicit menu-item telemetry; it is syntax-checked only.

## Sprinkles viewport dropdown fields — 10 November 2026

Sprinkles viewport dropdowns, 10 November 2026 (P5-17): Camera and Gizmo
mode now show current values in full-width combobox-style fields instead of
button clusters. The pinned combobox.rs MD trigger presentation supplies 28 px
height, 12 px text/padding, current-value label, right chevron and active-open
feedback; shared field labels use 10 px medium text and 3 px spacing. Native
Bevy menus reuse the existing four camera/three gizmo choices and guarded handlers.
Popup placement flips above near the window bottom and escapes panel clipping.
Gizmo mode reflects its existing Objects-only availability; busy/unfocused popups
close while preserving a newly opened text form's focus.

All 146 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Tests cover all choices at local/far origins,
current-value labels, exact source/history/revision preservation, disabled/focus/
modal guards, clipped short-window placement, and actual theme/menu/selector
geometry across modes and supported panel widths. Evidence:
`build/sprinkles-review/select-tests-final.log`, `select-clippy.log` and
`select-build.log`. Native rendering/input/GPU acceptance remains pending while
desktop use is stopped. The prepared native qualifier now covers all seven
choices and gizmo availability; it has only been syntax-checked for this binary.

## Sprinkles document selector and zone rename access — 10 November 2026

Sprinkles document selector, 10 November 2026 (P5-17): the top-left
header now shows the current zone name, upstream unsaved '* ' prefix and chevron
through the shared native dropdown. Its ghost trigger follows pinned
project_selector.rs and is bounded to 320 px; a clipped label wrapper protects
long Unicode names and the arrow. The menu reuses New/Open/Save as/Reload and
adds visual access to the retained zone-rename form. Explicit Rename zone targets
the zone even while an object is selected; ordinary selection rename and Shift F8
remain. File actions retain their existing guarded/undoable behavior.

All 148 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Tests cover local/far zone rename with a selected
object, exact document equality, dirty marker / rename undo, undoable New and
restored unsaved content, and long Unicode header geometry with actual theme.
New/Undo retains the existing file-binding behavior: a restored document remains
unsaved after New cleared its binding. Evidence:
`build/sprinkles-review/document-selector-tests-final.log`,
`document-selector-clippy.log` and `document-selector-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped.
The prepared native qualifier now checks document-menu rename/cancel and marker
transitions around apply/undo; it has only been syntax-checked for this binary.

## Sprinkles panel headings and focused summaries — 10 November 2026

Sprinkles panel heading/summary cleanup, 10 November 2026 (P5-17):
browser and property headings now use compact 12 px semibold display text and
title case, matching the shared panel-section typography. Inspector summaries
lead with selected content/tool information; document name/save state stay in
the document selector, window title and footer. Offline-playtest controls,
placement refusal reasons, global-position summaries and tool metadata remain.
The exact-attribute form uses the same property heading style.

All 148 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence:
`build/sprinkles-review/panel-headings-tests-final.log`,
`panel-headings-clippy.log` and `panel-headings-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped;
no native window, OS input or image inspection was used for this slice.

## Sprinkles-style aligned attribute summaries — 10 November 2026

Sprinkles-style aligned attribute summaries, 10 November 2026 (P5-17):
selected objects, plants, markers and guide points/widths now show aligned label/
value rows in the default inspector. Shared capture supplies the same original
f64 values as the exact editor: 10 px medium muted labels, 12 px tabular values,
20 px minimum rows and bounded label/value columns. Full global coordinates are
shown without formatting them through local f32 positions or three-decimal
rounding. Names/kinds/IDs, marker definition/annotation and guide-point metadata
stay in the selection description. Object drags show the candidate pose with an
explicit preview label; source data remains untouched. Summaries hide during
offline playtest. Attribute edits retain the existing Apply/Cancel form.

All 150 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. New CPU layout checks cover all six exact-editor
fixtures at local/far origins, correct visible fields/labels, readable bounded
columns, exact draft/summary consistency, preview/source transitions, playtest
hiding and complete source/revision/history/dirty preservation. Evidence:
`build/sprinkles-review/attribute-summary-tests-final-v3.log`,
`attribute-summary-clippy-final.log` and `attribute-summary-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped.
The prepared qualifier reads actual rendered-value Text components via telemetry
and compares all seven object values with source; it is syntax-checked only.

## Direct attribute-value entry into exact editing — 10 November 2026

Direct attribute-value editing, 10 November 2026 (P5-17): selecting an
inspector value now opens the retained exact editor at that field. Ghost value
buttons use the shared 28 px interaction style and a short edit hint. Native
ActivateOnPress opens the draft before release, preserving text-input ownership.
The entry point binds to rendered target/type/revision/guide-point identity and
refuses stale/hidden/unfocused/busy values. Normal F9/menu transform keeps its
first-field entry. All values stage the existing full validated transaction;
Apply/Cancel, no-op and undo/redo semantics remain.

All 153 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Tests enter every field on all five selected-target
fixtures at local/far origins, verify exact focus/draft strings and real Cancel
redo preservation, reject stale/hidden input, and exercise held press/release
reflow plus repeated reopening without source edits. Evidence:
`build/sprinkles-review/attribute-actions-tests-final.log`,
`attribute-actions-clippy.log` and `attribute-actions-build.log`.
Native rendering/input/GPU acceptance remains pending while desktop use is stopped.
The prepared native qualifier now clicks all seven object values and checks the
focused field, exact draft and cancellation; it has only been syntax-checked.

## Complete selected-inspector headless layout checks — 10 November 2026

Complete inspector layout checks, 10 November 2026 (P5-17): a new CPU
font/layout case assembles the production inspector, theme, selectors, menus,
scrollbars, summaries, footer and exact-form controllers with real object/plant/
marker/point-guide/whole-guide selections at a distant origin. All 20 combinations
of selection, 300/600 px inspector widths and 420/1440 px window heights reveal
each visible inspector button/slider/info block through native ScrollIntoView.
Bounds stay within the panel/window and the complete document, revision, dirty
state and pending redo remain unchanged. No production behavior changed here.

All 154 focused Rust tests, formatting and denied-warning workspace/all-target
Clippy pass. Evidence: `build/sprinkles-review/inspector-matrix-tests-final.log`
and `inspector-matrix-clippy.log`. Native visual/input/GPU acceptance remains
pending while desktop use is stopped. Windows cross/native execution is unverified:
only x86_64-unknown-linux-gnu is installed and no MinGW cross compiler is present.
This is headless Linux CPU layout evidence, without a renderer or OS input.

## Sprinkles browser actions and file rows — 10 November 2026

Sprinkles browser controls and file rows, 10 November 2026 (P5-17):
content search/filter/paging actions now use shared 28 px MD button geometry and
12 px labels. File rows pair original pinned Sprinkles/Remix 16 px folder/file
icons with bare filenames and shared Ghost/Active states. Bounded scan/paging,
selection, filename validation and explicit overwrite confirmation are retained.
Menu keyboard hints keep muted 10 px secondary typography through state updates.
Icon hashes, upstream Git blobs and licensing are recorded under third_party.

All 154 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence: `build/sprinkles-review/browser-polish-tests.log`,
`browser-polish-clippy.log` and `browser-polish-build.log`. Native visual/input/GPU acceptance remains pending;
no windows or desktop input were used while the user needs the display.

## Sprinkles-style tool shortcut columns — 10 November 2026

Sprinkles-style tool shortcut columns, 10 November 2026 (P5-17):
Tools rows now separate action names from right-aligned 10 px muted keyboard
hints, using the same secondary typography as native menus. Each workspace keeps
its existing action bindings, visibility/active states and live brush-value
labels. The retained action captions supply shortcuts; no keyboard mapping or
source transaction changed.

All 154 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence: `build/sprinkles-review/tool-captions-tests.log`,
`tool-captions-clippy.log` and `tool-captions-build.log`. The complete selected-
inspector CPU layout matrix passes. Native visual/input/GPU acceptance remains
pending while desktop use is stopped.

## Shared matching native sliders — 10 November 2026

Shared Sprinkles-style native sliders, 10 November 2026 (P5-17):
tool settings and exact-attribute forms now share a thin 4 px Zinc track and
compact rounded blue thumb. Hover, keyboard focus and disabled track states use
the adopted palette. The full 24 px control area, 14 px horizontal thumb width,
travel bounds, native Bevy input, step/range/precision and validated draft/source
controllers are retained. This is a Theia-specific matching component, not an
imported upstream slider; the reviewed upstream widgets have no slider.

All 154 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence: `build/sprinkles-review/shared-sliders-tests.log`,
`shared-sliders-clippy.log` and `shared-sliders-build.log`. CPU checks include
slider keyboard behavior, exact draft/source preservation and the complete
selected-inspector layout matrix. Native pointer/visual/GPU acceptance remains
pending while desktop use is stopped.

## Aligned tool-setting names and values — 10 November 2026

Aligned tool-setting values, 10 November 2026 (P5-17): tool settings
now present 10 px medium muted names beside right-aligned 12 px tabular values
and units, above the retained native sliders. Exact-form slider labels share
the same field typography. Existing setting captions still supply status text,
units and precision; no range, keyboard mapping, source edit or transaction changed.

All 154 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. Evidence: `build/sprinkles-review/setting-labels-tests.log`,
`setting-labels-clippy.log` and `setting-labels-build.log`. CPU layout includes all
selected-inspector widths/heights and slider/exact-draft checks. Native visual/
input/GPU acceptance remains pending while desktop use is stopped.

## Current content-browser native qualifier preparation — 10 November 2026

Content-browser native qualifier preparation, 10 November 2026 (P5-17):
`tools/qualify_zone_content.py` now reads current browser/inspector rectangles
instead of the retired right-hand coordinates, routes wheel input into that
browser and opens Edit spawn through the Zone menu. It records layout geometry,
uses the shared client environment filter and verifies the binary hash remains
unchanged through a run. Python syntax and --help checks pass. No native run,
window, screenshot or desktop input was performed; this is prepared coverage,
not new visual/input/GPU acceptance. Production code did not change.

## Current grid/pan native qualifier preparation — 10 November 2026

Grid/pan native qualifier preparation, 10 November 2026 (P5-17):
`tools/qualify_zone_grids.py` now uses current canvas height for projected pan
expectations, actual browser/inspector rectangles for gesture-ownership checks,
and View/Zone menus for camera and spawn commands. It records layout geometry,
uses the shared client environment filter and guards the binary hash. Python
syntax and --help checks pass. No native run, window, capture or desktop input
was performed. Existing recorded GPU evidence remains limited to its binaries;
this script preparation adds no visual/input/GPU acceptance or production change.

## Current naming/property native qualifier preparation — 10 November 2026

Naming/property native qualifier preparation, 10 November 2026 (P5-17):
`tools/qualify_zone_names.py` now checks exact fields against the current
inspector rectangle and chooses the visible inspector Transform action rather
than a hidden menu item with the same command. It records layout geometry,
filters native client environment and verifies a stable binary hash. Python
syntax and --help checks pass. No native run, window, capture or desktop input
was performed; production code and native acceptance are unchanged.

## Native slider-event headless qualification — 10 November 2026

Native Bevy slider-event headless checks, 10 November 2026 (P5-17):
a new CPU font/layout case sends actual PointerPress/PointerRelease events
through Bevy's native slider and pointer-focus observers. At local/far origins,
yaw, plant scale, marker radius and point/whole-guide width reach both authored
range ends, acquire focus and clear Pressed on release. Only the selected draft
field changes; all other original strings, source document, revision, dirty
state and pending redo stay unchanged. No production code changed.

All 155 focused Rust tests, formatting and denied-warning workspace/all-target
Clippy pass. Evidence: `build/sprinkles-review/native-track-focused.log`,
`native-track-tests.log` and `native-track-clippy.log`. This is Linux CPU event/
layout evidence without an OS input device, Winit window, renderer or GPU.
Native visual/input/GPU acceptance remains pending while desktop use is stopped.

## Resize affordances follow input ownership — 10 November 2026

Consistent resize affordances, 10 November 2026 (P5-17): divider
highlight/cursor feedback now shares the drag controller's window/menu/modal/
playtest/preview availability checks. A hovered scrollbar suppresses idle resize
feedback as well as new resize acquisition. Active permitted resizing keeps
feedback while crossing a scrollbar. The cursor uses Sprinkles' native column-
resize icon. Width limits, retained drag/cancel handling and source/history stay.

All 156 focused Rust tests, formatting, denied-warning workspace/all-target
Clippy and editor build pass. A CPU regression covers cursor/highlight removal
and blocked acquisition while menus/scrollbars own input, restoration when
available, focus-loss cleanup and source/width/history preservation. Evidence:
`build/sprinkles-review/resize-feedback-tests.log`, `resize-feedback-clippy.log`
and `resize-feedback-build.log`. No OS input/window/renderer was used; native
visual/input/GPU acceptance remains pending while desktop use is stopped.

## Close-enough UI migration native acceptance — 10 November 2026

P5-17 Sprinkles-style UI migration accepted on Linux, 10 November 2026:
the user accepts a close, usable match without pixel-perfect comparison. Native
local/far runs pass menus/selectors/context actions, direct attribute entry,
exact-edit/undo preservation, panel/section behavior, resize, dialog and attribute
focus scrolling, real wheel/thumb input in short windows and playtest return.
Both runs restore exact source content and preserve the source file, with zero
Vulkan diagnostics on AMD Radeon RX 7900 XTX (RADV NAVI31). Native sizes include
3440x1440, 3440x600 and 3440x420. The qualified binary SHA-256 is
8bbfefa2104ec5f0167abc964febf61c8fed24e6480518a81bd4085d6279e550.

Evidence: `build/zone-sprinkles-close-enough-20261110-v2/qualification.json` and
`build/zone-sprinkles-close-enough-far-20261110-v2/qualification.json`. Harness
fixes keep wheel batches within the desktop helper's bound and wait for applied
focus-reveal geometry after resize. Earlier failed runs were harness failures,
not acceptance. All 156 focused Rust tests, formatting, denied-warning workspace/
all-target Clippy and the editor build previously pass for this production code.
Windows cross/native execution remains unverified; no multiplayer claim is made.

The UI migration slice is complete at this Linux acceptance scope. Begin the
next Sprinkles integration slice by porting reviewed runtime/editor source to
Bevy 0.20. The compatibility probe is under ignored build/sprinkles-port; no
particle runtime or authoring workspace has yet been added to the Theia editor.
Full authoritative spell VFX/audio P2-13 remains separate. Desktop use is renewed;
keep working locally without pushing.

## Quest-area catalog export foundation — 12 November 2026

Schema 9 stores `quest_areas` separately from annotation markers. Each entry has
an authoring ID, stable `definition`, name, inclusive global f64 `min`/`max`, and
optional `map_objectives`. Shared `put_quest_area` / `delete_quest_area` commands,
rename, translation and history are supported. Duplication requires explicitly
choosing a fresh gameplay definition via put; generic copy cannot alias one.

```bash
target/debug/theia-zone-editor --zone content/zones/quest-areas.zone.json \
  --export-quest-areas build/starter-areas.json \
  --quests content/quests/development-journal-areas.json \
  --content-id theia.areas.starter --revision 1
```

The command validates references using the real runtime quest and area catalogs
and refuses an existing destination. This is explicit standalone catalog export,
not a normal gameplay bundle update. World export currently refuses area-bearing
zones until gameplay publication is connected. Visual controls are the next slice.

## Quest-area inspector — 12 November 2026

P5-17 / P3-15: loaded schema-9 areas appear under the **Quest areas** explorer
filter. Select one to edit its name and exact global minimum/maximum coordinates
in the existing permanent Attributes text fields. Enter or leaving the field
commits a validated edit; Save persists the zone file. Invalid/inverted bounds
leave history and source unchanged. Frame centers the bounds; Delete removes the
area through shared undo/redo. Duplicate, Ground and placement gizmos are not
available for area volumes. Definition IDs and minimap-link counts are currently
read-only metadata. CPU layout and real Bevy edit-event checks run without an OS
window, renderer or device input; native appearance remains unqualified.

## Quest-area creation and binding — 12 November 2026

**Add → Quest area at spawn** creates a selected 8 m volume containing the spawn,
clamped inside the editor's finite authoring bounds. **Attributes → Quest binding**
contains permanent Area ID and Map objectives fields. Enter or blur applies the
change through shared validation/history; Save writes the source. Map objectives
accepts comma-separated objective IDs (empty clears the links). Format errors and
duplicate links are rejected immediately; catalog export checks reference
existence. These minimap links are presentation links: the quest catalog's own
reach-area objectives define authoritative area credit. No automatic quest
rewriting occurs when an Area ID changes. Native interaction remains pending.

## Combined gameplay publication — 12 November 2026

P5-17 / P3-15 reuses the launcher's shared Rust gameplay validators. To export
geometry and explicit quest areas together:

```sh
target/debug/theia-zone-editor \
  --zone content/zones/quest-areas.zone.json \
  --export build/starter-world-v1 \
  --gameplay-template content/development-gameplay-classic-quest-areas.json \
  --content-id theia.starter.authored --revision 1
```

The template must support areas (schema 16+). Export retains all other profile
bytes, replaces its area role, updates bundle/catalog identity and revision, and
validates the complete result before publishing. The output includes the normal
world files plus `gameplay/gameplay.manifest.json`, consumable by the existing
launcher's `--gameplay-manifest`. The authoring source includes the full areas;
world export without a template still rejects them explicitly. Invalid references,
unsupported world bounds and existing destinations refuse without replacing
sources or leaving a partial published folder.

The actual headless editor plus launcher snapshot passed four positive/rejection
cases in `build/zone-gameplay-headless-20261112-v2/verification.json`. This does not
prove connected quest credit, creature navigation, placed quest NPCs or pickups.
The template retains its existing quest/contact/creature definitions; area editing
does not automatically move those entities or rewrite quests. Native UI/GPU
acceptance remains pending while the display is reserved.

The same combined export is available through **File → Export world → Gameplay
template (optional)**. Supply the existing manifest path when the zone contains
quest areas; leave it blank for ordinary geometry/particle exports. The worker
uses the same validated snapshot path and reports errors in the dialog. This
four-field form supports Tab/Shift-Tab; chunk export keeps two visible fields.
Headless dialog qualification covers both modes; native appearance remains open.

Selecting a quest area now displays a twelve-edge bounds outline in the viewport.
It updates with committed inline edits and hides in offline playtest; it neither
blocks picking nor adds collision. The implementation uses one standard Bevy mesh
and material for twelve reusable entities. Far-coordinate and lifecycle checks
pass headlessly. Native rendering/appearance remains to be checked when display
use is available; no pixel or GPU acceptance is claimed for this addition.

## Starter basin recipe — 12 November 2026

In a blank document, Add → Starter basin terrain builds an editable painted
height patch and central spawn with one undo action. Existing authoring content
is never overwritten by this preset. Its large opening faces +Z; a narrow arch
can be used as a landmark without replacing the entire opening with a collider.
The small dressed example uses the locally staged Synty library:

```sh
python3 tools/edit_zone.py --zone content/zones/starter-basin.base.json \
  --commands content/zones/starter-basin.commands.jsonl \
  --save-path build/starter-basin-draft.zone.json
```

The normal launcher discovers `build/starter-forest-scenery-v2/catalog.json`.
Keep original asset payloads local. The current recipe is terrain/scenery only;
NPC quest assignments and pickups are still separate work. Offline radial
walk/jump tests exercise the intended opening and closed sides at local/far
origins. This is not exhaustive navigation, connected Classic containment or
native appearance acceptance.

## Authored prediction source foundation — 12 November 2026

Exports now also write `authored.world.json` (schema 1), binding the document's
original global origin to the exported world manifest identity. The optional
network source consumes immutable collision/visual snapshots and translates only
collider centres into existing local prediction frames. It rejects origins with
nonzero Y, packages outside the current continental domain, particle catalogs,
shared images and material-profile dependencies. Existing offline and monolithic
export paths retain their prior supported scope. No authored client launch or
quest-playtest acceptance is claimed yet.

`content/zones/starter-zone.base.json` and `starter-zone.commands.jsonl` contain
the same terrain/scenery recipe centred near [8000, 0, 8000], within the current
quest-runtime domain. They are logical source data only; Synty payloads remain
in ignored build directories. Next: standard GLB presentation with readiness
gates, secure client/server launch, then validated NPC/quest authoring.

## Bounded authored client presentation — 12 November 2026

Client/server and `tools/play.py` now accept an explicit `--authored-world` export.
It uses the existing secure predicted movement path, with immutable Bevy GLB
presentation and origin-aware static collision. The normal generated/saved modes
are retained. For a manual session when display use is available:

```sh
python3 tools/play.py --authored-world build/starter-zone-world-v1 --classic
```

The starter export is built from `starter-zone.base.json` plus its commands and
the staged Synty library. The launcher does not silently select saved terrain for
this mode. It currently rejects full gameplay bundles, account persistence and
creature patrols requiring saved-terrain navigation. This is a bounded walking/
dialogue preparation mode, not completed starter-zone gameplay. Native interactive
and connected acceptance remain separate. The minimap's existing height-grid
raster is not yet adapted to this scene; quest-area data support is unchanged.

An opt-in actual-asset test renders offscreen (no desktop/input access), verifies
13 mesh instances, readiness withdrawal/restoration when a mesh is removed, and
root placement after a 128 m frame change. It ran on RX 7900 XTX / RADV NAVI31,
Mesa 26.2.6 Vulkan. `build/authored-gpu-test.log` and
`build/starter-zone-offscreen-v3.png` record the result. This test disables pipelined
render scheduling and does not exercise an interactive client or network session.
An earlier offscreen harness used unsupported default tonemapping; the final run
uses the runtime's Reinhard tonemapping and has no logged rendering errors.

The optional `authored_renet` integration test now runs two actual encrypted peers
against this exported scene with independent collision sources and the Classic
driver. It checks withheld-readiness immobility, movement after readiness and a
stale-identity client's mismatch detection. `build/authored-renet-test.log` records
the four-second run. CPU readiness is explicitly supplied by the harness; this
does not combine the separate GPU test into native-client or quest acceptance.

## Authored navigation and patrol — 12 November 2026

```sh
target/debug/theia-nav-bake bake-authored build/starter-zone-world-v1 \
  build/starter-zone-navigation-v1 content/zones/starter-zone.patrol.json
target/debug/theia-nav-bake verify-authored build/starter-zone-world-v1 \
  build/starter-zone-navigation-v1
```

The route input has a logical ID and global f64 waypoints. Bake uses the existing
agent dimensions, nine 128 m tiles around its first waypoint, actual admitted
static surfaces, 8 m clipping padding and existing input/output budgets. It supports
triangle meshes, oriented boxes, hulls and heightfields; other collision shapes
currently refuse the bake. Camera-only shapes are excluded. There is no terrain
resampling or render-mesh-to-box substitution. Published navigation binds the
origin-aware source identity; original source files and existing outputs stay
unchanged. The exported world and navigation are still separate artifacts.

Verification checks package integrity, navigation sampling, nearby collision
support and capsule clearance at 0.2 m route intervals (bounded to 8192 samples).
The actual starter patrol passes 158 samples. A separate source-backed test uses
the existing authoritative patrol implementation for 900 fixed steps: one loop,
47.98 m travelled, zero blocks and unchanged static collider count. Four navigation
routes to clear exterior ground 70 m from the centre use the same +Z opening.
Initial requests at the cliff toe (60 m) were outside the strict navigation
sampling tolerance; no tolerance was widened to accept those boundary points.
These are bounded offline/simulation qualifications, not native creature/quest
playback or exhaustive route coverage. See `build/authored-nav-routes.log`,
`build/authored-nav-verify.log`, `build/authored-nav-rejections.json` and
`build/authored-patrol-test.log`. Route authoring UI remains separate work.

## Dressed valley connected patrol — 12 November 2026

The normal launcher now accepts an authored world with its matching navigation
and creature profile. The village walking recipe also supports this first
nonaggressive patrol in the northwest grove:

```bash
python3 tools/play.py --authored-world build/starter-valley-village-world-v2 \
  --classic --creatures --binaries target/debug \
  --creature-profile content/gameplay/starter-valley-wolf-patrol.json \
  --navigation build/starter-valley-navigation-v1 \
  --creature-asset build/creature-wolf-stage-v1/wolf.glb
```

The route begins around global X=8100, Z=8130, roughly 92 m west and 62 m north
of the village origin. The existing resource-combat profile provides Arcane Bolt;
this slice does not add retaliation, pursuit, loot, quests or account persistence.
Those modes retain their separate prerequisites. F5 in the editor remains the
offline movement playtest; the command above starts the connected game client.

Patrol preparation and target creation retain validated XYZ feet, so a path on
the ground beneath a roof does not jump to the roof. A synthetic overhead-floor
test verifies preparation, installation and 300 simulated steps. Two actual
encrypted peers receive moving target and combat state while Classic prediction
and stale-world rejection pass; explicit CPU readiness is test scaffolding.
The normal client `--configuration-report` (no renderer/network) matches that
server identity. New native interaction and wolf-in-valley rendering are not
claimed by these checks. See current build/check evidence in `docs/STATUS.md`.

When available, the launcher now prefers the locally qualified 22-clip Synty
character at `build/classic-directional-client-qualified-v1-20261103/art/player`.
This includes directional locomotion, jump, sword stance/attack/hit and death.
`--players DIRECTORY` still chooses an explicit package; older combat and basic
locomotion stages remain fallbacks. Selecting art does not grant equipment or
abilities. These prepared Synty stages do not yet contain a spell-casting clip.
The editor's offline F5 pawn remains a capsule pending the requested player-art
integration; the preference above applies to the connected client.

## Authored player spawn admission — 12 November 2026

P5-17 / P3-15: authored player admission now retains all three coordinates of the
hash-bound collision package's validated spawn. The server selects that saved
spawn before preparing its initial patch. It no longer substitutes development
XZ or searches downward from 256 m, which could place an under-roof spawn on the
roof. Explicit developer-spawn overrides refuse for authored worlds; move the
spawn in the editor and export again instead. Generated/saved development worlds
retain their existing XZ controls and four placement offsets.

Authored spawn slots cycle through the package's explicit supported positions;
players can share a point because actors do not collide with each other. Account
restoration retains the separately validated full position instead of performing
an unrelated second XZ spawn query. This does not enable authored account/combat
launch modes: those existing qualification guards remain in place. Guide/NPC
placement and combat home/respawn integration still need their own full-position
contract before arbitrary interior gameplay is supported.

Four unit checks cover the source binding, a nondefault [8192,8192] origin with
ground-floor and rooftop spawn slots, restored ground-floor placement, rejection
of missing explicit spawns and mismatched development coordinates. Four launcher
checks include the actual captured server argv and refusal of explicit XZ
overrides without starting any child process. Additional network/executable
qualification is recorded in the status checkpoint for this slice.

The new valley is ready for bounded walking/camera/scenery testing. Open the
editable source and press F5 to enter or leave its offline sandbox:

```sh
python3 tools/edit_zone.py --no-build --zone build/starter-valley-village-v2.zone.json
```

Run the authoritative game-client walking version with the rebuilt debug pair:

```sh
python3 tools/play.py --authored-world build/starter-valley-village-world-v2 \
  --classic --binaries target/debug
```

822 ordinary network tests pass with 11 opt-in skips; the actual two-peer authored
Renet check is additionally executed against this valley and passes (including
stale-world refusal). Rebuilt server CLI startup/shutdown and override rejection
pass. The actual client GPU readiness/rebase/asset-withdrawal check passes on RX
7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan with 114 world mesh primitives, without
an OS window/input. Evidence is `build/authored-valley-spawn-renet-test.log`,
`authored-valley-server-smoke.json`, `starter-valley-client-gpu-readiness.log` and
`starter-valley-client-offscreen.png`. These separate network and offscreen checks
do not claim native end-to-end input/Windows qualification. Quest contacts,
multiple wolves, pickups and account/combat wiring are not yet part of this scene.
