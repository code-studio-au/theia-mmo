# Theia Human Campaign — Current State and Authoritative Sources

**Current-state entry point — 2 November 2026.** Work from this page, not historical drafts. The Human launch campaign is **levels 1–80**. The story is approved; executable Rust quest content is **not yet complete**. Git history, rather than live superseded documents, preserves deleted iterations.

## Source-of-truth hierarchy

1. **Narrative canon:** [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md) — locked plot, 184 editorial quest concepts and immutable story outcomes. Any contradiction in downstream files resolves here.
2. **Quest authoring contract:** [Rust Quest Authoring Contract](Human_Rust_Quest_Authoring_Contract.md) — required IDs, evidence, giver/turn-in, choices, rewards, retries, credit scopes and planned versus implemented capabilities.
3. **Legacy idea bank:** [Side Quest, Zone and Character Archive](Human_Legacy_Sidequest_Zone_Character_Archive.md) — non-canonical optional regional hooks and historical contacts retained for future review, not active quest definitions.
4. **Provenance and migration:** [Editorial ID Crosswalk Draft](Human_Quest_Editorial_ID_Crosswalk_Draft.json) — all 92 original H-series identifiers and all 24 HS-series optional IDs with source objectives; mappings not yet approved. Do not allocate stable runtime IDs based solely on titles.
5. **Working quest data:** [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) — **one consolidated 184-quest / 684-stage working inventory** with typed objectives, source editorial candidates, explicit blockers, and no inferred hard level gates. This is **not** executable Rust data. The nine older stage JSON files have been verified against the manifest and removed from the active tree; Git history preserves them. Edit only the consolidated manifest.
6. **Dialogue:** [Branching Key Scenes and Reveal-Gate Reference](Human_1_80_Key_Scenes_Branching_Dialogue.md) and per-level NPC dialogue scripts. Dialogue is first pass and subject to reveal-gate review.
7. **Encounters:** [Encounter Barks and Cinematic Timing](Human_1_80_Encounter_Barks_and_Cinematic_Timing.md) — authored design, not integrated encounter code.
8. **Actual implementation authority:** existing Rust/Bevy code and [Quest Objectives Foundation](../../../QUEST_OBJECTIVES_FOUNDATION.md). Rust validates quest progress and authoritative events; Luau is optional for unusual scripted behavior and not confirmed integrated. [Rust–Luau Boundary](Human_Rust_Luau_Quest_Scripting_Boundary.md) is a proposal.

## What is approved versus not

**Approved:** campaign narrative and ending, Veilbreaker destroys Veil, Binding dissolves immediately, Shroud begins dissipating, Seven survive, Nhal manifests gradually and is shattered at level 80, Oth'varen dies physically, and divine essence remains.

**Not approved as executable data:** provisional 156-main/28-optional split, level bands as hard gates, current completion aliases, original-to-new editorial ID mapping, exact NPC IDs, item consumption, objective credit scope beyond current engine capabilities, rewards, world positions and scripting integration.

## Cleanup policy

- **Delete:** explicitly superseded 1–60 and provisional 1–80 narrative drafts after their active decisions are reflected in the canonical campaign; the JavaScript quest runtime prototype and tests because they conflict with the Rust implementation direction.
- **Keep temporarily:** historical H-series register **only while** original objective/ID provenance remains unresolved; its story claims are not authoritative. The current editorial crosswalk explicitly records unresolved mappings.
- **Keep pending reconciliation:** stage-level quest packs, implementation briefs and numerical progression material if they contain unique details not yet merged into current quest data. Do not delete unique work merely to reduce file count.
- **Future target:** one validated Rust-compatible quest definition set, one canonical narrative source, referenced dialogue and encounter assets, and no redundant stage packs. Delete redundant drafts only after the content is reconciled and links updated.
- **Repository scope:** this cleanup covers Human story/quest design. It does not delete unrelated engineering research, world lore, gameplay source, or other race campaigns without separate audit.

## Authoring review status

All 184 quests have completed first-pass Rust authoring review. Safe editorial corrections have been applied directly to the sole manifest: spoiler-safe Grey Surveyor contacts, Q157 ongoing Shroud fade, and an explicit Q181 Oth'varen death fact between alignment confirmation and final inversion. Proposed optional access conditions and the level-80 finale gate are recorded as **non-executable review metadata**, not silently activated. The new Q181 story-fact objective requires future Rust support. The 184-quest count is unchanged; the authored stage count is now 684.

## Production register retirement

The duplicate 184-entry `Human_1_80_Production_Quest_Specification.md` has been retired from the active tree. All 184 historical location bands, player-purpose descriptions and lore summaries are preserved as **unapproved editorial context** in [Location and Lore Reference](Human_1_80_Location_and_Lore_Editorial_Reference.md); its outdated hard-level and generic-stage claims were not migrated. The current manifest remains the sole structured working quest inventory. The 17 legacy playable packs/briefs have also been retired, with originals recoverable through Git history.

## Next reconciliation milestones

1. Verify all 92 H-series and 24 HS-series original quest IDs against canonical Q001–Q184 and record reused/modified/split/merged/retired/new.
2. Allocate stable runtime IDs, preserving original editorial IDs and separate Q reference.
3. Review and complete the single current authoring manifest, allocate approved IDs, and generate validated Rust-compatible content from it. The nine old stage JSONs have been deleted after validation; do not recreate them.
4. Consolidate any unique dialogue, stage or encounter details from redundant drafts before removing them.
5. Validate all references and implement/test against the existing Rust quest foundation.

**Do not call the Human quest content production-ready until these milestones and engine tests pass.**

## Obsidian retirement (in progress)

The creator has discontinued Obsidian. Obsidian-only navigation (`Story Browser.base`, `Quest Index.md`, `Welcome.md`) has been removed. Legacy H/HS per-quest Obsidian notes are being retired; some were deleted, but **the removal is not yet complete or verified**. Do not treat surviving H/HS vault pages or old source-library copies as active quest content. Preserve the 184-quest manifest, canonical narrative, approved dialogue and engine agreement. Before removing source-library duplicates or old H/HS registers, verify original editorial provenance and links.
