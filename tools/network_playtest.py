#!/usr/bin/env python3
"""Managed local Classic network playtest: one real client per enabled test player."""
import argparse
from contextlib import ExitStack
from dataclasses import asdict, dataclass
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import tempfile
import time

from launch_environment import client_environment
from network_profile import Profile
from udp_impairment import Relay

MAX_SCENARIO_BYTES = 16 * 1024


@dataclass(frozen=True)
class Player:
    client_id: int
    feet: tuple[float, float, float]
    network: Profile
    controller: Path | None = None


@dataclass(frozen=True)
class Scenario:
    identity: tuple[int, ...]
    players: tuple[Player, ...]

    @classmethod
    def load(cls, path):
        with Path(path).open('rb') as source:
            data = source.read(MAX_SCENARIO_BYTES + 1)
        if len(data) > MAX_SCENARIO_BYTES:
            raise ValueError('Test scenario exceeds 16 KiB')
        document = json.loads(data)
        if (not isinstance(document, dict) or set(document) != {'schema', 'world_identity', 'players'}
                or type(document['schema']) is not int or document['schema'] != 1):
            raise ValueError('Expected test scenario schema 1')
        identity = document['world_identity']
        if (not isinstance(identity, list) or len(identity) != 32
                or any(type(b) is not int or not 0 <= b <= 255 for b in identity)):
            raise ValueError('Expected 32-byte authored world identity')
        entries = document['players']
        if not isinstance(entries, list) or not 1 <= len(entries) <= 2:
            raise ValueError('A test scenario supports one or two players')
        players, ids = [], set()
        for entry in entries:
            if (not isinstance(entry, dict) or set(entry) - {'client_id', 'feet', 'enabled', 'network', 'controller'}
                    or not {'client_id', 'feet'} <= set(entry)):
                raise ValueError('Unsupported test player fields')
            client_id, feet = entry['client_id'], entry['feet']
            if type(client_id) is not int or client_id not in (1, 2) or client_id in ids:
                raise ValueError('Test player IDs must be distinct 1 or 2')
            ids.add(client_id)
            if (not isinstance(feet, list) or len(feet) != 3
                    or any(type(v) not in (int, float) or (type(v) is float and not math.isfinite(v)) for v in feet)
                    or not all(0 <= feet[i] < 16000 for i in (0, 2)) or abs(feet[1]) > 2048):
                raise ValueError('Expected finite XYZ feet in the supported authored world domain')
            enabled = entry.get('enabled', True)
            if type(enabled) is not bool:
                raise ValueError('Test player enabled must be a boolean')
            profile = entry.get('network', {})
            if not isinstance(profile, dict) or set(profile) - {'delay_ms', 'jitter_ms', 'loss_percent', 'seed', 'outages'}:
                raise ValueError('Unsupported network profile fields')
            profile = dict(profile)
            if 'outages' in profile:
                windows = profile['outages']
                if not isinstance(windows, list) or any(not isinstance(window, list) for window in windows):
                    raise ValueError('Outages must be arrays of [start_ms, duration_ms]')
                profile['outages'] = tuple(tuple(window) for window in windows)
            network = Profile(**profile)
            controller = entry.get('controller')
            if controller is not None:
                if not isinstance(controller, str) or not controller or len(controller) > 1024:
                    raise ValueError('Controller must be a script path or null for manual input')
                controller = (Path(path).resolve().parent / controller).resolve()
                if enabled and (not controller.is_file() or controller.stat().st_size > 32 * 1024):
                    raise ValueError('Controller script is missing or exceeds 32 KiB')
            if enabled:
                players.append(Player(client_id, tuple(feet), network, controller))
        if not players:
            raise ValueError('Enable at least one test player')
        return cls(tuple(identity), tuple(sorted(players, key=lambda player: player.client_id)))

    def server_profile(self):
        return dict(schema=1, world_identity=self.identity,
                    players=[dict(client_id=p.client_id, feet=p.feet) for p in self.players])


