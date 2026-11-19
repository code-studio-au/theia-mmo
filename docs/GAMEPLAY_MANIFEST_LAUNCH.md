Schema-15 public avatar-life bundle qualified, 3 November 2026:
`content/development-gameplay-classic-public-life.json` retains all 22 exact
schema-14 inputs, explicitly selecting both public NPC timing and avatar life
contracts. Normal client/server identities agree after NPC timing; earlier
schemas keep their meanings. 131 client / 47 validator / 35 Python checks,
1,123 workspace tests (50 skips), formatting and Clippy pass. Actual native
remote death/fresh life and normal quest completion/database restart pass.
[Exact evidence, privacy and limits](PUBLIC_AVATAR_LIFE.md). Preserve
`build/avatar-life-client-qualified-v1-20261103`. Local only.

Schema-14 public NPC attack bundle native checks pass, 3 November 2026:
`content/development-gameplay-classic-public-attacks.json` explicitly selects
`--public-creature-attacks` on both normal binaries with the same 22 exact inputs
as schema 13. World identity adds the derived public contract after threat;
older manifests keep their behavior. Two owned OIDC accounts/native clients
confirm victim-only incoming strikes and a healthy unselected observer rendering
attacks aimed at the actual remote victim (positions 7.53 m apart). Normal
unmodified-wolf quest completion and PostgreSQL restart/rejoin also pass: head
13, two completed quests, 125 XP and no duplicate progress/rewards. Native
1920x1200 on Linux RX 7900 XTX / RADV Vulkan; all three successful runs have zero
GPU diagnostics. Final workspace regression passes 1,106 tests (50 skips); preserve
`build/creature-attack-client-qualified-v1-20261103`. No Windows/capacity claim. [Contract and current proof](PUBLIC_CREATURE_ATTACKS.md).

Schema-13 damage-threat bundle qualified, 3 November 2026:
`content/development-gameplay-classic-threat.json` adds an exact bounded threat
policy to the canonical schema-12 journal, for 22 roles. Complete context and
exclusive versions/roles are checked by Rust and the copied snapshot is
revalidated before children launch. Client/server identity includes the same
policy bytes after reset. 44 validator / 33 Python checks, full 1,074-test
workspace regression, formatting/Clippy and normal OIDC/SQL native completion/
restart pass. Older bundles stay unchanged. [Exact proof and limits](CREATURE_THREAT.md).

Schema-12 expanded journal qualified, 3 November 2026:
`content/development-gameplay-classic-journal.json` selects 21 exact-byte roles.
Its `quest` role is the bounded expanded catalogue (64 KiB development maximum)
and includes completion XP; the external legacy completion recipe is excluded.
Canonical `quests/` and `gameplay/` paths are staged together without importing
story planning data. Offline checks resolve every local quest contact, semantic
choice and creature; earlier schemas retain their interpretations. The launcher
forwards `--quest-journal` to both children. 127 client / 41 validator / 31 Python
checks and five actual normal OIDC/SQL native runs pass. [Quest behavior, proof
scope, private combat fixture and launch command](QUEST_ENGINE.md).

Schema-11 reset adoption qualified, 2 November 2026: the explicit
`development-gameplay-classic-reset.json` bundle adds the bounded `creature_reset`
role (twenty-two exact-byte profiles). It requires matching pursuit, solo credit,
membership and typed combat; older bundles remain unchanged. All 49 Python / 39 validator checks and seven normal native cases pass; [behavior and evidence](CREATURE_RETURN_RESET.md).

# Current solo-credit bundle — 2 November 2026

`content/gameplay/development-gameplay-classic-credit.json` uses schema 10 and
21 exact-byte roles, adding `creature_credit` to schema 9's membership bundle.
It requires the explicit schema-2 solo tag-owner recipient policy; ownership-only
schema 1 is rejected in normal application mode. All role/context, portable
path, alias, policy version and byte bounds are checked before launch. The
launcher seals every role and forwards the same staged `--creature-credit` to
both processes. Identity binds it after membership and before economy wrappers.

