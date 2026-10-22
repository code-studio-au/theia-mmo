#!/usr/bin/env python3
"""Native Linux graybox qualification on the caller's display, using real Renet.

Only the game's window is captured. Binaries/assets and credentials remain local.
Run without competing builds or another walkthrough for useful frame-time data.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import tempfile
import time
from process_metrics import Recorder
from udp_impairment import Relay


def stop(process):
    if process.poll() is None:
        process.send_signal(signal.SIGINT)
        try:
            process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def run(output, binaries, players, *, duration=65, clients=1, tour=True, validation=True, speed=32, continent=False, prediction=False, impairment=False, hold_jump=False):
    if hold_jump and not (prediction and tour):
        raise ValueError("Held-jump reset qualification requires prediction and the teleport tour")
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    env = dict(os.environ)
    env.update(THEIA_PERFORMANCE_UNCAPPED='1', THEIA_GRAYBOX_AUTOPLAY='1', THEIA_GRAYBOX_AUTOPLAY_SPEED=str(speed), RUST_LOG='info')
    if validation:
        env.update(VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer', WGPU_VALIDATION='1')
    else:
        for key in ['VK_INSTANCE_LAYERS', 'VK_LOADER_DEBUG', 'WGPU_VALIDATION', 'WGPU_DEBUG']:
            env.pop(key, None)
    if continent:
        env["THEIA_GRAYBOX_CONTINENT"] = "1"
    if hold_jump:
        env["THEIA_GRAYBOX_HOLD_JUMP_RESET"] = "1"
    else:
        env.pop("THEIA_GRAYBOX_HOLD_JUMP_RESET", None)
    processes = []
    logs = []
    relays = [Relay() for _ in range(clients)] if impairment else []
    try:
        with tempfile.TemporaryDirectory(prefix='tickets-', dir=output) as tickets:
            server_log = (output/'server.log').open('w')
            logs.append(server_log)
            server = subprocess.Popen([str(binaries/'theia-server'), '127.0.0.1:0', str(output/'unused.der'),
                '--graybox-world', *(['--prediction'] if prediction else []), '--dev-speed', str(speed), '--dev-spawn', '10,8000' if continent else '8000,8000', '--renet-tickets', tickets,
                *(['--dev-tour'] if tour else []), *(['--dev-relays', ','.join(f'{r.address[0]}:{r.address[1]}' for r in relays)] if relays else [])], env=env, stdout=server_log, stderr=subprocess.STDOUT)
            processes.append(server)
            deadline = time.monotonic()+30
            while time.monotonic() < deadline:
                assert server.poll() is None, 'Server failed; inspect server.log'
                match = re.search(r'listening=(\S+) local_tickets=', (output/'server.log').read_text())
                if match:
                    address = match[1]
                    break
                time.sleep(.05)
            else:
                raise TimeoutError('Server did not start')
            for relay in relays:
                host, port = address.rsplit(':', 1)
                relay.start((host, int(port)))
            with Recorder(output/'processes', os.getpid(), interval=.5):
                for index in range(clients):
                    client_output = output/f'client-{index+1}'
                    client_env = dict(env, THEIA_GRAYBOX_QUALIFY=str(client_output), THEIA_GRAYBOX_QUALIFY_SECONDS=str(duration))
                    log = (output/f'client-{index+1}.log').open('w')
                    logs.append(log)
                    client_address = f'{relays[index].address[0]}:{relays[index].address[1]}' if relays else address
                    processes.append(subprocess.Popen([str(binaries/'theia-client'), '--graybox-world', *(['--prediction'] if prediction else []), '--player-assets', str(players),
                        '--renet-connect', client_address, str(Path(tickets)/f'client-{index+1}.token')], env=client_env, stdout=log, stderr=subprocess.STDOUT))
                deadline = time.monotonic()+duration+45
                while time.monotonic() < deadline:
                    failed = [p for p in processes[1:] if p.poll() not in (None, 0)]
                    assert not failed, 'Client failed; inspect native logs'
                    assert server.poll() is None, 'Authority failed'
                    if all(p.poll() == 0 for p in processes[1:]):
                        break
                    time.sleep(.2)
                else:
                    raise TimeoutError('Native qualification did not finish')
            stop(server)
    finally:
        for process in reversed(processes):
            stop(process)
        for log in logs:
            log.close()
        for relay in relays:
            relay.close()
    for relay in relays:
        assert relay.stats["overflow"] == 0 and relay.stats["high_water"] <= 512, relay.stats
        assert relay.stats["dropped"] > 0 and relay.stats["reordered"] > 0, relay.stats
    results = []
    for index in range(clients):
        text = (output/f'client-{index+1}.log').read_text()
        expected_replays = sum(line.split(' ERROR ', 1)[-1] == 'renetcode::client: Failed to decode packet: sequence already received' for line in text.splitlines()) if impairment else 0
        checked_text = '\n'.join(line for line in text.splitlines() if not (impairment and line.split(' ERROR ', 1)[-1] == 'renetcode::client: Failed to decode packet: sequence already received'))
        for bad in ['VUID-', 'Validation Error', 'panicked at', ' ERROR ', 'Prediction paused:']:
            assert bad not in checked_text, bad
        assert '7900 XTX' in text, 'Expected documented discrete GPU; inspect actual adapter'
        if validation:
            assert 'VK_LAYER_KHRONOS_validation' in text, 'Validation layer not observed'
        reports = [json.loads(line)['graybox'] for line in text.splitlines() if line.startswith('{"graybox":')]
        assert reports and max(r['resident'] for r in reports) <= 81
        result = json.loads((output/f'client-{index+1}'/'world.json').read_text())
        if prediction:
            assert result['prediction']['predicted_steps'] > 100, result
            assert result['prediction']['corrections'] > 100, result
            assert result['prediction']['max_history'] <= 64, result
            assert result['prediction']['queue_drops'] == 0, result
        assert result['mesh_assets'] <= result['resident']+16, result
        assert result['gpu_mesh_assets'] <= result['resident']+16, result
        assert all(r['mesh_assets'] <= r['resident']+16 for r in reports[5:]), 'Unbounded CPU mesh assets'
        assert max(r['gpu_mesh_assets'] for r in reports) <= 2*81+16, 'Unbounded GPU staging'
        assert result['loaded']-result['unloaded'] == result['resident'], result
        assert result['max_ground_error'] < .003 and result['probes'] > 100, result
        assert all(value > 0 for value in result['pose_changes']), result
        assert result['moving_frames'] > 100, result
        assert result['held_frames']/result['moving_frames'] < .5, 'Local presentation still holds between most authority updates'
        assert any(r['visible'] < r['resident'] for r in reports if r['resident'] > 20), 'No observed frustum culling'
        assert result['loaded'] > result['max_resident'] and result['unloaded'] > 0, result
        if speed >= 8:
            assert result['loaded'] > 100 and result['unloaded'] > 50, result
        result['expected_netcode_replay_rejections'] = expected_replays
        if continent:
            assert result['position'][0] > 15900, 'Did not traverse the continent'
        steady = [r for r in reports if r['seconds'] >= 8]
        frames = sum(r['frame_count'] for r in steady)
        mean = sum(r['mean_ms']*r['frame_count'] for r in steady)/frames
        result.update(mean_frame_ms=mean, fps=1000/mean, worst_one_second_p95_ms=max(r['p95_ms'] for r in steady), frames=frames)
        results.append(result)
        assert (output/f'client-{index+1}'/'scene.png').is_file()
    server_text = (output/'server.log').read_text()
    assert server_text.count('player_join ') == clients
    if hold_jump:
        assert any(c["held_jump_reset_samples"] > 100 for c in results), results
        assert all(c["held_jump_reset_failures"] == 0 for c in results), results
    if tour:
        assert server_text.count('terrain_teleport ') == 5, server_text
        assert any(c['visited'] >= 8 and c['patches'] >= 8 for c in results), 'No client completed the teleport/revisit route'
    result = dict(passed=True, prediction=prediction, network_impairment=impairment, held_jump_reset=hold_jump, relay_stats=[r.stats for r in relays], clients=results, validation=validation, tour=tour, dev_speed=speed,
        continent=continent, resolution=[1280,720], display='caller native desktop',
        client_sha256=hashlib.sha256((binaries/'theia-client').read_bytes()).hexdigest(),
        server_sha256=hashlib.sha256((binaries/'theia-server').read_bytes()).hexdigest(),
        character_sha256=hashlib.sha256((players/'character.glb').read_bytes()).hexdigest(),
        scope='Linux native real secure Renet, standard Bevy/Avian; bounded terrain traversal, not MMO population capacity')
    (output/'verification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binaries', type=Path, default=Path('target/release'))
    parser.add_argument('--players', type=Path, default=Path('build/synty-character-canonical-stage'))
    parser.add_argument('--duration', type=int, default=65)
    parser.add_argument('--clients', type=int, choices=[1,2], default=1)
    parser.add_argument('--hold-jump-teleport', action='store_true')
    parser.add_argument('--impairment', action='store_true', help='40ms + 0..60ms jitter, 10% loss, duplication/reordering over bounded loopback relays')
    parser.add_argument('--prediction', action='store_true')
    parser.add_argument('--continent', action='store_true', help='Continuous full-continent accelerated traversal')
    parser.add_argument('--no-tour', action='store_true')
    parser.add_argument('--no-validation', action='store_true')
    parser.add_argument('--speed', type=int, choices=[1,8,32], default=32)
    args = parser.parse_args()
    print(json.dumps(run(args.output, args.binaries.resolve(), args.players.resolve(), duration=args.duration,
        clients=args.clients, tour=not args.no_tour, validation=not args.no_validation, speed=args.speed, continent=args.continent, prediction=args.prediction, impairment=args.impairment, hold_jump=args.hold_jump_teleport), indent=2))
