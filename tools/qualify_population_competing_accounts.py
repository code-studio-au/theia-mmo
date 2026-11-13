"""Two authenticated native population clients: threat transfer, owner loot and durable rejoin.

Uses the normal account population wrapper and gameplay requests. The opt-in
client qualification driver equips/casts/collects; no OS input, pose writes or
prepared damage facts. PostgreSQL and OIDC services are owned disposable fixtures.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import time
import urllib.parse
import urllib.request

from account_login import AccountClient
from edit_zone import build_environment
from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from player_assets import default_player_assets
from private_identity import Identity, _NoRedirect
from private_postgres import Cluster
from qualify_world_input_os import wait, capture_ready


def read(folder, alive):
    assert alive(), 'Owned client/launcher exited; inspect its log'
    return json.loads((folder/'input-os.json').read_text())


def stop(child):
    if child is not None and child.poll() is None:
        child.send_signal(signal.SIGINT)
        try:
            child.wait(timeout=20)
        except subprocess.TimeoutExpired:
            child.kill(); child.wait(timeout=10)


def run(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False, mode=0o700)
    package, binaries = args.package.resolve(), args.binaries.resolve()
    previous_bags = None
    character_ids = None
    runs = []
    with Identity(out/'provider', Path('build/identity-sdk'), users=2, token_seconds=600, interactive=True) as identity, \
            Cluster(out/'cluster', Path('build/postgres-sdk')) as cluster:
        for resumed in (False, True):
            if resumed:
                cluster.restart()
            folder = out/('resumed' if resumed else 'fresh'); folder.mkdir()
            owner = folder/'owner'; owner.mkdir()
            finisher = folder/'finisher'; finisher.mkdir()
            runtime = owner/'owned/runtime'
            env = {k:v for k,v in build_environment(os.environ).items() if not k.startswith('THEIA_')}
            env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
                LD_LIBRARY_PATH=str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()),
                VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', WGPU_VALIDATION='1', RUST_LOG='info',
                THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                THEIA_ACCOUNT_SESSION_SECONDS='600', THEIA_OIDC_ISSUER=identity.issuer,
                THEIA_OIDC_CLIENT_ID=identity.client_id,
                THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                THEIA_TEST_BROWSER_ISSUER=identity.issuer, THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                THEIA_TEST_BROWSER_USER=identity.users[0][0], THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',
                THEIA_WORLD_INPUT_OS_QUALIFY=str(owner))
            if not resumed:
                env.update(THEIA_POPULATION_TYPED_QUALIFY=str(owner), THEIA_POPULATION_REWARD_QUALIFY='1',
                           THEIA_POPULATION_LOOT_QUALIFY='1')
            launcher = second = account = None
            ticket = folder/'finisher.token'
            try:
                with (owner/'launcher.log').open('w') as log, (finisher/'client.log').open('w') as second_log:
                    launcher = subprocess.Popen([sys.executable, 'tools/account_population_playtest.py',
                        '--package', str(package), '--players', str(args.players.resolve()), '--binaries', str(binaries),
                        '--output', str(owner/'owned'), '--items', str(Path('content/gameplay/development-loot-items.json').resolve()),
                        '--reward-catalog', str(Path('content/gameplay/development-creature-loot-rewards.json').resolve())],
                        env=env, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT)
                    def choices_ready():
                        assert launcher.poll() is None
                        return json.loads((runtime/'characters.json').read_text())
                    choices = wait(choices_ready, 60)
                    if resumed:
                        assert [c['id'] for c in choices['characters']] == [character_ids['PoolOwner']]
                        selection = dict(schema=1, character_id=character_ids['PoolOwner'])
                    else:
                        assert not choices['characters'] and choices['can_create']
                        selection = dict(schema=1, create_name='PoolOwner')
                    pending = runtime/'selection.pending'; pending.write_text(json.dumps(selection)); pending.replace(runtime/'selection.json')
                    state_a = lambda: read(owner, lambda: launcher.poll() is None)
                    a = wait(lambda: state_a() if state_a().get('connected') and state_a().get('public_population_attack') and state_a().get('inventory_bag') else None, 90)
                    ready = next(json.loads(line.removeprefix('account_game_ready '))
                                 for line in (runtime/'server.log').read_text().splitlines() if line.startswith('account_game_ready '))
                    base = ready['account_address']
                    account = AccountClient(base, alive=lambda: launcher.poll() is None)
                    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), _NoRedirect())
                    def browser(url):
                        callback = identity.authorize(url, user_index=1)
                        with opener.open(base+'/callback?'+urllib.parse.urlsplit(callback).query, timeout=8) as response:
                            assert response.status == 200 and response.headers.get('Cache-Control') == 'no-store'
                        return True
                    account.login(open_browser=browser)
                    character = character_ids['PoolFinisher'] if resumed else account.choose_character(read=lambda _: 'PoolFinisher', write=lambda _: None)
                    address = account.enter(character, ticket)
                    client_args = [part.decode() for part in Path(f'/proc/{a["pid"]}/cmdline').read_bytes().split(b'\0') if part]
                    assert '--public-creature-attacks' in client_args and '--population-config' in client_args
                    index = client_args.index('--renet-connect')
                    client_args[index+1:index+3] = [address, str(ticket)]
                    second_env = client_environment(env)
                    for key in list(second_env):
                        if key.startswith('THEIA_POPULATION_'):
                            second_env.pop(key)
                    second_env['THEIA_WORLD_INPUT_OS_QUALIFY'] = str(finisher)
                    # Choose the already nearby NPC from the first client's public
                    # registry. The driver still validates range/facing/liveness
                    # and sends ordinary selection/cast requests to authority.
                    if not resumed:
                        def nearby():
                            value = state_a()
                            p = value['dodge']['authority_feet']
                            candidates = [t['entity'] for t in value['target_entities']
                                          if sum((t['position'][i]-p[i])**2 for i in (0,2)) < 2.95**2
                                          and t['position'][2] < p[2]]
                            return min(candidates) if candidates else None
                        target = wait(nearby, 20)
                        second_env.update(THEIA_POPULATION_TYPED_QUALIFY=str(finisher),
                                          THEIA_POPULATION_KILL_QUALIFY='1',
                                          THEIA_POPULATION_TARGET_QUALIFY=str(target))
                    second = subprocess.Popen(client_args, env=second_env, stdout=second_log, stderr=subprocess.STDOUT)
                    state_b = lambda: read(finisher, lambda: second.poll() is None)
                    b = wait(lambda: state_b() if state_b().get('connected') and state_b().get('snapshot_players') == 2 and state_b().get('inventory_bag') else None, 90)
                    assert a['actor'] != b['actor']
                    started = time.monotonic()
                    wait(lambda: state_a() if time.monotonic()-started >= 10 else None, 20)
                    transfer = False
                    private_windup = False
                    remote_animation = False
                    if not resumed:
                        (owner/'combat.start').write_text('start\n')
                        def tagged():
                            value = state_a()
                            selected = value.get('selected_hostile')
                            assert selected in (None, target), (selected, target)
                            return next((h for h in value['combat_targets'] if h['entity'] == target and 0 < h['current'] < h['maximum']), None)
                        wait(tagged, 30)
                        (finisher/'combat.start').write_text('start\n')
                        def completed():
                            nonlocal transfer, private_windup, remote_animation
                            a, b = state_a(), state_b()
                            for value in (a,b):
                                assert value['connected'] and value['combat']['player_health'] > 0
                            pending = next((v['windup'] for v in a['public_population_attack']['actors'] if v['creature'] == target), None)
                            transfer |= bool(pending and pending['target'] == b['actor'])
                            private_windup |= b['combat']['incoming_windups'] > 0
                            remote_animation |= any(v['entity'] == target and v['clip'] == 3 and v['remote_victim'] for v in a['creature_animations'])
                            if pending and pending['target'] == b['actor']:
                                (folder/'transfer-sample.json').write_text(json.dumps(dict(owner=a, finisher=b), indent=2)+'\n')
                            proofs = []
                            for directory in (owner, finisher):
                                try:
                                    proof = json.loads((directory/'typed-combat.json').read_text())
                                except (FileNotFoundError, json.JSONDecodeError):
                                    return None
                                assert proof['passed'], proof
                                proofs.append(proof)
                            return proofs
                        proofs = wait(completed, 90)
                        assert transfer and private_windup and remote_animation
                        assert proofs[0]['target'] == proofs[1]['target'] == target
                        assert proofs[0]['corpse_pickups'] == 2 and proofs[0]['casts'] >= 1
                        assert proofs[1]['remaining'] == 0 and proofs[1]['corpse_pickups'] == 0
                    def settled():
                        a, b = state_a(), state_b()
                        return (a,b) if (a['committed_progression'] == 25 and b['committed_progression'] == 0
                            and a['inventory_bag']['revision'] == 4 and b['inventory_bag']['revision'] == 1
                            and a['available_rewards'] == b['available_rewards'] == 0) else None
                    a, b = wait(settled, 15)
                    assert a['committed_progression'] == 25 and b['committed_progression'] == 0
                    assert a['inventory_bag']['revision'] == 4 and b['inventory_bag']['revision'] == 1
                    assert a['available_rewards'] == b['available_rewards'] == 0
                    bags = [a['inventory_bag'], b['inventory_bag']]
                    if resumed:
                        assert bags == previous_bags
                    else:
                        previous_bags = bags
                    for directory in (owner, finisher):
                        (directory/'capture.request').write_text('competition\n')
                        wait(lambda: capture_ready(directory/'input-os.png'), 20)
                    runs.append(dict(resumed=resumed, transferred=transfer, private_finisher_windup=private_windup,
                                     remote_owner_animation=remote_animation, owner_xp=25, finisher_xp=0,
                                     bags=bags, proofs=proofs if not resumed else None))
            finally:
                stop(second)
                if account is not None:
                    account.close()
                if launcher is not None:
                    if launcher.poll() is None:
                        launcher.stdin.close()
                        try:
                            launcher.wait(timeout=30)
                        except subprocess.TimeoutExpired:
                            stop(launcher)
                    launcher.stdin.close()
                ticket.unlink(missing_ok=True)
                if (runtime/'server.log').exists():
                    shutil.copy2(runtime/'server.log', folder/'server.log')
            assert launcher.returncode == 0
            server = (folder/'server.log').read_text()
            saved = next(json.loads(line.removeprefix('account_persistence_summary ')) for line in server.splitlines() if line.startswith('account_persistence_summary '))
            transport = next(json.loads(line.removeprefix('transport_summary ')) for line in server.splitlines() if line.startswith('transport_summary '))
            accounts = next(json.loads(line.removeprefix('account_summary ')) for line in server.splitlines() if line.startswith('account_summary '))
            assert not saved['failed'] and saved['worker']['inventory_ensures'] == 2
            assert accounts['verified_logins'] == 2 and accounts['created_characters'] == (0 if resumed else 2)
            assert transport['max_players'] == 2 and transport['public_attack_backpressure_disconnects'] == 0
            if not resumed:
                assert transport['creature_threat']['transfers'] > 0 and transport['creature_threat']['health_lost'] == 100
                assert saved['worker']['experience_committed'] == 1 and saved['worker']['reward_committed'] == 3
            else:
                assert transport['creature_threat']['health_lost'] == 0
            runs[-1]['transport'] = transport
            names = cluster.query('SELECT display_name,character_id FROM theia.character_names ORDER BY display_name;')
            assert names.returncode == 0
            character_ids = dict(line.split('|') for line in names.stdout.splitlines())
            rows = cluster.query('''
                SELECT n.display_name,p.experience,
                  (SELECT count(*) FROM theia.reward_claims r WHERE r.realm_id=n.realm_id AND r.character_id=n.character_id),
                  (SELECT count(*) FROM theia.reward_claims r WHERE r.realm_id=n.realm_id AND r.character_id=n.character_id AND consumed_revision IS NOT NULL)
                FROM theia.character_names n JOIN theia.progression p USING(realm_id,character_id)
                ORDER BY n.display_name;
                SELECT count(*) FROM theia.experience_receipts;
                SELECT count(*),count(DISTINCT victim_life) FROM theia.reward_claims WHERE claim_scope=1;
            ''')
            assert rows.returncode == 0 and rows.stdout.splitlines() == ['PoolFinisher|0|0|0', 'PoolOwner|25|3|3', '1', '2|1'], rows.stdout
            (folder/'database-proof.txt').write_text(rows.stdout)
            for log in (owner/'launcher.log', finisher/'client.log'):
                assert check_gpu_diagnostics(log.read_text()) == 0
                adapters = [line for line in log.read_text().splitlines() if 'AdapterInfo' in line]
                assert adapters, 'Missing actual GPU device evidence'
                runs[-1].setdefault('gpu', []).extend(adapters)
    result = dict(passed=True, database_restart=True, authenticated_native_clients=2, runs=runs,
                  mouse_or_keyboard_automation=False, scope=__doc__)
    result['binaries'] = {name: hashlib.sha256((binaries/name).read_bytes()).hexdigest()
                          for name in ('theia-client','theia-server')}
    assert not any((out/'cluster'/name).exists() for name in ('password','pgpass'))
    (out/'qualification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--players', type=Path, default=default_player_assets())
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    print(json.dumps(run(parser.parse_args()), indent=2))
