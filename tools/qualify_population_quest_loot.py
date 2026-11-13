"""Normal population journal/conditional loot through owned OIDC, SQL and native UI.

An explicit one-pelt fixture isolates eligibility; editor exports are unchanged.
Quest accept/turn-in use PID-bound OS clicks. Combat/equipment/pickup use the
existing test-only driver sending ordinary requests, with no authoritative writes.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import urllib.parse

from account_population_playtest import PROFILES, RESET_PROFILE, THREAT_PROFILE
from edit_zone import build_environment
from gpu_validation import check_gpu_diagnostics
from player_assets import default_player_assets
from private_identity import Identity
from private_postgres import Cluster
from qualify_world_input_os import Desktop, wait, owned_window, capture_ready

QUEST = 'theia.dev.quest-wolf-pelts'


def run(args):
    out = args.output.resolve()
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    package, binaries = args.population.resolve(), args.binaries.resolve()
    journal = json.loads(Path('content/quests/development-journal-items.json').read_text())
    quest = next(q for q in journal['quests'] if q['id'] == QUEST)
    quest['prerequisites'] = []
    quest['description'] = 'Population qualification: bring one wolf pelt to the guide.'
    objective = quest['stages'][0]['objectives'][0]
    objective.update(required=1, label='Bring one wolf pelt')
    kill_objective = QUEST+'.objective.hunt'
    if args.kill_objective:
        quest['stages'][0]['objectives'].append(dict(kind='kill', id=kill_objective,
            label='Defeat one training creature', creature='theia.dev.training-creature', required=1))
        quest['description'] = 'Population qualification: defeat one creature and bring its pelt to the guide.'
    journal['quests'] = [quest]
    journal['id'] = 'theia.dev.population-quest-loot-fixture'
    (out/'journal.json').write_text(json.dumps(journal, indent=2)+'\n')
    profiles = []
    for flag, filename in [*PROFILES, RESET_PROFILE, THREAT_PROFILE]:
        if flag == '--items':
            filename = 'development-loot-items.json'
        elif flag == '--reward-catalog':
            filename = 'development-quest-required-loot-fixture.json'
        profiles.extend([flag, str((Path('content/gameplay')/filename).resolve())])
    access = json.loads((package/'population.json').read_text())['access_from']
    guide = f'{access[0]+1},{access[1]},{access[2]-1}'
    base_env = build_environment(os.environ)
    desktop = Desktop()
    results = []
    try:
        with Identity(out/'provider', Path('build/identity-sdk'), token_seconds=600, interactive=True) as identity, \
                Cluster(out/'cluster', Path('build/postgres-sdk')) as cluster:
            character = None
            previous_bag = None
            denied_roll = None
            for phase in range(3):
                if phase:
                    cluster.restart()
                folder = out/['unaccepted', 'accepted', 'resumed'][phase]
                folder.mkdir()
                runtime = folder/'owned/runtime' if args.account_package else folder/'runtime'
                env = {key: value for key, value in base_env.items() if not key.startswith('THEIA_')}
                env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
                    VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', WGPU_VALIDATION='1', RUST_LOG='info',
                    THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                    THEIA_ACCOUNT_SESSION_SECONDS='600', THEIA_OIDC_ISSUER=identity.issuer,
                    THEIA_OIDC_CLIENT_ID=identity.client_id,
                    THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                    THEIA_TEST_BROWSER_ISSUER=identity.issuer, THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                    THEIA_TEST_BROWSER_USER=identity.users[0][0], THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                    BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',
                    THEIA_WORLD_INPUT_OS_QUALIFY=str(folder))
                if phase < 2:
                    env.update(THEIA_POPULATION_TYPED_QUALIFY=str(folder), THEIA_POPULATION_KILL_QUALIFY='1',
                               THEIA_POPULATION_LOOT_QUALIFY='1', THEIA_POPULATION_LOOT_COUNT_QUALIFY=str(phase+1))
                    if phase == 0:
                        env['THEIA_POPULATION_REWARD_QUALIFY'] = '1'
                    elif args.kill_objective:
                        env['THEIA_POPULATION_WAIT_LOOT_QUALIFY'] = '1'
                command = [sys.executable, 'tools/play.py', '--authored-world', str(package/'world'),
                    '--population-config', str(package/'population.json'), '--classic', '--account-login',
                    '--equipment', '--status-effects', '--progression', '--rewards', '--public-avatar-life',
                    '--public-creature-attacks', '--creature-loot', '--quest-journal', '--quest-items', '--quest-loot',
                    '--quest-catalog', str(out/'journal.json'), '--guide-position', guide,
                    '--stop-on-stdin-eof', '--session-directory', str(runtime),
                    *(['--character-id', character] if character else ['--character-picker']),
                    *profiles, '--players', str(args.players.resolve()), '--binaries', str(binaries),
                    '--creature-asset', str(package/'creature.glb'), '--window-size', '1280x720']
                if args.account_package:
                    command = [sys.executable, 'tools/account_population_playtest.py',
                        '--package', str(package), '--players', str(args.players.resolve()),
                        '--binaries', str(binaries), '--output', str(folder/'owned'),
                        '--items', str(Path('content/gameplay/development-loot-items.json').resolve()),
                        '--reward-catalog', str(Path('content/gameplay/development-quest-required-loot-fixture.json').resolve()),
                        '--quest-catalog', str(out/'journal.json'), '--guide-position', guide,
                        '--dialogue-catalog', str(Path('content/quests/development-journal-dialogue.json').resolve()),
                        *(['--character-id', character] if character else [])]
                with (folder/'launcher.log').open('w') as log:
                    launcher = subprocess.Popen(command, env=env, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT)
                    try:
                        def state():
                            assert launcher.poll() is None, 'Population quest launcher exited'
                            return json.loads((folder/'input-os.json').read_text())
                        if character is None:
                            def picker():
                                assert launcher.poll() is None, 'Population quest picker exited'
                                return json.loads((runtime/'characters.json').read_text())
                            choices = wait(picker, 60)
                            assert choices['can_create'] and not choices['characters']
                            pending = runtime/'selection.pending'
                            pending.write_text(json.dumps(dict(schema=1, create_name='PeltArden')))
                            pending.replace(runtime/'selection.json')
                        observed = wait(lambda: (value if value['connected'] and value.get('quest_journal')
                            and value.get('inventory_bag') and len(value.get('combat_targets') or []) == 5
                            and (value.get('creature_visuals') or {}).get('players') == 5 else None)
                            if (value := state()) else None, 90)
                        start = time.monotonic()
                        wait(lambda: state() if time.monotonic()-start >= 10 else None, 20)
                        if previous_bag:
                            assert state()['inventory_bag'] == previous_bag
                        window = wait(lambda: owned_window(state()['pid']))
                        desktop.focus(window)
                        wait(lambda: state()['cursor']['focused'])

                        def progress():
                            return next((q for q in state()['quest_journal']['quests'] if q['id'] == QUEST), None)

                        def action(status):
                            assert owned_window(state()['pid']) == window
                            if not state()['dialogue_open']:
                                desktop.click(window, wait(lambda: state()['guide_screen']), state()['window'][2], 3)
                                wait(lambda: state()['dialogue_open'])
                            before = state()['quest_journal']['ui']['applied']
                            point = wait(lambda: next((b['point'] for b in state()['quest_journal']['buttons'] if b['quest'] == QUEST), None))
                            desktop.click(window, point, state()['window'][2], 1)
                            wait(lambda: progress() and progress()['status'] == status
                                 and state()['quest_journal']['ui']['applied'] == before+1)
                            desktop.key('Escape')
                            wait(lambda: not state()['dialogue_open'])

                        if phase == 1:
                            assert progress() is None
                            action('Active')
                            assert all(value == 0 for value in progress()['counts'].values())
                        elif phase == 2:
                            assert progress()['status'] == 'Completed'
                        else:
                            assert progress() is None
                        combat = None
                        if phase < 2:
                            (folder/'combat.start').write_text('start\n')
                            if phase == 1 and args.kill_objective:
                                wait(lambda: progress()['counts'][kill_objective] == 1
                                     and state()['committed_progression'] == 50, 80)
                                before_pickup = state()
                                assert progress()['status'] == 'Active'
                                assert progress()['counts'][objective['id']] == 0
                                assert before_pickup['inventory_bag']['revision'] == 3
                                assert not any(s and s['item'] == 'theia.dev.wolf-pelt' for s in before_pickup['inventory_bag']['slots'])
                                assert sum(t['current'] == 0 for t in before_pickup['combat_targets']) == 1
                                (folder/'kill-before-pickup.json').write_text(json.dumps(before_pickup, indent=2)+'\n')
                                (folder/'loot.start').write_text('collect\n')
                            def completed():
                                state()
                                value = json.loads((folder/'typed-combat.json').read_text())
                                assert value['passed'], value
                                return value
                            combat = wait(completed, 100)
                            assert combat['corpse_pickups'] == phase+1
                            wait(lambda: state()['committed_progression'] == 25*(phase+1))
                            if phase == 1:
                                wait(lambda: progress() and progress()['status'] == 'Ready')
                                action('Completed')
                        expected_xp, expected_revision, expected_tokens = (25, 3, 101) if phase == 0 else (75, 6, 102)
                        observed = wait(lambda: value if (value := state())['committed_progression'] == expected_xp
                            and value['inventory_bag']['revision'] == expected_revision else None)
                        bag = observed['inventory_bag']
                        count = lambda item: sum(s['count'] for s in bag['slots'] if s and s['item'] == item)
                        assert count('theia.dev.training-token') == expected_tokens
                        assert count('theia.dev.wolf-pelt') == 0
                        previous_bag = bag
                        if phase and args.kill_objective:
                            assert progress()['counts'][kill_objective] == 1
                            assert progress()['counts'][objective['id']] == 1
                        (folder/'capture.request').write_text('capture\n')
                        wait(lambda: capture_ready(folder/'input-os.png'))
                        results.append(dict(phase=phase, combat=combat, experience=expected_xp, bag=bag,
                            quest=progress(), health=observed['combat']['player_health']))
                    finally:
                        if launcher.poll() is None:
                            launcher.stdin.close()
                            try:
                                launcher.wait(timeout=30)
                            except subprocess.TimeoutExpired:
                                launcher.kill()
                                launcher.wait(timeout=10)
                        launcher.stdin.close()
                    assert launcher.returncode == 0
                selected = cluster.query('SELECT character_id FROM theia.characters;')
                if args.account_package:
                    frozen = json.loads((folder/'owned/launch.json').read_text())
                    assert frozen['quest_journal'] and frozen['quest_items'] and frozen['quest_loot']
                    assert len(frozen['profiles']) == 20 and frozen['guide_position'] == guide
                    for profile in frozen['profiles']:
                        assert hashlib.sha256((folder/'owned'/profile['file']).read_bytes()).hexdigest() == profile['sha256']
                assert selected.returncode == 0 and len(selected.stdout.splitlines()) == 1
                if character:
                    assert selected.stdout.strip() == character
                character = selected.stdout.strip()
                audit = cluster.query("""SELECT encode(victim_life,'hex'),generation_eligible,
                    generation_bag_revision,generation_journal_revision FROM theia.loot_rolls
                    WHERE roll_key='entry:theia.dev.wolf-pelt-loot' ORDER BY generation_bag_revision;""")
                assert audit.returncode == 0, audit.stderr
                rows = audit.stdout.splitlines()
                assert len(rows) == (1 if phase == 0 else 2), audit.stdout
                assert rows[0].split('|')[1] == 'f'
                assert all(len(row.split('|')) == 4 and all(row.split('|')[2:]) for row in rows)
                if phase:
                    assert rows[0] == denied_roll and rows[1].split('|')[1] == 't'
                    assert rows[0].split('|')[0] != rows[1].split('|')[0]
                else:
                    denied_roll = rows[0]
                (folder/'generation-rights.txt').write_text(audit.stdout)
                if args.kill_objective:
                    journal_rows = cluster.query('SELECT quest_id,status,encode(state,\'hex\') FROM theia.quest_journal_entries ORDER BY quest_id;')
                    assert journal_rows.returncode == 0, journal_rows.stderr
                    if phase == 0:
                        assert not journal_rows.stdout.strip()
                    else:
                        assert len(journal_rows.stdout.splitlines()) == 1
                        assert journal_rows.stdout.split('|')[:2] == [QUEST, '2']
                    (folder/'journal-sql.txt').write_text(journal_rows.stdout)
                claims = cluster.query('SELECT count(*),count(consumed_revision) FROM theia.reward_claims;')
                assert claims.returncode == 0 and claims.stdout.strip() == ('2|2' if phase == 0 else '4|4'), claims.stdout
                server = (runtime/'server.log').read_text()
                summary = lambda prefix: next(json.loads(line.removeprefix(prefix+' ')) for line in server.splitlines() if line.startswith(prefix+' '))
                saved = summary('account_persistence_summary')
                assert not saved['failed'] and saved['worker']['incompatible_gameplay_profiles'] == 0
                transport = summary('transport_summary')
                assert transport['ability_damage']['arcane']['health_lost'] == (100 if phase < 2 else 0)
                results[-1]['transport'] = transport
                results[-1]['persistence'] = saved
                for path in [folder/'launcher.log', *runtime.glob('*.log')]:
                    contents = path.read_text(errors='replace')
                    assert identity.password not in contents and cluster.password not in contents
                    assert check_gpu_diagnostics(contents) == 0
    finally:
        desktop.close()
    assert not any((out/'cluster'/name).exists() for name in ['password', 'pgpass'])
    result = dict(passed=True, runs=results, postgres_process_restarts=2, pid_bound_quest_clicks=True, frozen_account_package=args.account_package, accepted_kill_objective=args.kill_objective,
        fixture='Explicit one-pelt journal; normal pursuing editor population and ordinary gameplay requests',
        binaries={name: hashlib.sha256((binaries/name).read_bytes()).hexdigest() for name in ['theia-client', 'theia-server']})
    (out/'qualification.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(passed=True, output=str(out))))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--population', type=Path, required=True)
    parser.add_argument('--players', type=Path, default=default_player_assets())
    parser.add_argument('--binaries', type=Path, default=Path('target/debug'))
    parser.add_argument('--account-package', action='store_true', help='Qualify the frozen account wrapper instead of direct normal play')
    parser.add_argument('--kill-objective', action='store_true', help='Also require an accepted kill, observing its credit before corpse pickup')
    run(parser.parse_args())
