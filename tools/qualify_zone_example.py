#!/usr/bin/env python3
"""Qualify the original example draft, terrain framing and native playtest."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import subprocess
import sys
import time

from PIL import Image

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready

ROOT = Path(__file__).resolve().parents[1]


def run(output, content_list=False):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    draft = output / 'draft.json'
    source_paths = [ROOT/'content/zones/wayfarer-hollow.base.json', ROOT/'content/zones/wayfarer-hollow.commands.jsonl']
    source_hashes = [hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths]
    env = dict(os.environ, WGPU_BACKEND='vulkan',
               VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer')
    cases = []

    def case(name, args, verify):
        report = output / f'{name}.state.json'
        log_path = output / f'{name}.log'
        captures = []
        mailbox = output / f'{name}.mailbox'
        extra = ['--mailbox', str(mailbox)] if content_list else []
        with log_path.open('w') as log:
            process = subprocess.Popen([sys.executable, str(ROOT/'tools/edit_zone.py'), '--no-build', *args, *extra,
                                        '--report', str(report)], env=env, stdout=log, stderr=log)
            desktop = None
            try:
                window = wait(lambda: owned_window(process.pid), 45)
                desktop = Desktop()
                desktop.focus(window)

                def state():
                    assert process.poll() is None
                    return json.loads(report.read_text())

                wait(lambda: report.exists() and state()['viewport'] == [3440, 1440], 45)
                desktop.move(window, [1737,731], 1)
                desktop.move(window, [1720,720], 1)
                key = desktop.key
                next_request = 0

                def chord(name):
                    desktop.key_state('Control_L', True)
                    try:
                        key(name)
                    finally:
                        desktop.key_state('Control_L', False)

                def button(action):
                    entry = wait(lambda: next((b for b in state()['buttons'] if b['action'] == action),None))
                    desktop.move(window, entry['centre'], 1)
                    wait(lambda: any(b['action'] == action and b['hovered'] for b in state()['buttons']))
                    desktop.button(1, True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1, False)
                    wait(lambda: not state()['left_mouse_down'])

                def content_click(target):
                    entry = next(b for b in state()['content_list']['controls'] if b['target'] == target)
                    desktop.move(window,entry['centre'],1)
                    wait(lambda: any(b['target'] == target and b['hovered'] for b in state()['content_list']['controls']))
                    desktop.button(1,True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1,False)
                    wait(lambda: not state()['left_mouse_down'])

                def agent_command(command):
                    nonlocal next_request
                    next_request += 1
                    bridge = json.loads((mailbox/'state.json').read_text())
                    request = dict(session=bridge['session'],request_id=next_request,command=command)
                    (mailbox/'agent.tmp').write_text(json.dumps(request))
                    (mailbox/'agent.tmp').replace(mailbox/'request.json')
                    def receipt():
                        if not (mailbox/'receipt.json').exists():
                            return None
                        value = json.loads((mailbox/'receipt.json').read_text())
                        return value if value['request_id'] == next_request else None
                    return wait(receipt)

                def capture(label):
                    attempts = 0
                    def ready():
                        nonlocal attempts
                        attempts += 1
                        key('F12')
                        path = report.with_suffix('.png')
                        wait(lambda: capture_ready(path), 30)
                        with Image.open(path) as picture:
                            pixels = list(picture.convert('RGB').crop((300,0,3440,1400)).resize((785,350)).getdata())
                        counts = dict(
                            road=sum(r>120 and r>g*1.25 and g>b*1.15 for r,g,b in pixels),
                            blue=sum(b>120 and b>r*1.5 and b>g*1.2 for r,g,b in pixels),
                            foliage=sum(g>60 and g>r*1.8 and g>b*1.5 for r,g,b in pixels))
                        if counts['road'] < 60 or counts['blue'] < 18 or counts['foliage'] < 30:
                            path.unlink()
                            return False
                        captures.append(dict(label=label,attempts=attempts,pixels=counts))
                        path.rename(output/f'{name}.{label}.png')
                        return True
                    wait(ready,45)

                def frame_terrain():
                    before = state()
                    key('F2')
                    wait(lambda: state()['terrain_mode'])
                    key('f')
                    wait(lambda: state()['status'] == 'Focused selection')
                    heights = state()['document']['terrain']['heights']
                    centre = (min(heights)+max(heights))/2
                    view = state()['view']
                    assert all(abs(a-b)<.001 for a,b in zip(view['target'],[64,centre,64]))
                    distance = math.sqrt(64**2+64**2+((max(heights)-min(heights))/2)**2)*3+8
                    assert abs(view['distance']-distance)<.001
                    assert state()['document'] == before['document'] and state()['revision'] == before['revision']
                    return view

                def play_short_route():
                    before = state()
                    key('F5')
                    wait(lambda: state()['play_feet'] is not None)
                    start = state()['play_feet']
                    desktop.key_state('w', True)
                    try:
                        time.sleep(1.2)
                    finally:
                        desktop.key_state('w', False)
                    wait(lambda: state()['play_feet'][2] < start[2]-3.)
                    feet = state()['play_feet']
                    assert abs(feet[0]-64)<.1 and abs(feet[1])<.15
                    capture('playtest')
                    key('F5')
                    wait(lambda: state()['play_feet'] is None)
                    assert state()['document'] == before['document'] and state()['unsaved_changes'] == before['unsaved_changes']

                verify(state, key, chord, button, desktop, window, capture, frame_terrain, play_short_route, agent_command, content_click)
                capture('final')
                final = state()
                subprocess.run(['wmctrl','-ic',hex(window)], check=True)
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
        cases.append(dict(name=name,gpu=final['gpu'],viewport=final['viewport'],
                          validation_layer_active=True,allowed_resize_diagnostics=diagnostics,captures=captures))
        return final

    def fresh(state,key,chord,button,desktop,window,capture,frame,play,agent,content):
        assert not draft.exists()
        assert state()['document']['name'] == 'Wayfarer Hollow' and state()['revision'] == 4
        assert state()['unsaved_changes'] and len(state()['document']['plants']) == 48
        view = frame()
        # Initialize the owned synthetic wheel over a non-scrolling UI area;
        # its first valuator event can be consumed as a device baseline.
        desktop.move(window,[150,20],1)
        wait(lambda: state().get('cursor') == [150,20])
        desktop.scroll(1)
        time.sleep(.3)
        assert state()['view'] == view
        desktop.move(window,[1720,720],1)
        wait(lambda: state().get('cursor') == [1720,720])
        desktop.scroll(3)
        wait(lambda: state()['view']['distance'] < view['distance']*.9)
        capture('overview')
        chord('s')
        wait(lambda: draft.exists() and not state()['unsaved_changes'])
        play()
        key('F4')
        wait(lambda: state()['marker_mode'])
        button('NextMarker')
        key('equal')
        wait(lambda: state()['revision'] == 5)
        assert state()['document']['markers'][0]['radius_metres'] == 2.
        chord('s')
        wait(lambda: not state()['unsaved_changes'])

        if content_list:
            before = state()
            assert before['content_list']['count'] == 66 and before['content_list']['pages'] == 2
            assert len(before['content_list']['controls']) == 42
            for row in before['content_list']['controls']:
                x,y = row['centre']; w,h = row['size']
                assert x-w/2 >= 3140 and x+w/2 <= 3440 and y-h/2 >= 0 and y+h/2 <= 1440
            def unchanged():
                assert state()['document'] == before['document'] and state()['revision'] == before['revision']
                assert state()['history'] == before['history'] and not state()['unsaved_changes']
            content('Object(1)')
            wait(lambda: state()['selected'] == 1 and not state()['marker_mode'])
            unchanged()
            key('f')
            wait(lambda: abs(state()['view']['target'][0]-42)<.001)
            content('Next')
            wait(lambda: state()['content_list']['page'] == 1)
            content('Plant(5019)')
            wait(lambda: state()['plant_selected'] == 5019 and state()['vegetation_mode'])
            unchanged()
            key('f')
            plant = before['document']['plants'][-1]
            expected = list(plant['position']); expected[1] += .6*plant['scale']
            wait(lambda: all(abs(a-b)<.001 for a,b in zip(state()['view']['target'],expected)))
            # Keyboard selection reveals the corresponding page without framing.
            view = state()['view']
            key('F1')
            wait(lambda: not state()['vegetation_mode'] and any(b['action'] == 'Place(Box)' for b in state()['buttons']))
            key('Tab')
            wait(lambda: state()['selected'] == 1 and state()['content_list']['page'] == 0)
            assert state()['view'] == view
            unchanged()
            content('Next')
            wait(lambda: state()['content_list']['page'] == 1)
            content('Plant(5019)')
            wait(lambda: state()['plant_selected'] == 5019)
            content('Previous')
            wait(lambda: state()['content_list']['page'] == 0)
            content('Guide(100)')
            wait(lambda: state()['path_selected'] == 100 and state()['path_mode'])
            assert state()['path_point'] is None
            unchanged()
            key('f')
            wait(lambda: abs(state()['view']['target'][0]-56)<.001)
            content('Terrain')
            wait(lambda: state()['terrain_mode'] and not state()['path_mode'])
            assert state()['brush_point'] is None
            unchanged()
            content('Spawn')
            wait(lambda: state()['status'] == 'Focused playtest spawn')
            assert all(abs(a-b)<.001 for a,b in zip(state()['view']['target'],[64,1.1,116]))
            unchanged()
            # Page clicks must not place behind the panel with an active palette.
            key('F1')
            button('Place(Box)')
            content('Next')
            wait(lambda: state()['content_list']['page'] == 1)
            unchanged()
            right = next(b for b in state()['content_list']['controls'] if b['target'] == 'Previous')['centre']
            desktop.move(window,right,1)
            view = state()['view']
            desktop.scroll(2)
            desktop.button(3,True)
            try:
                desktop.move(window,[right[0]+10,right[1]+5],1)
            finally:
                desktop.button(3,False)
            time.sleep(.3)
            assert state()['view'] == view
            unchanged()
            content('Previous')
            wait(lambda: state()['content_list']['page'] == 0)
            key('F3')
            button('NewPath(Road)')
            wait(lambda: state()['path_draft_points'] == 0)
            content('Object(2)')
            wait(lambda: state()['status'] == 'Finish or cancel the preview before selecting content')
            assert state()['path_draft_points'] == 0 and state()['document'] == before['document']
            key('Escape')
            wait(lambda: state()['path_draft_points'] is None)
            unchanged()
            key('F5')
            wait(lambda: state()['play_feet'] is not None and not state()['content_list']['visible'])
            assert state()['content_list']['controls'] == []
            key('F5')
            wait(lambda: state()['play_feet'] is None and state()['content_list']['visible'])
            unchanged()
            content('Marker(2001)')
            wait(lambda: state()['marker_selected'] == 2001 and state()['marker_mode'])
            chord('d')
            copied = wait(lambda: state() if state()['revision'] == before['revision']+1 else None)
            assert copied['content_list']['count'] == 67
            copied_id = copied['marker_selected']
            assert copied_id > max(p['id'] for field in ['objects','paths','markers','plants'] for p in before['document'][field])
            assert any(b['target'] == f'Marker({copied_id})' and ' copy' in b['label'] for b in copied['content_list']['controls'])
            chord('z')
            wait(lambda: state()['revision'] == before['revision']+2 and state()['content_list']['count'] == 66)
            assert state()['document'] == before['document'] and not state()['unsaved_changes']
            marker = dict(before['document']['markers'][0]); marker['name'] = 'Agent refuge note'
            revision = state()['revision']
            receipt = agent(dict(command='apply',expected_revision=revision,edits=[dict(op='put_marker',marker=marker)]))
            assert receipt['accepted']
            wait(lambda: state()['revision'] == revision+1 and any('Agent refuge note' in b['label'] for b in state()['content_list']['controls']))
            content('Marker(2000)')
            wait(lambda: state()['marker_selected'] == 2000)
            chord('z')
            wait(lambda: state()['revision'] == revision+2)
            assert state()['document'] == before['document'] and not state()['unsaved_changes']
            assert any(before['document']['markers'][0]['name'] in b['label'] for b in state()['content_list']['controls'])
            frame()
            content('Next')
            wait(lambda: state()['content_list']['page'] == 1)
            revision = state()['revision']
            receipt = agent(dict(command='apply',expected_revision=revision,
                edits=[dict(op='delete_plant',id=p['id']) for p in before['document']['plants']]))
            assert receipt['accepted']
            wait(lambda: state()['revision'] == revision+1 and state()['content_list']['count'] == 18)
            assert state()['content_list']['pages'] == 1 and state()['content_list']['page'] == 0
            assert len(state()['content_list']['controls']) == 20
            chord('z')
            wait(lambda: state()['revision'] == revision+2 and state()['content_list']['count'] == 66)
            assert state()['document'] == before['document'] and not state()['unsaved_changes']
            assert len(state()['content_list']['controls']) == 42
            capture('content-list')

    original = case('fresh',['--example','--draft',str(draft)],fresh)
    stored_bytes = draft.read_bytes()

    def reopen(state,key,chord,button,desktop,window,capture,frame,play,agent,content):
        assert state()['revision'] == 0 and not state()['unsaved_changes']
        assert state()['document'] == original['document']
        assert state()['document']['markers'][0]['radius_metres'] == 2.
        assert draft.read_bytes() == stored_bytes
        frame()

    case('reopen',['--example','--draft',str(draft)],reopen)
    assert draft.read_bytes() == stored_bytes
    far = json.loads(draft.read_text())
    offset = [128000000000,20,-128000000000]
    def shift(point):
        return [a+b for a,b in zip(point,offset)]
    far['origin'] = shift(far['origin'])
    far['spawn'] = shift(far['spawn'])
    for obj in far['objects']:
        obj['pose']['position'] = shift(obj['pose']['position'])
    for item in far['plants']+far['markers']:
        item['position'] = shift(item['position'])
    for path in far['paths']:
        path['points'] = list(map(shift,path['points']))
    far['terrain']['chunk'] = [1000000000,-1000000000]
    far_path = output/'far.zone.json'
    far_path.write_text(json.dumps(far))

    def far_case(state,key,chord,button,desktop,window,capture,frame,play,agent,content):
        assert state()['document']['origin'] == offset and not state()['unsaved_changes']
        frame()
        if content_list:
            content('Object(2)')
            wait(lambda: state()['selected'] == 2)
            key('f')
            wait(lambda: all(abs(a-b)<.001 for a,b in zip(state()['view']['target'],[42,1.55,102])))
            assert state()['revision'] == 0 and not state()['unsaved_changes']
            frame()
        capture('overview')
        play()

    case('far',['--zone',str(far_path)],far_case)
    assert [hashlib.sha256(p.read_bytes()).hexdigest() for p in source_paths] == source_hashes
    summary = dict(cases=cases,source_sha256=source_hashes,
                   checks=['new example is memory-only until Save, preserving original source/recipe',
                           'user marker edit survives example reopen without reseeding',
                           'whole-terrain framing preserves model/history at local and far Y=20 origin',
                           'actual forward walking past approach posts with Avian contact and unchanged authoring data'],
                   scope='Linux owned native input at 3440x1440; offline authoring/playtest, no live quests or NPCs')
    summary['content_list'] = content_list
    if content_list:
        summary['checks'] += ['40-row paging, cross-mode selection/focus, source bounds and unchanged document/history',
                              'panel click/wheel/right-drag isolation while placing/sculpting; pending draft preservation',
                              'duplicate/undo and agent name changes update visible rows; playtest hides the list',
                              'keyboard selection reveals its page; batch deletion clamps pages and undo restores rows']
    (output/'qualification.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(summary,indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--content-list',action='store_true')
    args = parser.parse_args()
    run(args.output,args.content_list)
