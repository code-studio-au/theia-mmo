#!/usr/bin/env python3
"""Extract immutable embedded PNGs into standard shared Bevy image paths.

Writes a new package and shared resources beside it. The source and active chunk
index stay unchanged. This is image packaging, not terrain partitioning.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import shutil
import struct

from merge_static_glb import read
from package_world import (MANIFEST, MAX_COLLISION_BYTES, MAX_VISUAL_BYTES,
                           file_hash, identity, package, valid_id)
from shared_images import (MAX_IMAGE_BYTES, canonical, filename, png_pixels,
                           sampler, usage_fields)


def verify_source(source):
    manifest_path = source / MANIFEST
    if not 1 <= manifest_path.stat().st_size <= 8192:
        raise ValueError('Source manifest exceeds byte budget')
    manifest = json.loads(manifest_path.read_bytes())
    if manifest.get('schema') != 1:
        raise ValueError('Image extraction requires an embedded schema-1 source package')
    for name, field, limit in [('terrain.json', 'collision', MAX_COLLISION_BYTES),
                              ('terrain.glb', 'visual', MAX_VISUAL_BYTES)]:
        size, digest = file_hash(source / name, limit)
        if size != manifest[field + '_bytes'] or list(digest) != manifest[field + '_sha256']:
            raise ValueError('Source artifact disagrees with immutable manifest')
    return manifest


def separate_texture_uses(document):
    fields = list(usage_fields(document))
    uses = {}
    for field, color in fields:
        index = field['index']
        if type(index) is not int or not 0 <= index < len(document.get('textures', [])):
            raise ValueError('Unknown material texture')
        uses.setdefault(index, set()).add(color)
    for index, colors in uses.items():
        if len(colors) > 1:
            duplicate = len(document['textures'])
            document['textures'].append(copy.deepcopy(document['textures'][index]))
            for field, color in fields:
                if field['index'] == index and color == 'linear':
                    field['index'] = duplicate
    return {field['index'] for field, color in usage_fields(document) if color == 'linear'}


def extract_document(document, binary, shared):
    linear = separate_texture_uses(document)
    original_images = document.get('images', [])
    views = document.get('bufferViews', [])
    image_views = {image['bufferView'] for image in original_images}
    if any(accessor['bufferView'] in image_views for accessor in document.get('accessors', [])):
        raise ValueError('Image and geometry buffer views must be separate')
    resources, remap_images, new_images = {}, {}, []
    by_hash = {}
    for n, texture in enumerate(document.get('textures', [])):
        source = texture['source']
        if type(source) is not int or not 0 <= source < len(original_images):
            raise ValueError('Unknown texture image')
        original = original_images[source]
        if original.get('mimeType') != 'image/png':
            raise ValueError('Shared world images currently require PNG input')
        view = views[original['bufferView']]
        start, size = view.get('byteOffset', 0), view['byteLength']
        image = memoryview(binary)[start:start + size]
        if not 1 <= len(image) <= MAX_IMAGE_BYTES:
            raise ValueError('Shared PNG exceeds encoded byte budget')
        digest = hashlib.sha256(image).digest()
        descriptor = {'sha256': list(digest), 'bytes': size,
                      'color_space': 'linear' if n in linear else 'srgb',
                      'sampler': sampler(document, texture)}
        canonical([descriptor])
        name = filename(descriptor)
        path = shared / name
        if path.exists() or path.is_symlink():
            if path.resolve(strict=True) != path or file_hash(path, MAX_IMAGE_BYTES) != (size, digest):
                raise ValueError('Existing shared image is unsafe or corrupt')
        elif digest in by_hash:
            try:
                os.link(by_hash[digest], path)
            except OSError:
                with path.open('xb') as file:
                    file.write(image)
        else:
            with path.open('xb') as file:
                file.write(image)
        png_pixels(path)
        by_hash.setdefault(digest, path)
        if name not in remap_images:
            remap_images[name] = len(new_images)
            replacement = {key: value for key, value in original.items() if key not in ('uri', 'bufferView')}
            replacement['uri'] = '../shared/' + name
            new_images.append(replacement)
            resources[name] = descriptor
        texture['source'] = remap_images[name]
    canonical(list(resources.values()))
    document['images'] = new_images
    packed = bytearray()
    new_views, remap_views = [], {}
    for index, view in enumerate(views):
        if index in image_views:
            continue
        packed.extend(b'\0' * (-len(packed) % 4))
        remap_views[index] = len(new_views)
        changed = dict(view, byteOffset=len(packed), buffer=0)
        offset = view.get('byteOffset', 0)
        packed.extend(memoryview(binary)[offset:offset + view['byteLength']])
        new_views.append(changed)
    for accessor in document.get('accessors', []):
        accessor['bufferView'] = remap_views[accessor['bufferView']]
    document['bufferViews'] = new_views
    document['buffers'][0]['byteLength'] = len(packed)
    packed.extend(b'\0' * (-len(packed) % 4))
    return packed, canonical(list(resources.values()))


def externalize(source, destination, content_id, revision):
    source, destination = source.resolve(strict=True), destination.resolve()
    if destination.exists() or destination.is_relative_to(source):
        raise FileExistsError('Destination must be new and outside the immutable source package')
    if not valid_id(content_id) or type(revision) is not int or not 1 <= revision <= 0xffffffff:
        raise ValueError('Invalid new package identity/revision')
    manifest = verify_source(source)
    if content_id == manifest['content_id'] and revision <= manifest['revision']:
        raise ValueError('Changed representation requires a newer immutable revision')
    document, binary = read(source / 'terrain.glb')
    if any(p.get('targets') for mesh in document.get('meshes', []) for p in mesh.get('primitives', [])):
        raise ValueError('Morphing static world geometry is unsupported')
    destination.parent.mkdir(parents=True, exist_ok=True)
    shared = destination.parent / 'shared'
    shared.mkdir(exist_ok=True)
    if shared.resolve(strict=True) != shared:
        raise ValueError('Shared directory alias is not allowed')
    destination.mkdir()
    try:
        packed, images = extract_document(document, binary, shared)
        text = json.dumps(document, separators=(',', ':'), allow_nan=False).encode()
        text += b' ' * (-len(text) % 4)
        with (destination / 'terrain.glb').open('xb') as file:
            file.write(struct.pack('<4sII', b'glTF', 2, 28 + len(text) + len(packed)))
            file.write(struct.pack('<I4s', len(text), b'JSON'))
            file.write(text)
            file.write(struct.pack('<I4s', len(packed), b'BIN\0'))
            file.write(packed)
        visual_bytes, visual_hash = file_hash(destination / 'terrain.glb', MAX_VISUAL_BYTES)
        profile = json.loads((source / 'terrain.json').read_bytes())
        if profile.get('visual_sha256') != manifest['visual_sha256']:
            raise ValueError('Source collision/visual binding disagrees')
        profile['visual_sha256'] = list(visual_hash)
        (destination / 'terrain.json').write_text(json.dumps(profile, separators=(',', ':'), allow_nan=False) + '\n')
        if identity(verify_source(source)) != identity(manifest):
            raise ValueError('Source package changed during extraction')
        published = package(destination, content_id, revision, shared_images=True)
        return dict(published, shared_images=len(images), shared_encoded_bytes=sum(i['bytes'] for i in images),
                    original_visual_bytes=manifest['visual_bytes'], new_visual_bytes=visual_bytes,
                    scope='shared image extraction only; source and active chunk index unchanged; no terrain partitioning')
    except BaseException:
        shutil.rmtree(destination)
        raise


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('destination', type=Path)
    parser.add_argument('--content-id', required=True)
    parser.add_argument('--revision', type=int, required=True)
    args = parser.parse_args()
    print(json.dumps(externalize(args.source, args.destination, args.content_id, args.revision), indent=2))
