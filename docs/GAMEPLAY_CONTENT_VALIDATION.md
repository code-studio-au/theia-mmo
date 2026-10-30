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
