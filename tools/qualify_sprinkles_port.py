#!/usr/bin/env python3
"""Owned native checks of the directly reused Sprinkles authoring library."""
import argparse
import copy
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess

from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary, settings_fixture=None, widgets=False):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    fixture = output / 'effect.ron'
    shutil.copyfile('third_party/sprinkles/crates/bevy_sprinkles_editor/src/assets/examples/magic-puff.ron', fixture)
    original = fixture.read_bytes()
    report = output / 'state.json'
    env = client_environment(dict(os.environ, WGPU_BACKEND='vulkan',
        VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer',
        THEIA_PARTICLE_REPORT=str(report), THEIA_PARTICLE_EDITOR_DATA_DIR=str(output / 'editor-data')))
    if settings_fixture is not None:
        data_dir = output / 'editor-data'
        data_dir.mkdir()
        shutil.copyfile(settings_fixture, data_dir / 'editor.ron')
    desktop = Desktop()
    desktop.prepare_wheel()
    with (output / 'editor.log').open('w') as log:
        process = subprocess.Popen([str(binary), str(fixture)], env=env, stdout=log, stderr=log)
        try:
            window = wait(lambda: owned_window(process.pid), 60)
            desktop.focus(window)

            def state():
                assert process.poll() is None, 'Ported editor exited; inspect editor.log'
                return json.loads(report.read_text())

            wait(lambda: report.exists() and state()['project'] and state()['emitter_count'] > 0, 90)
            wait(lambda: state()['elapsed'] > 2)
            initial = copy.deepcopy(state())

            def click_kind(kind):
                entry = wait(lambda: next((item for item in state()['buttons'] if item['kind'] == kind), None))
                desktop.move(window, entry['centre'], 1)
                wait(lambda: any(item['kind'] == kind and item['hovered'] for item in state()['buttons']))
                desktop.button(1, True)
                try:
                    wait(lambda: state()['left_mouse_down'])
                finally:
                    desktop.button(1, False)
                wait(lambda: not state()['left_mouse_down'])

            def capture(label):
                path = report.with_suffix('.png')
                path.unlink(missing_ok=True)
                desktop.key('F12')
                wait(lambda: capture_ready(path), 30)
                path.rename(output / f'{label}.png')

            capture('reused-editor-preview')
            click_kind('play')
            wait(lambda: state()['runtime'] and all(item['paused'] for item in state()['runtime']))
            click_kind('play')
            wait(lambda: state()['runtime'] and not any(item['paused'] for item in state()['runtime']))
            click_kind('loop')
            wait(lambda: state()['runtime'] and not any(item['loop'] for item in state()['runtime']))
            click_kind('loop')
            wait(lambda: state()['runtime'] and all(item['loop'] for item in state()['runtime']))
            assert state()['project'] == initial['project'] and fixture.read_bytes() == original

            def amount_field():
                return next((field for field in state()['fields'] if field['field'] == 'emission.particles_amount'), None)

            field = wait(amount_field)
            for _ in range(64):
                field = amount_field()
                height = state()['viewport'][1]
                if 60 < field['centre'][1] < height - 40:
                    break
                prior = field['centre'][1]
                desktop.move(window, [field['centre'][0], height - 100], 1)
                desktop.scroll(-4 if prior >= height - 40 else 4)
                wait(lambda: amount_field()['centre'][1] != prior)
            else:
                raise AssertionError('Particle count field did not become visible')
            desktop.move(window, field['centre'], 1)
            desktop.button(1, True)
            desktop.button(1, False)
            desktop.key_state('Control_L', True)
            try:
                desktop.key('a')
            finally:
                desktop.key_state('Control_L', False)
            desktop.key('3')
            desktop.key('2')
            desktop.key('Return')
            wait(lambda: state()['project']['emitters'][0]['emission']['particles_amount'] == 32)
            expected = copy.deepcopy(initial['project'])
            expected['emitters'][0]['emission']['particles_amount'] = 32
            assert state()['project'] == expected and state()['dirty']
            assert fixture.read_bytes() == original
            capture('reused-emission-edit')
            def replace_text(field, value):
                desktop.click(window, field['centre'], 1, 1)
                desktop.key_state('Control_L', True)
                try:
                    desktop.key('a')
                finally:
                    desktop.key_state('Control_L', False)
                for character in value:
                    desktop.key(character)
                desktop.key('Return')

            if widgets:
                before_widgets = copy.deepcopy(state()['project'])
                for kind, path in [('curve', 'scale.scale_over_lifetime'),
                                   ('gradient', 'colors.color_over_lifetime'),
                                   ('color', 'colors.initial_color.color')]:
                    if kind == 'color':
                        variant = next(item for item in state()['buttons'] if item['field'] == 'colors.initial_color' and item['text'] == 'Solid')
                        desktop.click(window, variant['centre'], 1, 1)
                        wait(lambda: state()['popovers'] == 1)
                    def trigger():
                        return next((item for item in state()['buttons']
                            if item['widget'] == kind and item['field'] == path), None)
                    button = wait(trigger)
                    for _ in range(64):
                        button = trigger()
                        height = state()['viewport'][1]
                        if 16 < button['centre'][1] < height - 16:
                            break
                        prior = button['centre'][1]
                        desktop.move(window, [button['centre'][0], height - 100], 1)
                        desktop.scroll(-4 if prior >= height - 16 else 4)
                        wait(lambda: trigger()['centre'][1] != prior)
                    else:
                        raise AssertionError(f'{kind} trigger did not become visible')
                    previous_popovers = state()['popovers']
                    desktop.click(window, button['centre'], 1, 1)
                    wait(lambda: state()['popovers'] > previous_popovers)
                    opened = state()['elapsed']
                    wait(lambda: state()['elapsed'] > opened + 2)
                    capture(f'original-{kind}-editor')
                    if kind == 'curve':
                        maximum = next(field for field in state()['fields'] if field['field'] is None and float(field['value']) == 1.)
                        replace_text(maximum, '2')
                        wait(lambda: state()['project']['emitters'][0]['scale']['scale_over_lifetime']['x']['range']['max'] == 2)
                        curve = before_widgets['emitters'][0]['scale']['scale_over_lifetime']
                        curve['x']['range']['max'] = 2
                        curve.pop('name', None)
                    if kind == 'gradient':
                        def position_input(value):
                            return next(field for field in state()['fields'] if field['field'] is None and float(field['value'].rstrip('%')) == value)
                        replace_text(position_input(0), '25')
                        wait(lambda: state()['project']['emitters'][0]['colors']['color_over_lifetime']['stops'][0]['position'] == .25)
                        replace_text(position_input(25), '0')
                        wait(lambda: state()['project'] == before_widgets)
                    if kind == 'color':
                        alpha = next(field for field in state()['fields'] if field['field'] is None and field['value'].rstrip('%') == '100')
                        mode = next(item for item in state()['buttons'] if item['kind'] == 'button' and item['text'] == '' and item['centre'][0] > alpha['centre'][0] and abs(item['centre'][1] - alpha['centre'][1]) < 1)
                        desktop.click(window, mode['centre'], 1, 1)
                        option = wait(lambda: next((item for item in state()['buttons'] if item['text'] == 'Hex'), None))
                        desktop.click(window, option['centre'], 1, 1)
                        wait(lambda: any(field['field'] is None and field['value'].lower() == 'ffffff' for field in state()['fields']))
                        assert state()['project'] == before_widgets
                        def hex_input(value):
                            return next(field for field in state()['fields'] if field['field'] is None and field['value'].lower() == value)
                        replace_text(hex_input('ffffff'), 'ff0000')
                        wait(lambda: state()['project']['emitters'][0]['colors']['initial_color']['Solid']['color'] == [1,0,0,1])
                        replace_text(hex_input('ff0000'), 'ffffff')
                        wait(lambda: state()['project'] == before_widgets)
                    capture(f'original-{kind}-edit')
                    for _ in range(4):
                        if state()['popovers'] == 0:
                            break
                        previous_popovers = state()['popovers']
                        click_kind('close')
                        wait(lambda: state()['popovers'] < previous_popovers)
                    assert state()['popovers'] == 0
                    assert state()['project'] == before_widgets, f'Opening/closing {kind} changed the asset'
                    assert fixture.read_bytes() == original

            desktop.key_state('Control_L', True)
            try:
                desktop.key('s')
            finally:
                desktop.key_state('Control_L', False)
            wait(lambda: not state()['dirty'] and fixture.read_bytes() != original)
            final = copy.deepcopy(state())
            subprocess.run(['wmctrl', '-ic', hex(window)], check=True)
            process.wait(timeout=20)
            assert process.returncode == 0
        finally:
            desktop.close()
            if process.poll() is None:
                process.terminate()
                process.wait(timeout=15)
    logs = (output / 'editor.log').read_text()
    assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
    diagnostics = check_gpu_diagnostics(logs, allow_surface_resize=False)
    assert not re.search(r'\bERROR\b', logs), 'Unexpected editor error; inspect editor.log'
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == binary_hash
    result = dict(gpu=final['gpu'], gpu_diagnostics=diagnostics, validation_layer_active=True,
        binary_sha256=binary_hash, viewport=final['viewport'], emitter_count=final['emitter_count'],
        checks=['original Sprinkles UI and particle preview', 'native pause/resume and loop controls preserve asset/file',
                'original emission field commits only particle count through native text input', 'Ctrl S saves the RON asset'],
        widgets=widgets, settings_fixture=str(settings_fixture) if settings_fixture else None,
        scope='Linux native ported authoring tool; no embedded workspace, gameplay VFX or MMO capacity claim')
    if widgets:
        result['checks'].extend(['original curve range commits through native input', 'gradient stop position edits and restores exactly', 'colour hex edits and restores exactly', 'popover close preserves the expected effect and leaves source untouched until Save'])
    (output / 'qualification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/examples/theia_validate'))
    parser.add_argument('--settings-fixture', type=Path)
    parser.add_argument('--widgets', action='store_true', help='Open and close the original curve, gradient and colour editors through native input')
    args = parser.parse_args()
    run(args.output, args.binary, args.settings_fixture, args.widgets)
