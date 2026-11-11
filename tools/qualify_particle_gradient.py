#!/usr/bin/env python3
"""Native inline gradient, selection and dock-divider regression."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

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

            def stops():
                return particle()['project']['emitters'][0]['colors']['color_over_lifetime']['stops']

            def selection_intact():
                current = state()
                return current['particle_selected'] == 1 and current['property_tabs']['active'] == 'Attributes' and current['particle_workspace']['inline']

            row = wait(lambda: next((e for e in state()['content_list']['controls'] if e['target'] == 'Particle(1)' and min(e['size']) > 0), None))
            click(row['centre'])
            wait(selection_intact, 60)

            def gradient_button():
                return next((e for e in particle()['widget_buttons'] if e['widget'] == 'gradient' and e['field'] == 'colors.color_over_lifetime'), None)

            wait(gradient_button)
            for _ in range(20):
                button = gradient_button()
                if 650 < button['centre'][1] < 1330:
                    break
                old_y = button['centre'][1]
                desktop.move(window, [276, 1100], 1)
                desktop.scroll(-8 if old_y > 1330 else 8)
                wait(lambda: gradient_button()['centre'][1] != old_y)
            else:
                raise AssertionError('Gradient control did not scroll into view')

            started = time.monotonic()
            click(gradient_button()['centre'])
            wait(lambda: len(particle()['popovers']) == 1 and len(particle()['gradient_handles']) == 2, 60)
            opening_seconds = time.monotonic() - started
            assert selection_intact()
            initial_width = state()['ui_layout']['inspector'][2]
            handle = next(h for h in particle()['gradient_handles'] if h['index'] == 0)
            # Its leftmost handle overlaps the divider in the default dock layout.
            divider = state()['ui_layout']['canvas'][0]
            assert handle['centre'][0] < divider < handle['centre'][0] + 30
            desktop.move(window, handle['centre'], 1)
            desktop.button(1, True)
            wait(lambda: state()['left_mouse_down'])
            desktop.move(window, [handle['centre'][0] + 30, handle['centre'][1]], 1)
            desktop.button(1, False)
            wait(lambda: not state()['left_mouse_down'])
            wait(lambda: stops()[0]['position'] > .02)
            assert len(particle()['popovers']) == 1, 'Drag opened an unwanted color picker'
            assert state()['ui_layout']['inspector'][2] == initial_width and not state()['ui_layout']['resizing']
            assert selection_intact()

            bar = wait(lambda: particle()['gradient_bars'][0] if particle()['gradient_bars'] else None)
            click(bar['centre'], button=3)
            wait(lambda: len(stops()) == 3)
            wait(lambda: len(particle()['gradient_handles']) == 3)
            middle = next(h for h in particle()['gradient_handles'] if h['index'] == 1)
            click(middle['centre'], button=3)
            wait(lambda: len(particle()['popovers']) >= 2)
            delete = wait(lambda: next((e for e in particle()['widget_buttons'] if e['text'] == 'Delete' and min(e['size']) > 0), None))
            click(delete['centre'])
            wait(lambda: len(stops()) == 2 and len(particle()['gradient_handles']) == 2)
            wait(lambda: len([e for e in particle()['widget_buttons'] if e['widget'] == 'color']) == 2)
            assert selection_intact()

            color = wait(lambda: next((e for e in particle()['widget_buttons'] if e['widget'] == 'color' and e['text'] == '00FFFF'), None))
            before_color = list(stops()[0]['color'])
            color_started = time.monotonic()
            click(color['centre'])
            wait(lambda: len(particle()['popovers']) == 2, 60)
            color_open_seconds = time.monotonic() - color_started
            picker = max(particle()['popovers'], key=lambda e: e['size'][1])
            click([picker['centre'][0], picker['centre'][1] - picker['size'][1] / 2 + 110])
            wait(lambda: stops()[0]['color'] != before_color)
            assert selection_intact() and state()['ui_layout']['inspector'][2] == initial_width
            assert source.read_text() == original and effect.read_bytes() == effect_before
            desktop.key('F12')
            wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            report.with_suffix('.png').rename(output / 'gradient-color.png')
            saved_stops = [dict(stop) for stop in stops()]
            save = wait(lambda: next((e for e in state()['workspace_controls'] if e['target'] == 'Command(Save)' and not e['menu_item'] and min(e['size']) > 0), None))
            click(save['centre'])
            wait(lambda: effect.read_bytes() != effect_before and not particle()['dirty'], 60)
            assert selection_intact()
            saved_zone = source.read_bytes()
            gpu = state()['gpu']
        finally:
            desktop.close()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    # Reopen the files in a fresh editor to verify the saved value itself, not
    # merely that an asynchronous write completed or changed the file bytes.
    report = output / 'reopened-state.json'
    desktop = Desktop()
    with (output / 'reopened-editor.log').open('w') as log:
        process = subprocess.Popen([str(binary), '--zone', str(source), '--report', str(report)],
                                   env=env, stdout=log, stderr=log)
        try:
            window = wait(lambda: owned_window(process.pid), 60)
            desktop.focus(window)
            def reopened():
                assert process.poll() is None, 'Reopened editor exited unexpectedly'
                return json.loads(report.read_text())
            wait(lambda: report.exists() and reopened()['viewport'] == [3440, 1440], 60)
            row = wait(lambda: next((e for e in reopened()['content_list']['controls'] if e['target'] == 'Particle(1)' and min(e['size']) > 0), None))
            desktop.click(window, row['centre'], 1, 1)
            wait(lambda: reopened()['particle_workspace']['inline'], 60)
            wait(lambda: reopened()['particle_workspace']['project']['emitters'][0]['colors']['color_over_lifetime']['stops'] == saved_stops, 30)
            assert source.read_bytes() == saved_zone
        finally:
            desktop.close()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    log = (output / 'editor.log').read_text()
    log += (output / 'reopened-editor.log').read_text()
    assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in log
    diagnostics = check_gpu_diagnostics(log, allow_surface_resize=False)
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == binary_hash
    result = {'gpu': gpu, 'binary_sha256': binary_hash, 'gpu_diagnostics': diagnostics,
              'gradient_open_seconds': opening_seconds, 'color_open_seconds': color_open_seconds,
              'scope': 'Linux native; inline gradient drag, stop add/delete, color edit, explicit Save and fresh-editor reload'}
    (output / 'qualification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output, args.binary)
