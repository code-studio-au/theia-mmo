"""Native party controls using only owned-window mouse and keyboard input."""
import json
import time
from qualify_world_input_os import wait, capture_ready


def flow(desktop, windows, states, folder):
    before = [dict(journal=s()['quest_journal'], bag=s()['inventory_bag']) for s in states]
    keys = [s()['actor'] for s in states]
    last = [0., 0.]
    steps = []
    def view(index):
        return states[index]()['party']['state']
    def focus(index):
        desktop.focus(windows[index])
        w, h, scale = states[index]()['window']
        desktop.move(windows[index], [w*.55, h*.42], scale)
        wait(lambda: states[index]()['cursor']['focused'])
    def click(index, action, target=None):
        focus(index)
        delay = .6 - (time.monotonic()-last[index])
        if delay > 0: time.sleep(delay)
        def button():
            p = states[index]()['party']
            if p['state']['pending']: return None
            return next((b for b in p['buttons'] if b['action'] == action and (target is None or b['target'] == target)), None)
        b = wait(button, 5)
        sequence = view(index)['sequence']
        desktop.click(windows[index], b['point'], states[index]()['window'][2], 1)
        last[index] = time.monotonic()
        wait(lambda: view(index)['sequence'] > sequence and not view(index)['pending'], 5)
        assert view(index)['refusal'] is None, view(index)
        steps.append(dict(player=index+1, action=action, sequence=view(index)['sequence']))
        for i, state in enumerate(states):
            current = state()
            assert dict(journal=current['quest_journal'], bag=current['inventory_bag']) == before[i]
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
    for i in [0, 1]:
        wait(lambda: states[i]().get('party', {}).get('state') is not None, 20)
        assert view(i)['party'] is None and view(i)['incoming'] is None
        focus(i); desktop.key('o')
        wait(lambda: states[i]()['party']['open'], 5)
    invite(); capture(1, 'party-invitation.png')
    click(1, 'decline'); solo()
    invite(); click(0, 'cancel'); solo()
    join(); capture(0, 'party-members.png')
    click(0, 'promote', keys[1])
    wait(lambda: all(view(i)['party']['leader'] == keys[1] for i in [0,1]), 5)
    click(1, 'remove', keys[0]); solo()
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
    result = dict(passed=True, steps=steps, final=[view(i) for i in [0,1]], progress_unchanged=True,
                  interface_then_party_escape=True)
    (folder/'party-proof.json').write_text(json.dumps(result, indent=2)+'\n')
    return result
