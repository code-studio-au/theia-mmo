# First playable milestones

These are game outcomes, not a continuation of the old engine's M0–M6 schedule.

## Bootstrap — runnable local client

- [x] Independent Rust workspace and pinned Bevy application.
- [x] Original graybox scene, lit geometry, local movement and follow camera.
- [x] Controls, reset/quit, scripted movement and rendered screenshot path.
- [x] Execute native qualification and record limitations.

## First multiplayer slice — two players walking together

- One local Rust authoritative server with a fixed simulation tick.
- Review/reuse suitable protocol and Quinn code from the old project, preserving
  provenance and relevant tests. Do not copy the whole engine or assume its
  experimental protocol is production-ready.
- Two independently launched clients connect, receive distinct player identities
  and see each other's server-owned movement in the same scene.
- Clients submit bounded input intents; server controls speed, bounds and state.
- Predict the local presentation where needed; interpolate remote state. Test
  correction, disconnect/reconnect and malformed/stale inputs.
- A repeatable local two-client/server verification records actual connections
  and authoritative state, alongside a visible walkthrough.

## Representative game scene

- Qualify Bevy's ordinary glTF/GLB model and animation loading with an original or
  appropriately licensed character, terrain, building and props.
- Add world collision, character animation states and an ordinary game HUD.
- Define an asset layout and stable game identifiers; avoid custom cooked formats
  until a measured requirement needs them.
- Verify the same small scene and connection flow on native Windows.

## Gameplay loop, then measured scale

Add a server-owned interaction/combat loop, then persistence and accounts when the
playable experience needs them. Increase entities/clients while measuring actual
simulation, replication, asset and rendering costs. Interest management, larger
world streaming and services follow observed constraints. No MMO-scale capacity
is claimed by the bootstrap or first two-client milestone.
