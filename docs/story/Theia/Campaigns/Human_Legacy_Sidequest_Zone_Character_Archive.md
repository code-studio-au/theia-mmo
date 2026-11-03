# Human Legacy Ideas Archive — Side Quests, Zones and Characters

**Status: non-canonical idea bank; reviewed for retention, not approved for production.** The [current Human campaign](CURRENT_HUMAN_CAMPAIGN.md), [canonical 1–80 narrative](Human_1_80_Canonical_Campaign.md), and [184-quest authoring manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) take precedence. This archive is **not** a second active quest register. Original H/HS identifiers are historical editorial references, not runtime IDs.

## Source and migration policy

The original 92 H-series and 24 HS-series descriptions remain available in [the historical source register](../../THEIA_HUMAN_QUEST_REGISTER.md) and [draft crosswalk](Human_Quest_Editorial_ID_Crosswalk_Draft.json). [Crosswalk review](Human_Quest_Editorial_ID_Crosswalk_Review.md) identifies 64 exact-title H candidates, 28 other dispositions and distinct HS ideas that should not be collapsed merely because they share themes. This archive extracts potentially reusable concepts; it does not duplicate the entire obsolete 1–60 plot. Git history retains earlier deleted quest pages.

A candidate is **not confirmed unique** until checked against the current manifest, dialogue, encounter assets, character and zone records. If already represented, mark `accounted-for` in the crosswalk and do not generate a duplicate. If distinct, mark `sidequest-candidate` or `worldbuilding-candidate`; approve separately before assigning a new quest ID. Preserve old provenance. No historical level gates, Nhal reveal timing or alternate finale outcomes may be reinstated.

## Optional local side-quest candidates (HS-01–HS-24)

| Historical ID | Location / contact | Reusable standalone hook |
|---|---|---|
| HS-01 | Millhaven / Tomas | Seed stores, duplicate levy and village dispute |
| HS-02 | Millhaven / Heddan | Salvaged iron and public-tool repair priorities |
| HS-03 | Millhaven / Mera | Displaced family, returning worker and inn accommodation |
| HS-04 | Elderwood / Oren | Wardstone warning survey; cautious scholarship |
| HS-05 | Stoneford / Nera | Water and grain-weight milling dispute |
| HS-06 | Greyfall Quarry / Joss | Rescue trapped workers and recover survey equipment |
| HS-07 | Kar Durn / Brynja | Missing maintenance crew and hold infrastructure |
| HS-08 | Kar Durn / Rurik | Unpaid contractor and hold-law settlement |
| HS-09 | Kar Durn / forge services | Craft-service introduction through practical repairs |
| HS-10 | Sylvaren / Caelith | Nursery boundary repair and mundane threats |
| HS-11 | Sylvaren / Irielle | Injured predator and responsible hunting |
| HS-12 | Sylvaren / archive staff | Conflicting historical copies without forced certainty |
| HS-13 | Drom Kar / Urodan | Civic memorial and ordinary family records |
| HS-14 | Drom Kar / Tarek | Highland route survey and rescue |
| HS-15 | Drom Kar / craft services | Winter fuel and tool preparation |
| HS-16 | Tidehold / Ysra | Berth scheduling and stranded cargo rescue |
| HS-17 | Northwatch / Nessa | Fort maintenance, supply and optional contested-world variant |
| HS-18 | Northwatch / Darian | Return the belongings of fallen soldiers to families |
| HS-19 | Shroud approach / Aru'ven | Repair-material allocation within enclave limits |
| HS-20 | Shroud archive / Sel'ora | Recover civilian household records |
| HS-21 | Shroud frontier / Leth'ari | Protect cultivation fieldwork and instruments |
| HS-22 | Heartlands / Anwen | Consent-based conservative healing trial |
| HS-23 | Heartlands / Brenna | Caravan preparations for an expanded allied route |
| HS-24 | Millhaven / Darian | Recruit patrol training; possible optional epilogue |

