# Server-owned targeting prerequisite

Current continuation, 2026-10-28: [native targeting](TARGETING_RUNTIME.md) and the
optional [authoritative ability loop](COMBAT_AUTHORITY.md) now qualify. The
dated server-only prerequisite below is preserved as its original scope.

P1-04/08 and P2-11 continuation after the qualified ability-data prerequisite,
2026-10-26. This bounded slice adds server-owned static target identities and
selection to the real headless movement World. It does not claim a transported
NPC, rendered target, AI, damage or a playable combat loop.

Development targets are placed on already-prepared graybox collision before
admission. At most 16 exist; they share the world's monotonic entity allocator
and incarnation. They add no physics body: targeting never creates actor collision.
The server validates the caller's admitted actor, target incarnation/existence and
existing 1,024 m XZ relevance bounds. Selection can be cleared, disappears on
removal or leaving relevance, and cannot transfer to a replacement session.
Spatial relevance is not line of sight or spell range.

Acceptance: actual World join/input/fixed-step traversal through a target; stable
unique identities, invalid/stale actor and target rejection, bounded candidates,
selection/removal/reconnect and distant teleport pruning. No additional terrain
worker demand is introduced for these stationary targets. Run focused actual-world
tests, fmt and denied-warning workspace/all-target Clippy before qualification.
Protocol/UI and authoritative cast execution follow separate integration gates.
The four focused tests pass (`build/targeting-world-tests-v1.log`), followed by
workspace/all-target denied-warning Clippy and formatting. They exercise the
actual World, prepared collision, admitted actor IDs, validated input and fixed
steps; they are not network connections or rendered NPCs. Player and target
snapshots now share the same relevance helper without changing the existing
1,024 m policy. The combined full workspace regression passes **245 tests**, with ten explicit
private-fixture skips (`build/targeting-workspace-v1.log`), followed by
denied-warning Clippy/fmt (`build/targeting-clippy-final-v1.log`,
`build/targeting-fmt-final-v1.log`).
The normal launcher has not adopted development targets.
