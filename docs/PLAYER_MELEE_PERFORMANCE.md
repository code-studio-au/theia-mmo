# Melee-enabled client frame baseline — 2026-10-31

The source and binaries are unchanged from the [qualified melee slice](PLAYER_MELEE.md).
One normal native client uses actual local account/SQL services, the thirteen-role
melee-enabled bundle, the combat rig and animated wolf. Build jobs have finished,
presentation is uncapped and external Vulkan validation is disabled.

| Native mode | Frame-weighted mean | Frame cadence | Worst one-second p95 |
| --- | ---: | ---: | ---: |
| 1920×1200 windowed | 1.4055 ms | 711.50 FPS | 1.6138 ms |
| 3440×1440 fullscreen | 1.4032 ms | 712.63 FPS | 1.6082 ms |

Each result uses the last 15 one-second windows after startup/interaction warmup:
10,683 and 10,703 frames respectively. Both retain 81 chunks and record zero
unloads. The normal development input/widgets and active pursuit bundle run;
this is not a dedicated sustained-sword-action or GPU timestamp benchmark.

Hardware: Linux, Ryzen 7800X3D, RX 7900 XTX / RADV NAVI31, Mesa 26.2.6 Vulkan.
This measures frame cadence in a small near-spawn workload. It does not establish
population, terrain streaming, all-world navigation, Windows performance or a
controlled causal comparison with the prior pursuit build. Functional native
checks separately use Vulkan validation and record zero GPU diagnostics; their
frame rate is not the baseline above.

Evidence: `build/melee-performance-1920-v1-20261031`,
`build/melee-performance-3440-v1-20261031` and
`build/melee-performance-summary-v1.json`. Matched source/binaries/proof:
`build/melee-qualified-v1-20261031`. Owned services, private tickets and sealed
gameplay snapshots are cleaned after both runs. No source/content/art changes
were made for this measurement.