These are **candidates**, not 24 newly approved quests. Check thematic overlap, travel and spoiler gates before reuse. HS-24's old H5-20 prerequisite is historical only.

## Zone and civilian-life opportunities

- **Human Heartlands:** Millhaven, Stoneford, Elderwood and Greyfall Quarry provide low-stakes economic, rescue and civic storylines without making every incident part of the Unmade conspiracy.
- **Dwarven Kar Durn:** maintenance crews, contract law and forge services provide distinct cultural gameplay.
- **Sylvaren:** ecology, nurseries, hunting ethics and contested archival evidence offer non-combat and investigation activities.
- **Drom Kar:** memorial preservation, mountain navigation and winter preparedness support Dominion civilian perspectives.
- **Tidehold and Northwatch:** harbour logistics, garrison supply and consequences of ordinary warfare.
- **Kel'vari/Shroud approaches:** repair scarcity, civilian archives and cultivation teams, without inventing passage-network miracles or unrestricted knowledge transfers.

## Character and faction hooks

Tomas, Heddan, Mera, Oren, Nera, Joss, Brynja, Rurik, Caelith, Irielle, Urodan, Tarek, Ysra, Nessa, Darian, Aru'ven, Sel'ora, Leth'ari, Anwen and Brenna are **historical contact candidates**. Check each against existing character canon before adding or modifying an NPC. Preserve civilian roles, relationships, and ordinary motivations where compatible.

## Explicitly rejected historical story outcomes

H5-03's early Nhal identification, H5-12's early Thes'var intervention, H5-14's old level-60 finale, H5-15's conflicting Keld/Voss outcome, and H5-16's local severance alternative **must not** overwrite the locked level-80 ending. Retain their original text only as historical provenance, not as active quest content.

## Next review checklist

1. Compare each HS hook and 28 non-exact H dispositions against Q001–Q184 stages and current dialogue.
2. Record `accounted-for`, `distinct-candidate`, `conflicted-retired` or `needs-review` in an approved provenance ledger; do not assume title equality proves content equality.
3. Review unique regional NPC, settlement, encounter and service details against existing lore before extracting anything else.
4. Once original source provenance is represented and all links are migrated, remove redundant Obsidian copies and navigation; keep one readable Markdown archive.
5. Keep Rust quest runtime and stable IDs unchanged until authoring approval.

## Non-exact H-series ideas requiring individual disposition (28)

These are **candidate remnants**, not newly approved quests. Current Q references are proposed comparisons only. Preserve specific details only if not already represented in the current manifest or lore.

