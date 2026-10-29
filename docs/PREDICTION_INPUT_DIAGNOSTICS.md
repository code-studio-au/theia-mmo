# Completed-input prediction diagnostics

P2-04/P1-12 continuation, resumed 2026-10-26 from commit `9497a18`.
Prerequisites are the saved/generated terrain controller, bounded completed-tick
prediction history, secure Renet, and existing impaired-UDP qualification.

Each of the existing maximum 64 history entries records its completed input
state alongside its pose. Replaying a retained entry refreshes both measurements.
A matching-tick correction compares:

- Held intent: axes, sprint and jump level at that completed tick.
- Applied command sequence: which command the authority actually applied.
- Freshness: the simulation tick at which the command was received.

The driver partitions matched observations into held-intent agreement/disagreement
counts and maxima. Sequence/freshness disagreement has independent counters.
All observations are bounded aggregates; there is no growing correction trace.
Measurement fields do not select a solver, modify a command or affect movement.

Matching intent at one tick does not mean preceding inputs, receipt times,
collision readiness or full motion state match. Sequence disagreement can exist
while the same movement remains held. Freshness differences can be expected
because speculative commands are optimistically received immediately before their
target tick. These fields support diagnosis; they do not by themselves prove a
solver defect or establish why an earlier correction happened.

## Acceptance

- Analytical tests distinguish missing optimistic commands, identical intent with
  different sequence/freshness, and replay refreshing retained input measurements.
- Full workspace tests, denied-warning all-target Clippy and formatting pass.
- Actual encrypted saved/generated impaired-UDP reports retain bounded history,
  readiness withdrawal/regrant, traversal/jumping and stopped convergence checks.
  Agreement/disagreement counts sum to aligned observations; sequence/freshness
  mismatch counts never exceed that aligned population.
- Eight actual saved-terrain peers repeat spread/return and retirement without
  concurrent builds. Preserve identity, residency, input/queue and simulation
  acceptance. Record I/O timing separately and label current/peak RSS correctly.

## Evidence status

The full workspace suite passes **236 tests**, with ten explicit private-fixture
skips (`build/prediction-input-workspace-v2.log`). Denied-warning workspace/all-target
Clippy and formatting pass (`build/prediction-input-clippy-v2.log`,
`build/prediction-input-fmt-v2.log`). This supersedes the interrupted v1 attempt.
Actual encrypted UDP reports are under `build/prediction-input-network-v2`:

| Saved terrain profile / rate | Aligned samples | Different held intent | Largest error with same / different intent | Largest stable-readiness error |
|---|---:|---:|---:|---:|
| Moderate, 20 Hz | 176 | 0 | 0.4020 m / 0 | 0.0191 mm |
| Moderate, 30 Hz | 267 | 0 | 0.2680 m / 0 | 0.0162 mm |
| Moderate, 60 Hz | 396 | 0 | 0.5360 m / 0 | 0.0274 mm |
| Poor, 20 Hz | 73 | 3 | 1.2062 m / 1.6085 m | 1.6085 m |
| Poor, 30 Hz | 267 | 5 | 0.5361 m / 0.4771 m | 0.4771 m |

Moderate means 40–100 ms per direction, every tenth packet dropped; poor means
100–250 ms, every fifth dropped. Both include duplication/reorder. Pose errors in
the same/different-intent columns include pending readiness; the last column is a
separate readiness population. They must not be conflated. All supported cases
traverse/jump, hold/resume readiness, converge below 3 mm after stopping and retain
bounded history. At poor-link 60 Hz, actual RTT causes safe suspension, no commands
or prediction; that case is not qualified high-latency movement.

The poor 20/30 Hz cases respectively record 10/72 applied-sequence mismatches and
63/231 freshness mismatches. Both matching and different held intent can accompany
pose errors. These totals/maxima do not prove the cause of the largest outlier;
severe-link smoothness remains open. Moderate generated terrain also passes all
three rates. These are controlled wall-clock samples, not universal error bounds.

The uncontended 600-second soak now **passes** in
`build/prediction-input-soak-600-v1/verification.json`. All eight actors return
with their identities preserved; terrain residency, diagnostic partitions and
cleanup pass. Both simulation and I/O report zero missed deadlines. Among the
143,730 aligned observations, four different-intent samples occur across two
peers, with error maxima 0.2386/0.2294 m. The maximum error across matching-intent
samples is 0.0374 mm. This is an observed association, not a causal proof or a
universal maximum. Receipt freshness commonly differs even with agreeing intent;
that comparison is not independently a physics failure.

[Full soak scope and timings](SAVED_TERRAIN_SOAK.md). Exact binary/source/content
hashes are in `build/prediction-input-qualified-v1/artifacts.json`. No builds or
other project qualification tests ran alongside this soak. The subsequent
ability-data module is a separate P1-11 slice and was not part of these binaries.

No renderer, wire version, server input window, prediction clock, expiry or
smoothing behavior changes in this slice. Native GPU evidence remains the earlier
qualified clock/terrain checkpoint; no new native Windows or Internet/population
capacity claim follows from these headless checks.
