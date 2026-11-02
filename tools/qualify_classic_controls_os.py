#!/usr/bin/env python3
"""Real normal OIDC/PostgreSQL launcher with owned X11 classic-control input."""
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
import time
import urllib.parse
from qualify_world_input_os import Desktop, wait, owned_window, capture_ready
from private_identity import Identity
from private_postgres import Cluster
from gpu_validation import check_gpu_diagnostics


def turn(a,b):
    return math.atan2(math.sin((b-a)*math.tau/65536),math.cos((b-a)*math.tau/65536))


def checks(desktop,window,state,folder,remapped):
    def facing():return state()['classic_facing']
    def feet():return state()['dodge']['authority_feet']
    def stationary(a,b):assert math.hypot(b[0]-a[0],b[2]-a[2])<.025,(a,b)
    def settled():time.sleep(.35);return state()
    def hold(key,seconds=.3):
        desktop.key_state(key,True)
        try:time.sleep(seconds)
        finally:desktop.key_state(key,False)
        return settled()
    def drag(button,amount=65):
        width,height,scale=state()['window']
        desktop.move(window,[width*.7,height*.48],scale);desktop.button(button,True)
        try:desktop.look(amount,0)
        finally:desktop.button(button,False)
        return settled()
    wait(lambda:state()['classic_enabled'] and facing() is not None)
    wait(lambda:state()['dodge']['tick']>5)
    proof={}
    scale=state()['window'][2]
    # Use real guide picking/dialogue and panel keys; telemetry writes no intent.
    point=wait(lambda:state()['guide_screen'])
    (folder/'before-first-hover.json').write_text(json.dumps(state(),indent=2))
    desktop.move(window,point,scale);time.sleep(.4)
    (folder/'after-first-hover.json').write_text(json.dumps(state(),indent=2))
    (folder/'capture.request').write_text('initial classic view\n');wait(lambda:capture_ready(folder/'input-os.png'));(folder/'input-os.png').rename(folder/'initial-view.png')
    point=wait(lambda:state()['guide_screen'])
    desktop.click(window,point,scale,1);wait(lambda:state()['friendly_selected'])
    point=wait(lambda:state()['guide_screen'])
    desktop.click(window,point,scale,3);wait(lambda:state()['dialogue_open'])
    desktop.key('Escape');wait(lambda:not state()['dialogue_open'])
    if state()['inventory_open']:
        desktop.key('Escape');wait(lambda:not state()['inventory_open'])
    assert state()['friendly_selected'],'Panel Escape unexpectedly cleared target'
    heading=facing();before=feet()
    width,height,_=state()['window'];orbits=0
    def enemy_in_click_area():
        point=state()['enemy_screen']
        return point if point and width*.2<point[0]<width*.8 and height*.15<point[1]<height*.8 else None
    while not enemy_in_click_area() and orbits<32:
        drag(1);orbits+=1
    point=wait(enemy_in_click_area)
    desktop.click(window,point,scale,1)
    wait(lambda:state()['selected_hostile'] is not None)
    clicked_hostile=state()['selected_hostile']
    assert not state()['friendly_selected']
    assert abs(turn(heading,facing()))<.002;stationary(before,feet())
    assert state()['controls']['basic_requests']==0,'Left click requested an attack'
    for _ in range(orbits):drag(1,-65)
    point=wait(lambda:state()['guide_screen'])
    desktop.click(window,point,scale,1)
    wait(lambda:state()['friendly_selected'] and state()['selected_hostile'] is None)
    proof['left_click_selects_without_attack_or_facing_change']=True
    before=feet();heading=facing();result=hold('Left' if remapped else 'a')
    stationary(before,feet());assert abs(turn(heading,facing()))>.4
    proof['turn_in_place']={'before':heading,'after':facing(),'position':feet()}
    heading=facing();before=feet();yaw=state()['camera_angles'][0];drag(1)
    assert abs(turn(heading,facing()))<.002
    assert abs(math.atan2(math.sin(state()['camera_angles'][0]-yaw),math.cos(state()['camera_angles'][0]-yaw)))>.05
    stationary(before,feet());assert state()['friendly_selected']
    proof['left_orbit_independent']=True
    heading=facing();before=feet();drag(3)
    assert abs(turn(heading,facing()))>.05
    assert abs(math.atan2(math.sin(facing()*math.tau/65536-state()['camera_angles'][0]),math.cos(facing()*math.tau/65536-state()['camera_angles'][0])))<.003
    stationary(before,feet());assert state()['friendly_selected']
    proof['right_steers_stationary']=True
    heading=facing();yaw=heading*math.tau/65536
    for name,key,expected in [
        ('strafe_right','d' if remapped else 'e',[math.cos(yaw),-math.sin(yaw)]),
        ('strafe_left','a' if remapped else 'q',[-math.cos(yaw),math.sin(yaw)]),
        ('backward','s',[math.sin(yaw),math.cos(yaw)]),
        ('forward','w',[-math.sin(yaw),-math.cos(yaw)]),
    ]:
        before=feet();hold(key);after=feet();dx,dz=after[0]-before[0],after[2]-before[2]
        assert dx*expected[0]+dz*expected[1]>.6,(name,before,after)
        assert abs(turn(heading,facing()))<.002,(name,heading,facing())
        assert not state()['dialogue_open'],'Strafe key opened dialogue'
        proof[name]={'before':before,'after':after,'facing':facing()}
    width,height,scale=state()['window'];desktop.move(window,[width*.7,height*.48],scale)
    before=feet();desktop.button(1,True);desktop.button(3,True)
    try:desktop.look(25,0);time.sleep(.3)
    finally:desktop.button(3,False);desktop.button(1,False)
    settled();after=feet();assert math.hypot(after[0]-before[0],after[2]-before[2])>.6
    proof['both_buttons_forward']={'before':before,'after':after}
    desktop.key('i');wait(lambda:state()['inventory_open']);yaw=state()['camera_angles'][0];heading=facing();before=feet()
    # Inventory's own button must retain mouse input even with both held.
    desktop.move(window,[width-182,139],scale);desktop.button(1,True);desktop.button(3,True)
    try:time.sleep(.3)
    finally:desktop.button(3,False);desktop.button(1,False)
    settled();stationary(before,feet());assert abs(turn(heading,facing()))<.002
    assert abs(state()['camera_angles'][0]-yaw)<.0001
    # The click may close the backpack. Open it again for explicit Esc priority.
    if not state()['inventory_open']:desktop.key('i');wait(lambda:state()['inventory_open'])
    desktop.key('Escape');wait(lambda:not state()['inventory_open'])
    assert state()['friendly_selected']
    desktop.key('Escape');wait(lambda:not state()['friendly_selected'])
    proof['ui_mouse_and_escape_priority']=True
    before=feet();heading=facing();desktop.key('F8');desktop.key('v');settled()
    assert not state()['camera_mode']['requested'] and not state()['camera_mode']['active']
    assert state()['dodge']['snapshot'] is None and state()['dodge_animation'] is None
    stationary(before,feet());assert abs(turn(heading,facing()))<.002
    # Explicit hostile selection is independent of steering/orbit. The fixture
    # has one hostile; multi-target ordering is covered by shared numerical tests.
    desktop.key('Tab');wait(lambda:state()['selected_hostile'] is not None)
    selected=state()['selected_hostile'];assert selected==clicked_hostile
    heading=facing();before=feet()
    drag(1);assert state()['selected_hostile']==selected
    assert abs(turn(heading,facing()))<.002;stationary(before,feet())
    drag(3);assert state()['selected_hostile']==selected
    assert abs(turn(heading,facing()))>.05;stationary(before,feet())
    desktop.key_state('Shift_L',True)
    try:desktop.key('Tab')
    finally:desktop.key_state('Shift_L',False)
    settled();assert state()['selected_hostile']==selected
    desktop.key('i');wait(lambda:state()['inventory_open'])
    desktop.key('Escape');wait(lambda:not state()['inventory_open'])
    assert state()['selected_hostile']==selected
    desktop.key('Escape');wait(lambda:state()['selected_hostile'] is None)
    desktop.key_state('Shift_L',True)
    try:desktop.key('Tab')
    finally:desktop.key_state('Shift_L',False)
    wait(lambda:state()['selected_hostile']==selected)
    desktop.key('BackSpace');wait(lambda:state()['selected_hostile'] is None)
    proof['hostile_retention_orbit_steer_reverse_tab_and_clear']=True
    proof.update(action_camera_disabled=True,dodge_disabled=True,ad_strafe_remap=remapped)
    (folder/'classic-controls.json').write_text(json.dumps(proof,indent=2)+'\n')
    return proof


