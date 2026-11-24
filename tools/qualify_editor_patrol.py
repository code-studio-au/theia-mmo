"""Owned native patrol creation/edit/save, followed by real offline navigation bake."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
from launch_environment import client_environment
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from gpu_validation import check_gpu_diagnostics

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
parser.add_argument('--baker', type=Path, default=Path('target/debug/theia-nav-bake'))
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source, saved, report = [out / name for name in ['source.zone.json', 'saved.zone.json', 'state.json']]
original = json.dumps(dict(schema=13, name='Patrol guide check', origin=[8192,0,8192],
    spawn=[8192,.1,8198], objects=[], terrain=dict(chunk=[63,63], span_chunks=2, heights=[0.]*(65*65)), paths=[dict(id=10, name='Existing circuit', kind='patrol',
    width_metres=.7, points=[[8184,.03,8192],[8200,.03,8192],[8200,.03,8184],[8184,.03,8192]])])).encode()
source.write_bytes(original)
env = client_environment(dict(os.environ, DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000',
    WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
    THEIA_PARTICLE_EDITOR_DATA_DIR=str(out/'particle-data')))
x = None
with (out/'editor.log').open('w') as log:
    process = subprocess.Popen([str(args.binary.resolve()), '--zone', str(source), '--save-path', str(saved),
        '--report', str(report)], env=env, stdout=log, stderr=log)
    try:
        window = wait(lambda: owned_window(process.pid), 45)
        x = Desktop(); x.focus(window)
        def state():
            assert process.poll() is None, 'Editor exited'
            return wait(lambda: json.loads(report.read_text()), 2)
        wait(lambda: report.exists() and state().get('menus'), 45)
        def fresh():
            before = state()['input_elapsed']
            return wait(lambda: state() if state()['input_elapsed'] > before else None)
        def click(point):
            x.move(window, point, 1); fresh()
            x.button(1, True); x.button(1, False); fresh()
        def chord(key):
            x.key_state('Control_L', True); fresh()
            x.key(key); fresh(); x.key_state('Control_L', False); fresh()
        def select(identifier):
            row = wait(lambda: next((r for r in state()['content_list']['controls']
                if r['target'] == f'Guide({identifier})' and min(r['size']) > 0), None))
            click(row['centre'])
            wait(lambda: state()['path_selected'] == identifier)
        def tab(name):
            click(next(c['centre'] for c in state()['property_tabs']['controls'] if c['tab'] == name))
            wait(lambda: state()['property_tabs']['active'] == name)
        def capture(name):
            picture = report.with_suffix('.png'); picture.unlink(missing_ok=True)
            x.key('F12'); wait(lambda: capture_ready(picture), 30)
            picture.rename(out / name)
        select(10); x.key('f'); fresh()
        ground = state()['ground']
        assert all(ground)
        tab('Tools')
        button = wait(lambda: next((b for b in state()['buttons'] if b['action'] == 'NewPath(Patrol)' and min(b['size']) > 0), None))
        click(button['centre']); wait(lambda: state()['path_draft_points'] == 0)
        for count, point in enumerate(ground, 1):
            click(point); wait(lambda: state()['path_draft_points'] == count)
        x.key('Return'); wait(lambda: len(state()['document']['paths']) == 2)
        identifier = state()['path_selected']
        assert state()['document']['schema'] == 13 and state()['document']['paths'][1]['kind'] == 'patrol'
        select(identifier)
        click(state()['path_handles'][0]); wait(lambda: state()['path_point'] == 0)
        tab('Attributes')
        old = state()['document']['paths'][1]['points'][0][0]
        field = wait(lambda: next((f for f in state()['inline_attributes'] if f['field'] == 0 and min(f['size']) > 0), None))
        click(field['centre']); chord('a')
        for character in str(old+.125):
            x.key('period' if character == '.' else character); fresh()
        x.key('Return'); wait(lambda: state()['document']['paths'][1]['points'][0][0] == old+.125)
        select(identifier); chord('z'); wait(lambda: state()['document']['paths'][1]['points'][0][0] == old)
        assert source.read_bytes() == original and not saved.exists()
        capture('patrol-editor.png')
        x.key('F5'); wait(lambda: state()['play_feet'] is not None)
        capture('patrol-offline-preview.png')
        x.key('F5'); wait(lambda: state()['play_feet'] is None)
        chord('s'); wait(lambda: saved.exists() and not state()['unsaved_changes'])
        assert json.loads(saved.read_text()) == state()['document'] and source.read_bytes() == original
        gpu = state()['gpu']
    finally:
        if x: x.close()
        if process.poll() is None:
            process.send_signal(signal.SIGINT)
            try: process.wait(timeout=10)
            except subprocess.TimeoutExpired: process.terminate(); process.wait(timeout=10)
assert check_gpu_diagnostics((out/'editor.log').read_text()) == 0
world = out/'world'
result = subprocess.run([str(args.binary.resolve()), '--zone', str(saved), '--export', str(world),
    '--content-id', 'theia.test.patrol-authoring', '--revision', '1'], env=env, capture_output=True, text=True, timeout=60)
(out/'export.log').write_text(result.stdout+result.stderr)
assert result.returncode == 0
export = json.loads((world/'export.json').read_text())
assert len(export['patrol_route_requests']) == 2 and export['guide_paths'] == 0
request = next(r for r in export['patrol_route_requests'] if r['id'] == f'theia.zone.patrol.{identifier}')
result = subprocess.run([str(args.baker.resolve()), 'bake-authored', str(world), str(out/'navigation'),
    str(world/request['file'])], env=env, capture_output=True, text=True, timeout=120)
(out/'bake.log').write_text(result.stdout+result.stderr)
assert result.returncode == 0, result.stderr
navigation = json.loads((out/'navigation/navigation.manifest.json').read_text())
assert navigation['routes'][0]['id'] == request['id']
(out/'qualification.json').write_text(json.dumps(dict(passed=True, gpu=gpu, native_create=True,
    inline_edit_undo=True, explicit_save=True, source_preserved=True, exported_requests=2,
    navigation_tiles=len(navigation['tiles']), scope='Editor authoring and offline bake; no creature binding or runtime patrol admission'), indent=2)+'\n')
