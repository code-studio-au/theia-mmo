# Player sword melee — 2026-10-31

P2-12 / P3-06 / P3-08 / initial P3-14. The opt-in melee bundle adds a real,
server-validated sword action to the qualified pursuit/life/equipment foundation.
Matched source, binaries, art and proof are preserved in
`build/melee-qualified-v1-20261031`; earlier checkpoints remain available.
Changes remain local.

## Authority and data

Ability schema 4 adds an explicit required main-hand item ID and an optional
presentation cue. Schemas 1–3 and their content files remain intact. Cross-profile
startup validation rejects missing or unequippable weapon definitions. Execution
reads only the character-bound inventory owner's committed bag; pending equip
commands and cosmetic appearance cannot grant attacks. Inventory loading, failure
or closure withdraws readiness. Exact session identity, profile hash and monotonic
bag revisions prevent stale equipment reuse. Observation occurs before cast
handling and fixed simulation; no awaited SQL or transport work enters a tick.

`development-melee-abilities.json` retains Scorch on action 1 and adds Sword Slash
on action 2: **3 m, 500 ms windup, 1.5 s cooldown/GCD, zero mana, mobile**, requiring
committed `theia.dev.training-sword`. The thirteen-role manifest is
`development-gameplay-player-melee.json`. Ranges use authoritative 3D positions.
Equipment/life/readiness are checked at start and throughout windup; completion
and queue execution recheck target life, finite range and static LOS. Cancelling
a started swing retains its cooldown. No weapon-mesh hit detection or physical
player/NPC collision is introduced.

Two standard Bevy widgets expose the first two catalogue entries with keys 1/2,
independent server cooldown views and shared cast/GCD state. The original single
button and first-slot qualification remain supported. This is a development bar,
not the requested finished HUD.

## Standard animation and terrain readiness

Authoritative start/cancel events drive a bounded eight-actor cosmetic cue cache.
The optional 0.8 s SwordAttack uses standard Bevy masks: the authored strike owns
Spine_01 and descendants, including wrist, prop socket and digits; locomotion owns
hips, legs and root. Standing, walking and running use the same server ability.
Death/unequip override the layer, and owner death/fresh-life changes invalidate
old cues while retaining sequence tombstones. Animation never schedules damage.
Public remote-player health/death remains outside the current protocol.

Native testing exposed a real dependency bug: greybox movement/combat readiness
previously depended on whole-renderer screenshot readiness. An unrelated pending
pipeline could withdraw all ready terrain chunks and safely cancel a swing as
NotReady. The greybox now observes its own actual standard PBR specializations and
GPU mesh residency. Startup/pending terrain replacements still gate readiness;
unrelated UI, screenshot or skinned-character pipelines do not. Both terrain
LODs share one material and a tested vertex-layout contract. Offscreen chunks
must be resident on the GPU, but need not be drawn before granting readiness.
Whole-renderer screenshot readiness remains separate.

## Exact current evidence

`build/melee-full-exit-v4.json`: **571 workspace/all-target passes, 40 explicit
fixture skips, 113 groups**, formatting and denied-warning workspace/all-target
Clippy. Test binaries use CARGO_PROFILE_TEST_DEBUG=0; runtime debug assertions
remain enabled. `build/melee-python-v4.log`: **41 tests pass**. Current matched
source/binary candidate: `build/melee-app-candidate-v4-20261031`, 585 source files.

The server's 20/30/60 Hz tests cover committed unequip and inventory loss during
windup, queued gear revalidation, completion range, zero-mana successful damage
and foreign/retired session observations. Actual encrypted two-peer Renet gameplay
also passes pending-equip refusal, independent sword/wand ownership, committed
unequip cancellation and a subsequent valid hit observed by both peers. Its item
owner is deliberately synthetic; this is distinct from the real SQL test below.

`build/melee-oidc-native-v4-20261031` passes actual local Keycloak OIDC accounts,
PostgreSQL and two native Linux clients at 1920×1200. Ordinary widget/key inputs
exercise standing/walking/running hits (100→80→60→40), committed unequip during
windup, wand refusal and out-of-range refusal. Exactly one cancellation is the
expected EquipmentRequired; unrelated pipeline compilation occurs without
withdrawing an established terrain footprint. Owned services and private tickets
are cleaned. Both clients have **zero GPU diagnostics** with Vulkan validation. Qualification
injects Bevy pointer/key events through normal widgets; physical OS input/browser
delivery is not established.

`attack-verification.json` compares 53 actual local transforms in each sampled
pose to original GLB curves, independently for local and remote Idle/Walk/Run.
Maximum upper/lower local matrix error is **3.95e-6**; 32–33 upper-body targets
change from the underlying gait. This verifies authored strike layering and
preserved locomotion, not physical weapon contact. The first two failed native
tours remain diagnostic evidence; v3 also passes before final lifecycle hardening.

Hardware: Linux, RX 7900 XTX / RADV NAVI31 / Mesa 26.2.6 Vulkan, Ryzen 7800X3D.
Native Windows and MMO population/streaming capacity are not established here.
The earlier 30 standalone PostgreSQL contracts apply to unchanged persistence
source; actual native SQL flows are rerun separately.

The final unchanged source also passes focused equipment/moving-grip and rewarded
life/rejoin/restart/backend-loss regression at 1920×1200, the legacy four-clip rig
at 1280×720 and all three normal launcher cleanup modes with the new melee bundle
at captured 3440×1440 fullscreen. All 25 final native starts have zero GPU
diagnostics. Actual moving grip still matches the authored fingers and original
body/wrist/socket curves. Evidence: `build/melee-regressions-exit-v4.json` and the
matched checkpoint. The separate [quiet one-client baseline](PLAYER_MELEE_PERFORMANCE.md)
records 711.50/712.63 FPS at 1920×1200/3440×1440; it is not capacity evidence.

## Launch the qualified development bundle

```sh
python3 tools/play.py --account-login \
  --binaries build/melee-qualified-v1-20261031 \
  --gameplay-validator build/melee-qualified-v1-20261031/validate_gameplay \
  --gameplay-manifest content/gameplay/development-gameplay-player-melee.json \
  --players build/melee-qualified-v1-20261031/art/player \
  --creature-asset build/melee-qualified-v1-20261031/art/wolf/wolf.glb \
  --window-size 1920x1200
```

This uses the existing configured local OIDC/PostgreSQL launch environment and
prepared terrain/navigation. For ultrawide, use `--window-size 3440x1440 --fullscreen`.
Equip the training sword in the backpack, select the wolf with Tab and use 2 or
the Sword Slash widget within 3 m. Scorch remains on 1. Explicit --players selects
the combat rig; the default canonical rig and earlier experiments remain intact.

## Limits and next slices

Initial exact-item requirements are not a production weapon-class/stat system.
The development sword/wand meshes remain primitives. The combat rig and full
weight master are unchanged; its four-weight preview still fails the existing
5 mm production skin-fidelity gate. Broader animation blending, public NPC attack
timing and remote-player death remain open. New hit/miss/deflect probabilities,
ranged enemies, travelling magic/arrows, quest markers, minimap and HUD assets
remain in [the requested follow-on register](COMBAT_WORLD_FOLLOWUPS.md).
