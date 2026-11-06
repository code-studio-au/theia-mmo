# Local minimap — initial Classic playable qualified

## Requested quest guidance — 6 November 2026

P2-14 / P3-11 / P5-13 / P5-19. The user's local 3440×1440 captures at
18-50-31 and 18-52-27 add these requirements to the UI direction:

- Show quest objective/search regions, including relevant creature spawn areas
  and areas where required items can be collected. Associate guidance with the
  player's active, unfinished objectives and update it from committed progress.
- Show `!` at known NPCs offering a quest the player can currently obtain, and
  `?` at the correct NPC when an accepted quest is ready to turn in. Revert ready
  guidance if required holdings are lost; completed quests do not reappear.
- Keep area guidance distinct from live creature/item locations. Static authored
  search regions can be known from a quest; individual live pins require actual
  permitted observations. Marking a region does not grant progress or guarantee
  a spawn/drop. Preserve checked global metre projection, circular clipping,
  bounded image/node work and the existing terrain streaming/request budgets.

The current implementation below supports the single known guide's
available/ready state mapping, reach-volume shading and explicit objective-to-area
metadata, described below. A general multi-NPC marker catalogue remains separate
work. Existing development content retains its reach volumes; creature/collection
regions must be authored and reviewed before binding a changed saved profile. Reuse existing bound content
where it describes the actual region; do not invent locations from screenshots
or reveal unknown actors. Finish the current reader presentation slice, then
implement this in bounded guidance/data/UI slices with headless checks. Native
input checks remain paused while the user uses the desktop. The supplied game
screenshots and their chat/player artwork remain local references outside Git.

P2-14 / P3-13, presentation prerequisite for later P4-02 work. This does not
complete predictive streaming or change terrain residency.

Use standard Bevy UI with one 64x64 RGBA terrain image plus a 16x16 arrow (17 KiB), sampled at most four
times a second from already resident visual mesh grids. A north-up view covers
a 256-metre square centred on the player (north is -Z). Unknown/outside-world terrain remains dark.
There is no additional camera, world-size texture, tile reader or loading request.
The current greybox has no buildings/roads/material map layer.

Map conversion subtracts checked f64 global positions before bounded f32 UI
conversion, independently of render/physics origins. The player arrow follows
manual character facing; it does not rotate the camera or select/track a target.
Reuse committed, owner-bound guide/quest observations for a nearby !/?/G icon.
Do not expose unknown NPCs or create interaction/quest entitlement.

Place the map at the upper right and reserve adjacent space for health/mana. Keep backpack controls usable at supported
resolutions by putting its toggle at the lower right and its pane above it.
Hide the map while that pane occupies the right-hand UI space. Map pointer/wheel
events use existing standard Bevy UI ownership; no map click gameplay action.

Acceptance: global/chunk/world-edge projection and exact triangle sampling,
heading and stale/unknown guide filtering, fixed image/node budgets, no terrain
load requests, measured CPU update cost, and native UI/input checks at 1280x720,
1920x1200 and 3440x1440. Recheck normal quests/backpack/camera control and GPU
diagnostics. The initial Classic implementation is qualified below; the connection
confirmation playable remains preserved.


## Qualified implementation and native evidence — 3 November 2026

The 256-metre north-up square uses resident regular mesh grids, including their
actual two-triangle interpolation, instead of reevaluating a terrain recipe.
Out-of-world/unavailable samples stay dark. The map creates one 64x64 terrain
image and one 16x16 arrow, 17,408 RGBA bytes in total. The root and five child
nodes are created at startup; refreshes change the existing terrain image.
There is no second scene/camera, saved-tile read, new worker or residency request.

