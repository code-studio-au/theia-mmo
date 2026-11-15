"""Owned native editor account Start/create/select/Stop acceptance with private services."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import struct
import sys
import time
import urllib.parse
from edit_zone import build_environment
from private_identity import Identity
from private_postgres import Cluster
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from gpu_validation import check_gpu_diagnostics
from account_population_playtest import profile_role


def run(args):
    if args.quest_travel and not args.gameplay_template:
        raise ValueError('--quest-travel requires the starter gameplay template')
    args.creature_loot = args.creature_loot or bool(args.gameplay_template)
    expected_revision = 2 if args.quest_travel else (5 if args.gameplay_template else (4 if args.creature_loot else 2))
    expected_xp = 75 if args.quest_travel else (50 if args.gameplay_template else 25)
    out = args.output.resolve()
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    incoming = args.zone.read_bytes()
    document = json.loads(incoming)
    document['test_players'] = []
    results = []
    original_bag = None
    with Identity(out/'provider', Path('build/identity-sdk'), token_seconds=1200, interactive=True) as identity, \
            Cluster(out/'cluster', Path('build/postgres-sdk')) as cluster:
        for resumed in (False, True):
            if resumed:
                cluster.restart()
            folder = out/('resumed' if resumed else 'fresh')
            folder.mkdir()
            game = folder/'game'
            game.mkdir()
            source = folder/'source.zone.json'
            source.write_text(json.dumps(document, indent=2)+'\n')
            source_bytes = source.read_bytes()
            report = folder/'state.json'
            destination = folder/'package'
            env = {k:v for k,v in build_environment(os.environ).items() if not k.startswith('THEIA_')}
            env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
                VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', WGPU_VALIDATION='1', RUST_LOG='info',
                THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                THEIA_ACCOUNT_SESSION_SECONDS='1200', THEIA_OIDC_ISSUER=identity.issuer,
                THEIA_OIDC_CLIENT_ID=identity.client_id,
                THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                THEIA_TEST_BROWSER_ISSUER=identity.issuer, THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                THEIA_TEST_BROWSER_USER=identity.users[0][0], THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',
                THEIA_PARTICLE_EDITOR_DATA_DIR=str(folder/'particle-data'),
                THEIA_WORLD_INPUT_OS_QUALIFY=str(game))
            if args.fantasy_ui:
                env['THEIA_FANTASY_UI'] = str(args.fantasy_ui.resolve())
            if not resumed and not args.quest_travel:
                env.update(THEIA_POPULATION_TYPED_QUALIFY=str(game), THEIA_POPULATION_KILL_QUALIFY='1',
                           THEIA_POPULATION_REWARD_QUALIFY='1')
                if args.creature_loot:
                    env['THEIA_POPULATION_LOOT_QUALIFY'] = '1'
            desktop = None
            with (folder/'editor.log').open('w') as log:
                editor = subprocess.Popen([str(args.binary.resolve()), '--zone', str(source), '--report', str(report),
                    '--player-assets', str(args.players.resolve()), '--creature-definition', str(args.creature.resolve()),
                    '--creature-asset', str(args.creature_asset.resolve()),
                    *(['--scenery-library', str(args.scenery_library.resolve())] if args.scenery_library else []),
                    *(['--population-access-path', '1201'] if args.quest_travel else []),
                    *(['--gameplay-template', str(args.gameplay_template.resolve())] if args.gameplay_template else [])], env=env, stdout=log, stderr=log)
                try:
                    window = wait(lambda: owned_window(editor.pid), 60)
                    desktop = Desktop()
                    if args.quest_travel: desktop.prepare_wheel()
                    desktop.focus(window)
                    def state():
                        assert editor.poll() is None, 'Owned editor exited unexpectedly'
                        return wait(lambda: json.loads(report.read_text()), 2)
                    wait(lambda: state().get('menus'), 60)
                    previous, settled = None, time.monotonic()
                    def stable_geometry():
                        nonlocal previous, settled
                        geometry = [(m['name'],m['centre'],m['size']) for m in state()['menus'] if m['role']=='button']
                        if geometry != previous:
                            previous, settled = geometry, time.monotonic()
                        return time.monotonic()-settled >= .75
                    wait(stable_geometry, 20)
                    def fresh():
                        old = state()['input_elapsed']
                        return wait(lambda: state() if state()['input_elapsed'] > old else None)
                    def click(point):
                        desktop.move(window, point, 1)
                        fresh()
                        desktop.owned_focus(pointer=True)
                        desktop.button(1, True)
                        desktop.button(1, False)
                        fresh()
                    def type_text(point, value):
                        click(point)
                        desktop.key_state('Control_L', True)
                        fresh()
                        desktop.key('a')
                        fresh()
                        desktop.key_state('Control_L', False)
                        fresh()
                        for char in str(value):
                            desktop.key({'/':'slash','.':'period','-':'minus'}.get(char,char))
                            fresh()
                    def zone_menu():
                        click(next(m['centre'] for m in state()['menus'] if m['name']=='Zone' and m['role']=='button' and min(m['size'])>0))
                    def action(name):
                        return wait(lambda: next((c for c in state()['workspace_controls'] if c['target']==name and c['menu_item'] and min(c['size'])>0),None))
                    def picker_action(name):
                        return wait(lambda: next((c for c in (state().get('account_picker') or {}).get('controls',[]) if c['action']==name and min(c['size'])>0),None))
                    zone_menu()
                    click(action('AccountPopulationPlaytest')['centre'])
                    wait(lambda: state()['export_dialog'])
                    field = wait(lambda: next((f for f in state()['export_dialog']['fields'] if f['index']==0 and min(f['size'])>0),None))
                    type_text(field['centre'], destination)
                    assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index']==0)==str(destination)
                    if args.gameplay_template:
                        def selected_template():
                            fields = state()['export_dialog']['fields']
                            return next((f for f in fields if f['index']==10 and min(f['size'])>0
                                and f['text']==str(args.gameplay_template.resolve())),None) if all(min(f['size'])==0 for f in fields if f['index'] in [8,9]) else None
                        wait(selected_template)
                        desktop.key('F12')
                        wait(lambda: capture_ready(report.with_suffix('.png')),30)
                        report.with_suffix('.png').rename(folder/'selected-template.png')
                    elif args.creature_loot:
                        for index, path in [(8, 'content/gameplay/development-loot-items.json'),
                                            (9, 'content/gameplay/development-creature-loot-rewards.json')]:
                            field = next(f for f in state()['export_dialog']['fields'] if f['index']==index)
                            type_text(field['centre'], path)
                            assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index']==index)==path
                    if args.quest_travel:
                        assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index']==12)==''
                        def test_road(value):
                            # Tab reveal reaches fields even when the dialog body scrolls.
                            for _ in range(14):
                                field = next(f for f in state()['export_dialog']['fields'] if f['index']==12)
                                if field['focused']: break
                                desktop.key('Tab'); fresh()
                            assert field['focused']
                            fresh(); fresh()
                            field = next(f for f in state()['export_dialog']['fields'] if f['index']==12)
                            type_text(field['centre'], value)
                            assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index']==12)==str(value)
                        test_road(999999)
                        click(next(b['centre'] for b in state()['export_dialog']['buttons'] if b['action']=='Export'))
                        wait(lambda: 'AI test road guide ID is absent' in state()['export_dialog']['message'])
                        assert not destination.exists() and not state()['network_playtest']['active']
                        test_road(1203)
                        desktop.key('F12'); wait(lambda: capture_ready(report.with_suffix('.png')),30)
                        report.with_suffix('.png').rename(folder/'selected-ai-road.png')
                    click(next(b['centre'] for b in state()['export_dialog']['buttons'] if b['action']=='Export'))
                    wait(lambda: state()['network_playtest']['active'], 90)
                    owned = wait(lambda: next((p for p in (destination/'runs').glob('session-*') if p.is_dir()),None), 45)
                    def picker_ready():
                        value = state().get('account_picker')
                        return value if value and value.get('name') and min(value['name']['size'])>0 else None
                    picker = wait(picker_ready, 90)
                    desktop.key('F12')
                    wait(lambda: capture_ready(report.with_suffix('.png')), 30)
                    report.with_suffix('.png').rename(folder/'picker.png')
                    picker = state()['account_picker']
                    if resumed:
                        assert len(picker['characters'])==1 and picker['characters'][0].lower()=='editorarden', picker
                        click(picker_action('Existing(0)')['centre'])
                    else:
                        assert picker['characters']==[] and picker['can_create']
                        type_text(picker['name']['centre'], 'bad--name')
                        click(picker_action('Create')['centre'])
                        wait(lambda: (state().get('account_picker') or {}).get('message'))
                        assert not (owned/'runtime/selection.json').exists()
                        type_text(state()['account_picker']['name']['centre'], 'editorarden')
                        assert state()['account_picker']['name']['text']=='editorarden'
                        desktop.key('F12')
                        wait(lambda: capture_ready(report.with_suffix('.png')), 30)
                        report.with_suffix('.png').rename(folder/'corrected-name.png')
                        click(picker_action('Create')['centre'])
                    wait(lambda: state().get('account_picker') is None, 15)
                    def gameplay():
                        value = wait(lambda: json.loads((game/'input-os.json').read_text()), 2)
                        assert value.get('connected'), 'Owned client disconnected'
                        return value
                    def initial_gameplay():
                        value = json.loads((game/'input-os.json').read_text())
                        targets_ready = (args.quest_travel and resumed) or len(value.get('combat_targets') or [])==5
                        return value if value.get('connected') and targets_ready and value.get('inventory_bag') and (value.get('minimap') or {}).get('known_pixels',0)>0 else None
                    observed = wait(initial_gameplay, 90)
                    warm = time.monotonic()
                    wait(lambda: gameplay() if time.monotonic()-warm>=10 else None, 20)
                    quest_id = 'theia.dev.quest-wolf-pelts'
                    def quest():
                        return next((q for q in gameplay()['quest_journal']['quests'] if q['id']==quest_id),None)
                    def quest_action(status):
                        game_window = wait(lambda: owned_window(gameplay()['pid']))
                        desktop.focus(game_window)
                        wait(lambda: gameplay()['cursor']['focused'])
                        if not gameplay()['dialogue_open']:
                            desktop.click(game_window, wait(lambda: gameplay()['guide_screen']), gameplay()['window'][2], 3)
                            wait(lambda: gameplay()['dialogue_open'])
                        before = gameplay()['quest_journal']['ui']['applied']
                        point = wait(lambda: next((b['point'] for b in gameplay()['quest_journal']['buttons'] if b['quest']==quest_id),None))
                        desktop.click(game_window, point, gameplay()['window'][2], 1)
                        wait(lambda: quest() and quest()['status']==status and gameplay()['quest_journal']['ui']['applied']==before+1)
                        desktop.key('Escape')
                        wait(lambda: not gameplay()['dialogue_open'])
                        if gameplay()['inventory_open']:
                            desktop.key('i')
                            wait(lambda: not gameplay()['inventory_open'])
                    if args.quest_travel:
                        from quest_travel_os import flow
                        game_window = wait(lambda: owned_window(gameplay()['pid']))
                        assert not gameplay()['test_controller']['active']
                        assert gameplay()['test_controller']['travel']['elapsed_seconds']==0
                        flow(desktop, game_window, gameplay, game, resumed=resumed)
                    if args.gameplay_template and not args.quest_travel and not resumed:
                        quest_action('Active')
                        wait(lambda: gameplay()['minimap']['objective_regions']==1)
                    if not resumed and not args.quest_travel:
                        (game/'combat.start').write_text('start\n')
                        def killed():
                            proof = json.loads((game/'typed-combat.json').read_text())
                            assert proof['passed'],proof
                            return proof
                        proof = wait(killed, 100)
                        assert proof['remaining']==0 and proof['reward_collected'] and proof['bystanders_unchanged']
                        if args.gameplay_template:
                            wait(lambda: quest() and quest()['status']=='Ready')
                            assert all(count==1 for count in quest()['counts'].values())
                            quest_action('Completed')
                    def committed():
                        value = gameplay()
                        return value if value and value.get('committed_progression')==expected_xp and value.get('available_rewards')==0 and value['inventory_bag']['revision']==expected_revision else None
                    observed = wait(committed, 20)
                    if args.gameplay_template and not args.quest_travel:
                        assert quest()['status']=='Completed' and all(count==1 for count in quest()['counts'].values())
                        assert not any(slot and slot['item']=='theia.dev.wolf-pelt' for slot in observed['inventory_bag']['slots'])
                    if resumed:
                        assert observed['inventory_bag']==original_bag
                    else:
                        original_bag = observed['inventory_bag']
                    (game/'capture.request').write_text('capture\n')
                    wait(lambda: capture_ready(game/'input-os.png'), 20)
                    desktop.focus(window)
                    fresh()
                    zone_menu()
                    click(action('StopNetworkPlaytest')['centre'])
                    wait(lambda: not state()['network_playtest']['active'], 45)
                    assert 'failed' not in state()['status'].lower(),state()['status']
                    assert source.read_bytes()==source_bytes
                    exported_zone=json.loads((destination/'zone.json').read_text())
                    editor_zone=state()['document']
                    # Zone files serialize f32 heights with short decimals;
                    # report Values widen them to f64. Compare their exact f32 bits.
                    for zone in [exported_zone,editor_zone]:
                        if zone.get('terrain'):
                            zone['terrain']['heights']=[struct.pack('<f',h) for h in zone['terrain']['heights']]
                    assert exported_zone==editor_zone
                    exported=json.loads((destination/'population-export.json').read_text())['population_export']
                    assert exported['account_playtest'] and exported['clients']==1 and exported['count']==5
                    if args.gameplay_template:
                        manifest = destination/exported['gameplay_manifest']
                        roles = json.loads(manifest.read_text())
                        frozen = json.loads((owned/'launch.json').read_text())
                        assert len(frozen['profiles'])==(24 if args.quest_travel else 22)
                        assert frozen['gameplay_snapshot']['manifest_schema']==(21 if args.quest_travel else 19)
                        if args.quest_travel:
                            assert exported['test_travel']['road']==1203 and exported['test_travel']['authored_points']==2
                            route=json.loads((destination/'test-route.json').read_text())
                            assert route['start_paused'] and route['return_trip'] and len(route['waypoints'])==2
                            assert frozen['test_route']
                        assert frozen['guide_position']=='authored'
                        for profile in frozen['profiles']:
                            data = (owned/profile['file']).read_bytes()
                            assert hashlib.sha256(data).hexdigest()==profile['sha256']
                            assert data==(manifest.parent/roles[profile_role(profile['option'])]).read_bytes()
                    assert not (destination/'scenario.json').exists()
                    assert not list((owned/'runtime').glob('renet-*'))
                    server=(owned/'runtime/server.log').read_text()
                    account=next(json.loads(line.removeprefix('account_summary ')) for line in server.splitlines() if line.startswith('account_summary '))
                    assert account['verified_logins']==1 and account['created_characters']==(0 if resumed else 1)
                    saved=next(json.loads(line.removeprefix('account_persistence_summary ')) for line in server.splitlines() if line.startswith('account_persistence_summary '))
                    assert not saved['failed'] and saved['worker']['incompatible_gameplay_profiles']==0
                    if args.quest_travel:
                        sql=cluster.query('SELECT h.revision,p.experience,(SELECT count(*) FROM theia.quest_journal_entries q WHERE q.realm_id=h.realm_id AND q.character_id=h.character_id AND q.status=2) FROM theia.quest_journal_heads h JOIN theia.progression p USING(realm_id,character_id);')
                        assert sql.returncode==0 and sql.stdout.strip()=='9|75|3'
                    else:
                        sql=cluster.query('SELECT experience FROM theia.progression; SELECT consumed_revision FROM theia.reward_claims ORDER BY consumed_revision;')
                        assert sql.returncode==0 and sql.stdout.splitlines()==([str(expected_xp),'2','3','4'] if args.creature_loot else [str(expected_xp),'2'])
                    (folder/'database-proof.txt').write_text(sql.stdout)
                    desktop.key('F12')
                    wait(lambda: capture_ready(report.with_suffix('.png')),30)
                    report.with_suffix('.png').rename(folder/'stopped.png')
                    for file in [folder/'editor.log',owned/'runtime/server.log',owned.with_suffix('.launcher.log')]:
                        text=file.read_text()
                        assert check_gpu_diagnostics(text)==0
                        assert identity.password not in text and cluster.password not in text
                    results.append(dict(resumed=resumed, editor_start=True, picker='select' if resumed else 'create',
                                        editor_stop=True, experience=expected_xp, inventory_revision=expected_revision, source_unchanged=True,
                                        authored_quest_completed=bool(args.gameplay_template),
                                        paused_test_road=1203 if args.quest_travel else None,
                                        invalid_name_refused=not resumed))
                finally:
                    if desktop:
                        desktop.close()
                    if editor.poll() is None:
                        editor.send_signal(signal.SIGINT)
                        try: editor.wait(timeout=15)
                        except subprocess.TimeoutExpired:
                            editor.kill();editor.wait(timeout=10)
                    # Closing the editor also closes every inherited owner pipe.
                    wait(lambda: not list(destination.glob('runs/session-*/runtime/renet-*')),30)
    assert args.zone.read_bytes()==incoming
    assert not any((out/'cluster'/name).exists() for name in ['password','pgpass'])
    result=dict(passed=True,runs=results,database_restart=True,owned_services_closed=True,
                scope=('Actual editor export/create/select/Stop, explicit paused compass AI road and ordinary escort/quest input, durable restart' if args.quest_travel else 'Actual native editor menu, export, character creation/selection, gameplay and Stop; optional authored quest acceptance/turn-in; OS input restricted to owned editor and quest UI, no gameplay movement/camera automation'))
    (out/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--creature-loot', action='store_true', help='Select ordinary loot catalogues in the editor dialog')
    parser.add_argument('--gameplay-template', type=Path, help='Select the explicit one-kill/one-pelt schema-19 template and complete its authored quest')
    parser.add_argument('--quest-travel', action='store_true', help='Use the starter template and explicit paused road 1203; complete compass/escort and restart')
    parser.add_argument('--scenery-library', type=Path)
    parser.add_argument('--fantasy-ui', type=Path)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--zone',type=Path,required=True)
    parser.add_argument('--creature',type=Path,default=Path('content/gameplay/development-creature-passive.json'))
    parser.add_argument('--creature-asset',type=Path,default=Path('build/creature-wolf-stage-v1/wolf.glb'))
    parser.add_argument('--players',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args())
