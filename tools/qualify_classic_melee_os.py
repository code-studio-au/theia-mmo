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


def angle(value):return math.atan2(math.sin(value),math.cos(value))


def checks(desktop,window,state,folder,remapped,production=False,abilities=False,physical=False,white=False,ability_damage=False,engagement_only=False,membership=False,return_reset=False):
    reset_proof=None
    scale=state()['window'][2]
    def feet():return state()['dodge']['authority_feet']
    def swings():return state()['autoattack']['swings']
    def hold(key,seconds):
        desktop.key_state(key,True)
        try:time.sleep(seconds)
        finally:desktop.key_state(key,False)
        time.sleep(.25)
    def face(yaw):
        for _ in range(5):
            error=angle(yaw-state()['classic_facing']*math.tau/65536)
            if abs(error)<.04:return
            width,height,_=state()['window']
            desktop.move(window,[width*.5,height*.45],scale)
            desktop.button(3,True)
            error=angle(yaw-state()['camera_angles'][0])
            desktop.look(round(max(-400,min(400,-error/.004))),0)
            desktop.button(3,False);time.sleep(.25)
        assert abs(angle(yaw-state()['classic_facing']*math.tau/65536))<.1,state()
    def toward():
        a=feet();b=state()['enemy_global'];face(math.atan2(-(b[0]-a[0]),-(b[2]-a[2])))
    def distance():
        a=feet();b=state()['enemy_global'];return math.hypot(b[0]-a[0],b[2]-a[2])
    def approach():
        for _ in range(40):
            toward()
            if distance()<2.2:return
            hold('w',min(.4,max(.02,(distance()-2.0)/5)))
        raise AssertionError('Cannot approach target through normal movement')
    def click_hostile(button):
        before=state()['controls']['enemy_clicks']
        for dx,dy in [(20,-25),(-20,-25),(0,-30),(0,0),(35,0),(-35,0)]:
            point=wait(lambda:state()['enemy_screen'])
            desktop.click(window,[point[0]+dx,point[1]+dy],scale,button)
            time.sleep(.25)
            if state()['controls']['enemy_clicks']>before:return
        raise AssertionError('No visible wolf click reached the actor')
    def capture(name):
        (folder/'capture.request').write_text(name+'\n')
        wait(lambda:capture_ready(folder/'input-os.png'))
        (folder/'input-os.png').rename(folder/name)
    def equip(which):
        wanted='theia.dev.training-'+which
        if state()['main_hand_item']==wanted:return
        if not state()['inventory_open']:desktop.key('i')
        wait(lambda:state()['inventory_open'])
        desktop.click(window,wait(lambda:state()[which+'_slot_screen']),scale,1)
        desktop.click(window,wait(lambda:state()['equip_screen']),scale,1)
        wait(lambda:state()['main_hand_item']==wanted)
        if which=='sword':wait(lambda:state()['sword_equipped'])
        desktop.key('Escape');wait(lambda:not state()['inventory_open'])
    wait(lambda:state()['autoattack'] is not None)
    if not state()['sword_equipped']:
        if not state()['inventory_open']:desktop.key('i')
        wait(lambda:state()['inventory_open'])
        desktop.click(window,wait(lambda:state()['sword_slot_screen']),scale,1)
        desktop.click(window,wait(lambda:state()['equip_screen']),scale,1)
        wait(lambda:state()['sword_equipped'])
    if state()['inventory_open']:desktop.key('Escape')
    if production:
        wait(lambda:state()['combat']['incoming_windups']>0 and state()['combat']['player_health']<100)
        assert state()['combat']['melee_damage']==0 and state()['combat']['started']==0
        # Chase counters are checked after graceful shutdown; the enemy can
        # already have arrived while account login and equipment UI complete.
        if engagement_only:
            # No outgoing attacks: keep the ordinary wolf alive past the old
            # deadline using unmodified data and only read-only observations.
            started=time.monotonic();last_progress=started
            incoming=state()['combat']['incoming_physical_damage']
            while time.monotonic()-started < 35:
                current=state()['combat']
                assert current['player_deaths']==0 and current['deaths']==0,current
                assert current['event_gaps']==0,current
                assert current['started']==0 and state()['autoattack']['swings']==0,'Unexpected outgoing input during passive melee qualification'
                if current['incoming_physical_damage']>incoming:
                    incoming=current['incoming_physical_damage'];last_progress=time.monotonic()
                assert time.monotonic()-last_progress < 8,'Wolf stopped attacking during stationary melee'
                time.sleep(.1)
            capture('ongoing-melee-after-35-seconds.png')
            proof={'passed':True,'unmodified_gameplay_profiles':True,'stationary_contact_seconds':time.monotonic()-started,'no_outgoing_attacks':state()['autoattack']['swings']==0 and state()['combat']['started']==0 and state()['combat']['ability_events']==0,'combat':state()['combat']}
            assert proof['no_outgoing_attacks']
            (folder/'classic-melee.json').write_text(json.dumps(proof,indent=2)+'\n')
            return proof
    approach()
    heading=state()['classic_facing'];click_hostile(1)
    target=wait(lambda:state()['selected_hostile'])
    assert swings()==0 and state()['classic_facing']==heading
    capture('before-engage.png')
    (folder/'before-engage.json').write_text(json.dumps(state(),indent=2))
    click_hostile(3)
    time.sleep(1)
    capture('after-engage.png')
    (folder/'after-engage.json').write_text(json.dumps(state(),indent=2))
    wait(lambda:swings()>=1)
    assert abs(angle((state()['classic_facing']-heading)*math.tau/65536))<.01
    wait(lambda:state()['attack_animation']['local_starts']>=1)
    wait(lambda:state()['melee_labels']['visible_melee_frames']>0)
    if physical:
        wait(lambda:state()['combat']['physical_damage']>0)
        assert state()['combat']['last_physical_resolved']<=(2 if white else 1)*state()['combat']['last_physical_starting']
        if white:assert state()['combat']['ordinary_events']>0
    capture('first-melee.png')
    if return_reset:
        desktop.key('BackSpace')
        wait(lambda:state()['autoattack']['phase']=='Idle' and state()['selected_hostile'] is None)
        click_hostile(1);wait(lambda:state()['selected_hostile']==target)
        damaged=state();assert 0 < damaged['combat']['last_health'] < damaged['combat']['max_health']
        generation=damaged['combat']['player_generation'];start=feet()
        toward();face(state()['classic_facing']*math.tau/65536+math.pi)
        hold('w',10)
        wait(lambda:state()['combat']['in_combat'] is False and state()['combat']['last_health']==state()['combat']['max_health'],20)
        restored=state();c=restored['combat']
        assert restored['selected_hostile']==target and c['player_generation']==generation
        assert c['deaths']==0 and c['respawns']==0 and c['player_deaths']==0 and c['player_respawns']==0 and c['event_gaps']==0
        travelled=math.hypot(feet()[0]-start[0],feet()[2]-start[2]);assert travelled>5,travelled
        capture('returned-full-health.png')
        reset_proof={'damaged_health':damaged['combat']['last_health'],'restored_health':c['last_health'],'same_npc':target,'player_generation':generation,'travelled_metres':travelled,'no_death_or_respawn_before_reacquisition':True}
        incoming=c['incoming_windups'];approach();toward()
        wait(lambda:state()['combat']['in_combat'] is True and state()['combat']['incoming_windups']>incoming,15)
        click_hostile(3);previous=swings();wait(lambda:swings()>previous,6)
        capture('reacquired-after-reset.png')
        reset_proof['reacquired']=True
    if production:
        previous=swings();wait(lambda:swings()>previous)
        capture('active-swing.png')
        if ability_damage:
            # Use real combat/input: prepare a low-health wolf, stop basic swings,
            # then require the replacement special to supply the fatal fact.
            wait(lambda:0 < state()['combat']['last_health'] <= 16 or state()['combat']['deaths']>0,40)
            assert state()['combat']['deaths']==0, 'Basic swing killed before the ability-fatal check'
            desktop.key('BackSpace');wait(lambda:state()['autoattack']['phase']=='Idle' and state()['selected_hostile'] is None)
            click_hostile(1);wait(lambda:state()['selected_hostile'] is not None);toward()
            fatal=state()['combat']['ability_fatal_hits']
            for _ in range(8):
                if state()['combat']['deaths']>0:break
                approach();toward()
                offered=state()['combat']['ability_events'];rejected=state()['combat']['rejected_requests'];desktop.key('2')
                wait(lambda:state()['combat']['ability_events']>offered or state()['combat']['deaths']>0 or state()['combat']['rejected_requests']>rejected,4)
                if state()['combat']['deaths']==0:time.sleep(1.7)
            wait(lambda:state()['combat']['deaths']>0,4)
            assert state()['combat']['ability_fatal_hits']>fatal
            wait(lambda:sum(state()['melee_labels']['ability_visible']['special'].values())>0)
        else:
            desktop.key('2')
            wait(lambda:state()['combat']['deaths']>0,30)
        capture('wolf-defeated.png')
        if physical:
            wait(lambda:state()['melee_labels']['visible_incoming_frames']>0)
            assert state()['combat']['incoming_physical_damage']>0
        final=state();assert final['combat']['player_deaths']==0
        proof={'passed':True,'unmodified_gameplay_profiles':True,'passive_perception_chase_and_attack':True,'right_click_autoattack':True,'wolf_dies':True,'combat':final['combat'],'animation':final['attack_animation'],'damage_labels':final['melee_labels']}
        if ability_damage:proof['typed_ability_killing_hit']=True
        if reset_proof is not None:proof['return_reset']=reset_proof
        (folder/'classic-melee.json').write_text(json.dumps(proof,indent=2)+'\n');return proof
    toward();heading=state()['classic_facing']*math.tau/65536
    face(heading+math.pi)
    wait(lambda:state()['autoattack']['phase']=='Paused(NotFacing)')
    paused=swings();time.sleep(2.65)
    assert swings()==paused and state()['selected_hostile']==target
    capture('not-facing.png')
    toward();wait(lambda:swings()>paused)
    hold('s',.65)
    wait(lambda:state()['autoattack']['phase']=='Paused(OutOfRange)')
    paused=swings();time.sleep(2.65)
    assert swings()==paused and state()['selected_hostile']==target
    capture('out-of-range.png')
    approach();wait(lambda:swings()>paused)
    desktop.key('BackSpace');wait(lambda:state()['selected_hostile'] is None)
    wait(lambda:state()['autoattack']['phase']=='Idle')
    stopped=swings();time.sleep(2.65);assert swings()==stopped
    spell_proof=None
    if abilities:
        equip('wand')
        desktop.key('Tab');wait(lambda:state()['selected_hostile'] is not None)
        toward();forward=state()['classic_facing']*math.tau/65536;face(forward+math.pi)
        desktop.key('1')
        wait(lambda:state()['ability_reply']=='Rejected(NotFacing)')
        assert state()['combat']['last_mana']==100
        capture('ability-not-facing.png')
        toward();launches=state()['combat']['projectile_launches'];started=state()['combat']['started']
        desktop.key('1');wait(lambda:state()['combat']['started']>started)
        spell_proof={'cast_ticks':state()['combat']['last_cast_ticks'],'rate':state()['dodge']['rate'],'ability':state()['combat']['last_cast_ability']}
        assert spell_proof['cast_ticks']>=spell_proof['rate']//2
        if state()['active_cast']:capture('cast-time-ability.png')
        wait(lambda:state()['combat']['projectile_launches']>launches)
        wait(lambda:state()['combat']['minimum_mana']<100)
        wait(lambda:state()['active_cast'] is None)
        assert state()['autoattack']['phase']=='Idle'
        if ability_damage:
            wait(lambda:sum(state()['melee_labels']['ability_visible']['arcane'].values())>0)
            capture('arcane-impact.png')
        equip('sword');time.sleep(1.6)
    if ability_damage and state()['combat']['deaths']>0:
        # Outcome RNG can finish a fragile control target early. Continue the
        # separate instant-ability check on its actual fresh incarnation.
        needed=state()['combat']['deaths']
        wait(lambda:state()['combat']['respawns']>=needed,10)
        approach();click_hostile(1);wait(lambda:state()['selected_hostile'] is not None)
    prior_deaths=state()['combat']['deaths']
    # The instant sword ability remains distinct from persistent attack intent.
    if state()['selected_hostile'] is None:desktop.key('Tab')
    wait(lambda:state()['selected_hostile'] is not None)
    toward()
    started=state()['combat']['started'];desktop.key('2')
    wait(lambda:state()['combat']['started']>started)
    assert state()['autoattack']['phase']=='Idle'
    time.sleep(.25)
    if state()['combat']['deaths']==prior_deaths:
        approach();click_hostile(3)
        wait(lambda:state()['combat']['deaths']>prior_deaths,30)
    capture('wolf-defeated.png')
    final=state();proof={'passed':True,'cast_time_ability':spell_proof,'left_click_only_selects':True,'right_click_engages':True,'manual_facing_pause_resume':True,'range_pause_resume':True,'explicit_clear_stops':True,'instant_ability_is_separate':True,'wolf_dies':True,'combat':final['combat'],'animation':final['attack_animation'],'damage_labels':final['melee_labels']}
    (folder/'classic-melee.json').write_text(json.dumps(proof,indent=2)+'\n');return proof


