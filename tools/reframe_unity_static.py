"""Apply recorded Unity mesh frames to a new copy of a canonical prototype.

The qualified forest frames need translation/positive uniform scale only. All
other vertex streams, indices, materials and images retain their exact bytes.
No guessed recentering, source changes or implicit runtime map adoption.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import shutil
import numpy as np
from add_terrain_house import write_glb
from merge_static_glb import read
from static_geometry import accessor, hierarchy
from unity_editor_reference import inspect
from verify_unity_geometry import canonical


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def frame(reference):
    transform = np.diag([-1., 1., 1., 1.]) @ np.asarray(reference['canonical_mesh_to_unity_local'], dtype=float)
    if transform.shape != (4, 4) or not np.isfinite(transform).all() or not np.array_equal(transform[3], [0, 0, 0, 1]):
        raise ValueError('Invalid reference frame')
    scale = float(transform[0, 0])
    if not .001 <= scale <= 1000 or not np.allclose(transform[:3, :3], np.eye(3) * scale, atol=1e-12, rtol=0):
        raise ValueError('Reference requires a separately qualified nonuniform/rotated frame')
    if np.abs(transform[:3, 3]).max() > 4096:
        raise ValueError('Reference translation exceeds local asset budget')
    return transform


def transform_document(document, binary, transform):
    doc = copy.deepcopy(document)
    _, matrices = hierarchy(doc)
    if len(doc['nodes']) != 1 or len(doc['meshes']) != 1 or not np.array_equal(matrices[0], np.eye(4)):
        raise ValueError('Expected one identity static prototype')
    if doc.get('animations') or doc.get('skins'):
        raise ValueError('Expected static unskinned prototype')
    packed = bytearray(binary)
    replacements = {}
    for primitive in doc['meshes'][0]['primitives']:
        if primitive.get('targets') or primitive.get('mode', 4) != 4:
            raise ValueError('Expected static triangles')
        original = primitive['attributes']['POSITION']
        if original not in replacements:
            points = accessor(document, binary, original).astype(float)
            result = np.asarray(points @ transform[:3, :3].T + transform[:3, 3], dtype='<f4')
            if result.ndim != 2 or result.shape[1] != 3 or not np.isfinite(result).all() or abs(result).max() > 4096:
                raise ValueError('Corrected positions exceed bounded finite asset profile')
            packed.extend(b'\0' * (-len(packed) % 4))
            offset = len(packed)
            packed.extend(result.tobytes())
            view = len(doc['bufferViews'])
            doc['bufferViews'].append({'buffer': 0, 'byteOffset': offset, 'byteLength': result.nbytes, 'target': 34962})
            replacements[original] = len(doc['accessors'])
            doc['accessors'].append({'bufferView': view, 'componentType': 5126, 'type': 'VEC3', 'count': len(result),
                                     'min': result.min(0).tolist(), 'max': result.max(0).tolist()})
        primitive['attributes']['POSITION'] = replacements[original]
    doc['buffers'][0]['byteLength'] = len(packed)
    return doc, bytes(packed)


def reframe(source, original, project, output, probe):
    original = Path(original).resolve(strict=True)
    output = Path(output).resolve()
    project = Path(project).resolve(strict=True)
    if output.exists() or any(output.is_relative_to(p) or p.is_relative_to(output) for p in (original, project, Path(source).resolve())):
        raise ValueError('Use a fresh stage outside inputs')
    conversion = json.loads((original / 'conversion.json').read_text())
    if not conversion.get('canonical_mesh') or conversion.get('unity_frame_reference'):
        raise ValueError('Expected an uncorrected canonical prototype')
    if conversion.get('preview_sha256', conversion.get('asset_sha256')) != sha(original / 'preview.glb'):
        raise ValueError('Original prototype hash differs')
    before = canonical(source, original, probe)
    reference = inspect(source, project, probe, {conversion['model']})
    if not reference['metadata_byte_identical'] or reference['meshes_checked'] != reference['recorded_frame_matches']:
        raise ValueError('Unity source settings/point-set agreement remain unqualified')
    model, = reference['models']
    mesh, = [v for v in model['meshes'] if int(v['file_id']) == conversion['mesh_fileID']]
    transform = frame(mesh)
    doc, binary = read(original / 'preview.glb')
    corrected, packed = transform_document(doc, binary, transform)
    shutil.copytree(original, output)
    try:
        for name in ['independent-verification.json', 'canonical-reference.json', 'geometry-reference.json', 'assembly.json', 'assembly-reference.json', 'placements.glb']:
            path = output / name
            if path.exists():
                if name == 'placements.glb': path.unlink()
                else: path.rename(output / ('prior-' + name))
        (output / 'preview.glb').unlink()
        write_glb(corrected, packed, output / 'preview.glb')
        receipt = {'schema': 1, 'source_prototype': str(original), 'source_prototype_sha256': sha(original / 'preview.glb'),
                   'source_conversion_sha256': sha(original / 'conversion.json'), 'project': str(project),
                   'request_sha256': sha(project / 'Reference/request.json'), 'editor_report_sha256': reference['reference_sha256'],
                   'model': conversion['model'], 'mesh_file_id': str(conversion['mesh_fileID']),
                   'canonical_to_render_mesh': transform.tolist(), 'asset_sha256': sha(output / 'preview.glb'),
                   'scope': 'recorded Unity static mesh frame; unchanged normal/tangent/UV/index/material/image bytes'}
        (output / 'unity-frame.json').write_text(json.dumps(receipt, indent=2) + '\n')
        (output / 'unity-model-reference.json').write_text(json.dumps(reference, indent=2) + '\n')
        (output / 'prior-canonical-source-check.json').write_text(json.dumps(before, indent=2) + '\n')
        conversion['unity_frame_reference'] = receipt
        conversion['preview_sha256'] = receipt['asset_sha256']
        if 'asset_sha256' in conversion: conversion['asset_sha256'] = receipt['asset_sha256']
        (output / 'conversion.json').write_text(json.dumps(conversion, indent=2) + '\n')
        metadata = output / 'triplanar.json'
        if metadata.exists():
            value = json.loads(metadata.read_text())
            value['geometry_sha256'] = receipt['asset_sha256']
            metadata.write_text(json.dumps(value, indent=2) + '\n')
        return receipt
    except BaseException:
        shutil.rmtree(output)
        raise


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('source', type=Path); p.add_argument('original', type=Path); p.add_argument('project', type=Path); p.add_argument('output', type=Path)
    p.add_argument('--probe', type=Path, required=True)
    a = p.parse_args()
    print(json.dumps(reframe(a.source, a.original, a.project, a.output, a.probe)))
