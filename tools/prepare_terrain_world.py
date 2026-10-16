#!/usr/bin/env python3
"""Prepare a bounded terrain-only walking fixture; never modify the source export."""
import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import trimesh


def prepare(source, output, center):
    if output.exists():
        raise ValueError('Output directory must be new')
    center = np.asarray(center, dtype=float)
    if center.shape != (3,) or not np.isfinite(center).all():
        raise ValueError('Center must contain three finite coordinates')
    # Fail explicitly on missing dependencies; do not accept a loader's fallback
    # material. Hash the export dependencies before and after conversion.
    inputs = {source.resolve()}
    used, textures, diffuse = set(), {}, {}
    libraries = []
    for line in source.read_text().splitlines():
        parts = line.split()
        if not parts: continue
        if parts[0] == 'mtllib': libraries.extend(parts[1:])
        if parts[0] == 'usemtl': used.add(parts[1])
        if parts[0] == 'f' and len(parts) != 4:
            raise ValueError('Terrain must be triangulated')
    for library in libraries:
        path = source.parent / library
        inputs.add(path.resolve())
        current = None
        for line in path.read_text().splitlines():
            parts = line.strip().split(maxsplit=1)
            if not parts: continue
            if parts[0] == 'newmtl': current = parts[1]
            if parts[0] == 'Kd': diffuse[current] = [float(v) for v in parts[1].split()]
            if parts[0] == 'map_Kd':
                texture = (path.parent / parts[1]).resolve()
                if not texture.is_file(): raise ValueError(f'Missing terrain texture: {texture}')
                textures[current] = texture
                inputs.add(texture)
    if not used or not used.issubset(textures):
        raise ValueError('Every terrain material needs an available texture')
    hashes = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(inputs)}
    original = trimesh.load_scene(source, process=False, group_material=True)
    scene = trimesh.Scene()
    triangles = []
    for name, mesh in original.geometry.items():
        positions = mesh.triangles - center
        # Retain complete source triangles crossing the 48m visual patch. Legal
        # movement is restricted to +/-19m, leaving a visible camera margin.
        selected = np.all(positions[:, :, [0, 2]].max(axis=1) >= -24, axis=1)
        selected &= np.all(positions[:, :, [0, 2]].min(axis=1) <= 24, axis=1)
        if not selected.any():
            continue
        if mesh.visual.kind != 'texture' or mesh.visual.material.image is None:
            raise ValueError('Missing terrain texture')
        used, inverse = np.unique(mesh.faces[selected], return_inverse=True)
        part = trimesh.Trimesh(
            vertices=np.asarray(mesh.vertices[used] - center, dtype=np.float32),
            faces=inverse.reshape((-1, 3)), vertex_normals=mesh.vertex_normals[used],
            visual=trimesh.visual.TextureVisuals(uv=mesh.visual.uv[used], material=mesh.visual.material),
            process=False)
        material = part.visual.material.to_pbr()
        material.baseColorFactor = [*diffuse.get(mesh.visual.material.name, [1., 1., 1.]), 1.]
        material.metallicFactor, material.roughnessFactor = 0., 1.
        part.visual.material = material
        triangles.extend(part.triangles.tolist())
        scene.add_geometry(part, geom_name=name)
    if not triangles or len(triangles) > 8192:
        raise ValueError('Expected a small nonempty terrain patch')
    glb = trimesh.exchange.gltf.export_glb(scene, include_normals=True)
    profile = {'schema': 1, 'source_origin': center.tolist(),
               'visual_sha256': list(hashlib.sha256(glb).digest()), 'triangles': triangles}
    if any(hashlib.sha256(Path(p).read_bytes()).hexdigest() != h for p, h in hashes.items()):
        raise ValueError('Source changed while preparing terrain')
    output.mkdir(parents=True)
    (output / 'terrain.glb').write_bytes(glb)
    data = (json.dumps(profile, separators=(',', ':'), allow_nan=False) + '\n').encode()
    (output / 'terrain.json').write_bytes(data)
    report = {'source': str(source.resolve()), 'source_sha256': hashes[str(source.resolve())], 'source_hashes': hashes,
              'origin': center.tolist(), 'triangles': len(triangles), 'materials': len(scene.geometry),
              'visual_bytes': len(glb), 'world_sha256': hashlib.sha256(data).hexdigest(),
              'scope': 'terrain-only local walking patch; no map objects, liquids or character collision'}
    (output / 'preparation.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--center', required=True, help='source X,Y,Z origin in metres')
    args = parser.parse_args()
    print(json.dumps(prepare(args.source, args.output, [float(v) for v in args.center.split(',')]), indent=2))