37 validator tests and 47 launcher/account/environment Python checks pass.
The normal schema-10 OIDC/PostgreSQL/native checkpoint is
`build/creature-credit-client-qualified-v1-20261102`.
[Behavior, competing-player, durability and native evidence](CREATURE_CREDIT.md).
Earlier schemas and their snapshots remain available; dated notes below describe
historical scopes, rather than the current maximum schema/role count.

# Optional completion-XP bundle — 2026-10-31

`content/gameplay/development-gameplay-turn-in-xp.json` uses manifest schema 2
with eleven required profiles, adding `quest_turn_in_xp`. Schema 1 retains exactly
ten roles. The recipe is checked against the declared quest and XP profiles;
null, missing, extra, aliased and oversized inputs are rejected. Validator reports
retain envelope schema 1 and add `manifest_schema`. Comparing the same schema-1
manifest against the previous validator gives the same report and bundle digest
apart from that additive version field.

Actual normal-launcher qualification passes interrupt, child failure and pending
login cancellation using eleven-profile sealed snapshots:
`build/quest-turn-in-xp-launcher-v1-20261031/verification.json`. Both children
receive identical paths/hashes and the explicit reward option; cleanup removes
snapshots/tickets and closes the owned database. Native children retain the
existing service-environment isolation. This launcher test checks startup and
cleanup; the complete rewarded native gameplay is recorded separately in
[completion XP](QUEST_TURN_IN_XP.md). The complete source/binary/validator/proof checkpoint is
`build/quest-turn-in-xp-qualified-v1-20261031`. Earlier ten-role/journal-only evidence below remains valid.

Current continuation, 2026-10-31: `--quest-lifecycle` can be combined with this
bundle. Actual normal-launcher qualification passes all three cleanup modes with
the new client/server, matching ten-profile snapshots and service-environment
isolation: `build/quest-lifecycle-launcher-v1-20261031/verification.json`.
[Explicit quest behavior, native checks and limits](QUEST_LIFECYCLE.md).
The bundle format/default profiles are unchanged; mode selection is explicit.
Earlier checkpoints below retain their own source/binary scope.

Current continuation, 2026-10-31: service-environment filtering is implemented and
qualified by actual native requalification. The native child and offline validator now
exclude libpq password/service settings, database URLs and server account/OIDC/test
configuration. Parent/server/browser environment stays intact. Nineteen targeted
Python tests, formatting and workspace/all-target denied-warning Clippy pass.
Actual three-mode launcher requalification passes. Exact source/helper/proof:
`build/launcher-service-env-qualified-v1-20261031`. Rust source remains identical
to `build/quest-lifecycle-store-qualified-v1-20261031` (448 passes / 33 skips).
Earlier launch evidence below records the weaker URL-only check; it is preserved.

# Validated gameplay bundle launcher — P3-15 / initial P3-16

Status, 2026-10-31: launch-time adoption qualified.
The previous persistent training checkpoint remains preserved at
`build/zone-qualified-v1-20261030`. No client/server Rust or default catalogue
changes are part of this slice.

## Behavior and prerequisites

`tools/play.py --gameplay-manifest content/gameplay/development-gameplay.json`
selects all ten declared profiles: equipment-capable items, abilities, stats,
statuses, creature, dialogue, vendor, rewards, XP and kill objective. It implies
account login and those existing optional gameplay systems, using predicted saved
terrain and secure Renet. The existing local OIDC/PostgreSQL configuration, player
assets, saved terrain and baked navigation are prerequisites; see
[authenticated admission](AUTHENTICATED_ADMISSION.md). This is development launching,
not an in-game account screen or production zone authoring tool.

Build the actual schema validator with:

```sh
cargo build --locked -p theia-network --example validate_gameplay
```

Then launch with existing optimized game binaries:

```sh
python3 tools/play.py --gameplay-manifest content/gameplay/development-gameplay.json \
  --gameplay-validator target/debug/examples/validate_gameplay
```

Without `--gameplay-validator`, the launcher expects `validate_gameplay` alongside
`theia-client` and `theia-server` under `--binaries` (`.exe` on Windows).
The Rust example is built into `target/debug/examples`, not that default directory;
pass its path explicitly or install the built executable alongside the game.
Existing individual gameplay options continue to work without a manifest.

