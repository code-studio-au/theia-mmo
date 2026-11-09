#!/usr/bin/env python3
"""Native primitive placement dimensions and shared undo at 3440x1440."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output,binary,only_case=None):
    output=output.resolve();output.mkdir(parents=True,exist_ok=False);binary=binary.resolve()
    cases=[];oracles=0
    env=dict(os.environ,WGPU_BACKEND='vulkan',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',VK_LOADER_DEBUG='layer')
    for name,origin in [('local',[.125,.25,-.5]),('far',[128000000000.125,20.25,-128000000000.5])]:
        if only_case and name!=only_case:continue
        folder=output/name;folder.mkdir();report=folder/'state.json';logfile=folder/'editor.log'
        source=folder/'source.json';saved=folder/'saved.json'
        source.write_text(json.dumps(dict(schema=1,name='Placement dimensions',origin=origin,
            spawn=[origin[0],origin[1]+.1,origin[2]+6],objects=[dict(id=1,name='Precise box',primitive='box',
                pose=dict(position=[origin[0]+8.125,origin[1]+1.25,origin[2]-8.5],yaw_degrees=17.25,size=[2,2,2]))])))
        with logfile.open('w') as log:
            process=subprocess.Popen([str(binary),'--zone',str(source),'--save-path',str(saved),'--report',str(report)],env=env,stdout=log,stderr=log)
            desktop=None
            try:
                window=wait(lambda:owned_window(process.pid),45);desktop=Desktop();desktop.focus(window)
                def state():
                    assert process.poll() is None,'Editor exited'
                    return json.loads(report.read_text())
                wait(lambda:report.exists() and state()['viewport']==[3440,1440],45)
                desktop.move(window,[1737,731],1);desktop.move(window,[1720,720],1)
                def fresh():
                    elapsed=state()['input_elapsed'];wait(lambda:state()['input_elapsed']>elapsed);return state()
                def entries(group):return state()['content_list']['controls'] if group=='content' else state()[group]
                def press():
                    desktop.owned_focus(pointer=True);desktop.button(1,True)
                    try:wait(lambda:state()['left_mouse_down'])
                    finally:desktop.button(1,False)
                    wait(lambda:not state()['left_mouse_down']);fresh()
                def click(point):
                    desktop.move(window,point,1)
                    wait(lambda:state()['cursor'] and max(abs(a-b) for a,b in zip(state()['cursor'],point))<2)
                    press()
                def control(target,group='workspace_controls'):
                    item=wait(lambda:next((b for b in entries(group) if b['target']==target and all(v>0 for v in b['size'])),None))
                    desktop.move(window,item['centre'],1)
                    wait(lambda:any(b['target']==target and b['hovered'] for b in entries(group)))
                    press()
                def tool(action):click(next(b['centre'] for b in state()['buttons'] if b['action']==action and all(v>0 for v in b['size'])))
                def capture(label):
                    desktop.key('F12');screenshot=report.with_suffix('.png');wait(lambda:capture_ready(screenshot),30);screenshot.rename(folder/f'{label}.png')
                def oracle(before,edits,label):
                    nonlocal oracles
                    prior=folder/f'{label}.before.json';prior.write_text(json.dumps(before));commands=folder/f'{label}.commands.jsonl';expected=folder/f'{label}.expected.json'
                    commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=edits))+'\n')
                    subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),'--output',str(expected)],check=True,capture_output=True)
                    assert json.loads(expected.read_text())==state()['document'];oracles+=1
                initial=copy.deepcopy(fresh()['document'])
                records=[]
                def size(axis,desired):
                    before=copy.deepcopy(fresh())
                    target=f'PlacementSize({axis})'
                    item=wait(lambda:next((c for c in state()['workspace_controls'] if c['target']==target and c['size'][1]>0),None))
                    assert item['range']==[.25,32.]
                    x,y=item['centre'];w,h=item['size']
                    assert x-w/2>=3140 and x+w/2<=3440 and y-h/2>=840 and y+h/2<=1440
                    thumb=[x-w/2+7+(w-14)*(item['value']-.25)/31.75,y]
                    end=[x-w/2+7+(w-14)*(desired-.25)/31.75,y]
                    desktop.move(window,thumb,1);desktop.button(1,True)
                    try:
                        wait(lambda:state()['left_mouse_down'])
                        desktop.move(window,end,1);fresh()
                        assert state()['document']==before['document'] and state()['history']==before['history'] and state()['revision']==before['revision']
                    finally:desktop.button(1,False)
                    wait(lambda:not state()['left_mouse_down'])
                    wait(lambda:next(c for c in state()['workspace_controls'] if c['target']==target)['value']==desired)
                    assert state()['document']==before['document'] and state()['history']==before['history'] and state()['revision']==before['revision']
                for shape,dimensions in [('Box',[8.,3.25,2.5]),('Sphere',[4.25]*3),('Cylinder',[6.5,2.,6.5])]:
                    tool(f'Place({shape})');fresh()
                    axes=3 if shape=='Box' else 1 if shape=='Sphere' else 2
                    for axis in range(axes):size(axis,dimensions[axis])
                    current=fresh()
                    controls={c['target']:c for c in current['workspace_controls']}
                    for axis in range(3):assert (controls[f'PlacementSize({axis})']['size'][1]>0)==(axis<axes)
                    capture(f'{shape.lower()}-placement-controls')
                    before=copy.deepcopy(fresh());click(before['ground'][0])
                    wait(lambda:state()['revision']==before['revision']+1)
                    placed=copy.deepcopy(state()['document']);entry=placed['objects'][-1]
                    assert entry['primitive']==shape.lower() and entry['pose']['size']==dimensions
                    assert entry['pose']['position'][1]==origin[1]+dimensions[1]/2.
                    for axis in [0,2]:assert (entry['pose']['position'][axis]-origin[axis]).is_integer()
                    assert state()['history']==[1,0]
                    oracle(before['document'],[dict(op='place',object=entry)],shape.lower())
                    control('Command(Undo)');wait(lambda:state()['revision']==before['revision']+2)
                    assert state()['document']==initial and state()['history']==[0,1]
                    size(0,dimensions[0]+.25)
                    assert state()['history']==[0,1]
                    control('Command(Redo)');wait(lambda:state()['revision']==before['revision']+3)
                    assert state()['document']==placed and state()['history']==[1,0]
                    control('Command(Undo)');wait(lambda:state()['revision']==before['revision']+4)
                    assert state()['document']==initial and state()['history']==[0,1]
                    records.append(dict(shape=shape,dimensions=dimensions,full_document_oracle_equal=True,undo_redo=True))
                control('Command(Redo)');wait(lambda:state()['document']==placed)
                desktop.move(window,[1720,720],1);desktop.key('Escape');fresh()
                assert all(c['size'][1]==0 for c in state()['workspace_controls'] if c['target'].startswith('PlacementSize('))
                control('Command(Save)');wait(lambda:not state()['unsaved_changes'])
                assert json.loads(saved.read_text())==placed
                capture('final-placement');final=state()
                subprocess.run(['wmctrl','-ic',hex(window)],check=True);process.wait(timeout=20);assert process.returncode==0
            finally:
                if desktop:desktop.close()
                if process.poll() is None:process.terminate();process.wait(timeout=15)
        logs=logfile.read_text();assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
        cases.append(dict(name=name,gpu=final['gpu'],viewport=final['viewport'],records=records,validation_layer_active=True,gpu_diagnostics=check_gpu_diagnostics(logs,allow_surface_resize=False)))
        (folder/'qualification.json').write_text(json.dumps(cases[-1],indent=2)+'\n')
    result=dict(cases=cases,editor_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),oracles=oracles,checks=[
        'shape-specific quarter-step dimension sliders fit Properties and preserve source/revision/redo during held drags',
        'native box/sphere/cylinder placement uses chosen dimensions and half-height support with origin-relative snapping',
        'complete documents equal shared structured commands; undo/redo and exact saves retain placed dimensions'],scope='Linux owned input at 3440x1440; local/far fractional origins; no Windows or multiplayer claim')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'));parser.add_argument('--case',choices=['local','far'])
    args=parser.parse_args();run(args.output,args.binary,args.case)
