#!/usr/bin/env python3
"""Native world-axis move gestures and shared command equivalence at 3440x1440."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
    cases = []
    env = dict(os.environ, WGPU_BACKEND='vulkan', VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
               VK_LOADER_DEBUG='layer')
    for name, origin in [('local', [0,0,0]), ('far', [128000000000,20,-128000000000])]:
        source = output/f'{name}.source.json'
        report = output/f'{name}.state.json'
        saved = output/f'{name}.saved.json'
        logfile = output/f'{name}.log'
        source.write_text(json.dumps(dict(schema=1,name='Move handles',origin=origin,
            spawn=[origin[0],origin[1]+.1,origin[2]+6],objects=[dict(id=1,name='Move box',primitive='box',
                pose=dict(position=[origin[0]+8.125,origin[1]+1.25,origin[2]-8.5],yaw_degrees=15,size=[2,2,2]))])))
        with logfile.open('w') as log:
            process = subprocess.Popen([str(binary),'--zone',str(source),'--save-path',str(saved),
                '--report',str(report)],env=env,stdout=log,stderr=log)
            desktop = None
            try:
                window = wait(lambda:owned_window(process.pid),45)
                desktop = Desktop()
                desktop.focus(window)
                def state():
                    assert process.poll() is None, 'Editor exited'
                    return json.loads(report.read_text())
                wait(lambda:report.exists() and state()['viewport']==[3440,1440],45)
                desktop.move(window,[1737,731],1)
                desktop.move(window,[1720,720],1)
                def fresh():
                    elapsed=state()['input_elapsed']
                    wait(lambda:state()['input_elapsed']>elapsed)
                    return state()
                def click(point):
                    desktop.move(window,point,1)
                    def owned():
                        try: desktop.owned_focus(pointer=True); return True
                        except AssertionError: return False
                    wait(owned,15)
                    desktop.button(1,True)
                    try: wait(lambda:state()['left_mouse_down'])
                    finally: desktop.button(1,False)
                    wait(lambda:not state()['left_mouse_down'])
                    fresh()
                def control(target):
                    click(wait(lambda:next((b['centre'] for b in state()['workspace_controls']
                        if b['target']==target and all(x>0 for x in b['size'])),None)))
                def select():
                    click(next(b['centre'] for b in state()['content_list']['controls'] if b['target']=='Object(1)'))
                    wait(lambda:state()['selected']==1)
                def handles():
                    return [h for h in state()['move_handles'] if h['visible']]
                def handle(axis):
                    return wait(lambda:next((h for h in handles() if h['axis']==axis),None))
                def capture(label):
                    path=report.with_suffix('.png')
                    desktop.key('F12');wait(lambda:capture_ready(path),30)
                    path.rename(output/f'{name}.{label}.png')
                select();control('Frame')
                wait(lambda:len(handles())==3)
                initial=state()['document']
                control('Handles');wait(lambda:not state()['show_move_handles'] and not handles())
                control('Handles');wait(lambda:state()['show_move_handles'] and len(handles())==3)
                assert state()['document']==initial and state()['revision']==0
                # A click without movement must not snap fractional source values.
                click(handle(0)['screen'])
                assert state()['document']==initial and state()['revision']==0 and not state()['dragging']
                assert state()['status']=='Position unchanged'
                capture('handles')
                accepted=[]
                for axis in range(3):
                    before=copy.deepcopy(state()['document'])
                    revision=state()['revision']
                    arrow=handle(axis)
                    desktop.move(window,arrow['screen'],1)
                    desktop.button(1,True)
                    try:
                        wait(lambda:state()['dragging'])
                        desktop.move(window,arrow['target'],1)
                        preview=wait(lambda:state()['move_preview'] if state()['move_preview'] and
                            state()['move_preview']!=before['objects'][0]['pose'] else None)
                        assert state()['document']==before and state()['revision']==revision
                        assert preview['position'][axis]==round(preview['position'][axis])
                        for other in range(3):
                            if other!=axis: assert preview['position'][other]==before['objects'][0]['pose']['position'][other]
                        assert abs(preview['position'][axis]-before['objects'][0]['pose']['position'][axis]-2)<.6
                        capture(f'axis-{axis}-preview')
                    finally: desktop.button(1,False)
                    wait(lambda:not state()['dragging'] and state()['revision']==revision+1)
                    changed=state()['document']
                    assert changed['objects'][0]['pose']==preview and state()['history']==[axis+1,0]
                    prior=output/f'{name}.axis-{axis}.before.json';prior.write_text(json.dumps(before))
                    commands=output/f'{name}.axis-{axis}.commands.jsonl'
                    commands.write_text(json.dumps(dict(command='apply',expected_revision=0,
                        edits=[dict(op='transform',id=1,pose=preview)]))+'\n')
                    expected=output/f'{name}.axis-{axis}.expected.json'
                    subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),
                        '--output',str(expected)],check=True,capture_output=True)
                    assert json.loads(expected.read_text())==changed
                    accepted.append(copy.deepcopy(changed))
                for revision,document in zip(range(4,7),[accepted[1],accepted[0],initial]):
                    control('Command(Undo)');wait(lambda:state()['revision']==revision)
                    assert state()['document']==document
                for revision,document in zip(range(7,10),accepted):
                    control('Command(Redo)');wait(lambda:state()['revision']==revision)
                    assert state()['document']==document
                # Escape cancels a live preview; release outside the viewport
                # also cancels and never creates an undo entry.
                for cancel in ['escape','panel']:
                    select();arrow=handle(1)
                    desktop.move(window,arrow['screen'],1);desktop.button(1,True)
                    try:
                        wait(lambda:state()['dragging'])
                        desktop.move(window,arrow['target'],1)
                        wait(lambda:state()['move_preview']!=accepted[-1]['objects'][0]['pose'])
                        if cancel=='escape': desktop.key('Escape')
                        else: desktop.move(window,[100,700],1)
                    finally: desktop.button(1,False)
                    wait(lambda:not state()['dragging'])
                    assert state()['revision']==9 and state()['document']==accepted[-1] and state()['history']==[3,0]
                select();control('Transform');wait(lambda:state()['transform_edit'] is not None and not handles())
                desktop.key('Escape');wait(lambda:state()['transform_edit'] is None and len(handles())==3)
                control('Command(Play)');wait(lambda:state()['play_feet'] is not None and not handles())
                control('Command(Play)');wait(lambda:state()['play_feet'] is None)
                control('Command(Save)');wait(lambda:saved.exists() and not state()['unsaved_changes'])
                assert json.loads(saved.read_text())==accepted[-1]
                assert state()['revision']==9
                final=state()
                subprocess.run(['wmctrl','-ic',hex(window)],check=True);process.wait(timeout=20)
                assert process.returncode==0
            finally:
                if desktop: desktop.close()
                if process.poll() is None: process.terminate();process.wait(timeout=15)
        logs=logfile.read_text()
        assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
        cases.append(dict(name=name,viewport=final['viewport'],gpu=final['gpu'],validation_layer_active=True,
            gpu_diagnostics=check_gpu_diagnostics(logs,allow_surface_resize=False)))
    result=dict(passed=True,cases=cases,editor_sha256=binary_hash,checks=[
        'actual X/Y/Z handle gestures preview without source changes and match exact headless Transform documents',
        'axis-only snapping preserves other f64 coordinates; stationary click preserves fractional pose',
        'one-entry undo/redo, Escape/panel cancellation, checkbox/modal/play isolation and save'])
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    args=parser.parse_args();run(args.output,args.binary)