| Legacy ID | Original title | Proposed current comparison | Editorial assessment |
|---|---|---|---|
| H4-01 | An Expedition, Not an Army | Q084 | Approval and extraction overlap, but later canon requires Kel'vari consent. |
| H4-03 | A Living Homeland | Q087 | Kel'vari worker rescue retained; historical location differs. |
| H4-04 | The Frontier Beyond the Camp | Q089 | Frontier survey overlaps; mandatory WvW training is not part of the current canonical objective. |
| H4-09 | What the Village Can Give | Q032 | Local relief theme overlaps; original royal levy detail not confirmed. |
| H4-10 | A Third Shore | Q059 | Dominion coastal journey overlaps; original staging route differs. |
| H4-11 | The Missing Survey Team | Q039 | Rescue of a supply/survey team overlaps but is in a different arc. |
| H4-12 | Stolen Under Another Flag | Q041 | Hostile Dominion depot records overlap; original named witnesses not retained. |
| H4-13 | The Inherited Contract | Q112 | Contract evidence overlaps; Shroud archive destination differs. |
| H4-14 | A Way Out for the Bound | Q115 | Captive extraction overlaps; new Su'vareth dungeon changes location and sequence. |
| H4-16 | More Than a Confession | Q114 | Corroboration of Unmade orchestration overlaps. |
| H4-17 | The King's Correction | Q117 | Royal correction retained; specifics must be reviewed. |
| H4-18 | The Answer Cassian Avoided | Q117 | Cassian custody follow-up not clearly represented as separate current quest. |
| H5-01 | A Branch Still Growing | Q120 | Anchor network survey overlaps; Chainworks expansion is no longer the same story. |
| H5-02 | The Covenant's Terms | Q127 | Limited mandate overlaps; Tidehold council details differ. |
| H5-03 | A Name Beyond the Veil | Q143 | Original Nhal naming earlier than current Q143; do not migrate old timing. |
| H5-04 | The Crossing Again | Q032 | Home relief theme overlaps, not necessarily same narrative beat. |
| H5-05 | What Edric Asked | Q106 | Edric evidence overlaps; original independent-record warrant differs. |
| H5-06 | The Coast Remembers | Q059 | Coastal return overlaps; old named military records not retained. |
| H5-07 | The Team Behind the Line | Q039 | Rescue theme overlaps but original relay context differs. |
| H5-08 | The Relay's Schedule | Q130 | Relay schedule evidence overlaps, no verified one-to-one match. |
| H5-12 | The Architect's Objection | Q163 | Thes'var early intervention differs from current invitation and final death. |
| H5-13 | Everyone's Route Home | Q154 | Extraction preparation overlaps, different chapter timing. |
| H5-14 | Cut the Outer Links | Q155 | Old level-60 finale superseded by level-80 convergence. |
| H5-15 | The Ledger and the Warden | Q156 | Keld/Voss outcome differs from Khor'vun and Veilbreaker. |
| H5-16 | The Severed Chain | Q181 | Old local severance choice cannot replace canonical Nhal shattering. |
| H5-17 | What the Instruments Prove | Q162 | Technical evidence overlaps; original branch-specific results differ. |
| H5-18 | A Corrected Record | Q183 | Postwar report overlaps; Cassian sentence not confirmed in new canon. |
| H5-19 | A Different Legacy | Q170 | Shared technical knowledge overlaps, but new finale differs. |

**Review disposition:** 5 finale/reveal conflicts are historical-only; the other 23 entries are possible worldbuilding, quest-detail or regional follow-up material pending stage-by-stage comparison. No runtime IDs or canon changes are approved by this table.

## Potentially reusable quest implementation detail (from exact-title objective comparison)

The [exact-title audit](Human_Legacy_64_Exact_Title_Objective_Comparison.md) records eight legacy details that may enrich existing quests or separate local content: H2-05 hold permission for measuring-assembly recovery; H2-07 limited engineering commission and inspection report; H2-12 Sera's unstable observation site and comparison with Brynja's report; H2-14 physical isolation and emergency cutoff documentation; H2-17 Aelari material/procurement reference; H4-02 surveyed group access and maintenance terms; H4-08 restricted route disclosure and enclave security; H2-08 Northwatch garrison and extraction procedure. **All are unapproved candidates**, not new stages, canon or production requirements. Do not assume these are absent from dialogue or lore simply because manifest objective wording differs.

## Optional-quest thematic crosswalk (not equivalence)

The historical crosswalk proposes the following current quest references as **comparison targets only**. Shared setting or theme does not prove the original side quest is implemented or redundant.

| Original optional ID | Compare with current campaign | Disposition |
|---|---|---|
| HS-01 | Q006 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-02 | Q007 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-03 | Q032 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-04 | Q055 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-05 | Q015 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-06 | Q014 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-07 | Q051 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-08 | Q052 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-09 | Q007 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-10 | Q055 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-11 | Q055 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-12 | Q101 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-13 | Q063 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-14 | Q064 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-15 | Q063 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-16 | Q037 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-17 | Q046 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-18 | Q046 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-19 | Q088 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-20 | Q093 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-21 | Q087 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-22 | Q008 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-23 | Q017 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |
| HS-24 | Q033 | Standalone candidate; verify objective, NPC, access and outcome before deduplication |

