# Human Q001–Q017 — Legacy Gameplay Detail Migration

**Status: migrated editorial detail, Needs Review.** This is a companion to the **sole structured working inventory** [Human 1–80 Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json), not an alternative quest definition set. Source: `Human_Levels_1_10_Playable_Quest_Pack.md` (v0.1). Canonical story and [Codex Quest Engine](../../../QUEST_ENGINE.md) override outdated level gates or unsupported runtime claims.

## Shared early-game direction

Teach classic manual target selection, facing, weapon-timed autoattack, interaction, objective tracking, safe respawn and physical road travel. Millhaven's ordinary problems are not automatically Unmade conspiracies. Optional regional work can be done independently and supplies experience; none is mandatory for the Human main-story chain. Spawn positions, densities, NPC schedules, encounter balance and rewards require playtesting. Original chapter `minimum level 1/5` statements are **historical pacing proposals**, not authorized hard gates.

## Quest details preserved from former playable pack

| Quest | Distinct gameplay and narrative detail | Failure / QA |
|---|---|---|
| Q001 | Elara at Millhaven watch post introduces Darian at militia drill ground, Anwen at healing house and Brenna at merchant yard. Three visits are **unordered**, followed by report to Elara; yellow markers for eligible contacts | Per-character one-time conversation credit, safe logout and abandon/reaccept policy to be approved; no NPC interaction blocked by other players |
| Q002 | Darian sends player to two separate predator signs, a delayed worker and a low-risk wildlife rescue. Predator uses Classic-facing/range/leash behavior | Worker waits at safe checkpoint; no permanent death, village chain-pull or irrelevant kill credit |
| Q003 | Anwen's sealed parcel is delayed in guarded roadside store. **Inspect seal and contents** before delivery; observe patient treatment without claiming a known cause for unusually effective healing | Object evidence persists separately from inventory; parcel reclaimable; no duplicate turn-in or profession/class gate |
| Q004 | Track last waymarker, wheel marks, abandoned wagon and **damaged cargo frame**; interview surviving teamster on sheltered trail; report to Brenna and Elara | Ordinary wildlife/bandits may be avoided where allowed; teamster checkpoint safe; selected goods missing, foreshadowing Iven Rusk |
| Q005 | Farm steward or Elara directs fence inspection, livestock defense, ordinary materials and repairs; tutorial for facing/autoattack without crafting requirement | Defense wave retryable; livestock not permanently lost |
| Q006 | Tomas's seed ledger has a **duplicate requisition**, not a supernatural plot. Two fair restitution approaches affect thanks dialogue only | Record credit persists; choice persistence requires future capability |
| Q007 | Heddan's cart requires axle inspection, salvage iron, repair bench and **cart movement test** | Repair retryable, no profession requirement; iron consumption requires explicit decision |
| Q008 | Anwen asks two consenting patients about recovery, receives herbs, compares dated notes and files ledger | No timed deaths or missable testimony; no confirmed mana explanation |
| Q009 | Find Iven Rusk's camp using teamster evidence; **scout guards/routes** before engaging. Hostile guards can be disabled or defeated; Iven may surrender or be defeated but **must not be canonically killed**. Recover selected cargo, buyer order and damaged royal seal | Camp perimeter retry; case evidence persists; hostile patrols use social aggro, melee/ranged behavior and leash |
| Q010 | Elara/Heddan compare seal with legitimate fittings, record differences and refer to regional authority. Local experts cannot conclusively authenticate royal seal | No unique-item-loss softlock; giver/turn-in roles need confirmation |
| Q011 | Brenna starts a **physical caravan journey** via established road to Stoneford. River crossing and optional/mandatory road encounter need design decision; warehouse contact receives arrival | Caravan pauses/rejoins at safe checkpoints; no instant teleport or vanishing escort |
| Q012 | Perrin's stock book, courier docket and warehouse tally are independent clues; identify contradictory shipment and lodge authenticated comparison | Readable clues; deduction retryable without guessing-only fail state; copies persist |
| Q013 | Recover payment instruction from transfer cache, question Perrin under authority, compare ledgers and receive introduction to Mara Thorne | Perrin cannot be killed to destroy evidence; interview retryable; no premature Cassian attribution |
| Q014 | Joss's Greyfall Quarry crew is missing; trace shift bell and work route, defeat cave predators **or secure alternate safe route**, rescue workers and return tally | Rescue checkpointed; alternate safe-route design requires planned encounter capability |
| Q015 | Stoneford reeve hears miller and merchant claims; inspect bridge, **collect repair materials**, choose work order, reopen crossing | Acknowledgement varies but both legitimate needs addressed; no permanent economic penalty |
| Q016 | Brenna's warehouse watch inspects losses, patrols perimeter, intercepts **ordinary thieves** and recovers tools | Patrol/aggro tutorial, retryable watch; not an Unmade plot |
| Q017 | Elara or Brenna prepares recovered supplies for a **road caravan home**; player clears obstruction and delivers to Millhaven | Rejoin at safe checkpoint after leaving route; no endless escort reset |

## Implementation handoff retained

Resolve zone positions, NPC archetypes and stable IDs, semantic interaction actions, object IDs, spawn groups, enemy tuning, dialogue/localization, XP/items, class balance, journal UI, save/load, reconnect and automated completion tests. No numerical values or runtime IDs are approved here.

## Reconciliation and cleanup status

- All **17 Q001–Q017** legacy sections are represented in this migration note.
- Additional objective proposals (parcel seal, wheel marks, cart test, scouting, materials, etc.) are **not automatically inserted into the 684-stage manifest**. They need an editorial decision and then a single-manifest edit.
- Legacy level gates are explicitly superseded by the Rust authoring contract; the level-80 finale remains a separate intentional-gate question.
- Do not infer native support for escort, inspection, choices, inventory or encounter outcomes. Codex's current catalogue supports only kill and semantic NPC interaction.
- Original pack may be removed after this migration is accepted and its incoming references are checked. Git history retains the full original prose.
