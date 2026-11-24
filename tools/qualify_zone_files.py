#!/usr/bin/env python3
"""Native Bevy file-browser save/open/overwrite and shared history at 3440x1440."""
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
    output=output.resolve();output.mkdir(parents=True,exist_ok=False)
    binary=binary.resolve();binary_hash=hashlib.sha256(binary.read_bytes()).hexdigest()
    cases=[];input_notes=[]
    env=dict(os.environ,WGPU_BACKEND='vulkan',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',VK_LOADER_DEBUG='layer')
    for name,origin,other_origin in [('local',[0,0,0],[128000000000,20,-128000000000]),
                                    ('far',[128000000000,20,-128000000000],[0,0,0])]:
        if only_case and name!=only_case: continue
        folder=output/name;folder.mkdir()
        (folder/'nested').mkdir()
        def fixture(label,anchor,identifier):
            return dict(schema=1,name=label,origin=anchor,spawn=[anchor[0],anchor[1]+.1,anchor[2]+6],
                objects=[dict(id=identifier,name='Browser fixture',primitive='box',pose=dict(
                    position=[anchor[0]+8.125,anchor[1]+1.25,anchor[2]-8.5],yaw_degrees=17.25,size=[2,2,2]))])
        source=folder/'source.zone.json';source.write_text(json.dumps(fixture('Original zone',origin,1)))
        other=folder/'other.zone.json';other.write_text(json.dumps(fixture('Opened zone',other_origin,9)))
        overwrite=folder/'overwrite.zone.json';overwrite.write_bytes(other.read_bytes())
        invalid=folder/'invalid.zone.json';invalid.write_text('not a zone document')
        for index in range(16): (folder/f'page-{index:02}.json').write_bytes(source.read_bytes())
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
                def row(filename):
                    while state()['file_dialog']['page']>0:
                        page=state()['file_dialog']['page'];file_control('Previous');wait(lambda:state()['file_dialog']['page']==page-1)
                    for _ in range(state()['file_dialog']['pages']):
                        item=next((b for b in state()['file_controls'] if b['entry'] and b['entry']['name']==filename and all(x>0 for x in b['size'])),None)
                        if item:click(item);return
                        page=state()['file_dialog']['page']
                        assert page+1<state()['file_dialog']['pages'],f'Missing browser entry {filename}'
                        file_control('Next');wait(lambda:state()['file_dialog']['page']==page+1)
                    raise AssertionError(f'Missing browser entry {filename}')
                def capture(label):
                    desktop.key('F12');path=report.with_suffix('.png')
                    wait(lambda:capture_ready(path),30);path.rename(folder/f'{label}.png')
                def title_check():
                    current=state()
                    expected=f"{'* ' if current['unsaved_changes'] else ''}{current['document']['name']} — {Path(current['save_path']).name} — Theia Zone Studio"
                    wait(lambda:state()['window_title']==expected)
                title_check()
                initial=copy.deepcopy(state()['document']);initial_path=state()['save_path'];initial_view=state()['view']
                control('FileSaveAs');wait(lambda:state()['file_dialog'] and state()['file_dialog']['mode']=='SaveAs')
                assert state()['file_dialog']['pages']>=2
                replace('bad.glb');file_control('Apply')
                wait(lambda:'.json extension' in state()['status'])
                assert state()['file_dialog']['text']=='bad.glb' and state()['save_path']==initial_path
                assert state()['document']==initial and state()['revision']==0 and state()['history']==[0,0]
                replace('renamed');row('nested')
                wait(lambda:Path(state()['file_dialog']['folder'])==folder/'nested')
                assert state()['file_dialog']['text']=='renamed'
                file_control('Up');wait(lambda:Path(state()['file_dialog']['folder'])==folder)
                assert state()['file_dialog']['text']=='renamed'
                file_control('Next');wait(lambda:state()['file_dialog']['page']==1)
                file_control('Previous');wait(lambda:state()['file_dialog']['page']==0)
                assert state()['document']==initial and state()['view']==initial_view
                capture('save-as-ready');file_control('Apply')
                renamed=folder/'renamed.zone.json'
                wait(lambda:state()['file_dialog'] is None and renamed.exists() and not state()['unsaved_changes'])
                title_check()
                assert Path(state()['save_path'])==renamed and json.loads(renamed.read_text())==initial
                assert state()['revision']==0 and state()['history']==[0,0] and not saved.exists()
                # First overwrite request is a review step; typing or Cancel
                # never writes and the active file only changes on success.
                previous_bytes=overwrite.read_bytes()
                control('FileSaveAs');wait(lambda:state()['file_dialog'] is not None)
                row('overwrite.zone.json');file_control('Apply')
                wait(lambda:state()['file_dialog']['overwrite'])
                assert overwrite.read_bytes()==previous_bytes and Path(state()['save_path'])==renamed
                capture('overwrite-confirmation')
                replace('cancelled');assert not state()['file_dialog']['overwrite']
                file_control('Cancel');wait(lambda:state()['file_dialog'] is None)
                assert overwrite.read_bytes()==previous_bytes and not (folder/'cancelled.zone.json').exists()
                assert state()['document']==initial and state()['history']==[0,0] and not state()['unsaved_changes']
                chord(['Control_L','Shift_L'],'s');wait(lambda:state()['file_dialog'] is not None)
                row('overwrite.zone.json');file_control('Apply');wait(lambda:state()['file_dialog']['overwrite'])
                assert overwrite.read_bytes()==previous_bytes
                file_control('Apply');wait(lambda:state()['file_dialog'] is None)
                assert json.loads(overwrite.read_text())==initial and Path(state()['save_path'])==overwrite
                assert state()['revision']==0 and state()['history']==[0,0]
                # Ctrl O opens the browser. Malformed input preserves all live
                # state, then a selected valid zone uses the shared replace/undo.
                chord(['Control_L'],'o');wait(lambda:state()['file_dialog'] and state()['file_dialog']['mode']=='Open')
                row('invalid.zone.json');file_control('Apply')
                wait(lambda:state()['file_dialog'] and state()['status']!='Choose a zone file')
                assert state()['document']==initial and state()['revision']==0 and Path(state()['save_path'])==overwrite
                for key in ['F1','F2','F5','F8','F9','Delete']:desktop.key(key)
                assert state()['document']==initial and state()['revision']==0 and state()['play_feet'] is None
                row('other.zone.json');wait(lambda:state()['status']=='Choose a zone file')
                capture('open-ready');file_control('Apply')
                wait(lambda:state()['file_dialog'] is None and state()['revision']==1)
                opened=copy.deepcopy(state()['document'])
                assert opened['name']=='Opened zone' and opened['origin']==other_origin
                assert Path(state()['save_path'])==other and state()['history']==[1,0] and not state()['unsaved_changes']
                title_check();capture('opened-file-context')
                assert state()['view']['preset']=='Perspective'
                control('Command(Undo)');wait(lambda:state()['revision']==2)
                assert state()['document']==initial and Path(state()['save_path'])==other and state()['unsaved_changes']
                title_check();capture('unsaved-file-context')
                control('Command(Redo)');wait(lambda:state()['revision']==3)
                assert state()['document']==opened and not state()['unsaved_changes']
                title_check()
                # Reconciled IDs and frame conversion work after opening a
                # different global origin through the actual browser.
                click(next(b for b in state()['buttons'] if b['action']=='Place(Box)' and all(x>0 for x in b['size'])))
                fresh();click(dict(centre=state()['ground'][0]))
                wait(lambda:state()['revision']==4)
                assert [o['id'] for o in state()['document']['objects']]==[9,10]
                control('Command(Undo)');wait(lambda:state()['revision']==5)
                assert state()['document']==opened and state()['history']==[1,1]
                control('FileSaveAs');wait(lambda:state()['file_dialog'] is not None)
                replace('after-open');file_control('Apply');wait(lambda:state()['file_dialog'] is None)
                active=folder/'after-open.zone.json'
                assert Path(state()['save_path'])==active and json.loads(active.read_text())==opened
                assert state()['revision']==5 and state()['history']==[1,1]
                control('Command(Redo)');wait(lambda:state()['revision']==6)
                assert len(state()['document']['objects'])==2
                control('Command(Undo)');wait(lambda:state()['revision']==7)
                assert state()['document']==opened and not state()['unsaved_changes']
                control('FileSaveAs');wait(lambda:state()['file_dialog'] is not None)
                replace('missing/copy');file_control('Apply')
                wait(lambda:'No such file' in state()['status'])
                title_check()
                assert state()['file_dialog']['text']=='missing/copy' and Path(state()['save_path'])==active
                assert state()['document']==opened and state()['history']==[1,1] and not (folder/'missing').exists()
                desktop.key('Escape');wait(lambda:state()['file_dialog'] is None and not state()['unsaved_changes'])
                # Busy playtest/draft shortcuts do not fall through to Save or
                # discard pending authoring. Reload remains an explicit action.
                control('Command(Play)');wait(lambda:state()['play_feet'] is not None)
                before_mtime=active.stat().st_mtime_ns
                chord(['Control_L'],'o');chord(['Control_L','Shift_L'],'s')
                assert state()['file_dialog'] is None and active.stat().st_mtime_ns==before_mtime
                control('Command(Play)');wait(lambda:state()['play_feet'] is None)
                control('Mode(Guides)')
                click(next(b for b in state()['buttons'] if b['action']=='NewPath(Road)' and all(x>0 for x in b['size'])))
                wait(lambda:state()['path_draft_points']==0)
                chord(['Control_L'],'o')
                assert state()['file_dialog'] is None and state()['path_draft_points']==0 and state()['revision']==7
                desktop.key('Escape');wait(lambda:state()['path_draft_points'] is None)
                capture('final');final=state()
                subprocess.run(['wmctrl','-ic',hex(window)],check=True);process.wait(timeout=20)
                assert process.returncode==0
            finally:
                if desktop:desktop.close()
                if process.poll() is None:process.terminate();process.wait(timeout=15)
        logs=logfile.read_text();assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
        cases.append(dict(name=name,viewport=final['viewport'],gpu=final['gpu'],validation_layer_active=True,
            gpu_diagnostics=check_gpu_diagnostics(logs,allow_surface_resize=False)))
        (folder/'qualification.json').write_text(json.dumps(dict(case=cases[-1],editor_sha256=binary_hash),indent=2)+'\n')
    result=dict(passed=True,cases=cases,input_notes=input_notes,editor_sha256=binary_hash,checks=[
        'actual window title follows active path, zone name and dirty state through Save As, Open, undo/redo and failures',
        'standard browser paging, folder navigation, filename retention and extension validation',
        'new Save As, explicit overwrite confirmation/cancel/changed-name reset, keyboard shortcuts and exact writes without history',
        'invalid open refusal/correction, modal isolation, cross-origin validated load/undo/redo and reconciled fresh placement IDs',
        'Save As preserves redo; failed writes preserve active path/source/history; play/draft refusal and no shortcut fallthrough'])
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--case',choices=['local','far'])
    args=parser.parse_args();run(args.output,args.binary,args.case)
