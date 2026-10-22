#!/usr/bin/env python3
"""Independently compare partitioned geometry against its static source world."""
import argparse
import copy
import json
import itertools
from collections import Counter
from pathlib import Path
import numpy as np
from merge_static_glb import read
from shared_images import sampler
from static_geometry import accessor, hierarchy, TEXTURE_FIELDS


def triangles(document, binary, primitive, matrix):
    vertices = accessor(document, binary, primitive['attributes']['POSITION']).astype(float)
    vertices = vertices @ matrix[:3, :3].T + matrix[:3, 3]
    if 'indices' in primitive:
        faces = accessor(document, binary, primitive['indices']).reshape((-1, 3))
    else:
        faces = np.arange(len(vertices)).reshape((-1, 3))
    return vertices, faces


def material(document, index):
    if index is None:
        return None
    result = copy.deepcopy(document['materials'][index])
    for owner, field in TEXTURE_FIELDS:
        values = result if owner is None else result.get(owner, {})
        if field in values:
            texture = document['textures'][values[field]['index']]
            values[field]['index'] = {'image': document['images'][texture['source']], 'sampler': sampler(document, texture)}
    return result


def area(values):
    values = np.asarray(values, dtype=float)
    if values.size == 0:
        return 0.
    return float(np.linalg.norm(np.cross(values[:, 1] - values[:, 0], values[:, 2] - values[:, 0]), axis=1).sum() / 2)


class CollisionOracle:
    """Locate each complete output face on an original oriented source face."""
    def __init__(self, triangles, tolerance=1e-6, barycentric_tolerance=1e-7):
        self.source = np.asarray(triangles, dtype=float)
        self.tolerance = tolerance
        self.barycentric_tolerance = barycentric_tolerance
        self.bins = {}
        for index, triangle in enumerate(self.source):
            low = np.floor(triangle[:, [0, 2]].min(axis=0) / 16).astype(int)
            high = np.floor(triangle[:, [0, 2]].max(axis=0) / 16).astype(int)
            for key in itertools.product(range(low[0], high[0] + 1), range(low[1], high[1] + 1)):
                self.bins.setdefault(key, []).append(index)

    def check(self, triangles):
        for triangle in triangles:
            key = tuple(np.floor(triangle.mean(axis=0)[[0, 2]] / 16).astype(int))
            source = self.source[self.bins.get(key, [])]
            if not len(source):
                raise AssertionError('Collision face left source geometry')
            a = source[:, 0]
            first, second = source[:, 1] - a, source[:, 2] - a
            normals = np.cross(first, second)
            lengths = np.linalg.norm(normals, axis=1)
            delta = triangle[None, :, :] - a[:, None, :]
            aa = np.einsum('ij,ij->i', first, first)
            ab = np.einsum('ij,ij->i', first, second)
            bb = np.einsum('ij,ij->i', second, second)
            da = np.einsum('ijk,ik->ij', delta, first)
            db = np.einsum('ijk,ik->ij', delta, second)
            determinant = aa * bb - ab * ab
            valid = determinant > 1e-18
            safe = np.where(valid, determinant, 1.)[:, None]
            u = (da * bb[:, None] - db * ab[:, None]) / safe
            v = (db * aa[:, None] - da * ab[:, None]) / safe
            distance = abs(np.einsum('ijk,ik->ij', delta, normals)) / np.maximum(lengths, 1e-30)[:, None]
            epsilon = self.barycentric_tolerance
            inside = np.all((u >= -epsilon) & (v >= -epsilon) & (u + v <= 1 + epsilon) & (distance < self.tolerance), axis=1)
            normal = np.cross(triangle[1] - triangle[0], triangle[2] - triangle[0])
            assert np.any(valid & inside & (normals @ normal > 0)), 'Collision surface or winding changed'


def box_key(box):
    return json.dumps({key: [round(float(v), 8) for v in value] if isinstance(value, list) else value
                       for key, value in box.items()}, sort_keys=True)


def assert_interpolated(points, uv, source_points, source_uv):
    a = source_points[:, 0]
    first, second = source_points[:, 1] - a, source_points[:, 2] - a
    denominator = first[:, 0] * second[:, 2] - first[:, 2] * second[:, 0]
    if np.any(abs(denominator) < 1e-12):
        raise ValueError('Independent UV oracle requires nonvertical terrain triangles')
    for n, point in enumerate(points):
        delta = point - a
        u = (delta[:, 0] * second[:, 2] - delta[:, 2] * second[:, 0]) / denominator
        v = (first[:, 0] * delta[:, 2] - first[:, 2] * delta[:, 0]) / denominator
        selected = (u >= -1e-5) & (v >= -1e-5) & (u + v <= 1 + 1e-5)
        weights = np.stack([1 - u - v, u, v], axis=1)
        height = np.einsum('ij,ij->i', weights, source_points[:, :, 1])
        selected &= abs(height - point[1]) < .0005
        if uv is not None:
            expected = np.einsum('ij,ijk->ik', weights, source_uv)
            selected &= np.max(abs(expected - uv[n]), axis=1) < 3e-5
        if not np.any(selected):
            raise AssertionError(f'Clipped terrain point/UV left its source surface: {point}')


