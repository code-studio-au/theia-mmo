#!/usr/bin/env python3
"""Native Linux OS-input qualification for the bounded offline zone editor."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary, size):
    output.mkdir(parents=True, exist_ok=True)
    report = output / 'state.json'
    saved = output / 'zone.json'
    for path in [report, report.with_suffix('.png'), saved]:
        path.unlink(missing_ok=True)
    env = dict(os.environ, WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation')
    checks = []
    with (output / 'client.log').open('w') as log:
        process = subprocess.Popen([str(binary.resolve()), '--commands', 'content/zones/greybox.commands.jsonl',
                                    '--output', str(saved), '--report', str(report)], env=env, stdout=log, stderr=log)
        desktop = None
        try:
            window = wait(lambda: owned_window(process.pid), 45)
            desktop = Desktop()
            desktop.focus(window)
            def state():
                assert process.poll() is None, 'Editor exited during qualification'
                return json.loads(report.read_text())
            wait(lambda: report.exists() and len(state()['buttons']) == 12, 45)
            if size:
                width, height = map(int, size.split('x'))
                subprocess.run(['wmctrl', '-ir', hex(window), '-e', f'0,-1,-1,{width},{height}'], check=True)
                if size == '3440x1440':
                    subprocess.run(['wmctrl', '-ir', hex(window), '-b', 'add,fullscreen'], check=True)
                wait(lambda: state()['viewport'] == [width, height])
            assert all(0 <= b['centre'][1] - b['size'][1]/2 and b['centre'][1] + b['size'][1]/2 <= state()['viewport'][1] for b in state()['buttons'])
            def button(action):
                entry = next(b for b in state()['buttons'] if b['action'] == action)
                desktop.click(window, entry['centre'], 1, 1)
            def key(name):
                desktop.key(name)
            def chord(name):
                desktop.key_state('Control_L', True)
                try:
                    key(name)
                finally:
                    desktop.key_state('Control_L', False)
            def revision(previous):
                return wait(lambda: state() if state()['revision'] == previous + 1 else None)
            initial = state()
            assert initial['revision'] == 1 and len(initial['document']['objects']) == 5
            checks.append('actual startup commands populate five rendered primitives')
            button('Place(Box)')
            time.sleep(.3)
            desktop.click(window, state()['ground'][0], 1, 1)
            placed = revision(1)
            assert len(placed['document']['objects']) == 6 and placed['selected'] == 6
            checks.append('actual palette pointer placement shares command history')
            key('Escape')
            time.sleep(.3)
            start = next(o['screen'] for o in state()['objects'] if o['id'] == 6)
            before = state()
            desktop.move(window, start, 1)
            desktop.button(1, True)
            desktop.move(window, [start[0] + 70, start[1] - 12], 1)
            time.sleep(.4)
            assert state()['revision'] == before['revision'], 'Preview committed before release'
            desktop.button(1, False)
            moved = revision(before['revision'])
            assert moved['document']['objects'][-1]['pose']['position'] != before['document']['objects'][-1]['pose']['position']
            checks.append('native drag previews without revision and releases one transaction')
            for name in ['Right', 'Page_Up', 'r', 'equal']:
                previous = state()['revision']
                key(name)
                revision(previous)
            changed = state()
            shape = changed['document']['objects'][-1]['pose']
            assert shape['yaw_degrees'] == 15 and shape['size'] == [1.25]*3
            checks.append('actual keyboard translation, elevation, yaw and uniform scale')
            chord('z')
            revision(changed['revision'])
            previous = state()['revision']
            button('Redo')
            redone = revision(previous)
            assert redone['document'] == changed['document']
            checks.append('keyboard undo and widget redo restore exact shared document')
            previous = state()['revision']
            key('Delete')
            assert len(revision(previous)['document']['objects']) == 5
            previous = state()['revision']
            button('Undo')
            assert len(revision(previous)['document']['objects']) == 6
            point = next(o['screen'] for o in state()['objects'] if o['id'] == 6)
            desktop.click(window, point, 1, 1)
            wait(lambda: state()['selected'] == 6)
            chord('s')
            wait(lambda: json.loads(saved.read_text()) == state()['document'])
            saved_document = json.loads(saved.read_text())
            previous = state()['revision']
            key('Left')
            altered = revision(previous)
            previous = altered['revision']
            button('Load')
            loaded = revision(previous)
            assert loaded['document'] == saved_document
            previous = loaded['revision']
            chord('z')
            assert revision(previous)['document'] == altered['document']
            previous = state()['revision']
            chord('y')
            assert revision(previous)['document'] == saved_document
            checks.append('save/load exact round trip and undoable reload')
            button('Snap')
            wait(lambda: not state()['snap'])
            key('g')
            wait(lambda: state()['snap'])
            before_play = state()
            key('F5')
            playing = wait(lambda: state() if state()['play_feet'] is not None else None)
            start_feet = playing['play_feet']
            desktop.key_state('w', True)
            try:
                time.sleep(.65)
            finally:
                desktop.key_state('w', False)
            wait(lambda: state()['play_feet'][2] < start_feet[2] - 1.)
            key('F5')
            after_play = wait(lambda: state() if state()['play_feet'] is None else None)
            assert after_play['document'] == before_play['document']
            assert after_play['revision'] == before_play['revision']
            checks.append('F5 native shared-Avian walking returns without changing edits/history')
            key('F12')
            wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            checks.append('actual native Vulkan screenshot captured')
            summary = dict(checks=checks, gpu=after_play['gpu'], viewport=after_play['viewport'],
                           revision=after_play['revision'], history=after_play['history'],
                           scope='Linux synthetic OS input; offline editor and collision sandbox; no multiplayer or Windows evidence')
            (output/'qualification.json').write_text(json.dumps(summary, indent=2)+'\n')
            subprocess.run(['wmctrl', '-ic', hex(window)], check=True)
            process.wait(timeout=20)
            assert process.returncode == 0
        finally:
            if desktop:
                desktop.close()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    summary['allowed_resize_diagnostics'] = check_gpu_diagnostics((output/'client.log').read_text(), allow_surface_resize=bool(size))
    (output/'qualification.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--size', choices=['1280x720', '1920x1200', '3440x1440'])
    args = parser.parse_args()
    run(args.output, args.binary, args.size)
