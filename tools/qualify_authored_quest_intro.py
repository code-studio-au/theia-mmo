"""Owned authored-world guide quest, real OIDC/SQL, and durable restart/rejoin."""
import argparse
import json
import math
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import sys
import urllib.parse
from account_population_playtest import profile_role
from private_identity import Identity
from private_postgres import Cluster
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from qualify_quest_journal_os import flow
from gpu_validation import check_gpu_diagnostics

parser = argparse.ArgumentParser(description=__doc__)
for name in ('world', 'navigation', 'gameplay-manifest', 'players', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
parser.add_argument('--route', help='Singleton patrol route identity')
parser.add_argument('--population-package', type=Path, help='Checked complete population package; optional checked three-kill follow-up')
parser.add_argument('--sigint-stop', action='store_true', help='Qualify account SIGINT shutdown with its owner pipe still open')
parser.add_argument('--reach-area', action='store_true', help='Also complete the authored western-overlook quest')
parser.add_argument('--population-pelts', action='store_true', help='Complete normal conditional pelt pickup and turn-in after the population overlook quest')
parser.add_argument('--checked-travel', action='store_true', help='Use the checked client follower for quest travel; ordinary keys still drive combat and UI')
parser.add_argument('--quest-trail', type=Path, help='Also complete the three-kill follow-up using this checked route request')
parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
args = parser.parse_args()
if args.population_pelts and not (args.population_package and args.reach_area and args.checked_travel and args.quest_trail):
    parser.error('--population-pelts requires a population package, overlook quest and checked travel')
if args.sigint_stop and not args.population_package:
    parser.error('--sigint-stop requires a population account package')
if args.population_package:
    package = args.population_package.resolve()
    if args.route:
        parser.error('Population qualification excludes singleton patrol routes')
    if args.quest_trail and not args.checked_travel:
        parser.error('Population follow-up requires checked travel')
    if args.world.resolve() != package/'world' or args.navigation.resolve() != package/'navigation':
        parser.error('Population introduction must use its package world/navigation')
    if args.gameplay_manifest.resolve() != package/'world/gameplay/gameplay.manifest.json':
        parser.error('Population introduction must use its exported gameplay snapshot')
elif not args.route:
    parser.error('Singleton introduction requires --route')
if args.checked_travel and not args.quest_trail:
    parser.error('--checked-travel requires --quest-trail')
if args.reach_area and not args.quest_trail:
    parser.error('--reach-area requires --quest-trail')
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False, mode=0o700)
bundle = out / 'gameplay'
shutil.copytree(args.gameplay_manifest.resolve().parent, bundle)
manifest = bundle / args.gameplay_manifest.name
document = json.loads(manifest.read_text())
creature = (bundle / document['creature']).resolve()
assert creature.is_relative_to(bundle)
profile = json.loads(creature.read_text())
if args.route:
    profile['route'] = args.route
    creature.write_text(json.dumps(profile, indent=2) + '\n')
document['id'] = 'theia.starter.valley.gameplay'
document['revision'] = 1
manifest.write_text(json.dumps(document, indent=2) + '\n')
# Existing damage, speeds, rewards and quest semantics remain unchanged.
query = '''SELECT h.revision,p.experience,
    (SELECT count(*) FROM theia.quest_journal_entries q WHERE q.realm_id=h.realm_id AND q.character_id=h.character_id AND q.status=2)
    FROM theia.quest_journal_heads h JOIN theia.progression p USING(realm_id,character_id);'''
area = None
if args.reach_area:
    area_catalog = json.loads((bundle / document['quest_areas']).read_text())
    area = next(a for a in area_catalog['areas'] if a['id'] == 'theia.dev.area.western-overlook')
trail = None
if args.quest_trail:
    result = subprocess.run([str(args.binaries.resolve() / 'theia-nav-bake'), 'route-authored',
        str(args.world.resolve()), str(args.navigation.resolve()), str(args.quest_trail.resolve())],
        check=True, capture_output=True, text=True, timeout=60)
    (out / 'route-check.log').write_text(result.stdout)
    trail = json.loads(result.stdout.splitlines()[-1])['authored_navigation_route']['route']['points']
