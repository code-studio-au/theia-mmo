#!/usr/bin/env python3
"""Native registration on an isolated X display; never touches the user's desktop."""
import argparse
import json
import os
from pathlib import Path
import select
import subprocess
import sys
import time
from edit_local import environment
from private_identity import Identity
from private_postgres import Cluster
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from gpu_validation import check_gpu_diagnostics


def run(out, once=False):
    root=Path(__file__).resolve().parents[1];os.chdir(root)
    out=out.resolve();out.mkdir(parents=True,exist_ok=False,mode=0o700)
    readfd,writefd=os.pipe();x=None;wm=None;results=[]
    try:
        with (out/'x.log').open('w') as log:
            x=subprocess.Popen([str(root/'build/sprinkles-review/xvfb-root/usr/bin/Xvfb'),'-displayfd',str(writefd),'-screen','0','1920x1080x24','-nolisten','tcp','-noreset'],pass_fds=(writefd,),stdout=log,stderr=log)
        os.close(writefd);writefd=None
        assert select.select([readfd],[],[],15)[0]
        display=':'+os.read(readfd,32).decode().strip();assert display[1:].isdecimal()
        os.environ['DISPLAY']=display;os.environ.pop('WAYLAND_DISPLAY',None)
        with (out/'wm.log').open('w') as log:
            wm=subprocess.Popen(['metacity','--sm-disable'],stdout=log,stderr=log)
        with Identity(out/'provider',root/'build/identity-sdk',token_seconds=1800,interactive=True,registration=True) as identity, Cluster(out/'cluster',root/'build/postgres-sdk',isolate_process=True) as cluster:
            for resumed in ([False] if once else [False,True]):
                if resumed:cluster.restart()
                folder=out/('resumed' if resumed else 'fresh');folder.mkdir();game=folder/'game';game.mkdir()
                session=folder/'session'
                env=environment(os.environ,identity,cluster,folder,1800)
                env.update(DISPLAY=display,WINIT_UNIX_BACKEND='x11',VK_DRIVER_FILES='/usr/share/vulkan/icd.d/lvp_icd.json',WGPU_BACKEND='vulkan',WGPU_VALIDATION='1',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',BROWSER='/bin/false',THEIA_WORLD_INPUT_OS_QUALIFY=str(game))
                package=root/'build/modular-hero-v1/native-final-v2/fresh/package'
                cmd=[sys.executable,'tools/account_population_playtest.py','--package',str(package),'--players',str(root/'build/modular-hero-v1/art-v3'),'--binaries',str(root/'target/release'),'--output',str(session),'--parties','--window-size','1280x700','--fantasy-ui','/home/steven/Downloads/Synty Models/HUDs','--ui-preferences',str(folder/'prefs.json'),'--gameplay-manifest',str(package/'world/gameplay/gameplay.manifest.json')]
                desktop=None
                with (folder/'launcher.log').open('w') as log:
                    process=subprocess.Popen(cmd,env=env,stdin=subprocess.PIPE,stdout=log,stderr=log)
                    try:
                        def alive():assert process.poll() is None, f'Launcher exited: {folder}'
                        def state():alive();return json.loads((game/'input-os.json').read_text())
                        def menu():alive();return json.loads((session/'runtime/frontend-evidence.json').read_text())
                        s=wait(state,150);window=wait(lambda:owned_window(s['pid']),30)
                        desktop=Desktop(display);desktop.focus(window)
                        wait(lambda:menu()['page']=='Login',60);time.sleep(2)
                        def click(label):
                            rect=menu()['widgets'][label];desktop.click(window,[rect[0]+rect[2]/2,rect[1]+rect[3]/2],1,1);time.sleep(.5)
                        def type_field(label,value):
                            click(label);desktop.key_state('Control_L',True);desktop.key('a');desktop.key_state('Control_L',False);desktop.key('BackSpace')
                            for c in value:
                                shifted=c.isupper() or c in '@!_'
                                if shifted:desktop.key_state('Shift_L',True)
                                desktop.key({'-':'minus','_':'minus','@':'2','!':'1','.':'period'}.get(c,c.lower()))
                                if shifted:desktop.key_state('Shift_L',False)
                        def shot(name):
                            (game/'capture.request').write_text(name)
                            wait(lambda:capture_ready(game/'input-os.png'),60)
                            (game/'input-os.png').rename(folder/name)
                            (folder/(name+'.json')).write_text(json.dumps(menu(),indent=2))
                        if not resumed:
                            click('Registration(Open)');wait(lambda:menu()['page']=='Register')
                            shot('registration-empty.png')
                            assert not menu()['preview_visible']
                            back = menu()['widgets']['Registration(Back)']
                            submit = menu()['widgets']['Registration(Submit)']
                            assert back[0] + back[2] < submit[0], 'Back must sit outside the panel on the left'
                            click('Registration(Back)');wait(lambda:menu()['page']=='Login')
                            click('Registration(Open)');wait(lambda:menu()['page']=='Register')
                            click('Registration(Submit)');wait(lambda:'account name' in menu()['message'])
                            type_field('RegistrationField(0)','new-adventurer')
                            type_field('RegistrationField(1)','adventurer@example.invalid')
                            type_field('Registration(Password(0))','TestAccount123!')
                            type_field('Registration(Password(1))','Different123!')
                            click('Registration(Submit)');wait(lambda:'do not match' in menu()['message'])
                            assert json.loads((session/'runtime/frontend-state.json').read_text())['serial']==0
                            shot('registration-mismatch.png')
                            type_field('Registration(Password(1))','TestAccount123!')
                            click('Registration(Reveal(0))');click('Registration(Reveal(0))')
                            for name,width,height in [('quarter',1720,700),('half',1720,1000)]:
                                subprocess.run(['wmctrl','-i','-r',hex(window),'-e',f'0,0,0,{width},{height}'],check=True)
                                wait(lambda:abs(state()['window'][1]-height)<2,20);time.sleep(2);shot(f'registration-{name}.png')
                                for label,(a,b,w,h) in menu()['widgets'].items():
                                    assert a>=-1 and b>=-1 and a+w<=width+1 and b+h<=height+1,(label,[a,b,w,h],width,height)
                            click('Registration(Submit)');wait(lambda:menu()['page']=='Characters',40)
                            assert menu()['character_count']==0 and not menu()['preview_visible']
                            click('New');wait(lambda:menu()['page']=='Create')
                            type_field('Name','Gatewayarden')
                            wait(lambda:menu()['preview_ready'],90)
                            click('Create');wait(lambda:menu()['page']=='Characters' and menu()['character_count']==1,30)
                            shot('registration-created.png')
                        else:
                            # Duplicate account is refused, then existing credentials still work.
                            click('Registration(Open)');wait(lambda:menu()['page']=='Register')
                            for label,value in [('RegistrationField(0)','new-adventurer'),('RegistrationField(1)','adventurer@example.invalid'),('Registration(Password(0))','TestAccount123!'),('Registration(Password(1))','TestAccount123!')]:type_field(label,value)
                            click('Registration(Submit)');wait(lambda:'refused' in menu()['message'],40)
                            click('Registration(Back)');wait(lambda:menu()['page']=='Login')
                            type_field('Username','new-adventurer');type_field('Account(Password)','TestAccount123!');click('Login')
                            wait(lambda:menu()['page']=='Characters',40)
                            assert menu()['character_count']==1
                            rows=json.loads((session/'runtime/frontend-state.json').read_text())['characters']
                            assert rows[0]['name']=='Gatewayarden'
                            shot('registration-resumed.png')
                        results.append(dict(resumed=resumed,character_count=menu()['character_count'],gpu=state()['adapter'],isolated_display=True))
                        print(json.dumps(results[-1]),flush=True)
                    finally:
                        if desktop:desktop.close()
                        if process.poll() is None:
                            process.stdin.close()
                            try:process.wait(timeout=30)
                            except subprocess.TimeoutExpired:process.kill();process.wait()
                results[-1]['gpu_diagnostics']=check_gpu_diagnostics((folder/'launcher.log').read_text(),allow_surface_resize=True)
                for p in session.rglob('*.json'):
                    assert 'TestAccount123!' not in p.read_text(errors='replace'), 'Secret leaked to session file'
        (out/'qualification.json').write_text(json.dumps(dict(results=results),indent=2))
    finally:
        if writefd is not None:os.close(writefd)
        os.close(readfd)
        for process in [wm,x]:
            if process and process.poll() is None:
                process.terminate()
                try:process.wait(timeout=10)
                except subprocess.TimeoutExpired:process.kill();process.wait()

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--once',action='store_true');args=p.parse_args();run(args.output,args.once)
