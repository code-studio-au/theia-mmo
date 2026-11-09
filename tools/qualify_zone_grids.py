#!/usr/bin/env python3
"""Owned 3440x1440 input for adaptive reference grids without a terrain patch."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def run(output, binary):
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
                def control(target):
                    item=wait(lambda:next((b for b in state()['workspace_controls'] if b['target']==target and all(v>0 for v in b['size'])),None))
                    desktop.move(window,item['centre'],1)
                    wait(lambda:any(b['target']==target and b['hovered'] for b in state()['workspace_controls']))
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
                control('Command(Play)');wait(lambda:state()['play_feet'] is not None)
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
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    args=parser.parse_args();run(args.output,args.binary)
