"""Two authored contacts through ordinary owned-window input and committed UI."""
import json
import math
import time
from qualify_world_input_os import wait, capture_ready, owned_window

QUEST = 'theia.dev.quest-steward-delivery'
GUIDE = 'theia.dev.guide'
STEWARD = 'theia.dev.steward'


def flow(desktop, window, state, folder, resumed=False):
    assert owned_window(state()['pid']) == window
    scale = state()['window'][2]
    desktop.focus(window)
    wait(lambda: state()['cursor']['focused'])

    def journal():
        return state()['quest_journal']

    def quest():
        return next((q for q in journal()['quests'] if q['id'] == QUEST), None)

    def contact(npc):
        return next(c for c in state()['contacts'] if c['npc'] == npc)

    def capture(name):
        (folder/'capture.request').write_text(name+'\n')
        wait(lambda: capture_ready(folder/'input-os.png'))
        (folder/'input-os.png').rename(folder/name)

    def close():
        if state()['dialogue_open']:
            desktop.key('Escape')
            wait(lambda: not state()['dialogue_open'] and not state().get('dialogue_pending', False)
                 and not state().get('dialogue_ui', {}).get('pending', False))
        if state()['inventory_open']:
            desktop.key('Escape')
            wait(lambda: not state()['inventory_open'])

    def approach(npc):
        close()
        for _ in range(35):
            target = contact(npc)['global']
            player = state()['dodge']['authority_feet']
            yaw = math.atan2(-(target[0]-player[0]), -(target[2]-player[2]))
            for _ in range(8):
                heading = state()['classic_facing']*math.tau/65536
                if abs(math.atan2(math.sin(yaw-heading), math.cos(yaw-heading))) < .07:
                    break
                width, height, _ = state()['window']
                desktop.move(window, [width*.55, height*.42], scale)
                desktop.button(3, True)
                try:
                    angle = yaw-state()['camera_angles'][0]
                    angle = math.atan2(math.sin(angle), math.cos(angle))
                    desktop.look(round(max(-400, min(400, -angle/.004))), 0)
                finally:
                    desktop.button(3, False)
                time.sleep(.2)
            else:
                raise AssertionError('Ordinary camera facing did not settle')
            distance = math.hypot(target[0]-player[0], target[2]-player[2])
            if distance < 2.1:
                return
            desktop.hold_key('w', min(.35, max(.025, (distance-1.9)/5)))
            time.sleep(.15)
        raise AssertionError('Ordinary movement did not reach the contact')

    def speak(npc):
        approach(npc)
        desktop.click(window, wait(lambda: contact(npc)['screen']), scale, 3)
        wait(lambda: state()['dialogue_open'] and journal()['choices'])
        wait(lambda: not state().get('dialogue_pending', False))

    def action(expected):
        applied = journal()['ui']['applied']
        def point():
            return next((b['point'] for b in journal()['buttons'] if b['quest'] == QUEST), None)
        target = wait(point)
        for _ in range(24):
            pane = journal().get('offer_scroll')
            if not pane or pane['point'][1]-pane['size'][1]/2+5 <= target[1] <= pane['point'][1]+pane['size'][1]/2-5:
                break
            desktop.move(window, pane['point'], scale)
            desktop.scroll(-1 if target[1] > pane['point'][1] else 1)
            time.sleep(.15)
            target = wait(point)
        desktop.click(window, target, scale, 1)
        wait(lambda: quest() and quest()['status'] == expected and journal()['ui']['applied'] == applied+1
             and not journal()['ui']['pending'])

    contacts = wait(lambda: state()['contacts'] if len(state().get('contacts', [])) == 2 else None)
    assert {c['npc'] for c in contacts} == {GUIDE, STEWARD}
    assert len({c['entity'] for c in contacts}) == 2
    reward_proof = None
    if resumed:
        wait(lambda: quest() and quest()['status'] == 'Completed')
        assert journal()['revision'] == 3 and journal()['xp'] == 25
        assert journal()['ui']['submitted'] == 0
        speak(GUIDE)
        assert all(b['quest'] != QUEST for b in journal()['buttons'])
        capture('contacts-restored.png')
    else:
        speak(GUIDE)
        capture('guide-offer.png')
        action('Active')
        capture('guide-accepted.png')
        speak(STEWARD)
        wait(lambda: not state()['inventory_ui']['vendor_open'])
        assert all(b['quest'] != QUEST for b in journal()['buttons'])
        wanted = wait(lambda: next((c['point'] for c in journal()['choices'] if c['choice'] == 'theia.dev.steward.report'), None))
        desktop.click(window, wanted, scale, 1)
        wait(lambda: quest()['status'] == 'Ready')
        capture('steward-ready.png')
        # Return to the wrong contact while ready: no turn-in or reward button.
        speak(GUIDE)
        assert all(b['quest'] != QUEST for b in journal()['buttons'])
        capture('guide-cannot-turn-in.png')
        speak(STEWARD)
        assert not state()['inventory_ui']['vendor_open']
        before = state()['inventory_bag']
        action('Completed')
        wait(lambda: state()['inventory_bag']['revision'] == before['revision']+1)
        after = state()['inventory_bag']
        from quest_vendor_os import totals
        expected = totals(before)
        expected['theia.dev.training-token'] = expected.get('theia.dev.training-token', 0)+1
        assert totals(after) == expected
        assert after['main_hand'] == before['main_hand'] and after['catalog'] == before['catalog']
        reward_proof = dict(before=before, after=after, grant=dict(item='theia.dev.training-token', count=1))
        capture('steward-completed.png')
        assert journal()['revision'] == 3 and journal()['xp'] == 25
        speak(GUIDE)
    wait(lambda: state()['inventory_open'] and state()['inventory_ui']['vendor_open'])
    result = dict(passed=True, resumed=resumed, contacts=contacts, revision=journal()['revision'], xp=journal()['xp'],
                  quest=quest(), ui=journal()['ui'], reward=reward_proof, input='Ordinary OS mouse and keyboard; exact NPC mesh picking and painted quest buttons')
    (folder/'contacts-proof.json').write_text(json.dumps(result, indent=2)+'\n')
    return result
