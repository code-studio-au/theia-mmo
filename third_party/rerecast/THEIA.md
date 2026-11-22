This is an altered local copy of Rerecast 0.5.0 from crates.io.
Original crate checksum: 8e800c409bdb4f5762360b647a4ccfa3793a630ab2258a5d8d6c027c83b02317.
Original authorship and MIT/Apache licenses are retained.

Theia modification, 22 November 2026: in src/poly_mesh.rs, remove the two
u16 casts of squared XZ diagonal distances in triangulate. Keep i32 distances,
matching Recast Navigation v1.6.0 RecastMesh.cpp (triangulate, lines 344–346
and 376–378):
https://github.com/recastnavigation/recastnavigation/blob/v1.6.0/Recast/Source/RecastMesh.cpp

A 128 m Millhaven navigation tile at 0.2 m cells has edges spanning hundreds
of cells. Squared distances exceed 65,535; truncation changed ear selection
and rejected a valid contour with three obstacle holes. The same local contour
fails in unmodified 0.5.0 and triangulates with these two casts removed. Geometry,
obstacles, hole joining, region settings and simplification tolerances are unchanged.
Asset-derived diagnostic coordinates remain in ignored build output only.
The existing bounded navigation input and output limits remain in force. This
patch does not claim arbitrary u16-coordinate arithmetic is overflow-safe.