def physical_authority_proof(log,white=False,ability_damage=False,membership=False):
    """Check graceful normal-server summaries; never read provider credentials."""
    records = {}
    for line in log.read_text().splitlines():
        for prefix in ['transport_summary ', 'account_persistence_summary ']:
            if line.startswith(prefix):
                assert prefix not in records, 'Duplicate authority summary'
                records[prefix] = json.loads(line[len(prefix):])
    transport = records['transport_summary ']
    damage = transport['white_melee' if white else 'physical_damage']
    persistence = records['account_persistence_summary ']
    worker = persistence['worker']
    if white:
        assert damage['incoming']['attempts']>0 and damage['outgoing']['attempts']>0
        assert damage['base_draws']==damage['outcome_draws']==damage['incoming']['attempts']+damage['outgoing']['attempts']
        assert transport['physical_damage']['base_draws']==0
    else:
        assert damage['incoming_hits'] > 0 and damage['outgoing_hits'] > 0
        assert damage['base_draws'] == damage['incoming_hits'] + damage['outgoing_hits']
    assert transport['experience_opportunities_forwarded'] == 1
    assert transport['reward_opportunities_forwarded'] == 1
    assert transport['progression_states_sent'] >= 2
    assert transport['reward_states_sent'] >= 2
    assert not persistence['failed'] and not worker['failed'] and worker['error'] is None
    assert worker['experience_committed'] == 1 and worker['experience_replayed'] == 0
    assert worker['reward_offers'] == 1 and worker['reward_rejected'] == 0
    if membership:
        assert 120 <= transport['life_max_frame_bytes'] <= 144
        assert transport['life_states_sent']>0 and transport['life_backpressure_disconnects']==0
    proof={'damage': damage, 'durable_experience_commits': worker['experience_committed'],
           'reward_offers': worker['reward_offers'], 'loot_claim_tested': False}
    if ability_damage:
        ability=transport['ability_damage']
        assert ability['special']['attempts']>0
        assert ability['base_draws']==ability['landing_draws']==ability['special']['attempts']+ability['arcane']['attempts']
        assert ability['critical_draws']==sum(ability[k]['hits']+ability[k]['criticals'] for k in ['special','arcane'])
        assert ability['refusals']==0 and transport['hit_outcomes']['random_draws']==0
        proof['ability_damage']=ability
    return proof


