#!/usr/bin/env python3
"""Author a new bounded schema-2 index from immutable static chunk packages.

Only the requested new index is written. Packages and the active index stay
unchanged. This derives conservative contributors, not terrain partitioning.
"""
import argparse
import hashlib
import json
import math
from pathlib import Path

import numpy as np

from static_geometry import hierarchy, transformed_bounds
from merge_static_glb import read as read_static_glb
from package_world import (MANIFEST, MAX_COLLISION_BYTES, MAX_VISUAL_BYTES,
                           file_hash, identity)


def overlaps(owner, bounds, cell):
    for axis in range(2):
        relative = (owner[axis] - cell[axis]) * 128
        low, high = (relative + bounds[n][axis] for n in range(2))
        if not (0 <= low <= 128 if low == high else low < 128 and high > 0):
            return False
    return True


def collision_bounds(profile):
    points = []
    if profile.get('schema') == 4:
        from static_shapes import validate
        checked = validate(profile.get('static_shapes'), len(profile.get('triangles', [])), len(profile.get('solids', [])) + len(profile.get('camera_solids', [])) + bool(profile.get('triangles')))
        for box in checked['bounds']:
            points.extend(np.asarray(box, dtype=np.float64)[:, [0, 2]])
    for name in ('triangles', 'walk_surfaces'):
        triangles = np.asarray(profile.get(name, []), dtype=np.float32)
        if triangles.size:
            if triangles.ndim != 3 or triangles.shape[1:] != (3, 3):
                raise ValueError('Invalid collision triangle layout')
            points.extend(triangles.reshape(-1, 3)[:, [0, 2]])
    for name in ('solids', 'camera_solids'):
        for solid in profile.get(name, []):
            center = np.asarray(solid['center'], dtype=np.float32)
            size = np.asarray(solid['size'], dtype=np.float32)
            if center.shape != (3,) or size.shape != (3,) or np.any(size <= 0):
                raise ValueError('Invalid collision solid')
            points.extend([(center - size / np.float32(2))[[0, 2]],
                           (center + size / np.float32(2))[[0, 2]]])
    points = np.asarray(points, dtype=np.float64)
    if points.ndim != 2 or points.shape[1] != 2 or not np.isfinite(points).all():
        raise ValueError('Missing or nonfinite collision geometry')
    return np.stack([points.min(axis=0), points.max(axis=0)])


def package_bounds(directory, expected):
    manifest_path = directory / MANIFEST
    if not 1 <= manifest_path.stat().st_size <= 256 * 1024:
        raise ValueError('Invalid manifest byte budget')
    manifest = json.loads(manifest_path.read_bytes())
    if manifest.get('schema') not in (1,2,3) or list(bytes.fromhex(identity(manifest))) != expected:
        raise ValueError('Package identity disagrees with input index')
    for filename, field, limit in [('terrain.json', 'collision', MAX_COLLISION_BYTES),
                                   ('terrain.glb', 'visual', MAX_VISUAL_BYTES)]:
        size, digest = file_hash(directory / filename, limit)
        if size != manifest[field + '_bytes'] or list(digest) != manifest[field + '_sha256']:
            raise ValueError('Package artifacts disagree with manifest')
    profile = json.loads((directory / 'terrain.json').read_bytes())
    if profile.get('schema') != manifest['collision_schema'] or profile.get('visual_sha256') != manifest['visual_sha256']:
        raise ValueError('Collision/visual package binding disagrees')
    collision = collision_bounds(profile)
    # Reject animation, skins, external data and extensions before loading actual
    # positions. Accessor min/max metadata alone is not an extent proof.
    document, binary = read_static_glb(directory / 'terrain.glb',allow_external_images=manifest['schema']>=2)
    from material_profiles import profiles
    if bool(profiles(document)) != (manifest['schema']==3):
        raise ValueError('Bound material profiles require package schema 3')
    if manifest['schema']>=2:
        from shared_images import verify
        verify(document,directory.parent,manifest['shared_images'])
    if any(primitive.get('targets') for mesh in document.get('meshes', [])
           for primitive in mesh.get('primitives', [])):
        raise ValueError('Morphing chunk geometry is unsupported')
    # Bevy validates/culls transformed mesh-local AABB corners, which can extend
    # beyond the transformed vertices of a rotated sparse mesh. Bind that same
    # conservative envelope instead of only the tighter actual-vertex bounds.
    _, matrices = hierarchy(document)
    envelopes = [transformed_bounds(document, binary, node['mesh'], matrices[n])[:, [0, 2]]
                 for n, node in enumerate(document['nodes']) if n in matrices and 'mesh' in node]
    if not envelopes:
        raise ValueError('Missing static visual geometry')
    visual = np.stack([np.min([box[0] for box in envelopes], axis=0),
                       np.max([box[1] for box in envelopes], axis=0)])
    bounds = np.stack([np.minimum(collision[0], visual[0]),
                       np.maximum(collision[1], visual[1])])
    # Outward binary-grid rounding covers float32 collider/transform rounding
    # without introducing a halo at an exactly authored 0/128 metre seam.
    bounds[0] = np.floor(bounds[0] * 1024) / 1024
    bounds[1] = np.ceil(bounds[1] * 1024) / 1024
    if np.any(np.abs(bounds) > 1023):
        raise ValueError('Geometry exceeds bounded local contributor envelope')
    bounds[bounds == 0] = 0
    return bounds.tolist()


