# Human Q018–Q034 — Rust Quest Authoring Review

**Status: Needs Review — planning analysis, not executable content.** [Current manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the single working quest inventory; [canonical campaign](Human_1_80_Canonical_Campaign.md) governs narrative; [Codex quest engine agreement](../../../QUEST_ENGINE.md) governs implemented runtime features. This review makes **no** changes to the 184 quest entries.

## Engine constraints (2 November 2026)

Current Rust catalogue supports versioned quest definitions, prerequisites, ordered stages, `all/any/at_least` stage completion, native `kill` and semantic `npc_interaction` objectives, once-only character credit, and XP recipes. The new multi-quest journal/Store has qualified pure/SQL tests but is **not yet adopted into the normal playable path**. Object inspection, region travel, encounter-defense outcomes, scene observations, branching choices, party sharing and phasing remain planned. Cross-pack registry and saved-content migrations remain future work. Do not encode unsupported types as a generic conversation or assume a permanent 32-quest limit.

## Quest-by-quest authoring dispositions

| Quest | Proposed stage structure | Engine fit and blocking details |
|---|---|---|
| **Q018 An Investigator's Standard** | Intake with Mara → authenticate courier and warehouse copies in either order → receive inquiry authorization | NPC semantic actions native; document inspections planned. Require distinct `submit_case` and `authorize_inquiry` actions, not a generic Mara click |
| **Q019 Dates in the Register** | Lysa grants access → copy and compare two records → authenticate with Lysa | Separate Lysa access and final verification semantic actions; inspection needs trusted evidence |
| **Q020 The King's Confidence** | Authorized royal audience → Aldren testimony and Cassian lead → receive warrant | Do not require passive cutscene playback as the only evidence; record an authoritative completed-audience fact. Cassian must initially be credible |
| **Q021 A Convenient Suspect** | Garrick interview → check two independent military records in either order → report to Mara | Garrick's opposition to peace is not proof of Edric's murder; preserve exonerating record |
| **Q022 Before the Request** | Compare authorization and format → interview independent witness → report contradictions | Documentary evidence and witness provenance distinct; do not conflate Q025's optional witness account with the mandatory interview |
| **Q023 The Clerk's Queue (optional)** | Receive repair request → inspect public index → restore references → confirm | Ordinary civic service; must not grant mandatory restricted archive access |
| **Q024 Bread and Barracks (optional)** | Receive ration request → compare ration book and delivery manifest in either order → arrange supplies | Ordinary supply shortage, not a conspiracy; don't require inventory item without possession/consumption decision |
| **Q025 A Witness's Own Words (optional)** | Interview guidance → separate optional witness statement → register record | **Potential duplication:** main Q022 already requires an independent witness. Define a different witness or make Q025 a distinct interview-skills exercise; never gate Q027 |
| **Q026 The Walls Outside Court (optional)** | Receive patrol order → inspect road → defeat raiders → report | Native `kill` only if a specified creature/template and kill count are intended; `clear raider threat` may instead require planned encounter outcome |
| **Q027 The Price of an Office** | Obtain authorization → locate intermediary → protect extraction → secure payment copy → authenticate custody | Witness survival and payment provenance are mandatory. Escort/defend, discovery and inspection require future capabilities; preserve completed evidence across retries |
| **Q028 A Warrant, Not a Challenge** | Present case → Aldren orders lawful detention → Cassian escapes → receive pursuit warrant | Scene state must be server-authoritative and idempotent. No hostile attack against neutral court NPC; Cassian remains alive |
| **Q029 Before the Hidden Passage** | Secure north/south civilian exits in either order → make independent evidence backup → obtain entry authorization | Keep independent pre-dungeon checkpoint. Current `object_inspected` stage aliases must become explicit `exit_secured` evidence, not passive looking |
| **Q030 The Hidden Passage** | Enter solo instance → defeat Sorn → capture Cassian alive → return authenticated dispatches | Boss victory is not necessarily a creature-kill objective. Capture is a nonlethal authorized outcome. Instance reset preserves prior evidence; no mandatory group |
| **Q031 The Commission** | Confirm lawful custody → receive limited warrant → travel home → report to Elara | **Check turn-in identity:** final NPC is Elara, not necessarily Aldren. Warrant must not confer authority over allied peoples |
| **Q032 After the Arrest (optional)** | Receive restitution request → distribute goods → notify affected household → close ledger | **RESOLVED in structured inventories (12 Nov 2026):** Q031 is now the approved prerequisite. Optional human aftermath must not gate the universal Q035 summons |
| **Q033 The Watch Restored (optional)** | Receive patrol request → inspect marker → defend road → report | **RESOLVED in structured inventories (12 Nov 2026):** Q031 is now the approved prerequisite; defense still requires future event support |
| **Q034 A Village Still Working (optional)** | Receive relief task → record priority choice → deliver to selected location → report | **RESOLVED in structured inventories (12 Nov 2026):** Q031 is now the approved prerequisite; choice persistence and delivery still require planned capabilities |

## Evidence and stage design rules

- **Stable IDs:** Q018–Q034 are campaign references, not approved runtime IDs. Original H-series editorial IDs remain candidates until the crosswalk is reviewed.
- **Giver versus turn-in:** `contact` is insufficient. Confirm exact stable NPC templates and semantic action IDs for both roles. Q031 and Q030 particularly require separate scene/turn-in handling.
- **Stage grouping:** independent document comparisons can occur in any order within one `all` stage. A later report must be in a separate stage so an earlier interaction cannot retroactively complete it.
- **Historical evidence:** signed copies, witness statements, court warrants and boss/capture outcomes must persist as trusted facts, even if a physical document is lost.
- **Inventory:** if a quest uses cargo, ration supplies, or restitution goods, specify current possession versus historical acquisition and whether items are consumed at turn-in; no values are inferred here.
- **Rewards:** XP, currency and items remain unresolved; grant only once through the authoritative transaction.
- **Failure:** death, logout and disconnect must not destroy committed evidence. Encounter-specific progress can reset only within an explicitly retryable encounter.
- **Credit scope:** character by default. Party or instance sharing is planned, not currently available.
- **Luau:** not required for ordinary objectives. Court and boss scenes may later require scripted presentation, but their quest/world transitions remain Rust-authoritative.
- **Levels:** planning bands are not hard eligibility requirements.

## High-priority corrections proposed, not applied

1. Add **story-appropriate availability prerequisites to optional Q032–Q034**, while keeping them optional and outside the main-story prerequisite chain.
2. Disambiguate the mandatory independent witness in Q022 from optional Q025, or rewrite Q025 to avoid duplicate testimony.
3. Replace Q029's passive `inspect exit` with a verifiable secured-exit event.
4. Resolve Q031's giver/turn-in NPCs and authoritative commission evidence.
5. Refine Q018–Q022 document-inspection and NPC semantic action identifiers.

## Consolidated legacy context — 3 November 2026

**ARCHIVED source evidence / WORKING authoring detail, not additional approved objectives.** The [retired migration note](https://github.com/code-studio-au/theia-mmo/blob/450664657d81d6d93f3cf76a02c34bc81b1e34b5/docs/story/Theia/Campaigns/Human_Q018_Q034_Legacy_Pack_Migration.md) reconciled `Human_10_20_Playable_Quest_Pack.md` v0.1 and `Human_10_20_Implementation_Brief.md` v0.2. Both contained the same 17 Q018–Q034 references, not new quests. Their per-quest purposes and location bands follow the authoring dispositions above and [canonical campaign](Human_1_80_Canonical_Campaign.md); distinct unapproved site/design details remain in the [retained historical site proposals](Human_Legacy_Sidequest_Zone_Character_Archive.md#retired-production-location-and-lore-proposals). The source history preserves the full wording; these summaries do not supersede the canonical campaign.

- Retained shared design constraints: Classic manual target/facing and persistent weapon-timed autoattack; safe solo story instances; no mandatory PvP, group, profession or class gate. Hostile Dominion territory remains hostile. Ordinary travel uses roads, ships or fixed flight paths; exceptional Guardian/Kel'vari Passage requires consent and does not replace world travel.
- Recovery proposals retain safe respawn/reconnect checkpoints, reset hostile encounters, and preserve verified evidence and meaningful choices. Q034 relief can have persistent acknowledgements without a main-story fork. These are authoring requirements, not claims of implemented support.
- The old brief's sample-preservation choice belongs to **Q043**, not Q018–Q034. Its later dialogue effects are a cross-chapter continuity requirement, not a new objective here.
- The old minimum-level 10/15 and chapter unlock claims are pacing proposals, not approved eligibility. `HUMAN_Q###_DONE` is a draft alias, not a runtime ID. Repeated “accept → inspect → complete → report” text is generic scaffolding, not four verified gameplay stages.
- Outstanding authoring inputs include coordinates, spawn/actor IDs, interaction volumes, combat stats, XP/loot and localization. The sources supply no additional approved dialogue or production-ready data.
- Retain QA coverage for optional independence, scene replay, nonlethal arrest, boss reset, travel availability, reconnect, duplicate rewards, class-neutral completion and accessibility. A dialogue opening is not a verified report; one event must not cascade through later stages.

## Implementation handoff

Do **not** export this review as executable JSON or change Codex's engine code. After editorial approval, apply decisions to the **single current manifest** and update the canonical narrative only where needed. Before deployment, validate the new catalogue registry and content-version migration policy against `docs/QUEST_ENGINE.md`.
