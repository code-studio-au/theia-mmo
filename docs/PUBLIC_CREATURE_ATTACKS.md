# Public creature attack timing — 3 November 2026

P3-07 / P3-14 / P4-08. Prerequisites are the existing authoritative retaliation,
exact NPC/player lifetimes, pursuit/return, damage threat and standard wolf
presentation. The latest explicit normal playable is
`build/creature-attack-client-qualified-v1-20261103`. Public transport and schema-14
normal CLI/bundle/Bevy adoption are qualified. Earlier bundles still use their
original owner-private windups.

## Current observation and wire prerequisite

`World::enable_public_creature_attacks` explicitly opts into a pre-admission
Classic World identity wrapper after damage threat. It requires ready pursuit/life
and refuses duplicate or post-admission activation without changing identity.
The immutable public profile binds the existing exact player-life/retaliation
profile plus a public-contract domain. Existing World modes retain `None`.

A read-only World query exposes only profile hash, exact NPC entity key, completed
tick and an optional public target entity key with windup start/contact ticks.
It checks the current living connected exact player life internally, but never
publishes that private life generation, health, damage, hit outcome or reward
state. Return, cancellation, unavailable victims and NPC death expose no pending
attack. NPC respawn has a fresh entity key. Authority failures refuse a stale
observation. No attack, damage, facing, camera, control or target decision changes.

`NA01` is a strict fixed-size public timing frame:

| Data | Bytes |
| --- | --- |
| Tag, supported tick rate, exact public profile hash | 37 |
| NPC epoch/entity, tick, pending flag | 25 |
| Optional target entity and start/contact ticks | 24 |
| Total | 62 idle; 86 pending |

The target shares the source's checked nonzero World epoch. Source/target IDs
must be nonzero and different; flags, exact length/profile/rate and timing window
are validated before state changes. Pending duration equals the existing profile's
ceiling conversion at 20/30/60 Hz. This presents the adopted Theia retaliation
windup/recovery policy; it introduces no new Classic swing formula or imperial
units. Profile sources and timing uncertainty remain in the existing combat docs.

Receive state retains one observation for this single development NPC. Foreign
profiles/epochs/rates, malformed frames and contradictory same-tick updates
refuse atomically. Duplicates and older ticks cannot replay a cancelled attack or
restore an older NPC after a fresh-life observation. Presentation can query the
current authority tick to suppress an expired late-arriving windup without
predicting a hit or changing the player's explicit target/manual facing.

Ten focused checks pass: six wire/context/clock cases and four actual World
activation/owner-vs-observer/transfer/death/respawn/error cases, including
20/30/60 Hz. All 650 network tests pass (ten service-dependent skips), along with
formatting and all-target denied-warning Clippy. Preserve
`build/creature-attack-codec-qualified-v1-20261103`. No actual public replication,
normal CLI/manifest, renderer or native adoption is claimed.

## Implemented optional reliable transport

`GameplayClient.public_attacks` explicitly selects the three-channel configuration.
Inactive callers retain the original two channels. Channel 2 has 8 KiB of reliable
ordered queue memory and precedes event channel 1 and pose channel 0 in Renet
packet-generation priority; the existing 2,400-byte per-tick budget is unchanged.
An accepted chunk-readiness report gates the first full public observation, so
the priority channel cannot overtake the client's validated Welcome. Subsequent
observations suppress tick-only changes. No simulation-step transport await or
new gameplay command is introduced.

Both transport queue saturation and the bounded 16-event presentation queue fail
explicitly. A full server channel retires the peer through the normal authority
cleanup; a full/closed presentation queue disconnects without advancing receive
state. Uploads on the public channel, and NA01 uploads on the event channel,
disconnect the sender. Diagnostics count sent/received frames, refusals, queue
high water and frame sizes. The client watch state preserves public timing across
pose updates and checks its tick rate against movement corrections/snapshots.

The focused actual encrypted victim/observer scenario passes at 20/30/60 Hz under
20 ms delay, 30 ms jitter, configured loss/duplication and observed reordering.
Both clients see the same attack/cancel/transfer sequence; private observer
health and incoming strike ownership remain intact. Four additional boundary
cases cover legacy/priority configuration, readiness, full transport/presentation
queues, invalid rate/queue context and encrypted forged uploads on both channels.
All 655 network tests pass (ten service-dependent skips), formatting and
workspace/all-target denied-warning Clippy pass. Preserve
`build/creature-attack-runtime-qualified-v1-20261103`.
Admission/inventory use synthetic committed ports; no SQL/OIDC, native rendering,
Windows execution or population-capacity evidence is claimed by these checks.