def run(output,binaries,sizes=None):
    output=output.resolve();binaries=binaries.resolve();output.mkdir(mode=0o700,parents=True,exist_ok=False)
    bundle=output/'gameplay';shutil.copytree('content/gameplay',bundle)
    creature=bundle/'development-creature-deflect-pursuit.json';data=json.loads(creature.read_text());data.pop('aggro',None);creature.write_text(json.dumps(data)+'\n')
    remap=json.loads(Path('content/input/classic-controls.json').read_text())
    remap['bindings'].update(turn_left='ArrowLeft',turn_right='ArrowRight',strafe_left='KeyA',strafe_right='KeyD')
    remap_path=output/'ad-strafe-controls.json';remap_path.write_text(json.dumps(remap)+'\n')
    desktop=Desktop();results={};launcher=None
    try:
        with Identity(output/'provider',Path('build/identity-sdk'),token_seconds=300) as identity, Cluster(output/'cluster',Path('build/postgres-sdk')) as cluster:
            for index,size in enumerate(sizes or ['1920x1200','3440x1440','1280x720']):
                folder=output/size;folder.mkdir();remapped=size=='1280x720'
                env={k:v for k,v in os.environ.items() if not k.startswith('THEIA_') and k not in ['VK_INSTANCE_LAYERS','VK_LOADER_DEBUG','WGPU_VALIDATION','WGPU_DEBUG']}
                env.update(DISPLAY=':0',XDG_RUNTIME_DIR='/run/user/1000',LD_LIBRARY_PATH=str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()),THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],THEIA_OIDC_ISSUER=identity.issuer,THEIA_OIDC_CLIENT_ID=identity.client_id,THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,THEIA_TEST_BROWSER_ISSUER=identity.issuer,THEIA_TEST_BROWSER_REDIRECT=identity.redirect,THEIA_TEST_BROWSER_USER=identity.users[0][0],THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',THEIA_WORLD_INPUT_OS_QUALIFY=str(folder))
                with (folder/'launcher.log').open('w') as log:
                    command=[sys.executable,'tools/play.py','--account-login','--classic','--binaries',str(binaries),'--gameplay-manifest',str(bundle/'development-gameplay-projectiles.json'),'--players',str(binaries/'art/player'),'--creature-asset',str(binaries/'art/wolf/wolf.glb'),'--dev-spawn','8002,8002','--window-size',size]
                    if remapped:command.extend(['--controls-file',str(remap_path)])
                    if size=='3440x1440':command.append('--fullscreen')
                    launcher=subprocess.Popen(command,env=env,stdin=subprocess.PIPE,stdout=log,stderr=subprocess.STDOUT,text=True)
                    try:
                        launcher.stdin.write(('' if index==0 else 'n\n')+'ClassicArden'+chr(65+index)+'\n');launcher.stdin.flush()
                        def state():
                            assert launcher.poll() is None,'Owned launcher exited early'
                            return json.loads((folder/'input-os.json').read_text())
                        initial=wait(lambda:state() if state()['connected'] and state()['guide_screen'] else None,40)
                        status=Path(f'/proc/{initial["pid"]}/status').read_text()
                        assert int(re.search(r'^PPid:\s+(\d+)',status,re.M)[1])==launcher.pid
                        window=wait(lambda:owned_window(initial['pid']));desktop.focus(window);time.sleep(2)
                        proof=checks(desktop,window,state,folder,remapped)
                        (folder/'capture.request').write_text('capture owned classic window\n');wait(lambda:capture_ready(folder/'input-os.png'))
                        final=state();launcher.send_signal(signal.SIGINT);assert launcher.wait(timeout=15)==0
                        gpu=check_gpu_diagnostics((folder/'launcher.log').read_text(),allow_surface_resize=size=='3440x1440')
                        results[size]={'passed':True,'checks':proof,'window':final['window'],'adapter':final['adapter'],'gpu_diagnostics':gpu}
                        (output/'partial-results.json').write_text(json.dumps(results,indent=2)+'\n')
                    finally:
                        if launcher.poll() is None:
                            launcher.send_signal(signal.SIGINT)
                            try:launcher.wait(timeout=15)
                            except subprocess.TimeoutExpired:launcher.kill();launcher.wait()
                        launcher=None
    finally:desktop.close()
    result={'passed':True,'scope':'Native classic movement controls using X11 synthetic input to owned PID-matched windows; no complete combat milestone, physical hardware or Windows claim','fixture':'Copied gameplay data disables wolf perception to isolate controls; actual normal OIDC/PostgreSQL launcher, no gameplay input injection via telemetry','results':results}
    (output/'verification.json').write_text(json.dumps(result,indent=2)+'\n');return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path);parser.add_argument('--binaries',type=Path,required=True)
    parser.add_argument('--sizes',nargs='+',choices=['1920x1200','3440x1440','1280x720'])
    args=parser.parse_args();result=run(args.output,args.binaries,args.sizes);print(json.dumps({'passed':result['passed'],'sizes':list(result['results'])}))
