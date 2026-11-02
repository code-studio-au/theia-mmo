Current opt-in executable and rendered adoption is qualified in
[player-life native integration](PLAYER_LIFE_NATIVE.md). The checkpoint below
records its own prerequisite scope and historical remaining gates.

# Player-life codec and receive barrier — qualified 2026-10-31

P3-04 / P3-07 prerequisite following [headless authority](PLAYER_LIFE_AUTHORITY.md).
This checkpoint defines and tests frames/receive state. Subsequent encrypted
publication and client receive work is recorded in [runtime integration](PLAYER_LIFE_RUNTIME.md);
normal executable/bundle selection and Bevy/native adoption remain open.

PL01 is an owner-only server observation, 86–110 bytes. It carries the qualified
20/30/60 Hz rate, exact profile hash, runtime actor/life generation, completed tick,
movement stream, current/maximum health, optional scheduled respawn and optional
incoming windup. All integer fields use little endian. The windup's target is the
owner life; a different source epoch or self-creature is rejected. Maximum health
must match the configured stats. Dead observations require a positive respawn
deadline and cannot wind up. Future deadlines are bounded by the checked recipe
and declared rate; an elapsed respawn deadline remains valid while collision loads.
Unknown flags/tags, foreign owner/profile, zero identities, invalid phase/counts,
truncation, trailing bytes and oversized frames are rejected.

The stream counter must account for each death/respawn: at least 2g−1 for alive
generation g, or 2g while dead. Extra stream resets remain valid. Receive state
ignores older observations, recognizes exact duplicates and refuses conflicting
same-tick states, same-life health increases, death without a stream reset or a
new generation without a newer stream. Healing is not part of this initial mode.
Failed validation leaves the published observation and movement barrier unchanged.

Prediction starts only after an own life observation and matching actor/world/rate
correction. Death blocks it. A changed life or movement stream waits for its
matching completed correction before resuming. Ordinary health/windup updates keep
the established barrier, avoiding a new pause for each health message. These are
pure receive rules; app input suppression and presentation still need integration.
The existing 77-byte welcome is unchanged; the checked world identity already
binds the opt-in profile, and PL01 declares its own rate before movement data arrives.

The default exact-byte profile reference is
`099a307ba8f90d8e75677ba21d2fe6a6c6c5597beb2f04fcd87e11df72acace5`,
independently calculated from the length-delimited four source profiles. The
headless authority's real completed health/death/respawn observations round-trip
through the codec and unlock only their matching movement corrections.

Eight codec/receive tests, all **100 core / 76 network-library tests**, formatting
and denied-warning workspace/all-target Clippy pass with unchanged final source.
The network library count includes fourteen new life/authority tests and retained
legacy encrypted regressions. It does not establish new-mode transport privacy.
Exact source/docs/logs: `build/player-life-wire-qualified-v1-20261031`. No new
full-workspace/native result is claimed; prior playable binaries remain preserved
at `build/character-menu-qualified-v1-20261031`.

Next is actual owner-routed reliable Renet publication, backpressure/forged-state
refusal, matching client state/bridge handling and two encrypted clients. Then
complete standard Bevy windup/death/respawn UI, movement gating, opt-in launching
and actual native/default-mode regression. Durable vitals, production balancing,
physical input and native Windows remain open.
