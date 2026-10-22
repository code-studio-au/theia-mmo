#!/usr/bin/env python3
"""Prepare the complete exported tile as a local authoritative development world.

Keep the assembled visual BIN unchanged. M2 collision bounds become conservative
world AABBs; WMO buildings use solid visual bounds. Water cells are no-entry zones.
This intentionally does not infer interiors, swimming or mesh physics.
"""
import argparse
import hashlib
import itertools
import json
from pathlib import Path
import struct

import numpy as np

from add_map_liquids import surface
from assemble_map import placement_plan
from map_placement import TO_Y_UP
from merge_static_glb import read


def obj_geometry(path):
    vertices, faces = [], []
    for line in path.read_text().splitlines():
        parts = line.split()
        if not parts:
            continue
        if parts[0] == 'v':
            vertices.append([float(v) for v in parts[1:4]])
        elif parts[0] == 'f':
            if len(parts) != 4:
                raise ValueError('Expected triangulated OBJ')
            face = [int(v.split('/')[0]) for v in parts[1:]]
            if min(face) <= 0 or max(face) > len(vertices):
                raise ValueError('Expected positive, available OBJ indices')
            faces.append([v - 1 for v in face])
    vertices = np.asarray(vertices, dtype=np.float32)
    if vertices.size and (vertices.ndim != 2 or vertices.shape[1] != 3 or not np.isfinite(vertices).all()):
        raise ValueError('Invalid OBJ positions')
    return vertices, np.asarray(faces, dtype=int)


def envelope(points):
    lo, hi = points.min(axis=0), points.max(axis=0)
    if not np.isfinite(points).all() or (hi <= lo).any():
        raise ValueError('Invalid collision envelope')
    return {'center': ((lo + hi) / 2).tolist(), 'size': (hi - lo).tolist()}


def model_bounds(path):
    metadata = json.loads(path.with_suffix('.json').read_text())
    if metadata['fileType'] == 'm2':
        bounds = metadata['collisionBox']
        lo, hi = np.asarray(bounds['min']), np.asarray(bounds['max'])
        if np.array_equal(lo, [10000000] * 3) and np.array_equal(hi, [-10000000] * 3):
            return None, 'source empty collision-bounds sentinel'
        if lo.shape != (3,) or hi.shape != (3,) or not np.isfinite([lo, hi]).all() or (hi < lo).any():
            raise ValueError('Invalid source collision bounds')
        if (hi == lo).any():
            if np.array_equal(lo, np.zeros(3)) and np.array_equal(hi, np.zeros(3)):
                return None, 'source has no collision bounds'
            raise ValueError('Degenerate source collision bounds')
        # Source metadata is Z-up; exported OBJ vertices are (x,z,-y).
        corners = np.asarray(list(itertools.product(*zip(lo, hi))))
        return corners @ TO_Y_UP[:3, :3].T, 'M2 source collision bounds, conservative AABB'
    if metadata['fileType'] == 'wmo':
        vertices, _ = obj_geometry(path)
        return vertices, 'WMO solid visual envelope; interior unavailable'
    raise ValueError('Unsupported static model type')


def collision_plan(placements, origin):
    plan, inputs, _, _ = placement_plan(placements)
    transforms, cached, solids, audit = {'world': np.eye(4)}, {}, [], []
    for name, parent, model, relative in plan:
        matrix = transforms[parent] @ relative
        transforms[name] = matrix
        if model not in cached:
            cached[model] = model_bounds(model)
            inputs.update([model, model.with_suffix('.json')])
        points, policy = cached[model]
        record = {'placement': name, 'source': str(model), 'policy': policy}
        if points is not None:
            points = points @ matrix[:3, :3].T + matrix[:3, 3] - origin
            solid = envelope(points)
            record['solid_index'] = len(solids)
            solids.append(solid)
        audit.append(record)
    return solids, audit, inputs


