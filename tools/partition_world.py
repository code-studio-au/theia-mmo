#!/usr/bin/env python3
"""Partition a verified static shared-image world into new 128m packages.

Terrain selection is explicit. Models retain their hierarchy; collision triangles
are clipped and boxes stay whole. Outputs that exceed runtime proof budgets keep
only a candidate index/report and never replace the active development map.
"""
import argparse
import copy
import itertools
import json
import math
import os
from pathlib import Path
import shutil

import numpy as np
from add_terrain_house import write_glb
from index_chunk_contributors import overlaps, package_bounds
from merge_static_glb import read
from package_world import MANIFEST, MAX_COLLISION_BYTES, MAX_VISUAL_BYTES, file_hash, identity, package, valid_id
from shared_images import filename, verify
from static_geometry import accessor, clip_triangle, hierarchy, prune, transformed_bounds


def cell(point, origin):
    result = tuple(math.floor((float(point[n]) + origin[k]) / 128) for k, n in enumerate([0, 2]))
    if any(abs(n) > 7_000_000_000 for n in result):
        raise ValueError('Partition owner exceeds conservative global range')
    return result


def corner(owner, origin):
    return np.array([owner[0] * 128 - origin[0], 0., owner[1] * 128 - origin[1]])


def split_triangle(rows, origin):
    low = cell(rows[:, :3].min(axis=0), origin)
    high = cell(rows[:, :3].max(axis=0), origin)
    if (high[0] - low[0] + 1) * (high[1] - low[1] + 1) > 64:
        raise ValueError('Triangle exceeds bounded partition domain')
    if low == high:
        return [(low, rows)]
    pieces = []
    for owner in itertools.product(range(low[0], high[0] + 1), range(low[1], high[1] + 1)):
        point = corner(owner, origin)[[0, 2]]
        pieces.extend((owner, triangle) for triangle in clip_triangle(rows, point, point + 128))
    return pieces


def terrain_rows(document, binary, primitive, matrix):
    if np.linalg.det(matrix[:3, :3]) <= 1e-12:
        raise ValueError('Clipped terrain requires a nonsingular, orientation-preserving transform')
    names = ['POSITION'] + sorted(key for key in primitive['attributes'] if key != 'POSITION')
    if any(name not in ('POSITION', 'NORMAL', 'TEXCOORD_0', 'TEXCOORD_1', 'COLOR_0') for name in names):
        raise ValueError('Unsupported clipped terrain attribute')
    arrays, layout, offset = [], {}, 0
    for name in names:
        item = document['accessors'][primitive['attributes'][name]]
        if item['componentType'] != 5126 or item.get('normalized', False):
            raise ValueError('Clipped terrain requires floating attributes')
        values = accessor(document, binary, primitive['attributes'][name]).astype(float)
        if name == 'POSITION':
            if values.shape[1] != 3:
                raise ValueError('Terrain positions require VEC3')
            values = values @ matrix[:3, :3].T + matrix[:3, 3]
        elif name == 'NORMAL':
            if values.shape[1] != 3 or abs(np.linalg.det(matrix[:3, :3])) < 1e-12:
                raise ValueError('Invalid terrain normal transform')
            values = values @ np.linalg.inv(matrix[:3, :3])
            lengths = np.linalg.norm(values, axis=1)
            if np.any(lengths < 1e-12):
                raise ValueError('Invalid terrain normals')
            values /= lengths[:, None]
        if arrays and len(values) != len(arrays[0]):
            raise ValueError('Terrain attribute counts disagree')
        layout[name] = (offset, offset + values.shape[1])
        offset += values.shape[1]
        arrays.append(values)
    rows = np.concatenate(arrays, axis=1)
    if primitive.get('mode', 4) != 4 or primitive.get('targets'):
        raise ValueError('Only static terrain triangles are supported')
    if 'indices' in primitive:
        item = document['accessors'][primitive['indices']]
        if item['componentType'] not in (5121, 5123, 5125) or item['type'] != 'SCALAR':
            raise ValueError('Invalid terrain indices')
        indices = accessor(document, binary, primitive['indices']).reshape(-1)
    else:
        indices = np.arange(len(rows))
    if len(indices) % 3 or np.any(indices >= len(rows)):
        raise ValueError('Invalid terrain triangle indexing')
    return rows, indices.reshape((-1, 3)), layout


