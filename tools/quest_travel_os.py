"""Owned native input: authored collection travel and destination-only NPC escort."""
import json
import math
import subprocess
import time
from types import SimpleNamespace
from qualify_world_input_os import wait, capture_ready

COLLECT = 'theia.dev.quest-lost-compass'
FOLLOW = 'theia.dev.quest-follow-ranger'
INTRO = 'theia.dev.quest-meet-ranger'
ROWAN = 'theia.dev.villager'
RANGER = 'theia.dev.ranger'


def controls(desktop, window, state, folder, check_banner=False):
    scale = state()['window'][2]
    banner_layouts = []
    def journal(): return state()['quest_journal']
    def quest(q): return next((p for p in journal()['quests'] if p['id'] == q), None)
    def feet(): return state()['dodge']['authority_feet']
    def contact(npc):
        # Render entities can briefly refresh independently of the retained
        # authority state. Wait for the requested contact, never substitute one.
        return wait(lambda: next((c for c in state()['contacts'] if c['npc'] == npc), None), 5)
    def focus():
        desktop.focus(window)
        w, h, _ = state()['window']
        desktop.move(window, [w*.55, h*.42], scale)
        wait(lambda: state()['cursor']['focused'])
    def capture(name):
        (folder/'capture.request').write_text(name+'\n')
        wait(lambda: capture_ready(folder/'input-os.png'))
        (folder/'input-os.png').rename(folder/name)
    def banner_layout():
        s = state()
        bounds = s.get('test_controller_button')
        if not bounds or min(bounds[2:]) <= 0:
            return None
        x, y, w, h = bounds
        if abs(x+w/2-s['window'][0]/2) > 2 or y < 0 or y+h > 64 or w > s['window'][0]/2+2:
            return None
        for name in ['dialogue_body', 'offer_scroll']:
            pane = s['quest_journal'].get(name)
            if pane and min(pane['size']) > 0:
                px, py = pane['point']; pw, ph = pane['size']
                assert x+w <= px-pw/2 or px+pw/2 <= x or y+h <= py-ph/2 or py+ph/2 <= y, (bounds, pane)
        return dict(window=s['window'], bounds=bounds)
    def close():
        for _ in range(4):
            s=state()
            if not (s['dialogue_open'] or s['inventory_open'] or s['interface']['open'] or s.get('party', {}).get('open')): return
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
        # A second native client may map/activate while the checked travel
        # controller runs without OS input. Reacquire only our owned window
        # before the next interaction; Desktop still guards every input event.
        focus()
        approach(npc)
        desktop.click(window,wait(lambda: contact(npc)['screen']),scale,3)
        wait(lambda: state()['dialogue_open'] and not state()['dialogue_pending'])
        if check_banner:
            if not banner_layouts:
                revision = journal()['revision']
                for width in [3440, 1920]:
                    # This desktop promotes an exact display-sized request to
                    # fullscreen; leave that state before requesting a smaller
                    # client area. Affect only this owned native window.
                    subprocess.run(['wmctrl', '-i', '-r', hex(window), '-b', 'remove,fullscreen'], check=True)
                    wait(lambda: '_NET_WM_STATE_FULLSCREEN' not in subprocess.check_output(
                        ['xprop', '-id', hex(window), '_NET_WM_STATE'], text=True), 5)
                    subprocess.run(['wmctrl', '-i', '-r', hex(window), '-b', 'remove,maximized_vert,maximized_horz'], check=True)
                    subprocess.run(['wmctrl', '-i', '-r', hex(window), '-e', f'0,0,0,{width},1440'], check=True)
                    wait(lambda: state()['window'][:2] == [width, 1440])
                    focus()
                    banner_layouts.append(wait(banner_layout))
                    assert journal()['revision'] == revision and state()['dialogue_open']
                    capture(f'route-banner-{width}.png')
            wait(banner_layout)
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
        focus()
        close()
        # Dialogue range is wider than the checked follower's start tolerance.
        # Walk to the actual authored endpoint before handing over normal input.
        endpoint = [8194, 0, 8168] if leg == 'outward' else [8194, .634, 8260]
        start_deadline = time.monotonic()+20
        while math.hypot(feet()[0]-endpoint[0], feet()[2]-endpoint[2])>.45:
            assert time.monotonic()<start_deadline, 'Route-start approach stalled'
            walk(endpoint, stop=.3);time.sleep(.15)
        if check_banner and leg == 'outward':
            x, y, w, h = wait(banner_layout)['bounds']
            desktop.click(window, [x+w/2, y+h/2], scale, 1)
        else:
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
        focus()
        return t
    return SimpleNamespace(
        journal=journal, quest=quest, feet=feet, contact=contact, focus=focus, capture=capture, close=close, face=face, walk=walk, approach=approach, speak=speak, action=action, travel=travel,
        banner_layouts=banner_layouts)