def stop(process, server=False):
    if process.poll() is None:
        if server and os.name != 'nt':
            process.send_signal(signal.SIGINT)
        else:
            process.terminate()
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def run(scenario_path, world, players, binaries, output, *, duration=None, window_size='1280x720'):
    scenario = Scenario.load(scenario_path)
    if duration is not None and (not math.isfinite(duration) or not 0 < duration <= 3600):
        raise ValueError('Duration must be positive and at most one hour')
    size = re.fullmatch(r'(\d+)x(\d+)', window_size)
    if not size or not (640 <= int(size[1]) <= 7680 and 480 <= int(size[2]) <= 4320):
        raise ValueError('Window size must be within 640x480..7680x4320')
    world, players, binaries, output = (Path(p).resolve() for p in (world, players, binaries, output))
    suffix = '.exe' if os.name == 'nt' else ''
    server_binary, client_binary = (binaries / (name + suffix) for name in ('theia-server', 'theia-client'))
    for path in (server_binary, client_binary, players / 'character.glb', world / 'authored.world.json'):
        if not path.is_file():
            raise ValueError(f'Missing test dependency: {path}')
    if output.exists():
        raise ValueError('Choose a fresh playtest output directory')
    environment = client_environment(os.environ)
    # Ordinary manual controls until a controller is explicitly assigned.
    for name in list(environment):
        if name.startswith('THEIA_GRAYBOX_AUTOPLAY') or name == 'THEIA_GRAYBOX_CONTINENT':
            del environment[name]
    scene = ['--authored-world', str(world), '--prediction', '--classic']
    checked = subprocess.run([str(client_binary), *scene, '--configuration-report'],
                             env=environment, capture_output=True, text=True, timeout=30)
    if checked.returncode:
        raise ValueError(f'Client content validation failed: {checked.stderr.strip()}')
    start = checked.stdout.find('{')
    configuration = json.loads(checked.stdout[start:]) if start >= 0 else {}
    if configuration.get('classic_source') != list(scenario.identity):
        raise ValueError('Test scenario belongs to a different authored world')
    for controller in {p.controller for p in scenario.players if p.controller is not None}:
        checked_script = subprocess.run([str(client_binary), *scene, '--test-controller', str(controller), '--configuration-report'],
                                        env=environment, capture_output=True, text=True, timeout=30)
        if checked_script.returncode:
            raise ValueError(f'Invalid test controller: {checked_script.stderr.strip()}')
    output.mkdir(parents=True, exist_ok=False)
    (output / 'server-players.json').write_text(json.dumps(scenario.server_profile(), indent=2) + '\n')
    (output / 'client-configuration.json').write_text(json.dumps(configuration, indent=2) + '\n')
    processes, relays = [], []
    reason = 'startup failure'
    # Editor Stop writes this file; Ctrl-C and CLI SIGTERM also clean up children.
    stop_file = output / 'stop'
    cleanup_errors = []
    def cleanup(action, *args, **kwargs):
        try:
            action(*args, **kwargs)
        except Exception as error:
            cleanup_errors.append(str(error))
    try:
        with ExitStack() as files:
            tickets = files.enter_context(tempfile.TemporaryDirectory(prefix='tickets-', dir=output))
            for player in scenario.players:
                relays.append(Relay(player.network))
                files.callback(cleanup, relays[-1].close)
            endpoints = ','.join(f'{r.address[0]}:{r.address[1]}' for r in relays)
            server_log = output / 'server.log'
            server = subprocess.Popen([str(server_binary), '127.0.0.1:0', str(output / 'unused.der'),
                                       *scene, '--renet-tickets', tickets, '--test-players', str(output / 'server-players.json'),
                                       '--dev-relays', endpoints], env=environment,
                                      stdout=files.enter_context(server_log.open('w')), stderr=subprocess.STDOUT)
            processes.append(server)
            files.callback(cleanup, stop, server, server=True)
            deadline = time.monotonic() + 30
            while True:
                if stop_file.exists():
                    reason = 'stop requested during startup'
                    return
                if server.poll() is not None:
                    raise RuntimeError(f'Test server exited; see {server_log}')
                found = re.search(r'listening=(127\.0\.0\.1):(\d+) local_tickets=', server_log.read_text())
                if found:
                    backend = (found[1], int(found[2]))
                    break
                if time.monotonic() >= deadline:
                    raise TimeoutError('Test server startup timed out')
                time.sleep(.02)
            for player, relay in zip(scenario.players, relays):
                relay.start(backend)
                processes.append(subprocess.Popen([str(client_binary), *scene, '--window-size', window_size,
                    '--player-assets', str(players), *(['--test-controller', str(player.controller)] if player.controller else []), '--renet-connect', f'{relay.address[0]}:{relay.address[1]}',
                    str(Path(tickets) / f'client-{player.client_id}.token')], env=environment,
                    stdout=files.enter_context((output / f'client-{player.client_id}.log').open('w')), stderr=subprocess.STDOUT))
                files.callback(cleanup, stop, processes[-1])
            started = time.monotonic()
            print(f'Launching network playtest: {len(scenario.players)} clients; write {stop_file} or press Ctrl-C to stop.', flush=True)
            while True:
                exited = [p for p in processes if p.poll() is not None]
                if exited:
                    if any(p.returncode != 0 for p in exited):
                        raise RuntimeError(f'A playtest process failed; see {output}')
                    reason = 'a playtest window closed'
                    break
                if stop_file.exists():
                    reason = 'stop requested'
                    break
                if duration is not None and time.monotonic() - started >= duration:
                    reason = 'duration reached'
                    break
                if any(r.error is not None for r in relays):
                    raise RuntimeError('A test connection relay failed')
                time.sleep(.05)
    except KeyboardInterrupt:
        reason = 'interrupted'
    except Exception:
        reason = 'failed'
        raise
    finally:
        summary = dict(reason=reason, cleanup_errors=cleanup_errors, clients=[dict(client_id=p.client_id, network=asdict(p.network),
                       added_base_rtt_ms=p.network.added_rtt_ms, controller=str(p.controller) if p.controller else None, relay=r.stats)
                       for p, r in zip(scenario.players, relays)],
                       scope='Local encrypted development tickets; optional bounded movement controllers; no account persistence or scripted combat')
        (output / 'session.json').write_text(json.dumps(summary, indent=2) + '\n')
        if cleanup_errors:
            raise RuntimeError('Playtest cleanup failed: ' + '; '.join(cleanup_errors))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scenario', type=Path, required=True)
    parser.add_argument('--world', type=Path, required=True)
    parser.add_argument('--players', type=Path, required=True)
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--duration', type=float)
    parser.add_argument('--window-size', default='1280x720')
    args = parser.parse_args()
    def terminate(_signal, _frame):
        raise KeyboardInterrupt
    signal.signal(signal.SIGTERM, terminate)
    run(args.scenario, args.world, args.players, args.binaries, args.output,
        duration=args.duration, window_size=args.window_size)


if __name__ == '__main__':
    main()
