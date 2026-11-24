"""Owned native Start/Stop and owner-exit check for authored network playtests."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time

sys.path.insert(0, 'tools')
from launch_environment import client_environment
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from gpu_validation import check_gpu_diagnostics

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
parser.add_argument('--zone', type=Path, required=True)
parser.add_argument('--players', type=Path, required=True)
parser.add_argument('--scenery-library', type=Path, required=True)
parser.add_argument('--gameplay-template', type=Path, required=True)
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
p = None
runs = []
with (out / 'editor.log').open('w') as log:
    try:
        p = subprocess.Popen([str(args.binary.resolve()), '--zone', str(source),
            '--report', str(report), '--scenery-library', str(args.scenery_library.resolve()),
            '--player-assets', str(args.players.resolve())], env=env, stdout=log, stderr=log)
        w = wait(lambda: owned_window(p.pid), 45)
        x = Desktop()
        x.focus(w)
        def state():
            assert p.poll() is None, 'Editor exited unexpectedly'
            return json.loads(report.read_text())
        wait(lambda: report.exists() and state().get('menus'), 45)
        def fresh():
            old = state()['input_elapsed']
            wait(lambda: state()['input_elapsed'] > old)
            return state()
        def click(point):
            x.move(w, point, 1)
            fresh()
            x.owned_focus(pointer=True)
            x.button(1, True)
            x.button(1, False)
            fresh()
        def menu():
            click(next(m['centre'] for m in state()['menus']
                if m['name'] == 'Zone' and m['role'] == 'button' and min(m['size']) > 0))
        def command(name):
            c = wait(lambda: next((c for c in state()['workspace_controls']
                if c['target'] == name and c['menu_item'] and min(c['size']) > 0), None))
            click(c['centre'])
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
        def launch(index):
            x.focus(w)
            fresh()
            menu()
            command('NetworkPlaytest')
            wait(lambda: state()['export_dialog'])
            destination = out / f'snapshot-{index}'
            fill(0, destination)
            fill(3, args.gameplay_template.resolve())
            assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index'] == 4) == str(args.players.resolve())
            if index == 1:
                x.key('F12')
                wait(lambda: capture_ready(report.with_suffix('.png')))
            click(next(b['centre'] for b in state()['export_dialog']['buttons'] if b['action'] == 'Export'))
            wait(lambda: state()['network_playtest']['active'], 45)
            run = wait(lambda: next((r for r in (destination / 'runs').glob('session-*') if r.is_dir()), None), 45)
            runs.append(run)
            def finished():
                return all((run / f'client-{i}.log').exists() and '"finished":true' in (run / f'client-{i}.log').read_text() for i in (1, 2))
            wait(finished, 90)
            return run
        run = launch(1)
        x.focus(w)
        fresh()
        menu()
        command('StopNetworkPlaytest')
        wait(lambda: not state()['network_playtest']['active'], 45)
        wait(lambda: (run / 'session.json').exists())
        run = launch(2)
        # Abrupt owner death must close its private pipe without a Rust Drop.
        p.kill()
        p.wait(timeout=10)
        wait(lambda: (run / 'session.json').exists(), 45)
        for run in runs:
            result = json.loads((run / 'session.json').read_text())
            assert result['reason'] == 'stop requested' and not result['cleanup_errors'], result
            assert len(result['clients']) == 2 and all(c['started'] for c in result['clients'])
            assert all(c['relay']['overflow'] == 0 for c in result['clients'])
            server_lines = (run / 'server.log').read_text().splitlines()
            transport = next(json.loads(line.removeprefix('transport_summary ')) for line in server_lines if line.startswith('transport_summary '))
            assert transport['test_player_admissions'] == 2 and transport['test_player_refusals'] == 0
            assert transport['rejected_inputs'] == 0 and transport['rejected_chunk_ready'] == 0
            terrain = next(json.loads(line)['terrain_summary'] for line in server_lines if line.startswith('{"terrain_summary":'))
            assert terrain['movement_recoveries'] == 0
            assert not list(run.glob('tickets-*'))
            for path in run.glob('*.log'):
                assert check_gpu_diagnostics(path.read_text()) == 0
        assert source.read_bytes() == original and args.zone.read_bytes() == original
        (out / 'qualification.json').write_text(json.dumps(dict(passed=True,
            editor_start=True, editor_stop=True, abrupt_owner_exit_cleanup=True,
            two_real_clients=True, source_unchanged=True,
            runs=[str(r) for r in runs]), indent=2))
    finally:
        if x:
            x.close()
        if p and p.poll() is None:
            p.send_signal(signal.SIGINT)
            try:
                p.wait(timeout=10)
            except subprocess.TimeoutExpired:
                p.kill()
                p.wait(timeout=10)
assert check_gpu_diagnostics((out / 'editor.log').read_text()) == 0
