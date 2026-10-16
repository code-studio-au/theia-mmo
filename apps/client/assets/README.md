# Client assets

Bevy's default asset reader uses this directory during Cargo development runs.
The game scene generates original meshes in Rust and uses Bevy's embedded font.
The separate `reference/` folder now contains the user's compatibility test model;
see its README for provenance and missing UVs/materials.

Geometry-only GLB loading is qualified in a separate inspector. Keep source
Blender projects and export recipes alongside assets when
they are introduced; Bevy's asset pipeline replaces the old engine's cooker.

Prefer glTF 2.0, normally a self-contained `.glb`, for runtime deliveries. Retain
original source files and export settings. OBJ plus its MTL/textures can be an
intermediate source for static models; conversion cannot reconstruct absent UVs,
textures, rigs or animation. RAW heightmaps, if used, are terrain source data with
explicit dimensions/precision, not a general model format. Textures, animation,
general asset compatibility and gameplay collision are not qualified by the current
geometry-only reference. See [the import guide](../../../docs/ASSET_IMPORT.md).