def flow(desktop, window, state, folder, resumed=False, check_banner=False, prepare_party=False):
    scale = state()['window'][2]
    ui = controls(desktop, window, state, folder, check_banner)
    ui.focus()
    if resumed:
        wait(lambda: all(ui.quest(q) and ui.quest(q)['status']=='Completed' for q in [INTRO,COLLECT,FOLLOW]))
        assert ui.journal()['revision']==9 and ui.journal()['xp']==75
        assert ui.journal()['ui']['submitted']==0
        assert math.dist(ui.contact(RANGER)['global'],[8276,2.7,8128])<2
        assert not any(i['available'] for i in state()['ground_items'])
        ui.speak(RANGER)
        assert all(b['quest']!=FOLLOW for b in ui.journal()['buttons'])
        ui.capture('travel-restored.png')
        result=dict(passed=True,resumed=True,revision=ui.journal()['revision'],xp=ui.journal()['xp'])
    else:
        # Prove Tab works even with the bag open; Shift+Tab traverses backwards.
        desktop.key('i');wait(lambda: state()['inventory_open'])
        desktop.key('Tab');first=wait(lambda: state()['selected_hostile'])
        desktop.key('Tab');second=wait(lambda: state()['selected_hostile'] if state()['selected_hostile']!=first else None)
        desktop.key_state('Shift_L',True)
        try:
            desktop.key('Tab')
            # Keep the modifier held until the native client has processed the
            # chord. Startup rendering can take longer than the X11 key pulse.
            wait(lambda: state()['selected_hostile']==first)
        finally: desktop.key_state('Shift_L',False)
        ui.close()
        desktop.key('BackSpace');wait(lambda: state()['selected_hostile'] is None)
        # Hidden by default, controlled through the ordinary Interface menu.
        assert not state()['interface']['debug_stats']
        desktop.key('F10');wait(lambda: state()['interface']['open'])
        point=next(b['point'] for b in state()['interface']['buttons'] if b['action']=='debug')
        desktop.click(window,point,scale,1);wait(lambda: state()['interface']['debug_stats']);ui.close();ui.capture('debug-stats.png')
        desktop.key('F10');wait(lambda: state()['interface']['open'])
        desktop.click(window,next(b['point'] for b in state()['interface']['buttons'] if b['action']=='debug'),scale,1)
        wait(lambda: not state()['interface']['debug_stats']);ui.close()
        ui.speak(ROWAN);ui.action(COLLECT,'Active');ui.close()
        wait(lambda: any(i['available'] for i in state()['ground_items']))
        ui.capture('collect-from-meadow.png')
        outward=ui.travel('outward')
        ui.face([8194,0.634,8260])
        item=wait(lambda: next((i for i in state()['ground_items'] if i['available'] and i['visible'] and i['screen']),None))
        desktop.click(window,item['screen'],scale,3)
        wait(lambda: ui.quest(COLLECT)['status']=='Ready')
        assert ui.journal()['xp']==0
        ui.capture('deliver-to-rowan.png')
        returning=ui.travel('return')
        ui.speak(ROWAN);ui.action(COLLECT,'Completed')
        assert ui.journal()['xp']==25
        assert not any(s and s['item']=='theia.dev.lost-compass' for s in state()['inventory_bag']['slots'])
        ui.speak('theia.dev.steward');ui.action(INTRO,'Active')
        ui.speak(RANGER)
        greeting=wait(lambda: next((c['point'] for c in ui.journal()['choices'] if c['choice']=='theia.dev.ranger.greeting'),None))
        desktop.click(window,greeting,scale,1);wait(lambda: ui.quest(INTRO)['status']=='Ready');ui.action(INTRO,'Completed')
        if prepare_party:
            ui.close()
            result = dict(passed=True, prepared_party=True, revision=ui.journal()['revision'], xp=ui.journal()['xp'], outward=outward, returning=returning)
            assert result['revision'] == 6 and result['xp'] == 50
            (folder/'quest-travel-prepared.json').write_text(json.dumps(result, indent=2)+'\n')
            return result
        ui.speak(RANGER);home=ui.contact(RANGER)['global'];ui.action(FOLLOW,'Active');ui.close()
        time.sleep(10)
        paused=ui.contact(RANGER)['global'];waiting_distance=math.dist(ui.feet(),paused)
        assert 9.<waiting_distance<=10.1,(ui.feet(),paused,waiting_distance)
        time.sleep(1)
        assert math.dist(paused,ui.contact(RANGER)['global'])<.02
        assert math.dist(paused,ui.contact(RANGER)['rendered_global'])<.02
        assert ui.quest(FOLLOW)['status']=='Active' and ui.journal()['xp']==50
        ui.capture('ranger-waits.png')
        samples=[];deadline=time.monotonic()+180
        moving=False
        try:
            while ui.quest(FOLLOW)['status']=='Active':
                ranger=ui.contact(RANGER);target=ranger['global']
                samples.append(dict(npc=target,rendered=ranger['rendered_global'],player=ui.feet()))
                assert time.monotonic()<deadline, 'NPC did not arrive'
                distance=math.dist(ui.feet(),target)
                ui.face(target)
                if not moving and distance>6.:
                    desktop.key_state('w',True);moving=True
                elif moving and distance<3.:
                    desktop.key_state('w',False);moving=False
                time.sleep(.1)
        finally:
            if moving: desktop.key_state('w',False)
        assert ui.quest(FOLLOW)['status']=='Ready' and ui.journal()['xp']==50
        assert any(math.dist(s['npc'],s['rendered'])>.01 for s in samples), 'No interpolated contact samples'
        assert max(math.dist(s['npc'],s['rendered']) for s in samples)<3., 'Contact presentation lag exceeded budget'
        wait(lambda: math.dist(ui.contact(RANGER)['global'],ui.contact(RANGER)['rendered_global'])<.02)
        ui.capture('ranger-at-tree-ready.png')
        ui.speak(RANGER);ui.action(FOLLOW,'Completed')
        assert ui.journal()['revision']==9 and ui.journal()['xp']==75
        ui.capture('ranger-completed.png')
        result=dict(passed=True,resumed=False,revision=ui.journal()['revision'],xp=ui.journal()['xp'],outward=outward,returning=returning,home=home,paused=paused,waiting_distance=waiting_distance,follow_samples=samples,tab=[first,second,first])
    if check_banner:
        result.update(route_banner_layouts=ui.banner_layouts, route_banner_clicked_outward=True)
    (folder/'quest-travel-proof.json').write_text(json.dumps(result,indent=2)+'\n')
    return result
