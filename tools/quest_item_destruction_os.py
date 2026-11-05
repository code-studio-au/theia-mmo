"""PID-bound OS confirmation/destruction checks on an owned native character."""
import json
import time
from qualify_world_input_os import capture_ready, wait

STONE = 'theia.dev.training-stone'


def flow(desktop, window, state, folder, resumed=False):
    scale = state()['window'][2]

    def bag():
        return state()['inventory_bag']

    def count():
        return sum(slot['count'] for slot in bag()['slots'] if slot and slot['item'] == STONE)

    def journal():
        value = state()['quest_journal']
        return {key: value[key] for key in ['revision', 'quests', 'xp']}

    def ui():
        return state()['inventory_ui']

    def click(name, index=None):
        point = wait(lambda: state()['inventory_layout'][name])
        if index is not None:
            point = point[index]
        assert point is not None
        width, height = state()['window'][:2]
        assert 0 < point[0] < width and 0 < point[1] < height, (name, point)
        desktop.click(window, point, scale, 1)

    def capture(name):
        (folder/'capture.request').write_text(name+'\n')
        wait(lambda: capture_ready(folder/'input-os.png'))
        (folder/'input-os.png').rename(folder/name)

    if state()['dialogue_open']:
        desktop.key('Escape')
        wait(lambda: not state()['dialogue_open'] and not state().get('dialogue_pending', False) and not any(state().get('dialogue_ui', {}).get(key, False) for key in ['open', 'pending', 'close_when_open']))
    if not state()['inventory_open']:
        desktop.key('i')
        wait(lambda: state()['inventory_open'])
    wait(lambda: state()['inventory_layout'])
    initial_revision = bag()['revision']
    if resumed:
        assert count() == 27, bag()
        assert ui()['destroy_confirmation'] is None
        capture('resumed-destroyed-item.png')
        desktop.key('i')
        wait(lambda: not state()['inventory_open'])
        return dict(passed=True, saved_count=27, revision=initial_revision, confirmation_not_restored=True)
    assert count() == 28, bag()
    assert bag()['slots'][15] is None
    original_journal = journal()
    original_hand = state()['main_hand_item']
    original_submitted = ui()['submitted']
    source = next(i for i, slot in enumerate(bag()['slots']) if slot and slot['item'] == STONE)
    click('slots', source)
    wait(lambda: ui()['selected'] == source)
    click('split')
    wait(lambda: ui()['split'])
    click('slots', 15)
    wait(lambda: bag()['revision'] == initial_revision+1 and not ui()['pending'])
    assert count() == 28 and bag()['slots'][15]['count'] == 1
    assert journal() == original_journal
    click('slots', 15)
    wait(lambda: ui()['selected'] == 15)
    click('destroy')
    wait(lambda: ui()['destroy_confirmation'] == [15, initial_revision+1])
    time.sleep(.3)
    assert count() == 28 and ui()['submitted'] == original_submitted+1
    capture('destroy-confirmation.png')
    click('cancel')
    wait(lambda: ui()['destroy_confirmation'] is None and ui()['selected'] is None)
    assert count() == 28 and bag()['revision'] == initial_revision+1
    click('slots', 15)
    wait(lambda: ui()['selected'] == 15)
    click('destroy')
    wait(lambda: ui()['destroy_confirmation'] is not None)
    # Closing a panel cancels the old confirmation.
    desktop.key('i')
    wait(lambda: not state()['inventory_open'] and ui()['destroy_confirmation'] is None)
    desktop.key('i')
    wait(lambda: state()['inventory_open'])
    click('destroy')
    wait(lambda: ui()['destroy_confirmation'] is not None)
    assert count() == 28, 'Reopening must require a fresh confirmation'
    click('destroy')
    wait(lambda: bag()['revision'] == initial_revision+2 and not ui()['pending'])
    assert count() == 27 and bag()['slots'][15] is None
    assert ui()['submitted'] == original_submitted+2
    assert state()['main_hand_item'] == original_hand
    assert journal() == original_journal
    capture('destroyed-item.png')
    proof = dict(passed=True, normal_os_input=True, split_preserved_total=28,
                 confirmation_before_mutation=True, cancel_kept_item=True,
                 closing_cancelled_confirmation=True, destroyed_count=27,
                 revision_before=initial_revision, revision_after=bag()['revision'],
                 equipment_and_historical_quests_unchanged=True)
    (folder/'item-destruction.json').write_text(json.dumps(proof, indent=2)+'\n')
    desktop.key('i')
    wait(lambda: not state()['inventory_open'])
    return proof
