# Human Q035–Q062 — Rust Quest Authoring Review

**Status: Needs Review — authoring decisions proposed, not applied to executable content.** Narrative authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Sole structured working inventory: [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json). Implementation agreement: [Quest Engine](../../../QUEST_ENGINE.md).

## Scope and engine constraints

**28 quest entries reviewed**, Q035–Q062 inclusive. The Rust quest catalogue currently supports kill and semantic NPC-interaction objectives, ordered stages and `all/any/at_least` stage completion. The multi-quest journal and durable Store are qualified independently but not yet live in the normal World/client path. Document inspection, object use, travel, defense/escort, branching choice, scripted scenes and encounter outcome objectives are **planned**, not implemented. The 32-quest development budget is not a permanent production cap; cross-pack registry and saved-definition migration remain pending.

No quest is deleted, renumbered or assigned a runtime ID in this review. Level bands remain editorial pacing notes, not hard eligibility gates.

## Q035–Q037 — Universal Covenant summons and Tidehold jurisdiction (creator-locked revision)

**Narrative purpose:** Q035 `The Covenant Calls` is the same summons from High Ruler Corven's office after the player's own race-specific level 1–20 story is complete. It does **not** require Cassian's shipping ledger, Q031, the human king's warrant, or a human-only NPC. Local delivery and acknowledgement may vary; the issuing authority and destination do not. Q036 `Four Peoples, One Harbour` is Corven's first shared briefing, establishing the sovereignty of four Covenant peoples and issuing a narrowly scoped provision-depot inspection permit. Q037 compares damaged cargo, the original manifest and a false requisition as distinct evidence sources without assuming an Unmade conspiracy is already known.

**Proposed objective/stage treatment:** Q035 requires race-specific story-completion evidence (stable runtime flags remain unassigned), local summons receipt, established travel to Tidehold and registration at Corven's office. Q036 requires attending the briefing and receiving limited inspection authorization. Q037 documentary inspections may form an unordered `all` stage, followed by Ysra's authenticated conclusion. Region travel, NPC interaction and documentary inspection remain planned capabilities; do not import unreviewed editorial identifiers as runtime IDs.

**Continuity and availability:** Q032–Q034 are optional human aftermath after Q031 and do not gate Q035. The historical H2-01 ledger story is archival provenance, not the new shared opening. Q037's legacy HS-16 candidate is thematic only; a mundane missing berth must not be conflated with a falsified requisition.

## Q038–Q041 — Northwatch and hostile Dominion coast

**Narrative purpose:** Q038 uses scheduled ship travel and military check-in; Q039 rescues a stranded party; Q040 records hostile patrol patterns while excluding civilians; Q041 obtains original pre-raid losses through hostile depot infiltration and extraction.

**Proposed objective/stage treatment:** Q039 requires an encounter-rescue completion fact and safe-checkpoint escort result, not generic kill count. Q040 should collect patrol/civilian-boundary observations without rewarding civilian attacks. Q041 requires a protected evidence copy that persists on death/retry; separate escape from document discovery.

**Continuity and availability:** Q039 is main-story despite overlapping older rescue quests. No friendly Dominion officer, forced PvP, mandatory party or instant waypoint travel.

## Q042–Q045 — Signal House and first Guardian contact

**Narrative purpose:** Q042 introduces Sera and the unidentified Elder; Q043 presents the preserve-or-destroy sample choice; Q044 uses Brynja's preliminary remote safety note; Q045 is the solo Silent Signal encounter and Minor Resonator cutoff.

**Proposed objective/stage treatment:** Q043 requires persistent choice and conditional safe procedure; both branches preserve identical essential measurements. Q044 installation/cutoff checks need authoritative interaction facts. Q045 must distinguish Halvek defeat, cutoff activation and preservation of readings; dungeon reset must not erase already authenticated evidence.

**Continuity and availability:** Elder must not be called Oth'varen in player-visible UI. Brynja is not personally met until Q049. Q043 choices and boss phases are planned beyond the current native catalogue.

## Q046–Q048 — independent regional content and evidence handoff

**Narrative purpose:** Q046 is ordinary wartime defense; Q047 authenticates Signal House readings and obtains a Kar Durn engineering lead; Q048 records healing anomalies without claiming a cause.

**Proposed objective/stage treatment:** Q046 and Q048 remain optional and cannot gate Q047 or Q049. Q047 requires readings copied from Q045, not a physical sample that could have been destroyed in Q043. Q048 may group Anwen/Oren interviews in an unordered stage.

**Continuity and availability:** Q046's availability requires Northwatch access, not simply an empty prerequisite list if its giver is physically there. Q048 needs an accessible healer/ward location, not a hard story gate.

## Q049–Q053 — Kar Durn and the High Thane's letter

**Narrative purpose:** Q049 establishes legitimate engineering contract and first personal meeting with Brynja. Q050 authenticates original and substituted drawings. Q051 is optional rescue. Q052 resolves trade issues and engineering cooperation. Q053 obtains a letter and travels to Sylvaren.

