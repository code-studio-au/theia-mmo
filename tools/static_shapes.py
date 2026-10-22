"""Strict schema-4 shape budgets and conservative offline contributor bounds.

Runtime revalidates actual cooked Avian bounds. This module never cooks physics,
executes source scripts or changes schema-1/2/3 profiles.
"""
import math
import numpy as np

FRAME = np.float32(1023)
GEOMETRY = FRAME - np.float32(.001)
PADDING = .001


def keys(value, required, optional=()):
    if not isinstance(value, dict) or not set(required) <= set(value) or set(value) - set(required) - set(optional):
        raise ValueError('Unknown or missing static shape fields')


def array(value, shape, limit):
    if not isinstance(value, list):
        raise ValueError('Static shape numeric arrays must be lists')
    def numbers(v):
        if isinstance(v, list): return all(numbers(x) for x in v)
        return type(v) in (int, float) and math.isfinite(v) and abs(v) <= limit
    if not numbers(value):
        raise ValueError('Invalid static shape numeric values')
    result = np.asarray(value, dtype=np.float32)
    if result.shape != shape:
        raise ValueError('Static shape numeric dimensions differ')
    return result


def layers(value):
    if type(value) is not int or not 1 <= value <= 7:
        raise ValueError('Invalid static query layers')
    return value


def rotation(q):
    q = array(q, (4,), 2)
    norm = np.dot(q, q)
    if abs(float(norm) - 1) > .0001:
        raise ValueError('Invalid static box rotation')
    x, y, z, w = q / np.sqrt(norm)
    return np.array([[1-2*(y*y+z*z), 2*(x*y-z*w), 2*(x*z+y*w)],
                     [2*(x*y+z*w), 1-2*(x*x+z*z), 2*(y*z-x*w)],
                     [2*(x*z-y*w), 2*(y*z+x*w), 1-2*(x*x+y*y)]], dtype=np.float32)


