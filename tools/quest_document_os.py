"""Normal note pickup/reader flow, using only PID-bound native input.
Movement/corpse pointer helpers are adapted from quest_corpse_loot_os.py.
Telemetry observes layout/state; it never supplies game commands or mutations.
"""
import json
import math
import time
from qualify_world_input_os import wait, capture_ready
NOTE = 'theia.dev.roadside-orders'
QUEST = 'theia.dev.quest-roadside-orders'
READ = QUEST + '.objective.read'
BRING = QUEST + '.objective.bring'

def flow(desktop, window, state, folder, resumed=False, item_rewards=False, reward_capacity=False, quest_offers=False):
    scale = state()['window'][2]
    read_pointer_attempts = 0
    def count(item):
        return sum(s['count'] for s in state()['inventory_bag']['slots'] if s and s['item'] == item)
    def ui(): return state()['loot']['ui']
    def corpses(): return state()['loot']['corpses']
    def feet(): return state()['dodge']['authority_feet']
    def quest(): return next((q for q in state()['quest_journal']['quests'] if q['id'] == QUEST), None)
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

    def close_reader():
        if state()['documents']['open']:
            desktop.key('Escape'); wait(lambda: not state()['documents']['open'])
            assert state()['inventory_open'], 'Reader Escape must keep the backpack open'
    def close_panels():
        close_reader()
        if ui()['open']:
            desktop.key('Escape'); wait(lambda: not ui()['open'])
        if state()['dialogue_open']:
            desktop.key('Escape'); wait(lambda: not state()['dialogue_open'] and not state().get('dialogue_pending', False))
        if state()['inventory_open']:
            desktop.key('i'); wait(lambda: not state()['inventory_open'])
    def speak():
        close_panels(); approach([8004.,0.,8000.])
        desktop.click(window,wait(lambda:state()['guide_screen']),scale,3)
        wait(lambda:state()['dialogue_open'] and state()['inventory_open'])
    def action(status):
        before=state()['quest_journal']['ui']['applied']
        point=expose_offer(QUEST) if quest_offers else wait(lambda:next((b['point'] for b in state()['quest_journal']['buttons'] if b['quest']==QUEST),None))
        desktop.click(window,point,scale,1)
        wait(lambda:quest() and quest()['status']==status and state()['quest_journal']['ui']['applied']==before+1 and not state()['quest_journal']['ui']['pending'])
    def document_click(name):
        nonlocal read_pointer_attempts
        point=wait(lambda:state()['document_layout'][name])
        deadline=time.monotonic()+3
        while True:
            time.sleep(.3)
            current=state()['document_layout'][name]
            if current==point and current and min(current)>0:
                break
            assert time.monotonic()<deadline, ('Document control layout did not settle',name)
            point=current
        width,height,_=state()['window']; assert 0<point[0]<width and 0<point[1]<height, (name,point)
        if name=='read':
            desktop.move(window,point,scale)
            time.sleep(.2)
            (folder/f'read-{read_pointer_attempts}-before-click.json').write_text(json.dumps(state(),indent=2)+'\n')
            read_pointer_attempts += 1
        desktop.click(window,point,scale,1)
    def select_note():
        # Quest acceptance can expose/reflow the backpack before the next
        # layout pass. Observe a settled native hit target before clicking.
        point=state()['document_layout']['note_slot']
        deadline=time.monotonic()+3
        while True:
            time.sleep(.3)
            current=state()['document_layout']['note_slot']
            if current==point and current and min(current)>0:
                break
            assert time.monotonic()<deadline, 'Note slot layout did not settle'
            point=current
        slot=next(i for i,stack in enumerate(state()['inventory_bag']['slots']) if stack and stack['item']==NOTE)
        selected=state()['inventory_ui']['selected']
        if selected is not None and selected!=slot:
            desktop.click(window,state()['inventory_layout']['cancel'],scale,1)
            wait(lambda:state()['inventory_ui']['selected'] is None)
        if state()['inventory_ui']['selected']!=slot:
            before=state()['inventory_ui']['slot_clicks']
            document_click('note_slot')
            time.sleep(.4)
            if state()['inventory_ui']['slot_clicks']==before:
                # Retry only an unobserved click, never a selection toggle.
                document_click('note_slot')
        wait(lambda:state()['inventory_ui']['selected']==slot)
    def open_note():
        if not state()['inventory_open']:
            desktop.key('i');wait(lambda:state()['inventory_open'])
        before=state()['documents']['approved']
        select_note()
        clicks=state()['documents']['read_clicks']
        document_click('read')
        time.sleep(.4)
        wait(lambda:state()['documents']['open'] and state()['documents']['approved']==before+1)
        assert state()['documents']['page']==0
    def take_note():
        before=count(NOTE)
        corpse=wait(lambda:next(iter(corpses()),None),5)
        open_corpse(corpse)
        assert count(NOTE)==before, 'Viewing corpse loot must not acquire a note'
        click('all')
        wait(lambda:count(NOTE)==before+1 and not ui()['open'],10)
    def destroy_note():
        select_note()
        point=wait(lambda:state()['inventory_layout']['destroy'])
        desktop.click(window,point,scale,1)
        wait(lambda:state()['inventory_ui']['destroy_confirmation'] is not None)
        desktop.click(window,point,scale,1)
        wait(lambda:count(NOTE)==0 and not state()['inventory_ui']['pending'])
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
            assert time.monotonic()<deadline,'Document quest wolf fight timed out'
            assert state()['combat']['player_deaths']==0
            desktop.key('2');time.sleep(.25)

    def expose_offer(wanted):
        for _ in range(16):
            journal=state()['quest_journal']
            viewport=wait(lambda:state()['quest_journal'].get('offer_scroll'))
            button=wait(lambda:next((b for b in state()['quest_journal']['buttons'] if b['quest']==wanted),None))
            cx,cy=viewport['point'];width,height=viewport['size']
            assert width>0 and height>0 and height<=state()['window'][1]*.5, viewport
            top,bottom=cy-height/2,cy+height/2
            if top-.5<=button['point'][1]-button['size'][1]/2 and button['point'][1]+button['size'][1]/2<=bottom+.5:
                # Reflow after an accepted offer can move a row. Require a settled
                # clipped native target before sending the actual pointer event.
                time.sleep(.25)
                current=next((b for b in state()['quest_journal']['buttons'] if b['quest']==wanted),None)
                if current and current['point']==button['point']:return current['point']
                continue
            body_before=journal['dialogue_body']['scroll']
            bag_before=state()['inventory_layout']['scroll']
            offset=viewport['scroll'][1]
            desktop.move(window,viewport['point'],scale)
            lines=8 if button['point'][1]<top else -8
            first_wheel=desktop.wheel_fd is None
            desktop.scroll(lines)
            try:
                wait(lambda:state()['quest_journal']['offer_scroll']['scroll'][1]!=offset,1 if first_wheel else 20)
            except TimeoutError:
                if not first_wheel:raise
                # The virtual device was just created; the window backend may
                # still be registering it. One more real gesture is bounded;
                # a nonfunctional scroll still fails the unchanged offset check.
                desktop.scroll(lines)
                wait(lambda:state()['quest_journal']['offer_scroll']['scroll'][1]!=offset)
            time.sleep(.15)
            assert state()['quest_journal']['dialogue_body']['scroll']==body_before, 'Quest wheel scrolled the dialogue sibling'
            assert state()['inventory_layout']['scroll']==bag_before, 'Quest wheel scrolled the backpack sibling'
        raise AssertionError(('Bounded offer list did not expose requested quest',wanted))

    def reward_label(verb):
        if quest_offers and not state()['documents']['open']:expose_offer(QUEST)
        label=wait(lambda:next((label for label in state()['quest_journal']['action_labels'] if label['quest']==QUEST and label['text'].startswith(verb+':')),None))
        assert '2 x Training Token' in label['text'] and label['glyphs']>0, label
        row=next(b for b in state()['quest_journal']['buttons'] if b['quest']==QUEST)
        assert all(measured<=allocated+1 for measured,allocated in zip(label['layout_size'],label['size'])),('Reward glyphs exceed their label',label)
        assert all(measured<=allocated+1 for measured,allocated in zip(label['layout_size'],row['content_box_size'])),('Reward text exceeds its clickable row',label,row)
        assert label['size'][0]<state()['window'][0], label
        if reward_capacity:assert '2 x Training Sword' in label['text'], label
        return label

    def inventory_click(name,index=None):
        point=wait(lambda:state()['inventory_layout'][name])
        if index is not None:point=point[index]
        # Selection/status text can change the backpack's height. Wait for its
        # actual painted geometry instead of clicking the previous frame's row.
        deadline=time.monotonic()+3
        while True:
            time.sleep(.2)
            current=state()['inventory_layout'][name]
            if index is not None:current=current[index]
            if current==point and current and min(current)>0:break
            assert time.monotonic()<deadline, ('Backpack layout did not settle',name)
            point=current
        if name=='destroy':
            desktop.move(window,point,scale)
            time.sleep(.35)
            observed=state()
            (folder/'before-capacity-destroy.json').write_text(json.dumps(observed,indent=2)+'\n')
            assert not any(h.get('inventory_destroy') and h.get('pressed') for h in observed['mouse_hover']), 'Destroy retained Pressed after all pointer buttons were released'
        desktop.click(window,point,scale,1)

    def capacity_check():
        while None in state()['inventory_bag']['slots']:
            slots=state()['inventory_bag']['slots']
            source=next(i for i,s in enumerate(slots) if s and s['item']=='theia.dev.training-stone' and s['count']>1)
            target=slots.index(None)
            if state()['inventory_ui']['selected'] is not None:
                inventory_click('cancel');wait(lambda:state()['inventory_ui']['selected'] is None)
            inventory_click('slots',source);wait(lambda:state()['inventory_ui']['selected']==source)
            inventory_click('split');wait(lambda:state()['inventory_ui']['split'])
            revision=state()['inventory_bag']['revision']
            inventory_click('slots',target)
            wait(lambda:state()['inventory_bag']['revision']==revision+1 and not state()['inventory_ui']['pending'])
        before=json.loads(json.dumps(state()['inventory_bag']))
        journal=state()['quest_journal'];head=journal['revision'];xp=journal['xp'];rejected=journal['ui']['rejected']
        point=wait(lambda:next((b['point'] for b in state()['quest_journal']['buttons'] if b['quest']==QUEST),None))
        desktop.click(window,point,scale,1)
        wait(lambda:state()['quest_journal']['ui']['rejected']==rejected+1 and not state()['quest_journal']['ui']['pending'])
        status=wait(lambda:state()['quest_journal']['status'] if 'Free some space' in state()['quest_journal']['status']['text'] and state()['quest_journal']['status']['glyphs']>0 else None)
        assert state()['inventory_bag']==before and quest()['status']=='Ready'
        assert state()['quest_journal']['revision']==head and state()['quest_journal']['xp']==xp
        capture('fixed-reward-backpack-full.png')
        target=next(i for i,s in enumerate(before['slots']) if s and s['item']=='theia.dev.training-stone' and s['count']==1)
        if state()['inventory_ui']['selected'] is not None:
            inventory_click('cancel');wait(lambda:state()['inventory_ui']['selected'] is None)
        inventory_click('slots',target);wait(lambda:state()['inventory_ui']['selected']==target)
        inventory_click('destroy');wait(lambda:state()['inventory_ui']['destroy_confirmation'] is not None)
        inventory_click('destroy')
        wait(lambda:state()['inventory_bag']['slots'][target] is None and not state()['inventory_ui']['pending'])
        assert quest()['status']=='Ready'
        return dict(refusal=status,unchanged_bag_revision=before['revision'],unchanged_journal_revision=head,unchanged_xp=xp,freed_slot=target,normal_splits_and_confirmed_destruction=True)

    if resumed:
        wait(lambda:quest() and quest()['status']=='Completed')
        assert quest()['counts'][READ]==1 and quest()['counts'][BRING]==1
        assert count(NOTE)==0 and not state()['documents']['open']
        if item_rewards:
            assert count('theia.dev.training-token')==103, 'Fixed reward must survive rejoin without another grant'
            assert not any(label['quest']==QUEST for label in state()['quest_journal']['action_labels'])
            assert 'Items:' not in state()['quest_journal']['tracker']['text']
            if reward_capacity:assert count('theia.dev.training-sword')==2
        capture('resumed-document-completed.png')
        return dict(passed=True,resumed=True,revision=state()['quest_journal']['revision'],xp=state()['quest_journal']['xp'],completion_and_consumption_restored=True,fixed_reward_tokens=count('theia.dev.training-token') if item_rewards else None)
    assert quest() is None and count(NOTE)==0
    if not state()['sword_equipped']:
        desktop.key('i');wait(lambda:state()['inventory_open'])
        desktop.click(window,wait(lambda:state()['sword_slot_screen']),scale,1)
        desktop.click(window,wait(lambda:state()['equip_screen']),scale,1)
        wait(lambda:state()['sword_equipped'])
        close_panels()
    kill_wolf();take_note()
    open_note(); assert quest() is None
    capture('document-page-one.png')
    document_click('next');wait(lambda:state()['documents']['page']==1)
    capture('document-page-two.png')
    document_click('next');time.sleep(.2);assert state()['documents']['page']==1
    document_click('previous');wait(lambda:state()['documents']['page']==0)
    close_reader();speak()
    offer_fixture=None
    if quest_offers:
        journal=state()['quest_journal']
        assert len(journal['buttons'])==32, journal['buttons']
        (folder/'quest-offers-initial.json').write_text(json.dumps(journal,indent=2)+'\n')
        capture('quest-offers-initial.png')
        point=expose_offer('theia.dev.zz-offer-tail')
        capture('quest-offers-last.png')
        before=state()['quest_journal']['ui']['applied']
        desktop.click(window,point,scale,1)
        wait(lambda:any(q['id']=='theia.dev.zz-offer-tail' and q['status']=='Active' for q in state()['quest_journal']['quests']) and state()['quest_journal']['ui']['applied']==before+1 and not state()['quest_journal']['ui']['pending'])
        offer_fixture=dict(count=32,last_offer_accepted=True,initial=journal['offer_scroll'],final=wait(lambda:state()['quest_journal']['offer_scroll']))
    offer=reward_label('Accept') if item_rewards else None
    if item_rewards:capture('fixed-reward-offer.png')
    action('Active')
    assert quest()['counts'][READ]==0 and quest()['counts'][BRING]==1, 'Pre-accept reading must not grant later credit'
    open_note();wait(lambda:quest()['status']=='Ready')
    assert quest()['counts'][READ]==1 and quest()['counts'][BRING]==1
    if item_rewards:
        wait(lambda:'2 x Training Token' in state()['quest_journal']['tracker']['text'])
        reward_label('Complete')
    capture('document-ready.png')
    close_reader();assert state()['dialogue_open'], 'Reader Escape must retain guide dialogue'
    desktop.key('Escape');wait(lambda:not state()['dialogue_open'])
    destroy_note();wait(lambda:quest()['status']=='Active' and quest()['counts'][BRING]==0)
    assert quest()['counts'][READ]==1
    capture('document-loss-reverses-possession.png')
    kill_wolf()
    assert quest()['status']=='Active' and quest()['counts'][BRING]==0
    take_note();wait(lambda:quest()['status']=='Ready')
    assert quest()['counts'][READ]==1, 'Reacquisition must retain historical reading without another open'
    speak()
    turn_in=reward_label('Complete') if item_rewards else None
    tokens=count('theia.dev.training-token')
    assert not item_rewards or tokens==101, 'Only two explicit corpse pickups should precede fixed rewards'
    capacity=capacity_check() if reward_capacity else None
    action('Completed');wait(lambda:count(NOTE)==0)
    if item_rewards:
        wait(lambda:count('theia.dev.training-token')==tokens+2)
        assert 'Items:' not in state()['quest_journal']['tracker']['text']
        if reward_capacity:
            assert count('theia.dev.training-sword')==2
            wait(lambda:state()['quest_journal']['status']['text']=='')
    assert quest()['counts'][READ]==1 and quest()['counts'][BRING]==1
    capture('document-completed.png')
    assert read_pointer_attempts==2, 'Each note open must take exactly one pointer click'
    assert state()['combat']['deaths']==2 and state()['combat']['player_deaths']==0 and state()['combat']['event_gaps']==0
    return dict(passed=True,resumed=False,revision=state()['quest_journal']['revision'],xp=state()['quest_journal']['xp'],actions=state()['quest_journal']['ui']['applied'],kills=2,preaccept_read_excluded=True,page_navigation_local=True,history_survives_loss=True,possession_reverses=True,explicit_pickup_required=True,delivery_consumed=True,documents=state()['documents'],read_pointer_attempts=read_pointer_attempts,offer_fixture=offer_fixture,fixed_rewards=dict(offer=offer,turn_in=turn_in,tokens_before=tokens,tokens_after=count('theia.dev.training-token'),capacity=capacity) if item_rewards else None)
