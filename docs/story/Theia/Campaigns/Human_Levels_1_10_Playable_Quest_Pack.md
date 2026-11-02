# Human Levels 1–10 — Playable Quest Pack v0.1

**Status: Needs Review / production authoring.** Companion to [canonical Human campaign](Human_1_80_Canonical_Campaign.md), [production specification](Human_1_80_Production_Quest_Specification.md), and [dependency graph](Human_1_80_Quest_Dependencies.json). Canonical quest IDs Q001–Q017 and plot outcomes are unchanged. This pack defines encounter and interaction *design*, not approved map coordinates, XP values, NPC spawn IDs or tested implementation.

## Player experience goals

- First 10 levels introduce manual targeting, facing, basic autoattack, interaction, objective tracking, safe respawn and physical travel without automatic facing or instant waypoint transport.
- Millhaven is a lived-in place, not a village in which every problem is secretly an Unmade conspiracy.
- Player can take several independent regional quests in parallel. Main story is Q001–Q004, then Q009–Q013 at the level-5 chapter unlock.
- **No artificial requirement** to complete optional Q005–Q008 or Q014–Q017 to progress the main story. Regional content, combat and exploration supply XP for chapter unlocks.
- Distances, spawn density, equipment rewards and NPC schedules require map and combat playtesting.

## Quest design details

### Q001 — Before the Next Caravan (main, minimum level 1)
**Start/turn-in:** Elara, Millhaven village watch post. **Trigger:** new Human character enters village service area.
**Dialogue intent:** Elara is busy and practical: “If you want to help, learn who needs help before you promise to solve anything.”
**Stages:** (1) Speak to Darian at militia drill ground; (2) speak to Anwen at healing house; (3) speak to Brenna at merchant yard; (4) return to Elara for local service assignment.
**Interaction:** yellow quest markers on available contacts; dialogue completion credits, not inventory items. **Failure:** none; stages resume after logout. **Lore payoff:** the next caravan is overdue, but no supernatural explanation is implied.
**QA:** talking in any order counts once; abandoning/reaccepting retains or safely resets visit credits; NPC not blocked by another player's interaction.

### Q002 — The Road Patrol (main, minimum level 1)
**Start/turn-in:** Darian, Millhaven watch post. **Prerequisite:** Q001.
**Dialogue intent:** Darian wants to prove himself; Elara expects observation rather than recklessness.
**Stages:** (1) Inspect two different predator signs; (2) locate delayed worker on road; (3) defend worker from a small wildlife encounter; (4) return a route report to Darian.
**Combat:** low-risk melee enemy archetypes with classic target/facing/range rules; predator leashes to its territory; no auto-facing assistance. **Failure:** worker returns to a safe waiting point, encounter can be retried; no permanent NPC death. **Lore payoff:** road hazards are real and mundane.
**QA:** wildlife cannot chain-pull through village; worker cannot be griefed by another player; kill credit only for relevant encounter.

### Q003 — What the Healing House Needs (main, minimum level 1)
**Start/turn-in:** Anwen, Millhaven healing house. **Prerequisite:** Q002.
**Dialogue intent:** Anwen needs supplies rather than a hero's theory about her improving healing.
**Stages:** (1) Find delayed parcel in guarded roadside store; (2) inspect seal and contents; (3) return parcel to Anwen; (4) observe one patient treated.
**Interaction:** object credit persists independently of inventory; no crafting requirement. **Failure:** parcel remains claimable on retry. **Lore payoff:** Anwen's healing is more effective than expected, with no confirmed cause.
**QA:** duplicate parcel turn-ins prevented; any character class can complete.

### Q004 — The Missing Wheels (main, minimum level 1)
**Start/turn-in:** Brenna, Millhaven merchant yard. **Prerequisite:** Q003.
**Dialogue intent:** Brenna asks for proof of the caravan's route, not indiscriminate retaliation.
**Stages:** (1) Visit last confirmed waymarker; (2) follow wheel marks to abandoned wagon; (3) inspect damaged cargo frame; (4) find surviving teamster in sheltered side trail; (5) report to Brenna and Elara.
**Encounter:** wildlife or ordinary bandits may guard the route; player can clear or evade as encounter allows. **Failure:** teamster safe at checkpoint; clues copied into case log. **Lore payoff:** selected goods are missing, leading to Iven Rusk at level 5.
**QA:** the main quest completes before level 5; next chapter remains level-gated, not time-gated.

### Q005 — The Fence at Dawn (regional optional, minimum level 1)
**Start/turn-in:** Elara or farm steward, Millhaven farms. **Stages:** inspect broken fence; defend livestock from predators; collect ordinary repair materials; complete repairs. **Encounter:** teach facing and basic autoattack; do not require a profession. **Lore:** local predator pressure unrelated to Unmade. **Failure:** repeatable defensive wave, no livestock permanently lost.

### Q006 — Seed Before Winter (regional optional, minimum level 1)
**Start/turn-in:** Tomas, seed store. **Stages:** count seed sacks; compare levy entries; find duplicate requisition; deliver corrected record. **Gameplay:** investigation/dialogue, optional gathering route. **Lore:** mundane administrative error. **Choice:** two equally fair restitution approaches change thanks dialogue only. **Failure:** records persist.

