#!/usr/bin/env python3
"""Launch the continental graybox or an explicit legacy map, with child cleanup."""
import argparse
import os
from pathlib import Path
import re
import signal
import subprocess
import time
import tempfile


def play(world, players, clients, binaries, tick_hz=30, transport="renet", chunk_world=None, camera_collision=None, graybox=False, dev_speed=1, dev_spawn="8000,8000", dev_tour=False, prediction=False):
    if prediction and not graybox:
        raise ValueError("Prediction requires the graybox world")
    if not graybox and (dev_speed != 1 or dev_spawn != "8000,8000" or dev_tour):
        raise ValueError('Developer travel options require the graybox world')
    if (chunk_world or graybox) and transport != 'renet':
        raise ValueError('Chunk worlds require authenticated Renet readiness')
    suffix = '.exe' if os.name == 'nt' else ''
    server_binary = binaries / ('theia-server' + suffix)
    client_binary = binaries / ('theia-client' + suffix)
    required = [] if graybox else ([chunk_world / 'world.chunks.json', chunk_world / 'world.session.json'] if chunk_world else
                [world / 'world.manifest.json', world / 'terrain.json', world / 'terrain.glb'])
    scene = ['--graybox-world', *(['--prediction'] if prediction else [])] if graybox else (['--chunk-world', str(chunk_world)] if chunk_world else ['--terrain-world', str(world)])
    server_options = ['--dev-speed', str(dev_speed), '--dev-spawn', dev_spawn, *(['--dev-tour'] if dev_tour else [])] if graybox else []
    for path in [*required, players / ('character.glb' if graybox else 'human.glb'), server_binary, client_binary]:
        if not path.is_file():
            raise ValueError(f'Missing {path}; build with cargo build --release --locked --workspace; see docs/DEVELOPMENT_WORLD.md for asset preparation or select --binaries explicitly')
    session = Path('build/development-session')
    session.mkdir(parents=True, exist_ok=True)
    certificate = session / 'server-cert.der'
    certificate.unlink(missing_ok=True)
    tickets = tempfile.TemporaryDirectory(prefix="renet-", dir=session) if transport == "renet" else None
    processes = []
    try:
        with (session / 'server.log').open('w') as log:
            server = subprocess.Popen([str(server_binary), '127.0.0.1:0', str(certificate),
                                       *scene, *server_options, '--tick-hz', str(tick_hz),
                                       *(['--renet-tickets', tickets.name] if tickets else [])],
                                      stdout=log, stderr=subprocess.STDOUT)
            processes.append(server)
            address = None
            deadline = time.monotonic() + 20
            while time.monotonic() < deadline:
                if server.poll() is not None:
                    raise RuntimeError(f'Server failed; see {session / "server.log"}')
                match = re.search(r'listening=(\S+) (?:public_certificate|local_tickets)=', (session / 'server.log').read_text())
                if match and (tickets or certificate.exists()):
                    address = match[1]
                    break
                time.sleep(.05)
            if address is None:
                raise RuntimeError('Server readiness timed out')
            for index in range(clients):
                processes.append(subprocess.Popen([str(client_binary),
                    *(['--camera-collision', camera_collision] if camera_collision else []), *scene,
                    '--player-assets', str(players),
                    '--renet-connect' if tickets else '--connect', address,
                    str(Path(tickets.name) / f'client-{index+1}.token') if tickets else str(certificate)]))
            print('Theia development map: WASD, Shift to run, Space to jump, RMB orbit, wheel zoom, Escape to quit.', flush=True)
            while all(p.poll() is None for p in processes):
                time.sleep(.2)
            if any(p.poll() not in (None, 0) for p in processes):
                raise RuntimeError('A game process failed; inspect its output and build/development-session/server.log')
    except KeyboardInterrupt:
        pass
    finally:
        for process in reversed(processes):
            if process.poll() is None:
                # Let the POSIX server finish its simulation/transport summary.
                # Windows retains terminate until native console shutdown is qualified.
                if process is processes[0] and os.name != 'nt':
                    process.send_signal(signal.SIGINT)
                else:
                    process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()

        if tickets:
            tickets.cleanup()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    worlds = parser.add_mutually_exclusive_group()
    worlds.add_argument('--world', type=Path, help='Explicit legacy monolithic world')
    worlds.add_argument('--graybox-world', action='store_true', help='256 km² procedural development world (default)')
    worlds.add_argument('--chunk-world', type=Path, help='Opt-in prepared chunk index/session; requires Renet')
    parser.add_argument('--players', type=Path, help='Player assets; defaults to POLYGON for graybox')
    parser.add_argument('--prediction', action=argparse.BooleanOptionalAction, default=None, help='Movement prediction (default for graybox; --no-prediction retains server-pose interpolation)')
    parser.add_argument('--dev-speed', type=int, choices=[1, 8, 32], default=1)
    parser.add_argument('--dev-spawn', default='8000,8000', help='Graybox server spawn x,z in metres')
    parser.add_argument('--dev-tour', action='store_true', help='Server-owned teleport circuit for streaming qualification')
    parser.add_argument('--clients', type=int, choices=range(1, 9), default=1)
    parser.add_argument('--binaries', type=Path, default=Path('target/release'),
                        help='Optimized interactive binaries; use target/debug explicitly for debugging')
    parser.add_argument('--tick-hz', type=int, choices=[20, 30, 60], default=30)
    parser.add_argument('--transport', choices=['quinn', 'renet'], default='renet')
    parser.add_argument('--camera-collision', choices=['world', 'visual'],
                        help='Override authored world camera clearance or legacy visual-mesh refinement')
    args = parser.parse_args()
    graybox = args.graybox_world or (args.world is None and args.chunk_world is None)
    prediction = graybox if args.prediction is None else args.prediction
    players = args.players or Path('build/synty-character-canonical-stage' if graybox else 'build/reference-scene')
    play(args.world.resolve() if args.world else None, players.resolve(), args.clients, args.binaries.resolve(), args.tick_hz, args.transport,
         args.chunk_world.resolve() if args.chunk_world else None, args.camera_collision, graybox, args.dev_speed, args.dev_spawn, args.dev_tour, prediction)
