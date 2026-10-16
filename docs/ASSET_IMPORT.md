# GLB building/prop inspection

The client now enables Bevy 0.20.0's standard glTF loader, world-scene support,
automatic reflected type registration and PNG/JPEG decoders. It does not introduce
a custom runtime model format or depend on the old engine's asset cooker.

## Inspect an asset

```sh
cargo run --locked -p theia-client -- --inspect-asset apps/client/assets/reference/duskwoodhouse_medium.glb
```

The inspector is an independent mode: original model geometry, directional light,
neutral floor, bounds/counts and a 1.8-unit blue capsule for scale comparison. Hold
right mouse to orbit, use the wheel to zoom, Home to reframe and Escape to exit.
It keeps source transforms and units rather than rescaling the model to fit.
The asset root is the selected file's parent, so ordinary relative dependencies
can resolve through Bevy. Missing/decode-failed dependencies cause failure; they
are not silently replaced with claimed working textures. Only this self-contained
geometry-only reference has been qualified so far.

Capture through an actual graphical display or a prepared native headless display:

```sh
cargo run --locked -p theia-client -- --inspect-asset apps/client/assets/reference/duskwoodhouse_medium.glb --capture build/asset-check
python3 tools/verify_asset.py build/asset-check apps/client/assets/reference/duskwoodhouse_medium.glb
```

The capture waits for actual scene instantiation, propagated mesh bounds and a
rendering warmup, then writes `asset.json` and the real `scene.png`. The image check
is tailored to the current untextured reference; it is not a fidelity oracle for
arbitrary textured assets. Inspect the PNG and diagnostic logs too. Invalid GLB
headers fail before opening a window; loader/dependency errors and missing scenes
exit unsuccessfully. A successful screenshot is never substituted for load failure.

The game remains the existing procedural authoritative world. Imported visuals
do not automatically create server colliders or replace world geometry. An asset
needs explicit reviewed placement/collision before it enters that world. Environment
items remain static; players never physically block players/NPCs.

## Current reference and missing textures

The user's `duskwoodhouse_medium.glb` renders successfully, but contains no UVs,
material slots or images. It also lacks authored normals, so Bevy creates flat
normals, expanding the 425 source positions to 2016 render vertices. There are
672 triangles. The model is plain shaded; this does not qualify textured import,
skeletal animation, collision, WoW shaders or complete game-asset compatibility.
Source/provenance/hash are recorded in `assets/reference/README.md` under the client.

The supplied ZIP contains only the same GLB. No loose BLP files were found in
Downloads during inspection. Attaching converted textures alone cannot restore
the missing UV mapping/material assignments; a source export that retains them
is needed.

## Exporting from wow.export

Prefer a direct **GLB** visual-model export with textures included if the installed
version offers it. The project's [changelog](https://github.com/Kruithne/wow.export/blob/main/CHANGELOG.md)
records GLB resource packing from version 0.2.3. Exact UI/settings depend on the
installed version. A render model and a collision export are distinct outputs;
do not substitute a simplified collision mesh for the visual model.

If using Blender as an intermediate, use OBJ **with** its MTL and textures, not
STL. Check that UVs/materials exist and the model displays correctly in Blender,
then export glTF 2.0 Binary with materials/textures included. Prefer PNG/JPEG for
this tested loader configuration. BLP images can be converted offline; [Pillow's
format documentation](https://pillow.readthedocs.io/en/stable/handbook/image-file-formats.html#blp)
describes its decoder support. No BLP samples were available for actual conversion
qualification in this run. Do not claim conversion recreates missing UVs/materials.

Keep source files/export settings, establish metre scale and a sensible placement
pivot, and keep doors/props static. GLB is the delivery target; `.blend`, OBJ/MTL,
BLP and any terrain heightmaps remain source/intermediate data. Test original or
appropriately licensed representative art separately from compatibility references.
