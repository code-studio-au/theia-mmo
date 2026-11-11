#!/usr/bin/env python3
"""Actual client CLI preference checks. No App, window, renderer, network or service.

Requires built client/validator, normal development content, local HUD sources and
baked terrain/navigation. Files and reports stay under the caller's ignored build/.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
from gameplay_bundle import snapshot
from qualify_reward_server_config import ROLE_FLAGS

ROOT = Path(__file__).resolve().parents[1]


def run(args):
    output = args.output.resolve()
    if not output.is_relative_to((ROOT/'build').resolve()) or output.exists():
        raise ValueError('Choose a new ignored output directory under build/')
    output.mkdir(parents=True, mode=0o700)
    environment = {k: v for k, v in os.environ.items()
                   if k.upper() not in ('DISPLAY', 'WAYLAND_DISPLAY', 'XDG_RUNTIME_DIR', 'DATABASE_URL')
                   and not k.upper().startswith(('THEIA_', 'PG'))}
    native = ROOT/'build/native-sdk/usr/lib/x86_64-linux-gnu'
    environment['LD_LIBRARY_PATH'] = str(native)+':'+environment.get('LD_LIBRARY_PATH', '')
    preference = output/'interface.json'
    cases = []
    with snapshot(args.manifest, args.validator, output/'snapshot') as (paths, bundle):
        if bundle['manifest_schema'] != 21 or set(paths) != set(ROLE_FLAGS):
            raise ValueError('This check requires the normal schema-21 development bundle')
        command = [str(args.client.resolve()), '--configuration-report',
                   '--saved-terrain', str(args.terrain.resolve()), '--navigation', str(args.navigation.resolve()),
                   '--prediction', '--classic', '--quest-journal', '--quest-items', '--quest-loot',
                   '--creature-loot', '--public-creature-attacks', '--public-avatar-life', '--guide-position', '8004,8000']
        for role, flag in ROLE_FLAGS.items():
            command += [flag, str(paths[role])]
        skin = ['--fantasy-ui', str(args.fantasy_ui.resolve())]
        baseline_world = None

        def check(name, extra, expected=None):
            nonlocal baseline_world
            before = preference.read_bytes() if preference.exists() else None
            child = subprocess.run(command+extra, env=environment, capture_output=True, text=True, timeout=30)
            (output/f'{name}.stderr.log').write_text(child.stderr)
            after = preference.read_bytes() if preference.exists() else None
            assert before == after, f'{name}: CLI preflight must not write preferences'
            if expected is None:
                assert child.returncode != 0, f'{name}: invalid preference accepted'
                assert not child.stdout.strip(), f'{name}: refused case emitted a success report'
                cases.append(dict(name=name, refused=True, file_unchanged=True))
            else:
                assert child.returncode == 0, f'{name}: preflight failed; inspect stderr log'
                report = json.loads(child.stdout)
                expected = dict({'world_effects_visible': True}, **expected)
                assert not any(report[k] for k in ('renderer_started', 'network_started', 'saved_state_admitted'))
                assert report['interface'] == expected, name
                if baseline_world is None:
                    baseline_world = report['world']
                assert report['world'] == baseline_world, 'UI preferences changed gameplay identity'
                (output/f'{name}.json').write_text(child.stdout)
                cases.append(dict(name=name, refused=False, file_unchanged=True, interface=report['interface']))
            print(name, 'PASS', flush=True)

        check('baseline', skin, dict(enabled=True, preferred_scale=1., saved_preferences=False))
        flag = ['--ui-preferences', str(preference)]
        check('missing', skin+flag, dict(enabled=True, preferred_scale=1., saved_preferences=True))
        preference.write_text('{"schema":1,"ui_scale":1.25}')
        check('saved', skin+flag, dict(enabled=True, preferred_scale=1.25, saved_preferences=True))
        check('override', skin+flag+['--ui-scale', '0.75'], dict(enabled=True, preferred_scale=.75, saved_preferences=True))
        preference.write_text('{"schema":1,"ui_scale":1.25,"world_effects":false}')
        check('effects-hidden', skin+flag, dict(enabled=True, preferred_scale=1.25, saved_preferences=True, world_effects_visible=False))
        preference.write_text('{"schema":1,"ui_scale":1.25,"world_effects":true}')
        check('effects-visible', skin+flag, dict(enabled=True, preferred_scale=1.25, saved_preferences=True))
        check('effects-cli-override', skin+flag+['--hide-world-particles'], dict(enabled=True, preferred_scale=1.25, saved_preferences=True, world_effects_visible=False))
        preference.write_text('{"schema":1,"ui_scale":1.25,"world_effects":"false"}')
        check('invalid-effects', skin+flag)
        check('duplicate', skin+flag+flag)
        check('without-skin', flag)
        preference.write_text('{"schema":2,"ui_scale":1.25}')
        check('unknown-version', skin+flag)
        preference.write_text('{"schema":1,"ui_scale":2}')
        check('out-of-range', skin+flag)
        preference.write_bytes(b' '*4097)
        check('oversized', skin+flag)
        preference.unlink()
    with args.client.open('rb') as stream:
        client_hash = hashlib.file_digest(stream, 'sha256').hexdigest()
    (output/'verification.json').write_text(json.dumps(dict(
        passed=True, cases=cases, client_sha256=client_hash,
        world=baseline_world, desktop_used=False, services_started=False,
        scope='Actual client parser and configuration identity; no UI input, rendering, save task or gameplay qualification'), indent=2)+'\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--client', type=Path, default=ROOT/'target/debug/theia-client')
    parser.add_argument('--validator', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, default=ROOT/'content/development-gameplay-classic-item-rewards.json')
    parser.add_argument('--terrain', type=Path, default=ROOT/'build/graybox-terrain-v1')
    parser.add_argument('--navigation', type=Path, default=ROOT/'build/navigation-terrain-v1')
    parser.add_argument('--fantasy-ui', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    run(parser.parse_args())


if __name__ == '__main__':
    main()
