# Client assets

Bevy's default asset reader uses this directory during Cargo development runs.
The bootstrap scene currently generates its original meshes in Rust and uses
Bevy's embedded default font. No third-party game assets are included.

Qualify ordinary glTF/GLB loading here when the representative scene milestone
starts. Keep source Blender projects and export recipes alongside assets when
they are introduced; Bevy's asset pipeline replaces the old engine's cooker.