test_route = None
if args.checked_travel:
    test_route = out / 'test-route.json'
    test_route.write_text(json.dumps(dict(schema=1, name='Village to western grove',
        navigation=str(args.navigation.resolve()), waypoints=trail, start_paused=True, return_trip=True),indent=2)+'\n')
os.environ.setdefault('DISPLAY', ':0')
x = Desktop()
x.prepare_wheel()
launcher = None
results = []
previous_bag = None
previous_rolls = None
try:
    with Identity(out / 'provider', Path('build/identity-sdk'), token_seconds=1800, interactive=True) as identity, Cluster(out / 'cluster', Path('build/postgres-sdk'), isolate_process=True) as cluster:
        def loot_audit(life):
            assert len(life) == 16
            encoded = bytes(life).hex()
            # Only observe committed receipts. No SQL gameplay mutations or
            # timing assumptions about an absent corpse after a probability miss.
            result = cluster.query(f"""SELECT json_build_object(
                'rolls',(SELECT json_agg(json_build_object('key',roll_key,'ticket',ticket,
                    'eligible',generation_eligible,'bag',generation_bag_revision,
                    'journal',generation_journal_revision) ORDER BY roll_key)
                    FROM theia.loot_rolls WHERE victim_life=decode('{encoded}','hex')),
                'pending',(SELECT count(*) FROM theia.reward_claims
                    WHERE victim_life=decode('{encoded}','hex') AND consumed_revision IS NULL));""")
            assert result.returncode == 0, result.stderr
            value = json.loads(result.stdout)
            return value if len(value['rolls'] or []) == 2 else None

        for resumed in (False, True):
            folder = out / ('resumed' if resumed else 'fresh')
            folder.mkdir()
            if resumed:
                before = cluster.query(query)
                assert before.returncode == 0
                cluster.restart()
                after = cluster.query(query)
                assert after.returncode == 0 and after.stdout == before.stdout
            env = {k:v for k,v in os.environ.items() if not k.startswith('THEIA_')}
            env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
                VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
                LD_LIBRARY_PATH=str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()),
                THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                THEIA_ACCOUNT_SESSION_SECONDS='1800',
                THEIA_OIDC_ISSUER=identity.issuer, THEIA_OIDC_CLIENT_ID=identity.client_id,
                THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                THEIA_TEST_BROWSER_ISSUER=identity.issuer, THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                THEIA_TEST_BROWSER_USER=identity.users[0][0], THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',
                THEIA_WORLD_INPUT_OS_QUALIFY=str(folder))
            runtime = folder/'owned/runtime' if args.population_package else Path('build/development-session')
            existing_tickets = set(runtime.glob('renet-*'))
            with (folder / 'launcher.log').open('w') as log:
                command = [sys.executable, 'tools/play.py', '--account-login', '--classic',
                    '--binaries', str(args.binaries.resolve()), '--players', str(args.players.resolve()),
                    '--authored-world', str(args.world.resolve()), '--navigation', str(args.navigation.resolve()),
                    '--gameplay-manifest', str(manifest), '--gameplay-validator', str(args.binaries.resolve() / 'examples/validate_gameplay'),
                    '--window-size', '1280x720', *(['--test-route',str(test_route)] if test_route else [])]
                if args.population_package:
                    command = [sys.executable, 'tools/account_population_playtest.py',
                        '--package', str(package), '--players', str(args.players.resolve()),
                        '--binaries', str(args.binaries.resolve()), '--output', str(folder/'owned'),
                        '--gameplay-manifest', str(args.gameplay_manifest.resolve()),
                        *(['--test-route', str(test_route)] if test_route else [])]
                launcher = subprocess.Popen(command, env=env, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT, text=True, start_new_session=True)
                try:
                    if args.population_package:
                        def picker():
                            assert launcher.poll() is None, 'Owned population picker exited'
                            return json.loads((runtime/'characters.json').read_text())
                        choices = wait(picker, 90)
                        if resumed:
                            assert len(choices['characters']) == 1
                            selection = dict(schema=1, character_id=choices['characters'][0]['id'])
                        else:
                            assert choices['can_create'] and not choices['characters']
                            selection = dict(schema=1, create_name='ValleyArden')
                        pending = runtime/'selection.pending'
                        pending.write_text(json.dumps(selection))
                        pending.replace(runtime/'selection.json')
                    else:
                        launcher.stdin.write('1\n' if resumed else 'ValleyArden\n')
                        launcher.stdin.flush()
                    def state():
                        assert launcher.poll() is None, 'Owned launcher exited; inspect copied logs'
                        # The optional live report can be observed while being replaced.
                        return wait(lambda: json.loads((folder / 'input-os.json').read_text()), 2)
                    wait(lambda: (folder / 'input-os.json').is_file() or launcher.poll() is not None, 70)
                    initial = wait(lambda: state() if state()['connected'] and state()['quest_journal'] and state()['guide_screen'] else None, 60)
                    if args.population_package:
                        observed = wait(lambda: value if len((value := state())['combat_targets']) == 5
                            and (value.get('creature_visuals') or {}).get('attached') == 5
                            and len(value.get('creature_animations') or []) == 5 else None, 90)
                        assert all(t['current'] == t['maximum'] for t in observed['combat_targets'])
                        assert len({t['entity'] for t in observed['target_entities']}) == 5
                        (folder/'population-admission.json').write_text(json.dumps(dict(
                            targets=observed['combat_targets'], poses=observed['target_entities'],
                            animations=observed['creature_animations'], visuals=observed['creature_visuals'],
                            adapter=observed['adapter'], feet=observed['dodge']['authority_feet']), indent=2)+'\n')
                        declared = json.loads(args.gameplay_manifest.read_text())
                        records = json.loads((folder/'owned/launch.json').read_text())['profiles']
                        assert len(records) == 22
                        for record in records:
                            source = args.gameplay_manifest.parent/declared[profile_role(record['option'])]
                            assert (folder/'owned'/record['file']).read_bytes() == source.read_bytes()
                    parent = re.search(r'^PPid:\s+(\d+)', Path(f'/proc/{initial["pid"]}/status').read_text(), re.M)
                    assert int(parent[1]) == launcher.pid
                    window = wait(lambda: owned_window(initial['pid']))
                    x.focus(window)
                    subprocess.run(['wmctrl', '-i', '-r', hex(window), '-e', '0,40,40,1280,720'], check=True, timeout=3)
                    wait(lambda: state()['window'][:2] == [1280, 720] and state()['cursor']['focused']
                         and state().get('minimap', {}).get('known_pixels', 0) > 0)
                    x.move(window, [640, 360], initial['window'][2])
                    def cursor_ready():
                        position = state()['cursor']['position']
                        return position is not None and math.dist(position, [640,360]) < 2
                    wait(cursor_ready)
                    def play_state():
                        report = state()
                        assert report['connected'], 'Owned client disconnected during quest qualification'
                        return report
                    proof = flow(x, window, play_state, folder, resumed=resumed, intro_only=not bool(trail), authored_trail=trail, reach_areas=args.reach_area, authored_area=area, checked_travel=args.checked_travel, population=bool(args.population_package), population_pelts=args.population_pelts, loot_audit=loot_audit, population_pelt_revision=results[0]['revision'] if resumed and args.population_pelts else None)
                    if args.population_pelts:
                        bag = state()['inventory_bag']
                        if resumed:
                            assert bag == previous_bag, 'Completed pelt quest bag changed across restart'
                        previous_bag = bag
                        (folder/'pelt-bag.json').write_text(json.dumps(bag, indent=2)+'\n')
                    if args.population_package and not args.sigint_stop:
                        launcher.stdin.close()
                    else:
                        launcher.send_signal(signal.SIGINT)
                    assert launcher.wait(timeout=30) == 0
                    results.append(proof)
                except BaseException:
                    if launcher.poll() is None and (folder / 'input-os.json').is_file():
                        (folder / 'capture.request').write_text('failure.png\n')
                        try:
                            wait(lambda: capture_ready(folder / 'input-os.png'), 5)
                            shutil.copy2(folder / 'input-os.png', folder / 'failure.png')
                        except (TimeoutError, OSError):
                            pass
                    raise
                finally:
                    if launcher.poll() is None:
                        if args.population_package:
                            launcher.stdin.close()
                        else:
                            launcher.send_signal(signal.SIGINT)
                        try:
                            launcher.wait(timeout=30)
                        except subprocess.TimeoutExpired:
                            launcher.kill()
                            launcher.wait(timeout=10)
                    launcher.stdin.close()
                    shutil.copy2(runtime/'server.log', folder / 'server.log')
                    launcher = None
            assert not (set(runtime.glob('renet-*')) - existing_tickets)
            text = (folder / 'server.log').read_text()
            transport = next(json.loads(line.removeprefix('transport_summary ')) for line in text.splitlines() if line.startswith('transport_summary '))
            if args.population_package:
                assert transport['patrol'] is None
                assert transport['test_player_admissions'] == 0
                assert transport['dialogue_requests_rejected'] == 0
            else:
                assert transport['patrol']['moving_ticks'] > 0 and transport['patrol']['blocked'] == 0
            saved = next(json.loads(line.removeprefix('account_persistence_summary ')) for line in text.splitlines() if line.startswith('account_persistence_summary '))
            assert not saved['failed'] and saved['worker']['committed_records'] > 0
            db = cluster.query(query)
            assert db.returncode == 0
            (folder / 'database-proof.txt').write_text(db.stdout)
            assert db.stdout.strip() == f"{proof['revision']}|{proof['xp']}|{4 if args.population_pelts else 3 if area else (2 if trail else 1)}"
            if args.population_pelts:
                rolls = cluster.query("""SELECT encode(victim_life,'hex'),roll_key,ticket,
                    generation_eligible,generation_bag_revision,generation_journal_revision
                    FROM theia.loot_rolls ORDER BY victim_life,roll_key;""")
                assert rolls.returncode == 0, rolls.stderr
                (folder/'loot-rolls.txt').write_text(rolls.stdout)
                if resumed:
                    assert rolls.stdout == previous_rolls, 'Restart rerolled a saved creature life'
                previous_rolls = rolls.stdout
            if resumed:
                assert db.stdout == before.stdout
            for path in folder.glob('*.log'):
                text = path.read_text()
                assert identity.password not in text and cluster.password not in text
                assert check_gpu_diagnostics(text) == 0