def append_geometry(document, binary, groups, owner, origin):
    added = set()
    for name, original, triangles, layout in groups:
        rows = np.concatenate(triangles)
        rows[:, :3] -= corner(owner, origin)
        if 'NORMAL' in layout:
            low, high = layout['NORMAL']
            lengths = np.linalg.norm(rows[:, low:high], axis=1)
            if np.any(lengths < 1e-12):
                raise ValueError('Clipped normal is degenerate')
            rows[:, low:high] /= lengths[:, None]
        vertices, indices = np.unique(rows.astype(np.float32), axis=0, return_inverse=True)
        primitive = {key: copy.deepcopy(value) for key, value in original.items() if key not in ('attributes', 'indices')}
        primitive['attributes'] = {}
        for key, (low, high) in layout.items():
            values = np.ascontiguousarray(vertices[:, low:high], dtype='<f4')
            binary.extend(b'\0' * (-len(binary) % 4))
            view = len(document['bufferViews'])
            document['bufferViews'].append({'buffer': 0, 'byteOffset': len(binary), 'byteLength': values.nbytes})
            binary.extend(values.tobytes())
            index = len(document['accessors'])
            document['accessors'].append({'bufferView': view, 'componentType': 5126, 'count': len(values),
                                         'type': f'VEC{high - low}', 'min': values.min(axis=0).tolist(), 'max': values.max(axis=0).tolist()})
            primitive['attributes'][key] = index
        binary.extend(b'\0' * (-len(binary) % 4))
        values = np.ascontiguousarray(indices, dtype='<u4')
        view = len(document['bufferViews'])
        document['bufferViews'].append({'buffer': 0, 'byteOffset': len(binary), 'byteLength': values.nbytes})
        binary.extend(values.tobytes())
        primitive['indices'] = len(document['accessors'])
        document['accessors'].append({'bufferView': view, 'componentType': 5125, 'count': len(values), 'type': 'SCALAR'})
        mesh = len(document['meshes'])
        document['meshes'].append({'name': name, 'primitives': [primitive]})
        node = len(document['nodes'])
        document['nodes'].append({'name': name, 'mesh': mesh})
        document['scenes'][0]['nodes'].append(node)
        added.add(node)
    return added


