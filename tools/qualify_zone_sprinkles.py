#!/usr/bin/env python3
"""Native Sprinkles-style editor panels, menus and sections at 3440x1440."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary, only_case=None):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    cases = []
    for name, origin in [('local', [.125, .25, -.5]),
                         ('far', [128000000000.125, 20.25, -128000000000.5])]:
        if only_case and name != only_case:
            continue
        folder = output / name
        folder.mkdir()
        source, report = folder / 'source.json', folder / 'state.json'
        document = dict(schema=1, name='Sprinkles editor acceptance', origin=origin,
                        spawn=[origin[0], origin[1]+.0625, origin[2]+6], objects=[
                            dict(id=i, name=f'Authored box {i}', primitive='box', pose=dict(
                                position=[origin[0]+8.125 if i == 1 else origin[0]-64+(i % 7)*5,
                                          origin[1]+1.25,
                                          origin[2]-8.5 if i == 1 else origin[2]-64+(i // 7)*5],
                                yaw_degrees=17.25, size=[2, 2.5, 3])) for i in range(1, 44)])
        source.write_text(json.dumps(document))
        env = client_environment(dict(os.environ, WGPU_BACKEND='vulkan',
            VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer'))
        logfile = folder / 'editor.log'
        desktop = None
        with logfile.open('w') as log:
            process = subprocess.Popen([str(binary), '--zone', str(source), '--report', str(report)],
                                       env=env, stdout=log, stderr=log)
            try:
                window = wait(lambda: owned_window(process.pid), 45)
                desktop = Desktop()
                desktop.focus(window)

                def state():
                    assert process.poll() is None, 'Editor exited'
                    return json.loads(report.read_text())

                wait(lambda: report.exists() and state()['viewport'] == [3440, 1440], 45)
                desktop.move(window, [1720, 720], 1)

                def fresh():
                    elapsed = state()['input_elapsed']
                    wait(lambda: state()['input_elapsed'] > elapsed)
                    return state()

                def move(point):
                    desktop.move(window, point, 1)
                    wait(lambda: state()['cursor'] and max(abs(a-b) for a, b in zip(state()['cursor'], point)) < 2)

                def press():
                    desktop.button(1, True)
                    wait(lambda: state()['left_mouse_down'])

                def release():
                    desktop.button(1, False)
                    wait(lambda: not state()['left_mouse_down'])
                    fresh()

                def click(point):
                    move(point)
                    press()
                    release()

                def control(target, group='workspace_controls'):
                    entries = lambda: state()['content_list']['controls'] if group == 'content' else state()[group]
                    entry = wait(lambda: next((c for c in entries() if c['target'] == target and min(c['size']) > 0), None))
                    click(entry['centre'])

                def capture(label):
                    screenshot = report.with_suffix('.png')
                    screenshot.unlink(missing_ok=True)
                    desktop.key('F12')
                    wait(lambda: capture_ready(screenshot), 30)
                    screenshot.rename(folder / f'{label}.png')

                def menu(title):
                    entry = next(c for c in state()['menus'] if c['name'] == title and c['role'] == 'button')
                    click(entry['centre'])
                    wait(lambda: any(c['name'] == title and c['role'] == 'popup' and min(c['size']) > 0 for c in state()['menus']))

                initial = copy.deepcopy(fresh()['document'])
                assert initial == document
                control('Object(1)', 'content')
                wait(lambda: state()['selected'] == 1)

                baseline_revision, baseline_history = 0, [0, 0]

                def unchanged():
                    current = fresh()
                    assert current['document'] == initial and current['revision'] == baseline_revision and current['history'] == baseline_history

                for mode, flags in [
                    ('Sculpt', [True, False, False, False, False]),
                    ('Paint', [True, True, False, False, False]),
                    ('Guides', [False, False, True, False, False]),
                    ('Markers', [False, False, False, True, False]),
                    ('Plants', [False, False, False, False, True]),
                    ('Objects', [False, False, False, False, False]),
                ]:
                    control(f'Mode({mode})')
                    wait(lambda: [state()[key] for key in ['terrain_mode', 'terrain_paint', 'path_mode', 'marker_mode', 'vegetation_mode']] == flags)
                    unchanged()
                control('Object(1)', 'content')
                wait(lambda: state()['selected'] == 1)

                def resize(edge, target, expected):
                    move([edge, 700])
                    press()
                    wait(lambda: state()['ui_layout']['resizing'])
                    move([target, 700])
                    wait(lambda: [state()['ui_layout']['browser'][2], state()['ui_layout']['inspector'][2]] == expected)
                    unchanged()
                    release()
                    wait(lambda: not state()['ui_layout']['resizing'])
                    unchanged()

                browser_left = state()['ui_layout']['browser'][0]
                resize(browser_left + 300, browser_left + 420, [420, 360])
                resize(browser_left + 780, browser_left + 900, [420, 480])
                move([browser_left + 900, 700])
                press()
                move([browser_left + 1000, 700])
                wait(lambda: state()['ui_layout']['inspector'][2] == 580)
                desktop.key('Escape')
                wait(lambda: state()['ui_layout']['inspector'][2] == 480)
                release()
                unchanged()
                capture('resized-browser-inspector')

                tool = next(c for c in state()['buttons'] if c['action'] == 'Place(Box)' and min(c['size']) > 0)
                click(tool['centre'])
                settings_before = {c['target']: c['value'] for c in state()['workspace_controls'] if c['range'] is not None}
                section = next(c for c in state()['sections'] if c['title'] == 'Tool settings')
                click(section['centre'])
                wait(lambda: any(c['title'] == 'Tool settings' and not c['expanded'] for c in state()['sections']))
                wait(lambda: all(c['size'] == [0, 0] for c in state()['workspace_controls'] if c['range'] is not None))
                unchanged()
                capture('collapsed-tool-settings')
                click(section['centre'])
                wait(lambda: any(c['target'] == 'PlacementSize(0)' and min(c['size']) > 0 for c in state()['workspace_controls']))
                assert {c['target']: c['value'] for c in state()['workspace_controls'] if c['range'] is not None} == settings_before
                unchanged()

                menu('File')
                capture('file-menu')
                desktop.key('Down')
                desktop.key('Return')
                wait(lambda: state()['file_dialog'] is not None)
                unchanged()
                capture('file-browser')
                desktop.key('Escape')
                wait(lambda: state()['file_dialog'] is None)
                control('Transform')
                wait(lambda: state()['transform_edit'] is not None)
                capture('exact-attributes')
                original_fields = state()['transform_edit']['fields'][:]
                field = next(c for c in state()['transform_controls'] if c['target'] == 'Field(0)')
                label = [field['centre'][0] - field['size'][0]/2 - 50, field['centre'][1]]
                click(label)
                assert fresh()['transform_edit']['fields'] == original_fields
                move(label)
                press()
                move([label[0]+25, label[1]])
                expected_x = float(original_fields[0]) + .5
                wait(lambda: float(state()['transform_edit']['fields'][0]) == expected_x)
                release()
                unchanged()
                capture('numeric-label-drag')
                control('Cancel', 'transform_controls')
                wait(lambda: state()['transform_edit'] is None)
                unchanged()
                control('Transform')
                wait(lambda: state()['transform_edit'] is not None)
                assert state()['transform_edit']['fields'] == original_fields
                move(label)
                press()
                move([label[0]+25, label[1]])
                wait(lambda: float(state()['transform_edit']['fields'][0]) == expected_x)
                release()
                control('Apply', 'transform_controls')
                wait(lambda: state()['revision'] == 1 and state()['transform_edit'] is None)
                expected = copy.deepcopy(initial)
                expected['objects'][0]['pose']['position'][0] = expected_x
                assert state()['document'] == expected and state()['history'] == [1, 0]
                menu('Edit')
                control('Command(Undo)')
                wait(lambda: state()['revision'] == 2)
                baseline_revision, baseline_history = 2, [0, 1]
                unchanged()

                menu('File')
                control('New')
                wait(lambda: state()['revision'] == baseline_revision + 1)
                assert state()['document']['objects'] == [] and state()['history'] == [1, 0]
                menu('Edit')
                capture('edit-menu')
                control('Command(Undo)')
                wait(lambda: state()['revision'] == baseline_revision + 2)
                assert state()['document'] == initial and state()['history'] == [0, 1]
                assert [state()['ui_layout']['browser'][2], state()['ui_layout']['inspector'][2]] == [420, 480]
                assert json.loads(source.read_text()) == initial
                capture('restored-document')
                cases.append(dict(case=name, gpu=state()['gpu'], revision=state()['revision'],
                    panels=[420, 480], exact_source_restored=True, source_file_unchanged=True))
            finally:
                if desktop:
                    desktop.close()
                process.terminate()
                process.wait(timeout=15)
        text = logfile.read_text()
        assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in text
        cases[-1]['gpu_diagnostics'] = check_gpu_diagnostics(text)
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == binary_hash, 'Binary changed during native qualification'
    result = dict(cases=cases, editor_sha256=binary_hash,
        scope='Linux native 3440x1440; real OS input, Vulkan; no Windows or multiplayer claim')
    (output / 'qualification.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--case', choices=['local', 'far'])
    args = parser.parse_args()
    run(args.output, args.binary, args.case)
