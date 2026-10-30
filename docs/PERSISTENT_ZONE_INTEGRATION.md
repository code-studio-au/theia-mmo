# Persistent training-zone integration — initial P3-16 gate

Status, 2026-10-30: initial integration qualified and stopped safely.
Final proof: `build/zone-native-1920-v2-20261030/verification.json`;
immutable source/binaries/proof: `build/zone-qualified-v1-20261030`.
441 workspace tests pass / 30 explicit skips with unchanged source; formatting,
denied-warning workspace/all-target Clippy and nine Python helper tests pass.
Two actual local OIDC accounts/native Linux clients complete the full lifecycle
at 1920×1200 on RX 7900 XTX/RADV/Vulkan, including exact latest item receipts.
All owned provider/database services stop and private ticket files are removed.
The combined integration is qualified at this display size only; the prior vendor
checkpoint separately qualifies 1280×720, 1920×1200 and 3440×1440. No additional
display qualification is part of this stopped slice. Prior checkpoints remain
preserved: `build/vendor-qualified-v1-20261030` and
`build/gameplay-content-qualified-v1-20261030`. P3-17/18 remain unspecified.

## Scope and actual prerequisites

Combine the existing initial P3-01–12 gameplay systems on the saved graybox using
standard Bevy presentation, real OIDC accounts, encrypted admission and actual
PostgreSQL. P3-13's bounded saved streaming/distance LOD and P3-14's existing
locomotion/feedback are sufficient for this integration check; production vistas,
animation expansion and native Windows remain separate. P3-15 now validates the
existing source bundle; it is not silently adopted as a runtime manifest.

Use a named synthetic native driver, never enabled by normal play. Each fresh
account completes its own three-kill objective, observes committed XP, claims its
once-per-character reward, buys a different number of fixed barter offers and
equips a different owned item. Then close, logout/reconnect and restart the database
and server. Compare own rendered widgets, exact bags, equipment, XP/objectives and
receipts with SQL at every lifecycle boundary. Hold B's combat input until A's
flow completes to prove A's kills and purchases do not credit B.

Reuse the existing short native qualification XP curve (0/25/50) and explicit
instant/100-damage ability fixture, each bound to its actual bytes. This proves
integration rather than production balance, normal encounter timings or capacity.
Keep the default catalogues unchanged. Inputs use real Bevy keyboard/picking and
normal command paths, never direct credit/grant or database mutation commands.

## Concrete acceptance

- Two actual accounts start with private zero progress. A completes while B's
  objective/XP/backpack/claim remain unchanged. B then completes independently.
- Counts 0/1/2/3 and XP 0/25/50/50 are observed. Exactly one reward is consumed per
  character. A buys once/equips its sword; B buys twice/equips its wand. All item
  intents share the existing revision/pending controller and commit-before-reply.
- Actual widgets match committed SQL and stable known profile text. Capture the
  objective, reward, trade and equipped final panels; inspect the native scene.
- Fresh leases start with closed dialogue and recover exact bags/hands/XP/objectives
  after logout and database/server restart without another purchase or reward.
  Retain account ownership/replay/pose fencing/backend-loss checks and cleanup.
- Formatting, denied-warning Clippy and appropriate tests pass. Preserve exact
  source/binaries/proof separately from earlier qualified slices.

This is an initial persistent **training** integration gate. General quest
acceptance/turn-in rewards, authored story/production zone content, player death,
general NPC lifecycle and production economy remain open; a passing check does not
complete all of P3-16 or establish MMO-scale capacity. Physical OS input, 4K and
Windows are separate qualifications.
