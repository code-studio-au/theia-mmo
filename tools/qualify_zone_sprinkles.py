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


def run(output, binary, only_case=None, scroll=False, attributes=False, dialogs=False):
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
        if dialogs:
            for index in range(14):
                (folder / f"dialog-fixture-{index}.zone.json").write_text(json.dumps(document))
        env = client_environment(dict(os.environ, WGPU_BACKEND='vulkan',
            VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer'))
        logfile = folder / 'editor.log'
        desktop = None
        if scroll or attributes or dialogs:
            desktop = Desktop()
            desktop.prepare_wheel()
        with logfile.open('w') as log:
            process = subprocess.Popen([str(binary), '--zone', str(source), '--report', str(report)],
                                       env=env, stdout=log, stderr=log)
            try:
                window = wait(lambda: owned_window(process.pid), 45)
                if desktop is None:
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
                for title in ['Selection', 'Tools']:
                    other_sections = {c['title']:c['expanded'] for c in state()['sections'] if c['title'] != title}
                    section = next(c for c in state()['sections'] if c['title'] == title)
                    click(section['centre'])
                    wait(lambda:any(c['title']==title and not c['expanded'] for c in state()['sections']))
                    assert {c['title']:c['expanded'] for c in state()['sections'] if c['title'] != title} == other_sections
                    unchanged()
                    capture(f'collapsed-{title.lower()}')
                    section = next(c for c in state()['sections'] if c['title'] == title)
                    click(section['centre'])
                    wait(lambda:any(c['title']==title and c['expanded'] for c in state()['sections']))
                    assert {c['target']: c['value'] for c in state()['workspace_controls'] if c['range'] is not None} == settings_before
                    unchanged()
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
                if dialogs:
                    subprocess.run(['wmctrl','-ir',hex(window),'-b','remove,fullscreen'],check=True)
                    subprocess.run(['wmctrl','-ir',hex(window),'-e','0,0,0,3440,420'],check=True)
                    wait(lambda: state()['viewport'] == [3440,420])
                    def dialog_pane():
                        return next(p for p in state()['scroll_panes'] if p['scope']=='Dialog' and min(p['size']) > 0)
                    wait(lambda: dialog_pane()['position'][1] > 24)
                    def onscreen(entry):
                        assert min(entry['size']) > 0
                        assert entry['centre'][1]-entry['size'][1]/2 >= 12
                        assert entry['centre'][1]+entry['size'][1]/2 <= 408
                    for target in ['Field','Apply','Cancel']:
                        onscreen(next(c for c in state()['file_controls'] if c['target']==target))
                    before = copy.deepcopy(fresh())
                    pane = dialog_pane()
                    move(pane['centre'])
                    desktop.scroll(1)
                    wait(lambda: abs(dialog_pane()['position'][1] - max(0,pane['position'][1]-24)) < .1)
                    assert state()['file_dialog']['text'] == before['file_dialog']['text']
                    unchanged()
                    capture('short-file-dialog')
                desktop.key('Escape')
                wait(lambda: state()['file_dialog'] is None)
                if dialogs:
                    desktop.key('F8')
                    wait(lambda: state()['name_edit'] is not None)
                    for entry in state()['name_controls']:
                        onscreen(entry)
                    capture('short-name-dialog')
                    desktop.key('Escape')
                    wait(lambda: state()['name_edit'] is None)
                    unchanged()
                    subprocess.run(['wmctrl','-ir',hex(window),'-b','add,fullscreen'],check=True)
                    wait(lambda:state()['viewport']==[3440,1440])
                control('Transform')
                wait(lambda: state()['transform_edit'] is not None)
                capture('exact-attributes')
                if attributes:
                    exact = state()['transform_edit']['fields'][:]
                    subprocess.run(['wmctrl','-ir',hex(window),'-b','remove,fullscreen'],check=True)
                    subprocess.run(['wmctrl','-ir',hex(window),'-e','0,0,0,3440,420'],check=True)
                    wait(lambda:state()['viewport']==[3440,420])
                    first = next(c for c in state()['transform_controls'] if c['target']=='Field(0)')['entity']
                    visited=[]
                    while True:
                        focused=state()['focus']
                        if focused in visited:
                            assert focused==first
                            break
                        visited.append(focused)
                        def visible_focus():
                            c=next(c for c in state()['transform_controls'] if c['entity']==focused)
                            return c['centre'][1]-c['size'][1]/2 >= 92 and c['centre'][1]+c['size'][1]/2 <= 420
                        wait(visible_focus)
                        assert state()['transform_edit']['fields']==exact
                        desktop.key('Tab')
                        wait(lambda:state()['focus']!=focused)
                    wait(visible_focus)
                    assert len(visited)==len(exact)+1
                    def attribute_pane():
                        return next(p for p in state()['scroll_panes'] if p['scope']=='Attributes')
                    p=attribute_pane()
                    move([p['centre'][0],300])
                    desktop.scroll(-1)
                    expected=min(p['limit'][1],p['position'][1]+24)
                    wait(lambda:attribute_pane()['position'][1]==expected)
                    assert fresh()['transform_edit']['fields']==exact
                    unchanged()
                    capture('scrolled-exact-attributes')
                    desktop.key('Escape')
                    wait(lambda:state()['transform_edit'] is None)
                    unchanged()
                    subprocess.run(['wmctrl','-ir',hex(window),'-b','add,fullscreen'],check=True)
                    wait(lambda:state()['viewport']==[3440,1440])
                    control('Transform')
                    wait(lambda:state()['transform_edit'] is not None)
                    assert state()['transform_edit']['fields']==exact
                original_fields = state()['transform_edit']['fields'][:]
                label = next(c['centre'] for c in state()['transform_controls'] if c['target']=='Label(0)')
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
                if scroll:
                    before_scroll = copy.deepcopy(fresh())
                    control('Mode(Plants)')
                    subprocess.run(['wmctrl','-ir',hex(window),'-b','remove,fullscreen'],check=True)
                    subprocess.run(['wmctrl','-ir',hex(window),'-e','0,0,0,3440,600'],check=True)
                    wait(lambda: state()['viewport'] == [3440,600])
                    def browser_pane():
                        return next(p for p in state()['scroll_panes'] if p['scope']=='Browser')
                    browser = wait(lambda: browser_pane() if browser_pane()['limit'][1] > 24 else None)
                    browser_page = state()['content_list']['page']
                    move([browser['centre'][0],400])
                    desktop.scroll(-1)
                    wait(lambda: browser_pane()['position'][1] == 24)
                    assert state()['content_list']['page'] == browser_page
                    browser_rect = state()['ui_layout']['browser']
                    track_height = browser_rect[3]-6
                    thumb_height = max(24,track_height*browser['size'][1]/(browser['size'][1]+browser['limit'][1]))
                    thumb_offset = 24/browser['limit'][1]*(track_height-thumb_height)
                    thumb_x = browser_rect[0]+browser_rect[2]-4.5
                    move([thumb_x,browser_rect[1]+3+thumb_offset+thumb_height/2])
                    press()
                    move([thumb_x,540])
                    wait(lambda:abs(browser_pane()['position'][1]-browser['limit'][1])<.1)
                    release()
                    wait(lambda:not state()['ui_layout']['resizing'])
                    assert state()['ui_layout']['browser'][2] == browser_rect[2]
                    assert state()['content_list']['page'] == browser_page
                    capture('scrolled-short-browser')
                    assert state()['document'] == before_scroll['document']
                    assert state()['history'] == before_scroll['history'] and state()['revision'] == before_scroll['revision']
                    def pane_state():
                        return next(p for p in state()['scroll_panes'] if p['scope']=='Workspace')
                    pane = wait(lambda: pane_state() if pane_state()['limit'][1] > 24 else None)
                    move([pane['centre'][0],400])
                    desktop.scroll(-1)
                    wait(lambda: pane_state()['position'][1] == 24)
                    desktop.scroll(-8)
                    wait(lambda: abs(pane_state()['position'][1] - min(216,pane['limit'][1])) < .1)
                    desktop.scroll(8)
                    wait(lambda: pane_state()['position'][1] == 0)
                    inspector = state()['ui_layout']['inspector']
                    track_height = inspector[3]-6
                    thumb_height = max(24,track_height*pane['size'][1]/(pane['size'][1]+pane['limit'][1]))
                    offset = pane_state()['position'][1]/pane['limit'][1]*(track_height-thumb_height)
                    thumb_x = inspector[0]+inspector[2]-4.5
                    move([thumb_x,inspector[1]+3+offset+thumb_height/2])
                    press()
                    move([thumb_x,590])
                    wait(lambda:abs(pane_state()['position'][1]-pane['limit'][1])<.1)
                    release()
                    wait(lambda:not state()['ui_layout']['resizing'])
                    assert state()['ui_layout']['inspector'][2] == inspector[2]
                    capture('scrolled-short-inspector')
                    assert state()['document'] == before_scroll['document']
                    assert state()['history'] == before_scroll['history'] and state()['revision'] == before_scroll['revision']
                    subprocess.run(['wmctrl','-ir',hex(window),'-b','add,fullscreen'],check=True)
                    wait(lambda:state()['viewport']==[3440,1440])
                    control('Command(Play)')
                    wait(lambda:state()['play_feet'] is not None)
                    capture('playtest-after-scroll')
                    control('Command(Play)')
                    wait(lambda:state()['play_feet'] is None)
                    assert state()['document'] == before_scroll['document']
                    assert state()['history'] == before_scroll['history']

                cases.append(dict(case=name, gpu=state()['gpu'], revision=state()['revision'],
                    panels=[420, 480], exact_source_restored=True, source_file_unchanged=True))
            finally:
                if desktop:
                    desktop.close()
                process.terminate()
                process.wait(timeout=15)
        text = logfile.read_text()
        assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in text
        cases[-1]['gpu_diagnostics'] = check_gpu_diagnostics(text, allow_surface_resize=scroll or attributes or dialogs)
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == binary_hash, 'Binary changed during native qualification'
    result = dict(cases=cases, scrolling=scroll, attribute_scrolling=attributes, dialogs=dialogs, editor_sha256=binary_hash,
        scope='Linux native 3440x1440; shorter 3440x600 scroll check when enabled; real OS input, Vulkan; no Windows or multiplayer claim')
    (output / 'qualification.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--case', choices=['local', 'far'])
    parser.add_argument('--scroll', action='store_true', help='Check real wheel and scrollbar dragging in a shorter owned window')
    parser.add_argument('--attributes-scroll', action='store_true', help='Check Tab focus reveal and exact draft scrolling in a short owned window')
    parser.add_argument('--dialogs', action='store_true', help='Check file and naming dialog layout and wheel input in a short owned window')
    args = parser.parse_args()
    run(args.output, args.binary, args.case, args.scroll, args.attributes_scroll, args.dialogs)
