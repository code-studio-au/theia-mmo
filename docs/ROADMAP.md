# First playable milestones

These are game outcomes, not a continuation of the old engine's M0–M6 schedule.

## Bootstrap — runnable local client

- [x] Independent Rust workspace and pinned Bevy application.
- [x] Original graybox scene, lit geometry, local movement and follow camera.
- [x] Controls, reset/quit, scripted movement and rendered screenshot path.
- [x] Execute native qualification and record limitations.

## First multiplayer slice — two players walking together

- [x] One local Rust authoritative server with a fixed simulation tick.
- [x] Review/reuse suitable protocol and Quinn code from the old project, preserving
  provenance and relevant tests. Do not copy the whole engine or assume its
  experimental protocol is production-ready.
- [x] Two independently launched clients connect, receive distinct player identities
  and see each other's server-owned movement in the same scene.
- [x] Clients submit bounded input intents; server controls speed, bounds and state.
- [x] Basic remote interpolation, direct local server poses, disconnect/reconnect
  and malformed/stale input tests. Rendered poses agree with server snapshots.
- [x] Repeatable local two-client/server verification with actual GPU screenshots.
- [ ] Local prediction/reconciliation when needed for responsiveness; qualify
  jitter/loss/delay and recovery. Direct server poses currently provide correction.
- [ ] Manual two-client walkthrough on request; current GPU checks use an owned
  private headless display. Automatic session resume is not implemented.

## Representative game scene

- [x] Enable ordinary Bevy glTF loading and an independent bounds/scale inspector.
- [x] Qualify the user's geometry-only house GLB and move it into the repo as a
  compatibility reference. No authored UVs/materials/textures exist in that file.
- [ ] Qualify textured static assets and animation loading with original or
  appropriately licensed character, terrain, building and props.
  Local compatibility checks now render the user's textured farmhouse and character;
  stand/walk/run playback works in the inspector. The character export still fails
  strict glTF validation. Representative game content remains a separate step.
- [x] First explicit reference placement with conservative server collision:
  textured farmhouse inside the shared solid envelope, human players and a
  nonblocking idle wolf placeholder. [Scope and checks](REFERENCE_SCENE.md).
- [ ] Qualify final representative asset placement and colliders; viewing imported
  meshes does not automatically create server collision or usable interiors.
- [x] First static-world collision pass for the flat procedural scene: solid
  building, fixed door and rocks. Rendering and authority share authored geometry.
- [x] Orbit/zoom camera, camera-relative movement and static camera obstruction.
- [ ] Qualify general authored-world collision alongside imported assets; slopes,
  steps, jumping and mesh colliders are not in the flat prototype.
  The Ruins of Lordaeron terrain and 846 top-level placements plus 624 nested props
  now have a qualified [static visual assembly](MAP_OBJECTS.md). Exported liquid
  geometry also has a [static PBR preview](MAP_LIQUIDS.md). Server-owned terrain
  movement and liquid gameplay remain outstanding.
- [x] First movement-driven character animation states: human stand/walk/run for
  local and remote players, alongside the existing controls/connection HUD.
- [ ] Animation blending/combat states and the gameplay target/health/ability HUD.
  Environment geometry stays static, including doors. Players do not physically
  collide with either players or NPCs; targeting/hit detection is separate.
- Define an asset layout and stable game identifiers; avoid custom cooked formats
  until a measured requirement needs them.
- Verify the same small scene and connection flow on native Windows.

## Gameplay loop, then measured scale

Add a server-owned interaction/combat loop, then persistence and accounts when the
playable experience needs them. Increase entities/clients while measuring actual
simulation, replication, asset and rendering costs. Interest management, larger
world streaming and services follow observed constraints. No MMO-scale capacity
is claimed by the bootstrap or first two-client milestone.

The intended game is WoW-style: begin with a targetable NPC, one server-validated
ability, health/cooldown, then one quest and reward before broadening systems.
See [GAME_DIRECTION.md](GAME_DIRECTION.md) for the small starting-zone outcome.
