#!/usr/bin/env python3
"""Observe the bounded stroke counter, cap, commit and cancellation with native input."""
import argparse
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
    source = output/'source.json'
    zone = dict(schema=2,name='Stroke budget qualification',origin=[0,0,0],spawn=[16,.1,16],
                terrain=dict(chunk=[0,0],heights=[0]*1089),
                objects=[dict(id=i,name=f'Pointer {i}',primitive='box',
                    pose=dict(position=[x,.05,64],yaw_degrees=0,size=[.1,.1,.1]))
                    for i,x in [(1,32),(2,96)]])
    source.write_text(json.dumps(zone))
    report = output/'state.json'
    logfile = output/'editor.log'
    env = dict(os.environ,WGPU_BACKEND='vulkan',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',
               VK_LOADER_DEBUG='layer')
    with logfile.open('w') as log:
        process = subprocess.Popen([str(binary),'--zone',str(source),'--report',str(report)],
                                   stdout=log,stderr=log,env=env)
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
            initial = state()['document']
            key = desktop.key
            key('F2')
            wait(lambda: state()['terrain_mode'])
            key('f')
            wait(lambda: state()['view']['distance']>100)
            for _ in range(7):
                key('bracketleft')
            wait(lambda: abs(state()['brush_strength']-.05)<1e-7)
            positions = [o['screen'] for o in state()['objects']]
            desktop.move(window,positions[0],1)
            wait(lambda: state()['brush_preview']['visible'])
            desktop.button(1,True)
            try:
                wait(lambda: state()['stroke_progress'] and state()['stroke_progress']['stamps']==1)
                limit = state()['stroke_progress']['limit']
                assert limit==128
                for count in range(2,limit+1):
                    desktop.move(window,positions[(count-1)%2],1)
                    wait(lambda: state()['stroke_progress']['stamps']==count)
                    current = state()
                    assert current['revision']==0 and current['history']==[0,0]
                    assert current['document']==initial
                    assert f'({count}/{limit})' in current['status']
                capped = state()
                assert 'Stroke limit reached' in capped['status']
                assert capped['unsaved_changes']
                for index in [0,1,0]:
                    desktop.move(window,positions[index],1)
                    elapsed = state()['input_elapsed']
                    wait(lambda: state()['input_elapsed']>elapsed)
                    assert state()['stroke_progress']['stamps']==128
                    assert state()['terrain_preview_heights']==capped['terrain_preview_heights']
                    assert state()['revision']==0
                key('F12')
                wait(lambda: capture_ready(report.with_suffix('.png')),30)
                report.with_suffix('.png').rename(output/'capped.png')
            finally:
                desktop.button(1,False)
            committed = wait(lambda: state() if state()['revision']==1 else None)
            assert committed['history']==[1,0] and committed['stroke_progress'] is None
            assert committed['document']['terrain']['heights']==capped['terrain_preview_heights']

            def button(action, revision):
                entry = next(b for b in state()['buttons'] if b['action']==action)
                desktop.move(window,entry['centre'],1)
                wait(lambda: any(b['action']==action and b['hovered'] for b in state()['buttons']))
                desktop.button(1,True)
                try:
                    wait(lambda: state()['left_mouse_down'])
                finally:
                    desktop.button(1,False)
                wait(lambda: state()['revision']==revision)

            button('Undo',2)
            assert state()['document']==initial and not state()['unsaved_changes']
            button('Redo',3)
            assert state()['document']==committed['document']
            # Cancel a fresh preview without altering the existing undo entry.
            desktop.move(window,positions[1],1)
            desktop.button(1,True)
            try:
                wait(lambda: state()['stroke_progress'] and state()['stroke_progress']['stamps']==1)
                key('Escape')
                wait(lambda: state()['stroke_progress'] is None)
            finally:
                desktop.button(1,False)
            final = state()
            assert final['revision']==3 and final['history']==[1,0]
            assert final['document']==committed['document']
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
    result = dict(viewport=final['viewport'],gpu=final['gpu'],validation_layer_active=True,
                  gpu_diagnostics=diagnostics,stamps=128,checks=[
                      'live progress without committed source/history changes',
                      'explicit cap with frozen preview despite further input',
                      'one-entry commit matches preview, undo/redo and fresh-stroke cancellation'],
                  scope='Linux native 3440x1440 only; no command, schema, multiplayer or Windows change')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output,args.binary)
