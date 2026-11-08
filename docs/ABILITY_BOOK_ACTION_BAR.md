# Ability book and action-bar workflow

8 November 2026. P2-14 / P3-06 / P5-13. User screenshot references:
[UI direction](UI_FANTASY_WARRIOR.md#extended-interaction-references--7-november-2026).

Implement bounded slices using standard Bevy UI. First add a P-key ability book
in the opt-in fantasy HUD, showing the configured combat catalogue with icons,
readable authored metric costs/range/timing and paging. It sends no casts or
learning requests. Do not invent class tabs, ranks, passive abilities or learned
skill unlocks absent from current data. Current server configuration grants the
development ability set; persistent character learning remains future work.

Next use stable ability IDs for a twelve-slot action-bar layout. Dragging from
the book copies an action; rearranging bar slots swaps bindings; empty slots
cast nothing. Rebinding never clears server cooldowns, grants an ability or
changes targeting/facing. Resolve current cooldown/resource eligibility by the
bound ID, not its display slot. Cancel drag on focus/context loss or Escape;
dropping onto world/UI outside a slot makes no change or accidental world click.
Keep the initial layout session-local and preserve current default bindings.
Persistent per-character layout and key rebinding are separate slices.

Acceptance: focused keyboard/widget navigation; bounded complete catalogue pages;
Escape closes the book before clearing targets; UI blocks world clicks/camera
start gestures; configured metric descriptions match action tooltips. Native
small/standard/ultrawide layout and actual opens/closes must pass before qualifying
the new panel. Drag/rebinding requires actual pointer tests plus server-observed
casts from changed slots, unchanged cooldowns and invalid/focus-loss cancellation.
Implementation progress must distinguish the read-only book from rearrangement.


## Read-only book qualified — 8 November 2026

Implemented the P-key book and Interface → Abilities button in the opt-in HUD.
It reuses the configured ability icons and authored metric tooltip descriptions,
with three entries per page and bounded navigation through up to64 entries.
No class/rank/learning facts are invented, and it sends no cast commands.
Focused keyboard/widget guards and closed-panel refusal protect fresh input.
Escape priority is Interface, book, reader, loot and backpack before target clear.
Book surfaces use normal Bevy UI picking to block world-click gestures.

225 ordinary client tests pass, including the 64-entry paging/cost-description
coverage, focus/closed controls and the extended actual multi-panel Escape
priority test. Strict workspace/all-target Clippy and formatting pass. Evidence:
`build/player-identity-20261108/book-client-v2.log`,
`build/ability-book-20261108/client-v1.log`, `fmt-v1.log` and
`build/player-identity-20261108/book-clippy-v2.log`.

Before the latest desktop restriction, `build/ability-book-20261108/native-v1`
qualified four actual starts at1280×720, 1920×1200, 3440×1440 and SQL restart/
fresh login. P opens, Interface opens, actual Previous/Next controls clamp the
one-page fixture, Close/Escape close, and no keyboard cast is submitted by those
controls. Actual rows show icons and configured metric data within viewport
bounds; the small-window capture was visually inspected. Normal quest/combat/
progression and Interface Exit still pass. Zero GPU diagnostics on AMD Radeon
RX7900XTX / RADV NAVI31 / Vulkan / Mesa26.2.6 kisak. Services and the temporary
wheel device stopped. The native fixture has two configured abilities; the
complete64-entry catalogue is headless coverage, not native crowded-page proof.

The user has now prohibited mouse, keyboard and display access. Continue only
headlessly. Dragging, slot rearrangement and persisted layouts remain pending;
the preserved optimized `build/local-play-release-20261108-v1` predates this book.
