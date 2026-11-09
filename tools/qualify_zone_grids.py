#!/usr/bin/env python3
"""Owned 3440x1440 input for adaptive reference grids without a terrain patch."""
import argparse
import copy
import math
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary, pan=False):
    output=output.resolve();output.mkdir(parents=True,exist_ok=False)
    binary=binary.resolve();cases=[]
    env=dict(os.environ,WGPU_BACKEND='vulkan',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',VK_LOADER_DEBUG='layer')
    for name,origin in [('local',[0,0,0]),('far',[128000000000,20,-128000000000])]:
        folder=output/name;folder.mkdir()
        source=folder/'source.json';report=folder/'state.json';logfile=folder/'editor.log'
        source.write_text(json.dumps(dict(schema=1,name='Reference grid',origin=origin,
            spawn=[origin[0],origin[1]+.1,origin[2]+6],objects=[dict(id=1,name='Grid box',primitive='box',
                pose=dict(position=[origin[0]+8.125,origin[1]+1.25,origin[2]-8.5],yaw_degrees=17.25,size=[2,2,2]))])))
        with logfile.open('w') as log:
            process=subprocess.Popen([str(binary),'--zone',str(source),'--save-path',str(folder/'unused.json'),'--report',str(report)],env=env,stdout=log,stderr=log)
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
                def control(target,group='workspace_controls'):
                    def entries():return state()['content_list']['controls'] if group=='content' else state()[group]
                    item=wait(lambda:next((b for b in entries() if b['target']==target and all(v>0 for v in b['size'])),None))
                    desktop.move(window,item['centre'],1)
                    wait(lambda:any(b['target']==target and b['hovered'] for b in entries()))
                    desktop.owned_focus(pointer=True)
                    desktop.button(1,True)
                    try:wait(lambda:state()['left_mouse_down'])
                    finally:desktop.button(1,False)
                    wait(lambda:not state()['left_mouse_down']);fresh()
                def capture(label):
                    screenshot=report.with_suffix('.png')
                    if screenshot.exists():screenshot.unlink()
                    desktop.key('F12');wait(lambda:capture_ready(screenshot),30)
                    screenshot.rename(folder/f'{label}.png')
                def middle_drag(start,end,blocked=False):
                    desktop.move(window,start,1);before=copy.deepcopy(fresh())
                    desktop.button(2,True)
                    try:
                        wait(lambda:state()['middle_mouse_down'])
                        desktop.move(window,end,1);after=copy.deepcopy(fresh())
                    finally:desktop.button(2,False)
                    wait(lambda:not state()['middle_mouse_down']);fresh()
                    if blocked:
                        assert state()['view']==before['view']
                    else:
                        a=before['view'];b=after['view']
                        dx=after['cursor'][0]-before['cursor'][0];dy=after['cursor'][1]-before['cursor'][1]
                        if a['preset']=='Perspective':
                            yaw=a['yaw'];pitch=a['pitch']
                            right=[math.cos(yaw),0,-math.sin(yaw)]
                            up=[-math.sin(yaw)*math.sin(pitch),math.cos(pitch),-math.cos(yaw)*math.sin(pitch)]
                            span=2*a['distance']*math.tan(before['camera_projection']['fov']/2)
                        else:
                            right,up={'Top':([1,0,0],[0,0,-1]),'Front':([1,0,0],[0,1,0]),'Right':([0,0,-1],[0,1,0])}[a['preset']]
                            span=a['distance']
                        expected=[a['target'][i]+(-right[i]*dx+up[i]*dy)*span/1440 for i in range(3)]
                        assert max(abs(b['target'][i]-expected[i]) for i in range(3))<.0002,(a,b,expected)
                        assert a['preset']==b['preset'] and a['distance']==b['distance']
                        if a['preset']!='Perspective':
                            first=before['objects'][0]['screen'];second=after['objects'][0]['screen']
                            assert max(abs(second[i]-first[i]-[dx,dy][i]) for i in range(2))<.05
                        assert after['view']['target']!=before['view']['target']
                    assert state()['document']==before['document'] and state()['revision']==before['revision'] and state()['history']==before['history']
                initial=fresh()
                for preset,normal in [('Perspective',1),('Top',1),('Front',2),('Right',0)]:
                    control(f'View({preset})');wait(lambda:state()['view']['preset']==preset)
                    current=fresh();grid=current['grid'];step=grid['spacing']
                    assert grid['enabled'] and not grid['surface'] and not grid['surface_visible']
                    assert len(grid['lines'])==130 and all(line['visible'] for line in grid['lines'])
                    for line in grid['lines']:
                        assert abs(line['position'][normal]-.004)<1e-6
                        for axis in [i for i in range(3) if i!=normal]:
                            coordinate=line['position'][axis]/step
                            assert abs(coordinate-round(coordinate))<1e-4
                    if pan:
                        middle_drag([1720,720],[1820,760])
                        desktop.move(window,[1720,720],1);height=state()['view']['distance']
                        desktop.button(4,True);desktop.button(4,False)
                        wait(lambda:state()['view']['distance']<height)
                        middle_drag([1720,720],[1820,760])
                        # A press originating in either panel cannot acquire
                        # a pan by later crossing into the viewport.
                        middle_drag([150,400],[1720,720],blocked=True)
                        middle_drag([3290,400],[1720,720],blocked=True)
                        # Leaving the viewport cancels the gesture until a new press.
                        desktop.move(window,[1720,720],1);fresh();desktop.button(2,True)
                        try:
                            wait(lambda:state()['middle_mouse_down'])
                            desktop.move(window,[150,400],1);before=copy.deepcopy(fresh()['view'])
                            desktop.move(window,[1800,760],1);fresh();assert state()['view']==before
                        finally:desktop.button(2,False)
                        wait(lambda:not state()['middle_mouse_down'])
                    capture(preset.lower())
                    control('Grid');wait(lambda:not state()['grid']['enabled'])
                    assert not any(line['visible'] for line in state()['grid']['lines'])
                    control('Grid');wait(lambda:state()['grid']['enabled'])
                    assert state()['document']==initial['document'] and state()['history']==initial['history'] and state()['revision']==0
                # Pan follows the view target while every reference remains on a
                # zone-origin multiple of its displayed step, independent of f64 origin.
                target=state()['view']['target'];desktop.key_state('d',True)
                try:wait(lambda:state()['view']['target'][2]<target[2]-1)
                finally:desktop.key_state('d',False)
                fresh();assert state()['revision']==0
                if pan:
                    control('Object(1)','content');wait(lambda:state()['selected']==1)
                    control('Frame');control('View(Front)');fresh()
                    arrow=next(h for h in state()['move_handles'] if h['visible'] and h['axis']==1)
                    desktop.move(window,arrow['screen'],1);fresh();desktop.button(1,True)
                    try:
                        wait(lambda:state()['dragging'])
                        before=copy.deepcopy(state()['view'])
                        desktop.button(2,True)
                        try:
                            wait(lambda:state()['middle_mouse_down'])
                            desktop.move(window,arrow['target'],1);fresh()
                            assert state()['view']==before and state()['document']==initial['document']
                            desktop.key('Escape');wait(lambda:not state()['dragging'])
                        finally:desktop.button(2,False)
                    finally:desktop.button(1,False)
                    wait(lambda:not state()['left_mouse_down'] and not state()['middle_mouse_down'])
                    assert state()['revision']==0
                    control('SpawnTransform');wait(lambda:state()['transform_edit'] is not None)
                    middle_drag([1720,720],[1820,760],blocked=True)
                    desktop.key('Escape');wait(lambda:state()['transform_edit'] is None)
                control('Command(Play)');wait(lambda:state()['play_feet'] is not None)
                if pan:middle_drag([1720,720],[1820,760],blocked=True)
                assert not any(line['visible'] for line in state()['grid']['lines'])
                control('Command(Play)');wait(lambda:state()['play_feet'] is None)
                assert all(line['visible'] for line in state()['grid']['lines'])
                assert state()['document']==initial['document'] and state()['revision']==0
                final=state();subprocess.run(['wmctrl','-ic',hex(window)],check=True);process.wait(timeout=20)
                assert process.returncode==0
            finally:
                if desktop:desktop.close()
                if process.poll() is None:process.terminate();process.wait(timeout=15)
        logs=logfile.read_text();assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
        diagnostics=check_gpu_diagnostics(logs,allow_surface_resize=False)
        cases.append(dict(name=name,gpu=final['gpu'],viewport=final['viewport'],validation_layer_active=True,gpu_diagnostics=diagnostics))
    result=dict(cases=cases,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),checks=[
        '130 actual bounded nonterrain grid meshes on all view planes at local and far origins',
        'zone-origin-aligned spacing, mouse toggle, pan and playtest hide/restore preserve document/history'],scope='Linux native 3440x1440 only; no Windows or multiplayer claim')
    if pan: result['checks'].append('MMB projected drag at two zooms in all views, exact ortho screen tracking, source/history preservation, panel-origin and exit cancellation, held-transform/modal/play isolation')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--pan',action='store_true')
    args=parser.parse_args();run(args.output,args.binary,args.pan)
