16 November 2026 — prepare every bounded authored-world mesh before movement:
P2-02 / P2-08. A few camera-visible compiled pipelines do not establish readiness
for an entire retained authored scene. Native capture and diagnostics reproduced
all friendly contacts disappearing for roughly half a second during the first
quest approach: a newly visible world pipeline was still compiling, which revoked
all reported chunks despite all 116 CPU/GPU meshes remaining loaded. This can
occur in either client and was missed by the earlier stationary captures.
Use standard Bevy NoFrustumCulling temporarily on authored meshes needing pipeline
preparation, require a specialized compiled pipeline for every mesh, then remove
only the temporary overrides and resume ordinary culling. This is bounded scene
startup/recovery, not a general streaming prewarm or custom renderer. Preserve
CPU/GPU asset checks, genuine readiness retraction and authoritative movement
blocking when an asset is absent; do not keep stale readiness through asset loss.
Expose readiness/retraction counters and require zero retractions during the native
quest/escort run. Camera sweep, restoration and rebase checks cover the lifecycle.
A startup loading screen was discussed as presentation polish; none is added by
this correction. Final acceptance is recorded separately in STATUS.

15 November 2026 — party escort participation and lifecycle:
Following the user's any-participating-member-in-range rule, participants are
current party members whose committed journal has the same Active escort.
Unaccepted, Ready and Completed contacts remain private so other quests stay
available. Living participants may keep the shared ranger within her existing
10-metre bound; dead participants observe but cannot drive or receive arrival.
A late accepted participant or merged party adopts the furthest existing checked
route progress. Leaving or dissolution forks the current position without rewind;
disconnect removes that incarnation. These merge/eligibility choices are explicit
Theia policies, not claimed Classic rules. No search, database work or transport
await enters fixed ticks. The party v3 profile binds this behavior.
Arrival facts remain character-specific and require that living character inside
the destination range. Durable Ready state and speaking to the ranger remain
required for each player's own reward. A distant member gets no automatic credit.
Session restart still resets Active travel to home, while saved Ready/Completed
journeys restore at the destination. Shared XP/loot/general quest credit are
outside this slice. Headless implementation and native acceptance are recorded
separately in STATUS/PARTIES.

15 November 2026 — bounded party membership slice:
P4-11 follows authenticated actor admission and existing encrypted control/UI
integration. Adopt the five-member/leader model evidenced by the original 2004
Blizzard manual; record its limits and explicit session/invitation/request policies
in PARTIES.md. Implement membership independently of shared rewards or escorts.
Disconnect leaves the initial session-local party; death does not. No persistence
or distributed service is introduced by this slice.

15 November 2026 — deferred party escort behavior (user direction):
Keep current solo escorts private per character. Once the party system exists,
participating party members should see one shared ranger journey. Any participating
member within the escort range keeps the ranger moving; she waits when none are
in range. The user explicitly deferred implementation until the party system.
Eligibility, joins/leaves and completion credit still need a concrete party slice;
this decision does not claim shared escort behavior is implemented.

15 November 2026 — bound continuous scene readiness waits:
P2-02. Authored scene readiness may retract after successful initial loading.
Timeouts measure each uninterrupted pending interval, not total client uptime.
Readiness clears that interval; a later retraction starts a new bounded wait.
Keep the existing startup/recovery timeout and actual CPU/GPU readiness gates;
do not allow movement through an unprepared world or bypass missing assets.

15 November 2026 — interpolate authored population target presentation:
P3-07 / P4-03. Startup must opt pursuing population definitions into the existing
TargetMotion/RemotePawn presentation path, just as singleton patrols already do.
Use the maximum configured chase/return speed and current 1.5 m/s idle travel as
the nominal speed for the existing catch-up policy. Passive stationary definitions
remain unchanged. Do not derive authority, hit range or selection eligibility
from interpolated poses; no protocol or fixed simulation changes. Expose blended
frame counts alongside existing lag/speed diagnostics to verify actual native
presentation rather than inferring it from actor movement alone.
Choose creature gait from successive authoritative positions and server tick
intervals, retaining the existing brief stop grace. Render catch-up speed after
loading or screenshot stalls must not select a running clip for a walking actor.
Facing/placement still use the interpolated path; attacks/death retain priority.
Discontinuous positions and restarted timelines reset locomotion observation.

15 November 2026 — explicit bounded population patrol guides:
P3-07 / P4-03 / P5-17. Zone schema 15 adds an optional spawn-group Patrol guide
reference. Older groups remain stationary or keep their existing roaming radius.
Inspector assignment sets roaming to zero atomically; stored patrol plus roaming
is invalid. Each spawned actor prepares home → authored guide → reversed route →
home with the shared navigation and capsule clearance checks before admission.
Keep sparse guide points unchanged. Limit every guide/derived point to 64 m from
that actor's home and the outward route to 128 points (255 with return), inside
the existing loaded collision patch. These are initial Theia engineering bounds,
not Classic constants. Use existing 1.5 m/s idle travel and 2–6 second pauses;
combat return resumes the captured segment, while respawn resets to home. Only
the current ordinary pursuing definition and existing 16-actor runtime cap are
supported. No search/I/O enters fixed ticks; no passive patrol or streamed-route
claim. Route geometry participates in the shared gameplay identity.

15 November 2026 — resume population excursions at the interruption anchor:
P3-07 / P4-03. The pursuit controller captures the creature's actual location
when combat begins and returns there after disengagement. Preserve the roaming
excursion and next waypoint through that interruption, then use the existing idle
pause before continuing. Only respawn resets progress to home. Resetting progress
on every combat tick was inconsistent with the return anchor and could cut across
unvalidated corners. No Classic balance constants or navigation policy change.

15 November 2026 — interpolate moving contacts with existing remote poses:
Reuse RemotePawn for presentation of authoritative contact positions. Do not add
physics bodies, client navigation, optimistic quest state or movement requests.
Contact mesh/name presentation may trail receipt by a bounded interpolation
interval; interaction range remains authoritative. Missing contacts and identity
changes reset interpolation, and the existing remote discontinuity policy snaps
large relocations. This improves moving NPC presentation without a protocol or
simulation change; packet timing and low frame rates remain measurable limits.

## 15 November 2026 — shared escort export preflight

P4-03 / P3-11/12 / P5-17. An editor export must not appear playable merely because
its contact catalogue and population spawn points validate. Resolve each escort's
exact NPC marker and unique Quest destination before geometry publication; reject
missing/wrong/duplicate references and destinations beyond the 512 m preparation
budget. Use the same supported-home and grounded navigation/clearance helper in
the editor worker and authoritative server startup. Distances use capsule feet
consistently. The server still revalidates its startup content independently.

Prepare routes against the exact staged world/navigation before exposing a
population package. Failure removes staging and leaves source, prior exports and
Undo history intact. Record checked quest/NPC/destination IDs and derived steering
point counts in the export report. This is validation evidence, not player progress
or a route cache trusted by the runtime. No search or file I/O enters fixed ticks.

## 14 November 2026 — escort navigation, ground collection and optional diagnostics

P3-11/12/15 / P2-14 / P5-17. The user chose the escort as the destination-only
navmesh experiment, and the collection quest as a separate authored test path.
Use existing Landmass navigation and checked terrain/capsule grounding during
startup. An escort authors its home NPC and one Quest destination marker; it does
not author intermediate waypoints. Keep necessary terrain/corner steering points
and the existing dense clearance checks. No path search or collision file loading
enters the fixed simulation step.

Each character has a private escort journey. Elowen travels at the Classic forward run speed, 7 m/s while the
player remains within 10 m, pauses outside that radius, and becomes Ready at the
ancient-tree destination. Only speaking to that exact NPC turns in the quest.
Committed Ready/Completed journals restore the NPC at the destination; an
interrupted Active journey restarts from home on rejoin. Cooperative shared
escort groups and saving partial route progress remain outside this slice.

The reported compass-test whole-character stutter was traced to a one-tick render
tween restarted on each prediction update. At render rates below simulation rate,
frames alternated between one and two ticks of movement. Render the last two
reconciled positions using the continuous prediction-clock fraction instead;
retain one tick of presentation delay and never extrapolate beyond collision.
This is a presentation correction, not a change to authoritative movement or
Classic speeds. Automatic escort qualification holds ordinary forward input
continuously while following, with a small stop/resume distance band near the NPC.

Ground rewards use schema 5: one named position/area, one consuming possession
quest and one item. Existing async reward claims and bag/journal transactions own
availability, pickup and consumption. The runtime checks life, current position
and three-metre interaction range. The once-only delivery item cannot be destroyed
through the client inventory endpoint. The authored collection path has two
points; AI test travel derives grounded steering from those points.

The user confirmed 3440x1440 solo and 1920x1440 per player for two-player tests.
Preserve resolution when arranging multiple windows; overlap when needed rather
than shrinking each window below 1920 wide. Explicit smaller diagnostic windows
remain available for the existing responsive-layout qualifications.

Use standard Bevy UI for NPC names/roles, bag/vendor coin rows and slot-anchored,
viewport-clamped item tooltips. Tab and Shift+Tab remain usable while the backpack
or conversation is open. Enemy health bars require explicit selection or observed
attacks against the local player; a three-second simulation-time grace bridges
swing cooldowns, and death/out-of-combat clears it. This is presentation policy,
not an authoritative threat-table display.

Debug statistics are hidden by default and toggled through Interface [F10]. Save
the toggle alongside local UI preferences when a preference file is configured.
Display measured FPS/average frame time, window dimensions and observed network/
world stats. Hide the normal connected label; retain loading/disconnection text.

## 14 November 2026 — starter coins, vendor recovery and fixed HUD

User requests fixed visible minimap/objectives when the bag opens, automatic bag
opening only at an NPC selling items, persistent gold/silver/copper, additional
starter contacts, replacement sword/wand sales, player item sales and unlimited
vendor stock/cash. They also request a second NPC introduction quest and smoother
AI test travel. Work remains local.

P2-14 / P3-05 / P3-11 / P3-12 / P4-03 / P5-17: keep map/tracker anchors fixed;
place the bag in the adjacent column. Use a schema-3 item/bag copper ledger in the
existing character backpack transaction, not three floating-point balances or a
client-authoritative wallet. 100 copper = 1 silver, 100 silver = 1 gold. Initial
52,550 copper (5g 25s 50c) is a once-only new-character seed in the opt-in starter
profile. Existing persisted rows are never reseeded. These amounts/prices are
Theia starter tuning, not claims of historical Classic prices or account-wide
shared banking. Legacy item/bag schemas and barter content remain available.

Schema-2 vendor recipes exchange coins for items or backpack items for coins.
One click exchanges the displayed authored quantity. Equipped items must first
be unequipped; destruction pays nothing. Vendor stock and funds are unlimited:
only the player's balance, capacity, exact NPC/session and revision constrain a
trade. Keep cost, output, money and receipt in one async durable commit before
acknowledgement; retain exact-retry replay and stale/conflicting refusal.

Keep dense navigation/grounding checks, simplify redundant steering points within
3 cm in full 3D, then recheck every longer segment against the same navigation and
capsule clearance. Retain the original route if that check fails. Never force five
points through a bend, wall or floor change. Preparation remains outside fixed
simulation. The diagnostic AI player still sends ordinary movement input.

## 14 November 2026 — exact multiple-contact runtime and editor adoption

Continue the authorized P3-11 / P3-12 / P3-15 / P5-17 slice through actual native
play. Keep a separate opt-in valley manifest and retain the qualified single-guide
path. Bind all schema-3 anchor bytes and sorted placements to world identity;
validate the entire placement/catalogue bijection before allocating any entity.
Schema 3 permits 1..16 contacts in 16 KiB; legacy schemas retain 4 KiB limits.

DG02 carries at most sixteen 41-byte entries (catalogue index, incarnation, f64 XYZ),
709 bytes total. Publish presence before an open view on the existing reliable
channel. The client validates stable NPC/entity/position bindings, unique entities,
sorted indices, actor/profile identity and the exact node owner. DG01 stays the
single-profile compatibility path. This adds no simulation I/O or new transport.

Quest request content NPC must equal the definition of the actual open entity;
never infer it from proximity. Wrong contact/session/range/incarnation refuses
before consuming command sequence or queue capacity. Vendor permission additionally
checks its declared NPC. One contact's retirement closes only its own sessions.
Keep the core catalogue loader and legacy profile loader separate; application
entry points explicitly dispatch the validated content schema.

Use the existing editor NPC Catalog ID field and checked export. Render bounded
standard Bevy contact meshes and per-contact quest/map markers. The new report
quest accepts at the guide, turns in at the steward and awards one token through
the existing durable fixed-item transaction. Do not change combat profiles or
starting inventory to manufacture that proof. Native completion, purchase and
restart are recorded in STATUS. General NPC art/animation and moving contacts
remain separate slices.

## 14 November 2026 — playtest readability and idle population movement

The user reports overlapping pursuing enemies, absent idle wandering, confusing
quest UI and a flickering road, and asks for loot particles and use of the local
Synty UI. They authorize continuing and native checks while away. Address these
under P2-13/14, P3-07/09/11 and P5-17. Existing aggro/return and separate threat/
first-tag ownership remain the baseline.

Use soft authoritative NPC avoidance, never player/NPC physical collision. Keep
spawn spacing separate from a per-group roaming radius. Legacy groups default to
zero roaming; newly placed groups default to 4 m. The local valley launcher opts
its private draft into 4 m roaming. Radius 0..16 m, 1.5 m/s walking, 2..6 s pauses,
up to four checked excursions per home, 1.6 m preferred combat spacing and 1 m/s maximum
avoidance are explicit Theia readability policies, not evidenced Classic formulas.
Navigation/collision preparation stays outside the fixed step. An impossible
roaming request refuses; combat interrupts travel and normal return restarts the
checked excursion from home, without skipping its waypoints. Deliberate singleton patrol paths remain available; this
does not implement individually authored patrol assignments for grouped enemies.

Loot feedback follows local committed corpse availability, including partial
pickup and refusal. Reference screenshots inform layout only; use the user's
local licensed Synty icons/fonts and standard Bevy UI. Fix coplanar road rendering
in shared editor/export geometry without changing saved heights or collision.

## 13 November 2026 — spawn groups share ordinary NPC gameplay

The user clarifies that group spawning should only create independent enemies at
valid placements, with the same attack, loot and gameplay as individually placed
NPCs. Treat the current population restrictions as migration gaps, not separate
rules or a distinct game mode. Reuse the existing per-creature combat, credit,
loot, quest and AI systems; group policy controls placement/count/respawn only.
Close and verify each integration gap before enabling it in normal/editor launches.
Do not introduce alternate population damage, loot or player-facing rules.

## 13 November 2026 — authored respawn timing and protocol bounds

The already adopted spawn-zone authoring range is 1..86400 seconds. Extend the
underlying combat configuration ceiling to 86400000 ms, retaining its existing
100 ms development/test minimum and 5000 ms default. Use a separate bounded wire
respawn deadline; do not widen cast, queue, GCD or ability cooldown limits. Values
are Theia authoring policy in simulation seconds, not evidenced Classic formulas.
Existing group export remains blocked pending complete gameplay/model binding.

## 12 November 2026 — NPC quest inspection and authoring source

The user reports missing quest information when selecting the village NPC and asks
whether quests should be separate files, NPC quest groups or database records,
then delegates the design. Retain the existing validated quest catalogue as the
source of definitions: stable quest IDs, prerequisite quests, giver/turn-in NPC
IDs, stages/objectives and rewards. Derive the NPC inspector's list from these
links; do not duplicate a second quest list or encode requirements in filenames.
PostgreSQL remains responsible for per-character quest state and durable awards.
Initial P5-17 / P5-18 scope is read-only content inspection, with an explicit
validated gameplay-template selection and expandable quest details. This neither
adds live player eligibility to the editor nor changes the one-contact runtime
limit. Full quest authoring and general multi-NPC runtime remain separate slices.
The loaded template is session context and an export default, not an implicit
zone save or a live database editor. Invalid loads preserve the current context.

## 12 November 2026 — custom-bound NPC spawn groups requested

The user requests an NPC equivalent of the tree group, with custom editable bounds,
a monster type/model, target number to spawn and respawn delay. Treat this as a
separate spawn-group authoring feature, not decorative scattering or trusted client
spawns. Recommended boundary is an editable polygon with explicit vertical limits.
Creature definitions bind gameplay and presentation; the server owns population,
selection of valid spawn locations, fresh identities and respawn timing. Validate
candidate points against matching navigation, support/slope and world collision;
exclude unsuitable geometry/layers and report insufficient capacity rather than
forcing placements. Keep spawn bounds distinct from roaming/chase limits.
This requires extending the current single-creature runtime as well as editor UI.
First finish the patrol-export connection and show the user the current editor;
then implement bounded authoring/validation and actual multi-creature gameplay in
small tested slices. The spawn-group feature is requested, not yet implemented.

## 12 November 2026 — proposed editor network-test players

The user proposes exposing test player pawns with attachable/detachable AI
controllers and split camera views to exercise netcode. Recommended follow-on:
two independent authenticated clients, each using the normal input/ability path;
bounded repeatable scenario data first, separate from server-controlled NPC AI.
The user also proposes a per-character network latency property. Apply optional
delay/jitter/loss/outage to that connection's real traffic, in both directions;
show one-way versus added RTT and keep measured latency separate. Default to no
added impairment, with repeatable scenario seeds/presets. Never implement lag by
delaying only the visible pawn or slowing the authoritative simulation tick.
Detach returns manual control. Server authority, prediction and readiness remain
unchanged. Start with movement and side-by-side views, then combat scenarios and
controlled transport impairment. Reuse actual multi-client Renet qualification
instead of treating direct editor/offline pawn movement as network evidence.
Network Playtest should launch one authoritative server and one independent client
per enabled authored Test Player pawn, initially capped at two. Pawns hold spawn,
controller and per-connection impairment settings. Keep ordinary F5 offline.
Start/Stop manages the test processes and separate identities; separate windows
can precede an embedded split-view dashboard. This is a proposed P5-17 testing
workflow, not a decision to add an unrestricted scripting engine. The subsequent
P5-17 / P1-06 bounded implementation and its exact qualification are recorded at
the top of STATUS.md and in ZONE_EDITOR.md; embedded camera views and scripted
combat remain future work.

## 12 November 2026 — Bevy editor design inspiration

The user approves the supplied Bevy Editor Figma/SVG as inspiration where useful,
not a wholesale replacement or pixel-perfect target. Retain our left-hand stacked
explorer/properties, Sprinkles inline edits/popovers, immediate application and
explicit Save. Prioritize a separate asset browser for available models/effects,
clear scene hierarchy, compact transforms, restrained single borders and consistent
spacing. Model thumbnails and drag-to-place are implementation goals, not features
proved by the static design. Keep gameplay-facing sections rather than exposing
generic engine internals. Reference: https://www.figma.com/design/fkYfFPSBgnGkhbQd3HOMsL/Bevy-Editor?node-id=39198-9068
The supplied SVG was inspected locally; upstream implementation/reuse availability
was not verified. Current permission allows mouse/display if needed.

## 12 November 2026 — authored worlds through existing prediction

The quest runtime currently depends on procedural/saved-terrain patch preparation;
a validated monolithic editor export alone cannot run that gameplay loop. Bridge
bounded authored worlds through the existing `Source`/`Patch` worker and Classic
prediction pipeline before presenting NPC markers as playable quest contacts.
Preserve exact static collision, immutable standard GLB bytes and original global
origin. Bind the origin companion and package identity into admission identity.
Do not silently resample the editor's 2 m height grid into the saved-terrain 4 m
grid, replace scenery materials, or change original authoring coordinates.
The first source retains the existing continental domain and explicitly rejects
unsupported presentation dependencies. Client readiness must still wait for actual
scene/dependency/GPU preparation; CPU admission is not native rendering evidence.

## 12 November 2026 — curated Synty scenery collision

Reuse the locally staged Enchanted Forest standard-material assets in the
integrated editor. Keep immutable hash-checked visual bytes separate from an
explicit collision binding. For the starter selection, archway/ruins/table/chair/
log use source triangle surfaces; lantern and pebble clusters are nonblocking
decoration. This is a Theia static-surface adaptation, not imported Unity collider
semantics or a claim that arbitrary source render meshes are production collision.
Do not replace an arch with a solid envelope. Stage bounded geometry separately,
validate exact bytes, reject unresolved collision policy, and qualify Avian capsule
passage/contact before admitting offline playtest. Navigation, full footprint
fitting, gameplay hit shapes and native appearance remain separate requirements.