## HS-series stage-level triage — 2 November 2026

Compared the original HS descriptions with the **actual objective descriptions** in their proposed Q-series counterparts. These are editorial findings, not approval of runtime mappings. "Partial" means a similar action exists but the original standalone scenario is not demonstrably preserved.

| Legacy | Proposed Q | Stage-level assessment | Disposition |
|---|---|---|---|
| HS-01 | Q006 | Seed inventory and levy correction represented; original village dispute not explicit | Partial — keep village mediation hook |
| HS-02 | Q007 | Iron collection/repair represented, but public-tool priorities are different from cart repair | Distinct local craft candidate |
| HS-03 | Q032 | Restitution/family aid represented, but inn lodging and returning worker are different | Distinct social sidequest candidate |
| HS-04 | Q055 | Ward repair represented, but Oren's warning survey/scholarship absent from objective wording | Distinct scholarly survey candidate |
| HS-05 | Q015 | Miller involved, but water allocation/grain weight agreement differs from bridge repair | Distinct civic dispute candidate |
| HS-06 | Q014 | Quarry rescue and cave hazard represented; survey tools and optional elite variant not explicit | Partial — preserve details |
| HS-07 | Q051 | Missing maintenance crew/equipment and rescue represented | Strong overlap — verify character/route detail |
| HS-08 | Q052 | Trade dispute represented, but unpaid contractor and hold law differ | Distinct legal sidequest candidate |
| HS-09 | Q007 | Materials/repair represented, but craft-service tutorial differs | Distinct service tutorial candidate |
| HS-10 | Q055 | Nursery ward repair and hazard represented | Strong overlap — verify boundaries and nursery context |
| HS-11 | Q055 | Injured predator/hunting ethics not present in current stage descriptions | Distinct ecology sidequest candidate |
| HS-12 | Q101 | Comparison of conflicting historical accounts represented | Strong overlap — verify archival provenance |
| HS-13 | Q063 | Civic memorial/family records not represented in waterway assignment | Distinct memorial candidate |
| HS-14 | Q064 | Crew protection represented; highland route mapping and stranded survey differ | Distinct exploration candidate |
| HS-15 | Q063 | Winter fuel/tools not represented in waterway assignment | Distinct preparedness candidate |
| HS-16 | Q037 | Manifest comparison represented, but innocent scheduling error differs from forged requisition | Distinct harbour logistics candidate |
| HS-17 | Q046 | Defense and supply represented; fort maintenance/contested-world alternative not explicit | Partial — preserve optional variant idea |
| HS-18 | Q046 | Wartime losses represented, but return of personal belongings to families differs | Distinct aftermath candidate |
| HS-19 | Q088 | Access agreement represented; repair material scarcity/allocation not explicit | Distinct enclave logistics candidate |
| HS-20 | Q093 | Exposed archive recovery represented; civilian household-record emphasis worth retaining | Strong overlap — verify civilian framing |
| HS-21 | Q087 | Worker rescue and equipment return represented; cultivation instruments/team context may differ | Partial — preserve cultivation specificity |
| HS-22 | Q008 | Healer observations represented; consent-based trial and conservative reserve limits differ | Distinct healing research candidate |
| HS-23 | Q017 | Caravan preparation/travel represented; expansion to allied trade route not explicit | Partial — preserve route-expansion hook |
| HS-24 | Q033 | Patrol route and road defense represented; recruit training/epilogue timing differs | Distinct training/epilogue candidate |

**Important:** Even "strong overlap" does not authorize deletion of the original editorial ID or all unique NPC/zone details. No HS entry has been promoted to an approved playable quest. These findings justify archiving rather than blindly deleting potential regional stories.

## Non-exact H-series stage-level triage — 2 November 2026

Compared each of the 28 non-exact H-series source objectives with the authored stage descriptions of its proposed Q reference. **No canonical change or automatic mapping approval** follows. This table identifies what is clearly overlapping and what remains a possible separate encounter, side objective or lore detail.

