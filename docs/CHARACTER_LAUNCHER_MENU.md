# Front-end visual references — 16 November 2026

Planned direction from the user's WoW screenshots; no title/selection/loading
screen implementation is claimed by these notes. Local originals are
`/home/steven/Pictures/Screenshot from 2026-11-16 <time>.png` (hyphenated times).

- 15:32:19 / 15:32:23: atmospheric title backdrop, Theia branding and a compact
  centred connection-status panel with Cancel; Quit remains accessible. Use the
  existing authenticated browser/provider flow, with clear failure/retry states.
- 15:32:28: explicit retrieving-character-list state; Enter World disabled
  until a valid selection is available. Retain coherent background presentation
  while the list loads.
- 15:32:31: large selected-character preview, right-side scrollable roster with
  name/level/class/location where supported, highlighted selection, creation
  action, Back and prominent Enter World beneath the preview. Rotation controls
  are a preview interaction. Shop and other incidental buttons are not adopted.
- 15:32:38 / 15:32:44: full-screen Theia world/map illustration, framed progress
  bar and a short gameplay tip. Tie progress to actual loading work and world
  readiness; use an indeterminate state when totals are unavailable. Keep entry
  gated until the world is ready, with failures/cancellation handled explicitly.
  A loading screen complements the fixed initial mesh-pipeline preparation; it
  must not hide a recurring mid-play readiness loss.

Use existing Bevy UI and Theia/appropriately licensed art. Dependencies are
P3-01/P3-03 account/character admission, P2-02/P2-08 world readiness and existing
UI input ownership. Acceptance must cover cancel/retry, empty/existing rosters,
selection changes, readiness failures, and readable 1720x1400/1720x700 layouts.
Party invite/member/marking references live in [PARTIES.md](PARTIES.md).

# Character launcher menu — qualified 2026-10-31

P3-03 continuation. Prerequisite: read-only character-profile preflight qualified
at `build/gameplay-profiles-qualified-v1-20261031` (476 workspace passes / 40
explicit skips, 30 actual SQL contracts and actual OIDC/native refusal/restart).

The normal launcher previously created only the first character. An existing
account automatically selected its sole character and offered no creation option.
That made an incompatible development save difficult to work around through the
normal flow. The menu now lists existing characters, defaults to selection 1 and
offers N to create another below the eight-character limit. Creation still uses
the authenticated server API; ownership, names, capacity, world and spawn remain
server-controlled. At capacity, creation is not offered or submitted. Unsafe
response names are rejected before printing.

The normal launcher calls `choose_and_enter`. An exact known nonretryable content
conflict prints the public message and returns to selection, without automatic
migration or deleting any save. The user can choose a compatible character or
create another. Temporary retiring-character conflicts retain the bounded retry;
other account errors stop the operation and use existing cleanup. EOF/interrupt
still unwinds the launcher. This is console workflow, not a complete Bevy login UI.

Acceptance: bounded existing/default/new/cap choices, malformed menu response,
known-incompatible selection followed by creation/entry, retained transient retry,
actual OIDC creation on an account with existing characters, menu recovery into a
compatible native character, unchanged legacy save through restart, and normal
launcher startup/cleanup. Rust/content/binary compatibility must remain unchanged
from the preserved profile-preflight checkpoint. Physical terminal/browser input,
production account experience and native Windows are not established by fixtures.

Twelve account-helper tests / thirty focused Python checks pass, alongside
formatting and denied-warning workspace/all-target Clippy. Actual OIDC/native menu
creation, refusal/reselection, rewarded gameplay and restart pass at 1920×1200:
`build/character-menu-native-1920-v1-20261031`. Seven native processes record zero
Vulkan diagnostics on Linux RX 7900 XTX/RADV/Vulkan. The legacy row remains unchanged.
Actual normal launcher startup/cleanup passes interrupt, child failure and pending
login cancellation with eleven matching profiles:
`build/character-menu-launcher-v2-20261031`. Owned services stopped; private tickets
and temporary snapshots were removed.

The five changed Python files are `account_login.py`, `play.py`,
`test_account_login.py`, `qualify_authenticated_native.py` and
`qualify_account_launcher.py`. Rust/content and all
four preserved binaries are byte-for-byte unchanged from the previous checkpoint.
Its 476-pass / 40-skip workspace and 30-contract/33-command SQL results are retained,
not new reruns. Exact current source, binaries, validator, docs and public proof:
`build/character-menu-qualified-v1-20261031`. Physical input and native Windows
remain unqualified. No commit/push this session.
