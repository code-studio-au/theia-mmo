# Theia Human 1–80 — Quest Production Specification v0.2

**Status: implementation authoring draft, not production-ready or playtested.** Companion to [canonical campaign](Human_1_80_Canonical_Campaign.md), which remains authoritative for story decisions. This register supplies individual locations, contacts, objectives and lore for all 184 editorial quests, including previously underspecified late-game entries. The quest titles and canonical outcomes are unchanged.


## Playable quest packs (complete editorial coverage)

The following packs collectively cover all Q001–Q184, providing staged gameplay proposals, narrative intent, completion conditions and recovery behavior. They are **not** final implementation data; exact coordinates, dialogue, encounters, XP and item rewards still require development.

- [Levels 1–10](Human_Levels_1_10_Playable_Quest_Pack.md)
- [Levels 10–20](Human_10_20_Playable_Quest_Pack.md)
- [Levels 20–30](Human_20_30_Playable_Quest_Pack.md)
- [Levels 30–35](Human_30_35_Playable_Quest_Pack.md)
- [Levels 35–45](Human_35_45_Playable_Quest_Pack.md)
- [Levels 45–55](Human_45_55_Playable_Quest_Pack.md)
- [Levels 55–70](Human_55_70_Playable_Quest_Pack.md)
- [Levels 70–80](Human_70_80_Playable_Quest_Pack.md)
- [Levels 80 finale](Human_80_Finale_Playable_Quest_Pack.md)

## Runtime contract and content acceptance criteria

- **Quest states:** locked → available → accepted → objectives-complete → turned-in; optional abandoned and retryable-instance states. Each objective has stable server-authoritative credit and idempotent turn-in. The final story chapter unlocks at character level 80, not from XP gained in the finale.
- **Gating:** first story mission of each five-level chapter requires minimum level and previous chapter completion. Subsequent critical quests require the previous main-story step unless an explicit branch is authored. Optional/local missions must not block critical plot. The final ten missions are post-cap.
- **Solo combat:** all critical bosses and story dungeons must be completable solo with the approved Classic-style targeting, manual facing and autoattack; group scaling is optional. Boss deaths, key evidence and cinematics must be checkpointed and retryable without losing unique quest objects.
- **Travel:** ordinary ships and fixed-route flight paths plus overland travel. Guardian gate is restricted, low capacity and requires Kel'vari consent. No normal sea/foot arrival at Shroud before its dissolution. Enemy Dominion authorities remain hostile; covert archive raids are instanced or PvE-safe.
- **Evidence:** immutable case copy awarded on objective completion, not dependent on inventory; multiple independently verified sources for allegations; no critical evidence destroyed by optional player choices.
- **Choices:** preserve/destroy sample and trust decisions modify future dialogue and exhibits, not essential quest chains or canonical endings.
- **World presentation:** Veilbreaker succeeds only in its story instance at 65–70; Shroud begins dissipating immediately; Seven immediately freed; Nhal's presence grows gradually. Shared-world phasing and quest-stage presentation require implementation design and must not overwrite another player's progression.
- **Rewards:** assign XP and items against chapter interval budget in a balancing pass. Do not assign fictitious final values. Optional quests, open-world activities and dungeons must provide sufficient non-story XP to reach level 80.
- **Each quest's remaining implementation checklist:** exact map IDs/coordinates, NPC spawn IDs, faction/reputation rules, objective counters and interaction volumes, dialogue/localization IDs, combat AI and spawn tables, item IDs, reward XP, cutscene triggers, checkpoint and logout recovery, party behavior, and automated QA tests.

## Quest definitions

### 1–5 — The First Road

#### Q001 — Before the Next Caravan

- **Location:** Millhaven, farms and roads
- **Primary contact:** Elara
- **Player objective:** Meet the militia, healer and merchant; identify the immediate road and supply problems. Elara issues a local service assignment.
- **Lore / completion purpose:** Local missing caravan, Anwen's healing, everyday service.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q001_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q002 — The Road Patrol

- **Location:** Millhaven, farms and roads
- **Primary contact:** Darian
- **Player objective:** Scout predator signs and protect a delayed worker on the road; report conditions rather than an arbitrary kill count.
- **Lore / completion purpose:** Local missing caravan, Anwen's healing, everyday service.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q002_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q003 — What the Healing House Needs

- **Location:** Millhaven, farms and roads
- **Primary contact:** Anwen
- **Player objective:** Recover the delayed medical parcel from a safe roadside store and deliver it; establish why the caravan matters.
- **Lore / completion purpose:** Local missing caravan, Anwen's healing, everyday service.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q003_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q004 — The Missing Wheels

- **Location:** Millhaven, farms and roads
- **Primary contact:** Brenna
- **Player objective:** Trace the caravan's last confirmed stop, search its abandoned wagon and locate a surviving teamster.
- **Lore / completion purpose:** Local missing caravan, Anwen's healing, everyday service.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q004_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q005 — The Fence at Dawn

- **Location:** Millhaven, farms and roads
- **Primary contact:** Elara
- **Player objective:** Inspect broken pasture fencing, clear nearby predators and report the safe route; tutorial for manual facing and autoattack.
- **Lore / completion purpose:** Local missing caravan, Anwen's healing, everyday service.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q005_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q006 — Seed Before Winter

- **Location:** Millhaven, farms and roads
- **Primary contact:** Tomas
- **Player objective:** Count seed reserves and resolve a duplicate requisition without combat; unlock independent local quest board.
- **Lore / completion purpose:** Local missing caravan, Anwen's healing, everyday service.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q006_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q007 — A Cart That Can Move

- **Location:** Millhaven, farms and roads
- **Primary contact:** Heddan
- **Player objective:** Recover ordinary iron from a safe salvage site and repair a wheel assembly; teach gathering without profession gate.
- **Lore / completion purpose:** Local missing caravan, Anwen's healing, everyday service.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q007_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q008 — The Healer's Ledger

- **Location:** Millhaven, farms and roads
- **Primary contact:** Anwen
- **Player objective:** Interview patients, deliver supplies and document unexpectedly improved healing without claiming a cause.
- **Lore / completion purpose:** Local missing caravan, Anwen's healing, everyday service.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q008_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 5–10 — Beyond Millhaven

#### Q009 — An Unusual Buyer

- **Location:** Millhaven, Stoneford, Greyfall Quarry
- **Primary contact:** Elara
- **Player objective:** Disrupt Iven Rusk's camp, recover selected cargo and a damaged royal-sealed document; record the buyer's instructions.
- **Lore / completion purpose:** Cargo and royal records point to Mara.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q009_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q010 — A Seal Without a Name

- **Location:** Millhaven, Stoneford, Greyfall Quarry
- **Primary contact:** Elara / Heddan
- **Player objective:** Secure the document; compare a fitting with legitimate workshop stock; establish that authentication requires other authority.
- **Lore / completion purpose:** Cargo and royal records point to Mara.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q010_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q011 — At the River Crossing

