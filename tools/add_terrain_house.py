#!/usr/bin/env python3
"""Add one authored static farmhouse/foundation to a bounded terrain fixture.

The conservative envelope is explicit fixture data, never an inferred mesh collider.
Source GLBs and all embedded binary payloads remain unchanged.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

import numpy as np
import trimesh
from merge_static_glb import read, merge_documents


def write_glb(doc, binary, output):
    text = json.dumps(doc, separators=(',', ':'), allow_nan=False).encode()
    text += b' ' * (-len(text) % 4)
    with output.open('xb') as file:
        file.write(struct.pack('<4sII', b'glTF', 2, 28 + len(text) + len(binary)))
        file.write(struct.pack('<I4s', len(text), b'JSON'))
        file.write(text)
        file.write(struct.pack('<I4s', len(binary), b'BIN\0'))
        file.write(binary)


def clipped_heights(triangles, low, high):
    """Exact linear triangle/footprint intersection; no decoded heightmap guess."""
    heights = []
    for triangle in triangles:
        polygon = [np.asarray(v, dtype=float) for v in triangle]
        for axis, edge, sign in [(0, low, 1), (0, high, -1), (2, low, 1), (2, high, -1)]:
            result = []
            for previous, current in zip(polygon[-1:] + polygon[:-1], polygon):
                a, b = sign * (previous[axis] - edge), sign * (current[axis] - edge)
                if (a >= 0) != (b >= 0):
                    result.append(previous + (current - previous) * (a / (a - b)))
                if b >= 0:
                    result.append(current)
            polygon = result
        heights.extend(float(v[1]) for v in polygon)
    if not heights:
        raise ValueError('No terrain under authored house footprint')
    return min(heights), max(heights)


def add_house(terrain, house, output):
    if output.exists():
        raise ValueError('Output directory must be new')
    inputs = [terrain / 'terrain.json', terrain / 'terrain.glb', house]
    hashes = {str(p.resolve()): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    profile = json.loads(inputs[0].read_text())
    if profile['schema'] != 1 or profile.get('solids'):
        raise ValueError('Expected a terrain-only schema-1 fixture')
    if bytes(profile['visual_sha256']).hex() != hashes[str(inputs[1].resolve())]:
        raise ValueError('Terrain visual hash mismatch')
    ground, top = clipped_heights(profile['triangles'], -12.3, -3.7)
    bottom = ground - 0.1
    house_doc, house_bin = read(house)
    bounds = trimesh.load_scene(house, process=False).bounds
    if bounds is None or not np.isfinite(bounds).all() or np.any(bounds[1] <= bounds[0]):
        raise ValueError('House must have finite nonzero 3D bounds')
    scale = float(np.min(np.array([8., 6.8, 8.]) / (bounds[1] - bounds[0])))
    matrix = np.eye(4)
    matrix[:3, :3] *= scale
    matrix[:3, 3] = [-8. - bounds[:, 0].mean() * scale,
                        top - bounds[0, 1] * scale,
                        -8. - bounds[:, 2].mean() * scale]
    roots = house_doc['scenes'][0].get('nodes', [])
    house_doc['nodes'].append({'name': 'authored_static_farmhouse', 'children': roots,
                               'matrix': matrix.T.reshape(-1).tolist()})
    house_doc['scenes'][0]['nodes'] = [len(house_doc['nodes']) - 1]
    foundation = trimesh.creation.box(extents=[8.6, top - bottom, 8.6])
    foundation.apply_translation([-8., (top + bottom) / 2., -8.])
    foundation = trimesh.Trimesh(vertices=foundation.triangles.reshape((-1, 3)),
        faces=np.arange(len(foundation.faces) * 3).reshape((-1, 3)),
        vertex_normals=np.repeat(foundation.face_normals, 3, axis=0), process=False)
    foundation.visual = trimesh.visual.TextureVisuals(material=trimesh.visual.material.PBRMaterial(
        baseColorFactor=[0.22, 0.24, 0.26, 1.], metallicFactor=0., roughnessFactor=1.))
    foundation_glb = trimesh.exchange.gltf.export_glb(trimesh.Scene(foundation), include_normals=True)
    # Reuse the narrow existing static merger; terrain/house buffers are copied.
    output.mkdir(parents=True)
    write_glb(house_doc, house_bin, output / 'placed-house.glb')
    (output / 'foundation.glb').write_bytes(foundation_glb)
    terrain_doc, terrain_bin = read(inputs[1])
    foundation_doc, foundation_bin = read(output / 'foundation.glb')
    merged = merge_documents(terrain_doc, house_doc, len(terrain_bin))
    merged = merge_documents(merged, foundation_doc, len(terrain_bin) + len(house_bin))
    binary = terrain_bin + house_bin + foundation_bin
    merged['buffers'][0]['byteLength'] = len(binary)
    write_glb(merged, binary, output / 'terrain.glb')
    ceiling = top + 6.8
    profile['solids'] = [{'center': [-8., (bottom + ceiling) / 2., -8.],
                          'size': [8.6, ceiling - bottom, 8.6]}]
    profile['visual_sha256'] = list(hashlib.sha256((output / 'terrain.glb').read_bytes()).digest())
    (output / 'terrain.json').write_text(json.dumps(profile, separators=(',', ':'), allow_nan=False) + '\n')
    if any(hashlib.sha256(Path(p).read_bytes()).hexdigest() != h for p, h in hashes.items()):
        raise ValueError('Input changed during assembly; discard derived output')
    report = {'source_hashes': hashes, 'house_source_bounds': bounds.tolist(), 'house_transform': matrix.tolist(),
              'foundation_y': [bottom, top], 'solid': profile['solids'][0],
              'terrain_binary_preserved': binary[:len(terrain_bin)] == terrain_bin,
              'house_binary_preserved': binary[len(terrain_bin):len(terrain_bin) + len(house_bin)] == house_bin,
              'visual_sha256': bytes(profile['visual_sha256']).hex(),
              'profile_sha256': hashlib.sha256((output / 'terrain.json').read_bytes()).hexdigest(),
              'visual_bytes': (output / 'terrain.glb').stat().st_size,
              'scope': 'authored solid farmhouse and foundation on bounded terrain; no interior, mesh collider, dynamic door or NPC collision'}
    (output / 'house-placement.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('terrain', type=Path)
    parser.add_argument('house', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(add_house(args.terrain, args.house, args.output), indent=2))
