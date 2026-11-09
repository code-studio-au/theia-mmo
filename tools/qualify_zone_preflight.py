#!/usr/bin/env python3
"""Read-only actual Avian spawn preflight, with desktop connections disabled."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]


def run(output,binary):
    output=output.resolve();output.mkdir(parents=True,exist_ok=False);binary=binary.resolve()
    runtime=output/'no-desktop-runtime';runtime.mkdir(mode=0o700)
    env=dict(os.environ,DISPLAY='',WAYLAND_DISPLAY='preflight-disabled',XDG_RUNTIME_DIR=str(runtime))
    env.pop('WAYLAND_SOCKET',None)
    cases=[]
    def invoke(label,args,ready=True,launcher=False):
        executable=[sys.executable,str(ROOT/'tools/edit_zone.py'),'--no-build'] if launcher else [str(binary)]
        completed=subprocess.run([*executable,*map(str,args)],cwd=ROOT,env=env,capture_output=True,text=True,timeout=45)
        (output/f'{label}.stdout').write_text(completed.stdout);(output/f'{label}.stderr').write_text(completed.stderr)
        assert completed.returncode==(0 if ready else 1),(label,completed.returncode,completed.stderr)
        report=json.loads(completed.stdout)
        assert report['valid'] is True and report['playtest']['ready']==ready,report
        assert (report['playtest']['resolved_spawn'] is not None)==ready
        assert (report['playtest']['error'] is None)==ready
        if not ready:assert 'Playtest spawn refused' in report['playtest']['error']
        cases.append(dict(name=label,returncode=completed.returncode,report=report))
        return report
    for label,origin in [('local',[.125,.25,-.5]),('far',[128000000000.125,20.25,-128000000000.5])]:
        def fixture(height):
            return dict(schema=1,name='Spawn preflight',origin=origin,spawn=[origin[0],origin[1]+height,origin[2]+6],objects=[])
        source=output/f'{label}.json';source.write_text(json.dumps(fixture(.0625)));source_bytes=source.read_bytes()
        report=invoke(f'{label}-flat',['--zone',source,'--check-playtest'])
        assert report['revision']==0 and report['history']==[0,0] and report['objects']==0
        assert report['playtest']['requested_spawn']==fixture(.0625)['spawn']
        resolved=report['playtest']['resolved_spawn']
        assert abs(resolved[0]-origin[0])<.001 and abs(resolved[2]-origin[2]-6)<.001
        assert abs(resolved[1]-origin[1])<.03
        assert source.read_bytes()==source_bytes
        unsafe=output/f'{label}.unsupported.json';unsafe.write_text(json.dumps(fixture(.125)));unsafe_bytes=unsafe.read_bytes()
        report=invoke(f'{label}-unsupported',['--zone',unsafe,'--check-playtest'],ready=False)
        assert report['revision']==0 and report['history']==[0,0]
        # Normal source validation stays separate from physical spawn readiness.
        ordinary=subprocess.run([str(binary),'--zone',str(unsafe),'--headless'],env=env,capture_output=True,text=True,timeout=45)
        assert ordinary.returncode==0 and json.loads(ordinary.stdout)['valid']
        assert 'playtest' not in json.loads(ordinary.stdout)
        commands=output/f'{label}.fix.jsonl'
        commands.write_text(json.dumps(dict(command='apply',expected_revision=0,edits=[dict(op='set_spawn',position=fixture(.0625)['spawn'])]))+'\n')
        command_bytes=commands.read_bytes()
        report=invoke(f'{label}-memory-fix',['--zone',unsafe,'--commands',commands,'--check-playtest'])
        assert report['revision']==1 and report['history']==[1,0]
        assert report['playtest']['requested_spawn']==fixture(.0625)['spawn']
        assert unsafe.read_bytes()==unsafe_bytes and commands.read_bytes()==command_bytes
        blocked=copy.deepcopy(fixture(.0625));blocked['objects']=[dict(id=1,name='Blocked landing',primitive='box',pose=dict(
            position=[origin[0],origin[1]+1,origin[2]+6],yaw_degrees=23.5,size=[2,2,2]))]
        wall=output/f'{label}.blocked.json';wall.write_text(json.dumps(blocked));wall_bytes=wall.read_bytes()
        report=invoke(f'{label}-blocked',['--zone',wall,'--check-playtest'],ready=False)
        assert report['objects']==1 and wall.read_bytes()==wall_bytes
    for label,origin in [('terrain-local',[0,0,0]),('terrain-far',[128000000000,20,-128000000000])]:
        for axis in [33,65]:
            heights=[0 if x in (0,axis-1) or z in (0,axis-1) else 1 for z in range(axis) for x in range(axis)]
            terrain=dict(chunk=[origin[0]//128,origin[2]//128],heights=heights)
            if axis==65:terrain['resolution']='2m'
            source=output/f'{label}-{axis}.json'
            source.write_text(json.dumps(dict(schema=7,name='Raised terrain',origin=origin,
                spawn=[origin[0]+64,origin[1]+1.0625,origin[2]+64],terrain=terrain,objects=[])))
            before=source.read_bytes();report=invoke(f'{label}-{axis}',['--zone',source,'--check-playtest'])
            assert abs(report['playtest']['resolved_spawn'][1]-origin[1]-1)<.03
            assert source.read_bytes()==before
    # Read-only rejects mutating/file-report modes before preparing any output.
    source=output/'local.json';target=output/'existing-output.json';target.write_text('retain this exact destination')
    for index,extra in enumerate([['--output',target],['--output',output/'new-output.json'],
        ['--export',output/'new-export','--content-id','preflight-test','--revision','1'],
        ['--capture',output/'never.png'],['--report',output/'never.json'],['--mailbox',output/'never-mailbox']]):
        before=target.read_bytes()
        completed=subprocess.run([str(binary),'--zone',str(source),'--check-playtest',*map(str,extra)],env=env,capture_output=True,text=True,timeout=45)
        assert completed.returncode==1 and completed.stdout=='' and target.read_bytes()==before
        assert not any((output/name).exists() for name in ['new-output.json','new-export','never.png','never.json','never-mailbox'])
        (output/f'forbidden-{index}.stderr').write_text(completed.stderr)
    # The launcher must not create a missing draft parent in read-only mode.
    missing=output/'uncreated-parent'/'draft.json'
    report=invoke('launcher-empty',['--draft',missing,'--check-playtest'],launcher=True)
    assert report['objects']==0 and not missing.parent.exists()
    report=invoke('launcher-reopen',['--draft',source,'--check-playtest'],launcher=True)
    assert report['playtest']['requested_spawn']==json.loads(source.read_text())['spawn']
    example=output/'uncreated-example-parent'/'draft.json'
    report=invoke('launcher-example',['--example','--draft',example,'--check-playtest'],launcher=True)
    assert report['objects']>0 and not example.parent.exists()
    result=dict(cases=cases,editor_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),checks=[
        'actual Avian safe/unsupported/blocked spawn checks at fractional local/far origins',
        'in-memory SetSpawn repairs readiness with shared command revision/history and unchanged input bytes',
        'raised 4m/2m terrain uses actual triangle support at both origins',
        'output/export/window-report modes refused without writes; launcher defaults/reopen/example remain read-only'],
        scope='Linux CPU/headless only; DISPLAY/Wayland connections disabled; no renderer, native input, Windows or multiplayer qualification')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    args=parser.parse_args();run(args.output,args.binary)
