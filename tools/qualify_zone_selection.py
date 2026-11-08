#!/usr/bin/env python3
"""Native selection/framing checks for the 3440x1440 zone editor."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary, duplicate=False):
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    cases = []
    env = dict(os.environ, WGPU_BACKEND='vulkan',
               VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer')

    def case(name, arguments, empty=False):
        report = output / f'{name}.state.json'
        log_path = output / f'{name}.log'
        with log_path.open('w') as log:
            mailbox = output / f'{name}.mailbox'
            extra = ['--mailbox', str(mailbox)] if duplicate else []
            process = subprocess.Popen([str(binary), *arguments, *extra, '--report', str(report)],
                                       env=env, stdout=log, stderr=log)
            desktop = None
            try:
                window = wait(lambda: owned_window(process.pid), 45)
                desktop = Desktop()
                desktop.focus(window)

                def state():
                    assert process.poll() is None
                    return json.loads(report.read_text())

                wait(lambda: report.exists() and state()['viewport'] == [3440, 1440], 45)
                desktop.move(window, [1720, 720], 1)
                original = state()

                def key(name):
                    desktop.key(name)

                def chord(modifier, name):
                    desktop.key_state(modifier, True)
                    try:
                        key(name)
                    finally:
                        desktop.key_state(modifier, False)

                def button(action):
                    entry = next(b for b in state()['buttons'] if b['action'] == action)
                    desktop.move(window, entry['centre'], 1)
                    wait(lambda: any(b['action'] == action and b['hovered'] for b in state()['buttons']))
                    desktop.button(1, True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1, False)
                    wait(lambda: not state()['left_mouse_down'])

                def focus(target):
                    key('f')
                    wait(lambda: all(abs(a-b) < 0.001 for a,b in zip(state()['view']['target'], target)))

                def capture(label):
                    key('F12')
                    wait(lambda: capture_ready(report.with_suffix('.png')), 30)
                    report.with_suffix('.png').rename(output / f'{name}.{label}.png')

                def unchanged():
                    current = state()
                    assert current['document'] == original['document']
                    assert current['revision'] == original['revision']
                    assert current['history'] == original['history']
                    assert not current['unsaved_changes']

                seen_ids = {p['id'] for field in ['objects','paths','plants','markers'] for p in original['document'].get(field,[])}

                def copy_selected(collection, selection_field):
                    nonlocal original
                    before = state()
                    source = next(p for p in before['document'][collection] if p['id'] == before[selection_field])
                    chord('Control_L', 'd')
                    copied = wait(lambda: state() if state()['revision'] == before['revision'] + 1 else None)
                    new_id = copied[selection_field]
                    assert new_id not in seen_ids
                    seen_ids.add(new_id)
                    expected = json.loads(json.dumps(source))
                    expected['id'] = new_id
                    expected['name'] += ' copy'
                    def shifted(point):
                        return [point[0]+1., point[1], point[2]+1.]
                    if collection == 'objects':
                        expected['pose']['position'] = shifted(expected['pose']['position'])
                    elif collection == 'paths':
                        expected['points'] = list(map(shifted, expected['points']))
                        assert copied['path_point'] is None
                    else:
                        expected['position'] = shifted(expected['position'])
                    expected_document = json.loads(json.dumps(before['document']))
                    expected_document[collection].append(expected)
                    assert copied['document'] == expected_document and copied['unsaved_changes']
                    assert copied['history'] == [before['history'][0]+1, 0]
                    capture(f'copy-{collection}-{new_id}')
                    chord('Control_L', 'z')
                    wait(lambda: state()['revision'] == copied['revision']+1)
                    assert state()['document'] == before['document'] and not state()['unsaved_changes']
                    chord('Control_L', 'y')
                    wait(lambda: state()['revision'] == copied['revision']+2)
                    assert state()['document'] == copied['document']
                    chord('Control_L', 'z')
                    wait(lambda: state()['revision'] == copied['revision']+3)
                    assert state()['document'] == before['document'] and not state()['unsaved_changes']
                    original = state()
                    return new_id

                key('Tab')
                if empty:
                    wait(lambda: state()['status'] == 'No objects to select')
                    chord('Shift_L', 'Tab')
                    key('f')
                    wait(lambda: state()['status'] == 'Select a placement before focusing')
                    assert state()['selected'] is None and state()['view'] == original['view']
                    if duplicate:
                        chord('Control_L','d')
                        wait(lambda: state()['status'] == 'Select a placement or guide before duplicating')
                    unchanged()
                else:
                    wait(lambda: state()['selected'] == 1)
                    for expected in [2, 3, 1]:
                        key('Tab')
                        wait(lambda: state()['selected'] == expected)
                    chord('Shift_L', 'Tab')
                    wait(lambda: state()['selected'] == 3)
                    focus([0, -8, 0])
                    unchanged()
                    key('h')
                    wait(lambda: state()['revision'] == original['revision'] + 1)
                    assert abs(state()['document']['objects'][2]['pose']['position'][1] - .15) < 1e-9
                    chord('Control_L', 'z')
                    wait(lambda: state()['revision'] == original['revision'] + 2)
                    assert state()['document'] == original['document'] and not state()['unsaved_changes']
                    # Grounding itself is the only history change in this case.
                    original = state()
                    key('Tab')
                    wait(lambda: state()['selected'] == 1)
                    focus([400, 1.5, 400])
                    assert all(abs(a-b) < 2 for a,b in zip(state()['objects'][0]['screen'], [1720,720]))
                    key('Tab')
                    wait(lambda: state()['selected'] == 2)
                    focus([-350, 64, -350])
                    assert 300 < state()['view']['distance'] < 512
                    capture('large-object')
                    if duplicate:
                        copy_selected('objects','selected')
                    key('F4')
                    wait(lambda: state()['marker_mode'])
                    button('NextMarker')
                    focus([-300, 0, 300])
                    assert state()['view']['distance'] == 200
                    if duplicate:
                        copy_selected('markers','marker_selected')
                    key('F6')
                    wait(lambda: state()['vegetation_mode'])
                    button('NextPlant')
                    focus([350, 20, -300])
                    capture('plant')
                    if duplicate:
                        copy_selected('plants','plant_selected')
                    key('F3')
                    wait(lambda: state()['path_mode'])
                    button('NextPath')
                    focus([395, .03, -380])
                    key('Tab')
                    wait(lambda: state()['path_point'] == 0)
                    focus([380, .03, -380])
                    assert state()['view']['distance'] == 12
                    if duplicate:
                        copy_selected('paths','path_selected')
                        before = state()
                        bridge = json.loads((mailbox/'state.json').read_text())
                        request = dict(session=bridge['session'],request_id=1,
                            command=dict(command='apply',expected_revision=before['revision'],
                                edits=[dict(op='duplicate',id=1,new_id=5000,offset=[-2.,0.,-2.])]))
                        (mailbox/'agent.tmp').write_text(json.dumps(request))
                        (mailbox/'agent.tmp').replace(mailbox/'request.json')
                        wait(lambda: state()['revision'] == before['revision']+1)
                        receipt = wait(lambda: json.loads((mailbox/'receipt.json').read_text()) if (mailbox/'receipt.json').exists() else None)
                        assert receipt['accepted'] and state()['document']['objects'][-1]['id'] == 5000
                        chord('Control_L','z')
                        wait(lambda: state()['revision'] == before['revision']+2)
                        assert state()['document'] == before['document']
                        key('F1')
                        key('Tab')
                        wait(lambda: state()['selected'] == 1)
                        assert copy_selected('objects','selected') == 5001
                        key('F3')
                        wait(lambda: state()['path_mode'])
                        button('NewPath(Road)')
                        wait(lambda: state()['path_draft_points'] == 0)
                        revision = state()['revision']
                        chord('Control_L','d')
                        wait(lambda: state()['status'] == 'Finish or cancel the guide draft before duplicating')
                        assert state()['path_draft_points'] == 0 and state()['revision'] == revision
                        key('Escape')
                        wait(lambda: state()['path_draft_points'] is None)
                    unchanged()
                    view = state()['view']
                    key('F5')
                    wait(lambda: state()['play_feet'] is not None)
                    key('f')
                    key('Tab')
                    if duplicate:
                        chord('Control_L','d')
                        wait(lambda: state()['status'] == 'Leave playtest before editing')
                    assert state()['view'] == view
                    unchanged()
                    key('F5')
                    wait(lambda: state()['play_feet'] is None)
                capture('final')
                final = state()
                subprocess.run(['wmctrl', '-ic', hex(window)], check=True)
                process.wait(timeout=20)
                assert process.returncode == 0
            finally:
                if desktop:
                    desktop.close()
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=15)
        log_text = log_path.read_text()
        assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in log_text
        diagnostics = check_gpu_diagnostics(log_text, allow_surface_resize=False)
        cases.append(dict(name=name, gpu=final['gpu'], viewport=final['viewport'],
                          validation_layer_active=True, allowed_resize_diagnostics=diagnostics))

    source = output / 'zone.json'
    case('local', ['--commands', 'content/zones/selection.commands.jsonl', '--output', str(source)])
    far = json.loads(source.read_text())
    offset = [128000000000, 0, -128000000000]
    def shift(point):
        return [a+b for a,b in zip(point, offset)]
    far['origin'] = shift(far['origin'])
    far['spawn'] = shift(far['spawn'])
    for obj in far['objects']:
        obj['pose']['position'] = shift(obj['pose']['position'])
    for item in far['plants'] + far['markers']:
        item['position'] = shift(item['position'])
    for path in far['paths']:
        path['points'] = list(map(shift, path['points']))
    far_path = output / 'far.zone.json'
    far_path.write_text(json.dumps(far))
    case('far', ['--zone', str(far_path)])
    case('empty', ['--output', str(output / 'empty.zone.json')], empty=True)
    summary = dict(cases=cases,
                   checks=['forward/reverse object wrap and buried-object recovery with H/undo',
                           'object/plant/marker/guide/point framing without model, history or saved-state mutation',
                           'large-object framing beyond the old camera range and safe empty/playtest input',
                           'checked global-to-local framing at signed far origin'],
                   scope='Linux owned OS input, standard Bevy at 3440x1440; offline editor, no Windows or multiplayer claim')
    summary['duplication'] = duplicate
    if duplicate:
        summary['checks'] += ['Ctrl-D copies all four authoring kinds with exact undo/redo and whole-guide behavior',
                              'agent duplicate plus visual undo preserves monotonic shared ID allocation',
                              'pending guide draft and playtest refuse copying without data loss']
    (output / 'qualification.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--duplicate', action='store_true')
    args = parser.parse_args()
    run(args.output, args.binary, args.duplicate)
