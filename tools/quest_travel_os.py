"""Owned native input: authored collection travel and destination-only NPC escort."""
import json
import math
import time
from qualify_world_input_os import wait, capture_ready

COLLECT = 'theia.dev.quest-lost-compass'
FOLLOW = 'theia.dev.quest-follow-ranger'
INTRO = 'theia.dev.quest-meet-ranger'
ROWAN = 'theia.dev.villager'
RANGER = 'theia.dev.ranger'


def flow(desktop, window, state, folder, resumed=False):
    scale = state()['window'][2]
    def journal(): return state()['quest_journal']
    def quest(q): return next((p for p in journal()['quests'] if p['id'] == q), None)
    def feet(): return state()['dodge']['authority_feet']
    def contact(npc): return next(c for c in state()['contacts'] if c['npc'] == npc)
    def capture(name):
        (folder/'capture.request').write_text(name+'\n')
        wait(lambda: capture_ready(folder/'input-os.png'))
        (folder/'input-os.png').rename(folder/name)
    def close():
        for _ in range(4):
            s=state()
            if not (s['dialogue_open'] or s['inventory_open'] or s['interface']['open']): return
            desktop.key('Escape')
            time.sleep(.25)
        assert not state()['dialogue_open'] and not state()['inventory_open']
    def face(target):
        for _ in range(10):
            p=feet(); yaw=math.atan2(-(target[0]-p[0]),-(target[2]-p[2]))
            heading=state()['classic_facing']*math.tau/65536
            if abs(math.atan2(math.sin(yaw-heading),math.cos(yaw-heading)))<.06: return
            w,h,_=state()['window'];desktop.move(window,[w*.55,h*.42],scale)
            desktop.button(3,True)
            try:
                angle=yaw-state()['camera_angles'][0];angle=math.atan2(math.sin(angle),math.cos(angle))
                desktop.look(round(max(-400,min(400,-angle/.004))),0)
            finally: desktop.button(3,False)
            time.sleep(.15)
        raise AssertionError('Facing did not settle')
    def walk(target, stop=1.9):
        face(target)
        p=feet();distance=math.hypot(target[0]-p[0],target[2]-p[2])
        if distance>stop: desktop.hold_key('w',min(.4,max(.025,(distance-stop)/5)))
    def approach(npc):
        close()
        deadline=time.monotonic()+30
        while math.dist(feet(),contact(npc)['global'])>2.5:
            assert time.monotonic()<deadline, 'Contact approach stalled'
            walk(contact(npc)['global']);time.sleep(.1)
        face(contact(npc)['global'])
    def speak(npc):
        approach(npc)
        desktop.click(window,wait(lambda: contact(npc)['screen']),scale,3)
        wait(lambda: state()['dialogue_open'] and not state()['dialogue_pending'])
    def action(q, expected):
        before=journal()['ui']['applied']
        def button(): return next((b['point'] for b in journal()['buttons'] if b['quest']==q),None)
        point=wait(button)
        for _ in range(30):
            pane=journal().get('offer_scroll')
            if not pane or pane['point'][1]-pane['size'][1]/2+5<=point[1]<=pane['point'][1]+pane['size'][1]/2-5: break
            desktop.move(window,pane['point'],scale);desktop.scroll(-1 if point[1]>pane['point'][1] else 1);time.sleep(.15);point=wait(button)
        desktop.click(window,point,scale,1)
        wait(lambda: quest(q) and quest(q)['status']==expected and journal()['ui']['applied']==before+1 and not journal()['ui']['pending'])
    def travel(leg):
        close()
        # Dialogue range is wider than the checked follower's start tolerance.
        # Walk to the actual authored endpoint before handing over normal input.
        endpoint = [8194, 0, 8168] if leg == 'outward' else [8194, .634, 8260]
        start_deadline = time.monotonic()+20
        while math.hypot(feet()[0]-endpoint[0], feet()[2]-endpoint[2])>.45:
            assert time.monotonic()<start_deadline, 'Route-start approach stalled'
            walk(endpoint, stop=.3);time.sleep(.15)
        desktop.key('F7')
        def started():
            controller = state()['test_controller']
            assert not controller['travel']['failure'], controller['travel']
            return controller['active']
        wait(started)
        deadline=time.monotonic()+100
        while True:
            s=state()['test_controller'];t=s['travel']
            assert not t['failure'],t
            if not s['active'] and t['finished']: break
            assert time.monotonic()<deadline,t
            time.sleep(.25)
        assert t['leg']==leg,t
        return t
    desktop.focus(window);wait(lambda: state()['cursor']['focused'])
    if resumed:
        wait(lambda: all(quest(q) and quest(q)['status']=='Completed' for q in [INTRO,COLLECT,FOLLOW]))
        assert journal()['revision']==9 and journal()['xp']==75
        assert journal()['ui']['submitted']==0
        assert math.dist(contact(RANGER)['global'],[8276,2.7,8128])<2
        assert not any(i['available'] for i in state()['ground_items'])
        speak(RANGER)
        assert all(b['quest']!=FOLLOW for b in journal()['buttons'])
        capture('travel-restored.png')
        result=dict(passed=True,resumed=True,revision=journal()['revision'],xp=journal()['xp'])
    else:
        # Prove Tab works even with the bag open; Shift+Tab traverses backwards.
        desktop.key('i');wait(lambda: state()['inventory_open'])
        desktop.key('Tab');first=wait(lambda: state()['selected_hostile'])
        desktop.key('Tab');second=wait(lambda: state()['selected_hostile'] if state()['selected_hostile']!=first else None)
        desktop.key_state('Shift_L',True)
        try: desktop.key('Tab')
        finally: desktop.key_state('Shift_L',False)
        wait(lambda: state()['selected_hostile']==first)
        close()
        desktop.key('BackSpace');wait(lambda: state()['selected_hostile'] is None)
        # Hidden by default, controlled through the ordinary Interface menu.
        assert not state()['interface']['debug_stats']
        desktop.key('F10');wait(lambda: state()['interface']['open'])
        point=next(b['point'] for b in state()['interface']['buttons'] if b['action']=='debug')
        desktop.click(window,point,scale,1);wait(lambda: state()['interface']['debug_stats']);close();capture('debug-stats.png')
        desktop.key('F10');wait(lambda: state()['interface']['open'])
        desktop.click(window,next(b['point'] for b in state()['interface']['buttons'] if b['action']=='debug'),scale,1)
        wait(lambda: not state()['interface']['debug_stats']);close()
        speak(ROWAN);action(COLLECT,'Active');close()
        wait(lambda: any(i['available'] for i in state()['ground_items']))
        capture('collect-from-meadow.png')
        outward=travel('outward')
        face([8194,0.634,8260])
        item=wait(lambda: next((i for i in state()['ground_items'] if i['available'] and i['visible'] and i['screen']),None))
        desktop.click(window,item['screen'],scale,3)
        wait(lambda: quest(COLLECT)['status']=='Ready')
        assert journal()['xp']==0
        capture('deliver-to-rowan.png')
        returning=travel('return')
        speak(ROWAN);action(COLLECT,'Completed')
        assert journal()['xp']==25
        assert not any(s and s['item']=='theia.dev.lost-compass' for s in state()['inventory_bag']['slots'])
        speak('theia.dev.steward');action(INTRO,'Active')
        speak(RANGER)
        greeting=wait(lambda: next((c['point'] for c in journal()['choices'] if c['choice']=='theia.dev.ranger.greeting'),None))
        desktop.click(window,greeting,scale,1);wait(lambda: quest(INTRO)['status']=='Ready');action(INTRO,'Completed')
        speak(RANGER);home=contact(RANGER)['global'];action(FOLLOW,'Active');close()
        time.sleep(10)
        paused=contact(RANGER)['global'];distance=math.dist(feet(),paused)
        assert 9.<distance<=10.1,(feet(),paused,distance)
        time.sleep(1)
        assert math.dist(paused,contact(RANGER)['global'])<.02
        assert math.dist(paused,contact(RANGER)['rendered_global'])<.02
        assert quest(FOLLOW)['status']=='Active' and journal()['xp']==50
        capture('ranger-waits.png')
        samples=[];deadline=time.monotonic()+180
        moving=False
        try:
            while quest(FOLLOW)['status']=='Active':
                ranger=contact(RANGER);target=ranger['global']
                samples.append(dict(npc=target,rendered=ranger['rendered_global'],player=feet()))
                assert time.monotonic()<deadline, 'NPC did not arrive'
                distance=math.dist(feet(),target)
                face(target)
                if not moving and distance>6.:
                    desktop.key_state('w',True);moving=True
                elif moving and distance<3.:
                    desktop.key_state('w',False);moving=False
                time.sleep(.1)
        finally:
            if moving: desktop.key_state('w',False)
        assert quest(FOLLOW)['status']=='Ready' and journal()['xp']==50
        assert any(math.dist(s['npc'],s['rendered'])>.01 for s in samples), 'No interpolated contact samples'
        assert max(math.dist(s['npc'],s['rendered']) for s in samples)<3., 'Contact presentation lag exceeded budget'
        wait(lambda: math.dist(contact(RANGER)['global'],contact(RANGER)['rendered_global'])<.02)
        capture('ranger-at-tree-ready.png')
        speak(RANGER);action(FOLLOW,'Completed')
        assert journal()['revision']==9 and journal()['xp']==75
        capture('ranger-completed.png')
        result=dict(passed=True,resumed=False,revision=journal()['revision'],xp=journal()['xp'],outward=outward,returning=returning,home=home,paused=paused,waiting_distance=distance,follow_samples=samples,tab=[first,second,first])
    (folder/'quest-travel-proof.json').write_text(json.dumps(result,indent=2)+'\n')
    return result
