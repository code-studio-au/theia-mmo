# Optional completion XP — qualified 2026-10-31

Current complete checkpoint: `build/quest-turn-in-xp-qualified-v1-20261031`.
Final unchanged-source workspace: **475 passes / 37 explicit skips**
in 107 groups; formatting, denied-warning workspace/all-target Clippy and
23 focused Python checks pass. The implemented native flow and limitations are
recorded below. Earlier prerequisite sections retain their historical narrower
scope; they do not limit the later combined result.

# Quest turn-in XP — P3-10 / P3-11 / initial P3-16

Status, 2026-10-31: store and retained owner prerequisites qualified. All 69
selected network tests, formatting and denied-warning workspace/all-target Clippy
pass. Actual OIDC/encrypted-ticket/SQL retirement passes positive completion XP,
retained journal-only mode and automatic objectives with one executed probe.
Source/probe/proof: `build/quest-turn-in-xp-owner-qualified-v1-20261031`.
The SQL checkpoint retains 27 contracts / 32 commands and 84 core passes.
The journal-only live checkpoint remains at `build/quest-lifecycle-qualified-v1-20261031`
(464 workspace passes / 33 skips, actual two-account native/normal-launcher checks
at three sizes). No completion XP is enabled by normal executable selection yet.

## Scope and actual prerequisites

The first reward is XP-only, serialized by the existing character progression
owner with accepted quest actions and trusted kills. That owner already retains
work through retirement and publishes committed XP and quest state. Item turn-in
grants will require consistent coordination with the character's inventory owner
and are outside this slice. Existing first-kill personal loot remains independent.

The optional development recipe grants 25 XP for
`theia.dev.training-exercise`, bound to `theia.dev.progression`. Three ordinary
25-XP kills leave 75 XP; the proposed completion award reaches level 2 at 100 XP.
This is a new, unadopted recipe file. No existing profile, manifest selection,
kill reward, journal-only mode, engine pin or runtime balance changes.

`quests::turn_in_xp::Recipe` checks a bounded schema-1 source (at most 4096 bytes),
stable recipe/quest/progression IDs, the matching quest's configured kill reward
and a positive grant at most 1000 XP. Unknown/duplicate fields, foreign IDs and
invalid amounts refuse. Its distinct profile domain is reserved for exact-byte
adapter binding; logical core IDs alone do not establish content compatibility.

`progression::Profile::propose_grant` supplies bounded trusted arithmetic, sharing
the unchanged kill calculation. It clips at the level cap, keeps the XP revision
unchanged for a zero capped award and refuses a positive award when the revision
is exhausted. It establishes no entitlement or receipt consumption.

A combined proposal accepts TurnIn only on Ready state, preserves both inputs and
returns completion plus XP together. Stale, premature, already-completed and foreign
states refuse. XP calculation failure returns no partial completion. These are
pure proposals: returning one commits nothing, grants nothing and deduplicates no
external request.

## Adoption requirements

- Bind exact quest, recipe and progression bytes in compatible store/wire profiles
  and world identity. Keep journal-only mode available; never silently reward
  earlier completed journals or reuse their receipts under a changed recipe.
- Lock XP before quest state, matching the existing trusted-kill transaction order.
  Commit the completion state, actual capped XP award and canonical action receipt
  atomically. Receipt recovery returns current committed state without re-awarding.
- Keep one generation-bound FIFO for accepted actions and kills, sixteen SQL jobs,
  bounded encrypted requests and no database I/O inside the simulation step.
  An accepted turn-in must drain through logout before character-slot reuse.
- Publish committed XP and quest state before success acknowledgement. Add bounded
  recipe/mode launch validation and a truthful standard Bevy reward display.
- Qualify actual SQL rollback, capped/positive awards, stale/conflicting retries,
  lost answers and retirement; then actual encrypted and two-account native
  completion/reconnect/database restart, with unchanged legacy mode regressions.

P3-17/18 remain unspecified. The 464-pass live checkpoint describes its own frozen
source; it is not a new full-workspace result for this proposal prerequisite.

## Pure prerequisite evidence

`build/quest-turn-in-xp-core-tests-v1.log` passes all seven new cases: strict recipe
bounds/cross-references, three-kill level-up, premature/acceptance refusal, clipping
and zero capped award, positive-award revision exhaustion, foreign profiles/states,
and trusted-grant bounds/equivalence with the existing kill arithmetic. Inputs
remain unchanged on every outcome and completion retries cannot propose another
award. `build/quest-turn-in-xp-core-all-v1.log` passes all 84 core tests.
Formatting and denied-warning workspace/all-target Clippy pass. No new full-workspace,
SQL, encrypted or native reward qualification is claimed. Next is exact-byte-bound
catalogue adoption and atomic SQL/worker commands, followed by the live owner.

