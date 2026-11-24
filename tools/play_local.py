#!/usr/bin/env python3
"""Linux disposable authenticated playtest, isolated from subsequent repo builds.

Requires prepared binaries/art, baked terrain/navigation and owned local SDKs.
Every run creates a fresh synthetic account/database; this is not a saved-account
launcher. Close the game (or interrupt this parent) to stop its owned services.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import shlex
import signal
import subprocess
import sys
import time
import urllib.parse
from private_identity import Identity
from private_postgres import Cluster

ROOT = Path(__file__).resolve().parents[1]


def validate(args, root=ROOT):
    if sys.platform != "linux":
        raise ValueError("This owned playtest SDK launcher currently requires Linux")
    if not 60 <= args.session_seconds <= 43200:
        raise ValueError('Session lifetime must be 60..43200 seconds')
    if not re.fullmatch(r'[A-Za-z]{3,20}', args.character):
        raise ValueError('Character name must contain 3..20 letters')
    match = re.fullmatch(r'(\d+)x(\d+)', args.window_size)
    if not match or not (640 <= int(match[1]) <= 7680 and 480 <= int(match[2]) <= 4320):
        raise ValueError('Invalid playtest window size')
    if args.ui_scale is not None and not .75 <= args.ui_scale <= 1.5:
        raise ValueError('UI scale must be .75..1.5')
    output = args.output.resolve()
    if not output.is_relative_to((root/'build').resolve()) or output.exists():
        raise ValueError('Choose a new private output directory under build/')
    binaries = args.binaries.resolve()
    client = (args.client or binaries/'theia-client').resolve()
    server = (args.server or binaries/'theia-server').resolve()
    manifest = args.manifest.resolve()
    content = (root/'content').resolve()
    if not manifest.is_relative_to(content):
        raise ValueError('Choose a manifest under this repository content directory')
    for path in [client, server, binaries/'validate_gameplay',
                 binaries/'art/player/character.glb', binaries/'art/wolf/wolf.glb',
                 manifest, root/'build/graybox-terrain-v1/terrain.manifest.json',
                 root/'build/navigation-terrain-v1/navigation.manifest.json']:
        if not path.is_file():
            raise ValueError(f'Missing prepared playtest file: {path}')
    if not args.fantasy_ui.is_dir():
        raise ValueError('Missing local Fantasy Warrior HUD directory')
    return output, binaries, client, server, manifest.relative_to(content)


def freeze(args, root=ROOT):
    output, source, client, server, manifest = validate(args, root)
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    binaries = output/'binaries'
    binaries.mkdir()
    hashes = {}
    for name, path in [('theia-client', client), ('theia-server', server),
                       ('validate_gameplay', source/'validate_gameplay')]:
        shutil.copy2(path, binaries/name)
        with (binaries/name).open('rb') as stream:
            hashes[name] = hashlib.file_digest(stream, 'sha256').hexdigest()
    shutil.copytree(source/'art', binaries/'art')
    shutil.copytree(root/'content', output/'content')
    (output/'build.json').write_text(json.dumps(dict(
        binary_sha256=hashes, manifest=str(manifest), session_seconds=args.session_seconds,
        ui_scale=args.ui_scale, ui_preferences=str(args.ui_preferences.resolve()) if args.ui_preferences else None, window=args.window_size, fresh_synthetic_account=True,
        terrain_and_navigation='shared immutable baked sources',
        ui_art='external licensed local source'), indent=2)+'\n')
    command = [sys.executable, str(root/'tools/play.py'), '--account-login', '--classic',
               '--binaries', str(binaries), '--gameplay-manifest', str(output/'content'/manifest),
               '--players', str(binaries/'art/player'), '--creature-asset', str(binaries/'art/wolf/wolf.glb'),
               '--saved-terrain', str(root/'build/graybox-terrain-v1'),
               '--navigation', str(root/'build/navigation-terrain-v1'),
               '--fantasy-ui', str(args.fantasy_ui.resolve()),
               '--window-size', args.window_size, '--dev-spawn', '8002,8002']
    if args.ui_preferences:
        command += ['--ui-preferences', str(args.ui_preferences.resolve())]
    if args.ui_scale is not None or args.ui_preferences is None:
        command += ['--ui-scale', str(args.ui_scale if args.ui_scale is not None else 1.25)]
    return output, command


def stop_launcher(launcher):
    """Owned child exits before the fixture contexts tear down their services."""
    if launcher is None or launcher.poll() is not None:
        return
    os.killpg(launcher.pid, signal.SIGINT)
    try:
        launcher.wait(timeout=20)
    except subprocess.TimeoutExpired:
        os.killpg(launcher.pid, signal.SIGTERM)
        try:
            launcher.wait(timeout=10)
        except subprocess.TimeoutExpired:
            os.killpg(launcher.pid, signal.SIGKILL)
            launcher.wait()


def run(args):
    os.umask(0o077)
    output, command = freeze(args)
    launcher = None
    stopped = False

    def stop(*_):
        nonlocal stopped
        stopped = True

    signal.signal(signal.SIGINT, stop)
    signal.signal(signal.SIGTERM, stop)
    try:
        with Identity(output/'provider', ROOT/'build/identity-sdk',
                      token_seconds=args.session_seconds, interactive=True) as identity, \
                Cluster(output/'cluster', ROOT/'build/postgres-sdk', isolate_process=True) as cluster:
            try:
                if stopped:
                    return
                env = {k: v for k, v in os.environ.items() if not k.upper().startswith(('THEIA_', 'PG')) and k.upper() != 'DATABASE_URL'}
                env.update(THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                           THEIA_OIDC_ISSUER=identity.issuer, THEIA_OIDC_CLIENT_ID=identity.client_id,
                           THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                           THEIA_ACCOUNT_SESSION_SECONDS=str(args.session_seconds),
                           THEIA_TEST_BROWSER_ISSUER=identity.issuer,
                           THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                           THEIA_TEST_BROWSER_USER=identity.users[0][0],
                           THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                           BROWSER=shlex.join([sys.executable, str(ROOT/'tools/qualify_account_launcher.py'), '--owned-browser', '%s']))
                env.setdefault('DISPLAY', ':0')
                native = str(ROOT/'build/native-sdk/usr/lib/x86_64-linux-gnu')
                env['LD_LIBRARY_PATH'] = native+':'+env.get('LD_LIBRARY_PATH', '')
                with (output/'game.log').open('w') as log:
                    launcher = subprocess.Popen(command, cwd=output, env=env, stdin=subprocess.PIPE,
                                                stdout=log, stderr=subprocess.STDOUT, text=True,
                                                start_new_session=True)
                    launcher.stdin.write(args.character+'\n')
                    launcher.stdin.flush()
                    launcher.stdin.close()
                    (output/'status.json').write_text(json.dumps(dict(
                        state='running', supervisor_pid=os.getpid(), launcher_pid=launcher.pid,
                        user_controlled=True))+'\n')
                    print(f'Local playtest starting; private logs: {output}', flush=True)
                    while launcher.poll() is None and not stopped:
                        time.sleep(.25)
                    if launcher.poll() is not None and launcher.returncode:
                        raise RuntimeError('Playtest failed; inspect its private game.log')
            finally:
                stop_launcher(launcher)
    finally:
        stop_launcher(launcher)
        (output/'status.json').write_text(json.dumps(dict(state='stopped'))+'\n')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binaries', type=Path, required=True)
    parser.add_argument('--client', type=Path)
    parser.add_argument('--server', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--fantasy-ui', type=Path, required=True)
    parser.add_argument('--manifest', type=Path, default=ROOT/'content/development-gameplay-classic-item-rewards.json')
    parser.add_argument('--session-seconds', type=int, default=3600)
    parser.add_argument('--character', default='Arden')
    parser.add_argument('--window-size', default='3440x1440')
    parser.add_argument('--ui-scale', type=float, help='Explicit launch override; default 1.25 without a preference file')
    parser.add_argument('--ui-preferences', type=Path, help='Local UI preference file shared across runs; requires a compatible current client')
    run(parser.parse_args())


if __name__ == '__main__':
    main()
