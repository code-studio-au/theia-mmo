# Spell feedback — initial P2-13 slice qualified, 2026-10-28

Prerequisites: the qualified data-driven ability loop, reliable ordered combat
outcomes, checked global/local coordinates and standard Bevy presentation.
The preserved profile-1 ability binaries remain in
`build/combat-qualified-v2-20261028`; [their evidence](COMBAT_AUTHORITY.md) is not
overwritten. Profile-2 feedback acceptance is recorded below; broader production
spell effects, boss readability and native Windows remain open.

A death retires its entity before the next render update. Impact/death cues must
therefore carry authoritative checked f64 positions instead of depending on a
surviving model or a presentation cache. Combat profile 2 binds that change in the
welcome identity; CE02 events carry positions for cast start, damage, death and
respawn. Baseline wire 12, the selection-only profile and ability definitions stay
unchanged. Clients with the old combat profile must reject the welcome explicitly.

Use standard Bevy meshes/materials and audio assets/sinks for a small original
feedback slice. Data defines cue colour, size, duration, gain and priority within
strict budgets. Keep health/command authority and critical UI separate from
cosmetic admission. Bound visuals and voices, reserve critical capacity and test
protection under cosmetic saturation; do not claim production boss readability
from an original training cue. Source tones must be original, reproducible assets.

Source positions use centralized origin conversion every frame; transient cues
expire and retire their resources. Event sequence and simulation timestamps govern
presentation, not local damage guesses. Old/lost entities cannot move a cue to a
replacement ID. Keep server code headless and audio/render dependencies in the
client. No custom renderer, mixer or particle framework is required.

Acceptance: bounded profile-2 codec and actual authority/transport lifecycle tests;
standard Bevy native pulses and spatial audio from actual encrypted server events;
origin/retirement checks, resource/voice limits and protected critical cues under
cosmetic load; exact native GPU/audio evidence and game-window captures. Audio
capture must use an owned isolated sink, never the user's microphone or unrelated
desktop audio. Full regressions, denied-warning Clippy and formatting precede
adoption. Native Windows remains a separate gate.

The opt-in launcher is `python3 tools/play.py --spell-feedback`; it implies combat
and passes presentation data only to clients. Normal play, targeting and
`--combat` keep the audio plugin disabled. The retained asset/chunk inspectors also
keep it disabled. Linux audio compilation requires the normal ALSA development
package (`libasound2-dev` on Ubuntu). This machine uses the matching official
Ubuntu 1.2.11-1ubuntu0.3 development archive in ignored `build/native-sdk`; no system
packages were installed. Existing Wayland/udev SDK paths still apply. Bevy's audio
and WAV features add dependencies without replacing existing engine/plugin pins.

The native qualification option is `tools/qualify_graybox.py --spell-feedback
content/gameplay/development-spell-feedback.json` together with the normal combat
catalog/prediction fixture. It records only an owned null-sink monitor, audits
child process IDs, leaves the default audio sink unchanged and unloads its own
module during cleanup. PCM verification proves a game signal reached that sink;
it does not establish physical speaker output or perceptual spatial quality.

Explicit `THEIA_FEEDBACK_STRESS=1` injects 256 cosmetic presentation requests per
actual mapped outcome through the same bounded queue/admission path. These
are synthetic effects, not players or server gameplay events. Explicit
`THEIA_FEEDBACK_COSMETICS=0` suppresses cosmetic visuals/voices without suppressing
critical cues. Neither variable is enabled by ordinary play. The qualification
shifts the checked render frame once while a live spatial sink exists, without
teleporting or changing server coordinates. Rebase samples and checked coordinate
round-trip error are reported separately from actual view visibility and PCM.

Prototype spatial scale is 0.1 audio units per world metre, with a 0.3 master
volume and per-cue gains at most 0.3. Bevy's initial playback and final-camera
emitter/ear updates use the same scale. This keeps the original quiet tones useful
at ordinary third-person camera distances; world/physics coordinates remain metres.

The first native iteration (`build/spell-feedback-native-wifi-v1-20261028`) is
retained as failed overall acceptance. Its isolated PCM passed (RMS 0.003106,
peak 0.05051, no foreign streams/default-sink change), effects/sinks/rebase samples
were observed, and critical budgets did not overflow. One client nevertheless
recorded a startup placeholder displacement before any cast or forced rebase.
A transport correction can arrive between Update and Last; movement sampling now
waits until presentation has initialized its first admitted global pose. No
existing displacement gate or counter was relaxed. The same run exposed Bevy's
one-screenshot-per-render-target constraint: damaged/feedback requests coincided,
and the latter was skipped. Qualification now requests one screenshot per frame
and leaves other conditions pending. The second fixture also injects cosmetics
alongside every mapped outcome to prove concurrent critical/cosmetic admission,
not merely cosmetic load earlier in a cast.


## Qualified evidence and safe stop — 2026-10-28

