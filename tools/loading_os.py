"""Observe genuine entry and hold ordinary owned-window keys across readiness."""
import json
import math
import time
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def entry(state, folder):
    first = state()
    assert first['loading']['phase'] == 'Loading', 'Entry finished before input qualification attached'
    window = wait(lambda: owned_window(first['pid']), 15)
    desktop = Desktop()
    samples = []
    held = []
    try:
        desktop.focus(window)
        for key in ['w', '1', 'o']:
            if state()['loading']['phase'] != 'Loading':
                break
            desktop.key_state(key, True)
            held.append(key)
        assert held, 'No early loading input sampled'
        def sample():
            value = state()
            assert value['loading']['phase'] != 'Failed', value['loading']
            assert value['movement']['direction'] == [0, 0], value['movement']
            if value.get('dodge'):
                assert value['dodge']['input']['axes'] == [0, 0], value['dodge']
            assert not value['inventory_open'] and not value['dialogue_open']
            assert value.get('active_cast') is None and value.get('selected_hostile') is None
            assert not value['party']['open'], value['party']
            samples.append(dict(phase=value['loading']['phase'], position=value.get('rendered_body'),
                                progress_percent=value['loading']['progress_percent'],
                                input=value.get('dodge', {}).get('input') if value.get('dodge') else None))
            return value if value['loading']['phase'] == 'Playing' else None
        ready = wait(sample, 90)
        # Exercise OS repeat after the overlay closes while the key remains down.
        until = time.monotonic() + 1.2
        while time.monotonic() < until:
            sample(); time.sleep(.05)
        assert ready['loading']['entered_with'] and all(ready['loading']['entered_with'].values())
        assert ready['loading']['progress_percent'] == 100
        assert all(0 <= s['progress_percent'] <= 100 for s in samples)
        assert ready['loading']['blocked_key_frames'] > 0
        positions = [s['position'] for s in samples if s['position'] is not None]
        assert positions and all(math.dist([p[0], p[2]], [positions[0][0], positions[0][2]]) < .01 for p in positions)
        wait(lambda: capture_ready(folder/'loading-screen.png'), 10)
        loading = json.loads((folder/'loading-screen.json').read_text())
        width, height, _ = ready['window']
        for name in ['art', 'status', 'cancel']:
            x,y,w,h = loading['geometry'][name]
            assert w > 0 and h > 0 and x >= 0 and y >= 0 and x+w <= width+1 and y+h <= height+1, (name, loading)
        proof = dict(passed=True, held=held, initial=loading, entered=ready['loading'], samples=samples)
        (folder/'loading-entry-proof.json').write_text(json.dumps(proof, indent=2))
        return proof
    finally:
        for key in held: desktop.key_state(key, False)
        desktop.close()
