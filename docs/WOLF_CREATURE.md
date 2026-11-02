Update: The wolf now follows authoritative perception/chase/leash/return as well as patrol. [Latest playable proof and command](CREATURE_PURSUIT_NATIVE.md).
The dated checkpoint below remains preserved.

# Animated development wolf

2026-10-31, P1-10 / P2-08 / P2-09 / initial P3-14. Prerequisites are the
qualified creature patrol, combat/lifecycle observations and standard Bevy glTF
animation path. This opt-in slice replaces the training target's presentation;
it does not add a new creature definition, gameplay behavior or wire protocol.

Qualified local development checkpoint: `build/wolf-qualified-v1-20261031`.
Offline conversion, six inspector clips, actual two-account gameplay with both
current and legacy bundles, and normal fullscreen launcher cleanup pass. The
previous `build/player-life-qualified-v1-20261031` remains preserved. Changes
remain local; no commit or push was made.

## Source and conversion

The supplied Creatures Pack v1.0 Unity package contains a MeshTint Polygonal
Wolf with its own 41-bone rig. It does not use the Synty player's humanoid rig.
Explicit source payloads and metadata remain private under
`build/creature-wolf-source-v1`; package scripts/controllers are never executed.
The selected brown material's MainTex GUID matches the selected palette.

`tools/convert_creature_wolf.py`, running in the existing pinned Blender Python
environment, verifies extraction hashes and bone names/parents. It bakes source
world poses onto the mesh skeleton without copying incompatible local curves.
The character remains in metres, with glTF Y-up. Rig-object motion is rejected;
idle/walk/run loops must not carry net horizontal pelvis displacement above 1 cm.
Only the explicitly selected walk/run **without root** sources are used. Bone
sway, attack lunges and falling in death clips are presentation, never authority.

The output is `build/creature-wolf-stage-v1/wolf.glb`, with Idle, Walk, Run,
Attack, Hit and Death. It has 1,400 triangles and at most four skin influences;
there is no reduced-weight approximation. The standard material uses brown base
colour, metallic 0 and roughness 1. Unity emission/glow is deliberately omitted;
this is not Unity shader parity.

`tools/verify_creature_wolf.py` independently evaluates exported glTF interpolation,
hierarchies, inverse binds and skin matrices against every sampled reference
frame. All **216 frames** pass; maximum skin error is below 0.003 mm. It checks
one skin/material/image, six named animations, matching bind vertices and bounded
animated extents. The existing Synty verifier's skin reader now accepts an optional
bone count; its default remains 50.

The embedded palette PNG is byte-for-byte identical to its source. Khronos
validation reports zero errors, one `IMAGE_FEATURES_UNSUPPORTED` metadata
warning on the embedded PNG, and ten informational empty-node notices. The
warning is recorded rather than treating this as a zero-warning asset. Source
poses and conversion results do not establish Unity-import visual parity.

## Client behavior

Select the GLB explicitly with `--creature-asset FILE`. It requires predicted
creature combat, uses a separate Bevy asset source and does not change the
player's assets or gameplay bundle identity. Without this flag, the primitive
training target remains available. Asset failures or missing named clips fail
explicitly; a placeholder remains while the selected asset loads.

The existing target owner retains authoritative identity, position/interpolation,
relevance and chunk permission checks. Its child wolf faces the normal -Z heading
and has its feet offset from the authority body centre. There are no creature
colliders. A selected live target has a ground ring and Wolf target label. A
conservative six-clip bounds union supports standard frustum culling.

Actual interpolated global displacement selects idle/walk/run and heading without
turning floating-origin shifts into animation velocity. A 200 ms gait hold bridges
brief replication gaps without changing the presented or authoritative position.
The same low-speed fixture reduced walk restarts from roughly seventy to four
per peer; clips retain their phase during the intervening updates. Incoming **owner-only**
player-life windup observations select attack and face the observed victim; the
clip is time-scaled using the remaining observed windup. This introduces no public
attack stream: a passive peer is not guaranteed to see another player's incoming
attack animation. Existing critical incoming text remains authoritative.

Existing combat damage observations select hit; death takes precedence over
attack, attack takes precedence over hit, then locomotion. One-shot hit/death
clips do not loop. Attack animation restarts only for a new observed windup,
and clears when that observation clears. Clips never determine hit/damage times.
Blending, layered reactions and public attack timing remain later work.

