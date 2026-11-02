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

The [exact-title audit](Human_Legacy_Exact_Title_Mapping_Audit.md) records eight legacy details that may enrich existing quests or separate local content: H2-05 hold permission for measuring-assembly recovery; H2-07 limited engineering commission and inspection report; H2-12 Sera's unstable observation site and comparison with Brynja's report; H2-14 physical isolation and emergency cutoff documentation; H2-17 Aelari material/procurement reference; H4-02 surveyed group access and maintenance terms; H4-08 restricted route disclosure and enclave security; H2-08 Northwatch garrison and extraction procedure. **All are unapproved candidates**, not new stages, canon or production requirements. Do not assume these are absent from dialogue or lore simply because manifest objective wording differs.

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
