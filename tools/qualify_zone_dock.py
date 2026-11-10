#!/usr/bin/env python3
"""Guarded native Linux checks for the stacked left dock and compact status bar."""
import argparse, sys, os, json, hashlib, shutil, subprocess
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
from qualify_world_input_os import Desktop, wait, owned_window, capture_ready
from launch_environment import client_environment
from gpu_validation import check_gpu_diagnostics
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--case', choices=['local', 'far'], default='local')
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
origin = [0.125, 0.25, -0.5] if args.case == 'local' else [128000000000.125, 20.25, -128000000000.5]
document = dict(schema=1, name='Dock and status bar acceptance', origin=origin, spawn=[origin[0], origin[1] + 0.0625, origin[2] + 6], objects=[dict(id=i, name=f'Authored box {i}', primitive='box', pose=dict(position=[origin[0] + 8.125 if i == 1 else origin[0] - 64 + i % 7 * 5, origin[1] + 1.25, origin[2] - 8.5 if i == 1 else origin[2] - 64 + i // 7 * 5], yaw_degrees=17.25, size=[2, 2.5, 3])) for i in range(1, 44)])
source = out / 'source.json'
source.write_text(json.dumps(document))
original = source.read_bytes()
report = out / 'state.json'
binary = Path('target/debug/theia-zone-editor').resolve()
data = out / 'particle-data'
data.mkdir()
env = client_environment(dict(os.environ, WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', THEIA_PARTICLE_EDITOR_DATA_DIR=str(data)))
desktop = Desktop()
process = None
try:
    with (out / 'editor.log').open('w') as log:
        process = subprocess.Popen([str(binary), '--zone', str(source), '--report', str(report)], env=env, stdout=log, stderr=log)
        window = wait(lambda: owned_window(process.pid), 45)
        desktop.focus(window)

        def state():
            assert process.poll() is None
            return json.loads(report.read_text())
        wait(lambda: report.exists() and state()['viewport'] == [3440, 1440], 45)

        def fresh():
            elapsed = state()['input_elapsed']
            wait(lambda: state()['input_elapsed'] > elapsed)
            return state()

        def click(point):
            desktop.click(window, point, 1, 1)
            fresh()

        def capture(name):
            image = report.with_suffix('.png')
            image.unlink(missing_ok=True)
            desktop.key('F12')
            wait(lambda: capture_ready(image))
            shutil.copyfile(image, out / name)

        def chord(key):
            desktop.key_state('Control_L', True)
            desktop.key(key)
            desktop.key_state('Control_L', False)
            fresh()
        initial = state()['document']
        history = state()['history']
        revision = state()['revision']
        assert state()['ui_layout']['status_bar'] == [0, 1412, 3440, 28]
        assert state()['ui_layout']['inspector'][0] == state()['ui_layout']['browser'][0]
        layout = state()['ui_layout']
        edge = [layout['canvas'][0], 300]
        desktop.move(window, edge, 1)
        desktop.button(1, True)
        desktop.move(window, [edge[0] + 60, edge[1]], 1)
        desktop.button(1, False)
        wait(lambda: state()['ui_layout']['browser'][2] == 420)
        assert state()['ui_layout']['status_bar'] == [0, 1412, 3440, 28]
        row = next((c for c in state()['content_list']['controls'] if c['target'] == 'Object(1)'))
        click(row['centre'])
        wait(lambda: state()['selected'] is not None)
        started = state()['input_elapsed']
        wait(lambda: state()['input_elapsed'] > started + 3)
        capture('status-bar-selected.png')
        click([1200, 1426])
        assert state()['revision'] == revision
        layout = state()['ui_layout']
        click([layout['browser'][0] + 80, layout['browser'][1] + 16])
        wait(lambda: state()['ui_layout']['panel_collapsed']['explorer'])
        assert state()['ui_layout']['browser'][3] == 32
        click([layout['browser'][0] + 80, layout['browser'][1] + 16])
        wait(lambda: not state()['ui_layout']['panel_collapsed']['explorer'])
        layout = state()['ui_layout']
        split = [layout['browser'][0] + 100, layout['inspector'][1]]
        desktop.move(window, split, 1)
        desktop.button(1, True)
        desktop.move(window, [split[0], split[1] + 80], 1)
        desktop.button(1, False)
        wait(lambda: state()['ui_layout']['inspector'][1] > layout['inspector'][1] + 60)
        layout = state()['ui_layout']
        click([layout['inspector'][0] + 80, layout['inspector'][1] + 16])
        wait(lambda: state()['ui_layout']['panel_collapsed']['properties'])
        assert state()['ui_layout']['inspector'][3] == 32
        click([layout['inspector'][0] + 80, 1380 + 16])
        wait(lambda: not state()['ui_layout']['panel_collapsed']['properties'])
        trigger = next((c for c in state()['menus'] if c['name'] == 'Content filter' and c['role'] == 'button'))
        click(trigger['centre'])
        wait(lambda: any((c['name'] == 'Content filter' and c['role'] == 'popup' and (min(c['size']) > 0) for c in state()['menus'])))
        capture('filter-dropdown.png')
        item = next((c for c in state()['content_list']['controls'] if c['target'] == 'Filter(Objects)' and min(c['size']) > 0))
        click(item['centre'])
        wait(lambda: state()['content_list']['filter'] == 'Objects')
        click(trigger['centre'])
        wait(lambda: any((c['target'] == 'Filter(All)' and min(c['size']) > 0 for c in state()['content_list']['controls'])))
        item = next((c for c in state()['content_list']['controls'] if c['target'] == 'Filter(All)'))
        click(item['centre'])
        wait(lambda: state()['content_list']['filter'] == 'All')
        assert state()['document'] == initial and state()['history'] == history and (source.read_bytes() == original)
        chord('d')
        wait(lambda: state()['revision'] > revision)
        assert state()['unsaved_changes']
        capture('status-bar-unsaved.png')
        chord('z')
        wait(lambda: state()['document'] == initial)
        chord('s')
        wait(lambda: not state()['unsaved_changes'])
        while state()['content_list']['page'] > 0:
            click(next((c for c in state()['content_list']['controls'] if c['target'] == 'Previous'))['centre'])
        click(next((c for c in state()['content_list']['controls'] if c['target'] == 'Object(1)'))['centre'])
        wait(lambda: state()['selected'] == 1)
        capture('status-bar-final.png')
        result = dict(binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), adapter=state()['gpu'], layout=state()['ui_layout'], revision=state()['revision'], history=state()['history'], source_preserved_before_save=True, source_json_preserved=json.loads(source.read_text()) == initial)
finally:
    desktop.close()
    if process is not None and process.poll() is None:
        process.terminate()
        process.wait(timeout=15)
if process.returncode not in [0, -15]:
    raise AssertionError(process.returncode)
result['gpu_diagnostics'] = check_gpu_diagnostics((out / 'editor.log').read_text())
(out / 'qualification.json').write_text(json.dumps(result, indent=2))
print(json.dumps(result))
