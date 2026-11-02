# Human Q158–Q184 — Final Legacy Pack and Brief Reconciliation

**Status: Needs Review; editorial reconciliation, not executable quest data.** [Canonical Human Campaign](Human_1_80_Canonical_Campaign.md) is narrative authority; [Current Quest Authoring Manifest](Human_1_80_Current_Quest_Authoring_Manifest.json) is the sole structured inventory; [Codex Quest Engine](../../../QUEST_ENGINE.md) governs implemented capabilities. Companion: [Q158–Q184 Rust Authoring Review](Human_Q158_Q184_Rust_Authoring_Review.md).

## Sources checked

- `Human_70_80_Playable_Quest_Pack.md` v0.1 and `Human_70_80_Implementation_Brief.md` v0.2: **17 Q158–Q174** sections in each.
- `Human_80_Finale_Playable_Quest_Pack.md` v0.1 and `Human_80_Finale_Implementation_Brief.md` v0.2: **10 Q175–Q184** sections in each.
- All four documents contain generic gameplay and recovery language. No extra Q identifiers appear. The older briefs add repeated accept/inspect/report scaffolding, not verified extra quest objectives.

## Retained content and corrections

| Quest | Legacy purpose / retained design | Current authority |
|---|---|---|
| Q158 | Survey gradually appearing essence signatures | Manifestation after Q156 is slow and distinguishable from ordinary Unmade |
| Q159 | Optional rescue of exposed Kel'vari settlement | Must require post-Veilbreaker world phase and access |
| Q160 | Trace Thes'var's manifestation-stabilization instructions | Architect knowingly serves Nhal |
| Q161 | Witness inversion fragment manifestation | Fractured identity/pattern, not destroyed divine essence |
| Q162 | Compare ancient essence evidence with residual readings | Fragment fate and future identity deliberately unresolved |
| Q163 | Receive Architect transmission and verify destination | Corrupted argument is not trustworthy permission |
| Q164 | Test limited multi-Anchor synchronization | Hypothesis remains uncertain |
| Q165 | Guardian council grants conditional preparation authorization | Not a mandate to force final activation |
| Q166 | Scout approach and extraction routes | Distinct from Architect's earlier invitation |
| Q167 | Repair structural supports | Test limits and evidence of safe loads |
| Q168 | Optional ecological mitigation | Requires post-Shroud world state |
| Q169 | Optional ship/flight/Passage logistics | Restricted Passage requires separate consent; no waypoint teleport |
| Q170 | Train redundant crews and preserve procedures | Oth'varen does not foresee his own fatal correction |
| Q171 | Secure independent Covenant and Kel'vari consent | Sovereign permission cannot be transferred |
| Q172 | Survey vulnerable junction and emergency access | Do not reveal later sabotage or fatal correction |
| Q173 | Bounded final readiness test | Not the Nhal-shattering activation |
| Q174 | Final team, consent and supply checks | Explicit level-80 gate is a **proposal requiring approval and engine support**, not inferred from chapter band |
| Q175 | Enter Thes'var facility and preserve inversion machinery | No destruction of needed convergence controls |
| Q176 | Confront and physically kill Thes'var | His final sabotage becomes apparent only afterward |
| Q177 | Diagnose misalignment and failed conventional correction | Failure is verified, not a scripted player defeat |
| Q178 | Oth'varen discovers lethal correction and freely chooses | Knowledge of fatality comes only after sabotage |
| Q179 | Farewell and defensive station readiness | No alternate ending where Oth'varen survives |
| Q180 | Defend mortal stations while Seven freely participate | No re-Binding or compelled divine participation |
| Q181 | Correction holds → Oth'varen sees success → **physically dies** → final inversion shatters Nhal's coherent identity | Manifest now includes explicit death stage; divine essence persists |
| Q182 | Memorial and evidence of surviving Seven | Fragment destination deliberately not determined |
| Q183 | Report independently to Kel'vari and Covenant | Shroud fade began Q156; faction war remains |
| Q184 | Return to Millhaven by ordinary travel | Personal closure, no mandatory optional cleanup |

## Common authoring, recovery and QA retained

Strict Classic targeting/facing and autoattack, solo story accessibility, durable evidence and choices, safe retry checkpoints, no mandatory PvP/group/class/profession, physical road/ship/flight travel and consented restricted Passage only. Encounter failure resets active combat, not irreversible story facts or verified evidence. Each boss kill, rescue, correction, alignment and final inversion needs an independent trusted event. Never equate viewing a cutscene with authoritative world-state credit.

Actor/target IDs, semantic interactions, XP/loot, item possession and consumption, spawn geometry, encounter timing, localization, phasing and engine integration remain unresolved. Current Rust catalogue supports only kill and semantic NPC interaction; other objectives are planned. Cross-pack registry and versioned saved-journal migrations remain pending.

## Superseded source assumptions

The old level-70/75/80 chapter minimums are editorial pacing, not automatic hard gates. Q174's explicitly described finale eligibility requires a separate approval decision. Generic implementation-brief stages and old `HUMAN_Q###_DONE` aliases do not supersede the current manifest. The old Q181 description lacks the manifest's explicit physical-death stage; the new canonical sequencing governs.

## Cleanup disposition

The four older files contain the same **27 Q references** across their respective chapter pairs and no extra quests. Their unique gameplay purpose, location context, recovery rules and QA expectations are represented in the canonical narrative, current manifest, Rust authoring review and this migration note. **Do not delete until inbound references and remaining unique text are verified.** Git history preserves originals after eventual removal.

**Next:** audit inbound references for all 17 legacy packs/briefs, reconcile any remaining unique content, then delete redundant files. The goal is one active narrative authority, one working structured quest inventory, referenced dialogue/encounter assets and Codex's implementation contract.
