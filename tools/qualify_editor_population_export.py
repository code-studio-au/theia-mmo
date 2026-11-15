"""Qualify checked population export through an owned native editor window."""
import argparse
import json
import os
from pathlib import Path
import signal
import subprocess
import struct
import time
from launch_environment import client_environment
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from gpu_validation import check_gpu_diagnostics

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
parser.add_argument('--zone', type=Path, required=True)
parser.add_argument('--creature', type=Path, required=True)
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--scenery-library', type=Path)
parser.add_argument('--gameplay-template', type=Path)
parser.add_argument('--creature-asset', type=Path)
parser.add_argument('--access-path', type=int, help='Explicit saved road guide for checked population access')
parser.add_argument('--escort-destination', type=int, help='Delete/undo this destination and verify escort export rejection and route preparation')
parser.add_argument('--load-creature-context', action='store_true', help='Load the standalone definition through File, assign/undo it, and verify export defaults')
args = parser.parse_args()
if args.escort_destination is not None and (not args.gameplay_template or args.escort_destination <= 0):
    parser.error('--escort-destination requires a positive marker ID and gameplay template')
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
original = args.zone.read_bytes()
creature_original = args.creature.read_bytes()
source = out / 'source.zone.json'
seeded = original
if args.load_creature_context:
    draft = json.loads(original)
    assert len(draft['spawn_groups']) == 1
    draft['spawn_groups'][0]['creature'] = 'theia.creature.unresolved'
    seeded = (json.dumps(draft, indent=2) + '\n').encode()
source.write_bytes(seeded)
report = out / 'state.json'
env = client_environment(dict(os.environ, DISPLAY=':0', XDG_RUNTIME_DIR='/run/user/1000',
    WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
    THEIA_PARTICLE_EDITOR_DATA_DIR=str(out / 'particle-data')))
