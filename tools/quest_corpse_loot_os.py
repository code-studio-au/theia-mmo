#!/usr/bin/env python3
"""Normal PID-bound pointer/keyboard corpse pickup; telemetry is read-only."""
import json
import math
import time
from qualify_world_input_os import wait, capture_ready
PELT = 'theia.dev.wolf-pelt'
TOKEN = 'theia.dev.training-token'


def flow(desktop, window, state, folder, resumed=False, item_quest=False, quest_only=False):
    scale = state()['window'][2]
    def count(item):
        return sum(s['count'] for s in state()['inventory_bag']['slots'] if s and s['item'] == item)
    def ui():
        return state()['loot']['ui']
    def corpses():
        return state()['loot']['corpses']
    def feet():
        return state()['dodge']['authority_feet']
    def capture(name):
        (folder/'capture.request').write_text(name+'\n')
        wait(lambda: capture_ready(folder/'input-os.png'))
        (folder/'input-os.png').rename(folder/name)
    def face(point):
        yaw = math.atan2(-(point[0]-feet()[0]), -(point[2]-feet()[2]))
        for _ in range(8):
            error = math.atan2(math.sin(yaw-state()['classic_facing']*math.tau/65536), math.cos(yaw-state()['classic_facing']*math.tau/65536))
            if abs(error) < .05:
                return
            width, height, _ = state()['window']
            desktop.move(window, [width*.55,height*.42], scale)
            desktop.button(3, True)
            try:
                error = math.atan2(math.sin(yaw-state()['camera_angles'][0]),math.cos(yaw-state()['camera_angles'][0]))
                desktop.look(round(max(-400,min(400,-error/.004))),0)
            finally:
                desktop.button(3, False)
            time.sleep(.2)
        raise AssertionError('Manual corpse facing did not settle')
    def approach(point):
        for _ in range(45):
            target = point() if callable(point) else point
            distance = math.hypot(target[0]-feet()[0],target[2]-feet()[2])
            if distance < 1.8:
                return
            face(target)
            desktop.key_state('w', True)
            try:
                time.sleep(min(.3,max(.02,(distance-1.6)/5)))
            finally:
                desktop.key_state('w',False)
            time.sleep(.15)
        raise AssertionError('Normal movement did not reach corpse')
    def open_corpse(corpse, excluded_life=None):
        approach(corpse['position'])
        face(corpse['position'])
        # Orbit with the normal independent camera when a guide/player or
        # another corpse occludes the projected anchor. UI and ray hits retain
        # their normal ownership; the qualifier must find an exposed body.
        for orbit in range(3):
            if state()['dialogue_open']:
                desktop.key('Escape')
                wait(lambda: not state()['dialogue_open'])
            if orbit:
                width,height,_=state()['window']
                desktop.move(window,[width*.55,height*.42],scale)
                desktop.button(1,True)
                try:
                    desktop.look(220,0)
                finally:
                    desktop.button(1,False)
                time.sleep(.25)
            # The marker is a projected body anchor, not synthetic pointer injection.
            # Sample a small bounded neighbourhood of the actual frozen mesh.
            for dx,dy in [(0,0),(0,8),(0,-8),(8,0),(-8,0),(16,0),(-16,0),(0,16),(0,-16)]:
                observed = wait(lambda: next((c for c in corpses() if c['life']==corpse['life'] and c['screen']),None))
                x,y=observed['screen'];width,height,_=state()['window']
                if not (0<x+dx<width and 0<y+dy<height):
                    continue
                desktop.click(window,[x+dx,y+dy],scale,3)
                time.sleep(.3)
                if state()['dialogue_open']:
                    break
                if ui()['open']:
                    selected=state()['loot']['selected_life']
                    picked=next((c for c in corpses() if c['life']==selected),None)
                    assert picked is not None, 'Loot opened without a privately available corpse'
                    # Overlapping bodies resolve to the actual frontmost pointer
                    # hit. Track that body; never pretend the projected anchor is
                    # an instruction to select an obscured incarnation.
                    if selected != excluded_life:
                        return picked
                    desktop.key('Escape')
                    wait(lambda: not ui()['open'])
        raise AssertionError('Right-click did not open corpse loot')
    def click(name,index=None):
        point=wait(lambda: state()['loot']['layout'][name])
        if index is not None:
            point=point[index]
        assert point is not None
        width,height,_=state()['window'];assert 0<point[0]<width and 0<point[1]<height
        desktop.click(window,point,scale,1)
    def item_flow():
        quest_id = 'theia.dev.quest-wolf-pelts'
        objective = quest_id+'.objective.pelts'
        def journal():
            return state()['quest_journal']
        def quest():
            return next((q for q in journal()['quests'] if q['id']==quest_id), None)
        def progress():
            return (quest() or {}).get('counts',{}).get(objective,0)
        def close_panels():
            if ui()['open']:
                desktop.key('Escape');wait(lambda:not ui()['open'])
            if state()['dialogue_open']:
                desktop.key('Escape')
                wait(lambda:not state()['dialogue_open'] and not state().get('dialogue_pending',False) and not any(state().get('dialogue_ui',{}).get(k,False) for k in ['open','pending','close_when_open']))
            if state()['inventory_open']:
                desktop.key('i');wait(lambda:not state()['inventory_open'])
        def speak():
            close_panels()
            approach([8004.,0.,8000.])
            desktop.click(window,wait(lambda:state()['guide_screen']),scale,3)
            wait(lambda:state()['dialogue_open'])
        def action(status):
            before=journal()['ui']['applied']
            point=wait(lambda:next((b['point'] for b in journal()['buttons'] if b['quest']==quest_id),None))
            desktop.click(window,point,scale,1)
            wait(lambda:quest() and quest()['status']==status and journal()['ui']['applied']==before+1 and not journal()['ui']['pending'])
        def take_all():
            before=ui()['submitted']
            click('all')
            wait(lambda:ui()['submitted']>before,2)
        def inventory_click(name,index=None):
            point=wait(lambda:state()['inventory_layout'][name])
            if index is not None:point=point[index]
            desktop.click(window,point,scale,1)
        final_tokens = 106 if quest_only else 103
        final_revision = 27 if quest_only else 24
        final_xp = 250 if quest_only else 200
        def kill_wolf():
            close_panels()
            wait(lambda:state().get('enemy_health') is not None and state()['enemy_health']>0,15)
            approach(lambda:state()['enemy_global'])
            clicks=state()['controls']['enemy_clicks']
            for dx,dy in [(20,-25),(-20,-25),(0,-30),(0,0)]:
                point=wait(lambda:state()['enemy_screen'])
                desktop.click(window,[point[0]+dx,point[1]+dy],scale,3)
                try:
                    wait(lambda:state()['controls']['enemy_clicks']>clicks,.75)
                    break
                except TimeoutError:pass
            if state()['controls']['enemy_clicks']==clicks:
                close_panels();desktop.key('Tab')
                wait(lambda:state()['selected_hostile'] is not None)
            deaths=state()['combat']['deaths'];deadline=time.monotonic()+60
            while state()['combat']['deaths']==deaths:
                assert time.monotonic()<deadline,'Conditional quest wolf fight timed out'
                assert state()['combat']['player_deaths']==0
                desktop.key('2');time.sleep(.25)
        if resumed:
            assert quest()['status']=='Completed' and progress()==3
            assert count(PELT)==0 and count(TOKEN)==final_tokens and not corpses()
            assert state()['inventory_ui']['destroy_confirmation'] is None
            capture('resumed-completed-item-quest.png')
            return dict(passed=True,resumed=True,final_revision=journal()['revision'],pelts=0,tokens=final_tokens,completion_and_consumption_restored=True)
        assert quest() is None and count(PELT)==0 and count(TOKEN)==99
        historical={q['id']:q for q in journal()['quests']}
        if quest_only:
            # Old unaccepted lives carry only ordinary token rights. View and
            # collect those before acceptance; SQL also verifies denied receipts.
            wait(lambda:len(corpses())==3)
            for n in range(3):
                open_corpse(min(corpses(),key=lambda c:math.dist(c['position'],feet())))
                wait(lambda:len(state()['loot']['layout']['take'])==1)
                assert count(PELT)==0
                take_all();wait(lambda:not ui()['open'] and count(TOKEN)==100+n)
            wait(lambda:not corpses())
            capture('quest-only-old-corpses-no-pelts.png')
        speak();action('Active')
        assert progress()==0 and count(PELT)==0
        capture('item-quest-accepted.png')
        if state()['dialogue_open']:
            desktop.key('Escape');wait(lambda:not state()['dialogue_open'])
        if not state()['inventory_open']:
            desktop.key('i');wait(lambda:state()['inventory_open'])
        if not quest_only:wait(lambda:len(corpses())==3)
        for n in range(1,4):
            if quest_only:
                kill_wolf()
                wait(lambda:len(corpses())==1)
                assert count(PELT)==n-1 and progress()==n-1
                if not state()['inventory_open']:
                    desktop.key('i');wait(lambda:state()['inventory_open'])
            corpse=min(corpses(),key=lambda c:math.dist(c['position'],feet()))
            open_corpse(corpse)
            if n==1:
                time.sleep(.3)
                assert count(PELT)==0 and progress()==0 and quest()['status']=='Active'
                capture('item-quest-view-without-credit.png')
            take_all()
            wait(lambda:not ui()['open'] and count(PELT)==n and progress()==n and quest()['status']==('Ready' if n==3 else 'Active'))
        capture('item-quest-three-pelts-ready.png')
        before=state()['inventory_bag']['revision']
        journal_before_split=journal()['revision']
        source=next(i for i,s in enumerate(state()['inventory_bag']['slots']) if s and s['item']==PELT)
        assert state()['inventory_bag']['slots'][15] is None
        inventory_click('slots',source)
        wait(lambda:state()['inventory_ui']['selected']==source)
        inventory_click('split');wait(lambda:state()['inventory_ui']['split'])
        inventory_click('slots',15)
        wait(lambda:state()['inventory_bag']['revision']==before+1 and not state()['inventory_ui']['pending'])
        assert count(PELT)==3 and progress()==3 and quest()['status']=='Ready' and journal()['revision']==journal_before_split
        capture('item-quest-split-still-ready.png')
        inventory_click('slots',15);wait(lambda:state()['inventory_ui']['selected']==15)
        inventory_click('destroy')
        wait(lambda:state()['inventory_ui']['destroy_confirmation']==[15,before+1])
        assert count(PELT)==3 and quest()['status']=='Ready'
        inventory_click('destroy')
        wait(lambda:count(PELT)==2 and progress()==2 and quest()['status']=='Active' and not state()['inventory_ui']['pending'])
        capture('item-quest-destroy-reverted.png')
        if quest_only:
            kill_wolf()
        else:
            close_panels()
            wait(lambda:state().get('enemy_health') is not None and state()['enemy_health']>0,15)
            approach(lambda:state()['enemy_global'])
            clicks=state()['controls']['enemy_clicks']
            for dx,dy in [(20,-25),(-20,-25),(0,-30),(0,0)]:
                point=wait(lambda:state()['enemy_screen'])
                desktop.click(window,[point[0]+dx,point[1]+dy],scale,3)
                try:
                    wait(lambda:state()['controls']['enemy_clicks']>clicks,.75)
                    break
                except TimeoutError:
                    pass
            wait(lambda:state()['controls']['enemy_clicks']>clicks,1)
            deaths=state()['combat']['deaths'];deadline=time.monotonic()+60
            while state()['combat']['deaths']==deaths:
                assert time.monotonic()<deadline,'Fourth normal wolf fight timed out'
                assert state()['combat']['player_deaths']==0
                desktop.key('2');time.sleep(.25)
        wait(lambda:len(corpses())==1)
        wait(lambda:journal()['xp']==(250 if quest_only else 175))
        assert count(PELT)==2 and progress()==2 and quest()['status']=='Active'
        capture('item-quest-new-kill-without-credit.png')
        desktop.key('i');wait(lambda:state()['inventory_open'])
        open_corpse(corpses()[0])
        assert count(PELT)==2 and progress()==2
        take_all()
        wait(lambda:not ui()['open'] and count(PELT)==3 and progress()==3 and quest()['status']=='Ready')
        capture('item-quest-replacement-ready.png')
        speak();action('Completed')
        wait(lambda:count(PELT)==0 and count(TOKEN)==final_tokens)
        assert progress()==3 and journal()['xp']==final_xp and journal()['revision']==final_revision
        assert {q['id']:q for q in journal()['quests'] if q['id']!=quest_id}==historical
        assert journal()['ui']['submitted']==8 and journal()['ui']['applied']==8 and journal()['ui']['rejected']==0
        assert ui()['rejected']==0 and state()['inventory_ui']['rejected']==0 and not corpses()
        assert state()['combat']['event_gaps']==0 and state()['combat']['player_deaths']==0
        capture('item-quest-completed-consumed.png')
        result=dict(passed=True,resumed=False,final_revision=final_revision,pelts=0,tokens=final_tokens,quest_only=quest_only,view_and_kill_without_credit=True,
                    split_preserved_ready=True,destroy_reverted_to_two=True,replacement_pickup_restored_ready=True,
                    turn_in_consumed_three=True,historical_quests_unchanged=True)
        (folder/'item-quest-proof.json').write_text(json.dumps(result,indent=2)+'\n')
        return result
    if state()['dialogue_open']:
        desktop.key('Escape')
        wait(lambda: not state()['dialogue_open'] and not state().get('dialogue_pending',False) and not any(state().get('dialogue_ui',{}).get(k,False) for k in ['open','pending','close_when_open']))
    if not state()['inventory_open']:
        desktop.key('i')
        wait(lambda: state()['inventory_open'])
    journal = state()['quest_journal']['quests']
    journal_revision = state()['quest_journal']['revision']
    if item_quest:
        return item_flow()
    if resumed:
        assert count(PELT)==2 and count(TOKEN)==100
        wait(lambda: len(corpses())==2)
        capture('resumed-uncollected-corpses.png')
        for _ in range(2):
            corpse=min(corpses(),key=lambda c:math.dist(c['position'],feet()))
            open_corpse(corpse)
            click('all')
            wait(lambda: not ui()['open'])
        wait(lambda: count(PELT)==3 and count(TOKEN)==102 and not corpses())
        capture('resumed-all-loot-collected.png')
    else:
        assert count(PELT)==0 and count(TOKEN)==99
        wait(lambda: len(corpses())==3)
        corpse=min(corpses(),key=lambda c:math.dist(c['position'],feet()))
        revision=state()['inventory_bag']['revision']
        corpse=open_corpse(corpse)
        wait(lambda: len(state()['loot']['layout']['take'])==2)
        time.sleep(.4)
        assert count(PELT)==0 and state()['inventory_bag']['revision']==revision
        capture('corpse-loot-view-no-grant.png')
        desktop.key('Escape')
        wait(lambda: not ui()['open'])
        assert state()['inventory_open'] and count(PELT)==0 and len(corpses())==3
        open_corpse(corpse)
        click('take',0)
        wait(lambda: count(PELT)==1 and ui()['applied']==1 and len(state()['loot']['layout']['take'])==1)
        assert count(TOKEN)==99
        capture('one-pelt-picked-up.png')
        click('close')
        wait(lambda: not ui()['open'])
        # Leave that corpse's remaining item uncollected across logout.
        other=next(c for c in corpses() if c['life']!=corpse['life'])
        open_corpse(other,corpse['life'])
        click('all')
        wait(lambda: count(PELT)==2 and count(TOKEN)==100 and not ui()['open'] and ui()['applied']==3)
        assert len(corpses())==2
        capture('loot-all-committed.png')
    assert state()['quest_journal']['quests']==journal
    assert state()['quest_journal']['revision']==journal_revision
    assert ui()['rejected']==0
    return dict(passed=True,resumed=resumed,pelts=count(PELT),tokens=count(TOKEN),loot_ui=ui(),remaining_corpses=len(corpses()),quest_journal_unchanged=True)
