"""Native one-window title/create/roster/world flow with owned OIDC and PostgreSQL."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import urllib.parse
from edit_zone import build_environment
from private_identity import Identity
from private_postgres import Cluster
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from gpu_validation import check_gpu_diagnostics


def run(args):
    out=args.output.resolve(); out.mkdir(mode=0o700,parents=True,exist_ok=False)
    results=[]
    with Identity(out/'provider',Path('build/identity-sdk'),token_seconds=1800,interactive=True) as identity, Cluster(out/'cluster',Path('build/postgres-sdk')) as cluster:
        for resumed in ([False] if args.once else [False,True]):
            if resumed:cluster.restart()
            folder=out/('resumed' if resumed else 'fresh');folder.mkdir()
            game=folder/'game';game.mkdir()
            session=folder/'session'
            env={k:v for k,v in build_environment(os.environ).items() if not k.startswith('THEIA_')}
            env.update(DISPLAY=':0',XDG_RUNTIME_DIR='/run/user/1000',WGPU_BACKEND='vulkan',
                VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',WGPU_VALIDATION='1',RUST_LOG='info',
                THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],
                THEIA_ACCOUNT_SESSION_SECONDS='1800',THEIA_OIDC_ISSUER=identity.issuer,
                THEIA_OIDC_CLIENT_ID=identity.client_id,THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                THEIA_TEST_BROWSER_ISSUER=identity.issuer,THEIA_TEST_BROWSER_REDIRECT=identity.redirect,
                THEIA_TEST_BROWSER_USER=identity.users[0][0],THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],
                BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',
                THEIA_WORLD_INPUT_OS_QUALIFY=str(game),THEIA_ZONE_MAP_DIRECTORY=str(Path('build/covenant-seven-gentle-v2').resolve()))
            command=[sys.executable,'tools/account_population_playtest.py','--package',str(args.package.resolve()),
                '--players',str(args.players.resolve()),'--binaries',str(Path('target/release').resolve()),
                '--output',str(session),'--window-size','1920x1080',
                '--fantasy-ui','/home/steven/Downloads/Synty Models/HUDs',
                '--gameplay-manifest',str(args.package.resolve()/'world/gameplay/gameplay.manifest.json')]
            if resumed: command += ['--test-window', '4/4']
            desktop=None
            with (folder/'launcher.log').open('w') as log:
                process=subprocess.Popen(command,env=env,stdin=subprocess.PIPE,stdout=log,stderr=log)
                try:
                    def alive():
                        assert process.poll() is None, 'Launcher exited: '+str(folder/'launcher.log')
                    def state():
                        alive();return json.loads((game/'input-os.json').read_text())
                    def menu():
                        alive();return json.loads((session/'runtime/frontend-evidence.json').read_text())
                    initial=wait(lambda:state(),120)
                    pid=initial['pid'];window=wait(lambda:owned_window(pid),60)
                    desktop=Desktop();desktop.focus(window)
                    wait(lambda: menu()['page']=='Login',30)
                    time.sleep(2)
                    def click(label):
                        rect=menu()['widgets'][label]
                        desktop.move(window,[rect[0]+rect[2]/2,rect[1]+rect[3]/2],1)
                        time.sleep(.1);desktop.owned_focus(pointer=True)
                        desktop.button(1,True);time.sleep(.08);desktop.button(1,False);time.sleep(.3)
                    def screenshot(name):
                        (game/'capture.request').write_text(name)
                        wait(lambda:(alive() or capture_ready(game/'input-os.png')),30)
                        (game/'input-os.png').rename(folder/name)
                        (folder/(name+'.json')).write_text(json.dumps(menu(),indent=2))
                    def type_name(value):
                        click('Name');desktop.key_state('Control_L',True);desktop.key('a');desktop.key_state('Control_L',False)
                        for char in value:
                            desktop.key({'-':'minus'}.get(char,char));time.sleep(.035)
                    screenshot('login.png')
                    click('Login')
                    wait(lambda:menu()['page']=='Characters',60)
                    desktop.focus(window)
                    wait(lambda:menu()['preview_ready'],45)
                    screenshot('characters.png')
                    if not resumed:
                        click('New');wait(lambda:menu()['page']=='Create')
                        for i in [0,1,2,4,5,6]:click(f'Cycle({i}, true)')
                        click('Rotate(true)')
                        type_name('bad--name');click('Create')
                        wait(lambda:'name' in menu()['message'].lower() or 'interior' in menu()['message'].lower())
                        time.sleep(.5)
                        assert menu()['page']=='Create'
                        type_name('Heroarden');screenshot('creation.png');click('Create')
                        wait(lambda:menu()['page']=='Characters')
                        screenshot('saved-character.png')
                        expected=menu()['appearance']
                        click('New');wait(lambda:menu()['page']=='Create')
                        type_name('Oakwarden');click('Create');wait(lambda:menu()['page']=='Characters')
                        click('Select(0)');wait(lambda:menu()['appearance']==expected)
                        screenshot('two-saved-characters.png')
                    else:
                        click('Select(0)');wait(lambda:menu()['appearance']==expected)
                        click('New');wait(lambda:menu()['page']=='Create')
                        screenshot('small-creation.png')
                        click('Back');wait(lambda:menu()['page']=='Characters')
                        assert menu()['appearance']==expected
                        screenshot('restored-character.png')
                    wait(lambda:menu()['preview_ready'])
                    click('Enter')
                    observed=wait(lambda:state() if state().get('connected') and state().get('loading',{}).get('phase')=='Playing' else None,120)
                    assert observed['pid']==pid and owned_window(pid)==window,'The game window was replaced'
                    assert any(h['appearance']==expected for h in observed.get('modular_heroes',[])), 'Appearance changed on entry'
                    desktop.focus(window);desktop.hold_key('q',.4)
                    screenshot('world.png')
                    results.append(dict(resumed=resumed,pid=pid,window=window,same_window=True,appearance=expected,playing=True))
                    print(json.dumps(results[-1]),flush=True)
                finally:
                    if desktop:desktop.close()
                    if process.poll() is None:
                        process.stdin.close()
                        try:process.wait(timeout=25)
                        except subprocess.TimeoutExpired:process.kill();process.wait()
                diagnostics=check_gpu_diagnostics((folder/'launcher.log').read_text(),allow_surface_resize=True)
                results[-1]['surface_resize_diagnostics']=diagnostics
    (out/'qualification.json').write_text(json.dumps(dict(results=results),indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True)
    p.add_argument('--package',type=Path,default=Path('build/modular-hero-v1/native-final-v2/fresh/package'))
    p.add_argument('--players',type=Path,default=Path('build/modular-hero-v1/art-v3'))
    p.add_argument('--once',action='store_true');run(p.parse_args())