## Acceptance

- Use the existing Rust validator's actual schemas and cross-profile checks;
  Python checks the report envelope/copy bounds, not gameplay schema semantics.
- Validate the original bundle, copy each profile once with bounded reads, compare
  each exact SHA-256/size against the report, and revalidate the copied manifest.
  Refuse author edits during copying; later source edits cannot split child content.
- Pass identical snapshot paths to both children through existing CLI options.
  Keep the private temporary directory alive until both children stop; remove it
  on normal close, interrupt, child failure and login cancellation.
- Refuse individual profile overrides and incompatible transport/world/development
  modes before launching. Navigation, saved terrain, players, guide position and
  display selection remain independently configurable.
- Real owned OIDC/PostgreSQL/native launcher checks cover interrupt, child failure
  and pending-login interrupt, compare ten profile hashes in actual process argv,
  check SQL/item/progress behavior, strip client SQL URLs, remove private
  tickets/snapshots and stop owned services.
- Appropriate Python/Rust validator tests, formatting and denied-warning Clippy pass.

## Limits

The manifest does not replace authoritative world identity: existing runtime
profile byte hashes retain their current authority/compatibility roles. Terrain,
navmesh, player assets and optional spell feedback are not snapshotted by this
bundle. This does not add general quest acceptance/turn-in, new combat, engine
streaming capacity, a patcher, hot reload, native Windows or 4K qualification.
The explicit native helper uses synthetic keyboard/picking; physical OS input and
physical browser interaction remain separate. No new performance claim is made.

## Evidence

`build/gameplay-manifest-launcher-v1-20261031/verification.json` passes all three
cleanup modes with actual local provider/database and native Linux client. Actual
process argv confirms ten identical profile hashes, live snapshot files and removal
after children stop. SQL progression/objective/reward/equipment checks pass and the
native client receives neither configured SQL URL. Owned services stop. Display is 1280×720
on the existing RX 7900 XTX/RADV/Vulkan setup; no rendering source changed and no
new display/readability, physical browser or OS-input qualification is claimed.
The synthetic equipment driver uses the existing normal development profiles;
this does not repeat the earlier complete three-kill integration gate.

The unchanged runtime binaries are client
`1af7087e421fd5928f6799769b4a48fc516c57682cdff6d2bc9141618fffa9ee`
and normal server
`c723c348de5416361318c565c1c6b1ec0f436edf9146e198bdb2b9a69912a050`.
The actual validator is the previously frozen offline-content executable.
Source/helper/proof is frozen at `build/gameplay-launch-qualified-v1-20261031`;
existing full training/vendor/content checkpoints remain intact.

`build/gameplay-manifest-python-targeted-v1.log`: sixteen passes (seven new bundle
checks, five account and four GPU helper regressions). Eight unchanged actual Rust
validator tests pass; formatting and denied-warning workspace/all-target Clippy
pass. A broad host-Python discovery attempt also encountered three unrelated legacy
asset-test import failures (`bpy`, `xxhash`, `numpy` unavailable in that interpreter);
it is not reported as a passing full Python suite. No Rust source changed, so the
previous 441-pass/30-skip workspace result remains historical rather than rerun.

Follow-up found during lifecycle review: the first launch qualification checks
`THEIA_DATABASE_URL`/`THEIA_TEST_DATABASE_URL` removal only. The owned fixture also
supplies `PGPASSFILE`; that setting still reaches the native child in this first
checkpoint. The helper's earlier `client_database_environment_removed` flag must be
read as URL-only evidence. The qualified follow-up above now filters libpq/provider/test-browser
settings and asserts their absence in the actual native process environment.

Service-handoff follow-up evidence is
`build/gameplay-manifest-launcher-v2-20261031/verification.json`. Two actual native
runs check `/proc` keys for libpq, database URL, account, OIDC and test-browser
configuration, with the owned parent fixture's `PGPASSFILE` present. The third
pending-login cancellation starts no native client and reports that check as
inapplicable. All three retain exact ten-profile handoff and clean service/ticket/
snapshot shutdown. The same earlier frozen native client/server/validator binaries
are reused; this does not qualify migration-9 lifecycle runtime adoption.
