"""Stage/inspect a bounded built-in Plane reference from an empty Linux Editor project."""
import argparse
import json
from pathlib import Path
import shutil
import numpy as np
from unity_editor_reference import BUILD, VERSION, digest, read_json

COLLECTOR = Path(__file__).resolve().parent / 'qualification/TheiaBuiltinPlaneReference.cs'
GUID = '0000000000000000e000000000000000'


def stage(output, version):
    output = Path(output).resolve()
    if output.exists() or not output.is_relative_to(BUILD.resolve()) or output == BUILD.resolve():
        raise ValueError('Use a fresh ignored reference project')
    if not isinstance(version, str) or not VERSION.fullmatch(version):
        raise ValueError('Select exact Editor version')
    for directory in ['Assets/Editor', 'ProjectSettings', 'Packages', 'Reference']:
        (output / directory).mkdir(parents=True, exist_ok=True)
    (output / 'ProjectSettings/ProjectVersion.txt').write_text('m_EditorVersion: ' + version + '\n')
    (output / 'Packages/manifest.json').write_text('{"dependencies": {}}\n')
    shutil.copyfile(COLLECTOR, output / 'Assets/Editor/TheiaBuiltinPlaneReference.cs')
    request = {'schema': 1, 'unity_version': version, 'collector_sha256': digest(COLLECTOR),
               'primitive': 'Plane', 'guid': GUID, 'file_id': '10209'}
    (output / 'Reference/request.json').write_text(json.dumps(request, indent=2) + '\n')
    return request


def inspect(project):
    project = Path(project).resolve(strict=True)
    request = read_json(project / 'Reference/request.json', 65536)
    report = read_json(project / 'Reference/plane.json', 8 * 1024 * 1024)
    if (request['schema'] != 1 or report['schema'] != 1 or
            report['request_sha256'] != digest(project / 'Reference/request.json') or
            not VERSION.fullmatch(request['unity_version']) or report['unity_version'] != request['unity_version'] or
            report['platform'] != 'LinuxEditor'):
        raise ValueError('Built-in reference request/version differs')
    for item in [request, report]:
        if (item['collector_sha256'] != digest(COLLECTOR) or item['primitive'] != 'Plane' or
                item['guid'] != GUID or item['file_id'] != '10209'):
            raise ValueError('Built-in reference identity differs')
    if digest(project / 'Assets/Editor/TheiaBuiltinPlaneReference.cs') != digest(COLLECTOR):
        raise ValueError('Staged built-in collector differs')
    streams = {}
    for key, width in [('positions', 3), ('normals', 3), ('uv0', 2)]:
        values = np.asarray(report[key], dtype=float)
        if values.ndim != 1 or not 3 * width <= len(values) <= 65536 * width or len(values) % width or not np.isfinite(values).all():
            raise ValueError('Invalid bounded built-in vertex stream')
        streams[key] = values.reshape(-1, width)
    positions, normals, uv = (streams[k] for k in ['positions', 'normals', 'uv0'])
    if len(positions) != len(normals) or len(positions) != len(uv) or not np.allclose(np.linalg.norm(normals, axis=1), 1, atol=.0001, rtol=0):
        raise ValueError('Built-in normal/UV stream differs')
    indices = report['indices']
    if (not isinstance(indices, list) or not 3 <= len(indices) <= 393216 or len(indices) % 3 or
            any(type(i) is not int or not 0 <= i < len(positions) for i in indices)):
        raise ValueError('Invalid built-in triangle indices')
    center = np.array([report['bounds_center'][c] for c in 'xyz'])
    size = np.array([report['bounds_size'][c] for c in 'xyz'])
    if not np.isfinite([center, size]).all() or (size < 0).any() or (positions < center - size / 2 - .0001).any() or (positions > center + size / 2 + .0001).any():
        raise ValueError('Built-in bounds differ')
    return {'passed': True, 'project': str(project), 'unity_version': report['unity_version'],
            'guid': GUID, 'file_id': '10209', 'vertices': len(positions), 'triangles': len(indices) // 3,
            'request_sha256': digest(project / 'Reference/request.json'),
            'report_sha256': digest(project / 'Reference/plane.json'), 'collector_sha256': digest(COLLECTOR),
            'scope': 'actual built-in Plane positions/normals/UV0/indices in empty Linux Editor project; source binding and rendering separate'}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('mode', choices=['stage', 'inspect']); p.add_argument('project', type=Path); p.add_argument('--version')
    a = p.parse_args(); print(json.dumps(stage(a.project, a.version) if a.mode == 'stage' else inspect(a.project)))
