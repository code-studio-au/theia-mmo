"""Native party controls using only owned-window mouse and keyboard input."""
import json
import math
import time
from qualify_world_input_os import wait, capture_ready


def form(desktop, windows, states):
    """Invite and accept for consumers; the full control matrix is in flow()."""
    def focus(index):
        desktop.focus(windows[index])
        w, h, scale = states[index]()['window']
        desktop.move(windows[index], [w*.55, h*.42], scale)
        wait(lambda: states[index]()['cursor']['focused'])
        if not states[index]()['party']['open']:
            desktop.key('o')
            wait(lambda: states[index]()['party']['open'])
    def click(index, action, target=None):
        focus(index)
        button = wait(lambda: next((b for b in states[index]()['party']['buttons']
                                  if b['action'] == action and (target is None or b['target'] == target)), None))
        desktop.click(windows[index], button['point'], states[index]()['window'][2], 1)
    assert all(s()['party']['state']['party'] is None for s in states)
    click(0, 'invite', states[1]()['actor'])
    wait(lambda: states[1]()['party']['state']['incoming'] is not None, 5)
    click(1, 'accept')
    wait(lambda: all(s()['party']['state']['party'] and not s()['party']['state']['pending'] for s in states), 5)
    views = [s()['party']['state'] for s in states]
    assert views[0]['party'] == views[1]['party'] and len(views[0]['party']['members']) == 2
    assert all(v['refusal'] is None for v in views)
    return dict(passed=True, party=views[0]['party'], actions=2)


