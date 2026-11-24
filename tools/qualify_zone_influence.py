#!/usr/bin/env python3
"""Qualify terrain-conforming sample influence with owned 3440x1440 native input."""
import argparse
import json
import math
import os
from pathlib import Path
import struct
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def run(output, binary):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = binary.resolve()
    cases = []
    env = dict(os.environ, WGPU_BACKEND='vulkan',
               VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation', VK_LOADER_DEBUG='layer')
    for name, axis, origin, direction in [('hill',33,[0,0,0],1),
            ('far-valley',65,[128000000000,20,-128000000000],-1)]:
        spacing = 128/(axis-1)
        heights = [0 if x in (0,axis-1) or z in (0,axis-1) else
                   direction*(4+max(0,12-.5*(abs(x*spacing-64)+abs(z*spacing-64))))
                   for z in range(axis) for x in range(axis)]
        objects = [dict(id=i,name=f'Pointer {i}',primitive='box',
                       pose=dict(position=[origin[0]+x,origin[1]+h+.05,origin[2]+z],
                                 yaw_degrees=0,size=[.1,.1,.1]))
                   for i,x,z,h in [(1,64,64,direction*16),(2,72,64,direction*12),
                                    (3,4,64,direction*4),(4,129,64,0)]]
        patch = dict(chunk=[origin[0]//128,origin[2]//128],heights=heights)
        if axis == 65:
            patch['resolution'] = '2m'
        source = output/f'{name}.source.json'
        source.write_text(json.dumps(dict(schema=7,name='Influence qualification',origin=origin,
            spawn=[origin[0]+16,origin[1]+direction*4+.1,origin[2]+16],objects=objects,terrain=patch)))
        report = output/f'{name}.state.json'
        logfile = output/f'{name}.log'
        with logfile.open('w') as log:
            process = subprocess.Popen([str(binary),'--zone',str(source),'--report',str(report)],
                                       env=env,stdout=log,stderr=log)
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
                initial = state()['document']
                key('F2')
                wait(lambda: state()['terrain_mode'])
                key('f')
                wait(lambda: state()['view']['distance']>100)
                mesh_id = None
                mesh_count = None
                samples_checked = 0

                def check():
                    nonlocal mesh_id, mesh_count, samples_checked
                    s = state()
                    preview = s['brush_preview']
                    assert preview['visible']
                    assert preview['vertices']==axis*axis and preview['triangles']==2*(axis-1)**2
                    if mesh_id is None:
                        mesh_id,mesh_count = preview['mesh'],s['mesh_assets']
                    assert preview['mesh']==mesh_id and s['mesh_assets']==mesh_count
                    centre = [origin[0]+s['brush_point'][0],origin[2]+s['brush_point'][2]]
                    expected = {}
                    for z in range(1,axis-1):
                        for x in range(1,axis-1):
                            weight = f32(max(0,1-math.hypot((origin[0]-centre[0])+x*spacing,
                                                         (origin[2]-centre[1])+z*spacing)/s['brush_radius']))
                            if weight>0:
                                expected[z*axis+x] = f32(.5*weight)
                    actual = {p['index']:p for p in preview['samples']}
                    assert actual.keys()==expected.keys(),(name,centre,actual.keys(),expected.keys())
                    current = s['terrain_preview_heights'] or s['document']['terrain']['heights']
                    for i,alpha in expected.items():
                        position = actual[i]['position']
                        assert position[0]==(i%axis)*spacing and position[2]==(i//axis)*spacing
                        assert abs(position[1]-f32(f32(current[i])+f32(.04)))<2e-5,(position,current[i])
                        assert abs(actual[i]['alpha']-alpha)<1e-6
                    samples_checked += len(expected)
                    return s

                def point(identifier):
                    p = next(o['screen'] for o in state()['objects'] if o['id']==identifier)
                    desktop.move(window,p,1)
                    wait(lambda: state()['brush_preview']['visible'] and state()['brush_point'])
                    return check()

                def capture(label):
                    key('F12')
                    wait(lambda: capture_ready(report.with_suffix('.png')),30)
                    report.with_suffix('.png').rename(output/f'{name}.{label}.png')

                peak = point(1)
                assert max(p['position'][1] for p in peak['brush_preview']['samples'])-min(
                    p['position'][1] for p in peak['brush_preview']['samples'])>2
                capture('peak')
                point(2)
                capture('slope')
                point(3)
                capture('edge')
                outside = next(o['screen'] for o in state()['objects'] if o['id']==4)
                desktop.move(window,outside,1)
                wait(lambda: state()['brush_point'] and state()['brush_point'][0]>128
                     and not state()['brush_preview']['visible'])
                # Radius settings update the resident overlay without document edits.
                for action,radius in [('Radius(-1)',8),('Radius(1)',12)]:
                    entry = wait(lambda: next((b for b in state()['buttons'] if b['action']==action),None))
                    desktop.move(window,entry['centre'],1)
                    wait(lambda: any(b['action']==action and b['hovered'] for b in state()['buttons']))
                    desktop.button(1,True)
                    try:
                        wait(lambda: state()['left_mouse_down'])
                    finally:
                        desktop.button(1,False)
                    wait(lambda: state()['brush_radius']==radius)
                    point(2)
                # UI exclusion and mode/modal changes hide without source edits.
                desktop.move(window,[150,300],1)
                wait(lambda: not state()['brush_preview']['visible'])
                point(1)
                key('F8')
                wait(lambda: state()['name_edit'] is not None and not state()['brush_preview']['visible'])
                key('Escape')
                wait(lambda: state()['name_edit'] is None)
                key('F1')
                wait(lambda: not state()['brush_preview']['visible'])
                key('F2')
                point(1)
                assert state()['revision']==0 and state()['document']==initial and not state()['unsaved_changes']
                # Hold one small stamp: resident overlay must follow the draft,
                # while document/history remain unchanged until release.
                for _ in range(4):
                    key('bracketleft')
                wait(lambda: state()['brush_strength']==.125)
                desktop.button(1,True)
                try:
                    wait(lambda: state()['terrain_preview_heights'] is not None)
                    draft = check()
                    assert draft['revision']==0 and draft['history']==[0,0]
                    assert draft['document']==initial
                    capture('stroke')
                finally:
                    desktop.button(1,False)
                wait(lambda: state()['revision']==1 and state()['history']==[1,0])
                check()
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
        cases.append(dict(name=name,gpu=final['gpu'],viewport=final['viewport'],samples_checked=samples_checked,
                          validation_layer_active=True,gpu_diagnostics=diagnostics,mesh=mesh_id))
    result = dict(cases=cases,checks=['exact terrain triangles and shared per-sample falloff on hills, slopes and locked edges',
        'one resident mesh across pointer/radius changes, outside-patch and UI/mode/modal hiding without history',
        'temporary sculpt preview conforms before one-entry commit'],
        scope='Linux native 3440x1440 only; potential grid influence, no Windows or multiplayer claim')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


if __name__=='__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    args = parser.parse_args()
    run(args.output,args.binary)
