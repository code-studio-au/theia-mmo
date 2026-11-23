# Character customisation reference pass

P3-03 / P2-14, 23 November 2026. Prerequisites: native account roster/admission,
versioned seven-field cosmetic catalogue, modular hero, approved v3 UI sprites and
standard Bevy preview camera. The user now requests the supplied reference layout,
superseding the earlier compact-form-only restriction. Presets remain omitted.

The creation screen has one category rail (Hair, Face, Features and Outfit), a
persistent Appearance panel for body and skin tone, a central live character,
category-specific choice tiles on the right and fixed bottom actions. Randomize
changes only valid cosmetic choices. The name field, inline invalid-name message,
Back and Create Character remain native controls using existing account commands.
No appearance protocol, persistence schema or gameplay attributes change.

Choice tiles select exact catalogue values. Body changes clear unsupported facial
hair. Race, markings, additional body shape/height/build, highlights, accessories,
eye colour and scars remain visibly disabled. There are no duplicated category
tabs, preset controls or invented cosmetic options. Existing saved characters
retain their values and the roster keeps its separate composition.

The preview ground point is centred between the appearance panels. Its invisible
horizontal-drag area stays in that gap; gold foot arrows and zoom/reset sit above
the bottom actions. Login and roster retain their existing window-relative layout.

`tools/render_character_thumbnails.py` renders a 32-cell, 2048×1024 transparent atlas
from the actual private `character-variants.glb`, using Blender CPU Cycles and the
same part selection as the converter. Neutral thumbnails retain source UV details;
body, face, hair, facial-hair and outfit tiles use the correct body variant. The
atlas and its source hash/provenance stay beside private player assets under build,
not in Git. Run:

```
build/blender-tools/bin/python tools/render_character_thumbnails.py \
  --players build/modular-hero-v1/art-v3
```

The client loads the optional atlas through its standard players asset source.
Without an exported atlas, labelled choices remain usable. No custom renderer or
per-thumbnail live 3D cameras are added.

Validation: 357 client tests pass (five existing skips), formatting, denied-warning
all-target Clippy and matched release builds. Exact logs:
`build/character-layout-v3-tests-final.log`, `character-layout-v3-clippy-final.log`
and `character-layout-v3-build-final.log`.

The complete native fresh/restarted account run passes in
`build/character-layout-native-v2/qualification.json`: normal native login,
category/choice selection, body/facial-hair constraints, randomization, inline name
validation, eight-character roster, saved appearance restoration and actual
same-window world entry after PostgreSQL restart. Screenshots identified footer
links and tile caption spacing for follow-up. The final visual follow-up passes
in `build/character-layout-native-v4/qualification.json`, covering creation and
roster at 3440×1400, 1720×1400 and 1720×700, drag/release, zoom/reset, new icons and
tile padding. These are one client's allocation-size checks, not two/four-account
load tests. The final follow-up intentionally skips unchanged login/settings and
world-entry checks; their earlier scope remains separate. Screenshots were reviewed.

Linux AMD Radeon RX 7900 XTX / RADV NAVI31 / Mesa 26.2.7 / Vulkan; zero GPU
diagnostics in both accepted native runs. Native v1 exposed a duplicate heading
component, fixed and covered by the widget-tree regression. Native v3 stopped on
desktop focus loss and is not accepted qualification. No Windows, additional
cosmetic catalogue, preview stability during palette loading or extreme UI-scale
layout claim is made by this slice.

Next user request: native Create Account matching the supplied gateway reference,
with actual account-service integration, bounded masked inputs, validation and
unsupported links/options disabled. Open the qualified current build full screen
for the user while implementing that follow-up.
