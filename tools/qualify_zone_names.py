#!/usr/bin/env python3
"""Exercise shared naming with real Bevy text input at 3440x1440 only."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready

ROOT = Path(__file__).resolve().parents[1]


def run(output, binary):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    seed = output/'seed.json'
    subprocess.run([str(binary), '--headless', '--commands', str(ROOT/'content/zones/selection.commands.jsonl'),
                    '--output', str(seed)], check=True, capture_output=True)
    base = json.loads(seed.read_text())
    cases = []
    checks = []
    input_notes = []
    env = dict(os.environ, WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
               VK_LOADER_DEBUG='layer')

    def case(name, source, verify):
        report = output/f'{name}.state.json'
        logfile = output/f'{name}.log'
        mailbox = output/f'{name}.mailbox'
        with logfile.open('w') as log:
            process = subprocess.Popen([str(binary), '--zone', str(source), '--mailbox', str(mailbox),
                                        '--report', str(report)], env=env, stdout=log, stderr=log)
            desktop = None
            try:
                window = wait(lambda: owned_window(process.pid), 45)
                desktop = Desktop()
                desktop.focus(window)

                def state():
                    assert process.poll() is None, 'Editor exited'
                    return json.loads(report.read_text())

                wait(lambda: report.exists() and state()['viewport'] == [3440, 1440], 45)
                desktop.move(window, [1737,731], 1)
                desktop.move(window, [1720,720], 1)
                key = desktop.key

                def chord(modifier, name):
                    field = 'control_down' if modifier == 'Control_L' else 'shift_down'
                    # A modifier alone cannot edit content. Re-press only a missed
                    # modifier, before sending the actual command; never retry edits.
                    for attempt in range(3):
                        desktop.key_state(modifier, True)
                        try:
                            wait(lambda: state()[field], 1)
                            break
                        except TimeoutError:
                            desktop.key_state(modifier, False)
                            input_notes.append(dict(case=name, modifier=modifier, missed_attempt=attempt+1))
                    else:
                        raise AssertionError('Owned modifier was not delivered to Bevy')
                    try:
                        key(name)
                        elapsed = state()['input_elapsed']
                        wait(lambda: state()['input_elapsed'] > elapsed)
                    finally:
                        desktop.key_state(modifier, False)
                    wait(lambda: not state()[field])

                def type_text(text):
                    for character in text:
                        if character == ' ':
                            key('space')
                        elif character.isupper():
                            chord('Shift_L', character.lower())
                        else:
                            key(character)

                def replace_text(text):
                    chord('Control_L', 'a')
                    wait(lambda: state()['name_selection'] == [0,len(state()['name_edit'].encode())])
                    type_text(text)
                    wait(lambda: state()['name_edit'] == text)

                def click(entry):
                    desktop.move(window, entry['centre'], 1)
                    desktop.button(1, True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1, False)
                    wait(lambda: not state()['left_mouse_down'])

                def select(target, field, identifier):
                    entry = next(b for b in state()['content_list']['controls'] if b['target'] == target)
                    desktop.move(window, entry['centre'], 1)
                    wait(lambda: any(b['target'] == target and b['hovered'] for b in state()['content_list']['controls']))
                    click(entry)
                    wait(lambda: state()[field] == identifier)

                def modal_button(action):
                    entry = wait(lambda: next((b for b in state()['name_controls'] if b['action'] == action and all(x>0 for x in b['size'])), None))
                    desktop.move(window, entry['centre'], 1)
                    wait(lambda: any(b['action'] == action and b['hovered'] for b in state()['name_controls']))
                    click(entry)

                def capture(label):
                    path = report.with_suffix('.png')
                    path.unlink(missing_ok=True)
                    key('F12')
                    wait(lambda: capture_ready(path), 30)
                    destination = output/f'{name}.{label}.png'
                    path.replace(destination)

                next_request = 0

                def agent(edits):
                    nonlocal next_request
                    next_request += 1
                    bridge = json.loads((mailbox/'state.json').read_text())
                    request = dict(session=bridge['session'], request_id=next_request,
                                   command=dict(command='apply', expected_revision=state()['revision'], edits=edits))
                    (mailbox/'agent.tmp').write_text(json.dumps(request))
                    (mailbox/'agent.tmp').replace(mailbox/'request.json')
                    def receipt():
                        path = mailbox/'receipt.json'
                        if path.exists():
                            value = json.loads(path.read_text())
                            if value['request_id'] == next_request:
                                return value
                        return None
                    return wait(receipt)

                verify(state, key, chord, replace_text, select, modal_button, capture, agent, desktop, window, click)
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
        text = logfile.read_text()
        assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in text
        diagnostics = check_gpu_diagnostics(text, allow_surface_resize=False)
        cases.append(dict(name=name, gpu=final['gpu'], viewport=final['viewport'],
                          validation_layer_active=True, allowed_resize_diagnostics=diagnostics))

    def edit(state, key, chord, replace_text, select, modal_button, capture, agent, desktop, window, click):
        original = copy.deepcopy(state()['document'])
        assert not state()['unsaved_changes']
        target_specs = [('Object(1)', 'selected', 1, 'objects', 'West wall'),
                        ('Guide(1002)', 'path_selected', 1002, 'paths', 'North road'),
                        ('Marker(1000)', 'marker_selected', 1000, 'markers', 'Quest note'),
                        ('Plant(1001)', 'plant_selected', 1001, 'plants', 'Old tree')]
        for target, field, identifier, collection, name in target_specs:
            select(target, field, identifier)
            before = copy.deepcopy(state()['document'])
            revision = state()['revision']
            history = state()['history']
            view = state()['view']
            key('F8')
            wait(lambda: state()['name_edit'] is not None)
            assert state()['unsaved_changes']
            wait(lambda: state()['name_selection'] == [0,len(state()['name_edit'].encode())])
            replace_text(name)
            # Editor mode, view, history, load/save/playtest/delete/duplicate stay isolated.
            for shortcut in ['F1', 'F5', 'Tab', 'Delete']:
                key(shortcut)
            for shortcut in ['s', 'o', 'd', 'z', 'y']:
                chord('Control_L', shortcut)
            assert state()['document'] == before and state()['revision'] == revision
            assert state()['view'] == view and state()['history'] == history
            assert state()[field] == identifier and state()['play_feet'] is None
            replace_text(name)
            if identifier == 1:
                receipt = agent([dict(op='rename', id=1, name='Busy attempt')])
                assert not receipt['accepted'] and receipt['revision'] == revision
                capture('dialog')
                # Backdrop, world and underlying side panels cannot edit or navigate.
                click(next(b for b in state()['buttons'] if b['action'] == 'Place(Box)'))
                click(next(b for b in state()['content_list']['controls'] if b['target'] == 'Object(2)'))
                desktop.click(window, [2200,450], 1, 1)
                desktop.drag(window, [2200,450], 1)
                assert state()['document'] == before and state()['revision'] == revision
                assert state()['selected'] == 1 and state()['view'] == view
                # Invalid empty input must preserve buffer, document, selection and history.
                chord('Control_L', 'a')
                key('BackSpace')
                wait(lambda: state()['name_edit'] == '')
                key('Return')
                wait(lambda: 'Invalid object name' in state()['status'])
                assert state()['name_edit'] == '' and state()['document'] == before
                assert state()['history'] == history and state()['revision'] == revision
                capture('invalid')
                replace_text(name)
                modal_button('Apply')
            else:
                key('Return')
            wait(lambda: state()['name_edit'] is None and state()['revision'] == revision+1)
            expected = copy.deepcopy(before)
            next(item for item in expected[collection] if item['id'] == identifier)['name'] = name
            assert state()['document'] == expected
            wait(lambda: any(b['target'] == target and name in b['label'] for b in state()['content_list']['controls']))
            chord('Control_L', 'z')
            wait(lambda: state()['revision'] == revision+2)
            assert state()['document'] == before
            chord('Control_L', 'y')
            wait(lambda: state()['revision'] == revision+3)
            assert state()['document'] == expected
        # Zone naming is explicit even with a selected plant.
        before = copy.deepcopy(state()['document'])
        revision = state()['revision']
        chord('Shift_L', 'F8')
        wait(lambda: state()['name_edit'] == before['name'])
        replace_text('Hollow west')
        key('Return')
        wait(lambda: state()['name_edit'] is None and state()['revision'] == revision+1)
        expected = copy.deepcopy(before)
        expected['name'] = 'Hollow west'
        assert state()['document'] == expected
        # Unchanged submit, keyboard cancel and pointer cancel add no history.
        history = state()['history']
        for action in ['unchanged', 'escape', 'button']:
            key('F8')
            wait(lambda: state()['name_edit'] is not None)
            if action == 'unchanged':
                key('Return')
            else:
                replace_text('Discard me')
                if action == 'escape':
                    key('Escape')
                else:
                    modal_button('Cancel')
            wait(lambda: state()['name_edit'] is None)
            assert state()['document'] == expected and state()['history'] == history
            assert state()['revision'] == revision+1
        # Accepted agent rename updates list, then shared visual undo restores it.
        receipt = agent([dict(op='rename', id=1001, name='Agent tree')])
        assert receipt['accepted']
        wait(lambda: state()['revision'] == revision+2)
        wait(lambda: any('Agent tree' in b['label'] for b in state()['content_list']['controls']))
        chord('Control_L', 'z')
        wait(lambda: state()['revision'] == revision+3)
        assert state()['document'] == expected
        chord('Control_L', 's')
        wait(lambda: not state()['unsaved_changes'])
        assert json.loads(Path(state()['save_path']).read_text()) == expected
        assert original['origin'] == expected['origin']
        # Pending guide draft and playtest refuse opening without losing work.
        key('F3')
        wait(lambda: state()['path_mode'])
        entry = wait(lambda: next((b for b in state()['buttons'] if b['action'] == 'NewPath(Road)'), None))
        click(entry)
        wait(lambda: state()['path_draft_points'] == 0)
        key('F8')
        wait(lambda: 'Finish or cancel' in state()['status'])
        assert state()['name_edit'] is None and state()['path_draft_points'] == 0
        key('Escape')
        key('F5')
        wait(lambda: state()['play_feet'] is not None)
        key('F8')
        wait(lambda: 'Finish or cancel' in state()['status'])
        assert state()['name_edit'] is None
        key('F5')
        wait(lambda: state()['play_feet'] is None)
        checks.append('all authored kinds and zone: real text/select-all, shared undo/redo, invalid buffer retained, input isolation, pointer Apply/Cancel, busy agent and draft/playtest refusal, save')

    for name, origin in [('local', [0,0,0]), ('far', [128000000000,20,-128000000000])]:
        document = copy.deepcopy(base)
        document['origin'] = origin
        def shift(position):
            return [v+o for v,o in zip(position,origin)]
        document['spawn'] = shift(document['spawn'])
        for obj in document['objects']:
            obj['pose']['position'] = shift(obj['pose']['position'])
        for path in document['paths']:
            path['points'] = [shift(p) for p in path['points']]
        for group in ['plants','markers']:
            for item in document[group]:
                item['position'] = shift(item['position'])
        source = output/f'{name}.zone.json'
        source.write_text(json.dumps(document))
        case(name,source,edit)

    def reopened(state, key, chord, replace_text, select, modal_button, capture, agent, desktop, window, click):
        document = state()['document']
        assert document['name'] == 'Hollow west'
        assert document['objects'][0]['name'] == 'West wall'
        assert document['paths'][0]['name'] == 'North road'
        assert document['markers'][0]['name'] == 'Quest note'
        assert document['plants'][0]['name'] == 'Old tree'
        assert not state()['unsaved_changes'] and state()['history'] == [0,0]
        key('F8')
        wait(lambda: state()['name_edit'] == 'Hollow west')
        key('Escape')
        wait(lambda: state()['name_edit'] is None and not state()['unsaved_changes'])
        checks.append('far saved names reopen cleanly with unchanged coordinates and fresh history; F8 without selection edits zone')

    case('reopen', output/'far.zone.json', reopened)
    result = dict(passed=True, checks=checks, cases=cases, input_notes=input_notes,
                  editor_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
    (output/'qualification.json').write_text(json.dumps(result, indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=ROOT/'target/debug/theia-zone-editor')
    args = parser.parse_args()
    print(json.dumps(run(args.output,args.binary),indent=2))
