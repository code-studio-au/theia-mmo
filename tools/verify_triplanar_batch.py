"""Independently qualify each new source binding and classify the remaining plan."""
import argparse
import hashlib
import json
from pathlib import Path
from verify_unity_reframe import verify as verify_frame
from verify_unity_geometry import assembly


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def verify(source, shaders, batch, probe):
    batch = Path(batch)
    report = json.loads((batch / 'batch.json').read_text())
    base = Path(report['base'])
    prior = json.loads((base / 'batch.json').read_text())
    plan_path = Path(report['source_plan'])
    if sha(base / 'batch.json') != report['base_batch_sha256'] or sha(plan_path) != report['plan_sha256'] or prior['plan_sha256'] != report['plan_sha256']:
        raise ValueError('Source plan/base binding changed')
    groups = {}
    for draw in json.loads(plan_path.read_text())['draws']:
        key = hashlib.sha256(json.dumps([draw['mesh'], draw['materials']], sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        groups.setdefault(key, []).append(draw)
    old = {v['key'] for v in prior['converted']}
    remaining = {v['key'] for v in prior['unresolved']}
    converted = {v['key'] for v in report['converted']}
    unresolved = {v['key'] for v in report['unresolved']}
    if old & remaining or old | remaining != set(groups) or len(old) != len(prior['converted']) or len(remaining) != len(prior['unresolved']):
        raise ValueError('Base plan classification differs')
    if converted & unresolved or not (converted | unresolved) <= remaining or len(converted) != len(report['converted']) or len(unresolved) != len(report['unresolved']):
        raise ValueError('New classification duplicates/overlaps source bindings')
    if len(converted | unresolved) != report['selected_bindings'] or sum(len(groups[k]) for k in converted | unresolved) != report['selected_draws']:
        raise ValueError('Selected binding coverage differs')
    checks = []
    for record in report['converted']:
        draws = groups[record['key']]
        if record['mesh'] != draws[0]['mesh'] or record['materials'] != draws[0]['materials'] or record['draws'] != len(draws):
            raise ValueError('Source binding/count changed')
        stage = batch / 'prototypes' / record['key']
        frame = verify_frame(source, stage, probe, shaders)
        if not frame['passed'] or frame['asset_sha256'] != record['prototype_sha256']:
            raise ValueError('Source prototype qualification differs')
        wanted = [{'id': str(d['instance']) + ':' + str(d['renderer']), 'matrix': d['matrix']} for d in draws]
        actual = json.loads((stage / 'instances.json').read_text())
        if actual['source_plan_sha256'] != report['plan_sha256'] or actual['instances'] != wanted:
            raise ValueError('Placement IDs/matrices differ from complete source group')
        placed = assembly(stage / 'preview.glb', wanted, stage / 'placements.glb')
        if placed['asset_sha256'] != record['placements_sha256'] or placed['triangles'] != record['triangles']:
            raise ValueError('Source assembly changed')
        checks.append({'key': record['key'], 'draws': len(draws), 'prototype': frame, 'placements': placed})
    count = sum(len(groups[k]) for k in converted)
    if count != report['converted_draws'] or sum(len(groups[k]) for k in unresolved) != report['unresolved_draws']:
        raise ValueError('Batch converted/unresolved totals differ')
    return {'passed': True, 'batch_sha256': sha(batch / 'batch.json'), 'source_plan_sha256': report['plan_sha256'],
            'new_bindings': len(converted), 'new_draws': count, 'qualified_source_draws': sum(len(groups[k]) for k in old | converted),
            'remaining_draws': sum(len(groups[k]) for k in remaining - converted), 'remaining_bindings': sorted(remaining - converted),
            'max_editor_point_error_metres': max(v['prototype']['max_editor_point_error_metres'] for v in checks), 'checks': checks,
            'scope': 'fresh source material/Editor frame and exact source-plan placement checks; preserved base qualification remains separate'}


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('source', type=Path); p.add_argument('shaders', type=Path); p.add_argument('batch', type=Path)
    p.add_argument('--probe', type=Path, required=True); p.add_argument('--report', type=Path, required=True)
    a = p.parse_args()
    if a.report.exists(): raise FileExistsError('Use a fresh report')
    value = verify(a.source, a.shaders, a.batch, a.probe)
    a.report.write_text(json.dumps(value, indent=2) + '\n'); print(json.dumps({k: v for k, v in value.items() if k != 'checks'}))
