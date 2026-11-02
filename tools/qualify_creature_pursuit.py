#!/usr/bin/env python3
"""Owned OIDC/PostgreSQL and two actual native clients: opt-in creature pursuit."""
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
        identity_sdk, postgres_sdk, window_size="1920x1200"):
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    client = binaries / 'theia-client'
    authority = binaries / 'account_native_qualification'
    assert client.is_file() and authority.is_file()
    creature = output / 'qualification-creature.json'
    data = json.loads(Path('content/gameplay/development-creature-pursuit.json').read_text())
    # Keep the first acquisition deterministic across actual browser/GPU startup;
    # the saved wait must still resume after return, inside the forty-second run.
    data['wait_ms'] = 15000
    creature.write_text(json.dumps(data, indent=2) + '\n')
    retaliation = output / 'qualification-retaliation.json'
    data = json.loads(Path('content/gameplay/development-creature-retaliation.json').read_text())
    data['damage'] = 1
    retaliation.write_text(json.dumps(data, indent=2) + '\n')
    stats = Path('content/gameplay/development-character-stats.json').resolve()
    respawn = Path('content/gameplay/development-player-respawn.json').resolve()
    items = Path('content/gameplay/development-equipment-items.json').resolve()
    abilities = Path('content/gameplay/development-abilities.json').resolve()
    rewards = Path('content/gameplay/development-rewards.json').resolve()
    evidence = {'scope': 'Actual local OIDC accounts, owned PostgreSQL, encrypted native Linux clients; no physical keyboard/browser or Windows claim',
                'client_sha256': hashlib.sha256(client.read_bytes()).hexdigest(),
                'server_sha256': hashlib.sha256(authority.read_bytes()).hexdigest(),
                'window_size': window_size, 'fixture': {'patrol_wait_ms': 15000, 'damage': 1, 'leash_metres': 35}}
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
                   THEIA_TEST_RESPAWN_PROFILE=str(respawn), THEIA_TEST_NAVIGATION=str(navigation))
        for key in ['THEIA_TEST_DIALOGUE', 'THEIA_TEST_VENDORS', 'THEIA_TEST_PROGRESSION',
                    'THEIA_TEST_QUESTS', 'THEIA_TEST_QUEST_LIFECYCLE', 'THEIA_TEST_QUEST_TURN_IN_XP']:
            env[key] = '0'
        try:
            log = server_log.open('w'); handles.append(log)
            server = subprocess.Popen([str(authority)], env=env, stdout=log, stderr=subprocess.STDOUT)
            def ready():
                assert server.poll() is None, 'Owned pursuit authority exited before readiness'
                return next((json.loads(line.removeprefix('account_game_ready ')) for line in server_log.read_text().splitlines() if line.startswith('account_game_ready ')), None)
            base = wait(ready, 25, 'Pursuit authority did not become ready')['account_address']
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
                name = 'PursuitArden' if index == 0 else 'PursuitSelene'
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
                                 THEIA_GRAYBOX_QUALIFY=str(folder), THEIA_GRAYBOX_QUALIFY_SECONDS='40',
                                 THEIA_CREATURE_PURSUIT_QUALIFY=variant, VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
                                 VK_LOADER_DEBUG='layer', WGPU_VALIDATION='1', RUST_LOG='info')
                for key in list(child_env):
                    if key.startswith('THEIA_') and key.endswith(('_AUTOPLAY', '_QUALIFY')) and key not in ['THEIA_GRAYBOX_QUALIFY', 'THEIA_CREATURE_PURSUIT_QUALIFY']:
                        child_env.pop(key)
                log = (folder/'client.log').open('w'); handles.append(log)
                args = [str(client), '--saved-terrain', str(terrain), '--prediction', '--player-assets', str(players),
                        '--creature-asset', str(creature_asset), '--window-size', window_size,
                        '--item-catalog', str(items), '--reward-catalog', str(rewards),
                        '--creature-profile', str(creature.resolve()), '--navigation', str(navigation),
                        '--combat-catalog', str(abilities), '--character-stats', str(stats),
                        '--creature-retaliation', str(retaliation.resolve()), '--player-respawn', str(respawn),
                        '--renet-connect', address, str(ticket)]
                process = subprocess.Popen(args, env=child_env, stdout=log, stderr=subprocess.STDOUT)
                children.append(process)
                return process, folder
            first = native('A', a, ca)
            def attacked():
                report = read(first[1]/'pursuit.json')
                assert first[0].poll() is None, 'First native client exited before perception/melee'
                return report if report and report['attacked'] else None
            wait(attacked, 18, 'First native client was not perceived and attacked')
            second = native('B', b, cb)
            evidence['native'] = {}
            width, height = map(int, window_size.split('x'))
            for variant, (process, folder) in [('A', first), ('B', second)]:
                def complete():
                    report = read(folder/'pursuit.json')
                    assert process.poll() is None or process.returncode == 0, 'Native pursuit client failed'
                    capture = folder/'pursuit.png'
                    if not report or not report['passed'] or not capture.exists() or capture.stat().st_size < 24:
                        return None
                    header = capture.read_bytes()[:24]
                    assert header[:8] == b'\x89PNG\r\n\x1a\n' and struct.unpack('>II', header[16:24]) == (width, height)
                    return report
                evidence['native'][variant] = wait(complete, 40, 'Native pursuit tour/capture did not complete')
            for process in children:
                assert process.wait(timeout=45) == 0, 'Native pursuit did not exit cleanly'
            for variant, (_, folder) in [('A', first), ('B', second)]:
                text = (folder/'client.log').read_text()
                assert 'VK_LAYER_KHRONOS_validation' in text and check_gpu_diagnostics(text) == 0
                evidence['native'][variant]['gpu_diagnostics'] = 0
            assert evidence['native']['A']['wolf']['attack_frames'] >= 20
            assert evidence['native']['B']['minimum_health'] == 100
            for account in accounts: account.close()
            server.send_signal(signal.SIGINT)
            assert server.wait(timeout=15) == 0, 'Owned pursuit authority did not stop cleanly'
            text = server_log.read_text()
            transport = next(json.loads(line.removeprefix('transport_summary ')) for line in text.splitlines() if line.startswith('transport_summary '))
            metrics = transport['creature_pursuit']
            assert metrics['acquisitions'] == metrics['returns'] == metrics['arrivals'] == 1, metrics
            assert metrics['published'] > 0 and metrics['chase_ticks'] > 0 and metrics['attack_ticks'] > 0
            assert metrics['failed'] == metrics['blocked'] == 0
            assert transport['max_players'] == 2 and transport['queued_commands'] > 0
            assert transport['simulation']['missed_deadlines'] == 0
            assert transport['max_event_queue_bytes'] <= 8192 and transport['max_movement_queue_bytes'] <= 8192
            assert transport['patrol']['mode'] in ['idle', 'patrol'] and transport['patrol']['moving_ticks'] > 0
            anchor, final = metrics['last_anchor'], transport['patrol']['position']
            assert math.hypot(final[0]-anchor[0], final[2]-anchor[2]) > 1, 'NPC did not move along its resumed patrol'
            for report in evidence['native'].values():
                for before, after in zip(report['samples'], report['samples'][1:]):
                    delta = math.hypot(after['creature'][0]-before['creature'][0], after['creature'][2]-before['creature'][2])
                    assert delta <= 5*(after['seconds']-before['seconds'])+1, 'NPC observation teleported during pursuit/return'
            beyond = [sample['health'] for sample in evidence['native']['A']['samples']
                      if math.hypot(sample['owner'][0]-anchor[0], sample['owner'][2]-anchor[2]) > 40]
            assert beyond and len(set(beyond)) == 1, 'Damage continued after moving beyond the leash'
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


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binaries', type=Path, required=True)
    parser.add_argument('--players', type=Path, default=Path('build/synty-character-combat-stage-v2'))
    parser.add_argument('--creature-asset', type=Path, default=Path('build/creature-wolf-stage-v1/wolf.glb'))
    parser.add_argument('--terrain', type=Path, default=Path('build/graybox-terrain-v1'))
    parser.add_argument('--navigation', type=Path, default=Path('build/navigation-terrain-far-v1'))
    parser.add_argument('--identity-sdk', type=Path, default=Path('build/identity-sdk'))
    parser.add_argument('--postgres-sdk', type=Path, default=Path('build/postgres-sdk'))
    parser.add_argument('--window-size', default='1920x1200')
    args = vars(parser.parse_args())
    args = {key: value.resolve() if isinstance(value, Path) else value for key, value in args.items()}
    result = run(**args)
    print(json.dumps({'passed': result['passed'], 'output': str(args['output'])}))
