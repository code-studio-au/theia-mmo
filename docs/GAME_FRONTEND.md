# Native game entry and shared Theia UI

P3-01/P3-03 with P2-08/P2-09 and P2-14 presentation. Prerequisites are the
implemented account service, PostgreSQL appearance migration, modular hero
catalogue and standard Bevy UI. Login, character selection, creation and gameplay
share one native client/window.

The live preview is noninteractive outside its rotation controls. Clicking the
name field after customizing or zooming restores text focus. Create waits for
queued text edits before submitting the validated spelling. When the account
roster is returned, the newly added character is selected by its ID, including
when it sorts before existing entries. The native qualification created
`Zed`, changed the second character's face, zoomed three times, entered
`steve` after refocusing the field and verified the exact name in the game.
See `build/character-name-refocus-fixed-no-mana`.

The Login screen restores the painted cloaked adventurer and illustrated Theia
Online title. Username and masked password are live native inputs. Settings,
Support and Exit are native controls; no browser login button or browser fallback
is available in this game flow. Errors appear only when needed. Passwords are
bounded, cleared after submission, remasked on focus loss and never included in
widget telemetry, command files, process arguments or logs. The password visibility eye is inside the password field, with an accessible
Show/Hide password label. Password editing
supports Unicode, selection, cursor movement and deletion; clipboard paste is not
yet provided for this custom masked input.

The current working sign-in implementation is an adapter for the owned **local
Keycloak development provider**. The native client sends credentials in memory to
an ephemeral loopback listener using a private capability. The adapter accepts only
the configured local issuer's expected sign-in form and callback; it retains the
existing authorization-code, PKCE, state/nonce and server token validation. Other
providers or MFA steps fail with an in-game message. This does not claim production
native authentication, recovery or a deployed account service. The
local wrapper still owns its temporary provider/database lifetime. Historical
editor/console account routes retain their separate browser-based qualification.

Native Create Account now uses that same owned provider and code/PKCE callback.
Its separate reference-style screen includes confirmation, password rules and an
illustrated portrait; successful registration opens an empty account roster.
Availability is advertised by new local sessions; unsupported legal/news controls
remain disabled. See [ACCOUNT_REGISTRATION.md](ACCOUNT_REGISTRATION.md) for the
provider policy, artwork, qualification and deployment limits.

Back to Login on character selection uses the existing account logout route.
After the service confirms success, the client clears the roster, preview,
account name and pending character edits. A refused sign-out preserves the
current roster and credential for retry. The native two-account check creates
separate characters, verifies the second account opens without a preview,
restores the first account's original roster, then enters the world.
Evidence: `build/account-switch-v1/qualification.json`. Linux Vulkan llvmpipe
(LLVM 21.1.8, Mesa 26.2.7), zero GPU diagnostics. 25 Python and 362 client
tests pass, with five existing client skips.

The launcher design follows the user’s navy-and-gold reference. The first
planned slice installs and starts a verified local build, with login remaining
inside the game. Signed distribution follows the local install and update
checks. See [GAME_LAUNCHER.md](GAME_LAUNCHER.md).

Character selection has its own courtyard background, right-side account roster,
selected live hero, rotation controls and Enter World. An empty account has no
hero and cannot enter. New Character opens the native cosmetic creation screen;
its latest reference-layout pass is described in CHARACTER_CUSTOMISATION.md. Back
discards the draft; Create durably creates a server-owned character. The roster
panel is capped at 460 logical pixels, with 24-pixel horizontal name padding. Selected rows use an amber
fill and the regenerated chamfered frame instead of an external square focus outline.
The roster supports eight saved entries and selects actual returned records;
no invented classes, levels or locations appear. Enter World obtains the existing
short-lived admission ticket and starts authenticated Renet. Normal world loading
then admits controls. Empty accounts and returning from abandoned creation never
show the default preview character.

The preview uses standard Bevy glTF, skeletal animation, materials and a transparent
render-to-texture viewport on an isolated render layer. Its ground origin is
anchored at 40% of courtyard width and 81% of height on the roster. Creation
centres its ground point between the appearance panels, above the fixed footer.
Once a preview has settled, unrelated shader compilation cannot hide it. Page and
appearance changes still reset readiness, but a complete previous appearance
remains visible during a cosmetic load; Create/Enter stay gated until the requested
appearance settles. Leaving preview pages hides the model. Gameplay camera queries exclude the
preview. Waiting at Login does not consume the gameplay loading timeout.

The shared skin, installed with the existing `--fantasy-ui` presentation option,
is in `apps/client/src/ui_theme.rs`. It provides gold frames, sapphire
primary buttons, dark secondary buttons, cream text, hover/pressed/disabled/focus
states, gold icon links and input surfaces. It decorates existing native controls
without changing gameplay actions. Inventory, dialogue, quests, documents, party,
map, ability book and settings share the frame/button treatment; item and action
icons now use the approved regenerated sprite package. Dialogue/document reading panels now use navy
with cream text. Embedded Liberation Serif headings and Liberation Sans body text
replace the heavy HUD-pack typefaces; unmodified font files and their SIL OFL
license are in `content/ui/fonts/`. Artwork provenance is in
`content/ui/frontend/PROVENANCE.md`. The atlas/state integration and its exact
scope are described in [UI_SPRITE_INTEGRATION.md](UI_SPRITE_INTEGRATION.md).