def prepare(source, placements, liquids, visual, output, origin):
    if output.exists():
        raise ValueError('Output directory must be new')
    origin = np.asarray(origin, dtype=float)
    if origin.shape != (3,) or not np.isfinite(origin).all():
        raise ValueError('Origin must contain three finite coordinates')
    plan, sidecars, _, _ = placement_plan(placements)
    inputs = {source.resolve(), placements.resolve(), liquids.resolve(), visual.resolve(), *sidecars}
    for _, _, model, _ in plan:
        inputs.update([model, model.with_suffix('.json')])
    hashes = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(inputs)}
    vertices, faces = obj_geometry(source)
    points = np.asarray(vertices - origin, dtype=np.float32)
    triangles = points[faces]
    if not len(triangles) or len(triangles) > 131072 or not np.isfinite(triangles).all():
        raise ValueError('Invalid full-tile triangle budget')
    normals = np.cross(triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0])
    if (normals[:, 1] <= 1e-6).any():
        raise ValueError('Terrain must have upward nonvertical triangles')
    solids, audit, _ = collision_plan(placements, origin)
    object_count = len(solids)
    document = json.loads(liquids.read_text())
    if not isinstance(document['liquidChunks'], list) or len(document['liquidChunks']) != 256:
        raise ValueError('Expected 256 liquid chunk slots')
    for chunk_index, chunk in enumerate(document['liquidChunks']):
        for instance_index, instance in enumerate((chunk or {}).get('instances', [])):
            if not instance:
                continue
            if instance['chunkIndex'] != chunk_index or instance['instanceIndex'] != instance_index:
                raise ValueError('Liquid identity disagrees with its slot')
            mesh = surface(instance)
            if mesh is None:
                continue
            # Each visible cell is a square: preserve the mask instead of blocking
            # a whole patch rectangle. Tall columns forbid underwater walking.
            for pair in mesh.triangles.reshape((-1, 2, 3, 3)):
                cell = pair.reshape((-1, 3)) - origin
                lo, hi = cell.min(axis=0), cell.max(axis=0)
                lo[1], hi[1] = -512., 512.
                solids.append(envelope(np.asarray([lo, hi])))
    if len(solids) > 4096:
        raise ValueError('Too many static collision envelopes')
    bounds = points[:, [0, 2]]
    profile = {'schema': 2, 'source_origin': origin.tolist(),
               'bounds': [bounds.min(axis=0).tolist(), bounds.max(axis=0).tolist()],
               'triangles': triangles.tolist(), 'solids': solids}
    doc, binary = read(visual)
    root = len(doc['nodes'])
    doc['nodes'].append({'name': 'Theia development world origin',
                         'translation': (-origin).tolist(), 'children': doc['scenes'][0]['nodes']})
    doc['scenes'][0]['nodes'] = [root]
    text = json.dumps(doc, separators=(',', ':'), allow_nan=False).encode()
    text += b' ' * (-len(text) % 4)
    if any(hashlib.sha256(Path(p).read_bytes()).hexdigest() != h for p, h in hashes.items()):
        raise ValueError('Source changed during world preparation')
    output.mkdir(parents=True)
    target = output / 'terrain.glb'
    with target.open('xb') as file:
        file.write(struct.pack('<4sII', b'glTF', 2, 28 + len(text) + len(binary)))
        file.write(struct.pack('<I4s', len(text), b'JSON'))
        file.write(text)
        file.write(struct.pack('<I4s', len(binary), b'BIN\0'))
        file.write(binary)
    profile['visual_sha256'] = list(hashlib.sha256(target.read_bytes()).digest())
    data = (json.dumps(profile, separators=(',', ':'), allow_nan=False) + '\n').encode()
    if len(data) > 32 * 1024 * 1024:
        raise ValueError('World profile exceeds byte budget')
    (output / 'terrain.json').write_bytes(data)
    report = {'source_hashes': hashes, 'origin': origin.tolist(), 'bounds': profile['bounds'],
              'triangles': len(triangles), 'object_solids': object_count,
              'water_exclusion_cells': len(solids) - object_count,
              'visual_bytes': target.stat().st_size, 'profile_bytes': len(data),
              'world_sha256': hashlib.sha256(data).hexdigest(), 'placements': audit,
              'scope': 'complete supplied tile; terrain walking, conservative static AABBs, water excluded; no interiors or swimming'}
    (output / 'preparation.json').write_text(json.dumps(report, indent=2) + '\n')
    return {k: v for k, v in report.items() if k not in ('placements', 'source_hashes')}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('placements', type=Path)
    parser.add_argument('liquids', type=Path)
    parser.add_argument('visual', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--origin', default='180,34,2400')
    args = parser.parse_args()
    print(json.dumps(prepare(args.source, args.placements, args.liquids, args.visual,
                             args.output, [float(v) for v in args.origin.split(',')]), indent=2))
