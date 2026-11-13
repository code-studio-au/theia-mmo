"""Two owned native clients observe shared population windups and Bevy playback.

Uses real encrypted local tickets and ordinary pursuing NPC authority. The copied
fixture widens perception and reduces damage to keep observation deterministic;
this is presentation evidence, not OIDC/SQL, balance or durable grant acceptance.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import time

from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from player_assets import default_player_assets
from qualify_world_input_os import wait, capture_ready


def run(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False, mode=0o700)
    package = out/'package'
    shutil.copytree(args.package.resolve(), package)
    definition = package/json.loads((package/'population.json').read_text())['creature']
    creature = json.loads(definition.read_text())
    creature['aggro']['perception_metres'] = 12.0
    definition.write_text(json.dumps(creature)+'\n')
    retaliation = out/'retaliation.json'
    data = json.loads(Path('content/gameplay/development-creature-retaliation.json').read_text())
    data['damage'] = 1
    retaliation.write_text(json.dumps(data)+'\n')
    common = ['--authored-world', str(package/'world'), '--population-config', str(package/'population.json'),
              '--prediction', '--classic', '--public-creature-attacks', '--public-avatar-life',
              '--combat-catalog', str(Path('content/gameplay/development-abilities.json').resolve()),
              '--character-stats', str(Path('content/gameplay/development-character-stats.json').resolve()),
              '--creature-retaliation', str(retaliation),
              '--player-respawn', str(Path('content/gameplay/development-player-respawn.json').resolve())]
    env = client_environment(os.environ)
    for key in list(env):
        if key.startswith('THEIA_'):
            env.pop(key)
    env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
               LD_LIBRARY_PATH=str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()),
               VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', WGPU_VALIDATION='1', RUST_LOG='info')
    processes, handles, clients = [], [], []
    tickets = out/'tickets'
    evidence = {'scope': __doc__, 'tick_hz': args.tick_hz, 'perception_metres': 12, 'damage': 1}
    evidence['binaries'] = {name: hashlib.sha256((args.binaries/name).read_bytes()).hexdigest()
                            for name in ('theia-client', 'theia-server')}
    try:
        log = (out/'server.log').open('w'); handles.append(log)
        server = subprocess.Popen([str(args.binaries.resolve()/'theia-server'), '127.0.0.1:0',
            str(out/'unused.der'), *common, '--tick-hz', str(args.tick_hz), '--renet-tickets', str(tickets)],
            env=env, stdout=log, stderr=subprocess.STDOUT)
        processes.append(server)
        def ready():
            assert server.poll() is None, 'Owned server exited; inspect server.log'
            match = re.search(r'listening=(\S+) local_tickets=', (out/'server.log').read_text())
            return match.group(1) if match else None
        address = wait(ready, 60)
        for index in range(2):
            folder = out/f'client-{index+1}'; folder.mkdir()
            log = (folder/'client.log').open('w'); handles.append(log)
            child = subprocess.Popen([str(args.binaries.resolve()/'theia-client'), *common,
                '--player-assets', str(args.players.resolve()), '--creature-asset', str(package/'creature.glb'),
                '--window-size', '1280x720', '--test-window', f'{index+1}/2',
                '--renet-connect', address, str(tickets/f'client-{index+1}.token')],
                env=dict(env, THEIA_WORLD_INPUT_OS_QUALIFY=str(folder)), stdout=log, stderr=subprocess.STDOUT)
            processes.append(child); clients.append((child, folder))
        records = [dict(samples=0, actors=set(), remote=set(), simultaneous=0, playback={}, advanced=set(), ended=set()) for _ in clients]
        previous = [set(), set()]
        admitted_at = None
        deadline = time.monotonic()+75
        while time.monotonic() < deadline:
            assert server.poll() is None, 'Owned server exited during presentation'
            for index, (child, folder) in enumerate(clients):
                assert child.poll() is None, 'Owned native client exited; inspect client.log'
                try:
                    state = json.loads((folder/'input-os.json').read_text())
                except (FileNotFoundError, json.JSONDecodeError):
                    continue
                if not state['connected']:
                    continue
                frame = state.get('public_population_attack')
                if not frame or len(frame['actors']) != 5:
                    continue
                assert state['public_creature_attack'] is None
                record = records[index]
                record['samples'] += 1
                active = set()
                public = {a['creature']: a['windup'] for a in frame['actors']}
                for visual in state.get('creature_animations', []):
                    actor = visual['entity']
                    if visual['clip'] != 3 or not public.get(actor):
                        continue
                    pending = public[actor]
                    # Reports can straddle successive Bevy Update systems; only
                    # compare exact matching public attack serials.
                    if visual['serial'] != pending['completes']:
                        continue
                    target = visual['attack_target']
                    animation = visual['animation']
                    if target is None or animation is None:
                        continue
                    delta = [target[i]-visual['position'][i] for i in (0,2)]
                    length = math.hypot(*delta)
                    if length > .05:
                        dot = sum(delta[j]*visual['forward'][i] for j,i in enumerate((0,2)))/length
                        assert dot > .98, ('Attack does not face its victim', visual, dot)
                    active.add(actor); record['actors'].add(actor)
                    if visual['remote_victim']:
                        assert pending['target'] != state['actor']
                        record['remote'].add(actor)
                    key = (actor, visual['serial'])
                    seek = animation['seek']
                    if key in record['playback'] and seek > record['playback'][key]+.001:
                        record['advanced'].add(actor)
                    record['playback'][key] = seek
                record['simultaneous'] = max(record['simultaneous'], len(active))
                record['ended'].update(previous[index]-active)
                previous[index] = active
                if active:
                    (folder/'attack-sample.json').write_text(json.dumps(state, indent=2)+'\n')
            if admitted_at is None and all(r['samples'] for r in records):
                admitted_at = time.monotonic()
            # Native scene entities can precede GPU pipeline readiness. Keep the
            # same warmup used by ordinary account-population qualification.
            warmed = admitted_at is not None and time.monotonic()-admitted_at >= 10
            if warmed and all(r['simultaneous'] >= 2 and len(r['advanced']) >= 2 and len(r['ended']) >= 2 for r in records) and any(len(r['remote']) >= 2 for r in records):
                break
            time.sleep(.05)
        else:
            raise AssertionError(f'No complete independent playback observations: {records}')
        evidence['clients'] = []
        for record, (child, folder) in zip(records, clients):
            (folder/'capture.request').write_text('public-attacks\n')
            wait(lambda: capture_ready(folder/'input-os.png'), 20)
            result = {k: sorted(v) if isinstance(v,set) else v for k,v in record.items() if k != 'playback'}
            evidence['clients'].append(result)
    finally:
        for child in reversed(processes):
            if child.poll() is None:
                child.send_signal(signal.SIGINT)
                try:
                    child.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    child.kill(); child.wait(timeout=10)
        for handle in handles:
            handle.close()
        if tickets.exists():
            shutil.rmtree(tickets)
    evidence['gpu'] = []
    server_log = (out/'server.log').read_text()
    assert 'development_public_creature_attacks enabled=true contract=NP01' in server_log
    transport = next(json.loads(line.removeprefix('transport_summary '))
                     for line in server_log.splitlines() if line.startswith('transport_summary '))
    assert transport['max_players'] == 2 and transport['public_attack_states_sent'] > 0
    assert transport['public_attack_backpressure_disconnects'] == 0
    assert transport['player_life']['strikes'] > 0
    evidence['transport'] = transport
    for _, folder in clients:
        log = (folder/'client.log').read_text()
        assert check_gpu_diagnostics(log) == 0
        adapters = [line for line in log.splitlines() if 'AdapterInfo' in line]
        assert adapters, 'Missing actual GPU adapter evidence'
        evidence['gpu'].append(adapters)
    evidence['passed'] = True
    (out/'qualification.json').write_text(json.dumps(evidence, indent=2)+'\n')
    return evidence


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--players', type=Path, default=default_player_assets())
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--tick-hz', type=int, choices=(20,30,60), default=30)
    print(json.dumps(run(parser.parse_args()), indent=2))
