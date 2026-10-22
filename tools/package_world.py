#!/usr/bin/env python3
"""Bind a prepared local world to a stable logical ID and immutable revision.

Run after all visual/collision preparation. Writes only world.manifest.json;
never copies or modifies source assets. This is not a geometry or glTF validator.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct

MANIFEST = 'world.manifest.json'
MAX_COLLISION_BYTES = 32 * 1024 * 1024
MAX_VISUAL_BYTES = 2 * 1024 * 1024 * 1024


def valid_id(value):
    return (isinstance(value, str) and 1 <= len(value) <= 96
            and value[0] in 'abcdefghijklmnopqrstuvwxyz'
            and value[-1] in 'abcdefghijklmnopqrstuvwxyz0123456789'
            and all(c in 'abcdefghijklmnopqrstuvwxyz0123456789._-' for c in value)
            and '..' not in value)


def identity(document):
    name = document['content_id'].encode('ascii')
    schema = document['schema']
    if schema not in (1,2,3):
        raise ValueError('Unsupported package schema')
    data = (f'theia-world-package-v{schema}\0'.encode() + struct.pack('<H', len(name)) + name
            + struct.pack('<IIQ', document['revision'], document['collision_schema'], document['collision_bytes'])
            + bytes(document['collision_sha256']) + struct.pack('<Q', document['visual_bytes'])
            + bytes(document['visual_sha256']))
    if schema >= 2:
        from shared_images import canonical
        images = canonical(document['shared_images'])
        data += struct.pack('<H',len(images))
        for image in images:
            data += bytes(image['sha256']) + struct.pack('<QB4I',image['bytes'],int(image['color_space']=='linear'),*image['sampler'])
    return hashlib.sha256(data).hexdigest()


def file_hash(path, limit):
    before = path.stat()
    if not 1 <= before.st_size <= limit:
        raise ValueError(f'Content file outside byte budget: {path.name}')
    digest = hashlib.sha256()
    count = 0
    with path.open('rb') as file:
        while block := file.read(1024 * 1024):
            count += len(block)
            if count > limit:
                raise ValueError('Content grew beyond byte budget')
            digest.update(block)
    after = path.stat()
    if (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns) or count != before.st_size:
        raise ValueError('Content changed during packaging')
    return count, digest.digest()


def package(world, content_id, revision, shared_images=False):
    if not valid_id(content_id) or type(revision) is not int or not 1 <= revision <= 0xffffffff:
        raise ValueError('Invalid stable content ID or nonzero u32 revision')
    target = world / MANIFEST
    if target.exists():
        raise FileExistsError('Manifest already exists; publish a new prepared directory/revision')
    profile = world / 'terrain.json'
    size, collision_hash = file_hash(profile, MAX_COLLISION_BYTES)
    data = profile.read_bytes()
    if len(data) != size or hashlib.sha256(data).digest() != collision_hash:
        raise ValueError('Collision changed during packaging')
    document = json.loads(data)
    if type(document.get('schema')) is not int or document['schema'] not in (1, 2, 3, 4):
        raise ValueError('Unsupported collision schema')
    if document['schema'] == 4:
        from static_shapes import validate
        triangles = document.get('triangles', [])
        solids = document.get('solids', [])
        checked = validate(document.get('static_shapes'), len(triangles), len(solids) + len(document.get('camera_solids', [])) + bool(triangles))
        if document.get('walk_surfaces') or document.get('step_height', 0) != 0 or (not triangles and not solids and not checked['world_geometry']):
            raise ValueError('Invalid general static-shape profile')
    visual_size, visual_hash = file_hash(world / 'terrain.glb', MAX_VISUAL_BYTES)
    if document.get('visual_sha256') != list(visual_hash):
        raise ValueError('Visual does not match collision profile; finish preparation first')
    result = {'schema': 1, 'content_id': content_id, 'revision': revision,
              'collision_schema': document['schema'], 'collision_bytes': size,
              'collision_sha256': list(collision_hash), 'visual_bytes': visual_size,
              'visual_sha256': list(visual_hash)}
    if shared_images:
        from merge_static_glb import read
        from shared_images import collect
        from material_profiles import profiles
        gltf,_ = read(world/'terrain.glb',allow_external_images=True)
        result['schema'] = 3 if profiles(gltf) else 2
        result['shared_images'] = collect(gltf,world.parent)
    options = {'indent':2} if result['schema']==1 else {'separators':(',',':')}
    encoded = (json.dumps(result, allow_nan=False,**options) + '\n').encode()
    if len(encoded) > (8192 if result['schema']==1 else 256*1024):
        raise ValueError('Manifest exceeds byte budget')
    with target.open('xb') as file:
        file.write(encoded)
    return {'content_id': content_id, 'revision': revision, 'identity': identity(result),
            'collision_bytes': size, 'visual_bytes': visual_size}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('world', type=Path)
    parser.add_argument('--content-id', required=True)
    parser.add_argument('--revision', required=True, type=int)
    parser.add_argument('--shared-images',action='store_true')
    args = parser.parse_args()
    print(json.dumps(package(args.world.resolve(), args.content_id, args.revision,args.shared_images), indent=2))