## SQL/worker prerequisite checkpoint

Migration 10 appends optional XP metadata to the existing bounded action receipts.
Journal-only receipts retain all-NULL metadata. Rewarded receipts bind the exact
XP profile and record actual granted units plus committed XP revision/total;
partial metadata and positive acceptance grants violate SQL constraints.

`QuestLifecycleCatalog::from_json_with_turn_in_xp` keeps the old journal digest
unchanged and constructs a separate rewarded digest from the new lifecycle-XP
domain followed by three fixed 32-byte digests: journal lifecycle, exact recipe
and exact progression. Raw recipe hashing uses its own core profile domain.
Whitespace/content changes bind differently. Existing journal rows cannot silently
adopt that digest or acquire completion XP. The old command API refuses a rewarded
catalogue before changing state; the new combined command requires one.

The transaction authorizes the character/world, locks XP before quest state,
checks old canonical receipts, then commits completion/XP/receipt together.
Recovery returns current committed state and newly granted zero. Recorded XP
snapshots are checked against the exact recipe, profile and current monotonic
state; capped awards and later kills remain recoverable. The worker uses the same
sixteen-job FIFO and adds positive-award/unit/capped counters. Already executing
SQL survives a canceled answer; queueing alone remains no acknowledgement.

Four actual owned SQL contracts pass at
`build/quest-turn-in-xp-sql-focused-v2-20261031`: positive/clipped/capped grants,
owner/profile fences, trigger rollback, revision exhaustion, malformed receipt
recovery, retained executing answers, FIFO snapshot ordering, bounded queue,
backend termination and migration-9 journal preservation without silent rewards.
The first focused attempt expected a stale-command error for a request conflicting
with an existing acceptance receipt. Its assertion was corrected to use an actually
stale request; the failed log remains separate. The clean restart probe's first
build used the wrong `select_character` signature, corrected to the existing
checked binding API. The complete SQL regression/process-restart qualification now passes 27 actual
SQL contracts / 32 commands at `build/quest-turn-in-xp-postgres-v1-20261031`.
Available, Ready and completed characters, exact action/XP snapshots and old receipt
recovery survive fresh processes and owned database restart. All 84 core tests,
one exact-binding test, formatting and denied-warning workspace/all-target Clippy
pass. Source/proof: `build/quest-turn-in-xp-store-qualified-v1-20261031`.
No new full-workspace or native result is claimed. No normal runtime owner, wire/CLI
or native completion-XP mode is adopted yet.

## Wire/world and retained owner checkpoint

The existing QL01/QA01/QR01 frames retain their bounded sizes and client-owned
revision/NPC intent fields. A rewarded lifecycle digest also pins the exact XP
profile; world activation, admission-port configuration and client setup refuse a
mismatched profile. Legacy journal digest behavior stays unchanged. Seven wire
tests include an independently calculated binding vector, exact byte changes and
cross-mode observation refusal. All 62 library tests pass, including matching
rewarded guide activation/range checks and the existing actual encrypted
journal-only runtime test. This is 69 selected passes, not a new full workspace.

The generation-bound owner selects the combined SQL method for a rewarded
catalogue and publishes current XP, lifecycle and reply in one observation after
COMMIT. Normal lifecycle refusals retain the owner; unexpected failures close it.
The existing server publisher sends XP before lifecycle/reply on the reliable
channel. Accepted actions and kills retain their one FIFO across retirement.

`build/quest-turn-in-xp-retirement-v1-20261031/verification.json` passes actual
verified OIDC, encrypted ticket consumption and owned SQL against production gate
source. Two pre-accept kills leave the journal Available and earn 50 XP. Blocked
acceptance plus three queued kills survive logout to Ready / 125 XP. Blocked
turn-in plus a repeated old fact and later new kill survive logout to TurnedIn /
175 XP. The receipt records the intervening completion snapshot at 150 XP,
revision 6, with 25 newly granted XP; the queued kill follows at revision 7.
Fresh authorization cannot bypass retiring work. A fresh lease recovers both old
commands without another award or progress rollback. Exactly two action receipts,
six victim dispositions and three counted kills remain; seven positive XP commits
include the single completion award. No acceptance/completion/XP is advertised
before COMMIT.