**Proposed objective/stage treatment:** Q050's two drawings can be inspected in any order; Brynja's authentication is a later stage. Q052 must distinguish a valid trade settlement from the mandatory engineering evidence: review whether this trade quest needs to be on the critical path. Q053 needs High Thane Bromund giver and Caelith final contact separately.

**Continuity and availability:** Q051 must not gate Q052. Q053's letter is an introduction only, not transferable Aelari authority. The optional H/HS provenance overlap is unapproved.

## Q054–Q058 — Sylvaren permission and ward investigation

**Narrative purpose:** Q054 secures Caelith's independent research permission. Q055 is optional local nursery assistance. Q056 compares original, altered and maintenance ward records. Q057 secures Veylan testimony with corroboration. Q058 presents evidence to Seralyth and obtains a limited inspection reference.

**Proposed objective/stage treatment:** Q056 should permit record inspection in any order before authentication. Q057 testimony and correspondence must be separate facts. Q058 requires queen's authorization and explicit permitted disclosure; final return travel may be an independent stage.

**Continuity and availability:** Q055 must not gate Q056; preserve a local ordinary hazard unrelated to the Conclave. Do not make the High Thane's letter automatically grant archive access.

## Q059–Q062 — second Dominion investigation and Korrath lead

**Narrative purpose:** Q059 confirms second hostile expedition and safe extraction. Q060 obtains a Vaelari engineer's rejected component and inspection record from a hostile observation post. Q061 intercepts Unmade courier and preserves surviving document fragment. Q062 compares Dwarven, Aelari and Vaelari records and identifies older Korrath methods.

**Proposed objective/stage treatment:** Q060's records must remain available even without friendly Vaelari contact. Q061 requires an explicit courier encounter outcome and independent surviving-copy evidence; 'defeat or stop' cannot be an ambiguous kill requirement. Q062 three reports can be compared in any order, followed by pattern recognition and the Korrath reference.

**Continuity and availability:** Do not imply Korrath collective guilt. Keep all three evidence sources independent and the next research lead bounded.

## Per-quest implementation matrix

| Quest | Key authoritative evidence | Native now / planned | Specific blocker |
|---|---|---|---|
| Q035 | Sealed shipping reference, customs registration | NPC interaction / travel and inspection planned | Tidehold permit authority and turn-in contact |
| Q036 | Distinct allied inspection permit | NPC interaction / record inspection planned | Stable delegate actions; independent sovereignty |
| Q037 | Original manifest, false requisition, cargo comparison | NPC interaction / inspection planned | Three separate record targets |
| Q038 | Military authorization, ship arrival, quartermaster check-in | NPC interaction / travel planned | Scheduled ship travel event |
| Q039 | Worker rescue and safe return | NPC interaction / rescue and escort planned | Survivor state and retry |
| Q040 | Patrol observations, civilian boundary, report | NPC interaction / scouting planned | No civilian attack credit |
| Q041 | Original pre-raid losses and successful extraction | NPC interaction / inspection and travel planned | Hostile evidence provenance |
| Q042 | Signal readings and Surveyor consultation | NPC interaction / inspection planned | Concealed Surveyor identity |
| Q043 | Sample choice and retained measurements | Choice/interaction planned | Persistent branch; same essential evidence |
| Q044 | Brynja's remote note and completed safeguards | NPC interaction / object interaction planned | No premature personal meeting |
| Q045 | Halvek defeated, Minor Resonator isolated, readings saved | Boss/object outcomes planned | Solo instance checkpoint |
| Q046 | Wartime defense and civilian evacuation | NPC interaction / defense planned | Optional access to Northwatch |
| Q047 | Authenticated readings and material supplier lead | NPC interaction / inspection planned | Must work regardless of Q043 choice |
| Q048 | Healer interviews and independent ward observation | NPC interaction / inspection planned | Optional, no causal claim |
| Q049 | Engineering contract, arrival, Brynja introduction | NPC interaction / travel planned | First Brynja meeting |
| Q050 | Signed original versus substituted drawing | NPC interaction / inspection planned | Two independent documents |
| Q051 | Maintenance crew rescue and recovered tools | NPC interaction / rescue and items planned | Optional; no main-story dependency |
| Q052 | Trade docket and authorized cooperation | NPC interaction / inspection planned | Assess critical-path necessity |
| Q053 | High Thane's introduction and Caelith receipt | NPC interaction / travel and document planned | Separate giver and turn-in NPCs |
| Q054 | Explicit Aelari research permission | NPC interaction / inspection planned | Sovereign permission not inherited |
| Q055 | Nursery workers safe and ward repaired | NPC interaction / defense/object planned | Optional access to Sylvaren |
| Q056 | Original, altered and maintenance ward records | NPC interaction / inspection planned | Three-source evidence |
| Q057 | Veylan's testimony and corroborating letters | NPC interaction / travel and inspection planned | Non-coercive interview |
| Q058 | Seralyth's limited disclosure and travel return | NPC interaction / document/travel planned | Queen's consent |
| Q059 | Second hostile expedition and extraction plan | NPC interaction / travel and scouting planned | No friendly Dominion contacts |
| Q060 | Rejected component and Vaelari inspection copy | NPC interaction / inspection and infiltration planned | Hostile evidence acquisition |
| Q061 | Courier stopped, document fragment recovered | NPC interaction / encounter and inspection planned | Define nonlethal/kill outcome |
| Q062 | Three authenticated reports and Korrath method reference | NPC interaction / inspection planned | Avoid collective guilt |

