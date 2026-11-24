#!/usr/bin/env python3
"""Open a prepared village playtest with disposable local account services.

No build or terrain edits. Closing the game stops this launcher's services.
"""
import argparse
from contextlib import ExitStack
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

from edit_local import environment
from launch_environment import client_environment
from play_local import stop_launcher
from private_identity import Identity
from private_postgres import Cluster

ROOT = Path(__file__).resolve().parents[1]


def run(args):
    os.umask(0o077)
    package = args.package.resolve()
    output = args.output.resolve()
    binaries = args.binaries.resolve()
    manifest = package/'world/gameplay/gameplay.manifest.json'
    for path in (manifest, package/'population.json', package/'creature.glb',
                 binaries/'theia-client', binaries/'theia-server',
                 binaries/'examples/validate_gameplay', args.players/'character.glb'):
        if not path.is_file():
            raise ValueError(f'Missing prepared playtest input: {path}')
    if not output.is_relative_to(ROOT/'build') or output.exists():
        raise ValueError('Use a new private output directory under build/')
    output.mkdir(parents=True, mode=0o700)
    stopped = False

    def stop(*_):
        nonlocal stopped
        stopped = True

    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    launcher = None
    with ExitStack() as services:
        print('Starting isolated local account services...', flush=True)
        identity = services.enter_context(Identity(output/'provider', ROOT/'build/identity-sdk',
                                                   token_seconds=43200, interactive=True))
        cluster = services.enter_context(Cluster(output/'cluster', ROOT/'build/postgres-sdk',
                                                 isolate_process=True))
        env = environment(client_environment(os.environ), identity, cluster, output, 43200)
        env['THEIA_WORLD_INPUT_OS_QUALIFY'] = str(output)
        if args.map_directory:
            env['THEIA_ZONE_MAP_DIRECTORY'] = str(args.map_directory.resolve())
        command = [sys.executable, str(ROOT/'tools/account_population_playtest.py'),
                   '--package', str(package), '--players', str(args.players.resolve()),
                   '--binaries', str(binaries), '--output', str(output/'owned'),
                   '--gameplay-manifest', str(manifest), '--window-size', args.window_size,
                   '--fantasy-ui', str(args.fantasy_ui.resolve())]
        try:
            with (output/'launcher.log').open('w') as log:
                launcher = subprocess.Popen(command, cwd=ROOT, env=env, stdin=subprocess.PIPE,
                                            stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
                runtime = output/'owned/runtime'
                deadline = time.monotonic()+180
                selected = False
                (output/'status.json').write_text(json.dumps({'supervisor_pid':os.getpid(),
                    'launcher_pid':launcher.pid, 'state':'starting'})+'\n')
                while launcher.poll() is None and not stopped:
                    choices = runtime/'characters.json'
                    if not selected and choices.exists():
                        value = json.loads(choices.read_text())
                        if value['characters'] or not value['can_create']:
                            raise RuntimeError('Expected the fresh isolated playtest account')
                        pending = runtime/'selection.pending'
                        pending.write_text(json.dumps({'schema':1, 'create_name':'Millhaven'}))
                        pending.replace(runtime/'selection.json')
                        selected = True
                        print('Starting Millhaven with village NPCs, wolves, minimap and menus...', flush=True)
                    if not selected and time.monotonic()>deadline:
                        raise RuntimeError('Local account startup timed out; see launcher.log')
                    time.sleep(.25)
                if launcher.returncode:
                    raise RuntimeError('Playtest exited unsuccessfully; see launcher.log')
        finally:
            if launcher is not None:
                stop_launcher(launcher)
            (output/'status.json').write_text('{"state":"stopped"}\n')


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    for name in ('package', 'players', 'output', 'fantasy-ui'):
        p.add_argument('--'+name, type=Path, required=True)
    p.add_argument('--binaries', type=Path, default=ROOT/'target/release',
                   help='Prepared runtime executables (default: target/release)')
    p.add_argument('--map-directory', type=Path)
    p.add_argument('--window-size', default='3440x1440')
    run(p.parse_args())
