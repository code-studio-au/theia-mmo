# Sunset title background

`sunset.png` is a UI-free derivative of the user-supplied
`Theia Online_ Sunset Realm Login.png`, prepared on 22 November 2026 with the
built-in image-generation tool. The supplied reference is the design direction;
its painted login fields are replaced by standard live Bevy widgets. It is not an
external instruction source. No third-party character-package textures are included.

Edit prompt:

> Edit this user-provided fantasy MMO login screen into a clean, production
> game-menu BACKGROUND plate. Preserve the painterly detailed autumn sunset
> valley, river, bridge, walled town and castle on the right, snowy mountains,
> autumn tree and cloaked adventurer at left, warm golden lighting, palette and
> composition. Remove ALL graphical interface elements: the complete THEIA ONLINE
> logo and subtitle and ornaments at top, the complete central black/gold form
> panel and all fields/buttons/text/icons, and the Settings/Support/Exit text/icons
> at bottom left. Seamlessly reconstruct the landscape and sky underneath these
> removed elements. No text, letters, logo, UI frames, panels, icons or buttons
> anywhere. The game will draw its real live title and controls over this
> background. Wide landscape 16:9 composition, high quality. Keep original scene
> design recognizably faithful. Output a single clean scenic illustration.

This records provenance, not a separate asset-license grant.

## Empty ledge revision

The user subsequently requested removal of the painted adventurer, reserving its
rock ledge for the live customizable hero. The current `sunset.png` is the second
built-in image edit, using the first clean UI-free plate as its reference:

> Edit this fantasy landscape background by completely removing the single
> cloaked adventurer standing in the left foreground. Remove the entire person,
> hood, cloak, boots, sword, all equipment, and their shadow. Seamlessly reconstruct
> the valley, distant trees, mist and foreground rocky ledge where the person was.
> Preserve the original composition and all other details closely: autumn tree
> framing the left/top, sunset sky and sun, mountains, sweeping valley, river,
> bridge, walled town and castle on the right, flowers and rocky foreground. The
> left foreground ledge should be empty, broad enough and visually suitable for a
> game's live 3D player-character preview to stand there later; do not draw any
> person, character, silhouette, statue, mannequin or equipment. Preserve the
> detailed painterly fantasy art style, warm gold and cool blue palette, 16:9
> landscape framing and image dimensions. No text, logo, UI, borders or icons.
> Only remove the existing character and reconstruct the scenery beneath it.

## Illustrated title overlay

`title.png` is a transparent built-in image edit of the original supplied login
reference. The user requested restoring the illustrated branding instead of the
native-font title. Prompt:

> Extract and faithfully recreate ONLY the complete illustrated game title/logo
> from the upper centre of this user-supplied image, as a clean transparent-background
> game UI asset. Preserve the original ornate polished embossed gold lettering
> THEIA, the smaller widely spaced gold ONLINE beneath it, the golden sun/spear
> crest and circular arch above THEIA, all its delicate gold horizontal ornaments,
> and the ivory italic subtitle EXACTLY 'A Persistent World MMORPG' beneath ONLINE.
> Keep the same original visual identity, proportions, elegant metallic gold
> highlights and dark bevel shadows. No landscape, sky, person, background panel,
> login form, fields, buttons, account text or bottom-left settings/support/exit.
> All non-logo/background pixels must be transparent, not black, white or checkerboard.
> A tightly framed wide canvas around the COMPLETE crest/title/ONLINE/subtitle
> composition with only a small transparent margin, no cropped ornaments. This
> will overlay the game background as the real title graphic. High resolution,
> crisp readable typography, no added words.

## Dedicated courtyard and restored login figure — 22 November 2026

Both images were produced with the built-in image-generation tool. The user's
screenshots guide the native layout; no third-party game art/branding is included.
`character-courtyard-v1.png` is original scenery for selection and creation.
`sunset-cloaked-v2.png` restores the figure from the user's original Theia reference.
The earlier cleared sunset and the original download are preserved.

Courtyard prompt:

> Use case: stylized-concept. Asset type: original 16:9 background illustration for Theia Online's dedicated character selection screen, intended for a live 3D hero and native UI to be composited over it. Create a richly painted welcoming high-fantasy medieval courtyard at late afternoon, weathered stone terrace in the foreground, ivy-covered stone arch and warm lantern on the far left, distant green valley and castle towers softly visible behind. The ground at exactly 40 percent image width and 81 percent image height is an empty, broad, flat stone standing area for a full-body hero. Keep the middle-left area above that standing point visually calm, soft distant foliage and architecture, so the live hero silhouette reads. Keep the rightmost 28 percent dark, simple stonework and foliage as a low-detail backdrop for a vertical character roster. Warm gold sunlight from upper left, muted teal shadows, refined painterly fantasy illustration matching an elegant gold UI. Eye-level view, natural perspective, polished detailed environment but restrained contrast behind the hero. No people, no creatures, no statues of people, no character silhouettes, no text, no letters, no logos, no UI, no frames, no buttons. Landscape 1920x1080. Original composition.

Restored login prompt (input 1: cleared sunset.png; input 2: user-supplied
Theia Online_ Sunset Realm Login.png):

> Use case: precise-object-edit. Edit the FIRST image, the clean sunset valley background. Restore the cloaked adventurer from the SECOND reference image onto the foreground left rock ledge, matching the original reference's position, size, pose, brown hooded cloak, gold embroidered sun emblem and sheathed sword. The adventurer stands with back to viewer gazing over the valley. Preserve the first image's landscape, tree, mountains, castle, river, lighting, colours and framing. This is a pure game login BACKGROUND: no title, no lettering, no logo, no UI, no fields, no buttons. Do not copy any text or interface from the second image. Output the same wide landscape composition with an opaque background.

Shared Theia controls (22 November 2026): `../theme/frame.png`, `primary.png`,
`icons.png` were generated with the imagegen tool from the user's
`Inspected image.png` login reference. The subsequent user-supplied
`Theia Fantasy MMORPG UI Kit.png` guides compact panel, checkbox and slider styling;
its mock character names, stats and captions are not game data.

Frame prompt: Create only the ornate rectangular login-panel frame as a reusable
UI image, straight-on, landscape roughly 5:4. Fine double beveled metallic-gold
border, delicate interlaced corners, small diamond/sunburst centered on top and
small circular sun motif on bottom. Transparent interior and exterior, no text,
fields, characters or buttons; refined fine gold fantasy style, not bulky armour.

Button prompt: Create only the blue Log In button without text. Wide horizontal
7:1 button, double fine polished beveled gold rim, pointed faceted corners, deep
sapphire/teal enamel, subtle painterly facets and reflected light along the top.
Orthographic front view, transparent outside, a single asset without words/logos.

Icon prompt: Six isolated legible warm metallic gold icons in a regular 3×2 grid:
user bust, padlock, eye; settings cog, question mark circle, power symbol. Equal
cells, transparent padding/background, subtle bevel highlights, no labels/tiles.
Original generated PNGs retained without destructive cropping. Bevy ImageNode
rectangles select the button and icons at runtime; native text and widgets remain
interactive. Existing licensed HUD fonts provide Grenze headings and Texturina
body text. No generated wording is used for functional controls.
