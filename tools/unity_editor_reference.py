"""Stage a script-free FBX reference project and inspect actual Editor mesh data.

Only our reviewed collector runs. Source scripts, scenes, materials and packages
are not imported. This is mesh/root-pivot evidence, not a scene/material exporter.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

import numpy as np
from unity_model_ids import identity, bound_candidates
from unity_scene import Sources
from verify_unity_geometry import point_error


COLLECTOR = Path(__file__).resolve().parent / 'qualification/TheiaMeshReference.cs'
BUILD = Path(__file__).resolve().parents[1] / 'build'
VERSION = re.compile(r'(?:20\d\d|[6-9]\d{3})\.\d+\.\d+[abfp]\d+')
FILE_ID = re.compile(r'-?(?:0|[1-9][0-9]*)')


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def read_json(path, limit):
    if not 1 <= Path(path).stat().st_size <= limit:
        raise ValueError('Reference JSON exceeds byte budget')
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('Duplicate reference JSON key')
            result[key] = value
        return result
    return json.loads(Path(path).read_text(), object_pairs_hook=unique,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError('Nonfinite JSON')))


def stage(source, output, models, version):
    output = Path(output).resolve()
    if not output.is_relative_to(BUILD.resolve()) or output == BUILD.resolve():
        raise ValueError('Keep licensed reference projects inside ignored build/')
    if output.exists():
        raise FileExistsError('Use a fresh reference project')
    if not VERSION.fullmatch(version) or not 1 <= len(models) <= 32 or len(set(models)) != len(models):
        raise ValueError('Select an exact Editor version and 1..32 unique FBXs')
    sources = Sources(source)
    inputs = []
    for name in models:
        if not name.lower().endswith('.fbx'):
            raise ValueError('Reference staging accepts only FBX models')
        model = sources.checked(name)
        sources.metadata(name)
        identity(model)  # Reject LFS pointers/malformed/nonbinary models before staging.
        binding = sources.manifest['selected'][name]
        inputs.append({'path': name, 'guid': binding['guid'], 'sha256': binding['sha256'],
                       'meta_sha256': sources.manifest['metadata'][name]['sha256']})
    output.mkdir(parents=True)
    (output / 'Assets/Editor').mkdir(parents=True)
    (output / 'ProjectSettings').mkdir()
    (output / 'Packages').mkdir()
    (output / 'Reference/original-meta').mkdir(parents=True)
    (output / 'ProjectSettings/ProjectVersion.txt').write_text('m_EditorVersion: ' + version + '\n')
    (output / 'Packages/manifest.json').write_text('{"dependencies": {}}\n')
    shutil.copyfile(COLLECTOR, output / 'Assets/Editor/TheiaMeshReference.cs')
    for item in inputs:
        target = output / item['path']
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(sources.checked(item['path']), target)
        shutil.copyfile(sources.checked(item['path'], True), str(target) + '.meta')
        shutil.copyfile(sources.checked(item['path'], True), output / 'Reference/original-meta' / (item['guid'] + '.meta'))
    request = {'schema': 1, 'archive_sha256': sources.manifest['archive_sha256'], 'unity_version': version,
               'collector_sha256': digest(COLLECTOR), 'models': inputs}
    (output / 'Reference/request.json').write_text(json.dumps(request, indent=2) + '\n')
    return {'project': str(output), 'unity_version': version, 'models': len(inputs),
            'request_sha256': digest(output / 'Reference/request.json'),
            'collector_sha256': request['collector_sha256'], 'editor_executed': False}


def checked_id(value):
    if not isinstance(value, str) or not FILE_ID.fullmatch(value) or not -(1 << 63) <= int(value) < (1 << 63):
        raise ValueError('Expected signed 64-bit persistent file ID string')
    return int(value)


def matrix(values):
    result = np.asarray(values, dtype=np.float64)
    if result.shape != (16,) or not np.isfinite(result).all():
        raise ValueError('Expected finite row-major matrix')
    result = result.reshape(4, 4)
    if not np.allclose(result[3], [0, 0, 0, 1], atol=1e-7, rtol=0) or abs(np.linalg.det(result[:3, :3])) < 1e-12:
        raise ValueError('Expected invertible affine model transform')
    return result


def mesh_points(mesh):
    checked_id(mesh['file_id'])
    points = np.asarray(mesh['positions'], dtype=np.float64)
    if points.ndim != 1 or not 3 <= points.size <= 6000000 or points.size % 3 or not np.isfinite(points).all():
        raise ValueError('Invalid mesh position budget/values')
    points = points.reshape(-1, 3)
    if not 1 <= len(mesh['submeshes']) <= 256:
        raise ValueError('Invalid submesh budget')
    indices = []
    for sub in mesh['submeshes']:
        ii = sub['indices']
        if sub['topology'] != 0 or not isinstance(ii, list) or not ii or len(ii) % 3 or any(type(i) is not int for i in ii):
            raise ValueError('Expected triangle indices')
        indices.extend(ii)
        if len(indices) > 8000000:
            raise ValueError('Index budget exceeded')
    if min(indices) < 0 or max(indices) >= len(points):
        raise ValueError('Out-of-range reference mesh index')
    center = np.array([mesh['bounds_center'][axis] for axis in 'xyz'], dtype=float)
    size = np.array([mesh['bounds_size'][axis] for axis in 'xyz'], dtype=float)
    if not np.isfinite([center, size]).all() or (size < 0).any():
        raise ValueError('Invalid reference mesh bounds')
    # This checks the reported bounds, not an assumption that they are tight.
    if (points < center - size * .5 - .0001).any() or (points > center + size * .5 + .0001).any():
        raise ValueError('Reference mesh vertices escape reported bounds')
    return points[np.unique(indices)]


def validate_model(model):
    if not 1 <= len(model['meshes']) <= 256 or not 1 <= len(model['nodes']) <= 65536:
        raise ValueError('Invalid model reference budget')
    meshes = {}
    for mesh in model['meshes']:
        identifier = checked_id(mesh['file_id'])
        if identifier in meshes:
            raise ValueError('Duplicate imported mesh ID')
        meshes[identifier] = mesh_points(mesh)
    nodes = {}
    for node in model['nodes']:
        identifier = checked_id(node['file_id'])
        if identifier in nodes:
            raise ValueError('Duplicate imported transform ID')
        nodes[identifier] = node
        if node['mesh_file_id'] and checked_id(node['mesh_file_id']) not in meshes:
            raise ValueError('Unbound hierarchy mesh ID')
    roots = [n for n in nodes.values() if n['parent_id'] == '']
    if len(roots) != 1:
        raise ValueError('Expected exactly one imported model root')
    pending = set(nodes)
    complete = {}
    while pending:
        ready = [i for i in pending if not nodes[i]['parent_id'] or checked_id(nodes[i]['parent_id']) in complete]
        if not ready:
            raise ValueError('Cyclic or missing imported parent')
        for i in ready:
            node = nodes[i]
            local = matrix(node['local_matrix'])
            wanted = complete[checked_id(node['parent_id'])] @ local if node['parent_id'] else local
            actual = matrix(node['world_matrix'])
            if not np.allclose(actual, wanted, atol=.0001, rtol=1e-6):
                raise ValueError('Inconsistent imported hierarchy matrix')
            complete[i] = actual
            pending.remove(i)
    return meshes


def policies(node, actual, unity_frame=None):
    """Diagnostic alternatives only; no policy is selected/applied to exports."""
    points = np.asarray(node['vertices'], dtype=np.float64)
    used = np.unique([i for face in node['faces'] if len(face) >= 3 for i in face])
    points = points[used]
    geometry = np.array(node['geometry_to_node'], dtype=float)
    world = np.array(node['geometry_to_world'], dtype=float)
    frame = np.array(node['node_to_world'], dtype=float)
    local = points @ geometry[:3, :3].T + geometry[:3, 3]
    candidates = {
        'mesh_local': local * [-1, 1, 1],
        'geometry_to_world': (points @ world[:3, :3].T + world[:3, 3]) * [-1, 1, 1],
        'root_translation_in_local_frame': (local + np.linalg.solve(frame[:3, :3], frame[:3, 3])) * [-1, 1, 1]
    }
    if unity_frame is not None:
        # Compare both importers in the same model-world frame, then use the
        # actual persistent Unity transform to recover its mesh-local frame.
        inverse = np.linalg.inv(unity_frame)
        reflected_world = (points @ world[:3, :3].T + world[:3, 3]) * [-1, 1, 1]
        candidates['recorded_unity_frame'] = reflected_world @ inverse[:3, :3].T + inverse[:3, 3]
    result = {}
    for name, values in candidates.items():
        try:
            error = point_error(values, actual, .0001)
            result[name] = {'matches': True, 'max_point_error_metres': error}
        except ValueError:
            result[name] = {'matches': False, 'max_point_error_metres': None}
    return result


def inspect(source, project, probe, selected_models=None):
    sources = Sources(source)
    project = Path(project).resolve(strict=True)
    request_path = project / 'Reference/request.json'
    request = read_json(request_path, 65536)
    report_path = project / 'Reference/imported.json'
    report = read_json(report_path, 512 * 1024 * 1024)
    if request['schema'] != 1 or report['schema'] != 1 or request['archive_sha256'] != sources.manifest['archive_sha256']:
        raise ValueError('Reference schema/archive differs')
    if report['request_sha256'] != digest(request_path) or report['collector_sha256'] != request['collector_sha256'] or digest(project / 'Assets/Editor/TheiaMeshReference.cs') != request['collector_sha256'] or request['collector_sha256'] != digest(COLLECTOR):
        raise ValueError('Reference request/collector binding differs')
    if not VERSION.fullmatch(report['unity_version']) or report['platform'] not in ('LinuxEditor', 'WindowsEditor', 'OSXEditor'):
        raise ValueError('Missing actual Editor version/platform')
    if report['unity_version'] != request['unity_version']:
        raise ValueError('Actual Editor version differs from staged request')
    if not 1 <= len(request['models']) <= 32 or len(report['models']) != len(request['models']):
        raise ValueError('Reference models incomplete')
    requested = {v['path']: v for v in request['models']}
    imported = {v['path']: v for v in report['models']}
    if len(requested) != len(request['models']) or len(imported) != len(report['models']) or set(requested) != set(imported):
        raise ValueError('Missing/duplicate/extra reference model')
    if selected_models is not None and (not selected_models or not set(selected_models) <= set(requested)):
        raise ValueError('Selected reference models are missing')
    results = []
    for path, item in requested.items():
        if selected_models is not None and path not in selected_models:
            continue
        model_path = sources.checked(path)
        importer = sources.metadata(path)['ModelImporter']
        raw = sources.manifest['selected'][path]
        if any(item[k] != raw[k] for k in ('guid', 'sha256')) or item['meta_sha256'] != sources.manifest['metadata'][path]['sha256']:
            raise ValueError('Reference request differs from original source')
        if digest(project / 'Reference/original-meta' / (item['guid'] + '.meta')) != item['meta_sha256']:
            raise ValueError('Original metadata snapshot differs')
        model = imported[path]
        if any(model[k] != item[k] for k in ('guid', 'sha256')) or digest(project / path) != item['sha256']:
            raise ValueError('Imported model source binding differs')
        current_meta = digest(project / (path + '.meta'))
        if model['meta_sha256'] != current_meta:
            raise ValueError('Imported metadata changed after collection')
        actual = validate_model(model)
        binding, _ = bound_candidates(identity(model_path), importer)
        expected = {key: value for key, value in binding.items() if value['kind'] == 'Mesh'}
        if set(expected) != set(actual):
            raise ValueError('Actual Editor mesh IDs differ from source-derived IDs')
        data = json.loads(subprocess.check_output([str(Path(probe).resolve(strict=True)), str(model_path), 'right'], timeout=30))
        if data['ufbx_version'] != 23001 or data['source_axes'] != [0, 2, 4]:
            raise ValueError('Unqualified independent FBX basis')
        checks = []
        for identifier, mesh in expected.items():
            node, = [v for v in data['nodes'] if v['name'] == mesh['name']]
            if node['deformed']:
                raise ValueError('Deformed reference mesh needs separate qualification')
            transform_id, = [key for key, entry in binding.items() if entry['kind'] == 'Transform' and entry['model'] == mesh['model']]
            imported_node, = [v for v in model['nodes'] if checked_id(v['file_id']) == transform_id and v['mesh_file_id'] == str(identifier)]
            unity_frame = matrix(imported_node['world_matrix'])
            correction = np.linalg.inv(unity_frame) @ np.diag([-1., 1., 1., 1.]) @ np.array(node['node_to_world'])
            checks.append({'file_id': str(identifier), 'name': mesh['name'],
                           'transform_file_id': str(transform_id),
                           'canonical_mesh_to_unity_local': correction.tolist(),
                           'policies': policies(node, actual[identifier], unity_frame),
                           'unity_bounds': [actual[identifier].min(0).tolist(), actual[identifier].max(0).tolist()]})
        results.append({'path': path, 'metadata_byte_identical': current_meta == item['meta_sha256'],
                        'meshes': checks, 'imported_nodes': model['nodes']})
    all_checks = [mesh for model in results for mesh in model['meshes']]
    matched = [mesh['policies']['recorded_unity_frame'] for mesh in all_checks if mesh['policies']['recorded_unity_frame']['matches']]
    return {'schema': 1, 'scope': 'actual Editor mesh-local point sets/root transforms; diagnostic policy comparisons, no automatic converter correction or full scene parity',
            'unity_version': report['unity_version'], 'platform': report['platform'],
            'archive_sha256': request['archive_sha256'], 'reference_sha256': digest(report_path),
            'collector_sha256': report['collector_sha256'], 'probe_sha256': digest(probe),
            'meshes_checked': len(all_checks), 'recorded_frame_matches': len(matched),
            'max_matched_point_error_metres': max((v['max_point_error_metres'] for v in matched), default=None),
            'metadata_byte_identical': all(model['metadata_byte_identical'] for model in results), 'models': results}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='mode', required=True)
    s = commands.add_parser('stage')
    s.add_argument('source', type=Path)
    s.add_argument('project', type=Path)
    s.add_argument('--model', action='append', required=True)
    s.add_argument('--unity-version', required=True)
    v = commands.add_parser('inspect')
    v.add_argument('source', type=Path)
    v.add_argument('project', type=Path)
    v.add_argument('--probe', type=Path, required=True)
    v.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.mode == 'stage':
        print(json.dumps(stage(args.source, args.project, args.model, args.unity_version)))
    else:
        if args.report.exists():
            raise FileExistsError('Use a new inspection report')
        result = inspect(args.source, args.project, args.probe)
        args.report.write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps({k: value for k, value in result.items() if k != 'models'}))
