#!/usr/bin/env python3
"""Headless real-editor quest-area export, reference checks and file preservation."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def run(output, binary):
    output.mkdir(parents=True, exist_ok=False)
    source = Path('content/zones/quest-areas.zone.json')
    quests = Path('content/quests/development-journal-areas.json')
    original = source.read_bytes(), quests.read_bytes()
    target = output / 'areas.json'
    env = dict(os.environ)
    env.pop('DISPLAY', None)
    env.pop('WAYLAND_DISPLAY', None)
    command = [str(binary), '--zone', str(source), '--export-quest-areas', str(target),
               '--quests', str(quests), '--content-id', 'theia.areas.starter', '--revision', '1']
    def check(name, args, success):
        result = subprocess.run(args, env=env, capture_output=True, text=True, timeout=30)
        (output / f'{name}.log').write_text(result.stdout + result.stderr)
        assert (result.returncode == 0) == success, name
        return result
    result = check('valid', command, True)
    assert json.loads(result.stdout)['quest_references_valid']
    saved = target.read_bytes()
    catalog = json.loads(saved)
    area = json.loads(original[0])['quest_areas'][0]
    assert catalog['areas'][0]['min'] == area['min'] and catalog['areas'][0]['max'] == area['max']
    assert catalog['areas'][0]['map_objectives'] == area['map_objectives']
    check('existing-output', command, False)
    assert target.read_bytes() == saved
    bad = json.loads(original[0])
    bad['quest_areas'][0]['map_objectives'] = ['theia.missing.objective']
    bad_source = output / 'bad.zone.json'
    bad_source.write_text(json.dumps(bad))
    rejected_target = output / 'must-not-exist.json'
    altered = command.copy()
    altered[altered.index('--zone') + 1] = str(bad_source)
    altered[altered.index('--export-quest-areas') + 1] = str(rejected_target)
    check('unknown-objective', altered, False)
    assert not rejected_target.exists()
    check('mixed-exports', command + ['--export', str(output / 'world')], False)
    assert (source.read_bytes(), quests.read_bytes()) == original
    assert not list(output.glob('.quest-areas-*.tmp'))
    report = dict(passed=True, cases=4, source_preserved=True, desktop_used=False,
                  catalog_sha256=hashlib.sha256(saved).hexdigest(),
                  scope='Real headless editor and runtime catalog/reference validation; no server bundle activation or visual editor interaction')
    (output / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output.resolve(), args.binary.resolve())