def author(directory, output, resident_limit=32):
    if type(resident_limit) is not int or not 1 <= resident_limit <= 32:
        raise ValueError('Contributor limit must be 1..32')
    if output.exists():
        raise FileExistsError('Output index must be new; active content is not overwritten')
    source = directory / 'world.chunks.json'
    if not 1 <= source.stat().st_size <= 32 * 1024:
        raise ValueError('Input index exceeds byte budget')
    original = json.loads(source.read_bytes())
    entries = original.get('chunks')
    frame = original.get('query_origin')
    if original.get('schema') not in (1, 2) or original.get('chunk_metres') != 128 or not isinstance(entries, list) or not 1 <= len(entries) <= 64:
        raise ValueError('Unsupported or over-budget input index')
    if not isinstance(frame, list) or len(frame) != 3 or not all(type(v) in (int, float) and math.isfinite(v) and abs(v) <= 1e12 for v in frame):
        raise ValueError('Invalid query frame')
    seen = set()
    published = []
    for entry in entries:
        owner = entry['id']
        if not isinstance(owner, list) or len(owner) != 2 or any(type(v) is not int or abs(v) > 7_000_000_000 for v in owner) or tuple(owner) in seen:
            raise ValueError('Invalid or duplicate package owner')
        if any(abs(owner[n] * 128 - frame[n * 2]) > 3000 for n in range(2)) or abs(frame[1]) > 3000:
            raise ValueError('Package owner outside bounded query frame')
        seen.add(tuple(owner))
        bounds = package_bounds(directory / f'chunk_{owner[0]}_{owner[1]}', entry['package'])
        published.append({'id': owner, 'package': entry['package'], 'bounds': bounds})
    published.sort(key=lambda e: e['id'])
    maximum = 0
    for cell in published:
        required = {tuple(cell['id'])}
        required.update(tuple(e['id']) for e in published if overlaps(e['id'], e['bounds'], cell['id']))
        maximum = max(maximum, len(required))
        if len(required) > resident_limit:
            raise ValueError('Cell contributors exceed selected residency budget')
    document = {'schema': 2, 'chunk_metres': 128, 'query_origin': frame, 'chunks': published}
    encoded = (json.dumps(document, separators=(',', ':'), allow_nan=False) + '\n').encode()
    if len(encoded) > 32 * 1024:
        raise ValueError('Authored index exceeds byte budget')
    # Recheck every input byte binding before publishing the derived metadata.
    if json.loads(source.read_bytes()) != original:
        raise ValueError('Input index changed during authoring')
    for entry in published:
        owner = entry['id']
        package = directory / f'chunk_{owner[0]}_{owner[1]}'
        manifest = json.loads((package / MANIFEST).read_bytes())
        if list(bytes.fromhex(identity(manifest))) != entry['package']:
            raise ValueError('Package changed during authoring')
        for filename, field, limit in [('terrain.json', 'collision', MAX_COLLISION_BYTES),
                                       ('terrain.glb', 'visual', MAX_VISUAL_BYTES)]:
            size, digest = file_hash(package / filename, limit)
            if size != manifest[field + '_bytes'] or list(digest) != manifest[field + '_sha256']:
                raise ValueError('Package artifact changed during authoring')
    with output.open('xb') as file:
        file.write(encoded)
    return {'chunks': len(published), 'maximum_cell_contributors': maximum,
            'index_sha256': hashlib.sha256(encoded).hexdigest(),
            'scope': 'bounded static package contributor authoring; no tile partitioning or streaming capacity claim'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--resident-limit', type=int, choices=range(1, 33), default=32)
    args = parser.parse_args()
    print(json.dumps(author(args.directory.resolve(), args.output.resolve(), args.resident_limit), indent=2))
