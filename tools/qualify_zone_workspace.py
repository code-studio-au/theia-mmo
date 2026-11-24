#!/usr/bin/env python3
"""Exercise the application shell and standard sliders with owned ultrawide input."""
import argparse
import copy
import json
import os
from pathlib import Path
import subprocess

from gpu_validation import check_gpu_diagnostics
from qualify_world_input_os import Desktop, owned_window, wait, capture_ready
from qualify_zone_flatten import normalized


def run(output,binary,scatter=False,guides=False,views=False):
    assert sum((scatter,guides,views))<=1, "Qualify workspace extensions in separate fixtures"
    output=output.resolve()
    output.mkdir(parents=True,exist_ok=False)
    binary=binary.resolve()
    cases=[]
    input_notes=[]
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
        if guides:
            fixture=json.loads(source.read_text())
            fixture['paths']=[dict(id=1000,name='Property route',kind='road' if axis==33 else 'stream',
                width_metres=4,points=[[origin[0]+48,origin[1]+.03,origin[2]+88],
                    [origin[0]+64,origin[1]+.03,origin[2]+88],[origin[0]+72,origin[1]+.03,origin[2]+84]])]
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
                if guides:
                    control('Mode(Guides)')
                    if axis==33:
                        click(next(b for b in state()['content_list']['controls'] if b['target']=='Next'))
                        wait(lambda:state()['content_list']['page']==1)
                    click(next(b for b in state()['content_list']['controls'] if b['target']=='Guide(1000)'))
                    wait(lambda:state()['path_selected']==1000 and state()['path_point'] is None)
                    def form_control(target):
                        click(wait(lambda:next((b for b in state()['transform_controls'] if b['target']==target and all(x>0 for x in b['size'])),None)))
                    def field(index,value):
                        form_control(f'Field({index})')
                        wait(lambda:state()['transform_edit']['active']==index)
                        for attempt in range(3):
                            desktop.key_state('Control_L',True)
                            try:
                                wait(lambda:state()['control_down'],1)
                                break
                            except TimeoutError:
                                desktop.key_state('Control_L',False)
                                input_notes.append(dict(case=name,modifier='Control_L',missed_attempt=attempt+1))
                        else: raise AssertionError('Owned modifier was not delivered')
                        try: desktop.key('a')
                        finally: desktop.key_state('Control_L',False)
                        wait(lambda:not state()['control_down'] and state()['transform_edit']['selection']==[0,len(state()['transform_edit']['fields'][index])])
                        for character in value:
                            desktop.key({'.':'period','-':'minus'}.get(character,character))
                        wait(lambda:state()['transform_edit']['fields'][index]==value)
                    def oracle(before,edit,label):
                        prior=output/f'{name}.{label}.before.json';prior.write_text(json.dumps(before))
                        commands=output/f'{name}.{label}.commands.jsonl'
                        commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=[edit]))+'\n')
                        expected=output/f'{name}.{label}.expected.json'
                        subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),
                            '--output',str(expected)],check=True,capture_output=True)
                        assert normalized(json.loads(expected.read_text()))==normalized(state()['document'])
                    before=state()['document']
                    control('Transform');wait(lambda:state()['transform_edit'] is not None)
                    assert len(state()['transform_edit']['fields'])==1
                    field(0,'3.5')
                    desktop.key('Tab');fresh();assert state()['transform_edit']['active']==0
                    capture('guide-width-ready')
                    form_control('Apply');wait(lambda:state()['revision']==8 and state()['transform_edit'] is None)
                    wide=state()['document']
                    oracle(before,dict(op='put_path',path=wide['paths'][0]),'guide-width')
                    control('Command(Undo)');wait(lambda:state()['revision']==9)
                    assert state()['document']==before
                    control('Command(Redo)');wait(lambda:state()['revision']==10)
                    assert state()['document']==wide
                    control('Point(1)');wait(lambda:state()['path_point']==0)
                    control('Transform');wait(lambda:state()['transform_edit'] is not None)
                    assert len(state()['transform_edit']['fields'])==4
                    # Collapsing the first segment must fail atomically.
                    field(0,str(origin[0]+64))
                    form_control('Apply');wait(lambda:'Path segments' in state()['status'])
                    assert state()['document']==wide and state()['revision']==10
                    field(0,str(origin[0]+48.125))
                    field(3,'0.1');form_control('Apply')
                    wait(lambda:'Path width' in state()['status'])
                    assert state()['document']==wide and state()['history']==[2,0]
                    field(3,'3.25')
                    desktop.key('Tab');wait(lambda:state()['transform_edit']['active']==0)
                    capture('guide-point-ready')
                    form_control('Apply');wait(lambda:state()['revision']==11 and state()['transform_edit'] is None)
                    moved=state()['document']
                    assert moved['paths'][0]['points'][1:]==wide['paths'][0]['points'][1:]
                    assert state()['history']==[3,0]
                    oracle(wide,dict(op='put_path',path=moved['paths'][0]),'guide-point')
                    control('Command(Undo)');wait(lambda:state()['revision']==12)
                    assert state()['document']==wide
                    control('Command(Redo)');wait(lambda:state()['revision']==13)
                    assert state()['document']==moved
                    # Mouse topology controls retain the shared one-entry history.
                    control('Command(InsertPoint)');wait(lambda:state()['revision']==14)
                    inserted=state()['document']
                    assert len(inserted['paths'][0]['points'])==4 and state()['path_point']==1
                    control('Command(RemovePoint)');wait(lambda:state()['revision']==15)
                    assert state()['document']==moved
                    control('Command(Undo)');wait(lambda:state()['revision']==16)
                    assert state()['document']==inserted
                    control('Command(Undo)');wait(lambda:state()['revision']==17)
                    assert state()['document']==moved
                    control('Command(Redo)');wait(lambda:state()['revision']==18)
                    assert state()['document']==inserted
                    control('Command(Redo)');wait(lambda:state()['revision']==19)
                    assert state()['document']==moved
                    # Explicit selection after topology reconciliation.
                    control('WholePath');control('Point(1)');wait(lambda:state()['path_point']==0)
                    control('Command(RemovePoint)');wait(lambda:state()['revision']==20)
                    assert len(state()['document']['paths'][0]['points'])==2
                    control('Command(RemovePoint)');fresh()
                    assert state()['revision']==20
                    control('Command(Undo)');wait(lambda:state()['revision']==21)
                    assert state()['document']==moved
                    control('WholePath');control('Point(-1)');wait(lambda:state()['path_point']==2)
                    control('Point(1)');wait(lambda:state()['path_point'] is None)
                    assert state()['revision']==21
                    control('Command(Save)');wait(lambda:not state()['unsaved_changes'])
                    assert normalized(json.loads(saved.read_text()))==normalized(moved)
                    control('Mode(Objects)');fresh()
                    assert all(not all(x>0 for x in b['size']) for b in state()['workspace_controls']
                        if b['target'] in ('Point(-1)','Point(1)','WholePath','Command(InsertPoint)','Command(RemovePoint)'))
                    if axis==33:
                        click(next(b for b in state()['content_list']['controls'] if b['target']=='Previous'))
                        wait(lambda:state()['content_list']['page']==0)
                    click(next(b for b in state()['content_list']['controls'] if b['target']=='Object(1)'))
                    control('Transform');wait(lambda:state()['transform_edit'] is not None)
                    assert len(state()['transform_edit']['fields'])==7
                    form_control('Field(6)');wait(lambda:state()['transform_edit']['active']==6)
                    form_control('Cancel');wait(lambda:state()['transform_edit'] is None)
                    assert state()['revision']==21 and state()['document']==moved
                if views:
                    before=copy.deepcopy(state()['document'])
                    def view(preset):
                        control(f'View({preset})')
                        wait(lambda:state()['view']['preset']==preset and state()['camera_projection']['kind']==
                            ('Perspective' if preset=='Perspective' else 'Orthographic'))
                    def oracle(prior_document,edit,label):
                        prior=output/f'{name}.{label}.before.json';prior.write_text(json.dumps(prior_document))
                        commands=output/f'{name}.{label}.commands.jsonl'
                        commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=[edit]))+'\n')
                        expected=output/f'{name}.{label}.expected.json'
                        subprocess.run([str(binary),'--headless','--zone',str(prior),'--commands',str(commands),
                            '--output',str(expected)],check=True,capture_output=True)
                        assert normalized(json.loads(expected.read_text()))==normalized(state()['document'])
                    def grid_check(preset):
                        current=fresh();grid=current['grid']
                        surface=preset in ('Top','Perspective')
                        assert grid['surface']==surface and grid['enabled']
                        assert grid['surface_visible']==surface
                        assert len(grid['lines'])==130
                        visible=[line for line in grid['lines'] if line['visible']]
                        assert len(visible)==(0 if surface else 130)
                        if surface:
                            assert grid['spacing']==(4 if axis==33 else 2)
                        else:
                            spacing=grid['spacing'];span=current['view']['distance']
                            assert span/20<=spacing<=span/4
                            normal=2 if preset=='Front' else 0
                            for line in visible:
                                assert abs(line['position'][normal]-.004)<1e-6
                                assert all(v>0 for v in line['scale'])
                                for coordinate in [i for i in range(3) if i!=normal]:
                                    value=line['position'][coordinate]/spacing
                                    assert abs(value-round(value))<1e-4
                    for preset in ['Top','Front','Right','Perspective']:
                        view(preset);grid_check(preset)
                        control('Grid');wait(lambda:not state()['grid']['enabled'])
                        assert not any(line['visible'] for line in state()['grid']['lines'])
                        assert not state()['grid']['surface_visible']
                        control('Grid');wait(lambda:state()['grid']['enabled']);grid_check(preset)
                        assert state()['document']==before and state()['revision']==7 and state()['history']==[1,1]
                    view('Front');control('Frame');grid_check('Front')
                    spacing=state()['grid']['spacing']
                    desktop.move(window,[1720,720],1)
                    for _ in range(9):
                        desktop.button(5,True);desktop.button(5,False)
                    wait(lambda:state()['grid']['spacing']>spacing);grid_check('Front')
                    capture('front-grid')
                    control('Mode(Sculpt)');control('Frame');view('Top')
                    wait(lambda:state()['camera_projection']['height']==state()['view']['distance'])
                    # Canonical pan/zoom settings do not create source history.
                    target=state()['view']['target']
                    desktop.key_state('w',True)
                    try:wait(lambda:state()['view']['target'][2]<target[2])
                    finally:desktop.key_state('w',False)
                    control('Frame')
                    desktop.move(window,[1720,720],1)
                    height=state()['camera_projection']['height']
                    position=state()['camera_position']
                    desktop.button(4,True);desktop.button(4,False)
                    wait(lambda:state()['camera_projection']['height']<height)
                    assert state()['camera_position']==position
                    control('Frame');fresh()
                    point=state()['terrain_points'][1]
                    desktop.move(window,point,1);hit=wait(lambda:state()['brush_point'])
                    settings=state()
                    desktop.button(1,True)
                    try:wait(lambda:state()['stroke_progress'] and state()['stroke_progress']['stamps']==1)
                    finally:desktop.button(1,False)
                    wait(lambda:state()['revision']==8)
                    sculpted=copy.deepcopy(state()['document'])
                    oracle(before,dict(op='brush',brush=dict(centre=[origin[0]+hit[0],origin[2]+hit[2]],
                        radius_metres=settings['brush_radius'],operation=dict(kind='raise',metres=settings['brush_strength']))),'top-sculpt')
                    capture('top-sculpt')
                    control('Command(Undo)');wait(lambda:state()['revision']==9)
                    assert state()['document']==before
                    control('Command(Redo)');wait(lambda:state()['revision']==10)
                    assert state()['document']==sculpted
                    control('Command(Undo)');wait(lambda:state()['revision']==11)
                    assert state()['document']==before
                    control('Mode(Objects)')
                    click(next(b for b in state()['content_list']['controls'] if b['target']=='Object(1)'))
                    control('Frame');view('Front');fresh()
                    arrow=wait(lambda:next((h for h in state()['move_handles'] if h['visible'] and h['axis']==1),None))
                    for point in [arrow['screen'],arrow['target']]:
                        assert point and 300<point[0]<3140 and 92<point[1]<1388, point
                    desktop.move(window,arrow['screen'],1);desktop.button(1,True)
                    try:
                        wait(lambda:state()['dragging']);desktop.move(window,arrow['target'],1)
                        preview=wait(lambda:state()['move_preview'] if state()['move_preview'] and state()['move_preview']!=before['objects'][0]['pose'] else None)
                        camera=copy.deepcopy(state()['view'])
                        desktop.key_state('w',True);desktop.button(3,True)
                        try:
                            desktop.move(window,[arrow['target'][0]+20,arrow['target'][1]],1)
                            desktop.button(4,True);desktop.button(4,False);fresh()
                            assert state()['view']==camera and state()['camera_projection']['kind']=='Orthographic'
                        finally:desktop.button(3,False);desktop.key_state('w',False)
                        assert state()['document']==before and state()['revision']==11
                        capture('front-move')
                    finally:desktop.button(1,False)
                    wait(lambda:state()['revision']==12 and not state()['dragging'])
                    moved=copy.deepcopy(state()['document'])
                    assert moved['objects'][0]['pose']==preview
                    for other in [0,2]: assert preview['position'][other]==before['objects'][0]['pose']['position'][other]
                    assert preview['position'][1]==before['objects'][0]['pose']['position'][1]+2
                    oracle(before,dict(op='transform',id=1,pose=preview),'front-move')
                    control('Command(Undo)');wait(lambda:state()['revision']==13)
                    assert state()['document']==before
                    control('Command(Redo)');wait(lambda:state()['revision']==14)
                    assert state()['document']==moved
                    view('Right');control('Frame')
                    target=state()['view']['target']
                    desktop.key_state('d',True)
                    try:wait(lambda:state()['view']['target'][2]<target[2])
                    finally:desktop.key_state('d',False)
                    target=state()['view']['target']
                    desktop.key_state('w',True)
                    try:wait(lambda:state()['view']['target'][1]>target[1])
                    finally:desktop.key_state('w',False)
                    control('Frame');capture('right-view')
                    # Orbiting canonical views returns to perspective.
                    desktop.move(window,[1720,720],1);desktop.button(3,True)
                    try:
                        desktop.move(window,[1740,730],1)
                        wait(lambda:state()['view']['preset']=='Perspective' and state()['camera_projection']['kind']=='Perspective')
                    finally:desktop.button(3,False)
                    view('Top');control('Transform')
                    wait(lambda:state()['transform_edit'] is not None)
                    control('View(Front)');assert state()['view']['preset']=='Top'
                    control('Grid');assert state()['grid']['enabled']
                    desktop.key('Escape');wait(lambda:state()['transform_edit'] is None)
                    control('Mode(Guides)');tool('NewPath(Road)')
                    wait(lambda:state()['path_draft_points']==0)
                    control('View(Front)');assert state()['view']['preset']=='Top' and state()['path_draft_points']==0
                    control('Grid');assert state()['grid']['enabled']
                    desktop.key('Escape');wait(lambda:state()['path_draft_points'] is None)
                    view('Right');control('Command(Play)')
                    wait(lambda:state()['play_feet'] is not None and state()['camera_projection']['kind']=='Perspective')
                    assert not any(line['visible'] for line in state()['grid']['lines'])
                    assert not state()['grid']['surface_visible']
                    control('Grid');assert state()['grid']['enabled']
                    control('View(Front)');assert state()['view']['preset']=='Right'
                    control('Command(Play)');wait(lambda:state()['play_feet'] is None and state()['camera_projection']['kind']=='Orthographic')
                    grid_check('Right')
                    assert state()['view']['preset']=='Right' and state()['revision']==14 and state()['document']==moved
                    control('Command(Save)');wait(lambda:not state()['unsaved_changes'])
                    assert normalized(json.loads(saved.read_text()))==normalized(moved)
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
    result=dict(cases=cases,input_notes=input_notes,checks=['mouse workspace buttons, snap checkbox, contextual standard slider click and drag',
        'settings preserve history; actual paint stamp equals headless command, one-entry undo/redo/save',
        'outliner selection, docked numeric transform matches headless command with one-entry undo/redo, name action, framing and playtest isolation'],scope='Linux native 3440x1440 only; first application-shell slice')
    if scatter: result['checks'].append('vegetation slider boundaries, integer count, tool visibility, history isolation and seeded scatter/headless equality, one-entry undo/redo/save')
    if guides: result['checks'].append('guide width and point properties, segment/range validation, exact headless equivalence, one-entry undo/redo/save, mouse point selection/topology, two-point minimum and primitive row restoration')
    if views: result['checks'].append('grid toggle and terrain surface visibility, adaptive vertical reference spacing and plane alignment, zoom, no history, modal/draft/play isolation and restore')
    if views: result['checks'].append('orthographic top/front/right projections, canonical pan/zoom, top sculpt and front Y move equal exact agent commands, camera frozen during held transform, orbit perspective, modal/draft/play isolation and restored projection/save')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    parser.add_argument('--scatter',action='store_true')
    parser.add_argument('--guides',action='store_true')
    parser.add_argument('--views',action='store_true')
    args=parser.parse_args();run(args.output,args.binary,args.scatter,args.guides,args.views)
