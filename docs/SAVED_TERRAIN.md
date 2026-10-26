# Saved continental terrain

P1-10, P2-06/07 and initial P3-13 continuation, 2026-10-23. Prerequisites:
qualified 16 km procedural traversal, checked coordinates, bounded terrain workers,
shared Avian controller, readiness and Renet prediction. This slice replaces the
runtime height recipe with immutable saved height samples; it does not add zones,
region ownership, sharding, navigation or a terrain editor.

## Acceptance

- Bake and independently verify the complete 125 × 125 tile continent, including
  every shared edge; retain the procedural baseline.
- Both visual LODs and collision use the same checked saved height samples. A
  modified interior sample must change both geometry and real Avian support.
- Bind admission/corrections to the manifest and per-tile hashes; reject missing,
  corrupt, wrong-coordinate, invalid-height and discontinuous data explicitly.
- No tile I/O or geometry generation in fixed simulation. Preserve bounded
  worker queues, collision support during replacement and actual mesh retirement.
- Demonstrate real encrypted impaired-UDP prediction, native full crossing,
  teleports/revisit, camera/animation and the actual default launcher.
- Qualify native GPU behavior and retained modes before default adoption.

## Prepare and run

From the repository root, with the existing Linux native SDK environment when
required:

```sh
cargo run --release --locked -p theia-network --example bake_terrain -- bake build/graybox-terrain-v1
cargo run --release --locked -p theia-network --example bake_terrain -- verify build/graybox-terrain-v1
cargo build --release --locked --workspace
python3 tools/play.py
python3 tools/play.py --saved-terrain build/graybox-terrain-v1 --clients 2
# Retained runtime-generated world:
python3 tools/play.py --graybox-world
# Retained server-pose interpolation, using saved terrain:
python3 tools/play.py --no-prediction
```

The default launcher selects `build/graybox-terrain-v1`; it requires an explicit
successful offline bake. It never silently generates missing terrain during play.
Baking refuses an existing destination. Use a fresh directory and revision for a
new package; the old content remains available. No saved tiles/private player
models/binaries are committed. The POLYGON character preparation is unchanged.

## Format and identity

`terrain.manifest.json` schema 1 specifies a stable logical content ID, positive
revision, 128 m tile size, 125 tiles per axis, 33 samples per axis, and 15,625
SHA-256 tile digests in Z-major/X-minor order. The whole manifest is limited to
8 MiB. A domain-separated SHA-256 over its exact bytes is the runtime world
identity; metadata/height edits therefore require a new admitted world identity.
The manifest/index is resident metadata. All terrain samples and meshes are loaded
on demand, independently on client and server; no terrain geometry crosses UDP.

Each fixed 4,380-byte `tile_X_Z.height` has magic `THTILE01`, signed little-endian
64-bit X/Z identifiers, then 33 × 33 little-endian f32 heights in metres, Z-major.
Samples are spaced 4 m apart, including both shared edges. This first format
supports finite heights in -128..128 m. Near visuals and collision use all samples;
far visuals decimate to 9 × 9 vertices using shared regular triangulation. Visual
skirts remain excluded from collision. This is a narrow development format,
not final terrain materials/caves/objects/navigation or production compression.

Opening loads only the manifest. Every worker tile read verifies bounded length,
hash, embedded coordinates and height range. Paths are generated from checked
integer IDs and resolved inside the package. A collision patch additionally
checks bit-identical shared edges before publishing any of its nine tiles.
The offline verifier scans every tile and both axes of seams using bounded row
memory; runtime does not scan the whole package at startup.

World wire/Quinn ALPN is now **12**. The reliable welcome is **77 bytes**: the
previous identity/origin fields plus one strict 0/1 terrain-stream frame-policy
byte. Large-world validation is explicit rather than inferred from one hard-coded
procedural content digest. Fixed imported maps retain policy 0; generated and
saved continental terrain use policy 1. Rebuild both applications and issue fresh
tickets. Existing command/correction profile layouts remain unchanged.

## Loading and failure policy

The 9 × 9 visual radius, inner 5 × 5 high detail, 3 × 3 collision patch, independent
query/render origins, readiness overlap and recipient-local actor relevance remain
as documented in [the graybox contract](GRAYBOX_WORLD.md). Existing bounded workers
perform file reads and geometry construction. The fixed simulation only queries
already prepared patches. Old supporting server collision remains resident while
a destination loads; returning to retired terrain reads it again.

Runtime tile loads fail explicitly: the client withdraws readiness and exits with
an error; the server pump returns an error before the next simulation step. There
is no recipe fallback, repeated retry flood or publication of partial patches.
Invalid spawn support denies admission/teleport rather than panicking. Automatic
retries, package download/update and recovery UI remain future work. Files are
immutable for an admitted session: modifying them in place violates the contract
and fails their captured manifest checks.

## Evidence

The bounded saved-terrain slice passes and the launcher adopts it by default.
Implementation and focused saved-terrain tests pass. Actual saved-terrain
prediction under impaired encrypted UDP passes at 20/30/60 Hz. The first full
suite found 1.9 micrometres of grounded Y variation in a retained test that demanded
bit-identical poses. It now requires exact stopped X/Z and less than 0.01 mm
vertical variation, consistent with the live collision-probe precision contract.
The failing attempt remains in `build/saved-terrain-workspace-tests-v1.log`.
The final executed workspace, UDP, native and adoption results follow below.


