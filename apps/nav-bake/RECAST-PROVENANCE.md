`src/contour_holes.rs` is an altered Rust adaptation of contour-hole joining in
[Recast Navigation v1.6.0 RecastContour.cpp](https://github.com/recastnavigation/recastnavigation/blob/v1.6.0/Recast/Source/RecastContour.cpp).
The original copyright and zlib license are retained in `LICENSE-RECAST`.

Rerecast 0.5.0 remains an unmodified registry dependency. Its contour builder
does not join hole contours before polygon triangulation. The dressed starter
valley exposed two oppositely wound inner contours and an `InvalidContour` bake
failure. The adapter groups holes by region and area, sorts them left to right,
chooses a visible short bridge, and doubles the bridge endpoints as Recast does.
It retains the original height/region flags and all obstacle boundaries.

Differences from C++: i64 predicates/distance; deterministic distance ties;
8,192 contour vertex and 8,000,000 edge-check limits; ambiguous/missing outlines
or unbridgeable holes fail the bake instead of logging and continuing. Visibility
uses the actual candidate endpoints (the v1.6.0 source's `diags[i]` edge exclusion
is not copied). No new renderer, runtime path service or backend fork is added.
Qualification and remaining limitations are recorded in `docs/STATUS.md` at
the repository root; no asset-derived contour coordinates are distributed.
