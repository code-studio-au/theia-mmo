"""Native one-window title/create/roster/world flow with owned OIDC and PostgreSQL."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import urllib.parse
from PIL import Image
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
                BROWSER='/bin/false',
                THEIA_WORLD_INPUT_OS_QUALIFY=str(game),THEIA_ZONE_MAP_DIRECTORY=str(Path('build/covenant-seven-gentle-v2').resolve()))
            command=[sys.executable,'tools/account_population_playtest.py','--package',str(args.package.resolve()),
                '--players',str(args.players.resolve()),'--binaries',str(Path('target/release').resolve()),
                '--output',str(session),'--parties','--window-size',('1280x700' if args.layout_check else '1920x1080'),
                '--fantasy-ui','/home/steven/Downloads/Synty Models/HUDs','--ui-preferences',str(folder/'interface.json'),
                '--gameplay-manifest',str(args.package.resolve()/'world/gameplay/gameplay.manifest.json')]
            if resumed: command += ['--test-window', '4/4']
            if args.chat_check: command += ['--chat-area', '7']
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
                    def select_hero():
                        roster=json.loads((session/'runtime/frontend-state.json').read_text())['characters']
                        index=next(i for i,c in enumerate(roster) if c['name'].casefold()=='heroarden')
                        click(f'Select({index})')
                    def screenshot(name):
                        (game/'capture.request').write_text(name)
                        wait(lambda:(alive() or capture_ready(game/'input-os.png')),30)
                        (game/'input-os.png').rename(folder/name)
                        (folder/(name+'.json')).write_text(json.dumps(menu(),indent=2))
                    def layout_matrix(page):
                        if not args.layout_check or resumed:return
                        # Resize only this owned client to the single/half/quarter
                        # allocation sizes; this is layout evidence, not multiplayer load.
                        for label,width,height in [('single',3440,1400),('two',1720,1400),('four',1720,700)]:
                            desktop.owned_focus()
                            subprocess.run(['wmctrl','-i','-r',hex(window),'-e',f'0,0,30,{width},{height}'],check=True)
                            wait(lambda:abs(state()['window'][0]-width)<2 and abs(state()['window'][1]-height)<2,15)
                            time.sleep(2)
                            screenshot(f'{page}-{label}.png')
                            for name,(x,y,w,h) in menu()['widgets'].items():
                                assert x>=-1 and y>=-1 and x+w<=width+1 and y+h<=height+1, (page,label,name,[x,y,w,h])
                        subprocess.run(['wmctrl','-i','-r',hex(window),'-e','0,0,30,1920,1080'],check=True)
                        wait(lambda:abs(state()['window'][0]-1920)<2 and abs(state()['window'][1]-1080)<2,15)
                        time.sleep(2)
                    def type_name(value):
                        click('Name');desktop.key_state('Control_L',True);desktop.key('a');desktop.key_state('Control_L',False);desktop.key('BackSpace')
                        for char in value:
                            desktop.key({'-':'minus'}.get(char,char));time.sleep(.035)
                    def type_field(label, value):
                        click(label);desktop.key_state('Control_L',True);desktop.key('a');desktop.key_state('Control_L',False)
                        desktop.key('BackSpace')
                        for char in value:
                            shifted=char.isupper() or char=='_'
                            if shifted:desktop.key_state('Shift_L',True)
                            desktop.key({'-':'minus','_':'minus'}.get(char,char.lower()))
                            if shifted:desktop.key_state('Shift_L',False)
                            time.sleep(.025)
                    assert menu()['theme']['sprite_package']=='sprites-v3'
                    screenshot('login.png')
                    layout_matrix('login')
                    user_rect=menu()['widgets']['Username']; password_rect=menu()['widgets']['Account(Password)']
                    assert all(abs(user_rect[i]-password_rect[i])<1 for i in [2,3]), 'Login field sizes differ'
                    for label,filename in [('Account(Settings)','settings-link-hover.png'),('Account(Support)','support-link-hover.png'),('Exit','exit-link-hover.png'),('Account(Reveal)','eye-hover.png')]:
                        rect=menu()['widgets'][label]
                        desktop.move(window,[rect[0]+rect[2]/2,rect[1]+rect[3]/2],1);time.sleep(.2)
                        screenshot(filename)
                    click('Account(Settings)')
                    screenshot('settings-open.png')
                    footer_points={c['name']:c['point'] for c in menu()['theme']['kit']['footer']}
                    wait(lambda:any(c['name']=='vsync' for c in menu()['theme']['controls']),10)
                    def control(name):
                        item=wait(lambda:next((c for c in menu()['theme']['controls']+menu()['theme']['kit']['dropdowns']+menu()['theme']['kit']['footer'] if c['name']==name),None),10)
                        desktop.move(window,item['point'],1);time.sleep(.1);desktop.owned_focus(pointer=True)
                        desktop.button(1,True);time.sleep(.08);desktop.button(1,False);time.sleep(.4)
                    original_vsync=menu()['theme']['vsync']
                    control('vsync');wait(lambda:menu()['theme']['vsync']!=original_vsync)
                    desktop.key('space');wait(lambda:menu()['theme']['vsync']==original_vsync)
                    click('Account(Interface)')
                    control('scale');desktop.key('Home');wait(lambda:abs(menu()['theme']['scale']-.75)<.001)
                    desktop.key('End');wait(lambda:abs(menu()['theme']['scale']-1.5)<.001)
                    desktop.key('Home');time.sleep(.2)
                    for _ in range(5):desktop.key('Right');time.sleep(.1)
                    wait(lambda:abs(menu()['theme']['scale']-1.)<.001)
                    screenshot('settings.png')
                    for c in menu()['theme']['kit']['footer']:
                        assert max(abs(a-b) for a,b in zip(c['point'],footer_points[c['name']]))<1, 'Settings footer moved between tabs'
                    click('Account(Settings)');screenshot('graphics.png')
                    control('fps-menu');click('Account(Cap(120))');control('apply')
                    control('fps-menu');click('Account(Cap(30))');control('cancel')
                    wait(lambda:menu()['theme']['frame_cap']==120)
                    click('Account(Settings)');control('fps-menu');click('Account(Cap(60))')
                    control('view-menu');click('Account(Distance(Nearby))');control('ok')
                    click('Account(Support)');screenshot('support.png');click('Account(Close)')
                    type_field('Username',identity.users[0][0])
                    if not resumed:
                        type_field('Account(Password)','incorrect-password')
                        click('Account(Reveal)');screenshot('password-eye.png');click('Account(Reveal)')
                        click('Login')
                        wait(lambda:'Sign-in failed' in menu()['message'],30)
                        screenshot('login-retry.png')
                    type_field('Account(Password)',identity.users[0][1])
                    screenshot('masked-password.png')
                    x,y,w,h=menu()['widgets']['Username']
                    with Image.open(folder/'masked-password.png') as shot:
                        field=shot.convert('RGB').crop((round(x+48),round(y+8),round(x+w-16),round(y+h-8)))
                        assert sum(min(pixel)>160 for pixel in field.getdata())>50, 'Username is covered by its field artwork'
                    click('Login')
                    wait(lambda:menu()['page']=='Characters',60)
                    desktop.focus(window)
                    if resumed:
                        wait(lambda:menu()['preview_ready'],45)
                    else:
                        assert menu()['character_count']==0 and not menu()['preview_visible'] and not menu()['preview_ready']
                        assert 'Enter' in menu()['disabled']
                        click('New');wait(lambda:menu()['page']=='Create')
                        wait(lambda:menu()['preview_ready'],45)
                        click('Back');wait(lambda:menu()['page']=='Characters')
                        assert not menu()['preview_visible'] and 'Enter' in menu()['disabled']
                    screenshot('characters.png')
                    if not resumed:
                        click('New');wait(lambda:menu()['page']=='Create')
                        for i in [0,1,2,4,5,6]:click(f'Cycle({i}, true)')
                        rect=menu()['widgets']['Rotate'];point=[rect[0]+rect[2]/2,rect[1]+rect[3]/2]
                        initial_yaw=menu()['preview_yaw']
                        desktop.move(window,point,1);desktop.owned_focus(pointer=True);desktop.button(1,True)

                        for dx in [20,45,70,90]:desktop.move(window,[point[0]+dx,point[1]],1)
                        time.sleep(.2)
                        wait(lambda:abs(menu()['preview_yaw']-initial_yaw)>.1)
                        turned_yaw=menu()['preview_yaw']

                        for dx in [60,30,0,-25,-50]:desktop.move(window,[point[0]+dx,point[1]],1)
                        time.sleep(.2)
                        wait(lambda:abs(menu()['preview_yaw']-turned_yaw)>.1)
                        desktop.button(1,False);time.sleep(.2);released_yaw=menu()['preview_yaw']
                        desktop.move(window,point,1);time.sleep(.2)
                        assert abs(menu()['preview_yaw']-released_yaw)<.001, 'Preview still rotates after release'
                        screenshot('rotate-drag.png')
                        start_zoom=menu()['preview_zoom']
                        click('Zoom(true)');wait(lambda:menu()['preview_zoom']>start_zoom)
                        screenshot('zoom-in.png')
                        click('Zoom(false)');wait(lambda:abs(menu()['preview_zoom']-start_zoom)<.001)
                        for _ in range(6):click('Zoom(true)')
                        wait(lambda:'Zoom(true)' in menu()['disabled'])
                        screenshot('zoom-maximum.png')
                        layout_matrix('zoom-maximum')
                        click('ResetView');wait(lambda:abs(menu()['preview_zoom']-1)<.001 and abs(menu()['preview_yaw'])<.001)
                        layout_matrix('creation')
                        type_name('bad--name');click('Create')
                        wait(lambda:menu()['invalid_name'])
                        screenshot('invalid-name.png')
                        type_name('');wait(lambda:not menu()['invalid_name'])
                        type_name('bad--name');wait(lambda:menu()['invalid_name'])
                        time.sleep(.5)
                        assert menu()['page']=='Create'
                        type_name('Heroarden');wait(lambda:not menu()['invalid_name']);screenshot('creation.png')
                        # Native EditableText glyphs must remain above the field artwork.
                        x,y,w,h=menu()['widgets']['Name']
                        with Image.open(folder/'creation.png') as shot:
                            field=shot.convert('RGB').crop((round(x+8),round(y+6),round(x+w-8),round(y+h-6)))
                            assert sum(min(pixel)>160 for pixel in field.getdata())>100, 'Character name is covered by its field artwork'
                        click('Create')
                        wait(lambda:menu()['page']=='Characters')
                        screenshot('saved-character.png')
                        expected=menu()['appearance']
                        click('New');wait(lambda:menu()['page']=='Create')
                        type_name('Oakwarden');click('Create');wait(lambda:menu()['page']=='Characters')
                        select_hero();wait(lambda:menu()['appearance']==expected)
                        screenshot('two-saved-characters.png')
                        # A ready character must stay visible while native UI pipelines change.
                        wait(lambda:menu()['preview_ready'])
                        for _ in range(20):
                            assert menu()['preview_ready'] and menu()['preview_visible']
                            time.sleep(.1)
                        for name in ['Birchwarden','Elmwarden','Ashwarden','Pinewarden','Firwarden','Yewwarden']:
                            click('New');wait(lambda:menu()['page']=='Create')
                            type_name(name);wait(lambda:menu()['preview_ready']);click('Create');wait(lambda:menu()['page']=='Characters')
                        select_hero();wait(lambda:menu()['appearance']==expected)
                        assert menu()['character_count']==8 and 'New' in menu()['disabled']
                        screenshot('full-roster.png')
                        layout_matrix('roster')
                    else:
                        select_hero();wait(lambda:menu()['appearance']==expected)
                        assert menu()['character_count']==8 and 'New' in menu()['disabled']
                        screenshot('restored-character.png')
                    wait(lambda:menu()['preview_ready'])
                    if not args.frontend_only:
                        click('Enter')
                        observed=wait(lambda:state() if state().get('connected') and state().get('loading',{}).get('phase')=='Playing' else None,120)
                        assert observed['pid']==pid and owned_window(pid)==window,'The game window was replaced'
                        assert any(h['appearance']==expected for h in observed.get('modular_heroes',[])), 'Appearance changed on entry'
                        desktop.focus(window);desktop.hold_key('q',.4)
                        screenshot('world.png')
                        w,h,scale=state()['window']
                        desktop.click(window,[60,h/scale-340],scale,1);time.sleep(.4);screenshot('world-chat.png')
                        desktop.click(window,[158,h/scale-340],scale,1);time.sleep(.4)
                        desktop.key('F10');time.sleep(.7);screenshot('world-settings.png')
                        graphics=next(b for b in state()['interface']['buttons'] if b['action']=='graphics')
                        desktop.click(window,graphics['point'],1,1);time.sleep(.5);screenshot('world-graphics.png')
                        desktop.key('F10');time.sleep(.2)
                        assert not state()['interface']['open']
                        for key,name in [('i','inventory'),('j','journal'),('p','abilities'),('o','party')]:
                            desktop.key(key);time.sleep(.7)
                            if name=='inventory':assert state()['inventory_open']
                            if name=='abilities':assert state()['ability_book']['open']
                            if name=='party':assert state()['party']['open']
                            screenshot('world-'+name+'.png')
                            desktop.key(key);time.sleep(.2)
                        desktop.click(window,[w/scale-35,h/scale-34],scale,1);time.sleep(.5);screenshot('world-menu.png')
                        desktop.key('Escape')
                        if args.chat_check:
                            from qualify_chat_composer import qualify
                            saved_name=next(c['name'] for c in json.loads((session/'runtime/frontend-state.json').read_text())['characters'] if c['name'].casefold()=='heroarden')
                            chat_result=qualify(desktop,window,state,screenshot,saved_name)
                    results.append(dict(sprite_package='sprites-v3',native_password_login=True,settings_apply_cancel=True,roster_count=8,resumed=resumed,pid=pid,window=window,same_window=True,appearance=expected,playing=not args.frontend_only,layout_check=args.layout_check))
                    if args.chat_check: results[-1]['chat']=chat_result
                    print(json.dumps(results[-1]),flush=True)
                finally:
                    if desktop:desktop.close()
                    if process.poll() is None:
                        process.stdin.close()
                        try:process.wait(timeout=25)
                        except subprocess.TimeoutExpired:process.kill();process.wait()
                diagnostics=check_gpu_diagnostics((folder/'launcher.log').read_text(),allow_surface_resize=True)
                results[-1]['surface_resize_diagnostics']=diagnostics
                for path in folder.rglob('*'):
                    if path.is_file() and path.suffix in ('.json','.log'):
                        text=path.read_text(errors='replace')
                        assert identity.users[0][1] not in text, 'Credential appeared in diagnostic output'
    (out/'qualification.json').write_text(json.dumps(dict(results=results),indent=2))

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True)
    p.add_argument('--package',type=Path,default=Path('build/modular-hero-v1/native-final-v2/fresh/package'))
    p.add_argument('--players',type=Path,default=Path('build/modular-hero-v1/art-v3'))
    p.add_argument('--layout-check',action='store_true')
    p.add_argument('--frontend-only',action='store_true',help='Qualify menus/preview without entering the world')
    p.add_argument('--chat-check',action='store_true',help='Exercise real account Say/Area composer after world entry')
    p.add_argument('--once',action='store_true')
    args=p.parse_args()
    if args.chat_check and args.frontend_only: p.error('--chat-check requires world entry')
    run(args)
