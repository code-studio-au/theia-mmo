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
headlessly. The following headless candidate adds dragging and slot rearrangement; persisted layouts remain pending;
the preserved optimized `build/local-play-release-20261108-v1` predates this book.


## Action-bar binding candidate — 8 November 2026

Implemented session-local twelve-slot bindings by stable ability ID. The default
layout preserves the first configured actions. With the book open, dragging a
book entry copies its binding; dragging a populated bar slot swaps it with the
destination, including an empty slot. Empty slots send no casts. The standard
Bevy widgets rebuild only when the binding revision changes; icon, tooltip,
key label and cooldown display resolve the bound ability. Duplicate bindings
share the server ability cooldown. Rearrangement sends no gameplay mutations.

Fresh drag events require a focused game Window and a matching admitted owner,
world and epoch with current snapshot membership. Escape/closed book, focus or
admission loss, changed source page and pointer cancellation discard the local
drag. Drop validation checks the original entity, source binding and current
context. Ending outside a slot leaves the layout unchanged. While editing,
bar widget activation is discarded; existing keyboard ability bindings remain
available. Normal UI picking and existing world gesture guards are retained.

Prerequisites are the configured catalogue, existing P-key book, twelve primary
bindings and authoritative ability-ID cast path. P2-14 / P3-06 / P5-13 remain
partial: persistent per-character layouts, learning/class unlocks, key rebinding,
additional action bars and native drag qualification are separate work.

Headless acceptance covers bounded IDs/slots, empty/no-op/overflow behavior,
stale source/page/admission rejection, actual widget rebuilds and cooldown fills.
The encrypted Renet loopback uses production Network and drag handlers with
synthetic Bevy events and a headless server: book copy, bar swap, focus-loss
cancellation, an actual accepted cast/damage and premature-repeat refusal all
pass while the original server cooldown is retained. It has no OS window,
input device, renderer, OIDC or SQL. It does not establish native pointer behavior
or GPU layout correctness. The new drag hint and rebuilt bar are unqualified
rendering candidates under the user's desktop prohibition; the earlier read-only
book native evidence and frozen optimized identity pack remain preserved.

231 ordinary client tests pass (one licensed-art case explicitly ignored), plus
strict workspace/all-target Clippy and formatting. Evidence lives under
`build/ability-book-20261108`: `bar-client-v6.log`, `bar-clippy-v5.log`,
`bar-fmt-v2.log` and `bar-build-v2.log`. Earlier `bar-renet-v1` through `v3` are
diagnostic compile/expected-rejection assertion failures; `bar-renet-v4` passes,
with the expanded event/focus scenario included in the full v6 suite.


## Native action-bar qualification — 8 November 2026

After the user restored mouse/keyboard/display permission, the candidate passed
four actual Linux starts:1280×720,1920×1200,3440×1440 and SQL process restart/
fresh login at1280×720. Book-copy, populated-to-empty move, populated-to-populated
swap and restoration pass with actual pointer drags. Dropping outside slots leaves
the revision/bindings/selection unchanged and sends no widget casts. A temporary
owned X11 focus window proves focus-loss cancellation during an actual drag.
The earlier root-focus attempt (`bar-native-v1`) was a harness timeout and is not
accepted evidence; `bar-native-v2` is the final four-start result.

The normal quest fights use the copied `=` slot through actual keyboard events.
Server Sword Slash events and all three kills/four quest actions pass in each
fresh size; duplicate-slot cooldown fills match in119 positive samples (44/39/36
by size). Completed quests/125 XP survive PostgreSQL process restart and fresh
login. Bindings remain session-local: they reset on the fresh client, which then
repeats edit/cancel checks. This is not persisted action-bar evidence.

Native screenshots show the small-window book/hint/bar within viewport bounds;
the1280 capture was inspected. GPU diagnostics are clean on AMD Radeon RX7900XTX /
RADV NAVI31 / Vulkan / Mesa26.2.6 kisak. Evidence:
`build/ability-book-20261108/bar-native-v2/verification.json`, per-size proofs and
captures, plus binary hashes in `binaries-bar-v2/build.json`. This is controlled
Linux functional interaction, not physical-input, Windows, FPS/crowd or non-unit
monitor-DPI qualification. All owned provider/SQL/server/client processes and the
temporary input/focus fixtures stopped. Optimized identity and earlier read-only
book packs remain preserved; the new bar pack is debug, not optimized.
