# Engineering documentation

Start here for the adopted architecture, implementation plan and acceptance
criteria. Progress logs and earlier task-status snapshots are preserved in the
[engineering archive](../archive/engineering/README.md).

## Reading order

1. Read the repository [contributor instructions](../../AGENTS.md),
   [status](../STATUS.md), [decisions](../DECISIONS.md) and [roadmap](../ROADMAP.md).
2. Read the [architecture](ARCHITECTURE.md),
   [implementation plan](IMPLEMENTATION_PLAN.md) and [task catalogue](CODEX_TASKS.md).
3. Check the [alignment record](ALIGNMENT.md) for the relevant slice and its
   prerequisites. Its dated entries retain their original scope.
4. Use the applicable [subject guide](../README.md) and define concrete acceptance
   checks before changing behavior.

For controls or combat, follow [THEIA-COMBAT-001](../gameplay/COMBAT_TARGETING_MOVEMENT_SPEC.md),
[the movement contract](../gameplay/MOVEMENT_CAMERA_TARGET.md) and
[Classic formula evidence](../gameplay/CLASSIC_FORMULAS.md). These supersede the
hybrid movement and facing proposals retained in historical plans.

## Plans and evidence

| Document | Purpose |
| --- | --- |
| [Architecture](ARCHITECTURE.md) | Adopted system boundaries and design constraints |
| [Implementation plan](IMPLEMENTATION_PLAN.md) | Planned slices and sequencing |
| [Task catalogue](CODEX_TASKS.md) | Task IDs, prerequisites and acceptance checks |
| [Alignment record](ALIGNMENT.md) | Dated implementation and migration decisions |
| [Hosting and operations](HOSTING_AND_OPERATIONS_REQUIREMENTS.md) | Adopted hosting requirements; runbooks live with infrastructure |
| [Validation record](VALIDATION.md) | Historical test evidence and reusable check commands |
| [Documentation review](DOCUMENTATION_REVIEW.md) | Review scope, cleanup decisions and remaining gaps |
| [Imported contributor reference](AGENTS.reference.md) | Original handoff provenance; active rules are in the root AGENTS.md |

A task definition is not a completion claim. A local, synthetic or historical
check proves only its recorded scope. Use current source and the latest relevant
status entry to decide what still needs implementation or qualification.
Keep P3-17/18 unspecified until the user supplies their scope.