The same executed probe passes journal-only retirement at
`build/quest-turn-in-xp-journal-retirement-v1-20261031` and automatic-objective
retirement at `build/quest-turn-in-xp-automatic-retirement-v1-20261031`.
These drive leases with already authorized synthetic action inputs. They establish
neither live NPC entitlement nor a complete native rewarded-gameplay flow.
Next is validated manifest/CLI selection, standard Bevy reward presentation and
combined actual native qualification. Earlier live binaries stay preserved.

## Manifest/CLI/Bevy adoption under native qualification

A new schema-2 manifest adds the required `quest_turn_in_xp` profile to the ten
existing roles. Schema 1 retains exactly ten roles and rejects a reward role;
schema 2 requires a non-null regular recipe path. The actual Rust validator checks
the core recipe against the declared quest and XP profiles, with the same canonical
path/alias and bounded-byte rules. Reports keep envelope schema 1 and expose
`manifest_schema`; the legacy report/digest is unchanged apart from that additive
field. Ten Rust validator tests and 23 focused Python checks pass, including sealed
eleven-role snapshots and cleanup after author edits or invalid input.

`tools/play.py --gameplay-manifest content/gameplay/development-gameplay-turn-in-xp.json`
selects explicit rewarded mode and hands identical private eleven-profile paths to
both children. Individual recipe overrides cannot be mixed with a manifest.
Direct launchers may select `--quest-lifecycle --quest-turn-in-xp <recipe.json>`
with matching progression/creature/guide configuration. The standard Bevy guide
panel displays the configured completion XP and notes that the level cap applies.
The journal itself keeps its existing two-line footprint.

Executable selection and reward text are implemented; combined native qualification
is in progress. Its fixture keeps the default XP curve and uses the existing
instant-damage ability fixture. The native driver must observe 75 XP at Ready and
100 XP / level 2 at completion, before success, with exactly two durable action
receipts and a 25-XP completion snapshot. Rejoin/database restart must make no new
award. Personal loot, barter and equipment remain independent and must still pass.
Fresh compatible development characters are used for the rewarded mode; existing
journal-only rows reject the new digest and remain unchanged. Automatic migration
or retroactive rewards are not implemented.

## Combined native and final workspace checkpoint

Actual two-account OIDC/encrypted/SQL native runs now pass at 1280×720 windowed
(`build/quest-turn-in-xp-native-1280-v1-20261031`), 1920×1200 windowed
(`build/quest-turn-in-xp-native-1920-v2-20261031`) and 3440×1440 borderless
fullscreen (`build/quest-turn-in-xp-native-3440-v2-20261031`). Each character
accepts through standard Bevy controls, reaches Ready after three authoritative
kills at 75 XP, and turns in once to reach 100 XP / level 2 on the default curve.
The committed XP observation precedes turn-in success. Two exact action receipts
record acceptance at zero and completion at 100 XP, with a single 25-XP award.
Private progress, personal loot, barter and equipment still pass. Reconnect,
server/database restart and backend-loss checks preserve state without new awards.

The windowed runs have zero Vulkan validation errors. Fullscreen records one
previously documented upstream surface-resize diagnostic per native startup;
other validation errors fail qualification. The first ultrawide windowed attempt
completed fresh gameplay but was rejected because the window manager reduced its
height to 1368; it is not counted as a 3440×1440 pass. The first 1920 attempt
exposed an outdated helper expectation for capped journal-only XP; corrected
helper checks now verify the default curve. Neither required a game binary change.

The same optimized client and native server probe are used for all three passing
runs. Their SHA-256 values are respectively
`c29dc6e9bc8e3c612c3705e81bab3ecef52f96e55bbf72004274b310eac90c48` and
`3570dc5f472876a95bfd1a6c3450f6fc6aa0128cf79e72f5313baf6e43ff6124`.
Owned providers/databases stopped and private ticket files were removed in every
passing run. Native drivers send Bevy keyboard/picking events; physical OS input,
native Windows, physical 4K, production balance and population capacity are not
qualified by these tests. Normal eleven-profile launcher startup/cleanup now passes
all three modes at `build/quest-turn-in-xp-launcher-v1-20261031`. The same current
binaries also pass the full journal-only native flow at 1920×1200, including
reconnect, database/server restart and backend loss, with no completion reward:
`build/quest-turn-in-xp-legacy-native-1920-v1-20261031`. Its windowed run has zero
Vulkan validation errors. The final unchanged-source workspace gate and all final checks pass; exact source,
matched binaries, validator and public proof are preserved in the complete
checkpoint named above.