def validate(data, legacy_triangles=0, legacy_bodies=0):
    keys(data, (), ('meshes', 'objects', 'heightfields'))
    meshes, objects, fields = (data.get(k, []) for k in ['meshes', 'objects', 'heightfields'])
    if type(legacy_triangles) is not int or not 0 <= legacy_triangles <= 131072 or type(legacy_bodies) is not int or legacy_bodies < 0 or any(not isinstance(v, list) for v in [meshes, objects, fields]) or len(meshes) > 128 or len(fields) > 8 or len(objects) + len(fields) > 4096 or legacy_bodies + len(objects) + len(fields) > 8188:
        raise ValueError('Static shape body/pool budget exceeded')
    parsed = []
    triangles, vertices = legacy_triangles, 0
    for mesh in meshes:
        keys(mesh, ('vertices', 'indices'))
        if not isinstance(mesh['vertices'], list) or not isinstance(mesh['indices'], list):
            raise ValueError('Invalid shared collision mesh arrays')
        points = array(mesh['vertices'], (len(mesh['vertices']), 3), 4096)
        indices = mesh['indices']
        triangles += len(indices)
        vertices += len(points)
        if len(points) < 3 or not indices or triangles > 131072 or vertices > 262144 or any(not isinstance(t, list) or len(t) != 3 or any(type(i) is not int or not 0 <= i < len(points) for i in t) for t in indices):
            raise ValueError('Invalid or over-budget shared collision mesh')
        indices = np.asarray(indices, dtype=np.uint32)
        t = points[indices]
        n = np.cross(t[:, 1] - t[:, 0], t[:, 2] - t[:, 0])
        if not (np.einsum('ij,ij->i', n, n) > 1e-20).any():
            raise ValueError('Shared mesh has no nondegenerate surface')
        parsed.append(points)
    used = set()
    expanded_triangles = expanded_vertices = 0
    bounds = []
    world_bounds = []

    def append(low, high, queries):
        box = np.asarray([low, high], dtype=np.float32)
        if not np.isfinite(box).all() or (abs(box) > GEOMETRY).any():
            raise ValueError('Static shape exceeds local physics frame')
        # Runtime SIMD/shape bounds can round differently; metadata is
        # deliberately conservative and runtime checks actual cooked AABBs.
        bounds.append((box.astype(float) + np.array([[-PADDING]*3, [PADDING]*3])).tolist())
        if queries & 3: world_bounds.append(box)

    for body in objects:
        if not isinstance(body, dict):
            raise ValueError('Invalid static object')
        kind = body.get('kind')
        queries = layers(body.get('queries', 7))
        if kind == 'mesh':
            keys(body, ('kind', 'mesh', 'affine', 'convex'), ('queries',))
            i = body['mesh']
            if type(i) is not int or not 0 <= i < len(meshes) or type(body['convex']) is not bool:
                raise ValueError('Unbound shared collision mesh/type')
            if body['convex'] and (len(parsed[i]) < 4 or np.linalg.matrix_rank(parsed[i].astype(float) - parsed[i][0]) < 3):
                raise ValueError('Static convex hull has no volume')
            used.add(i)
            expanded_triangles += len(meshes[i]['indices'])
            expanded_vertices += len(parsed[i])
            if expanded_triangles > 262144 or expanded_vertices > 1048576:
                raise ValueError('Instantiated collision geometry exceeds budget')
            matrix = array(body['affine'], (16,), 4096).reshape(4, 4)
            if not np.array_equal(matrix[3], [0,0,0,1]) or abs(float(np.linalg.det(matrix.astype(float)))) <= 1e-12:
                raise ValueError('Invalid/singular static affine transform')
            points = parsed[i]
            center = matrix[:3, 3]
            transformed = points @ matrix[:3, :3].T + center
            append(transformed.min(0), transformed.max(0), queries)
        elif kind == 'box':
            keys(body, ('kind', 'center', 'size', 'rotation'), ('queries',))
            center = array(body['center'], (3,), FRAME)
            size = array(body['size'], (3,), 1024)
            if (size <= 0).any(): raise ValueError('Invalid static box size')
            extent = abs(rotation(body['rotation'])) @ (size / np.float32(2))
            append(center - extent, center + extent, queries)
        elif kind == 'capsule':
            keys(body, ('kind', 'center', 'radius', 'half_segment'), ('queries',))
            center = array(body['center'], (3,), FRAME)
            half = array(body['half_segment'], (3,), FRAME)
            radius = body['radius']
            if type(radius) not in (int, float) or not math.isfinite(radius) or not 0 < radius <= FRAME:
                raise ValueError('Invalid static capsule radius')
            extent = abs(half) + np.float32(radius)
            append(center - extent, center + extent, queries)
        else:
            raise ValueError('Unknown static object kind')
    if used != set(range(len(meshes))):
        raise ValueError('Unused shared collision geometry')
    samples = 0
    for field in fields:
        keys(field, ('rows', 'columns', 'heights_column_major', 'center', 'scale'), ('queries',))
        rows, columns = field['rows'], field['columns']
        if any(type(v) is not int or not 2 <= v <= 1025 for v in [rows, columns]):
            raise ValueError('Invalid heightfield dimensions')
        heights = array(field['heights_column_major'], (rows * columns,), 4096)
        samples += len(heights)
        if samples > 263169: raise ValueError('Heightfield sample budget exceeded')
        center = array(field['center'], (3,), FRAME)
        scale = array(field['scale'], (3,), FRAME)
        if (scale <= 0).any(): raise ValueError('Invalid heightfield scale')
        low, high = center - scale / np.float32(2), center + scale / np.float32(2)
        low[1] = heights.min() * scale[1] + center[1]
        high[1] = heights.max() * scale[1] + center[1]
        append(low, high, layers(field.get('queries', 7)))
    return {'bounds': bounds, 'world_geometry': bool(world_bounds),
            'pool_triangles': triangles - legacy_triangles, 'pool_vertices': vertices,
            'instantiated_triangles': expanded_triangles, 'instantiated_vertices': expanded_vertices,
            'height_samples': samples, 'bodies': len(objects) + len(fields)}
