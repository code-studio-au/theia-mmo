# Hit, miss and deflect — qualified, 2026-10-31

P3-06 / initial P4-08 adds server-owned outcomes to player direct-damage abilities,
following committed equipment-gated melee. P1-11 catalogue identity, authoritative
range/LOS/equipment/life validation and bounded reliable delivery are prerequisites.

Matched source, binaries, art and proof: `build/outcomes-qualified-v1-20261031`.
The outcome checkpoint has **589 workspace passes / 40 explicit fixture skips in
114 groups**, denied-warning workspace/all-target Clippy, formatting and **41
Python tests**. Actual local OIDC/PostgreSQL, two native clients, source-curve
poses, prior melee/equipment/life/legacy regressions and normal launcher cleanup
pass. All **31 final native starts record zero GPU diagnostics**. Later isolated
projectile work is documented separately in [PROJECTILE_TRAVEL.md](PROJECTILE_TRAVEL.md);
it is not part of this checkpoint's 593-source-file workspace result.

## Authority and data

Schema 5 accepts immutable checked hit tables only for direct damage. Explicit
integer hit/miss/deflect weights total 10,000 basis points. Miss occupies the first
bucket, deflect the next, and hit the remainder of a checked roll in [0, 10,000).
Creature schema 2 declares current deflect eligibility; old schema-1 creatures
remain ineligible. An ineligible deflect bucket becomes hit without changing miss
probability or consuming another roll. Model appearance implies no defence stat.

An owned ChaCha20 stream receives an OS seed once before admission, outside the
fixed step. Only an otherwise valid completion consumes an unbiased logical draw.
Rejected commands, cancelled windups and duplicate sequences consume none. Legacy
abilities allocate no stream and retain their guaranteed direct-damage behavior.
Test-only seeds permit deterministic replay; live commands expose no roll or seed.

Only Hit enters the existing damage/reward path. Miss and Deflect produce distinct
resolved events with no health, status, reward or experience change. Started costs,
GCD and cooldowns remain spent. Equipment, range, LOS, readiness and life failures
remain rejection/cancellation reasons. Metrics distinguish resolved outcomes and
logical random draws from cosmetic counters.

The opt-in `development-gameplay-hit-outcomes.json` keeps Scorch and sets Sword
Slash to nominal **85% hit / 10% miss / 5% deflect**, with an explicitly eligible
development creature. These are prototype weights. Only player direct damage uses
tables; NPC attacks, status/healing outcomes, shield/class rules and final balance
remain open. No Bevy/Avian pins change. rand =0.9.5 (default features off) and
rand_chacha =0.9.0 were already locked; all 807 package versions, sources and
checksums remain unchanged. Only dependency edges were added.

## Wire and presentation

Schema-5 catalogue-bound CE03 frames add avoidance events; schemas 1–4 preserve
CE02 bytes. A 16-avoidance batch is 845 bytes within the 1,200-byte budget. Unknown
kinds, impossible table outcomes, foreign keys, malformed lengths, invalid global
positions and extra payloads fail closed. The fixed 29-byte attack request has no
roll/seed extension. Existing bounded reliable delivery/backpressure remains.

Standard Bevy UI queues at most 32 pending and 32 active one-second cosmetic
labels: yellow “Miss” and cyan “Deflected”. Labels project checked f64 globals
through the central render frame, hide outside the viewport, drift/fade, ignore
picking and never change gameplay results. Cosmetic overflow cannot drop the
underlying authoritative combat event. The initial layout uses a simple fixed
horizontal offset; general stacking, occlusion and HUD styling remain follow-ons.

## Evidence

Pure policy: `build/hit-table-qualified-v1-20261031` preserves five exhaustive and
replay tests, including all 10,000 rolls, invalid u16 rolls and malformed tables.
Focused integration also covers actual encrypted two-peer avoidance after a
synthetic inventory-owner commit. That focused test is not OIDC/SQL evidence.

Final receipts are `build/outcomes-final-exit-v1.json`,
`build/outcomes-native-exit-v3.json` and `build/outcomes-regressions-exit-v1.json`.
The final full tests run without debug symbols but retain runtime debug assertions.
Forty persistence/asset fixture tests are explicitly skipped; this slice does not
claim a fresh standalone PostgreSQL contract-suite run.

Each final hit/miss/deflect native case uses actual owned Keycloak accounts,
PostgreSQL, secure Renet and two Linux Vulkan clients at 1920×1200. Certain weights
are server-side fixtures, not client-selected rolls. Each records three resolutions
and three draws, with exactly three of the chosen outcome. Hit reduces enemy
health 100→80→60→40; miss and deflect leave it at 100 on both peers. Unequip during
windup, wrong equipment and range refusal retain their authoritative checks.

Both peers retain source-matched standing/walking/running attack poses. Captures
show actual floating Miss/Deflected glyphs. Capture waits for nonempty glyph layout,
positive computed bounds and inherited visibility across three frames. The first
native matrix captured before visibility propagated; those diagnostic artifacts
remain preserved but are not the final presentation proof. A verifier asset-path
typo was also corrected without changing candidate binaries or art.

Compatibility checks include the old schema-4 melee/CE02 path, moving sword grip
and equipment at 1920×1200, rewarded player life/respawn/rejoin/restart/backend loss,
and canonical older gameplay at 1280×720. Normal new-bundle launcher cleanup passes
all three modes, with native captures at actual 3440×1440 fullscreen and private
tickets/snapshots removed. Owned provider/database processes stop after testing.

Native device: Linux Ryzen 7800X3D, RX 7900 XTX, RADV NAVI31 / Mesa 26.2.6 Vulkan.
There is no native Windows or fresh standalone FPS result for this checkpoint.
The [earlier melee frame measurements](PLAYER_MELEE_PERFORMANCE.md) belong to their
saved binaries. Validation-run screenshot FPS is not a performance baseline.
One enemy and at most eight development players do not establish MMO capacity.

## Launch

With the existing local OIDC/PostgreSQL setup and prepared development terrain:

```sh
python3 tools/play.py --account-login \
  --binaries build/outcomes-qualified-v1-20261031 \
  --gameplay-validator build/outcomes-qualified-v1-20261031/validate_gameplay \
  --gameplay-manifest content/gameplay/development-gameplay-hit-outcomes.json \
  --players build/outcomes-qualified-v1-20261031/art/player \
  --creature-asset build/outcomes-qualified-v1-20261031/art/wolf/wolf.glb \
  --window-size 1920x1200
```

Use `--window-size 3440x1440 --fullscreen` for the qualified ultrawide layout.
The qualified melee and older checkpoints remain preserved. Source art is local
and proprietary; the four-weight preview still fails its separate production
5 mm skin-fidelity gate. Primitive development weapon meshes, public remote-player
death/NPC attack timing and animation blending remain open. All changes are local;
no commit or push is implied by qualification.
