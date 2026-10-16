# Local compatibility reference

`duskwoodhouse_medium.glb` was supplied by the user from Downloads and moved here
after actual Bevy rendering succeeded, as requested. The user identifies wow.export
as the source tool; the file reports `Khronos glTF Blender I/O v4.2.57` as its last
exporter. It is reference content for local import testing, not original Theia
game content. No asset license was supplied. Nothing has been published/pushed.

Original bytes are preserved, SHA-256:
`3e3fbcca525a8f4b20381b650315cce62956baecf629bd8c6fd202d960946aa6`.

The 9884-byte file contains one mesh, 425 source vertices and 672 triangles.
It has no authored normals, UVs, materials, images, skin or animation. Bevy
generates flat normals and uses a default material; that does not reconstruct
textures. Source bounds span approximately 18.43 x 16.78 x 25.01 units, with the
minimum Y at -0.823. Unit scale has not been independently established.

The inspector preserves all source transforms/scale, fits the camera and floor to
actual instantiated mesh bounds, and places a separate 1.8-unit capsule beside it.
There is no gameplay movement or collision in this inspection mode. The existing
authoritative world and its fixed geometry are separate.
