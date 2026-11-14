"""PID-bound guide trade through the ordinary vendor button and durable bag view."""
import json
from qualify_world_input_os import wait, capture_ready, owned_window


def totals(bag):
    result = {}
    for stack in bag['slots']:
        if stack:
            result[stack['item']] = result.get(stack['item'], 0) + stack['count']
    return result


def flow(desktop, window, state, folder, offer, resumed=False, expected_bag=None):
    assert owned_window(state()['pid']) == window
    desktop.focus(window)
    wait(lambda: state()['cursor']['focused'] and state()['inventory_open']
         and state()['inventory_ui']['vendor_open'] and not state()['inventory_ui']['pending'])

    def capture(name):
        (folder/'capture.request').write_text(name+'\n')
        wait(lambda: capture_ready(folder/'input-os.png'))
        (folder/'input-os.png').rename(folder/name)

    if resumed:
        assert state()['inventory_bag'] == expected_bag
        assert state()['inventory_ui']['vendor_clicks'] == 0
        capture('vendor-restored.png')
        result = dict(passed=True, resumed=True, bag=expected_bag, no_repeat_purchase=True)
        (folder/'vendor-proof.json').write_text(json.dumps(result, indent=2)+'\n')
        return result

    before = state()['inventory_bag']
    counts = totals(before)
    cost, grant = offer['cost'], offer['grant']
    assert counts.get(cost['item'], 0) >= cost['count']
    counts[cost['item']] -= cost['count']
    counts[grant['item']] = counts.get(grant['item'], 0) + grant['count']
    counts = {item: count for item, count in counts.items() if count}
    capture('vendor-before.png')
    applied = state()['inventory_ui']['applied']
    point = wait(lambda: next((button['point'] for button in
        (state().get('inventory_layout') or {}).get('offers', []) if button['index']==0), None))
    width, height, scale = state()['window']
    assert 0 < point[0] < width and 0 < point[1] < height
    desktop.click(window, point, scale, 1)
    observed = wait(lambda: value if (value := state())['inventory_bag']['revision']==before['revision']+1
        and value['inventory_ui']['applied']==applied+1 and not value['inventory_ui']['pending'] else None)
    after = observed['inventory_bag']
    assert totals(after)==counts
    assert after['main_hand']==before['main_hand'] and after['catalog']==before['catalog']
    assert observed['inventory_ui']['vendor_clicks']==1 and observed['inventory_ui']['rejected']==0
    capture('vendor-purchased.png')
    result = dict(passed=True, resumed=False, offer=offer, before=before, bag=after,
                  ui=observed['inventory_ui'], input='ordinary OS left click on the painted offer')
    (folder/'vendor-proof.json').write_text(json.dumps(result, indent=2)+'\n')
    return result
