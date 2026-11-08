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


def run(output):
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
        with log_path.open('w') as log:
            process = subprocess.Popen([sys.executable, str(ROOT/'tools/edit_zone.py'), '--no-build', *args,
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
                desktop.move(window, [1720,720], 1)
                key = desktop.key

                def chord(name):
                    desktop.key_state('Control_L', True)
                    try:
                        key(name)
                    finally:
                        desktop.key_state('Control_L', False)

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

                verify(state, key, chord, button, desktop, window, capture, frame_terrain, play_short_route)
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

    def fresh(state,key,chord,button,desktop,window,capture,frame,play):
        assert not draft.exists()
        assert state()['document']['name'] == 'Wayfarer Hollow' and state()['revision'] == 4
        assert state()['unsaved_changes'] and len(state()['document']['plants']) == 48
        view = frame()
        desktop.move(window,[1720,720],1)
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

    original = case('fresh',['--example','--draft',str(draft)],fresh)
    stored_bytes = draft.read_bytes()

    def reopen(state,key,chord,button,desktop,window,capture,frame,play):
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

    def far_case(state,key,chord,button,desktop,window,capture,frame,play):
        assert state()['document']['origin'] == offset and not state()['unsaved_changes']
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
    (output/'qualification.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(summary,indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    args = parser.parse_args()
    run(args.output)