finally:
    x.close()
(out / 'qualification.json').write_text(json.dumps(dict(passed=True, runs=results,
    private_services_closed=True, database_restart=True,
    stop='SIGINT with owner pipe open' if args.sigint_stop else 'owner EOF' if args.population_package else 'SIGINT',
    scope=('Dressed authored population: all four quests, normal conditional pelt rolls, ordinary corpse pickup and consumption/turn-in, checked travel and durable restart' if args.population_pelts else 'Dressed authored population: introduction, three kills, checked outward/return travel, authored overlook occupancy/turn-in and durable rejoin; no pelt quest completion' if args.population_package and area else 'Dressed authored population: five native actors, checked travel, three independent creature kills, guide follow-up and durable rejoin; no pelt/area quest completion' if args.population_package and trail else 'Dressed authored population: five native actors, complete 22-profile snapshot, guide introduction, checkpoint and PostgreSQL restart/rejoin; no population combat or travel claim' if args.population_package else 'Authored introduction, three-kill and overlook quests, ordinary input traversal, checkpoint and rejoin' if area else 'Authored introduction and three-kill follow-up, ordinary input traversal, checkpoint and rejoin; no area quest acceptance' if trail else 'One authored guide introduction quest, server patrol, checkpoint and rejoin; no wolf-kill/area quest acceptance')), indent=2))
