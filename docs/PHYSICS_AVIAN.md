# Avian runtime migration — 2026-10-19

The [shared kinematic continuation](KINEMATIC_CONTROLLER.md) now adds graybox
gravity/jumping and advances the active protocol to v7. Normal imported maps
now also use the shared controller with validated spawns and explicit recovery.
The original Avian migration evidence below used v5; coordinate/identity work subsequently used v6.

The normal game client now uses Avian, and the authoritative server uses the same
Avian scene-query backend. This is an active runtime migration. The GLB inspection
mode remains a visual asset viewer and does not establish gameplay collision.

## Pins and features

- Bevy: exactly `0.20.0`.
- Avian: `avian3d` `0.8.0-dev`, Git revision
  `1070759b2145dc078abedace91dc7fe78013a45c`.
- Provenance: upstream [Update to Bevy 0.20](https://github.com/avianphysics/avian/commit/1070759b2145dc078abedace91dc7fe78013a45c).
- `Cargo.lock` fixes the Git source and resolved dependencies; no floating branch
  or vendored engine source is used.
- Avian defaults are disabled; `3d` and `parry-f32` are enabled. The client owns
  renderer/window/UI features. The server uses headless Bevy ECS/math/transform/
  time facilities without `bevy_render`, `bevy_winit` or `wgpu`. Both paths resolve
  one Bevy 0.20.0.

## Runtime

`apps/client/src/physics.rs` installs standard `PhysicsPlugins` and spawns static
`RigidBody`/`Collider` entities for the approved world. Startup logs and diagnostic
captures identify the revision and collider count. A client test queries the actual
Avian collider tree and hits the fixed door.

`crates/network/src/physics.rs` supplies the shared renderer-free scene-query
adapter for server movement, offline client movement and client camera clearance.
It uses Avian plugins, the collider tree, `SpatialQuery` ray/shape/intersection
queries and reusable Bevy query state. Geometry is built once. A short mutex
protects query-state access; no rendering or awaited I/O runs in the simulation step.

Source terrain and approved floor/ramp triangles compile into an Avian triangle
mesh. Environment envelopes become Avian cuboids. Ground height and slope use
Avian rays. Capsule sweeps follow grounded 3D displacement, including ramp height
changes; axis sliding and bounded support probes retain the walking rules. The
hand-written box sweep and runtime terrain grid are removed. Source-coverage
validation remains bounded; barycentric sampling survives only as a test oracle.

Movement uses a 0.5 mm cast skin and suppresses solver-scale contact creep. Contact
assertions and independent native capsule/box verification use a 1 mm tolerance.
The camera casts a conservative 0.3 m cube; Bevy static-mesh ray casts still
supplement clearance for visual geometry absent from the profile.

Only approved world geometry enters blocking queries. Characters remain
nonblocking and receive no dynamic rigid bodies. Layers distinguish movement,
support mesh and camera geometry. Old schema-1 blocking columns retain their
movement contract, with separate finite camera colliders; schema-2 uses actual
envelope heights.

The town profile creates **3,977 static colliders**: 3,972 movement/camera boxes,
four camera-only boxes and one mesh with 65,504 terrain plus 367 support triangles.
The graybox has eight colliders. Counts describe geometry, not capacity.

Wire/ALPN is version **5**, preventing old/new collision backends from connecting.
Rebuild/restart both applications together. Profile schema, visuals and hash checks
remain unchanged. Quinn remains the launcher transport during the separate Renet
session migration.

## Scope and verification

This migrates existing graybox/reference/map profiles. It does not automatically
derive precise collision for every visible WMO/M2, open all buildings, create a
navmesh, or add prediction, gravity, jumping, swimming or platforms. The grounded
walker still chooses the highest approved support surface at an XZ point; general
multi-floor locomotion remains open. The farmhouse/blacksmith fixtures remain
regression inputs for further content/controller work.

Tests cover thin-wall capsule sweeps, retreat and layer filtering; real client
plugin/collider-tree integration; terrain heights/slopes/holes, wall sliding, ramps,
interior access, actor pass-through and actual authoritative QUIC/Renet connections.
Existing configured 20/30/60 Hz and movement-speed checks remain in force.
See [VALIDATION](VALIDATION.md) for executed commands, devices and native evidence.
Native Windows graphics remain separately unqualified.
