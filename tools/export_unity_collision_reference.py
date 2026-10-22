"""Export a bounded object-collision comparison fixture from verified Editor data.

This is a native Avian qualification input, not a runtime content package. All
source/reference checks run first. Terrain retains its separate full-grid gate.
"""
import argparse
import json
from pathlib import Path
import numpy as np
from unity_editor_reference import inspect as inspect_models, digest
from unity_serialized_mesh_reference import inspect as inspect_serialized
from unity_builtin_plane_reference import inspect as inspect_plane
from verify_unity_scene_collision import verify


def export(source, scene, models, serialized, plane, probe, output):
    output = Path(output).resolve()
    build = Path(__file__).resolve().parents[1] / 'build'
    if output.exists() or output == build or not output.is_relative_to(build) or not 1 <= len(models) <= 8 or not 1 <= len(serialized) <= 4:
        raise ValueError('Use a fresh ignored fixture and bounded reference lists')
    checked = verify(source, scene)
    references = []
    geometry = {}

    def insert(guid, mesh):
        key = guid, mesh['file_id']
        points = np.array(mesh['positions'], dtype=np.float32).reshape(-1, 3)
        faces = np.concatenate([np.array(s['indices'], dtype=np.uint32).reshape(-1, 3) for s in mesh['submeshes']])
        points[:, 0] *= -1
        faces = faces[:, [0, 2, 1]]
        result = {'vertices': points.tolist(), 'indices': faces.tolist()}
        if key in geometry and geometry[key] != result:
            raise ValueError('Duplicate actual Editor geometry differs')
        geometry[key] = result

    for project in models:
        report = inspect_models(source, project, probe)
        if report['recorded_frame_matches'] != report['meshes_checked'] or not report['metadata_byte_identical']:
            raise ValueError('Qualify actual model frame geometry and unchanged metadata first')
        references.append({'project': str(Path(project).resolve()), 'report_sha256': report['reference_sha256']})
        data = json.loads((Path(project) / 'Reference/imported.json').read_text())
        for model in data['models']:
            for mesh in model['meshes']:
                insert(model['guid'], mesh)
    for project in serialized:
        report = inspect_serialized(source, project)
        references.append({'project': str(Path(project).resolve()), 'report_sha256': report['report_sha256']})
        data = json.loads((Path(project) / 'Reference/imported.json').read_text())
        for model in data['models']:
            for mesh in model['meshes']:
                insert(model['guid'], mesh)
    report = inspect_plane(plane)
    references.append({'project': str(Path(plane).resolve()), 'report_sha256': report['report_sha256']})
    data = json.loads((Path(plane) / 'Reference/plane.json').read_text())
    insert(data['guid'], {'file_id': data['file_id'], 'positions': data['positions'],
                         'submeshes': [{'indices': data['indices']}]})
    actual = json.loads((Path(scene) / 'Reference/imported.json').read_text())
    meshes = []
    mesh_indices = {}
    colliders = []
    excluded = []
    reflection = np.diag([-1., 1., 1., 1.])

    def vector(value):
        return [-value['x'], value['y'], value['z']]

    for shape in actual['colliders']:
        if not shape['active'] or not shape['enabled']:
            excluded.append({'id': shape['global_id'], 'reason': 'inactive or disabled source component'})
            continue
        if shape['kind'] == 'TerrainCollider':
            excluded.append({'id': shape['global_id'], 'reason': 'separate full-grid terrain qualification'})
            continue
        if shape['trigger'] or shape['attached_rigidbody']:
            raise ValueError('The object fixture accepts only static nontrigger source colliders')
        matrix = np.array(shape['world_matrix']).reshape(4, 4)
        mesh = None
        if shape['kind'] == 'MeshCollider':
            key = shape['mesh']['guid'], shape['mesh']['file_id']
            if key not in mesh_indices:
                if key not in geometry:
                    raise ValueError('Missing independently verified actual collision geometry')
                mesh_indices[key] = len(meshes)
                meshes.append(geometry[key])
            mesh = mesh_indices[key]
        colliders.append({'id': shape['global_id'], 'kind': shape['kind'],
                          'matrix': (reflection @ matrix @ reflection).reshape(-1).tolist(),
                          'mesh': mesh, 'convex': shape['convex'],
                          'center': vector(shape['center']),
                          'size': [shape['size'][a] for a in 'xyz'],
                          'radius': shape['radius'], 'height': shape['height'],
                          'direction': shape['capsule_direction'],
                          'lossy_scale': [shape['lossy_scale'][a] for a in 'xyz'],
                          'bounds_center': vector(shape['bounds_center']),
                          'bounds_size': [shape['bounds_size'][a] for a in 'xyz'],
                          'probes': [{'origin': vector(p['origin']), 'direction': vector(p['direction']),
                                      'maximum': p['maximum'], 'hit': p['hit'],
                                      'distance': p['distance'], 'point': vector(p['point']),
                                      'normal': vector(p['normal'])} for p in shape['probes']]})
    result = {'schema': 1, 'profile': 'unity-object-collision-comparison-v1',
              'meshes': meshes, 'colliders': colliders}
    payload = json.dumps(result, separators=(',', ':'), allow_nan=False) + '\n'
    if not 1 <= len(colliders) <= 8192 or len(payload.encode()) > 128 * 1024 * 1024:
        raise ValueError('Object collision fixture exceeds byte/body budget')
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open('x') as stream:
        stream.write(payload)
    receipt = {'schema': 1, 'fixture_sha256': digest(output), 'scene_verification': checked,
               'references': references, 'excluded': excluded,
               'mesh_count': len(meshes), 'object_colliders': len(colliders),
               'scope': 'Actual source object comparison fixture; RH mirrored-X geometry with reversed triangles; no runtime content admission, collider cooking or Avian parity claim'}
    receipt_path = output.with_suffix('.receipt.json')
    if receipt_path.exists():
        output.unlink()
        raise FileExistsError('Use a fresh receipt')
    receipt_path.write_text(json.dumps(receipt, indent=2) + '\n')
    return {k: receipt[k] for k in ['fixture_sha256', 'mesh_count', 'object_colliders', 'scope']}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path)
    parser.add_argument('scene', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--model-reference', action='append', type=Path, required=True)
    parser.add_argument('--serialized-reference', action='append', type=Path, required=True)
    parser.add_argument('--plane-reference', type=Path, required=True)
    parser.add_argument('--probe', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(export(args.source, args.scene, args.model_reference,
                            args.serialized_reference, args.plane_reference, args.probe, args.output)))
