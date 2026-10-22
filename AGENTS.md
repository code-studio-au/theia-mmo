# Theia MMO — contributor instructions

Read `docs/STATUS.md`, `docs/DECISIONS.md` and `docs/ROADMAP.md` before editing.
For feature work, read `docs/engineering/README.md`, `ARCHITECTURE.md`,
`IMPLEMENTATION_PLAN.md` and `CODEX_TASKS.md` under `docs/engineering/`, plus the
current alignment record. The user adopted this architecture on 2026-10-19.
The imported `AGENTS.reference.md` remains provenance; these are the active rules.

- Build a playable MMO using Bevy. Prefer its standard renderer, assets, UI and
  animation over custom engine infrastructure unless measured game requirements
  justify an exception.
- Rust client and authoritative server; retain Go for services when needed.
  Server owns gameplay truth. Offline client movement is a temporary sandbox,
  not a trusted network protocol or production movement authority.
- Linux primary, Windows first-class. Use Bevy's platform abstractions; distinguish
  Linux execution, Windows cross-builds and native Windows execution in evidence.
- Target Renet with authenticated/encrypted netcode transport initially; Quinn is
  the existing prototype baseline and a fallback only after documented validation
  failure. Target Avian initially, Rapier only after a documented need. Verify
  Bevy/plugin compatibility before pinning; do not silently mix engine versions.
  Current physics pins are Bevy `=0.20.0` and Avian `0.8.0-dev` revision
  `1070759b2145dc078abedace91dc7fe78013a45c`; see `docs/PHYSICS_AVIAN.md`.
- Use checked f64 global positions, bounded f32 local rendering/physics frames,
  Y-up metres, integer chunk/region IDs and centralized origin conversion.
- World collision is predominantly static. Scripted authoritative elevators and
  moving platforms are allowed by the adopted architecture; vegetation wind and
  bending are presentation effects. Buildings, doors and ordinary props remain
  static unless separately specified. Players never physically collide with other
  players or NPCs. World collision, navigation and gameplay hit detection are
  separate requirements; nonblocking NPCs can still be targeted and hit.
- Keep the server headless and modular initially, with configurable 30 Hz fixed
  simulation, bounded input/network queues, independent rendering and no database
  or awaited transport I/O in the simulation step. PostgreSQL persistence is async;
  critical economy transactions must be durable before success acknowledgement.
- Build data-driven tab-target combat with mobile casting and standard Bevy
  presentation. Protect critical telegraphs/audio under cosmetic load. Follow the
  architecture for terrain chunks, navigation and eventual region ownership;
  completed experiments do not establish streaming or MMO-scale capacity.
- Keep this repository independent of `theia-engine`. Adapt reviewed components
  selectively, with provenance and tests; never introduce a path dependency on it.
- Complete small playable slices. Do not add distributed services, custom ECS,
  renderers or elaborate streaming systems before a game requirement needs them.
- Identify Pn-nn task IDs, actual prerequisites and concrete acceptance checks per
  slice. The waves overlap; imported list-order dependencies are not rigid gates.
  Keep P3-17/18 unspecified until the user supplies their scope. Preserve working
  experiments until replacement slices pass. Do not expand bespoke house fixtures
  as the general collision solution.
- Record implemented behavior separately from plans. Never equate proxy counts,
  local movement or mocked connections with working multiplayer.
- Run `cargo fmt --check`, denied-warning Clippy and appropriate tests. Run native
  GPU checks when rendering changes; document exact devices and limitations.
- Keep build outputs, credentials and private conversation/account exports out of
  Git. Keep changes local until the user explicitly authorizes pushing.
