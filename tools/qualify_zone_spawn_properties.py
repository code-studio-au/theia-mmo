#!/usr/bin/env python3
"""Exact playtest spawn fields through owned standard Bevy input at 3440x1440."""
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
    cases=[];input_notes=[]
    env=dict(os.environ,WGPU_BACKEND='vulkan',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',VK_LOADER_DEBUG='layer')
    for name,origin in [('local',[0,0,0]),('far',[128000000000,20,-128000000000])]:
        if only_case and name!=only_case:continue
        folder=output/name;folder.mkdir();report=folder/'state.json';logfile=folder/'editor.log'
        source=folder/'source.json';saved=folder/'saved.json'
        source.write_text(json.dumps(dict(schema=1,name='Spawn properties',origin=origin,
            spawn=[origin[0]+8.125,origin[1]+.125,origin[2]+6.5],objects=[dict(id=1,name='Untouched object',primitive='box',
            pose=dict(position=[origin[0]+24,origin[1]+1,origin[2]+24],yaw_degrees=17.25,size=[2,2,2]))])))
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
                def entries(group):
                    return state()['content_list']['controls'] if group=='content' else state()[group]
                def click(item,group):
                    desktop.move(window,item['centre'],1)
                    wait(lambda:any(b['target']==item['target'] and b['hovered'] for b in entries(group)))
                    desktop.owned_focus(pointer=True);desktop.button(1,True)
                    try:wait(lambda:state()['left_mouse_down'])
                    finally:desktop.button(1,False)
                    wait(lambda:not state()['left_mouse_down']);fresh()
                def control(target,group='workspace_controls'):
                    item=wait(lambda:next((b for b in entries(group) if b['target']==target and all(v>0 for v in b['size'])),None))
                    click(item,group)
                def form(target):control(target,'transform_controls')
                def field(index,value):
                    form(f'Field({index})');wait(lambda:state()['transform_edit']['active']==index)
                    for attempt in range(3):
                        desktop.key_state('Control_L',True)
                        try:wait(lambda:state()['control_down'],1);break
                        except TimeoutError:
                            desktop.key_state('Control_L',False);input_notes.append(dict(case=name,modifier='Control_L',missed_attempt=attempt+1))
                    else:raise AssertionError('Owned modifier was not delivered')
                    try:desktop.key('a')
                    finally:desktop.key_state('Control_L',False)
                    wait(lambda:not state()['control_down'] and state()['transform_edit']['selection']==[0,len(state()['transform_edit']['fields'][index])])
                    for character in value:desktop.key({'.':'period','-':'minus'}.get(character,character))
                    wait(lambda:state()['transform_edit']['fields'][index]==value)
                def open_form():
                    control('SpawnTransform');wait(lambda:state()['transform_edit'] is not None)
                    assert len(state()['transform_edit']['fields'])==3 and state()['transform_edit']['id'] is None
                    assert all(not all(v>0 for v in b['size']) for b in state()['transform_controls'] if b['target'] in ('Field(3)','Field(4)','Field(5)','Field(6)'))
                def capture(label):
                    desktop.key('F12');screenshot=report.with_suffix('.png');wait(lambda:capture_ready(screenshot),30);screenshot.rename(folder/f'{label}.png')
                initial=copy.deepcopy(fresh()['document'])
                control('Object(1)','content');wait(lambda:state()['selected']==1)
                open_form();form('Apply');wait(lambda:state()['transform_edit'] is None)
                assert state()['revision']==0 and state()['history']==[0,0]
                open_form();field(0,'nan');form('Apply')
                wait(lambda:'finite number' in state()['status'])
                assert state()['document']==initial and state()['revision']==0
                field(0,str(origin[0]+999));form('Apply');fresh()
                assert state()['transform_edit'] is not None and state()['document']==initial and state()['revision']==0
                position=[origin[0]+4.125,origin[1]+.125,origin[2]+6.5]
                for index,value in enumerate(position):field(index,str(value))
                capture('spawn-fields');form('Apply');wait(lambda:state()['revision']==1 and state()['transform_edit'] is None)
                changed=copy.deepcopy(state()['document']);assert changed['spawn']==position
                assert changed['objects']==initial['objects'] and state()['history']==[1,0] and state()['selected']==1
                prior=folder/'before.json';prior.write_text(json.dumps(initial));commands=folder/'commands.jsonl';expected=folder/'expected.json'
                commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=[dict(op='set_spawn',position=position)]))+'\n')
                subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),'--output',str(expected)],check=True,capture_output=True)
                assert json.loads(expected.read_text())==changed
                control('Command(Undo)');wait(lambda:state()['revision']==2);assert state()['document']==initial
                history=state()['history'];open_form();form('Apply');wait(lambda:state()['transform_edit'] is None)
                assert state()['revision']==2 and state()['history']==history
                control('Command(Redo)');wait(lambda:state()['revision']==3);assert state()['document']==changed
                open_form();field(0,str(origin[0]+99));form('Cancel');wait(lambda:state()['transform_edit'] is None)
                assert state()['document']==changed and state()['revision']==3
                # The existing solver searches only 10 cm below the requested
                # support. A valid authored point at 12.5 cm is not a playable
                # landing; refusal must preserve source/history before correction.
                control('Command(Play)');wait(lambda:'Playtest spawn refused' in state()['status'])
                assert state()['play_feet'] is None and state()['document']==changed and state()['revision']==3
                capture('unsupported-spawn')
                open_form();field(1,str(origin[1]+.0625));form('Apply')
                wait(lambda:state()['revision']==4 and state()['transform_edit'] is None)
                corrected=copy.deepcopy(state()['document'])
                corrected_position=[position[0],origin[1]+.0625,position[2]]
                assert corrected['spawn']==corrected_position
                prior=folder/'correction.before.json';prior.write_text(json.dumps(changed))
                commands=folder/'correction.commands.jsonl';expected=folder/'correction.expected.json'
                commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=[dict(op='set_spawn',position=corrected_position)]))+'\n')
                subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),'--output',str(expected)],check=True,capture_output=True)
                assert json.loads(expected.read_text())==corrected
                control('Command(Undo)');wait(lambda:state()['revision']==5);assert state()['document']==changed
                control('Command(Redo)');wait(lambda:state()['revision']==6);assert state()['document']==corrected
                control('Command(Play)');wait(lambda:state()['play_feet'] is not None)
                assert abs(state()['play_feet'][0]-4.125)<.001 and abs(state()['play_feet'][2]-6.5)<.001
                control('SpawnTransform');assert state()['transform_edit'] is None and state()['revision']==6
                control('Command(Play)');wait(lambda:state()['play_feet'] is None)
                control('Mode(Guides)')
                item=next(b for b in state()['buttons'] if b['action']=='NewPath(Road)' and all(v>0 for v in b['size']))
                desktop.move(window,item['centre'],1);desktop.button(1,True)
                try:wait(lambda:state()['left_mouse_down'])
                finally:desktop.button(1,False)
                wait(lambda:state()['path_draft_points']==0)
                control('SpawnTransform');assert state()['transform_edit'] is None and state()['path_draft_points']==0
                desktop.key('Escape');wait(lambda:state()['path_draft_points'] is None)
                control('Command(Save)');wait(lambda:not state()['unsaved_changes'])
                assert json.loads(saved.read_text())==corrected
                capture('saved-spawn');final=state();subprocess.run(['wmctrl','-ic',hex(window)],check=True);process.wait(timeout=20);assert process.returncode==0
            finally:
                if desktop:desktop.close()
                if process.poll() is None:process.terminate();process.wait(timeout=15)
        logs=logfile.read_text();assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
        cases.append(dict(name=name,gpu=final['gpu'],viewport=final['viewport'],validation_layer_active=True,gpu_diagnostics=check_gpu_diagnostics(logs,allow_surface_resize=False)))
        (folder/'qualification.json').write_text(json.dumps(cases[-1],indent=2)+'\n')
    result=dict(cases=cases,input_notes=input_notes,editor_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),checks=[
        'three exact world-coordinate spawn fields and hidden unused rows, finite/range rejection, cancel, no-op and pending redo',
        'complete native document equals shared headless SetSpawn, one-entry undo/redo/save, unsupported spawn refusal/correction, actual Avian playtest placement and busy refusal'],scope='Linux owned OS input at 3440x1440 only; no Windows or multiplayer claim')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--case',choices=['local','far'])
    args=parser.parse_args();run(args.output,args.binary,args.case)
