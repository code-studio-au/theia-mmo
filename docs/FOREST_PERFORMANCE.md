# Forest interactive performance investigation

P1-12 / P2-05 / P2-08 / P2-10, 2026-10-22. The user reported approximately
10 FPS in the full forest walkthrough. Earlier native checks established content,
collision, material and connection correctness; they did not establish acceptable
interactive frame time. The walkthrough had used unoptimized debug binaries.

## Measured debug baseline

An isolated native compositor runs one normal connected character at the dry
forest spawn, 1280 × 720, on the Linux Ryzen 7 7800X3D / RX 7900 XTX machine.
Vulkan validation and compilation workloads are absent during sampling. Each
case discards ten one-second reports and samples the following fifteen reports.
The complete geometry, textures and lighting are unchanged.

| Debug camera configuration | Mean frame time | P95 frame time | Mean extra mesh query time | Mean FPS |
|---|---:|---:|---:|---:|
| Existing visual-mesh refinement | 344.93 ms | 353.36 ms | 272.91 ms | 2.90 |
| Existing world collision clearance alone | 102.08 ms | 113.29 ms | 0.0067 ms | 9.80 |

The extra camera pass casts five CPU rays against render meshes. Its candidates
include the whole 524,288-triangle terrain visual. The existing world-camera
path already queries accelerated Avian geometry, including full-sample terrain,
boxes, capsules and meshes. Its CAMERA query role remains independent of gameplay
hit detection and nonblocking actors.

The measured mesh timing is exclusively the extra refinement; it does not claim
that the earlier world collision query costs only 0.0067 ms. Both cases have the
same requested stationary orbit, but actual camera retraction differs: visual
foliage triangles are treated as solid by the old pass. This can also change the
visible workload. These are representative measurements, not every route/view.

The unchanged debug binary and hashes are saved in
`build/forest-performance-debug-baseline-v1/`. Raw frame/query samples and reports
are in `build/forest-camera-performance-v2-20261022/`. The initial failed harness
attempt is retained separately. No scene reduction or texture/lighting quality
change was used to obtain this comparison.

## Changes under qualification

Normal chunk worlds now use their authored world camera collision geometry.
Legacy monolithic proxy maps retain visual-mesh refinement. The explicit
`--camera-collision world|visual` override allows controlled comparisons. This
does not disable camera clearance or alter authoritative movement/collision.

`THEIA_PERFORMANCE_LOG=1` records actual wall-clock frame intervals and the extra
mesh pass's self time; it is absent in ordinary play. The temporary diagnostic
environment switch used for the initial A/B experiment has been removed.

The walkthrough launcher now expects optimized `target/release` binaries;
`--binaries target/debug` remains an explicit debugging option. Build with:

```sh
cargo build --release --locked --workspace
```

## Optimized measurements

The first optimized background-window test measured approximately 40 FPS in
both modes. Bevy's normal unfocused-window wait policy limited that comparison.
A separate explicit benchmark now uses continuous updates and `AutoNoVsync`
with `THEIA_PERFORMANCE_UNCAPPED=1`. Ordinary play retains its normal focus and
presentation policy. Build workloads and validation remain absent during sampling.

| Optimized uncapped camera mode | Sampled frames | Mean frame time | P95 frame time | Mean extra mesh time | Mean FPS |
|---|---:|---:|---:|---:|---:|
| Visual refinement | 982 | 15.41 ms | 16.23 ms | 12.68 ms | 64.90 |
| Authored world collision | 1,508 | 10.04 ms | 17.91 ms | 0.00009 ms | 99.65 |

These are the same full scene at the same requested stationary orbit, 1280 × 720,
with ten warm-up and fifteen sampled reports after actual chunk startup readiness.
Camera retraction and therefore the visible workload can differ. The remaining
world-mode frame-time spikes are retained in the report; this is an average
throughput improvement, not a guarantee of 60 FPS on every frame or view.

Raw samples, actual executable hashes and the measured scope are in
`build/forest-camera-release-performance-v2-20261022/`. The background-paced
optimized results remain in `build/forest-camera-release-performance-v1-20261022/`.
No source geometry, textures, material quality, collision or lighting is reduced.

All 45 client tests pass, including actual wall clearance with world mode and
zero visual mesh scans, plus the retained visual-camera and rebase tests.
Denied-warning workspace/all-target Clippy, format/diff checks and 221 asset
Python tests pass. The optimized client was reopened on the actual desktop with normal presentation
and focus policies (no uncapped benchmark override). Fifteen recent reports,
875 frames, measure 57.89 FPS average, 17.27 ms mean and 22.73 ms P95. Interaction
and focus are not constrained in this live check. The frame-time spikes remain
visible rather than being discarded. The window stays open for the user to walk;
no competing build or qualification workload is running.

The live report is `build/forest-walkthrough-focused-performance-20261022.json`;
its log is `build/forest-walkthrough-optimized-20261022.log`. Both native paths
report the discrete RX 7900 XTX / RADV NAVI31 Vulkan adapter.

This qualifies one machine/view/build. Broader routes, frame pacing, mips/LOD,
source lighting and eventual multi-character/MMO capacity remain separate work.
Do not equate functional screenshots or asset counts with a performance pass.
