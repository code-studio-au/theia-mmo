# Local minimap — initial Classic playable qualified

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
