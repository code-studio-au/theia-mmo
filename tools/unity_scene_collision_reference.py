"""Stage data-only source content for an actual Editor scene collision audit.

The package's selected extraction contains no scripts, shaders, assemblies or
packages. Only the reviewed collector is executable; the scene opens in edit
mode, is never saved, and every original staged input is checked before/after.
"""
import argparse
import collections
import json
from pathlib import Path
import shutil
import numpy as np
from unity_scene import Sources
from unity_editor_reference import BUILD, VERSION, digest, read_json, matrix

COLLECTOR = Path(__file__).resolve().parent / 'qualification/TheiaSceneCollisionReference.cs'
EXTENSIONS = {'.fbx', '.prefab', '.png', '.mat', '.asset', '.terrainlayer', '.unity', '.tga', '.jpg'}


def stage(source, output, scene, version):
    sources = Sources(source)
    output = Path(output).resolve()
    if output.exists() or output == BUILD.resolve() or not output.is_relative_to(BUILD.resolve()) or sources.root.is_relative_to(output):
        raise ValueError('Use a fresh ignored scene reference project')
    if not VERSION.fullmatch(version) or not scene.endswith('.unity') or scene not in sources.manifest['selected']:
        raise ValueError('Bind an exact Editor version and original scene')
    inputs = []
    for name, binding in sorted(sources.manifest['selected'].items()):
        if Path(name).suffix.lower() not in EXTENSIONS:
            raise ValueError('Executable or unqualified source input is excluded')
        sources.checked(name)
        sources.metadata(name)
        inputs.append({'path': name, 'guid': binding['guid'], 'sha256': binding['sha256'],
                       'meta_sha256': sources.manifest['metadata'][name]['sha256']})
    if not 1 <= len(inputs) <= 768:
        raise ValueError('Source input budget exceeded')
    try:
        for directory in ['Assets/Editor', 'ProjectSettings', 'Packages', 'Reference']:
            (output / directory).mkdir(parents=True, exist_ok=True)
        (output / 'ProjectSettings/ProjectVersion.txt').write_text('m_EditorVersion: ' + version + '\n')
        (output / 'Packages/manifest.json').write_text('{"dependencies": {}}\n')
        shutil.copyfile(COLLECTOR, output / 'Assets/Editor/TheiaSceneCollisionReference.cs')
        for item in inputs:
            target = output / item['path']
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(sources.checked(item['path']), target)
            shutil.copyfile(sources.checked(item['path'], True), str(target) + '.meta')
        request = {'schema': 1, 'archive_sha256': sources.manifest['archive_sha256'],
                   'unity_version': version, 'collector_sha256': digest(COLLECTOR),
                   'scene': scene, 'inputs': inputs}
        (output / 'Reference/request.json').write_text(json.dumps(request, indent=2) + '\n')
        return request
    except BaseException:
        if output.exists():
            shutil.rmtree(output)
        raise


