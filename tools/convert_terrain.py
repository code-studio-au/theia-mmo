#!/usr/bin/env python3
"""Convert one textured wow.export terrain OBJ to a separate inspection GLB.

Uses trimesh's standard OBJ/glTF exchange. Does not import placements or liquids.
Install tools/requirements-assets.txt in an isolated environment first.
"""
import argparse
import hashlib
import json
from pathlib import Path

import trimesh


def convert(source, output):
    if output.exists() or source.resolve() == output.resolve():
        raise ValueError('Output must be a separate new file')
    # Terrain qualification requires all authored materials/textures, not defaults.
    libraries, used, triangles = [], set(), 0
    with source.open() as file:
        for line in file:
            parts = line.split()
            if not parts:
                continue
            if parts[0] == 'mtllib':
                libraries.extend(parts[1:])
            elif parts[0] == 'usemtl':
                used.add(parts[1])
            elif parts[0] == 'f':
                if len(parts) != 4:
                    raise ValueError('Expected triangulated terrain faces')
                triangles += 1
    inputs = {source.resolve()}
    materials, diffuse = {}, {}
    for library in libraries:
        path = source.parent / library
        inputs.add(path.resolve())
        current = None
        with path.open() as file:
            for line in file:
                parts = line.strip().split(maxsplit=1)
                if not parts:
                    continue
                if parts[0] == 'newmtl':
                    current = parts[1]
                elif parts[0] == 'map_Kd':
                    texture = (path.parent / parts[1]).resolve()
                    if not texture.is_file():
                        raise ValueError(f'Missing texture: {texture}')
                    materials[current] = texture
                    inputs.add(texture)
                elif parts[0] == 'Kd':
                    diffuse[current] = [float(v) for v in parts[1].split()]
    if not used or not used.issubset(materials):
        raise ValueError('Every terrain material must have an available diffuse texture')
    hashes = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(inputs)}
    scene = trimesh.load_scene(source, process=False, group_material=True)
    if len(scene.geometry) != len(used) or sum(len(m.faces) for m in scene.geometry.values()) != triangles:
        raise ValueError('Conversion changed material grouping or triangle count')
    for mesh in scene.geometry.values():
        if mesh.visual.kind != 'texture' or mesh.visual.uv is None or mesh.visual.material.image is None:
            raise ValueError('Conversion lost UVs or a terrain texture')
        material = mesh.visual.material.to_pbr()
        # A texture-only wow.export MTL has no authored tint. Avoid the exchange
        # library's grey fallback, while preserving an explicitly supplied Kd.
        tint = diffuse.get(mesh.visual.material.name, [1., 1., 1.])
        material.baseColorFactor = [*tint, 1.]
        material.metallicFactor, material.roughnessFactor = 0., 1.
        mesh.visual.material = material
    data = trimesh.exchange.gltf.export_glb(scene, include_normals=True)
    if any(hashlib.sha256(Path(p).read_bytes()).hexdigest() != h for p, h in hashes.items()):
        raise ValueError('Source changed during conversion; retry after export completes')
    with output.open('xb') as file:
        file.write(data)
    return {'source': str(source.resolve()), 'output': str(output.resolve()),
            'source_hashes': hashes, 'sha256': hashlib.sha256(data).hexdigest(),
            'bytes': len(data), 'triangles': triangles, 'materials': len(used),
            'bounds': scene.bounds.tolist(), 'trimesh': trimesh.__version__,
            'scope': 'textured terrain geometry only; no placements, liquids or gameplay collision'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(convert(args.source, args.output), indent=2))
