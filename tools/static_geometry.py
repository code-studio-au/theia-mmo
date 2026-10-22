"""Bounded static glTF graph/geometry operations for offline chunk authoring."""
import copy
import itertools
import numpy as np
import trimesh
from static_material_extensions import validate as validate_extensions

COMPONENTS = {5120: 'i1', 5121: 'u1', 5122: '<i2', 5123: '<u2', 5125: '<u4', 5126: '<f4'}
WIDTHS = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4}
TEXTURE_FIELDS = [('pbrMetallicRoughness', 'baseColorTexture'), ('pbrMetallicRoughness', 'metallicRoughnessTexture'),
                  (None, 'normalTexture'), (None, 'occlusionTexture'), (None, 'emissiveTexture')]


def accessor(document, binary, index):
    item = document['accessors'][index]
    if item.get('sparse') or item['componentType'] not in COMPONENTS or item['type'] not in WIDTHS:
        raise ValueError('Unsupported static accessor layout')
    view = document['bufferViews'][item['bufferView']]
    dtype = np.dtype(COMPONENTS[item['componentType']])
    width = WIDTHS[item['type']]
    count = item['count']
    stride = view.get('byteStride', dtype.itemsize * width)
    offset = item.get('byteOffset', 0)
    if (type(count) is not int or not 1 <= count <= 8_000_000 or view.get('buffer', 0) != 0
            or stride < dtype.itemsize * width or offset < 0
            or offset + (count - 1) * stride + dtype.itemsize * width > view['byteLength']
            or view.get('byteOffset', 0) + view['byteLength'] > len(binary)):
        raise ValueError('Accessor exceeds bounded embedded view')
    result = np.ndarray((count, width), dtype=dtype, buffer=binary,
                        offset=view.get('byteOffset', 0) + offset, strides=(stride, dtype.itemsize))
    if not np.isfinite(result).all():
        raise ValueError('Nonfinite static vertex data')
    return result


def local_matrix(node):
    if 'matrix' in node:
        if any(key in node for key in ('translation', 'rotation', 'scale')):
            raise ValueError('Node mixes matrix and TRS')
        matrix = np.asarray(node['matrix'], dtype=float).reshape((4, 4), order='F')
    else:
        rotation = np.asarray(node.get('rotation', [0., 0., 0., 1.]), dtype=float)
        scale = np.asarray(node.get('scale', [1., 1., 1.]), dtype=float)
        translation = np.asarray(node.get('translation', [0., 0., 0.]), dtype=float)
        if rotation.shape != (4,) or scale.shape != (3,) or translation.shape != (3,) or abs(np.linalg.norm(rotation) - 1.) > 1e-5:
            raise ValueError('Invalid static TRS transform')
        matrix = trimesh.transformations.quaternion_matrix(rotation[[3, 0, 1, 2]])
        matrix[:3, :3] = matrix[:3, :3] @ np.diag(scale)
        matrix[:3, 3] = translation
    if not np.isfinite(matrix).all() or not np.array_equal(matrix[3], [0., 0., 0., 1.]):
        raise ValueError('Nonfinite or projective static transform')
    return matrix


def hierarchy(document):
    validate_extensions(document)
    if any(document.get(key) for key in ('animations', 'skins')):
        raise ValueError('Animated or extended scenery needs a separate partition contract')
    nodes = document.get('nodes', [])
    if not 1 <= len(nodes) <= 65536 or len(document.get('scenes', [])) != 1:
        raise ValueError('Static scene hierarchy outside budget')
    parents, matrices = {}, {}
    pending = [(index, None, np.eye(4), 0) for index in document['scenes'][0].get('nodes', [])]
    while pending:
        index, parent, transform, depth = pending.pop()
        if type(index) is not int or not 0 <= index < len(nodes) or index in parents or depth > 128:
            raise ValueError('Invalid, shared, cyclic or excessive node hierarchy')
        node = nodes[index]
        if 'skin' in node or 'camera' in node:
            raise ValueError('Only static scenery nodes are supported')
        world = transform @ local_matrix(node)
        parents[index], matrices[index] = parent, world
        pending.extend((child, index, world, depth + 1) for child in node.get('children', []))
    return parents, matrices


def transformed_bounds(document, binary, mesh, matrix):
    points = []
    for primitive in document['meshes'][mesh]['primitives']:
        vertices = accessor(document, binary, primitive['attributes']['POSITION'])
        low, high = vertices.min(axis=0), vertices.max(axis=0)
        corners = np.array(list(itertools.product(*zip(low, high))))
        points.append(corners @ matrix[:3, :3].T + matrix[:3, 3])
    points = np.concatenate(points)
    return np.stack([points.min(axis=0), points.max(axis=0)])


def clip_triangle(rows, low, high):
    """Sutherland-Hodgman; all columns interpolate with position columns XYZ."""
    polygon = [np.array(row, dtype=float) for row in rows]
    for axis, edge, sign in [(0, low[0], 1), (0, high[0], -1), (2, low[1], 1), (2, high[1], -1)]:
        if not polygon:
            return []
        clipped = []
        for previous, current in zip(polygon[-1:] + polygon[:-1], polygon):
            a, b = sign * (previous[axis] - edge), sign * (current[axis] - edge)
            if (a >= 0) != (b >= 0):
                point = previous + (current - previous) * (a / (a - b))
                point[axis] = edge
                clipped.append(point)
            if b >= 0:
                clipped.append(current)
        polygon = clipped
    result = []
    for n in range(1, len(polygon) - 1):
        triangle = np.stack([polygon[0], polygon[n], polygon[n + 1]])
        normal = np.cross(triangle[1, :3] - triangle[0, :3], triangle[2, :3] - triangle[0, :3])
        if np.dot(normal, normal) > 1e-12:
            result.append(triangle)
    return result


