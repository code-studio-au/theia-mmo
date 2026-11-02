# Floating guide quest markers — qualified, 2026-11-01

P3-11 / P3-12 / initial P3-14. The server's committed private quest lifecycle,
known guide presence, admitted identity, terrain residency, checked global
coordinates, standard Bevy camera/UI and Avian queries are the prerequisites.
Matched source/binaries/art/proof: `build/world-input-qualified-v1-20261101`.

One noninteractive standard Bevy marker appears above the known guide:
`! Quest available`, `... Quest in progress`, or `? Turn in quest`.
Shapes/text distinguish states in addition to colour. A turned-in quest has no
marker. Only committed observed progress selects a phase; pending actions never
advance it. Markers do not grant interaction range, eligibility, loot or XP.

Unknown/despawned guides, disconnected/foreign owners, mismatched quest profiles,
unloaded guide chunks, off-screen anchors and distances beyond 60 metres hide
the marker. Checked global positions pass through the central render frame.
Font size uses ten whole-pixel sizes from 15–24 logical pixels, retaining unchanged
text/fonts rather than rebuilding them each frame. One Avian ray per projected
frame tests static SOLID/GROUND occlusion. Characters remain nonblocking.

Five checks cover committed/restored/pending phases, identity/profile/residency,
distance edges, large-coordinate rebasing and actual Avian wall/terrain queries.
They pass within the **91 client tests**, with formatting, denied-warning
workspace/all-target Clippy and **41 Python checks**. This is a client/tool slice;
the previous 608-pass/40-skip whole-workspace result is retained, not rerun here.

The final native flow observes visible laid-out glyphs in all three phases,
removal on turn-in and absence through rejoin and database/server restart.
Two actual accounts retain private progress, server XP/rewards and backend-loss
refusal. Dedicated unobstructed screenshots show each marker above the guide.
Final X11 OS input runs also capture the marker at 1280×720, 1920×1200 and actual
3440×1440 fullscreen. All **14 final native starts have zero GPU diagnostics** on
Linux RX 7900 XTX / RADV NAVI31, Vulkan, Mesa 26.2.6.
[Exact control/native proof and scope](PLAYER_CONTROLS.md).

Initial diagnostics remain under `build/quest-marker-native-1920x1200-v{1,2,3}`:
an extra capture counter stalled, an evidence variable was wrong, then overly fast
scripted dialogue produced refusals. The capture helper now returns immediately
for an existing image; the checker uses its owned folder; the driver spaces
keyboard dialogue input and records bounded rejection reasons. The final full
flow has no rejected dialogue requests. These are fixture fixes, not weakened
server checks. Stopped disposable databases/identity runtimes were removed;
logs, screenshots and structured evidence remain.

This initial implementation covers one guide/one quest and uses default-font
text. General quest/NPC associations, imported icon assets, authored friendly
characters and broader HUD design remain open. No new native Windows, physical
hardware input, sustained performance, population or streaming-capacity claim is
made. Source art and the earlier production skin-fidelity limitation are unchanged.
