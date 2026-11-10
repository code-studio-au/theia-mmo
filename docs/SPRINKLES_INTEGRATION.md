# Sprinkles authoring integration (P5-17)

The accepted editor styling migration is recorded in `ZONE_EDITOR.md`. This
slice reuses the actual Sprinkles runtime and editor, pinned at
`0e978f5a9afc6b456b3693c8761e73a6c7e155bd`, instead of recreating its particle
authoring controls. Source, asset licenses and file hashes are under
`third_party/sprinkles/`.

The prerequisites are the accepted Linux authoring shell and the adopted exact
Bevy 0.20.0 engine. Upstream targets Bevy 0.19.0. Both imported crates use our
workspace engine; introducing a second Bevy engine or downgrading Theia is not
part of this slice. Avian remains pinned independently and unchanged.

## First host integration

Build the companion binary alongside the zone editor:

```sh
cargo build --locked -p theia-zone-editor -p bevy_sprinkles_editor
```

The zone editor's **File → Particle editor…** command opens the actual retained
Sprinkles UI in a separate authoring window. Repeated requests retain one child
process. Normal workspace input/modal/preview guards apply. Starting the tool
does not edit or save the zone. The particle window owns its RON projects and
must save them explicitly. Closing the zone editor leaves that window alive,
so it can finish saving its work.

Projects, examples and preferences default to `~/.theia/particles`; the
`THEIA_PARTICLE_EDITOR_DATA_DIR` override supports isolated native checks.
Existing `~/.sprinkles` preferences/projects are not migrated or overwritten.
The tool can also open an existing effect directly:

```sh
target/debug/theia-particle-editor /absolute/path/to/effect.ron
```

The library extraction is intentional: `bevy_sprinkles_editor::standalone_app`
hosts the original plugins, widgets, inspector bindings, preview and RON
load/save behavior. The companion executable is a thin entry point. This is
the first integration boundary, not a same-window particle workspace.

## Compatibility adaptations

- Flat pointer events, native button/focus components, explicit native text
  inputs and Bevy 0.20 scene composition preserve the original UI behavior.
  A bounded main-row layout keeps panels and the preview within the window.
- Shader buffers retain encase/WGSL storage encoding; replacement writes have
  a round-trip test. Extracted fallback textures target the render app explicitly.
- Original shaders use WESL imports, paths and conditional attributes. PBR alpha
  discard follows the Bevy 0.20 signature. This follows the
  [official shader migration](https://bevy.org/learn/migration-guides/0-19-to-0-20/).
- Embedded asset paths use the extracted library's namespace. The selected code
  license is MIT; separate upstream asset licenses are retained.
- Immediate and retained Bevy gizmos both produced Vulkan synchronization
  hazards for preview bounds on RX 7900 XTX. Standard Bevy line meshes/materials
  display bounds and collider outlines instead. Shared assets remain bounded;
  selection, transforms, disabled colors and cleanup have a CPU regression.
  The UI camera excludes scene overlays. No custom renderer is introduced.
- Imported legacy BSN syntax/Interaction remains scoped compatibility debt.
  Deprecated allowances and the scene macro's generated unit-binding lint are
  confined to the imported editor, without weakening Theia's workspace checks.

## Acceptance and limits

Final direct-source validation passes at
`build/sprinkles-direct-native-20261110-final-v1/qualification.json`:
six-emitter Magic Puff preview, pause/resume, loop changes preserving the asset,
real inspector scrolling, particle-count entry through native text input,
exact expected in-memory modification and explicit Ctrl S RON save. Vulkan
validation reports zero diagnostics on AMD Radeon RX 7900 XTX (RADV NAVI31),
Linux, 1366×768. Validation-host SHA-256:
`5205585100a6c8fb43381eb8c1c652b087abbf6f78f9710e395cd9d906ef11e8`.

`build/zone-particle-host-20261110-v2/qualification.json` passes the actual File
command, single-child behavior, clean close/return and exact zone/revision/history
and source-file preservation, alongside the retained native menu/property flows.
The zone window is 3440×1440, on the same GPU/driver/backend with zero diagnostics
across both applications. Zone binary SHA-256:
`e578c063d4d9b5295cb5af5fadda31fa0c7cc40be020eebbbbae52af20ed5a96`.
Companion binary SHA-256:
`8d25fb3ca4d49968d7350add1e968f45d01bd5a9ed952a0e1dfd6ec11f8ebb02`.

All 172 focused Rust unit/integration tests and three runtime doctests pass;
formatting and denied-warning workspace/all-target Clippy pass. Dependency
inspection confirms no Bevy 0.19/0.20.1 modules, no removed package versions,
and no Sprinkles/render/window dependency added to the server. First probe
failures are retained as failures: shader/layout/native gizmo issues were fixed;
the host v1 failure was a harness assumption about worker-thread child ownership.
The harness now discovers children of all owned process threads and cleans up
its known child. Failed probes do not establish acceptance.

Reproduce after building the companion and validation host:

```sh
cargo build --locked -p theia-zone-editor -p bevy_sprinkles_editor --bins --example theia_validate
python3 tools/qualify_sprinkles_port.py --output build/particle-port-check
python3 tools/qualify_zone_sprinkles.py --output build/particle-host-check --case local --particles
```

Remaining work is the same-window workspace, particle asset discovery/reference
and placement, and then client presentation integration. A particle preview is
cosmetic authoring evidence: it does not establish authoritative spell effects,
world collision/navigation behavior, protected critical telegraphs, audio,
network replication or MMO population/cosmetic-load capacity. Full P2-13 remains
separate. Windows build/native rendering and the complete upstream preset/widget
matrix still require qualification.
