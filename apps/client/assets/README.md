# Client assets

Bevy's default asset reader uses this directory during Cargo development runs.
The bootstrap scene currently generates its original meshes in Rust and uses
Bevy's embedded default font. No third-party game assets are included.

Qualify ordinary glTF/GLB loading here when the representative scene milestone
starts. Keep source Blender projects and export recipes alongside assets when
they are introduced; Bevy's asset pipeline replaces the old engine's cooker.

Prefer glTF 2.0, normally a self-contained `.glb`, for runtime deliveries. Retain
original source files and export settings. OBJ plus its MTL/textures can be an
intermediate source for static models; conversion cannot reconstruct absent UVs,
textures, rigs or animation. RAW heightmaps, if used, are terrain source data with
explicit dimensions/precision, not a general model format. Imported models and
the Bevy glTF loader are not yet enabled or qualified in this application.
