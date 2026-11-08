#!/usr/bin/env python3
"""Native sampled-level flattening checks, including headless command equivalence."""
import argparse
import json
import os
from pathlib import Path
import struct
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def normalized(document):
    document = json.loads(json.dumps(document))
    document['terrain']['heights'] = [struct.unpack('<f', struct.pack('<f', h))[0]
                                       for h in document['terrain']['heights']]
    return document


def run(output, binary, strength=False):
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    cases = []
    env = dict(os.environ, WGPU_BACKEND='vulkan',
               VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer')
    for name, axis, origin, base, direction in [
        ('coarse', 33, [0, 0, 0], 4., 1),
        ('fine-far', 65, [128000000000, 20, -128000000000], -4., -1),
    ]:
        source = output / f'{name}.source.json'
        saved = output / f'{name}.saved.json'
        patch = dict(chunk=[origin[0]//128, origin[2]//128],
                     heights=[0. if x in (0, axis-1) or z in (0, axis-1) else base
                              for z in range(axis) for x in range(axis)])
        if axis == 65:
            patch['resolution'] = '2m'
        zone = dict(schema=7 if axis == 65 else 2, name='Flatten qualification', origin=origin,
                    spawn=[origin[0]+16, origin[1]+base+.1, origin[2]+16], objects=[], terrain=patch)
        source.write_text(json.dumps(zone))
        report = output / f'{name}.state.json'
        log_path = output / f'{name}.log'
        with log_path.open('w') as log:
            process = subprocess.Popen([str(binary), '--zone', str(source), '--save-path', str(saved),
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
                desktop.move(window, [1720, 720], 1)
                initial = state()['document']
                key = desktop.key

                def chord(name):
                    desktop.key_state('Control_L', True)
                    try:
                        key(name)
                    finally:
                        desktop.key_state('Control_L', False)

                key('F2')
                wait(lambda: state()['terrain_mode'])
                point = state()['terrain_points'][0]
                desktop.move(window, point, 1)
                wait(lambda: state()['brush_point'] and abs(state()['brush_point'][1]-base) < .001)
                key('i')
                wait(lambda: state()['brush_tool'] == 'Flatten' and abs(state()['flatten_height']-base) < .001)
                key('g')
                wait(lambda: not state()['snap'])
                key('Page_Up')
                wait(lambda: abs(state()['flatten_height']-(base+.1)) < 1e-7)
                key('Page_Down')
                wait(lambda: abs(state()['flatten_height']-base) < 1e-7)
                key('g')
                wait(lambda: state()['snap'])
                key('Page_Up' if direction == 1 else 'Page_Down')
                target = base + direction
                wait(lambda: abs(state()['flatten_height']-target) < 1e-7)
                assert state()['revision'] == 0 and state()['document'] == initial
                # Paint mode must not turn an eyedropper key into a height edit.
                key('F7')
                wait(lambda: state()['terrain_paint'])
                paint_tool = state()['brush_tool']
                key('i')
                assert state()['brush_tool'] == paint_tool and state()['flatten_height'] == target
                key('F2')
                wait(lambda: not state()['terrain_paint'])
                key('i')
                wait(lambda: state()['brush_tool'] == 'Flatten' and abs(state()['flatten_height']-base) < .001)
                key('Page_Up' if direction == 1 else 'Page_Down')
                wait(lambda: state()['flatten_height'] == target)
                centre = state()['brush_point']
                desktop.button(1, True)
                try:
                    wait(lambda: state()['left_mouse_down'])
                finally:
                    desktop.button(1, False)
                edited = wait(lambda: state() if state()['revision'] == 1 else None)
                assert edited['history'] == [1, 0]
                heights = edited['document']['terrain']['heights']
                interior = [heights[z*axis+x] for z in range(1,axis-1) for x in range(1,axis-1)]
                assert max(interior) > base if direction > 0 else min(interior) < base
                # The same structured command, validated through the headless
                # path, must produce the exact f32 samples of this visual stamp.
                commands = output / f'{name}.commands.jsonl'
                commands.write_text(json.dumps(dict(command='apply', expected_revision=0,
                    edits=[dict(op='brush', brush=dict(centre=[origin[0]+centre[0],origin[2]+centre[2]],
                        radius_metres=12., operation=dict(kind='flatten',height=target,blend=.5)))]))+'\n')
                reference = output / f'{name}.reference.json'
                subprocess.run([str(binary), '--headless', '--zone', str(source), '--commands', str(commands),
                                '--output', str(reference)], check=True, capture_output=True, text=True)
                assert normalized(json.loads(reference.read_text())) == normalized(edited['document'])
                chord('z')
                wait(lambda: state()['revision'] == 2)
                assert state()['document'] == initial
                chord('y')
                wait(lambda: state()['revision'] == 3)
                assert state()['document'] == edited['document']
                chord('s')
                wait(lambda: saved.exists() and not state()['unsaved_changes'])
                assert normalized(json.loads(saved.read_text())) == normalized(edited['document'])
                key('F5')
                wait(lambda: state()['play_feet'] is not None)
                assert abs(state()['play_feet'][1] - base) < .2
                key('F5')
                wait(lambda: state()['play_feet'] is None)
                assert state()['document'] == edited['document']
                if strength:
                    before = state()
                    action = 'Brush(Raise)' if direction > 0 else 'Brush(Lower)'
                    entry = next(b for b in state()['buttons'] if b['action'] == action)
                    desktop.move(window, entry['centre'], 1)
                    wait(lambda: any(b['action'] == action and b['hovered'] for b in state()['buttons']))
                    desktop.button(1, True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1, False)
                    wait(lambda: state()['brush_tool'] == ('Raise' if direction > 0 else 'Lower'))
                    for _ in range(7):
                        key('bracketleft')
                    wait(lambda: abs(state()['brush_strength']-.05) < 1e-7)
                    for _ in range(9):
                        key('bracketright')
                    wait(lambda: state()['brush_strength'] == 8.)
                    for _ in range(6):
                        key('bracketleft')
                    wait(lambda: state()['brush_strength'] == .125)
                    assert state()['revision'] == before['revision'] and state()['document'] == before['document']
                    assert not state()['unsaved_changes']
                    desktop.move(window, point, 1)
                    centre = wait(lambda: state()['brush_point'])
                    # Wait for a report after moving back from the palette.
                    wait(lambda: state()['brush_point'] and state()['brush_point'] == centre)
                    desktop.button(1, True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1, False)
                    sculpted = wait(lambda: state() if state()['revision'] == before['revision']+1 else None)
                    assert sculpted['history'] == [before['history'][0]+1,0]
                    delta = [a-b for a,b in zip(sculpted['document']['terrain']['heights'], before['document']['terrain']['heights'])]
                    assert max(abs(d) for d in delta) <= .125001
                    assert max(delta) > 0 if direction > 0 else min(delta) < 0
                    strength_source = output / f'{name}.strength.source.json'
                    strength_source.write_text(json.dumps(before['document']))
                    strength_commands = output / f'{name}.strength.commands.jsonl'
                    strength_commands.write_text(json.dumps(dict(command='apply',expected_revision=0,
                        edits=[dict(op='brush',brush=dict(centre=[origin[0]+centre[0],origin[2]+centre[2]],
                            radius_metres=12.,operation=dict(kind='raise' if direction>0 else 'lower',metres=.125)))]))+'\n')
                    strength_reference = output / f'{name}.strength.reference.json'
                    subprocess.run([str(binary),'--headless','--zone',str(strength_source),'--commands',str(strength_commands),
                                    '--output',str(strength_reference)],check=True,capture_output=True,text=True)
                    assert normalized(json.loads(strength_reference.read_text())) == normalized(sculpted['document'])
                    chord('z')
                    wait(lambda: state()['revision'] == sculpted['revision']+1)
                    assert state()['document'] == before['document'] and not state()['unsaved_changes']
                    chord('y')
                    wait(lambda: state()['revision'] == sculpted['revision']+2)
                    assert state()['document'] == sculpted['document']
                    chord('s')
                    wait(lambda: not state()['unsaved_changes'])
                    assert normalized(json.loads(saved.read_text())) == normalized(sculpted['document'])
                    key('F5')
                    wait(lambda: state()['play_feet'] is not None)
                    key('bracketleft')
                    wait(lambda: state()['status'] == 'Leave playtest before editing')
                    assert state()['brush_strength'] == .125 and state()['document'] == sculpted['document']
                    key('F5')
                    wait(lambda: state()['play_feet'] is None)
                key('F12')
                wait(lambda: capture_ready(report.with_suffix('.png')), 30)
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
                          level=target, validation_layer_active=True, allowed_resize_diagnostics=diagnostics))
    summary = dict(cases=cases, checks=['positive/negative terrain sampling and 1m/0.1m level controls without history',
        'paint-mode isolation, one-entry visual stroke and exact headless command equivalence',
        'undo/redo/save and actual offline Avian playtest on 4m and far-origin 2m terrain'],
        scope='Linux owned native input, 3440x1440 only; no Windows or multiplayer claim')
    summary['adjustable_strength'] = strength
    if strength:
        summary['checks'] += ['raise/lower strength clamps to 0.05/8m, preserves history until stamping, and refuses playtest changes',
                              '0.125m sculpt has exact matching headless f32 samples, one-entry undo/redo/save on both grids']
    (output / 'qualification.json').write_text(json.dumps(summary, indent=2)+'\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--strength', action='store_true')
    args = parser.parse_args()
    run(args.output, args.binary, args.strength)
