#!/usr/bin/env python3
"""Owned actual Keycloak/PostgreSQL and native encrypted game admission checks."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
from private_identity import Identity, _NoRedirect
from account_login import AccountClient, AccountError
from private_postgres import Cluster
from gpu_validation import check_gpu_diagnostics


def run(output, identity_sdk, postgres_sdk, client, terrain, players, server_binary=None, inventory=False, validation=False, equipment=False, rewards=False, progression=False, window_size="1280x720", fullscreen=False, quests=False, dialogue=False, vendors=False, zone=False):
    if not zone and dialogue and any([inventory,equipment,rewards,progression,quests]):
        raise ValueError('Focused native dialogue fixture runs separately from item/combat automation')
    if not zone and vendors and any([inventory,equipment,rewards,progression,quests]):
        raise ValueError('Focused native vendor fixture runs separately from item/combat automation')
    if zone:inventory=equipment=rewards=progression=quests=dialogue=vendors=True
    dialogue = dialogue or vendors
    dialogue_profile=Path('content/gameplay/development-dialogue.json').resolve()
    guide_position='15044,15040' if zone else '15404,12860'
    progression = progression or quests
    rewards = rewards or progression
    equipment = equipment or rewards
    inventory = inventory or equipment
    item_catalog = Path('content/gameplay/development-equipment-items.json' if equipment or vendors else 'content/gameplay/development-items.json').resolve()
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    progression_profile=output/'qualification-progression.json'
    if progression:
        data=json.loads(Path('content/gameplay/development-progression.json').read_text())
        data['level_thresholds']=[0,25,50]
        progression_profile.write_text(json.dumps(data)+'\n')
    quest_profile=Path('content/gameplay/development-quest.json').resolve()
    combat_profile=Path('content/gameplay/development-abilities.json').resolve()
    if quests:
        data=json.loads(combat_profile.read_text())
        data['abilities'][0]['cast_ms']=0
        data['abilities'][0]['cooldown_ms']=0
        data['abilities'][0]['effect']['amount']=100
        combat_profile=output/'qualification-abilities.json'
        combat_profile.write_text(json.dumps(data)+'\n')
        combat_profile=combat_profile.resolve()
    objective_gate=(output/'allow-second-objective-finish').resolve()
    if server_binary is None:
        with (output/'build.log').open('w') as log:
            subprocess.run(['cargo','build','--locked','-p','theia-server','--example','account_native_qualification','-j2'],stdout=log,stderr=subprocess.STDOUT,check=True,timeout=600)
        server_binary=Path('target/debug/examples/account_native_qualification').resolve()
    binary=server_binary
    evidence={'server_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),
              'client_sha256':hashlib.sha256(client.read_bytes()).hexdigest(),
              'window_size':window_size,'fullscreen':fullscreen,
              'scope':'Actual local OIDC accounts, owned PostgreSQL and native Linux encrypted clients; no physical browser or Windows execution'}
    evidence['known_surface_resize_diagnostics']={}
    server=None;children=[];handles=[];bearers=[];private=[]
    with Identity(output/'provider',identity_sdk,users=2) as identity, Cluster(output/'cluster',postgres_sdk) as cluster:
        env=dict(cluster.test_environment(),THEIA_TEST_OIDC_ISSUER=identity.issuer,
                 THEIA_TEST_OIDC_CLIENT=identity.client_id,THEIA_TEST_ACCOUNT_BIND=urllib.parse.urlsplit(identity.redirect).netloc,
                 THEIA_TEST_SAVED_TERRAIN=str(terrain),THEIA_TEST_DIALOGUE='1' if dialogue else '0',THEIA_TEST_GUIDE_POSITION=guide_position,THEIA_TEST_VENDORS="1" if vendors else "0",THEIA_TEST_INVENTORY="1" if inventory or vendors else "0",THEIA_TEST_EQUIPMENT="1" if equipment or vendors else "0",THEIA_TEST_REWARDS="1" if rewards else "0",THEIA_TEST_PROGRESSION="1" if progression else "0",THEIA_TEST_PROGRESSION_PROFILE=str(progression_profile.resolve()),THEIA_TEST_QUESTS="1" if quests else "0",THEIA_TEST_COMBAT_PROFILE=str(combat_profile),THEIA_TEST_NAVIGATION=str(Path('build/navigation-terrain-far-v1').resolve()))
        private.extend([cluster.password]+[p for _,p in identity.users])
        opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),_NoRedirect())
        base=None;server_log=None;accounts={};ticket_counter=0
        def wait(predicate,seconds=25,label='Owned fixture timeout'):
            deadline=time.monotonic()+seconds
            while time.monotonic()<deadline:
                value=predicate()
                if value:return value
                time.sleep(.05)
            raise TimeoutError(label)
        def start(name,spawn):
            nonlocal server,base,server_log
            server_log=output/(name+'.log');log=server_log.open('w');handles.append(log)
            server=subprocess.Popen([str(binary)],env=dict(env,THEIA_TEST_SPAWN=spawn),stdout=log,stderr=subprocess.STDOUT)
            def ready():
                assert server.poll() is None,'Owned authority exited before readiness'
                return next((json.loads(line.removeprefix('account_game_ready ')) for line in server_log.read_text().splitlines() if line.startswith('account_game_ready ')),None)
            base=wait(ready)['account_address']
        def http(path,session=None,body=None,expected=200,method=None):
            data=json.dumps(body).encode() if body is not None else None
            headers={'Content-Type':'application/json'}
            if session:headers['Authorization']='Bearer '+session
            request=urllib.request.Request(base+path,data=data,headers=headers,method=method or ('POST' if body is not None else 'GET'))
            try:response=opener.open(request,timeout=8)
            except urllib.error.HTTPError as error:response=error
            with response:
                status=response.code;raw=response.read(65537)
                assert response.headers.get('Cache-Control')=='no-store'
            assert status==expected,f'Unexpected account API status {status}, wanted {expected}, path {path.split("?")[0]}'
            assert len(raw)<=65536
            if not raw:return None
            try:return json.loads(raw)
            except json.JSONDecodeError:return None
        def login(user):
            account=AccountClient(base,alive=lambda:server.poll() is None)
            def browser(url):
                private.append(url)
                callback=identity.authorize(url,user_index=user);private.append(callback)
                private.extend(urllib.parse.parse_qs(urllib.parse.urlsplit(callback).query).get('code',[]))
                http('/callback?'+urllib.parse.urlsplit(callback).query)
                return True
            account.login(open_browser=browser)
            session=account._session
            private.append(session);accounts[session]=account
            assert session not in repr(account)
            return session
        def enter(session,character,expected=200):
            nonlocal ticket_counter
            if expected!=200:return http('/characters/enter',session,{'character_id':character},expected)
            ticket_counter+=1
            path=output/f'launcher-{ticket_counter}.token';bearers.append(path)
            address=accounts[session].enter(character,path)
            assert path.stat().st_mode & 0o777==0o600
            data=list(path.read_bytes());path.unlink()
            return {'address':address,'ticket':data}
        def admissions():
            return [json.loads(line.removeprefix('account_admit ')) for line in server_log.read_text().splitlines() if line.startswith('account_admit ')]
        def ticket_file(name,ticket):
            path=output/(name+'.token');fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
            with os.fdopen(fd,'wb') as file:file.write(bytes(ticket['ticket']))
            bearers.append(path);return path
        def native(name,ticket,seconds=12,autoplay=True):
            folder=output/name;folder.mkdir();log=(folder/'client.log').open('w');handles.append(log)
            path=ticket_file(name,ticket)
            client_env=dict(os.environ,DISPLAY=':0',XDG_RUNTIME_DIR='/run/user/1000',THEIA_PERFORMANCE_UNCAPPED='1',
                            THEIA_GRAYBOX_QUALIFY=str(folder),THEIA_GRAYBOX_QUALIFY_SECONDS=str(seconds),RUST_LOG='info')
            if autoplay:client_env['THEIA_GRAYBOX_AUTOPLAY']='1'
            else:client_env.pop('THEIA_GRAYBOX_AUTOPLAY',None)
            for key in ['VK_INSTANCE_LAYERS','VK_LOADER_DEBUG','WGPU_VALIDATION','WGPU_DEBUG']:client_env.pop(key,None)
            if validation:
                client_env.update(VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',VK_LOADER_DEBUG='layer',WGPU_VALIDATION='1')
            if inventory and not zone and name.endswith(('-a','-b')):
                client_env['THEIA_EQUIPMENT_QUALIFY' if equipment else 'THEIA_INVENTORY_QUALIFY']='B' if name.endswith('-b') else 'A'
                client_env.pop('THEIA_INVENTORY_QUALIFY' if equipment else 'THEIA_EQUIPMENT_QUALIFY',None)
            else:
                client_env.pop('THEIA_INVENTORY_QUALIFY',None);client_env.pop('THEIA_EQUIPMENT_QUALIFY',None)
            if rewards and not zone and name.endswith(('-a','-b')):client_env['THEIA_REWARDS_QUALIFY']='1'
            else:client_env.pop('THEIA_REWARDS_QUALIFY',None)
            if progression and not zone and name.endswith(('-a','-b')):client_env['THEIA_PROGRESSION_QUALIFY']='1'
            else:client_env.pop('THEIA_PROGRESSION_QUALIFY',None)
            progression_args=['--progression-catalog',str(progression_profile.resolve())] if progression else []
            if quests and not zone and name.endswith(('-a','-b')):
                client_env['THEIA_QUESTS_QUALIFY']='1'
                if name=='first-b':client_env['THEIA_QUEST_FINISH_GATE']=str(objective_gate)
                else:client_env.pop('THEIA_QUEST_FINISH_GATE',None)
            else:
                client_env.pop('THEIA_QUESTS_QUALIFY',None);client_env.pop('THEIA_QUEST_FINISH_GATE',None)
            if dialogue and not vendors and not zone and name.endswith(('-a','-b')):client_env['THEIA_DIALOGUE_QUALIFY']='1'
            else:client_env.pop('THEIA_DIALOGUE_QUALIFY',None)
            if vendors and not zone and name.endswith(('-a','-b')):client_env['THEIA_VENDOR_QUALIFY']='1'
            else:client_env.pop('THEIA_VENDOR_QUALIFY',None)
            if zone and name.endswith(('-a','-b')):
                client_env['THEIA_ZONE_QUALIFY']='1'
                if name=='first-b':client_env['THEIA_ZONE_COMBAT_GATE']=str(objective_gate)
                else:client_env.pop('THEIA_ZONE_COMBAT_GATE',None)
            else:
                client_env.pop('THEIA_ZONE_QUALIFY',None);client_env.pop('THEIA_ZONE_COMBAT_GATE',None)
            vendor_args=['--vendor-catalog',str(Path('content/gameplay/development-vendor.json').resolve())] if vendors else []
            dialogue_args=['--dialogue-catalog',str(dialogue_profile),'--guide-position',guide_position] if dialogue else []
            quest_args=['--quest-catalog',str(quest_profile)] if quests else []
            reward_args=['--reward-catalog',str(Path('content/gameplay/development-rewards.json').resolve()),'--creature-profile',str(Path('content/gameplay/development-creature-patrol.json').resolve()),'--navigation',str(Path('build/navigation-terrain-far-v1').resolve()),'--combat-catalog',str(combat_profile)] if rewards else []
            child=subprocess.Popen([str(client),'--window-size',window_size,*(['--fullscreen'] if fullscreen else []),*vendor_args,*dialogue_args,*quest_args,*progression_args,*reward_args,*(['--item-catalog',str(item_catalog)] if inventory or vendors else []),'--saved-terrain',str(terrain),'--prediction','--player-assets',str(players),
                                    '--renet-connect',ticket['address'],str(path)],env=client_env,stdout=log,stderr=subprocess.STDOUT)
            children.append(child);return child,folder
        def finish(item,connected):
            child,folder=item
            assert child.wait(timeout=60)==0,'Native qualification failed'
            report=json.loads((folder/'world.json').read_text())
            assert report['connected']==connected, 'Unexpected native connection state'
            assert bool(report['network_error'])!=connected
            text=(folder/'client.log').read_text()
            evidence['known_surface_resize_diagnostics'][folder.name]=check_gpu_diagnostics(text,allow_surface_resize=fullscreen)
            if validation:assert 'VK_LAYER_KHRONOS_validation' in text
            return report
        def zone_report(item,complete=True):
            folder=item[1]
            try:report=json.loads((folder/'zone.json').read_text())
            except (FileNotFoundError,json.JSONDecodeError):return None
            if not complete:return report
            images=['zone-dialogue.png','zone-complete.png','zone-closed.png']
            if not report['resumed']:images+=['zone-objective.png','zone-reward.png','zone-trade.png']
            return report if report['passed'] and all((folder/name).is_file() for name in images) else None
        def check_dialogue(item):
            folder=item[1]
            if zone:return wait(lambda:zone_report(item),40,'Combined native zone flow did not complete')
            if vendors:
                def ready_vendor():
                    try:report=json.loads((folder/'vendor.json').read_text())
                    except (FileNotFoundError,json.JSONDecodeError):return None
                    return report if report['passed'] and all((folder/name).is_file() for name in ['vendor-offer.png','vendor-committed.png','vendor-closed.png']) else None
                report=wait(ready_vendor,20,'Native committed vendor widgets incomplete')
                character=selene if folder.name.endswith('-b') else arden
                row=inventory_rows()[character]
                assert report['snapshot']==row['bag'] and report['revision']==row['revision']
                assert report['revision']==(2 if folder.name.endswith('-b') else 1)
                assert report['window_size']==[int(n) for n in window_size.split('x')]
                assert report['pointer_inputs']==(report['revision'] if report['initial_revision']==0 else 0)
                return report
            def ready():
                try:report=json.loads((folder/'dialogue.json').read_text())
                except (FileNotFoundError,json.JSONDecodeError):return None
                images=['dialogue-welcome.png','dialogue-branch.png','dialogue-terminal.png','dialogue-closed.png','dialogue-walkaway.png']
                return report if report['passed'] and all((folder/name).is_file() for name in images) else None
            report=wait(ready,20,'Native informational dialogue did not complete actual input/widgets')
            assert report['initial_closed_revision_zero']
            assert report['keyboard_events']>=4 and report['pointer_inputs']>=1
            assert report['ui']['pointer_actions']>=1 and report['ui']['submitted']>=5
            assert report['invalid_layouts']==0 and report['max_guide_physics_bodies']==0
            assert report['window_size']==[int(n) for n in window_size.split('x')]
            assert report['phase']==7 and not report['open']
            return report
        def inventory_ready(item):
            if zone:return zone_report(item)
            folder=item[1];path=folder/'inventory.json'
            try:report=json.loads(path.read_text())
            except (FileNotFoundError,json.JSONDecodeError):return None
            return report if report['passed'] and (folder/'inventory.png').is_file() else None
        def inventory_rows():
            receipt=",encode(last_command,'hex') AS last_command" if zone else ""
            result=cluster.query("SELECT coalesce(json_agg(r ORDER BY character),'[]'::json)::text FROM (SELECT character_id::text AS character,encode(catalog_hash,'hex') AS catalog_hash,revision,convert_from(state,'UTF8')::json AS bag"+receipt+" FROM theia.backpacks WHERE realm_id='74686569-6100-0000-0000-000000000007'::uuid) r;")
            assert result.returncode==0,'Owned inventory row verification failed'
            return {r['character']:r for r in json.loads(result.stdout)}
        def progression_rows():
            result=cluster.query("SELECT coalesce(json_agg(r ORDER BY character),'[]'::json)::text FROM (SELECT character_id::text AS character,revision,experience FROM theia.progression WHERE realm_id='74686569-6100-0000-0000-000000000007'::uuid) r;")
            assert result.returncode==0,'Owned XP row verification failed'
            return {r['character']:r for r in json.loads(result.stdout)}
        def check_progression(item,character):
            if zone:
                report=wait(lambda:zone_report(item),15,'Combined native XP display incomplete')
                row=progression_rows()[character]
                assert report['xp']['total']==row['experience']==50 and report['xp']['revision']==row['revision']
                assert report['xp']['level']==3 and report['text_samples']>10 and report['invalid_text']==0
                return report['xp']
            folder=item[1]
            def ready():
                path=folder/'progression.json'
                if not path.is_file():return None
                try:report=json.loads(path.read_text())
                except json.JSONDecodeError:return None
                return report if report['passed'] and (folder/'progression.png').is_file() else None
            report=wait(ready,10,'Native committed XP display incomplete')
            row=progression_rows()[character]
            assert report['revision']==row['revision'] and report['total']==row['experience']
            assert report['total']>=25 and report['level']>=2
            assert report['window_size']==[int(n) for n in window_size.split('x')], 'Actual native window size differs from requested qualification size'
            return report
        def quest_rows():
            result=cluster.query("SELECT coalesce(json_agg(r ORDER BY character),'[]'::json)::text FROM (SELECT character_id::text AS character,revision,kill_count,encode(profile_hash,'hex') AS profile_hash FROM theia.quest_objectives WHERE realm_id='74686569-6100-0000-0000-000000000007'::uuid) r;")
            assert result.returncode==0,'Owned objective row verification failed'
            return {r['character']:r for r in json.loads(result.stdout)}
        def quest_report(item,count=3):
            if zone:
                report=zone_report(item)
                return report['quest'] if report and report['quest']['count']==count else None
            folder=item[1];path=folder/'quests.json'
            if not path.is_file():return None
            try:report=json.loads(path.read_text())
            except json.JSONDecodeError:return None
            return report if report['valid'] and report['count']==count and (folder/f'quest-count-{count}.png').is_file() else None
        def check_quest(item,character,resumed,count=3):
            report=wait(lambda:quest_report(item,count),10,'Native objective display incomplete')
            row=quest_rows()[character]
            assert report['count']==row['kill_count']==count and report['revision']==row['revision']==count
            assert bytes(report['profile_hash']).hex()==row['profile_hash']
            assert report['window_size']==[int(n) for n in window_size.split('x')]
            if count==3:
                assert report['passed'] and report['complete']
                if not resumed:
                    assert [v['count'] for v in report['seen']]==[0,1,2,3]
                    assert report['seen'][-2]['xp_total']==report['seen'][-1]['xp_total']==50
                    assert report['casts']>=2
            return report
        def reward_rows():
            result=cluster.query("SELECT coalesce(json_agg(r ORDER BY character),'[]'::json)::text FROM (SELECT character_id::text AS character,claim_id::text AS claim,reward_id,consumed_revision FROM theia.reward_claims WHERE realm_id='74686569-6100-0000-0000-000000000007'::uuid) r;")
            assert result.returncode==0,'Owned reward row verification failed'
            return {r['character']:r for r in json.loads(result.stdout)}
        def check_inventory(item,character,resumed):
            if zone:
                report=wait(lambda:zone_report(item),15,'Combined native bag display incomplete')
                variant_b=item[1].name.endswith('-b');desired=4 if variant_b else 3
                assert report['resumed']==resumed and report['snapshot']==inventory_rows()[character]['bag']
                receipt=inventory_rows()[character]['last_command']
                assert isinstance(receipt,str) and len(bytes.fromhex(receipt))==32
                assert report['revision']==desired and report['invalid_layouts']==report['invalid_text']==0
                assert report['layout_samples']>10 and report['text_samples']>10 and not report['ui']['pending']
                if not resumed:
                    assert report['pointer_inputs']==(5 if variant_b else 4)
                    assert report['ui']['applied']==report['ui']['submitted']==desired and report['ui']['rejected']==0
                    assert report['ui']['reward_clicks']==1 and report['ui']['equipment_clicks']==1
                    assert report['ui']['vendor_clicks']==(2 if variant_b else 1)
                else:assert report['pointer_inputs']==report['casts']==report['ui']['submitted']==0
                return report
            report=inventory_ready(item);assert report,'Native inventory qualification incomplete'
            assert report['resumed']==resumed
            assert report['snapshot']==inventory_rows()[character]['bag'],'Client displayed another character or incorrect backpack'
            assert report['layout_samples']>10 and report['invalid_layouts']==0
            assert report['ui']['pending'] is False
            if not resumed:
                assert report['pointer_inputs']==(11 if rewards else 10 if equipment else 16)
                assert report['ui']['applied']==(5 if rewards else 4 if equipment else 6) and report['ui']['rejected']==1 and report['ui']['submitted']==(6 if rewards else 5 if equipment else 7)
                if equipment:
                    assert len(report['equipment_phases'])==4
                    assert report['ui']['equipment_clicks']==5
                    for name in ['equipment-first','equipment-replaced','equipment-empty','equipment-final']:
                        assert (item[1]/(name+'.png')).is_file()
            if rewards:
                assert report['rewards'] and report['ui']['reward_pending'] is False
                if not resumed:
                    assert report['earned']['count']==1 and report['earned']['bag_revision']==4
                    assert report['reward_casts']>0 and report['ui']['reward_clicks']==1
            if equipment:
                assert report['equipment'] is not None
                metrics=report['equipment_metrics'];assert metrics['last_error'] is None
                assert metrics['max_socket_error']<.002 and metrics['max_scale_error']<.002 and metrics['max_grip_offset']<.04
            return report
        def first_pose(folder,expected):
            reports=[json.loads(line)['graybox'] for line in (folder/'client.log').read_text().splitlines() if line.startswith('{"graybox":')]
            assert reports and reports[0]['seconds']<3,'First native pose was not observed before autoplay'
            error=max(abs(a-b) for a,b in zip(reports[0]['position'],expected))
            assert error<.05,'Native client restored another character or an incorrect saved pose'
            return error
        def rows():
            result=cluster.query("SELECT coalesce(json_agg(r ORDER BY character),'[]'::json)::text FROM (SELECT character_id::text AS character,owner_id::text AS owner,ARRAY[x,y,z] AS position,checkpoint_sequence AS sequence FROM theia.characters WHERE realm_id='74686569-6100-0000-0000-000000000007'::uuid) r;")
            assert result.returncode==0,'Owned row verification failed'
            return {r['character']:r for r in json.loads(result.stdout)}
        def stop_server():
            server.send_signal(signal.SIGINT)
            assert server.wait(timeout=15)==0,'Account authority shutdown failed'
            text=server_log.read_text()
            summary=next(json.loads(line.removeprefix('account_persistence_summary ')) for line in text.splitlines() if line.startswith('account_persistence_summary '))
            assert not summary['failed'] and summary['worker']['stale_records']==0
            transport=next(json.loads(line.removeprefix('transport_summary ')) for line in text.splitlines() if line.startswith('transport_summary '))
            assert transport['max_players']==2 and transport['queued_commands']>0
            assert transport['simulation']['missed_deadlines']==0
            assert transport['max_control_batch']<=8 and transport['max_input_batch']<=32
            assert transport['max_event_queue_bytes']<=8192 and transport['max_movement_queue_bytes']<=8192
            evidence['transport_before']=transport
            if dialogue:
                assert transport['dialogue_requests_accepted']>=(4 if vendors else 20) and transport['dialogue_requests_rejected']==0
                assert transport['dialogue_replies_sent']>=(4 if vendors else 20) and transport['guide_presences_sent']>=4
                assert transport['dialogue_backpressure_disconnects']==0
            return summary
        try:
            start('before','15040,15040' if rewards else '15400,12860')
            a=login(0);b=login(1)
            arden=accounts[a].choose_character(read=lambda _: 'Arden');selene=accounts[b].choose_character(read=lambda _: 'Selene')
            enter(b,arden,404)
            http('/characters/enter',a,{'character_id':arden,'owner_id':'forbidden'},400)
            ta=enter(a,arden);tb=enter(b,selene)
            enter(a,arden,409);enter(b,selene,409)
            # A failed browser launch cancels its proof-bound challenge without
            # replacing the already collected account credential.
            cancelled=AccountClient(base)
            try:cancelled.login(open_browser=lambda _:False)
            except AccountError:pass
            else:raise AssertionError('Failed browser launch unexpectedly logged in')
            assert http('/development/auth-metrics')['pending_logins']==0
            first_a=native('first-a',ta,45 if zone else 32 if dialogue else 45 if rewards else 24 if inventory else 14);first_b=native('first-b',tb,45 if zone else 32 if dialogue else 45 if rewards else 24 if inventory else 14)
            wait(lambda:len(admissions())==2,label='Two authenticated native players did not join')
            initial=admissions();assert all(r['position'][0]>15000 for r in initial)
            time.sleep(4)
            if zone:
                complete_a=wait(lambda:zone_report(first_a),30,'A never completed the combined training flow')
                waiting_b=wait(lambda:zone_report(first_b,False),5,'B has no initial private observation')
                assert waiting_b['quest']['count']==waiting_b['xp']['total']==waiting_b['revision']==0
                assert quest_rows()[selene]['kill_count']==progression_rows()[selene]['experience']==inventory_rows()[selene]['revision']==0
                assert inventory_rows()[selene]['last_command'] is None
                assert selene not in reward_rows()
                distinct_quest={'a':complete_a['quest'],'b':waiting_b['quest']}
                objective_gate.write_text('allow independent B zone inputs\n')
            if dialogue:first_dialogue={'a':check_dialogue(first_a),'b':check_dialogue(first_b)}
            if inventory:
                wait(lambda:inventory_ready(first_a) and inventory_ready(first_b),40 if rewards else 15,'Native inventory widgets never completed')
                first_inventory={'a':check_inventory(first_a,arden,False),'b':check_inventory(first_b,selene,False)}
                assert first_inventory['a']['snapshot']!=first_inventory['b']['snapshot'],'Distinct characters did not get distinct item arrangements'
            if quests and not zone:
                wait(lambda:quest_report(first_a) and quest_report(first_b,1),35,'Independent objective counts never reached complete/active states')
                distinct_quest={'a':check_quest(first_a,arden,False),'b':check_quest(first_b,selene,False,1)}
                assert distinct_quest['a']['count']!=distinct_quest['b']['count']
                objective_gate.write_text('continue owned objective qualification\n')
                wait(lambda:quest_report(first_b),20,'Second character did not complete its own objective')
                first_quests={'a':check_quest(first_a,arden,False),'b':check_quest(first_b,selene,False)}
            if zone:first_quests={'a':check_quest(first_a,arden,False),'b':check_quest(first_b,selene,False)}
            if progression:first_progression={'a':check_progression(first_a,arden),'b':check_progression(first_b,selene)}
            enter(a,arden,409)
            http('/auth/logout' ,a,{},204)
            finish(first_a,False);report_b=finish(first_b,True)
            if equipment and not zone:
                equipment_motion={}
                for label,item in [('a',first_a),('b',first_b)]:
                    metrics=json.loads((item[1]/'inventory.json').read_text())['equipment_metrics']
                    assert metrics['local_visible_samples']>100 and metrics['remote_visible_samples']>100 and metrics['visible_moving_samples']>100,metrics
                    assert metrics['last_error'] is None
                    equipment_motion[label]=metrics
            # The reward fixture lasts longer than the provider's deliberately
            # short sixty-second identity lifetime. Renew before the next lease;
            # never lengthen or bypass production account/session validation.
            if rewards or dialogue:b=login(1)
            # All native processes have exited. Reservation is still unavailable
            # until authority removal and its exact final checkpoint commit.
            def reenter():
                try:return enter(b,selene)
                except AssertionError as error:
                    if 'status 409' in str(error):return None
                    raise
            fresh_b=wait(reenter,10,'Final checkpoint never released the character')
            saved=rows();assert all(r['sequence']>0 for r in saved.values())
            assert saved[selene]['position']!=initial[1]['position']
            enter(a,arden,401)
            a2=login(0);fresh_a=enter(a2,arden)
            second_a=native('second-a',fresh_a,22 if dialogue else 16 if inventory else 12);second_b=native('second-b',fresh_b,22 if dialogue else 16 if inventory else 12)
            wait(lambda:len(admissions())==4,label='Fresh admissions failed')
            replacement=admissions()[2:]
            assert {tuple(r['position']) for r in replacement}=={tuple(r['position']) for r in saved.values()},'Rejoin did not restore exact committed poses'
            assert not {r['actor'] for r in initial}&{r['actor'] for r in replacement}
            assert all(r['input_stream']==1 for r in replacement)
            replay=native('replayed-ticket',ta,8,False)
            finish(replay,False)
            assert len(admissions())==4,'A spent ticket created another actor'
            finish(second_a,True);finish(second_b,True)
            rejoin_errors={'a':first_pose(second_a[1],saved[arden]['position']),'b':first_pose(second_b[1],saved[selene]['position'])}
            if dialogue:rejoined_dialogue={'a':check_dialogue(second_a),'b':check_dialogue(second_b)}
            if inventory:
                rejoined_inventory={'a':check_inventory(second_a,arden,True),'b':check_inventory(second_b,selene,True)}
                items_before_restart=inventory_rows()
            if progression:
                rejoined_progression={'a':check_progression(second_a,arden),'b':check_progression(second_b,selene)}
                progression_before=progression_rows()
            if quests:
                rejoined_quests={'a':check_quest(second_a,arden,True),'b':check_quest(second_b,selene,True)}
                quests_before=quest_rows()
            reward_before=reward_rows() if rewards else None
            if rewards:
                assert set(reward_before)=={arden,selene}
                assert all(r['consumed_revision']==(1 if zone else 5) for r in reward_before.values())
                assert len({r['claim'] for r in reward_before.values()})==2
            vendor_before_restart=inventory_rows() if vendors else None
            summary_before=stop_server();saved_before=rows()
            cluster.restart();assert rows()==saved_before,'Database restart changed committed characters'
            if vendors:assert inventory_rows()==vendor_before_restart,'Database restart changed committed purchases'
            if inventory:assert inventory_rows()==items_before_restart,'Database restart changed committed backpacks'
            if progression:assert progression_rows()==progression_before,'Database restart changed XP'
            if quests:assert quest_rows()==quests_before,'Database restart changed objective progress'
            start('after','8000,8000')
            http('/characters',a2,expected=401)
            anew=login(0);bnew=login(1)
            assert [r['id'] for r in http('/characters',anew)['characters']]==[arden]
            assert [r['id'] for r in http('/characters',bnew)['characters']]==[selene]
            after_b=native('after-b',enter(bnew,selene),30 if dialogue else 20 if inventory else 14);after_a=native('after-a',enter(anew,arden),30 if dialogue else 20 if inventory else 14)
            wait(lambda:len(admissions())==2,label='Distant restore after restart failed')
            restored=admissions()
            assert {tuple(r['position']) for r in restored}=={tuple(r['position']) for r in saved_before.values()}
            assert restored[0]['epoch']!=initial[0]['epoch']
            # Kill only the unique database connection owned by this fixture.
            time.sleep(3)
            if dialogue:restarted_dialogue={'a':check_dialogue(after_a),'b':check_dialogue(after_b)}
            if inventory:
                wait(lambda:inventory_ready(after_a) and inventory_ready(after_b),10,'Restarted native backpack load failed')
                restarted_inventory={'a':check_inventory(after_a,arden,True),'b':check_inventory(after_b,selene,True)}
            if progression:restarted_progression={'a':check_progression(after_a,arden),'b':check_progression(after_b,selene)}
            if quests:restarted_quests={'a':check_quest(after_a,arden,True),'b':check_quest(after_b,selene,True)}
            result=cluster.query("SELECT json_agg(pid)::text FROM pg_stat_activity WHERE application_name='theia-development-persistence';")
            assert result.returncode==0;pids=json.loads(result.stdout);assert len(pids)==1
            result=cluster.query(f'SELECT pg_terminate_backend({int(pids[0])});')
            assert result.returncode==0 and result.stdout.strip()=='t'
            assert server.wait(timeout=12)!=0,'Authority falsely succeeded after losing persistence'
            finish(after_a,False);finish(after_b,False)
            restart_errors={'a':first_pose(after_a[1],saved_before[arden]['position']),'b':first_pose(after_b[1],saved_before[selene]['position'])}
            evidence.update(native_own_character_pose_errors_metres={'rejoin':rejoin_errors,'restart':restart_errors},rows_before_rejoin=saved,rows_before_restart=saved_before,
                            passed=True,launcher_account_helper_live=True,launcher_private_ticket_permissions=True,failed_browser_cancels_challenge=True,physical_browser_tested=False,actual_accounts=2,actual_native_players=2,foreign_selection_denied=True,
                            concurrent_admission_denied=True,revocation_disconnected=True,spent_ticket_denied=True,
                            exact_committed_rejoin=True,fresh_runtime_actor_and_input=True,
                            actual_database_and_authority_restart=True,changed_default_spawn=True,exact_distant_restore=True,
                            database_loss_failed_closed=True,summary_before=summary_before,
                            native_before=report_b,initial=initial,replacement=replacement,restored=restored)
            if inventory:
                assert inventory_rows()==items_before_restart
                evidence["inventory"]={"passed":True,"first":first_inventory,"rejoined":rejoined_inventory,"restarted":restarted_inventory,"rows_before_restart":items_before_restart,"rows_after_backend_loss":inventory_rows(),"per_character_contents_match":True,"distinct_arrangements":True,"keyboard_toggle_and_actual_native_widget_path":True,"physical_os_mouse_tested":False,"validation":validation}
            if quests:
                assert quest_rows()==quests_before
                evidence['quests']={'passed':True,'distinct_active_complete':distinct_quest,'first':first_quests,'rejoined':rejoined_quests,'restarted':restarted_quests,'rows_before_restart':quests_before,'rows_after_backend_loss':quest_rows(),'default_goal':3,'native_fixture_ability':{'cast_ms':0,'damage':100},'quest_advances_after_xp_cap':True,'each_own_character_counter':True}
            if progression:
                assert progression_rows()==progression_before
                evidence['progression']={'passed':True,'first':first_progression,'rejoined':rejoined_progression,'restarted':restarted_progression,'rows_before_restart':progression_before,'rows_after_backend_loss':progression_rows(),'qualification_thresholds':[0,25,50],'default_thresholds':[0,100,250],'grant_per_kill':25,'actual_kill_to_committed_native_level':True}
            if rewards:
                assert reward_rows()==reward_before
                evidence['rewards']={'passed':True,'rows_before_restart':reward_before,'rows_after_backend_loss':reward_rows(),'each_own_qualifying_kill_and_widget_claim':True,'durable_once_through_rejoin_and_restart':True}
            if dialogue and not zone:
                for table in (['reward_claims','progression','quest_objectives'] if vendors else ['backpacks','reward_claims','progression','quest_objectives']):
                    count=cluster.query(f'SELECT count(*) FROM theia.{table};')
                    assert count.returncode==0 and count.stdout.strip()=='0','Informational dialogue changed persistent gameplay value'
                if vendors:assert inventory_rows()==vendor_before_restart,'Reconnect/backend loss changed committed purchases'
                evidence['vendors' if vendors else 'dialogue']={'passed':True,'first':first_dialogue,'rejoined':rejoined_dialogue,'restarted':restarted_dialogue,'guide_position':guide_position,'actual_native_keyboard_and_picking':True,'physical_os_input_tested':False,'ephemeral_sessions_start_closed':True,'walkaway_closes':True}
            if zone:
                evidence['zone']={'passed':True,'first':first_inventory,'rejoined':rejoined_inventory,'restarted':restarted_inventory,'private_progress_before_b_inputs':distinct_quest,'all_item_actions_share_one_stream':True,'native_keyboard_and_picking':True,'physical_os_input_tested':False,'accelerated_fixture_ability_and_xp_curve':True}
            if equipment and not zone:
                evidence['equipment']={'passed':True,'own_and_remote_visible_motion':equipment_motion,'public_cosmetics_exclude_private_bags':True,'durable_rejoin_and_restart':True,'actual_widget_equip_replace_reject_unequip':True}
        finally:
            for child in reversed(children):
                if child.poll() is None:
                    child.terminate()
                    try:child.wait(timeout=5)
                    except subprocess.TimeoutExpired:child.kill();child.wait()
            if server is not None and server.poll() is None:
                server.send_signal(signal.SIGINT)
                try:server.wait(timeout=15)
                except subprocess.TimeoutExpired:server.kill();server.wait()
            for handle in handles:handle.close()
            for path in bearers:path.unlink(missing_ok=True)
        for path in output.rglob('*.log'):
            text=path.read_text(errors='replace')
            assert not any(value and value in text for value in private),'Private identity/session data leaked to diagnostics'
    evidence['owned_provider_and_database_stopped']=identity.server is None and cluster.server is None
    evidence['private_ticket_files_removed']=all(not path.exists() for path in bearers)
    (output/'verification.json').write_text(json.dumps(evidence,indent=2)+'\n')
    return evidence


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    parser.add_argument('--identity-sdk',type=Path,default=Path('build/identity-sdk'))
    parser.add_argument('--postgres-sdk',type=Path,default=Path('build/postgres-sdk'))
    parser.add_argument('--fullscreen',action='store_true')
    parser.add_argument('--window-size',default='1280x720')
    parser.add_argument('--client',type=Path,default=Path('build/persistence-qualified-v1-20261029/theia-client'))
    parser.add_argument('--terrain',type=Path,default=Path('build/graybox-terrain-v1'))
    parser.add_argument('--players',type=Path,default=Path('build/synty-character-canonical-stage'))
    parser.add_argument('--inventory',action='store_true')
    parser.add_argument('--equipment',action='store_true')
    parser.add_argument('--zone',action='store_true',help='Combined persistent training objective/reward/trade/equipment lifecycle')
    parser.add_argument('--vendors',action='store_true',help='Focused two-account native barter input/commit/restart fixture')
    parser.add_argument('--dialogue',action='store_true',help='Focused informational dialogue input/panel/closure fixture')
    parser.add_argument('--quests',action='store_true',help='Qualify objective progression/completion through actual fast-fixture kills')
    parser.add_argument('--progression',action='store_true',help='Also qualify committed XP using an explicit accelerated curve')
    parser.add_argument('--rewards',action='store_true')
    parser.add_argument('--validation',action='store_true')
    parser.add_argument('--server',type=Path,help='Use an exact already built qualification executable')
    args=parser.parse_args()
    result=run(args.output,*[getattr(args,name).resolve() for name in ['identity_sdk','postgres_sdk','client','terrain','players']],server_binary=args.server.resolve() if args.server else None,inventory=args.inventory,validation=args.validation,equipment=args.equipment,rewards=args.rewards,progression=args.progression,window_size=args.window_size,fullscreen=args.fullscreen,quests=args.quests,dialogue=args.dialogue,vendors=args.vendors,zone=args.zone)
    print(json.dumps({key:result[key] for key in ['passed','scope','actual_native_players','exact_distant_restore','database_loss_failed_closed']},indent=2))
