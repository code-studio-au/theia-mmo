# Human Q001–Q017 — Rust Quest Authoring Review

**Status: content-authoring review; not an executable catalogue.** Narrative authority: [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md). Working inventory: [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json). Runtime contract: [Quest Engine Agreement](../../../QUEST_ENGINE.md). Codex owns runtime and persistence integration.

## Hard boundaries

- Keep 184 Q-series entries unchanged while reviewing; editorial H/HS provenance is still candidate-only. Never allocate an H-series runtime ID from title similarity.
- No permanent 32-quest production limit. Exportable packs and cross-pack registry are planned, not implemented.
- Saved journals currently bind exact catalogue bytes. Do not publish executable campaign content before version compatibility and migration rules are qualified.
- Native Rust catalogue currently accepts only `kill` and `npc_interaction`, with stable NPC and semantic action IDs. Other objective kinds remain authored intentions, not silently downgraded to NPC clicks.
- Standard quests require no Luau. XP values, actor IDs, creature IDs, item consumption and true eligibility gates must be reviewed before export.
- Existing Q001–Q017 stage list is a **planning order**, not a mandatory one-stage-per-interaction runtime structure.

## Quest-by-quest disposition

The final column preserves **WORKING proposals from ARCHIVED source evidence**, not approved additions to the single manifest or implemented behavior. Alternative outcomes, additional evidence steps and recovery choices require editorial acceptance before a manifest change.

