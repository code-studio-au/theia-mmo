#!/usr/bin/env python3
"""Native grid presets, relative-origin placement and shared undo at 3440x1440."""
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
        source.write_text(json.dumps(dict(schema=1,name='Grid presets',origin=origin,
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
                def step(index,expected,busy=False):
                    before=copy.deepcopy(state());item=next(b for b in before['workspace_controls'] if b['target']=='MoveGrid')
                    x,y=item['centre'];width=item['size'][0]
                    click([x-width/2+7+(width-14)*index/6,y])
                    if not busy:wait(lambda:state()['move_step_metres']==expected)
                    else:assert state()['move_step_metres']==before['move_step_metres']
                    assert state()['document']==before['document'] and state()['revision']==before['revision'] and state()['history']==before['history']
                def capture(label):
                    desktop.key('F12');screenshot=report.with_suffix('.png');wait(lambda:capture_ready(screenshot),30);screenshot.rename(folder/f'{label}.png')
                def oracle(before,edits,label):
                    nonlocal oracles
                    prior=folder/f'{label}.before.json';prior.write_text(json.dumps(before));commands=folder/f'{label}.commands.jsonl';expected=folder/f'{label}.expected.json'
                    commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=edits))+'\n')
                    subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),'--output',str(expected)],check=True,capture_output=True)
                    assert json.loads(expected.read_text())==state()['document'];oracles+=1
                def quantized(position):
                    for axis in [0,2]:assert ((position[axis]-origin[axis])/.25).is_integer(),position
                initial=copy.deepcopy(fresh()['document']);assert state()['move_step_metres']==1
                for index,expected in [(6,8),(0,.125),(2,.5),(1,.25),(0,.125)]:step(index,expected)
                control('Object(1)','content');wait(lambda:state()['selected']==1);control('Frame')
                arrow=next(h for h in state()['move_handles'] if h['visible'] and h['axis']==1)
                before=copy.deepcopy(state());start=arrow['screen'];end=[a+(b-a)*.1875 for a,b in zip(start,arrow['target'])]
                desktop.move(window,start,1);fresh();desktop.button(1,True)
                try:
                    wait(lambda:state()['dragging']);desktop.move(window,end,1)
                    preview=wait(lambda:state()['move_preview'] if state()['move_preview'] and state()['move_preview']!=before['document']['objects'][0]['pose'] else None)
                    assert preview['position'][1]==before['document']['objects'][0]['pose']['position'][1]+.375,preview
                    assert preview['position'][0]==before['document']['objects'][0]['pose']['position'][0]
                    assert preview['position'][2]==before['document']['objects'][0]['pose']['position'][2]
                    assert state()['document']==before['document'] and state()['revision']==0
                    capture('fine-axis-preview')
                finally:desktop.button(1,False)
                wait(lambda:state()['revision']==1);moved=copy.deepcopy(state()['document'])
                oracle(initial,[dict(op='transform',id=1,pose=preview)],'fine-axis')
                control('Command(Undo)');wait(lambda:state()['revision']==2);assert state()['document']==initial
                step(6,8);step(1,.25);assert state()['history']==[0,1]
                control('Command(Redo)');wait(lambda:state()['revision']==3);assert state()['document']==moved
                # Keyboard nudging remains relative, with the selected metre step.
                desktop.move(window,[1720,720],1);before=copy.deepcopy(fresh());desktop.key('Right')
                wait(lambda:state()['revision']==4)
                pose=state()['document']['objects'][0]['pose']
                assert pose['position'][0]==before['document']['objects'][0]['pose']['position'][0]+.25
                assert state()['move_step_metres']==.25
                oracle(before['document'],[dict(op='transform',id=1,pose=pose)],'object-nudge')
                control('Spawn','content');control('View(Top)')
                # Deliberately offset the projected whole-metre ground point so
                # the placement demonstrates the finer grid rather than luck.
                def ground(offset):
                    base=state()['ground'][0];return [base[0]+offset[0],base[1]+offset[1]]
                tool('Place(Box)');before=copy.deepcopy(state());click(ground([37,19]))
                wait(lambda:state()['revision']==before['revision']+1)
                placed=state()['document']['objects'][-1];quantized(placed['pose']['position'])
                assert any((placed['pose']['position'][i]-origin[i])%1 for i in [0,2])
                oracle(before['document'],[dict(op='place',object=placed)],'object-place')
                # The same setting controls plant and marker placement and keys.
                for mode,action,collection,edit_name in [('Plants','Plant(Tree, false)','plants','put_plant'),('Markers','PlaceMarker(Npc)','markers','put_marker')]:
                    control(f'Mode({mode})');assert state()['move_step_metres']==.25
                    tool(action);before=copy.deepcopy(state());click(ground([37 if mode=='Plants' else -22,19]))
                    wait(lambda:state()['revision']==before['revision']+1)
                    entry=copy.deepcopy(state()['document'][collection][-1]);quantized(entry['position'])
                    field='plant' if mode=='Plants' else 'marker'
                    oracle(before['document'],[dict(op=edit_name,**{field:entry})],f'{mode.lower()}-place')
                    before=copy.deepcopy(state());desktop.move(window,[1720,720],1);fresh();desktop.key('Right')
                    wait(lambda:state()['revision']==before['revision']+1)
                    changed=copy.deepcopy(state()['document'][collection][-1])
                    assert changed['position'][0]==entry['position'][0]+.25 and state()['move_step_metres']==.25
                    oracle(before['document'],[dict(op=edit_name,**{field:changed})],f'{mode.lower()}-nudge')
                control('Mode(Guides)');tool('NewPath(Road)');wait(lambda:state()['path_draft_points']==0)
                before=copy.deepcopy(state());click(ground([37,19]));click(ground([127,19]))
                wait(lambda:state()['path_draft_points']==2);step(0,.125,busy=True)
                tool('FinishPath');wait(lambda:state()['revision']==before['revision']+1)
                path=copy.deepcopy(state()['document']['paths'][-1])
                for point in path['points']:quantized(point)
                oracle(before['document'],[dict(op='put_path',path=path)],'guide-place')
                before=copy.deepcopy(state());desktop.move(window,[1720,720],1);fresh();desktop.key('Right')
                wait(lambda:state()['revision']==before['revision']+1)
                changed=state()['document']['paths'][-1]
                assert all(b[0]==a[0]+.25 for a,b in zip(path['points'],changed['points']))
                oracle(before['document'],[dict(op='put_path',path=changed)],'guide-nudge')
                control('Command(Save)');wait(lambda:not state()['unsaved_changes'])
                assert json.loads(saved.read_text())==state()['document']
                capture('grid-properties');final=state();subprocess.run(['wmctrl','-ic',hex(window)],check=True);process.wait(timeout=20);assert process.returncode==0
            finally:
                if desktop:desktop.close()
                if process.poll() is None:process.terminate();process.wait(timeout=15)
        logs=logfile.read_text();assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
        cases.append(dict(name=name,gpu=final['gpu'],viewport=final['viewport'],validation_layer_active=True,gpu_diagnostics=check_gpu_diagnostics(logs,allow_surface_resize=False)))
        (folder/'qualification.json').write_text(json.dumps(cases[-1],indent=2)+'\n')
    result=dict(cases=cases,editor_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),oracles=oracles,checks=[
        'standard discrete grid slider endpoints/presets preserve source/history and pending redo',
        'fine axis drag, keyboard nudges and primitive/plant/marker/guide placement use the selected origin-relative grid',
        'complete native documents equal shared headless commands; busy guide draft refuses preference changes; exact save'],scope='Linux owned input at 3440x1440; fractional local/far origins; no Windows or multiplayer claim')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'));parser.add_argument('--case',choices=['local','far'])
    args=parser.parse_args();run(args.output,args.binary,args.case)
