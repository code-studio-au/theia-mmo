# Theia MMO

Theia Online is a fantasy MMO in development, built in Rust with Bevy. This
repository contains the game client, authoritative server, zone editor, runtime
content and engineering tools. The gameplay direction is inspired by WoW Classic:
persistent targets, manual facing, character-relative movement, weapon-timed
autoattacks and hotkey abilities.

The project has playable development slices, including multiplayer movement,
combat, quests and persistent characters. It is still a prototype: production
content, broad platform qualification and MMO-scale capacity remain unfinished.
[Current status](docs/STATUS.md) records what has actually been implemented and
verified; [decisions](docs/DECISIONS.md) record the adopted direction.

## Related projects

| Project | Purpose |
| --- | --- |
| [theia-story — README](https://github.com/code-studio-au/theia-story#readme) | The narrative home for the world bible, lore, campaigns, quests, zone briefs and NPC/character writing. It records canon decisions and approval states; narrative plans do not establish implemented gameplay. |
| [theia-launcher — guide](docs/operations/GAME_LAUNCHER.md) | The Linux desktop launcher installs and updates the game, verifies release files, displays news and realm status, and starts hosted tester sign-in. Its source currently lives in [`apps/launcher/`](apps/launcher) in this repository. |
| [theia-website — README](https://github.com/code-studio-au/theia-website#readme) | The public Theia Online website, built with Astro and TypeScript and published through GitHub Pages. It contains game and world information, artwork, Markdown news articles, an RSS feed and download links. |

Keep `theia-story` beside this checkout for local cross-repository documentation
links. Runtime code, executable content and engineering documentation remain here.

Browse the [documentation index](docs/README.md) for subject guides and archived evidence.

## What is available

Features depend on the selected world and gameplay bundle; the bare development
launcher opens a movement sandbox.

| Area | Implemented development scope | Details |
| --- | --- | --- |
| Multiplayer | Encrypted Renet sessions, authoritative movement and collision, client prediction and reconciliation | [Networking](docs/networking/NETWORKING_RENET.md), [prediction](docs/networking/PREDICTION_CONTRACT.md) |
| World | Saved heightfield terrain, bounded streaming, static scenery collision and local origin rebasing | [Saved terrain](docs/world/SAVED_TERRAIN.md), [rebasing](docs/world/FLOATING_ORIGIN.md) |
| Characters | Native login and roster, character creation, modular appearance and account-backed persistence | [Frontend](docs/client/GAME_FRONTEND.md), [customisation](docs/client/CHARACTER_CUSTOMISATION.md) |
| Gameplay | Explicit combat profiles, NPC population, dialogue, quest journals, inventory, equipment and rewards | [Quest loop](docs/gameplay/QUEST_ENGINE.md), [authoring and population playtests](docs/editor/ZONE_EDITOR.md) |
| Interface | Themed HUD and settings, Say/Area chat and confirmed Say speech bubbles | [HUD](docs/client/IN_GAME_HUD.md), [chat](docs/gameplay/LOCAL_CHAT.md) |
| Authoring | Terrain sculpting/painting, scenery placement, group transforms, NPC/quest bindings, undo/redo and export | [Zone editor](docs/editor/ZONE_EDITOR.md) |
| Distribution | Linux desktop launcher, client installation/updates and hosted tester sign-in | [Game launcher](docs/operations/GAME_LAUNCHER.md) |

Millhaven is the current populated test zone. The 16 km × 16 km graybox remains a
separate terrain and movement test world. Earlier imported-map experiments are
retained as regression fixtures; forest reproduction is frozen.

## Development setup

Run commands from the repository root. Install Rust through rustup;
[rust-toolchain.toml](rust-toolchain.toml) pins Rust **1.99.0**, rustfmt and Clippy.
Python **3.11 or newer** is needed for the development tooling. Rendering requires
a working GPU driver and graphical desktop session.

On Ubuntu/Mint, install the native build dependencies used by Linux CI:

```sh
sudo apt install build-essential pkg-config libasound2-dev libwayland-dev \
  libxkbcommon-dev libx11-dev libxi-dev libxrandr-dev libxcursor-dev
```

Linux is the primary development platform. Windows is a first-class target; use
the Rust MSVC toolchain with Visual Studio C++ build tools and the Windows SDK.
Build/test coverage and native graphical playtest evidence are separate; consult
[validation](docs/engineering/VALIDATION.md) for recorded platform checks. Several local
service and packaging helpers currently require Linux.

Licensed source art and generated packages under `build/` are not included in a
fresh checkout. Follow the relevant asset-preparation guide before using a
workflow that requires them. Asset conversion has its own Python dependencies;
see [asset import](docs/assets/ASSET_IMPORT.md) and
[modular player assets](docs/client/MODULAR_HERO.md).

### Start with the offline sandbox

```sh
cargo run --release --locked -p theia-client
```

The no-argument development client opens the offline sandbox without account
services or a prepared multiplayer world. This is useful for checking that the
client builds and renders. Local movement here does not exercise server authority.

### Open the zone editor

```sh
python3 tools/edit_zone.py --example
```

This builds the debug editor and opens the editable Wayfarer Hollow example.
It saves separately to `build/wayfarer-hollow.zone.json` and reopens that draft
on later launches. Omit `--example` for `build/zone-draft.json`, or use
`--draft FILE` to choose another draft. The editor defaults to 3440×1440 fullscreen.
Locally staged scenery and player art are selected when available.

Use the editor for terrain, objects, vegetation, paths and gameplay bindings.
F5 enters or leaves offline playtest. To validate a saved zone and its capsule
spawn without opening a window:

```sh
python3 tools/edit_zone.py --zone path/to/zone.json --check-playtest
```

The check reports JSON and leaves the zone unchanged. The
[editor guide](docs/editor/ZONE_EDITOR.md) covers controls, structured commands, export,
imported terrain and account population playtests. For populated scenes, use its
optimized editor/client/server build and `tools/edit_local.py` workflow; the
convenient debug editor is unsuitable for judging full-scene performance.

### Run the multiplayer graybox

First prepare licensed player assets following the
[character conversion guide](docs/assets/SYNTY_CHARACTER_PREVIEW.md) or
[modular hero guide](docs/client/MODULAR_HERO.md). Then bake the terrain once and build
matching release binaries:

```sh
cargo run --release --locked -p theia-network --example bake_terrain -- bake build/graybox-terrain-v1
cargo build --release --locked --workspace
python3 tools/play.py --clients 2
```

The launcher uses `target/release`, saved graybox terrain and the available local
player assets. Pass `--players PATH` to select a directory containing
`character.glb`. Omit `--clients 2` for one client. Renet and movement prediction
are enabled by default; `--no-prediction` selects server-pose interpolation.

This workflow starts local development sessions. It does not enable the full
account/quest loop. See [graybox setup](docs/world/GRAYBOX_WORLD.md) for travel options
and retained maps, and [saved terrain](docs/world/SAVED_TERRAIN.md) for the tile format.
Quinn remains an explicit regression option through `--transport quinn`.

### Play with accounts, combat and quests

Use a prepared, versioned gameplay bundle with matching world, navigation and
assets. The [zone editor guide](docs/editor/ZONE_EDITOR.md) documents disposable local
account/population sessions; the [quest guide](docs/gameplay/QUEST_ENGINE.md) documents
quest bundles and their prerequisites. These workflows add local identity and
PostgreSQL services. Disposable editor sessions retain progress across Stop/Start
within that session, but a new invocation creates a fresh account/database.

For the installed game and hosted tester flow, follow the
[desktop launcher guide](docs/operations/GAME_LAUNCHER.md). It describes release installation,
updates, realm configuration and the supported sign-in paths.

The adopted controls are defined in
[THEIA-COMBAT-001](docs/gameplay/COMBAT_TARGETING_MOVEMENT_SPEC.md): W/S move, A/D turn,
Q/E strafe, right mouse steers, left mouse orbits independently and Tab selects
hostile targets. Earlier sandbox profiles retain their own controls; consult the
selected mode's guide. Combat facing and hit validation belong to the server.

## Architecture and repository layout

The workspace pins **Bevy 0.20.0** and **Avian 0.8.0-dev** at revision
`1070759b2145dc078abedace91dc7fe78013a45c`. See
[physics integration](docs/world/PHYSICS_AVIAN.md) for compatibility and limitations.

The client uses standard Bevy rendering, assets, animation and UI. The headless
server owns gameplay truth with a configurable fixed simulation step, normally
30 Hz. Network and persistence work stay outside that step; critical economy
transactions must be durable before success is acknowledged. Coordinates use
checked global `f64` positions, bounded local `f32` frames and Y-up metres.
Players and NPCs do not physically block one another; world collision, navigation
and gameplay hit detection are separate systems.

| Path | Purpose |
| --- | --- |
| `apps/client/` | Bevy game client and native interface |
| `apps/server/` | Authoritative game server |
| `apps/zone-editor/` | Zone authoring and playtest integration |
| `apps/nav-bake/` | Offline navigation baking |
| `crates/` | Shared gameplay, networking, authentication, persistence, zone data and effects |
| `content/` | Runtime definitions, authoring examples and UI assets |
| `tools/` | Launchers, asset conversion, packaging and qualification tools |
| `infrastructure/` | Deployment configuration and realm gateway |
| `third_party/` | Reviewed vendored integrations and provenance |
| `docs/` | Engineering decisions, contracts, status and validation evidence |
| `build/` | Generated local assets, packages and evidence; kept out of Git |

The repository is independent of `theia-engine`. The
[engineering architecture](docs/engineering/ARCHITECTURE.md) describes the adopted
design; the [implementation plan](docs/engineering/IMPLEMENTATION_PLAN.md) and
[task catalogue](docs/engineering/CODEX_TASKS.md) describe work still to do.

## Checks and contributing

Read [AGENTS.md](AGENTS.md), [status](docs/STATUS.md),
[decisions](docs/DECISIONS.md) and [roadmap](docs/ROADMAP.md) before editing.
For feature work, also follow the engineering reading order in `AGENTS.md`.
Identify the task IDs, prerequisites and concrete acceptance checks for each
playable slice. Keep implemented behavior distinct from plans and historical
experiments.

Run the standard Rust checks:

```sh
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Run the relevant Python tests for tooling changes. Rendering changes also need
native GPU checks with the device, platform and limitations recorded. A basic
client rendering smoke check is:

```sh
cargo run --release --locked -p theia-client -- --smoke-test build/smoke
python3 tools/verify_smoke.py build/smoke
```

The verifier requires Pillow. The smoke test needs a graphical display and checks
one offline client; it does not establish multiplayer, persistence or production
capacity. See [validation](docs/engineering/VALIDATION.md) for broader checks and evidence.
Keep credentials, private exports, licensed source assets and build outputs out
of Git. Keep changes local until pushing is explicitly authorized.
