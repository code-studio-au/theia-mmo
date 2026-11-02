# Pursuit client frame baseline — 2026-10-31

The exact source and executables are unchanged from
[the qualified playable pursuit checkpoint](CREATURE_PURSUIT_NATIVE.md).
One normal native client, real account/SQL services, the combat rig and animated
wolf were exercised near spawn with the pursuit bundle enabled. Build jobs had
finished; presentation was uncapped and external Vulkan validation was disabled.

| Native mode | Frame-weighted mean | Frame cadence | Worst one-second p95 |
| --- | ---: | ---: | ---: |
| 1920×1200 windowed | 1.4024 ms | 713.08 FPS | 1.5974 ms |
| 3440×1440 fullscreen | 1.4037 ms | 712.41 FPS | 1.6214 ms |

Each result uses the last 15 one-second windows after startup/interaction warmup:
10,712 and 10,700 frames respectively. Both retain 81 terrain chunks with zero
unloads. Authority observes three acquisitions, 78 published path requests,
chasing/attacking and completed returns during each full run; no failed or
blocked paths are recorded. These runs include normal development input/widgets,
not a continuously stationary camera or a dedicated GPU timestamp benchmark.

Hardware: Linux, Ryzen 7800X3D, RX 7900 XTX / RADV NAVI31, Mesa 26.2.6 Vulkan.
The results describe client frame cadence in this small near-spawn workload.
They do not establish streaming, NPC/population capacity, all-world navigation,
Windows performance or a controlled attribution of AI cost versus earlier builds.
The separate native functional runs retain active Vulkan validation and zero GPU
diagnostics; their frame rate is not used as this baseline.

Evidence: `build/pursuit-performance-1920-v1-20261031`,
`build/pursuit-performance-3440-v1-20261031` and
`build/pursuit-performance-summary-v1.json`. Matched log hashes, binary/source
references and proof are preserved in
`build/pursuit-performance-qualified-v1-20261031`. Owned services, private tickets
and sealed gameplay snapshots are cleaned after both runs. No source, gameplay
content or asset changes were needed for this measurement.
