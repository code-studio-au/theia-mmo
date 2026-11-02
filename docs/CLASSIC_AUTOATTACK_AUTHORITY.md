Current follow-up: [explicit melee transport](CLASSIC_MELEE_TRANSPORT.md) replaces
the historical runtime refusal described below. Normal application presentation
is still pending; the following evidence records the preceding authority slice.

# Basic melee authority — local headless implementation

P2-12 / P3-06 / P3-08 extends the [qualified clock](CLASSIC_MELEE_CLOCK.md) and
[persistent targeting](CLASSIC_TARGETING.md). Headless qualification passes.
The playable checkpoint remains `build/classic-targeting-qualified-v2-20261101`.
The complete strict combat milestone remains five of thirteen checks.

## Current behavior

An inactive classic predicted combat World can enable a bounded weapon profile
and its matching item catalog. Their exact bytes are hashed into gameplay
identity after ability/player-life configuration and before the later World
roles and outer classic identity. Enabling after admission, enabling twice or
using a legacy World fails. The normal launcher does not enable this profile.

Trusted session-owned requests carry world epoch, current movement stream,
monotonic request sequence and an explicit target or stop. Old epoch/stream,
unknown owner and repeated/old sequences are ignored. This prevents an old-life
request arriving after respawn from restarting engagement. Accepted engagement
explicitly selects a valid known creature; stopping engagement leaves selection.
Changing/clearing selection ends the old engagement. Stop-on-target-change is
a provisional Theia policy; Classic preference/default calibration remains open. Re-engage and retarget
preserve cooldown; a changed committed weapon resets its period. An item revision
with the same weapon preserves readiness.

Actual fixed steps check committed equipment availability and item-catalog hash,
actor/target life, current authoritative character facing, metric centre-distance
reach, static Avian LOS, prepared collision and client readiness for both actor
and target. Range/facing/LOS/readiness/equipment failures pause without clearing
engagement. No movement, automatic approach, facing assistance or camera input is
introduced. Movement/jumping alone do not prohibit a valid basic hit.

The current integration temporarily pauses basic attacks during an active cast
while letting the weapon countdown elapse; GCD alone does not own that timer.
The cast completion and a valid basic swing can occur in the same tick. Per-
ability continue/suspend/delay/reset data and ability-facing validation remain
required before final combat acceptance; this temporary cast gate is not a claim
of exact Classic spell behavior.

Each actual basic hit has a separate `MeleeDamage` event with weapon, owner,
swing sequence, victim, position and applied/remaining health. The shared damage
routine handles clipping, death, status cleanup and retaliation. Reward/XP fact
extraction recognizes a fatal weapon hit for the same checked creature life,
with duplicate fact suppression. Durable grants still occur outside simulation.
Target removal/death, owner death/respawn and leaving the session end engagement.
An exhausted swing counter or event budget cannot authorize another hit.

## Boundaries and next adoption

The existing event codec rejects basic weapon events. The existing Renet owner
refuses an autoattack-enabled World before serving a client. These are explicit
adoption boundaries while bounded engage/cancel/state and weapon-event transport
are implemented; no hidden conversion into repeated Sword Slash casts is used.
Normal-client right-click still belongs to the older ability experiment.

The training weapon remains a Theia fixture: 2400 ms / 15 base damage / 3 m /
180-degree arc, with initially immediate readiness. Basic hits currently always
hit; Classic avoidance/skill/AP/modifier rules remain separate evidenced work.
Current status data does not implement stun/disarm; future disabling effects
need an authoritative eligibility rule. Ranged basic attacks and full PvP are
outside this prototype slice.

Headless checks include actual World/Avian steps at 20/30/60 Hz, repeated engage,
stop/clear/removal, weapon and revision changes, cast/GCD separation, facing and
range pause/resume, a real occluding collider, withdrawn/regranted readiness,
retaliation-driven owner death/respawn with stale-request rejection, and fatal
patrol reward/XP facts. Fixture relocation/occluder changes are trusted test
setup, not client-authorized world changes. These tests do not establish network
or rendered autoattack. **153 core tests, 458 network passes / ten explicit
skips / 68 groups and 113 client tests** pass, with formatting and workspace/
all-target denied-warning Clippy. Thirteen new authority checks cover this slice.
Proof: `build/classic-autoattack-results-v3.json` and matching logs. Exact source
and proof: `build/classic-autoattack-authority-qualified-v1-20261101`.
The only client change explicitly ignores the unadopted event variant; existing
rendering behavior is unchanged. No new native/GPU or multiplayer-autoattack
claim is made.

Next: explicit secure transport and profile binding, then normal client
right-click/start-stop presentation, per-ability facing/swing policies, and the
one-hostile native acceptance run. Dodge remains deferred. All changes stay
local and unpushed.
