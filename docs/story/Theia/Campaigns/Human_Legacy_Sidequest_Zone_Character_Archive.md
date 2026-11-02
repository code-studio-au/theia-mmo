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