def inspect(source, project):
    sources = Sources(source)
    project = Path(project).resolve(strict=True)
    request_path = project / 'Reference/request.json'
    path = project / 'Reference/imported.json'
    request = read_json(request_path, 512 * 1024)
    report = read_json(path, 64 * 1024 * 1024)
    collector = project / 'Assets/Editor/TheiaSceneCollisionReference.cs'
    if request['schema'] != 1 or report['schema'] != 1 or request['archive_sha256'] != sources.manifest['archive_sha256'] or request['unity_version'] != report['unity_version'] or report['platform'] != 'LinuxEditor' or report['request_sha256'] != digest(request_path) or request['collector_sha256'] != digest(COLLECTOR) or report['collector_sha256'] != digest(collector) or digest(collector) != digest(COLLECTOR):
        raise ValueError('Scene reference version/collector/source binding differs')
    expected = set(sources.manifest['selected'])
    actual = {item['path'] for item in request['inputs']}
    if actual != expected or len(actual) != len(request['inputs']) or not 1 <= len(actual) <= 768:
        raise ValueError('Scene source input coverage differs')
    for item in request['inputs']:
        name = item['path']
        binding = sources.manifest['selected'][name]
        sources.checked(name)
        sources.metadata(name)
        if Path(name).suffix.lower() not in EXTENSIONS or any(item[k] != binding[k] for k in ['guid', 'sha256']) or item['meta_sha256'] != sources.manifest['metadata'][name]['sha256'] or digest(project / name) != item['sha256'] or digest(project / (name + '.meta')) != item['meta_sha256']:
            raise ValueError('Scene staged input or metadata changed')
    if report['scene_guid'] != sources.manifest['selected'][request['scene']]['guid'] or report['playing'] or report['resolved_behaviours'] != 0:
        raise ValueError('Scene identity or source-script exclusion differs')
    colliders = report['colliders']
    if not 1 <= len(colliders) <= 8192 or len({v['global_id'] for v in colliders}) != len(colliders):
        raise ValueError('Scene collider identity/count budget differs')
    active = collections.Counter()
    total = collections.Counter()
    hits = misses = 0
    for shape in colliders:
        if shape['kind'] not in {'MeshCollider', 'BoxCollider', 'CapsuleCollider', 'TerrainCollider'} or shape['attached_rigidbody']:
            raise ValueError('Unsupported source collision body')
        matrix(shape['world_matrix'])
        total[shape['kind']] += 1
        live = shape['active'] and shape['enabled']
        if live:
            active[shape['kind']] += 1
        if len(shape['probes']) != (6 if live else 0):
            raise ValueError('Scene actual-query coverage differs')
        for probe in shape['probes']:
            origin = np.array([probe['origin'][a] for a in 'xyz'])
            direction = np.array([probe['direction'][a] for a in 'xyz'])
            point = np.array([probe['point'][a] for a in 'xyz'])
            normal = np.array([probe['normal'][a] for a in 'xyz'])
            if not np.isfinite(np.concatenate([origin, direction, point, normal, [probe['maximum'], probe['distance']]])).all() or abs(np.linalg.norm(direction) - 1) > .0001 or probe['maximum'] <= 0:
                raise ValueError('Invalid actual collider probe')
            if probe['hit']:
                hits += 1
                if not 0 <= probe['distance'] <= probe['maximum'] or not np.allclose(origin + direction * probe['distance'], point, atol=.0002, rtol=0) or abs(np.linalg.norm(normal) - 1) > .001:
                    raise ValueError('Actual collider hit geometry differs')
            else:
                misses += 1
    if hits < 100:
        raise ValueError('Actual Editor physics queries were not established')
    return {'passed': True, 'unity_version': report['unity_version'], 'platform': report['platform'],
            'collector_sha256': digest(COLLECTOR), 'report_sha256': digest(path),
            'total_colliders': dict(total), 'active_enabled_colliders': dict(active),
            'actual_probe_hits': hits, 'actual_probe_misses': misses,
            'metadata_byte_identical': True, 'missing_behaviours': report['missing_behaviours'],
            'scope': 'Actual source scene Editor collider identities/world matrices/components/native probes; independent raw hierarchy and Avian parity remain separate'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='mode', required=True)
    command = commands.add_parser('stage')
    command.add_argument('source', type=Path)
    command.add_argument('project', type=Path)
    command.add_argument('--scene', required=True)
    command.add_argument('--unity-version', required=True)
    command = commands.add_parser('inspect')
    command.add_argument('source', type=Path)
    command.add_argument('project', type=Path)
    command.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.mode == 'stage':
        result = stage(args.source, args.project, args.scene, args.unity_version)
        print(json.dumps({'inputs': len(result['inputs'])}))
    else:
        if args.report.exists():
            raise FileExistsError('Use a fresh report')
        result = inspect(args.source, args.project)
        args.report.write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result))
