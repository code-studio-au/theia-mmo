"""Stage only original PNGs/metadata and a read-only effective sampler collector."""
import argparse
import json
import math
from pathlib import Path
import shutil
from PIL import Image
from unity_editor_reference import BUILD, VERSION, digest, read_json
from unity_scene import Sources

COLLECTOR = Path(__file__).resolve().parent / 'qualification/TheiaTextureReference.cs'


def stage(source, output, textures, version):
    output = Path(output).resolve()
    if output.exists() or not output.is_relative_to(BUILD.resolve()) or output == BUILD.resolve():
        raise ValueError('Use a fresh ignored reference project')
    if not VERSION.fullmatch(version) or not 1 <= len(textures) <= 32 or len(set(textures)) != len(textures):
        raise ValueError('Select exact Editor version and 1..32 unique textures')
    sources = Sources(source); inputs = []
    for name in textures:
        if not name.startswith('Assets/') or not name.lower().endswith('.png'):
            raise ValueError('Reference staging accepts source PNG assets only')
        original = sources.checked(name); sources.metadata(name)
        binding = sources.manifest['selected'][name]
        if original.stat().st_size > 64 * 1024 * 1024:
            raise ValueError('Texture byte budget exceeded')
        inputs.append({'path': name, 'guid': binding['guid'], 'sha256': binding['sha256'], 'meta_sha256': sources.manifest['metadata'][name]['sha256']})
    (output / 'Assets/Editor').mkdir(parents=True)
    for name in ['ProjectSettings', 'Packages', 'Reference/original-meta']: (output / name).mkdir(parents=True)
    (output / 'ProjectSettings/ProjectVersion.txt').write_text('m_EditorVersion: ' + version + '\n')
    (output / 'Packages/manifest.json').write_text('{"dependencies": {}}\n')
    shutil.copyfile(COLLECTOR, output / 'Assets/Editor/TheiaTextureReference.cs')
    for item in inputs:
        target = output / item['path']; target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(sources.checked(item['path']), target)
        shutil.copyfile(sources.checked(item['path'], True), str(target) + '.meta')
        shutil.copyfile(sources.checked(item['path'], True), output / 'Reference/original-meta' / (item['guid'] + '.meta'))
    request = {'schema': 1, 'archive_sha256': sources.manifest['archive_sha256'], 'unity_version': version,
               'collector_sha256': digest(COLLECTOR), 'textures': inputs}
    (output / 'Reference/request.json').write_text(json.dumps(request, indent=2) + '\n')
    return request


def inspect(source, project):
    source = Sources(source); project = Path(project).resolve(strict=True)
    request_path = project / 'Reference/request.json'
    request = read_json(request_path, 65536); report = read_json(project / 'Reference/textures.json', 256 * 1024)
    if request['schema'] != 1 or report['schema'] != 1 or request['archive_sha256'] != source.manifest['archive_sha256'] or report['request_sha256'] != digest(request_path):
        raise ValueError('Texture request/source binding differs')
    if report['unity_version'] != request['unity_version'] or report['platform'] != 'LinuxEditor' or report['collector_sha256'] != digest(COLLECTOR) or request['collector_sha256'] != digest(COLLECTOR) or digest(project / 'Assets/Editor/TheiaTextureReference.cs') != digest(COLLECTOR):
        raise ValueError('Texture Editor/collector identity differs')
    inputs = {v['path']: v for v in request['textures']}; actual = {v['path']: v for v in report['textures']}
    if len(inputs) != len(request['textures']) or len(actual) != len(report['textures']) or set(inputs) != set(actual) or not 1 <= len(inputs) <= 32:
        raise ValueError('Texture reference coverage differs')
    for name, item in inputs.items():
        if not name.startswith('Assets/') or not name.lower().endswith('.png') or not (project / name).resolve(strict=True).is_relative_to(project):
            raise ValueError('Texture reference path escapes project')
        original = source.checked(name); source.metadata(name); value = actual[name]
        for key, path in [('sha256', original), ('meta_sha256', source.checked(name, True))]:
            if item[key] != digest(path) or value[key] != digest(path): raise ValueError('Texture source/reference hash differs')
        if item['guid'] != source.manifest['selected'][name]['guid'] or value['guid'] != item['guid'] or value['file_id'] != '2800000':
            raise ValueError('Texture persistent identity differs')
        if digest(project / name) != item['sha256'] or digest(Path(str(project / name) + '.meta')) != item['meta_sha256'] or digest(project / 'Reference/original-meta' / (item['guid'] + '.meta')) != item['meta_sha256']:
            raise ValueError('Editor texture or original metadata changed')
        if any(type(value[k]) is not int or not 1 <= value[k] <= 4096 for k in ['width', 'height']):
            raise ValueError('Texture reference dimensions exceed budget')
        with Image.open(original) as image:
            if image.format != 'PNG' or image.size != (value['width'], value['height']):
                raise ValueError('Editor resized source PNG or image format differs')
        for key, valid in [('filter', range(3)), ('wrap_u', range(4)), ('wrap_v', range(4)), ('aniso', range(17)), ('mip_count', range(1, 14))]:
            if type(value[key]) is not int or value[key] not in valid:
                raise ValueError('Invalid effective texture sampler/mip state')
        if type(value['mip_bias']) not in (int, float) or not math.isfinite(value['mip_bias']) or abs(value['mip_bias']) > 16:
            raise ValueError('Invalid effective texture mip bias')
        if any(type(value[k]) is not bool for k in ['srgb', 'mipmaps', 'convert_normal']):
            raise ValueError('Invalid texture reference boolean')
    return {'passed': True, 'project': str(project), 'unity_version': report['unity_version'],
            'request_sha256': digest(request_path), 'report_sha256': digest(project / 'Reference/textures.json'),
            'collector_sha256': digest(COLLECTOR), 'textures': list(actual.values()),
            'scope': 'actual Linux Editor effective texture/importer state with unchanged original bytes; no rendered Unity parity'}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); sub = p.add_subparsers(dest='mode', required=True)
    s = sub.add_parser('stage'); s.add_argument('source', type=Path); s.add_argument('output', type=Path); s.add_argument('--texture', action='append', required=True); s.add_argument('--version', required=True)
    v = sub.add_parser('inspect'); v.add_argument('source', type=Path); v.add_argument('project', type=Path); v.add_argument('--report', type=Path, required=True)
    a = p.parse_args()
    if a.mode == 'stage': print(json.dumps(stage(a.source, a.output, a.texture, a.version)))
    else:
        if a.report.exists(): raise FileExistsError('Use a fresh report')
        result = inspect(a.source, a.project); a.report.write_text(json.dumps(result, indent=2) + '\n'); print(json.dumps(result))
