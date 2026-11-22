`src/contour_holes.rs` is an altered Rust adaptation of contour-hole joining in
[Recast Navigation v1.6.0 RecastContour.cpp](https://github.com/recastnavigation/recastnavigation/blob/v1.6.0/Recast/Source/RecastContour.cpp).
The original copyright and zlib license are retained in `LICENSE-RECAST`.

Rerecast 0.5.0 now has a two-cast squared-distance correction in a licensed local
copy; see `../../third_party/rerecast/THEIA.md` for evidence and exact provenance. Its contour builder
does not join hole contours before polygon triangulation. The dressed starter
valley exposed two oppositely wound inner contours and an `InvalidContour` bake
failure. The adapter groups holes by region and area, sorts them left to right,
chooses a visible short bridge, and doubles the bridge endpoints as Recast does.
It retains the original height/region flags and all obstacle boundaries.

Differences from C++: i64 predicates/distance; deterministic distance ties;
8,192 contour vertex and 8,000,000 edge-check limits; ambiguous/missing outlines
or unbridgeable holes fail the bake instead of logging and continuing. Visibility
uses the actual candidate endpoints (the v1.6.0 source's `diags[i]` edge exclusion
is not copied). No new renderer or runtime path service is added. The two-cast
Rerecast correction is a local backend patch as documented above.
Qualification and remaining limitations are recorded in `docs/STATUS.md` at
the repository root; no asset-derived contour coordinates are distributed.

The runtime static query also uses the licensed local Landmass 0.9.3 copy in
`../../third_party/landmass`. Its existing island-edge link tolerance is exposed
as an option, retaining the upstream 0.01 m default. Theia bounds the tolerance
to two height cells, at most 0.1 m and the admitted step height, with the original
0.01 m minimum. This admits independently quantized shared terrain edges while
retaining strict layer sampling and separate collision/clearance validation.
See `../../third_party/landmass/THEIA.md` for the exact change and checksum.
