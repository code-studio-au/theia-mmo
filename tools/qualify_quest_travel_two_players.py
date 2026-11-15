"""Two authenticated native players complete private travel quests, then rejoin after SQL restart.

Uses ordinary owned-window input and the checked compass follower. No pose writes,
progress grants, fake connections or shared-party-credit claim.
"""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import urllib.parse
import urllib.request

from account_login import AccountClient
from edit_zone import build_environment
from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from private_identity import Identity, _NoRedirect
from private_postgres import Cluster
from qualify_population_competing_accounts import stop
from qualify_world_input_os import Desktop, owned_window, wait
from quest_travel_os import flow, RANGER


def retained(state):
    journal = state['quest_journal']
    return dict(revision=journal['revision'], xp=journal['xp'], quests=journal['quests'],
                bag=state['inventory_bag'])


def observe_background(observer, ticks):
    # Restored dialogue is intentionally short. Explicitly sample live updates
    # instead of requiring incidental UI calls to supply enough observations.
    def advancing():
        observer()
        return len(ticks) > 10
    wait(advancing, 5)


def run(args):
    started = time.monotonic()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False, mode=0o700)
    package, binaries = args.package.resolve(), args.binaries.resolve()
    manifest = package/'world/gameplay/gameplay.manifest.json'
    route = package/'test-route.json'
    assert manifest.is_file() and route.is_file(), 'Requires an exported account quest-travel package'
    sources = [package/name for name in ['population.json', 'zone.json', 'test-route.json']]
    sources += sorted(path for path in manifest.parent.rglob('*') if path.is_file())
    hashes = {path: hashlib.sha256(path.read_bytes()).hexdigest() for path in sources}
    character_ids = None
    previous = None
    runs = []
    with Identity(out/'provider', Path('build/identity-sdk'), users=2, token_seconds=1800, interactive=True) as identity, \
            Cluster(out/'cluster', Path('build/postgres-sdk')) as cluster:
        for resumed in (False, True):
            if resumed:
                cluster.restart()
            print('Quest travel: restored accounts' if resumed else 'Quest travel: fresh accounts', flush=True)
            folder = out/('resumed' if resumed else 'fresh')
            owner, guest = folder/'owner', folder/'guest'
            owner.mkdir(parents=True); guest.mkdir()
            runtime = owner/'owned/runtime'
            env = {k: v for k, v in build_environment(os.environ).items() if not k.startswith('THEIA_')}
            env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
                LD_LIBRARY_PATH=str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()),
                VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', WGPU_VALIDATION='1', RUST_LOG='info',
                THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                THEIA_ACCOUNT_SESSION_SECONDS='1800', THEIA_OIDC_ISSUER=identity.issuer,
                THEIA_OIDC_CLIENT_ID=identity.client_id,
                THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                THEIA_TEST_BROWSER_ISSUER=identity.issuer, THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                THEIA_TEST_BROWSER_USER=identity.users[0][0], THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',
                THEIA_WORLD_INPUT_OS_QUALIFY=str(owner))
            launcher = second = account = desktop = None
            ticket = folder/'guest.token'
            try:
                with (owner/'launcher.log').open('w') as log, (guest/'client.log').open('w') as guest_log:
                    launcher = subprocess.Popen([sys.executable, 'tools/account_population_playtest.py',
                        '--package', str(package), '--players', str(args.players.resolve()),
                        '--binaries', str(binaries), '--output', str(owner/'owned'),
                        '--gameplay-manifest', str(manifest), '--test-route', str(route),
                        '--fantasy-ui', str(args.fantasy_ui.resolve()), '--window-size', '1920x1440'],
                        env=env, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT)
                    def choices_ready():
                        assert launcher.poll() is None, 'Owned account launcher exited'
                        return json.loads((runtime/'characters.json').read_text())
                    choices = wait(choices_ready, 90)
                    if resumed:
                        assert [c['id'] for c in choices['characters']] == [character_ids['QuestOwner']]
                        selection = dict(schema=1, character_id=character_ids['QuestOwner'])
                    else:
                        assert not choices['characters'] and choices['can_create']
                        selection = dict(schema=1, create_name='QuestOwner')
                    pending = runtime/'selection.pending'
                    pending.write_text(json.dumps(selection)); pending.replace(runtime/'selection.json')
                    def state(directory, alive):
                        def sample():
                            assert alive(), 'Owned client exited'
                            return json.loads((directory/'input-os.json').read_text())
                        return wait(sample, 3)
                    def report_ready(directory, alive):
                        assert alive(), 'Owned process exited before its client report; inspect its log'
                        return (directory/'input-os.json').exists()
                    state_a = lambda: state(owner, lambda: launcher.poll() is None)
                    wait(lambda: report_ready(owner, lambda: launcher.poll() is None), 90)
                    a = wait(lambda: value if (value := state_a()).get('connected') and value.get('inventory_bag') and value.get('quest_journal') else None, 90)
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
                    character = character_ids['QuestGuest'] if resumed else account.choose_character(read=lambda _: 'QuestGuest', write=lambda _: None)
                    address = account.enter(character, ticket)
                    client_args = [part.decode() for part in Path(f'/proc/{a["pid"]}/cmdline').read_bytes().split(b'\0') if part]
                    index = client_args.index('--renet-connect')
                    client_args[index+1:index+3] = [address, str(ticket)]
                    second_env = client_environment({k: v for k, v in env.items() if not k.startswith('THEIA_')})
                    second_env['THEIA_WORLD_INPUT_OS_QUALIFY'] = str(guest)
                    second = subprocess.Popen(client_args, env=second_env, stdout=guest_log, stderr=guest_log)
                    state_b = lambda: state(guest, lambda: second.poll() is None)
                    wait(lambda: report_ready(guest, lambda: second.poll() is None), 90)
                    b = wait(lambda: value if (value := state_b()).get('connected') and value.get('inventory_bag') and value.get('quest_journal') else None, 90)
                    wait(lambda: len((state_a().get('public_avatar_life') or {}).get('actors', [])) == 2 and len((state_b().get('public_avatar_life') or {}).get('actors', [])) == 2, 30)
                    for observe in [state_a, state_b]:
                        wait(lambda: {RANGER, 'theia.dev.villager', 'theia.dev.steward'}.issubset(
                            c['npc'] for c in observe().get('contacts', [])), 45)
                    assert a['actor'] != b['actor']
                    windows = [wait(lambda: owned_window(a['pid']), 30), wait(lambda: owned_window(b['pid']), 30)]
                    for value in [state_a(), state_b()]:
                        assert value['window'][:2] == [1920., 1440.], value['window']
                    print('Both authenticated clients ready at 1920x1440', flush=True)
                    desktop = Desktop()
                    untouched = retained(state_b())
                    def peer_ranger():
                        return wait(lambda: next((c for c in state_b()['contacts'] if c['npc'] == RANGER), None), 5)['global']
                    home_b = peer_ranger()
                    isolation_samples = 0
                    peer_ticks = [set(), set()]
                    def isolated_a():
                        nonlocal isolation_samples
                        peer = state_b()
                        assert peer['connected'], 'Background player disconnected'
                        peer_ticks[0].add(peer['dodge']['tick'])
                        assert retained(peer) == untouched, 'Player one changed player two progress'
                        if not resumed:
                            assert math.dist(peer_ranger(), home_b) < .02, 'Player one moved player two ranger'
                        isolation_samples += 1
                        return state_a()
                    proof_a = flow(desktop, windows[0], isolated_a, owner, resumed=resumed,
                                   check_banner=args.route_banner and not resumed)
                    observe_background(isolated_a, peer_ticks[0])
                    print('First player verified; second player progress unchanged', flush=True)
                    complete_a = retained(state_a())
                    def isolated_b():
                        nonlocal isolation_samples
                        peer = state_a()
                        assert peer['connected'], 'Background player disconnected'
                        peer_ticks[1].add(peer['dodge']['tick'])
                        assert retained(peer) == complete_a, 'Player two changed player one progress'
                        isolation_samples += 1
                        return state_b()
                    proof_b = flow(desktop, windows[1], isolated_b, guest, resumed=resumed)
                    observe_background(isolated_b, peer_ticks[1])
                    print('Second player verified; first player progress unchanged', flush=True)
                    completed = [retained(state_a()), retained(state_b())]
                    for value in completed:
                        assert value['revision'] == 9 and value['xp'] == 75
                        assert len(value['quests']) == 3 and all(q['status'] == 'Completed' for q in value['quests'])
                        assert value['bag']['revision'] == 2
                    if resumed:
                        assert completed == previous
                    else:
                        previous = completed
                    assert all(len(ticks) > 10 for ticks in peer_ticks), 'Background observation did not advance'
                    runs.append(dict(resumed=resumed, window_sizes=[[1920,1440],[1920,1440]],
                        isolation_samples=isolation_samples, retained=completed,
                        distinct_background_ticks=[len(ticks) for ticks in peer_ticks],
                        proofs=[proof_a, proof_b]))
            finally:
                if desktop: desktop.close()
                stop(second)
                if account: account.close()
                if launcher:
                    if launcher.poll() is None:
                        launcher.stdin.close()
                        try: launcher.wait(timeout=30)
                        except subprocess.TimeoutExpired: stop(launcher)
                    launcher.stdin.close()
                ticket.unlink(missing_ok=True)
                if (runtime/'server.log').exists():
                    shutil.copy2(runtime/'server.log', folder/'server.log')
            assert launcher.returncode == 0
            server = (folder/'server.log').read_text()
            def summary(prefix):
                return next(json.loads(line.removeprefix(prefix)) for line in server.splitlines() if line.startswith(prefix))
            saved = summary('account_persistence_summary ')
            transport = summary('transport_summary ')
            accounts = summary('account_summary ')
            assert not saved['failed'] and saved['worker']['inventory_ensures'] == 2
            assert accounts['verified_logins'] == 2 and accounts['created_characters'] == (0 if resumed else 2)
            assert transport['max_players'] == 2 and transport['public_attack_backpressure_disconnects'] == 0
            runs[-1]['transport'] = transport
            names = cluster.query('SELECT display_name,character_id FROM theia.character_names ORDER BY display_name;')
            assert names.returncode == 0
            found = dict(line.split('|') for line in names.stdout.splitlines())
            if character_ids: assert found == character_ids
            character_ids = found
            rows = cluster.query('''SELECT n.display_name,h.revision,p.experience,
                (SELECT count(*) FROM theia.quest_journal_entries q WHERE q.realm_id=h.realm_id AND q.character_id=h.character_id AND q.status=2)
                FROM theia.character_names n JOIN theia.quest_journal_heads h USING(realm_id,character_id)
                JOIN theia.progression p USING(realm_id,character_id) ORDER BY n.display_name;''')
            assert rows.returncode == 0 and rows.stdout.splitlines() == ['QuestGuest|9|75|3', 'QuestOwner|9|75|3'], rows.stdout
            (folder/'database-proof.txt').write_text(rows.stdout)
            print('Both character journals and XP verified in PostgreSQL', flush=True)
            for log in [owner/'launcher.log', guest/'client.log']:
                assert check_gpu_diagnostics(log.read_text()) == 0
                adapters = [line for line in log.read_text().splitlines() if 'AdapterInfo' in line]
                assert adapters
                runs[-1].setdefault('gpu', []).extend(adapters)
    assert all(hashlib.sha256(path.read_bytes()).hexdigest() == digest for path, digest in hashes.items())
    assert not any((out/'cluster'/name).exists() for name in ['password','pgpass'])
    result = dict(passed=True, elapsed_seconds=time.monotonic()-started,
        authenticated_native_clients=2, database_restart=True,
        source_unchanged=True, runs=runs, scope=__doc__)
    (out/'qualification.json').write_text(json.dumps(result, indent=2)+'\n')
    print('Two-account quest travel and database restart passed', flush=True)
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--package', type=Path, required=True)
    parser.add_argument('--players', type=Path, required=True)
    parser.add_argument('--fantasy-ui', type=Path, required=True)
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--route-banner', action='store_true', help='Check dialogue clearance at both playtest sizes and activate outward travel with the banner')
    run(parser.parse_args())
