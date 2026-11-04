# Quest authoring reconciliation review — 4 November 2026

Reviewed the supplied `theia-quest-authoring-reconciliation-a0c31c1.patch`
against remote main `ad2f17093b806413420802313da64d48617ac542`. The original patch
changes ten story files and applies cleanly to that base. The paused MMO working
checkout was not rebased, pulled, reset or included in this branch.

The patch reconciles authoring documents and JSON only. These files remain
non-executable. It does not implement expiry, world phasing, story facts or the
Human campaign in the Rust quest engine.

## Q169 death-event reference

The manifest's Q169 `authoring_availability.cutoff` resolves exactly to Q181's
`q181.objective.othvaren_death`, with editorial fact
`othvaren_physical_death`. That objective is stage 3: after stage 2 confirms
alignment while Oth'varen is alive, and before stage 4 performs final inversion.
It requires trusted server evidence and retains the historical fact on failure.
The reference therefore targets the authored death event, not Q181 acceptance,
final inversion, completion or its later Sera turn-in.

The referenced conversation is Q169 `q169.objective.04`, whose editorial target
is `othvaren_journal_interpretation`. The authored cutoff rules consistently:

- Prevent new acceptance after the death event.
- Expire accepted Q169 quests lacking that completed conversation, explaining why.
- Preserve completed quests and recorded evidence.
- Leave completed-conversation/unfinished-custody handling unresolved.

These checks validate authoring references, not a running death-event adapter.
Stable actor/target IDs are null. The runtime has no adopted Q169 death binding or
expired quest state; event-driven expiry, durable replay/rejoin and concurrent
conversation/death ordering still require implementation and acceptance checks.
The cutoff must observe the player's authoritative story phase, not an unrelated
player's kill or a temporary combat despawn.

## Remaining stuck-quest cases — no new creator decision

Q169 stage 5 preserves the journal with custodians, but its final turn-in contact
still names Oth'varen. Two cases must be resolved before executable adoption:

- The player finished his conversation, but custody is unfinished when he dies.
  The supplied decision explicitly leaves this open; do not infer expiry.
- The player finished conversation and custody but has not finalized turn-in or
  received rewards when he dies. The patch does not explicitly define whether
  that ready quest counts as completed for the preservation rule. A dead-NPC
  turn-in must not become an impossible requirement.

This review does not approve another contact, resurrection, automatic completion
or a new expiry condition. The authoring-only reconciliation can be reviewed as
such; it does not certify production freedom from these stuck states. Resolve
those policies and implement their runtime checks before exporting Q169.

## Validation performed

JSON parsing and targeted structural/reference checks passed:

| Check | Result |
|---|---|
| Quest references | Exactly Q001–Q184, unique |
| Stages / objectives | 685 / 687, globally unique |
| Existing stage/objective IDs | All preserved |
| Quest prerequisite edges | 158, resolved and acyclic |
| Stage transitions | Resolved, acyclic, all stages reachable |
| Derived dependency reference | Titles and completion-alias prerequisites match manifest |
| Canonical titles | All 184 match |
| Main-story independence | No transitive dependency on 28 regional optional quests |
| Q159 / Q168 / Q169 access | Q156 prerequisite, no additional hard minimum level |
| Q176 / Q181 turn-in | Sera |
| Runtime and stable target IDs | Remain unassigned |
| Q169 cutoff / conversation references | Resolve to the intended authored objectives |
| Patch whitespace | `git diff --check` passes |

Two stale summaries were corrected during review: the manifest no longer calls
all Q169 post-death availability open, and the finale review no longer calls
Q174's reconciled authoring metadata conflicting. No creative policy was added.
Rust, rendering and gameplay code were not changed; this document does not claim
new Rust test, GPU or executable-campaign validation.
