"""Rebuild affected static bindings with actual Editor frames; preserve others."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import numpy as np
from merge_static_glb import read
from static_geometry import accessor
from unity_scene import Sources
from unity_editor_reference import mesh_points
from reframe_unity_static import reframe
from verify_unity_reframe import verify as verify_frame
from verify_unity_geometry import assembly, point_error
from unity_geometry import assemble


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def rebuild(source, base, summary_path, projects, output, probe):
    sources = Sources(source)
    base, output = Path(base).resolve(strict=True), Path(output).resolve()
    if output.exists() or output.is_relative_to(base) or output.is_relative_to(sources.root):
        raise ValueError('Use a new output outside inputs')
    summary = json.loads(Path(summary_path).read_text())
    report = json.loads((base / 'batch.json').read_text())
    if summary['source_plan_sha256'] != report['plan_sha256'] or sha(report['source_plan']) != report['plan_sha256']:
        raise ValueError('Source scene plan differs')
    plan = json.loads(Path(report['source_plan']).read_text())
    if plan['archive_sha256'] != sources.manifest['archive_sha256']:
        raise ValueError('Source archive differs')
    references, geometry = {}, {}
    report_hashes = set()
    for p in projects:
        p = Path(p).resolve(strict=True)
        request = json.loads((p / 'Reference/request.json').read_text())
        actual = json.loads((p / 'Reference/imported.json').read_text())
        if actual['request_sha256'] != sha(p / 'Reference/request.json') or request['archive_sha256'] != plan['archive_sha256']:
            raise ValueError('Editor source request differs')
        digest = sha(p / 'Reference/imported.json')
        report_hashes.add(digest)
        for m in actual['models']:
            name = m['path']
            sources.checked(name); sources.metadata(name)
            binding = sources.manifest['selected'][name]
            if m['sha256'] != binding['sha256'] or m['guid'] != binding['guid'] or m['meta_sha256'] != sources.manifest['metadata'][name]['sha256']:
                raise ValueError('Actual Editor source/metadata differs')
            if name in references: raise ValueError('Duplicate model reference')
            references[name] = p
            for mesh in m['meshes']:
                geometry[binding['guid'], int(mesh['file_id'])] = mesh_points(mesh) * [-1, 1, 1]
    for value in summary['reports']:
        if sha(value['path']) != value['sha256']:
            raise ValueError('Reference inspection changed')
        reference = json.loads(Path(value['path']).read_text())
        if reference['reference_sha256'] not in report_hashes or not reference['metadata_byte_identical'] or reference['recorded_frame_matches'] != reference['meshes_checked']:
            raise ValueError('Source model inspection is incomplete')
    affected = {v['key'] for v in summary['affected_current_batch_bindings']}
    if not affected <= {v['key'] for v in report['converted']}:
        raise ValueError('Correction keys differ from converted bindings')
    for record in report['converted']:
        path = base / 'prototypes' / record['key']
        if sha(path / 'preview.glb') != record['prototype_sha256'] or sha(path / 'placements.glb') != record['placements_sha256']:
            raise ValueError('Prior qualified artifact changed')
    shutil.copytree(base, output)
    (output / 'batch.json').rename(output / 'prior-batch.json')
    corrections = []
    for record in report['converted']:
        if record['key'] not in affected: continue
        original = base / 'prototypes' / record['key']
        target = output / 'prototypes' / record['key']
        model = json.loads((original / 'conversion.json').read_text())['model']
        shutil.rmtree(target)
        reframe(source, original, references[model], target, probe)
        checked = verify_frame(source, target, probe)
        for name in ['independent-verification.json', 'canonical-reference.json']:
            (target / name).write_text(json.dumps(checked, indent=2) + '\n')
        instances = json.loads((target / 'instances.json').read_text())['instances']
        placed = assemble(target / 'preview.glb', instances, target / 'placements.glb')
        oracle = assembly(target / 'preview.glb', instances, target / 'placements.glb')
        (target / 'assembly.json').write_text(json.dumps(placed, indent=2) + '\n')
        (target / 'assembly-reference.json').write_text(json.dumps(oracle, indent=2) + '\n')
        record.update(prototype_sha256=checked['asset_sha256'], placements_sha256=placed['asset_sha256'], expected_bounds=oracle['expected_bounds'])
        corrections.append({'key': record['key'], 'draws': record['draws'], 'max_editor_point_error_metres': checked['max_editor_point_error_metres']})
        print(json.dumps(corrections[-1]), flush=True)
    coverage = []
    for record in report['converted']:
        path = output / 'prototypes' / record['key']
        doc, binary = read(path / 'preview.glb')
        points = []
        for p in doc['meshes'][0]['primitives']:
            values = accessor(doc, binary, p['attributes']['POSITION'])
            points.extend(values[np.unique(accessor(doc, binary, p['indices']).ravel())])
        reference = geometry[record['mesh']['guid'], record['mesh']['fileID']]
        error = point_error(reference, points, .0001)
        if record['key'] not in affected:
            for name in ['preview.glb', 'placements.glb']:
                if (path / name).read_bytes() != (base / 'prototypes' / record['key'] / name).read_bytes():
                    raise ValueError('Unaffected artifact bytes changed')
        coverage.append({'key': record['key'], 'asset_sha256': sha(path / 'preview.glb'), 'max_editor_point_error_metres': error})
    report['unity_frame_correction'] = {'base': str(base), 'base_batch_sha256': sha(base / 'batch.json'),
        'reference_summary_sha256': sha(summary_path), 'corrected_bindings': len(corrections),
        'corrected_draws': sum(v['draws'] for v in corrections), 'unchanged_bindings': len(coverage) - len(corrections),
        'all_prototypes_checked_against_editor': len(coverage), 'corrections': corrections}
    (output / 'batch.json').write_text(json.dumps(report, indent=2) + '\n')
    receipt = {'passed': True, 'batch_sha256': sha(output / 'batch.json'), 'prototypes': coverage,
               'max_editor_point_error_metres': max(v['max_editor_point_error_metres'] for v in coverage)}
    (output / 'editor-point-coverage.json').write_text(json.dumps(receipt, indent=2) + '\n')
    return report['unity_frame_correction']


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('source', type=Path); p.add_argument('base', type=Path); p.add_argument('summary', type=Path); p.add_argument('output', type=Path)
    p.add_argument('--project', type=Path, action='append', required=True); p.add_argument('--probe', type=Path, required=True)
    a = p.parse_args(); print(json.dumps(rebuild(a.source, a.base, a.summary, a.project, a.output, a.probe)))
