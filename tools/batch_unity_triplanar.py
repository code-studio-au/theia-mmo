"""Qualify remaining exact triplanar bindings against actual Editor mesh frames.

Run in the isolated Blender environment. Unsupported profiles retain explicit
reasons; no shader/texture fallback is counted as converted source content.
"""
import argparse
import hashlib
import json
from pathlib import Path
from unity_scene import Sources, records
from preview_unity_static import preview
from reframe_unity_static import reframe
from verify_unity_reframe import verify
from unity_geometry import assemble
from verify_unity_geometry import assembly


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def batch(source, shaders, base, projects, output, probe):
    sources = Sources(source)
    base, output = Path(base).resolve(strict=True), Path(output).resolve()
    if output.exists() or output.is_relative_to(base) or output.is_relative_to(sources.root):
        raise ValueError('Use a fresh output outside inputs')
    original_report = base / 'batch.json'
    base_hash = sha(original_report)
    prior = json.loads(original_report.read_text())
    plan_path = Path(prior['source_plan'])
    if sha(plan_path) != prior['plan_sha256']:
        raise ValueError('Source plan changed')
    plan = json.loads(plan_path.read_text())
    if plan['archive_sha256'] != sources.manifest['archive_sha256']:
        raise ValueError('Source archive differs')
    groups = {}
    for d in plan['draws']:
        key = hashlib.sha256(json.dumps([d['mesh'], d['materials']], sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        groups.setdefault(key, []).append(d)
    references = {}
    for project in projects:
        project = Path(project).resolve(strict=True)
        for m in json.loads((project / 'Reference/request.json').read_text())['models']:
            if m['path'] in references: raise ValueError('Duplicate Editor model reference')
            references[m['path']] = project
    selected = []
    for r in prior['unresolved']:
        ds = groups[r['key']]
        refs = ds[0]['materials']
        if not refs or any(v != refs[0] for v in refs): continue
        material = sources.by_guid[refs[0]['guid']]
        fields, = [v['fields'] for v in records(sources.checked(material)).values() if v['kind'] == 21]
        if fields['m_Shader'].get('guid') == '66f8401a4c881134f8803eceae302347':
            selected.append((r, ds))
    if not selected: raise ValueError('No unresolved exact triplanar bindings')
    output.mkdir(parents=True)
    report = {'schema': 1, 'source_plan': str(plan_path), 'plan_sha256': prior['plan_sha256'],
              'base': str(base), 'base_batch_sha256': base_hash, 'selected_bindings': len(selected),
              'selected_draws': sum(len(ds) for _, ds in selected), 'converted': [], 'unresolved': []}
    for record, draws in selected:
        record = dict(record)
        record.pop('reason', None); record.pop('exception', None)
        witness = draws[0]
        raw = output / 'source' / record['key']
        target = output / 'prototypes' / record['key']
        try:
            preview(source, witness['source'], raw, False, True, plan_path=plan_path, instance=witness['instance'], renderer=witness['renderer'], triplanar_shader_source=shaders)
            conversion = json.loads((raw / 'conversion.json').read_text())
            reframe(source, raw, references[conversion['model']], target, probe)
            checked = verify(source, target, probe, shaders)
            (target / 'independent-verification.json').write_text(json.dumps(checked, indent=2) + '\n')
            instances = [{'id': str(d['instance']) + ':' + str(d['renderer']), 'matrix': d['matrix']} for d in draws]
            (target / 'instances.json').write_text(json.dumps({'source_plan_sha256': report['plan_sha256'], 'instances': instances}) + '\n')
            placed = assemble(target / 'preview.glb', instances, target / 'placements.glb')
            oracle = assembly(target / 'preview.glb', instances, target / 'placements.glb')
            (target / 'assembly.json').write_text(json.dumps(placed, indent=2) + '\n')
            (target / 'assembly-reference.json').write_text(json.dumps(oracle, indent=2) + '\n')
            record.update(source=witness['source'], instance=witness['instance'], renderer=witness['renderer'],
                          prototype_sha256=checked['asset_sha256'], placements_sha256=placed['asset_sha256'],
                          triangles=oracle['triangles'], expected_bounds=oracle['expected_bounds'],
                          max_editor_point_error_metres=checked['max_editor_point_error_metres'])
            report['converted'].append(record)
            print('QUALIFIED', record['key'], len(draws), witness['source'], flush=True)
        except (ValueError, KeyError, IndexError, TypeError) as error:
            record.update(reason=str(error), exception=type(error).__name__)
            report['unresolved'].append(record)
            print('UNRESOLVED', record['key'], len(draws), str(error), flush=True)
        if sha(plan_path) != report['plan_sha256'] or sha(original_report) != base_hash:
            raise ValueError('Source plan/base changed during conversion')
        (output / 'batch.json').write_text(json.dumps(report, indent=2) + '\n')
    report['converted_draws'] = sum(r['draws'] for r in report['converted'])
    report['unresolved_draws'] = sum(r['draws'] for r in report['unresolved'])
    (output / 'batch.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('source', type=Path); p.add_argument('shaders', type=Path); p.add_argument('base', type=Path); p.add_argument('output', type=Path)
    p.add_argument('--project', action='append', type=Path, required=True); p.add_argument('--probe', type=Path, required=True)
    a = p.parse_args(); r = batch(a.source, a.shaders, a.base, a.project, a.output, a.probe)
    print(json.dumps({k: v for k, v in r.items() if k not in ['converted', 'unresolved']}))
