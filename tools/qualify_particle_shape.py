#!/usr/bin/env python3
"""Native inline particle shape dropdown and radius regression."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from qualify_world_input_os import Desktop, capture_ready, owned_window, wait


def run(output, binary):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    data = output / 'particle-data'
    library = data / 'library'
    library.mkdir(parents=True)
    effect = library / 'vfx.campfire.ron'
    shutil.copyfile('third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/magic-puff.ron', effect)
    effect_before = effect.read_bytes()
    source = output / 'zone.json'
    report = output / 'state.json'
    zone = {'schema': 8, 'name': 'Inline gradient qualification', 'origin': [0, 0, 0],
            'spawn': [0, .1, 6], 'objects': [], 'particles': [
                {'id': 1, 'name': 'Campfire', 'effect': 'vfx.campfire',
                 'position': [0, 0, 0], 'yaw_degrees': 0, 'scale': 1}]}
    original = json.dumps(zone)
    source.write_text(original)
    env = client_environment(dict(os.environ, WGPU_BACKEND='vulkan',
        VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer',
        THEIA_PARTICLE_EDITOR_DATA_DIR=str(data)))
    desktop = Desktop()
    desktop.prepare_wheel()
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

            def click(point, button=1):
                desktop.move(window, point, 1)
                wait(lambda: state()['cursor'] and max(abs(a-b) for a, b in zip(state()['cursor'], point)) < 2)
                desktop.button(button, True)
                desktop.button(button, False)

            def particle():
                return state()['particle_workspace']

            def selection_intact():
                current = state()
                return current['particle_selected'] == 1 and current['property_tabs']['active'] == 'Attributes' and current['particle_workspace']['inline']

            row = wait(lambda: next((e for e in state()['content_list']['controls'] if e['target'] == 'Particle(1)' and min(e['size']) > 0), None))
            click(row['centre'])
            wait(selection_intact, 60)

            def shape_button():
                return next((e for e in particle()['widget_buttons'] if e['widget'] == 'variant' and e['field'] == 'emission.shape'), None)

            wait(shape_button)
            for _ in range(20):
                button = shape_button()
                if 650 < button['centre'][1] < 1330:
                    break
                old_y = button['centre'][1]
                desktop.move(window, [276, 1100], 1)
                desktop.scroll(-8 if old_y > 1330 else 8)
                wait(lambda: shape_button()['centre'][1] != old_y)
            else:
                raise AssertionError('Shape control did not scroll into view')

            click(shape_button()['centre'])
            wait(lambda: len(particle()['popovers']) == 1)
            width = state()['ui_layout']['inspector'][2]
            divider = state()['ui_layout']['canvas'][0]

            def choose(current, target):
                combo = wait(lambda: next((e for e in particle()['widget_buttons'] if e['widget'] is None and e['text'] == current and min(e['size']) > 0), None))
                assert abs(divider - combo['centre'][0]) < combo['size'][0] / 2
                click([divider, combo['centre'][1]])
                wait(lambda: len(particle()['popovers']) == 2)
                option = wait(lambda: next((e for e in particle()['widget_buttons'] if e['widget'] is None and e['text'] == target and min(e['size']) > 0), None))
                assert abs(divider - option['centre'][0]) < option['size'][0] / 2
                click([divider, option['centre'][1]])
                wait(lambda: len(particle()['popovers']) == 1 and shape_button()['text'] == target)
                assert selection_intact() and state()['ui_layout']['inspector'][2] == width
                assert not state()['ui_layout']['resizing']

            choose('Point', 'Sphere')
            radius = wait(lambda: next((e for e in particle()['fields'] if e['field'] == 'emission.shape.radius' and min(e['size']) > 0), None))
            assert radius['value'] == '1.0'
            click(radius['centre'])
            desktop.key_state('Control_L', True)
            try:
                desktop.key('a')
            finally:
                desktop.key_state('Control_L', False)
            desktop.key('2')
            desktop.key('Return')
            wait(lambda: particle()['project']['emitters'][0]['emission']['shape']['Sphere']['radius'] == 2.)
            assert selection_intact() and state()['ui_layout']['inspector'][2] == width
            desktop.key('F12')
            wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            report.with_suffix('.png').rename(output / 'shape-radius.png')

            choose('Sphere', 'Point')
            wait(lambda: not any(e['field'] == 'emission.shape.radius' for e in particle()['fields']))
            choose('Point', 'Ring')
            wait(lambda: all(any(e['field'] == 'emission.shape.' + field for e in particle()['fields']) for field in ('radius', 'inner_radius')))
            assert selection_intact() and state()['ui_layout']['inspector'][2] == width
            assert source.read_text() == original and effect.read_bytes() == effect_before
            gpu = state()['gpu']
        finally:
            desktop.close()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    log = (output / 'editor.log').read_text()
    assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in log
    diagnostics = check_gpu_diagnostics(log, allow_surface_resize=False)
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == binary_hash
    result = {'gpu': gpu, 'binary_sha256': binary_hash, 'gpu_diagnostics': diagnostics,
              'scope': 'Linux native; shape dropdown over divider, live radius edit, fieldless and fieldful shape switching, selection and source preservation'}
    (output / 'qualification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output, args.binary)
