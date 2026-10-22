"""Measure source collision ownership and shared geometry before runtime admission.

Counts and conservative footprints are offline evidence, not streaming capacity.
No packages, limits, source objects or current runtime defaults are changed.
"""
import argparse
import collections
import itertools
import json
import math
from pathlib import Path
import numpy as np
from unity_editor_reference import digest, read_json
from index_chunk_contributors import overlaps


def bounds(body, meshes):
    matrix = np.array(body['matrix'], dtype=float).reshape(4, 4)
    if body['kind'] == 'MeshCollider':
        points = np.array(meshes[body['mesh']]['vertices'])
        points = points @ matrix[:3, :3].T + matrix[:3, 3]
    elif body['kind'] == 'BoxCollider':
        points = np.array(list(itertools.product([-1., 1.], repeat=3))) * np.array(body['size']) / 2 + body['center']
        points = points @ matrix[:3, :3].T + matrix[:3, 3]
    elif body['kind'] == 'CapsuleCollider':
        axis = body['direction']
        scale = abs(np.array(body['lossy_scale']))
        radius = body['radius'] * max(scale[i] for i in range(3) if i != axis)
        half = max(body['height'] * scale[axis] - 2 * radius, 0) / 2
        direction = matrix[:3, axis] / np.linalg.norm(matrix[:3, axis])
        center = matrix[:3, :3] @ body['center'] + matrix[:3, 3]
        extent = abs(direction) * half + radius
        return np.array([center - extent, center + extent])
    else:
        raise ValueError('Unknown source object shape')
    return np.array([points.min(0), points.max(0)])


def analyze(objects, terrain, output):
    output = Path(output).resolve()
    build = Path(__file__).resolve().parents[1] / 'build'
    if output.exists() or output == build or not output.is_relative_to(build):
        raise ValueError('Use a fresh ignored analysis report')
    for path in [objects, terrain]:
        receipt = read_json(Path(path).with_suffix('.receipt.json'), 1024 * 1024)
        if receipt['fixture_sha256'] != digest(path):
            raise ValueError('Bound source fixture changed')
    document = read_json(objects, 128 * 1024 * 1024)
    ground = read_json(terrain, 128 * 1024 * 1024)
    if document['profile'] != 'unity-object-collision-comparison-v1' or ground['profile'] != 'source-full-grid-heightfield-reference-v1':
        raise ValueError('Unknown source representation')
    cells = {}

    def cell(owner):
        if owner not in cells:
            cells[owner] = {'objects': collections.Counter(), 'mesh_indices': set(),
                            'expanded_mesh_triangles': 0, 'expanded_mesh_vertices': 0,
                            'bounds': None, 'heightfield_samples': 0, 'implicit_terrain_triangles': 0}
        return cells[owner]

    def extend(entry, box):
        if entry['bounds'] is None:
            entry['bounds'] = box.copy()
        else:
            entry['bounds'][0] = np.minimum(entry['bounds'][0], box[0])
            entry['bounds'][1] = np.maximum(entry['bounds'][1], box[1])

    for body in document['colliders']:
        box = bounds(body, document['meshes'])
        owner = tuple(int(v) for v in np.floor(box.mean(0)[[0, 2]] / 128))
        entry = cell(owner)
        entry['objects'][body['kind']] += 1
        if body['mesh'] is not None:
            mesh = document['meshes'][body['mesh']]
            entry['mesh_indices'].add(body['mesh'])
            entry['expanded_mesh_triangles'] += len(mesh['indices'])
            entry['expanded_mesh_vertices'] += len(mesh['vertices'])
        extend(entry, box)
    half_rows = (ground['rows'] - 1) // 2
    half_columns = (ground['columns'] - 1) // 2
    if ground['rows'] != 513 or ground['columns'] != 513:
        raise ValueError('The bounded forest quarter comparison uses the full 513 grid')
    for x, z in itertools.product(range(2), repeat=2):
        center = np.array(ground['center'], dtype=float)
        center[[0, 2]] += (np.array([x, z]) - .5) * np.array(ground['scale'])[[0, 2]] / 2
        extent = np.array(ground['scale']) / 4
        low, high = center - extent, center + extent
        low[1], high[1] = min(ground['heights_column_major']), max(ground['heights_column_major'])
        owner = tuple(int(v) for v in np.floor(center[[0, 2]] / 128))
        entry = cell(owner)
        entry['heightfield_samples'] += (half_rows + 1) * (half_columns + 1)
        entry['implicit_terrain_triangles'] += 2 * half_rows * half_columns
        extend(entry, np.array([low, high]))
    owners = []
    for owner, entry in sorted(cells.items()):
        unique_triangles = sum(len(document['meshes'][i]['indices']) for i in entry['mesh_indices'])
        unique_vertices = sum(len(document['meshes'][i]['vertices']) for i in entry['mesh_indices'])
        local = entry['bounds'][:, [0, 2]] - np.array(owner) * 128
        owners.append({'id': list(owner), 'objects': dict(entry['objects']),
                       'mesh_pool_entries': len(entry['mesh_indices']),
                       'pool_triangles': unique_triangles, 'pool_vertices': unique_vertices,
                       'expanded_mesh_triangles': entry['expanded_mesh_triangles'],
                       'expanded_mesh_vertices': entry['expanded_mesh_vertices'],
                       'heightfield_samples': entry['heightfield_samples'],
                       'implicit_terrain_triangles': entry['implicit_terrain_triangles'],
                       'bounds_local_xz': local.tolist()})
    center = np.array(ground['center'])[[0, 2]]
    extent = np.array(ground['scale'])[[0, 2]] / 2
    low, high = center - extent, center + extent
    core = list(itertools.product(*(range(math.floor(lo / 128), math.ceil(hi / 128)) for lo, hi in zip(low, high))))
    contributors = [{'id': list(target), 'providers': [e['id'] for e in owners if overlaps(e['id'], e['bounds_local_xz'], target)]} for target in core]
    result = {'schema': 1, 'objects_sha256': digest(objects), 'terrain_sha256': digest(terrain),
              'owners': owners, 'finite_terrain_cell_contributors': contributors,
              'maximum_measured_providers': max(len(c['providers']) for c in contributors),
              'maximum_pool_triangles': max(o['pool_triangles'] for o in owners),
              'maximum_expanded_mesh_triangles': max(o['expanded_mesh_triangles'] for o in owners),
              'maximum_heightfield_samples': max(o['heightfield_samples'] for o in owners),
              'scope': 'Offline shared-geometry and exact source-shape footprint ownership projection; no visual contributors, runtime publication, latency, admission or MMO capacity claim'}
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('x') as stream:
        stream.write(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('objects', type=Path)
    parser.add_argument('terrain', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    result = analyze(args.objects, args.terrain, args.output)
    print(json.dumps({k: result[k] for k in ['maximum_measured_providers', 'maximum_pool_triangles', 'maximum_expanded_mesh_triangles', 'maximum_heightfield_samples', 'scope']}))