A Died event retains a visual-only corpse at its observed global position for at
most 15 seconds; Respawned retires that key immediately. It remains unselectable
because the server removed the dead target. The new incarnation starts a new
visual/interpolation/animation state. Corpse presentation rechecks confirmed
chunks and render-frame conversion. Disconnects and epoch changes clear retained
cues. The cache is bounded at sixteen cues; death evicts a cosmetic hit first.

## Evidence and limits

Six inspector runs, `build/wolf-native-clips-v1`, pass on native Linux,
RX 7900 XTX / RADV Vulkan with active Khronos validation, one animation player,
changing transforms and zero GPU validation diagnostics. Walk was visually
inspected against the 1.8 m reference capsule. This proves standalone standard
Bevy playback, not networked gameplay or Windows execution.

Final unchanged-source workspace: **529 passes / 40 explicit fixture skips in
111 groups**, formatting, denied-warning workspace/all-target Clippy and 28 Python
tests pass. Four new client tests cover death under cosmetic pressure, old-corpse
retirement, bounded expiry and gait continuity across short observation gaps.

The exact qualified client passes actual OIDC/PostgreSQL/two-account gameplay at
1920×1200 with player life and 1280×720 with the legacy rewarded bundle. Across
fourteen native starts, active Khronos validation reports zero GPU diagnostics.
Current-mode observations include incoming attacks, nonfatal hit reactions, six
shared enemy deaths and fresh visuals after respawn, plus unchanged quest/reward,
rejoin, server/database restart and backend-loss checks. The legacy one-hit fixture
correctly transitions directly to death. No new standalone SQL-contract run is
claimed; thirty prior contracts are retained against unchanged persistence code.

Normal thirteen-profile launching passes interrupt, child failure and pending-login
interruption at **captured 3440×1440 fullscreen**, including private-ticket,
service and sealed-profile cleanup. The model flag reaches the client only.
No GPU diagnostics occur in these launcher runs. The qualification fixture now
waits two completed layout passes after panel opening/viewport changes: diagnostics
showed a transient 800×600 viewport during fullscreen startup. Settled geometry
still has strict bounds checks. Captured PNG dimensions must match the requested
size; the earlier 3440×1368 window-manager-resized run is explicitly excluded from
3440×1440 evidence.

Quiet single-client runs use the same executable, all thirteen profiles, animated
wolf and original POLYGON player, without concurrent builds or external validation:

| Actual size | Mode | Final-window frames | Mean frame time | Average FPS | Worst one-second p95 |
|---|---|---:|---:|---:|---:|
| 1920×1200 | Windowed | 10,800 | 1.3904 ms | 719.22 | 1.5822 ms |
| 3440×1440 | Fullscreen | 10,813 | 1.3889 ms | 720.01 | 1.5974 ms |

Each uses the final fifteen one-second windows, frame-weighted means and 81
resident chunks with no unloads. These are uncapped near-spawn gameplay baselines,
not GPU timestamp measurements, aggregate p95s, minimum-FPS guarantees, crowded
combat, streaming-tour or MMO capacity results. Ordinary desktop programs remained.
RX 7900 XTX / RADV NAVI31, Mesa 26.2.6 Vulkan; Ryzen 7800X3D, 128 GiB RAM. The
physical monitor is 120 Hz. [Previous pre-wolf baseline](PLAYER_LIFE_PERFORMANCE.md).

Raw proof roots are `build/wolf-life-1920-native-v3-20261031`,
`build/wolf-legacy-1280-native-v3-20261031`,
`build/wolf-launcher-fullscreen-v2-20261031`, and
`build/wolf-performance-{1920,3440}-v2-20261031`. The checkpoint preserves source,
matched executables, copied art/verification, non-story documents and selected
proof files. Client DWARF was removed before all final native runs; its exact
processed hash is retained, while original symbols remain in `target/release`.

Flat selection rings can intersect sloped terrain; terrain-conforming markers,
foot IK, blending, public NPC attack timing, general creature definitions and
native Windows qualification remain open. The player still uses the original
four-clip development skin, including its previously measured preview limitation.

```sh
python3 tools/play.py \
  --binaries build/wolf-qualified-v1-20261031 \
  --gameplay-validator build/wolf-qualified-v1-20261031/validate_gameplay \
  --gameplay-manifest content/gameplay/development-gameplay-player-life.json \
  --creature-asset build/creature-wolf-stage-v1/wolf.glb \
  --window-size 1920x1200
```

The normal launcher still requires configured local identity/database services;
see [development setup](DEVELOPMENT_WORLD.md). Use `--window-size 3440x1440
--fullscreen` for the qualified ultrawide mode. Omit `--creature-asset` to retain
the primitive target. No proprietary source payloads or generated art enter Git.
