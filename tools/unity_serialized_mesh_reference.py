"""Collect actual Editor serialized mesh assets and verify original packed bytes.

Only original .asset containers and metadata enter the scratch project. Referenced
source scripts are never staged. The reviewed read-only collector exports Mesh
subassets; source objects and their geometry remain unchanged.
"""
import argparse
import json
from pathlib import Path
import shutil
import numpy as np
from unity_scene import Sources, records
from unity_editor_reference import BUILD, VERSION, digest, read_json, mesh_points

COLLECTOR = Path(__file__).resolve().parent / 'qualification/TheiaSerializedMeshReference.cs'


def decode(fields):
    vertex = fields['m_VertexData']
    count = vertex['m_VertexCount']
    channels = vertex['m_Channels']
    if type(count) is not int or not 1 <= count <= 2000000 or not channels or channels[0] != {'stream': 0, 'offset': 0, 'format': 0, 'dimension': 3} or any(c['dimension'] for c in channels[1:]):
        raise ValueError('Only bounded position-only float32 serialized meshes are qualified')
    stream = fields['m_StreamData']
    if stream['size'] or stream['offset'] or stream['path'] or fields['m_MeshCompression'] or fields['m_BindPose']:
        raise ValueError('External/compressed/skinned serialized geometry is unsupported')
    raw = bytes.fromhex(vertex['_typelessdata'])
    if len(raw) != 12 * count or vertex['m_DataSize'] != len(raw):
        raise ValueError('Serialized vertex byte count differs')
    points = np.frombuffer(raw, dtype='<f4').reshape(-1, 3)
    if not np.isfinite(points).all() or fields['m_IndexFormat'] != 0:
        raise ValueError('Invalid serialized positions/index format')
    indices = bytes.fromhex(fields['m_IndexBuffer'])
    if len(indices) % 2 or len(indices) > 16000000:
        raise ValueError('Invalid serialized index buffer')
    submeshes = []
    if not 1 <= len(fields['m_SubMeshes']) <= 256:
        raise ValueError('Serialized submesh budget differs')
    for sub in fields['m_SubMeshes']:
        offset, length = sub['firstByte'], sub['indexCount']
        if any(type(v) is not int for v in [offset, length, sub['baseVertex']]) or offset < 0 or offset % 2 or length < 3 or length % 3 or offset + 2 * length > len(indices) or sub['topology'] != 0:
            raise ValueError('Invalid serialized triangle slice')
        values = np.frombuffer(indices, dtype='<u2', count=length, offset=offset).astype(np.int64) + sub['baseVertex']
        if values.min() < 0 or values.max() >= count:
            raise ValueError('Serialized index out of range')
        submeshes.append(values)
        if sum(len(v) for v in submeshes) > 8000000:
            raise ValueError('Serialized submesh index budget exceeded')
    return points, submeshes


def stage(source, output, assets, version):
    output = Path(output).resolve()
    if output.exists() or not output.is_relative_to(BUILD.resolve()) or output == BUILD.resolve():
        raise ValueError('Use a fresh ignored reference project')
    if not VERSION.fullmatch(version) or not 1 <= len(assets) <= 32 or len(set(assets)) != len(assets):
        raise ValueError('Select an exact Editor version and 1..32 unique assets')
    sources = Sources(source)
    inputs = []
    for name in assets:
        if not name.endswith('.asset'):
            raise ValueError('Only serialized mesh asset containers are accepted')
        rr = records(sources.checked(name))
        meshes = [r for r in rr.values() if r['kind'] == 43]
        if not meshes:
            raise ValueError('Serialized asset has no mesh subassets')
        for mesh in meshes:
            decode(mesh['fields'])
        sources.metadata(name)
        binding = sources.manifest['selected'][name]
        inputs.append({'path': name, 'guid': binding['guid'], 'sha256': binding['sha256'],
                       'meta_sha256': sources.manifest['metadata'][name]['sha256']})
    output.mkdir(parents=True)
    (output / 'Assets/Editor').mkdir(parents=True)
    (output / 'ProjectSettings').mkdir()
    (output / 'Packages').mkdir()
    (output / 'Reference').mkdir()
    (output / 'ProjectSettings/ProjectVersion.txt').write_text('m_EditorVersion: ' + version + '\n')
    (output / 'Packages/manifest.json').write_text('{"dependencies": {}}\n')
    shutil.copyfile(COLLECTOR, output / 'Assets/Editor/TheiaSerializedMeshReference.cs')
    for item in inputs:
        target = output / item['path']
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(sources.checked(item['path']), target)
        shutil.copyfile(sources.checked(item['path'], True), str(target) + '.meta')
    request = {'schema': 1, 'archive_sha256': sources.manifest['archive_sha256'],
               'unity_version': version, 'collector_sha256': digest(COLLECTOR), 'models': inputs}
    (output / 'Reference/request.json').write_text(json.dumps(request, indent=2) + '\n')
    return request