def run(output,binaries,sizes=None,production=False,abilities=False,physical=False,white=False,ability_damage=False,engagement_only=False,membership=False,credit=False,reset=False,return_reset=False):
    reset=reset or return_reset
    credit=credit or reset
    membership=membership or credit
    ability_damage=ability_damage or membership
    white=white or ability_damage or engagement_only
    production=production or engagement_only or return_reset
    physical=physical or white
    output=output.resolve();binaries=binaries.resolve();output.mkdir(mode=0o700,parents=True,exist_ok=False)
    bundle=output/'gameplay';shutil.copytree('content/gameplay',bundle)
    if not production:
        creature=bundle/'development-creature-deflect-pursuit.json';data=json.loads(creature.read_text());
        if white:
            data['aggro']['perception_metres']=1.0
            data['aggro']['chase_speed']=0.1
        else:data.pop('aggro',None)
        data['speed_metres_per_second']=0.1;creature.write_text(json.dumps(data)+'\n')
        retaliation=bundle/'development-creature-retaliation.json';data=json.loads(retaliation.read_text());data['damage']=1;retaliation.write_text(json.dumps(data)+'\n')
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
                # A disconnected physical display has no reliable VSync clock.
                # Use the existing explicit presentation diagnostic only on that desktop.
                disconnected=not re.search(r"^\S+ connected",subprocess.check_output(["xrandr","--current"],env=env,text=True),re.M)
                if disconnected:env["THEIA_PERFORMANCE_UNCAPPED"]="1"
                with (folder/'launcher.log').open('w') as log:
                    command=[sys.executable,'tools/play.py','--account-login','--classic','--binaries',str(binaries),'--gameplay-manifest',str(bundle/('development-gameplay-classic-reset.json' if reset else 'development-gameplay-classic-credit.json' if credit else 'development-gameplay-classic-membership.json' if membership else 'development-gameplay-classic-ability-damage.json' if ability_damage else 'development-gameplay-classic-outcomes.json' if white else 'development-gameplay-classic-physical.json' if physical else 'development-gameplay-classic-melee.json')),'--players',str(binaries/'art/player'),'--creature-asset',str(binaries/'art/wolf/wolf.glb'),'--dev-spawn','8002,8002','--window-size',size]
                    if remapped:command.extend(['--controls-file',str(remap_path)])
                    if size=='3440x1440' and not disconnected:command.append('--fullscreen')
                    launcher=subprocess.Popen(command,env=env,stdin=subprocess.PIPE,stdout=log,stderr=subprocess.STDOUT,text=True)
                    try:
                        launcher.stdin.write(('' if index==0 else 'n\n')+'MeleeArden'+chr(65+index)+'\n');launcher.stdin.flush()
                        def state():
                            assert launcher.poll() is None,'Owned launcher exited early'
                            return wait(lambda:json.loads((folder/'input-os.json').read_text()),2)
                        initial=wait(lambda:state() if state()['connected'] and state()['guide_screen'] else None,40)
                        expected=[int(v) for v in size.split('x')]
                        assert initial['window'][:2]==expected,('Actual window differs from requested size',initial['window'],expected)
                        status=Path(f'/proc/{initial["pid"]}/status').read_text()
                        assert int(re.search(r'^PPid:\s+(\d+)',status,re.M)[1])==launcher.pid
                        window=wait(lambda:owned_window(initial['pid']));desktop.focus(window);time.sleep(2)
                        proof=checks(desktop,window,state,folder,remapped,production,abilities,physical,white,ability_damage,engagement_only,membership,return_reset)
                        if membership:
                            if engagement_only:
                                wait(lambda:state()['combat']['in_combat'] is True and state()['combat_layout']['combat_label_frames']>0)
                            else:
                                wait(lambda:state()['combat']['in_combat'] is False and state()['combat_layout']['peace_label_frames']>0)
                            final_membership=state();combat=final_membership['combat'];layout=final_membership['combat_layout']
                            assert combat['membership_events']>0 and combat['combat_entries']>0 and layout['combat_label_frames']>0
                            if not engagement_only:assert combat['combat_exits']>0 and combat['membership_opponents']==0
                            proof['combat']=combat
                            proof['membership']={'authoritative':True,'in_combat':combat['in_combat'],'entries':combat['combat_entries'],'exits':combat['combat_exits'],'combat_glyph_frames':layout['combat_label_frames'],'peace_glyph_frames':layout['peace_label_frames'],'status':final_membership['combat_ui']['life_status']}
                            (folder/'classic-melee.json').write_text(json.dumps(proof,indent=2)+'\n')
                        (folder/'capture.request').write_text('capture owned classic window\n');wait(lambda:capture_ready(folder/'input-os.png'))
                        final=state()
                        if production:
                            children=Path(f'/proc/{launcher.pid}/task/{launcher.pid}/children').read_text().split()
                            servers=[int(pid) for pid in children if Path(f'/proc/{pid}/exe').resolve()==binaries/'theia-server']
                            assert len(servers)==1,'Expected one owned server process'
                            os.kill(servers[0],signal.SIGINT)
                        else:launcher.send_signal(signal.SIGINT)
                        assert launcher.wait(timeout=15)==0
                        if production:
                            shutil.copy2('build/development-session/server.log',folder/'server.log')
                            summaries=[]
                            for line in (folder/'server.log').read_text().splitlines():
                                try:record=json.loads(line)
                                except ValueError:continue
                                if 'creature_pursuit_summary' in record:summaries.append(record['creature_pursuit_summary'])
                            assert len(summaries)==1,'Missing graceful pursuit summary'
                            pursuit=summaries[0]
                            assert pursuit['acquisitions']>0 and pursuit['chase_ticks']>0 and pursuit['attack_ticks']>0,pursuit
                            proof['authority_pursuit']=pursuit
                            if engagement_only:
                                assert pursuit['returns']==0 and pursuit['acquisitions']==1,pursuit
                            elif physical:proof['authority_physical']=physical_authority_proof(folder/'server.log',white,ability_damage,membership)
                            if reset:
                                resets=[json.loads(line)['creature_reset_summary'] for line in (folder/'server.log').read_text().splitlines() if line.startswith('{"creature_reset_summary":')]
                                assert len(resets)==1,'Missing graceful reset summary'
                                proof['authority_reset']=resets[0]
                                if return_reset:
                                    assert resets[0]['resets']>=1 and resets[0]['health_restored']>0
                                    assert pursuit['returns']>=1 and pursuit['acquisitions']>=2
                                else:assert resets[0]['resets']==0

                            (folder/'classic-melee.json').write_text(json.dumps(proof,indent=2)+'\n')
                        gpu=check_gpu_diagnostics((folder/'launcher.log').read_text(),allow_surface_resize=size=='3440x1440')
                        results[size]={'passed':True,'checks':proof,'window':final['window'],'disconnected_display_uncapped':disconnected,'fullscreen_requested':size=='3440x1440' and not disconnected,'adapter':final['adapter'],'gpu_diagnostics':gpu}
                        (output/'partial-results.json').write_text(json.dumps(results,indent=2)+'\n')
                    finally:
                        if launcher.poll() is None:
                            launcher.send_signal(signal.SIGINT)
                            try:launcher.wait(timeout=15)
                            except subprocess.TimeoutExpired:launcher.kill();launcher.wait()
                        launcher=None
    finally:desktop.close()
    result={'passed':True,'scope':'Native classic basic melee using normal OIDC/PostgreSQL and X11 input to owned PID-matched windows; no complete combat milestone, physical hardware or Windows claim','fixture':'Unmodified checked-in classic melee gameplay profiles' if production else 'Copied gameplay data disables wolf perception, slows patrol and sets retaliation damage to 1 to isolate melee controls; normal OIDC/PostgreSQL launcher, no telemetry gameplay input','results':results}
    result['physical_profile'] = physical
    result['ordinary_profile'] = white
    result['ability_profile'] = ability_damage
    result['engagement_only'] = engagement_only
    result['membership_profile'] = membership
    result['credit_profile'] = credit
    result['reset_profile'] = reset
    result['return_reset'] = return_reset
    if physical and not production:
        result['fixture'] = 'Copied gameplay data disables perception and slows patrol; physical NPC damage remains defined by the explicit physical profile. Normal OIDC/PostgreSQL and OS input.'
    if white and not production:
        result['fixture']='Copied gameplay data uses 1 m perception and 0.1 m/s patrol/chase to isolate controls; ordinary damage/outcome definitions unchanged. Normal OIDC/PostgreSQL and owned OS input.'
    (output/'verification.json').write_text(json.dumps(result,indent=2)+'\n');return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path);parser.add_argument('--binaries',type=Path,required=True)
    parser.add_argument('--creature-reset',action='store_true',help='Use schema-11 return health/effect reset')
    parser.add_argument('--return-reset',action='store_true',help='Normal wolf: damage, leave, observe restoration, reacquire and kill')
    parser.add_argument('--solo-credit',action='store_true',help='Use schema-10 solo tag-owner credit')
    parser.add_argument('--membership',action='store_true',help='Use PL02 combat state and require visible entry/release indicators')
    parser.add_argument('--engagement-only',action='store_true',help='Unmodified wolf: require 35 seconds of uninterrupted passive melee, without killing it')
    parser.add_argument('--sizes',nargs='+',choices=['1920x1200','3440x1440','1280x720'])
    parser.add_argument('--production',action='store_true',help='Use unmodified profiles for perception, chase and kill checks')
    parser.add_argument('--abilities',action='store_true',help='Also test wand casting and manual ability facing in the control fixture')
    parser.add_argument('--physical',action='store_true',help='Use explicit physical damage definitions and stat templates')
    parser.add_argument('--white',action='store_true',help='Use explicit ordinary-melee outcome profile')
    parser.add_argument('--ability-damage',action='store_true',help='Use typed special/Arcane damage and require an ability-fatal production kill')
    args=parser.parse_args()
    if args.engagement_only:args.production=True
    if args.abilities and (args.production or args.return_reset):parser.error('--abilities uses the isolated control fixture')
    result=run(args.output,args.binaries,args.sizes,args.production,args.abilities,args.physical,args.white,args.ability_damage,args.engagement_only,args.membership,args.solo_credit,args.creature_reset,args.return_reset);print(json.dumps({'passed':result['passed'],'sizes':list(result['results'])}))