### Q007 — A Cart That Can Move (regional optional, minimum level 1)
**Start/turn-in:** Heddan, workshop. **Stages:** inspect damaged axle; retrieve safe salvage iron; interact with repair bench; test cart movement. **Gameplay:** collection and object interaction without required crafting skill. **Lore:** ordinary practical work. **Failure:** bench interaction retryable.

### Q008 — The Healer's Ledger (regional optional, minimum level 1)
**Start/turn-in:** Anwen, healing house. **Stages:** ask two consenting patients about recovery; deliver ordinary herbs; compare dated notes; lodge ledger. **Lore:** beneficial mana awakening, not explained. **Failure:** no timed patient deaths or missable testimony.

### Q009 — An Unusual Buyer (main, minimum level 5)
**Start/turn-in:** Elara, Millhaven. **Prerequisite:** Q004 and level 5.
**Dialogue intent:** Elara authorizes limited action against the outlaw camp, not revenge on everyone nearby.
**Stages:** (1) Find camp from teamster's evidence; (2) scout entry and guard routes; (3) disable or defeat Iven Rusk's hostile guards; (4) secure targeted cargo and buyer instructions; (5) recover damaged royal-sealed document; (6) return evidence.
**Combat:** basic humanoid patrols with social aggro, melee/ranged differences, clear leash/reset; Iven Rusk can be defeated or arrested through scripted surrender, not killed as a mandatory lore fact. **Failure:** retry from camp perimeter; evidence case copy persists. **Lore:** the buyer specified unusual items.

### Q010 — A Seal Without a Name (main, minimum level 5)
**Start/turn-in:** Elara and Heddan, Millhaven workshop. **Prerequisite:** Q009.
**Stages:** (1) Inspect seal; (2) compare legitimate workshop fittings; (3) record differences; (4) obtain Elara's referral to regional authority. **Lore:** document may be connected to royal administration, but local experts cannot authenticate it conclusively. **Failure:** no unique item loss.

### Q011 — At the River Crossing (main, minimum level 5)
**Start:** Brenna, Millhaven yard. **Turn-in:** Stoneford warehouse contact. **Prerequisite:** Q010.
**Stages:** (1) Meet departing caravan; (2) travel along established road; (3) inspect river crossing; (4) defend one optional or mandatory controlled road encounter (tuning pending); (5) inspect Stoneford warehouse arrival.
**Travel:** physical road journey; no teleport. **Failure:** caravan pauses at a safe checkpoint rather than disappearing; if player leaves route, rejoin at checkpoint. **Lore:** shipment investigation expands regionally.

### Q012 — Three Books (main, minimum level 5)
**Start/turn-in:** Mara's local correspondent, Stoneford warehouse. **Prerequisite:** Q011.
**Stages:** (1) inspect Perrin's stock book; (2) inspect courier docket; (3) inspect warehouse tally; (4) identify contradictory shipment entry; (5) lodge authenticated comparison. **Gameplay:** investigation with readable clue presentation, no guessing-only fail state. **Lore:** evidence of deliberate falsification. **Failure:** copies persist, puzzle can be retried.

### Q013 — The Office After Edric (main, minimum level 5)
**Start:** Stoneford correspondent. **Turn-in:** Elara or authorized regional official. **Prerequisite:** Q012.
**Stages:** (1) recover transfer-cache payment instruction; (2) question Perrin under regional authority; (3) compare with prior ledgers; (4) receive introduction to Mara Thorne. **Lore:** transfer instructions connect to post-Delegation administrative reorganization, without identifying Cassian yet. **Failure:** Perrin interview always retryable and cannot be killed to destroy the evidence.

### Q014 — The Quarry Bell (regional optional, minimum level 5)
**Contact:** Joss, Greyfall Quarry. **Stages:** locate shift bell; trace last work crew; defeat cave predators or secure alternate safe route; rescue workers; return shift tally. **Lore:** quarry safety and ordinary threats. **Failure:** rescued workers checkpointed.

### Q015 — One Bridge, Two Claims (regional optional, minimum level 5)
**Contact:** Stoneford reeve. **Stages:** inspect bridge supports; hear miller and merchant claims; recover repair materials; select work order; reopen crossing. **Choice:** acknowledgement dialogue differs, crossing repaired either way. **Failure:** no permanent regional economic penalty.

### Q016 — The Warehouse Night Watch (regional optional, minimum level 5)
**Contact:** Brenna, Stoneford warehouse. **Stages:** inspect missing stock; patrol perimeter; intercept ordinary thieves; recover stolen tools. **Combat:** social aggro and patrol routes, no Unmade reveal. **Failure:** reset watch encounter.

### Q017 — The Long Way Home (regional optional, minimum level 5)
**Contact:** Elara or Brenna, Stoneford caravan yard. **Stages:** prepare recovered supplies; travel with caravan via safe road; help at one obstruction; deliver supplies in Millhaven. **Lore:** show consequences of early player service. **Failure:** rejoin caravan at checkpoint; no endless escort loop.

## Implementation handoff checklist

For each Q001–Q017: resolve map zone/coordinates, NPC archetype and IDs, object interact IDs, spawn groups, combat difficulty, NPC dialog text and localization, reward XP/loot, class balance, quest tracking UI, save/load and disconnect recovery, and automated completion tests. None of these values is invented in this draft.

**Pacing review:** The original level-1 and level-5 chapters were designed to combine main story and independent regional work. The pack does not require all optional quests. Test the real time-to-level against the progression prototype before deciding final quest counts.