Linux, Ryzen 7800X3D, RX 7900 XTX (RADV NAVI31), Vulkan, Mesa 26.2.5 kisak;
1280×720 game windows, Bevy 0.20.0 and unchanged pinned Avian revision. Audio uses
Bevy/rodio/ALSA through the host PulseAudio-on-PipeWire 1.0.5 service, routed only to
the explicitly owned 48 kHz stereo null sink during capture. Matching official
Ubuntu ALSA SDK archive SHA-256:
`b83795b1ba0e40cfa8363359fd17a06260838d8ae0f315531771a62f7c7e215a`.
The ignored SDK needed its unversioned link resolved to the host's matching
`libasound.so.2.0.0`; no host package installation was performed. The server's
standalone dependency tree contains no Bevy renderer/audio, rodio or CPAL packages.

- `build/spell-feedback-workspace-v1.log`: 278 passed, zero failed, ten private
  fixture skips. The later startup-sampling/qualification changes passed all 57
  final client tests in `build/spell-feedback-client-final.log`.
  `build/spell-feedback-clippy-final.log` and `build/spell-feedback-fmt-final.log`
  pass denied-warning workspace/all-target Clippy and formatting.
- `build/spell-feedback-native-wifi-v2-20261028/verification.json`: two actual
  encrypted clients, 35 seconds, 40–100 ms/direction, 10% loss plus duplicates and
  reordering, active Khronos Vulkan validation. Each reports 15 damage outcomes,
  three deaths, two respawns, 20 cast starts and zero event gaps/backpressure/
  rejected requests. Five casts cancel after the other actor kills their target.
  Actual moving-cast authority samples: 160/163. Each injected 10,240 synthetic
  cosmetic presentation requests, with all 18 critical visuals and all 18 critical
  voices admitted under overlapping cosmetic load; every critical voice obtained
  a real spatial sink. Critical queue/visual/voice overflow: zero. Peak visuals 50,
  voices 26; cosmetic drops are explicit. Ground error ≤1.73 µm; 385 terrain chunks
  loaded/304 retired, ≤81 resident per client. Visual coordinate round-trip error
  ≤0.36 µm; actual live voices/visuals survive forced render-origin shifts. No
  large movement displacements, no validation errors. Screenshots include
  `client-1/feedback.png`, damage, death and respawn states.
- Server: 20 accepted casts, zero rejected, 90 event deliveries, maximum reliable
  queue 545 bytes; no backpressure disconnects, simulation/I/O missed deadlines,
  skipped intervals or movement recoveries. Maximum simulation work 0.369 ms.
- Saturated two-client frame rates were 74.45/77.37 FPS with validation and capture
  instrumentation active; this is neither an uncapped ceiling nor population
  capacity. Actual private-sink PCM RMS 0.004372, peak 0.05371, with 402,149 nonzero
  samples and no clipping, foreign streams or default-sink change.
- `build/spell-feedback-critical-only-v1-20261028/verification.json`: one actual
  impaired client, 35 seconds, cosmetics disabled while 6,400 synthetic requests
  still exercise admission. All 12 critical visual/voice cues remain, peak 2/2,
  actual spatial sinks and visible samples, live rebase samples and no critical
  overflow. PCM RMS 0.0001237, peak 0.002472, 150,884 nonzero samples. Both capture
  modules and recorders were removed; no desktop or microphone source was used.
- `build/spell-feedback-baseline-v1-20261028/verification.json`: same binaries,
  ordinary no-combat/audio-disabled saved-terrain prediction, 35 seconds, one
  client, validation off, 826.78 FPS uncapped. This isolated empty-world baseline
  is not comparable to instrumented two-client saturation as a hardware ceiling.
- `build/spell-feedback-launcher-v1-20261028/verification.json`: real two-client
  feedback launch passed interruption and injected child failure, removed all
  owned game processes/tickets, and passed feedback arguments only to clients.

Preserved binaries and exact original data:
`build/spell-feedback-qualified-v1-20261028/sha256.json`.

| Artifact | SHA-256 |
| --- | --- |
| Client | `010f4d8af8e21d8e294b5892169a4070381c8b088a302fdf01b2e33a5f6cd70d` |
| Server | `6904878e0fd34b69c4567b18fdbb4466ba73445cf680128f5aeba34a27e70387` |
| Ability data | `c5c53ea53fc01df78e9be45a733731257bccc76f31853e12c5b09952b54d377c` |
| Feedback data | `ffdfaa7d096d06c01abf3a0f69b46dfdee4ecac64bdbe3509aa2620fb5abc366` |
| External character | `e0711a9603665cf25ec19e5b12539cc9a6f9db70a4658bdc87a74064fc0a0dfe` |

No player HP/PvP, AI/navigation, persistence/rewards, cast animations, production
telegraph readability or Windows execution is claimed. Input automation uses real
production ButtonInput handling and encrypted acknowledgements, not physical
keyboard/mouse delivery. PCM establishes output to the isolated sink, not speaker
output or perceptual HRTF quality. All changes remain local/uncommitted. Stopped
safely after documentation at the user's request; P2-14 action-slot work is next
planned and has not started. Private story/Obsidian changes remain untouched.