- **Location:** Millhaven, Stoneford, Greyfall Quarry
- **Primary contact:** Brenna
- **Player objective:** Accompany a caravan's arrival in Stoneford and check the regional warehouse; complete travel through a protected PvE route if desired.
- **Lore / completion purpose:** Cargo and royal records point to Mara.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q011_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q012 — Three Books

- **Location:** Millhaven, Stoneford, Greyfall Quarry
- **Primary contact:** Mara's local correspondent
- **Player objective:** Compare Perrin's stock book, courier docket and warehouse tally; identify one shipment that cannot match all three.
- **Lore / completion purpose:** Cargo and royal records point to Mara.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q012_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q013 — The Office After Edric

- **Location:** Millhaven, Stoneford, Greyfall Quarry
- **Primary contact:** Elara
- **Player objective:** Recover a payment instruction from Perrin's transfer cache and interview him under regional authority; obtain Elara's introduction to Mara.
- **Lore / completion purpose:** Cargo and royal records point to Mara.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q013_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q014 — The Quarry Bell

- **Location:** Millhaven, Stoneford, Greyfall Quarry
- **Primary contact:** Joss
- **Player objective:** Locate missing quarry workers, defeat cave predators and return the shift tally.
- **Lore / completion purpose:** Cargo and royal records point to Mara.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q014_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q015 — One Bridge, Two Claims

- **Location:** Millhaven, Stoneford, Greyfall Quarry
- **Primary contact:** Stoneford reeve
- **Player objective:** Inspect bridge damage and mediate transport priorities; reopen the ordinary crossing.
- **Lore / completion purpose:** Cargo and royal records point to Mara.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q015_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q016 — The Warehouse Night Watch

- **Location:** Millhaven, Stoneford, Greyfall Quarry
- **Primary contact:** Brenna
- **Player objective:** Patrol the warehouse perimeter and intercept thieves unrelated to the conspiracy.
- **Lore / completion purpose:** Cargo and royal records point to Mara.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q016_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q017 — The Long Way Home

- **Location:** Millhaven, Stoneford, Greyfall Quarry
- **Primary contact:** Elara
- **Player objective:** Escort a returned caravan segment to Millhaven; report on the changed route and survivors.
- **Lore / completion purpose:** Cargo and royal records point to Mara.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q017_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 10–15 — The King's Account

#### Q018 — An Investigator's Standard

- **Location:** Valecrown and approaches
- **Primary contact:** Mara
- **Player objective:** Present the sealed case in Valecrown; verify courier and warehouse testimony independently before receiving court access.
- **Lore / completion purpose:** Edric, Aldren, Cassian, Garrick and corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q018_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q019 — Dates in the Register

- **Location:** Valecrown and approaches
- **Primary contact:** Lysa
- **Player objective:** Check authorised office records; make an authenticated copy of the post-delegation reorganisation dates.
- **Lore / completion purpose:** Edric, Aldren, Cassian, Garrick and corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q019_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q020 — The King's Confidence

- **Location:** Valecrown and approaches
- **Primary contact:** Aldren / Cassian
- **Player objective:** Attend the audience, hear Aldren's account and accept a continuing warrant; Cassian supplies a plausible military lead.
- **Lore / completion purpose:** Edric, Aldren, Cassian, Garrick and corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q020_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q021 — A Convenient Suspect

- **Location:** Valecrown and approaches
- **Primary contact:** Garrick
- **Player objective:** Check the alleged military source against Garrick's preserved logs; establish his opposition to peace is not proof of involvement.
- **Lore / completion purpose:** Edric, Aldren, Cassian, Garrick and corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q021_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q022 — Before the Request

- **Location:** Valecrown and approaches
- **Primary contact:** Mara
- **Player objective:** Compare the authorisation time, office format and independently interviewed witness; record the contradictions without an accusation.
- **Lore / completion purpose:** Edric, Aldren, Cassian, Garrick and corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q022_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q023 — The Clerk's Queue

- **Location:** Valecrown and approaches
- **Primary contact:** Lysa
- **Player objective:** Help restore a damaged public index to learn how Valecrown's records are accessed.
- **Lore / completion purpose:** Edric, Aldren, Cassian, Garrick and corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q023_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q024 — Bread and Barracks

- **Location:** Valecrown and approaches
- **Primary contact:** City quartermaster
- **Player objective:** Investigate a legitimate ration shortage and secure a non-corrupt supply solution.
- **Lore / completion purpose:** Edric, Aldren, Cassian, Garrick and corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q024_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q025 — A Witness's Own Words

- **Location:** Valecrown and approaches
- **Primary contact:** Mara
- **Player objective:** Interview a witness without leading questions; record and compare the independent account.
- **Lore / completion purpose:** Edric, Aldren, Cassian, Garrick and corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q025_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q026 — The Walls Outside Court

- **Location:** Valecrown and approaches
- **Primary contact:** Garrick
- **Player objective:** Inspect a neglected approach road and repel opportunistic raiders, separating wartime danger from political suspicion.
- **Lore / completion purpose:** Edric, Aldren, Cassian, Garrick and corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q026_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 15–20 — The Betrayal Within

#### Q027 — The Price of an Office

- **Location:** Valecrown and Millhaven
- **Primary contact:** Mara
- **Player objective:** Secure the surviving intermediary and independently stored payments linking Cassian to the security compromise and intended killing.
- **Lore / completion purpose:** Cassian captured alive, royal follow-up warrant, local closure.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q027_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q028 — A Warrant, Not a Challenge

- **Location:** Valecrown and Millhaven
- **Primary contact:** Aldren
- **Player objective:** Present the corroborated case in private; witness lawful detention ordered and Cassian's flight. No attack on a neutral court NPC.
- **Lore / completion purpose:** Cassian captured alive, royal follow-up warrant, local closure.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q028_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q029 — Before the Hidden Passage

- **Location:** Valecrown and Millhaven
- **Primary contact:** Mara
- **Player objective:** Secure civilian exits and establish an independent evidence-copy checkpoint before Cassian's capture.
- **Lore / completion purpose:** Cassian captured alive, royal follow-up warrant, local closure.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q029_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q030 — The Hidden Passage

- **Location:** Valecrown and Millhaven
- **Primary contact:** Mara
- **Player objective:** Finish the solo story dungeon; stop Sorn, secure a dispatch copy and capture Cassian alive. Group play is optional.
- **Lore / completion purpose:** Cassian captured alive, royal follow-up warrant, local closure.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q030_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q031 — The Commission

- **Location:** Valecrown and Millhaven
- **Primary contact:** Aldren / Elara
- **Player objective:** Confirm removal from office and lawful custody; receive the follow-up warrant and report home. Close the betrayal arc without absolving Dominion.
- **Lore / completion purpose:** Cassian captured alive, royal follow-up warrant, local closure.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q031_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q032 — After the Arrest

