"""Owned native create/preview/enter/restart qualification on an authored zone."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import urllib.parse
import urllib.request
from edit_zone import build_environment
from private_identity import Identity, _NoRedirect
from account_login import AccountClient
from launch_environment import client_environment
from qualify_population_competing_accounts import stop
from private_postgres import Cluster
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from gpu_validation import check_gpu_diagnostics

EXPECTED = dict(schema=1, body=1, face=1, hair=2, facial_hair=0, skin=1, hair_colour=1, outfit=1)


def run(args):
    out = args.output.resolve()
    out.mkdir(mode=0o700, parents=True, exist_ok=False)
    original = args.zone.read_bytes()
    results = []
    with Identity(out/'provider', Path('build/identity-sdk'), users=2, token_seconds=1800, interactive=True) as identity, Cluster(out/'cluster', Path('build/postgres-sdk')) as cluster:
        for resumed in ((False,) if args.once else (False, True)):
            if resumed:
                cluster.restart()
            folder = out/('resumed' if resumed else 'fresh'); folder.mkdir()
            game = folder/'game'; game.mkdir()
            source = folder/'zone.json'; source.write_bytes(original)
            report = folder/'state.json'
            package = folder/'package'
            env = {k:v for k,v in build_environment(os.environ).items() if not k.startswith('THEIA_')}
            env.update(DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000', WGPU_BACKEND='vulkan',
                VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', WGPU_VALIDATION='1', RUST_LOG='info',
                THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                THEIA_ACCOUNT_SESSION_SECONDS='1800', THEIA_OIDC_ISSUER=identity.issuer,
                THEIA_OIDC_CLIENT_ID=identity.client_id, THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                THEIA_TEST_BROWSER_ISSUER=identity.issuer, THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                THEIA_TEST_BROWSER_USER=identity.users[0][0], THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',
                THEIA_PARTICLE_EDITOR_DATA_DIR=str(folder/'particle-data'), THEIA_WORLD_INPUT_OS_QUALIFY=str(game),
                THEIA_FANTASY_UI='/home/steven/Downloads/Synty Models/HUDs', THEIA_ZONE_MAP_DIRECTORY=str(Path('build/covenant-seven-gentle-v2').resolve()))
            command = [str(args.binary.resolve()), '--zone',str(source),'--report',str(report),
                '--player-assets',str(args.players.resolve()),'--creature-definition',str(args.creature.resolve()),
                '--creature-asset',str(args.creature_asset.resolve()),'--scenery-library',str(args.scenery_library.resolve()),
                '--gameplay-template',str(args.gameplay_template.resolve())]
            desktop = None
            guest = account = guest_log = None
            ticket = folder/"guest.token"
            with (folder/'editor.log').open('w') as log:
                editor = subprocess.Popen(command, env=env, stdout=log, stderr=log)
                try:
                    window = wait(lambda: owned_window(editor.pid), 60)
                    desktop = Desktop(); desktop.focus(window)
                    def state():
                        assert editor.poll() is None, 'Owned editor exited'
                        return json.loads(report.read_text())
                    wait(lambda: state().get('menus'),60)
                    time.sleep(1)
                    def fresh():
                        before = state()['input_elapsed']
                        return wait(lambda: state() if state()['input_elapsed']>before else None)
                    def click(point):
                        desktop.move(window,point,1); fresh(); desktop.owned_focus(pointer=True)
                        desktop.button(1,True); desktop.button(1,False); fresh()
                    def type_text(point,value):
                        click(point); desktop.key_state('Control_L',True); fresh(); desktop.key('a'); fresh(); desktop.key_state('Control_L',False); fresh()
                        for char in str(value):
                            desktop.key({'/':'slash','.':'period','-':'minus'}.get(char,char)); fresh()
                    def menu(action):
                        click(next(m['centre'] for m in state()['menus'] if m['name']=='Zone' and m['role']=='button' and min(m['size'])>0))
                        control = wait(lambda: next((c for c in state()['workspace_controls'] if c['target']==action and c['menu_item'] and min(c['size'])>0),None))
                        click(control['centre'])
                    def picker_action(action):
                        control = wait(lambda: next((c for c in (state().get('account_picker') or {}).get('controls',[]) if c['action']==action and min(c['size'])>0),None))
                        click(control['centre'])
                    def screenshot(name):
                        image=report.with_suffix('.png'); image.unlink(missing_ok=True)
                        desktop.key('F12'); wait(lambda:capture_ready(image),30); image.rename(folder/name)
                    menu('AccountPopulationPlaytest')
                    wait(lambda: state()['export_dialog'])
                    field=next(f for f in state()['export_dialog']['fields'] if f['index']==0)
                    type_text(field['centre'],package)
                    click(next(b['centre'] for b in state()['export_dialog']['buttons'] if b['action']=='Export'))
                    wait(lambda: state()['network_playtest']['active'],120)
                    wait(lambda: (state().get('account_picker') or {}).get('preview',{}).get('ready'),120)
                    if resumed:
                        picker=state()['account_picker']; assert [name.lower() for name in picker['characters']]==['heroarden'], picker
                        assert picker['appearance']==EXPECTED
                        screenshot('saved-character.png')
                        picker_action('Existing(0)'); picker_action('Enter')
                    else:
                        assert state()['account_picker']['characters']==[]
                        screenshot('default-character.png')
                        for field in [0,1,2,4,5,6]: picker_action(f'Cycle({field}, true)')
                        wait(lambda: state()['account_picker']['preview']['ready'] and state()['account_picker']['preview']['heroes'][0]['appearance']==EXPECTED)
                        picker_action('Rotate(true)'); screenshot('custom-character.png')
                        type_text(state()['account_picker']['name']['centre'],'bad--name'); picker_action('Create')
                        wait(lambda:'interior' in state()['account_picker']['message'].lower() or 'name' in state()['account_picker']['message'].lower())
                        type_text(state()['account_picker']['name']['centre'],'Heroarden'); picker_action('Create')
                    wait(lambda: state().get('account_picker') is None,15)
                    def gameplay():
                        return json.loads((game/'input-os.json').read_text())
                    observed=wait(lambda: gameplay() if gameplay().get('connected') and gameplay().get('loading',{}).get('phase')=='Playing' and any(h['appearance']==EXPECTED for h in gameplay().get('modular_heroes',[])) else None,120)
                    game_window=wait(lambda: owned_window(observed['pid']),30); desktop.focus(game_window)
                    wait(lambda: gameplay()['cursor']['focused'])
                    desktop.hold_key('q', .7)
                    (game/'capture.request').write_text('hero-in-world\n'); wait(lambda:capture_ready(game/'input-os.png'),30)
                    (game/'input-os.png').rename(game/'hero-in-world.png')
                    (folder/'gameplay.json').write_text(json.dumps(gameplay(),indent=2))
                    runtime=next(package.glob('runs/session-*/runtime'))
                    ready=next(json.loads(line.removeprefix('account_game_ready ')) for line in (runtime/'server.log').read_text().splitlines() if line.startswith('account_game_ready '))
                    base=ready['account_address']
                    account=AccountClient(base, alive=lambda:editor.poll() is None)
                    opener=urllib.request.build_opener(urllib.request.ProxyHandler({}), _NoRedirect())
                    def browser(url):
                        callback=identity.authorize(url,user_index=1)
                        with opener.open(base+'/callback?'+urllib.parse.urlsplit(callback).query,timeout=8) as response: assert response.status==200
                        return True
                    account.login(open_browser=browser)
                    other=dict(schema=1,body=0,face=2,hair=4,facial_hair=2,skin=3,hair_colour=3,outfit=0)
                    if resumed:
                        rows=account.characters(); assert len(rows)==1 and rows[0]['appearance']==other
                        character=rows[0]['id']
                    else: character=account.create_character('Herobriar',other)['id']
                    address=account.enter(character,ticket)
                    command=[part.decode() for part in Path(f'/proc/{observed["pid"]}/cmdline').read_bytes().split(b'\0') if part]
                    index=command.index('--renet-connect'); command[index+1:index+3]=[address,str(ticket)]
                    for key,value in [('--window-size','1720x1400'),('--test-window','2/2')]:
                        if key in command:command[command.index(key)+1]=value
                        else:command.extend([key,value])
                    guest_dir=folder/'guest';guest_dir.mkdir()
                    guest_env=client_environment({k:v for k,v in env.items() if not k.startswith('THEIA_')})
                    guest_env['THEIA_WORLD_INPUT_OS_QUALIFY']=str(guest_dir)
                    guest_log=(guest_dir/'client.log').open('w')
                    guest=subprocess.Popen(command,env=guest_env,stdout=guest_log,stderr=guest_log)
                    def guest_state():
                        assert guest.poll() is None
                        return json.loads((guest_dir/'input-os.json').read_text())
                    def both(state):
                        values=state().get('modular_heroes',[])
                        return state().get('loading',{}).get('phase')=='Playing' and len(values)==2 and all(any(h['appearance']==expected for h in values) for expected in [EXPECTED,other])
                    wait(lambda:both(gameplay) and both(guest_state),120)
                    guest_window=wait(lambda:owned_window(guest.pid),30);desktop.focus(guest_window)
                    (guest_dir/'capture.request').write_text('two-heroes\n');wait(lambda:capture_ready(guest_dir/'input-os.png'),30)
                    (guest_dir/'input-os.png').rename(guest_dir/'two-heroes.png')
                    (folder/'two-players.json').write_text(json.dumps([gameplay()['modular_heroes'],guest_state()['modular_heroes']],indent=2))
                    stop(guest);guest=None;guest_log.close();guest_log=None
                    account.close();account=None
                    desktop.focus(window); fresh(); menu('StopNetworkPlaytest')
                    wait(lambda:not state()['network_playtest']['active'],45)
                    assert 'failed' not in state()['status'].lower(),state()['status']
                    assert source.read_bytes()==original
                    sql=cluster.query("SELECT encode(appearance,'hex') FROM theia.characters;")
                    assert sql.returncode==0 and set(sql.stdout.splitlines())=={'01010200010101','00020402030300'},sql.stdout
                    (folder/'appearance.txt').write_text(sql.stdout)
                    resize_diagnostics = 0
                    for path in [folder/'editor.log',folder/'guest/client.log',*package.glob('runs/session-*/runtime/server.log'),*package.glob('runs/*.launcher.log')]:
                        text=path.read_text(); resize_diagnostics += check_gpu_diagnostics(text, allow_surface_resize=True)
                        assert identity.password not in text and cluster.password not in text
                    results.append(dict(resumed=resumed,preview=True,saved_appearance=True,live_appearance=True,two_authenticated_players=True,source_unchanged=True,surface_resize_diagnostics=resize_diagnostics,gpu=state()['gpu']))
                finally:
                    stop(guest)
                    if guest_log: guest_log.close()
                    if account: account.close()
                    ticket.unlink(missing_ok=True)
                    if desktop: desktop.close()
                    if editor.poll() is None:
                        editor.send_signal(signal.SIGINT)
                        try:editor.wait(20)
                        except subprocess.TimeoutExpired:editor.kill();editor.wait()
                    wait(lambda:not list(package.glob('runs/session-*/runtime/renet-*')),30)
    assert args.zone.read_bytes()==original
    result=dict(passed=True,runs=results,database_restart=not args.once)
    (out/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--once',action='store_true',help='Check fresh native integration without repeating the already-qualified restart')
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/release/theia-zone-editor'))
    parser.add_argument('--zone',type=Path,default=Path('build/millhaven-population-v1/zone.json'))
    parser.add_argument('--players',type=Path,default=Path('build/modular-hero-v1/art-v3'))
    parser.add_argument('--creature',type=Path,default=Path('build/millhaven-population-v1/gameplay/creature.json'))
    parser.add_argument('--creature-asset',type=Path,default=Path('build/starter-valley-population-editor-native-v2/package/creature.glb'))
    parser.add_argument('--scenery-library',type=Path,default=Path('build/starter-valley-scenery-v2/catalog.json'))
    parser.add_argument('--gameplay-template',type=Path,default=Path('build/millhaven-population-v1/gameplay/manifest.json'))
    run(parser.parse_args())
