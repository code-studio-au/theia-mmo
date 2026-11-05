#!/usr/bin/env python3
"""Normal PID-bound pointer/keyboard corpse pickup; telemetry is read-only."""
import math
import time
from qualify_world_input_os import wait, capture_ready
PELT = 'theia.dev.wolf-pelt'
TOKEN = 'theia.dev.training-token'


def flow(desktop, window, state, folder, resumed=False):
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
            distance = math.hypot(point[0]-feet()[0],point[2]-feet()[2])
            if distance < 1.8:
                return
            face(point)
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
    if state()['dialogue_open']:
        desktop.key('Escape')
        wait(lambda: not state()['dialogue_open'] and not state().get('dialogue_pending',False) and not any(state().get('dialogue_ui',{}).get(k,False) for k in ['open','pending','close_when_open']))
    if not state()['inventory_open']:
        desktop.key('i')
        wait(lambda: state()['inventory_open'])
    journal = state()['quest_journal']['quests']
    journal_revision = state()['quest_journal']['revision']
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