- **Location:** Valecrown and Millhaven
- **Primary contact:** Elara
- **Player objective:** Return recovered goods and inform affected households without disclosing protected witnesses.
- **Lore / completion purpose:** Cassian captured alive, royal follow-up warrant, local closure.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q032_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q033 — The Watch Restored

- **Location:** Valecrown and Millhaven
- **Primary contact:** Darian
- **Player objective:** Lead a local patrol and protect a newly reopened road without a mandatory escort timer.
- **Lore / completion purpose:** Cassian captured alive, royal follow-up warrant, local closure.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q033_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q034 — A Village Still Working

- **Location:** Valecrown and Millhaven
- **Primary contact:** Brenna
- **Player objective:** Choose a local relief priority and complete the delivery; the royal case ends but village life continues.
- **Lore / completion purpose:** Cassian captured alive, royal follow-up warrant, local closure.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q034_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 20–25 — The Enemy Shore

#### Q035 — A Ledger Reaches the Sea

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Mara
- **Player objective:** Trace the passage ledger's shipping reference to Tidehold; register the case with alliance customs through the human warrant.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q035_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q036 — Four Peoples, One Harbour

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Corven
- **Player objective:** Attend the alliance briefing and distinguish the human commission from alliance authority; obtain a bounded inspection permit.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q036_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q037 — Short of the Manifest

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Ysra
- **Player objective:** Audit the provision depot and secure exposed goods; separate ordinary loss from the false requisition chain.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q037_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q038 — Northwatch Bound

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Nessa
- **Player objective:** Sail to Northwatch Landing, meet the garrison and Darian's support unit, and learn arrival/extraction procedures.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q038_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q039 — The Stranded Supply Party

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Darian
- **Player objective:** Rescue a Covenant supply party by a PvE coastal route; return its military route report. No hostile-player interaction required.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q039_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q040 — The Defender's Coast

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Nessa
- **Player objective:** Scout Rukhan's military posts and civilian boundaries; select a patrol-avoidance or guarded approach without entering civilian homes.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q040_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q041 — A Loss Before the Raid

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Mara's expedition clerk
- **Player objective:** Recover depot accounting from a military store; compare Nemi's serial records with a private consignment removed before the reported attack.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q041_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q042 — The Surveyor at the Signal House

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Sera
- **Player objective:** Find Sera, secure an unstable observation position and compare the small device's housing with Brynja's report.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q042_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q043 — Keep the Evidence Safe

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Sera / Grey Surveyor
- **Player objective:** Choose sealed sample preservation or safe sample destruction after copying measurements; both retain the critical technical record.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q043_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q044 — Instructions With Limits

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Brynja
- **Player objective:** Prepare separate mechanical isolators and a documented shutdown procedure; the Surveyor states its limits and emergency cutoff.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q044_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q045 — The Silent Signal

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Sera
- **Player objective:** Complete the solo dungeon; defeat Halvek and disable the Minor Resonator while protecting essential observations and available captives.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q045_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q046 — A Landing Still at War

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Nessa / Garrick
- **Player objective:** Support an NPC defensive action or equivalent instanced task; report the hidden operation separately from the continuing Dominion attack.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q046_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q047 — What the Measurements Say

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Mara
- **Player objective:** Authenticate the device report in Tidehold and connect its procurement to a restricted Aelari material reference.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q047_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q048 — A Different Kind of Healing

- **Location:** Tidehold, Northwatch, hostile Dominion military sites, Heartlands
- **Primary contact:** Anwen / Oren
- **Player objective:** Return home, document a real healing benefit and make a local ward safe; distinguish the beneficial awakening from deliberate interference.
- **Lore / completion purpose:** Minor Resonator; preliminary Brynja note before meeting her; sample choice has dialogue echoes.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q048_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 25–30 — Whose Knowledge Is Trusted

#### Q049 — The Mountain Contract

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Garrick
- **Player objective:** Establish why the altered fitting and the expedition's legitimate repair needs require a Kar Durn visit.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q049_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q050 — No Fault in the Drawing

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Brynja
- **Player objective:** Inspect her original plans and a broker's substituted specification; secure hold permission to recover a measuring assembly.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q050_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q051 — The Feeder Gallery

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Rurik
- **Player objective:** Restore maintenance access by securing its existing route and returning equipment; document a mundane pressure fault and neglected maintenance.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q051_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q052 — Grain for Iron

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Brynja / hold authority
- **Player objective:** Settle the contested shipment with verified records; secure a limited engineering commission and an inspection report.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q052_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q053 — The King's Correspondence

- **Location:** Kar Durn royal court → Sylvaren
- **Primary contact:** High King / Caelith
- **Player objective:** Obtain High King's sealed introduction and deliver Brynja's findings to Caelith.
- **Lore / completion purpose:** Diplomatic handoff to Aelari investigation.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q053_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q054 — Permission to Ask

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Caelith
- **Player objective:** Enter Sylvaren and request a limited material inquiry; hear the ward-tenders' concerns before visiting restricted sites.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q054_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q055 — The Nursery Route

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Caelith
- **Player objective:** Rescue workers and secure a woodland maintenance path threatened by local fauna and a neglected branch-ward.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q055_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q056 — The Boundary Altered

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Caelith
- **Player objective:** Compare an abandoned research ward with its approved baseline; establish deliberate alteration without blaming every forest hazard.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q056_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q057 — A Price for Observation

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Mara / Veylan
- **Player objective:** Recover Veylan's sale correspondence and obtain an accountable statement under Aelari authority; separate his grievance from his concealment.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q057_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q058 — What the Queen Will Share

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Seralyth
- **Player objective:** Present the local work and findings; receive permission for a limited sample, Caelith's assistance and the coastal lead.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q058_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q059 — Back Across the Water

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Nessa
- **Player objective:** Depart Tidehold via Northwatch for the second Dominion operation; establish the coastal extraction route.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q059_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q060 — The Rejected Component

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Sera
- **Player objective:** Retrieve Ilyss's inspection copy from a military observation post by guarded PvE assault or a stealth route; no friendly enemy dialogue.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q060_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q061 — The Courier Who Burned Both Copies

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Mara's field agent
- **Player objective:** Intercept an Unmade courier before the cached archive is destroyed; retain redundancy through an already lodged duplicate.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q061_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q062 — A Common Hand

- **Location:** Kar Durn, Sylvaren, hostile Dominion coast, Tidehold
- **Primary contact:** Mara
- **Player objective:** Compare Aelari and Vaelari records and authorise the Korrath inquiry; establish common suppression, not universal infiltration.
- **Lore / completion purpose:** High King's letter connects Dwarven to Aelari capital; Korrath holds missing knowledge.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q062_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 30–35 — The First Inversion

