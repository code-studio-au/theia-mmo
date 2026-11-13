"""Owned native passive-population damage/death/respawn through normal game keys."""
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
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False, mode=0o700)
    common = ['--authored-world', str(args.world.resolve()), '--prediction', '--classic',
              '--population-config', str(args.population.resolve()),
              '--combat-catalog', str(Path('content/gameplay/development-abilities.json').resolve())]
    env = client_environment(os.environ)
    for key in list(env):
        if key.startswith('THEIA_') and key.endswith(('_AUTOPLAY', '_QUALIFY')):
            env.pop(key)
    env.update(DISPLAY=':0', WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
               WGPU_VALIDATION='1', RUST_LOG='info', THEIA_WORLD_INPUT_OS_QUALIFY=str(out))
    server = client = desktop = None
    tickets = out/'tickets'
    proof = None
    try:
        with (out/'server.log').open('w') as server_log, (out/'client.log').open('w') as client_log:
            server = subprocess.Popen([str(args.binaries.resolve()/'theia-server'), '127.0.0.1:0',
                str(out/'unused-cert.der'), *common, '--renet-tickets', str(tickets)], stdout=server_log, stderr=server_log)
            def ready():
                assert server.poll() is None, 'Owned server exited'
                match = re.search(r'listening=(\S+) local_tickets=', (out/'server.log').read_text())
                return match.group(1) if match else None
            address = wait(ready, 45)
            client = subprocess.Popen([str(args.binaries.resolve()/'theia-client'), *common,
                '--creature-asset', str(args.creature.resolve()), '--player-assets', str(args.players.resolve()),
                '--window-size', '1600x1000', '--test-window', '1/1',
                *(['--test-route', str(args.route.resolve())] if args.route else []),
                '--renet-connect', address, str(tickets/'client-1.token')], env=env, stdout=client_log, stderr=client_log)
            admitted = False
            def state():
                assert client.poll() is None, 'Owned client exited; inspect client.log'
                value = json.loads((out/'input-os.json').read_text())
                if admitted:
                    assert value['connected'], 'Client disconnected during population combat'
                return value
            def health():
                return {h['entity']:h for h in state()['combat_targets'] or []}
            wait(lambda: len(health()) == 5 and (state().get('creature_visuals') or {}).get('players',0) >= 5, 60)
            admitted = True
            window = wait(lambda: owned_window(client.pid), 30)
            desktop = Desktop(); desktop.focus(window)
            width, height, scale = state()['window']
            desktop.move(window, [width*.5, height*.35], scale)
            wait(lambda: state()['cursor']['focused'])
            def key(name):
                # Travel and server cooldown waits can leave this window in the
                # background. Activate only this still-owned client per action.
                assert client.poll() is None and owned_window(client.pid)==window
                desktop.focus(window)
                wait(lambda: state()['cursor']['focused'])
                desktop.key(name)
            def capture(name):
                (out/'capture.request').write_text(name+'\n')
                wait(lambda: capture_ready(out/'input-os.png'), 10)
                (out/'input-os.png').rename(out/name)
            def arrived(leg):
                controller = state().get('test_controller')
                if not controller:
                    return None
                travel = controller['travel']
                assert travel['failure'] is None, travel
                return controller if travel['leg']==leg and travel['finished'] and not controller['active'] else None
            outward = None
            if args.route:
                outward = wait(lambda: arrived('outward'), 120)
            initial = health()
            assert all(h['current']==h['maximum']==100 for h in initial.values()), initial
            entities = {t['entity']:t for t in state()['target_entities']}
            key('Tab')
            selected = wait(lambda: state()['selected_hostile'], 10)
            assert selected in initial and selected in entities
            home = entities[selected]
            capture('selected.png')
            damage = []
            for expected in [80,60,40,20,0]:
                key('1')
                wait(lambda: health().get(selected,{}).get('current')==expected, 8)
                current = health()
                assert all(current[key]==value for key,value in initial.items() if key!=selected), current
                damage.append(dict(expected=expected, targets=list(current.values()), reply=state()['ability_reply']))
                if expected:
                    time.sleep(2.1)  # Observe ordinary cast/cooldown; never bypass the server clock.
            dead = state()
            assert dead['combat_targets'] and health()[selected]['respawn_at'] is not None
            wait(lambda: state()['creature_visuals']['dead_frames']>0 and state()['creature_visuals']['hit_frames']>0, 5)
            capture('dead.png')
            def respawned():
                current = health()
                fresh = set(current)-set(initial)
                if len(current)==5 and len(fresh)==1 and selected not in current and all(h['current']==100 for h in current.values()):
                    return next(iter(fresh))
                return None
            fresh = wait(respawned, 65)
            replacement = wait(lambda: next((t for t in state()['target_entities'] if t['entity']==fresh),None), 10)
            assert replacement['epoch']==home['epoch'] and math.dist(replacement['position'],home['position']) < .001
            assert state()['selected_hostile'] is None, 'Retired selection survived a fresh actor incarnation'
            assert all(health()[key]==value for key,value in initial.items() if key!=selected)
            wait(lambda: state()['creature_visuals']['attached']>=6 and state()['creature_visuals']['players']>=6,10)
            capture('respawned.png')
            if args.expect_particles:
                before = health()
                key('F9')
                wait(lambda: not state()['world_particles']['effects_enabled'])
                capture('effects-hidden.png')
                assert health()==before
                key('F9')
                wait(lambda: state()['world_particles']['effects_enabled'])
                capture('effects-restored.png')
                assert health()==before
            returned = None
            if args.route and json.loads(args.route.read_text()).get('return_trip'):
                key('F7')
                returned = wait(lambda: arrived('return'),120)
                capture('returned.png')
            final = state()
            assert final['creature_visuals']['dropped_cues']==0
            if args.expect_particles:
                particles = final['world_particles']
                assert particles['placements']==args.expect_particles and particles['live_roots']==args.expect_particles,particles
                assert particles['released_systems']==args.expect_particles and particles['visible_emitter_frames']>0,particles
                assert particles['unready_emitter_frames']==0 and particles['visibility_key_toggles']==2,particles
            proof = dict(passed=True, original=selected, replacement=fresh, unchanged_other_actors=4,
                damage=damage, dead=dead['combat_targets'], final=final['combat_targets'],
                creature_visuals=final['creature_visuals'], outward=outward, returned=returned,
                world_particles=final.get('world_particles'),
                scope='One native five-creature passive pool, normal Tab/1 inputs, independent death and fresh-ID respawn; optional checked travel, no quests or pursuit')
    finally:
        if desktop: desktop.close()
        for process in [client, server]:
            if process and process.poll() is None:
                process.send_signal(signal.SIGINT)
                try: process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.kill();process.wait(timeout=5)
        if tickets.exists():
            for token in tickets.iterdir(): token.unlink()
            tickets.rmdir()
    assert check_gpu_diagnostics((out/'client.log').read_text())==0
    transport = next(json.loads(line.removeprefix('transport_summary ')) for line in (out/'server.log').read_text().splitlines() if line.startswith('transport_summary '))
    assert transport['rejected_inputs']==0 and transport['movement_recoveries']==0,transport
    proof['transport']=transport
    (out/'qualification.json').write_text(json.dumps(proof,indent=2)+'\n')


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('world','population','creature','output'):
        parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--route',type=Path)
    parser.add_argument('--expect-particles',type=int,default=0)
    parser.add_argument('--players',type=Path,default=default_player_assets())
    parser.add_argument('--binaries',type=Path,default=Path('target/debug'))
    run(parser.parse_args())