def prune(document, binary, selected, with_mapping=False):
    """Retain selected mesh nodes, their ancestors and exact transitive assets."""
    parents, _ = hierarchy(document)
    if not selected or any(index not in parents or 'mesh' not in document['nodes'][index] for index in selected):
        raise ValueError('Selected static mesh nodes must exist')
    keep = set(selected)
    for index in selected:
        while parents[index] is not None:
            index = parents[index]
            keep.add(index)
    node_ids = sorted(keep)
    mesh_ids = sorted({document['nodes'][n]['mesh'] for n in selected})
    meshes = [copy.deepcopy(document['meshes'][n]) for n in mesh_ids]
    accessor_ids, material_ids = set(), set()
    for mesh in meshes:
        for primitive in mesh['primitives']:
            if primitive.get('mode', 4) != 4 or primitive.get('targets'):
                raise ValueError('Only static triangle primitives are supported')
            accessor_ids.update(primitive['attributes'].values())
            if 'indices' in primitive:
                accessor_ids.add(primitive['indices'])
            if 'material' in primitive:
                material_ids.add(primitive['material'])
    material_ids = sorted(material_ids)
    materials = [copy.deepcopy(document['materials'][n]) for n in material_ids]
    texture_ids = set()
    for material in materials:
        for owner, field in TEXTURE_FIELDS:
            values = material if owner is None else material.get(owner, {})
            if field in values:
                texture_ids.add(values[field]['index'])
    texture_ids = sorted(texture_ids)
    textures = [copy.deepcopy(document['textures'][n]) for n in texture_ids]
    image_ids = sorted({t['source'] for t in textures})
    sampler_ids = sorted({t['sampler'] for t in textures if 'sampler' in t})
    images = [copy.deepcopy(document['images'][n]) for n in image_ids]
    if any('bufferView' in image or 'uri' not in image for image in images):
        raise ValueError('Partitioning requires bound external shared images')
    accessor_ids = sorted(accessor_ids)
    accessors = [copy.deepcopy(document['accessors'][n]) for n in accessor_ids]
    if any('sparse' in a or 'bufferView' not in a for a in accessors):
        raise ValueError('Sparse/implicit partition accessors are unsupported')
    view_ids = sorted({a['bufferView'] for a in accessors})
    mappings = {key: {old: new for new, old in enumerate(ids)} for key, ids in
                [('node', node_ids), ('mesh', mesh_ids), ('accessor', accessor_ids), ('material', material_ids),
                 ('texture', texture_ids), ('image', image_ids), ('sampler', sampler_ids), ('view', view_ids)]}
    nodes = []
    for n in node_ids:
        node = copy.deepcopy(document['nodes'][n])
        if 'mesh' in node:
            if n in selected:
                node['mesh'] = mappings['mesh'][node['mesh']]
            else:
                del node['mesh']
        if 'children' in node:
            node['children'] = [mappings['node'][v] for v in node['children'] if v in keep]
        nodes.append(node)
    for mesh in meshes:
        for primitive in mesh['primitives']:
            primitive['attributes'] = {key: mappings['accessor'][value] for key, value in primitive['attributes'].items()}
            if 'indices' in primitive:
                primitive['indices'] = mappings['accessor'][primitive['indices']]
            if 'material' in primitive:
                primitive['material'] = mappings['material'][primitive['material']]
    for material in materials:
        for owner, field in TEXTURE_FIELDS:
            values = material if owner is None else material.get(owner, {})
            if field in values:
                values[field]['index'] = mappings['texture'][values[field]['index']]
    for texture in textures:
        texture['source'] = mappings['image'][texture['source']]
        if 'sampler' in texture:
            texture['sampler'] = mappings['sampler'][texture['sampler']]
    packed, views = bytearray(), []
    for n in view_ids:
        view = copy.deepcopy(document['bufferViews'][n])
        if view.get('buffer', 0) != 0 or view.get('byteOffset', 0) < 0 or view['byteLength'] <= 0 or view.get('byteOffset', 0) + view['byteLength'] > len(binary):
            raise ValueError('Invalid retained binary view')
        packed.extend(b'\0' * (-len(packed) % 4))
        start = view.get('byteOffset', 0)
        payload = memoryview(binary)[start:start + view['byteLength']]
        view.update(buffer=0, byteOffset=len(packed))
        packed.extend(payload)
        views.append(view)
    for item in accessors:
        item['bufferView'] = mappings['view'][item['bufferView']]
    result = {'asset': copy.deepcopy(document['asset']), 'scene': 0,
              'scenes': [dict(copy.deepcopy(document['scenes'][0]), nodes=[mappings['node'][n] for n in document['scenes'][0]['nodes'] if n in keep])],
              'nodes': nodes, 'meshes': meshes, 'materials': materials, 'textures': textures, 'images': images,
              'samplers': [copy.deepcopy(document.get('samplers', [])[n]) for n in sampler_ids],
              'accessors': accessors, 'bufferViews': views, 'buffers': [{'byteLength': len(packed)}]}
    retained_extensions = {name for material in materials for name in material.get('extensions', {})}
    for key in ['extensionsUsed', 'extensionsRequired']:
        declarations = [name for name in document.get(key, []) if name in retained_extensions]
        if declarations:
            result[key] = declarations
    validate_extensions(result)
    packed.extend(b'\0' * (-len(packed) % 4))
    return (result, packed, mappings) if with_mapping else (result, packed)
