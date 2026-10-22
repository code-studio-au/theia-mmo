"""Independently verify explicit selective-light omission and unchanged assets."""
import argparse
import hashlib
import json
from pathlib import Path
from verify_unity_lighting import verify


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def inspect(stage):
    stage = Path(stage).resolve(strict=True)
    receipt = json.loads((stage / 'standard-lighting.json').read_text())
    if receipt['schema'] != 1 or receipt['profile'] != 'standard-unlinked-lighting-preview-v1':
        raise ValueError('Unknown standard lighting adaptation')
    source = Path(receipt['source']).resolve(strict=True)
    checked = json.loads(json.dumps(verify(source)))
    if receipt['source_verification'] != checked:
        raise ValueError('Source qualification differs')
    original = {str(p.relative_to(source)): sha(p) for p in source.rglob('*') if p.is_file()}
    if receipt['source_files'] != original:
        raise ValueError('Source files differ')
    actual = {str(p.relative_to(stage)): sha(p) for p in stage.rglob('*') if p.is_file()}
    actual.pop('standard-lighting.json')
    lighting_hash = actual.pop('lighting.json')
    original.pop('lighting.json')
    if actual != original or lighting_hash != receipt['lighting_sha256']:
        raise ValueError('Adaptation changed files beyond lighting metadata')
    before = json.loads((source / 'lighting.json').read_text())
    after = json.loads((stage / 'lighting.json').read_text())
    omitted = []
    retained = []
    for light in before['lights']:
        (retained if light['mask'] == 4294967295 else omitted).append(light)
    before['lights'] = retained
    if after != before or not retained or receipt['omitted_lights'] != omitted:
        raise ValueError('Selective omission changed supported lights or other settings')
    return {'passed': True, 'retained_lights': len(retained),
            'omitted_lights': len(omitted), 'asset_sha256': checked['asset_sha256'],
            'scope': 'Explicit supported-light subset; no replacement lighting or source receiver-mask parity'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('stage', type=Path)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args()
    if args.report.exists():
        raise FileExistsError('Use a fresh report')
    result = inspect(args.stage)
    args.report.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))
