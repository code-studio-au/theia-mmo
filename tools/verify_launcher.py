#!/usr/bin/env python3
"""Qualify Linux launcher child/ticket cleanup on the caller's native display.

Runs real application binaries. This uses /proc for ownership evidence and is
not Windows execution qualification. Run with no other development launcher.
"""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time


def run(output, binaries, world, players, *, use_default_world=False, chunk_world=False):
    if sys.platform != 'linux':
        raise RuntimeError('This process-ownership qualifier requires Linux /proc')
    output.mkdir(parents=True, exist_ok=True)
    session = Path('build/development-session')
    if use_default_world and chunk_world:
        raise ValueError('Select default monolithic world or explicit chunk world')
    if use_default_world:
        expected_packages = []
    elif chunk_world:
        initial = json.loads((world / 'world.session.json').read_text())['initial_chunks']
        expected_packages = [json.loads((world / f'chunk_{x}_{z}' / 'world.manifest.json').read_text()) for x,z in initial]
    else:
        expected_packages = [json.loads((world / 'world.manifest.json').read_text())]
    results = []
    for mode in ['interrupt', 'child_failure']:
        before = set(session.glob('renet-*'))
        owned = []
        with (output / f'{mode}.log').open('w') as log:
            launcher = subprocess.Popen([
                sys.executable, 'tools/play.py', '--clients', '2',
                '--binaries', str(binaries), *([] if use_default_world else ['--chunk-world' if chunk_world else '--world', str(world)]), '--players', str(players),
            ], stdout=log, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + 60
                while time.monotonic() < deadline:
                    assert launcher.poll() is None, 'Launcher exited before client admission'
                    children = Path(f'/proc/{launcher.pid}/task/{launcher.pid}/children')
                    owned = [int(pid) for pid in children.read_text().split()]
                    server_log = session / 'server.log'
                    text = server_log.read_text() if server_log.exists() else ''
                    graybox_ready = not use_default_world or ('"graybox":' in (output/f'{mode}.log').read_text() and text.count('chunks=9') >= 2)
                    if len(owned) == 3 and 'local_tickets=' in text and text.count('player_join ') == 2 and graybox_ready:
                        break
                    time.sleep(.05)
                else:
                    raise AssertionError('Launcher admission timed out')
                assert len(set(owned)) == 3
                if use_default_world:
                    # Allow commands after the two initial full-detail ready reports.
                    time.sleep(.5)
                if use_default_world:
                    for pid in owned:
                        argv = Path(f'/proc/{pid}/cmdline').read_bytes().split(b'\0')
                        assert b'--prediction' in argv, 'Default launcher omitted the qualified prediction profile'
                        assert b'--saved-terrain' in argv, 'Default launcher did not load saved terrain'
                for expected_package in expected_packages:
                    expected = f"world_package content_id={expected_package['content_id']} revision={expected_package['revision']} collision_verified=true"
                    assert expected in text, 'Launcher selected the wrong content package/revision'
                ticket_directories = set(session.glob('renet-*')) - before
                assert len(ticket_directories) == 1
                if mode == 'interrupt':
                    launcher.send_signal(signal.SIGINT)
                else:
                    # Identify the owned client explicitly; never signal a name
                    # match or an unrelated process from another session.
                    clients = [pid for pid in owned if Path(f'/proc/{pid}/comm').read_text().strip() == 'theia-client']
                    assert len(clients) == 2
                    os.kill(clients[0], signal.SIGTERM)
                code = launcher.wait(timeout=20)
                assert (code == 0) if mode == 'interrupt' else (code != 0)
                if use_default_world:
                    summary = next(json.loads(line.removeprefix('transport_summary ')) for line in server_log.read_text().splitlines() if line.startswith('transport_summary '))
                    assert summary['queued_commands'] > 0, 'Default profile never queued actual commands'
                assert all(not Path(f'/proc/{pid}').exists() for pid in owned), 'Owned game child survived'
                assert not any(p.exists() for p in ticket_directories), 'Temporary tickets survived'
                results.append({'mode': mode, 'launcher_exit': code, 'owned_children': len(owned),
                                'children_removed': True, 'tickets_removed': True, 'transport': 'renet',
                                'default_world': use_default_world, 'chunk_world': chunk_world, 'prediction': use_default_world, 'saved_terrain': use_default_world,
                                **({'content_id': expected_packages[0]['content_id'], 'revision': expected_packages[0]['revision']} if expected_packages and not chunk_world else {}),
                                'packages': [{'content_id': p['content_id'], 'revision': p['revision']} for p in expected_packages]})
            finally:
                if launcher.poll() is None:
                    launcher.send_signal(signal.SIGINT)
                    launcher.wait(timeout=20)
                # These IDs came from the launcher we own. Clean up after a
                # failed assertion without touching other application sessions.
                for pid in owned:
                    if Path(f'/proc/{pid}').exists():
                        os.kill(pid, signal.SIGTERM)
    report = {'schema': 1, 'passed': True, 'platform': sys.platform, 'results': results}
    (output / 'verification.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binaries', type=Path, default=Path('target/release'))
    parser.add_argument('--world', type=Path, default=Path('build/development-world-town-v3'))
    parser.add_argument('--players', type=Path)
    parser.add_argument('--use-default-world', action='store_true', help='Verify the default continental graybox launcher and POLYGON player')
    parser.add_argument('--chunk-world', action='store_true', help='Treat --world as an opt-in chunk index/session directory')
    args = parser.parse_args()
    players = args.players or Path('build/synty-character-canonical-stage' if args.use_default_world else 'build/reference-scene')
    run(args.output.resolve(), args.binaries.resolve(), args.world.resolve(), players.resolve(),
        use_default_world=args.use_default_world, chunk_world=args.chunk_world)
