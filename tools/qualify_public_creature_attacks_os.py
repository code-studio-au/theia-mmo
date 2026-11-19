#!/usr/bin/env python3
"""Owned normal schema-14 server, two OIDC accounts/native clients, PID-bound X11."""
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
import urllib.request

from account_login import AccountClient
from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from private_identity import Identity, _NoRedirect
from private_postgres import Cluster
from qualify_world_input_os import Desktop, wait, owned_window, capture_ready


def read_state(folder, alive):
    assert alive(), 'Owned native process exited early'
    return wait(lambda:json.loads((folder/'input-os.json').read_text()),2)


def face(desktop, window, state, target):
    for _ in range(8):
        current = state()
        feet = current['dodge']['authority_feet']
        yaw = math.atan2(-(target[0]-feet[0]), -(target[2]-feet[2]))
        error = math.atan2(math.sin(yaw-current['classic_facing']*math.tau/65536), math.cos(yaw-current['classic_facing']*math.tau/65536))
        if abs(error) < .06:
            return
        width, height, scale = current['window']
        desktop.move(window, [width*.55,height*.42], scale)
        time.sleep(.1)
        desktop.button(3, True)
        try:
            error = math.atan2(math.sin(yaw-current['camera_angles'][0]), math.cos(yaw-current['camera_angles'][0]))
            desktop.look(round(max(-400,min(400,-error/.004))),0)
        finally:
            desktop.button(3,False)
        time.sleep(.2)
    raise AssertionError('Manual facing did not reach the wolf')


