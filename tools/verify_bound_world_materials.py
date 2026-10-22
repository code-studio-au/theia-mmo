#!/usr/bin/env python3
"""Independently compare bound packages against qualified source packages."""
import argparse
import copy
import json
from pathlib import Path
from merge_static_glb import read
from package_world import identity
from shared_images import verify as verify_images
from unity_editor_reference import digest, read_json


def require(value, message):
    if not value:
        raise ValueError(message)


def verify(source, output):
    source, output = Path(source), Path(output)
    receipt = read_json(output / 'reference.json', 4 * 1024 * 1024)
    require(receipt['source_reference_sha256'] == digest(source / 'reference.json'), 'Source receipt differs')
    require(not (output / 'world.session.json').exists(), 'Reference unexpectedly publishes a session')
    before_index = read_json(source / 'world.chunks.json', 32768)
    after_index = read_json(output / 'world.chunks.json', 32768)
    require(len(before_index['chunks']) == len(after_index['chunks']) == len(receipt['packages']), 'Package coverage differs')
    expected_index = copy.deepcopy(before_index)
    materials = 0
    image_paths = set()
    for before_entry, after_entry in zip(expected_index['chunks'], after_index['chunks']):
        owner = before_entry['id']
        name = f'chunk_{owner[0]}_{owner[1]}'
        old, new = source / name, output / name
        left, first_binary = read(old / 'terrain.glb', allow_external_images=True)
        right, second_binary = read(new / 'terrain.glb', allow_external_images=True)
        require(first_binary == second_binary, 'Geometry binary differs')
        require(len(left.get('materials', [])) == len(right.get('materials', [])), 'Material count differs')
        catalog = read_json(old / 'triplanar.json', 65536)
        profiles = {p['id']: p for p in catalog['profiles']}
        for a, b in zip(left.get('materials', []), right.get('materials', [])):
            extras = b.get('extras', {})
            identifier = a.get('extras', {}).get('theia_triplanar')
            if identifier is None:
                continue
            profile = profiles[identifier]
            config = extras.pop('theia_triplanar_profile', None)
            expected = dict(schema=1, profile='synty-triplanar-world-v1', id=identifier,
                            tiling=profile['tiling'], falloff=profile['falloff'],
                            source_anchor=[0, 0, 0], textures={})
            for role, texture in profile['textures'].items():
                color = 'l' if role.endswith('normal') else 's'
                filename = f"{texture['sha256']}-{color}-10497-10497-9729-9729.png"
                expected['textures'][role] = '../shared/' + filename
                target = output / 'shared' / filename
                require(digest(target) == texture['sha256'] == digest(old / texture['file']), 'Source PNG bytes differ')
                image_paths.add(filename)
            require(config == expected, 'Source material parameters or dependencies differ')
            materials += 1
        require(left == right, 'Scene fields or fallback materials differ')
        first = read_json(old / 'terrain.json', 32 * 1024 * 1024)
        second = read_json(new / 'terrain.json', 32 * 1024 * 1024)
        first['visual_sha256'] = list(bytes.fromhex(digest(new / 'terrain.glb')))
        require(first == second, 'Collision differs')
        previous = read_json(old / 'world.manifest.json', 262144)
        manifest = read_json(new / 'world.manifest.json', 262144)
        require(manifest['schema'] == 3 and manifest['content_id'] == previous['content_id'] and
                manifest['revision'] == previous['revision'] + 1, 'Package version differs')
        require(manifest['visual_sha256'] == second['visual_sha256'] and
                manifest['collision_sha256'] == list(bytes.fromhex(digest(new / 'terrain.json'))), 'Manifest content binding differs')
        document, _ = read(new / 'terrain.glb', allow_external_images=True)
        verify_images(document, output, manifest['shared_images'])
        before_entry['package'] = list(bytes.fromhex(identity(manifest)))
        require(before_entry == after_entry, 'Contributor geometry or identity differs')
    require(expected_index == after_index, 'Index frame or fields differ')
    for image in (source / 'shared').iterdir():
        require(digest(image) == digest(output / 'shared' / image.name), 'Core image bytes differ')
    require(receipt['index_sha256'] == digest(output / 'world.chunks.json'), 'Index receipt binding differs')
    return dict(passed=True, packages=len(after_index['chunks']), bound_materials=materials,
                projection_images=len(image_paths), source_reference_sha256=digest(source / 'reference.json'),
                reference_sha256=digest(output / 'reference.json'),
                scope='Exact geometry, collision, scene/fallback fields, projection parameters and PNG bytes; no runtime or adoption claim')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(verify(args.source, args.output), indent=2))