Live player presentation passes through the existing checked render frame before
map projection. Classic manual yaw rotates the player arrow; the map remains
north-up. Committed guide filtering is shared with the world quest marker:
unknown/disconnected/foreign/unloaded/distant contexts refuse. The nearby guide
shows ! for available, ? for ready, G otherwise; completed quests do not become
available again. Icons are cosmetic and noninteractive. Range, colors and this
map layout are Theia presentation choices, not an adopted Classic combat formula.

Health/mana are placed beside the map in graybox mode. The backpack toggle is
at the lower right, its pane above it; the map hides while the bag occupies that
space, including the existing vendor auto-open behavior. UI clicks, right-drag
and wheel stay owned by standard Bevy UI. Unchanged values do not dirty map UI
layout or text every frame.

All 1,133 workspace tests pass (50 explicit service skips), including 135 client
tests; formatting and denied-warning workspace/all-target Clippy pass. New math
checks cover large global coordinates, world/chunk edges, both mesh triangles,
invalid data and both terrain LOD grids. The existing stale/foreign/unknown-guide
check also covers the shared map entry point.

`build/minimap-journal-native-v2-20261103` passes normal schema-15 OIDC/PostgreSQL
quests at 1280x720, 1920x1200 and actual 3440x1440 fullscreen, plus database process
restart and fresh-ticket rejoin. Each fresh character completes two quests and
three unmodified-wolf kills, reaching head 13 / 125 XP with four committed UI
actions. Rejoin preserves completion/rewards. Captures verify available, active,
ready and completed-guide map phases, backpack access, map input ownership and
the actual UI-transformed arrow matching authoritative facing after manual A turn.

During each four-second visible idle sample, 15–16 terrain image refreshes leave
81 loaded / zero retired terrain tiles, one collision patch and exactly 90 saved
tile reads unchanged. Maximum terrain raster cost is 329 us; maximum complete
map presentation cost is 352 us. Active presentation averages 25.64–26.74 us per
frame in these runs. These are local measured costs, not general hardware or
population/FPS guarantees.

All four native starts have zero GPU diagnostics on Linux AMD RX 7900 XTX /
RADV Vulkan / Mesa 26.2.6. Input is synthetic PID-bound X11, with owned services
stopped and plaintext password files removed. No native Windows or MMO capacity
claim. Preserve exact source/binary hashes and selected proof in
`build/minimap-client-qualified-v1-20261103`. All changes remain local.

The first attempt remains recorded: its screenshot exposed overlapping
health/map panels, now corrected. Its optional map capture waited for visibility
while normal vendor dialogue had opened the backpack; the harness now
deliberately closes that local pane when inspecting the map beside quest dialogue.
No gameplay rule, damage, quest catalogue or timeout was relaxed.

## Remaining scope

The initial checkpoint qualified the normal Classic graybox path. Legacy heading
and native traversal were follow-ups there; the subsequent qualification below
covers them separately. General world-map/continent atlas,
zoom, exploration fog, authored object/road layers, other NPC/party icons,
destination waypoints and icon decluttering/art remain open. The current nearby
guide can share map pixels with the player when standing beside it. Imported
forest/other-map modes have not gained a cartographic layer or new qualification.

## Traversal and retained walking follow-up qualified — 3 November 2026

The arrow now also reads the existing displayed character rotation when a retained
non-Classic walking mode has no explicit facing field. Classic manual/controller
and authoritative-facing paths keep their behavior. This changes map presentation,
not movement, camera, targeting or an adoption of the older automatic-facing policy.
A regression checks character headings independently of the camera/default north.

The final full workspace passes 1,134 tests (50 explicit skips), including 136
client checks. Two Python checker regressions, formatting and workspace/all-target
denied-warning Clippy also pass. Preserve
`build/minimap-traversal-client-qualified-v1-20261103` separately from the initial
Classic map and connection-confirmation checkpoints. Changes remain local.

### Accelerated retained-mode diagnostic