#### Q063 — The City That Endured

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Urodan
- **Player objective:** Arrive at Drom Kar, hear its custodial limits and secure a water-route work assignment before archive access.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q063_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q064 — Water and Memory

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Urodan / Tarek
- **Player objective:** Secure the existing maintenance route and retrieve records from the endangered store; obtain the omitted warning.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q064_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q065 — The Missing Margin

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Dromak
- **Player objective:** Authenticate the warning and obtain council permission for restricted comparison; no definitive judgment on the Vorrath exile.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q065_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q066 — A Name for the Guardians

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Sera
- **Player objective:** Receive a private account of the order and its multiracial origins; accept associate status without compulsory membership.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q066_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q067 — The Instruction Without Its Diagram

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Grey Surveyor
- **Player objective:** Compare the Regional Anchor plan with Halden's incomplete shutdown order; obtain the missing stability material.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q067_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q068 — Reach Them Before the Charge

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Sera
- **Player objective:** Reach the freight works and interrupt Halden's premature destruction attempt in a retryable story scene.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q068_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q069 — The Broken Measure

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Halden / Sera
- **Player objective:** Complete the solo dungeon, rescue the cell and isolate the Regional Anchor's supports before core shutdown.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q069_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q070 — Copies for the People at Risk

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Grey Surveyor
- **Player objective:** Receive and distribute full safety instructions to the cleared team; lodge the measured evidence independently.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q070_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q071 — Make the Measure Small

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Oth'varen / Brynja
- **Player objective:** Set up a fully isolated Minor Resonator trial with instruments, physical cutoff and reserve limits.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q071_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q072 — A Brief Reversal

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Oth'varen
- **Player objective:** Conduct the controlled inversion trial; stop at its measured duration and record both result and uncertainty.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q072_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q073 — The Safer Argument

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Halden / Sera
- **Player objective:** Review severance and bounded-inversion plans; select a provisional preference that may be changed at the final cutoff.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q073_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q074 — The Pattern Beneath

- **Location:** Drom Kar research chamber
- **Primary contact:** Sera
- **Player objective:** Compare six measured disturbances and mark common timing.
- **Lore / completion purpose:** Coordination becomes evident.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q074_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q075 — The Anchor's Defenses

- **Location:** Chainworks exterior
- **Primary contact:** Nessa
- **Player objective:** Scout guard routes and recover Harbinger orders.
- **Lore / completion purpose:** Oma'vira first identified by title.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q075_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q076 — The Harbinger's Design

- **Location:** Chainworks records vault
- **Primary contact:** Sera
- **Player objective:** Secure signed instructions and destroyed-gate schematic.
- **Lore / completion purpose:** Oma'vira named; Shroud destination established.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q076_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q077 — Against the Harbinger

- **Location:** Chainworks solo dungeon
- **Primary contact:** Sera / Nessa
- **Player objective:** Free captives, sever conditioning controls and kill Oma'vira.
- **Lore / completion purpose:** First founder permanently defeated.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q077_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q078 — The First Major Reversal

- **Location:** Drom Kar, freight works, mainland Chainworks
- **Primary contact:** Brynja / Sera
- **Player objective:** Defend the cutoff while qualified specialists invert the captured Major Anchor; record bounded success and maintenance duties.
- **Lore / completion purpose:** Oma'vira killed; bounded Major Anchor inversion; her destroyed Passage gate points to Shroud.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q078_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q079 — The Harbinger's Secret

- **Location:** Chainworks archive
- **Primary contact:** Sera
- **Player objective:** Recover gate destination record and verify gate is unusable.
- **Lore / completion purpose:** Evidence of Shroud access.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q079_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 35–40 — The Gate They Kept

#### Q080 — Aftermath of Inversion

- **Location:** Drom Kar laboratory
- **Primary contact:** Sera
- **Player objective:** Show the captured Passage records and request explanation.
- **Lore / completion purpose:** Guardian secrecy challenged.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q080_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q081 — The Gate They Kept

- **Location:** Guardian sanctuary
- **Primary contact:** Sera
- **Player objective:** Present proof of the destroyed gate and demand disclosure.
- **Lore / completion purpose:** Secret Guardian Passage acknowledged.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q081_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q082 — The Price of Secrecy

- **Location:** Restricted archive
- **Primary contact:** Guardian archivist
- **Player objective:** Inspect gate maintenance and consent logs.
- **Lore / completion purpose:** Secrecy protected Kel'vari.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q082_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q083 — The Route That Never Closed

- **Location:** Drom Kar, hidden Guardian gate, Lantern Redoubt, Kel'vari homeland
- **Primary contact:** Tarek / Dromak
- **Player objective:** Survey the separate low-capacity Passage, test a small-group return path and deliver an expedition proposal to Corven.
- **Lore / completion purpose:** Guardians have limited secret gate; Kel'vari nearly erased from history; sovereign consent; Redoubt is longstanding.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q083_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q084 — Permission to Enter

- **Location:** Guardian sanctuary
- **Primary contact:** Sera / Kel'vari custodian
- **Player objective:** Obtain permission for a small supervised delegation.
- **Lore / completion purpose:** Kel'vari consent mandatory.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q084_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q085 — The Hidden Crossing

- **Location:** Restricted Guardian Passage
- **Primary contact:** Tarek
- **Player objective:** Cross with approved expedition and verify return capacity.
- **Lore / completion purpose:** First Shroud entry.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q085_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q086 — Lantern Redoubt

- **Location:** Drom Kar, hidden Guardian gate, Lantern Redoubt, Kel'vari homeland
- **Primary contact:** Nessa / Aru'ven
- **Player objective:** Cross in a small surveyed group and agree access/maintenance terms for this outer site; no continent-wide gate restoration.
- **Lore / completion purpose:** Guardians have limited secret gate; Kel'vari nearly erased from history; sovereign consent; Redoubt is longstanding.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q086_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q087 — Those Who Remained

- **Location:** Kel'vari settlement
- **Primary contact:** Aru'ven
- **Player objective:** Rescue work party and meet surviving Kel'vari.
- **Lore / completion purpose:** First direct revelation.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q087_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q088 — Permission to Remain

- **Location:** Lantern Redoubt
- **Primary contact:** Aru'ven
- **Player objective:** Agree boundaries, supplies and site access.
- **Lore / completion purpose:** Kel'vari sovereignty.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q088_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q089 — The Broken Path

- **Location:** Ancient Passage ruins
- **Primary contact:** Tarek
- **Player objective:** Recover intrusion traces linked to Oma'vira.
- **Lore / completion purpose:** Continuing threat.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q089_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q090 — A Warning Preserved

- **Location:** Kel'vari record store
- **Primary contact:** Sel'ora
- **Player objective:** Stabilize damaged warning and preserve copy.
- **Lore / completion purpose:** Sixth scholar clue.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q090_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q091 — Questions for the Sixth

