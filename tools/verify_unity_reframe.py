"""Independent unchanged-stream and actual Editor point-set checks for reframing."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import numpy as np
from merge_static_glb import read
from static_geometry import accessor, hierarchy
from unity_scene import Sources
from unity_editor_reference import inspect, mesh_points
from verify_unity_geometry import canonical, point_error
from verify_unity_static import verify as verify_static


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def compare(old_doc, old_binary, new_doc, new_binary, scale, translation):
    """Check expected positions and prove all other bytes/graph fields unchanged."""
    if not new_binary.startswith(old_binary):
        raise ValueError('Original binary attribute/image bytes changed')
    before, after = copy.deepcopy(old_doc), copy.deepcopy(new_doc)
    if after['accessors'][:len(before['accessors'])] != before['accessors'] or after['bufferViews'][:len(before['bufferViews'])] != before['bufferViews']:
        raise ValueError('Original accessor/view definitions changed')
    maximum = 0.
    for original, actual in zip(before['meshes'][0]['primitives'], after['meshes'][0]['primitives']):
        old_index, new_index = original['attributes']['POSITION'], actual['attributes']['POSITION']
        old = accessor(old_doc, old_binary, old_index).astype(float)
        observed = accessor(new_doc, new_binary, new_index).astype(float)
        wanted = old * scale + translation
        if observed.shape != wanted.shape or not np.allclose(observed, wanted, atol=.00001, rtol=0):
            raise ValueError('Recorded source frame not applied to positions')
        maximum = max(maximum, float(abs(observed - wanted).max()))
        if new_index < len(before['accessors']):
            raise ValueError('New positions must use new accessors')
        actual['attributes']['POSITION'] = old_index
    after['accessors'] = after['accessors'][:len(before['accessors'])]
    after['bufferViews'] = after['bufferViews'][:len(before['bufferViews'])]
    after['buffers'] = before['buffers']
    if before != after:
        raise ValueError('Frame correction changed topology/UV/normal/material/image or graph')
    return maximum


def verify(source, stage, probe, shader_source=None):
    stage = Path(stage)
    receipt = json.loads((stage / 'unity-frame.json').read_text())
    conversion = json.loads((stage / 'conversion.json').read_text())
    original = Path(receipt['source_prototype'])
    project = Path(receipt['project'])
    if sha(original / 'preview.glb') != receipt['source_prototype_sha256'] or sha(original / 'conversion.json') != receipt['source_conversion_sha256']:
        raise ValueError('Predecessor prototype changed')
    if sha(stage / 'preview.glb') != receipt['asset_sha256'] or conversion.get('unity_frame_reference') != receipt or conversion['preview_sha256'] != receipt['asset_sha256']:
        raise ValueError('Corrected prototype/receipt binding differs')
    if receipt['request_sha256'] != sha(project / 'Reference/request.json') or receipt['editor_report_sha256'] != sha(project / 'Reference/imported.json'):
        raise ValueError('Editor reference binding changed')
    canonical(source, original, probe)
    prior = json.loads((original / 'conversion.json').read_text())
    expected = copy.deepcopy(prior)
    expected['unity_frame_reference'] = receipt
    expected['preview_sha256'] = receipt['asset_sha256']
    if 'asset_sha256' in expected: expected['asset_sha256'] = receipt['asset_sha256']
    if expected != conversion:
        raise ValueError('Source conversion fields changed')
    if prior.get('material_plan', {}).get('profile') in ('unity-standard-opaque-v1','unity-standard-glass-blend-v1'):
        verify_static(source, original)
    elif prior.get('material_plan', {}).get('profile') == 'synty-triplanar-world-v1' and shader_source is not None:
        from verify_unity_triplanar import verify as verify_triplanar
        verify_triplanar(source, shader_source, original)
        settings = json.loads((original / 'triplanar.json').read_text())
        expected_settings = copy.deepcopy(settings)
        expected_settings['geometry_sha256'] = receipt['asset_sha256']
        if json.loads((stage / 'triplanar.json').read_text()) != expected_settings:
            raise ValueError('Triplanar binding changed beyond corrected geometry digest')
        for texture in settings['textures'].values():
            if (stage / texture['file']).read_bytes() != (original / texture['file']).read_bytes():
                raise ValueError('Triplanar texture bytes changed')
    elif prior.get('profile') == 'unity-mixed-crystal-pbr-static-v1' and shader_source is not None:
        from verify_unity_mixed import verify as verify_mixed
        verify_mixed(source,shader_source,original)
    elif isinstance(prior.get('profile'),dict) and prior['profile'].get('profile') in ('synty-full-static-foliage-v1','synty-full-emissive-preview-v1') and shader_source is not None:
        from verify_unity_foliage import verify as verify_foliage
        verify_foliage(source, shader_source, original)
    else:
        raise ValueError('Frame material verification requires a separately qualified profile')
    result = inspect(source, project, probe, {prior['model']})
    if not result['metadata_byte_identical'] or result['meshes_checked'] != result['recorded_frame_matches']:
        raise ValueError('Actual reference import/profile is unqualified')
    model, = result['models']
    mesh, = [v for v in model['meshes'] if v['file_id'] == str(prior['mesh_fileID'])]
    transform = np.array(mesh['canonical_mesh_to_unity_local'], dtype=float)
    transform[0, :] *= -1
    scale = transform[0, 0]
    if not 0 < scale <= 1000 or not np.allclose(transform[:3, :3], np.eye(3) * scale, atol=1e-12, rtol=0):
        raise ValueError('Unqualified rotated/nonuniform correction')
    if not np.array_equal(transform, np.asarray(receipt['canonical_to_render_mesh'])):
        raise ValueError('Recorded correction differs from actual reference frame')
    old_doc, old_binary = read(original / 'preview.glb')
    new_doc, new_binary = read(stage / 'preview.glb')
    rounding = compare(old_doc, old_binary, new_doc, new_binary, scale, transform[:3, 3])
    _, matrices = hierarchy(new_doc)
    if len(new_doc['nodes']) != 1 or not np.array_equal(matrices[0], np.eye(4)):
        raise ValueError('Corrected mesh must retain identity node')
    imported = json.loads((project / 'Reference/imported.json').read_text())
    actual_model, = [v for v in imported['models'] if v['path'] == prior['model']]
    actual_mesh, = [v for v in actual_model['meshes'] if v['file_id'] == str(prior['mesh_fileID'])]
    wanted = mesh_points(actual_mesh) * [-1, 1, 1]
    points = []
    for primitive in new_doc['meshes'][0]['primitives']:
        values = accessor(new_doc, new_binary, primitive['attributes']['POSITION'])
        indices = accessor(new_doc, new_binary, primitive['indices']).ravel()
        points.extend(values[np.unique(indices)])
    error = point_error(wanted, points, .0001)
    # Independently check source/archive binding, beyond receipt claims.
    sources = Sources(source)
    if prior['archive_sha256'] != sources.manifest['archive_sha256']:
        raise ValueError('Source archive differs')
    return {'passed': True, 'asset_sha256': receipt['asset_sha256'], 'max_editor_point_error_metres': error,
            'expected_bounds': [wanted.min(0).tolist(), wanted.max(0).tolist()],
            'triangles': sum(len(accessor(new_doc, new_binary, p['indices']).ravel()) // 3 for p in new_doc['meshes'][0]['primitives']),
            'max_frame_rounding_metres': rounding, 'source_prototype_sha256': receipt['source_prototype_sha256'],
            'editor_report_sha256': receipt['editor_report_sha256'], 'unity_version': result['unity_version'],
            'scope': 'actual Unity mesh-local point sets and invariant source topology/UV/normal/tangent/material/image streams; no full scene/collision adoption'}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('source', type=Path); p.add_argument('stage', type=Path)
    p.add_argument('--probe', type=Path, required=True); p.add_argument('--report', type=Path, required=True)
    p.add_argument('--shader-source', type=Path)
    a = p.parse_args()
    if a.report.exists(): raise FileExistsError('Use a fresh report')
    r = verify(a.source, a.stage, a.probe, a.shader_source); a.report.write_text(json.dumps(r, indent=2) + '\n'); print(json.dumps(r))