`build/minimap-continent-native-v3-20261103` uses real encrypted development
tickets, saved terrain and the retained predicted walking mode at explicit 32x
development speed, no teleport tour, one client, 1920x1200 and ordinary pacing.
It crosses the 16-km X extent and records 125 render-origin changes. Its one-second
trace independently samples 62 distinct origins in 86 eligible observations;
it does not sample every transition. Maximum numerical map-to-displayed-player
coordinate error is 0.000010751 m, under the unchanged 0.003-m check.
The actual UI arrow agrees with the displayed character forward vector.

Terrain loads/retirements are 2,345 / 2,300, maximum residency 81 and final edge
residency 45. Saved reads total 3,464; the map keeps its fixed 17-KiB image budget.
Exactly 2,048 map pixels remain known at the world edge; off-world terrain stays
dark. All four existing character gait pose checks change, and physics/render
ground discrepancy remains under 0.003 m. Map raster/presentation maxima in this
accelerated run are 612 / 627 us. GPU diagnostics are zero on the same Linux
RX 7900 XTX / RADV device; external Vulkan validation is off.

This is an accelerated compatibility/coordinate diagnostic along one route, not
normal gameplay speed, OIDC/persistence, exhaustive world-tile coverage, or an MMO
population/FPS capacity test. Numerical coordinate error is not visual pixel or
terrain precision: the map still has four-metre terrain texels at this scale.

### Normal Classic keyboard traversal

`build/minimap-walk-native-v1-20261103` repeats actual normal schema-15
OIDC/PostgreSQL quest completion and map controls/layout at 1280x720, 1920x1200
and 3440x1440 fullscreen. After database restart/rejoin at 1920x1200, real
PID-bound W input walks 208.3 m in a 41.9-second check through chunk owners
(62,62), (61,62), (60,62), with render revision 1 -> 2 and new origin
(7680,0,7936). No synthetic authority teleport or speed override is used.

Map coordinates agree with the displayed body within 0.000010269 m in sampled
probes. Terrain genuinely retires: loaded/retired totals end at 116 / 52, collision
patches at four and saved reads at 153, all within the existing bounded pipeline.
The now-distant guide icon disappears. Quest head 13 / 125 XP remains unchanged.
All four normal starts pass with zero GPU diagnostics; owned services and
plaintext password files are cleaned.

### Measurement corrections retained

The first accelerated checker compared predicted map position with the delayed
authority pose. At 32x speed that legitimate presentation lead is about 25.6 m.
Reports now distinguish displayed position/forward from authority; the 0.003-m
map alignment check remains. A second checker expectation incorrectly required
over 80 distinct origins from a one-second trace while origins changed about
twice a second. The final report distinguishes total changes from actual
sampled coverage and checks a >15-km observed span. Both earlier failed check
logs and the corrected v2 captured-data analysis remain; final v3 passes the
complete executable check. No game rule, speed limit or alignment tolerance was
relaxed to hide an engine failure.


## Initial active quest-area overlay — 6 November 2026

P2-14 / P3-11 / P5-13 / P5-19. The existing bound quest-area catalogue now has
an immutable presentation accessor. The client selects regions only from a
validated current-owner journal, active quests, their current stages and
unfinished `reach_area` objectives. Unaccepted, ready/completed, disconnected,
foreign-owner/epoch/hash and invalid-revision contexts produce no search region.
This is static quest geography; it does not need or reveal a live guide/creature.

A restrained blue wash is composited into the fresh existing 64×64 terrain
raster at its existing 250-ms refresh schedule. Overlaps tint once. X/Z projection
uses f64 global differences; a circular skin clips samples to its round map, and
the legacy square keeps rectangular coverage. Vertical reach conditions remain
authoritative gameplay; the map is a 2D search aid. Off-map regions produce no
pixels. No camera, image, node, worker, network command, terrain load or saved-tile
request is added; existing terrain/arrow payload remains 17,408 RGBA bytes.
Region completion clears on the next scheduled raster refresh. Metrics report
selected region count and tinted pixels, not live creature counts or visible
NPC capacity. Hidden maps clear the current objective metrics.