def flow(desktop, windows, states, folder):
    initial_progress = [dict(journal=s()['quest_journal'], bag=s()['inventory_bag']) for s in states]
    keys = [s()['actor'] for s in states]
    last = [0., 0.]
    steps = []
    movement = []
    member_proofs = []
    menu_proofs = []
    def view(index):
        return states[index]()['party']['state']
    def focus(index):
        desktop.focus(windows[index])
        w, h, scale = states[index]()['window']
        desktop.move(windows[index], [w*.55, h*.42], scale)
        wait(lambda: states[index]()['cursor']['focused'])
    def click(index, action, target=None, source=None):
        focus(index)
        delay = .6 - (time.monotonic()-last[index])
        if delay > 0: time.sleep(delay)
        def button():
            p = states[index]()['party']
            if p['state']['pending']: return None
            return next((b for b in p['buttons'] if b['action'] == action and (target is None or b['target'] == target) and (source is None or b['source'] == source)), None)
        b = wait(button, 5)
        sequence = view(index)['sequence']
        desktop.click(windows[index], b['point'], states[index]()['window'][2], 1)
        last[index] = time.monotonic()
        wait(lambda: view(index)['sequence'] > sequence and not view(index)['pending'], 5)
        assert view(index)['refusal'] is None, view(index)
        steps.append(dict(player=index+1, action=action, sequence=view(index)['sequence']))
        for i, state in enumerate(states):
            current = state()
            assert dict(journal=current['quest_journal'], bag=current['inventory_bag']) == initial_progress[i]
    def capture(index, name):
        focus(index)
        path = folder/('owner' if index == 0 else 'guest')
        (path/'capture.request').write_text(name+'\n')
        wait(lambda: capture_ready(path/'input-os.png'))
        (path/'input-os.png').rename(path/name)
    def invite():
        click(0, 'invite', keys[1])
        wait(lambda: view(1)['incoming'] is not None, 5)
    def join():
        invite()
        click(1, 'accept')
        wait(lambda: all(view(i)['party'] and len(view(i)['party']['members']) == 2 for i in [0,1]), 5)
        assert view(0)['party'] == view(1)['party']
    def solo():
        wait(lambda: all(view(i)['party'] is None and view(i)['incoming'] is None and not view(i)['outgoing'] for i in [0,1]), 5)
        wait(lambda: all(not s()['party']['member_frames']['visible'] and not s()['party']['member_frames']['frames'] for s in states))
    def member_frames():
        for i in [0, 1]:
            def painted():
                sample = states[i]()
                frames = sample['party']['member_frames']
                other = states[1-i]()
                if not frames['visible'] or len(frames['frames']) != 1: return None
                frame = frames['frames'][0]
                if frame['actor'] != keys[1-i]: return None
                if len(frames['labels']) != 2 or not all(v['rendered'] for v in frames['labels']): return None
                expected = [other['combat_ui']['player_health_text'], other['combat_ui']['mana_text']]
                if sorted(v['text'] for v in frames['labels']) != sorted(expected): return None
                if frame['leader'] != (view(i)['party']['leader'] == keys[1-i]): return None
                identities = frames['identities']
                expected_names = [frame['name']] + (['Leader'] if frame['leader'] else [])
                if sorted(v['text'] for v in identities) != sorted(expected_names): return None
                if not all(v['rendered'] for v in identities): return None
                if len(frames['bars']) != 2: return None
                for bar in frames['bars']:
                    current, maximum = frame[bar['pool']]
                    if abs(bar['width'] - bar['maximum_width'] * current / maximum) > 1.5: return None
                x,y,w,h = frame['bounds']
                assert 0 <= x and 64 <= y and w > 200 and 40 < h < 110
                assert x+w < sample['window'][0] and y+h < sample['window'][1]-190
                assert frame['name'] == other['combat_ui']['player_name']
                return frames
            member_proofs.append(dict(player=i+1, frames=wait(painted, 8)))
    def open_member(index):
        focus(index)
        frame = wait(lambda: states[index]()['party']['member_frames']['frames'][0])
        x,y,w,h = frame['bounds']
        before = states[index]()
        desktop.click(windows[index], [x+w/2,y+h/2], before['window'][2], 3)
        menu = wait(lambda: value if (value := states[index]()['party']['member_menu'])['visible'] and value['bounds'] else None, 5)
        assert menu['name'] == states[1-index]()['combat_ui']['player_name']
        x,y,w,h = menu['bounds']
        assert 0 <= x and 0 <= y and x+w <= before['window'][0] and y+h <= before['window'][1]
        controls = [b['action'] for b in states[index]()['party']['buttons'] if b['source']=='member']
        leader = view(index)['party']['leader'] == keys[index]
        assert ('promote' in controls) == leader and ('remove' in controls) == leader
        assert 'manage' in controls and 'menu-close' in controls
        after = states[index]()
        assert after['camera_angles'] == before['camera_angles'] and after['classic_facing'] == before['classic_facing']
        for key in ['enemy_clicks','friendly_clicks','basic_requests','interaction_requests']:
            assert after['controls'][key] == before['controls'][key], key
        menu_proofs.append(dict(player=index+1,menu=menu,actions=controls))
        return menu
    for i in [0, 1]:
        wait(lambda: states[i]().get('party', {}).get('state') is not None, 20)
        assert view(i)['party'] is None and view(i)['incoming'] is None
        focus(i); desktop.key('o')
        wait(lambda: states[i]()['party']['open'], 5)
        start = states[i]()['dodge']['authority_feet']
        desktop.hold_key('w', .35)
        distance = wait(lambda: value if (value := math.dist(start, states[i]()['dodge']['authority_feet'])) > .4 else None, 5)
        movement.append(distance)
    # Incoming invitations must be actionable without opening party management.
    focus(1); desktop.key('o')
    wait(lambda: not states[1]()['party']['open'])
    invite()
    prompt = wait(lambda: states[1]()['party']['invitation']
                  if states[1]()['party']['invitation']['visible'] and states[1]()['party']['invitation']['bounds'] else None)
    x, y, width, height = prompt['bounds']
    viewport = states[1]()['window']
    assert abs(x+width/2-viewport[0]/2)<2 and 64<=y and y+height<viewport[1]/2, prompt
    capture(1, 'party-invitation.png')
    before_drag = states[1]()
    desktop.drag(windows[1], [x+width/2, y+height/2], viewport[2])
    after_drag = states[1]()
    assert after_drag['camera_angles']==before_drag['camera_angles']
    assert after_drag['classic_facing']==before_drag['classic_facing']
    for key in ['enemy_clicks','friendly_clicks','basic_requests','interaction_requests']:
        assert after_drag['controls'][key]==before_drag['controls'][key], key
    desktop.key('o'); wait(lambda: states[1]()['party']['open'])
    wait(lambda: not states[1]()['party']['invitation']['visible'])
    desktop.key('o'); wait(lambda: states[1]()['party']['invitation']['visible'])
    click(1, 'decline'); solo()
    wait(lambda: not states[1]()['party']['invitation']['visible'])
    invite(); click(0, 'cancel'); solo()
    wait(lambda: not states[1]()['party']['invitation']['visible'])
    join()
    member_frames()
    focus(0); desktop.key('o'); wait(lambda: not states[0]()['party']['open'])
    capture(0, 'party-member-frames.png')
    frame = states[0]()['party']['member_frames']['frames'][0]
    x,y,w,h = frame['bounds']
    before_frame_drag = states[0]()
    desktop.drag(windows[0], [x+w/2, y+h/2], before_frame_drag['window'][2])
    after_frame_drag = states[0]()
    assert not after_frame_drag['party']['member_menu']['visible'], 'Frame drag opened a context menu'
    assert after_frame_drag['camera_angles'] == before_frame_drag['camera_angles']
    assert after_frame_drag['classic_facing'] == before_frame_drag['classic_facing']
    for key in ['enemy_clicks','friendly_clicks','basic_requests','interaction_requests']:
        assert after_frame_drag['controls'][key] == before_frame_drag['controls'][key], key
    manage = wait(lambda: next((b for b in states[0]()['party']['buttons'] if b['label'] == 'Manage party'), None))
    desktop.click(windows[0], manage['point'], states[0]()['window'][2], 1)
    wait(lambda: states[0]()['party']['open'])
    capture(0, 'party-members.png')
    wait(lambda: not states[1]()['party']['invitation']['visible'])
    focus(1); desktop.key('o'); wait(lambda: states[1]()['party']['open'])
    for i in [0,1]:
        menu = open_member(i)
        if i == 0: capture(i, 'party-member-menu.png')
        desktop.key('Escape')
        wait(lambda: not states[i]()['party']['member_menu']['visible'])
        assert states[i]()['party']['open'], 'Context Escape also closed Party'
        open_member(i)
        before = states[i]()
        desktop.click(windows[i], [before['window'][0]*.8, before['window'][1]*.5], before['window'][2], 1)
        wait(lambda: not states[i]()['party']['member_menu']['visible'])
        after = states[i]()
        assert after['camera_angles'] == before['camera_angles'] and after['classic_facing'] == before['classic_facing']
        for key in ['enemy_clicks','friendly_clicks','basic_requests','interaction_requests']:
            assert after['controls'][key] == before['controls'][key], key
    # The member shortcut opens management even when it was closed.
    focus(1); desktop.key('o'); wait(lambda: not states[1]()['party']['open'])
    open_member(1)
    manage = wait(lambda: next((b for b in states[1]()['party']['buttons'] if b['source']=='member' and b['action']=='manage'), None))
    desktop.click(windows[1], manage['point'], states[1]()['window'][2], 1)
    wait(lambda: states[1]()['party']['open'] and not states[1]()['party']['member_menu']['visible'])
    open_member(0)
    close = wait(lambda: next((b for b in states[0]()['party']['buttons'] if b['source']=='member' and b['action']=='menu-close'), None))
    desktop.click(windows[0], close['point'], states[0]()['window'][2], 1)
    wait(lambda: not states[0]()['party']['member_menu']['visible'])
    assert states[0]()['party']['open']
    open_member(0)
    focus(1)
    wait(lambda: not states[0]()['party']['member_menu']['visible'])
    open_member(0)
    click(0, 'promote', keys[1], source='member')
    wait(lambda: not states[0]()['party']['member_menu']['visible'])
    wait(lambda: all(view(i)['party']['leader'] == keys[1] for i in [0,1]), 5)
    member_frames()
    open_member(1)
    click(1, 'remove', keys[0], source='member'); solo()
    wait(lambda: all(not s()['party']['member_menu']['visible'] for s in states))
    join(); click(0, 'leave'); solo()
    join()
    focus(0); desktop.key('F10')
    wait(lambda: states[0]()['interface']['open'], 5)
    desktop.key('Escape')
    wait(lambda: not states[0]()['interface']['open'], 5)
    assert states[0]()['party']['open'], 'Interface Escape also closed Party'
    desktop.key('Escape')
    wait(lambda: not states[0]()['party']['open'], 5)
    desktop.key('o')
    wait(lambda: states[0]()['party']['open'], 5)
    capture(0, 'party-final.png')
    result = dict(passed=True, steps=steps, movement_metres=movement, final=[view(i) for i in [0,1]], progress_unchanged=True,
                  interface_then_party_escape=True, invitation_without_panel=True,
                  invitation_bounds=prompt['bounds'], invitation_drag_does_not_steer_or_interact=True, member_frames=member_proofs,
                  member_frame_drag_does_not_steer_or_interact=True, member_manage_shortcut=True, member_menus=menu_proofs,
                  member_menu_escape_outside_and_focus_dismissal=True, member_menu_actions=True)
    (folder/'party-proof.json').write_text(json.dumps(result, indent=2)+'\n')
    return result