- **Location:** Lantern Redoubt
- **Primary contact:** Sel'ora
- **Player objective:** Request supervised archive audience.
- **Lore / completion purpose:** Oth'varen introduction.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q091_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 40–45 — The Sixth Scholar

#### Q092 — The Custodian's Conditions

- **Location:** Oth'irun and Kel'vari archive approaches
- **Primary contact:** Sel'ora
- **Player objective:** Meet the archive party at a protected approach, accept access limits and recover its missing copyist.
- **Lore / completion purpose:** Grey Surveyor revealed as Oth'varen, sixth resistant scholar; historical records independently verified.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q092_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q093 — An Audience Earned

- **Location:** Oth'irun archive
- **Primary contact:** Sel'ora
- **Player objective:** Secure exposed records and access approval.
- **Lore / completion purpose:** Earned trust.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q093_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q094 — Six Names in the Record

- **Location:** Oth'irun and Kel'vari archive approaches
- **Primary contact:** Sel'ora / Mara
- **Player objective:** Compare the project list, verifier image and distinctive stability sequence; request the Surveyor's explanation.
- **Lore / completion purpose:** Grey Surveyor revealed as Oth'varen, sixth resistant scholar; historical records independently verified.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q094_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q095 — The Scholar Who Remained

- **Location:** Oth'irun archive
- **Primary contact:** Leth'ari
- **Player objective:** Match verifier mark to ancient instruments.
- **Lore / completion purpose:** Surveyor identity corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q095_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q096 — The Verifier's Mark

- **Location:** Oth'irun and Kel'vari archive approaches
- **Primary contact:** Leth'ari
- **Player objective:** Match a project signature with independently preserved instrument records.
- **Lore / completion purpose:** Grey Surveyor revealed as Oth'varen, sixth resistant scholar; historical records independently verified.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q096_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q097 — The Last Witness

- **Location:** Oth'irun and Kel'vari archive approaches
- **Primary contact:** Oth'varen
- **Player objective:** Hear his identity and firsthand account, then cross-check independent records; trust may be qualified rather than absolute.
- **Lore / completion purpose:** Grey Surveyor revealed as Oth'varen, sixth resistant scholar; historical records independently verified.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q097_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q098 — Echoes Before the Sundering

- **Location:** Ancient archive
- **Primary contact:** Sel'ora
- **Player objective:** Reconstruct early interference from original records.
- **Lore / completion purpose:** First Anchor project subverted.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q098_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q099 — The Six Reunited

- **Location:** Historical projection chamber
- **Primary contact:** Sel'ora
- **Player objective:** Witness six scholars' recorded dispute.
- **Lore / completion purpose:** Five accepted corruption, sixth resisted.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q099_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q100 — The Cost of Keeping Quiet

- **Location:** Oth'irun and Kel'vari archive approaches
- **Primary contact:** Sera
- **Player objective:** Rescue threatened custodians using an authorised route disclosure; preserve enclave security through restricted case access.
- **Lore / completion purpose:** Grey Surveyor revealed as Oth'varen, sixth resistant scholar; historical records independently verified.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q100_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q101 — Two Histories

- **Location:** Oth'irun archive
- **Primary contact:** Sel'ora
- **Player objective:** Compare conflicting historical copies.
- **Lore / completion purpose:** History incomplete.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q101_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q102 — The Delegation's Shadow

- **Location:** Oth'irun archive
- **Primary contact:** Mara
- **Player objective:** Compare Delegation dates with archival entries.
- **Lore / completion purpose:** Modern conspiracy lead.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q102_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q103 — The Scholar's Request

- **Location:** Oth'varen's study
- **Primary contact:** Oth'varen
- **Player objective:** Accept bounded Shattered Delegation investigation.
- **Lore / completion purpose:** Modern investigation begins.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q103_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 45–50 — The Hand Behind the War

#### Q104 — The Delegation That Failed

- **Location:** Oth'irun archive
- **Primary contact:** Mara
- **Player objective:** Collect surviving Delegation dispatches.
- **Lore / completion purpose:** Identify contradictions.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q104_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q105 — Two Accounts of One Tragedy

- **Location:** Oth'irun archive
- **Primary contact:** Sel'ora
- **Player objective:** Compare separately sourced faction records.
- **Lore / completion purpose:** Evidence diverges.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q105_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q106 — The King's Lost Adviser

- **Location:** Valecrown archive
- **Primary contact:** Mara
- **Player objective:** Recover Edric's correspondence and witness records.
- **Lore / completion purpose:** Human grievance revisited.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q106_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q107 — The Brother Who Came for Peace

- **Location:** Covenant intelligence office
- **Primary contact:** Mara
- **Player objective:** Reconstruct Great Clan chief's reluctant official attendance.
- **Lore / completion purpose:** Dominion grief understood.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q107_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q108 — The Records They Buried

- **Location:** Hostile Dominion military archive
- **Primary contact:** Mara's field agent
- **Player objective:** Infiltrate, copy suppressed security orders and escape.
- **Lore / completion purpose:** No friendly Dominion interaction.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q108_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q109 — Road to the Meeting

- **Location:** Historic delegation site
- **Primary contact:** Mara
- **Player objective:** Survey arrival routes and guard positions.
- **Lore / completion purpose:** Attack opportunity mapped.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q109_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q110 — Evidence Planted Twice

- **Location:** Ruined conference compound
- **Primary contact:** Mara
- **Player objective:** Recover planted clues and compare provenance.
- **Lore / completion purpose:** Both sides framed.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q110_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q111 — The Witness Who Disappeared

- **Location:** Neutral witness refuge
- **Primary contact:** Darian
- **Player objective:** Find survivor and protect extraction.
- **Lore / completion purpose:** Independent corroboration.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q111_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q112 — The Delegation's Second Ledger

- **Location:** Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud
- **Primary contact:** Mara
- **Player objective:** Compare human and hostile-coast contract records without relying on friendly Dominion contacts.
- **Lore / completion purpose:** Su'vareth killed; official anti-peace Great Clan delegate was High Ruler's brother; no friendly Dominion NPCs; recovered Veilbreaker orders provide actionable lead.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q112_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q113 — The False Safe House

- **Location:** Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud
- **Primary contact:** Sera
- **Player objective:** Locate Su'vareth's headquarters using three independent trails and expose its decoy.
- **Lore / completion purpose:** Su'vareth killed; official anti-peace Great Clan delegate was High Ruler's brother; no friendly Dominion NPCs; recovered Veilbreaker orders provide actionable lead.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q113_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q114 — The Veiled Hand

- **Location:** Guardian sanctuary
- **Primary contact:** Oth'varen
- **Player objective:** Present corroborated pattern identifying Su'vareth.
- **Lore / completion purpose:** Founder named.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q114_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q115 — The Witness Vault

