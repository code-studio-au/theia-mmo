"""Independent float64 source-surface oracle for an actual collision fixture.

Uses exhaustive triangle intersections, not Avian/Parry or PhysX. Convex inputs
must first prove closed convex surfaces; boxes/capsules retain actual Editor
probes. This is separate from the strict legacy Physics query comparison.
"""
import argparse
import collections
import copy
import json
from pathlib import Path
import numpy as np
from unity_editor_reference import digest, read_json


def convex_surface(vertices, faces):
    vertices = np.asarray(vertices, dtype=float)
    faces = np.asarray(faces, dtype=np.int64)
    if vertices.ndim != 2 or vertices.shape[1] != 3 or faces.ndim != 2 or faces.shape[1] != 3 or not len(faces) or not np.isfinite(vertices).all() or faces.min() < 0 or faces.max() >= len(vertices):
        raise ValueError('Invalid convex source geometry')
    edges = collections.Counter(tuple(sorted((int(a), int(b)))) for face in faces for a, b in zip(face, np.roll(face, -1)))
    if set(edges.values()) != {2} or len(np.unique(faces)) != len(vertices) or len(vertices) - len(edges) + len(faces) != 2 or np.linalg.matrix_rank(vertices - vertices[0]) != 3:
        raise ValueError('Source convex mesh is not a closed volume')
    triangles = vertices[faces]
    normals = np.cross(triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0])
    length = np.linalg.norm(normals, axis=1)
    if (length < 1e-12).any():
        raise ValueError('Degenerate convex source faces')
    normals /= length[:, None]
    d = -np.einsum('ij,ij->i', normals, triangles[:, 0])
    flip = normals @ vertices.mean(0) + d > 0
    normals[flip] *= -1
    d[flip] *= -1
    if (vertices @ normals.T + d).max() > 1e-7:
        raise ValueError('A declared source convex surface is concave')
    return True


def intersect(triangles, probe):
    triangles = np.asarray(triangles, dtype=float)
    origin = np.asarray(probe['origin'], dtype=float)
    direction = np.asarray(probe['direction'], dtype=float)
    e1 = triangles[:, 1] - triangles[:, 0]
    e2 = triangles[:, 2] - triangles[:, 0]
    h = np.cross(direction, e2)
    determinant = np.einsum('ij,ij->i', e1, h)
    safe = np.where(abs(determinant) > 1e-12, determinant, 1)
    offset = origin - triangles[:, 0]
    u = np.einsum('ij,ij->i', offset, h) / safe
    q = np.cross(offset, e1)
    v = q @ direction / safe
    distance = np.einsum('ij,ij->i', e2, q) / safe
    valid = (abs(determinant) > 1e-12) & (u >= -1e-8) & (v >= -1e-8) & (u + v <= 1 + 1e-8) & (distance >= 0) & (distance <= probe['maximum'])
    result = copy.deepcopy(probe)
    result['normal_candidates'] = []
    if not valid.any():
        result.update(hit=False, distance=0, point=[0, 0, 0], normal=[0, 0, 0])
        return result
    nearest = float(distance[valid].min())
    candidates = valid & (distance <= nearest + .002)
    normals = np.cross(e1[candidates], e2[candidates])
    normals /= np.linalg.norm(normals, axis=1)[:, None]
    normals[normals @ direction > 0] *= -1
    unique = np.unique(np.round(normals, 12), axis=0)
    if len(unique) > 64:
        raise ValueError('Ambiguous source surface exceeds bounded normal candidates')
    result.update(hit=True, distance=nearest, point=(origin + direction * nearest).tolist(),
                  normal=unique[0].tolist(), normal_candidates=unique.tolist())
    return result


def qualify(fixture, output):
    fixture = Path(fixture).resolve(strict=True)
    output = Path(output).resolve()
    build = Path(__file__).resolve().parents[1] / 'build'
    if output.exists() or output.with_suffix('.receipt.json').exists() or output == build or not output.is_relative_to(build):
        raise ValueError('Use a fresh ignored source-geometry oracle output')
    receipt = read_json(fixture.with_suffix('.receipt.json'), 1024 * 1024)
    if receipt['fixture_sha256'] != digest(fixture):
        raise ValueError('Source comparison fixture binding differs')
    document = read_json(fixture, 128 * 1024 * 1024)
    if document['schema'] != 1 or document['profile'] != 'unity-object-collision-comparison-v1':
        raise ValueError('Unknown actual source object comparison fixture')
    convex = {o['mesh'] for o in document['colliders'] if o['convex']}
    for index in convex:
        mesh = document['meshes'][index]
        convex_surface(mesh['vertices'], mesh['indices'])
    objects = []
    for body in document['colliders']:
        probes = copy.deepcopy(body['probes'])
        if body['kind'] == 'MeshCollider':
            geometry = document['meshes'][body['mesh']]
            matrix = np.array(body['matrix']).reshape(4, 4)
            vertices = np.array(geometry['vertices']) @ matrix[:3, :3].T + matrix[:3, 3]
            triangles = vertices[np.array(geometry['indices'])]
            probes = [intersect(triangles, p) for p in probes]
        objects.append({'id': body['id'], 'probes': probes})
    result = {'schema': 1, 'profile': 'independent-source-collision-geometry-v1',
              'fixture_sha256': digest(fixture), 'objects': objects}
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('x') as stream:
        stream.write(json.dumps(result, separators=(',', ':'), allow_nan=False) + '\n')
    proof = {'oracle_sha256': digest(output), 'fixture_sha256': digest(fixture),
             'objects': len(objects), 'convex_source_meshes_proven_closed_and_convex': len(convex),
             'scope': 'Float64 exhaustive two-sided source triangles; closed convex geometry proof; native primitive probes retained; independent of Avian and PhysX cooking'}
    output.with_suffix('.receipt.json').write_text(json.dumps(proof, indent=2) + '\n')
    return proof


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('fixture', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    print(json.dumps(qualify(args.fixture, args.output)))