| Legacy | Current | Review finding |
|---|---|---|
| H4-01 | Q084 | Expedition mandate overlaps; preserve multi-authority resources and extraction-plan detail |
| H4-03 | Q087 | Kel'vari worker rescue overlaps; preserve water-run and ancient-defence hazard as candidate setting detail |
| H4-04 | Q089 | Current Passage investigation differs from old NPC WvW tutorial; preserve optional frontier training idea separately |
| H4-09 | Q032 | Household relief overlaps; preserve royal levy revision/winter stores as potential civic follow-up |
| H4-10 | Q059 | Dominion voyage overlaps; storm-coast anchorage and extraction boat may be separate location details |
| H4-11 | Q039 | Personnel rescue overlaps; old pilgrimage-site and PvE-route context differs |
| H4-12 | Q041 | Depot infiltration/records overlap; Voss receipt and Shorvek objection require character/provenance check |
| H4-13 | Q112 | Contract evidence overlaps; old Shroud station archive route differs |
| H4-14 | Q115 | Captive rescue overlaps; retain independent exit/conditioning-control idea without changing Su'vareth finale |
| H4-16 | Q114 | Multi-source Unmade evidence substantially overlaps; preserve false-attribution specifics if not elsewhere recorded |
| H4-17 | Q117 | Royal correction overlaps; review related prosecutions may be a distinct consequence |
| H4-18 | Q117 | Cassian lawful-custody confrontation absent from stage wording; optional legal follow-up, no player sentencing |
| H5-01 | Q120 | Anchor measurement overlaps; old Chainworks expansion not current canon; archive technical design only |
| H5-02 | Q127 | Limited mandate overlaps; Tidehold council setting differs from royal audience |
| H5-03 | Q143 | **Conflict:** do not restore earlier Nhal name/reveal timing |
| H5-04 | Q032 | Relief overlaps; old homecoming/winter circuit could be optional local follow-up |
| H5-05 | Q106 | Edric correspondence overlaps; protected independent-record warrant is a possible procedural detail |
| H5-06 | Q059 | **Old voyage count conflicts** with current sequence; Rukhan/Nemi defence and garrison records are historical character hooks only |
| H5-07 | Q039 | Extraction rescue overlaps; relay/coastal context differs |
| H5-08 | Q130 | Relay measurement overlaps; timing tables/support assembly and solo infiltration are separate encounter ideas |
| H5-12 | Q163 | **Conflict:** old Thes'var intervention and survival cannot override canonical timing/death |
| H5-13 | Q154 | Extraction staging overlaps; medical stores and enclave boundary protections add possible operational detail |
| H5-14 | Q155 | **Conflict:** old level-60 finale must not replace irreversible activation and level-80 convergence |
| H5-15 | Q156 | **Conflict:** Voss/Keld fight must not replace Khor'vun and Veilbreaker outcomes |
| H5-16 | Q181 | **Conflict:** old severance/bounded-inversion choice must not replace Oth'varen death and Nhal identity shattering |
| H5-17 | Q162 | Technical residual-evidence theme overlaps; reject old branch-specific ending assumption |
| H5-18 | Q183 | Reporting overlaps; Cassian sentencing is **unconfirmed** and must not be promoted as canon |
| H5-19 | Q170 | Distributed technical records/training overlap; reject old ending-dependent interpretation |

**Disposition:** All 28 have been compared at the objective-description level. The five explicitly conflicting finale/reveal rows remain historical-only. Additional timing/sequence contradictions (e.g. H5-06 voyage count) must also be excluded. The remaining useful details stay in this non-canonical archive until checked against dialogue, zones and NPC records. A comparison is not proof that the old source copies are safe to delete.

## HS-01–HS-24 candidate comparison against current manifest — 3 November 2026

