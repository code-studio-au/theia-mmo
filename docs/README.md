# Theia MMO documentation

Use the subject guides below for implementation contracts and development
workflows. Start with the [project README](../README.md) to build or run the game.
The [contributor instructions](../AGENTS.md) define the required reading order.

## Project records

- [Status](STATUS.md): implemented behavior and dated verification evidence.
- [Decisions](DECISIONS.md): adopted direction and explicit supersessions.
- [Roadmap](ROADMAP.md): milestones and their history.
- [Engineering](engineering/README.md): architecture, implementation plan and task acceptance criteria.

## Subject guides

| Folder | Contents |
| --- | --- |
| [Gameplay](gameplay/README.md) | Combat, movement, NPCs, quests, inventory, progression, parties and chat |
| [Client](client/README.md) | Native frontend, HUD, controls, character presentation, maps and UI artwork |
| [World](world/README.md) | Terrain, collision, coordinates, streaming, world packages and regression maps |
| [Assets](assets/README.md) | Asset preparation, materials, animation inputs and provenance |
| [Editor](editor/README.md) | Zone authoring, playtests and Sprinkles integration |
| [Networking](networking/README.md) | Renet, retained Quinn diagnostics, prediction and transport qualification |
| [Accounts](accounts/README.md) | Authentication, character ownership, admission and PostgreSQL persistence |
| [Operations](operations/README.md) | Desktop launcher, release delivery and infrastructure runbooks |
| [Archive](archive/README.md) | Superseded handoffs, frozen experiments and dated audit/test snapshots |

## Reading historical material

Feature guides can contain several dated stages of the same implementation.
Read their latest applicable update before an older limitation or command.
Qualification applies to the named revision, content profile, platform and inputs;
it does not automatically qualify today's build. Old release numbers, pending
work, permissions and local `build/` paths are historical context.

The archive preserves unique decisions, reproduction details and evidence needed
to understand retained code and regressions. It is not an active work queue.
[The documentation review](engineering/DOCUMENTATION_REVIEW.md) records the cleanup
and checks. The sibling story repository links directly to the quest guides in
`gameplay/`; the obsolete forwarding pages have been removed.

## Ownership

Story, lore, campaigns, zone briefs and NPC/character writing live in
[theia-story](../../theia-story/README.md). Runtime definitions, assets, engineering
contracts and validation belong here. Licenses and vendored provenance remain
beside the code or assets they describe. Tool README templates remain with their
generators because they are used to produce asset packages.
