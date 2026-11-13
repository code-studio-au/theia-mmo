"""Qualify population Start/Stop through an owned native editor window."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import time
from launch_environment import client_environment
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from gpu_validation import check_gpu_diagnostics

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
parser.add_argument('--zone', type=Path, required=True)
parser.add_argument('--creature', type=Path, required=True)
parser.add_argument('--players', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--assign-patrol', action='store_true', help='Assign the first authored patrol to player one through the editor, including undo')
parser.add_argument('--creature-asset', type=Path, help='Optional embedded creature appearance to seed and freeze through the dialog')
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
original = args.zone.read_bytes()
source = out / 'source.zone.json'
source.write_bytes(original)
report = out / 'state.json'
env = client_environment(dict(os.environ, DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000',
    RUST_LOG='info', WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
    THEIA_PARTICLE_EDITOR_DATA_DIR=str(out / 'particle-data')))
x = None
with (out / 'editor.log').open('w') as log:
    p = subprocess.Popen([str(args.binary.resolve()), '--zone', str(source), '--report', str(report), '--player-assets', str(args.players.resolve()),
        *(['--creature-asset', str(args.creature_asset.resolve())] if args.creature_asset else [])],
        env=env, stdout=log, stderr=log)
    try:
        w = wait(lambda: owned_window(p.pid), 45)
        x = Desktop(); x.focus(w)
        def state():
            assert p.poll() is None, 'Editor exited unexpectedly'
            return json.loads(report.read_text())
        wait(lambda: report.exists() and state().get('menus'), 45)
        # Initial font/window layout can move menu bounds after the first report.
        # Wait for stable native geometry before using any reported click centre.
        previous = None
        stable_since = time.monotonic()
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            geometry = [(m['name'],m['centre'],m['size']) for m in state()['menus'] if m['role']=='button']
            if geometry != previous:
                previous = geometry
                stable_since = time.monotonic()
            elif time.monotonic()-stable_since >= .75:
                break
            time.sleep(.05)
        else:
            raise TimeoutError('Editor startup menu geometry did not settle')
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

        assigned = None
        if args.assign_patrol:
            def menu(name):
                click(next(m['centre'] for m in state()['menus'] if m['name']==name and m['role']=='button' and min(m['size'])>0))
            player = state()['document']['test_players'][0]
            path = next(path for path in state()['document']['paths'] if path['kind']=='patrol')
            assigned = f"theia.zone.patrol.{path['id']}"
            menu('Content filter')
            click(wait(lambda: next((c for c in state()['content_list']['controls'] if c['target']=='Filter(TestPlayers)' and min(c['size'])>0), None))['centre'])
            click(wait(lambda: next((c for c in state()['content_list']['controls'] if c['target']==f"TestPlayer({player['id']})" and min(c['size'])>0), None))['centre'])
            def assign():
                menu('Property actions')
                click(wait(lambda: next((c for c in state()['workspace_controls'] if c['target']=='Command(TestPlayerRoute(1))' and c['menu_item'] and min(c['size'])>0), None))['centre'])
                wait(lambda: state()['document']['test_players'][0].get('controller')==assigned)
            assign()
            x.key_state('Control_L', True); x.key('z'); x.key_state('Control_L', False)
            wait(lambda: state()['document']['test_players'][0].get('controller')==player.get('controller'))
            assign()

        click(next(m['centre'] for m in state()['menus']
            if m['name'] == 'Zone' and m['role'] == 'button' and min(m['size']) > 0))
        control = wait(lambda: next((c for c in state()['workspace_controls']
            if c['target'] == 'PopulationPlaytest' and c['menu_item'] and min(c['size']) > 0), None))
        click(control['centre'])
        wait(lambda: state()['export_dialog'])
        if args.creature_asset:
            assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index']==7) == str(args.creature_asset.resolve())
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
        wait(lambda: state()['network_playtest']['active'], 60)
        run = wait(lambda: next((p for p in (destination/'runs').glob('session-*') if p.is_dir()), None), 45)
        def connected():
            return (run/'server.log').exists() and sum(line.startswith('test_player_admit ') for line in (run/'server.log').read_text().splitlines()) == 2
        wait(connected, 45)
        wait(lambda: '"finished":true' in (run/'client-2.log').read_text(), 45)
        travel = None
        if assigned:
            goal = json.loads((destination/'routes/player-1.json').read_text())['waypoints'][-1]
            def arrived():
                values = []
                for line in (run/'client-1.log').read_text().splitlines():
                    if line.startswith('{"test_controller":'):
                        try: values.append(json.loads(line)['test_controller'])
                        except json.JSONDecodeError: pass
                if not values: return None
                value = values[-1]
                assert value['travel']['failure'] is None, value
                return value if value['finished'] and not value['active'] and value['feet'] and sum((value['feet'][i]-goal[i])**2 for i in [0,2])**.5 < .3 else None
            travel = wait(arrived, 60)
        x.focus(w); fresh()
        click(next(m['centre'] for m in state()['menus'] if m['name']=='Zone' and m['role']=='button' and min(m['size'])>0))
        stop = wait(lambda: next((c for c in state()['workspace_controls'] if c['target']=='StopNetworkPlaytest' and c['menu_item'] and min(c['size'])>0), None))
        click(stop['centre'])
        wait(lambda: not state()['network_playtest']['active'], 45)
        summary = json.loads((run/'session.json').read_text())
        assert summary['reason']=='stop requested' and not summary['cleanup_errors'], summary
        assert summary['population']['count']==5 and len(summary['clients'])==2
        assert not list(run.glob('tickets-*'))
        transport = next(json.loads(line.removeprefix('transport_summary ')) for line in (run/'server.log').read_text().splitlines() if line.startswith('transport_summary '))
        assert transport['test_player_admissions']==2 and transport['test_player_refusals']==0
        assert transport['combat_states_sent']>=2 and transport['target_states_sent']>=2
        assert transport['rejected_inputs']==0 and transport['movement_recoveries']==0
        for log_file in run.glob('*.log'):
            assert check_gpu_diagnostics(log_file.read_text()) == 0
        result = json.loads((destination / 'population-export.json').read_text())['population_export']
        assert result['passed'] and result['count'] == 5, result
        if args.creature_asset:
            assert (destination/'creature.glb').read_bytes() == args.creature_asset.read_bytes()
            assert result['appearance']['file'] == 'creature.glb'
            assert summary['creature_appearance'] and summary['creature_asset']==str(destination/'creature.glb')
        assert source.read_bytes() == original and args.zone.read_bytes() == original
        assert json.loads((destination / 'zone.json').read_text()) == state()['document']
        x.key('F12'); wait(lambda: capture_ready(report.with_suffix('.png')), 30)
        report.with_suffix('.png').rename(out / 'finished.png')
        (out / 'qualification.json').write_text(json.dumps(dict(passed=True,
            invalid_spacing_refused=True, count=result['count'], source_unchanged=True,
            package=str(destination), started_clients=2, stopped=True, controller_finished=True,
            assigned_patrol=assigned, travel=travel, appearance=result.get('appearance'), run=str(run)), indent=2))
    finally:
        if x: x.close()
        if p.poll() is None:
            p.send_signal(signal.SIGINT)
            try: p.wait(timeout=10)
            except subprocess.TimeoutExpired:
                p.kill(); p.wait(timeout=10)
assert check_gpu_diagnostics((out / 'editor.log').read_text()) == 0
