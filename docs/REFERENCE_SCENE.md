# Local reference game scene

The user selected the human export for players, RustyWolfPup for an enemy NPC,
and the farmhouse for a static building/housing reference. The opt-in scene now
uses those roles together, including connected remote players. This is local
compatibility content; no new third-party models/textures are committed.

## Run

Prepared local assets currently live in ignored `build/reference-scene/`:

```sh
cargo run --locked -p theia-client -- --reference-scene build/reference-scene
```

WASD/arrows move, Shift runs, hold RMB to orbit, wheel to zoom, Home resets the
camera and Escape quits. Add the usual connection arguments after the asset flag
to use the same models in multiplayer:

```sh
cargo run --locked -p theia-server
cargo run --locked -p theia-client -- --reference-scene build/reference-scene --connect 127.0.0.1:5000 build/server-cert.der
```

Run the second command in two terminals. The server does not load art or trust
client placement/collision. The plain graybox remains available without the flag.

## Placement and animation

Human meshes are children of existing player roots; server poses, identities,
camera visibility, interpolation and despawning remain in charge. Uniform scaling
fits the measured source height to the 1.8-unit pawn, with a local feet offset
and a forward-axis correction. Source GLBs are not modified at runtime.
Actual displacement selects stand/walk/run at rest, walking speed and running
speed. Network presentation samples displacement over server tick intervals;
offline presentation samples fixed-step movement. Holding movement into a wall
therefore returns to idle. Clips restart only when the gait changes. Transitions
currently cut directly; blending, stride matching and combat states are future work.

The wolf is an idle, half-scale enemy **presentation placeholder** at XZ `(3, -4)`.
It has no collider and never blocks players. It does not yet have a server-owned
NPC identity, AI, targeting, health or attacks. Those require the next gameplay
slice; the animated inspector's attack clip is not gameplay combat.

The farmhouse replaces the procedural building visuals at XZ `(-7, -7)`, fitted
uniformly inside an enlarged shared envelope. Both the server and graybox now use
the same 8 x 6.4 x 8 body, roof reaching Y=6.8 and fixed protruding door. Camera
obstruction uses those boxes too. This is conservative, solid-building collision,
not a traversable interior or mesh collider. The house stays static. Ownership,
housing persistence, furnishing and usable interiors are not implemented.

## Prepare fresh derived files

From the repository root, choose an output directory with no existing outputs:

```sh
mkdir -p build/reference-scene
python3 tools/align_glb.py /home/steven/wow.export/creatures/RustyWolfPup.glb build/reference-scene/wolf.glb --repair-metadata
python3 tools/align_glb.py /home/steven/wow.export/character/human/male/humanmale_hd.glb build/reference-scene/human.glb --repair-metadata
cp /home/steven/wow.export/world/wmo/azeroth/buildings/moonbrook_human_farm/moonbrook_human_farm.glb build/reference-scene/house.glb
```

The offline tool accepts only single-buffer embedded GLBs without declared
extensions or external images. It aligns buffer views, optionally recomputes
existing accessor extrema and removes clips with empty channels/samplers. It
refuses to overwrite the source or an existing output. It does not normalize skin
weights, merge duplicate joints, optimize vertices or establish general glTF
validity. Validate derived files separately. All original source files remain
unchanged; byte comparisons confirm every binary view payload is preserved.

The prepared wolf has zero Khronos validation errors, two warnings about non-root
skinned meshes and two unused-UV informational messages. Its 46 clips remain.
The original source's alignment and extrema issues are fixed only in this copy.
Prepared SHA-256: `18fcc6cbf83df29f5c2f6120f167fb2d17de7a12b60bb51127cbe3a37320ae90`.

The prepared human has 413 clips after removing three empty entries. Repairing
metadata permits deeper full validation, which finds 321,698 duplicate-joint-index
errors and 178,048 non-normalized-weight errors, plus 13 warnings and 16 infos.
These full results supersede any inference that the earlier truncated report
listed every problem. Rendering works in the tested Bevy paths, but this remains
a nonconformant compatibility asset, not a production-qualified character.
Prepared SHA-256: `3ab8ce78d24ed7582f715973d53b3d191f78d40fe1786d526f527608d8ebb032`.
The house is copied byte-for-byte; original hashes are in the
[source checks](ASSET_CHECK_2026_10_16.md).

## Executed checks — 2026-10-16

Formatting, denied-warning all-target Clippy, build, 23 Rust tests and four Python
repair tests pass. Tests include wall/door authority over actual QUIC, unchanged
player pass-through, idle despite blocked input, gait selection and fitting all
house bounds inside the shared envelope. Repair tests check binary preservation,
metadata correction, rejected inputs and refusal to overwrite files.

On RX 7900 XTX / RADV with actual Khronos Vulkan validation activation:

- A real-time reference smoke renders all three assets. Sampled human transform
  changes occur in stand, walk and run; wolf stand also changes. The scripted
  player crosses within 0.001 units of the wolf's centre without blocking.
- The imported-house world smoke reaches the fixed door at Z=-2.54 and verifies
  camera retraction/restoration against the enlarged static geometry.
- Two actual connected clients render two animated humans, one wolf and one house
  each, with server-owned poses agreeing between clients. The verifier checks
  instantiated animation players and locomotion logs; screenshots are also inspected.
- Original graybox two-client and offline movement smokes pass with the enlarged
  shared building. No VUIDs, validation errors or panics occur in the successful runs.

Reproduce the reference smoke on a graphical display:

```sh
target/debug/theia-client --reference-scene build/reference-scene --reference-smoke build/reference-check
target/debug/theia-client --reference-scene build/reference-scene --world-smoke build/reference-world-check
python3 tools/verify_smoke.py build/reference-world-check
python3 tools/verify_multiplayer.py build/reference-network-check --reference-assets build/reference-scene
python3 -m unittest discover -s tools -p 'test_*.py'
```

Ignored `build/qualification/reference-scene/` records source/derived hashes,
binary-preservation checks, native logs, reports and screenshots. Validator reports
remain under `build/qualification/rusty-wolf-pup/`. Evidence qualifies these local
assets and selected states, not native Windows, all clips, shader fidelity, MMO
capacity, terrain collision or a complete NPC/housing system.