The settings reference guides a category sidebar and grouped existing controls;
unsupported graphics options from the mockup are visible, greyed out and disabled.
Reference categories include Graphics, Audio, Gameplay, Controls, Key Bindings,
Interface, HUD, Social, Accessibility and Language. Only implemented pages and
controls activate; Developer remains available for the existing playtest tools.
Party management, invitation, member frames and member menus use the shared gold
trim. The illustrated reference portraits/levels/roles are not invented game data.
The lower-left HUD has General and Combat Log tabs. Combat history remains bounded
and server-sourced; tab switching preserves it and disconnect clears it. The General
tab supports real Say/Area composition when explicitly admitted with --chat-area.
Enter opens/submits; Escape and focus loss discard drafts. Only server-confirmed
messages enter history; refusals remain separate. Trade/Guild/Party tabs remain
disabled. See LOCAL_CHAT.md for qualification and remaining chat scope.

Both front-end and game settings expose the same accessible VSync checkbox and
interface-size slider (75–150%). They update existing persisted preferences, not a
second settings store. Slider arrows, Home/End and checkbox Space/Enter work through
Bevy's standard headless widgets. View distance and FPS limits retain their actual
existing resources through dropdowns. Apply records the current preview values;
OK accepts and closes; Cancel or Escape restores the last applied values. Changes
preview live through the existing preference store, so this is not crash-isolated
draft persistence. Decorative images ignore picking. The zone editor is separate
from this game UI skin.

The P2-14 Graphics panel uses the primary Bevy window and its current monitor
for three display modes. Windowed uses an ordinary decorated window and lets
the player choose a size. Borderless fills the monitor at its desktop resolution.
Fullscreen uses exclusive video modes on supported backends, with resolutions
reported by the monitor. Wayland offers Windowed and Borderless.

Apply and OK preview display changes. Keep Changes saves the new mode and
resolution. Revert, Escape or a 15-second timeout restores the previous setting.
Cancel discards unapplied choices. F11 switches between windowed and borderless
for the current session. The configured UI preference file has a companion
`.display.json` file for confirmed choices, so previous game releases can still
read the ordinary preferences. Dedicated tiled test windows retain their launch
layout and leave these controls disabled.

Local client release 0.1.0.5 passed 369 client tests with five existing ignores,
formatting and denied-warning Clippy. Private X11 with Vulkan llvmpipe verified
1280×720 windowed, 1920×1080 borderless and exclusive, deferred Apply, OK and
Keep Changes, Cancel, manual and timed reversion, and restored preferences after
relaunch. The run logged 13 instances of the known Vulkan swapchain resize
diagnostic, above the usual threshold, and no other GPU diagnostics. The private
X server lacked DRI3 for RADV presentation. Native hardware and Windows remain
unverified for this feature. See `build/graphics-display-native-a6foawve/qualification.json`.

Client 0.1.0.6 gives Graphics controls in both menus a shared body font and
uses the same geometry and skin for all dropdown fields and options. Unavailable
controls use the same proportions with a muted disabled state. The Settings
panel has a raised title plaque, navy gradients, fine title-bar highlights,
flat sidebar rows and a framed harbour preview. The private X11 title screen
verified all four dropdowns and the header close action at 1600×900 and
1280×720 with Vulkan llvmpipe and zero GPU diagnostics. See
`build/graphics-reference-native-xwt92vw2/qualification.json`.

Client 0.1.0.11 shares the Graphics page heading, gradient section bars and
control typography with Interface, Support and Developer. Interface groups
its scaling controls and tools, Support groups sign-in and account guidance,
and Developer groups movement, position and session controls. Private Linux
X11 checks exercised Interface scaling and Support at 1600×900 and 1280×720
with Vulkan llvmpipe and zero GPU diagnostics. The 370 client tests passed,
including Developer actions, with five existing ignores. The offscreen
Interface preview's combat-feedback overlap assertion also fails on 0.1.0.10.
See `build/settings-pages-native-ekpv67tl/qualification.json`.

`tools/play.py --game-front-end` requires one Classic account client and excludes
explicit character IDs and the old editor picker. Account population launch uses
it by default; `--editor-character-picker` retains the earlier route. Public bounded
JSON commands carry character/cosmetic choices only; credentials use the separate
private in-memory handoff. Status and admission publish atomically. The network
thread waits asynchronously for the ticket while the native window stays responsive.
Closing the window cancels account work and stops the owned server. No account I/O
or awaited transport is added to simulation.

Acceptance covers invalid/valid native sign-in, empty roster, abandoned creation,
eight saved characters, restart/restored appearance, same-process world entry,
checkbox/slider mouse and keyboard input, normal/small window layout, stable model
visibility and native GPU diagnostics. See STATUS for the final evidence and limits.
The Millhaven loading screen retains the supplied sunlit village with generated
gold MILLHAVEN lettering; its progress and Cancel remain functional native UI.

The reference-kit pass adds a shared textured navy panel sprite with double gold
trim and curling corner ornaments, dark navy and amber bevelled button sprites,
and a 24-icon gold atlas. Nine-slice rendering preserves corners across panel
sizes. Main Menu and bottom-right HUD shortcuts open existing game windows.
Inventory/Skills/Quest Journal/Party tabs show unsupported sections disabled;
character deletion, registration and password recovery remain disabled too.
The character roster keeps padded names and an amber selected row without an
external square focus outline. The narrower customization panel and inline
password eye retain existing account and cosmetic actions. Settings use a fixed
reference composition fitted independently to the window, keeping the complete
footer reachable on shorter displays without changing the preferred HUD scale.

23 November responsive preview refinement: Login, character selection and the
compact creation panel use the entire client window, including two/four-player
tiles. Background artwork preserves its proportions by centred cropping. Drag
left/right over the character's invisible bounding area to rotate it; gold curved
arrows at the feet indicate the gesture. The three buttons below provide zoom out,
reset view and zoom in (75–130%). Empty accounts show neither model nor controls.
Active menu text is white. No preset system or additional cosmetic capability is
introduced. See UI_SPRITE_INTEGRATION.md for evidence.
