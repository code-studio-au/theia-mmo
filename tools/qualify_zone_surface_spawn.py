#!/usr/bin/env python3
"""Native surface-spawn authoring and offline Avian entry on both terrain grids."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from qualify_zone_flatten import normalized


def run(output, binary):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    cases = []
    env = dict(os.environ,WGPU_BACKEND='vulkan',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
               VK_LOADER_DEBUG='layer')
    for name,axis,origin,base in [('coarse',33,[0,0,0],4.),
            ('fine-far',65,[128000000000,20,-128000000000],-4.)]:
        source = output/f'{name}.source.json'
        saved = output/f'{name}.saved.json'
        report = output/f'{name}.state.json'
        logfile = output/f'{name}.log'
        patch = dict(chunk=[origin[0]//128,origin[2]//128],heights=[
            0 if x in (0,axis-1) or z in (0,axis-1) else base
            for z in range(axis) for x in range(axis)])
        if axis==65:
            patch['resolution']='2m'
        source.write_text(json.dumps(dict(schema=7,name='Surface spawn qualification',origin=origin,
            spawn=[origin[0]+16,origin[1]+.1,origin[2]+16],objects=[],terrain=patch)))
        with logfile.open('w') as log:
            process = subprocess.Popen([str(binary),'--zone',str(source),'--save-path',str(saved),
                                        '--report',str(report)],env=env,stdout=log,stderr=log)
            desktop = None
            try:
                window = wait(lambda: owned_window(process.pid),45)
                desktop = Desktop()
                desktop.focus(window)

                def state():
                    assert process.poll() is None
                    return json.loads(report.read_text())

                wait(lambda: report.exists() and state()['viewport']==[3440,1440],45)
                desktop.move(window,[1737,731],1)
                desktop.move(window,[1720,720],1)
                key = desktop.key
                key('F2')
                wait(lambda: state()['terrain_mode'])
                key('f')
                wait(lambda: state()['view']['distance']>100)

                def fresh():
                    elapsed = state()['input_elapsed']
                    wait(lambda: state()['input_elapsed']>elapsed)
                    return state()

                def move(offset):
                    point = state()['terrain_points'][0]
                    desktop.move(window,[point[0]+offset,point[1]+9],1)
                    return fresh()

                def button(action, revision=None):
                    entry = wait(lambda: next((b for b in state()['buttons'] if b['action']==action),None))
                    desktop.move(window,entry['centre'],1)
                    wait(lambda: any(b['action']==action and b['hovered'] for b in state()['buttons']))
                    desktop.button(1,True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1,False)
                    wait(lambda: not state()['left_mouse_down'])
                    if revision is not None:
                        wait(lambda: state()['revision']==revision)

                def reference(label, before, position, actual):
                    prior = output/f'{name}.{label}.before.json'
                    commands = output/f'{name}.{label}.commands.jsonl'
                    expected = output/f'{name}.{label}.expected.json'
                    prior.write_text(json.dumps(before))
                    commands.write_text(json.dumps(dict(command='apply',expected_revision=0,
                        edits=[dict(op='set_spawn',position=position)]))+'\n')
                    subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),
                                    '--output',str(expected)],check=True,capture_output=True)
                    assert normalized(json.loads(expected.read_text()))==normalized(actual)

                for label,offset in [('snapped',17),('precise',53)]:
                    if label=='precise':
                        key('F7')
                        wait(lambda: state()['terrain_paint'])
                        key('g')
                        wait(lambda: not state()['snap'])
                    before = move(offset)
                    point = before['brush_point']
                    xz = [round(point[0]),round(point[2])] if before['snap'] else [point[0],point[2]]
                    position = [origin[0]+xz[0],origin[1]+base+.1,origin[2]+xz[1]]
                    key('p')
                    after = wait(lambda: state() if state()['revision']==before['revision']+1 else None)
                    assert after['document']['spawn']==position
                    assert after['history']==[before['history'][0]+1,0]
                    reference(label,before['document'],position,after['document'])
                    key('p')
                    wait(lambda: state()['status']=='Spawn already at this surface position')
                    assert state()['revision']==after['revision'] and state()['history']==after['history']
                    button('Undo',after['revision']+1)
                    assert state()['document']==before['document']
                    button('Redo',after['revision']+2)
                    assert state()['document']==after['document']
                # The retained button uses the same checked sampler without snapping.
                key('F1')
                wait(lambda: not state()['terrain_mode'])
                before = state()
                expected = [origin[0]+before['view']['target'][0],origin[1]+base+.1,
                            origin[2]+before['view']['target'][2]]
                button('Spawn',before['revision']+1)
                assert state()['document']['spawn']==expected
                reference('view',before['document'],expected,state()['document'])
                key('F2')
                wait(lambda: state()['terrain_mode'])
                before = state()
                desktop.move(window,[150,300],1)
                fresh()
                key('p')
                fresh()
                assert state()['document']==before['document'] and state()['revision']==before['revision']
                # P belongs to the standard text field while a modal is open.
                key('F8')
                wait(lambda: state()['name_edit'] is not None)
                key('p')
                wait(lambda: state()['name_edit']=='p')
                key('Escape')
                wait(lambda: state()['name_edit'] is None)
                assert state()['document']==before['document']
                for _ in range(7):
                    key('bracketleft')
                wait(lambda: abs(state()['brush_strength']-.05)<1e-7)
                move(0)
                desktop.button(1,True)
                try:
                    wait(lambda: state()['stroke_progress'] and state()['stroke_progress']['stamps']==1)
                    preview = state()
                    key('p')
                    wait(lambda: state()['status']=='Finish or cancel the stroke before placing spawn')
                    assert state()['document']==before['document'] and state()['revision']==before['revision']
                    assert state()['terrain_preview_heights']==preview['terrain_preview_heights']
                    key('Escape')
                    wait(lambda: state()['stroke_progress'] is None)
                finally:
                    desktop.button(1,False)
                assert state()['document']==before['document']
                key('F5')
                wait(lambda: state()['play_feet'] is not None)
                assert abs(state()['play_feet'][1]-base)<.2
                key('p')
                fresh()
                assert state()['document']==before['document'] and state()['revision']==before['revision']
                key('F5')
                wait(lambda: state()['play_feet'] is None)
                button('Save')
                wait(lambda: saved.exists() and not state()['unsaved_changes'])
                assert normalized(json.loads(saved.read_text()))==normalized(state()['document'])
                key('F12')
                wait(lambda: capture_ready(report.with_suffix('.png')),30)
                final = state()
                subprocess.run(['wmctrl','-ic',hex(window)],check=True)
                process.wait(timeout=20)
                assert process.returncode==0
            finally:
                if desktop:
                    desktop.close()
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=15)
        logs = logfile.read_text()
        assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
        diagnostics = check_gpu_diagnostics(logs,allow_surface_resize=False)
        cases.append(dict(name=name,viewport=final['viewport'],gpu=final['gpu'],
                          validation_layer_active=True,gpu_diagnostics=diagnostics))
    result = dict(cases=cases,checks=['snapped/precise cursor and view-centre spawn match headless SetSpawn',
        'one-entry undo/redo, unchanged cursor no-op and exact save',
        'panel/modal/playtest isolation, pending stroke refusal without preview loss',
        'actual offline Avian spawn on positive/negative terrain at local/far origins'],
        scope='Linux native 3440x1440 only; terrain support, no prop-stack or multiplayer-authority claim')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output,args.binary)
