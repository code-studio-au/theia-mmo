# First ability-data contract

Current continuation, 2026-10-28: the catalog is now consumed by the optional
server-owned/native [ability slice](COMBAT_AUTHORITY.md), qualified by 272 workspace
tests and actual impaired two-client Linux GPU checks. The dated data-only
prerequisite below describes its original boundary, not the current runtime.

P1-11 prerequisite slice, 2026-10-26. Actual prerequisites are stable content IDs,
qualified simulation rates and the adopted mobile tab-target combat architecture.
The current movement/terrain authority remains unchanged. This work prepares data
for P2-11/12; it does not implement targeting, damage application or a playable
combat loop.

## Design and acceptance

Load a bounded schema-1 JSON catalog into immutable validated definitions in
`theia-core`, independent of rendering and transport. Use existing serde packages;
introduce no plugin or engine dependency. Stable logical ability IDs, positive
range, one flat damage effect, cast/cooldown duration, mobile-cast permission and
per-ability global-cooldown participation are the initial supported fields.
Mobile casting and global-cooldown participation default to true. The supplied
prototype uses the adopted configurable 1,500 ms GCD and 300 ms queue window.

Reject unknown schemas/fields/effects, invalid/duplicate IDs, malformed/nonfinite
ranges, invalid damage, excessive timing and oversized/count-exceeding catalogs.
The initial bounds are 64 KiB JSON, 64 abilities, 128 m range, 60 s cast/cooldown,
10 s GCD, 1 s queue window and 1,000,000 damage. These are prototype admission
budgets, not class balance or production capacity. Reject unsupported content
explicitly rather than silently ignoring it.

Convert all millisecond durations to simulation ticks with the existing ceiling
conversion at 20/30/60 Hz. Zero cast duration remains instant; zero GCD or explicit
off-GCD remains zero. Do not implement wall-clock cooldowns. Acceptance requires
real development-data parsing, fail-closed malformed/budget cases, mobile/off-GCD
options, tick-rounding cases, denied-warning Clippy and formatting.

## Integration boundaries

The example ability is development data, independent of pending story approval.
The future server resolves its own catalog and owns target/lifecycle/range checks,
cast execution, cooldown/queue state, damage, death and respawn. Clients must not
submit authoritative effect values or timestamps. Receipt sequencing, reliable
combat events, target overrides, line-of-sight policy, range-at-start/completion,
cast cancellation and presentation require the P2-11/12 integration design.
No account persistence, economy, creature AI or navmesh is added by this schema.

## Qualified data prerequisite

The actual checked-in development catalog loads successfully in all five new
schema/timing tests. All **16 core tests** pass (11 existing plus five ability
contract tests; `build/ability-core-tests-v1.log`). Workspace/all-target
denied-warning Clippy and formatting pass (`build/ability-clippy-final-v2.log`,
`build/ability-fmt-final-v1.log`). Only existing workspace serde/serde_json packages
are added to the core's dependencies; no package version or engine pin changes.

The offline validator builds and actually accepts the development data at
20/30/60 Hz, with expected tick timing. Unsupported-effect and oversized files
exit unsuccessfully without emitting an accepted catalog:
`build/ability-validator-v1/verification.json`. Run it with:

```sh
cargo run --locked -p theia-core --example validate_abilities -- content/gameplay/development-abilities.json 30
```

An initial final-Clippy invocation omitted this machine's required native SDK
`PKG_CONFIG_PATH` and failed in Wayland dependency discovery
(`build/ability-clippy-final-v1.log`). The corrected v2 invocation uses the recorded
SDK environment and passes; this was not a Rust source failure. No native GPU
check is implied by the offline validator. The preceding 236-test full workspace
and 600-second soak are the movement diagnostic checkpoint, before this module.
No combined 241-test workspace result is claimed.

The bounded data prerequisite is complete. P1-11 remains partial until actual
authoritative gameplay consumes these definitions; targeting, casts and combat
remain unimplemented. Existing terrain/movement regressions remain preserved.
