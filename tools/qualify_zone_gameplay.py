#!/usr/bin/env python3
"""Actual headless editor combined world/gameplay export and launcher snapshot."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gameplay_bundle import snapshot


def run(output, binary, validator):
    output.mkdir(parents=True, exist_ok=False)
    source = Path('content/zones/quest-areas.zone.json')
    template = Path('content/development-gameplay-classic-quest-areas.json')
    env = {k: v for k, v in os.environ.items() if k not in ('DISPLAY', 'WAYLAND_DISPLAY')}
    source_bytes = source.read_bytes()
    with snapshot(template, validator, output / 'input-snapshot') as (paths, initial):
        hashes = {kind: hashlib.sha256(path.read_bytes()).hexdigest() for kind, path in paths.items()}
    world = output / 'world'
    command = [str(binary), '--zone', str(source), '--export', str(world),
               '--gameplay-template', str(template), '--content-id', 'theia.starter.authored',
               '--revision', '2']

    def check(name, args, success):
        result = subprocess.run(args, env=env, capture_output=True, text=True, timeout=60)
        (output / f'{name}.log').write_text(result.stdout + result.stderr)
        assert (result.returncode == 0) == success, name
        return result

    report = json.loads(check('valid', command, True).stdout.splitlines()[-1])
    assert report['quest_areas'] == 1 and report['gameplay']['validated']
    manifest = world / 'gameplay/gameplay.manifest.json'
    with snapshot(manifest, validator, output / 'launch-snapshot') as (paths, published):
        assert published == report['gameplay']
        for kind, path in paths.items():
            if kind != 'quest_areas':
                assert hashlib.sha256(path.read_bytes()).hexdigest() == hashes[kind]
        area = json.loads(paths['quest_areas'].read_bytes())['areas'][0]
        authored = json.loads(source_bytes)['quest_areas'][0]
        assert all(area[k] == authored[k] for k in ('min', 'max', 'map_objectives'))
    saved = {str(p.relative_to(world)): hashlib.sha256(p.read_bytes()).hexdigest()
             for p in world.rglob('*') if p.is_file()}
    check('existing-output', command, False)
    assert saved == {str(p.relative_to(world)): hashlib.sha256(p.read_bytes()).hexdigest()
                     for p in world.rglob('*') if p.is_file()}
    bad = json.loads(source_bytes)
    bad['quest_areas'][0]['definition'] = 'theia.area.missing'
    bad_source = output / 'bad.zone.json'
    bad_source.write_text(json.dumps(bad))
    altered = command.copy()
    altered[altered.index('--zone') + 1] = str(bad_source)
    altered[altered.index('--export') + 1] = str(output / 'invalid')
    check('missing-reach-area', altered, False)
    assert not (output / 'invalid').exists()
    check('template-without-export', [str(binary), '--headless', '--gameplay-template', str(template)], False)
    assert source.read_bytes() == source_bytes
    with snapshot(template, validator, output / 'source-recheck') as (_, checked):
        assert checked == initial
    assert not list(output.glob('.zone-export-*'))
    result = dict(passed=True, cases=4, desktop_used=False, sources_preserved=True,
                  launcher_snapshot_valid=True, bundle=published['validation_bundle_sha256'],
                  scope='Actual editor export and launcher snapshot validation; no connected gameplay or native rendering qualification')
    (output / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--validator', type=Path, default=Path('target/debug/examples/validate_gameplay'))
    args = parser.parse_args()
    run(args.output.resolve(), args.binary.resolve(), args.validator.resolve())
