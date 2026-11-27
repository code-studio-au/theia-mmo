# Creature return reset — 2 November 2026

P3-07 / P3-06 / P4-08. Prerequisites: authoritative pursuit/retaliation and typed
ordinary/ability damage, exact NPC/player lifetimes, membership, contribution/tag
reset generations and a bounded status registry. Preserve the qualified
schema-10 solo-credit playable checkpoint until replacement acceptance passes.

## Current boundary and selected fixture

Ordinary/typed combat already rejected damage in Return/Blocked before this
slice. Legacy generic combat can remain damageable while returning. Credit mode
already clears membership/tag/contributions and advances an encounter generation;
old casts/projectiles cannot retag the resolved encounter. The missing work is
health restoration and effect cleanup. This corrects earlier broad wording that
all older profiles remained damageable.

`development-creature-return-reset.json` declares schema 1,
`creature_return_reset_fixture_v1` and the exact creature definition. The explicit
policy restores the living creature's maximum health and removes effects aimed
at that NPC on entry to Return or Blocked. These are Theia implementation rules,
not a claim of Classic's precise timing/regen algorithm. No distance, speed,
leash or other metric value changes; existing Y-up metres and fixed server ticks
remain. Threat and social aggro are separate work, not inferred from a reset.

## Opt-in World qualification

`enable_creature_reset` requires pre-admission classic typed combat, membership,
credit and the matching single patrol creature. Strict 4 KiB parsing rejects
unknown fields, unsupported versions/rulesets and invalid IDs. Exact validated
bytes wrap World identity after credit and before economy wrappers. Normal adoption is now separately qualified below; this section records the
preceding opt-in World boundary.

Each step synchronizes reset after actual pursuit and credit reset, before casts,
projectile impacts and due status effects advance. Entry restores health once
and clears target effects. It does not change NPC entity identity, position,
route, death/respawn deadline, player health or cooldowns, and emits no kill or
respawn fact. Existing retaliation cancellation, configured reacquisition guard,
path return and saved patrol recovery retain their authority.

A dead creature is never resurrected by this policy. Fresh respawn remains the
existing fresh-key lifecycle. A confirmed reset keeps old in-flight encounter
contexts invalid after patrol resumes, while a fresh eligible hit can establish
a new claim. Reset failures latch an authority error before partial healing;
outgoing/incoming damage and private life publication fail closed. Generic
periodic damage is also suppressed through return/error for the opted-in mode.
No awaited I/O, database operation, path search or new transport channel occurs
inside the simulation step. Bounded reset/heal/effect-removal metrics are exposed
and printed in the normal server shutdown summary.

Seven focused checked-terrain/Avian tests pass. Multi-rate reset/cleanup cases use
20/30/60 Hz. They cover exact policy identity/admission, ordinary/typed legacy
behavior, blocked-route reset, one-time health/effect cleanup, membership/windup
and tag release, stale contexts after resumed patrol, fresh post-reset attacks,
death/fresh respawn and invalid-health refusal. The current typed catalogue has
no legitimate DoT ability; effect cleanup is tested through an explicitly
injected trusted existing Registry entry, not claimed as playable DoT support.
All 259 core / 638 network (ten skips) / 123 client tests, formatting and
workspace/all-target denied-warning Clippy pass. Exact source/proof:
`build/creature-reset-world-qualified-v1-20261102`. Existing UDP regression uses
older profiles; the new reset mode needs its own encrypted observation checks.

Next acceptance: actual encrypted peer observations of restored health/released
combat without false deaths/awards; then a versioned normal bundle and native
leash/return/reacquisition checks. No new-profile native, Windows, population or
MMO-capacity claim is made by this headless candidate.

## Encrypted movement/observation qualification

One actual encrypted two-peer test passes at 20/30/60 Hz with 20 ms delay, 30 ms
jitter, periodic loss/duplication and observed reordering. The fighting player's
ordinary prediction driver consumes real movement intent and moves away until
authoritative pursuit returns; no test hook sets Return or HP in this run.
Both clients observe the same existing NPC restored to maximum health and
released combat. The owner retains the same player life/movement stream; the
observer remains undamaged. Reliable public captures contain no death or
respawn, and neither XP nor reward work is emitted.

The fixture deliberately starts the NPC at 20 current HP with its original
1000 maximum and uses zero-avoidance specials, synthetic admitted inventory and
1 m perception. This is a transport/reset observation check, not normal balance,
OIDC, persistence, native navigation or MMO capacity evidence. It demonstrates
movement-triggered return; it does not assert which particular path/leash cause
wins or claim an exact Classic leash distance. Existing static path/return
rules remain. Effect cleanup has the separate injected-Registry headless proof.
All 259 core / 639 network (ten skips) / 123 client tests, formatting and
workspace/all-target denied-warning Clippy pass. Exact source and proof are
preserved in `build/creature-reset-transport-qualified-v1-20261102` (743 source
files). Normal manifest and native adoption remain the next acceptance slice.


## Normal schema-11 adoption qualified

`development-gameplay-classic-reset.json` adds `creature_reset` to the twenty-one
schema-10 roles. The offline validator requires all twenty-two distinct files,
a supported bounded policy and the matching pursuit creature. Missing/null
context, older schemas with the role, aliased files, oversized profiles and
unsupported fields/rulesets refuse before launch. The launcher retains exact
validated snapshots until child shutdown and forwards the policy to both peers.
`--creature-reset` requires explicit solo credit; the client and World bind the
same exact-byte policy after credit and before economy identity wrappers.
Existing schema-10 and older bundles remain available. PL02 / CE08 are unchanged;
normal health replication and the existing combat indicator display reset state.

All 259 core / 641 network (ten skips) / 123 client / 49 Python tests,
formatting, workspace/all-target denied-warning Clippy and four release builds
pass; the network total includes 39 offline validator checks. Seven normal
OIDC/PostgreSQL native runs pass: 35-second passive melee at 1920x1200, controls/
casting at 1920x1200, 3440x1440 fullscreen and 1280x720, and return/reacquire/kill
at all three sizes. Native production data is unmodified. Each return run uses
only ordinary X11 input: 89 current HP restores to 100 on the same selected NPC,
combat clears, the player walks back, the NPC reacquires and a later typed special
kills it. There is exactly one reset, two acquisitions, two combat entries/exits,
one durable XP grant and one training offer. No death/respawn occurs before
reacquisition; no player life changes. Passive melee has no return or reset.

All seven runs have zero GPU diagnostics and combat-event gaps. Linux AMD Radeon
RX 7900 XTX / RADV Vulkan / Mesa 26.2.6, connected display. Health/peace and final
kill captures were inspected for readability. Preserve exact source, binaries,
art and proof in `build/creature-reset-client-qualified-v1-20261102` (744 source
files); the previous solo-credit playable remains available. Native runs do not
exercise a DoT, loot consumption, parties, Windows or MMO capacity. The separately
qualified World test covers injected status cleanup. Bounded threat/target
transfer is the next encounter prerequisite before population expansion.