def verify_partition(source, output, terrain_meshes):
    (output / 'partition-verification.json').unlink(missing_ok=True)
    document, binary = read(source / 'terrain.glb', allow_external_images=True)
    _, world = hierarchy(document)
    report = json.loads((output / 'partition-report.json').read_bytes())
    index = json.loads((output / 'world.chunks.candidate.json').read_bytes())
    origin = np.array(index['query_origin'], dtype=float)
    expected_models = {n for n in world if 'mesh' in document['nodes'][n] and document['nodes'][n]['mesh'] not in set(terrain_meshes)}
    model_nodes = set()
    terrain_groups, output_area = {}, {}
    terrain_oracles = {}
    collision_area = 0.
    source_profile = json.loads((source / 'terrain.json').read_bytes())
    collision_oracle = CollisionOracle(source_profile['triangles'])
    boxes = {field: Counter() for field in ('solids', 'camera_solids')}
    for n, matrix in world.items():
        node = document['nodes'][n]
        if node.get('mesh') not in terrain_meshes:
            continue
        for part, primitive in enumerate(document['meshes'][node['mesh']]['primitives']):
            positions, faces = triangles(document, binary, primitive, matrix)
            uv = accessor(document, binary, primitive['attributes']['TEXCOORD_0'])[faces] if 'TEXCOORD_0' in primitive['attributes'] else None
            terrain_groups[(n, part)] = (positions[faces], uv)
            terrain_oracles[(n, part)] = CollisionOracle(positions[faces], tolerance=.0005, barycentric_tolerance=1e-5)
            output_area[(n, part)] = 0.
    for item in report['chunks']:
        x, z = item['id']
        directory = output / f'chunk_{x}_{z}'
        reduced, packed = read(directory / 'terrain.glb', allow_external_images=True)
        _, transforms = hierarchy(reduced)
        offset = np.array([x * 128., 0., z * 128.]) - origin
        for mapping in item['model_nodes']:
            old, new = mapping['source'], mapping['output']
            assert old not in model_nodes
            model_nodes.add(old)
            matrix = transforms[new].copy()
            matrix[:3, 3] += offset
            np.testing.assert_allclose(matrix, world[old], atol=1e-9, rtol=0)
            before = document['meshes'][document['nodes'][old]['mesh']]['primitives']
            after = reduced['meshes'][reduced['nodes'][new]['mesh']]['primitives']
            assert len(before) == len(after)
            for a, b in zip(before, after):
                assert set(a['attributes']) == set(b['attributes'])
                for attribute in a['attributes']:
                    np.testing.assert_array_equal(accessor(document, binary, a['attributes'][attribute]), accessor(reduced, packed, b['attributes'][attribute]))
                if 'indices' in a:
                    np.testing.assert_array_equal(accessor(document, binary, a['indices']), accessor(reduced, packed, b['indices']))
                assert material(document, a.get('material')) == material(reduced, b.get('material'))
        for n, node in enumerate(reduced['nodes']):
            if not node.get('name', '').startswith('clipped_'):
                continue
            _, original, part = node['name'].split('_')
            key = (int(original), int(part))
            primitive = reduced['meshes'][node['mesh']]['primitives'][0]
            positions, faces = triangles(reduced, packed, primitive, transforms[n])
            assert np.all(positions[:, [0, 2]] >= -1e-6) and np.all(positions[:, [0, 2]] <= 128.000001)
            positions += offset
            uv = accessor(reduced, packed, primitive['attributes']['TEXCOORD_0']) if 'TEXCOORD_0' in primitive['attributes'] else None
            source_points, source_uv = terrain_groups[key]
            assert_interpolated(positions, uv, source_points, source_uv)
            terrain_oracles[key].check(positions[faces])
            output_area[key] += area(positions[faces])
        profile = json.loads((directory / 'terrain.json').read_bytes())
        collision_area += area(profile['triangles'])
        collision_faces = np.asarray(profile['triangles'], dtype=float).reshape((-1, 3, 3)) + offset
        collision_oracle.check(collision_faces)
        for field in boxes:
            for box in profile.get(field, []):
                box = copy.deepcopy(box)
                box['center'] = (np.asarray(box['center']) + offset).tolist()
                boxes[field][box_key(box)] += 1
    assert model_nodes == expected_models
    maximum_error = 0.
    for key, (source_triangles, _) in terrain_groups.items():
        expected = area(source_triangles)
        error = abs(output_area[key] - expected)
        maximum_error = max(maximum_error, error)
        assert error < max(.001, expected * 1e-6), (key, error)
    for field in boxes:
        assert boxes[field] == Counter(box_key(box) for box in source_profile.get(field, [])), 'Whole collision boxes changed'
    expected = area(source_profile['triangles'])
    assert abs(collision_area - expected) < max(.001, expected * 1e-6)
    result = {'passed': True, 'model_instances_exact': len(model_nodes), 'terrain_groups': len(terrain_groups),
              'maximum_terrain_area_error': maximum_error, 'collision_area_error': abs(collision_area - expected),
              'collision_surfaces_and_winding': True, 'collision_boxes_exact': sum(sum(values.values()) for values in boxes.values()),
              'terrain_surfaces_and_winding': True,
              'scope': 'offline source geometry/UV/material/transform equivalence; no runtime streaming or capacity claim'}
    (output / 'partition-verification.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--terrain-mesh', type=int, action='append', required=True)
    args = parser.parse_args()
    print(json.dumps(verify_partition(args.source.resolve(), args.output.resolve(), args.terrain_mesh)))
