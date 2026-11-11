#!/usr/bin/env python3
"""Exercise particle deletion and guarded exit through the real native menus."""
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
    source = output / 'source.zone.json'
    report = output / 'state.json'
    zone = {
        'schema': 8, 'name': 'Particle delete and exit qualification',
        'origin': [0, 0, 0], 'spawn': [0, .1, 6], 'objects': [],
        'particles': [{'id': 1, 'name': 'New particle', 'effect': 'vfx.unassigned',
                       'position': [0, 0, 0], 'yaw_degrees': 0, 'scale': 1}],
    }
    original = json.dumps(zone)
    source.write_text(original)
    data = output / 'particle-editor-data'
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

            def click(point):
                desktop.move(window, point, 1)
                wait(lambda: state()['cursor'] and max(abs(a-b) for a, b in zip(state()['cursor'], point)) < 2)
                desktop.button(1, True)
                wait(lambda: state()['left_mouse_down'])
                desktop.button(1, False)
                wait(lambda: not state()['left_mouse_down'])

            def menu(name):
                entry = next(e for e in state()['menus'] if e['name'] == name and e['role'] == 'button' and min(e['size']) > 0)
                click(entry['centre'])
                return wait(lambda: next((e for e in state()['menus'] if e['name'] == name and e['role'] == 'popup' and min(e['size']) > 0), None))

            def command(name, target):
                popup = menu(name)
                def inside(e):
                    return all(abs(e['centre'][i] - popup['centre'][i]) < popup['size'][i] / 2 for i in (0, 1))
                entry = wait(lambda: next((e for e in state()['workspace_controls'] if e['target'] == target and e['menu_item'] and min(e['size']) > 0 and inside(e)), None))
                click(entry['centre'])

            def select_particle():
                row = wait(lambda: next((e for e in state()['content_list']['controls'] if e['target'] == 'Particle(1)' and min(e['size']) > 0), None))
                click(row['centre'])
                wait(lambda: state()['particle_selected'] == 1)

            def chord(key):
                desktop.key_state('Control_L', True)
                try:
                    desktop.key(key)
                finally:
                    desktop.key_state('Control_L', False)

            select_particle()
            command('Property actions', 'Command(Delete)')
            wait(lambda: not state()['document'].get('particles', []))
            assert state()['unsaved_changes'] and source.read_text() == original
            chord('z')
            wait(lambda: len(state()['document'].get('particles', [])) == 1)
            select_particle()
            command('Zone', 'Command(Delete)')
            wait(lambda: not state()['document'].get('particles', []))
            assert state()['unsaved_changes'] and source.read_text() == original

            command('File', 'Exit')
            wait(lambda: state()['exit_confirmation'])
            desktop.key('F12')
            wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            report.with_suffix('.png').rename(output / 'exit-confirmation.png')
            desktop.key('Escape')
            wait(lambda: not state()['exit_confirmation'])
            assert state()['unsaved_changes'] and not state()['document'].get('particles', [])
            command('File', 'Exit')
            wait(lambda: state()['exit_confirmation'])
            desktop.key('Tab')
            desktop.key('Return')
            process.wait(timeout=20)
            assert process.returncode == 0
            assert source.read_text() == original
        finally:
            desktop.close()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    text = (output / 'editor.log').read_text()
    assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in text
    diagnostics = check_gpu_diagnostics(text, allow_surface_resize=False)
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == binary_hash
    final = json.loads(report.read_text())
    result = {'gpu': final['gpu'], 'binary_sha256': binary_hash, 'gpu_diagnostics': diagnostics,
              'scope': 'Linux native; particle deletion through Properties and Zone menus, undo, dirty exit cancel and confirmation; source preserved'}
    (output / 'qualification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output, args.binary)