| Quest | Proposed stage grouping / content design | Current-engine feasibility | Open review | Preserved legacy detail / recovery proposals |
|---|---|---|---|---|
| Q001 Before the Next Caravan | Stage A: ask Darian, Anwen and Brenna in any order (`all`); Stage B: report to Elara | Native semantic NPC interactions, once verified | Four NPC IDs and four distinct action IDs; preserve H1-01 candidate | Elara at Millhaven watch post introduces Darian at militia drill ground, Anwen at healing house and Brenna at merchant yard. Three visits are **unordered**, followed by report to Elara; yellow markers for eligible contacts **Recovery/QA proposal:** Per-character one-time conversation credit, safe logout and abandon/reaccept policy to be approved; no NPC interaction blocked by other players |
| Q002 The Road Patrol | Inspect two distinct signs; find worker; defend rescue; report to Darian | Partial: NPC report native; inspection and defense planned | Rescue credit must be a trusted encounter fact; retry only failed encounter | Darian sends player to two separate predator signs, a delayed worker and a low-risk wildlife rescue. Predator uses Classic-facing/range/leash behavior **Recovery/QA proposal:** Worker waits at safe checkpoint; no permanent death, village chain-pull or irrelevant kill credit |
| Q003 What the Healing House Needs | Recover sealed parcel; deliver to Anwen; witness treatment | Planned item/interaction and scene evidence | Decide physical quest item vs persistent evidence token, and consumption at delivery | Anwen's sealed parcel is delayed in guarded roadside store. **Inspect seal and contents** before delivery; observe patient treatment without claiming a known cause for unusually effective healing **Recovery/QA proposal:** Object evidence persists separately from inventory; parcel reclaimable; no duplicate turn-in or profession/class gate |
| Q004 The Missing Wheels | Inspect marker and wagon; interview teamster; report to Brenna | Partial: semantic conversations native | Inspection and witness access require authorized world facts | Track last waymarker, wheel marks, abandoned wagon and **damaged cargo frame**; interview surviving teamster on sheltered trail; report to Brenna and Elara **Recovery/QA proposal:** Ordinary wildlife/bandits may be avoided where allowed; teamster checkpoint safe; selected goods missing, foreshadowing Iven Rusk |
| Q005 The Fence at Dawn (optional) | Inspect fence; defend livestock; collect repair materials; repair | Planned interactions/defense | Preserve independence from main chain | Farm steward or Elara directs fence inspection, livestock defense, ordinary materials and repairs; tutorial for facing/autoattack without crafting requirement **Recovery/QA proposal:** Defense wave retryable; livestock not permanently lost |
| Q006 Seed Before Winter (optional) | Count sacks and compare ledger in either order; correct duplicate with Tomas | Partial: Tomas conversation native | Keep ledger correction as historical fact, not possession | Tomas's seed ledger has a **duplicate requisition**, not a supernatural plot. Two fair restitution approaches affect thanks dialogue only **Recovery/QA proposal:** Record credit persists; choice persistence requires future capability |
| Q007 A Cart That Can Move (optional) | Inspect axle; gather salvage; repair cart | Planned inventory/object use | Decide whether iron is consumed by repair; no profession gate | Heddan's cart requires axle inspection, salvage iron, repair bench and **cart movement test** **Recovery/QA proposal:** Repair retryable, no profession requirement; iron consumption requires explicit decision |
| Q008 The Healer's Ledger (optional) | Interview two patients in either order; deliver herbs; file observations | Partial: interviews native | Herb acquisition vs possession and consumption unresolved; no magical explanation | Anwen asks two consenting patients about recovery, receives herbs, compares dated notes and files ledger **Recovery/QA proposal:** No timed deaths or missable testimony; no confirmed mana explanation |
| Q009 An Unusual Buyer | Locate camp; defeat defenders; recover cargo and inspect order/seal; report | Partial: kill native, other events planned | Cargo must not be permanently losable story evidence | Find Iven Rusk's camp using teamster evidence; **scout guards/routes** before engaging. Hostile guards can be disabled or defeated; Iven may surrender or be defeated but **must not be canonically killed**. Recover selected cargo, buyer order and damaged royal seal **Recovery/QA proposal:** Camp perimeter retry; case evidence persists; hostile patrols use social aggro, melee/ranged behavior and leash |
| Q010 A Seal Without a Name | Compare fitting and workshop reference; obtain Heddan's interpretation | Partial: Heddan interaction native | Confirm giver and turn-in: Elara / Heddan is ambiguous | Elara/Heddan compare seal with legitimate fittings, record differences and refer to regional authority. Local experts cannot conclusively authenticate royal seal **Recovery/QA proposal:** No unique-item-loss softlock; giver/turn-in roles need confirmation |
| Q011 At the River Crossing | Join caravan; reach Stoneford; visit warehouse; report arrival | Partial: conversation native; travel planned | Protected PvE travel, not a fast-travel waypoint | Brenna starts a **physical caravan journey** via established road to Stoneford. River crossing and optional/mandatory road encounter need design decision; warehouse contact receives arrival **Recovery/QA proposal:** Caravan pauses/rejoins at safe checkpoints; no instant teleport or vanishing escort |
| Q012 Three Books | Read three records in any order; identify contradiction through explicit semantic action | Partial: final action may be NPC interaction if designed as such; inspection/choice planned | Wrong answer retryable; don't count greeting as correct deduction | Perrin's stock book, courier docket and warehouse tally are independent clues; identify contradictory shipment and lodge authenticated comparison **Recovery/QA proposal:** Readable clues; deduction retryable without guessing-only fail state; copies persist |
| Q013 The Office After Edric | Secure payment copy; interview Perrin; obtain introduction | Partial: conversations native | Evidence copy must survive item loss; confirm authorized official | Recover payment instruction from transfer cache, question Perrin under authority, compare ledgers and receive introduction to Mara Thorne **Recovery/QA proposal:** Perrin cannot be killed to destroy evidence; interview retryable; no premature Cassian attribution |
| Q014 The Quarry Bell (optional) | Find crew; defeat ordinary hazard; recover tally; report | Partial: kill native | Crew rescue and tally require distinct evidence | Joss's Greyfall Quarry crew is missing; trace shift bell and work route, defeat cave predators **or secure alternate safe route**, rescue workers and return tally **Recovery/QA proposal:** Rescue checkpointed; alternate safe-route design requires planned encounter capability |
| Q015 One Bridge, Two Claims (optional) | Inspect supports; hear both parties in either order; record repair priority; repair bridge | Partial: interviews native | Choice persists as local acknowledgement; both needs ultimately addressed | Stoneford reeve hears miller and merchant claims; inspect bridge, **collect repair materials**, choose work order, reopen crossing **Recovery/QA proposal:** Acknowledgement varies but both legitimate needs addressed; no permanent economic penalty |
| Q016 The Warehouse Night Watch (optional) | Review losses; patrol; defeat thieves; recover tools | Partial: kill native | Patrol and tools planned; avoid treating ordinary theft as Unmade | Brenna's warehouse watch inspects losses, patrols perimeter, intercepts **ordinary thieves** and recovers tools **Recovery/QA proposal:** Patrol/aggro tutorial, retryable watch; not an Unmade plot |
| Q017 The Long Way Home (optional) | Prepare supplies; travel with caravan; clear obstruction; deliver | Partial: conversation native | Escort/region objectives planned; independent of main-story progression | Elara or Brenna prepares recovered supplies for a **road caravan home**; player clears obstruction and delivers to Millhaven **Recovery/QA proposal:** Rejoin at safe checkpoint after leaving route; no endless escort reset |

