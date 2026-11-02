# Human Q080–Q103 — Legacy Pack and Brief Reconciliation

**Status: Needs Review; editorial source reconciliation, not executable content.** The [Human 1–80 Canonical Campaign](Human_1_80_Canonical_Campaign.md) governs narrative, the [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the sole structured working inventory, and [Codex Quest Engine](../../../QUEST_ENGINE.md) governs implemented runtime capabilities. Compare with [Q080–Q103 Rust Authoring Review](Human_Q080_Q103_Rust_Authoring_Review.md).

## Source inventory and disposition

- `Human_35_45_Playable_Quest_Pack.md` v0.1: all **24 Q080–Q103** entries, named location bands, player-purpose summaries and generic recovery rules.
- `Human_35_45_Implementation_Brief.md` v0.2: the same **24 Q080–Q103** entries, mostly generic stage templates, broad QA and build acceptance criteria.

Both contain historical minimum-level 35/40 assumptions and draft `HUMAN_Q###_DONE` completion aliases. **Neither is authoritative for runtime eligibility or permanent IDs.** Generic “accept → inspect → report” templates do not override the current manifest's actual stage targets.

## Chapter-specific details retained

| Quest | Source purpose and setting | Current interpretation |
|---|---|---|
| Q080 Aftermath of Inversion | Sera, Drom Kar laboratory; present captured Passage records | Challenge Guardian secrecy without prematurely revealing the surviving gate |
| Q081 The Gate They Kept | Sera acknowledges a surviving restricted Passage | Disclosure is an independently recorded fact; Sera's choice can be criticized |
| Q082 The Price of Secrecy | Examine maintenance history and custodial consent | Player-facing text must conceal Kel'vari identity until Q086 |
| Q083 The Route That Never Closed | Optional survey of low-capacity restricted gate | Gate cannot be accessible before Q081/Q082 disclosure and permission |
| Q084 Permission to Enter | Petition Guardians and independent custodian | Custodian is not named Kel'vari before first-contact reveal |
| Q085 The Hidden Crossing | Use limited Passage and register at Lantern Redoubt | Gate transit and arrival must not trigger premature reveal |
| Q086 Lantern Redoubt | First meeting with living Kel'vari | First mandatory race reveal, persist per-character flag |
| Q087 Those Who Remained | Optional Kel'vari worker rescue | Must be available only after first contact/authorized access |
| Q088 Permission to Remain | Aru'ven grants bounded research access | Kel'vari sovereignty and explicit consent retained |
| Q089 The Broken Path | Examine ancient Passage ruins and Oma'vira traces | Ruins stay destroyed; no new gate opening |
| Q090 A Warning Preserved | Optional archive preservation | Requires authorized archive location; optional copy not mandatory evidence |
| Q091 Questions for the Sixth | Request supervised meeting through Sel'ora | Sixth Scholar remains unidentified |
| Q092 The Custodian's Conditions | Accept archive rules and protect copyist | Mandatory copyist aid distinct from optional Q093 store defense |
| Q093 An Audience Earned | Optional archive-store protection | Must not gate Q094 |
| Q094 Six Names in the Record | Authenticate six scholar names and verifier mark | Independent evidence without identity revelation |
| Q095 The Scholar Who Remained | Consult Grey Surveyor and check annotations | Precise corrections are clues, not yet conclusive proof |
| Q096 The Verifier's Mark | Compare ancient and modern signatures | Independent authentication required |
| Q097 The Last Witness | Grey Surveyor reveals he is Oth'varen | Nameplate/dialogue label changes only after shared reveal |
| Q098 Echoes Before the Sundering | Review original project records and alterations | Oth'varen is a witness, not omniscient |
| Q099 The Six Reunited | Witness archival scholarly disagreement | Recorded projection is evidence, not a perfect reconstruction |
| Q100 The Cost of Keeping Quiet | Optional Kel'vari custodian protection | Requires access to archive frontier |
| Q101 Two Histories | Optional independent post-Sundering records | Neither history alone establishes full truth |
| Q102 The Delegation's Shadow | Reconcile Shattered Delegation dates and dispatch | No early Su'vareth attribution without evidence |
| Q103 The Scholar's Request | Oth'varen proposes bounded investigation | Separate Covenant permission and hostile Dominion evidence acquisition |

## Shared gameplay and QA retained

Classic manual targeting/facing and weapon-timed autoattack; safe solo story progression; independent evidence receipts; safe death/disconnect recovery; no mandatory PvP, party, class or profession. Travel remains physical by road, ship and flight paths, with the restricted Passage as a consent-based exception. Optional regional activities must not be hidden prerequisites for mandatory reveals. Dialogue, journals, subtitles and NPC nameplates must respect the Q086 and Q097 reveal gates.

Actual map coordinates, spawn/actor IDs, stable semantic interaction actions, XP/loot, combat tuning, localization and runtime integration remain unresolved. Codex's current Rust catalogue supports kill and semantic NPC interaction; exploration, inspection, defense, scene and phasing objectives remain planned. No permanent 32-quest catalogue limit is assumed.

## Superseded assumptions

- Chapter level bands are pacing guidance, not automatically executable minimum levels.
- Broad chapter-level “Kel'vari/Oth'varen reveal” summaries are **not** knowledge the player has at Q080.
- Generic stage scaffolding in the old implementation brief does not supersede actual manifest objectives.
- Draft completion aliases are not permanent runtime IDs.

## Cleanup disposition

The two old sources contain exactly the same 24 Q references and no extra quests. Their relevant location context, gameplay/retry requirements, first-contact sequencing and QA expectations are recorded here, in the canonical narrative, the manifest and the Q080–Q103 authoring review. **Do not delete either source until inbound references and any remaining unique details are checked.** Git history will retain the original content after deletion.

**Next:** reconcile Q104–Q128 legacy material, preserving the Shattered Delegation's political complexity and Su'vareth's permanent death.
