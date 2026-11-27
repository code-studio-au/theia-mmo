Targeting HUD follow-up, 1 November 2026: shortened the cycle hint to prevent
an overlap with the action bar at 1280×720. The final binary passes 113 client
tests, workspace Clippy, formatting and a fresh normal native 720p controls/
targeting run with zero GPU diagnostics; its capture is visually inspected.
Shared core/network sources are byte-identical to the qualified 146/445 result.
The preceding checkpoint retains the three-size feature checks. Current source,
binaries and proof: `build/classic-targeting-qualified-v2-20261101`.
Weapon-timed autoattack is next. Changes remain local and unpushed.

# Classic persistent targeting — local implementation, 1 November 2026

P2-11 / P2-14, with existing P2-04/05/07 classic facing and secure targeting
as prerequisites. This slice implements section 4 of THEIA-COMBAT-001; it does
not implement weapon-timed autoattack or combat facing checks.

The classic profile retains an existing hostile selection across heading,
attack range, LOS and ordinary interest/terrain unloading changes. One selected
entity's bounded metadata remains in the owner targeting view outside the normal
1024 m XZ interest window. This does not load terrain or authorize attacks. New
selections still require normal interest eligibility. Explicit clear, retirement
and a new owner session clear selection. Legacy profiles keep their old policy.
The classic identity changes to `theia-classic-facing-targeting-v2`; TC03/TP03
message layouts remain unchanged. Existing v1 checkpoint binaries remain intact.

Tab and Shift+Tab explicitly cycle resident, known, alive hostile candidates.
Unknown health does not imply death. Character facing supplies front preference,
then 3D distance in metres and stable entity ID break ties; reverse wraps the
same ordering. With no eligible candidate, selection is preserved. With no
eligible current selection, either direction starts at the first candidate.
Camera rotation does not participate. This deterministic search is a Theia
policy, not a verified reconstruction of Classic's exact algorithm.

The bounded classic controls JSON accepts `targeting.prefer_front` and optional
`targeting.maximum_distance_metres` (positive, at most 4096 m; null disables the
extra search radius). These are search settings, independent of attack range.
Existing movement-only files retain defaults. Panels and lost window focus gate
keyboard targeting. Friendly selection clears when Tab actually chooses a
hostile, rather than when a Tab press finds no eligible target.

The one immutable guide retains friendly selection when its presentation/chunk
is absent. Disconnect, a known replacement identity or changed world epoch
invalidates it. General dynamic friendly entities will require an explicit
removal observation; absence from interest is not a removal event. Interactions
and mouse picking still require resident, visible known actors.

Acceptance checks: deterministic forward/reverse and front/range boundaries;
actual World interest crossing and removal; unchanged legacy targeting;
formatting, denied-warning workspace Clippy and core/network/client tests;
native GPU and OS-input checks before claiming playable qualification.
Qualification passes: **146 core tests**, **445 network passes / ten explicit
skips / 68 groups**, **113 client tests**, formatting and workspace/all-target
denied-warning Clippy. The updated actual encrypted-UDP test passes at
20/30/60 Hz with target select/retain/clear/removal. There are **36 relevant
Python checks**; the 17 gameplay-bundle checks also pass against the final release
validator. The initial Python invocation used an outdated debug validator; it is
not acceptance evidence.

Three final normal OIDC/PostgreSQL starts at 1920×1200, 3440×1440 fullscreen and
1280×720 pass synthetic X11 input with **zero GPU diagnostics** on Linux AMD
Radeon RX 7900 XTX / RADV NAVI31, Vulkan, Mesa 26.2.6 kisak. Checks cover mouse
selection without attack/facing change, Tab selecting the same hostile,
Shift+Tab, independent orbit/steering retention, Escape panel-before-target,
explicit clear, disabled dodge/action camera and movement/alternate A/D strafe.
The one-hostile native fixture cannot demonstrate multi-hostile ordering;
shared numerical checks cover that. World tests prove four-kilometre retention,
restricted new discovery and removal. The native harness initially timed out
because the wolf was outside view; ordinary LMB camera orbit fixes the harness.
No gameplay request is injected through telemetry.

Proof: `build/classic-targeting-results-v{1,3,4}.json`, matching logs,
`build/classic-targeting-native-v3/verification.json` and per-size captures.
Exact source, binaries, art and proof are preserved in
`build/classic-targeting-qualified-v1-20261101`. Prior checkpoints remain intact.
No new FPS/MMO-capacity, full-workspace test, native Windows or physical-input
claim. The classic combat milestone remains incomplete: **five of thirteen**
checks, with weapon-timed autoattack/facing still next. See the
[next combat contract](CLASSIC_AUTOATTACK_PLAN.md). All changes remain local,
uncommitted and unpushed.
