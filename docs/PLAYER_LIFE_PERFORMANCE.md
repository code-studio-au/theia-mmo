# Player-life gameplay frame baseline — 2026-10-31

P1-12 instrumentation baseline, using the exact pre-wolf client/server in
`build/player-life-qualified-v1-20261031`. Unlike the correctness runs, this run
used one native client with external GPU validation disabled and no concurrent
Cargo/rustc build or other Theia client. Ordinary desktop applications remained.

At **1920×1200**, the final fifteen one-second telemetry windows contain 11,207
frames: frame-weighted mean **1.34018 ms**, or **746.17 FPS** uncapped. The worst
individual one-second p95 is **1.54453 ms**. This is not an aggregate p95 or a
minimum-frame-rate guarantee. The physical monitor is 3440×1440 at 120 Hz.
Device: RX 7900 XTX / RADV NAVI31, Mesa 26.2.6 Vulkan; Ryzen 7800X3D, 128 GiB RAM.

The run used actual owned OIDC/PostgreSQL admission, all thirteen gameplay
profiles, the animated POLYGON player, one patrol target and one guide near the
normal spawn. It followed inventory/equipment fixture actions and retained 81
chunks in one patch. No chunks unloaded during the measured windows. This is a
quiet gameplay baseline, not a streaming tour, forest benchmark, crowded fight,
MMO capacity result or measurement of the subsequently added animated wolf.

Proof: `build/player-life-performance-1920-v1-20261031/verification.json`,
`interrupt.log` and `frame-analysis.json`; the inventory PNG confirms the actual
1920×1200 window. Cleanup passed for this interrupt run. The earlier playable
checkpoint separately retains the three-mode launcher cleanup checks.

Reproduction command (requires the existing local source assets and SDKs):

```sh
python3 tools/qualify_account_launcher.py \
  build/NEW-player-life-performance \
  --client build/player-life-qualified-v1-20261031/theia-client \
  --server build/player-life-qualified-v1-20261031/theia-server \
  --player-life \
  --gameplay-manifest content/gameplay/development-gameplay-player-life.json \
  --gameplay-validator build/player-life-qualified-v1-20261031/validate_gameplay \
  --mode interrupt --steady-seconds 20 --window-size 1920x1200
```