**Coverage: 24/24 historical optional hooks. Status: thematic comparison only; no migration or quest approval.** All 24 retain their original objectives in the draft crosswalk above. This table records the actual current candidate stages and original access conditions, so similarities do not silently erase distinct civilian content.

| Historical ID | Proposed current candidate | Actual current stages | Historical access |
|---|---|---|---|
| HS-01 | Q006 — Seed Before Winter | Count seed sacks; Check levy ledger; Correct requisition | Standalone |
| HS-02 | Q007 — A Cart That Can Move | Inspect axle; Collect iron; Repair cart | Standalone |
| HS-03 | Q032 — After the Arrest | Receive household restitution ledger; Distribute returned goods; Inform affected family without revealing protected witness; Close restitution record | Standalone |
| HS-04 | Q055 — The Nursery Route | Receive local rescue request; Reach damaged woodland ward; Rescue forest workers from ordinary hazards; Protect and repair local ward; Report local outcome | Standalone |
| HS-05 | Q015 — One Bridge, Two Claims | Inspect supports; Hear miller; Hear merchant; Choose repair priority; Reopen crossing | Standalone |
| HS-06 | Q014 — The Quarry Bell | Find missing crew; Clear cave hazard; Recover tally; Return findings | Standalone |
| HS-07 | Q051 — The Feeder Gallery | Receive maintenance incident report; Enter damaged tunnels; Secure trapped crew from hazards; Recover service equipment; Report neglected maintenance rather than conspiracy | Standalone |
| HS-08 | Q052 — Grain for Iron | Review grain and iron deliveries; Compare conflicting delivery dates; Secure engineering cooperation; Close legitimate trade dispute | Standalone |
| HS-09 | Q007 — A Cart That Can Move | Inspect axle; Collect iron; Repair cart | Standalone |
| HS-10 | Q055 — The Nursery Route | Receive local rescue request; Reach damaged woodland ward; Rescue forest workers from ordinary hazards; Protect and repair local ward; Report local outcome | Standalone |
| HS-11 | Q055 — The Nursery Route | Receive local rescue request; Reach damaged woodland ward; Rescue forest workers from ordinary hazards; Protect and repair local ward; Report local outcome | Standalone |
| HS-12 | Q101 — Two Histories | Read first post-Sundering history; Read independently preserved history; Record differences | Standalone |
| HS-13 | Q063 — The City That Endured | Receive waterway work assignment; Reach damaged channel; Secure water route; Obtain archive referral | Standalone |
| HS-14 | Q064 — Water and Memory | Receive tunnel access; Reach old store; Protect crew; Recover warning | Standalone |
| HS-15 | Q063 — The City That Endured | Receive waterway work assignment; Reach damaged channel; Secure water route; Obtain archive referral | Standalone |
| HS-16 | Q037 — Short of the Manifest | Inspect damaged delivery; Read official manifest; Compare requisition against cargo; Authenticate false requisition | Standalone |
| HS-17 | Q046 — A Landing Still at War | Receive Northwatch defense briefing; Defend supply evacuation against conventional Dominion attack; Protect wounded and equipment; Report ordinary wartime losses | Standalone |
| HS-18 | Q046 — A Landing Still at War | Receive Northwatch defense briefing; Defend supply evacuation against conventional Dominion attack; Protect wounded and equipment; Report ordinary wartime losses | After HS-17 |
| HS-19 | Q088 — Permission to Remain | Hear Kel'vari access conditions; Register research boundaries; Receive permission to remain | Story arrival or equivalent independent access |
| HS-20 | Q093 — An Audience Earned | Receive exposed record assignment; Protect threatened store; Rehouse records; Report | Story arrival or equivalent independent access |
| HS-21 | Q087 — Those Who Remained | Receive missing worker request; Reach authorized site; Protect workers; Return equipment and report | Story arrival or equivalent independent access |
| HS-22 | Q008 — The Healer's Ledger | Interview first patient; Interview second patient; Deliver herbs; File observations | After HS-03 or local healer introduction |
| HS-23 | Q017 — The Long Way Home | Prepare returned supplies; Travel with caravan; Clear obstruction; Deliver supplies | After HS-05 |
| HS-24 | Q033 — The Watch Restored | Receive restored patrol route; Inspect new patrol marker; Defend road against ordinary threats; Record road reopened | H5-20; optional epilogue |

