#!/usr/bin/env python3
"""Native New zone, untitled Save As and playtest at 3440x1440."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output,binary,only_case=None,shortcuts=False):
    output=output.resolve();output.mkdir(parents=True,exist_ok=False)
    binary=binary.resolve();binary_hash=hashlib.sha256(binary.read_bytes()).hexdigest()
    cases=[];input_notes=[]
    env=dict(os.environ,WGPU_BACKEND='vulkan',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',VK_LOADER_DEBUG='layer')
    for name,origin,other_origin in [('local',[0,0,0],[128000000000,20,-128000000000]),
                                    ('far',[128000000000,20,-128000000000],[0,0,0])]:
        if only_case and name!=only_case: continue
        folder=output/name;folder.mkdir()
        def fixture(label,anchor,identifier):
            return dict(schema=1,name=label,origin=anchor,spawn=[anchor[0],anchor[1]+.1,anchor[2]+6],
                objects=[dict(id=identifier,name='Browser fixture',primitive='box',pose=dict(
                    position=[anchor[0]+8.125,anchor[1]+1.25,anchor[2]-8.5],yaw_degrees=17.25,size=[2,2,2]))])
        source=folder/'source.zone.json';source.write_text(json.dumps(fixture('Original zone',origin,1)))
        report=folder/'state.json';logfile=folder/'editor.log';saved=folder/'draft.saved.json'
        with logfile.open('w') as log:
            process=subprocess.Popen([str(binary),'--zone',str(source),'--save-path',str(saved),'--report',str(report)],
                env=env,stdout=log,stderr=log)
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
                def click(item):
                    desktop.move(window,item['centre'],1)
                    fresh()
                    if 'target' in item:
                        wait(lambda:any(b['target']==item['target'] and b['hovered'] for b in
                            state()['workspace_controls']+state()['file_controls']))
                    else:
                        wait(lambda:state()['cursor'] and all(abs(a-b)<2 for a,b in zip(state()['cursor'],item['centre'])))
                    def owned():
                        try:desktop.owned_focus(pointer=True);return True
                        except AssertionError:return False
                    wait(owned,15)
                    desktop.button(1,True)
                    try:wait(lambda:state()['left_mouse_down'])
                    finally:desktop.button(1,False)
                    wait(lambda:not state()['left_mouse_down']);fresh()
                def control(target):
                    click(wait(lambda:next((b for b in state()['workspace_controls'] if b['target']==target and all(x>0 for x in b['size'])),None)))
                def file_control(target):
                    click(wait(lambda:next((b for b in state()['file_controls'] if b['target']==target and all(x>0 for x in b['size'])),None)))
                def chord(modifiers,key):
                    pressed=[]
                    try:
                        for modifier in modifiers:
                            flag='control_down' if modifier=='Control_L' else 'shift_down'
                            for attempt in range(3):
                                desktop.key_state(modifier,True)
                                try:wait(lambda:state()[flag],1);break
                                except TimeoutError:
                                    desktop.key_state(modifier,False)
                                    input_notes.append(dict(case=name,modifier=modifier,missed_attempt=attempt+1))
                            else:raise AssertionError('Owned modifier was not delivered')
                            pressed.append(modifier)
                        desktop.key(key);fresh()
                    finally:
                        for modifier in reversed(pressed):desktop.key_state(modifier,False)
                    wait(lambda:not state()['control_down'] and not state()['shift_down'])
                def replace(text):
                    file_control('Field');chord(['Control_L'],'a')
                    wait(lambda:state()['file_dialog']['selection']==[0,len(state()['file_dialog']['text'].encode())])
                    for character in text:desktop.key({'.':'period','/':'slash','-':'minus',' ':'space'}.get(character,character))
                    wait(lambda:state()['file_dialog']['text']==text)
                def capture(label):
                    desktop.key('F12');path=report.with_suffix('.png')
                    wait(lambda:capture_ready(path),30);path.rename(folder/f'{label}.png')
                original_bytes=source.read_bytes()
                initial=copy.deepcopy(state()['document'])
                for item in state()['workspace_controls']:
                    if item['centre'][1]<92 and all(v>0 for v in item['size']):
                        x,y=item['centre'];w,h=item['size']
                        assert x-w/2>=0 and x+w/2<=3440 and y-h/2>=0 and y+h/2<=92,item
                if shortcuts:
                    view=copy.deepcopy(state()['view'])
                    chord(['Control_L'],'f');wait(lambda:state()['name_edit'] is not None)
                    assert state()['text_form_kind']=='search' and state()['view']==view
                    chord(['Control_L'],'n');fresh()
                    assert state()['document']==initial and state()['revision']==0 and state()['name_edit'] is not None
                    desktop.key('Escape');wait(lambda:state()['name_edit'] is None)
                    chord(['Control_L'],'n')
                else:
                    control('New')
                wait(lambda:state()['save_path'] is None)
                blank=copy.deepcopy(state()['document'])
                assert blank['objects']==[] and blank['origin']==[0,0,0]
                assert state()['revision']==1 and state()['history']==[1,0] and state()['unsaved_changes']
                assert 'Untitled' in state()['window_title']
                control('Command(Load)');fresh()
                assert state()['revision']==1 and state()['document']==blank and state()['save_path'] is None
                capture('new-untitled')
                control('Command(Undo)');wait(lambda:state()['revision']==2)
                assert state()['document']==initial and state()['save_path'] is None
                control('Command(Redo)');wait(lambda:state()['revision']==3)
                assert state()['document']==blank and state()['save_path'] is None
                control('New');fresh()
                assert state()['revision']==3 and state()['history']==[1,0]
                control('Command(Save)');wait(lambda:state()['file_dialog'] is not None)
                assert state()['file_dialog']['mode']=='SaveAs' and state()['file_dialog']['text']=='untitled.zone.json'
                assert Path(state()['file_dialog']['folder'])==folder
                assert source.read_bytes()==original_bytes and not saved.exists()
                capture('first-save-browser')
                file_control('Cancel');wait(lambda:state()['file_dialog'] is None)
                assert state()['save_path'] is None and state()['document']==blank
                control('Command(Play)');wait(lambda:state()['play_feet'] is not None)
                capture('untitled-playtest')
                if shortcuts:
                    chord(['Control_L'],'n');chord(['Control_L'],'f')
                    assert state()['name_edit'] is None
                else:
                    control('New')
                fresh()
                assert state()['play_feet'] is not None and state()['revision']==3
                control('Command(Play)');wait(lambda:state()['play_feet'] is None)
                control('Command(Save)');wait(lambda:state()['file_dialog'] is not None)
                replace('missing/new');file_control('Apply')
                wait(lambda:'No such file' in state()['status'])
                assert state()['save_path'] is None and state()['document']==blank
                replace('new zone');file_control('Apply')
                active=folder/'new zone.zone.json'
                wait(lambda:state()['file_dialog'] is None and active.exists())
                assert json.loads(active.read_text())==blank and Path(state()['save_path'])==active
                assert not state()['unsaved_changes'] and state()['revision']==3 and state()['history']==[1,0]
                assert source.read_bytes()==original_bytes and not saved.exists()
                control('Command(Undo)');wait(lambda:state()['revision']==4)
                assert state()['document']==initial and Path(state()['save_path'])==active and state()['unsaved_changes']
                control('Command(Redo)');wait(lambda:state()['revision']==5)
                assert state()['document']==blank and not state()['unsaved_changes']
                capture('saved-new-zone');final=state()
                subprocess.run(['wmctrl','-ic',hex(window)],check=True);process.wait(timeout=20)
                assert process.returncode==0
            finally:
                if desktop:desktop.close()
                if process.poll() is None:process.terminate();process.wait(timeout=15)
        logs=logfile.read_text();assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
        cases.append(dict(name=name,viewport=final['viewport'],gpu=final['gpu'],validation_layer_active=True,
            gpu_diagnostics=check_gpu_diagnostics(logs,allow_surface_resize=False)))
        (folder/'qualification.json').write_text(json.dumps(dict(case=cases[-1],editor_sha256=binary_hash),indent=2)+'\n')
    result=dict(passed=True,shortcuts=shortcuts,cases=cases,input_notes=input_notes,editor_sha256=binary_hash,checks=[
        'New creates untitled source with one undo, preserves the original file and keeps header controls inside 3440x92',
        'Undo/redo restores prior content while retaining untitled file identity; repeated empty New preserves history',
        'first Save opens Save As in previous directory; cancel and failed save preserve untitled source',
        'untitled Avian playtest runs and disables New; successful Save assigns a path without history and restores clean state'])
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--case',choices=['local','far'])
    parser.add_argument('--shortcuts',action='store_true',help='Exercise Ctrl+N/Ctrl+F and modal/play guards')
    args=parser.parse_args();run(args.output,args.binary,args.case,args.shortcuts)