Four new headless cases cover accepted/ready/completed and invalid contexts,
absence of live guide data, circular corners/alpha, empty/malformed buffers,
world-edge clipping and identical shading around 0 and 1,000,000,000 metres.
All **180 client cases** including local art and **15 focused network/quest-profile
cases** pass, along with formatting, strict workspace/all-target Clippy and build.
A 32-region/200-sample colour-pass test on **AMD Ryzen 7 7800X3D**, development test
binary, measures p95 **400 us**, maximum **498 us**. It excludes active-goal
selection, terrain raster work, image upload and frame rendering; it is not a
full-map/FPS/MMO capacity guarantee. Exact log:
`build/ui-reference-20261106/map-guidance-cost-v1.log`.

Six `map-guidance-v1-*` windowless Linux GPU cases render a labelled metadata-only
search region over the existing checker map. The exporter asserts a selected
region and nonzero tinted pixels while retaining the preceding HUD/reader-free
layout gates. The reader branch also passes once. Evidence:
`build/ui-reference-20261106/map-guidance-verification-v1.json` and
`map-guidance-reader-v1/report.json`. Device remains RX 7900 XTX / RADV Vulkan /
Mesa 26.2.6. There is no OS window, mouse/keyboard input, live quest credit or
connected region-render proof in those exports. Desktop testing remains paused.

This first mapping uses actual reach volumes. It does **not** guess a creature's
spawn/search territory from its instantaneous position or mark where quest loot
is guaranteed. Explicit validated objective-to-area metadata for kill/collection
and related goals is next. The existing `!`/`?` mapping remains single-guide
support; general known-NPC locations, hover descriptions, tracking preferences,
map layers, native interaction, Windows and broader combined timing remain open.


## Explicit objective search regions — 6 November 2026

P2-14 / P3-11 / P5-13 / P5-19. An area catalogue can now include optional
`map_objectives` on each region. These are **executable objective IDs**, never
quest, creature or item IDs. The following isolated example requires the selected
quest catalogue to define `example.collect-pelts`:

```json
{
  "schema": 1,
  "id": "example.quest-areas",
  "revision": 2,
  "areas": [
    {
      "id": "example.wolf-territory",
      "name": "Wolf territory",
      "min": [7900, -10, 7900],
      "max": [7950, 10, 7950],
      "map_objectives": ["example.collect-pelts"]
    }
  ]
}
```

Coordinates above are illustrative, not an installed spawn definition. Authors
must use reviewed creature spawn/patrol or collectible-placement territory for
real content. Multiple regions may link to the same objective, and multiple
objectives may share a region. Links are optional, unique within each region,
limited to 32 per region and 512 per catalogue. Existing 32-region / 16-KiB / 1-to-
1024-metre extent development limits still apply. Unknown objective IDs, invalid
IDs, duplicate links, malformed fields and oversized inputs refuse. A map-only
catalogue requires a valid objective link; reach catalogues still require every
`reach_area` selector to resolve. Existing namespace collision checks remain.

Selection uses only validated current-owner, active, current-stage, unfinished
objectives. Explicit mappings work for kill, possession, document and interaction
objectives as well as reach goals. Regions are deduplicated, then use the same
bounded blue raster pass. Confirmed inventory reconciliation from three pelts to
two restores collection guidance. Optimistic backpack edits never select regions.
A search hint neither grants a kill/read/interaction nor promises a spawn/drop.
The headless World fixture verifies that standing inside a kill-search region
emits no reach observation and leaves its kill count at zero.