## High-priority proposed changes

1. **Q052 critical path:** Review whether an ordinary grain/iron trade settlement is necessary to unlock Q053, or whether its engineering cooperation should be granted through Q050 with Q052 moved to optional regional content. Preserve the sovereign High Thane Bromund introduction.
2. **Q046, Q048, Q051 and Q055 access:** They currently have empty quest prerequisites. That can be correct for regional side quests, but availability must respect region access, not invent mandatory storyline dependencies.
3. **Q043 choice:** Persist preserve/destroy decision; later evidence and Q047 must remain valid in either case. Item possession and consumption must be separately specified if a sample object is issued.
4. **Q045:** Treat Halvek's defeat, Resonator cutoff and evidence preservation as separate authoritative outcomes; do not collapse them into one generic kill objective.
5. **Q053:** Confirm distinct giver (High Thane Bromund) and turn-in (Caelith) contacts and the letter's evidence/possession policy.
6. **Q061:** Decide whether stopping the courier is an encounter-success fact rather than a mandatory kill, preserving the intended alternative.

## Cross-cutting authoring rules

- Stable H/HS editorial IDs are migration candidates, not approved runtime IDs. Q references remain stable editorial ordering only.
- Every NPC action requires a semantic interaction ID; greeting, requesting permission and submitting evidence are different events.
- Historical evidence (signed copies, interviews, technical readings) persists after item loss. Current inventory possession and item consumption at turn-in are separate future fields.
- XP, currency and items remain unbalanced; do not invent rewards.
- Character credit is the initial supported scope. Party sharing and phasing are planned.
- Encounter retries must not replay or erase previously committed case evidence.
- No standard quest requires Luau by default.
- Exportable content packs and version migrations must follow Codex's future registry/persistence compatibility contract, not a fixed 32-quest split.

## Consolidated legacy context — 3 November 2026

**ARCHIVED source evidence / WORKING authoring detail.** The [retired migration note](https://github.com/code-studio-au/theia-mmo/blob/eeeeabae9eb4037093f7fd39e60532d722bacd81/docs/story/Theia/Campaigns/Human_Q035_Q062_Legacy_Pack_Migration.md) compared `Human_20_30_Playable_Quest_Pack.md` v0.1 and `Human_20_30_Implementation_Brief.md` v0.2. Both covered the same 28 Q035–Q062 references and introduced no additional quest IDs. The chapter purposes and continuity requirements are retained above; location bands follow the [canonical campaign](Human_1_80_Canonical_Campaign.md), while distinct unapproved site/design details remain in the [retained historical site proposals](Human_Legacy_Sidequest_Zone_Character_Archive.md#retired-production-location-and-lore-proposals), not another quest inventory.

- The old 20/25 minimum-level assumptions, `HUMAN_Q###_DONE` aliases and generic “accept, inspect two sources, report” templates are not approved gates, runtime identities or replacements for distinct objective evidence.
- Retain Classic manual target/facing/range and weapon-timed autoattack; solo-accessible story dungeons; no mandatory PvP, class, profession or group requirement. Road, ship and fixed flight paths remain ordinary travel. Exceptional Guardian Passage requires permission and must not be prematurely revealed.
- Retry proposals reset active encounters while retaining verified evidence, persistent choices and authorized permissions. Q043's sample-preservation choice changes follow-up reactions only, not mandatory story outcome or essential evidence.
- Do not infer collective Dwarven culpability from substituted plans, or universal infiltration from the three independent Q062 sources.
- QA still covers ordered journal progress, distinct interaction actions, duplicate rewards, logout/reconnect, travel availability, spawn/leash behavior, dialogue knowledge gates and class-neutral completion. Coordinates, enemy tuning, item consumption, XP/loot, localization and client/World integration remain unresolved.
- Chapter size does not prescribe one executable catalogue or a fixed development-limit partition. No inspection, travel, defense, object use, choice or escort capability is approved or implemented merely because an old brief describes it.

## Next pass

Continue with Q063–Q079 (Korrath, Guardian scientific dispute, first inversion, Oma'vira) and check cross-chapter transitions before applying approved revisions to the sole manifest.
