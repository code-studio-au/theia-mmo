#!/usr/bin/env python3
"""Create an original three-chunk floor/wall fixture for bounded residency proof.

No imported assets. Visual and collision floor have a 1m seam overlap; each root
has exactly one half-open 128m owner. The default MMO tile is not changed.
"""
import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import trimesh
from package_world import package


def create(output, base_x=0, base_z=0, static_shapes=False):
    if output.exists():
        raise ValueError('Output directory must be new')
    if type(base_x) is not int or type(base_z) is not int or abs(base_x) > 7_000_000_000 or abs(base_z) > 7_000_000_000:
        raise ValueError('Fixture base outside conservative global bounds')
    output.mkdir(parents=True)
    vertices = np.array([[-1., 0., -1.], [-1., 0., 129.], [129., 0., -1.], [129., 0., 129.]])
    faces = np.array([[0, 1, 2], [2, 1, 3]])
    entries = []
    for x in [base_x - 1, base_x, base_x + 1]:
        directory = output / f'chunk_{x}_{base_z}'
        directory.mkdir()
        floor = trimesh.Trimesh(vertices=vertices, faces=faces, process=False)
        floor.visual.vertex_colors = np.tile([[160, 70, 45, 255], [70, 145, 85, 255], [50, 90, 170, 255]][x - base_x + 1], (4, 1))
        scene = trimesh.Scene(floor)
        solids = []
        if x == base_x:
            wall = trimesh.creation.box(extents=[0.2, 2., 8.])
            wall.apply_translation([64., 1., 64.])
            scene.add_geometry(wall)
            solids.append({'center': [64., 1., 64.], 'size': [0.2, 2., 8.]})
        visual = trimesh.exchange.gltf.export_glb(scene)
        (directory / 'terrain.glb').write_bytes(visual)
        profile = {'schema': 3, 'spawns': [], 'bounds': [[-1., -1.], [129., 129.]],
                   'source_origin': [0., 0., 0.], 'visual_sha256': list(hashlib.sha256(visual).digest()),
                   'triangles': vertices[faces].tolist(), 'solids': solids}
        if static_shapes:
            profile.update(schema=4, triangles=[], solids=[], static_shapes={
                'objects':[dict(kind='box',rotation=[0,0,0,1],**box) for box in solids],
                'heightfields':[{'rows':2,'columns':2,'heights_column_major':[0]*4,
                                 'center':[64,0,64],'scale':[130,1,130]}]})
        (directory / 'terrain.json').write_text(json.dumps(profile, separators=(',', ':')) + '\n')
        published = package(directory, f'theia.fixture.chunk.x{x}-z{base_z}', 1)
        entries.append({'id': [x, base_z], 'package': list(bytes.fromhex(published['identity']))})
    index = {'schema': 1, 'chunk_metres': 128, 'query_origin': [base_x * 128., 0., base_z * 128.], 'chunks': entries}
    (output / 'world.chunks.json').write_text(json.dumps(index, indent=2) + '\n')
    session = {'schema': 1, 'resident_budget': 2,
               'initial_chunks': [[base_x - 1, base_z], [base_x, base_z]],
               'spawns': [[base_x * 128. + 126., 0., base_z * 128. + 32.],
                          [base_x * 128. + 124., 0., base_z * 128. + 32.]],
               'fall_plane': -18.}
    if static_shapes:
        from index_chunk_contributors import author
        authored = output / 'world.contributors.json'
        author(output, authored, resident_limit=3)
        authored.replace(output / 'world.chunks.json')
        session['resident_budget'] = 3
        session['initial_chunks'] = [entry['id'] for entry in entries]
    (output / 'world.session.json').write_text(json.dumps(session, indent=2) + '\n')
    return {'chunks': len(entries), 'query_origin': index['query_origin'],
            'scope': 'original bounded residency fixture; not streamed imported terrain'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--base-x', type=int, default=0)
    parser.add_argument('--base-z', type=int, default=0)
    parser.add_argument('--static-shapes', action='store_true', help='Schema-4 fields/boxes with complete schema-2 contributor bounds')
    args = parser.parse_args()
    print(json.dumps(create(args.output, args.base_x, args.base_z, args.static_shapes), indent=2))
