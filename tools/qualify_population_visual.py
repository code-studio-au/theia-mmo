"""Owned real server/native client population-art check; no OS movement automation."""
import argparse
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from player_assets import default_player_assets
from qualify_world_input_os import wait, capture_ready


def run(args):
    if not math.isfinite(args.seconds) or not 1 <= args.seconds <= 300:
        raise ValueError("Qualification duration must be 1..300 seconds")
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    common = ['--authored-world', str(args.world.resolve()), '--prediction', '--classic',
              '--combat-catalog', str(Path('content/gameplay/development-abilities.json').resolve()),
              '--population-config', str(args.population.resolve())]
    route_args = ['--test-route', str(args.route.resolve())] if args.route is not None else []
    environment = client_environment(os.environ)
    for key in list(environment):
        if key.startswith('THEIA_') and key.endswith(('_AUTOPLAY', '_QUALIFY')):
            environment.pop(key)
    environment.update(WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
                       WGPU_VALIDATION='1', RUST_LOG='info',
                       THEIA_GRAYBOX_QUALIFY=str(out), THEIA_GRAYBOX_QUALIFY_SECONDS=str(args.seconds))
    suffix = '.exe' if os.name == 'nt' else ''
    server_binary = args.binaries.resolve()/('theia-server'+suffix)
    client_binary = args.binaries.resolve()/('theia-client'+suffix)
    tickets = out/'tickets'
    server = client = None
    try:
        with (out/'server.log').open('w') as server_log, (out/'client.log').open('w') as client_log:
            server = subprocess.Popen([str(server_binary), '127.0.0.1:0', str(out/'unused-cert.der'),
                                       *common, '--renet-tickets', str(tickets)],
                                      stdout=server_log, stderr=server_log)
            def ready():
                assert server.poll() is None, 'Owned server exited; see server.log'
                found = re.search(r'listening=(\S+) local_tickets=', (out/'server.log').read_text())
                return found.group(1) if found else None
            address = wait(ready, 45)
            client = subprocess.Popen([str(client_binary), *common, *route_args, '--creature-asset', str(args.creature.resolve()),
                                       '--player-assets', str(args.players.resolve()), '--window-size', '1600x1000',
                                       '--test-window', '1/1', '--renet-connect', address, str(tickets/'client-1.token')],
                                      env=environment, stdout=client_log, stderr=client_log)
            assert client.wait(timeout=args.seconds+60) == 0, 'Native population client failed; see client.log'
        assert capture_ready(out/'scene.png'), 'Missing native screenshot'
        report = json.loads((out/'world.json').read_text())
        assert report['connected'] and report['network_error'] is None
        assert report['target_count'] == 5, report
        visuals = report['creature_visuals']
        assert visuals['attached'] == 5 and visuals['players'] == 5, visuals
        assert visuals['max_visuals'] == 5 and visuals['transitions'][0] >= 5, visuals
        assert visuals['dropped_cues'] == 0, visuals
        assert check_gpu_diagnostics((out/'client.log').read_text()) == 0
        travel = None
        if args.route is not None:
            reports = [json.loads(line)['test_controller'] for line in (out/'client.log').read_text().splitlines()
                       if line.startswith('{"test_controller":')]
            assert reports, 'No route controller observations'
            travel = reports[-1]
            assert travel['finished'] and travel['travel']['finished'] and travel['travel']['failure'] is None, travel
            assert not travel['active'], 'Arrival must release ordinary player input'
            goal = json.loads(args.route.read_text())['waypoints'][-1]
            assert sum((travel['feet'][i]-goal[i])**2 for i in [0,2])**.5 <= .3, travel
            assert abs(travel['feet'][1]-goal[1]) <= .3, travel
            assert report['moving_frames'] > 0, report
        (out/'qualification.json').write_text(json.dumps(dict(
            passed=True, targets=5, creature_visuals=visuals, connected=True, travel=travel,
            scope='Actual native Linux Vulkan client with five passive creature rigs; no OS movement, model-catalog resolution, pursuit or Windows claim'), indent=2)+'\n')
    finally:
        for process in [client, server]:
            if process and process.poll() is None:
                process.send_signal(signal.SIGINT) if os.name != 'nt' else process.terminate()
                try: process.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    process.kill(); process.wait(timeout=5)
        if tickets.exists():
            for token in tickets.iterdir(): token.unlink()
            tickets.rmdir()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--world', type=Path, required=True)
    parser.add_argument('--population', type=Path, required=True)
    parser.add_argument('--route', type=Path, help='Optional checked route controller; no OS movement input')
    parser.add_argument('--seconds', type=float, default=25, help='Bounded native observation duration (1..300 seconds)')
    parser.add_argument('--players', type=Path, default=default_player_assets())
    parser.add_argument('--creature', type=Path, default=Path('build/creature-wolf-stage-v1/wolf.glb'))
    parser.add_argument('--output', type=Path, required=True)
    run(parser.parse_args())
