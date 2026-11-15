"""Starter coins, recovery weapons, NPC quest delivery and fixed HUD via ordinary OS input."""
import json
import math
import time
from qualify_world_input_os import wait, capture_ready, owned_window

QUEST = 'theia.dev.quest-meet-ranger'
GUIDE = 'theia.dev.steward'
STEWARD = 'theia.dev.ranger'
MERCHANT = 'theia.dev.weapon-merchant'


def flow(desktop, window, state, folder, offers, resumed=False, expected_bag=None):
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
        wait(lambda: state()['dialogue_open'] and state()['dialogue_ui']['open'])
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

    contacts = wait(lambda: state()['contacts'] if len(state().get('contacts', [])) == 5 else None)
    assert {c['npc'] for c in contacts} == {GUIDE, STEWARD, MERCHANT, 'theia.dev.guide', 'theia.dev.villager'}
    assert len({c['entity'] for c in contacts}) == 5
    from quest_vendor_os import totals

    def click_visible(get_point):
        for _ in range(100):
            point = wait(get_point)
            pane = state()['inventory_layout']['pane']
            low, high = pane['point'][1]-pane['size'][1]/2+24, pane['point'][1]+pane['size'][1]/2-24
            if low <= point[1] <= high:
                desktop.click(window, point, scale, 1)
                return
            desktop.move(window, pane['point'], scale)
            desktop.scroll(-2 if point[1] > high else 2)
            time.sleep(.12)
        raise AssertionError('Inventory control never entered its clipped viewport')

    def fixed_hud():
        close()
        before=state()
        assert before['minimap']['visible'] and before['minimap']['bounds']
        desktop.key('i')
        opened=wait(lambda: s if (s:=state())['inventory_open'] and s['inventory_layout'] else None)
        time.sleep(.3)
        opened=state()
        assert opened['minimap']['visible']
        assert opened['minimap']['bounds']==before['minimap']['bounds']
        assert opened['quest_journal']['tracker']['point']==before['quest_journal']['tracker']['point']
        assert opened['quest_journal']['tracker']['size']==before['quest_journal']['tracker']['size']
        desktop.move(window, opened['inventory_layout']['slots'][0], scale)
        tooltip = wait(lambda: state()['inventory_layout']['tooltip'] if state()['inventory_layout']['tooltip']['visible'] else None)
        assert 'Training Stone' in tooltip['text']
        bounds = tooltip['bounds']; slot = opened['inventory_layout']['slots'][0]
        assert bounds['point'][0] + bounds['size'][0]/2 < slot[0], 'Tooltip must sit beside the hovered bag slot'
        assert 0 <= bounds['point'][1] - bounds['size'][1]/2 <= state()['window'][1]
        capture('bag-coins-and-tooltip.png')
        desktop.move(window, [state()['window'][0]/2, state()['window'][1]/2], scale)
        capture('fixed-hud-bag-open.png')
        return dict(minimap=opened['minimap']['bounds'],tracker=opened['quest_journal']['tracker'])

    def destroy(suffix):
        before=state()['inventory_bag']
        item='theia.dev.training-'+suffix
        slot=next(i for i,s in enumerate(before['slots']) if s and s['item']==item)
        click_visible(lambda: state()['inventory_layout']['slots'][slot])
        wait(lambda: state()['inventory_ui']['selected']==slot)
        click_visible(lambda: state()['inventory_layout']['destroy'])
        wait(lambda: state()['inventory_ui']['destroy_confirmation'] is not None)
        click_visible(lambda: state()['inventory_layout']['destroy'])
        wait(lambda: state()['inventory_bag']['revision']==before['revision']+1 and not state()['inventory_ui']['pending'])
        assert totals(state()['inventory_bag']).get(item,0)==0
        assert state()['inventory_bag']['copper']==before['copper']

    trades=[]
    def trade(offer_id):
        ordered=sorted(offers,key=lambda o:o['id'])
        index=next(i for i,o in enumerate(ordered) if o['id']==offer_id)
        offer=ordered[index]
        before=state()['inventory_bag']
        counts=totals(before); copper=before['copper']
        for side,sign in [('cost',-1),('grant',1)]:
            leg=offer[side]
            if 'copper' in leg: copper+=sign*leg['copper']
            else: counts[leg['item']]=counts.get(leg['item'],0)+sign*leg['count']
        counts={k:v for k,v in counts.items() if v}
        applied=state()['inventory_ui']['applied']
        click_visible(lambda: next((b['point'] for b in state()['inventory_layout']['offers'] if b['index']==index),None))
        after=wait(lambda: s['inventory_bag'] if (s:=state())['inventory_bag']['revision']==before['revision']+1 and s['inventory_ui']['applied']==applied+1 and not s['inventory_ui']['pending'] else None)
        assert totals(after)==counts and after['copper']==copper
        assert after['main_hand']==before['main_hand']
        assert state()['inventory_ui']['rejected']==0
        trades.append(dict(offer=offer,before=before,bag=after))
        capture('trade-'+str(len(trades))+'.png')

    hud=fixed_hud()
    reward_proof = None
    if resumed:
        wait(lambda: quest() and quest()['status'] == 'Completed')
        assert journal()['revision'] == 3 and journal()['xp'] == 25
        assert journal()['ui']['submitted'] == 0
        assert state()['inventory_bag']==expected_bag
        assert state()['inventory_ui']['vendor_clicks']==0
        speak(STEWARD)
        assert not state()['inventory_open']
        assert all(b['quest'] != QUEST for b in journal()['buttons'])
        capture('economy-restored.png')
    else:
        assert state()['inventory_bag']['copper']==52550
        destroy('sword')
        destroy('wand')
        capture('weapons-deleted.png')
        for npc in ['theia.dev.guide','theia.dev.villager']:
            speak(npc)
            assert not state()['inventory_open'], 'Non-vendor opened the bag'
        speak(GUIDE)
        assert not state()['inventory_open']
        capture('steward-offer.png')
        action('Active')
        capture('guide-accepted.png')
        speak(STEWARD)
        wait(lambda: not state()['inventory_ui']['vendor_open'])
        assert all(b['quest'] != QUEST for b in journal()['buttons'])
        wanted = wait(lambda: next((c['point'] for c in journal()['choices'] if c['choice'] == 'theia.dev.ranger.greeting'), None))
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
        assert state()['inventory_bag']==before
        capture('ranger-completed.png')
        assert journal()['revision'] == 3 and journal()['xp'] == 25
    speak(MERCHANT)
    wait(lambda: state()['inventory_open'] and state()['inventory_ui']['vendor_open'])
    if not resumed:
        for suffix in ['buy-sword','buy-wand','sell-training-stone','sell-training-sword','buy-sword']:
            trade('theia.dev.weapons.'+suffix)
        assert state()['inventory_bag']['copper']==48805
    else:
        assert state()['inventory_bag']==expected_bag
    assert state()['minimap']['visible']
    result = dict(passed=True,resumed=resumed,contacts=contacts,revision=journal()['revision'],xp=journal()['xp'],
                  quest=quest(),ui=journal()['ui'],hud=hud,
                  vendor=dict(passed=True,bag=state()['inventory_bag'],trades=trades,no_repeat_purchase=resumed),
                  input='Ordinary owned-window mouse and keyboard; no currency or item injection')
    (folder/'economy-proof.json').write_text(json.dumps(result,indent=2)+'\n')
    return result
