"""Bind qualified prototype frames and exact source placements for catalog inputs."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
from verify_unity_geometry import assembly


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def prepare(batch, output):
    batch = Path(batch).resolve(strict=True)
    output = Path(output).resolve()
    if output.exists() or not output.is_relative_to(Path(__file__).resolve().parents[1] / 'build'):
        raise ValueError('Use a fresh output under ignored build/')
    report = json.loads((batch / 'batch.json').read_text())
    plan_path = Path(report['source_plan'])
    if sha(plan_path) != report['plan_sha256']:
        raise ValueError('Source plan changed')
    groups = {}
    for draw in json.loads(plan_path.read_text())['draws']:
        key = hashlib.sha256(json.dumps([draw['mesh'], draw['materials']], sort_keys=True, separators=(',', ':')).encode()).hexdigest()
        groups.setdefault(key, []).append({'id': str(draw['instance']) + ':' + str(draw['renderer']), 'matrix': draw['matrix']})
    output.mkdir(parents=True)
    entries = []
    for record in report['converted']:
        source = batch / 'prototypes' / record['key']
        proof = json.loads((source / 'independent-verification.json').read_text())
        if not proof['passed'] or sha(source / 'preview.glb') != record['prototype_sha256'] or proof['asset_sha256'] != record['prototype_sha256']:
            raise ValueError('Prototype source qualification changed')
        instances = json.loads((source / 'instances.json').read_text())
        if instances['source_plan_sha256'] != report['plan_sha256'] or instances['instances'] != groups[record['key']]:
            raise ValueError('Exact source placement set changed')
        checked = assembly(source / 'preview.glb', instances['instances'], source / 'placements.glb')
        if checked['asset_sha256'] != record['placements_sha256'] or checked['instances'] != record['draws']:
            raise ValueError('Placed geometry binding changed')
        stage = output / record['key']; stage.mkdir()
        shutil.copyfile(source / 'placements.glb', stage / 'preview.glb')
        config = json.loads((source / 'triplanar.json').read_text())
        if config['geometry_sha256'] != record['prototype_sha256']:
            raise ValueError('Prototype material geometry binding changed')
        for texture in config['textures'].values():
            if texture['file'] not in ['triplanar-' + role + '.png' for role in ['top', 'side', 'top_normal', 'side_normal']] or sha(source / texture['file']) != texture['sha256']:
                raise ValueError('Prototype texture binding changed')
            shutil.copyfile(source / texture['file'], stage / texture['file'])
        config['geometry_sha256'] = checked['asset_sha256']
        (stage / 'triplanar.json').write_text(json.dumps(config, indent=2) + '\n')
        checked.update(prototype_proof_sha256=sha(source / 'independent-verification.json'),
                       prototype_sha256=record['prototype_sha256'], source_plan_sha256=report['plan_sha256'],
                       source=str(source), scope='independent exact source-plan placements plus bound qualified Editor-frame prototype/material predecessor')
        (stage / 'independent-verification.json').write_text(json.dumps(checked, indent=2) + '\n')
        entries.append({'stage': str(stage), 'material_guid': record['materials'][0]['guid'], 'draws': record['draws']})
    (output / 'prepared.json').write_text(json.dumps(entries, indent=2) + '\n')
    return entries


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('batch', type=Path); p.add_argument('output', type=Path)
    a = p.parse_args(); print(json.dumps(prepare(a.batch, a.output)))
