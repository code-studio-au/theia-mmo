"""Bind every qualified terrain vertex/triangle to a standard heightfield test.

The original GLB stays untouched. Sorted X/Z coordinates determine explicit
column-major heights; complete oriented topology must match the default diagonal.
"""
import argparse
import json
from pathlib import Path
import numpy as np
from merge_static_glb import read
from static_geometry import accessor, hierarchy
from unity_editor_reference import digest
from verify_terrain_packages import oriented_rows

BUILD = Path(__file__).resolve().parents[1] / 'build'

def prepare(source, output):
    source = Path(source).resolve(strict=True)
    output = Path(output).resolve()
    build = BUILD.resolve()
    if output.exists() or output.with_suffix('.receipt.json').exists() or output == build or not output.is_relative_to(build):
        raise ValueError('Use a fresh ignored heightfield reference')
    conversion = json.loads((source / 'conversion.json').read_text())
    path = source / 'terrain.glb'
    source_hash = digest(path)
    if conversion['outputs']['terrain.glb'] != source_hash:
        raise ValueError('Original terrain binding differs')
    document, binary = read(path)
    _, frames = hierarchy(document)
    if len(document['nodes']) != 1 or len(document['meshes']) != 1 or not np.array_equal(frames[0], np.eye(4)):
        raise ValueError('Require the canonical single identity terrain')
    primitive, = document['meshes'][0]['primitives']
    vertices = accessor(document, binary, primitive['attributes']['POSITION']).astype(np.float32)
    faces = accessor(document, binary, primitive['indices']).reshape(-1, 3).astype(np.int64)
    if not np.isfinite(vertices).all() or faces.min() < 0 or faces.max() >= len(vertices):
        raise ValueError('Invalid terrain source geometry')
    xs, zs = np.unique(vertices[:, 0]), np.unique(vertices[:, 2])
    if not 2 <= len(xs) <= 1025 or not 2 <= len(zs) <= 1025 or len(vertices) != len(xs) * len(zs):
        raise ValueError('Terrain source is not a bounded complete grid')
    if not np.array_equal(np.diff(xs), np.full(len(xs) - 1, xs[1] - xs[0])) or not np.array_equal(np.diff(zs), np.full(len(zs) - 1, zs[1] - zs[0])):
        raise ValueError('Terrain samples are not exactly regularly spaced')
    lookup = np.full((len(zs), len(xs)), -1, dtype=np.int64)
    coordinates = np.column_stack([np.searchsorted(zs, vertices[:, 2]), np.searchsorted(xs, vertices[:, 0])])
    if len(np.unique(coordinates, axis=0)) != len(vertices):
        raise ValueError('Duplicate terrain XZ coordinates')
    lookup[coordinates[:, 0], coordinates[:, 1]] = np.arange(len(vertices))
    p00, p10, p01, p11 = lookup[:-1, :-1], lookup[1:, :-1], lookup[:-1, 1:], lookup[1:, 1:]
    expected = np.concatenate([np.stack([p00, p10, p01], -1).reshape(-1, 3),
                               np.stack([p10, p11, p01], -1).reshape(-1, 3)])
    if not np.array_equal(oriented_rows(faces), oriented_rows(expected)):
        raise ValueError('Source terrain has a different diagonal, holes or oriented coverage')
    heights = vertices[lookup, 1]
    centers = vertices[faces].astype(float).mean(1)
    result = {'schema': 1, 'profile': 'source-full-grid-heightfield-reference-v1',
              'source_sha256': source_hash, 'rows': len(zs), 'columns': len(xs),
              'scale': [float(xs[-1] - xs[0]), 1., float(zs[-1] - zs[0])],
              'center': [float((xs[-1] + xs[0]) / 2), 0., float((zs[-1] + zs[0]) / 2)],
              'heights_column_major': heights.T.reshape(-1).tolist(),
              'vertices': vertices.tolist(), 'indices': faces.tolist(),
              'triangle_centers': centers.tolist()}
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('x') as stream:
        stream.write(json.dumps(result, separators=(',', ':'), allow_nan=False) + '\n')
    receipt = {'fixture_sha256': digest(output), 'source_sha256': source_hash,
               'vertices': len(vertices), 'triangles': len(faces),
               'scope': 'All source samples and complete oriented default-diagonal topology; triangle-centroid expectations from original GLB; no Avian or runtime replacement claim'}
    output.with_suffix('.receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    return receipt


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(prepare(args.source, args.output)))