x = None
with (out / 'editor.log').open('w') as log:
    p = subprocess.Popen([str(args.binary.resolve()), '--zone', str(source), '--report', str(report),
        *(['--scenery-library', str(args.scenery_library.resolve())] if args.scenery_library else []),
        *(['--gameplay-template', str(args.gameplay_template.resolve())] if args.gameplay_template else []),
        *(['--creature-asset', str(args.creature_asset.resolve())] if args.creature_asset else [])],
        env=env, stdout=log, stderr=log)
    try:
        w = wait(lambda: owned_window(p.pid), 45)
        x = Desktop(); x.focus(w)
        def state():
            assert p.poll() is None, 'Editor exited unexpectedly'
            return wait(lambda: json.loads(report.read_text()), 2)
        wait(lambda: report.exists() and state().get('menus'), 45)
        geometry = None
        stable = time.monotonic()
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            current = [(m['name'], m['centre'], m['size']) for m in state()['menus'] if m['role'] == 'button']
            if current != geometry:
                geometry, stable = current, time.monotonic()
            elif time.monotonic() - stable >= .75:
                break
            time.sleep(.05)
        else:
            raise TimeoutError('Editor menu geometry did not settle')
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
            try:
                wait(lambda: state()['control_down'])
                x.key('a')
                fresh()
            finally:
                x.key_state('Control_L', False)
            wait(lambda: not state()['control_down'])
            names = {'/':'slash', '.':'period', '-':'minus', '_':'underscore', ':':'colon'}
            for char in str(value):
                key = names.get(char, char)
                x.key(key)
                fresh()
            fresh()
            assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index'] == index) == str(value)

        if args.load_creature_context:
            def menu(name):
                click(next(m['centre'] for m in state()['menus'] if m['name'] == name and m['role'] == 'button' and min(m['size']) > 0))
            def chord(key):
                x.key_state('Control_L', True)
                try:
                    wait(lambda: state()['control_down'])
                    x.key(key); fresh()
                finally:
                    x.key_state('Control_L', False)
                wait(lambda: not state()['control_down'])
            def file_control(name):
                click(wait(lambda: next((c for c in state()['file_controls'] if c['target'] == name and min(c['size']) > 0), None))['centre'])
            before_load = state()['document']
            menu('File')
            click(wait(lambda: next((c for c in state()['workspace_controls'] if c['target'] == 'CreatureOpen' and c['menu_item'] and min(c['size']) > 0), None))['centre'])
            wait(lambda: state()['file_dialog'] and state()['file_dialog']['mode'] == 'Creature')
            file_control('Field'); chord('a')
            target = str(args.creature.resolve())
            for character in target:
                x.key({'/':'slash', '.':'period', '-':'minus', '_':'underscore'}.get(character, character))
                fresh()
            wait(lambda: state()['file_dialog']['text'] == target)
            x.key('F12'); wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            report.with_suffix('.png').rename(out / 'creature-browser.png')
            file_control('Apply')
            wait(lambda: state()['file_dialog'] is None and state()['status'].startswith('Loaded creature definition'))
            assert state()['document'] == before_load
            group = before_load['spawn_groups'][0]['id']
            menu('Content filter')
            click(wait(lambda: next((c for c in state()['content_list']['controls'] if c['target'] == 'Filter(SpawnGroups)' and min(c['size']) > 0), None))['centre'])
            click(wait(lambda: next((c for c in state()['content_list']['controls'] if c['target'] == f'SpawnGroup({group})' and min(c['size']) > 0), None))['centre'])
            wait(lambda: state()['spawn_group_selected'] == group)
            click(next(c['centre'] for c in state()['property_tabs']['controls'] if c['tab'] == 'Attributes'))
            def bind():
                click(wait(lambda: next((c for c in state()['workspace_controls'] if c['target'] == 'Command(BindSpawnCreature)' and not c['menu_item'] and min(c['size']) > 0), None))['centre'])
            creature_id = json.loads(args.creature.read_text())['id']
            bind(); wait(lambda: state()['document']['spawn_groups'][0]['creature'] == creature_id)
            chord('z'); wait(lambda: state()['document'] == before_load)
            bind(); wait(lambda: state()['document']['spawn_groups'][0]['creature'] == creature_id)
            x.key('F12'); wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            report.with_suffix('.png').rename(out / 'creature-assigned.png')

        if args.escort_destination is not None:
            click(next(m['centre'] for m in state()['menus'] if m['name'] == 'Content filter' and m['role'] == 'button' and min(m['size']) > 0))
            click(wait(lambda: next((c for c in state()['content_list']['controls'] if c['target'] == 'Filter(Markers)' and min(c['size']) > 0), None))['centre'])
            click(wait(lambda: next((c for c in state()['content_list']['controls'] if c['target'] == f'Marker({args.escort_destination})' and min(c['size']) > 0), None))['centre'])
            wait(lambda: state()['marker_selected'] == args.escort_destination)
            x.key('Delete')
            wait(lambda: not any(m['id'] == args.escort_destination for m in state()['document']['markers']))

        click(next(m['centre'] for m in state()['menus']
            if m['name'] == 'File' and m['role'] == 'button' and min(m['size']) > 0))
        control = wait(lambda: next((c for c in state()['workspace_controls']
            if c['target'] == 'ExportPopulation' and c['menu_item'] and min(c['size']) > 0), None))
        click(control['centre'])
        wait(lambda: state()['export_dialog'])
        destination = out / 'package'
        fill(0, destination)
        if args.load_creature_context:
            assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index'] == 3) == str(args.creature.resolve())
        else:
            fill(3, args.creature.resolve())
        fill(5, '33')
        def submit():
            click(next(b['centre'] for b in state()['export_dialog']['buttons'] if b['action'] == 'Export'))
        submit()
        wait(lambda: 'Minimum spacing' in state()['export_dialog']['message'])
        assert not destination.exists()
        fill(5, '3')
        if args.access_path is not None:
            fill(11, '0')
            submit()
            wait(lambda: 'Access road guide ID' in state()['export_dialog']['message'])
            assert not destination.exists()
            fill(11, args.access_path)
        if args.gameplay_template:
            assert next(f['text'] for f in state()['export_dialog']['fields'] if f['index'] == 10) == str(args.gameplay_template.resolve())
        if args.escort_destination is not None:
            submit()
            wait(lambda: 'Escort' in state()['export_dialog']['message'] and 'destination' in state()['export_dialog']['message'], 60)
            assert not destination.exists(), 'Invalid escort must never publish a package'
            x.key('F12'); wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            report.with_suffix('.png').rename(out / 'missing-escort-destination.png')
            click(next(b['centre'] for b in state()['export_dialog']['buttons'] if b['action'] == 'Close'))
            wait(lambda: state()['export_dialog'] is None)
            x.key_state('Control_L', True)
            try:
                wait(lambda: state()['control_down'])
                x.key('z'); fresh()
            finally:
                x.key_state('Control_L', False)
            wait(lambda: any(m['id'] == args.escort_destination for m in state()['document']['markers']))
            click(next(m['centre'] for m in state()['menus'] if m['name'] == 'File' and m['role'] == 'button' and min(m['size']) > 0))
            click(wait(lambda: next((c for c in state()['workspace_controls'] if c['target'] == 'ExportPopulation' and c['menu_item'] and min(c['size']) > 0), None))['centre'])
            wait(lambda: state()['export_dialog'])
            fill(0, destination); fill(3, args.creature.resolve()); fill(5, '3')
            if args.access_path is not None: fill(11, args.access_path)
        x.key('F12'); wait(lambda: capture_ready(report.with_suffix('.png')), 30)
        report.with_suffix('.png').rename(out / 'dialog.png')
        submit()
        wait(lambda: state()['export_dialog']['finished'], 180)
        result = json.loads((destination / 'population-export.json').read_text())['population_export']
        assert result['passed'] and result['count'] == 5, result
        assert result['access_path'] == args.access_path
        assert bool(result['gameplay_manifest']) == bool(args.gameplay_template)
        if args.escort_destination is not None:
            assert any(q['destination'] == args.escort_destination and q['steering_points'] >= 2 for q in result['quest_travel'])
        assert source.read_bytes() == seeded and args.zone.read_bytes() == original
        assert args.creature.read_bytes() == creature_original
        exported = json.loads((destination / 'zone.json').read_text())
        observed = state()['document']
        # Saved f32 heights use shortest round-trip decimals; JSON telemetry
        # promotes them to f64. Compare exact f32 values, leaving global f64 intact.
        for document in (exported, observed):
            if document.get('terrain'):
                document['terrain']['heights'] = [struct.pack('<f', h) for h in document['terrain']['heights']]
        assert exported == observed
        x.key('F12'); wait(lambda: capture_ready(report.with_suffix('.png')), 30)
        report.with_suffix('.png').rename(out / 'finished.png')
        (out / 'qualification.json').write_text(json.dumps(dict(passed=True,
            invalid_spacing_refused=True, invalid_access_id_refused=args.access_path is not None,
            access_path=args.access_path, gameplay_snapshot=bool(args.gameplay_template),
            count=result['count'], source_unchanged=True,
            invalid_escort_refused=args.escort_destination is not None,
            quest_travel=result.get('quest_travel', []),
            direct_creature_load=args.load_creature_context, assignment_undo=args.load_creature_context,
            package=str(destination)), indent=2))
    finally:
        if x: x.close()
        if p.poll() is None:
            p.send_signal(signal.SIGINT)
            try: p.wait(timeout=10)
            except subprocess.TimeoutExpired:
                p.kill(); p.wait(timeout=10)
assert check_gpu_diagnostics((out / 'editor.log').read_text()) == 0
