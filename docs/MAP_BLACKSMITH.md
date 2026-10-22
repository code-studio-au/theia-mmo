# Second enterable map building: blacksmith main room

The complete development tile now retains the enterable farmhouse and adds the
original blacksmith's main ground-floor room. `python3 tools/play.py` selects
`build/development-world-town/`, starting near the blacksmith entrance ramp.
Its forge annex, deep side bays, side porches and roof remain excluded. Other
unqualified buildings retain their solid envelopes.

## Prepare and run

Prepare the [full outdoor tile](DEVELOPMENT_WORLD.md), then its
[farmhouse access version](MAP_FARMHOUSE.md). Choose a new output directory:

```sh
build/asset-tools/bin/python tools/open_map_blacksmith.py \
  build/development-world-access \
  /home/steven/wow.export/maps/2999/adt_31_27_ModelPlacementInformation.csv \
  build/development-world-town
python3 tools/play.py
```

Add `--clients 2` for a second animated player. Walk up the brown ramp into the
blacksmith. WASD/arrows move, Shift runs, RMB orbits, wheel zooms, Home resets the
camera and Escape quits. The earlier farmhouse-spawn version remains selectable
with `--world build/development-world-access`. Both buildings are open in the new
town version; changing the origin does not move either building within the map.

## Authored geometry and scope

This layout is pinned to original placement `adt_65748538`,
`duskwood_blacksmith_set0.obj`, with SHA-256
`f1e01da8e73ae9c72d93fae2459cb02697c13aec6c68f9033e2db2a51ed01678`.
Placement/model/sidecar hashes must agree with the original visual-world audit.
Prepared inputs are hashed before/after derivation and existing outputs are
rejected. All proprietary source and derived assets stay local and ignored.

Named groups `blacksmithinterior0114` and `blacksmithinterior017` provide 226
approved floor triangles, near source Y=1.81. The fixed eastern entrance is
approximately 4.3 units wide, with 4.53 units of conservative headroom above the
landing. Four visible ramp/landing triangles bridge the outside ground to that
floor. Original model scale, rotation, materials, meshes and furnishing placements
are retained; no door opens or closes.

The blacksmith's old solid envelope is replaced with 576 short conservative
wall/exclusion pieces around the main room and its doorway. A further 463 coarser
pieces block the unqualified exterior forge, bays and side porches from outside.
Removing only the old envelope without those exterior boundaries would allow
walking through the annex from the surrounding terrain. Camera-only roof/lintel
boxes supplement the existing static-mesh ray casts.

The derived world has 3,972 movement boxes, four camera-only boxes, all 65,504
terrain triangles and 367 approved support triangles: the retained farmhouse's
133 floor/four ramp triangles plus 226 blacksmith floor/four ramp triangles.
Existing furnishing and water boundaries remain. Movement remains server-owned,
players/NPCs remain nonblocking, and the existing slope/small-seam rules apply.

This qualifies the main room and central entrance approach. It does not qualify
the whole forge workshop, side bays/porches, roof, general stairs or stacked
floors. Conservative envelopes can stop a pawn before the visible wall and block
visibly empty corners. There is no general mesh movement physics, jumping,
falling, swimming, NPC gameplay or persistence in this slice.

The prior farmhouse scene BIN remains byte-for-byte as a prefix of the new GLB;
only the small new ramp payload is appended. All existing terrain/support/solid/
camera geometry is rebased together to the new source origin. `access.json`
records the blacksmith layout and test route; `farmhouse-access.json` records
the retained farmhouse route in the new coordinates. Runtime profile schema 2
and wire/ALPN version 3 remain unchanged; the world hash identifies the new data.

## Verification

```sh
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
build/asset-tools/bin/python -m unittest discover -s tools/asset_tests
python3 tools/test_align_glb.py
python3 tools/verify_multiplayer.py build/blacksmith-check --terrain-world build/development-world-town --player-assets build/reference-scene --map-access
```

The checker uses the authored route and source-space qualification points from
the selected building report. It checks entry, north-wall contact while input
sequences advance, return outdoors and re-entry, plus source height, authoritative/
rendered positions, animation, static clearance and the interior camera. Both
clients wait for real scene/pipeline readiness before their diagnostic routes.

## Executed evidence — 2026-10-16

Formatting, denied-warning all-target Clippy, workspace build, 42 Rust tests and
35 Python tests (31 asset-tool plus four GLB repair tests) pass. New tests cover
rebasing the first building's floors, walls and ceiling together without modifying
its input profile, plus clearance and continuous wall coverage at the rotated
eastern blacksmith doorway.

The 704,123,496-byte GLB passes Khronos validation with zero errors/warnings and
the existing empty-emitter info. Original source/prepared hashes remain unchanged,
and the prior farmhouse-world BIN is retained byte-for-byte as a prefix. Its 137
support triangles remain within 0.00000185 units after rebasing; terrain rebasing
error is below 0.0000145 units. The profile is 12,204,375 bytes. Sampled forge and
exterior exclusion points are covered by the final static envelopes.

Both six-stage CPU routes pass through the actual authoritative walker in the
new town profile: blacksmith entry/contact/exit/re-entry, and the retained farmhouse
route starting at its rebased outdoor approach. This is separate from networked
rendering evidence.

The final blacksmith session uses one actual Rust/QUIC server and two independent
Bevy clients on **AMD Radeon RX 7900 XTX / RADV NAVI31**, in an owned private
headless Wayland display. Active Khronos Vulkan validation, ready pipelines,
source-floor interpolation, local/remote/server XYZ and sampled stand/walk/run
checks pass with no validation errors or panics. Final screenshots are inspected.
The interior screenshot shows the player on the original stone floor among the
retained weapon and armour displays; the exterior shows the new fixed ramp.

The final local interior position is approximately `(10.96,-13.06)`, with capsule
centre Y=3.622 versus Y=2.089 outside. The north-boundary checkpoint advances input
sequences while remaining outside the authored exclusion. Interior camera distance
is about 8.68, with 15 additional static-mesh retractions recorded during the route.
These are correctness observations, not frame-rate or latency measurements.

The normal launcher also reaches town readiness and cleans up its server/client
on Ctrl-C. An initial paired check passed before the exterior exclusions were
broadened. The final boundaries then correctly blocked the old secondary test
route; that diagnostic now walks/runs away from the entrance instead of expecting
to pass through the blocked porch. The gameplay movement controls are unchanged.

Ignored `build/qualification/map-blacksmith/` retains source audit, section image,
CPU routes, source/geometry comparison, validation, test/build/lint logs, launcher
lifecycle proof and accepted `native-final/` results. Initial/rejected runs remain
separate. Third-party content stays out of Git. Windows, general collision/physics,
other interiors, swimming, navigation, performance and MMO scale remain unqualified.
The current profile uses 3,976 of its 4,096 combined box budget; further interior
work must review that representation/budget rather than assuming unlimited boxes.

A subsequent native farmhouse regression also passes with the final executable
and revised secondary route, using the earlier farmhouse-spawn profile. Together
with the new town profile's retained-farmhouse CPU route and geometry comparison,
this distinguishes checked regression evidence from an unexecuted second network
spawn inside the new town profile. Both accepted sessions have matching current
executable hashes. Four accepted real GPU clients are recorded across these two
sessions, with active Vulkan validation and no validation errors/panics.