## Q001 proposed native-only shape

This is **illustrative authoring guidance**, not valid executable JSON because actual stable NPC and semantic interaction IDs have not been verified.

- Quest: Q001; editorial candidate H1-01; runtime ID **unassigned**.
- Giver and turn-in: Elara, stable NPC ID **unassigned**.
- Prerequisites: none; level 1 is a narrative starting band, not a hard eligibility gate.
- Stage 1: three distinct semantic NPC interactions (`ask_about_road`, `ask_about_healing_house`, `ask_about_caravan`), `completion=all`, any order.
- Stage 2: `report_findings` interaction with Elara, `completion=all`.
- Credit: per character, historical trusted interaction receipts; repeat once.
- Reward: XP **TBD**, no invented item/currency grant.
- Failure: committed conversations persist across death/reconnect; no encounter reset. Abandonment behavior **unapproved**.
- Luau: not needed. Semantic interaction IDs are proposals and require Codex/content registry confirmation.

## Explicit content-versus-runtime blockers

1. **NPC/object identity:** editorial aliases such as `anwen` and `medical_parcel` are not installed content IDs.
2. **Objective types:** `object_inspected`, `object_interacted`, `region_discovered`, `region_reached`, `encounter_defended`, `scene_observed`, `dialogue_choice_recorded`, `patrol_completed` and generic `encounter_defeated` cannot be presumed equivalent to native `kill` or `npc_interaction`.
3. **Quest items:** do not assume collecting an object means holding it at turn-in. Each item needs `historical_acquisition` versus `current_possession`, and an explicit consumption policy.
4. **Version migration:** the current exact-byte catalogue binding makes even safe content evolution an explicit compatibility concern; wait for Codex's registry/version migration contract.
5. **Cross-pack prerequisites:** planned registry must resolve IDs across packs; don't manufacture a fixed 32-quest split.
6. **Authorship versus deployment:** no Q001–Q017 definition should be imported until the content validator, actor IDs and runtime authority are ready.

## Consolidated legacy gameplay context — 3 November 2026

The [retired detail-migration note](https://github.com/code-studio-au/theia-mmo/blob/06c8f0d18b7362ce0e30b65738e121c1c2b50e5d/docs/story/Theia/Campaigns/Human_Q001_Q017_Gameplay_Detail_Migration.md) preserved `Human_Levels_1_10_Playable_Quest_Pack.md` v0.1. Its 17 quest-specific gameplay and recovery records are now in the final column of the existing matrix above, rather than a separate quest-detail table.

Teach Classic manual targeting, facing, weapon-timed autoattack, interaction, objective tracking, safe respawn and physical road travel. Millhaven's ordinary problems are not automatically Unmade conspiracies. Optional regional work supplies experience independently and is not required by the main-story chain. Historical minimum levels 1/5 are pacing proposals, not hard eligibility gates.

Parcel-seal inspection, wheel marks, cart movement tests, scouting, repair materials and other extra details remain **WORKING**, not automatic additions to the 684-stage working manifest. The finale's proposed level-80 gate remains a separate OPEN question. Never infer native escort, inspection, choice, inventory or encounter-outcome support from this prose.

Zone positions, spawn densities/groups, NPC schedules/archetypes, object IDs, enemy tuning, dialogue/localization, class balance, journal UI, save/load, reconnect and automated completion tests remain unresolved alongside the identity/reward/persistence decisions above. Playtesting must establish balance and numerical values; none are approved here.

## Next authoring pass

Refine Q018–Q034 using the same evidence-first approach; then reconcile dialogue stage conditions and the first solo dungeon. Update the **single manifest** only after review decisions are accepted; this file records recommendations and does not create a second quest inventory.
