# Actual town client startup/resource checkpoint

P1-10/P1-12/P2-06/P2-07/P2-08, execution-order row 5, 2026-10-20.
The actual partitioned town now passes normal encrypted two-client startup,
standard Bevy GLB/image loading and live Avian checks with ten initial packages
and with all 26 packages. This qualifies startup/residency for this dataset on
one Linux machine. Live replacement/staging, broader performance and default
adoption remain separate gates.

## Conservative bounds correction

The first actual client run rejected startup with `Visual exceeds indexed
contributor bounds`. The offline author used transformed source vertices,
whereas the client validates transformed mesh-local AABB corners. For rotated
sparse meshes, those corners can extend beyond every actual vertex. Translation
and box fixtures had not exposed the difference.

`index_chunk_contributors.py` now derives actual accessor bounds and transforms
all mesh-local AABB corners through the bounded static hierarchy. It no longer
uses a secondary scene importer or depends on reserved scene/node names. The
same strict runtime rejection remains in place. No tolerance or readiness check
was loosened. A rotated sparse-triangle regression demonstrates a missing corner
at x=198.485... while source vertices reach only x=195.657.... Existing tests
continue rejecting corrupt artifacts, animation, excess envelopes and budgets.

A new private stage, `build/town-resource-runtime-stage-v2`, changes only its index
metadata: 25 package envelopes expand conservatively. All package identities,
GLB/collision/image bytes, finite-world bounds and trusted spawns remain unchanged.
The maximum cell closure stays nine providers and the source spawns still require
ten. Dataset/admission identity changes with the corrected index. Earlier stages
and the monolithic launcher stay unchanged. Future partitioning automatically uses
the corrected author; earlier actual-vertex envelope reports are historical.

## Executed native cases

`build/town-resource-qualified-native-v2-20261020/` contains two actual two-client
cases, four reports/captures, on **AMD Radeon RX 7900 XTX / RADV NAVI31 Vulkan**
with active Khronos validation. Cases load ten packages/2,783 static colliders,
then all 26 packages/4,001 static colliders. Independent checks require exact
world identity/readiness acknowledgement, root ownership, source-bound shared
image/material settings and handle sharing, authority/rendered pose agreement,
actual capsule/triangle clearance, live Avian/camera, two-pawn pixels and no
unexpected movement recovery. The all-package screenshot was visually inspected.

The 26-package case starts with all packages resident; it is **not** ten-to-26
live publication. The source image union is 377 variants initially and 488 for
all packages. The new startup cases do not execute the old bespoke house routes;
those retain the preceding world-edge qualification's separate evidence.

## Sampled process/resource results

`tools/process_metrics.py` samples only descendant client/server processes via
Linux `/proc`. It records RSS, kernel high-water RSS, proportional set size and
swap when available. GPU counters come from DRM fdinfo, deduplicated by device
and client ID. The deprecated amdgpu `drm-memory-*` aliases do not add to resident
counters. Semantics follow the
[Linux DRM usage-stat specification](https://docs.kernel.org/gpu/drm-usage-stats.html).

Requested sampling interval is 250 ms, with actual timestamps retained. Short
peaks can be missed; GPU residency/CPU RSS/PSS/shared counters are distinct and
must not be added as a total physical footprint. These runs expose one measured
amdgpu DRM client per application client. Driver VRAM counters include buffers
beyond textures. Missing counters are unavailable, not zero. The headless server
opens no observed DRM client.

| Startup set | Per-client peak RSS | Per-client peak driver VRAM residency | Approx. client observation to ready marker |
|---|---:|---:|---:|
| Ten packages | 1.82–1.83 GiB | 1.54–1.60 GiB | 15.7–15.9 s |
| All 26 packages | 2.30 GiB | 1.84–1.85 GiB | 22.2 s |

Each row is one concurrent pair on a machine with 30.49 GiB OS-reported RAM and a 24 GiB discrete
GPU. Client readiness times are differences between sampled process appearance
and sampled marker-file appearance, not exact asset-decode timers. Their cadence
is approximate and observable in the retained samples. Server peak RSS is about
111.5 MiB / 152.3 MiB respectively. RSS includes shared mappings; PSS and raw
per-counter observations are also retained. Maxima can occur at different times.

This is a warm-cache development/debug-build qualification with Vulkan validation,
not a release benchmark, cold startup measurement, FPS/latency percentile, memory
budget guarantee, portable hardware minimum or MMO capacity claim. The sampler
itself and concurrent clients affect timings. CPU/GPU estimates in the older
budget analysis remain content arithmetic, not these measured process counters.

Evidence per case: `*-resources/samples.jsonl`, `*-resources/summary.json`,
`*-events.json`, client `network.json`/`scene.png`/`run.log`, server log and
`verification.json`. `build/town-resources-qualified-summary.json` binds the
current tools, application binaries and lockfile. The failed first run is retained
with `REJECTED.md`; its partial memory observations are not a residency baseline.

## Checks and remaining gates

Fresh fmt, denied-warning Clippy and **88 Python tests** pass: 84 asset tests plus
four GLB tests. New checks cover transformed sparse-mesh AABBs and measurement
units/FD duplication/legacy aliases. Rust application sources and binaries did not
change, so the previous 158-test plus three private-test checkpoint remains
historical rather than being reported as rerun for offline tools.

Next, measure actual live candidate preparation/publication, retained shared
image/resource lifetimes and retirement cleanup, including larger additions and
failed/superseded candidates. Then finish reversal/asymmetric-failure lifecycle
checks and rerun map regressions before adoption. Full source-world perimeter
accessibility, native Windows and sustained crowded/soak qualification remain
open. The normal launcher stays monolithic town revision 2 at 30 Hz, and all
private assets/output remain ignored. No whole task/phase is complete.