def partition(source, output, content_id, revision, terrain_meshes, origin=(0., 0.)):
    source, output = source.resolve(strict=True), output.resolve()
    if output.exists() or output.is_relative_to(source):
        raise FileExistsError('Partition output must be new and outside source')
    if not valid_id(content_id) or type(revision) is not int or not 1 <= revision <= 0xffffffff:
        raise ValueError('Invalid partition identity/revision')
    origin = tuple(float(v) for v in origin)
    if len(origin) != 2 or not all(math.isfinite(v) and abs(v) <= 896_000_000_000 for v in origin):
        raise ValueError('Invalid global XZ origin')
    manifest = json.loads((source / MANIFEST).read_bytes())
    if manifest['schema'] != 2:
        raise ValueError('Partition input requires a shared-image package')
    for name, field, limit in [('terrain.glb', 'visual', MAX_VISUAL_BYTES), ('terrain.json', 'collision', MAX_COLLISION_BYTES)]:
        size, digest = file_hash(source / name, limit)
        if size != manifest[field + '_bytes'] or list(digest) != manifest[field + '_sha256']:
            raise ValueError('Partition source binding mismatch')
    document, binary = read(source / 'terrain.glb', allow_external_images=True)
    verify(document, source.parent, manifest['shared_images'])
    parents, matrices = hierarchy(document)
    terrain_meshes = set(terrain_meshes)
    if not terrain_meshes or any(type(v) is not int or not 0 <= v < len(document['meshes']) for v in terrain_meshes):
        raise ValueError('Explicit terrain mesh selection is required')
    models, terrain, collision, solids, cameras = {}, {}, {}, {}, {}
    selected_instances = 0
    for n, matrix in matrices.items():
        node = document['nodes'][n]
        if 'mesh' not in node:
            continue
        mesh = node['mesh']
        if mesh not in terrain_meshes:
            bounds = transformed_bounds(document, binary, mesh, matrix)
            owner = cell(bounds.mean(axis=0), origin)
            models.setdefault(owner, set()).add(n)
            continue
        selected_instances += 1
        for primitive_index, primitive in enumerate(document['meshes'][mesh]['primitives']):
            rows, faces, layout = terrain_rows(document, binary, primitive, matrix)
            pieces = {}
            for face in faces:
                for owner, triangle in split_triangle(rows[face], origin):
                    pieces.setdefault(owner, []).append(triangle)
            for owner, triangles in pieces.items():
                terrain.setdefault(owner, []).append((f'clipped_{n}_{primitive_index}', primitive, triangles, layout))
    if selected_instances == 0:
        raise ValueError('No selected terrain instances in the source scene')
    profile = json.loads((source / 'terrain.json').read_bytes())
    if profile['schema'] != 3:
        raise ValueError('Partition collision requires schema-3 general triangles')
    for values in profile['triangles']:
        for owner, triangle in split_triangle(np.asarray(values, dtype=float), origin):
            collision.setdefault(owner, []).append((triangle[:, :3] - corner(owner, origin)).tolist())
    for field, target in [('solids', solids), ('camera_solids', cameras)]:
        for original in profile.get(field, []):
            owner = cell(original['center'], origin)
            value = copy.deepcopy(original)
            value['center'] = (np.asarray(value['center']) - corner(owner, origin)).tolist()
            target.setdefault(owner, []).append(value)
    owners = sorted(set(models) | set(terrain) | set(collision) | set(solids) | set(cameras))
    if not 1 <= len(owners) <= 64:
        raise ValueError('Partition exceeds bounded index entry budget')
    output.mkdir(parents=True)
    try:
        shared = output / 'shared'
        shared.mkdir()
        for image in manifest['shared_images']:
            original = source.parent / 'shared' / filename(image)
            try:
                os.link(original, shared / original.name)
            except OSError:
                shutil.copyfile(original, shared / original.name)
        entries, reports = [], []
        for owner in owners:
            if not collision.get(owner) and not solids.get(owner):
                raise ValueError(f'Visual-only collision contract requires runtime support: {owner}')
            candidate = copy.deepcopy(document)
            data = bytearray(binary)
            roots = candidate['scenes'][0]['nodes']
            wrapper = len(candidate['nodes'])
            candidate['nodes'].append({'name': 'Chunk source rebase', 'translation': (-corner(owner, origin)).tolist(), 'children': roots})
            candidate['scenes'][0]['nodes'] = [wrapper]
            selected = set(models.get(owner, set())) | append_geometry(candidate, data, terrain.get(owner, []), owner, origin)
            if not selected:
                raise ValueError(f'Collision-only cell needs an explicit visual contract: {owner}')
            candidate['buffers'][0]['byteLength'] = len(data)
            reduced, packed, mappings = prune(candidate, data, selected, with_mapping=True)
            x, z = owner
            directory = output / f'chunk_{x}_{z}'
            directory.mkdir()
            write_glb(reduced, packed, directory / 'terrain.glb')
            values = {'schema': 3, 'bounds': [[0., 0.], [128., 128.]], 'source_origin': profile['source_origin'], 'spawns': [],
                      'triangles': collision.get(owner, []), 'solids': solids.get(owner, []), 'camera_solids': cameras.get(owner, []),
                      'visual_sha256': list(file_hash(directory / 'terrain.glb', MAX_VISUAL_BYTES)[1])}
            (directory / 'terrain.json').write_text(json.dumps(values, separators=(',', ':'), allow_nan=False) + '\n')
            name = f'{content_id}.x{"m" if x < 0 else "p"}{abs(x)}.z{"m" if z < 0 else "p"}{abs(z)}'
            published = package(directory, name, revision, shared_images=True)
            binding = list(bytes.fromhex(published['identity']))
            bounds = package_bounds(directory, binding)
            entries.append({'id': list(owner), 'package': binding, 'bounds': bounds})
            reports.append({'id': list(owner), 'model_instances': len(models.get(owner, [])), 'terrain_primitives': len(terrain.get(owner, [])),
                            'model_nodes': [{'source': node, 'output': mappings['node'][node]} for node in sorted(models.get(owner, []))],
                            'collision_triangles': len(values['triangles']), 'solids': len(values['solids']), 'nodes': len(reduced['nodes']),
                            'meshes': len(reduced['meshes']), 'textures': len(reduced['textures']), 'visual_bytes': published['visual_bytes'],
                            'collision_bytes': published['collision_bytes']})
        contributors = {tuple(e['id']): sorted({tuple(e['id'])} | {tuple(p['id']) for p in entries if overlaps(p['id'], p['bounds'], e['id'])}) for e in entries}
        maximum = max(map(len, contributors.values()))
        index = {'schema': 2, 'chunk_metres': 128, 'query_origin': [origin[0], 0., origin[1]], 'chunks': entries}
        encoded = json.dumps(index, separators=(',', ':'), allow_nan=False) + '\n'
        if len(encoded.encode()) > 32 * 1024:
            raise ValueError('Candidate index exceeds byte budget')
        (output / 'world.chunks.candidate.json').write_text(encoded)
        initial = set()
        spawns = [[p[0] + origin[0], p[1], p[2] + origin[1]] for p in profile.get('spawns', [])]
        for point in spawns:
            for dx, dz in itertools.product([-.355, 0., .355], repeat=2):
                owner = (math.floor((point[0] + dx) / 128), math.floor((point[2] + dz) / 128))
                if owner not in contributors:
                    raise ValueError('Trusted spawn footprint outside partition domain')
                initial.update(contributors[owner])
        ready = maximum <= 4 and 0 < len(initial) <= 4
        if ready:
            (output / 'world.chunks.json').write_text(encoded)
            (output / 'world.session.json').write_text(json.dumps({'schema': 1, 'resident_budget': 4, 'initial_chunks': [list(v) for v in sorted(initial)],
                'spawns': spawns, 'fall_plane': min(p[1] for p in spawns) - 64.}, separators=(',', ':')) + '\n')
        # Verify immutable sources once more before publishing the report.
        if json.loads((source / MANIFEST).read_bytes()) != manifest:
            raise ValueError('Source manifest changed during partitioning')
        for name, field, limit in [('terrain.glb', 'visual', MAX_VISUAL_BYTES), ('terrain.json', 'collision', MAX_COLLISION_BYTES)]:
            size, digest = file_hash(source / name, limit)
            if size != manifest[field + '_bytes'] or list(digest) != manifest[field + '_sha256']:
                raise ValueError('Source changed during partitioning')
        report = {'chunks': reports, 'terrain_instances': selected_instances, 'model_instances': sum(len(v) for v in models.values()),
                  'maximum_cell_contributors': maximum, 'initial_required_packages': [list(v) for v in sorted(initial)], 'runtime_metadata_emitted': ready,
                  'contributors': [{'id': list(k), 'packages': [list(v) for v in values]} for k, values in contributors.items()],
                  'scope': 'offline real geometry/assets partition; runtime proof budgets and adoption remain separate'}
        (output / 'partition-report.json').write_text(json.dumps(report, indent=2) + '\n')
        return report
    except BaseException:
        shutil.rmtree(output)
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--content-id', required=True)
    parser.add_argument('--revision', type=int, required=True)
    parser.add_argument('--terrain-mesh', type=int, action='append', required=True)
    parser.add_argument('--origin-x', type=float, default=0.)
    parser.add_argument('--origin-z', type=float, default=0.)
    args = parser.parse_args()
    result = partition(args.source, args.output, args.content_id, args.revision, args.terrain_mesh, (args.origin_x, args.origin_z))
    print(json.dumps({key: result[key] for key in ['terrain_instances', 'model_instances', 'maximum_cell_contributors', 'initial_required_packages', 'runtime_metadata_emitted']}))