## 12 November 2026 — complete starter-zone authoring milestone

The user directs autonomous local development toward a complete WoW-style starter
zone designed in the integrated editor: authored terrain and static boundaries
with one intended exit, placed NPCs with quest assignments, quest item pickups,
quest areas and existing minimap integration. Reuse the authoritative gameplay
catalogs and validation; authoring annotations do not become gameplay implicitly.
Keep explicit Save and shared undo/redo. Continue across local commits, without
mouse or display access until the user changes that constraint. P3-17/18 remain
unspecified.

Finish available headless particle checks, record display-dependent and GPU-soak
limits, and move to P5-17 / P3-15 authoring slices supporting P3-11/12 and the
P3-16 starter zone. A single intended exit needs collision/navigation and movement
checks, not just visual cliffs. Start with the existing quest-area runtime format,
then NPC/contact/quest binding and pickups; validate references before export.
A complete playable zone requires server admission and persistent quest/playtest
evidence in addition to editor previews. No completion is claimed by this decision.

## 11 November 2026 — particle placement data boundary (P2-13)

Extend authoring zone schema 8 with bounded particle placements using checked
global f64 anchors, Y-up yaw, uniform scale and the existing stable content ID
rules. The effect ID is a logical unresolved reference, never a machine path;
the headless zone model has no renderer dependency. The anchor boundary does not
validate an effect's visual bounds. Until package support is implemented, export
of a zone containing these placements fails explicitly rather than dropping them.
This data slice does not claim an editor placement workflow, packaged assets,
client effects, server gameplay authority or an MMO VFX budget.

## 10 November 2026 — integrated editor refinement, inline values and split panels

The user reopens P5-17 usability refinement after reviewing the integrated UI.
Adopt inline value editing with immediate application and no separate Apply/edit
panel in the normal workflow. Numeric attributes must be permanent visible
textboxes, directly editable without a click-to-reveal state. A single outer
frame owns background/border; its transparent text area must remain centred
inside the frame in both focused and unfocused states. Valid edits mark the document dirty; the user
explicitly chooses Save for disk writes. Preserve exact f64 coordinates,
validation and undo/redo. Keep the dock on the left, following the user's explicit correction. Stack a persistent object explorer above selected
attributes with a draggable horizontal divider; both panels must be independently
collapsible and resizable. Replace the filter-button grid with a dropdown,
organize attributes under relevant section headings, use three-dot accessors and
Sprinkles popovers where the underlying functionality exists, simplify popup
frames/padding and correct input text alignment/baselines/clipping.

Add a compact full-width bottom status bar for tool hints, selection context,
feedback and save state. Move repeated instructions/selection summaries out of
the explorer/property bodies; retain actual editable attributes and metadata.
Use actual counts rather than invented renderer/memory statistics.

Focus product work and native acceptance on the integrated Sprinkles workspace.
Its runtime/UI library is shared with the retained standalone host, but separate
standalone support/polish is no longer a requirement. Remove the companion launch
option from Theia. Keep changes local and continue across commits until stopped.

## 10 November 2026 — original Sprinkles UI in the shared editor window

Use the retained `ParticleAuthoringPlugin` as an in-window workspace, alongside
Theia's existing authoring tools. Keep mutually exclusive input/update ownership,
explicit zone/source camera identities and isolated preview render layers.
Preserve checked zone data and undo/redo while the original UI edits separate
RON particle assets. Returning restores focus/cursor/root/camera/title and manual
preview pause behavior; retain the standalone host until later workflow slices
replace its role. This implements the user's direct-reuse direction, without
claiming world placement or full gameplay VFX/audio P2-13.
[Implementation and native evidence](SPRINKLES_INTEGRATION.md).

## 10 November 2026 — first direct Sprinkles source host

Reuse the pinned original runtime/editor as local non-published workspace crates,
with Bevy 0.20 adapters and selected MIT code licensing plus separate asset
notices. The first host is a companion authoring window launched from the zone
editor's File menu. Preserve the zone document/history and allow the particle
window to finish saving if its parent closes. Keep standalone data under
`.theia/particles`. This boundary establishes source reuse and native authoring;
it is not same-window hosting or full gameplay VFX. Use standard Bevy line meshes
for preview outlines after measured native gizmo upload synchronization hazards.
[Implementation and evidence](SPRINKLES_INTEGRATION.md).

## 10 November 2026 — accept a close Sprinkles match and begin direct integration

The user accepts a close, usable Sprinkles style/layout/function match without a
pixel-perfect side-by-side review. Mouse, keyboard and display use are renewed.
Finish native functional/GPU checks and fix material issues, then begin Sprinkles
particle/editor integration. Prefer reuse of reviewed upstream runtime and editor
source; adapt where Bevy 0.19/0.20 APIs or Theia's authoring model require it. Keep
Theia's current engine/physics pins and guarded source/history semantics. A first
editor integration slice does not complete authoritative spell VFX/audio P2-13 or
establish MMO-scale particle budgets. Keep changes local; no push is authorized.

## 10 November 2026 — P5-17 shell follows Sprinkles top bar

Continue the adopted UI migration with upstream's 52 px top bar, primary Save
and a named inspector section for Theia viewport preferences. Preserve guarded
commands, exact authoring data, shared viewport coordinate conversion and offline
playtest semantics. This is implemented headlessly; native visual/input/GPU
acceptance waits for renewed desktop authorization. The user still needs the
mouse/display; work continues locally through commits without native input.

## 10 November 2026 — desktop use stopped; continue headlessly

The user needs the mouse and display again. Stop native windows, desktop input
and capture inspection. No native editor/input process remains. Continue P5-17
headlessly across local commits; attribute-scroll native acceptance must wait
for renewed desktop authorization. Completed native evidence remains valid for
its recorded binaries.

## 10 November 2026 — continue P5-17 through commits, desktop use renewed

The user reauthorizes mouse and display use as needed and explicitly requests
continuous progress until stopped, including across local commits. Resume owned
native-window qualification; keep changes local without pushing. Prior desktop
restrictions are historical and do not apply while this authorization remains.

## 10 November 2026 — desktop use stopped again

The user needs the mouse and display. Stop desktop input, capture inspection and
native windows; continue headlessly. No editor/input process remains. Keep
completed native evidence separate from pending preview/playtest checks.

## 10 November 2026 — desktop use renewed for P5-17

The user reauthorizes mouse and display use as needed. Resume guarded native
editor input, screenshot review and GPU qualification. Prior headless-only
restrictions remain historical; do not infer native acceptance from CPU checks.
Keep all changes local and do not push.

## 10 November 2026 — match Sprinkles layout and interaction closely

The user explicitly extends the adopted direction beyond styling: match
Sprinkles as closely as practical in panel arrangement, property/attribute
presentation and editing, menus and visual feature access. The browser,
inspector and viewport structure replaces the old stacked right-hand layout.
Retain the Theia authoring domain and exact validated transactions. Native
acceptance remains pending while mouse and display use are stopped.

## 10 November 2026 — adopted Sprinkles editor UI direction (P5-17)

The user adopts Sprinkles as the concrete look, feel and interaction reference
for the full Theia editor. Adapt its native Bevy UI components where practical;
create matching controls for Theia-specific functions. Keep the existing validated
command/history, exact f64 fields, source schemas, agent interface and offline
playtest underneath the presentation. Continue local implementation and commits
until stopped; mouse, keyboard and display use are authorized. Do not push.

Reviewed upstream: `doceazedo/sprinkles` commit
`0e978f5a9afc6b456b3693c8761e73a6c7e155bd` (Bevy 0.19). Adapt to Theia's
existing Bevy 0.20 pin; no engine downgrade. MIT-selected UI provenance is in
`third_party/sprinkles/`. Particle runtime remains a candidate requiring its own
P2-13 compatibility, rebasing, budget and native GPU qualification.

Initial UI prerequisites are the retained editor command/history, standard Bevy
widgets and existing 3440 x 1440 layout. First slice: shared Sprinkles tokens,
button states and Inter typography across workspaces, palette, outliner,
properties and dialogs. Follow with resizable panels and section/inspector
interaction. Acceptance: formatting, denied-warning Clippy, retained focused
Rust tests, CPU layout and actual Linux 3440 x 1440 input/GPU checks. Planned
migration is not a claim of completed UI or particle adoption.

## 9 November 2026 — retain our editor and adopt application-style controls