- **Location:** Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud
- **Primary contact:** Oth'varen
- **Player objective:** Complete the solo dungeon; defeat Malrec, interrupt the prepared projection and secure the authenticated transaction archive.
- **Lore / completion purpose:** Su'vareth killed; official anti-peace Great Clan delegate was High Ruler's brother; no friendly Dominion NPCs; recovered Veilbreaker orders provide actionable lead.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q115_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q116 — Orders That Outlive Her

- **Location:** Oth'irun, Valecrown, contested delegation site, hostile Dominion military archive, Shroud
- **Primary contact:** Mara
- **Player objective:** Authenticate standing contingency orders and alert affected sites; do not imply Su'vareth survived.
- **Lore / completion purpose:** Su'vareth killed; official anti-peace Great Clan delegate was High Ruler's brother; no friendly Dominion NPCs; recovered Veilbreaker orders provide actionable lead.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q116_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q117 — A Truth Neither Side Wants

- **Location:** Valecrown court
- **Primary contact:** Aldren / Mara
- **Player objective:** Submit verified case and secure correction.
- **Lore / completion purpose:** War remains.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q117_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q118 — The War That Remains

- **Location:** Oth'varen sanctuary
- **Primary contact:** Oth'varen
- **Player objective:** Translate Su'vareth orders and identify exact Anchor lead.
- **Lore / completion purpose:** Veilbreaker immediate lead.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q118_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 50–55 — The Warden's Approaches

#### Q119 — The Warden's Orders

- **Location:** Drom Kar
- **Primary contact:** Sera
- **Player objective:** Authenticate translated delivery orders.
- **Lore / completion purpose:** First Anchor destination.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q119_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q120 — Network Awakens

- **Location:** Covenant Regional Anchor
- **Primary contact:** Sera
- **Player objective:** Install sensors and compare abnormal readings.
- **Lore / completion purpose:** Distributed activity.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q120_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q121 — The First Breach

- **Location:** Anchor perimeter
- **Primary contact:** Nessa
- **Player objective:** Defeat Unmade installers and preserve equipment.
- **Lore / completion purpose:** Active manipulation.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q121_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q122 — A Signal Answered

- **Location:** Signal station
- **Primary contact:** Sera
- **Player objective:** Intercept Warden's relay traffic.
- **Lore / completion purpose:** Command hierarchy.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q122_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q123 — The Warden's Approaches

- **Location:** Contested mountain route
- **Primary contact:** Nessa
- **Player objective:** Scout convoys and identify supply terminus.
- **Lore / completion purpose:** Distributed logistics.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q123_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q124 — Lines of Defense

- **Location:** Covenant frontier
- **Primary contact:** Garrick
- **Player objective:** Defend outpost and evacuate civilians.
- **Lore / completion purpose:** War threat.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q124_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q125 — The Anchor Beneath

- **Location:** Buried installation
- **Primary contact:** Tarek
- **Player objective:** Recover synchronization schematics.
- **Lore / completion purpose:** Anchors amplify tear, not Binding.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q125_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q126 — A Measure of Time

- **Location:** Drom Kar research chamber
- **Primary contact:** Sera / Oth'varen
- **Player objective:** Compare activation timing across sites.
- **Lore / completion purpose:** Convergence pattern.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q126_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q127 — The Gathering Storm

- **Location:** Valecrown court
- **Primary contact:** Aldren
- **Player objective:** Present proof and obtain limited mandate.
- **Lore / completion purpose:** Political authority.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q127_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q128 — Before the Veil Breaks

- **Location:** Guardian staging camp
- **Primary contact:** Nessa
- **Player objective:** Verify extraction, supply and instrument teams.
- **Lore / completion purpose:** Operation prepared.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q128_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 55–60 — The Warden's Network

#### Q129 — The Warden Unmasked

- **Location:** Fortified relay
- **Primary contact:** Nessa
- **Player objective:** Recover signed Khor'vun orders.
- **Lore / completion purpose:** Warden confirmed.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q129_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q130 — Anchors in Concert

- **Location:** Linked relay sites
- **Primary contact:** Tarek
- **Player objective:** Measure synchronized signatures at three locations.
- **Lore / completion purpose:** Distributed operation.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q130_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q131 — Siege of Lantern Redoubt

- **Location:** Lantern Redoubt
- **Primary contact:** Aru'ven
- **Player objective:** Defend civilians and extraction gate.
- **Lore / completion purpose:** Kel'vari endangered.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q131_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q132 — The Pattern Breaks

- **Location:** Guardian station
- **Primary contact:** Sera
- **Player objective:** Locate vulnerable control relay.
- **Lore / completion purpose:** Tactical opportunity.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q132_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q133 — Into the Warden's Forge

- **Location:** Relay solo dungeon
- **Primary contact:** Nessa
- **Player objective:** Fight defenders and reach relay controls.
- **Lore / completion purpose:** Earned local victory.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q133_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q134 — The Severed Signal

- **Location:** Relay control chamber
- **Primary contact:** Sera
- **Player objective:** Disable synchronization branch and preserve data.
- **Lore / completion purpose:** Partial disruption.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q134_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q135 — Aftermath of the Break

- **Location:** Guardian staging camp
- **Primary contact:** Sera
- **Player objective:** Compare surviving signals after branch shutdown.
- **Lore / completion purpose:** Main operation persists.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q135_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q136 — What the Warden Left Behind

- **Location:** Relay archive
- **Primary contact:** Mara
- **Player objective:** Recover First Voice instructions.
- **Lore / completion purpose:** Vaer'uun lead.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q136_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 60–65 — The First Voice

#### Q137 — Records of the First Voice

- **Location:** Kel'vari archive
- **Primary contact:** Sel'ora
- **Player objective:** Retrieve Vaer'uun's early research papers.
- **Lore / completion purpose:** Pre-corruption history.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q137_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q138 — The Forgotten Investigation

- **Location:** Abandoned research facility
- **Primary contact:** Oth'varen
- **Player objective:** Recover recorded scientific debate.
- **Lore / completion purpose:** Nuanced original scholar.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q138_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q139 — The Missing Records

- **Location:** Restricted archive
- **Primary contact:** Sel'ora
- **Player objective:** Restore omitted research pages.
- **Lore / completion purpose:** Hidden history.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q139_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q140 — The Voice That Answers

- **Location:** Ancient communicator
- **Primary contact:** Vaer'uun transmission
- **Player objective:** Receive invitation and verify destination.
- **Lore / completion purpose:** Calculated manipulation of player.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q140_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q141 — Beneath the Binding

- **Location:** Ancient research site
- **Primary contact:** Sera
- **Player objective:** Inspect Binding evidence and interrupt Unmade exploit.
- **Lore / completion purpose:** Truth mixed with trap.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q141_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q142 — Truth Withheld

