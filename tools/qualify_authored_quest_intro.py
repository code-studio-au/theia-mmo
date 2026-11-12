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
from private_identity import Identity
from private_postgres import Cluster
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from qualify_quest_journal_os import flow
from gpu_validation import check_gpu_diagnostics

parser = argparse.ArgumentParser(description=__doc__)
for name in ('world', 'navigation', 'gameplay-manifest', 'players', 'output'):
    parser.add_argument('--' + name, type=Path, required=True)
parser.add_argument('--route', required=True)
parser.add_argument('--reach-area', action='store_true', help='Also complete the authored western-overlook quest')
parser.add_argument('--quest-trail', type=Path, help='Also complete the three-kill follow-up using this checked route request')
parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
args = parser.parse_args()
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
os.environ.setdefault('DISPLAY', ':0')
x = Desktop()
x.prepare_wheel()
launcher = None
results = []
try:
    with Identity(out / 'provider', Path('build/identity-sdk'), token_seconds=1800, interactive=True) as identity, Cluster(out / 'cluster', Path('build/postgres-sdk')) as cluster:
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
            existing_tickets = set(Path('build/development-session').glob('renet-*'))
            with (folder / 'launcher.log').open('w') as log:
                command = [sys.executable, 'tools/play.py', '--account-login', '--classic',
                    '--binaries', str(args.binaries.resolve()), '--players', str(args.players.resolve()),
                    '--authored-world', str(args.world.resolve()), '--navigation', str(args.navigation.resolve()),
                    '--gameplay-manifest', str(manifest), '--gameplay-validator', str(args.binaries.resolve() / 'examples/validate_gameplay'),
                    '--window-size', '1280x720']
                launcher = subprocess.Popen(command, env=env, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT, text=True)
                try:
                    launcher.stdin.write('1\n' if resumed else 'ValleyArden\n')
                    launcher.stdin.flush()
                    def state():
                        assert launcher.poll() is None, 'Owned launcher exited; inspect copied logs'
                        # The optional live report can be observed while being replaced.
                        return wait(lambda: json.loads((folder / 'input-os.json').read_text()), 2)
                    wait(lambda: (folder / 'input-os.json').is_file() or launcher.poll() is not None, 70)
                    initial = wait(lambda: state() if state()['connected'] and state()['quest_journal'] and state()['guide_screen'] else None, 60)
                    parent = re.search(r'^PPid:\s+(\d+)', Path(f'/proc/{initial["pid"]}/status').read_text(), re.M)
                    assert int(parent[1]) == launcher.pid
                    window = wait(lambda: owned_window(initial['pid']))
                    x.focus(window)
                    subprocess.run(['wmctrl', '-i', '-r', hex(window), '-e', '0,40,40,1280,720'], check=True, timeout=3)
                    wait(lambda: state()['window'][:2] == [1280, 720] and state()['cursor']['focused']
                         and state().get('minimap', {}).get('known_pixels', 0) > 0)
                    x.move(window, [640, 360], initial['window'][2])
                    wait(lambda: state()['cursor']['position'] and math.dist(state()['cursor']['position'], [640,360]) < 2)
                    def play_state():
                        report = state()
                        assert report['connected'], 'Owned client disconnected during quest qualification'
                        return report
                    proof = flow(x, window, play_state, folder, resumed=resumed, intro_only=not bool(trail), authored_trail=trail, reach_areas=args.reach_area, authored_area=area)
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
                        launcher.send_signal(signal.SIGINT)
                        try:
                            launcher.wait(timeout=30)
                        except subprocess.TimeoutExpired:
                            launcher.kill()
                            launcher.wait(timeout=10)
                    launcher.stdin.close()
                    shutil.copy2('build/development-session/server.log', folder / 'server.log')
                    launcher = None
            assert not (set(Path('build/development-session').glob('renet-*')) - existing_tickets)
            text = (folder / 'server.log').read_text()
            transport = next(json.loads(line.removeprefix('transport_summary ')) for line in text.splitlines() if line.startswith('transport_summary '))
            assert transport['patrol']['moving_ticks'] > 0 and transport['patrol']['blocked'] == 0
            saved = next(json.loads(line.removeprefix('account_persistence_summary ')) for line in text.splitlines() if line.startswith('account_persistence_summary '))
            assert not saved['failed'] and saved['worker']['committed_records'] > 0
            db = cluster.query(query)
            assert db.returncode == 0
            (folder / 'database-proof.txt').write_text(db.stdout)
            assert db.stdout.strip() == f"{proof['revision']}|{proof['xp']}|{3 if area else (2 if trail else 1)}"
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
    scope=('Authored introduction, three-kill and overlook quests, ordinary input traversal, checkpoint and rejoin' if area else 'Authored introduction and three-kill follow-up, ordinary input traversal, checkpoint and rejoin; no area quest acceptance' if trail else 'One authored guide introduction quest, server patrol, checkpoint and rejoin; no wolf-kill/area quest acceptance')), indent=2))
