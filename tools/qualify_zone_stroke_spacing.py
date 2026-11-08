#!/usr/bin/env python3
"""Exercise fast and interrupted terrain drags against shared headless commands."""
import argparse
import json
import math
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
    for name,axis,origin in [('coarse',33,[0,0,0]),('fine-far',65,[128000000000,20,-128000000000])]:
        source = output/f'{name}.source.json'
        report = output/f'{name}.state.json'
        saved = output/f'{name}.saved.json'
        logfile = output/f'{name}.log'
        patch = dict(chunk=[origin[0]//128,origin[2]//128],heights=[0]*(axis*axis))
        if axis==65:
            patch['resolution']='2m'
        zone = dict(schema=7,name='Stroke spacing qualification',origin=origin,
                    spawn=[origin[0]+16,origin[1]+.1,origin[2]+16],terrain=patch,
                    objects=[dict(id=i,name=f'Pointer {i}',primitive='box',pose=dict(
                        position=[origin[0]+x,origin[1]+.05,origin[2]+z],yaw_degrees=0,size=[.1,.1,.1]))
                        for i,x,z in [(1,32,64),(2,96,64),(3,64,32),(4,129,64)]])
        source.write_text(json.dumps(zone))
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
                for _ in range(7):
                    key('bracketleft')
                wait(lambda: abs(state()['brush_strength']-.05)<1e-7)

                def move(identifier):
                    point = next(o['screen'] for o in state()['objects'] if o['id']==identifier)
                    desktop.move(window,point,1)
                    elapsed = state()['input_elapsed']
                    wait(lambda: state()['input_elapsed']>elapsed)

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

                def reference(label, before, edits, actual):
                    prior = output/f'{name}.{label}.before.json'
                    commands = output/f'{name}.{label}.commands.jsonl'
                    expected = output/f'{name}.{label}.expected.json'
                    prior.write_text(json.dumps(before))
                    commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=edits))+'\n')
                    subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),
                                    '--output',str(expected)],check=True,capture_output=True)
                    assert normalized(json.loads(expected.read_text()))==normalized(actual)

                counts = []
                for label in ['sculpt','paint']:
                    if label=='paint':
                        key('F7')
                        wait(lambda: state()['terrain_paint'])
                    before = state()
                    move(1)
                    desktop.button(1,True)
                    try:
                        wait(lambda: state()['stroke_progress'] and state()['stroke_progress']['stamps']==1)
                        move(2)  # One real OS jump of about 64 m.
                        wait(lambda: state()['stroke_progress']['stamps']>=32)
                        preview = state()
                        edits = preview['stroke_edits']
                        assert 32<=len(edits)<=34
                        centres = [e.get('brush',e.get('paint'))['centre'] for e in edits]
                        for a,b in zip(centres,centres[1:]):
                            assert abs(math.dist(a,b)-2)<.0001,(a,b)
                        assert preview['revision']==before['revision'] and preview['document']==before['document']
                        assert preview['history']==before['history']
                        # A stationary hold adds no time-dependent stamps.
                        elapsed = state()['input_elapsed']
                        wait(lambda: state()['input_elapsed']>elapsed+.4)
                        assert state()['stroke_edits']==edits
                    finally:
                        desktop.button(1,False)
                    committed = wait(lambda: state() if state()['revision']==before['revision']+1 else None)
                    assert committed['history']==[before['history'][0]+1,0]
                    reference(label,before['document'],edits,committed['document'])
                    counts.append(len(edits))
                    button('Undo',committed['revision']+1)
                    assert state()['document']==before['document']
                    button('Redo',committed['revision']+2)
                    assert state()['document']==committed['document']
                    if label=='paint':
                        button('Undo',committed['revision']+3)
                # Leaving the panel/patch breaks continuity rather than bridging
                # across ground that the user never dragged over.
                before = state()
                move(1)
                desktop.button(1,True)
                try:
                    wait(lambda: state()['stroke_progress'] and state()['stroke_progress']['stamps']==1)
                    desktop.move(window,[150,300],1)
                    wait(lambda: state()['brush_point'] is None)
                    move(2)
                    wait(lambda: state()['stroke_progress']['stamps']==2)
                    move(4)
                    wait(lambda: state()['brush_point'] and state()['brush_point'][0]>128)
                    move(3)
                    wait(lambda: state()['stroke_progress']['stamps']==3)
                    edits = state()['stroke_edits']
                    assert len(edits)==3
                finally:
                    desktop.button(1,False)
                wait(lambda: state()['revision']==before['revision']+1)
                reference('interrupted',before['document'],edits,state()['document'])
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
        cases.append(dict(name=name,viewport=final['viewport'],gpu=final['gpu'],stamps=counts,
                          validation_layer_active=True,gpu_diagnostics=diagnostics))
    result = dict(cases=cases,checks=['64 m OS jumps filled with 2 m sculpt/paint stamps, stable stationary hold',
        'preview source/history isolation and one-entry release with exact headless document equivalence',
        'undo/redo/save and panel/outside-patch discontinuities'],
        scope='Linux native 3440x1440 only; bounded 128 accepted edits, no Windows/multiplayer claim')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output,args.binary)