- **Location:** Oth'varen sanctuary
- **Primary contact:** Oth'varen
- **Player objective:** Confront Oth'varen with independently copied evidence.
- **Lore / completion purpose:** Reluctant disclosure.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q142_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q143 — Name in the Records

- **Location:** Kel'vari archive
- **Primary contact:** Sel'ora
- **Player objective:** Authenticate original reference to Nhal.
- **Lore / completion purpose:** Name of threat revealed.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q143_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q144 — The First Voice's Design

- **Location:** Unmade research site
- **Primary contact:** Sera
- **Player objective:** Disable mechanism exploiting player's access.
- **Lore / completion purpose:** Vaer'uun used investigation.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q144_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q145 — The First Voice

- **Location:** First Voice solo dungeon
- **Primary contact:** Sera
- **Player objective:** Defeat Vaer'uun and isolate apparatus.
- **Lore / completion purpose:** Founder permanently dies.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q145_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q146 — The Silence Beneath

- **Location:** Sanctuary
- **Primary contact:** Oth'varen
- **Player objective:** Compare Vaer'uun claims with historical testimony.
- **Lore / completion purpose:** Keeper's role suspected.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q146_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 65–70 — The Truth of the Binding

#### Q147 — The Weight of Silence

- **Location:** Oth'varen sanctuary
- **Primary contact:** Sera
- **Player objective:** Demand full account of Binding and Keeper.
- **Lore / completion purpose:** Truth reckoning.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q147_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q148 — The Sealed Testimony

- **Location:** Kel'vari restricted archive
- **Primary contact:** Sel'ora
- **Player objective:** Recover independent Divine War testimonies.
- **Lore / completion purpose:** Seven not equally informed.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q148_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q149 — The Missing Account

- **Location:** Ancient testimony vault
- **Primary contact:** Sel'ora
- **Player objective:** Reconstruct omitted Keeper account.
- **Lore / completion purpose:** Deliberate deception.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q149_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q150 — Guardian's Doubt

- **Location:** Historical chamber
- **Primary contact:** Guardian testimony archive
- **Player objective:** Examine Guardian's post-Binding discovery.
- **Lore / completion purpose:** Guardian deceived too.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q150_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q151 — The Seven Bound

- **Location:** Sanctuary
- **Primary contact:** Oth'varen
- **Player objective:** Assemble evidence and hear full Binding account.
- **Lore / completion purpose:** Binding contains Veil; Anchors separate.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q151_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q152 — The Cost of Containment

- **Location:** Guardian research chamber
- **Primary contact:** Sera
- **Player objective:** Compare inversion readings with Binding history.
- **Lore / completion purpose:** Possible response not yet proof.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q152_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q153 — The Warden's Final Pattern

- **Location:** Anchor monitoring network
- **Primary contact:** Tarek
- **Player objective:** Trace Khor'vun final convergence.
- **Lore / completion purpose:** Veilbreaker imminent.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q153_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q154 — Before the Veil Breaks

- **Location:** Expedition staging ground
- **Primary contact:** Nessa
- **Player objective:** Prepare evacuation and assault support.
- **Lore / completion purpose:** Stakes understood.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q154_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q155 — The Last Approach

- **Location:** Veilbreaker complex
- **Primary contact:** Nessa
- **Player objective:** Disable defenses and reach central apparatus.
- **Lore / completion purpose:** Arrival too late.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q155_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q156 — Veilbreaker

- **Location:** Veilbreaker solo dungeon
- **Primary contact:** Nessa / Sera
- **Player objective:** Kill Khor'vun and attempt cutoff after irreversible threshold.
- **Lore / completion purpose:** Veil destroyed; Binding immediately dissolves.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q156_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q157 — The Shroud Begins to Fall

- **Location:** Kel'vari frontier
- **Primary contact:** Aru'ven
- **Player objective:** Evacuate residents as Shroud starts dissipating.
- **Lore / completion purpose:** Seven immediately freed; Nhal manifests slowly.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q157_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 70–75 — The Manifestations

#### Q158 — Essence Without Form

- **Location:** Receding Shroud
- **Primary contact:** Sera
- **Player objective:** Observe manifestations and collect readings.
- **Lore / completion purpose:** Nhal distinct from Unmade.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q158_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q159 — The Lands Revealed

- **Location:** Exposed Kel'vari approaches
- **Primary contact:** Aru'ven
- **Player objective:** Escort displaced residents to shelter.
- **Lore / completion purpose:** Shroud's physical consequences.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q159_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q160 — The Architect's Hand

- **Location:** Unmade installation
- **Primary contact:** Mara
- **Player objective:** Steal Thes'var stabilization orders.
- **Lore / completion purpose:** Unmade support Nhal manifestation.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q160_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q161 — Fractured Manifestation

- **Location:** Inverted Major Anchor
- **Primary contact:** Sera / Oth'varen
- **Player objective:** Measure manifestation fracturing under inversion.
- **Lore / completion purpose:** Continuity broken, essence remains.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q161_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q162 — What Cannot Be Destroyed

- **Location:** Guardian laboratory
- **Primary contact:** Oth'varen
- **Player objective:** Compare fragments with ancient essence evidence.
- **Lore / completion purpose:** Divine essence indestructible.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q162_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q163 — The Architect Speaks

- **Location:** Ancient communicator
- **Primary contact:** Thes'var transmission
- **Player objective:** Receive Architect's invitation and challenge claims.
- **Lore / completion purpose:** Corrupted rational antagonist.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q163_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q164 — Convergence Principle

- **Location:** Distributed Anchor sites
- **Primary contact:** Brynja / Sera
- **Player objective:** Test synchronized bounded readings.
- **Lore / completion purpose:** Coordinated inversion theory.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q164_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q165 — The Guardian's Decision

- **Location:** Guardian council
- **Primary contact:** Sera
- **Player objective:** Present risks and obtain conditional approval.
- **Lore / completion purpose:** Informed decision.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q165_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q166 — The Architect's Invitation

- **Location:** Architect facility approaches
- **Primary contact:** Nessa
- **Player objective:** Map defenses and extraction route.
- **Lore / completion purpose:** Final site identified.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q166_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 75–80 — Before the Alignment

#### Q167 — Stone and Strain

- **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp
- **Primary contact:** Brynja
- **Player objective:** Inspect one inverted installation's structural supports and repair a documented failure.
- **Lore / completion purpose:** Consent, testing and independent crews; Oth'varen expects to survive; level-80 finale only after cap.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q167_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q168 — Life at the Edges

- **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp
- **Primary contact:** Caelith
- **Player objective:** Measure ecological effects near another installation and mitigate harm.
- **Lore / completion purpose:** Consent, testing and independent crews; Oth'varen expects to survive; level-80 finale only after cap.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q168_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q169 — The Route Between