After reviewing external editor options, the user chooses continued work on our
editor. Prioritize Blender-like application workflows using standard Bevy UI:
visible workspaces, a command bar, outliner/properties regions and direct sliders,
buttons and toggles. Increasing control size does not address this request.
Keep visual/agent validation and undo underneath the presentation. The first
slice uses built-in Slider/Checkbox widgets for editor-local terrain presets and
moves the exact numeric Transform transaction into a docked properties region.
Source/schema/runtime pins remain unchanged. Names remain a conventional dialog;
full docking, file browsing and viewport transform handles are subsequent work.
[Contract/evidence](ZONE_EDITOR.md#application-style-workspaces-and-properties--9-november-2026).

## 8 November 2026 — editor marker visibility and terrain asset reuse

Keep the spawn marker a standard unlit, noncolliding Bevy primitive, anchored
through checked conversion and hidden during playtest. Cache actual terrain
origin/patch inputs so unrelated document revisions do not replace resident
surface/grid meshes. Native review caught an immediate blank world capture
following an unchanged-terrain rebuild; pixel qualification now verifies the
marker in the rendered world rather than relying on transform metadata alone.
[Contract/evidence](ZONE_EDITOR.md#readable-spawn-marker-and-retained-terrain-meshes--8-november-2026).

## 8 November 2026 — surface-supported playtest spawn shortcut

Use P over ground in Sculpt/Paint to author the existing spawn through SetSpawn,
with 1 m X/Z snapping when enabled, sampled terrain/flat support and the retained
0.1 m clearance. Route the existing view-centre button through the same checked
local/global conversion and surface sampler. Preserve pending strokes, modal/UI/
playtest isolation and the shared command validation/history boundary. Cursor
placement that equals the current spawn skips visual history; the existing button
retains its transaction behavior. Physics still qualifies entry separately.
[Contract/evidence](ZONE_EDITOR.md#surface-supported-playtest-spawn--8-november-2026).

## 8 November 2026 — spatially sampled bounded terrain drags

Connect observed valid terrain cursor positions with stamps every 2 m in X/Z,
using checked existing local/global conversion and unchanged sculpt/paint edits.
Carry the last attempted stamp across events; stationary/subspacing input adds
no work. Bound each event to remaining transaction capacity before allocation.
Leaving the canvas or terrain patch breaks continuity, preventing an unseen
connecting stroke after reentry. Preserve preview validation, the 128 accepted
edit cap, release commit and Escape cancellation. No time-dependent accumulation,
new command/source schema or renderer is introduced.
[Contract/evidence](ZONE_EDITOR.md#spatially-sampled-terrain-drags--8-november-2026).

## 8 November 2026 — terrain-conforming sample influence

Share one bounded per-sample falloff calculation between sculpt, paint and visual
feedback, retaining exact command output and locked borders. Show amber vertex
alpha on the exact standard-Bevy terrain triangles, lifted 4 cm and following the
temporary stroke preview. Reuse one resident mesh and hide invalid/empty targets;
keep feedback outside picking, physics and document history. This shows potential
grid influence, not an analytic radius outline or a prediction of tool-specific
height/quantized-colour changes. No decal infrastructure or schema is introduced.
[Contract/evidence](ZONE_EDITOR.md#terrain-conforming-sample-influence--8-november-2026).

## 8 November 2026 — exact numeric primitive transforms

Use built-in Bevy text inputs and the modal TabNavigation API for seven global
position/yaw/size fields. Keep f64 round-trip strings and parse finite numbers;
submit all fields in one existing Transform command. Reuse full-candidate shape,
coordinate, revision and undo validation. Preserve invalid text for correction,
isolate world/mailbox input during either dialog and avoid global tab handlers
that would capture the editor's existing object/guide cycling. No schema or
coordinate-conversion path is added; numeric fields enter global truth directly.
Both forms clear the previous attempt's status on actual text edits and restore
pending instructions; only Apply judges the candidate.
[Contract/evidence](ZONE_EDITOR.md#numeric-primitive-transform-fields--8-november-2026).

## 8 November 2026 — shared names and standard Bevy text input

Use narrow `rename` / `rename_zone` edits with existing candidate validation and
transactional undo. Keep IDs, geometry and annotation definitions unchanged.
Use the pinned Bevy TextInput/EditableText infrastructure in a bounded modal;
retain its prepared font/layout styles when resetting field text. Isolate world
input and mailbox mutations during a name draft; invalid submissions retain the
buffer for correction, unchanged visual submissions add no history. No schema,
text-editing framework, engine/plugin pin or chat composer is introduced.
[Contract/evidence](ZONE_EDITOR.md#shared-names-and-standard-bevy-text-input--8-november-2026).

## 8 November 2026 — adjustable sculpt and paint blends

Extend bracket amount controls to Flatten/Smooth and Paint, with independent
editor-local blend presets bounded to the retained domain's 0.05..1 range.
Preserve 0.5 sculpt / 1.0 paint defaults and existing raise/lower metre controls.
Display exact percentages and pass presets through the same validated stroke
commands/undo. Keep terrain colour weights normalized and collision independent.
[Contract/evidence](ZONE_EDITOR.md#adjustable-sculpt-and-paint-blends--8-november-2026).

## 8 November 2026 — bounded standard-Bevy content list on the ultrawide editor

Use a fixed 40-row primary-pointer UI pool and paging for named authored content,
reusing the existing mode transitions and frame-selection path. Keep browsing
outside document history, refresh it from validated revisions, reveal selections
and clamp shrinking pages. Exclude both side panels from world mouse actions and
hide the content list during playtest. Preserve pending drafts on row selection
attempts. No hierarchy, scene schema or custom renderer is added.
[Contract/evidence](ZONE_EDITOR.md#paged-content-selection-at-3440x1440--8-november-2026).

## 8 November 2026 — structural guide-point edits with shared validation

Add narrow insert/remove point edits with explicit guide/index/global position,
retaining the existing 2..64 points and spacing/span/document bounds. Insert
subdivides the selected or first segment visually; Backspace removes the selected
committed point while preserving draft-pop behavior. Clear stale point selection
when structural counts change, including agent edits/undo, then let successful
visual edits select their new/next point. Use a shared opaque cyan point material
to distinguish the selected handle from an amber ribbon.
[Contract/evidence](ZONE_EDITOR.md#inserting-and-removing-guide-points--8-november-2026).

## 8 November 2026 — original replayable example and whole-terrain framing

Provide an original primitive-only Wayfarer Hollow base/command recipe and an
explicit launcher `--example` path. Seed new drafts in memory with a separate
save destination; reopen existing edited drafts without reseeding or replacing
source. Demonstrate existing terrain, grading, painting, props, vegetation,
notes, collision and export together. F in Sculpt/Paint frames terrain through
the same checked normal Bevy camera path. Keep source annotations/stream guides
separate from live NPC/quest/water behavior and leave default game content alone.
[Contract/evidence](ZONE_EDITOR.md#wayfarer-hollow-example-and-terrain-framing--8-november-2026).

## 8 November 2026 — adjustable bounded raise/lower strength

Expose the retained brush domain's 0.05..8 m stamp amount with bracket keys to
halve/double it, retaining the 2 m default. Display the actual amount on both
buttons and keep this editor preset separate from saved terrain. Reuse the
existing raise/lower command, validated previews and one-entry stroke history.
[Contract/evidence](ZONE_EDITOR.md#adjustable-raise-lower-strength--8-november-2026).

## 8 November 2026 — bounded duplicate command for greyboxing

Copy authored objects, plants, markers and whole guides through one shared
`duplicate` edit with explicit source/new IDs and f64 offset. Preserve metadata,
geometry and schema; append a UTF-8-bounded copy suffix and revalidate the full
candidate against existing ID, document and per-kind budgets. Ctrl-D uses the
monotonic shared allocator, offsets one grid step in X/Z and selects the new
copy. Keep draft/preview/playtest refusals and one-command undo semantics.
[Contract/evidence](ZONE_EDITOR.md#shared-duplication-for-greyboxing--8-november-2026).

## 8 November 2026 — sampled and adjustable flatten level

Expose the existing validated flatten brush's nonzero level through I surface
sampling and PageUp/Down adjustments. Use checked domain surface sampling from
cursor X/Z, local height relative to zone-origin Y, bounded ±64 m, 1 m steps or
0.1 m with snap off. Keep the preset editor-local; commit strokes through the
retained brush command, candidate validation and history. Paint mode ignores the
height sampler. [Contract/evidence](ZONE_EDITOR.md#sampled-and-adjustable-flatten-level--8-november-2026).

## 8 November 2026 — keyboard recovery and bounded selection framing

Cycle object selection with Tab/Shift-Tab in source order, independent of mesh
visibility, and extend F framing to objects, plants and markers. Keep existing
guide-point/whole-guide controls and document history separate from camera state.
Extend the bounded editor orbit range from 240 m to 512 m to frame the largest
permitted primitive. Reuse checked global/local conversion and standard Bevy
camera transforms. [Contract/evidence](ZONE_EDITOR.md#keyboard-selection-and-placement-framing--8-november-2026).

## 8 November 2026 — saved content comparison in the editor summary

Track the selected destination's last loaded/saved document independently of
history revision. Show unsaved state for differing content and pending drafts;
restore clean state on exact undo, successful Save/Load or draft cancellation.
Keep Save failure and accepted agent edits distinct from durable file success.
Use standard Bevy text; retain explicit Save and existing formats.
[Contract/evidence](ZONE_EDITOR.md#saved-and-unsaved-authoring-state--8-november-2026).

## 8 November 2026 — explicit terrain grounding through shared history

Keep saved placement/guide heights stable during sculpting. Add explicit terrain
projection for typed placement IDs and guide-point indices, preserving X/Z and
accounting for primitive centre versus foot anchors. Reuse exact terrain queries,
whole-candidate validation and bounded transaction/history rules; a whole guide
is one undo. Keep prop stacking, full footprint fitting and runtime collision
independent. [Contract/evidence](ZONE_EDITOR.md#explicit-terrain-grounding--8-november-2026).

## 8 November 2026 — optional finite 2 m terrain authoring grid

Zone schema 7 adds a typed 2 m / 4 m sample resolution on the existing 128 m
patch. Resample heights over the retained exact triangles and colour weights
with deterministic normalized integer rounding. Keep sculpt, paint, grading,
render/query and export on the same active grid. Preserve coarse samples on
refinement and use full-document undo for coarsening's loss of intervening fine
detail. Extend only the shared bounded generator's allowed cell counts; retain
normal game procedural/saved-terrain 8/32-cell policies and existing runtime
package format. No streaming, LOD or capacity claim follows from 4,225 vertices.
[Contract/evidence](ZONE_EDITOR.md#optional-2-m-terrain-samples--8-november-2026).

## 8 November 2026 — one-command editor entry with nonwriting save-path selection

Provide a small Python launcher for the locked editor build and default draft
reopen. Reuse the existing Linux SDK when present and the client environment
helper; preserve explicit editor/caller paths. Add `--save-path` to select the
later GUI save destination without a startup write. Keep `--output` semantics
for validated headless pipelines. Invalid source never falls back to replacing
it with a blank draft. [Contract/evidence](ZONE_EDITOR.md#one-command-editor-launcher-and-deferred-save-destination--8-november-2026).

## 8 November 2026 — normalized terrain colour weights via standard vertex colours

Zone schema 6 adds optional grass/dirt/rock/sand u8 weights per terrain vertex,
normalized to 255 and grass-locked on the border. Share sculpt's validated stroke
preview/history for visual and agent paint commands. Render linear blended colour
through normal Bevy vertex colours/white PBR and export standard glTF COLOR_0 with
bound identity. Collision, heights and gameplay surface rules stay independent.
Retain the finite patch/byte budgets and source compatibility. Texture/material
catalogues, biomes and LOD require further scoped work.
[Contract/evidence](ZONE_EDITOR.md#terrain-colour-painting--8-november-2026).

## 8 November 2026 — individual guide-point editing with standard Bevy handles

Add narrow `move_path_point` edits to the existing document/undo validator. Select
ribbons and point handles using normal mesh ray casts; pool at most 64 handles
on one mesh. Preserve whole-guide moves and cycle point selection with Tab in
both directions. Guide-only F focus uses bounded local coordinates and the full
authoring extent. Keep handles out of playtest collision and exported content.
A point move validates its adjacent segments and entire candidate before commit.
[Contract/evidence](ZONE_EDITOR.md#individual-guide-point-editing--8-november-2026).

## 8 November 2026 — grade existing terrain from committed guide heights

Add a bounded `grade_path` operation to the existing visual/agent command history.
Use nearest horizontal segments, explicit authored guide heights, width, surface
offset and linear falloff/blend; preserve the locked zero border and checked
local height budget. Keep source guide and other placements unchanged. Reuse the
resulting heightfield for normal render, offline Avian and world-package export.
The visual default grades 0.03 m below the selected ribbon with 4 m falloff. Drafts
refuse without being discarded. Partial/falloff repetition is a new blend; exact
mailbox retries remain cached. No new terrain/water/navigation runtime format.
[Contract/evidence](ZONE_EDITOR.md#guide-driven-terrain-grading--8-november-2026).

## 8 November 2026 — bounded seeded greybox vegetation

Zone schema 5 adds finite tree/shrub visual proxies and deterministic bounded
scatter stamps. Share authoring IDs, checked global/local conversion, exact
terrain support and atomic history with other editor commands. Use standard
Bevy geometry/materials and the existing GLB exporter, with inverse-transpose
normal transformation for ellipsoid foliage. Keep vegetation nonblocking in this
explicit greybox slice; collision/navigation/gameplay and production asset/LOD
policies require separate implemented contracts. The 512-plant authoring cap is
not evidence of population or MMO-scale capacity. Qualify the editor only at the
user's 3440x1440 viewport. [Contract/evidence](ZONE_EDITOR.md#greybox-vegetation-placementscatter--8-november-2026).

## 8 November 2026 — NPC/quest markers remain explicit authoring annotations

Extend the bounded zone document to schema 4 with shared-ID NPC/quest markers,
checked placement/footprints and optional syntactically validated catalogue keys.
Use normal Bevy badges/footprints for visual editing and the same atomic command
history for agents. Hide annotations during playtest. Preserve them only as source
metadata in exports; runtime geometry/identity stay unchanged until an explicit
compiler validates and binds NPC/quest references to authoritative content.
See [the marker contract and evidence](ZONE_EDITOR.md#npcquest-authoring-markers--8-november-2026).

## 8 November 2026 — 3440x1440 editor and bounded road/stream guides

The user restricts tool development to 3440x1440. Open the editor through standard
Bevy primary-monitor borderless fullscreen and qualify only that editor viewport
from now on; retain older evidence as historical. Keep normal client display
support separate. Schema-3 optional road/stream guides share IDs, validation,
undo and save/load with primitive/terrain edits. Export standard render-only
ribbons through the existing world package; terrain grading, water simulation,
navigation and collision remain separate requirements. Explicit finite limits
and the combined serialized byte budget apply before state/history mutation.
See [the current contract and evidence](ZONE_EDITOR.md#roadstream-guides-and-3440x1440-tool--8-november-2026).

## 8 November 2026 — authored zones export through the existing world pipeline

Export standard Bevy meshes as self-contained GLB and existing schema-4 collision
packages with explicit validated spawns and bound identity. Keep boxes analytic;
encode spheres as existing zero-length capsules and cylinders as adaptively
bounded convex meshes, welding render seams only in the collision copy. Rebase
through checked local conversion; preserve the exact authoring document/origin
separately. Refuse oversized/invalid/occupied outputs and publish the manifest
last in a fresh directory. No new runtime format, transport or default-map change.
Actual headless validation, independent convex/glTF checks and native encrypted
pairs pass, including direct hillside and far-instance traversal. This is the
retained raw imported-world development profile; full Classic account/gameplay
bundle integration and richer zone content remain separate.
[Contract and evidence](ZONE_EDITOR.md#export-into-admitted-world-packages-p5-17--p3-15--p1-10--p2-0708).

## 8 November 2026 — bounded terrain brushes share command validation

Add one schema-2 128 m / 33x33 height patch to the zone editor using integer chunk
IDs, global f64 origins and bounded local f32 sample heights. Keep borders flat
for this first finite patch; use a real cutout in surrounding flat ground.
Raise/lower/flatten/smooth previews validate through disposable command state;
release commits the same bounded stamp transaction as one live undo entry.
Bevy surface/grid meshes and Avian use the same existing regular triangulation.
Keep terrain authoring outside authoritative simulation and preserve schema-1
primitive documents. The actual Avian/ray and three-size native gates pass;
normal server package export, multi-tile seams and richer tools remain open.
[Contract and evidence](ZONE_EDITOR.md#bounded-terrain-brushes-p5-17--p3-15--p1-10--p2-0607).

## 8 November 2026 — live zone commands use an opt-in local mailbox

Use one explicit local directory and one pending request for agent edits in the
open zone editor. Bind requests to a fresh editor session, positive monotonic
request IDs and current revision. Cache the latest exact request/receipt so a
retry cannot reapply after visual undo. Refuse during drag/playtest; publish
current validated document and bounded receipts outside fixed-step simulation.
Receipt success means an editor edit, not a file save or server publication.
No new network endpoint or gameplay authority is added. The shared core and
native mixed-agent walkthrough pass; [contract and evidence](ZONE_EDITOR.md#live-local-agent-mailbox-p5-17--p3-15).

## 8 November 2026 — zone authoring uses one validated command history

Prioritize a lightweight standard Bevy authoring app under P5-17/P3-15. Keep
zone documents/transactions independent of rendering; visual edits and structured
agent commands share validation and undo. Start with primitive placements,
transforms, snapping, bounded history, versioned files and an offline Avian
playtest using the existing capsule solver. Terrain brushes, paths/streams,
vegetation and NPC/quest markers follow; chat composer/bubbles are deferred.
Keep the game stack and normal multiplayer path; no new renderer, ECS or network
transport. Use bounded f64 authoring coordinates and existing local conversion.
Local commits and desktop qualification are authorized; no push.
[Contract, prerequisites and evidence](ZONE_EDITOR.md).

## 8 November 2026 — client chat waits independently of gameplay

Explicitly bind the opt-in chat profile after the full gameplay/Classic identity.
Publish bounded receipt state through shared immutable inbox views and preserve it
across snapshots. Permit one outstanding request; a five-second local timeout
discards queued text and marks the chat view without disconnecting gameplay.
Only a matching reply clears pending state; late replies cannot clear newer work.
Keep the normal client opt-out until the composer and native/account acceptance
pass. Stop at this completed worker checkpoint per the latest user request.
[Evidence](LOCAL_CHAT.md#real-client-worker-integration--8-november-2026).

## 8 November 2026 — world chat serial bounds long-lived client replay state

Assign a checked monotonically increasing server delivery serial across speakers
and channels, distinct from client request sequences. Keep64 received messages
and one private reply; no per-speaker replay map grows across join/leave churn.
Validate context before ignoring stale data; conflicting latest serials and
regressing chat time fail before mutation. CD02/v3 is opt-in only; normal
protocol13 remains unchanged. [Evidence](LOCAL_CHAT.md#bounded-client-inbox-and-world-delivery-serial--8-november-2026).

## 8 November 2026 — bounded private chat replies reserve gameplay capacity

Reply to the admitted sender with an exact owner/world-scoped outcome; queue at
most four replies per peer and flush one alongside one chat delivery after
priority publishing. Reserve half the shared reliable queue against chat use,
since future gameplay arrives after the current publishing pass. Drop new replies
under pressure with counters; a future one-pending composer must handle timeout.
The explicit chat compatibility profile advances to v2; normal protocol13 remains
unchanged. [Evidence](LOCAL_CHAT.md#private-replies-and-gameplay-queue-reserve--8-november-2026).

## 8 November 2026 — opt-in server chat compatibility and priority

Bind the initial one-area Say/Area mode into the complete world identity without
changing default protocol13 worlds. Require named authorized leases; synchronize
live leases and checked poses in the I/O owner loop and purge retired recipients.
Flush at most one bounded chat frame per recipient after gameplay-priority
publishing; never await transport or persistence from the simulation step.
Retain existing input limits and refuse forged/disabled uploads. Raw encrypted
fixtures qualify this candidate; normal client chat and explicit submission
replies remain separate work. The user has restored desktop access as needed.
[Scope and evidence](LOCAL_CHAT.md#opt-in-encrypted-owner-loop-candidate--8-november-2026).

## 8 November 2026 — bounded Say/Area chat prerequisite

Use server-selected names, authenticated actor incarnations and current checked
global positions. Client chat intents contain only sequence/channel/text. Start
with Say at25 metres in3-D and server-authored Area membership, with a shared
three-message/five-second budget. Those values are provisional Theia extensions.
Keep plain text at256 UTF-8 bytes; reject controls and direction spoofing marks.
Muted/rate-limited/no-area requests consume their sequence without spending
accepted capacity. Persisted/account moderation, runtime routing and native
composer/bubbles remain separate acceptance work. [Scope](LOCAL_CHAT.md).

## 8 November 2026 — action-bar bindings are local presentation state

Use twelve bounded optional ability IDs with current defaults; book drops copy,
bar drops swap. Keep cooldowns, eligibility, target and facing in the existing
authoritative cast path. Editing requires the open book and current focused,
admitted actor context; stale or cancelled drags make no binding change.
Persisted layouts and learned/class eligibility require separate work. Continue
headlessly under the latest user restriction; synthetic event/loopback acceptance
does not qualify native drag behavior. [Scope](ABILITY_BOOK_ACTION_BAR.md#action-bar-binding-candidate--8-november-2026).

## 8 November 2026 — selected character names are public scoped metadata

Capture the saved SQL-selected name in the authorized admission lease; retain
opaque nonce-only ticket userdata. Use bounded reliable PN01 metadata for the
same visible-player roster as movement, with no ownership or account identifiers.
Defer under queue pressure and allow only one changed view per completed tick.
Clients validate admitted owner/world/epoch and monotonic updates; UI additionally
requires current snapshot membership. Names never establish authority. Protocol13
is required for the integrated path; paired protocol12 playtest packs remain
preserved. [Scope and qualification](PLAYER_DISPLAY_IDENTITY.md).

## 7 November 2026 — shared player display-name validation

Reuse the existing temporary 3–20 ASCII player-name policy as a pure core value,
with the persistence re-export and database error mapping retained. Names remain
display/lookup data; authenticated bindings establish ownership. Future gameplay
metadata must carry the server-selected saved character name, scoped to the
admitted actor/world, with no local launcher-name authority or account identifiers.
NPC authored names retain their separate policy. Parser extraction and one actual
SQL character contract pass; the standalone bounded codec is now qualified,
while runtime replication remains planned and protocol12
remains active. [Work and acceptance](PLAYER_DISPLAY_IDENTITY.md).

## 2026-11-07 — focus gates fresh input, not accepted work

Require one focused game Window for new ability/world/backpack/loot/reader/
Interface input. Discard unsubmitted local choices on focus loss and do not
replay them on return. Preserve persistent targets and autoattack, accepted
transaction/reply processing, matching read completions and prior Loot All intent.
Do not pause authoritative state receipt or reset gameplay just because focus
changes. These current guards have headless regression coverage; actual native
focus/event ordering remains pending while the user forbids desktop/input/display.

## 2026-11-07 — bounded received combat history in the opt-in HUD

Use ordinary Bevy UI and a six-message local history to make received owner combat
outcomes readable in the bottom-left reference area. Filter unrelated actors,
suppress duplicate/older batches and reuse existing own-life validation. Show
server-resolved damage and reported overkill without recalculating a hit or
calling resolved damage health lost. Keep chat and social systems separate.
This presentation remains a candidate until native layout and event-flow checks
are permitted; no display/input testing is authorized now.

## 2026-11-07 — explicit local interface preference persistence

Extend the session-only scale panel with an optional `--ui-preferences FILE`.
Keep normal launches without this argument session-only. Validate schema 1,
finite 75–150% scale and a 4 KiB bound before creating the App. Reject malformed
files without rewriting them. An explicit launch scale overrides the saved value;
opening the client alone never saves that override. Save actual preferred-size
changes through one bounded Bevy I/O task at a time, replacing a complete temporary
file; fitted viewport size is never persisted. Report save failure while retaining
session behavior. This is a device-local preference, not account persistence.
Desktop/keyboard/display checks are forbidden by the current user instruction.

## 2026-11-06 — explicit bounded local playtest session lifetime

Keep the existing five-minute default. Permit an explicit server session policy
of 60 seconds to 12 hours, always capped by signed identity expiry and retaining
immediate revocation. Use a one-hour policy only for an intentionally configured
local playtest issuer/session. Keep automatic renewal separate; don't bypass
expiry in the game client. [Implementation and checks](AUTHENTICATED_ADMISSION.md).

## 2026-11-06 — session-only interface size controls

Use ordinary Bevy buttons to adjust the existing preferred UI scale in game.
Expose 75/100/125/150% and both preferred/fitted sizes in Interface [F10]. Preserve
current small-window fitting, Classic controls and the existing panel/target
Escape chain by giving Interface first close priority. Keep the setting local to
the current session until a separate saved-preference slice; opening it does not
pause an authoritative world. [Implementation and evidence](UI_FANTASY_WARRIOR.md#in-game-session-ui-scale--6-november-2026).

## 2026-11-06 — additional UI references refine panels and character screens

Keep the adopted lower-HUD composition. Use the user's four new inline images
for parchment quest/dialogue bodies, dark portrait headers and icon-grid bags,
framed equipment/service windows, and future character-selection presentation.
They guide appearance; visible repair, currency, faction/class or chat controls
need their own working systems before being exposed. Assess licensed local art
and portrait sourcing rather than treating reference pictures as runtime assets.
[Reference mapping and immediate work](UI_FANTASY_WARRIOR.md#additional-panel-and-character-screen-references--6-november-2026).

## 2026-11-06 — AntonilzUI screenshot guides the general HUD

Adopt the user's supplied WoW Classic addon screenshot as the visual composition
reference: compact lower-centre player/target frames, bottom actions/cast/XP,
upper-right circular minimap and objectives, readable world nameplates and an
unobstructed centre. Use restrained local Synty art selectively. This refines
UI-001 presentation; it does not change strict Classic controls or gameplay
truth. The current opt-in skin is an earlier draft, not an implementation of the
new reference. [Reference, exact gaps and next acceptance checks](UI_FANTASY_WARRIOR.md#adopted-visual-reference--6-november-2026).

## 2026-11-06 — preferred UI scale with bounded layout fitting

Use ordinary Bevy `UiScale` for the opt-in fantasy skin. Accept a 75–150% client
preference, default 100%; fit down when the current window cannot retain the
1280×720 logical layout and restore the preference as space returns. Keep world
coordinates, camera and window dimensions independent. Windowless physical-pixel
exports and a headless Window-component test establish rendering/fit behavior;
live pointer, native resize, DPI and complete-pane checks remain pending while
the user uses the desktop. [Implementation and exact scope](UI_FANTASY_WARRIOR.md#preferred-ui-scale-and-resize-fitting).

## 2026-11-06 — general UI with local Fantasy Warrior art

Move beyond quest mechanics to general Classic-style UI, following the user's
new direction. Reuse selected local PNGs and TrueType fonts with ordinary Bevy
widgets and a separate asset source. Rebuild behavior rather than importing
Unity prefabs. Keep the first skin opt-in pending normal connected and input
qualification. Windowless GPU exports may proceed without desktop/mouse use;
they are fixture previews and do not establish live interaction correctness.
[Asset assessment, implementation and acceptance](UI_FANTASY_WARRIOR.md).

## 2026-11-06 — explicit client content configuration report

Expose a developer CLI-only report using the same identity expression as normal
network setup, returning before any network worker, App, window or renderer.
Require exclusive predicted Classic graybox configuration and reject connection
arguments. This permits exact normal client/server configuration comparisons while
desktop testing is unavailable, without adding gameplay authority or duplicating
the identity algorithm. [Headless proof and limits](QUEST_ITEM_REWARDS.md#actual-clientserver-configuration-identities-agree-headlessly).

## 2026-11-06 — opt-in exact fixed reward manifest role

Use cumulative manifest schema 21 with one additional `quest_item_rewards` role
and matching client/server `--quest-item-rewards` paths. Keep schema 20 and all
older roles/hashes unchanged. Bind exact fixed reward bytes before conditional
quest loot, and require the account service's stored/wire journal agreement
before selecting reward ownership. Check prerequisite flags after parsing all
arguments. [Configuration proof and limits](QUEST_ITEM_REWARDS.md#implemented-normal-configuration-prerequisite).
Reward UI and combined native proof remain pending while desktop testing is paused.

## 2026-11-06 — explicit fixed reward World admission

Require explicit reward mode rather than broadening legacy World constructors.
Validate exact inventory/XP, installed quest and reward namespaces, and inactive
exclusive setup before changing World identity. The existing runtime client may
then consume the reward-bound private journal under normal exact context and
pending-command checks. All items, completion and XP still originate in the
retained SQL owner. [Headless World/encrypted proof and limits](QUEST_ITEM_REWARDS.md#implemented-explicit-world-and-encrypted-bridge).
Normal manifest/CLI/UI and combined native qualification remain separate.

## 2026-11-06 — explicit fixed reward owner and capacity refusal

Require identical stored/wire reward-bound journal hashes before installing the
retained inventory/progression owner. Keep ordinary owner constructors closed to
reward-bearing profiles. Treat a full reward bag as a nonfatal inventory-full
journal refusal and append code 10 without changing existing codes/frame sizes.
Accepted SQL work and queued item operations retain the current logout drain;
only committed projections can acknowledge success. [Qualified headless scope](QUEST_ITEM_REWARDS.md#implemented-retained-owner-adoption).
World/client/manifest adoption remains separate, and desktop tests are paused.

## 2026-11-06 — atomic fixed quest reward storage

Adopt the existing quest action API and original receipt for fixed bundles.
Under the existing XP → backpack → journal lock order, use current locked bag
holdings and capacity, then commit consumption, grants, final quest state, XP and
receipt together. The quest command carries its expected quest revision; no new
bag revision is added to the wire intent. Successful retries recover the original
receipt before fresh holdings checks and return current saved bag/journal state.
No migration or parallel inventory API is needed. Store/worker support is now
qualified; keep World, client and retained-owner admission closed until their
separate ordering/ownership proofs pass. [Scope/evidence](QUEST_ITEM_REWARDS.md#implemented-atomic-storeworker-turn-in).

## 2026-11-06 — explicit fixed quest item reward policy

Use a separate bounded fixed-reward catalogue against known quest and item
catalogues. Preserve existing quest schemas and legacy profile hashes. Bind the
exact reward bytes after the item-bound journal/XP/area/document digest; reward-
only quests still bind inventory. The pure proposal consumes delivery items,
checks the whole grant bundle and reconciles other quests against final holdings
with one bag revision. Consume-before-capacity is an explicit Theia policy;
choice rewards, currency, overflow mail and automatic equipment remain separate.
Keep reward-bearing runtime/Store/owner admission closed until durable completion,
items, XP and original receipts commit together. [Scope and evidence](QUEST_ITEM_REWARDS.md).

## 2026-11-05 — original document-open receipts

Reuse existing quest-event receipts with distinct request/command hash domains
for authorized item-document opens. Validate current bag revision, slot and item
under bag-before-journal locks, and seal original active-stage eligibility before
commit. Exact successful retries recover that original authorization without
becoming fresh reads; new requests require current ownership. Keep historical
read credit separate from item possession and allow existing bound bag/journal
transactions to operate without granting read credit. No extra storage migration
or parallel item transaction API is needed. [Implemented headless scope](QUEST_DOCUMENTS.md#implemented-durable-authorized-document-opening).
Retained owner/wire/UI adoption remains the next slice.

## 2026-11-05 — explicit quest-required development loot

Adopt opt-in manifest schema 19 / `--quest-loot` with item-journal and per-life
loot modes. Bind exact item/journal/reward identities before admission; retain
first committed generation rights and separately check fresh current need before
paging or pickup. Do not reclassify ordinary drops as quest-only. Keep the 60% /
25% provisional random content separate from guaranteed low-damage native fixtures.
[Qualified scope and limits](LOOT_TABLES.md#normal-quest-only-loot-qualified--5-november-2026).
Reading a note remains a separate planned historical objective; item possession
alone must not grant read credit.

## 2026-11-05 — first committed quest-loot generation rights

For the initial async server, serialize conditional generation against a locked
backpack/journal pair at its first durable generation transaction. Persist that
eligibility decision, ticket and observed revisions together with any winning
claim. Later acceptance, removal or replay does not replace the original rights.
This is an explicit Theia commit-order policy; it does not claim an exact
simulation death-tick snapshot. The Classic-targeted source stores initial corpse
permissions, but does not define our asynchronous transaction ordering.
[Storage semantics and remaining runtime work](LOOT_TABLES.md#quest-only-generation-storage-prerequisite--5-november-2026).
Preserve current-need checks before paging and pickup as separate requirements;
generation grants availability only. No world-event log or whole-corpse atomic
generation is established by these per-entry transactions.

## 2026-11-05 — explicit normal inventory-bound item quests

Adopt opt-in manifest schema 18 and `--quest-items` only with the exact bound
inventory/journal profiles. Count current backpack holdings across stacks,
reverse Ready after removal, and recheck/consume required items atomically with
turn-in and XP. Keep the three-pelt development quest and guaranteed native
fixture distinct from ordinary random-loot content. [Qualified scope](QUEST_ITEM_OBJECTIVES.md#normal-item-quest-qualification-complete--5-november-2026).
Quest-only generation rights remain a separate prerequisite; ordinary item drops
must not silently acquire quest restrictions. Preserve older profile versions.

## 2026-11-05 — authored creature drop chances and explicit corpse pickup

Use independent integer drop chances and optional exclusive groups in reward
schema 3; preserve old guaranteed recipes. Persist misses and shared group tickets
as well as winning availability, bound to recipient, victim life, content and
original metric position. Use the retained async authority owner; opening a loot
pane grants no inventory. Preserve guaranteed native-test content separately
from provisional 60% pelt / 25% token gameplay content. [Reference, semantics and
adoption limits](LOOT_TABLES.md). Quest-only eligibility and live possession quests
remain gated until their inventory/journal integration is qualified. Standard
Bevy corpse picking/UI now pass the documented Linux native pickup/rejoin flow.

## 2026-11-04 — separate durable loot availability from pickup

A fatal creature life creates a recipient-bound loot entitlement at its original
checked metric corpse position. Availability/viewing do not add inventory or
possession progress. Pickup commits bag, claim receipt and affected journals
together. Keep life receipts durable and load bounded pending pages; consumed
history is not a resident owner list. Preserve once-character content and require
live corpse/life/range checks before runtime adoption. [Storage proof and limits](QUEST_ITEM_OBJECTIVES.md#per-life-loot-storage-prerequisite-qualified--4-november-2026).

## 2026-11-04 — explicit per-creature-life reward scope

Preserve schema-1 once-character recipes. Use schema 2 for explicit repeatable
victim-life entitlements, reusing the authoritative progression KillId type.
Pure selectors establish no kill/recipient authority. Keep runtime and storage
admission closed until durable uniqueness, bounded availability and publication
are installed. [Core evidence and next work](QUEST_ITEM_OBJECTIVES.md#per-life-loot-core-prerequisite--4-november-2026).

## 2026-11-04 — confirmed item destruction native adoption

Retain split/merge support and the exact confirmed backpack slot/revision in
IR02 destruction requests. Require a fresh confirmation after close/reopen or
changed inventory. Preserve server-owned validation and committed views, including
newer bags accompanying matching replies. Normal Linux native destruction and
restart/rejoin now pass; per-life loot and possession-quest admission remain next.
[Qualified scope](QUEST_ITEM_OBJECTIVES.md#confirmed-destruction-ui-qualified--4-november-2026).

## 2026-11-04 — retained item-aware quest owner publication

Initialize inventory before item-aware progression and route supported bag writes
through compound SQL. Publish only committed views through retained character
generations; preserve higher bag/journal/XP versions and command replies across
late responses. Keep live admission closed until remaining loot/UI/native slices
pass. Treat reading a quest note as a separately authored historical condition,
not an implicit consequence of possession. [Evidence and scope](QUEST_ITEM_OBJECTIVES.md).

## 2026-11-03 — atomic inventory-bound quest storage

Bind exact item content into new item-quest profiles. Lock XP (when needed), then
backpack, then journal; commit consumption, completion, XP and receipt together.
Reconcile current holdings on acceptance/load/replay and in compound bag writes.
Retain accepted compound jobs through response loss and shutdown. Keep live
admission closed until all owners and mutation paths use coherent committed views.
[Qualified storage evidence and remaining work](QUEST_ITEM_OBJECTIVES.md).

## 2026-11-03 — versioned final-stage item possession proposals

Add quest schema 3 with explicit current-possession and consume flags, initially
restricted to an all-required final hand-in stage. Preserve earlier historical
facts and completed delivery evidence. Use monotonic mutation revisions for
reversible item quantities while retaining old exact historical-only validation.
Recheck supplied current bags at pure turn-in, prepare consumption and completion
together, and recalculate other quests sharing the materials. Permit codec
readiness reversal only for that defined item scope. Keep live admission closed
until the inventory-aware durable path is installed.
[Qualified core/codec evidence and remaining work](QUEST_ITEM_OBJECTIVES.md).

## 2026-11-03 — qualified reach-area launch and batch snapshot heads

Adopt opt-in manifest 16 and a separate three-quest engineering catalogue,
preserving existing definitions/content and requiring exact area/profile binding.
For multi-goal batches, the committed journal head is a monotonic snapshot
version at least as high as every contained logical revision, not a receipt
count. Share that checked rule between storage and the encrypted fixture.
Actual PostgreSQL overlap/rollback/replay/restart and normal native quest/rejoin
proof pass. Do not migrate existing characters or claim general region support.
[Qualified behavior, native proof and limits](QUEST_REACH_AREAS.md).
Next honor the user's possession-based item delivery request, allowing stack
splits and recalculating readiness after committed item removal.

## 2026-11-03 — bounded typed area observations through retained ownership

Reserve existing character-owner Port capacity before authoritative area
observation. Retry unavailable capacity without claiming credit; suppress
unchanged accepted batches while waiting for commit. Reuse the ordered trusted
producer sequence and retained async owner, leaving client wire actions at
accept/turn-in. Publish only committed journals and preserve command replies.
[Queue/encrypted fixture proof and remaining combined native work](QUEST_REACH_AREAS.md).

## 2026-11-03 — atomic stage-bound area observation receipts

Reuse the existing journal head lock, changed-row writes, receipt table and
bounded critical worker for trusted area batches. Bind exact area bytes and the
original eligible quest/version/stage/area payload in separate hash domains.
Commit all affected quests with their receipt; retain accepted work across
response loss and shutdown. Reject unbound/empty batches and generic raw area
events. Grant no occupancy XP; retain ordinary turn-in transactions.
[Actual SQL/restart proof and remaining live integration](QUEST_REACH_AREAS.md).

## 2026-11-03 — explicit area-aware quest profile and server sampling

Bind exact static area bytes into an opt-in quest/XP profile hash. Keep area-free
hashes unchanged; share reference/namespace checks with persistence and require
World bounds and installed-content checks before admission. Sample only current
alive authoritative capsule feet with prepared collision and client footprint
coverage, never client coordinates. Normal manifests remain unsupported until
their area role and durable owner are connected. [Qualified scope](QUEST_REACH_AREAS.md).

## 2026-11-03 — stage-scoped area objective proposals

Adopt sampled alive authoritative feet occupancy for the forthcoming reach-area
adapter. Add schema-2 selectors and bounded logical observations that seal quest,
definition version and original active stage. Apply each batch once per original
stage; never retroactively credit later acceptance or cascade through stages.
Any/at-least stages retain exactly their completion quorum. The observation is a
pure proposal, not proof of position; World binding and durable publication remain
required. Preserve existing schema-1 content and fail closed at live admission
until that adapter is installed. [Evidence and remaining work](QUEST_REACH_AREAS.md).

## 2026-11-03 — read-only executable quest update review

P3-11 / P3-15: add a bounded advisory comparison of two closed executable
catalogues before content installation/migration work. Preserve exact byte hashes
and stable IDs/versions; classify current schema-1 gameplay versus presentation/
layout changes and flag definition/version issues without copying prose. A
review-only normalized fingerprint never replaces runtime catalogue binding.
Keep saved compatibility, World-reference validation and migration separate;
source files and live behavior stay unchanged. [Workflow](QUEST_PACK_AUTHORING.md).

## 2026-11-03 — normal public visual-life adoption

Adopt explicit schema 15 with the same 22 exact gameplay inputs, deriving AV01
after public NPC timing in World identity. Reuse standard Bevy Death/fresh-life
reset and old sword-cue invalidation for remote actors; retain owner-private
control/vitals barriers. Reset remote pose interpolation only on initial/fresh
public life after an appropriately recent authoritative snapshot. Same-life
movement keeps blending. Actual native finished-corpse/in-view and respawn proof,
dead owner input/cast checks, normal quest/database-restart regression and all
1,123 workspace tests pass. [Scope](PUBLIC_AVATAR_LIFE.md). No new death formula,
corpse-run rules, durable health, normal default/older version replacement or
same-backend-endpoint reconnect qualification. All changes remain local.

## 2026-11-03 — public lifecycle shares bounded priority transport

Share the existing priority public presentation channel for AV01 and NA01,
with exact per-contract receive validation and separate bounded event queues.
Gate both on validated readiness after Welcome; publish full/changed life rosters
without owner-private vitals. Reject uploads and fail/retire on capacity loss.
Actual encrypted death/respawn/leave/rejoin and simultaneous NPC/AV transfer pass
at 20/30/60 Hz. Rejoin qualification uses a new endpoint and injects old encrypted
traffic. One earlier same-relay-endpoint replacement timed out; immediate reuse
of that endpoint remains separate unqualified transport investigation scope.
Keep the five-second deadline and existing normal versions unchanged until the
new normal/Bevy slice qualifies. [Evidence](PUBLIC_AVATAR_LIFE.md).

## 2026-11-03 — separate public visual player lifecycle

P3-04 / P3-14 / P3-16: close the observer death-pose gap with a separate optional
projection of exact actor/life identity and alive/dead status. Deliberately expose
the existing life generation as visual identity so fresh-life animation resets
survive render coalescing; this grants no command or respawn authority. Keep
health, results, incoming windups and movement-stream barriers private/unchanged.
Use bounded sorted full rosters under the current eight-player development cap.
Bind a separate profile/World domain and qualify the World/codec prerequisite
before transport, new normal bundle selection and standard Bevy adoption.
[Boundary and acceptance](PUBLIC_AVATAR_LIFE.md). This is Theia presentation
metadata, not a new Classic formula or a population-capacity claim.

## 2026-11-03 — explicit normal public NPC timing adoption

Adopt schema 14 as a separate exact-byte normal bundle, retaining all 22 schema-13
inputs and adding the derived public timing profile after threat in World
identity. Enable the same explicit flag in both normal executables. Use standard
Bevy wolf clips, suppress expired/cancelled windups and face the actual victim;
never alter the player's target or manual facing. Owner-private vitals and
incoming damage remain separate. Actual native victim/observer and normal
quest/database-restart regressions pass, as do 1,106 workspace tests.
[Qualified scope and checkpoint](PUBLIC_CREATURE_ATTACKS.md). No default/older
bundle replacement, new combat formula or broader NPC capacity claim.

## 2026-11-03 — public attack stream uses optional priority transport

Use a separate bounded reliable ordered channel ahead of ordinary event/pose
traffic for the single development NPC's public attack timing. Retain legacy
two-channel configuration in inactive modes. Gate publication on accepted chunk
readiness, reject client uploads and retire/disconnect on capacity failure.
Keep private life/damage facts on the owner path; no new authority command.
The actual impaired victim/observer and refusal checks qualify the component;
normal launcher/Bevy/native adoption is a separate next slice.
[Contract and evidence](PUBLIC_CREATURE_ATTACKS.md).

## 2026-11-03 — public NPC timing without private victim state

P3-07 / P3-14 / P4-08: expose an explicit opt-in NPC windup contract containing
public entity identities and existing authoritative ticks, without health,
damage outcomes or private player-life generations. Bind the exact existing
life/retaliation profile and a public version domain into World identity. Keep
player targets/facing/controls and gameplay timing unchanged. Qualify observation
and codec before a dedicated prioritized reliable stream and standard Bevy
bystander animation. Default modes remain unchanged until normal adoption passes.
[Implemented boundary and pending acceptance](PUBLIC_CREATURE_ATTACKS.md).

## 2026-11-03 — bounded closed-catalogue compilation

Add an explicit offline handoff from globally validated authoring packs to a
closed runtime catalogue. Preserve all source quest IDs/versions/behavior and
include every transitive prerequisite; refuse current runtime count/objective/
byte overflow rather than dropping dependencies. Keep output metadata and exact
provenance explicit, then run normal World-reference validation. This does not
install content, load runtime packs, expand saved history or migrate character
progress. [Qualified workflow and limits](QUEST_PACK_AUTHORING.md).

## 2026-11-03 — offline quest pack preflight before runtime adoption

Implement the adopted global-registry direction first as a bounded read-only
core registry and file/index validator. Keep live `Catalog::from_json` closed;
do not expose partially linked catalogues to journals. Use explicit development
budgets, stable global definition IDs, checked cross-pack prerequisites and exact
source hashes. Keep narrative planning, World-reference admission, runtime pack
selection/history and saved-version migration separate. [Authoring contract and
proof](QUEST_PACK_AUTHORING.md). This does not raise the live 32-record limit.

## 2026-11-03 — explicit damage-threat development mode

P3-07 / P4-08: adopt the qualified damage-only World adapter through a new
schema-13 journal/threat bundle and explicit `--creature-threat`. Retain 1:1
actual-health-loss threat and current-reach 110%/130% transfer from the pinned
empirical Classic research; exact equality/tie order, transfer visibility,
recovery and tag-departure behavior remain explicit Theia policies. Keep metres,
server-owned victim selection and independent credit/reward ownership. Preserve
older modes until separately replaced. Healing, stances, taunts, social/group
threat and population scaling remain future work. [Evidence and limitations](CREATURE_THREAT.md).

## 2026-11-02 — quest catalogue scale direction

Keep narrative planning manifests separate from executable content. Under the
user's authorization to follow recommendations, plan manageable files/packs plus
a globally validated stable-ID registry and cross-pack prerequisites, rather than
a permanent 32-quest campaign cap. Bound active progress separately from retained
completion history; choose production budgets from measurements. Define explicit
saved-version/catalogue compatibility and migrations before adopting production
content updates. These are plans; current exact-byte single-catalogue validation
and development limits remain implemented. [Scope](QUEST_ENGINE.md).

## 2026-11-02 — quest-engine expansion and authoring agreement

P3-11 / P3-12 / P3-15 / initial P3-16: follow the reviewed Rust quest design by
extending the existing pure core and authoritative World/async persistence
boundaries. Use immutable versioned JSON definitions with stable IDs, a bounded
multi-quest journal and normalized trusted events. Begin with prerequisites,
ordered stages, kill and semantic NPC-interaction objectives. Retain atomic,
idempotent XP completion; preserve legacy rows until replacement acceptance.
Keep narrative Markdown separate from runtime definitions; do not silently turn
story drafts into approved gameplay. Defer scripting, conditional branches,
party/world scopes and general rewards until their own slices need them.
[Implemented scope and acceptance](QUEST_ENGINE.md). No new crate/service split.

## 2026-11-02 — explicit living-creature return reset

P3-07 / P3-06 / P4-08: schema-11 adds a bounded exact-byte policy restoring
maximum living NPC health and clearing target effects once on Return/Blocked.
It preserves entity life and existing navigation/leash rules; death/respawn stays
separate. No kill or award comes from reset. This is an explicit Theia policy,
not an exact Classic regeneration claim. [Qualified behavior and limits](CREATURE_RETURN_RESET.md).

## 2026-11-02 — opt-in creature-credit reset boundary

P3-09 / P3-10 / P3-11 / P4-08: the ownership-only development profile clears its
claim on authoritative Return/Blocked or tag-owner departure, advances an
encounter generation and refuses outgoing damage through Return/Blocked. Casts
capture that generation so delayed old spells cannot retag. Death alone retains
a connected session claim; pursuit reset independently abandons it. These are
explicit initial Theia policies. Full evade/healing/immunity, party eligibility,
shared credit and rewards are separate. Normal manifests retain their older
behavior. [Implemented scope and proof](CREATURE_CREDIT.md).

## 2026-11-02 — temporary melee-contact leash refresh

P3-07 / P3-06: replace the unconditional 30-second acquisition cutoff with a
soft allowance refreshed by authoritative visible melee reach. Retain the
captured 35 m hard boundary, blocked-route and target-life checks, two-second
LOS grace and bounded return travel. This is a Theia contact policy, not a claim
of Classic's exact damage/threat refresh algorithm. Ranged refresh and full
threat/evade/reset remain separate slices. [Behavior and acceptance](ENEMY_AI_REFERENCE.md).

## 2026-11-01 — explicit ability eligibility and swing rules

Ability schema 7 requires declared minimum range, manual facing and an explicit
continue/suspend/delay/reset interaction. Legacy profiles retain their behavior.
The initial stationary 500ms Arcane Bolt suspends swings; instant Sword Slash
continues them. These are versioned Theia fixtures, not verified Classic balance.
[Qualified behavior and limits](CLASSIC_ABILITY_RULES.md) includes atomic costs,
queued/completion revalidation and committed weapon swaps. No automatic facing,
action camera or dodge is introduced. Follow with checked damage resolution.

# World-system principles and working policies — 1 November 2026

Adopt the supplied broader Classic-system notes as design direction, reconciled
in [world systems / repo assessment](CLASSIC_WORLD_SYSTEMS_REFERENCE.md).
Preserve the strict initial controls, metric scale and Classic evidence policy.
Prioritize interacting combat, credit, population and recovery systems before
building complete zones. Implement them within the authoritative modular World;
no distributed combat service or new AI engine is implied.

Under the user's standing authorization to use recommended decisions, select:

- A versioned Classic-oriented combat/progression model with explicit level-80
  extensions, attack categories and positional eligibility. Validate formulas
  and rounding before adopting coefficients; prototype numbers remain fixtures.
- Ordinary creature ownership from the first qualifying effective damaging
  contribution, with separate tag/contribution/XP/quest/loot policies. Explicit
  shared quest/story targets can grant wider quest credit. Party eligibility and
  distribution rules are further work. Existing last-hit development credit is
  preserved until a versioned replacement passes competing-player tests.
- Corpse recovery and authored safe graveyards as intended overworld recovery,
  with explicit solo-story checkpoint rules. Exact penalties and repair economy
  remain balancing work; the current three-second home respawn remains a fixture.

These decisions describe direction, not completed gameplay. Complete the current
strict one-hostile combat slice, then add central combat membership, credit/tag
interfaces, a bounded encounter/spawn group and recovery. Exact historical claims
in the supplied notes require source/build verification before code adoption.
P3-17/P3-18 remain unspecified.

# Formula fidelity and metric units — 1 November 2026

Follow WoW Classic formulas as closely as evidence permits. Preserve 1 world
unit = 1 metre, m/km and m/s, checked f64 global positions and bounded f32 local
frames. Record source ruleset, unit mapping, uncertainty and Theia extensions;
do not mix expansion formulas or label fixtures as Classic balance. The level-80
direction remains. [Evidence policy and sources](CLASSIC_FORMULAS.md).

# Locked decision — 1 November 2026: strict WoW Classic foundation

Adopt [THEIA-COMBAT-001 v1.0](COMBAT_TARGETING_MOVEMENT_SPEC.md): traditional
character-relative movement, independent LMB orbit, RMB steering, both-button
forward, configurable bindings, persistent explicit targets, manual facing and
weapon-timed persistent melee autoattack separate from abilities. The server
validates position/facing, range/LOS, swing timing, resources and outcomes.
Selection and camera movement never turn the character toward an enemy.
Movement and interruption restrictions belong to ability data.

This supersedes the earlier WoW/GW2 hybrid for initial play. Defer all active
dodge work, action camera/permanent mouse-look, reticle targeting, automatic
facing/tracking and repeating weapon skills as basic attacks. Preserve existing
experiments and evidence without treating them as the new accepted controls.
[Migration, gaps and slice acceptance](MOVEMENT_CAMERA_TARGET.md) are recorded;
the runtime migration has not been implemented by this decision. Next define the
fundamental combat loop, then prove the thirteen one-hostile milestone checks.
No engine, network, coordinate or persistence architecture change is implied.

Earlier entries below are chronological records and yield to this decision where
they conflict.

# Superseded initial-scope decision — dodge key, 2026-11-01

Use V for the initial normal dodge binding in either camera mode. Cinnamon's
current `org.cinnamon.desktop.wm.preferences mouse-button-modifier` is `<Alt>`;
the earlier Left Alt plan would conflict with window mouse controls. Rebinding
remains a later settings task. V is a bounded one-shot request, not auto-repeat.
Stationary dodge goes backward relative to the camera view; moving dodge follows
world travel. Existing server-owned grounded/collision/cooldown and casting
rules apply. Native acceptance now passes at all three supported desktop sizes;
[implementation and evidence](DODGE_MOVEMENT.md).

# Superseded direction — 2026-11-01: WoW/GW2 player feel

Adopt GW2-oriented responsive movement and third-person camera with optional
action camera, retaining WoW-style tab targeting, action bars and interaction.
Classic cursor mode remains the default. Preserve authoritative collision,
prediction and mobile combat; UI/focus owns cursor release.
[Concrete behavior, slice order and acceptance](MOVEMENT_CAMERA_TARGET.md).
The first optional action-camera slice is now [qualified](PLAYER_CONTROLS.md);
full movement/facing tuning and live dodge remain open.

# Current decisions — 2026-11-01: quest markers and player controls

Use standard Bevy text/picking/UI and current authoritative observations for
initial one-guide quest markers. Markers have bounded visibility and static
occlusion; they never grant interactions or quest progress. [Current validation](QUEST_MARKERS.md).
The user adopted mouse enemy/friendly selection, right-click basic attack or NPC
interaction, Tab enemy cycling, 1–5 action slots, F context interaction and active
dodge. Use Left Alt as the recommended default dodge binding. Distinguish camera
dragging from clicks and let UI consume pointer events. Selection presentation
cannot authorize damage or dialogue. Implement dodge with checked authoritative
movement/prediction in its own slice; no implicit invulnerability is adopted.
[Concrete requested scope](COMBAT_WORLD_FOLLOWUPS.md). These controls are planned,
not yet qualified. No engine/plugin or network choice changes.

# Current decisions — 2026-10-31: finite authoritative magic travel

Adopt the opt-in schema-6 magic slice while retaining old instant catalogues and
CE02/CE03 paths. Server launch/impact, exact life identity, static solid/ground
sweeps and finite accumulated distance/lifetime own outcomes. Reserve capacity
before spending, retain started costs on cancelled travel and permit post-launch
equipment changes. CE04 carries launch/terminal presentation. Use standard Bevy
placeholder sphere travel with independent bounded cosmetic budgets.
[Qualified contract, evidence and limits](PROJECTILE_TRAVEL.md). Ballistic arrows,
bow equipment, ranged enemy casting and general projectile volumes remain open.
No engine/plugin pins change. Stop after this slice as requested by the user.

# Current decisions — 2026-10-31: server-owned hit outcomes

Adopt the opt-in schema-5 direct-damage hit/miss/deflect slice. Use checked integer
weights, pre-admission seeded ChaCha20, one draw only at valid resolution and
explicit current defender eligibility. Preserve old guaranteed-damage catalogues
and CE02 bytes; CE03 carries new outcomes. Cosmetic labels cannot alter results.
[Qualified behavior, evidence and limits](HIT_OUTCOMES.md). Initial prototype
weights are 85/10/5, with no inferred shield/class stats. NPC outcomes and travelling
projectiles remain separate work; engine/plugin pins are unchanged.

# Current decisions — 2026-10-31: committed melee and separate terrain/capture readiness

Adopt the opt-in schema-4 sword slice and preserve older bundles/checkpoints.
Exact committed main-hand item requirements are server facts; pending equipment
or cosmetic appearance never grants damage. Standard Bevy masks split upper-body
strike presentation from locomotion. Greybox movement readiness checks actual
terrain PBR specializations and GPU meshes, independently of whole-renderer
screenshot readiness. Native evidence demonstrates cosmetic pipeline compilation
without readiness withdrawal. [Contract and qualified evidence](PLAYER_MELEE.md).
General weapon classes, hit outcomes, ranged enemy abilities and public remote
player death remain later slices; no engine/plugin pins change.

# Current checkpoint — 2026-10-31: opt-in transient life/retaliation qualified

Adopt the thirteen-profile development bundle for checked first-attacker melee
retaliation, private health, dead-input suppression and supported fresh-stream
respawn. Keep the existing manifests and playable checkpoints. Health resets on
fresh admission in this development mode; durable vitals and production combat
logout policy remain explicit future decisions. Death animation, chasing and
general threat are not part of this slice. [Exact implementation and evidence](PLAYER_LIFE_NATIVE.md).

Two-account OIDC/native/SQL gameplay passes at three resolutions, along with
normal launcher cleanup. Final unchanged-source workspace: 525 passes / 40 skips;
Clippy, formatting and 28 Python tests pass. Exact matched set:
`build/player-life-qualified-v1-20261031`. Native Windows and standalone FPS / MMO
capacity remain unqualified. All changes remain local.

# Current decision — 2026-10-31: critical life transitions use bounded reliable delivery

P3-04 / P3-07 life state uses the existing owner-only reliable ordered Renet event
channel plus a 16-event client presentation queue. Queue exhaustion disconnects
or retires the affected peer; windup, death and respawn transitions are never
silently dropped to preserve cosmetic throughput. The watch bridge remains the
latest immutable state, while the bounded event queue preserves transitions for
presentation. Client health setters are forbidden. [Runtime contract and next
playable acceptance](PLAYER_LIFE_RUNTIME.md).

# Current decision — 2026-10-31: bounded opt-in melee retaliation

The next P3-07 / P3-04 extension holds the first exact attacker for bounded memory,
with one checked windup/recovery and life/reach/LOS checks at both boundaries.
No chase or general threat service is introduced. First qualify pure policy and
transient development health/respawn, then fixed-step/private-wire/standard Bevy
adoption. [Contract and gates](CREATURE_RETALIATION.md). Existing patrol/training
continues unchanged; durable vitals and production balance remain open.

# Current checkpoint — 2026-10-31: existing-account character menu qualified

P3-03: the console launcher lists existing characters and offers N to create
another below the eight-character cap. A known incompatible-profile refusal returns
to selection without rewriting saves; temporary retirement conflicts keep their
bounded retry. Actual OIDC creation on an existing account, incompatible-to-compatible
menu recovery, two native players, rewarded gameplay and database/server restart
pass at 1920×1200 windowed with zero Vulkan diagnostics. Normal eleven-profile
launcher startup/cleanup passes all three modes. [Contract and proof](CHARACTER_LAUNCHER_MENU.md).

Thirty focused Python checks, formatting and denied-warning workspace/all-target
Clippy pass. Rust/content/binaries are byte-for-byte unchanged from the previous
476-pass / 40-skip workspace and 30-contract SQL checkpoint; those suites were not
rerun for the five Python-file changes. Matched binaries/source/evidence:
`build/character-menu-qualified-v1-20261031`. This is console workflow; physical
terminal/browser input, a complete Bevy login UI and native Windows remain open.
Changes remain local. Prior checkpoints stay preserved.

# Prior checkpoint — 2026-10-31: character gameplay-profile preflight qualified

P3-03 / P3-02: enabled saved inventory/XP/quest profiles are checked in a read-only
transaction while the Selecting reservation is held, before issuing a game ticket.
Expected content incompatibility returns a clear nonretryable HTTP 409 and leaves
the SQL worker available. Missing optional state remains compatible. No saved data
is initialized, migrated or reseeded by this operation; active-owner checks remain.

Actual OIDC/native/SQL at 1920×1200 windowed passes refusal with no ticket, unchanged
legacy journal data through database/server restart, then fresh rewarded training,
loot/barter/equipment and rejoin. Zero Vulkan diagnostics on Linux RX 7900 XTX/RADV.
The client/core/network/content source is unchanged from the prior three-size
reward checkpoint; a new three-size server test is not claimed.
Final unchanged-source workspace: **476 tests / 40 explicit skips**
in 108 groups. Formatting, denied-warning Clippy, 24 focused Python checks
and 30 actual SQL contracts / 33 commands pass, including cancelled-answer and
bounded-queue preflight tests. [Contract and proof](CHARACTER_PROFILE_PREFLIGHT.md). Matched binaries,
source, validator and evidence: `build/gameplay-profiles-qualified-v1-20261031`.
All changes remain local. General migrations, complete saved-value validation,
physical OS input and native Windows remain open. Prior checkpoints are preserved.

# Prior checkpoint — 2026-10-31: optional quest completion XP qualified

P3-10 / P3-11 / initial P3-16: the opt-in rewarded training quest now accepts at
the guide, counts three authoritative kills, then commits quest completion,
25 XP and its action receipt atomically. Ready at 75 XP becomes 100 XP / level 2
on the default XP curve. Recovered commands and rejoin/restart never award twice.
Standard Bevy guide controls display the configured reward. Item turn-in grants,
abandonment/repeatables, groups and a complete authored production zone remain open.

Actual two-account OIDC/encrypted/SQL native flows pass at 1280×720 and 1920×1200
windowed (zero Vulkan diagnostics), plus 3440×1440 borderless fullscreen (one
previously documented upstream surface-resize diagnostic per startup). Private
progress, personal loot, barter, equipment, reconnect, database/server restart and
backend-loss checks pass. Current binaries also pass the retained journal-only
native flow. The normal eleven-profile schema-2 launcher passes all three cleanup
modes; schema-1 ten-profile validation identity is preserved. Fresh compatible
characters are required; saved content profiles are never silently migrated.

Final unchanged-source workspace passes **475 tests / 37 explicit skips**
in 107 groups. Formatting, denied-warning workspace/all-target Clippy and
23 focused Python checks pass. Separately retained prerequisites cover 27 actual
SQL contracts / 32 commands and three actual OIDC/ticket/SQL retirement modes.
[Exact scope, prerequisites and proof](QUEST_TURN_IN_XP.md). Immutable source, matched optimized
binaries, validator and public evidence: `build/quest-turn-in-xp-qualified-v1-20261031`.
Native Windows, physical 4K, physical OS input and MMO population/performance
capacity are not qualified. All changes remain local; no commit/push this session.
Earlier journal-only and automatic checkpoints remain preserved.

# Stopped safely — 2026-10-30: initial persistent training-zone integration qualified

The combined two-account OIDC/encrypted/PostgreSQL native run passes at
1920×1200 on Linux (RX 7900 XTX, RADV/Vulkan). Each character independently
completes three kills, earns XP, claims one reward, barters and equips a distinct
item. Exact bags, hands, progress and latest item receipts survive reconnect and
database/server restart; ownership/replay fencing and backend-loss checks pass.
[Scope, fixture settings and limits](PERSISTENT_ZONE_INTEGRATION.md).

**441 workspace tests pass / 30 explicit skips**, with unchanged final source.
Formatting, denied-warning workspace/all-target Clippy and nine Python helper tests
pass. Exact source, binaries and proof are preserved in
`build/zone-qualified-v1-20261030`; prior vendor and content checkpoints remain.
This combined flow is qualified at 1920×1200 only; prior vendor checks cover
1280×720 and 3440×1440 separately. Physical OS input, native Windows, 4K and
MMO-scale capacity remain unqualified. Accelerated qualification fixtures do not
change default gameplay balance.

Development is stopped at the user's request; all current repository changes are
prepared for commit and push, including concurrent story/Obsidian edits. Private
conversation exports stay local and ignored. This is initial P3-16 training
integration, not a completed authored production zone. P3-17/18 remain unspecified.
Earlier notes below are historical.

# Active development — 2026-10-30: initial P3-15 offline gameplay validator qualified

The ten-profile development bundle now has an offline validator using the actual
core/runtime schemas. It checks item/NPC/creature references and ability/resource/
status compatibility, bounds file reads/paths and reports deterministic exact-byte
hashes. [Tool, acceptance and limits](GAMEPLAY_CONTENT_VALIDATION.md). Runtime profile loading stays as
qualified; this manifest is an authoring check, not an authoritative world identity.

**441 workspace tests pass / 30 explicit skips**, with unchanged final source.
Formatting, denied-warning workspace/all-target Clippy, eight focused tool tests
and eight actual CLI refusal cases pass. Source inputs remain unchanged. Exact
source, tool/test executables, docs and proof: `build/gameplay-content-qualified-v1-20261030`.
All previously qualified vendor/gameplay source files remain byte-identical; the
playable checkpoint remains `build/vendor-qualified-v1-20261030` (actual SQL and
native purchasing at 1280×720, 1920×1200 and 3440×1440). No new runtime/native,
performance or MMO-capacity claim is made for this offline tool.

Next is a bounded P3-16 persistent training-zone integration check combining the
existing initial gameplay systems through login, play, logout and reconnect;
its acceptance still needs implementation/verification. Production authoring UI,
navigation/terrain/assets, general quest acceptance/turn-in and other remaining
systems stay open. P3-17/18 remain unspecified. Changes remain local/uncommitted;
concurrent story edits are untouched. Earlier notes below are historical.

# Active development — 2026-10-30: initial P3-12 live vendor barter qualified

The guide offers one positive-cost barter through the existing shared item
owner/sequence: one Training Token for three Training Stones. Live guide/session/
range checks precede acceptance; accepted queued/executing work survives logout.
The Bevy shop retains displayed offer/revisions and acknowledges committed items.
[Exact current behavior, evidence and limitations](VENDOR_BARTER_FOUNDATION.md).

**433 workspace tests pass / 30 explicit skips**, with unchanged final source.
Formatting, denied-warning workspace/all-target Clippy, nine Python helper tests,
twenty actual SQL contracts/twenty-six commands, actual item/reward/vendor retirement
and mixed-launcher cleanup pass. Two actual OIDC/native accounts pass purchases,
own SQL/UI comparison, close, reconnect, database/server restart, distant restore
and backend failure at **1280×720, 1920×1200 and 3440×1440 fullscreen**. Actual
widgets were inspected. Windowed GPU validation is clean; fullscreen records only
one specifically allowed upstream wgpu resize diagnostic per startup. Physical
4K, physical OS input and native Windows remain unqualified.

Exact source, five executed binaries, non-story docs and proof are frozen at
`build/vendor-qualified-v1-20261030`; earlier checkpoints remain preserved. Currency,
selling, stock/buyback, general quest acceptance/turn-in and production NPC/player
lifecycle remain open. No new frame-rate or MMO-population claim is made.
Next is a small P3-15 offline gameplay-content validation slice using the existing
checked schemas. P3-17/18 remain unspecified. Changes remain local/uncommitted;
concurrent story edits are untouched. Earlier notes below are historical.

# Active development — 2026-10-30: P3-12 barter database prerequisite qualified

The positive-cost barter now has an atomic backpack cost/output/receipt transaction
and uses the existing bounded critical worker. Exact latest retries recover without
another charge; superseded/conflicting input rejects. [Exact scope/proof](VENDOR_BARTER_FOUNDATION.md).

**Twenty actual SQL contracts/twenty-six commands pass**, including three new
purchase contracts: rollback, owner/world/byte binding, shared-revision races,
full-bag recovery, cancelled answers, queue saturation, backend loss and actual
process/database restart. Eight barter/36 core regression tests remain qualified;
core source is unchanged. Workspace check, formatting and denied-warning Clippy
pass. Exact prerequisite source/probe/proof: `build/vendor-store-qualified-v1-20261030`.

Live NPC/range entitlement, accepted item-owner retention, encrypted purchase
commands and shop UI remain open. No native purchase or new full-workspace claim
is made. The prior playable NPC/display remains frozen at
`build/dialogue-qualified-v1-20261030`: 418 workspace passes/27 skips and actual
OIDC/native dialogue at 1280×720, 1920×1200 and 3440×1440. P3-17/18 remain unspecified.
All changes remain local/uncommitted; story edits are untouched. Earlier notes
below are historical.

# Active development — 2026-10-30: initial P3-12 informational dialogue qualified

One noncombat guide now supports private authoritative branches, bounded Renet
requests/acknowledgements and a standard Bevy panel. E talks, F1–F4 or buttons choose,
Esc closes without exiting, and walking away closes the session. Clicks retain the
observed choice/revision. [Exact source/evidence/limits](NPC_DIALOGUE_FOUNDATION.md).

**418 workspace tests pass / 27 explicit skips**, with unchanged final source.
All seventeen SQL contracts/twenty-three commands separately pass; ten private-asset
cases remain skipped. Formatting, denied-warning workspace/all-target Clippy and
nine helper tests pass. Two actual OIDC accounts/native encrypted clients pass at
**1280×720, 1920×1200 and 3440×1440 fullscreen**: distinct branches, actual Bevy
keyboard/picking, close/reopen/walkaway, fresh sessions after reconnect/database/
server restart, distant restore and backend loss. Informational dialogue creates
no item/reward/XP/objective records. Actual panels were inspected. Windowed GPU
validation is clean; fullscreen has one or two specifically allowed upstream wgpu
resize diagnostics per startup and no other errors. Physical 4K/Windows remain open.

The mixed Scorch/resources/statuses/creature/equipment/reward/XP/objective/dialogue
launcher passes all three cleanup modes. Exact source, three executed binaries,
docs and proof: `build/dialogue-qualified-v1-20261030`. Previous checkpoints stay
preserved. This is the initial informational slice: vendors, quest acceptance/
turn-in, production NPC art, occlusion and real player-death closure remain open.
No new frame-rate or MMO-population claim is made. P3-17/18 remain unspecified.
All changes stay local/uncommitted; story edits are untouched. Earlier notes below
are historical.

# Active development — 2026-10-30: initial P3-11 quest objective slice qualified

One opt-in three-kill objective now uses trusted current-life facts, an atomic
XP/objective transaction, accepted-work retirement, private committed observations
and a standard Bevy journal. Completion grants no extra items or XP.
[Exact behavior, evidence and limits](QUEST_OBJECTIVES_FOUNDATION.md).

**396 workspace tests pass / 27 explicit skips**, with unchanged final source.
All seventeen actual PostgreSQL contracts/twenty-three commands separately pass;
ten private-asset skips remain skips. Formatting, denied-warning workspace/all-target
Clippy and nine Python helper tests pass. Actual OIDC/encrypted-ticket/SQL lease
checks retain accepted objective work through logout without duplicate credit.

Two actual accounts/native encrypted clients pass at **1920×1200 windowed** and
**3440×1440 fullscreen**: distinct own 1/3 versus 3/3 journals, completion after
XP caps, reconnect, database/server restart, distant restore and backend failure.
Actual widgets were inspected. Windowed GPU validation is clean; fullscreen has
one documented upstream wgpu resize diagnostic per startup and no other errors.
The ordinary 1280×720 mixed Scorch/resources/statuses/equipment/reward/XP/objective
launcher passes all three cleanup modes. Physical 4K and native Windows remain
unqualified. No new performance or MMO population capacity claim is made.

Exact source, four executables, documentation and proofs:
`build/quests-qualified-v1-20261030`. Prior checkpoints remain preserved. Next is
the bounded [P3-12 dialogue foundation](NPC_DIALOGUE_FOUNDATION.md), currently
plan only. General quest acceptance, dialogue/turn-in rewards and vendors remain
open. P3-17/18 remain unspecified. All changes remain local/uncommitted; concurrent
story edits are untouched. Nothing was pushed. Earlier notes below are historical.

# Project decisions

## Separate movement streams from world incarnation — 2026-10-22

The P2-04 command/reconciliation core binds a per-actor movement stream in addition
to world/actor identity. Teleport/recovery must replace that stream and discard
pending commands; it must not mutate a shared world incarnation to reset one
actor. Acknowledgements describe completed simulation ticks and receipt-based
input freshness, not packet count. Speculative history is capped at 64 steps;
unavailable collision, history overflow and incompatible readiness restore the
latest authoritative pose. These policies now run through the actual Renet authority/client using the
explicit TC01/TP01 movement profile alongside unchanged v11 snapshots/control.
The graybox launcher adopts prediction after native/controller qualification;
`--no-prediction` and imported maps retain the earlier movement path. [Scope](PREDICTION_CONTRACT.md).

## Verify source slots and Editor topology before mixed-material promotion — 2026-10-21

P1-10/P2-08/P2-10: source material indices alone do not establish slot fidelity.
Compare actual Editor submesh triangles; preserve source corner UVs/normals when
matching quad diagonals. Keep strict limits and use direct standard glTF streams
when editor normal re-encoding changes source directions. Crystal appearance is
an explicit static PBR preview with refraction/view effects separately recorded.
[Evidence](FOREST_MIXED_MATERIALS.md).

## Adapt untextured glass explicitly through standard PBR blend — 2026-10-21

P1-10/P2-08/P2-10: keep opaque eligibility strict; require an explicit source-state
profile for bounded glass tint/opacity/surface conversion. Use ordinary glTF/Bevy
BLEND and record Unity premultiplied lighting/colour-space differences. This
qualified visual adaptation does not establish shader equivalence or collision.
[Evidence](FOREST_GLASS.md).

## Keep independent foliage texture roles on standard glTF UV channels — 2026-10-21

P1-10/P2-08/P2-10: append a colour-only second UV channel for bounded source ST;
retain normal/AO UV0 and tangents. Verify source pixels, actual Editor frames,
exact mesh/material groups and native loaded material channels before promotion.
Prefab filenames do not identify renderer/material groups. The seventeen-draw
replacement passes; ordinary world adoption remains separate.
[Evidence](FOREST_COLOUR_UV.md).

## Resolve exact legacy sampler blocks through actual Editor evidence — 2026-10-21

P1-10/P2-08/P2-10: accept legacy sampler defaults only when hash-bound actual
Editor imports resolve the exact source texture to the qualified sampler profile.
Retain explicit source/metadata/collector identity and raw-source rejection without
evidence. This does not establish Unity compression/mips/BRDF parity.
[Evidence](FOREST_TEXTURE_DEFAULTS.md).

## Share identical material profiles while preserving image variants — 2026-10-21

P1-10/P2-08/P2-10: permit explicitly shared catalog IDs only for identical
material descriptors across distinct qualified inputs. Preserve contained Standard
image variants and verify exact source coverage before visual subset promotion.
The seventeen-binding batch passes; runtime adoption remains separate.
[Evidence](FOREST_TRIPLANAR_BATCH.md).

## Compose bounded material profiles through standard Bevy assets — 2026-10-21

P1-10/P2-08/P2-10: retain single-profile metadata and add a geometry-bound catalog
with exact material-ID mapping, typed content-addressed image paths, per-profile
extended materials and ordinary Standard geometry. Reject missing/unused/duplicate
or malformed mappings. Bound profiles, distinct images and decoded RGBA8 bytes;
report actual handles/material applications. Actual source composition, two-profile
GPU/pixel checks and retained regressions now qualify this origin-zero inspector
slice; rebase, streaming/mips and full forest adoption remain separate.
[Evidence](FOREST_MATERIAL_CATALOG.md).

## Apply recorded mesh frames without changing shading streams — 2026-10-21

P1-10/P2-08: after the user resumed, apply the actual Editor-derived frame to new
copies of affected canonical prototypes. The ten current bindings require only
translation; positive uniform scale is also qualified. Preserve other streams,
winding, material/image bytes and all unaffected artifacts. Independently compare
with actual Editor vertices and rebuild source placements before scene promotion.
Corrected static subset and standalone moss qualification now pass; normal map
adoption, multiple profiles and rebased texture phase remain separate gates.
Pixel checks require geometric screen-space interior and front-surface consistency
to exclude MSAA boundaries; colour limits remain unchanged.
[Evidence and limits](FOREST_MESH_FRAMES.md).

## Use actual Editor data before changing FBX root/pivot policy — 2026-10-20

P1-10/P2-08: serialized LOD bounds and independent FBX arithmetic do not establish
the problematic moss mesh's Unity import treatment. Stage only selected FBXs and
original metadata in an ignored scratch project and use a reviewed read-only
Editor collector. Compare actual mesh point sets/IDs and root transforms before
changing conversion; do not guess centring or compensate scale twice. Preserve
original metadata and report any Editor migration separately. The staged tools
and synthetic guards pass. Actual native Linux Editor compilation/import and
all 214 independent recorded-frame point-set checks now pass, covering all 112
scene FBXs with original metadata unchanged. The unadjusted policy differs on
19 meshes, affecting thirteen draws in the preserved current batch. Converter
correction and native replacement qualification remain for resumption after the
user's safe stop. [Contract and acceptance](FOREST_UNITY_REFERENCE.md).

## Bind conservative transformed mesh AABBs during authoring — 2026-10-20

Actual town startup exposed tighter offline transformed-vertex bounds than the
client's transformed local AABB checks. Author the latter envelope from actual
accessors and the static hierarchy; retain strict runtime validation. A new
index changes dataset/admission identity while package/image bytes remain bound
unchanged. Original fixtures and historical datasets stay available; the normal
launcher remains monolithic. [Evidence](TOWN_CLIENT_RESOURCES.md).

## Bind finite playable bounds separately from chunk geometry — 2026-10-20

Chunk-session schema 2 requires a checked global XZ rectangle and binds it into
world identity v4. Schema 1 retains its v3 identity and compatibility behavior.
Convert capsule-centre planes conservatively into the local query frame and
combine their sweeps with authority/readiness coverage in the shared movement
solver. Preserve vertical motion and internal seams; clamp lookahead to the
playable rectangle. Keep package geometry/contributor envelopes independent.
The normal map default and wire/ALPN 11 remain unchanged. [Contract](WORLD_EDGES.md).

## Share immutable images through standard Bevy asset paths — 2026-10-20

Manifest schema 2 binds PNG dependencies by SHA-256, size, colour space and glTF
sampler settings. Canonical relative variant paths let compatible materials share
one standard Bevy `Image` handle while incompatible settings retain distinct
handles. Verify declared/actual references, paths, hashes and resource budgets
before asset loading. Prepared chunks ignore unbound editor metadata through the
standard `AssetMetaCheck::Never` setting. Keep the headless collision path free
of image I/O; server admission still binds the full manifest identity.

Extract to new packages with original geometry/image bytes and material values;
retain schema-1 identities and the monolithic default. Actual town extraction
leaves a 5.48 MB GLB plus 488 shared resources, not a partitioned world or a
compressed 5.48 MB map. [Contract and evidence](SHARED_IMAGES.md).

## Retain spare residency until a new load requires the budget — 2026-10-20

A delayed native regression exposed a trailing-peer-first input order that issued
an unload-only intermediate offer before the leading peer requested a neighbour.
Retain existing spare packages when demand is already covered; replace them only
when new providers are needed. Keep the same resident cap and explicit unload
APIs. Deterministic signed-frame and actual encrypted UDP tests exercise that
arrival order before final native qualification. [Handover](CHUNK_HANDOVER.md).

## Derive complete cells from bound geometry contributors — 2026-10-20

Chunk index schema 2 binds conservative package-local combined XZ bounds into a
new canonical dataset identity. Derive contributors from those bounds at index
load; a cell is complete only when every required package is resident and ready.
Use that closure for coverage, lookahead, protected actors, recovery and retained
presentation. Validate actual collider extents during off-thread preparation and
actual transformed standard-Bevy mesh extents before readiness/publication.
Keep schema-1 experiments and the monolithic launcher as regression baselines.

Author new bounds from actual static GLBs and collision data without changing
packages. This bounded four-package/64-entry slice does not partition the town or
prove mesh fidelity. Town inventory shows 698.7 MB of 704.1 MB is embedded images;
shared immutable image binding is therefore a prerequisite to real imported
partitioning. Use standard Bevy asset loading, with measured budgets before
adoption. [Contract and scope](CHUNK_CONTRIBUTORS.md).

## Pair authority offers with retained Bevy/Avian publication — 2026-10-20

Wire/ALPN 10 carries bounded tagged chunk offers and readiness acknowledgements.
Use the existing authority activation controller with an opt-in lookahead policy;
prepare one client candidate off-thread and stage standard GLB roots hidden.
Before retirement, withdraw removed coverage and await the exact acknowledgement
and post-application snapshot. Protect current/presented actor footprints, both
remote interpolation endpoints and recovery support. Preserve retained entities;
replace matching static Avian bodies and wait for live query/render readiness
before granting the full replacement set. Hide remote presentation outside a
lagging client's active coverage. Failed candidates retain the active pair.

This is a bounded original-fixture path, not general map streaming. Keep the
monolithic town default until geometry contributors/partitioning and measured
content budgets qualify. [Contract and open checks](CHUNK_HANDOVER.md).

## Refine head-on wall contact with Avian feature manifolds — 2026-10-20

A real network regression exposed sub-millimetre wall drift on sloped support.
A deterministic 20/30/60 Hz fixture reproduces it. Refine near-head-on wall sweep
distance and normal using a matching standard Avian feature manifold; retain
conservative CCD behavior for glancing contacts and incompatible features. Keep
correction bounded and inside the requested sweep. Do not quantize movement axes
or loosen exact persistent-contact assertions. Preserve tiny intentional tangent
input, diagonal sliding, retreat, support validation and capsule/sole fallback.
Final qualification is recorded separately in [validation](VALIDATION.md).

## Sweep movement against transient chunk coverage — 2026-10-20

Enforce missing authority/client coverage inside the shared capsule solver,
including gravity-driven horizontal projections and step-up alternatives. Use
bounded checked cell footprints and infinite-height outer edges; keep shared
seams open and prevent missing-corner/gap shortcuts. Preserve free vertical
motion and release ordinary sliding when loading/readiness permits it. Do not use
recovery as the loading barrier. Validate candidate ground snaps before applying
imprecise convex support distances. [Contract](CHUNK_BOUNDARIES.md).

## Acknowledge readiness after simulation before allowing retirement — 2026-10-20

Wire/ALPN 9 binds authority acknowledgement to the exact report, epoch and a
post-application simulation tick. Coalesce one pending reply per peer and respect
reliable-channel capacity. Client confirmation requires both the latest report's
acknowledgement and a snapshot at or after that tick; old acknowledgements cannot
confirm a superseding request. Presentation/interpolation retention remains an
additional gate before actual unloading. [Contract](CHUNK_READINESS.md).

## Gate chunk movement on each client's actual initial readiness — 2026-10-20

Wire/ALPN 8 adds bounded epoch/sequence-checked chunk readiness on the secure
Renet control channel. Derive actor identity from the connection; require both
authority residency and that actor's reported capsule-footprint coverage for
intentional horizontal movement. Keep gravity, jump and acknowledgements running.
Publish initial readiness only after standard Bevy assets and live Avian queries
are ready. Keep dynamic unloading separate until withdrawal acknowledgements and
presentation retention are implemented. [Contract](CHUNK_READINESS.md).

## Share chunk startup and bind its initial selection — 2026-10-20

Use bounded `world.session.json` for trusted global spawns, initial chunk IDs,
resident budget and fall plane in normal server/client startup. Keep dataset
identity independent of query frames; project scenery and actual Avian bodies
through the explicit client content frame. Bind initial IDs in the version-2
session fingerprint until paired dynamic readiness exists, rejecting a different
initial set over actual Renet even when dataset identity matches. Keep ordinary
Bevy assets/camera/presentation and the qualified monolithic launcher default.
[Contract and adoption limit](CHUNK_STARTUP.md).

## Revalidate current bodies before chunk publication — 2026-10-20

Use a background collision candidate tied to the world epoch and residency
generation. Revalidate full actor footprints, penetration, grounded/airborne
support and trusted recovery spawns at publication. Bound local owner-side
management to one in-flight transaction; retain gravity/jump while an unloaded
neighbor prevents horizontal crossing. Qualify through the existing actual Renet
loop before pairing normal Bevy residency. Keep normal monolithic map selection
until the paired adoption gate passes. [Contract](CHUNK_ACTIVATION.md).

## Promote schema-3 town after actual native qualification — 2026-10-20

Default to the verified revision-2 town with explicit spawns after general mesh,
map route/jump/full-tile, distant frame, spawn-free chunk lifecycle and startup
rejection checks pass. Verify the actual launcher default's content ID/revision,
process and ticket cleanup. Keep source geometry fidelity explicit: migrated
maps retain their boxes/floors; the original mesh fixture proves the general path.
Refresh standard Avian query trees on zero-delta frame changes, and use mesh
sweeps for schema-3 indoor cameras. Require visible local characters and useful
camera distance in native acceptance, not only entity counts or image colours.
See [qualification](GENERAL_COLLISION.md).

## General collision schema and explicit spawns before chunk activation — 2026-10-19

Add collision schema 3 for full 3D static triangle geometry and explicit local
capsule-bottom spawns. Query-only chunks may omit spawns; normal session worlds
must provide validated support/headroom. Preserve legacy package behavior and
normal launcher defaults while qualifying new content through real clients.
Keep geometry/spawn identity in the existing versioned manifest. The user requested
a safe stop before native qualification/default adoption; see the
[checkpoint contract](GENERAL_COLLISION.md).

## Adopt shared movement for imported development maps — 2026-10-19

Normal imported-map authority now uses the shared capsule controller. Resolve and
validate historical development spawns once, keep ongoing support local to the
body, and retain conservative collision proxies as their actual geometry. Add a
finite-world movement envelope only during normal-world adoption, never at chunk
ownership seams. Unsupported falls or solver rejection recover only the affected
actor to its trusted spawn, preserving held jump and sequence/session identity.
Make recovery observable and reject it during ordinary route qualification.
This development policy is not gameplay death/respawn or final collision authoring.
See [contract](KINEMATIC_CONTROLLER.md).

## Share the capsule controller before migrating imported terrain — 2026-10-19

Use one fixed-step, world-only Avian query controller for graybox authority and
its offline sandbox: full-capsule sweeps, bounded slide/step/ground acquisition,
gravity and held-level jump. Server owns motion state; connected clients present
its poses. Qualify actual geometry and real secure replication before replacing
the imported map's legacy walker. Wire v7 adds a jump level; no client pose,
velocity or timestep becomes authoritative. Keep authored map routes as
regressions. General map activation and chunk adoption remain separate gates.
See [contract](KINEMATIC_CONTROLLER.md).

## Qualify standard Bevy chunk publication before normal adoption — 2026-10-19

Keep the imported map unchanged while an actual-client diagnostic proves bounded
worker preparation, ordinary GLB/world-scene readiness and publication with live
Avian bodies. Preserve retained roots; failed candidates retain old presentation/
collision. Check real rendered pixels and post-physics queries, not root counts
alone. Use existing Reinhard tonemapping without introducing LUT dependencies.
This is a bounded original-fixture gate; moving actor checks, authority activation
and normal chunk adoption remain explicit work. See [contract](CHUNK_PRESENTATION.md).

## Prove chunk ownership/residency on original geometry first — 2026-10-19

Use the checked half-open global 128 m grid and explicit bounded query frames.
Validate staged packages and protected owner chunks before publishing an active
collision set. Start with synchronous, finite headless proof outside simulation;
record actual queries and unload effects. A visual-root descriptor does not prove
rendering or asynchronous streaming. Keep the normal imported tile intact while
qualifying a standard-Bevy consumer and publication lifecycle separately.
See [scope](CHUNK_RESIDENCY.md).

## Stable package identity before chunk/controller expansion — 2026-10-19

Introduce an explicit manifest for the existing prepared world format, binding a
stable logical ID and revision to independent collision/visual bytes. Keep source
exports and artifact hashes distinct from world incarnation/session identities.
The headless loader verifies collision only; the client additionally streams
visual hashing before ordinary Bevy loading. Normal development launch requires
the manifest; standalone legacy fixtures remain compatible, and malformed present
manifests cannot downgrade. Preserve height-surface controller limits and wire v6.
See [package contract](CONTENT_PACKAGES.md). Keep the qualified 30 Hz default.

## Qualify the existing scheduler and retain skip policy — 2026-10-19

Keep the fixed configurable scheduler and one pending intent per peer. Under
executor stalls, skip missed wall periods and slow simulation time; do not add
unbounded catch-up. Gameplay input expiry remains simulation-time based. Add fixed
memory timing histograms and qualify the actual Renet loop with eight authenticated
clients, burst/replay traffic, expiry, a controlled stall and input-budget removal.
Keep simulation cost, transport pump cost, wall cadence and rendered evidence
separate. [Scope and evidence](SCHEDULER_QUALIFICATION.md).

## Client render/Avian rebasing from canonical content — 2026-10-19

Use one client origin resource and canonical static-root translations, with global
pawn/interpolation endpoints. Reproject actual collider Positions/Transforms;
parent transforms alone do not move Avian bodies. Run this static client physics
world in PostUpdate after placement so queries/transform propagation/camera
refinement agree before extraction. This does not move server truth or change
wire v6. Server queries keep the fixed bounded content frame; dynamic contacts and
multi-chunk ownership remain separate work. See [implementation](FLOATING_ORIGIN.md).

## Global authority and explicit incarnation/frame contracts — 2026-10-19

Store checked f64 actor positions and project into bounded f32 Avian/content
frames. Serialize an explicit f64 origin with local offsets, preserving the
1,200-byte/eight-peer budget. Wire-frame recentring must not move authority or
presentation. The reliable welcome establishes the fixed content anchor and
random world incarnation; reject old-epoch traffic before sequence coalescing.
Actor IDs are connection-owned, monotonic within an epoch, and distinct from
transport/account IDs. Wire/ALPN becomes 6. This is a bounded fixture migration;
automatic renderer/physics floating origin and region ownership remain later work.
See [coordinate/identity contract](COORDINATES_IDENTITY.md).

## Renet becomes the local launcher default — 2026-10-19

The bounded replacement gate now passes against the actual executable runtime:
controlled impaired UDP, queue/traffic/RTT measurements, short and timeout outages,
fresh-ticket replacement and captured old-packet rejection. The normal launcher
and multiplayer qualifier default to Renet; explicit Quinn and legacy standalone
CLI behavior remain regression options. This is the planned architecture migration,
not a Quinn fallback or production authentication/capacity claim. Native platform
and remaining security limits are recorded in [qualification](RENET_QUALIFICATION.md).
The next foundation work follows live coordinate/identity contracts.

## Foundation order and opt-in Renet bridge — 2026-10-19

The user currently prioritizes engineering foundations over playability. Reconcile
and follow actual task dependencies, preserving the imported register and keeping
partial work open. See [execution order](engineering/EXECUTION_ORDER.md).
Renet now has an executable server/Bevy/launcher path, selected explicitly with
`--transport renet`. Its local issuer keeps the signing key server-side and supplies
short-lived, one-session tickets; this is not production account authentication.
Retain Quinn as the launcher default until controlled network qualification passes.
See [bridge contract](NETWORKING_RENET.md). No transport failure/fallback decision,
new repository, commit or push is implied.

## Avian development revision adopted — 2026-10-19

The user explicitly requested adopting and pinning the Bevy 0.20-compatible Avian
development revision and changing the client over. Pin Avian `0.8.0-dev` at
`1070759b2145dc078abedace91dc7fe78013a45c` with Bevy `=0.20.0`; do not track a
floating branch. Standard Avian plugins/colliders are active in the normal client.
Server and offline movement use the same Avian query backend. Renderer/window
features belong to the client; the server remains headless. Existing profiles and
grounded support policy remain compatibility inputs with explicit limitations.
Wire/ALPN advances to 5. See [physics scope](PHYSICS_AVIAN.md).

## Active adoption and first foundation migration — 2026-10-19

The user's follow-up “Make all changes needed to align” adopts the supplied
architecture as the target in this repository and authorizes tested migrations.
The blanket static-environment restriction is superseded by predominantly static
collision with scripted authoritative platforms/elevators and presentation-only
vegetation effects. Doors, buildings and ordinary props remain static unless
separately specified; character pass-through remains required. Root AGENTS.md now
records the active rules. No new repository, commit or push is created.

`theia-core` owns checked global/local coordinate and timestep contracts. The
existing authority now supports configured 20/30/60 Hz with fixed-step movement,
time-based input expiry and deadline/work metrics; snapshots advertise tick rate
in wire/ALPN version 4. Renet/netcode 2.0.0 have an initial secure, bounded-channel
UDP proof. Quinn remains the launcher implementation during migration, not a
declared fallback. Global runtime poses and Avian movement remain future slices.

Published Avian 0.7 uses Bevy 0.19. Upstream 0.8-dev targets Bevy 0.20 and is a
revision-pinned qualification candidate; do not mix engine versions or claim
Avian integration merely from reading its manifest. See the
[alignment record](engineering/ALIGNMENT.md) and [live task status](engineering/TASK_STATUS.md).

## October 19 engineering handoff integration

On 2026-10-19 the user requested that the supplied handoff be added as engineering
planning material and that this repository be assessed against it. See the
[engineering index](engineering/README.md) for the source documents, local
interpretation and execution queue; the [assessment](engineering/REPOSITORY_ASSESSMENT.md)
records current implementation evidence separately from intended architecture.

The supplied initial targets are Renet, Avian, f64 global/f32 local frames and
the six-phase gameplay/world plan. These supersede the old roadmap's next-step
ordering as planning targets; they do not make those systems implemented or
authorize wholesale replacement without tested migration slices. Quinn remains
the current local transport. Its earlier selection below records history, rather
than a reason to skip Renet validation. The imported architecture's `theia-engine`
link does not move this project or introduce a dependency on that repository.

The attachment's moving platforms/elevators and environment animation conflict
with the existing explicit static-world constraints. That scope remains unresolved;
preserve the current constraints until the user resolves it. Character pass-through,
server authority and Bevy rendering remain requirements. Imported contributor
instructions are preserved as `AGENTS.reference.md`, not installed as active rules.

This change adds documentation and an audit only. No runtime migration, new repo,
commit or push is performed. Keeping this repo and migrating its foundations is
the audit recommendation, not an assertion that the user chose a repository move.

## Earlier decisions and implementation history

Approved direction from the project discussion, 2026-10-16. These decisions apply
to this MMO repository; historical engine decisions remain in `theia-engine`.

| Decision | Consequence |
|---|---|
| Build the MMO using Bevy | Standard rendering, asset, animation and UI systems first; specialized plugins require a game need |
| Fresh repository, preserve the old engine | No wholesale copy or runtime dependency on the experimental renderer |
| Keep Theia as the project name | Bevy is the engine foundation; this is a game project |
| Rust client and authoritative server | Client input/presentation never establishes trusted gameplay truth |
| QUIC via Quinn remains the intended transport | Implement the bounded local walking slice first; review and record any adapted old code |
| Linux primary, Windows first-class | Preserve portability and qualify platforms separately |
| Static authored world; players do not collide with other players or NPCs | Environment collision, targeting/hit detection and navigation are separate requirements |
| All environment items remain static, including doors | No opening doors, moving props or destructible environment; character animation is separate |
| Go services when a concrete requirement needs them | No distributed-service scaffold in the first playable slice |
| Start with two clients in one authoritative world | Do not substitute synthetic proxy counts for working multiplayer |
| Use the supplied human for players, RustyWolfPup for enemy NPCs and the farmhouse for building/housing tests | Opt-in local reference scene first; NPC authority/combat and housing gameplay are separate from visual roles |

The previous custom ash/Vulkan renderer, mandatory task/mesh capability, Slang-first
shaders, custom GPU ECS, visibility-buffer pipeline and bespoke vector UI are not
requirements of this application. Use Bevy's renderer/device capabilities and
normal conventions. In particular, this scene uses Bevy's Y-up coordinates with
movement in XZ; the old engine's Z-up convention is not copied into game data.

Bootstrap implementation choices: pin [Bevy 0.20.0](https://bevy.org/news/bevy-0-20/),
use Rust 2024 and Rust 1.99.0, and keep keyboard
movement explicitly offline until the authoritative slice. Keep changes local
until the user authorizes a GitHub push.

The user's subsequent direction is a WoW-style MMORPG. See [game direction](GAME_DIRECTION.md)
for the intended gameplay and first starting-zone loop. The implementation uses
original content and standard Bevy systems; these targets do not imply existing
combat, quests or WoW asset compatibility. Local networking choices and reviewed
code provenance are documented in [NETWORKING.md](NETWORKING.md).

The graybox now has a single shared geometry definition for rendering and server
collision, with a flat-ground box sweep and a local orbit camera. This qualifies
only the current scene, not general terrain physics. Scope and limitations are
recorded in [WORLD_AND_CAMERA.md](WORLD_AND_CAMERA.md).

Building/prop delivery target: glTF 2.0, preferably GLB, with original/intermediate
sources retained. Standard Bevy loading is enabled, including its normal automatic
reflected type registration. A separate visual inspector preserves source scale
and never establishes server collision. The user's supplied house is a local
compatibility reference; it is not original Theia world content. Current import
scope and source provenance are recorded in [ASSET_IMPORT.md](ASSET_IMPORT.md).

A bounded terrain-only walking fixture now qualifies authoritative ground height
and sampled slope clearance using the same source triangles as its visual GLB.
It is an opt-in implementation slice, with world consistency in wire version 2,
not approval of general mesh physics or a new cooked-world architecture. See
[TERRAIN_WALKING.md](TERRAIN_WALKING.md).

The terrain fixture can now use the already approved human player visuals through
standard Bevy asset loading/animation. This optional presentation reuses the same
authoritative player roots; it adds no character collision or protocol authority.
The separate reference scene retains the wolf and farmhouse.

The first terrain static-object slice uses an explicitly authored solid farmhouse/
foundation envelope shared by server movement and client camera clearance. This
is conservative ground-blocking collision, not automatic general mesh collision,
traversable housing or a change to nonblocking players/NPCs. See
[TERRAIN_STATIC_OBJECTS.md](TERRAIN_STATIC_OBJECTS.md).

The user requested walking inside the house. A separate authored ground-floor
fixture now qualifies a permanently open doorway, fixed access ramp, measured
walls and source floor support for the fixed player size. The old solid-house
fixtures stay available. This adds no movable environment or player/NPC blocking;
general multi-floor physics and housing systems remain separate. See
[ENTERABLE_HOUSE.md](ENTERABLE_HOUSE.md).

The user chose to keep the supplied WoW assets for gameplay development and use
**the full supplied map tile** as the ongoing development world before targeting,
abilities and quests. `tools/play.py` is the normal development entry point;
`--terrain-world` still selects the world for both applications. Source/derived
third-party content stays local and ignored. Original/replacement content remains
future work. The full-map outdoor pass uses conservative M2 metadata bounds,
solid WMO envelopes and no-entry water cells; it does not establish general
interiors or swimming. See [development-world scope](DEVELOPMENT_WORLD.md).

The next map slice opens the original placed farmhouse's ground floor, retains
its furnishings and original scale/rotation, and adds a fixed ramp. Other map
buildings remain solid. The default launcher selects `build/development-world-access`;
the prior outdoor profile remains selectable. Static map mesh ray casts supplement
camera clearance only; authoritative movement still uses the shared profile.
Terrain ambient fill is increased for visibility. [Scope](MAP_FARMHOUSE.md).

The next local map slice retains the farmhouse and opens the original blacksmith
main room, with pinned source floors, conservative walls/exterior exclusions and
a fixed ramp. The default launcher selects `build/development-world-town` and
spawns near the blacksmith. Other interiors and the forge annex remain separate.
No new movement authority or wire/profile version is introduced. [Scope](MAP_BLACKSMITH.md).

## Offline partitioning preserves contributors before runtime budget changes

2026-10-20: P1-10/P2-06/P2-07/P2-08 author actual terrain/collision at 128-metre
planes and assign whole static model mesh instances once, preserving ancestor
transforms and original buffers. Per-package graph pruning retains canonical
shared image keys. Unsupported animated/extended layouts and reflected terrain
fail explicitly. The source world and normal launcher remain unchanged.

Actual town content measures nine providers for a cell and ten at current spawns.
The existing four-package runtime contract is not weakened to publish this data:
only a candidate index is emitted. Qualify measured actor/lookahead/recovery/
staging budgets, world-edge policy and real town publication before adoption.
Offline/source comparisons and original-fixture native evidence are recorded
separately in [validation](VALIDATION.md); neither closes a whole task or phase.

## Budget analysis precedes the wider runtime contract

2026-10-20: contributor-complete capsule analysis measures recovery plus one/two
actors at 18/22 packages, with all 26 finite town packages reached by conservative
multi-actor XZ demand. Verified all-indexed content has 4,001 static colliders and
about 1.51 GB estimated base RGBA8 image bytes. These are offline bounds, not
reachable-route, live allocation or load-latency evidence.

Use a proposed 32-package hard ceiling and 26-package town session for the next
qualified runtime slice. Bind budget to admission, validate checked chunk
addresses and control-message sizes, define world edges and measure actual
multi-root staging/resources before adoption. The current four-package contract
and monolithic default remain active. [Design and acceptance](CHUNK_BUDGETS.md).

## Qualified finite runtime capacity and admission budget

2026-10-20: measured town demand justifies a 32-package hard ceiling while
retaining each session's explicit smaller budget. World identity v3 binds that
budget; wire/ALPN 11 permits at most 32 canonical checked-global addresses within
unchanged 1,200-byte messages. Native original fixtures prove 32 initial roots
and six new providers in one delayed paired publication. The opt-in private town
authority qualifies ten-to-26-package publication, not rendered town adoption.

The collider/index/local-frame/queue ceilings and 30 Hz default stay unchanged.
Keep the monolithic default until explicit world-edge policy, actual-town client
resources and remaining lifecycle cases pass. [Contract](CHUNK_CAPACITY.md).

## Preserve full source skins when previewing standard-renderer adaptation

2026-10-20: the Synty sample needs pose baking across different FBX bind poses,
not direct curve copying. Use standard Blender export and Bevy animation rather
than custom renderer/skeleton infrastructure. Make the skin mesh an identity
scene root so ordinary bounds and actual skinning agree in metres.

Retain a full-influence reference GLB. The standard four-weight preview is an
explicit measured approximation: Jump reaches 6.974 mm skin difference and fails
the initial 5 mm gate. Preview rendering does not approve character adoption,
foot planting or gameplay animation. Keep source/private outputs ignored and the
normal game unchanged. [Contract](SYNTY_CHARACTER_PREVIEW.md).

## Resolve trusted recovery guesses and preserve exact metadata floats

2026-10-20: presentation uses the same shared support/clearance-resolved recovery
feet as authority; raw metadata remains an identity input. Enable serde_json
`float_roundtrip` on its existing locked version after a source-height one-bit
identity mismatch. Keep exact independent identity checks and existing wire
layout. Observe bounded standard-Bevy root/asset lifetimes and synchronize native
captures before declaring a live publication qualified. The monolithic launcher
remains active. [Contract and scope](TOWN_LIVE_PUBLICATION.md).

## Use Enchanted Forest as the environment import target

2026-10-20: the user replaces Elven Realm with Enchanted Forest v1.01.
Proceed with the available forest source and explicit standard-Bevy conversion;
Elven missing-LFS recovery is superseded. Preserve current town functionality
until forest geometry/material/collision and native multiplayer checks pass.
[Evidence and acceptance](SYNTY_ENCHANTED_FOREST_IMPORT.md).

## Explicit source material profiles for forest conversion

2026-10-20: adapt the identified opaque Unity Standard subset through ordinary
Blender/glTF PBR, preserving source albedo/emission, scalar surface parameters,
backface culling and repeat/bilinear sampling. Reject unsupported active features;
do not treat a custom shader as Standard merely because it exposes similar keys.
Bind independent geometry/pixel/material checks to each qualified asset. This is
PBR adaptation, not Unity shader/lighting equivalence or assembled-map adoption.
[Contract and executed evidence](SYNTY_ENCHANTED_FOREST_IMPORT.md).


## Preserve source hierarchy and classify only proven unused flat overrides

2026-10-20: prepare static forest instance plans in source coordinates with
explicit highest-detail LOD selection and independent matrix/binding checks.
Only absent target IDs in fully materialized flat prefabs with the matching
source GUID are classified as unused serialized overrides. Keep nested/model
inheritance and unknown live properties unresolved; never invent replacement
transforms or material bindings. Geometry-axis and rendered/collision adoption
gates remain separate. [Contract](UNITY_STATIC_PLACEMENTS.md).


## Source-specific static foliage profiles through standard PBR

2026-10-20: bind the basic vegetation adapter to reviewed shader bytes; consume
binary per-face vertex selectors, derive colour-floor PNGs with exact source
alpha, and use ordinary glTF MASK/double-sided/surface settings. Record mesh-normal
and colour-workflow approximations explicitly. Keep preview LOD selection separate
from the default highest-detail scene plan. Full vegetation and normal-map effects
require their own qualification, not automatic material substitution.
[Contract and evidence](FOREST_FOLIAGE.md).


## Compile model variants only from checked aliases and complete authored poses

2026-10-20: the static single-mesh model-variant profile requires explicit source
materials and every final root TRS component in the outer instance. Bind inherited
IDs to actual source references, preserve collider records as unqualified data,
and reject missing pose components instead of inventing imported defaults.
Verify category completeness and source matrices independently; geometry/importer
axes and scene adoption remain separate. [Contract](UNITY_MODEL_VARIANTS.md).


## Calibrated forest mesh poses through standard glTF TRS

2026-10-20: use source-specific mesh-local X reflection and H M H placement
conversion, backed by independent ufbx/Unity LOD-bound evidence. Preserve the two
explicit source-bound mismatches. Express affine shear as two ordinary TRS nodes
and cache a winding-correct mirrored mesh variant for negative parity, keeping
normal Bevy rendering and shared materials/images. Five actual source placements
and an original asymmetric fixture qualify the bounded slice; full terrain,
materials, collision and map adoption remain separate. [Evidence](UNITY_GEOMETRY.md).


## Preserve full TerrainData samples and qualify rendering differences explicitly

2026-10-20: decode the forest's binary Unity 2020 terrain offline using pinned
UnityPy, checked source GUID/fileIDs and all serialized patch extrema. Retain
all height/control samples; represent ground with standard glTF. The initial
five-layer colour bake is an explicit finite-resolution approximation with
geometric normals, not terrain shader or normal-map parity. Preserve authored
normal bindings and keep map adoption open. Partition full-sample collision to
fit existing budgets rather than silently decimating. [Evidence](UNITY_TERRAIN.md).


## Full-sample forest ground through existing bounded packages

2026-10-20: publish four cell-aligned ground packages without clipping/decimation,
using existing schema-3 collision and shared-image contracts. Preserve exact
source attributes and finite world bounds, with ordinary Renet/Bevy/Avian and
one image handle across roots. Add an explicit chunk-world launcher option while
retaining the default town. Ground-only qualification does not adopt forest
objects/materials or prove Unity TerrainCollider parity. Each package fills its
current triangle allowance; review full-scene collision representation/budgets
before adding object meshes. [Evidence](FOREST_TERRAIN_COLLISION.md).


## Explicit static full-vegetation PBR profiles and canonical prototypes

2026-10-20: source-identical noise colours may collapse to constant PBR colour
floors; other spatial colour/effect profiles stay unresolved. Apply explicit
normal-map decoding/import policy and source alpha through standard glTF while
recording unapplied wind/highlights, filtering and Unity parity limits. Keep mesh
prototypes separate from source placement, preserving original instance matrices.
Match exact f32 cyclic positions and bounded UV export rounding in the independent
oracle after the tree exposed a quantized-key boundary issue.
[Evidence and scope](FULL_FOREST_FOLIAGE.md).


## Bind foliage batches to source overrides and qualify the assembled subset

2026-10-20: group exact source mesh/material references after independent whole-
plan validation; require raw source instance overrides and highest-detail LOD.
Retain unresolved categories explicitly. Deduplicate standard shared images and
independently check every final source pose/material/attribute. Repair only uniquely
defined zero derived tangents and bounded quaternion rounding; source geometry
remains unchanged. Standard Bevy MikkTSpace handles absent streams with documented
glTF portability warnings. This qualifies a visual subset, not full scene collision
or multiplayer adoption. [Contract and evidence](FOREST_FOLIAGE_BATCH.md).


## Preserve mesh-local geometry separately from imported FBX node placement

2026-10-20: extend the existing opaque Standard profile through exact scene
bindings. Bake file-axis/units with Blender, then clear object-node placement
already supplied by source scene matrices. Reject unqualified geometric transforms.
Check canonical positions independently with ufbx. Compare normalized nonzero
source normal directions against unit glTF normals, recording actual nonunit
source samples; preserve geometry/UVs. Reuse terrain-only assembly bases when
batched props already include ruins. Remaining variants/shaders and authored object
collision still precede adoption. [Evidence](FOREST_STATIC_BATCH.md).


## Qualify explicit single-model variants without inventing imported root poses

2026-10-20: opt in to the existing reviewed variant compiler only for the canonical
Standard path. Require inherited IDs, explicit nested materials and complete outer
source TRS; independently derive raw renderer/mesh/material references and verify
all source-category poses. Reuse prior content only after source/final-scene/hash
checks, in a new batch with recorded input hashes and unchanged prior bytes.
General nested prefabs remain outside this profile. Full forest collision,
materials and adoption remain open. [Evidence](FOREST_VARIANT_GEOMETRY.md).


## Preserve world-projected material behavior through the standard PBR extension

2026-10-20: the pinned triplanar source requires world-space projection that ordinary
mesh UVs cannot preserve across instances. Use Bevy ExtendedMaterial rather than a
new renderer or per-instance resource-heavy baking. Preserve signed projections,
epsilon blend and explicit texture/normal sampling policy. Qualify native colour/
normal diagnostics against independent camera rays/source pixels. Keep this bounded
origin-zero, one-profile inspector path separate from normal chunk/rebase adoption.
Treat large uncalibrated mesh/root-pivot offsets as an importer-reference gate;
do not invent centering corrections. [Evidence and limits](FOREST_TRIPLANAR.md).

## Forest source sky — 2026-10-21

Forest sky continuation, 2026-10-21: the source skydome now joins the qualified
subset: **18,651 source draws / eight unresolved**. Actual Editor geometry, source
world-height gradient and placement pass independent checks. Standard unlit and
explicit Bevy shadow markers pass four native Vulkan cases; three legacy modes,
two native failure paths and encrypted town/farmhouse regressions pass. Fresh fmt,
denied-warning Clippy, 38 client Rust and 193 Python tests pass. Three Khronos
files have zero errors/seven retained warnings. Seven water draws and one emissive
willow remain, followed by lighting, rebase/resources/mips, collision and adoption.
All work stays local. [Evidence](FOREST_SKY.md).

## Forest static water — 2026-10-21

Forest water continuation, 2026-10-21: four bindings / seven source water draws
join the qualified visual subset: **18,658 source draws / one unresolved**. Actual
Linux Unity proves built-in Plane identity/geometry; exact Editor FBX triangles,
raw source poses and explicit static PBR factors pass independent checks. Full
composition, eleven native Vulkan cases and four rehashed faults pass. Ten Khronos
files have zero errors/seven retained warnings. Fresh fmt, denied-warning Clippy
and 197 Python tests pass; runtime binaries remain unchanged. Emissive willow,
source water effects, lighting, rebase/resources/mips, collision and adoption remain
open. All work stays local. [Evidence](FOREST_WATER.md).

## Forest emissive/static coverage — 2026-10-21

Forest emissive continuation, 2026-10-21: the willow completes **18,659 planned
static source draws / zero unresolved static draws**. Source mask HDR emission
uses standard glTF strength 2.6 with an explicit base-floor preview omitting spatial
colour noise. Exact source/Editor geometry, full composition, seven native Vulkan
cases, four rehashed faults and encrypted town/farmhouse regressions pass. Fresh
fmt, denied-warning Clippy, 38 client Rust and 201 Python tests pass; four Khronos
files have zero errors/seven retained warnings. Source effects/particles, lighting,
resources/rebase/mips, object collision and normal map adoption remain open. Next:
qualify shared images against the measured duplicate-image inventory. All work
stays local. [Evidence](FOREST_EMISSIVE.md).

## Forest shared image resources — 2026-10-21

Forest resource continuation, 2026-10-21: the complete static scene now shares
**37 actual core image handles instead of 77** across 298 slots. Loaded CPU Image
payloads reduce 3,124,764,672 → 1,421,877,248 bytes; both paired native views are
pixel-identical outside the HUD. Complete 944-accessor/material/PNG/role/sampler
checks, four rehashed faults and three legacy modes pass. Fresh fmt, denied-warning
Clippy, 38 client Rust and 202 Python tests pass; Khronos has zero errors/seven
retained warnings. All 18,659 static source draws remain covered. This is inspector
sharing; package admission stays unchanged. Lighting, authored collision, rebase/
mips and runtime adoption remain open. All work stays local.
[Evidence](FOREST_RESOURCES.md).

## Forest source lighting boundary — 2026-10-21

Forest lighting continuation, 2026-10-21: actual Unity imports qualify source
renderer layers and cookie alpha; all 26 offline source lights are preserved.
Native pixels expose unsupported selective receiver masks and a disabled cookie
feature. The client enables standard Bevy light textures and rejects selective
masks; a separately verified inspector copy explicitly omits three masked suns,
retaining 22 point lights, one spotlight, ambient and fog. Two full-scene and two
cookie GPU cases, three legacy modes, three failure paths and encrypted town/
farmhouse regressions pass. Four rehashed faults reject. Fresh fmt, denied-warning
Clippy, 41 client Rust and 204 Python tests pass. This is an explicit lighting
approximation; selective suns, authored collision, rebase/mips and normal forest
adoption remain open. Next: source collider geometry and measured package capacity.
All work stays local. [Evidence](FOREST_LIGHTING.md).

## Forest authored collision reference — 2026-10-21

Forest authored-collision reference, 2026-10-21: actual Linux Unity now covers
all 79 raw prefab mesh references, including 39 new collision FBXs and 17 exact
serialized meshes. The actual scene and independent raw hierarchy agree on all
2,776 collider components, with 2,763 active/enabled and maximum matrix difference
0.046 mm. Native targeted rays provide 16,578 reference cases. Four rebound
pose/mesh/state/capsule faults reject; 213 Python tests pass. The invisible downward
water collider, nonuniform primitives, mirrored/sheared meshes and full terrain
budgets remain explicit constraints. Next: native Avian object-query comparison
and measured representation/capacity. Runtime object admission and forest adoption
remain open. All work stays local. [Evidence](FOREST_COLLIDER_REFERENCE.md).

## Forest Avian source geometry — 2026-10-21

Forest Avian geometry continuation, 2026-10-21: a real headless Bevy/Avian
world matches 2,762 active source object colliders across 16,572 targeted rays,
maximum distance difference 0.27 mm. A float64 source-surface oracle separately
proves all 17 convex inputs are closed hulls. Four rehashed expected-hit/distance/
normal/mesh faults reject in actual Avian. Strict Unity ray equality remains a
recorded diagnostic failure because cooking, backfaces and sheared-mesh behaviour
differ; it is not promoted as parity. Fresh format/diff checks, denied-warning
workspace/all-target Clippy and 215 Python tests pass. Terrain, package budgets,
water policy, movement/camera and forest adoption remain separate gates. Next:
measure the source representation against existing runtime package limits. All
work stays local. [Evidence](FOREST_COLLISION_GEOMETRY.md).

## Forest full-sample heightfield prerequisite — 2026-10-21

Forest heightfield continuation, 2026-10-21: standard Avian full/adjoining
heightfields retain every source sample and oriented triangle surface. All
2,362,371 source-vertex/centroid queries and 54 capsule seam route runs at 20/30/60
Hz pass against retained triangle packages; maximum heightfield error is 7.16
micrometres. Rectangular axis/diagonal checks pass; two rehashed sample/scale faults
reject. Fresh fmt, denied-warning workspace/all-target Clippy and 217 Python tests
pass. Offline collision-only layout measures at most 47,245 pooled input triangles,
259,623 instantiated triangles, 66,049 height samples per provider and eight
contributors per finite terrain cell. These are not runtime capacity/publication
claims. Next: bounded versioned shape/heightfield admission; existing packages and
defaults remain preserved. All work stays local. [Evidence](FOREST_HEIGHTFIELDS.md).

## Bounded static-shape admission — 2026-10-21

Static-shape admission continuation, 2026-10-21: collision schema 4 now uses
standard Avian shared mesh inputs, oriented boxes, capsules and full-sample fields
through the normal package/client/server paths. Complete source binding covers
1,731,286 mesh vertices and 263,169 unique heights; all 804,029 source object/
terrain rays pass. Four actual Avian faults and five independent geometry faults
reject. Original encrypted two-client, ±128-billion-metre contributor, town and
farmhouse cases pass on Linux RX 7900 XTX with Vulkan validation. Fresh format,
denied-warning Clippy, complete default network suite, 41 client and 226 Python
tests pass. Existing budgets/defaults remain preserved. Forest visual/material
publication, rebase/resources, water policy and actual-map adoption remain open.
All work stays local. [Contract and evidence](STATIC_SHAPES.md).

## Forest complete reference packaging — 2026-10-21

Forest package continuation, 2026-10-21: ten source-preserving candidate packages
retain all 18,660 draws, 18,925 primitive instances, 6,137,378 triangles and complete
object/full-sample collision, with at most eight cell contributors. Independent
stream/pose/material/collision checks and four rebound faults pass; actual package
loading plus all 804,029 source rays pass. Ten Khronos files have zero errors and
22 retained generated-tangent warnings. Standard unlit/HDR-emissive shared-image
admission also passes native encrypted material/town/farmhouse cases, ten semantic
faults, the complete default network suite and 41 client tests. Fresh fmt/diff,
denied-warning Clippy and 227 Python tests pass. No forest session/default is
published; custom material rebasing/admission, flags/lighting, resources/mips,
water and actual map movement/camera remain open. All work stays local.
[Packages](FOREST_SHAPE_PACKAGES.md), [standard materials](STATIC_MATERIALS.md).

## Forest source texture phase — 2026-10-21

Forest texture-phase continuation, 2026-10-21: the standard Bevy material
extension now accepts a bounded source translation and uploads only its periodic
texture phase. Fifteen native Vulkan views pass: 1,056 independent source-pixel
checks, nine paired colour/normal/lit cases with at most one RGB8 level difference,
a deliberately wrong origin rejected, and both complete forest views pixel-exact
outside HUD. Three translated GLBs have zero Khronos errors/warnings. Fresh fmt/
diff, denied-warning Clippy, 42 client and 218 asset Python tests pass. This is
bounded inspection qualification; normal-world automatic frame updates and bound
runtime material/image admission remain next, followed by flags/lighting,
resources/mips, water and actual-map movement/camera/adoption. All work stays local.
[Evidence and scope](TRIPLANAR_PHASE.md).

## Bound runtime projection materials — 2026-10-21

Forest runtime-material continuation, 2026-10-21: package schema 3 now binds
self-contained projection parameters, global f64 source anchors and canonical
PNG dependencies; a saved older reader rejects it before startup. The normal
client gates root readiness, shares actual materials/images and updates bounded
phase uniforms with live render rebasing. Six encrypted two-client fixture views
at ±128 billion metres pass, including 828 independent source pixels (colour ≤1
RGB8 level, normals exact). Five native failure paths, fresh town/farmhouse and
three retained handover cases pass. Position-based gait diagnostics preserve the
existing movement/animation acceptance checks. Complete default network tests,
44 client tests, 221 asset Python tests, fresh fmt/diff and denied-warning Clippy
pass. This qualifies the original runtime slice; forest embedding, actual forest
resources/rebase, flags/lighting/mips, water and movement/camera/adoption remain
open. All work stays local. [Contract and evidence](RUNTIME_MATERIALS.md).

## Complete forest runtime candidate — 2026-10-21

Forest normal-runtime continuation, 2026-10-21: ten bound material packages
preserve complete source geometry/collision; independent comparisons, four
rehashed faults, all 804,029 source rays and glTF validation pass. A separate
candidate excludes exactly one invisible water collider. Two real encrypted
Linux/Vulkan clients load the full scene, share four projection materials and
47 actual images, and pass visibility/readiness/rebase checks. Per-client base
RGBA CPU image data is 1,589,649,408 bytes (not total memory). The actual-map
survey records 27 clear starts and 41 rejected routes; movement, lighting,
mips/resources, water and broader camera/adoption remain open. Town stays the
default. All work stays local. [Evidence and limits](FOREST_RUNTIME.md).

## Interactive performance gate — 2026-10-22

P1-12 / P2-05 / P2-08 / P2-10. Require representative optimized, native frame-time
measurements before scene adoption. Correctness runs with validation, functional
screenshots and asset counts do not establish acceptable performance. Preserve
legacy camera regressions and use the existing accelerated CAMERA query geometry
for normal chunk worlds. Keep explicit debug/visual-refinement modes for diagnosis.
Current measured scope and pending checks are in [the performance record](FOREST_PERFORMANCE.md).


## 2026-10-29 — Initial transient periodic status effects (P3-06)

Scorch is opt-in development data, not final class/story balance. Apply only at
validated cast completion; charge mana at actual start. Keep one instance per
source/target/definition; refresh retains the next nominal tick. Bound sixteen
instances world-wide/eight per target and recheck capacity at completion. First
hit follows one interval; an exact-expiration tick is included. Fixed simulation
ticks determine damage, independent of rendering/wall time.

For this prototype, source departure cancels future ticks; target death/removal
clears all its statuses, and fresh respawn carries none. No durable/offline combat
credit is implied. Preserve CS01/CS02 resource observations; SS01 separately sends
bounded status rows with admitted catalog identity and explicit authority tick
rate. Client labels are observations only. Initial native Linux slice passes;
[scope and evidence](STATUS_EFFECTS_FOUNDATION.md). Follow-on creature AI must define
its navigation/steering prerequisite rather than treating task list order as proof
of available pathfinding.


## 2026-10-30 — Initial creature patrol consumes offline routes (P3-07 / P4-03)

Use source/profile/hash-bound precomputed local navmesh routes for the first
nonblocking creature. Keep generation/query libraries outside the authority's
normal dependency graph and all path search/file I/O outside the fixed tick.
Existing Avian static collision decides movement; missing/blocked paths stop,
with no straight-line fallback. Keep home collision resident independently of
player travel. Death pauses the patrol; fresh runtime respawn returns home and
restarts its wait. Moving-target visuals reuse existing interpolation with bounded
normal catch-up, while server positions drive combat. The tested initial slice
passes; general chasing requires separately bounded worker queries and generation
fences. [Scope and qualification](CREATURE_AI_FOUNDATION.md).


## 2026-10-30 — Initial main-hand equipment and public appearance (P3-08)

Keep backpack and main hand in one bounded state/revision/transaction. Equip
replaces by returning the displaced item to its source slot; unequip requires an
empty explicit destination. Schema 2 requires its nullable main-hand field, checked
nonstackable eligibility and appearance ID; preserve schema-1 behavior and bytes.
Do not migrate/reseed existing characters on catalog mismatch. Gear grants no
statistics or damage in this first slice.

IR02/IS02 explicitly version equipment/private item observations. EA01 separately
publishes only relevant runtime actor/appearance IDs, with epoch/catalog checks;
private contents/revisions remain private. Both kinds of item request share one
pending stream and durable receipt boundary. Standard Bevy primitives parent to
the inspected POLYGON Hand_R skin joint; source scale/joint mismatches are explicit
errors, and no cosmetic prop becomes a collider or movement authority. Empty HUD
layout containers pass pointer input through while child controls remain pickable.
[Qualified behavior, driver/source/binary evidence and limits](EQUIPMENT_FOUNDATION.md).


## 2026-10-31 — Initial animated creature presentation (P3-14)

Use the supplied MeshTint wolf's own 41-bone rig and six clips with standard Bevy
assets/animation; the player retains its existing POLYGON rig. Bind this opt-in art
to existing authority observations, with no art-driven collision or damage. Keep
sixteen bounded presentation cues, visual corpses for at most fifteen seconds,
fresh visual state per respawn key, and a 200 ms gait hold for short replication
gaps. Owner-only windup remains owner-only; public NPC attack timing is separate
future work. Read captured dimensions when qualifying ultrawide windows, and wait
for opened/resized UI layout to settle before scripted clicks/bounds audits.
[Qualified behavior, evidence and limits](WOLF_CREATURE.md).

## 2026-10-31 — Preserve authored prop sockets independently of body deformation

The POLYGON combat development conversion retains both prop leaf bones even
though neither carries body skin weights. Equipment uses the authored `Prop_R`
origin/axes; the existing 50-bone rig keeps its explicit legacy wrist fixture.
Do not treat zero deform weights as evidence that an attachment bone can be
removed. Standard Bevy animation presents the owner's authoritative death and
fresh life without adding gameplay movement or exposing private remote health.
This remains an opt-in development conversion with a failed production 5 mm
four-weight fidelity gate. [Behavior and actual qualification](PLAYER_ANIMATION_SOCKETS.md).

## 2026-10-31 — Public equipped stance and masked moving finger grip

For the development POLYGON rig, select standing SwordIdle from the exact
server-owned public actor/epoch/sword appearance. While moving, use standard Bevy
masks for the eleven authored right-hand digit poses, preserving the original
wrist/socket/body gait. Pending/private UI selections are not equipment authority.
Wand replacement/unequip removes the layer; death overrides it. Keep independent
optional clip slots and the original rig fallback. Full upper-body mobile combat
and authored equipped locomotion remain separate work. [Qualification](EQUIPPED_PLAYER_STANCE.md).

## 2026-10-31 — Reuse reviewed static query behind one bounded path worker

For the P3-07/P4-03 pursuit prerequisite, share the existing Landmass 0.9.3 query
adapter between the offline baker and the network crate. Keep the search outside
the simulation, with one outstanding request/reply and bounded local package,
route, lifetime and exact identity/revision checks. No package or engine version
changes. Landmass has no hard CPU deadline; expired results do not publish, but
expiry does not cancel searches. No NPC dynamic-path consumer or streamed-world
navigation is implied. [Implemented contract and qualification](CREATURE_PATH_WORKER.md).

## 2026-10-31 — Adopt initial pursuit in an opt-in development bundle

Use server-owned perception/LOS and exact target life, a fixed captured patrol
anchor for leash/return, and saved route/wait progress. Run checked local paths
on the bounded worker and retain collision/grounding authority. Return cancels
attacks; production threat/evade balance and streamed navigation remain open.
Keep old profile bytes/checkpoints and reject incomplete life/retaliation setup.
[Final playable qualification](CREATURE_PURSUIT_NATIVE.md). New player melee,
projectiles, outcomes, quest markers, minimap and HUD work remains in the follow-on
register; P3-17/P3-18 remain unspecified.

## 2026-11-03 — Confirm encrypted Renet readiness before World admission

Adopt transport revision 1 with exact NC01 reliable preface, eight bounded
five-second waiters and no game traffic/World actor/account consumption before
confirmation. Dropping renetcode 2.0.0's initial KeepAlive reproduced the earlier
Welcome timeout independently of endpoint reuse; continuous game payloads
suppressed its library retry. Keep pinned libraries, crypto and five-second
client deadline unchanged; version the Renet protocol ID separately from movement
wire/Quinn 12. Existing ticket/account/terrain checks follow confirmation.
Both Renet ends need rebuilding and fresh tickets.

Full 1,128-test workspace, strict Clippy/formatting, two focused loss-policy
checks, 48 baseline/seeded repeated cases and normal OIDC/PostgreSQL/native
quest/restart rejoin pass. Preserve the separate 46/48 periodic-loss diagnostic:
it can erase essentially all snapshots after Welcome. Do not call that universal
reconnect recovery or silently replace its evidence with seeded loss.
[Contract and exact limits](RENET_CONNECTION_CONFIRMATION.md). Local only;
`build/connection-confirmation-client-qualified-v1-20261103`.

## 2026-11-03 — Build a bounded local map from resident render grids

Adopt the initial Classic graybox minimap through standard Bevy UI: north-up
256-metre square, 64x64 terrain image at up to four refreshes/second and a 16x16
manual-heading arrow. Reuse resident mesh data and committed owner/guide filters;
do not read saved terrain, force chunks, add a map camera or create gameplay
requests. Keep health/mana beside it and hide it while the right-hand backpack
occupies the space.

All 1,133 workspace checks (50 skips), strict Clippy/fmt and four normal native
quest/restart starts pass across the three supported sizes with zero GPU
diagnostics. Measured raster maximum 329 us; visible idle refresh leaves
81 tiles / one patch / 90 saved reads unchanged. Scope is the Classic greybox;
legacy heading, native seam traversal and broader map layers remain follow-ups.
[Exact evidence and limits](MINIMAP.md). Preserve
`build/minimap-client-qualified-v1-20261103`; changes remain local.

## 2026-11-03 — Compare map coordinates with displayed presentation

Retained walking without explicit facing uses the existing displayed character
rotation for its arrow; do not change its gameplay or the adopted Classic policy.
Qualification must compare the map with rendered player coordinates, separately
from delayed authority, and distinguish sampled frame origins from total origin
changes. Preserve original failed checker evidence.

Normal owned-account W traversal crosses three chunk owners and a render rebase
without changing quest rewards. A separate accelerated development-ticket route
covers the 16-km X extent with 125 origin changes and bounded retirement; it is
not normal-speed/OIDC or MMO capacity evidence. Full 1,134-test workspace,
strict Clippy/fmt and five native starts pass.
[Exact evidence and scope](MINIMAP.md). Preserve
`build/minimap-traversal-client-qualified-v1-20261103`; changes stay local.

## 3 November 2026 — version-bound Classic ground movement calibration

Implement the documented Classic travel follow-up under a new ground-movement-v3
world identity, including exact embedded speed-data bytes. Adopt explicit Theia
mapping of one source gameplay movement unit/s to one m/s: walk 2.5, run 7,
backward run 4.5, with normalized diagonals and backward selection from accepted
manual character facing. The pinned Blizzard UI supports the base-7 reference
and walk/run toggle; pinned CMaNGOS Classic supplies supporting directional
speed evidence with its uncertainty retained. This is not proof of complete
Blizzard movement parity. See [source/version/mapping detail](CLASSIC_GROUND_MOVEMENT.md).

Controls schema 2 uses a configurable walk/run toggle, default NumpadDivide,
and running by default. Shift is no longer a Classic travel boost. Legacy
non-Classic development movement and immutable earlier binaries retain 5/8 m/s.
Old controls schema is explicitly refused; client/server must rebuild together
and obtain fresh tickets. No movement layout, transport revision, physics pin,
world scale, NPC speed or dodge-policy change. Implementation qualification
remains pending; preserve the prior qualified playable.

Ground movement qualification completed: all 1,139 workspace tests / 50 skips,
formatting and strict Clippy; 27 native speed checks at three resolutions plus
normal unmodified wolf/quests, database restart/rejoin and streamed W traversal.
Five final native starts have zero Linux GPU diagnostics. Exact checkpoint:
`build/classic-ground-client-qualified-v1-20261103`. The earlier interrupted
run and measurement corrections remain documented, not promoted to passes.

## 3 November 2026 — directional locomotion art prerequisite

Use the supplied non-root-motion Base Locomotion clips as optional standard GLB
animation, retaining the exact existing skin, sockets and eight clips. Fourteen
additional walk/run directions pass source/retained-content checks and native
inspection; do not infer live directional gameplay from converted art.
[Exact proof and pending selection](CLASSIC_DIRECTIONAL_LOCOMOTION.md).
The 5 mm production-fidelity gate remains failed; the measured four-weight
preview limitation is unchanged. Preserve the ground-v3 playable until runtime
selection, priority, grip and local/remote checks pass. No dodge work is resumed.

## 3 November 2026 — optional Classic directional selection qualified

Use checked global interpolation segments relative to displayed manual heading;
never turn the actor toward motion. Complete optional fourteen-direction art
selects eight-way walk/run, including backward running at the shared 4.5 m/s
rate. Retain old-art/profile fallback and the deferred dodge slot namespace.
Preserve compatible normalized moving-clip phase; crossfades/stride calibration
remain separate. Existing standard Bevy finger/upper-body masks cover all bases;
life/critical pose and attack barriers retain priority.

[Runtime evidence and exact fixture distinctions](CLASSIC_DIRECTIONAL_LOCOMOTION.md):
1,145 workspace tests / 50 skips, strict Clippy, two-account source-pose checks,
new-art ultrawide controls, normal unmodified new/old-art quest and restart/rejoin
flows, and actual new-art streamed traversal. Seven final native starts have
zero Linux GPU diagnostics. Preserve
`build/classic-directional-client-qualified-v1-20261103`; do not infer full
animation, skin-production fidelity, Windows or population capacity.

## 12 November 2026 — bounded larger starter-zone authoring

Following the user's request for a village and distinct quest outskirts, extend
the finite editor height patch to 512 m using schema-11 chunk spans. Reuse height
samples, brushes, standard Bevy meshes and actual static collision; do not infer
MMO streaming from this authoring footprint. Bound grids at 129² samples and
encoded documents/command inputs at 2 MiB (the current painted valley is 1.27 MB).
Keep two-metre spacing for patches up to 256 m; reject oversized resampling with
source/history intact. Preserve the smaller existing scenes and explicit Save.
Actual quest contacts, wolf patrols, navigation residency and authored spawn
selection need their own runtime slices before this becomes a connected zone.
