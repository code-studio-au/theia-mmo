# Native game entry

P3-01/P3-03 with P2-08/P2-09 presentation. The prerequisite is the existing owned
account service, PostgreSQL appearance migration and modular hero catalogue.
The front end runs in the same Bevy client/window as the eventual gameplay.

The normal account population launcher opens Login, then the account's saved
characters. New Character opens the seven-option creation form and animated
rotatable preview. Save Character durably creates the record and returns to the
roster. Enter World obtains the existing short-lived admission ticket and starts
the secure Renet connection. The existing world-readiness screen then admits
controls. Back abandons the unsaved appearance draft. Existing records are selected,
not cosmetically rewritten. Empty accounts have no invented placeholder characters.

The artwork follows the supplied sunset reference, with its painted adventurer
removed at the user’s request. The live preview is transparent and its projected
ground origin stays at the rock ledge (18.5% artwork width, 78.5% artwork height), including
rotation, window/UI-scale changes and aspect-preserving image gutters; live Bevy widgets replace its
painted form. See `content/ui/frontend/PROVENANCE.md` for the derivative and prompt.
Login launches the existing system-browser OIDC flow. Password entry, registration,
password recovery and a deployed account service are not implemented here.
The local editor wrapper's account/database lifetime remains unchanged.

`tools/play.py --game-front-end` requires one Classic account client and excludes
explicit character IDs and the old editor picker. The population launcher uses it
by default without an explicit character ID; `--editor-character-picker` retains
the previous qualification route. The client receives a private session directory.
Bounded JSON commands contain only public character IDs or cosmetic/name choices;
OIDC credentials stay in the Python account client. Status and admission publish
atomically. The network thread waits asynchronously for the one-use ticket while
the Bevy window remains responsive. Closing the window cancels account work and
stops the owned server through the existing supervisor. No account HTTP work or
awaited I/O is added to the simulation.

The preview uses standard Bevy GLTF, skeletal animation, palette materials and a
render-to-texture UI viewport. Its isolated render layer and explicit exclusion
from gameplay camera queries protect picking, projection, camera collision,
view-distance changes and HUD markers. World entry retains the established input
and readiness gate; waiting at Login does not consume its loading timeout.

Acceptance: bounded state/command validation; failed sign-in retry; invalid name;
create, select, preview and enter; database restart restores appearance; stable
native process/window identity from Login to gameplay; native GPU diagnostics;
fmt, denied-warning Clippy and relevant regression tests. These checks pass; final
native evidence is `build/game-front-end-v1/native-v5/qualification.json`. Fresh and
restarted PostgreSQL runs reach Playing with the selected saved appearance and
stable process/window identity within each run. Normal 1920x1080 and smaller
1720x700 layouts were inspected on RX 7900 XTX / RADV NAVI31 / Mesa 26.2.7 / Vulkan,
with zero final GPU diagnostics. See STATUS for exact suite counts. This is Linux
functional evidence, not Windows or performance qualification.

The title, crest, gold ornaments and original subtitle are a separate transparent
illustration (`content/ui/frontend/title.png`) restored at the user's request.
Native fonts are retained for functional form fields, option labels and buttons.

The latest Millhaven loading image is `content/ui/loading/millhaven-sunlit-v2.png`,
a derivative of the user-supplied sunlit village with matching gold MILLHAVEN
lettering. It is embedded for Millhaven's exact package IDs, including the imported
population village. The original download and older illustrations are retained.
The displayed progress and Cancel remain functional native loading controls.