## Normal presentation acceptance (now qualified)

Renet 2.0.0 remains pinned in Cargo.lock. Its `ConnectionConfig` and
`remote_connection.rs` document/use channel-vector send priority; no dependency
change was needed. Normal CLI/bundle identity and standard Bevy presentation are now qualified
below; preserve the earlier transport checkpoint as component evidence.

Use standard Bevy wolf clips and existing
bounded global/local pose conversion for bystander animation. Late/expired
messages must not restart an old attack. No automatic player facing/targeting.
Version normal bundle/CLI identity explicitly and run owned native observer/victim
checks before changing the normal playable. Multi-NPC/region routing, population
capacity, public remote-player death and broader combat cosmetics remain later.

## Qualified normal adoption

`content/development-gameplay-classic-public-attacks.json` uses schema 14 and the
same 22 exact gameplay inputs as schema 13. The explicit manifest version selects
`--public-creature-attacks` for both normal binaries and binds the derived public
profile into the same World identity order. Earlier bundles keep their original
behavior. No new content-balance rules, Classic formula, controls or saved-quest
migration are introduced.

Standard Bevy wolf clips now prefer public timing when available, including idle
cancellation; they never fall back to an older private windup in that mode.
Presentation suppresses attacks expired at the latest known authority tick.
Wolf facing reconstructs the actual victim's checked global position from owner
correction or remote snapshots. An unavailable remote victim never substitutes
the local observer. The player's manual facing and selected target are unchanged.
The bounded public event queue is drained each presentation update and counted
separately from owner-private incoming windups; wolf diagnostics distinguish
public attack frames and remote-victim frames.

All 129 client, 46 actual gameplay-validator and 34 Python launcher checks pass,
as do formatting and workspace/all-target denied-warning Clippy. The new
presentation check covers remote-victim positions, missing targets, source
identity and expired/idle timing at 20/30/60 Hz. Optimized normal binaries and an owned two-account OIDC/PostgreSQL/native
check pass at 1920x1200: the unselected observer stays at 100 HP with zero private
incoming windups, receives public timing and renders attacks aimed at the remote
victim 7.53 m away. The victim takes actual damage. Screenshot and frame/stream
proof are in `build/public-attack-native-v2-20261103`; the first attempt stopped
at the PID/pointer ownership guard during an unsettled window focus switch, then
the harness waited for focus without weakening the guard.

The same unmodified schema-14 bundle passes normal two-quest/three-kill completion
and a PostgreSQL-process restart/rejoin: head 13, 125 XP, no duplicate rewards,
no player death. Fresh threat accounting is 23 facts / 300 actual HP lost / three
clears; rejoin adds none. Proof is
`build/public-attack-journal-native-v1-20261103`. All three successful native runs
have zero GPU diagnostics on Linux AMD RX 7900 XTX, RADV Vulkan, Mesa 26.2.6;
1920x1200 native windows on the connected 3440x1440 display. Input is synthetic
PID-bound X11, not physical hardware. Owned services/password files are cleaned.
No Windows or population-capacity claim. Final full workspace regression passes all 1,106 tests (50 explicit skips).
Formatting and workspace/all-target denied-warning Clippy pass. Preserve
`build/creature-attack-client-qualified-v1-20261103`: exact source and binary
fingerprints, optimized normal binaries, existing art, network/refusal logs,
validator/launcher checks and whitelisted native proof. No provider/database or
credential directories are copied into that checkpoint. All work stays local.

With normal account/PostgreSQL configuration and prepared player/wolf assets,
select this version explicitly through the existing launcher:

```sh
python3 tools/play.py --classic --account-login \
  --gameplay-manifest content/development-gameplay-classic-public-attacks.json \
  --binaries build/creature-attack-client-qualified-v1-20261103 \
  --players build/creature-attack-client-qualified-v1-20261103/art/player \
  --creature-asset build/creature-attack-client-qualified-v1-20261103/art/wolf/wolf.glb
```

Next bounded presentation gap: public remote-player death/fresh-life visuals.
That is not implemented by NA01. Preserve owner-private vitals and exact server
life authority when adding a separate public visual lifecycle contract. Broader
NPC counts/region routing and population qualification remain separate work.
