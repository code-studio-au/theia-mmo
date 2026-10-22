#!/usr/bin/env python3
"""Publish a bounded static GLB collision world/chunk with explicit local spawns.

Visual and collision GLBs are separate authored inputs. No collision inference,
source edits, decimation, external dependencies, animation or skinning is allowed.
Without --spawn, the result is query-only chunk content, not a session world.
"""
import argparse
import copy
import json
from pathlib import Path

import numpy as np
import trimesh
from add_terrain_house import write_glb
from merge_static_glb import read
from package_world import file_hash, package, valid_id, MAX_COLLISION_BYTES, MAX_VISUAL_BYTES


def prepare(visual, collision, output, spawns, content_id, revision, origin=(0., 0., 0.), bounds=None):
    if output.exists():
        raise ValueError('Output directory must be new')
    if not valid_id(content_id) or type(revision) is not int or not 1 <= revision <= 0xffffffff:
        raise ValueError('Invalid package identity/revision')
    origin = np.asarray(origin, dtype=float)
    spawn_points = np.asarray(spawns, dtype=float).reshape((-1, 3))
    if origin.shape != (3,) or not np.isfinite(origin).all() or len(spawn_points) > 8 or not np.isfinite(spawn_points).all() or (abs(spawn_points) > 1023).any():
        raise ValueError('Invalid bounded origin/spawns')
    inputs = {Path(visual).resolve(), Path(collision).resolve()}
    hashes = {str(p): file_hash(p, MAX_VISUAL_BYTES)[1].hex() for p in inputs}
    documents = {p: read(p) for p in inputs}
    for doc, _ in documents.values():
        if len(doc.get('nodes', [])) > 8192:
            raise ValueError('Too many static nodes')
        for mesh in doc.get('meshes', []):
            for primitive in mesh['primitives']:
                if primitive.get('mode', 4) != 4 or primitive.get('targets'):
                    raise ValueError('Only static triangle primitives are supported')
    scene = trimesh.load_scene(collision, process=False)
    triangles = []
    count = 0
    for node in sorted(scene.graph.nodes_geometry):
        matrix, geometry = scene.graph[node]
        mesh = scene.geometry[geometry]
        if not isinstance(mesh, trimesh.Trimesh) or not len(mesh.faces):
            raise ValueError('Collision requires triangle mesh geometry')
        count += len(mesh.faces)
        if count > 131072:
            raise ValueError('Collision triangle budget exceeded')
        points = np.asarray(mesh.vertices) @ matrix[:3, :3].T + matrix[:3, 3] - origin
        triangles.append(np.asarray(points[mesh.faces], dtype=np.float32))
    if not triangles:
        raise ValueError('Empty collision geometry')
    triangles = np.concatenate(triangles)
    normals = np.cross(triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0])
    if not np.isfinite(triangles).all() or (abs(triangles) > 1023).any() or (np.sum(normals * normals, axis=1) <= 1e-12).any():
        raise ValueError('Degenerate or out-of-frame collision geometry')
    xz = triangles.reshape((-1, 3))[:, [0, 2]]
    bounds = np.asarray([xz.min(axis=0), xz.max(axis=0)] if bounds is None else bounds, dtype=float)
    if bounds.shape != (2, 2) or not np.isfinite(bounds).all() or (abs(bounds) > 1023).any() or (bounds[0] >= bounds[1]).any():
        raise ValueError('Invalid explicit/derived collision bounds')
    if len(spawn_points) and ((spawn_points[:, [0, 2]] < bounds[0] + .355).any() or (spawn_points[:, [0, 2]] > bounds[1] - .355).any()):
        raise ValueError('Spawn capsule outside world bounds')
    visual_doc, binary = documents[Path(visual).resolve()]
    visual_doc = copy.deepcopy(visual_doc)
    if np.any(origin):
        root = len(visual_doc['nodes'])
        visual_doc['nodes'].append({'name': 'Theia content frame', 'translation': (-origin).tolist(),
                                    'children': visual_doc['scenes'][0]['nodes']})
        visual_doc['scenes'][0]['nodes'] = [root]
    profile = {'schema': 3, 'bounds': bounds.tolist(), 'source_origin': origin.tolist(),
               'triangles': triangles.tolist(), 'spawns': spawn_points.tolist()}
    # Verify sources before publication. Runtime performs authoritative support/
    # headroom checks; the publisher never substitutes its own movement solver.
    if any(file_hash(Path(p), MAX_VISUAL_BYTES)[1].hex() != digest for p, digest in hashes.items()):
        raise ValueError('Source changed during preparation')
    output.mkdir(parents=True)
    write_glb(visual_doc, binary, output / 'terrain.glb')
    profile['visual_sha256'] = list(file_hash(output / 'terrain.glb', MAX_VISUAL_BYTES)[1])
    data = (json.dumps(profile, separators=(',', ':'), allow_nan=False) + '\n').encode()
    if len(data) > MAX_COLLISION_BYTES:
        raise ValueError('Collision byte budget exceeded')
    (output / 'terrain.json').write_bytes(data)
    result = package(output, content_id, revision)
    result.update(triangles=count, spawns=len(spawn_points), source_hashes=hashes,
                  scope='bounded authored static triangle collision; runtime validates spawn support/headroom')
    (output / 'preparation.json').write_text(json.dumps(result, indent=2) + '\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('visual', type=Path)
    parser.add_argument('collision', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--content-id', required=True)
    parser.add_argument('--revision', type=int, required=True)
    parser.add_argument('--spawn', type=float, nargs=3, action='append', default=[])
    parser.add_argument('--origin', type=float, nargs=3, default=[0., 0., 0.])
    parser.add_argument('--bounds', type=float, nargs=4, metavar=('MIN_X', 'MIN_Z', 'MAX_X', 'MAX_Z'))
    args = parser.parse_args()
    bounds = [args.bounds[:2], args.bounds[2:]] if args.bounds else None
    print(json.dumps(prepare(args.visual, args.collision, args.output, args.spawn,
                             args.content_id, args.revision, args.origin, bounds), indent=2))