**Notable differences requiring preservation:** HS-02's public-tool priority choice and HS-09's crafting demonstration both point to Q007's cart repair, but are distinct ideas; HS-04's Oren wardstone scholarship, HS-10's nursery boundary and HS-11's injured predator all point to Q055's worker rescue, not one interchangeable quest. HS-13's memorial and HS-15's winter reserve both point to Q063's waterway task. HS-17 fort maintenance and HS-18 recovery of fallen soldiers' belongings both point to Q046's defense, but HS-18 has a distinct memorial consequence and historical dependency. HS-19 enclave repair-material allocation differs from Q088's access permission. HS-22's consent-based healing trial is not Q008's ordinary patient ledger. HS-24's historical post-H5 epilogue condition must not become a level gate or replace Q033's road patrol. Other close thematic matches still require contact, access and consequence checks.

**Disposition:** preserve all 24 as noncanonical optional hooks pending independent editorial review. Do not add 24 active quests, merge source IDs, change the 184-quest manifest, or delete the original register solely on this stage comparison.

## Historical H/HS mapping review — consolidated

Original review findings preserved below. These are unapproved editorial dispositions, not runtime IDs or additional canonical quests.

### Original crosswalk review (historical evidence)

**Status: all legacy entries triaged; mapping approval outstanding.** This is the active migration review companion to [Human Quest Editorial-ID Crosswalk](Human_Quest_Editorial_ID_Crosswalk_Draft.json) and [Current Human Campaign](CURRENT_HUMAN_CAMPAIGN.md). Canonical story remains [Human 1–80 Campaign](Human_1_80_Canonical_Campaign.md).

## Coverage

- **92 original H-series quests:** 64 exact-title candidates, 28 other proposed dispositions. None automatically inherits a stable runtime ID.
- **24 original HS-series optional quests:** thematic candidates identified, but not proven equivalent. Several distinct HS quests map to one new Q entry, so they may represent separate regional content worth retaining.
- **184 current campaign entries:** editorial Q001–Q184 references; runtime identity and provenance approval remain open.
- **No hard level gates approved by this review.** All prior level bands and `minLevel` data remain planning annotations until a specific runtime gate is intentionally adopted.

## Canon conflicts that must not be resurrected

- **H5-03 — A Name Beyond the Veil:** proposed Q143; Original Nhal naming earlier than current Q143; do not migrate old timing.
- **H5-12 — The Architect's Objection:** proposed Q163; Thes'var early intervention differs from current invitation and final death.
- **H5-14 — Cut the Outer Links:** proposed Q155; Old level-60 finale superseded by level-80 convergence.
- **H5-15 — The Ledger and the Warden:** proposed Q156; Keld/Voss outcome differs from Khor'vun and Veilbreaker.
- **H5-16 — The Severed Chain:** proposed Q181; Old local severance choice cannot replace canonical Nhal shattering.

## Approval rules

1. Compare each legacy source objective, contact, prerequisites and consequence with current quest-stage data. Classify `reused`, `modified`, `split`, `merged`, `retired`, or `new`; record rationale and reviewer approval.
2. If several old quests suggest the same Q entry, **do not** merge editorial IDs. Either explicitly record a multi-source merge or preserve separate regional content.
3. If an old optional quest has a unique standalone purpose, keep it as a candidate for a separate regional quest; do not erase it by mapping to a similar story beat.
4. Allocate immutable runtime IDs only after provenance decisions. Existing `HUMAN_Q###_DONE` flags are temporary design aliases.
5. Validate objective evidence, current inventory possession, consumption at turn-in, NPC IDs, credit scope, rewards and retry rules against the Rust authoring contract.
6. Only then remove the old H/HS register from the active tree. Git history will preserve it, and the approved migration ledger will retain its original IDs and objectives.

