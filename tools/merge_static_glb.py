"""Join static GLBs with qualified material extensions, retaining mesh references.

This narrow offline helper rejects animation, skins and cameras. External images
require an explicit caller opt-in and caller-side path/dependency validation.
It rebases standard glTF indices; it never decodes/re-encodes image payloads.
"""
import copy
import json
import struct
from static_material_extensions import validate as validate_extensions


def read(path, allow_external_images=False):
    with path.open('rb') as file:
        header = file.read(12)
        if len(header) != 12 or struct.unpack('<4sII', header) != (b'glTF', 2, path.stat().st_size):
            raise ValueError('Expected complete GLB 2.0')
        size, kind = struct.unpack('<I4s', file.read(8))
        if kind != b'JSON' or size % 4:
            raise ValueError('Expected aligned JSON chunk')
        doc = json.loads(file.read(size))
        size, kind = struct.unpack('<I4s', file.read(8))
        binary = file.read(size)
        if kind != b'BIN\0' or len(binary) != size or size % 4 or file.read(1):
            raise ValueError('Expected exactly one aligned BIN chunk')
    validate_extensions(doc)
    if any(doc.get(k) for k in ('animations', 'skins', 'cameras')):
        raise ValueError('Only static, unskinned camera-free GLBs are supported')
    if len(doc.get('scenes', [])) != 1 or len(doc.get('buffers', [])) != 1 or 'uri' in doc['buffers'][0]:
        raise ValueError('Expected one scene and one embedded buffer')
    if not 0 <= len(binary) - doc['buffers'][0]['byteLength'] <= 3:
        raise ValueError('Invalid embedded buffer length')
    if not allow_external_images and any('uri' in image or 'bufferView' not in image for image in doc.get('images', [])):
        raise ValueError('External images are unsupported')
    if any('sparse' in a or 'bufferView' not in a for a in doc.get('accessors', [])):
        raise ValueError('Sparse/implicit accessors are unsupported')
    if any('skin' in n or 'camera' in n for n in doc.get('nodes', [])):
        raise ValueError('Nonstatic node')
    for view in doc.get('bufferViews', []):
        if view.get('buffer', 0) != 0 or view.get('byteOffset', 0) < 0 or view['byteLength'] <= 0 or view.get('byteOffset', 0) + view['byteLength'] > doc['buffers'][0]['byteLength']:
            raise ValueError('Buffer view outside embedded data')
    return doc, binary


def merge_documents(first, second, binary_offset, allow_external_images=False):
    validate_extensions(first); validate_extensions(second)
    result, added = copy.deepcopy(first), copy.deepcopy(second)
    keys = ('accessors', 'bufferViews', 'images', 'samplers', 'textures', 'materials', 'meshes', 'nodes')
    offset = {key: len(result.get(key, [])) for key in keys}
    for view in added.get('bufferViews', []):
        if view.get('buffer', 0) != 0:
            raise ValueError('Unexpected buffer index')
        view['byteOffset'] = view.get('byteOffset', 0) + binary_offset
    for accessor in added.get('accessors', []):
        if 'sparse' in accessor or 'bufferView' not in accessor:
            raise ValueError('Sparse/implicit accessors are unsupported')
        accessor['bufferView'] += offset['bufferViews']
    for image in added.get('images', []):
        if allow_external_images and 'uri' in image and 'bufferView' not in image:
            continue
        if 'uri' in image or 'bufferView' not in image:
            raise ValueError('External images are unsupported')
        image['bufferView'] += offset['bufferViews']
    for texture in added.get('textures', []):
        texture['source'] += offset['images']
        if 'sampler' in texture:
            texture['sampler'] += offset['samplers']
    for material in added.get('materials', []):
        for owner, fields in [(material, ('normalTexture', 'occlusionTexture', 'emissiveTexture')),
                              (material.get('pbrMetallicRoughness', {}), ('baseColorTexture', 'metallicRoughnessTexture'))]:
            for field in fields:
                if field in owner:
                    owner[field]['index'] += offset['textures']
    for mesh in added.get('meshes', []):
        for primitive in mesh['primitives']:
            if primitive.get('targets'):
                raise ValueError('Morph targets are unsupported')
            for key, value in primitive['attributes'].items():
                primitive['attributes'][key] = value + offset['accessors']
            if 'indices' in primitive:
                primitive['indices'] += offset['accessors']
            if 'material' in primitive:
                primitive['material'] += offset['materials']
    for node in added.get('nodes', []):
        if 'skin' in node or 'camera' in node:
            raise ValueError('Nonstatic node')
        if 'mesh' in node:
            node['mesh'] += offset['meshes']
        if 'children' in node:
            node['children'] = [n + offset['nodes'] for n in node['children']]
    for key in keys:
        if key in added:
            result.setdefault(key, []).extend(added[key])
    result['scenes'][0].setdefault('nodes', []).extend(n + offset['nodes'] for n in added['scenes'][0].get('nodes', []))
    result['asset'] = {'version': '2.0', 'generator': 'Theia offline static map assembly'}
    for key in ['extensionsUsed','extensionsRequired']:
        values=sorted(set(first.get(key,[]))|set(second.get(key,[])))
        if values:result[key]=values
    return result


def merge(first, second, output):
    a, binary_a = read(first)
    b, binary_b = read(second)
    doc = merge_documents(a, b, len(binary_a))
    doc['buffers'][0]['byteLength'] = len(binary_a) + len(binary_b)
    text = json.dumps(doc, separators=(',', ':'), allow_nan=False).encode()
    text += b' ' * (-len(text) % 4)
    size = 28 + len(text) + len(binary_a) + len(binary_b)
    with output.open('xb') as file:
        file.write(struct.pack('<4sII', b'glTF', 2, size))
        file.write(struct.pack('<I4s', len(text), b'JSON'))
        file.write(text)
        file.write(struct.pack('<I4s', len(binary_a) + len(binary_b), b'BIN\0'))
        file.write(binary_a)
        file.write(binary_b)
