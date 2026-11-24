# Imported 1 m heightfields and the Millhaven walkthrough

17 November 2026 — P1-10, P2-06/07/08 and P3-13/15 continuation.
The user requested a walkthrough of the complete supplied Millhaven V3 terrain,
4,096 × 3,072 metres, retaining its 1 m source samples. Stoneford V1 remains a
reviewed source for subsequent integration. Neither archive's embedded notes
authorize content edits or establish runtime acceptance.

Prerequisites: checked global/local coordinates, immutable saved-terrain identity,
bounded terrain workers, full-sample Avian heightfield representation, Classic
authoritative movement/prediction and ordinary Bevy mesh presentation. The existing
continental loader provides whole-zone traversal. Subsequent schema-16/17 editor
slices support whole-zone object authoring and saved sculpt/paint edits over this immutable base; see
ZONE_EDITOR.md. The bounded static-package export index remains unchanged.

## Import and launch

Extract `millhaven_v3_1m_elevation_f32.npy` from the user's
`Millhaven_V3_Preview_1m.zip` to an ignored local staging directory. Then:

```sh
python3 tools/import_heightfield.py build/millhaven-v3-source/millhaven_v3_1m_elevation_f32.npy build/millhaven-v3-1m --content-id theia.terrain.millhaven-v3
cargo run --release --locked -p theia-network --example bake_terrain -- verify build/millhaven-v3-1m
cargo build --release --locked -p theia-client -p theia-server
python3 tools/play.py --saved-terrain build/millhaven-v3-1m --dev-spawn 2048,1810 --prediction --classic
```

The ordinary launcher uses the existing local player art and encrypted Renet
authority, with 30 Hz simulation and normal Classic controls. This is a terrain
walkthrough without account/persistence/gameplay content. Keep the existing
editor/draft and other world packages; nothing is adopted as their replacement.
Choose another safe starting X,Z with `--dev-spawn` to inspect distant terrain.
W/S move, A/D turn, Q/E strafe, Space jumps; the existing camera controls apply.

The importer accepts little-endian float32 NPY arrays indexed `[z,x]`, absolute
metre elevations, and explicit 1/2/4 m spacing. It requires complete 128 m tiles,
finite heights within ±1,024 m and a rectangular region inside the existing
0..16,000 m world domain. `--chunk-origin X Z` sets the integer tile origin;
Stoneford's supplied world placement would use `32 0`. Sources are immutable;
destinations must be fresh. No smoothing, flipping, offsetting or downsampling
occurs. `import.json` records source hash, dimensions and complete independent
source-to-tile bit comparisons, including duplicate shared-edge samples.

## Saved-terrain schema 2

Schema 1 remains the existing 125², 33²-sample development continent with its
original identity domain, encoding and ±128 m height limit. Schema 2 replaces
`chunks_per_axis` with required `chunks_xz` and `chunk_origin`, accepts 33/65/129
samples per 128 m tile, and uses identity domain
`theia-saved-height-terrain-v2\0`. Ambiguous or unsupported layouts reject.

The tile envelope stays `THTILE01`, two little-endian signed i64 tile IDs and
Z-major little-endian f32 samples. A 1 m tile is 66,588 bytes. The manifest binds
all tile SHA-256 digests in Z-major/X-minor order. Reads check exact size, digest,
embedded coordinates, finite height bounds and path containment. Opening reads
only metadata. Offline verification scans every tile and shared edge; runtime
collision publication checks every sample along each resident shared edge.

Millhaven contains 768 tiles, 12,590,081 unique heights and 12,780,288 stored samples
including shared edges. Encoded tile data totals 51,139,584 bytes. Source NPY hash:
`c61f3276c62f0eafc6134c379e66a1d4552078f77f8a29ce1220e7af08edced4`.
Every imported source height is bit-preserved.

## Source review

Both supplied float32 arrays contain 4,097 x 3,073 finite heights. Millhaven's
east edge and Stoneford's west edge match bit-for-bit at all 3,073 samples.
Millhaven ranges from 169.0 to 325.924 m; Stoneford from 145.2 to 256.296 m.
The float32 NPY is the import source; decoding Millhaven's supplied uint16 PNG
adds up to 5.035 mm of height error under its declared encoding.