## Remaining repository cleanup

- Superseded 1–60 drafts and the JavaScript prototype have already been removed.
- The root H-series register remains **temporarily** until mapping approval. The duplicated Obsidian source copy has already been retired; do not recreate it.
- Earlier playable packs and implementation briefs were retired after consolidation; continue checking any surviving notes for unique mechanics and dialogue before deletion.
- Unrelated engine and world lore documentation is outside this Human campaign cleanup.

## Verified branch provenance — 3 November 2026

GitHub comparison of `main` against `story/launch-main-story-1-60` reports the historical story branch **0 commits ahead and 534 behind** `main` at the time checked. This means no branch-only commits require cherry-picking; it does **not** establish semantic equivalence of all old story documents or authorize deletion of the protected H/HS source register. Retire the branch only after final content/backlink verification and an authorized branch deletion action. The 92 H + 24 HS objective/stage comparisons are documented, but editorial ID mappings remain unapproved.

## Five H5 historical conflict dispositions — 3 November 2026

Rechecked the five conflict-flagged source objectives and `proposed_relation` fields in the 116-entry crosswalk against the current campaign and manifest. These are **historical source dispositions, not approved one-to-one quest migrations**:

| Historical source | Proposed modern thematic reference | Safe editorial disposition |
|---|---|---|
| H5-03 | Q143 | Preserve independent network-record/testimony idea, but **retire the early Nhal-name reveal**; Q143 alone authenticates the name. |
| H5-12 | Q163 | Preserve possible observation-post evacuation/diagram-protection side hook; **do not insert an early Thes'var boss encounter** or alter his Q176 permanent death. |
| H5-14 | Q155 | Preserve worker evacuation and recoverable courier evidence as optional encounter ideas; **do not substitute the old level-60 finale** for the level-70 Veilbreaker or level-80 convergence. |
| H5-15 | Q156 | Preserve safe-retry/defence-support ideas only if independently compatible; **do not replace Khor'vun with Keld/Voss** or change the immediate Veil/Binding transition. |
| H5-16 | Q181 | Preserve bounded severance as historical research context only; **do not offer a branching finale** that bypasses Oth'varen's physical death and Nhal's coherent shattering. |

The original objectives and candidate references remain intact in the crosswalk and root historical register. No new runtime IDs, canonical quest changes, or approval of optional quest additions follows from this classification. The archive remains the destination for distinct future sidequest ideas.

## Optional HS-01–HS-12 stage comparison — 3 November 2026

The first twelve historical optional quests were compared with current manifest objectives. None is approved as a one-to-one migration. HS-01, HS-02, HS-06, HS-08 and HS-10 have partial objective overlap with Q006, Q007, Q014, Q052 and Q055. HS-07 and HS-12 have strong thematic overlap with Q051 and Q101 but still need detail review. HS-03, HS-04, HS-05, HS-09 and HS-11 retain distinctive standalone sidequest concepts not represented by the proposed current stages: displaced-family settlement, wardstone interpretation, mill allocation, craft demonstration and injured-predator hunting practice. Preserve all original source outcomes and standalone access in the historical crosswalk and idea bank. Do not assign runtime IDs or add quests automatically. Review HS-13–HS-24 next.

## HS-13–HS-24 review — 3 November 2026

The remaining twelve optional HS source objectives were checked against the proposed current manifest stages. HS-13–16 and HS-18–19 preserve distinctive local tasks not present in those stages. HS-22–24 also retain distinct optional healing, caravan and patrol-training ideas. HS-17, HS-20 and HS-21 partially overlap current defense, archive and worker-protection quests but retain additional local context. Historical dependencies and enclave permissions remain source evidence, not approved runtime prerequisites. No one-to-one HS mappings, new quests or runtime IDs were approved.
