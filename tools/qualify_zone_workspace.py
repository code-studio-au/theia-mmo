#!/usr/bin/env python3
"""Exercise the application shell and standard sliders with owned ultrawide input."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from qualify_zone_flatten import normalized


def run(output,binary,scatter=False):
    output=output.resolve()
    output.mkdir(parents=True,exist_ok=False)
    binary=binary.resolve()
    cases=[]
    env=dict(os.environ,WGPU_BACKEND='vulkan',VK_INSTANCE_LAYERS='VK_LAYER_KHRONOS_validation',VK_LOADER_DEBUG='layer')
    for name,axis,origin in [('coarse',33,[0,0,0]),('fine-far',65,[128000000000,20,-128000000000])]:
        source=output/f'{name}.source.json'
        report=output/f'{name}.state.json'
        saved=output/f'{name}.saved.json'
        logfile=output/f'{name}.log'
        terrain=dict(chunk=[origin[0]//128,origin[2]//128],heights=[0]*(axis*axis))
        if axis==65: terrain['resolution']='2m'
        source.write_text(json.dumps(dict(schema=7,name='Application workspace',origin=origin,
            spawn=[origin[0]+16,origin[1]+.1,origin[2]+16],terrain=terrain,
            objects=[dict(id=1,name='Inspector box',primitive='box',pose=dict(
                position=[origin[0]+24,origin[1]+1,origin[2]+24],yaw_degrees=0,size=[2,2,2]))])))
        if axis==33:
            fixture=json.loads(source.read_text())
            fixture['objects'] += [dict(id=i,name=f'Browser object {i}',primitive='box',pose=dict(
                position=[origin[0]+i*3,origin[1]+1,origin[2]-24],yaw_degrees=0,size=[2,2,2]))
                for i in range(2,32)]
            source.write_text(json.dumps(fixture))
        with logfile.open('w') as log:
            process=subprocess.Popen([str(binary),'--zone',str(source),'--save-path',str(saved),'--report',str(report)],
                                     env=env,stdout=log,stderr=log)
            desktop=None
            try:
                window=wait(lambda:owned_window(process.pid),45)
                desktop=Desktop()
                desktop.focus(window)
                def state():
                    assert process.poll() is None,'Editor exited'
                    return json.loads(report.read_text())
                wait(lambda:report.exists() and state()['viewport']==[3440,1440],45)
                desktop.move(window,[1737,731],1)
                desktop.move(window,[1720,720],1)
                initial=state()
                def fresh():
                    elapsed=state()['input_elapsed']
                    wait(lambda:state()['input_elapsed']>elapsed)
                    return state()
                def entry(target):
                    return wait(lambda:next((b for b in state()['workspace_controls'] if b['target']==target
                        and b['size'][0]>0 and b['size'][1]>0),None))
                def click(item):
                    desktop.move(window,item['centre'],1)
                    fresh()
                    # A new fullscreen window may report layout before the WM
                    # presents its first frame. Wait read-only for ownership;
                    # never retry a mutating click or steal focus back.
                    def pointer_owned():
                        try:
                            desktop.owned_focus(pointer=True)
                            return True
                        except AssertionError:
                            return False
                    wait(pointer_owned,15)
                    desktop.button(1,True)
                    try:wait(lambda:state()['left_mouse_down'])
                    finally:desktop.button(1,False)
                    wait(lambda:not state()['left_mouse_down'])
                    fresh()
                def control(target):click(entry(target))
                def tool(action):
                    click(wait(lambda:next((b for b in state()['buttons'] if b['action']==action and b['size'][0]>0),None)))
                def slider(target,fraction,drag=False):
                    item=entry(target)
                    x,y=item['centre'];width=item['size'][0]
                    point=[x-width/2+7+(width-14)*fraction,y]
                    if drag:
                        value=item['value'];low,high=item['range']
                        thumb=[x-width/2+7+(width-14)*(value-low)/(high-low),y]
                        desktop.move(window,thumb,1)
                        desktop.button(1,True)
                        try:
                            wait(lambda:state()['left_mouse_down'])
                            desktop.move(window,point,1)
                            fresh()
                        finally:desktop.button(1,False)
                        wait(lambda:not state()['left_mouse_down'])
                    else:click(dict(centre=point))
                    return fresh()
                def capture(label):
                    desktop.key('F12')
                    wait(lambda:capture_ready(report.with_suffix('.png')),30)
                    report.with_suffix('.png').rename(output/f'{name}.{label}.png')
                # Header clicks while placement is armed cannot place world objects.
                tool('Place(Box)')
                control('Snap')
                wait(lambda:not state()['snap'])
                control('Snap')
                wait(lambda:state()['snap'])
                assert state()['document']==initial['document'] and state()['revision']==0
                # Keep a guide draft when other workspaces or Play are requested.
                control('Mode(Guides)')
                wait(lambda:state()['path_mode'])
                tool('NewPath(Road)')
                for point in state()['guide_points'][:2]:
                    click(dict(centre=point))
                wait(lambda:state()['path_draft_points']==2)
                control('Mode(Paint)')
                control('Command(Play)')
                assert state()['path_mode'] and state()['path_draft_points']==2 and state()['play_feet'] is None
                assert state()['document']==initial['document'] and state()['revision']==0
                desktop.key('Escape')
                wait(lambda:state()['path_draft_points'] is None)
                # Workspaces and settings are reachable without function keys.
                for target,field in [('Mode(Plants)','vegetation_mode'),('Mode(Markers)','marker_mode'),
                                      ('Mode(Guides)','path_mode'),('Mode(Paint)','terrain_paint'),('Mode(Sculpt)','terrain_mode')]:
                    control(target)
                    wait(lambda:state()[field])
                    assert state()['document']==initial['document'] and state()['revision']==0
                if axis==33:
                    assert state()['content_list']['pages']==2
                    for target,page in [('Next',1),('Previous',0)]:
                        click(next(b for b in state()['content_list']['controls'] if b['target']==target))
                        wait(lambda:state()['content_list']['page']==page)
                    assert all(92<c['centre'][1] and c['centre'][1]+c['size'][1]/2<=840
                        for c in state()['content_list']['controls'] if c['target'].startswith('Object('))
                control('Snap')
                wait(lambda:not state()['snap'])
                control('Snap')
                wait(lambda:state()['snap'])
                control('Frame')
                wait(lambda:state()['view']['distance']>100)
                slider('Radius',.25)
                assert abs(state()['brush_radius']-11)<.15,state()['brush_radius']
                slider('Strength',.2,True)
                assert abs(state()['brush_strength']-1.64)<.1,state()['brush_strength']
                tool('Brush(Flatten)')
                wait(lambda:state()['brush_tool']=='Flatten')
                slider('Blend',.25,True)
                assert abs(state()['sculpt_blend']-.2875)<.02
                slider('Level',.52,True)
                assert abs(state()['flatten_height']-2.56)<.2,state()['flatten_height']
                sculpt_blend=state()['sculpt_blend']
                control('Mode(Paint)')
                wait(lambda:state()['terrain_paint'])
                slider('Blend',.5)
                assert abs(state()['paint_blend']-.525)<.02
                assert state()['sculpt_blend']==sculpt_blend
                assert state()['document']==initial['document'] and state()['history']==initial['history']
                capture('paint-properties')
                # Real slider presets feed the same validated stamp/undo path.
                before=state()
                point=before['terrain_points'][0]
                desktop.move(window,point,1)
                hit=fresh()['brush_point']
                desktop.button(1,True)
                try:wait(lambda:state()['stroke_progress'] and state()['stroke_progress']['stamps']==1)
                finally:desktop.button(1,False)
                edited=wait(lambda:state() if state()['revision']==1 else None)
                assert edited['history']==[1,0]
                centre=[origin[0]+hit[0],origin[2]+hit[2]]
                prior=output/f'{name}.before.json';prior.write_text(json.dumps(before['document']))
                commands=output/f'{name}.commands.jsonl'
                commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=[dict(op='paint_terrain',paint=dict(
                    centre=centre,radius_metres=before['brush_radius'],surface='dirt',blend=before['paint_blend']))]))+'\n')
                expected=output/f'{name}.expected.json'
                subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),'--output',str(expected)],check=True,capture_output=True)
                assert normalized(json.loads(expected.read_text()))==normalized(edited['document'])
                control('Command(Undo)');wait(lambda:state()['revision']==2)
                assert state()['document']==initial['document']
                control('Command(Redo)');wait(lambda:state()['revision']==3)
                assert state()['document']==edited['document']
                control('Command(Save)')
                wait(lambda:saved.exists() and not state()['unsaved_changes'])
                # Persistent inspector actions open the retained standard fields.
                control('Mode(Objects)')
                wait(lambda:not state()['terrain_mode'])
                item=next(b for b in state()['content_list']['controls'] if b['target']=='Object(1)')
                click(item);wait(lambda:state()['selected']==1)
                control('Transform');wait(lambda:state()['transform_edit'] is not None)
                fields=[b for b in state()['transform_controls'] if b['target'].startswith('Field(')]
                assert all(b['centre'][0]>3140 and b['centre'][1]>840 for b in fields)
                field=next(b for b in fields if b['target']=='Field(1)')
                click(field)
                wait(lambda:state()['transform_edit']['active']==1 and state()['transform_edit']['selection']==[0,len(state()['transform_edit']['fields'][1])])
                for key in ['3','period','5']:desktop.key(key)
                wait(lambda:state()['transform_edit']['fields'][1]=='3.5')
                expected_pose=dict(edited['document']['objects'][0]['pose'])
                expected_pose['position']=list(expected_pose['position'])
                expected_pose['position'][1]=3.5
                # The far case uses a global Y origin of 20, so this is an
                # intentionally airborne/below-ground authoring coordinate.
                click(next(b for b in state()['transform_controls'] if b['target']=='Apply'))
                wait(lambda:state()['revision']==4 and state()['transform_edit'] is None)
                assert state()['document']['objects'][0]['pose']==expected_pose
                assert state()['history']==[2,0]
                transform_commands=output/f'{name}.transform.commands.jsonl'
                transform_commands.write_text(json.dumps(dict(command='apply',expected_revision=0,
                    edits=[dict(op='transform',id=1,pose=expected_pose)]))+'\n')
                transform_prior=output/f'{name}.transform.before.json';transform_prior.write_text(json.dumps(edited['document']))
                transform_expected=output/f'{name}.transform.expected.json'
                subprocess.run([str(binary),'--headless','--zone',str(transform_prior),'--commands',str(transform_commands),
                                '--output',str(transform_expected)],check=True,capture_output=True)
                assert normalized(json.loads(transform_expected.read_text()))==normalized(state()['document'])
                control('Command(Undo)');wait(lambda:state()['revision']==5)
                assert state()['document']==edited['document']
                control('Command(Redo)');wait(lambda:state()['revision']==6)
                assert state()['document']['objects'][0]['pose']==expected_pose
                control('Command(Undo)');wait(lambda:state()['revision']==7)
                assert state()['document']==edited['document']
                control('Rename');wait(lambda:state()['name_edit']=='Inspector box')
                desktop.key('Escape');wait(lambda:state()['name_edit'] is None)
                control('Frame')
                wait(lambda:state()['view']['distance']<40)
                capture('object-properties')
                control('Command(Play)');wait(lambda:state()['play_feet'] is not None)
                control('Mode(Paint)');assert not state()['terrain_mode']
                control('Command(Play)');wait(lambda:state()['play_feet'] is None)
                assert state()['revision']==7 and state()['history']==[1,1]
                if scatter:
                    # Editor-local scatter sliders feed one shared seeded command.
                    before=state()
                    control('Mode(Plants)')
                    wait(lambda:state()['vegetation_mode'])
                    kind='Tree' if axis==33 else 'Shrub'
                    tool(f'Plant({kind}, true)')
                    slider('ScatterRadius',.3)
                    assert abs(state()['scatter_radius']-10.3)<.2
                    slider('ScatterCount',1.,True)
                    assert state()['scatter_count']==64
                    slider('ScatterCount',0.,True)
                    assert state()['scatter_count']==1
                    slider('ScatterCount',.25,True)
                    assert state()['scatter_count']==17
                    presets=state()
                    assert presets['document']==before['document'] and presets['history']==before['history']
                    assert presets['brush_radius']==before['brush_radius']
                    assert presets['sculpt_blend']==before['sculpt_blend'] and presets['paint_blend']==before['paint_blend']
                    # Single placement hides the scatter-specific settings.
                    tool(f'Plant({kind}, false)')
                    fresh()
                    assert all(b['size'][0]==0 or b['size'][1]==0 for b in state()['workspace_controls']
                        if b['target'] in ('ScatterRadius','ScatterCount'))
                    tool(f'Plant({kind}, true)')
                    capture('scatter-properties')
                    # Frame terrain for a safe central surface hit.
                    control('Mode(Sculpt)');control('Frame')
                    wait(lambda:state()['view']['distance']>100)
                    control('Mode(Plants)');tool(f'Plant({kind}, true)')
                    point=state()['terrain_points'][0]
                    desktop.move(window,point,1)
                    hit=wait(lambda:state()['plant_point'])
                    click(dict(centre=point))
                    placed=wait(lambda:state() if state()['revision']==8 else None)
                    assert len(placed['document']['plants'])==17 and placed['history']==[2,0]
                    # The cancelled guide reserved one ID; all fresh plant IDs
                    # must follow the original objects and that reserved draft.
                    first_id=max(o['id'] for o in initial['document']['objects'])+2
                    assert [p['id'] for p in placed['document']['plants']]==list(range(first_id,first_id+17))
                    prior=output/f'{name}.scatter.before.json';prior.write_text(json.dumps(before['document']))
                    commands=output/f'{name}.scatter.commands.jsonl'
                    commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=[dict(
                        op='scatter_vegetation',scatter=dict(first_id=first_id,seed=first_id,kind=kind.lower(),
                            centre=[hit[0],hit[2]],radius_metres=presets['scatter_radius'],count=17))]))+'\n')
                    expected=output/f'{name}.scatter.expected.json'
                    subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),
                        '--output',str(expected)],check=True,capture_output=True)
                    assert normalized(json.loads(expected.read_text()))==normalized(placed['document'])
                    control('Command(Undo)');wait(lambda:state()['revision']==9)
                    assert state()['document']==before['document']
                    control('Command(Redo)');wait(lambda:state()['revision']==10)
                    assert state()['document']==placed['document']
                    control('Command(Save)');wait(lambda:not state()['unsaved_changes'])
                    assert normalized(json.loads(saved.read_text()))==normalized(placed['document'])
                    capture('scatter-result')
                final=state()
                subprocess.run(['wmctrl','-ic',hex(window)],check=True)
                process.wait(timeout=20)
                assert process.returncode==0
            finally:
                if desktop:desktop.close()
                if process.poll() is None:process.terminate();process.wait(timeout=15)
        logs=logfile.read_text()
        assert 'Insert instance layer "VK_LAYER_KHRONOS_validation"' in logs
        diagnostics=check_gpu_diagnostics(logs,allow_surface_resize=False)
        cases.append(dict(name=name,gpu=final['gpu'],viewport=final['viewport'],validation_layer_active=True,gpu_diagnostics=diagnostics))
    result=dict(cases=cases,checks=['mouse workspace buttons, snap checkbox, contextual standard slider click and drag',
        'settings preserve history; actual paint stamp equals headless command, one-entry undo/redo/save',
        'outliner selection, docked numeric transform matches headless command with one-entry undo/redo, name action, framing and playtest isolation'],scope='Linux native 3440x1440 only; first application-shell slice')
    if scatter: result['checks'].append('vegetation slider boundaries, integer count, tool visibility, history isolation and seeded scatter/headless equality, one-entry undo/redo/save')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--scatter',action='store_true')
    args=parser.parse_args();run(args.output,args.binary,args.scatter)
