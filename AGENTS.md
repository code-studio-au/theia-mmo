# Theia MMO — contributor instructions

Read `docs/STATUS.md`, `docs/DECISIONS.md` and `docs/ROADMAP.md` before editing.

- Build a playable MMO using Bevy. Prefer its standard renderer, assets, UI and
  animation over custom engine infrastructure unless measured game requirements
  justify an exception.
- Rust client and authoritative server; retain Go for services when needed.
  Server owns gameplay truth. Offline client movement is a temporary sandbox,
  not a trusted network protocol or production movement authority.
- Linux primary, Windows first-class. Use Bevy's platform abstractions; distinguish
  Linux execution, Windows cross-builds and native Windows execution in evidence.
- All environment items are static: terrain, buildings, doors and props have no
  opening, movement or destruction. Character movement/animation is separate.
  Players never physically collide with other
  players or NPCs. World collision and gameplay hit detection are separate
  requirements; nonblocking NPCs can still be targeted and hit.
- Keep this repository independent of `theia-engine`. Adapt reviewed components
  selectively, with provenance and tests; never introduce a path dependency on it.
- Complete small playable slices. Do not add distributed services, custom ECS,
  renderers or elaborate streaming systems before a game requirement needs them.
- Record implemented behavior separately from plans. Never equate proxy counts,
  local movement or mocked connections with working multiplayer.
- Run `cargo fmt --check`, denied-warning Clippy and appropriate tests. Run native
  GPU checks when rendering changes; document exact devices and limitations.
- Keep build outputs, credentials and private conversation/account exports out of
  Git. Keep changes local until the user explicitly authorizes pushing.
