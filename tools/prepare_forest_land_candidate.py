#!/usr/bin/env python3
"""Create a provisional land-walking forest candidate without invisible water collision.

The complete reference remains immutable. Swimming, buoyancy and water volumes
are not implemented; visible water keeps its existing presentation approximation.
This tool deliberately publishes neither a session nor a new launcher default.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
from package_world import package
from unity_editor_reference import digest, read_json

WATER_OBJECT = 'GlobalObjectId_V1-2-198eb44694e31524baead99cabfac386-864140990-0'


def prepare(source, reference, output):
    source, reference = Path(source).resolve(strict=True), Path(reference).resolve(strict=True)
    output = Path(output).resolve()
    build = Path(__file__).resolve().parents[1] / 'build'
    if output.exists() or output == build or not output.is_relative_to(build):
        raise ValueError('Use a fresh ignored candidate output')
    if (source / 'world.session.json').exists():
        raise ValueError('Expected an immutable reference without a session')
    metadata = read_json(reference / 'reference.json', 2 * 1024 * 1024)
    matches = [row for row in metadata['objects'] if row['id'] == WATER_OBJECT]
    if len(matches) != 1:
        raise ValueError('Expected exactly one qualified invisible water collider')
    row = matches[0]
    owner = row['owner']
    name = f'chunk_{owner[0]}_{owner[1]}'
    original = source / name
    before = read_json(original / 'terrain.json', 32 * 1024 * 1024)
    profile = next(p for p in metadata['profiles'] if p['owner'] == owner)
    path = reference / profile['file']
    if digest(path) != profile['sha256']:
        raise ValueError('Qualified source collision bytes changed')
    expected = read_json(path, 32 * 1024 * 1024)
    expected['visual_sha256'] = before['visual_sha256']
    if before != expected:
        raise ValueError('Candidate collision differs from qualified source')
    objects = before['static_shapes']['objects']
    removed = objects[row['collider']]
    if removed['kind'] != 'mesh' or removed['convex']:
        raise ValueError('Qualified water collider type changed')
    shutil.copytree(source, output, copy_function=os.link)
    try:
        target = output / name
        objects.pop(row['collider'])
        shapes = before['static_shapes']
        used = sorted({o['mesh'] for o in objects if o['kind'] == 'mesh'})
        remap = {old: new for new, old in enumerate(used)}
        shapes['meshes'] = [shapes['meshes'][i] for i in used]
        for obj in objects:
            if obj['kind'] == 'mesh':
                obj['mesh'] = remap[obj['mesh']]
        path = target / 'terrain.json'
        path.unlink()
        path.write_text(json.dumps(before, separators=(',', ':'), allow_nan=False) + '\n')
        manifest = read_json(target / 'world.manifest.json', 262144)
        (target / 'world.manifest.json').unlink()
        published = package(target, manifest['content_id'], manifest['revision'] + 1, shared_images=True)
        path = output / 'world.chunks.json'
        index = read_json(path, 32768)
        next(c for c in index['chunks'] if c['id'] == owner)['package'] = list(bytes.fromhex(published['identity']))
        path.unlink()
        path.write_text(json.dumps(index, separators=(',', ':')) + '\n')
        path = output / 'reference.json'
        path.unlink()
        receipt = dict(schema=1, profile='forest-land-candidate-v1',
            source_reference_sha256=digest(source / 'reference.json'),
            source_shape_reference_sha256=digest(reference / 'reference.json'),
            excluded_source_object=row, excluded_shape=removed,
            index_sha256=digest(output / 'world.chunks.json'),
            policy='Invisible source water plane excluded from all queries; visible water is presentation-only; no swimming/buoyancy',
            scope='Provisional complete-visual land-walking candidate; no session, default adoption or complete movement qualification')
        path.write_text(json.dumps(receipt, indent=2) + '\n')
        return receipt
    except BaseException:
        shutil.rmtree(output)
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['source', 'reference', 'output']:
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    print(json.dumps(prepare(args.source, args.reference, args.output), indent=2))
