Zone authoring workspace, 8 November 2026 (P5-17 / P3-15 / P1-10; P2-06/07):
H and structured `project_to_surface` now ground selected objects, plants,
markers and guide points through shared candidate validation and undo. Primitive
centres account for half-height; foot anchors use the exact terrain support.
Whole guides are one undo entry. The summary also distinguishes saved content
from unsaved visual/agent edits and pending drafts. This grounds anchors rather than prop stacks
or full slope footprints. Tab/Shift-Tab also cycle hidden/distant objects; F
frames objects, plants, markers and the existing guide/point selection without
changing history. Committed guides also insert/remove points through Insert /
Backspace and narrow shared commands; structural changes clear stale point
selection while visual edits explicitly select the new/next point. A standard-Bevy
right-side content browser also selects named entries across modes through a
fixed 40-row pool and paging, without changing document/history. Both side
panels are excluded from world mouse actions; the list hides in playtest. F8 /
Shift-F8 also rename selected content / the zone through standard Bevy TextInput
and narrow shared commands, with modal input isolation and one-entry undo. Names
retain existing UTF-8 byte validation; geometry/definitions are unchanged.
Local/far/reopen name-dialog native checks pass with zero GPU diagnostics.
Flatten also samples the surface with I and adjusts its local
level in 1 m / 0.1 m steps, sharing exact visual/headless stamp output.
Ctrl-D and structured `duplicate` commands copy any authoring kind with fresh
shared IDs, bounded names/offsets and one-entry history. Local/far/empty native
copy/framing and positive/negative flatten cases pass. Raise/lower strength also
halves/doubles within 0.05..8 m, with exact visual/headless 0.125 m stamp checks
on both grids. Brackets also adjust independent sculpt/paint blends within
5..100%, retaining 50% / 100% defaults and exact 12.5% command equivalence.
`python3 tools/edit_zone.py --example` now opens/reopens a separate
Wayfarer Hollow draft from an original four-transaction recipe. F also frames
the terrain patch. Its graded walking route and shelter pass actual Avian checks;
local/far 3440x1440 editor runs and two real Renet client pairs pass with zero GPU
diagnostics, including far origin Y=20. Markers remain authoring notes and the
stream remains a visual guide. 65 focused Rust cases, seven launcher cases, formatting
and strict workspace/all-target Clippy pass. The full native 3440x1440 walkthrough also passes
with Vulkan validation active and zero GPU diagnostics; [contract and evidence](../ZONE_EDITOR.md#wayfarer-hollow-example-and-terrain-framing--8-november-2026).
Local commits only; no push. Chat composer/bubbles remain deferred.

Current focus, 8 November 2026 (P5-17 / P3-15 / P1-10; terrain P2-06/07):
The user prioritizes terrain, greyboxing and zone authoring and defers chat
composer/bubbles. The standard Bevy editor now has shared validated visual/agent
history, primitive transforms/snapping, bounded terrain brushes, road/stream
guides, NPC/quest annotations, greybox vegetation/scatter, save/load, offline
playtest and export into the retained Renet world-package path. Actual prerequisites
are the pinned stack, checked coordinates and existing query/package pipeline;
later waves and P5-16 are not blanket gates. Guide-driven terrain grading also shares validated commands and undo.
Individual guide points also share validated moves, visual picking and camera focus.
Terrain colour painting adds shared preview/history and standard vertex-colour export.
A one-command launcher reopens drafts and defers initial writes until explicit Save.
Optional 2 m terrain refinement/coarsening also shares validation/history and export.
Current qualification is 50 focused
Rust cases, strict workspace Clippy, default-fullscreen 3440x1440 OS-input checks
and actual exported-package encrypted client pairs. The editor targets only
3440x1440 under the user's current direction. Local commits only; no push.
[Exact contracts, acceptance evidence and remaining scope](../ZONE_EDITOR.md).
Older task/queue entries below retain their historical scope; P3-17/18 remain
unspecified. Live NPC/quest publication, general vegetation assets/behavior,
terrain grading/materials and broader terrain authoring remain separate work.

# Theia MMO — Integrated Engineering Implementation Plan

**Current initial control/combat scope, 1 November 2026:**
[THEIA-COMBAT-001](../COMBAT_TARGETING_MOVEMENT_SPEC.md) and the
[migration slices](../MOVEMENT_CAMERA_TARGET.md) supersede earlier hybrid/dodge
priorities. Reuse existing task IDs and actual prerequisites. Defer dodge/action
camera; establish classic controls, authoritative manual facing, persistent
selection and weapon-timed autoattack for the one-hostile milestone.

Approved six phases, **112 planned stages**, ten overlapping waves and twelve continuous workstreams. This is a reconstructed actionable plan from the voice discussion, **not a verified repository audit**. Phase 3 originally listed 18 stages but only 16 were individually named; placeholders are explicitly marked. Phase 4–6 numbering below is a proposed normalization of the approved scope, not a verbatim previously approved numbered list.

## Phase 1 — 12 stages
- **P1-01** Repository audit and reusable/obsolete classification
- **P1-02** Rust workspace, Bevy client, headless server and shared crates
- **P1-03** World coordinates and spatial conventions
- **P1-04** Entity identity and lifecycle
- **P1-05** 30Hz fixed timestep, bounded input queues
- **P1-06** Command schemas, sequencing, acknowledgements and loss tests
- **P1-07** Renet integration and validation (replaces comparative transport study)
- **P1-08** Replication protocol contracts
- **P1-09** Kinematic movement and collision contracts
- **P1-10** Asset/terrain/animation content contracts
- **P1-11** Data-driven ability/combat schemas
- **P1-12** Tests, instrumentation and benchmark baselines

## Phase 2 — 14 stages
- **P2-01** Bevy client bootstrap
- **P2-02** Connection/session management
- **P2-03** Entity replication and lifecycles
- **P2-04** Prediction and reconciliation
- **P2-05** Third-person camera/controls
- **P2-06** Terrain chunk loading
- **P2-07** Collision integration
- **P2-08** GLB and asset loading
- **P2-09** Character skeletal animation
- **P2-10** Lighting/materials/atmosphere
- **P2-11** Targeting and interactions
- **P2-12** One authoritative ability, damage, death and respawn
- **P2-13** Spell VFX and audio hooks
- **P2-14** Health/target/action-bar UI and integrated demo

## Phase 3 — 18 stages
- **P3-01** Authentication and sessions
- **P3-02** PostgreSQL persistence/migrations
- **P3-03** Character creation/selection
- **P3-04** Stats and resources
- **P3-05** Inventory and items
- **P3-06** Abilities and status effects
- **P3-07** Creature AI
- **P3-08** Equipment and appearance
- **P3-09** Loot and rewards
- **P3-10** Experience and leveling
- **P3-11** Quest objectives
- **P3-12** NPC dialogue and vendors
- **P3-13** Terrain streaming and initial LOD
- **P3-14** Animation, VFX and audio expansion
- **P3-15** Content authoring and validation
- **P3-16** One complete persistent zone and login/logout proof
- **P3-17** UNSPECIFIED in original transcript: needs reconciliation
- **P3-18** UNSPECIFIED in original transcript: needs reconciliation

## Phase 4 — 20 stages
- **P4-01** World region ownership
- **P4-02** Predictive terrain/object streaming
- **P4-03** Streamed navigation and spatial queries
- **P4-04** Advanced interest management
- **P4-05** Adaptive replication
- **P4-06** Cross-region handoff
- **P4-07** Population and overload management
- **P4-08** Advanced combat interrupts/CC/AoE/healing/threat
- **P4-09** Classes and specializations
- **P4-10** Group encounters
- **P4-11** Parties and group rules
- **P4-12** Chat/moderation
- **P4-13** Friends and presence
- **P4-14** Guild persistence
- **P4-15** Secure trading
- **P4-16** Animation LOD and crowd policies
- **P4-17** VFX density budgets
- **P4-18** Time/weather integration
- **P4-19** Multi-zone scale profiling
- **P4-20** World and multiplayer integration gate

## Phase 5 — 24 stages
- **P5-01** Rendering benchmarks
- **P5-02** Lighting and shadows
- **P5-03** Atmosphere and fog
- **P5-04** Stylized materials
- **P5-05** Water
- **P5-06** Terrain LOD and seams
- **P5-07** Streaming memory budgets
- **P5-08** Vegetation instancing and wind
- **P5-09** Visibility/culling
- **P5-10** Animation blending and layers
- **P5-11** Equipment visuals
- **P5-12** Animation LOD
- **P5-13** Character readability
- **P5-14** Particle expansion
- **P5-15** Projectiles/AoE effects
- **P5-16** VFX priority and density limits
- **P5-17** World editor
- **P5-18** Creature/item/ability/quest editors
- **P5-19** Asset conversion and license validation
- **P5-20** Developer inspection tools
- **P5-21** Client performance tuning
- **P5-22** Server profiling
- **P5-23** Deployment/monitoring/recovery
- **P5-24** Integrated visual and load validation

## Phase 6 — 24 stages
- **P6-01** Release scope and quality gates
- **P6-02** Minimum playable content
- **P6-03** Internal playtests/defect tracking
- **P6-04** Class/combat balance
- **P6-05** Progression and economy validation
- **P6-06** Quest/encounter chains
- **P6-07** Onboarding/accessibility
- **P6-08** Closed alpha accounts/environment
- **P6-09** Installer/patcher
- **P6-10** Closed alpha stabilization
- **P6-11** Concurrent gameplay load test
- **P6-12** Crowded encounter test
- **P6-13** Recovery drills
- **P6-14** Security/exploit testing
- **P6-15** Closed beta population
- **P6-16** Long soak testing
- **P6-17** Production deployment/support
- **P6-18** Moderation/support workflows
- **P6-19** Release candidate validation
- **P6-20** Operational readiness review
- **P6-21** Go/no-go
- **P6-22** Final load test
- **P6-23** Launch
- **P6-24** Launch stabilization

## Ten approved overlapping execution waves
- W1: Phase 1 foundations and early rendering/terrain/animation/persistence spikes
- W2: Phase 2 client/session/replication/prediction foundations
- W3: Phase 2 playable terrain/camera/animation/combat/UI integration
- W4: Phase 3 accounts, persistence, characters, inventory and ability foundation
- W5: Phase 3 full persistent slice, creatures/quests/loot/tooling
- W6: Phase 4 world partition, navigation, interest and scale
- W7: Phase 4 advanced combat, social, guilds, trading and presentation foundations
- W8: Phase 4 completion and Phase 5 visuals, animation, VFX and performance
- W9: Phase 5 tooling/operations and Phase 6 internal alpha preparation
- W10: Phase 6 external tests, hardening, release and stabilization

## Twelve continuous workstreams
Architecture/workspace; simulation/physics; networking/replication; world/terrain/streaming; rendering/vegetation; assets/tooling; animation/VFX/audio; gameplay/combat/AI; persistence/economy; social/UI; security/operations; testing/profiling/release.

## Delivery gates
W1: headless authoritative server, compiling Bevy client, architecture contracts and asset proof. W2: two clients exchange authoritative movement. W3: two players fight one enemy in a rendered world. W4: persistent character and inventory. W5: one complete questing zone survives logout/reconnect. W6: seamless multi-region travel with measured capacity. W7: multiplayer social/group combat. W8: measured visual and simulation performance in crowded scenes. W9: internal alpha and content tooling. W10: closed alpha/beta, security/recovery and launch readiness.

## Dependency rules
Coordinate types before terrain, physics, networking and navigation; scheduler and identities before replication; replication before prediction; ability schema before combat/VFX; character skeleton contracts before equipment/mounts; persistence before transactional inventory/trades; terrain collision before streamed navmesh; region ownership before handoffs; prioritization before crowded raid effects; instrumentation before optimization. Workstreams overlap and no phase is a rigid waterfall.
