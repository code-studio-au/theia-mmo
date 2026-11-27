# Stats and resources — P3-04 initial qualified slice

Status: initial opt-in development slice qualified on native Linux. Broader
P3-04 and persistent vitals remain open.
The authenticated-admission checkpoint is preserved separately. Actual prerequisites are the bounded ability catalog,
authoritative cast lifecycle, server-tick timing, reliable combat state and Bevy
combat UI. Character/account ownership now provides the later persistence binding;
inventory, equipment and creature AI are not prerequisites for this first slice.

Implement one explicit development profile with checked maximum health, maximum
mana and mana regeneration. Use immutable bounded JSON definitions and integer
resource arithmetic, with carry for fractional per-tick regeneration. A second
ability-data schema can declare mana costs; preserve the existing schema-1,
zero-cost development ability and qualified binaries as regression paths. Initial
100 health, 100 mana, 5 mana/second and a 25-mana Arcane Bolt are development test
parameters, not approved class design or final game balance.

The authority owns current values. Charge mana once when a cast actually starts;
queued requests do not spend until their start, and superseded/replayed/invalid
requests do not spend. Recheck availability at queued start. Insufficient mana
must not start a cast or its GCD/cooldown. An interrupted started cast keeps its
cost in this initial profile. Regeneration follows simulation ticks at 20/30/60
Hz, caps at maximum and cannot bank a hidden surplus while full. No renderer,
wall-clock timer, database operation or transport await enters this computation.

Replicate authoritative player vitals through a bounded, explicitly identified
combat profile. Clients must reject incompatible definitions or malformed resource
state. Use standard Bevy health/mana bars and action feedback; do not make the
client's displayed values authoritative. Existing target health, mobile casting,
death/respawn and protected feedback remain working.

The first resource profile is opt-in. Current ordinary pose persistence does not
save combat cooldowns or vitals; full resources on a fresh development actor must
be recorded as a temporary sandbox policy, not durable MMO state. Persisting
player vitals and closing reconnect/reset exploits is required before promoting
this profile as persistent-zone gameplay. Do not silently add resource columns to
old migrations or claim the whole P3-04/P3-16 scope is complete.

Acceptance for the first playable slice: definition and arithmetic bounds; exact
regeneration at all qualified tick rates; once-only costs across replay, queue,
replacement, failure and interruption; actual encrypted two-client resource
replication; an exhausted cast rejected and later available after regeneration;
authoritative Bevy bars/feedback on native Linux; existing movement/target/combat,
account admission and ordinary launcher regressions. Record exact assets/binaries,
devices and limitations. Production class attributes, gear modifiers, persistent
vitals, player-damage/death gameplay, native Windows and final balancing remain
separate work.


Implemented locally: checked profiles/pools and schema-2 costs; authoritative
spending at actual start, queued-start rechecks and tick regeneration; bounded
CS02 combat observations retaining the original CS01 zero-cost wire profile;
opt-in standard Bevy player bars and resource feedback. Five authority tests cover
20/30/60 Hz, replay, queue replacement, failure, interruption and fresh-actor reset.
All five encrypted combat tests pass, including two independently spending clients
under loss/jitter/duplication, insufficient-mana rejection and recovery. The test
waits for cast completion before checking a direct rejection: a request received
during a cast legitimately queues and reports a later failure as an event.

`python3 tools/play.py --resources` selects the transient development profile;
`--spell-feedback` can be combined with it. `--character-stats` overrides the
checked profile and `--abilities` overrides the catalog. Ordinary `--combat`
retains its original zero-cost catalog. Native qualification and broader regression
checks below pass. This qualifies the initial development slice only.


Qualification — 2026-10-29:

- `build/resources-core-network-tests-v4.log`: 238 passing / ten explicit private
  fixture skips; `build/resources-apps-auth-persistence-tests-v1.log`: 71 passing /
  three explicit PostgreSQL skips. Together these cover all workspace targets.
  The PostgreSQL contracts were separately executed on unchanged persistence
  source in prior account checkpoints. No skipped test counts as a pass.
- The final presentation-only adjustments pass `resources-fmt-final-v3.log`,
  `resources-workspace-clippy-v5.log` and the native run below. The layout change
  also passed two focused UI tests in `resources-ui-regression-v2.log`.
  Five Python HTTP helper tests pass in `resources-account-helper-regression-v1.log`.
  `resources-server-dependencies-v1.txt` confirms the standalone normal server
  graph excludes renderer/audio/UI/wgpu.
- `build/resources-native-wifi-v3-20261029/verification.json`: two real native
  encrypted clients, impaired UDP, real widget/keyboard commands, mobile casting,
  independent mana, depletion, six mana rejections per client, regeneration and
  later casts, target death/respawn and protected audio/visual feedback under
  cosmetic load. Mana minima were 0 and 1; maximum bar-fill error was 0.467 px.
  Final damaged/depleted/recovered screenshots were inspected. Earlier v1 exposed
  overlapping panels and awkward cost wrapping; v2 passed with fixed layout;
  final v3 additionally replaces misleading Ready text with Needs mana.
- `build/resources-legacy-baseline-v1-20261029/verification.json`: ordinary
  resource-disabled, one-client empty-world baseline, 816.15 FPS uncapped,
  validation off. This is separate from the two-client validation/stress run
  (75.97/73.04 FPS) and does not establish MMO population capacity.
- `build/resources-launcher-v1-20261029/verification.json`: actual ordinary
  `play.py --resources` routes the explicit catalog/stats to server and both
  clients; interruption and child failure remove owned children and tickets.
- `build/resources-account-launcher-v1-20261029/verification.json`: new binaries
  with actual owned Keycloak/PostgreSQL pass account launcher interruption, child
  failure and pending-browser cancellation. Joined modes commit final poses;
  database ownership closes and private credentials are removed. This account
  regression uses the ordinary resource-disabled world. Earlier admission proofs
  remain preserved separately; no durable resource claim is made.

Native device: Linux, Ryzen 7800X3D, AMD Radeon RX 7900 XTX (RADV NAVI31),
Mesa 26.2.5 kisak, Vulkan, 1280×720. Validation layers enabled for the two-client
resource run; disabled for the ordinary performance baseline. No native Windows
execution or physical-speaker/HRTF quality claim. Audio capture reads only an
owned null sink; desktop/microphone streams are excluded.

Final client SHA-256:
`799612612325ca91e99efa9bdac048d45ec6a903c9dc5985a5e4f055f5cbd543`.
Final server SHA-256:
`77bac3baf8d129ef17a0d5786574b4547a2dadd8dfcee7d0492c2b5ac55210a0`.
Exact source, binaries, assets hashes and evidence are preserved in
`build/resources-qualified-v1-20261029`; superseded native source/binaries are
retained separately there. No new commit or push accompanies this checkpoint.
