#!/usr/bin/env python3
"""Owned OIDC/PostgreSQL and two actual native clients: player sword melee."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import signal
import struct
import subprocess
import time
import urllib.parse
import urllib.request

from account_login import AccountClient
from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from private_identity import Identity, _NoRedirect
from private_postgres import Cluster


def read(path):
    try:
        return json.loads(path.read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        return None


def wait(predicate, seconds, label):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(.05)
    raise TimeoutError(label)


def run(output, binaries, players, creature_asset, terrain, navigation,
        identity_sdk, postgres_sdk, window_size="1920x1200", outcome=None, projectile=False, world_input=False):
    if world_input and projectile:raise ValueError("World input mode qualifies melee separately from projectiles")
    tour = "projectile" if projectile else "melee"
    if projectile and outcome is None:
        outcome = "hit"
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    client = binaries / 'theia-client'
    authority = binaries / 'account_native_qualification'
    assert client.is_file() and authority.is_file()
    creature = output / 'qualification-creature.json'
    data = json.loads(Path('content/gameplay/development-creature-pursuit.json').read_text())
    # Keep the first acquisition deterministic across actual browser/GPU startup;
    # both clients are gated before the player attack tour starts.
    data['wait_ms'] = 60000 if projectile else 15000
    if projectile:
        data.pop('aggro', None)
    if outcome is not None:
        data.update(schema=2, can_deflect=True)
    creature.write_text(json.dumps(data, indent=2) + '\n')
    retaliation = output / 'qualification-retaliation.json'
    data = json.loads(Path('content/gameplay/development-creature-retaliation.json').read_text())
    data['damage'] = 1
    retaliation.write_text(json.dumps(data, indent=2) + '\n')
    stats = Path('content/gameplay/development-character-stats.json').resolve()
    respawn = Path('content/gameplay/development-player-respawn.json').resolve()
    items = Path('content/gameplay/development-equipment-items.json').resolve()
    abilities = Path('content/gameplay/development-projectile-abilities.json' if projectile else 'content/gameplay/development-melee-abilities.json').resolve()
    if outcome is not None:
        data = json.loads(abilities.read_text())
        data['schema'] = 6 if projectile else 5
        weights = {name: (10000 if name == outcome else 0) for name in ['hit', 'miss', 'deflect']}
        data['abilities'][0 if projectile else 1]['hit_table'] = {name+'_bp': value for name, value in weights.items()}
        if projectile:
            data['abilities'][0]['projectile'].update(speed_metres_per_second=3., lifetime_ms=10000)
        abilities = output/'qualification-abilities.json'
        abilities.write_text(json.dumps(data, indent=2)+'\n')
        abilities = abilities.resolve()
    statuses = Path('content/gameplay/development-statuses.json').resolve()
    rewards = Path('content/gameplay/development-rewards.json').resolve()
    evidence = {'scope': 'Actual local OIDC accounts, owned PostgreSQL, encrypted native Linux clients; no physical keyboard/browser or Windows claim',
                'client_sha256': hashlib.sha256(client.read_bytes()).hexdigest(),
                'server_sha256': hashlib.sha256(authority.read_bytes()).hexdigest(),
                'window_size': window_size, 'expected_outcome': outcome, 'fixture': {'patrol_wait_ms': 60000 if projectile else 15000, 'damage': 1, 'leash_metres': None if projectile else 35, 'projectile': projectile, 'stationary_target': projectile, 'projectile_speed_mps': 3 if projectile else None}}
    children, handles, tickets, accounts, private = [], [], [], [], []
    server = None
    server_log = output / 'server.log'
    with Identity(output/'provider', identity_sdk, users=2, token_seconds=180) as identity, Cluster(output/'cluster', postgres_sdk) as cluster:
        private.extend([cluster.password] + [p for _, p in identity.users])
        env = dict(cluster.test_environment(), THEIA_TEST_OIDC_ISSUER=identity.issuer,
                   THEIA_TEST_OIDC_CLIENT=identity.client_id,
                   THEIA_TEST_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                   THEIA_TEST_SAVED_TERRAIN=str(terrain), THEIA_TEST_SPAWN='15040,15040',
                   THEIA_TEST_INVENTORY='1', THEIA_TEST_EQUIPMENT='1', THEIA_TEST_REWARDS='1',
                   THEIA_TEST_PLAYER_LIFE='1', THEIA_TEST_CREATURE_PROFILE=str(creature.resolve()),
                   THEIA_TEST_RETALIATION_PROFILE=str(retaliation.resolve()),
                   THEIA_TEST_RESPAWN_PROFILE=str(respawn), THEIA_TEST_NAVIGATION=str(navigation), THEIA_TEST_COMBAT_PROFILE=str(abilities))
        for key in ['THEIA_TEST_DIALOGUE', 'THEIA_TEST_VENDORS', 'THEIA_TEST_PROGRESSION',
                    'THEIA_TEST_QUESTS', 'THEIA_TEST_QUEST_LIFECYCLE', 'THEIA_TEST_QUEST_TURN_IN_XP']:
            env[key] = '0'
        try:
            log = server_log.open('w'); handles.append(log)
            server = subprocess.Popen([str(authority)], env=env, stdout=log, stderr=subprocess.STDOUT)
            def ready():
                assert server.poll() is None, 'Owned melee authority exited before readiness'
                return next((json.loads(line.removeprefix('account_game_ready ')) for line in server_log.read_text().splitlines() if line.startswith('account_game_ready ')), None)
            base = wait(ready, 25, 'Melee authority did not become ready')['account_address']
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), _NoRedirect())
            def login(index):
                account = AccountClient(base, alive=lambda: server.poll() is None)
                accounts.append(account)
                def browser(url):
                    private.append(url)
                    callback = identity.authorize(url, user_index=index); private.append(callback)
                    with opener.open(base+'/callback?'+urllib.parse.urlsplit(callback).query, timeout=8) as response:
                        assert response.status == 200 and response.headers.get('Cache-Control') == 'no-store'
                    return True
                account.login(open_browser=browser); private.append(account._session)
                name = 'MeleeArden' if index == 0 else 'MeleeSelene'
                character = account.choose_character(read=lambda _: name, write=lambda _: None)
                return account, character
            a, ca = login(0); b, cb = login(1)
            assert ca != cb
            def native(variant, account, character):
                folder = output/variant; folder.mkdir()
                ticket = output/(variant+'.token'); tickets.append(ticket)
                address = account.enter(character, ticket)
                assert ticket.stat().st_mode & 0o777 == 0o600
                libs = str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve())
                child_env = dict(client_environment(os.environ), DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000',
                                 LD_LIBRARY_PATH=libs, THEIA_PERFORMANCE_UNCAPPED='1',
                                 THEIA_GRAYBOX_QUALIFY=str(folder), THEIA_GRAYBOX_QUALIFY_SECONDS='50',
                                 THEIA_PLAYER_MELEE_QUALIFY=variant, THEIA_PLAYER_MELEE_OUTCOME=outcome or 'hit', THEIA_PLAYER_MELEE_GATE=str((output/'go').resolve()), VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
                                 VK_LOADER_DEBUG='layer', WGPU_VALIDATION='1', RUST_LOG='info')
                for key in list(child_env):
                    if key.startswith('THEIA_') and key.endswith(('_AUTOPLAY', '_QUALIFY')) and key not in ['THEIA_GRAYBOX_QUALIFY', 'THEIA_PLAYER_MELEE_QUALIFY']:
                        child_env.pop(key)
                if world_input:child_env['THEIA_WORLD_INPUT_QUALIFY']='1'
                if projectile:
                    for key in ['THEIA_PLAYER_MELEE_QUALIFY','THEIA_PLAYER_MELEE_OUTCOME','THEIA_PLAYER_MELEE_GATE']:
                        child_env.pop(key, None)
                    child_env.update(THEIA_PROJECTILE_QUALIFY=variant, THEIA_PROJECTILE_OUTCOME=outcome, THEIA_PROJECTILE_GATE=str((output/'go').resolve()))
                log = (folder/'client.log').open('w'); handles.append(log)
                args = [str(client), '--saved-terrain', str(terrain), '--prediction', '--player-assets', str(players),
                        '--creature-asset', str(creature_asset), '--window-size', window_size,
                        '--item-catalog', str(items), '--reward-catalog', str(rewards),
                        '--creature-profile', str(creature.resolve()), '--navigation', str(navigation),
                        '--combat-catalog', str(abilities), '--status-catalog', str(statuses), '--character-stats', str(stats),
                        '--creature-retaliation', str(retaliation.resolve()), '--player-respawn', str(respawn),
                        '--renet-connect', address, str(ticket)]
                process = subprocess.Popen(args, env=child_env, stdout=log, stderr=subprocess.STDOUT)
                children.append(process)
                return process, folder
            first = native('A', a, ca)
            def ready_peer(peer):
                report = read(peer[1]/(tour+'.json'))
                assert peer[0].poll() is None, 'Native melee peer exited before startup gate'
                return report if report and report['ready'] else None
            wait(lambda: ready_peer(first), 25, 'First native melee peer did not load')
            second = native('B', b, cb)
            wait(lambda: ready_peer(second), 25, 'Second native melee peer did not load')
            (output/'go').write_text('both native peers ready\n')
            evidence['native'] = {}
            width, height = map(int, window_size.split('x'))
            for variant, (process, folder) in [('A', first), ('B', second)]:
                def complete():
                    report = read(folder/(tour+'.json'))
                    assert process.poll() is None or process.returncode == 0, 'Native melee client failed'
                    capture = folder/(tour+'.png')
                    if not report or not report['passed'] or not capture.exists() or capture.stat().st_size < 24:
                        return None
                    header = capture.read_bytes()[:24]
                    assert header[:8] == b'\x89PNG\r\n\x1a\n' and struct.unpack('>II', header[16:24]) == (width, height)
                    return report
                evidence['native'][variant] = wait(complete, 45, 'Native melee tour/capture did not complete')
            for process in children:
                assert process.wait(timeout=45) == 0, 'Native melee did not exit cleanly'
            for variant, (_, folder) in [('A', first), ('B', second)]:
                text = (folder/'client.log').read_text()
                assert 'VK_LAYER_KHRONOS_validation' in text and check_gpu_diagnostics(text) == 0
                evidence['native'][variant]['gpu_diagnostics'] = 0
            if world_input:
                controls=evidence['native']['A']['world_input']
                assert controls['enemy_clicks']>=2 and controls['basic_requests']==1,controls
            assert evidence['native']['A']['cancelled_by_unequip'] and evidence['native']['A']['range_refused']
            if projectile:
                assert evidence['native']['A']['wrong_weapon_refused']
                for report in evidence['native'].values():
                    assert report['shot_health'] == 100 and report['shot_resolved'] == 0 and report['pre_damage_frames'] > 0
                    visual = report['visuals']
                    assert visual['launches'] == visual['spawns'] == visual['terminals'] == 3
                    assert visual['visible_frames'] > 0 and visual['travel_frames'] > 0 and visual['dropped'] == 0
            else:
                assert evidence['native']['A']['wand_refused']
                assert evidence['native']['B']['attacks']['remote_starts'] >= 3
            assert evidence['native']['A']['combat']['last_cancelled_reason'] == 'EquipmentRequired'
            assert evidence['native']['A']['combat']['cancelled'] == 1
            if not projectile:
                assert evidence['native']['A']['unrelated_pipeline_pending_frames'] > 0
            for account in accounts: account.close()
            server.send_signal(signal.SIGINT)
            assert server.wait(timeout=15) == 0, 'Owned melee authority did not stop cleanly'
            text = server_log.read_text()
            transport = next(json.loads(line.removeprefix('transport_summary ')) for line in text.splitlines() if line.startswith('transport_summary '))
            ready_actors = set()
            for line in text.splitlines():
                if not line.startswith('chunk_ready '):
                    continue
                fields = dict(field.split('=', 1) for field in line.split()[1:])
                actor = int(fields['actor'])
                if int(fields['chunks']):
                    ready_actors.add(actor)
                else:
                    assert actor not in ready_actors, 'Cosmetic rendering withdrew a ready terrain footprint'
            assert len(ready_actors) == 2
            assert transport['max_players'] == 2 and transport['queued_commands'] > 0
            assert transport['cast_requests_accepted'] >= 4 and transport['cast_requests_rejected'] >= 2
            assert transport['max_event_queue_bytes'] <= 8192 and transport['max_movement_queue_bytes'] <= 8192
            if outcome is not None:
                outcomes = transport['hit_outcomes']
                assert outcomes['resolved'] == outcomes['random_draws'] == 3
                for name, field in [('hit', 'hits'), ('miss', 'misses'), ('deflect', 'deflects')]:
                    assert outcomes[field] == (3 if name == outcome else 0)
                for report in evidence['native'].values():
                    assert report['expected_outcome'] == outcome
                    assert report['health'] == (40 if outcome == 'hit' else 100)
                    if outcome != 'hit':
                        assert report['labels']['spawned_misses' if outcome == 'miss' else 'spawned_deflects'] == 3
                        assert report['labels']['visible_'+outcome+'_frames'] > 0
                        assert report['labels']['dropped'] == 0
            if projectile:
                travel = transport['projectiles']
                assert travel['launched'] == travel['impacts'] == 3 and travel['peak_active'] <= 1
                assert all(travel[name] == 0 for name in ['expired','obstructed','unavailable','invalid_sources','invalid_targets','capacity_refusals'])
            evidence['transport'] = transport
            for path in [server_log, first[1]/'client.log', second[1]/'client.log']:
                text = path.read_text()
                assert all(secret not in text for secret in private if secret), 'Private fixture material appeared in a public log'
        finally:
            for process in children:
                if process.poll() is None:
                    process.send_signal(signal.SIGINT)
                    try: process.wait(timeout=5)
                    except subprocess.TimeoutExpired: process.kill(); process.wait()
            for account in accounts: account.close()
            if server is not None and server.poll() is None:
                server.send_signal(signal.SIGINT)
                try: server.wait(timeout=15)
                except subprocess.TimeoutExpired: server.kill(); server.wait()
            for path in tickets: path.unlink(missing_ok=True)
            for handle in handles: handle.close()
    evidence.update(passed=True, owned_provider_and_database_stopped=identity.server is None and cluster.server is None,
                    private_tickets_removed=all(not path.exists() for path in tickets))
    assert evidence['owned_provider_and_database_stopped'] and evidence['private_tickets_removed']
    (output/'verification.json').write_text(json.dumps(evidence, indent=2, sort_keys=True)+'\n')
    return evidence


def argument_parser(description=__doc__):
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binaries', type=Path, required=True)
    parser.add_argument('--players', type=Path, default=Path('build/synty-character-combat-stage-v2'))
    parser.add_argument('--creature-asset', type=Path, default=Path('build/creature-wolf-stage-v1/wolf.glb'))
    parser.add_argument('--terrain', type=Path, default=Path('build/graybox-terrain-v1'))
    parser.add_argument('--navigation', type=Path, default=Path('build/navigation-terrain-far-v1'))
    parser.add_argument('--identity-sdk', type=Path, default=Path('build/identity-sdk'))
    parser.add_argument('--postgres-sdk', type=Path, default=Path('build/postgres-sdk'))
    parser.add_argument('--window-size', default='1920x1200')
    parser.add_argument('--world-input',action='store_true',help='Use native mesh left-click selection and right-click basic sword attack')
    parser.add_argument('--outcome', choices=['hit','miss','deflect'], help='Exercise a certain server-side result; no client-selected roll')
    return parser


if __name__ == '__main__':
    args = vars(argument_parser().parse_args())
    args = {key: value.resolve() if isinstance(value, Path) else value for key, value in args.items()}
    result = run(**args)
    print(json.dumps({'passed': result['passed'], 'output': str(args['output'])}))