Measured Millhaven route grades remain below 8% outside documented bridge
openings. This does not prove complete crossing geometry: the southern and
eastern farm tracks also intersect the main river near X,Z 1901,1483 and
2653,1384. Riverbed rises around crossings need review before water is added.
Some named level pads retain noticeable variation: merchant yard 1.224 m,
southwest farm 0.889 m and bandit terrace 1.782 m across their supplied bounds.
Central-difference slopes peak near 77 degrees at Millhaven's bandit cut and
81 degrees at Stoneford's quarry. These are landscape/content observations,
not import corruption; the walkthrough retains them for inspection.

The independent audit, methods and overview/detail plots are retained locally
under `build/heightfield-source-review/`. Stoneford has not been integrated into
this Millhaven runtime package.

## Runtime behavior

- Preserve the existing bounded workers, queue capacities, collision readiness,
  prediction/reconciliation, rendering origin and authoritative movement.
- Clip tile demand, spawn/teleport admission and readiness IDs to the imported
  rectangle. Existing swept capsule coverage holds players inside loaded ground,
  including the finite map boundary; no implicit flat terrain is fabricated.
- Collision uses every 1 m or 2 m sample through standard Avian heightfields.
  Column-major physics ordering and the `[a,c,b]` / `[b,c,d]` mesh diagonal agree.
  Existing 4 m collision keeps its retained triangle representation.
- Keep at most 81 visual tiles: the inner 5 × 5 uses full source detail and the
  outer ring uses 4 m meshes for fine imports. Legacy 4 m sources retain their
  existing 16 m distant LOD. Downward visual skirts remain outside collision.
- Collision uses the existing clipped 3 × 3 patch per demanded actor owner;
  fixed simulation performs no tile I/O or geometry generation. Old support is
  retained during replacement. Missing/corrupt tiles fail explicitly.
- The horizontal spawn lookup now searches from 2,048 m, above the admitted
  imported elevations. Near-position restored/authored spawn semantics remain.
- Diagnostic timed Classic command scripts can run against saved terrain as well
  as authored worlds; checked waypoint routes retain their authored-world rules.
- The existing minimap reads fine resident meshes directly; its sample budget
  remains fixed. The diagnostic overlay reports the imported extent and spacing.
  The map covers 256 m around the player; portions outside the zone are dark.
  Near a boundary this can leave a dark strip or half-map. Native checks require
  every in-bounds minimap sample to be present. The later whole-zone overview
  aligns this package's overview and minimap with the source's +Z-north convention;
  other worlds retain the old compass orientation. Original sample coordinates
  remain preserved. See ZONE_OVERVIEW.md; press M for the entire zone and landmarks.

## Acceptance and remaining scope

Passed checks include complete source/sample/seam comparisons, malformed and
rehashed-data refusal, visual/Avian surface agreement, high-elevation admission,
20/30/60 Hz seam movement, no simulation-step reads, bounded load/retirement,
native Classic/Renet walkthroughs, GPU diagnostics, fmt and denied-warning Clippy.
Final native evidence is `build/millhaven-1m-native-v2/qualification.json`:
five locations, 59.99-60.01 FPS at the normal 60 FPS cap and 1280x720 on Linux
RX 7900 XTX / RADV NAVI31 / Vulkan / Mesa 26.2.6, with zero validation diagnostics.
This is sampled route evidence, not a claim that every route has been traversed.

The source-independent importer tests run with
`python3 -m unittest tools.asset_tests.test_import_heightfield -v`.
Rust regression coverage is in `crates/network/tests/imported_heightfield.rs`.
Its ignored Millhaven fixture test additionally requires absolute
`THEIA_IMPORTED_TERRAIN` and `THEIA_IMPORTED_REPORT` paths. Native qualification:
`python3 tools/qualify_imported_terrain.py --output build/millhaven-1m-native-new`.
This launches owned local server/client pairs, drives ordinary Classic commands
and captures their game viewport; it injects no desktop input. Run network and
client test commands separately to avoid linking renderer features into every
headless integration-test executable.