### Executed checkpoints — 2026-10-23

- `build/saved-terrain-workspace-tests-v2.log`: **228 Rust tests pass**, zero
  failures, 10 explicit private-fixture tests ignored. Three saved-terrain checks
  additionally pass the strengthened non-spawn retirement/revisit assertion in
  `build/saved-terrain-revisit-tests-v1.log`. Workspace/all-target denied-warning
  Clippy passes (`build/saved-terrain-clippy-final.log`); fmt/diff pass. All 221
  asset-tool tests and four GLB alignment tests pass.
- Actual encrypted impaired UDP (`build/saved-terrain-network-v2/saved-prediction-wifi.json`):
  20/30/60 Hz, 40 ms + 0–60 ms jitter, 10% loss, duplicates/reorder. First local
  driver response 29.32 / 35.30 / 9.63 ms, stopped errors 0.00764 / 0.00779 /
  0.00383 mm; history peaks 7 / 9 / 14 and zero producer drops. These are
  headless driver timings, not rendered input-to-photon measurements.
- Full native saved-terrain crossing (`build/saved-terrain-continent-v1/verification.json`):
  80 seconds, Linux RX 7900 XTX / RADV NAVI31 / Mesa 26.2.5, Ryzen 7800X3D,
  Vulkan, 1280×720, prediction, developer speed 32, validation off. All 125
  columns visited; 2,345 visual mesh loads / 2,300 retirements, maximum 81 roots,
  45 roots at the finite eastern edge. Zero movement recoveries/missed deadlines.
  Client reads 3,464 tiles / 15,172,320 bytes; server reads 1,119 tiles /
  4,901,220 bytes. Real CPU/GPU mesh inventories remain bounded. Ground-query
  disagreement peaks at 1.907 micrometres.
- Same crossing: **818.59 FPS**, mean **1.22161 ms**, worst one-second p95
  **1.38107 ms**, 58,999 post-warmup frames, including the stationary edge finish.
  This is close to the earlier procedural prediction result (821.70 FPS), not
  evidence of a meaningful speed change. Files were recently baked/verified:
  this is warm-filesystem-cache evidence, not a cold-storage latency benchmark
  or populated MMO capacity claim.

The package contains 68,437,500 height bytes plus a 1,816,180-byte manifest,
70,253,680 bytes total. Manifest SHA-256:
`897328cff112a83f5e70453709ad36e4323e9c349b84b3e04b2c61a0923ece13`;
world identity:
`4eafed3b4a212645f73f1d5602d61bcc2b9b154b30b0bc6ec740fdd6cbf6103a`.
Release client:
`6285281bba51d7be749b15afeaa09a6aa9353b52bbb92330631805f2a64ba11b`;
server:
`6db7284e4a87b5fa513ef7a2de4af4ee872617a1f98428f344ca2700ab8467c0`.
The retained pre-saved prediction binaries are in `build/prediction-qualified-v6`;
these and all raw native reports/packages remain ignored local evidence.


- Two native saved-terrain clients (`build/saved-terrain-native-wifi-v1/verification.json`):
  65 seconds, normal speed, five teleports/return, Vulkan validation and the same
  impaired UDP profile. All four actual animation clips, bounded meshes and
  saved tile reads pass. 96.58 / 96.89 FPS with validation; history peaks 10 each,
  no producer drops. The tour actor checks **1,233 held-jump reset samples**, zero
  failures. Raw replay pose-delta peaks are 0.1005 / 0.2681 m; they include clock/
  readiness catch-up and are not equal-time prediction errors or a smoothness budget.
- Native corrupt tile (`build/saved-terrain-native-corrupt-v1/verification.json`):
  a saved client with the same admitted manifest but a changed centre tile emits
  the hash error and exits with code 1, without panic or Vulkan VUID. The valid
  authority survives. No generated fallback is used.
- Actual default launcher (`build/saved-terrain-launcher-v1/verification.json`):
  both clients/server explicitly select saved terrain and prediction. Real
  commands are queued. Interruption and injected client failure both remove all
  three owned children and temporary tickets.

- Retained town (`build/saved-terrain-town-v2/verification.json`): two encrypted
  native clients, map interior/exterior/camera/animation checks, 2 m render/wire
  rebases and Vulkan validation pass. The v1 attempt captured successfully but
  its verifier still expected wire 11; it remains a failed verification attempt.
  The verifier now explicitly requires wire 12 in snapshots/readiness acks.
- Retained generated source (`build/saved-terrain-generated-regression-v1/verification.json`):
  fresh 20-second native predicted traversal, all four clips, actual mesh
  retirement and bounded inventories pass on the same release binaries.

The bounded Linux slice is complete; entire source tasks remain partial. Cold
storage, long soaks, authored materials/objects/caves, terrain editing, production
LOD/vistas/flying, dense populations, native Windows and realm/region services
remain unqualified. Further movement diagnostics/polish is a separate P2-04 slice.
