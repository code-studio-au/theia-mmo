#!/usr/bin/env python3
"""Original textured fixtures for full startup and multi-provider publication."""
import argparse
import hashlib
import json
from pathlib import Path
import tempfile

import numpy as np
from PIL import Image
import trimesh
from externalize_world_images import externalize
from index_chunk_contributors import author
from package_world import package


def create(output, mode='startup', base=0):
    if mode not in ('startup', 'publication') or output.exists() or type(base) is not int or abs(base) > 7_000_000_000:
        raise ValueError('Choose startup/publication and a new output directory')
    output.mkdir(parents=True)
    cells = [(x, z) for x in range(8) for z in range(4)] if mode == 'startup' else [(x, 0) for x in range(7)]
    entries = []
    with tempfile.TemporaryDirectory(prefix='theia-capacity-') as temporary:
        for x, z in cells:
            directory = Path(temporary) / f'chunk_{x}_{z}'
            directory.mkdir()
            if mode == 'startup' or x <= 1:
                vertices = np.array([[0., 0., 0.], [0., 0., 128.], [128., 0., 0.], [128., 0., 128.]])
                faces = np.array([[0, 1, 2], [2, 1, 3]])
                mesh = trimesh.Trimesh(vertices=vertices, faces=faces, process=False)
                triangles, solids = vertices[faces].tolist(), []
            else:
                center = [140. - x * 128, 1., 60. + x * 6]
                mesh = trimesh.creation.box(extents=[1., 2., 2.])
                mesh.apply_translation(center)
                triangles, solids = [], [{'center': center, 'size': [1., 2., 2.]}]
            mesh.visual = trimesh.visual.TextureVisuals(uv=np.zeros((len(mesh.vertices), 2)),
                material=trimesh.visual.material.PBRMaterial(baseColorTexture=Image.new('RGBA', (2, 2), (0, 255, 0, 255))))
            visual = trimesh.exchange.gltf.export_glb(trimesh.Scene(mesh))
            (directory / 'terrain.glb').write_bytes(visual)
            profile = {'schema': 3, 'bounds': [[0., 0.], [128., 128.]], 'source_origin': [0., 0., 0.], 'spawns': [],
                       'triangles': triangles, 'solids': solids, 'visual_sha256': list(hashlib.sha256(visual).digest())}
            (directory / 'terrain.json').write_text(json.dumps(profile) + '\n')
            content = f'theia.fixture.capacity.{mode}.x{x}.z{z}'
            package(directory, content, 1)
            owner = [base + x, base + z]
            result = externalize(directory, output / f'chunk_{owner[0]}_{owner[1]}', content, 2)
            entries.append({'id': owner, 'package': list(bytes.fromhex(result['identity']))})
    index = {'schema': 1, 'chunk_metres': 128, 'query_origin': [base * 128., 0., base * 128.], 'chunks': entries}
    (output / 'world.chunks.json').write_text(json.dumps(index) + '\n')
    author(output, output / 'world.chunks.candidate.json')
    (output / 'world.chunks.candidate.json').replace(output / 'world.chunks.json')
    session = {'schema': 1, 'resident_budget': len(cells),
               'initial_chunks': [entry['id'] for entry in entries] if mode == 'startup' else [[base, base]],
               'spawns': [[base * 128. + 124., 0., base * 128. + 32.], [base * 128. + 122., 0., base * 128. + 34.]],
               'fall_plane': -18.}
    (output / 'world.session.json').write_text(json.dumps(session) + '\n')
    return {'mode': mode, 'packages': len(cells), 'initial': len(session['initial_chunks'])}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--mode', choices=['startup', 'publication'], default='startup')
    parser.add_argument('--base', type=int, default=0)
    args = parser.parse_args()
    print(json.dumps(create(args.output.resolve(), args.mode, args.base)))