def inspect(source, project):
    sources = Sources(source)
    project = Path(project).resolve(strict=True)
    request = read_json(project / 'Reference/request.json', 65536)
    path = project / 'Reference/imported.json'
    report = read_json(path, 256 * 1024 * 1024)
    collector = project / 'Assets/Editor/TheiaSerializedMeshReference.cs'
    if request['schema'] != 1 or report['schema'] != 1 or report['platform'] != 'LinuxEditor' or request['archive_sha256'] != sources.manifest['archive_sha256'] or request['unity_version'] != report['unity_version'] or report['request_sha256'] != digest(project / 'Reference/request.json') or request['collector_sha256'] != digest(COLLECTOR) or report['collector_sha256'] != digest(collector) or digest(collector) != digest(COLLECTOR):
        raise ValueError('Serialized mesh reference request/version/collector differs')
    expected = {item['path']: item for item in request['models']}
    actual = {item['path']: item for item in report['models']}
    if not 1 <= len(expected) <= 32 or len(expected) != len(request['models']) or set(expected) != set(actual) or len(actual) != len(report['models']):
        raise ValueError('Serialized reference coverage differs')
    meshes = []
    for name, item in expected.items():
        binding = sources.manifest['selected'][name]
        sources.checked(name)
        sources.metadata(name)
        if not name.endswith('.asset') or any(item[k] != binding[k] for k in ['guid', 'sha256']) or item['meta_sha256'] != sources.manifest['metadata'][name]['sha256']:
            raise ValueError('Serialized reference source binding differs')
        if digest(project / name) != item['sha256'] or digest(project / (name + '.meta')) != item['meta_sha256'] or any(actual[name][k] != item[k] for k in ['guid', 'sha256', 'meta_sha256']) or actual[name]['nodes']:
            raise ValueError('Read-only reference changed source/metadata or has unexpected hierarchy')
        raw = {i: r['fields'] for i, r in records(sources.checked(name)).items() if r['kind'] == 43}
        observed = {int(m['file_id']): m for m in actual[name]['meshes']}
        if set(raw) != set(observed) or len(observed) != len(actual[name]['meshes']):
            raise ValueError('Serialized mesh persistent IDs differ')
        for identifier, fields in raw.items():
            mesh = observed[identifier]
            mesh_points(mesh)
            points, indices = decode(fields)
            if mesh['name'] != fields['m_Name'] or not np.array_equal(points.ravel(), mesh['positions']) or len(indices) != len(mesh['submeshes']) or any(not np.array_equal(a, b['indices']) for a, b in zip(indices, mesh['submeshes'])):
                raise ValueError('Actual Editor geometry differs from original serialized bytes')
            meshes.append({'path': name, 'guid': item['guid'], 'file_id': str(identifier),
                           'vertices': len(points), 'triangles': sum(len(v) // 3 for v in indices)})
    return {'passed': True, 'unity_version': report['unity_version'], 'platform': report['platform'],
            'collector_sha256': digest(COLLECTOR), 'report_sha256': digest(path), 'meshes': meshes,
            'metadata_byte_identical': True,
            'scope': 'Actual imported serialized mesh positions/indices exactly equal original packed bytes; no convex cooking, world pose or collision-query parity'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='mode', required=True)
    command = commands.add_parser('stage')
    command.add_argument('source', type=Path)
    command.add_argument('project', type=Path)
    command.add_argument('--asset', action='append', required=True)
    command.add_argument('--unity-version', required=True)
    command = commands.add_parser('inspect')
    command.add_argument('source', type=Path)
    command.add_argument('project', type=Path)
    command.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.mode == 'stage':
        print(json.dumps(stage(args.source, args.project, args.asset, args.unity_version)))
    else:
        if args.report.exists():
            raise FileExistsError('Use a fresh report')
        result = inspect(args.source, args.project)
        args.report.write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result))