The terrain uses the existing neutral diagnostic material. Water surfaces,
swimming, bridge decks, cave interiors, buildings, scenery and navigation are
separate content. The supplied roads and riverbeds are terrain shapes only;
the review's missing crossings, uneven pads and sharp cuts remain unchanged.
This slice does not add full-map editor sculpting, water simulation, arbitrary
continental coordinates, production terrain materials or MMO capacity claims.
Linux native evidence and native Windows qualification remain distinct.

Validation evidence is recorded in STATUS and the local `build/heightfield-*`,
`build/imported-heightfield-tests-v2.log`, `build/millhaven-1m-physics.json` and
native walkthrough reports. Generated terrain, credentials and binaries stay
outside Git. No push is authorized.

## Full-zone terrain view — 17 November 2026

`THEIA_TERRAIN_VIEW=full python3 tools/play.py --saved-terrain <package>
--prediction --classic --dev-spawn X,Z` opts the client into all visual tiles of
that saved package. The explicit setting supports at most 32 tiles per axis
(1024 total); larger continents and non-saved sources refuse this mode. Unset or
`nearby` retains the existing radius-four / 81-tile visual policy.

Full view uses the same one-job visual worker and nearest-first scheduling.
The inner five-by-five tiles retain source detail; fine imported terrain uses
4 m distant meshes. Existing coarse sources retain their previous distant LOD.
The ordinary Bevy perspective far plane expands to the package diagonal plus
height/camera margin. HUD and qualification reports expose the actual visual
budget. Server authority, collision patches and near-detail readiness remain
unchanged; this does not require all distant tiles before movement can start.

For the prepared WM v3 package this means 460 visual tiles across 2560 x 2944 m,
with 2 m near terrain and 4 m distant terrain. The editor already loads all 460.
It is a bounded authoring/playtest vista option, not continent-wide or MMO-scale
capacity qualification. Native acceptance for the latest build is recorded in
STATUS and the local full-terrain-view evidence.

### Graphics menu view distance — 17 November 2026

F10 → Graphics now exposes Nearby, Far and Maximum in clients using the Fantasy
UI. Use `--ui-preferences` to persist the choice (ordinary account playtests
already provide it). Maximum uses full-zone rendering within the bounds above;
for larger/generated terrain it falls back to a 15-tile radius. Nearby/Far use
4/8-tile radii. The setting applies live; reducing it retires extra meshes and
restores camera clipping. Explicit saved preferences and live choices supersede
the legacy environment switch. Existing terrain-only launchers without the HUD
retain that environment switch. The WM launcher was not forced to Maximum.
Native switching, persistence/restart and cleanup are accepted on Linux; see
STATUS and build/graphics-view-checks/native/qualification.json. The supplied
terrain was used as a fixture only; its design work remains set aside.

### Original Covenant H1 + H2 walkthrough — 17 November 2026

User correction: use Millhaven and Stoneford from
`Covenant_All_20_Zone_Heightmaps_v1.zip`, not the older Millhaven V3 preview and
not any World Machine export. The local preparation is
`build/covenant-h1-h2-original-v1/prepare.py`; both zone R16 arrays are checked
against the exact corresponding shared master PNG crop. The combined original
window covers 4683.59375 × 3457.03125 m, including connecting/context terrain.
Use the archive's linear uint16 → 0..1600 m conversion; preserve horizontal scale
and relative placement. Bilinear conversion from 3.90625 m source samples to the
supported 2 m grid and constant east/south edge extension produce 4736 × 3584 m,
37 × 28 tiles (1036). No smoothing, World Machine processing, vertical offset or
normalization. Full source metadata, masks, raw arrays and hashes are retained.
The importer verifies all 4,377,100 stored samples including shared tile edges.

`python3 build/covenant-h1-h2-original-v1/play.py` starts in Millhaven; append
`--site stoneford` or `--site pass` to select the other recorded positions. The
HUD enables F10 → Graphics and the explicitly local F10 → Developer speed page.
Graphics preferences use the ordinary local preference path; every new developer
session creates its own 1× control file. This remains a terrain walkthrough,
separate from the populated village/NPC/wolf authored scene. Maximum uses the
bounded moving ring because this combined package exceeds the full-view width
limit. The existing editor's maximum imported width is also smaller than this
combined runtime package; opening both as one editor document is not claimed.
Native validation is recorded in STATUS after execution.
