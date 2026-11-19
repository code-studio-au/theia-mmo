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

Closed quest-subset compiler qualified, 3 November 2026:
[Authoring workflow](QUEST_PACK_AUTHORING.md) now includes explicit bounded
`--compile-selection`. It preserves definitions and full prerequisite closure,
emits exact source/selection/output provenance and reparses the result through
the existing closed catalogue validator. Actual compiled output passes schema-13
manifest admission; unknown installed contacts still refuse. Full 1,089-test
workspace, 45 gameplay / 11 authoring-tool checks, formatting and Clippy pass.
No normal bundle, runtime pack loader or save migration changes.

Separate offline quest pack/index workflow, 3 November 2026:
[Authoring guide](QUEST_PACK_AUTHORING.md): `validate_quest_packs` validates a
bounded global executable-quest namespace/prerequisite graph and reports exact
source hashes. Seven file tests, 297 core tests and actual CLI success/refusal
pass; normal gameplay manifest checks remain separate and retain 44 checks.
This index is not a normal runtime manifest and does not load cross-pack journals
or validate installed world targets. No gameplay schema/default changes.

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

Current continuation, 2026-10-31: the validator now also supports the opt-in
[launch-time snapshot handoff](GAMEPLAY_MANIFEST_LAUNCH.md) through `tools/play.py`.
The original offline tool remains unchanged; the manifest is still not itself an
authoritative world identity. Notes below describe the initial offline checkpoint.

# Offline gameplay content validation — initial P3-15 slice

Status, 2026-10-30: initial offline validator qualified. Final unchanged-source
workspace passes **441 tests / 30 explicit skips** in 100 groups. Formatting,
denied-warning workspace/all-target Clippy, eight focused tool tests and eight
actual CLI refusal cases pass. Exact source, executed tool/test binaries, docs and
proof are frozen at `build/gameplay-content-qualified-v1-20261030`.

All 516 previously qualified source files remain byte-identical to
`build/vendor-qualified-v1-20261030`; the only new source inputs are the manifest
and offline example tool. That playable checkpoint retains twenty actual SQL
contracts and native purchasing at three display sizes. No new rendering/native
or SQL run is needed or claimed for this offline-only change. Changes remain local;
P3-17/18 remain unspecified.

## Scope and actual prerequisites

Provide an offline example tool for the existing ten-profile development gameplay
bundle: items, abilities, stats, statuses, creature, informational dialogue, vendor,
rewards, XP and objective. Reuse the actual core/runtime parsers and cross-reference
checks. This is P3-15 preparation for an authored persistent zone (P3-16), using
P1-10 and the qualified initial P3-04 through P3-12 schemas. Remaining production
terrain/vista work does not block these data checks.

The initial manifest has schema one, a checked stable ID, a positive revision and
exactly those ten required relative JSON paths. Limit the manifest to 8 KiB and
paths to 256 portable ASCII bytes. Reject absolute/traversal/empty-component paths,
backslashes, outside-root symlinks, canonical path aliases, missing/nonregular files,
unsupported/duplicate fields and oversized inputs. Read each profile once, bounded
by its existing byte budget, and validate/hash those same bytes.

Check existing ability/resource/status compatibility with current development
combat defaults; vendor/reward item catalogues and item definitions; the vendor's
NPC against the declared guide; and reward/XP/objective creature IDs against the
one declared development creature. Do not duplicate these schemas in a Python
approximation. Preserve the existing direct client/server profile options.

## Concrete acceptance

- The checked-in development bundle validates through the actual CLI, with a
  deterministic report of relative profile paths, sizes and exact SHA-256 hashes.
  Manifest or profile byte changes alter the validation-bundle digest.
- Meaningful tests reject malformed/bounded manifests, unsafe/missing/nonregular/
  oversized files, wrong NPC/item/creature references and invalid ability/status/
  resource combinations. Tests modify owned temporary copies of content.
- Error output identifies the failing category without echoing raw profile data.
  The tool performs no writes to source profiles and starts no game, provider or SQL.
- Formatting, denied-warning Clippy and appropriate tool/regression checks pass.
  Preserve the qualified playable vendor checkpoint and record exact source/tool
  evidence separately from runtime or MMO-scale claims.

This first validator checks declared profiles/references. Navigation route loading,
terrain collision/spawn support, visual assets, production economy, native runtime
adoption of the manifest, general authoring UI and story-to-game conversion remain
separate work. Its validation-bundle digest is not an authoritative world identity.

## Run and evidence

```sh
cargo run --locked -p theia-network --example validate_gameplay -- \
  content/gameplay/development-gameplay.json
```

Success prints a JSON report; invalid input exits nonzero without a report.
The report lists ten sorted role/path/byte-count/SHA-256 records, the exact manifest
hash and a domain-separated validation-bundle hash. It validates the current
one-creature development bundle with existing development target health/respawn
settings. It does not install the manifest into the client/server or replace
terrain/world identity. Existing explicit profile CLI options stay available.

`build/gameplay-content-cli-v1-20261030/verification.json` records two identical
successful reports, correct byte hashes/counts and eight real CLI refusals: wrong
NPC, missing status reference, wrong items, traversal, malformed/oversized profile,
missing required manifest field and duplicate field. Error output does not echo
the injected private-content sentinel. All eleven input files remain unchanged.

`build/gameplay-validation-tests-v1.log` records eight focused passes, additionally
covering source immutability, whitespace-sensitive identities, strict revision/ID/
field bounds, portable path variants, missing/nonregular files, all declared
creature-reference families, mana/status compatibility and actual Linux symlink
escape/alias refusal. Windows execution remains unqualified. The final workspace
log is `build/gameplay-content-workspace-tests-v1.log`; the unchanged-source summary,
format and denied-warning Clippy proofs use the same prefix. Existing SQL/private-
asset skips retain their previous qualified or explicitly unqualified scope.
