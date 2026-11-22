# Native game entry and shared Theia UI

P3-01/P3-03 with P2-08/P2-09 and P2-14 presentation. Prerequisites are the
implemented account service, PostgreSQL appearance migration, modular hero
catalogue and standard Bevy UI. Login, character selection, creation and gameplay
share one native client/window.

The Login screen restores the painted cloaked adventurer and illustrated Theia
Online title. Username and masked password are live native inputs. Settings,
Support and Exit are native controls; no browser login button or browser fallback
is available in this game flow. Errors appear only when needed. Passwords are
bounded, cleared after submission, remasked on focus loss and never included in
widget telemetry, command files, process arguments or logs. Password editing
supports Unicode, selection, cursor movement and deletion; clipboard paste is not
yet provided for this custom masked input.

The current working sign-in implementation is an adapter for the owned **local
Keycloak development provider**. The native client sends credentials in memory to
an ephemeral loopback listener using a private capability. The adapter accepts only
the configured local issuer's expected sign-in form and callback; it retains the
existing authorization-code, PKCE, state/nonce and server token validation. Other
providers or MFA steps fail with an in-game message. This does not claim production
native authentication, registration, recovery or a deployed account service. The
local wrapper still owns its temporary provider/database lifetime. Historical
editor/console account routes retain their separate browser-based qualification.

The user adopted a future launcher that updates/starts the client, with login
remaining inside the game. An update distribution service/patching launcher has
not been implemented by this UI slice.

Character selection has its own courtyard background, right-side account roster,
selected live hero, rotation controls and Enter World. An empty account has no
hero and cannot enter. New Character opens the existing seven-option creation
form. Back discards the draft; Save durably creates a server-owned character.
The roster supports eight saved entries and selects actual returned records;
no invented classes, levels or locations appear. Enter World obtains the existing
short-lived admission ticket and starts authenticated Renet. Normal world loading
then admits controls. Empty accounts and returning from abandoned creation never
show the default preview character.

The preview uses standard Bevy glTF, skeletal animation, materials and a transparent
render-to-texture viewport on an isolated render layer. Its ground origin is
anchored at 40% of courtyard width and 81% of height across window/UI scale changes.
Once a preview has settled, unrelated shader compilation cannot hide it. Page and
appearance changes still reset readiness. Gameplay camera queries exclude the
preview. Waiting at Login does not consume the gameplay loading timeout.

The shared skin, installed with the existing `--fantasy-ui` presentation option,
is in `apps/client/src/ui_theme.rs`. It provides gold frames, sapphire
primary buttons, dark secondary buttons, cream text, hover/pressed/disabled/focus
states, gold icon links and input surfaces. It decorates existing native controls
without changing gameplay actions. Inventory, dialogue, quests, documents, party,
map, ability book and settings share the frame/button treatment; item and action
icons retain their existing assets. Parchment dialogue keeps its readable ink body.
Grenze headings/large labels and Texturina smaller body text come from the
existing licensed HUD pack; inputs use the same native font set. Artwork provenance is in `content/ui/frontend/PROVENANCE.md`.

Both front-end and game settings expose the same accessible VSync checkbox and
interface-size slider (75–150%). They update existing persisted preferences, not a
second settings store. Slider arrows, Home/End and checkbox Space/Enter work through
Bevy's standard headless widgets. View distance and FPS limits retain their actual
existing controls. Decorative images ignore picking. The zone editor is separate
from this game UI skin.

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
