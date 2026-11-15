"""Owned real server/native client population-art check; no OS movement automation."""
import argparse
import json
import math
import os
from pathlib import Path
import re
import signal
import subprocess
import time
from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from player_assets import default_player_assets
from qualify_world_input_os import wait, capture_ready


def run(args):
    if not math.isfinite(args.seconds) or not 1 <= args.seconds <= 300:
        raise ValueError("Qualification duration must be 1..300 seconds")
    if args.resource_cast and (not args.player_life or args.seconds < 25):
        raise ValueError('Resource cast qualification requires player life and at least 25 seconds')
    if args.observe_idle_travel and (not args.player_life or args.seconds < 25 or args.route):
        raise ValueError('Idle travel requires player life, at least 25 seconds and a stationary player')
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    common = ['--authored-world', str(args.world.resolve()), '--prediction', '--classic',
              '--combat-catalog', str(args.combat_catalog.resolve()),
              '--population-config', str(args.population.resolve())]
    if args.player_life:
        common.extend(['--character-stats', str(Path('content/gameplay/development-character-stats.json').resolve()),
                       '--creature-retaliation', str(Path('content/gameplay/development-creature-retaliation.json').resolve()),
                       '--player-respawn', str(Path('content/gameplay/development-player-respawn.json').resolve()),
                       '--public-avatar-life'])
    route_args = ['--test-route', str(args.route.resolve())] if args.route is not None else []
    environment = client_environment(os.environ)
    for key in list(environment):
        if key.startswith('THEIA_') and key.endswith(('_AUTOPLAY', '_QUALIFY')):
            environment.pop(key)
    environment.update(WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
                       WGPU_VALIDATION='1', RUST_LOG='info',
                       THEIA_GRAYBOX_QUALIFY=str(out), THEIA_GRAYBOX_QUALIFY_SECONDS=str(math.ceil(args.seconds)))
    if args.player_life:
        environment['THEIA_WORLD_INPUT_OS_QUALIFY'] = str(out)
    if args.resource_cast:
        environment['THEIA_POPULATION_RESOURCE_QUALIFY'] = str(out)
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
                                       '--player-assets', str(args.players.resolve()), '--window-size', '1920x1440',
                                       '--test-window', '1/1', '--renet-connect', address, str(tickets/'client-1.token')],
                                      env=environment, stdout=client_log, stderr=client_log)
            if args.resource_cast:
                def ready_to_cast():
                    assert client.poll() is None, 'Native client exited before cast readiness'
                    state = json.loads((out/'input-os.json').read_text())
                    return state['connected'] and len(state.get('combat_targets') or []) == 5 and (state.get('minimap') or {}).get('known_pixels', 0) > 0
                wait(ready_to_cast, 15)
                warmed = time.monotonic()
                wait(lambda: ready_to_cast() and time.monotonic()-warmed >= 10, 15)
                (out/'combat.start').write_text('start\n')
            idle_samples = []
            if args.observe_idle_travel:
                deadline = time.monotonic() + args.seconds + 60
                while client.poll() is None and time.monotonic() < deadline:
                    try:
                        sample = json.loads((out/'input-os.json').read_text())
                        if sample.get('connected') and len(sample.get('target_entities', [])) == 5:
                            idle_samples.append(sample['target_entities'])
                    except (FileNotFoundError, json.JSONDecodeError):
                        pass
                    time.sleep(.25)
            assert client.wait(timeout=args.seconds+60) == 0, 'Native population client failed; see client.log'
        assert capture_ready(out/'scene.png'), 'Missing native screenshot'
        report = json.loads((out/'world.json').read_text())
        assert report['connected'] and report['network_error'] is None
        assert report['target_count'] == 5, report
        visuals = report['creature_visuals']
        assert visuals['attached'] == 5 and visuals['players'] == 5, visuals
        assert visuals['max_visuals'] == 5, visuals
        if args.observe_idle_travel:
            # A rig may finish loading after its authority starts the patrol;
            # it need not enter the stationary animation first.
            assert sum(visuals['transitions']) >= 5 and visuals['moving_frames'] > 0, visuals
        else:
            assert visuals['transitions'][0] >= 5, visuals
        assert visuals['dropped_cues'] == 0, visuals
        assert check_gpu_diagnostics((out/'client.log').read_text()) == 0
        life = None
        if args.player_life:
            state = json.loads((out/'input-os.json').read_text())
            combat = state['combat']
            assert combat['player_health'] == 100 and combat['player_generation'] == 1, combat
            assert combat['player_deaths'] == 0, combat
            actors = state['public_avatar_life']['actors']
            assert len(actors) == 1 and actors[0]['actor'] == state['actor'], actors
            assert actors[0]['generation'] == 1 and not actors[0]['dead'], actors
            life = dict(health=100, generation=1, deaths=0, public_actors=actors)
        resource_cast = None
        if args.resource_cast:
            resource_cast = json.loads((out/'typed-combat.json').read_text())
            assert resource_cast['passed'] and resource_cast['casts'] == 1, resource_cast
            assert resource_cast['remaining'] == 80 and resource_cast['mana_spent'], resource_cast
            assert resource_cast['bystanders_unchanged'] and resource_cast['player_health'] == 100, resource_cast
            assert resource_cast['inventory_revision'] is None, resource_cast
            assert state['combat']['started'] == 1 and state['combat']['rejected_requests'] == 0, state['combat']
            assert state['combat']['minimum_mana'] == 75 and state['combat']['damage'] == 1, state['combat']
        travel = None
        idle_travel = None
        if args.observe_idle_travel:
            (out/'idle-travel.json').write_text(json.dumps(idle_samples, indent=2)+'\n')
            assert len(idle_samples) > 20, 'Insufficient connected movement observations'
            initial = {t['entity']: t['position'] for t in idle_samples[0]}
            moved = set()
            for sample in idle_samples:
                assert set(t['entity'] for t in sample) == set(initial), 'Actor identities changed'
                moved.update(t['entity'] for t in sample if math.dist(t['position'], initial[t['entity']]) > 1.)
            assert len(moved) == 5, f'Only {len(moved)} creatures moved over one metre'
            idle_travel = dict(moved=sorted(moved), samples=len(idle_samples))
        if args.route is not None:
            reports = [json.loads(line)['test_controller'] for line in (out/'client.log').read_text().splitlines()
                       if line.startswith('{"test_controller":')]
            assert reports, 'No route controller observations'
            travel = reports[-1]
            assert travel['finished'] and travel['travel']['finished'] and travel['travel']['failure'] is None, travel
            assert not travel['active'], 'Arrival must release ordinary player input'
            points = json.loads(args.route.read_text())['waypoints']
            goal = points[0 if travel['travel']['leg'] == 'return' else -1]
            assert sum((travel['feet'][i]-goal[i])**2 for i in [0,2])**.5 <= .3, travel
            assert abs(travel['feet'][1]-goal[1]) <= .3, travel
            assert report['moving_frames'] > 0, report
        (out/'qualification.json').write_text(json.dumps(dict(
            passed=True, targets=5, creature_visuals=visuals, connected=True, travel=travel, player_life=life, resource_cast=resource_cast, idle_travel=idle_travel,
            scope='Actual native Linux Vulkan client with five configured creature rigs; no OS movement, model-catalog resolution, combat pursuit or Windows claim'), indent=2)+'\n')
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
    parser.add_argument('--resource-cast', action='store_true', help='Qualify one ordinary 25-mana/20-damage cast without OS input')
    parser.add_argument('--combat-catalog', type=Path, default=Path('content/gameplay/development-abilities.json'), help='Exact combat catalogue to qualify, including an editor-exported snapshot')
    parser.add_argument('--player-life', action='store_true', help='Qualify standalone population health and public life state')
    parser.add_argument('--observe-idle-travel', action='store_true', help='Observe every authoritative creature moving over one metre while the player remains idle')
    parser.add_argument('--route', type=Path, help='Optional checked route controller; no OS movement input')
    parser.add_argument('--seconds', type=float, default=25, help='Bounded native observation duration (1..300 seconds)')
    parser.add_argument('--players', type=Path, default=default_player_assets())
    parser.add_argument('--creature', type=Path, default=Path('build/creature-wolf-stage-v1/wolf.glb'))
    parser.add_argument('--output', type=Path, required=True)
    run(parser.parse_args())
