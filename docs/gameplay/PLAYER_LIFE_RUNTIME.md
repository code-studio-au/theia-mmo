Current opt-in executable and rendered adoption is qualified in
[player-life native integration](PLAYER_LIFE_NATIVE.md). The checkpoint below
records its own prerequisite scope and historical remaining gates.

# Player-life encrypted runtime — 2026-10-31

P3-04 / P3-07 follow [headless authority](PLAYER_LIFE_AUTHORITY.md) and the
[bounded codec](PLAYER_LIFE_PROTOCOL.md). Normal executable/bundle selection and
Bevy presentation remain subsequent gates. The preserved playable checkpoint is
`build/character-menu-qualified-v1-20261031`.

The Renet owner publishes each changed life observation on the existing reliable
ordered critical event channel immediately after the completed simulation step.
It sends only that connection's actor, with no tick-only heartbeat. It records a
frame as delivered to the transport only after capacity succeeds. Exhausted
server capacity retires the affected connection instead of dropping a transition.
Client-originated PL01 frames are invalid controls and disconnect their sender;
there is no client command to set health.

The receiving worker checks the welcome actor/world, exact-byte life profile and
movement tick rate before publishing immutable life state through the existing
watch bridge. A bounded 16-event presentation queue preserves windup, damage,
death and respawn transitions across watch coalescing. A full or closed queue
fails the connection before committing the new observation. This queue must be
drained by the eventual Bevy integration; it is deliberately not an unbounded log.
The movement gate is available in the received life state, but the rendered
prediction driver has not yet adopted it.

Five focused tests cover actual encrypted two-peer UDP at 20/30/60 Hz under
20 ms delay, 30 ms jitter, every-eleventh packet loss and every-seventeenth packet
duplication; private incoming windup/health, death and matching respawn corrections;
stalled client presentation with the other peer still playing; stale life-stream
movement followed by accepted current-stream input; forged other-player health
with sender disconnection; and incompatible profile rejection before watch/event
publication. One of the five is specifically a server reliable-queue unit test,
not an additional multiplayer execution. The fixture slows the non-chasing patrol
to 0.1 m/s so it remains in melee reach; production content is unchanged.

These executions use authenticated/encrypted development tickets and real
headless World/Avian collision. They do not qualify OIDC, PostgreSQL persistence,
rendered controls, native GPU presentation, production combat logout semantics or
MMO capacity for this new mode. Prior account, durable quest/economy and native
checkpoints retain their recorded scopes. Health remains explicitly transient.

## Qualified source and checks

`build/player-life-runtime-qualified-v1-20261031` freezes 560 source files with
hashes, full workspace logs and non-story documentation. The unchanged-source
workspace/all-target run passes **519 tests / 40 explicit fixture skips in 111
groups**; denied-warning workspace/all-target Clippy and formatting pass. The
network library has 81 passing tests, including the five new runtime checks;
eight separate life-wire tests and 100 core tests also pass within that run.
Server, persistence, content and Python helper source are unchanged from the
prior codec checkpoint; the previous 30 actual SQL contracts are retained evidence,
not newly executed SQL contracts. No new native binaries are qualified here.

## Next playable acceptance

Keep P3-04/P3-07 opt-in until a matched client/server/content bundle passes.
The client must wait for its initial life observation, suppress prediction and
new gameplay actions while dead, continue applying authoritative corrections,
and discard old command/clock debt. After respawn, require the new movement
stream and completed life barrier before sampling fresh input. Ordinary incoming
windup or damage updates must not reset an already valid movement clock.

Use the existing standard Bevy health display with authoritative PL01 health;
legacy combat vitals must not overwrite it with stale maximum health. Display
incoming attack and respawn deadlines from server ticks, with readable state
text. Drain the bounded life queue even under reduced cosmetic quality. New
opt-in profile files must bind both executable world identities and the validated
launcher manifest; legacy bundles retain their exact identities.

Native acceptance requires two actual Bevy peers: one takes a visible windup,
dies, cannot move/cast while dead, then respawns at supported streamed home and
moves on its new stream. The passive peer retains independent health/control.
Recheck quest rewards and account rejoin/restart with the transient life policy
explicit. Qualify 1280×720 and 1920×1200 windows plus 3440×1440 fullscreen on the
Linux GPU, record measured frame/layout behavior and known surface diagnostics,
and retain the existing account launcher cleanup checks. Windows execution and
persistent health remain separate acceptance work.
