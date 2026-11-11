#!/usr/bin/env python3
"""Native editor particle-to-chunk export using the File menu and background job."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time

from gpu_validation import check_gpu_diagnostics
from launch_environment import client_environment
from qualify_world_input_os import Desktop, capture_ready, owned_window, wait


def run(output, binary, empty_zone=False, replace=False, clear=False, projected=False):
    output.mkdir(parents=True, exist_ok=False)
    name = f'exportdemo{time.time_ns()}'
    world = Path('build')/(name+'-particles')
    assert not world.exists()
    base_world = Path('build/editor-particle-chunk-world-validated-20261111' if replace else 'build/chunk-paired-native-20261020/fixture-good')
    if projected:
        assert not replace and not clear
        base_world = Path('build/runtime-materials-fixture-v1')
    chunk_name = 'chunk_1000000000_-1000000000' if projected else 'chunk_0_0'
    before_index = (base_world/'world.chunks.json').read_bytes()
    before_manifest = (base_world/chunk_name/'world.manifest.json').read_bytes()
    data = output/'particle-data/library'
    data.mkdir(parents=True)
    effect = data/'vfx.test.ron'
    shutil.copyfile('apps/client/assets/vfx/projectile-glimmer.ron', effect)
    effect_before = effect.read_bytes()
    source, report = output/'zone.json', output/'state.json'
    edits = json.loads(Path('content/zones/greybox.commands.jsonl').read_text().splitlines()[0])['edits']
    objects = [edit['object'] for edit in edits if edit['op']=='place']
    if empty_zone or projected:
        objects = []
    origin = [128000000000,0,-128000000000] if projected else [0,0,0]
    position = [origin[0]+60,1.3,origin[2]+50] if projected else [122,1.3,32]
    local_position = [60.,1.2999999523162842,50.] if projected else [122.,1.2999999523162842,32.]
    particles = [] if clear else [{'id':10,'name':'Test','effect':'vfx.test','position':position,
                                  'yaw_degrees':0,'scale':1}]
    source.write_text(json.dumps({'schema':8, 'name':name, 'origin':origin, 'spawn':[origin[0],.1,origin[2]+6],
        'objects':objects, 'particles':particles}))
    before = source.read_bytes()
    env = client_environment(dict(os.environ, WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
        VK_LOADER_DEBUG='layer', THEIA_PARTICLE_EDITOR_DATA_DIR=str(data.parent)))
    desktop = Desktop()
    with (output/'editor.log').open('w') as log:
        process = subprocess.Popen([str(binary), '--zone', str(source), '--report', str(report)], env=env, stdout=log, stderr=log)
        try:
            window = wait(lambda: owned_window(process.pid), 60)
            desktop.focus(window)
            def state():
                assert process.poll() is None, 'Editor exited during export'
                return json.loads(report.read_text())
            wait(lambda: report.exists() and state()['viewport'] == [3440,1440], 60)
            def click(point):
                desktop.move(window, point, 1.)
                wait(lambda: state()['cursor'] and max(abs(a-b) for a,b in zip(state()['cursor'],point)) < 2)
                desktop.button(1, True)
                wait(lambda: state()['left_mouse_down'])
                desktop.button(1, False)
                wait(lambda: not state()['left_mouse_down'])
            file_button = next(e for e in state()['menus'] if e['name']=='File' and e['role']=='button' and min(e['size'])>0)
            click(file_button['centre'])
            target = 'ReplaceParticleChunks' if replace else 'ExportParticleChunks'
            item = wait(lambda: next((e for e in state()['workspace_controls'] if e['target']==target and e['menu_item'] and min(e['size'])>0),None))
            click(item['centre'])
            dialog = wait(lambda: state()['export_dialog'])
            assert len(dialog['fields']) == 3
            def focused(index):
                dialog = state()['export_dialog']
                return dialog and next(field for field in dialog['fields'] if field['index']==index)['focused']
            wait(lambda: focused(0))
            desktop.key('Tab')
            wait(lambda: focused(1))
            chunk_path = str(base_world)
            for char in chunk_path:
                desktop.key({'/':'slash','-':'minus'}.get(char,char))
            wait(lambda: next(field for field in state()['export_dialog']['fields'] if field['index']==1)['text']==chunk_path)
            desktop.key('Tab')
            wait(lambda: focused(0))
            desktop.key_state('Shift_L',True)
            desktop.key('Tab')
            desktop.key_state('Shift_L',False)
            wait(lambda: focused(1))
            desktop.key('F12')
            wait(lambda: capture_ready(report.with_suffix('.png')), 30)
            button = next(button for button in state()['export_dialog']['buttons'] if button['action']=='Export')
            click(button['centre'])
            wait(lambda: state()['export_dialog'] and state()['export_dialog']['finished'], 60)
            manifest = json.loads((world/chunk_name/'world.manifest.json').read_text())
            assert manifest['schema']==(5 if projected else 2 if clear else 4) and manifest['revision']==(2 if projected else 4 if replace else 3)
            if clear:
                assert not (world/chunk_name/'particles').exists()
            else:
                catalog = (world/chunk_name/'particles/catalog.json').read_bytes()
                assert manifest['particle_sha256']==list(hashlib.sha256(catalog).digest())
                assert json.loads(catalog)['placements'][0]['position']==local_position
            assert source.read_bytes()==before and effect.read_bytes()==effect_before
            assert (base_world/'world.chunks.json').read_bytes()==before_index
            assert (base_world/chunk_name/'world.manifest.json').read_bytes()==before_manifest
            close = next(button for button in state()['export_dialog']['buttons'] if button['action']=='Close')
            click(close['centre'])
            wait(lambda: state()['export_dialog'] is None)
        finally:
            if process.poll() is None:
                process.send_signal(signal.SIGINT)
                try: process.wait(timeout=5)
                except subprocess.TimeoutExpired: process.kill(); process.wait()
            desktop.close()
    log = (output/'editor.log').read_text()
    assert 'VK_LAYER_KHRONOS_validation' in log and check_gpu_diagnostics(log)==0
    result = {'passed':True,'world':str(world.resolve()),'revision':2 if projected else 4 if replace else 3,'projected':projected,'empty_zone':empty_zone,'replace':replace,'clear':clear,'source_unchanged':True,
              'effect_unchanged':True,'gpu_diagnostics':0}
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--binary', type=Path, default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--empty-zone', action='store_true')
    parser.add_argument('--replace', action='store_true')
    parser.add_argument('--clear', action='store_true')
    parser.add_argument('--projected', action='store_true')
    args = parser.parse_args()
    assert not args.clear or args.replace
    print(json.dumps(run(args.output.resolve(), args.binary.resolve(), args.empty_zone, args.replace, args.clear, args.projected),indent=2))
