"""Directional-animation checks inside the owned two-account native fixture."""
import json
import math
import time
from qualify_world_input_os import wait, capture_ready


def checks(desktop, window_a, window_b, state_a, state_b, output, face):
    owner, observer = output/'owner', output/'observer'
    actor = state_a()['actor']
    proof = {}

    def row(state):
        return next((r for r in state()['player_locomotion']['actors'] if r['actor'] == actor), None)

    def captured(state, group, name, local):
        return any(s['local'] == local and s['clip'] == name for s in state()[group]['samples'])

    def capture(name):
        for folder in [owner, observer]:
            (folder/'capture.request').write_text(name+'.png\n')
        for folder in [owner, observer]:
            wait(lambda: capture_ready(folder/'input-os.png'), 5)
            (folder/'input-os.png').rename(folder/(name+'.png'))

    # Move both accounts away from the wolf using ordinary input. The copied
    # animation fixture limits passive perception to 1 m; damage/speeds stay real.
    for window, state in [(window_b, state_b), (window_a, state_a)]:
        desktop.focus(window)
        time.sleep(.2)
        desktop.key_state('q', True)
        try:
            deadline = time.monotonic()+2.8
            while time.monotonic() < deadline:
                desktop.owned_focus()
                assert state()['combat']['player_health'] > 0
                time.sleep(.1)
        finally:
            desktop.key_state('q', False)
        wait(lambda: state()['movement']['direction'] == [0., 0.])
    desktop.focus(window_b)
    width, height, scale = state_b()['window']
    desktop.move(window_b, [width*.55, height*.42], scale)
    for _ in range(30):
        if state_b()['camera_distance'] >= 14:
            break
        desktop.button(5, True)
        desktop.button(5, False)
    assert state_b()['camera_distance'] >= 14
    desktop.focus(window_a)
    scale = state_a()['window'][2]
    desktop.key('i')
    wait(lambda: state_a()['inventory_open'])
    desktop.click(window_a, wait(lambda: state_a()['sword_slot_screen']), scale, 1)
    desktop.click(window_a, wait(lambda: state_a()['equip_screen']), scale, 1)
    wait(lambda: state_a()['sword_equipped'])
    desktop.key('Escape')
    wait(lambda: not state_a()['inventory_open'])
    wait(lambda: row(state_a) and row(state_a)['clip'] == 'SwordIdle'
         and row(state_b) and row(state_b)['clip'] == 'SwordIdle')
    heading = state_a()['classic_facing']
    directions = [('', ['w']), ('Back', ['s']), ('Right', ['e']), ('Left', ['q']),
                  ('ForwardRight', ['w', 'e']), ('BackLeft', ['s', 'q']),
                  ('ForwardLeft', ['w', 'q']), ('BackRight', ['s', 'e'])]
    measurements = {}
    for prefix, running in [('Walk', False), ('Run', True)]:
        if state_a()['movement']['sprint'] != running:
            desktop.key('KP_Divide')
            wait(lambda: state_a()['movement']['sprint'] == running)
        for suffix, keys in directions:
            name = prefix+suffix
            try:
                for key in keys:
                    desktop.key_state(key, True)

                def ready():
                    desktop.owned_focus()
                    a, b = row(state_a), row(state_b)
                    return (a, b) if (a and b and a['clip'] == b['clip'] == name
                                      and a['grip'] and b['grip']
                                      and captured(state_a, 'grip', name, True)
                                      and captured(state_b, 'grip', name, False)) else None

                a, b = wait(ready, 4)
                assert state_a()['classic_facing'] == heading, 'Animation changed manual facing'
                assert state_a()['combat']['player_health'] == 100, 'Animation fixture unexpectedly engaged'
                point = next((p['screen'] for p in state_b()['remote_player_screens'] if p['actor'] == actor), None)
                assert point and 0 < point[0] < width and 0 < point[1] < height, (name, point)
                if name in ['WalkBack', 'WalkLeft', 'RunRight', 'RunBackRight']:
                    capture(name)
                measurements[name] = dict(local=a, remote=b, remote_in_view=True)
                (output/'directional-partial.json').write_text(json.dumps(measurements, indent=2)+'\n')
            finally:
                for key in reversed(keys):
                    desktop.key_state(key, False)
            wait(lambda: state_a()['movement']['direction'] == [0., 0.])
            time.sleep(.25)
    proof['directions'] = measurements
    before_a = state_a()['player_locomotion']['metrics']['phase_preservations']
    before_b = state_b()['player_locomotion']['metrics']['phase_preservations']
    try:
        desktop.key_state('w', True)
        wait(lambda: row(state_a)['clip'] == row(state_b)['clip'] == 'Run')
        desktop.key_state('e', True)
        wait(lambda: row(state_a)['clip'] == row(state_b)['clip'] == 'RunForwardRight')
        desktop.key_state('w', False)
        wait(lambda: row(state_a)['clip'] == row(state_b)['clip'] == 'RunRight')
    finally:
        desktop.key_state('w', False)
        desktop.key_state('e', False)
    wait(lambda: state_a()['movement']['direction'] == [0., 0.])
    assert state_a()['player_locomotion']['metrics']['phase_preservations'] > before_a
    assert state_b()['player_locomotion']['metrics']['phase_preservations'] > before_b
    proof['continuous_direction_changes_observed'] = True
    # Jump remains above directional movement and retains authored closed digits.
    try:
        desktop.key_state('w', True)
        desktop.key_state('space', True)
        wait(lambda: row(state_a)['clip'] == row(state_b)['clip'] == 'Jump', 2)
        wait(lambda: captured(state_a, 'grip', 'Jump', True)
             and captured(state_b, 'grip', 'Jump', False), 2)
    finally:
        desktop.key_state('space', False)
        desktop.key_state('w', False)
    wait(lambda: row(state_a)['clip'] == row(state_b)['clip'] == 'SwordIdle', 4)
    proof['jump_precedes_directional_locomotion'] = True
    # Approach/provoke through actual input. No telemetry command grants attacks,
    # damage, position, facing, equipment or health.
    for _ in range(50):
        target = state_a()['enemy_global']
        face(desktop, window_a, state_a, target)
        feet = state_a()['dodge']['authority_feet']
        distance = math.hypot(target[0]-feet[0], target[2]-feet[2])
        if distance < 2.4:
            break
        desktop.key_state('w', True)
        try:
            time.sleep(min(.18, max(.01, (distance-2.2)/7)))
        finally:
            desktop.key_state('w', False)
        time.sleep(.15)
    else:
        raise AssertionError('Directional fixture did not approach the wolf')
    face(desktop, window_a, state_a, state_a()['enemy_global'])
    before = state_a()['controls']['basic_requests']
    for dx, dy in [(20, -25), (-20, -25), (0, -30), (0, 0)]:
        point = wait(lambda: state_a()['enemy_screen'])
        desktop.click(window_a, [point[0]+dx, point[1]+dy], scale, 3)
        if state_a()['controls']['basic_requests'] > before:
            break
    assert state_a()['controls']['basic_requests'] > before
    try:
        desktop.key_state('s', True)
        wait(lambda: captured(state_a, 'attack_animation', 'RunBack', True)
             and captured(state_b, 'attack_animation', 'RunBack', False), 7)
        capture('RunBackAttack')
    finally:
        desktop.key_state('s', False)
        desktop.key('BackSpace')
    wait(lambda: state_a()['selected_hostile'] is None)
    proof['sword_attack_preserves_directional_lower_body'] = True
    proof.update(passed=True, owner_actor=actor, source='real OS input; actual versioned two-account OIDC/PostgreSQL/native gameplay',
                 fixture='Copied 1-m passive perception; original damage, movement, weapon outcomes and life timings')
    (output/'directional-proof.json').write_text(json.dumps(proof, indent=2)+'\n')
    return proof
