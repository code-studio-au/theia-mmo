# Eight-peer saved-terrain soak

P1-05/06/12 and P2-03/04/06/07 continuation, 2026-10-23. Prerequisites: the
qualified saved continent, secure Renet, shared Avian movement, bounded prediction
clock/queues and native two-client evidence. This test uses eight actual encrypted
UDP clients with independent saved-data/query workers and the production driver.
It runs client/server tasks inside one headless process. Published readiness is
**collision-only test readiness**, not GPU/native-client readiness. It makes no
claim about eight rendered clients, Internet deployment or MMO population capacity.

## Acceptance

- Eight authenticated peers with unique authority actor IDs; every peer initially
  receives the real eight-player snapshot, then moves its own authority actor.
- Independent cardinal/diagonal run/jump/stop routes spread beyond the existing
  1,024 m relevance filter and return over retired cells. No simulated connection
  objects, position submission, proxy NPC counts or untrusted teleport commands.
- Actual saved file reads, collision publication and retirement. At most nine
  published authority query patches (eight actors plus admission), two per
  headless query runtime (admission plus moving actor), one pending worker per
  runtime. Native client residency remains separately qualified at 81 visual roots.
- No movement recoveries, malformed input/readiness rejections, missed simulation
  deadlines or skipped fixed intervals; transport queues <=8,192 bytes/channel
  and client command/history bounds retained. Simulation defaults to 30 Hz.
- Record timing, source/driver/transport counters, visited cells and sampled Linux
  RSS. The 1 GiB combined-process ceiling is a conservative test guard, not a
  production memory budget. One-second RSS samples can miss short-lived peaks.
- Remove bearer tickets after controlled completion. Ctrl+C writes an interrupted,
  failed qualification report rather than declaring the requested duration passed.

## Run

With the baked package and existing native SDK environment:

```sh
cargo build --release --locked -p theia-network --example saved_terrain_soak
target/release/examples/saved_terrain_soak build/graybox-terrain-v1 NEW_OUTPUT 180
```

The output directory must be new. Duration is 20–600 seconds, default 180. Routes
reverse every 120 seconds to revisit cells while remaining inside the finite map.
A 20-second smoke checks admission/movement/queues; it cannot qualify the longer
retirement/relevance checks. A duration >=150 seconds requires measured interest
pruning. All samples/reports/tickets remain under ignored build output directories;
use a build path for NEW_OUTPUT. Windows has no /proc RSS sample and requires its
own native execution evidence. No GPU/animation/presentation evidence comes from
this program; use the [native clock/terrain qualification](PREDICTION_CONTRACT.md).

## Ten-minute return-identity qualification — 2026-10-26

`build/saved-soak-eight-600-v3/verification.json` passes: 600 requested seconds,
600.17 elapsed, all eight peers observe pruning and the same eight identities
return, and temporary tickets are removed. Actual simulation runs 18,002 steps
at 30 Hz with zero missed simulation deadlines, skipped intervals, movement
recoveries or input/readiness rejections. Authority generates 245 collision
patches and retires 236, maximum/final nine. Each peer visits 7–9 chunks, generates
29–39 patches and retires 27–37; query residency stays at two and history at four.
Movement/event queue peaks are 856/178 bytes against 8,192-byte limits.
Simulation work p95 upper bound is 1,048.575 microseconds; maximum 9,554.6
microseconds. The separate 5 ms I/O pump records **61 missed deadlines** during
concurrent compilation/tests; this is retained, not reported as zero server-wide
misses. This run is not an isolated throughput benchmark.

Sampled combined peak RSS is **169,689,088 bytes (161.83 MiB)**. Actual samples in
the final 120 seconds range from 130,297,856 to 131,182,592 bytes (124.26–125.11
MiB), below the early peak. The progress log's `rss_bytes` field prints the running
peak; the JSON samples contain current RSS. These repeated routes support bounded
residency/memory for ten minutes, not an indefinite leak-free guarantee.
One same-tick stable-readiness error reaches **0.2193 m** despite loopback; input
agreement is not yet classified. Readiness stability alone does not establish
identical applied commands. Further correction diagnosis remains open.

Qualified example binary SHA256:
`02296f614128e9d4cb1dc57c29f703e73644de9ddf12b669501e5057c13d5c33`;
example source SHA256:
`7f1a8de880d3a0773936ecbca25cc109f727e0c5f1d86f1154e45737ddf53b44`.
Saved manifest SHA256 remains
`897328cff112a83f5e70453709ad36e4323e9c349b84b3e04b2c61a0923ece13`.
Linux Ryzen 7800X3D, actual secure loopback UDP, headless collision readiness.
No GPU or native Windows execution is claimed. The v3 return-identity checks
previously left unqualified at the stop below now pass. Changes remain local.

## Completed checkpoint and safe stop — 2026-10-23

The eight-peer, 180-second run passes:
`build/saved-soak-eight-v2/verification.json` (`passed: true`, no failures,
`tickets_removed: true`). All eight actual peers move about 720 m from spawn,
visit 5–9 chunks each and observe relevance pruning. Authority collision residency
peaks at nine patches: 71 generated, 62 retired. Each client query runtime peaks
at two patches and retires 5–11. There are 5,402 simulation steps, zero missed
server deadlines, skipped intervals, movement recoveries or readiness rejections.
Simulation work p95 upper bound is 1,048.575 microseconds, maximum 1,101.893
microseconds. Sampled combined RSS peaks at 174,407,680 bytes (166.33 MiB).
RSS growth slows during this run but does not establish a long-term plateau.

The earlier 20-second smoke also passes
(`build/saved-soak-smoke-v1/verification.json`), with actual ticket cleanup.
The completed 180-second run uses the v2 example. Subsequent example-only checks
for the same eight identities returning after relevance pruning, plus additional
trace fields, compile and pass denied-warning workspace/all-target Clippy
(`build/saved-soak-clippy-v3.log`, `build/saved-soak-build-v3.log`), but have **not
been runtime qualified**. Do not attribute these additional checks to the v2 pass.
The 234-test full workspace checkpoint predates the new soak example and server
terrain telemetry; it is not a fresh full-suite result for those additions.

The user requested a safe stop. The build completed; no longer soak was started.
No 600-second memory plateau, native Windows, eight rendered clients or MMO
population capacity is qualified. Changes remain local and uncommitted.
