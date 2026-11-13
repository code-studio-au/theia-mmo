"""Qualify passive population export through an owned native editor window."""
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
parser.add_argument('--zone', type=Path, required=True)
parser.add_argument('--creature', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
original = args.zone.read_bytes()
source = out / 'source.zone.json'
source.write_bytes(original)
report = out / 'state.json'
env = client_environment(dict(os.environ, DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000',
    WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
    THEIA_PARTICLE_EDITOR_DATA_DIR=str(out / 'particle-data')))
x = None
with (out / 'editor.log').open('w') as log:
    p = subprocess.Popen([str(args.binary.resolve()), '--zone', str(source), '--report', str(report)],
        env=env, stdout=log, stderr=log)
    try:
        w = wait(lambda: owned_window(p.pid), 45)
        x = Desktop(); x.focus(w)
        def state():
            assert p.poll() is None, 'Editor exited unexpectedly'
            return json.loads(report.read_text())
        wait(lambda: report.exists() and state().get('menus'), 45)
        def fresh():
            old = state()['input_elapsed']
            return wait(lambda: state() if state()['input_elapsed'] > old else None)
        def click(point):
            x.move(w, point, 1); fresh()
            x.owned_focus(pointer=True)
            x.button(1, True); x.button(1, False); fresh()
        def fill(index, value):
            field = wait(lambda: next((f for f in state()['export_dialog']['fields'] if f['index'] == index and min(f['size']) > 0), None))
            if field['text'] == str(value):
                return
            click(field['centre'])
            wait(lambda: any(f['index'] == index and f['focused'] for f in state()['export_dialog']['fields']))
            x.key_state('Control_L', True)
            fresh()
            x.key('a')
            fresh()
            x.key_state('Control_L', False)
            fresh()
            names = {'/':'slash', '.':'period', '-':'minus', '_':'underscore', ':':'colon'}
            for char in str(value):
                key = names.get(char, char)
                x.key(key)
                fresh()
            fresh()
            assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index'] == index) == str(value)

        click(next(m['centre'] for m in state()['menus']
            if m['name'] == 'File' and m['role'] == 'button' and min(m['size']) > 0))
        control = wait(lambda: next((c for c in state()['workspace_controls']
            if c['target'] == 'ExportPopulation' and c['menu_item'] and min(c['size']) > 0), None))
        click(control['centre'])
        wait(lambda: state()['export_dialog'])
        destination = out / 'package'
        fill(0, destination)
        fill(3, args.creature.resolve())
        fill(5, '33')
        def submit():
            click(next(b['centre'] for b in state()['export_dialog']['buttons'] if b['action'] == 'Export'))
        submit()
        wait(lambda: 'Minimum spacing' in state()['export_dialog']['message'])
        assert not destination.exists()
        fill(5, '3')
        x.key('F12'); wait(lambda: capture_ready(report.with_suffix('.png')), 30)
        report.with_suffix('.png').rename(out / 'dialog.png')
        submit()
        wait(lambda: state()['export_dialog']['finished'], 60)
        result = json.loads((destination / 'population-export.json').read_text())['population_export']
        assert result['passed'] and result['count'] == 5, result
        assert source.read_bytes() == original and args.zone.read_bytes() == original
        assert json.loads((destination / 'zone.json').read_text()) == json.loads(original)
        x.key('F12'); wait(lambda: capture_ready(report.with_suffix('.png')), 30)
        report.with_suffix('.png').rename(out / 'finished.png')
        (out / 'qualification.json').write_text(json.dumps(dict(passed=True,
            invalid_spacing_refused=True, count=result['count'], source_unchanged=True,
            package=str(destination)), indent=2))
    finally:
        if x: x.close()
        if p.poll() is None:
            p.send_signal(signal.SIGINT)
            try: p.wait(timeout=10)
            except subprocess.TimeoutExpired:
                p.kill(); p.wait(timeout=10)
assert check_gpu_diagnostics((out / 'editor.log').read_text()) == 0