- **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp
- **Primary contact:** Tarek
- **Player objective:** Travel by ship, flight path and overland to check network connections; no unlimited teleportation.
- **Lore / completion purpose:** Consent, testing and independent crews; Oth'varen expects to survive; level-80 finale only after cap.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q169_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q170 — Instructions That Survive Us

- **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp
- **Primary contact:** Oth'varen / Halden
- **Player objective:** Train independent teams and deposit redundant safety records for operation without the ancient scholar.
- **Lore / completion purpose:** Consent, testing and independent crews; Oth'varen expects to survive; level-80 finale only after cap.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q170_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q171 — The Consent of Neighbors

- **Location:** Covenant and Kel'vari councils
- **Primary contact:** Corven / Aru'ven
- **Player objective:** Obtain separate operational permissions.
- **Lore / completion purpose:** No sovereignty override.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q171_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q172 — A Failure We Can See

- **Location:** Distributed Anchors, Covenant and Kel'vari regions, Guardian camp
- **Primary contact:** Tarek
- **Player objective:** Detect the weak junction and establish an emergency correction route before final activation.
- **Lore / completion purpose:** Consent, testing and independent crews; Oth'varen expects to survive; level-80 finale only after cap.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q172_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q173 — The Last Measure

- **Location:** Guardian test chamber
- **Primary contact:** Sera
- **Player objective:** Run bounded distributed alignment test.
- **Lore / completion purpose:** Theory plausible, not guaranteed.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q173_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q174 — Before the Final Crossing

- **Location:** Staging camp
- **Primary contact:** Nessa
- **Player objective:** Verify teams and complete readiness.
- **Lore / completion purpose:** Finale requires level 80.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q174_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

### 80 finale — The Shattering of Nhal

#### Q175 — Inside the Architect's Design

- **Location:** Architect central facility
- **Primary contact:** Nessa
- **Player objective:** Infiltrate and disable defenses while preserving alignment mechanism.
- **Lore / completion purpose:** Final operation begins.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q175_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q176 — The Architect's Last Argument

- **Location:** Solo boss chamber
- **Primary contact:** Thes'var
- **Player objective:** Defeat and kill Thes'var.
- **Lore / completion purpose:** Last founder dead.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q176_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q177 — The Broken Alignment

- **Location:** Control junction
- **Primary contact:** Sera
- **Player objective:** Assess Thes'var's sabotage and misalignment.
- **Lore / completion purpose:** New lethal correction required.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q177_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q178 — The Scholar's Correction

- **Location:** Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven
- **Primary contact:** Oth'varen / Sera
- **Player objective:** Maintain evacuation and instrument stations as Oth'varen stabilises the alignment and dies after seeing it hold.
- **Lore / completion purpose:** Thes'var killed; sabotage misaligns convergence; Oth'varen realizes fatal correction only then, tells Sera and player, physically dies after alignment holds; Nhal's coherent identity shattered, divine essence persists for later expansion; Seven survive, war continues.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q178_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q179 — The Sixth Scholar's Choice

- **Location:** Convergence entrance
- **Primary contact:** Oth'varen
- **Player objective:** Witness Oth'varen tell Sera and player his choice.
- **Lore / completion purpose:** Death not foreknown.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q179_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q180 — The Seven Hold

- **Location:** Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven
- **Primary contact:** Guardian teams
- **Player objective:** Coordinate distributed support of six fields with the Seven; solo-accessible objectives, no combined player ritual.
- **Lore / completion purpose:** Thes'var killed; sabotage misaligns convergence; Oth'varen realizes fatal correction only then, tells Sera and player, physically dies after alignment holds; Nhal's coherent identity shattered, divine essence persists for later expansion; Seven survive, war continues.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q180_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q181 — The Shattering of Nhal

- **Location:** Final convergence chamber
- **Primary contact:** Sera / Oth'varen
- **Player objective:** Defend correction, complete inversion and confirm Nhal's unified identity shattered.
- **Lore / completion purpose:** Oth'varen physically dies, divine essence remains.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q181_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q182 — Silence After

- **Location:** Guardian sanctuary
- **Primary contact:** Sera
- **Player objective:** Secure survivors and verify Seven's survival.
- **Lore / completion purpose:** Mourning and victory.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q182_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q183 — A World Unbound

- **Location:** Oth'irun, Tidehold, Valecrown
- **Primary contact:** Mara / Corven
- **Player objective:** Deliver restricted final reports.
- **Lore / completion purpose:** War not resolved.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q183_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

#### Q184 — The Road Home

- **Location:** Thes'var central facility, distributed network, Oth'irun, Valecrown, Millhaven
- **Primary contact:** Elara
- **Player objective:** Post-finale epilogue: revisit Millhaven, close local relationships and record the cost and consequence of victory.
- **Lore / completion purpose:** Thes'var killed; sabotage misaligns convergence; Oth'varen realizes fatal correction only then, tells Sera and player, physically dies after alignment holds; Nhal's coherent identity shattered, divine essence persists for later expansion; Seven survive, war continues.
- **Quest flow:** Accept from contact → travel/inspect/defend/interact as specified → award objective credit and evidence copy where relevant → return to authorized turn-in. Final interaction is retry-safe and idempotent.
- **Prerequisite:** Chapter minimum level and prior main-story mission for critical-path quests; regional quest independence to be classified in the content database. **Completion flag:** `HUMAN_Q184_DONE`.
- **Failure handling:** Combat resets at encounter boundary; essential NPCs and evidence persist across death/logout; no irreversible failure from PvP or group availability.

## Remaining production blockers

1. **Classify each of 184 entries** as critical story, regional, or optional; assign explicit prerequisite graph. A linear dependency across all 184 entries is incorrect because many are optional.
2. **Resolve named map geometry** against the approved pre/post map, then set coordinates, spawn IDs, transport links, PvE/PvP boundaries, path lengths and travel timing.
3. **Write NPC dialogue and scenes** (including Sera's Guardian-gate disclosure, Vaer'uun's calculated invitation, Keeper's deception, Khor'vun's too-late defeat, Thes'var's last argument and Oth'varen's final conversation).
4. **Design encounters** with AI, abilities, facing/range requirements, combat levels, checkpoint logic, party scaling and cinematic transitions.
5. **Balance the progression economy:** reconcile the old 163-story-step prototype against this 184-entry mixed inventory; design enough independent regional quests and optional dungeons for the 25/45/20/6/4 XP target.
6. **Test critical canon transitions** in separate chapter-stage worlds, especially immediate Binding dissolution and Shroud fade at 65–70 and the final divine-essence shattering at 80.
7. **Review final authority:** this document describes implementation proposals; any conflict with the canonical campaign resolves in favor of the canonical campaign.

**Validation:** 184 unique Q IDs, 92 targeted previously incomplete records, 0 unresolved title-only records after enrichment.
