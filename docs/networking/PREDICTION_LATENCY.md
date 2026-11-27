# Saved-terrain prediction latency envelope

P1-06/12 and P2-04 continuation, resumed 2026-10-26. Prerequisites are the
qualified saved terrain, shared controller, completed-tick correction contract and
bounded client clock. This extends real encrypted UDP evidence; it changes no
server authority, movement clock, command window, expiry or wire format.

## Contract and acceptance

The existing command window is at most eight ticks and at most one third of a
second. The prediction driver reserves two ticks of that window. Its measured
half-RTT guard consequently suspends command production above these RTT values:

| Simulation rate | Future window | RTT suspension threshold |
|---|---|---|
| 20 Hz | 7 ticks | 500 ms |
| 30 Hz | 8 ticks | 400 ms |
| 60 Hz | 8 ticks | 200 ms |

These are code thresholds, not promises of smooth play at those limits. Delay
variation, correction age, readiness changes and input expiry can narrow useful
operation. A correction older than 500 ms separately suspends production. The
existing default remains 30 Hz. No command window is enlarged for this test.

The existing deterministic poor-link relay adds 100–250 ms to each direction,
drops every fifth packet and duplicates every eleventh retained packet; differing
delays also cause reordering. The saved-terrain integration test exercises:

- At 20 and 30 Hz: local response, actual authoritative seam traversal, jumping,
  readiness withdrawal holding movement, regrant resuming movement, stopped pose
  convergence below 3 mm and bounded prediction history/transport queues.
- At 60 Hz: actual measured RTT outside the supported window produces no movement
  commands despite held run/jump input; authority remains stationary and history
  stays bounded. This is safe suspension, not successful high-latency movement.

The existing saved/generated 40–100 ms-per-direction profile remains a regression
at all three rates. Both tests use actual production Renet and prediction code,
with collision-only headless readiness. GPU presentation, native Windows, real
Internet traffic and MMO population capacity need separate evidence.

## Execution status

The focused integration test passes:
`build/saved-prediction-poor-v3.log` and
`build/saved-prediction-poor-v3/saved-prediction-poor.json`.
At 20/30 Hz, first local response is 45.63/28.34 ms, stopped error is
7.79/0.36 micrometres and history peaks at 12/17 steps. Both runs have zero
movement recoveries and missed simulation deadlines. Largest stable-readiness
same-tick errors are **0.4020 m / 0.0166 mm**; these remain explicitly recorded.
At 60 Hz, 395 observations suspend production, zero commands are queued and no
movement is predicted. The 20/30 Hz cases use 14 seconds each to retain the same
minimum 50 stable aligned observations under greater loss; 60 Hz observes two
seconds of safe suspension. This does not qualify rendered smoothness.

Earlier attempts remain in `build/saved-prediction-poor-v1.log` (a new assertion
incorrectly compared capsule feet with body-centre streaming focus) and `-v2.log`
(the original ten-second window collected fewer than 50 stable aligned samples).
The final test compares like coordinates and extends observation time; movement,
queue/history, response and convergence checks were not weakened.
Reports are emitted under `THEIA_NETWORK_REPORT_DIR`; generated assets, reports
and tickets stay out of Git. The refreshed full workspace suite, including the new regression, passes
**235 tests** with ten explicit private-fixture skips
(`build/resume-workspace-tests-v2.log`). Denied-warning workspace/all-target
Clippy and formatting pass (`build/resume-clippy-v2.log`, `build/resume-fmt-v2.log`).
The earlier baseline refresh also passes 234 tests before this added regression.
Python checks pass 221 asset tests and four GLB alignment tests; the first alignment
invocation used the wrong import root, corrected by unittest discovery in
`build/resume-align-v2.log`.
