"""Owned native spawn draft editing; no live population or source overwrite."""
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
parser.add_argument('--output', type=Path, required=True)
args = parser.parse_args()
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=False)
source, saved, report = [out/name for name in ('source.zone.json', 'saved.zone.json', 'state.json')]
original = json.dumps(dict(schema=14, name='NPC spawn draft check', origin=[8192,0,8192],
    spawn=[8192,.1,8198], objects=[], terrain=dict(chunk=[63,63], span_chunks=2, heights=[0.]*(65*65)),
    spawn_groups=[dict(id=10, name='Western wolves', enabled=True, creature='theia.dev.training-creature',
    volume=dict(polygon=[[8184,8184],[8200,8184],[8200,8192],[8192,8192],[8192,8200],[8184,8200]], min_y=-2,max_y=8),
    target_alive=5,respawn_seconds=30,seed=42)])).encode()
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
                if r['target'] == f'SpawnGroup({identifier})' and min(r['size']) > 0), None))
            click(row['centre']); wait(lambda: state()['spawn_group_selected'] == identifier)
        def menu(name):
            click(next(m['centre'] for m in state()['menus'] if m['name']==name and m['role']=='button' and min(m['size'])>0))
        def command(name):
            click(wait(lambda: next((c for c in state()['workspace_controls'] if c['target']==f'Command({name})' and c['menu_item'] and min(c['size'])>0), None))['centre'])
        def edit(collection, field, value):
            x.key('F9'); fresh()
            for _ in range(24):
                row = next((f for f in state()[collection] if f['field']==field and f['focused']), None)
                if row: break
                x.key('Tab'); fresh()
            assert row, state()[collection]
            chord('a')
            for ch in str(value): x.key({'-':'minus','.':'period',' ':'space'}.get(ch,ch))
            x.key('Return'); fresh()
        def capture(name):
            picture = report.with_suffix('.png'); picture.unlink(missing_ok=True)
            x.key('F12'); wait(lambda: capture_ready(picture), 30); picture.rename(out/name)
        menu('Content filter')
        click(wait(lambda: next((c for c in state()['content_list']['controls'] if c['target']=='Filter(SpawnGroups)' and min(c['size'])>0), None))['centre'])
        select(10); x.key('f'); fresh()
        click(next(c['centre'] for c in state()['property_tabs']['controls'] if c['tab']=='Attributes'))
        wait(lambda: state()['spawn_group_preview']['visible_edges']==18)
        edit('inline_attributes',4,7)
        wait(lambda: state()['document']['spawn_groups'][0]['target_alive']==7)
        edit('inline_attributes',5,45)
        wait(lambda: state()['document']['spawn_groups'][0]['respawn_seconds']==45)
        edit('spawn_group_fields','SpawnSeed',18446744073709551615)
        wait(lambda: state()['document']['spawn_groups'][0]['seed']==18446744073709551615)
        toggle = next(c for c in state()['workspace_controls'] if c['target']=='SpawnEnabled' and min(c['size'])>0)
        click(toggle['centre']); wait(lambda: not state()['document']['spawn_groups'][0]['enabled'])
        capture('spawn-attributes.png')
        select(10); chord('d'); wait(lambda: len(state()['document']['spawn_groups'])==2)
        x.key('Delete'); wait(lambda: len(state()['document']['spawn_groups'])==1)
        chord('z'); wait(lambda: len(state()['document']['spawn_groups'])==2)
        chord('z'); wait(lambda: len(state()['document']['spawn_groups'])==1)
        menu('Add'); command('AddSpawnGroup')
        wait(lambda: len(state()['document']['spawn_groups'])==2)
        wait(lambda: state()['spawn_group_preview']['visible_edges']==12)
        select(10); x.key('f'); fresh(); capture('spawn-volume.png')
        x.key('F5'); wait(lambda: state()['play_feet'] is not None)
        wait(lambda: state()['spawn_group_preview']['visible_edges']==0)
        x.key('F5'); wait(lambda: state()['play_feet'] is None)
        assert source.read_bytes()==original and not saved.exists()
        chord('s'); wait(lambda: saved.exists() and not state()['unsaved_changes'])
        assert json.loads(saved.read_text())==state()['document'] and source.read_bytes()==original
        gpu = state()['gpu']
    finally:
        if x: x.close()
        if process.poll() is None:
            process.send_signal(signal.SIGINT)
            try: process.wait(timeout=10)
            except subprocess.TimeoutExpired: process.terminate(); process.wait(timeout=10)
assert check_gpu_diagnostics((out/'editor.log').read_text())==0
(out/'qualification.json').write_text(json.dumps(dict(passed=True,gpu=gpu,
    concave_preview=True,create_filter=True,inline_count_delay_seed=True,enabled=True,
    copy_delete_undo=True,hidden_in_play=True,explicit_save=True,source_preserved=True,
    scope='Population authoring drafts only; no live spawning or boundary vertex editing'),indent=2)+'\n')
