#!/usr/bin/env python3
"""Native menu clicks at the dock divider must dispatch without resizing."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from qualify_world_input_os import Desktop, capture_ready, owned_window, wait


def run(output, binary):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    source, report = output / 'zone.json', output / 'state.json'
    zone = {'schema': 8, 'name': 'Menu divider qualification', 'origin': [0, 0, 0],
            'spawn': [0, .1, 6], 'objects': [], 'particles': [
                {'id': 1, 'name': 'Particle', 'effect': 'vfx.unassigned',
                 'position': [0, 0, 0], 'yaw_degrees': 0, 'scale': 1}]}
    original = json.dumps(zone)
    source.write_text(original)
    data = output / 'particle-data'
    data.mkdir()
    env = client_environment(dict(os.environ, WGPU_BACKEND='vulkan',
        VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer',
        THEIA_PARTICLE_EDITOR_DATA_DIR=str(data)))
    desktop = Desktop()
    with (output / 'editor.log').open('w') as log:
        process = subprocess.Popen([str(binary), '--zone', str(source), '--report', str(report)],
                                   env=env, stdout=log, stderr=log)
        try:
            window = wait(lambda: owned_window(process.pid), 60)
            desktop.focus(window)

            def state():
                assert process.poll() is None, 'Editor exited unexpectedly'
                return json.loads(report.read_text())

            wait(lambda: report.exists() and state()['viewport'] == [3440, 1440], 60)

            def move(point):
                desktop.move(window, point, 1)
                wait(lambda: state()['cursor'] and max(abs(a-b) for a, b in zip(state()['cursor'], point)) < 2)

            def click(point):
                move(point)
                desktop.button(1, True)
                wait(lambda: state()['left_mouse_down'])
                desktop.button(1, False)
                wait(lambda: not state()['left_mouse_down'])

            def menu_trigger(name):
                return next(e for e in state()['menus'] if e['name'] == name and e['role'] == 'button' and min(e['size']) > 0)

            def move_divider(target):
                old = state()['ui_layout']['canvas'][0]
                move([old, 850])
                desktop.button(1, True)
                wait(lambda: state()['left_mouse_down'] and state()['ui_layout']['resizing'])
                move([target, 850])
                wait(lambda: abs(state()['ui_layout']['canvas'][0] - target) < 2)
                desktop.button(1, False)
                wait(lambda: not state()['left_mouse_down'] and not state()['ui_layout']['resizing'])

            def open_menu(name):
                click(menu_trigger(name)['centre'])
                return wait(lambda: next((e for e in state()['menus'] if e['name'] == name and e['role'] == 'popup' and min(e['size']) > 0), None))

            def click_on_divider(popup, target):
                edge = state()['ui_layout']['canvas'][0]
                def contains(e):
                    return all(abs(e['centre'][i] - popup['centre'][i]) < popup['size'][i] / 2 for i in (0, 1))
                item = wait(lambda: next((e for e in state()['workspace_controls'] if e['target'] == target and e['menu_item'] and min(e['size']) > 0 and contains(e)), None))
                assert abs(edge - item['centre'][0]) < item['size'][0] / 2 - 2, (edge, item, popup)
                width = state()['ui_layout']['inspector'][2]
                move([edge, item['centre'][1]])
                desktop.button(1, True)
                wait(lambda: state()['left_mouse_down'])
                assert not state()['ui_layout']['resizing']
                desktop.button(1, False)
                wait(lambda: not state()['left_mouse_down'])
                assert state()['ui_layout']['inspector'][2] == width

            move_divider(menu_trigger('View')['centre'][0] + 10)
            popup = open_menu('View')
            desktop.key('F12')
            wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            report.with_suffix('.png').rename(output / 'view-menu-divider.png')
            click_on_divider(popup, 'View(Top)')
            wait(lambda: state()['view']['preset'] == 'Top')

            particle = wait(lambda: next((e for e in state()['content_list']['controls'] if e['target'] == 'Particle(1)' and min(e['size']) > 0), None))
            click(particle['centre'])
            wait(lambda: state()['particle_selected'] == 1)
            move_divider(menu_trigger('Zone')['centre'][0] + 10)
            popup = open_menu('Zone')
            click_on_divider(popup, 'Command(Delete)')
            wait(lambda: not state()['document'].get('particles', []))
            assert source.read_text() == original
            gpu = state()['gpu']
        finally:
            desktop.close()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    text = (output / 'editor.log').read_text()
    assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in text
    diagnostics = check_gpu_diagnostics(text, allow_surface_resize=False)
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == binary_hash
    result = {'gpu': gpu, 'binary_sha256': binary_hash, 'gpu_diagnostics': diagnostics,
              'scope': 'Linux native; View and Zone menu commands pressed directly over the dock divider'}
    (output / 'qualification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output, args.binary)