def run(output, binaries, player_death=False, locomotion=False):
    if locomotion and not player_death:
        raise ValueError('Directional regression requires the schema-15 public-life profile')
    output=output.resolve();binaries=binaries.resolve()
    output.mkdir(mode=0o700,parents=True,exist_ok=False)
    bundle=output/'content'
    shutil.copytree('content/gameplay',bundle/'gameplay')
    shutil.copytree('content/quests',bundle/'quests')
    manifest=Path('content/development-gameplay-classic-public-life.json' if player_death else 'content/development-gameplay-classic-public-attacks.json')
    shutil.copy2(manifest,bundle)
    if locomotion:
        creature=bundle/'gameplay/development-creature-deflect-pursuit.json'
        data=json.loads(creature.read_text());data['aggro']['perception_metres']=1.0
        creature.write_text(json.dumps(data)+'\n')
    owner=output/'owner';observer=output/'observer';owner.mkdir();observer.mkdir()
    os.environ.setdefault('DISPLAY',':0')
    desktop=Desktop();launcher=None;second=None;second_account=None;second_log=None
    try:
        with Identity(output/'provider',Path('build/identity-sdk'),users=2,token_seconds=300) as identity, Cluster(output/'cluster',Path('build/postgres-sdk')) as cluster:
            env={k:v for k,v in os.environ.items() if not k.startswith('THEIA_') and k not in ['VK_INSTANCE_LAYERS','VK_LOADER_DEBUG','WGPU_VALIDATION','WGPU_DEBUG']}
            env.update(DISPLAY=':0',XDG_RUNTIME_DIR='/run/user/1000',LD_LIBRARY_PATH=str(Path('build/native-sdk/usr/lib/x86_64-linux-gnu').resolve()),THEIA_DATABASE_URL=cluster.test_environment()['THEIA_TEST_DATABASE_URL'],THEIA_OIDC_ISSUER=identity.issuer,THEIA_OIDC_CLIENT_ID=identity.client_id,THEIA_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,THEIA_TEST_BROWSER_ISSUER=identity.issuer,THEIA_TEST_BROWSER_REDIRECT=identity.redirect,THEIA_TEST_BROWSER_USER=identity.users[0][0],THEIA_TEST_BROWSER_PASSWORD=identity.users[0][1],BROWSER=f'{sys.executable} {Path("tools/qualify_account_launcher.py").resolve()} --owned-browser %s',THEIA_WORLD_INPUT_OS_QUALIFY=str(owner))
            if locomotion:env['THEIA_DIRECTIONAL_LOCOMOTION_QUALIFY']='1'
            with (owner/'launcher.log').open('w') as log:
                try:
                    command=[sys.executable,'tools/play.py','--account-login','--classic','--binaries',str(binaries),'--gameplay-manifest',str(bundle/manifest.name),'--players',str(binaries/'art/player'),'--creature-asset',str(binaries/'art/wolf/wolf.glb'),'--dev-spawn','8002,8002','--guide-position','8004,8000','--window-size','1920x1200']
                    launcher=subprocess.Popen(command,env=env,stdin=subprocess.PIPE,stdout=log,stderr=subprocess.STDOUT,text=True)
                    launcher.stdin.write('PublicArden\n');launcher.stdin.flush()
                    state_a=lambda:read_state(owner,lambda:launcher.poll() is None)
                    a=wait(lambda:state_a() if state_a().get('connected') and state_a().get('public_creature_attack') and state_a().get('enemy_screen') else None,60)
                    window_a=wait(lambda:owned_window(a['pid']))
                    server_log=Path('build/development-session/server.log')
                    ready=next(json.loads(line.removeprefix('account_game_ready ')) for line in server_log.read_text().splitlines() if line.startswith('account_game_ready '))
                    base=ready['account_address']
                    second_account=AccountClient(base,alive=lambda:launcher.poll() is None)
                    opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),_NoRedirect())
                    def browser(url):
                        callback=identity.authorize(url,user_index=1)
                        with opener.open(base+'/callback?'+urllib.parse.urlsplit(callback).query,timeout=8) as response:
                            assert response.status==200 and response.headers.get('Cache-Control')=='no-store'
                        return True
                    second_account.login(open_browser=browser)
                    character=second_account.choose_character(read=lambda _:'PublicSelene',write=lambda _:None)
                    ticket=output/'observer.token'
                    address=second_account.enter(character,ticket)
                    client_args=[part.decode() for part in Path(f'/proc/{a["pid"]}/cmdline').read_bytes().split(b'\0') if part]
                    assert '--public-creature-attacks' in client_args
                    connect=client_args.index('--renet-connect')
                    client_args[connect+1:connect+3]=[address,str(ticket)]
                    observer_env=client_environment(env)
                    for key in list(observer_env):
                        if key.startswith('THEIA_TEST_BROWSER_') or key in ['THEIA_DATABASE_URL','THEIA_OIDC_ISSUER','THEIA_OIDC_CLIENT_ID','THEIA_ACCOUNT_BIND','BROWSER']:
                            observer_env.pop(key,None)
                    observer_env['THEIA_WORLD_INPUT_OS_QUALIFY']=str(observer)
                    second_log=(observer/'client.log').open('w')
                    second=subprocess.Popen(client_args,env=observer_env,stdout=second_log,stderr=subprocess.STDOUT)
                    state_b=lambda:read_state(observer,lambda:second.poll() is None)
                    b=wait(lambda:state_b() if state_b().get('connected') and state_b().get('public_creature_attack') and state_b().get('snapshot_players')==2 else None,60)
                    window_b=wait(lambda:owned_window(b['pid']))
                    assert a['actor']!=b['actor']
                    locomotion_proof=None
                    if locomotion:
                        from directional_locomotion_os import checks
                        locomotion_proof=checks(desktop,window_a,window_b,state_a,state_b,output,face)
                    desktop.focus(window_b)
                    time.sleep(.3)
                    desktop.key_state('e',True)
                    try:time.sleep(6.4 if player_death else 1.4)
                    finally:desktop.key_state('e',False)
                    desktop.focus(window_a)
                    time.sleep(.3)
                    for _ in range(35):
                        a=state_a();target=a['enemy_global'];feet=a['dodge']['authority_feet']
                        distance=math.hypot(target[0]-feet[0],target[2]-feet[2])
                        if distance<2.5 or a['combat']['incoming_windups']>0:break
                        face(desktop,window_a,state_a,target)
                        desktop.key_state('w',True)
                        try:time.sleep(min(.3,max(.04,(distance-2.3)/5)))
                        finally:desktop.key_state('w',False)
                        time.sleep(.1)
                    else:raise AssertionError('Owner did not approach wolf')
                    a=wait(lambda:state_a() if state_a()['combat']['incoming_windups']>0 and 0<state_a()['combat']['player_health']<100 else None,15)
                    b=wait(lambda:state_b() if state_b()['creature_visual']['remote_victim_frames']>0 and state_b()['combat']['public_attack_windups']>=2 else None,15)
                    assert b['combat']['incoming_windups']==0 and b['combat']['player_health']==100
                    assert b['selected_hostile'] is None
                    distance=math.dist(a['dodge']['authority_feet'],b['dodge']['authority_feet'])
                    assert distance>2.,'Observer must be distinct from the victim position'
                    def pending(s):
                        return (s.get('public_creature_attack') or {}).get('windup') or {}
                    b=wait(lambda:state_b() if pending(state_b()).get('target')==a['actor'] else None,10)
                    lifecycle = None
                    if player_death:
                        def avatar(state, actor):
                            return next((a for a in (state.get('public_avatar_life') or {}).get('actors', []) if a['actor']==actor), None)
                        def capture(folder, name, state):
                            (folder/'capture.request').write_text(name+'\n')
                            wait(lambda:capture_ready(folder/'input-os.png'))
                            (folder/'input-os.png').rename(folder/name)
                            (folder/(name+'.json')).write_text(json.dumps(state(),indent=2)+'\n')
                        assert '--public-avatar-life' in client_args
                        desktop.focus(window_b)
                        time.sleep(.3)
                        face(desktop,window_b,state_b,state_a()['dodge']['authority_feet'])
                        # Frame the distant remote body using ordinary LMB orbit;
                        # a valid off-screen animation is not visible native proof.
                        width,height,scale=state_b()['window']
                        desktop.move(window_b,[width*.55,height*.42],scale)
                        desktop.button(1,True)
                        try:desktop.look(0,round((.10-state_b()['camera_angles'][1])/.003))
                        finally:desktop.button(1,False)
                        def visible_remote(s):
                            point=next((r['screen'] for r in s.get('remote_player_screens',[]) if r['actor']==a['actor']),None)
                            return point and 100<point[0]<width-100 and 180<point[1]<height-280
                        wait(lambda:visible_remote(state_b()),5)
                        desktop.focus(window_a)
                        time.sleep(.3)
                        desktop.key('i')
                        wait(lambda:state_a()['inventory_open'])
                        scale=state_a()['window'][2]
                        desktop.click(window_a,wait(lambda:state_a()['wand_slot_screen']),scale,1)
                        desktop.click(window_a,wait(lambda:state_a()['equip_screen']),scale,1)
                        wait(lambda:state_a()['main_hand_item']=='theia.dev.training-wand')
                        desktop.key('Escape')
                        desktop.key('Tab')
                        wait(lambda:state_a()['selected_hostile'] is not None)
                        face(desktop,window_a,state_a,state_a()['enemy_global'])
                        prior_casts=state_a()['combat']['started']
                        desktop.key('1')
                        wait(lambda:state_a()['combat']['started']>prior_casts,5)
                        live_casts=state_a()['combat']['started']
                        original_generation=wait(lambda:avatar(state_b(),a['actor']))['generation']
                        dead=wait(lambda:state_a() if state_a()['combat']['player_health']==0 else None,90)
                        corpse=wait(lambda:state_b() if avatar(state_b(),a['actor']) and avatar(state_b(),a['actor'])['dead'] and state_b()['player_life_animation']['remote_dead_frames']>0 and state_b()['player_life_animation']['maximum_remote_death_pose_changes']>0 else None,2)
                        assert corpse['combat']['player_health']==100 and corpse['combat']['incoming_windups']==0
                        desktop.focus(window_a)
                        time.sleep(.3)
                        before=state_a()['dodge']['authority_feet']
                        desktop.key_state('w',True)
                        try:
                            desktop.key('1')
                            time.sleep(.3)
                        finally:desktop.key_state('w',False)
                        held=state_a()
                        assert held['combat']['player_health']==0
                        assert math.hypot(held['dodge']['authority_feet'][0]-before[0],held['dodge']['authority_feet'][2]-before[2])<.02
                        assert held['active_cast'] is None
                        assert held['combat']['started']==live_casts
                        capture(owner,'owner-dead-held.png',state_a)
                        corpse=wait(lambda:state_b() if avatar(state_b(),a['actor'])['dead'] and state_b()['player_life_animation']['death_finished_frames']>0 else None,2)
                        assert visible_remote(corpse)
                        desktop.focus(window_b)
                        time.sleep(.2)
                        capture(observer,'remote-death.png',state_b)
                        recovered=wait(lambda:state_b() if avatar(state_b(),a['actor']) and not avatar(state_b(),a['actor'])['dead'] and avatar(state_b(),a['actor'])['generation']==original_generation+1 and state_b()['player_life_animation']['remote_life_restarts']>0 and state_b()['avatar_pose_resets']>=2 else None,15)
                        assert recovered['combat']['player_health']==100 and recovered['combat']['incoming_windups']==0
                        assert recovered['selected_hostile'] is None
                        assert visible_remote(recovered)
                        desktop.focus(window_b)
                        time.sleep(.3)
                        capture(observer,'remote-fresh-life.png',state_b)
                        lifecycle=dict(passed=True,old_generation=original_generation,new_generation=avatar(recovered,a['actor'])['generation'],remote_death_starts=corpse['player_life_animation']['remote_death_starts'],remote_dead_frames=corpse['player_life_animation']['remote_dead_frames'],remote_death_pose_changes=corpse['player_life_animation']['maximum_remote_death_pose_changes'],remote_life_restarts=recovered['player_life_animation']['remote_life_restarts'],avatar_pose_resets=recovered['avatar_pose_resets'],remote_avatar_in_view=True,remote_corpse_pose_finished=True,owner_live_cast_verified=True,owner_dead_input_held=True,owner_dead_cast_count_unchanged=True,observer_health=recovered['combat']['player_health'])
                    for folder,state,window in [(owner,state_a,window_a),(observer,state_b,window_b)]:
                        desktop.focus(window)
                        time.sleep(.3)
                        if folder==observer:face(desktop,window,state,state()['enemy_global'])
                        (folder/'capture.request').write_text('public-wolf.png\n')
                        wait(lambda:capture_ready(folder/'input-os.png'))
                        (folder/'input-os.png').rename(folder/'public-wolf.png')
                        (folder/'proof-state.json').write_text(json.dumps(state(),indent=2)+'\n')
                    proof=dict(passed=True,scope='Normal versioned OIDC/PostgreSQL/encrypted native Linux clients; synthetic PID-bound X11 input; no Windows/capacity claim',owner_actor=a['actor'],observer_actor=b['actor'],owner_health=a['combat']['player_health'],observer_health=b['combat']['player_health'],observer_private_windups=b['combat']['incoming_windups'],observer_public_windups=b['combat']['public_attack_windups'],observer_remote_victim_frames=b['creature_visual']['remote_victim_frames'],observer_selected_target=b['selected_hostile'],distinct_positions_metres=distance,adapter=b['adapter'],window=b['window'])
                    proof['manifest_schema']=15 if player_death else 14
                    proof['public_player_lifecycle']=lifecycle
                    proof['directional_locomotion']=locomotion_proof
                finally:
                    if second is not None and second.poll() is None:
                        second.terminate();second.wait(timeout=10)
                    if second_log is not None:second_log.close()
                    if second_account is not None:second_account.close()
                    (output/'observer.token').unlink(missing_ok=True)
                    if launcher is not None:
                        if launcher.poll() is None:
                            launcher.send_signal(signal.SIGINT)
                            try:launcher.wait(timeout=30)
                            except subprocess.TimeoutExpired:launcher.kill();launcher.wait()
                        launcher.stdin.close()
            shutil.copy2('build/development-session/server.log',output/'server.log')
            checks=0
            for path in [owner/'launcher.log',observer/'client.log']:
                checks += check_gpu_diagnostics(path.read_text())
            assert not checks,checks
            proof['gpu_diagnostics']=0
            assert 'development_public_creature_attacks enabled=true contract=NA01' in (output/'server.log').read_text()
        assert not any((output/'cluster'/name).exists() for name in ['password','pgpass'])
        proof['owned_services_stopped']=True
        (output/'verification.json').write_text(json.dumps(proof,indent=2)+'\n')
        return proof
    finally:
        desktop.close()


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--binaries',type=Path,required=True)
    parser.add_argument('--player-death',action='store_true',help='Normal schema-15 remote death/fresh-life and owner input barrier')
    parser.add_argument('--locomotion',action='store_true',help='Directional local/remote grip/attack checks; copied 1-m passive perception, requires --player-death')
    args=parser.parse_args()
    print(json.dumps(run(args.output,args.binaries,args.player_death,args.locomotion),indent=2))
