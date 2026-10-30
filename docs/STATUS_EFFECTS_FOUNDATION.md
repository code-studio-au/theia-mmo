# Abilities and status effects — P3-06 initial slice

Status: initial opt-in periodic-damage slice qualified on native Linux. Checked
definitions, authoritative timing, encrypted two-client replication, selected-target
Bevy presentation and protected feedback pass. This does not complete all of P3-06.
Actual prerequisites are checked ability definitions, authoritative cast lifecycle,
server-tick timing, bounded reliable combat events and selected-target Bevy UI.
Inventory is a useful adjacent completed slice, not a mechanical prerequisite for
periodic damage. This scope does not introduce classes, equipment or creature AI.

Add one opt-in development status ability and an immutable bounded status catalog.
Keep the qualified schema-1/schema-2 Bolt profiles available. Bind status definitions
to the admitted combat profile and explicitly version resource/status observations;
never silently reinterpret old packets or content. Development names, cadence and
damage are test parameters, not final balance or story decisions.

The initial ability applies a single periodic-damage status at successful cast
completion after authoritative target/range/line-of-sight checks. Mana follows the
existing actual-start cost policy. A failed completion does not fabricate an
applied status. Start with a one-second interval and six-second duration so exact
cadence can be checked at 20/30/60 Hz. First damage occurs after one interval;
the final interval at expiration is included. Duration/cadence use checked integer
simulation ticks, independent of render time or wall-clock sleeps.

One status instance per source/target/definition. Refresh extends duration while
retaining the next scheduled tick; repeatedly refreshing cannot manufacture early
damage. Different sources remain distinct. Bound both per-target and world-wide
active instances, and define rejection/cancellation before mutating beyond those
limits. Stable ordering must make simultaneous ticks and lethal hits reproducible.
Account for all new lifecycle/damage/death events within the existing bounded
reliable event budgets; no silent loss of critical events for a live client.

Clear statuses when their target dies, is removed or respawns with a fresh ID.
For this first transient sandbox slice, removing a source cancels its future ticks;
already applied damage remains. Restart clears transient combat state. This is an
explicit development policy, not persistent-zone status ownership, offline credit,
loot/quest attribution or a completed MMO death model. Durable vitals/combat state
remain required before promoting the complete persistent zone.

Show selected-target status name and remaining authoritative duration using standard
Bevy presentation. The client displays server observations and cannot schedule
hits or expiration. Preserve mobile casting, target death/respawn, resources,
backpack interactions and protected visual/audio feedback. Do not introduce custom
rendering, a second ECS or a distributed effect service.

Acceptance: definition bounds and unsupported-data rejection; exact ticks and
expiration at every qualified rate; refresh/no-extra-tick behavior; independent
sources, deterministic lethal hits, source/target retirement, fresh respawn and
capacity rejection; malformed/truncated/profile-mismatched observations; actual
encrypted two-client status replication; native Linux indicator and feedback under
cosmetic load; ordinary zero-cost/resource and account/backpack regressions.
Record exact source, binaries, device and Linux/Windows limits. Native Windows,
player buffs/debuffs, healing, dispels, crowd control, area effects, threat and
class/gear scaling remain separate work; this does not complete all of P3-06.


Implemented prerequisite: a checked schema-1 status catalog (32 KiB / sixteen
definitions, bounded temporary ASCII labels, 50–60,000 ms interval, duration at
least one interval and at most 60,000 ms, positive bounded periodic damage). The
development Scorch definition is 5 damage every second for six seconds; this is
test data, not final class balance. Checked schedules preserve their nominal next
tick across refresh, include an exact-expiration tick, reject missed due ticks
rather than unbounded catch-up, and fail atomically on clock regression/overflow.
Four focused contracts pass at 20/30/60 Hz, including fractional duration edges and
near-u64-exhaustion (`build/status-core-tests-v2.log`); core/all-target denied-warning
Clippy and all core tests pass. This prerequisite applies no damage and establishes
no entity/status ownership or network gameplay.

The initial implementation bounds active instances to sixteen world-wide and
eight per target. Reliable status observations use their own versioned small
frame, preserving CS01/CS02 resource observations: adding sixteen full status rows
to the existing worst-case 1,082-byte resource frame would exceed its 1,200-byte
budget. Status changes remain separate from transient presentation requests
and from durable inventory transactions.


Implemented authority and transport (2026-10-29): ability schema 3 explicitly
references a checked status ID and requires the resource/status profiles. Their
exact definition bytes bind the admitted world identity. Schema-1 zero-cost and
schema-2 resource profiles retain their existing identities and CS01/CS02 bytes.
The server applies Scorch only after successful completion checks. Capacity is
checked before cost and again at completion; a started cast retains its cost if
another source takes the final slot. Sixteen world/eight target limits also bound
the deterministic target/definition/source tick loop. Source removal cancels its
future ticks; target death/removal clears all its instances before fresh respawn.

