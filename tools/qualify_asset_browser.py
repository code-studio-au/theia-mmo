#!/usr/bin/env python3
"""Owned Linux editor input check: model browsing, placement/history and dock isolation."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess

from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from qualify_world_input_os import Desktop, capture_ready, owned_window, wait


def run(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    source = out / 'source.zone.json'
    original = args.zone.read_bytes()
    source.write_bytes(original)
    report = out / 'state.json'
    env = client_environment(dict(os.environ, WGPU_BACKEND='vulkan',
                                 VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
                                 THEIA_PARTICLE_EDITOR_DATA_DIR=str(out / 'particle-data')))
    desktop, process = None, None
    with (out / 'editor.log').open('w') as log:
        try:
            process = subprocess.Popen([str(args.binary.resolve()), '--zone', str(source),
                                        '--scenery-library', str(args.library.resolve()),
                                        '--report', str(report)], env=env, stdout=log, stderr=log)
            window = wait(lambda: owned_window(process.pid), 45)
            desktop = Desktop()
            desktop.focus(window)

            def state():
                assert process.poll() is None, 'Owned editor exited'
                return json.loads(report.read_text())

            wait(lambda: report.exists() and state().get('asset_search') is not None, 45)

            def fresh():
                elapsed = state()['input_elapsed']
                wait(lambda: state()['input_elapsed'] > elapsed)
                return state()

            def control(target):
                return wait(lambda: next((c for c in state()['workspace_controls']
                                          if c['target'] == target and min(c['size']) > 0
                                          and not c['menu_item']), None))

            def click(point):
                desktop.move(window, point, 1)
                fresh()
                wait(lambda: (desktop.owned_focus(pointer=True), True)[1], 10)
                desktop.button(1, True)
                desktop.button(1, False)
                fresh()

            def chord(key):
                desktop.key_state('Control_L', True)
                desktop.key(key)
                desktop.key_state('Control_L', False)
                fresh()

            initial = state()['document']
            click(control('AssetBrowser')['centre'])
            wait(lambda: state()['ui_layout']['asset_browser']['open'])
            left = state()['ui_layout']['browser']
            inspector = state()['ui_layout']['inspector']
            category = next(m for m in state()['menus'] if m['name'] == 'Category' and m['role'] == 'button')
            click(category['centre'])
            category_index = 1 + sorted({a['category'] for a in json.loads(args.library.read_text())['assets']}).index('Buildings')
            item = wait(lambda: next((c for c in state()['workspace_controls']
                        if c['target'] == f'AssetCategory({category_index})' and min(c['size']) > 0), None))
            click(item['centre'])
            wait(lambda: state()['ui_layout']['asset_browser']['category'] == category_index)
            click(state()['asset_search']['centre'])
            for letter in 'hut': desktop.key(letter)
            wait(lambda: state()['asset_search']['value'] == 'hut')
            desktop.key('Return')
            fresh()
            assert state()['document'] == initial
            cards = [c for c in state()['workspace_controls'] if c['target'].startswith('AssetSelect(') and min(c['size']) > 0]
            assert len(cards) == 2, cards
            click(cards[0]['centre'])
            assert state()['document'] == initial
            click(control('AssetPlace')['centre'])
            wait(lambda: len(state()['document'].get('scenery', [])) == len(initial.get('scenery', [])) + 1)
            chord('z')
            wait(lambda: state()['document'] == initial)
            # Drag only the owned asset divider. The left stack must not change.
            before = state()['ui_layout']['asset_browser']['height']
            canvas = state()['ui_layout']['canvas']
            point = [canvas[0] + canvas[2] / 2, canvas[1] + canvas[3]]
            desktop.move(window, point, 1)
            fresh()
            desktop.button(1, True)
            wait(lambda: state()['ui_layout']['resizing'])
            desktop.move(window, [point[0], point[1] - 70], 1)
            wait(lambda: state()['ui_layout']['asset_browser']['height'] > before + 50)
            desktop.button(1, False)
            fresh()
            assert state()['ui_layout']['browser'] == left
            assert state()['ui_layout']['inspector'] == inspector
            assert state()['document'] == initial
            # Actual native screenshot, then hide browser without saving source.
            desktop.key('F12')
            wait(lambda: capture_ready(report.with_suffix('.png')))
            click(control('AssetBrowser')['centre'])
            wait(lambda: not state()['ui_layout']['asset_browser']['open'])
            assert source.read_bytes() == original
            result = dict(passed=True, gpu=state()['gpu'], category_selection=True,
                          search=True, placement_undo=True, independent_resize=True,
                          source_unchanged=True, native_linux=True, windows_tested=False)
        finally:
            if desktop is not None: desktop.close()
            if process is not None and process.poll() is None:
                process.send_signal(signal.SIGINT)
                try: process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.terminate()
                    process.wait(timeout=10)
    result['gpu_diagnostics'] = check_gpu_diagnostics((out / 'editor.log').read_text())
    (out / 'qualification.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--zone', type=Path, required=True)
    parser.add_argument('--library', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    run(parser.parse_args())