This additive schema-1 field is a Theia presentation extension. Exact area bytes
remain part of the existing profile hash on network and persistence paths;
metadata changes therefore need catalogue revision/review and explicit saved-
profile migration decisions. Omitted metadata preserves existing bytes/hashes.
Document-only profiles can explicitly bind validated search metadata without a
reach objective; they still reject unrelated/empty context. No schema migration,
protocol field, terrain request, NPC observation or gameplay mutation is added.
Existing development/campaign files and manifests were deliberately not changed;
new creature/collection geography is supported and tested, not silently installed
into a saved development world.

Validation: **181 client cases** including local art; **385 core/persistence
cases** (73 database-dependent cases remain ignored); **26 quest-journal network
cases**, including encrypted reach-area routing, plus the new document-only
binding case. Formatting, strict workspace/all-target Clippy and client build
pass. Exact logs are under `build/ui-reference-20261106/map-links-*`.
The prior six overlay GPU layouts remain the raster evidence; this follow-up
changes metadata and region selection, not projection or widget geometry.
An additional 3440×1440 / UI scale 1.25 windowless regression passes on RX 7900
XTX / RADV Vulkan / Mesa 26.2.6: zero windows, one fixture region, 418 tinted
pixels (`map-links-gpu-v1/report.json`). This is a synthetic layout, not a live
quest/creature scene.
Native connected overlay/input and Windows evidence remain pending. Desktop
and mouse tests remain prohibited while the user uses the machine.


## Read-only hover descriptions — 6 November 2026

P2-14 / P3-11 / P5-13 / P5-19. Hovering the observed guide pin shows its
available, active or ready quest titles; ready turn-ins keep priority over offers.
Unavailable prerequisite quests are omitted. Hovering a shaded search region
shows its active unfinished objective labels and confirmed counts. Overlapping
regions deduplicate each objective, descriptions show at most six entries plus
a continuation hint, and the full catalogue remains in the journal/dialogue.
Search regions remain authored hints rather than live spawn/drop observations.

One standard Bevy tooltip sits left of the minimap. It is noninteractive and
never queues selection, dialogue, movement or quest actions. It clears on pointer
exit, circular-map corner, hidden map or invalid/foreign progress. Bevy's relative
cursor position handles node scale/DPI; read-only guide/tooltip bounds now preserve
preferred UI scale while removing window DPI. The previous map-root bounds export
has the same conversion fix. Existing terrain raster, request and image budgets
remain unchanged. The guide is still the sole observed NPC contact; this does not
add a general multi-NPC registry, map clicks, tracking controls or live loot pins.

Four new headless cases cover relative/circular cursor projection, active bound
search geometry and foreign-owner refusal, contact offer filtering, and clearing
a noninteractive tooltip. **185 client cases** including local art, formatting,
strict workspace/all-target Clippy and build pass. Six `map-hover-v1-*` windowless
Linux GPU layouts show an explicitly labelled contact fixture, assert that its
tooltip fits and leaves the map clear, and retain the HUD/backpack/area gates.
Device: RX 7900 XTX / RADV Vulkan / Mesa 26.2.6. Evidence:
`build/ui-reference-20261106/map-hover-verification-v1.json` and the corresponding
client/Clippy/build logs. No native scale/DPI inference follows from those exports.

Desktop testing is authorized again by the user's latest continuation request.
Connected Linux qualification passes at 1280×720 / scale 1, with actual local
OIDC, encrypted Renet, owned PostgreSQL, PID-bound X11 pointer/keyboard and
restart/rejoin. Real `!`/`?` hovers show the correct title without changing the
committed quest revision, and pointer exit clears text. The completed guide
remains generic after rejoin. The unchanged two-quest flow finishes at journal
revision 11 / XP 125, with zero GPU diagnostics; pointer/wheel ownership, 17,408
image bytes and unchanged idle streaming pass. Owned services stop, temporary
wheel device is removed and plaintext password files are removed. Evidence:
`build/ui-reference-20261106/native-map-hover-v1/verification.json`. This does not
qualify native search-region hovering, enlarged UI, Windows or physical hardware
input. Earlier paused entries record permission at those checkpoints.
