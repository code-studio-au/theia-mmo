#!/usr/bin/env python3
"""Stage selected verified static prototypes for the editor; no native window.

Source payloads and outputs remain local. Accept independently verified embedded
standard GLBs, including the explicit static foliage adaptation. Source wind,
animated highlights, projected shaders and animation are not implemented here.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import tempfile

import numpy as np
from merge_static_glb import read
from static_geometry import accessor, hierarchy, transformed_bounds

MAX_ASSET_BYTES = 32 * 1024 * 1024
MAX_LIBRARY_BYTES = 128 * 1024 * 1024


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def collision_mesh(document, binary, trunk_only=False):
    """Explicit Theia static-surface adaptation; not Unity collider import."""
    vertices, indices = [], []
    for primitive in document['meshes'][0]['primitives']:
        if trunk_only:
            material = primitive.get('material')
            if material is None or document['materials'][material].get('name') != 'trunk':
                continue
        if primitive.get('mode', 4) != 4 or primitive.get('targets'):
            raise ValueError('Collision needs static triangle primitives')
        points = accessor(document, binary, primitive['attributes']['POSITION'])
        if points.shape[1] != 3:
            raise ValueError('Collision needs position triplets')
        faces = (accessor(document, binary, primitive['indices']).reshape(-1)
                 if 'indices' in primitive else np.arange(len(points)))
        if len(faces) % 3 or faces.dtype.kind not in 'ui' or np.any(faces < 0) or np.any(faces >= len(points)):
            raise ValueError('Invalid collision triangle indices')
        offset = len(vertices)
        vertices.extend(points.tolist())
        for face in faces.reshape((-1, 3)):
            a, b, c = points[face]
            if np.linalg.norm(np.cross(b-a, c-a)) > 1e-10:
                indices.append((face + offset).tolist())
    if not indices or len(indices) > 65536 or len(vertices) > 262144:
        raise ValueError('Collision mesh exceeds budget or has no surface')
    return json.dumps(dict(vertices=vertices, indices=indices), separators=(',', ':'), allow_nan=False).encode()


def stage(batch, selection, output, prototypes=()):
    batch = batch.resolve(strict=True)
    selection = selection.resolve(strict=True)
    output = output.absolute()
    if output.exists() or batch.is_relative_to(output) or selection.is_relative_to(output):
        raise ValueError('Use a new output folder outside the inputs')
    chosen = json.loads(selection.read_bytes())
    if chosen['schema'] != 1 or not 1 <= len(chosen['assets']) <= 64:
        raise ValueError('Unsupported scenery selection')
    records = [(r, batch / 'prototypes' / r['key'])
               for r in json.loads((batch / 'batch.json').read_bytes())['converted']]
    for prototype_path in prototypes:
        root = Path(prototype_path).resolve(strict=True)
        if root.is_relative_to(output):
            raise ValueError('Use an output outside the prototype inputs')
        conversion = json.loads((root / 'conversion.json').read_bytes())
        if conversion.get('material_plan', {}).get('profile') != 'unity-standard-normal-v1':
            raise ValueError('Standalone prototypes require the qualified Standard normal profile')
        records.append((dict(source=conversion['prefab'], key=digest(root / 'conversion.json'),
                             profile='standard-normal', prototype_sha256=conversion['preview_sha256']), root))
    prepared, used, total = [], set(), 0
    for item in chosen['assets']:
        identifier = item['id']
        if not re.fullmatch(r'[a-z][a-z0-9._-]{0,94}[a-z0-9]', identifier) or '..' in identifier or identifier in used:
            raise ValueError('Invalid or duplicate scenery ID')
        used.add(identifier)
        profile = item.get('profile', 'standard-opaque')
        if profile not in ('standard-opaque', 'full-static', 'standard-normal'):
            raise ValueError('Unsupported scenery material profile')
        prototype = item.get('prototype')
        if prototype is not None and not re.fullmatch('[0-9a-f]{64}', prototype):
            raise ValueError('Invalid explicit prototype key')
        matches = [(r, root) for r, root in records if Path(r['source']).stem == item['prefab']
                   and r.get('profile') == profile and (prototype is None or r['key'] == prototype)]
        if len(matches) != 1:
            raise ValueError(f'Expected one qualified material binding: {item["prefab"]}')
        record, root = matches[0]
        if not re.fullmatch('[0-9a-f]{64}', record['key']):
            raise ValueError('Invalid prototype key')
        asset = root / 'preview.glb'
        size = asset.stat().st_size
        total += size
        if not 0 < size <= MAX_ASSET_BYTES or total > MAX_LIBRARY_BYTES:
            raise ValueError('Scenery byte budget exceeded')
        proof = json.loads((root / 'independent-verification.json').read_bytes())
        sha = digest(asset)
        if proof.get('passed') is not True or proof.get('asset_sha256') != sha or record['prototype_sha256'] != sha:
            raise ValueError('Prototype verification no longer matches source bytes')
        doc, binary = read(asset)
        _, transforms = hierarchy(doc)
        if len(doc['nodes']) != 1 or len(doc['meshes']) != 1 or list(transforms) != [0] or not np.array_equal(transforms[0], np.eye(4)):
            raise ValueError('Scenery prototype must have one canonical mesh node')
        if doc['nodes'][0].get('mesh') != 0 or doc.get('extensionsUsed') or doc.get('extensionsRequired'):
            raise ValueError('Scenery requires ordinary standard-material glTF')
        bounds = transformed_bounds(doc, binary, 0, np.eye(4))
        if not np.isfinite(bounds).all() or np.abs(bounds).max() > 128:
            raise ValueError('Scenery bounds exceed local prototype limits')
        conversion = json.loads((root / 'conversion.json').read_bytes())
        if profile == 'standard-normal':
            geometry = json.loads((root / 'canonical-reference.json').read_bytes())
            if (conversion.get('material_plan', {}).get('profile') != 'unity-standard-normal-v1'
                    or conversion.get('preview_sha256') != sha
                    or geometry.get('passed') is not True or geometry.get('asset_sha256') != sha
                    or 'normal' not in proof.get('verified_material', {})):
                raise ValueError('Standard normal prototype lacks matching geometry/material evidence')
        if profile == 'full-static':
            if conversion.get('profile', {}).get('profile') != 'synty-full-static-foliage-v1' or conversion.get('canonical_mesh') is not True:
                raise ValueError('Foliage needs the qualified canonical static profile')
            reference = json.loads((root / 'geometry-reference.json').read_bytes())
            points = np.asarray(reference['vertices_z_up_metres'], dtype=np.float64)
            if points.ndim != 2 or points.shape[1] != 3 or not np.isfinite(points).all():
                raise ValueError('Invalid foliage geometry reference')
            points = points[:, [0, 2, 1]] * [1, 1, -1]
            expected_bounds = [points.min(axis=0), points.max(axis=0)]
        else:
            expected_bounds = [proof['bounds_y_up_metres']['min'], proof['bounds_y_up_metres']['max']]
        if not np.allclose(bounds, expected_bounds, atol=1e-5, rtol=0):
            raise ValueError('Scenery bounds differ from independent verification')
        entry = {k: item[k] for k in ('id', 'name', 'category')}
        entry.update(file=f'{sha}.glb', bytes=size, sha256=sha, bounds=bounds.tolist(),
                     triangles=proof['triangles'], source=dict(prefab=record['source'],
                     archive_sha256=conversion['archive_sha256'],
                     verification_sha256=digest(root / 'independent-verification.json'),
                     profile=record['profile'], prototype=record['key']))
        if profile == 'full-static':
            entry['source']['limitations'] = ['Static rest geometry; source wind, gust highlights and LOD crossfade are unapplied']
            entry['source']['geometry_reference_sha256'] = digest(root / 'geometry-reference.json')
            entry['source']['conversion_sha256'] = digest(root / 'conversion.json')
        if profile == 'standard-normal':
            entry['source']['limitations'] = [conversion['normal_scope'], conversion['material_plan']['scope']]
            entry['source']['geometry_reference_sha256'] = digest(root / 'canonical-reference.json')
            entry['source']['conversion_sha256'] = digest(root / 'conversion.json')
        choice = item.get('collision')
        collision = None
        if choice in ('triangle_mesh', 'trunk_triangles'):
            if choice == 'trunk_triangles' and profile != 'full-static':
                raise ValueError('Trunk collision requires qualified foliage branches')
            collision = collision_mesh(doc, binary, trunk_only=choice == 'trunk_triangles')
            entry['source']['collision_scope'] = choice
            collision_hash = hashlib.sha256(collision).hexdigest()
            if len(collision) > MAX_ASSET_BYTES:
                raise ValueError('Collision byte budget exceeded')
            total += len(collision)
            entry['collision'] = dict(kind='triangle_mesh', file=f'{collision_hash}.collision.json',
                                      bytes=len(collision), sha256=collision_hash)
        elif choice == 'decorative':
            entry['collision'] = dict(kind=choice)
        else:
            raise ValueError('Choose explicit triangle_mesh, trunk_triangles or decorative collision')
        if total > MAX_LIBRARY_BYTES:
            raise ValueError('Scenery byte budget exceeded')
        prepared.append((entry, asset, collision))
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.scenery-', dir=output.parent) as folder:
        temporary = Path(folder)
        for entry, asset, collision in prepared:
            shutil.copyfile(asset, temporary / entry['file'])
            if collision is not None:
                (temporary / entry['collision']['file']).write_bytes(collision)
            if digest(temporary / entry['file']) != entry['sha256']:
                raise ValueError('Source asset changed during staging')
        catalog = dict(schema=1, id=chosen['id'], assets=[entry for entry, _, _ in prepared])
        (temporary / 'catalog.json').write_text(json.dumps(catalog, indent=2) + '\n')
        output.mkdir()  # Refuse a racing/existing destination.
        try:
            for file in temporary.iterdir():
                file.rename(output / file.name)
        except Exception:
            shutil.rmtree(output)
            raise
    return dict(assets=len(prepared), bytes=total, catalog=str(output / 'catalog.json'),
                collision='explicit source triangles or nonblocking decoration', desktop_used=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('batch', type=Path)
    parser.add_argument('selection', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--prototype', action='append', type=Path, default=[],
                        help='additional independently verified Standard normal prototype directory')
    args = parser.parse_args()
    print(json.dumps(stage(args.batch, args.selection, args.output, args.prototype), indent=2))
