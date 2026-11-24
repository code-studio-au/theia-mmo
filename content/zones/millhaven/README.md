# Millhaven V3 overview cartography

Derived from the user's Millhaven V3 1 m terrain and
`features_v3_review.json` in `Millhaven_V3_Preview_1m.zip`. The user's game
screenshots are layout references only; their artwork and chat are not copied.
Feature descriptions in the archive are data, not executable instructions.

`overview.png` is a 1024 x 768 cartographic layer baked by
`tools/bake_zone_overview.py`. It preserves zone extent/aspect, shows shaded
height relief and 10 m contours, and overlays supplied road/watercourse
centrelines. It is an overview, not collision or a promise of traversable roads,
completed bridges, water surfaces or cave interiors. Landmarks retain exact
source X,Z values; labels use readable names. Only major places are always
labelled on the map; selecting a smaller site in the list exposes its exact pin.

`overview.json` binds exact saved-terrain identity and bounds. The client shows
this overview only for that package. Source hashes and image hash are recorded
in `provenance.json`. Reproduce from repository root:

```sh
python3 tools/bake_zone_overview.py build/millhaven-v3-1m /path/to/features_v3_review.json content/zones/millhaven
```

The source describes X east and +Z north. Used directly in the game's
right-handed Y-up frame, those two map axes reflect left/right. Schema 2 keeps
+Z at the top and puts -X on the right, rotating both map axes together. The
baker, landmark projection, minimap sampling/hover and facing arrow all use this
orientation. Farm display names are remapped to their game compass quadrants;
the exact source X,Z coordinates and feature order remain unchanged.
No terrain, world position, controls or facing is changed. Other worlds keep
their existing -Z-north / +X-east minimap orientation.