Five authority contracts pass, including exact cadence at 20/30/60 Hz, replay,
refresh, independent sources, lethal-hit ordering and the completion capacity race
(`build/status-authority-regression-tests-v2.log`, alongside legacy combat/resource
regressions). SS01 status observations are separately bounded at 550 bytes, carry
the authority tick rate and reject invalid deadlines, duplicates, ordering,
foreign epochs and unknown definitions. Three codec contracts pass. The runtime
fails closed on an unrequested status profile, inconsistent tick rate, missing
profile or reliable-channel backpressure; it never accepts a status command.

An actual encrypted two-client test through delayed/lossy/duplicating UDP relays
passes (`build/status-encrypted-test-v2.log`): initial empty state, independent
sources observed by both clients, refresh without a second instance, periodic
damage events, source disconnect removing only that source and remaining-effect
expiration. This uses developer-issued encrypted tickets, not OIDC/database
admission; status state remains transient.

`python3 tools/play.py --status-effects --spell-feedback` selects the new opt-in
Scorch profiles. It implies resources/combat/targeting, uses the existing movement
and cast paths, and adds a standard Bevy selected-target status display with
source label and remaining server-tick duration. The display never schedules
hits or clears server state. Critical periodic-hit cues reuse bounded protected
feedback with a distinct serial per event, including repeated cast sequences.
Native GPU, integrated backpack and ordinary launcher checks now pass; see the
qualified evidence below.

Current source regression: `cargo test --locked --workspace --all-targets -j1`
passes 336 tests across 71 groups with fifteen explicit skips
(`build/status-workspace-tests-v1.log`). The five PostgreSQL contracts execute
separately and pass, including restart, latest-intent recovery and backend loss
(`build/status-postgres-v1-20261029/verification.json`); ten private-asset skips
remain skips. Workspace/all-target denied-warning Clippy passes
(`build/status-workspace-clippy-final-v1.log`), formatting passes, and all five HTTP
helper tests pass. The standalone server dependency graph excludes rendering,
audio, UI and wgpu (`build/status-server-dependencies-v1.txt`). These checks alone do not
establish native presentation or Windows execution.


Native qualification: `build/status-native-wifi-v1-20261029/verification.json`
passes with two actual encrypted native clients, delayed/lossy/duplicating UDP,
Vulkan validation and 32,768 cosmetic requests per client. Each observes 84
periodic damage events and four deaths/fresh respawns; mana rejection counts are
seven/six, with recovery. Both render their own and the other source, show cleared
state, and deliver all 88 critical visuals/voices each with no critical overflow.
Actual resource/cast/health fill-width error is below 0.467 px. Active/cleared and
backpack screenshots were inspected. Device: Linux Ryzen 7800X3D / RX 7900 XTX,
RADV NAVI31 / Mesa 26.2.5 kisak Vulkan, 1280×720. Mean rates 69.99/69.75 FPS are
validation-plus-cosmetic-stress observations, not an ordinary-play baseline or MMO
population-capacity result. No native Windows execution is claimed.

Ordinary launcher interruption/child-failure passes with status effects and
feedback (`build/status-launcher-v1-20261029/verification.json`); resource-only and
zero-cost profiles also pass (`build/status-resource-regression-v1-20261029` and
`build/status-zero-cost-regression-v1-20261029`). Actual OIDC/PostgreSQL mixed
backpack/status/resource/feedback launch passes interruption, child failure and
pending-browser cancellation (`build/status-backpack-launcher-v1-20261029`). Its
native widget script performs six durable mutations and a rejected split; this
mixed launcher loads Scorch but does not cast it. The separate impaired native
proof above exercises Scorch. Owned processes, tickets, database ownership and
plaintext fixture credentials are cleaned up. Browser login uses an owned command
interceptor with a real provider exchange; physical browser/mouse interaction is
not claimed.

Engineering changes remain local. Qualified source/binaries/evidence are frozen in
`build/status-qualified-v1-20261029`; prior inventory/resource/admission checkpoints
remain preserved. Resources/statuses/cooldowns still reset on a fresh actor;
source departure cancels future periodic ticks. No durable combat, player damage
or death, healing, dispels, crowd control, equipment/class scaling, threat, reward
credit or persistent-zone status ownership is implied.


Ordinary baseline with the same binaries: the persistence/combat/status-disabled
single-client saved-terrain tour passes at **822.41 FPS uncapped**, validation off
(`build/status-legacy-baseline-v1-20261029/verification.json`). This is the empty
graybox traversal baseline, not a populated-world or MMO capacity result.
