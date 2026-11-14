#!/usr/bin/env python3
"""Open a manual editor/account playtest with disposable local services (Linux).

The editor uses a private copy of the zone and frozen executables. Account progress
survives Stop/Start while this editor remains open. Closing the editor stops its
owned services; a new invocation creates a fresh synthetic account/database.
No movement, combat, character selection or editor input is automated.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shlex
import shutil
import signal
import subprocess
import sys
import time
import urllib.parse

from private_identity import Identity
from private_postgres import Cluster
from play_local import stop_launcher

ROOT = Path(__file__).resolve().parents[1]
BINARIES = ('theia-zone-editor', 'theia-client', 'theia-server', 'theia-nav-bake',
            'examples/validate_gameplay')


def prepare(args, root=ROOT):
    if sys.platform != 'linux':
        raise ValueError('The disposable editor service launcher currently requires Linux')
    if not 60 <= args.session_seconds <= 43200:
        raise ValueError('Session lifetime must be 60..43200 seconds')
    if args.access_path is not None and not 0 < args.access_path <= 2**64-1:
        raise ValueError('Access path must be a positive u64 road ID')
    output = args.output.resolve()
    if not output.is_relative_to((root/'build').resolve()) or output.exists():
        raise ValueError('Choose a new private output directory under build/')
    sources = {name: args.binaries.resolve()/name for name in BINARIES}
    paths = [*sources.values(), args.zone, args.players/'character.glb',
             args.scenery_library, args.creature_definition, args.creature_asset,
             args.gameplay_template]
    for path in paths:
        if not path.is_file():
            raise ValueError(f'Missing prepared editor playtest file: {path}')
    # Check the document before creating any output or services. Runtime/editor
    # validation still owns the full schema, static collision and content checks.
    zone = json.loads(args.zone.read_text())
    if any(player.get('enabled') for player in zone.get('test_players', [])):
        raise ValueError('Manual account playtest requires a zone without enabled Test Players')
    output.mkdir(mode=0o700, parents=True)
    zone_path = output/'draft.zone.json'
    zone_path.write_bytes(args.zone.read_bytes())
    hashes = {}
    for name, source in sources.items():
        target = output/'binaries'/name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
        with target.open('rb') as stream:
            hashes[name] = hashlib.file_digest(stream, 'sha256').hexdigest()
    command = [str(output/'binaries/theia-zone-editor'), '--zone', str(zone_path),
               '--scenery-library', str(args.scenery_library.resolve()),
               '--player-assets', str(args.players.resolve()),
               '--creature-definition', str(args.creature_definition.resolve()),
               '--creature-asset', str(args.creature_asset.resolve()),
               '--gameplay-template', str(args.gameplay_template.resolve()),
               '--report', str(output/'editor-state.json')]
    if args.access_path is not None:
        command += ['--population-access-path', str(args.access_path)]
    (output/'build.json').write_text(json.dumps(dict(
        binary_sha256=hashes, zone_source=str(args.zone.resolve()),
        zone_sha256=hashlib.sha256(zone_path.read_bytes()).hexdigest(),
        session_seconds=args.session_seconds, manual_input=True,
        fresh_synthetic_account=True, access_path=args.access_path,
        assets='Existing local scenery, player and creature assets; export freezes gameplay'),
        indent=2)+'\n')
    return output, command


def environment(source, identity, cluster, output, seconds, root=ROOT):
    env = {k: v for k, v in source.items()
           if not k.upper().startswith(('THEIA_', 'PG')) and k.upper() != 'DATABASE_URL'}
    env.update(THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
               THEIA_OIDC_ISSUER=identity.issuer, THEIA_OIDC_CLIENT_ID=identity.client_id,
               THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
               THEIA_ACCOUNT_SESSION_SECONDS=str(seconds),
               THEIA_TEST_BROWSER_ISSUER=identity.issuer,
               THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
               THEIA_TEST_BROWSER_USER=identity.users[0][0],
               THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
               THEIA_PARTICLE_EDITOR_DATA_DIR=str(output/'particle-data'),
               BROWSER=shlex.join([sys.executable, str(root/'tools/qualify_account_launcher.py'),
                                   '--owned-browser', '%s']))
    env.setdefault('DISPLAY', ':0')
    native = str(root/'build/native-sdk/usr/lib/x86_64-linux-gnu')
    env['LD_LIBRARY_PATH'] = native+':'+env.get('LD_LIBRARY_PATH', '')
    return env


def run(args):
    os.umask(0o077)
    output, command = prepare(args)
    editor = None
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
                env = environment(os.environ, identity, cluster, output, args.session_seconds)
                with (output/'editor.log').open('w') as log:
                    editor = subprocess.Popen(command, cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                                              stdout=log, stderr=subprocess.STDOUT,
                                              start_new_session=True)
                    (output/'status.json').write_text(json.dumps(dict(
                        state='running', supervisor_pid=os.getpid(), editor_pid=editor.pid,
                        user_controlled=True))+'\n')
                    print(f'Manual editor starting; private draft and logs: {output}', flush=True)
                    while editor.poll() is None and not stopped:
                        time.sleep(.25)
                    if editor.poll() is not None and editor.returncode:
                        raise RuntimeError('Editor exited unsuccessfully; inspect its private editor.log')
            finally:
                finish_editor(editor)
    finally:
        (output/'status.json').write_text(json.dumps(dict(state='stopped'))+'\n')


def finish_editor(editor):
    """Let the editor's owner-pipe EOF drain its children before stopping SQL.

The window can exit before its launcher finishes. They share our explicitly
created process group, so parent exit alone is not a completed shutdown.
"""
    if editor is None:
        return
    stop_launcher(editor)
    for deadline_seconds, sig in [(30, signal.SIGINT), (20, signal.SIGTERM), (10, signal.SIGKILL)]:
        deadline = time.monotonic()+deadline_seconds
        while time.monotonic() < deadline:
            try:
                os.killpg(editor.pid, 0)
            except ProcessLookupError:
                return
            time.sleep(.1)
        try:
            os.killpg(editor.pid, sig)
        except ProcessLookupError:
            return


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('zone', 'scenery-library', 'players', 'creature-definition',
                 'creature-asset', 'gameplay-template', 'output'):
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--binaries', type=Path, default=ROOT/'target/debug')
    parser.add_argument('--access-path', type=int, help='Prefill the checked access-road ID')
    parser.add_argument('--session-seconds', type=int, default=3600)
    run(parser.parse_args())


if __name__ == '__main__':
    main()
