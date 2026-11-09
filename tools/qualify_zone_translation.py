#!/usr/bin/env python3
"""Structured mixed-entry offsets and atomic file output with displays disabled."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT=Path(__file__).resolve().parents[1]


def run(output,binary):
    output=output.resolve();output.mkdir(parents=True,exist_ok=False);binary=binary.resolve()
    runtime=output/'no-desktop-runtime';runtime.mkdir(mode=0o700)
    env=dict(os.environ,DISPLAY='',WAYLAND_DISPLAY='translation-disabled',XDG_RUNTIME_DIR=str(runtime));env.pop('WAYLAND_SOCKET',None)
    def execute(label,args,code=0):
        completed=subprocess.run([str(binary),*map(str,args)],cwd=ROOT,env=env,capture_output=True,text=True,timeout=45)
        (output/f'{label}.stdout').write_text(completed.stdout);(output/f'{label}.stderr').write_text(completed.stderr)
        assert completed.returncode==code,(label,completed.returncode,completed.stderr)
        return json.loads(completed.stdout) if completed.stdout else None
    seed=output/'seed.json'
    execute('seed',['--headless','--commands',ROOT/'content/zones/selection.commands.jsonl','--output',seed])
    base=json.loads(seed.read_text());cases=[]
    for label,origin in [('local',[.125,.25,-.5]),('far',[128000000000.125,20.25,-128000000000.5])]:
        zone=copy.deepcopy(base)
        def shifted(position,delta):return [a+b for a,b in zip(position,delta)]
        zone['origin']=shifted(zone['origin'],origin)
        zone['spawn']=[origin[0],origin[1]+.0625,origin[2]-64]
        for obj in zone['objects']:obj['pose']['position']=shifted(obj['pose']['position'],origin)
        for entry in zone['plants']+zone['markers']:entry['position']=shifted(entry['position'],origin)
        for marker in zone['markers']:marker['definition']='quest.authoring-note'
        for guide in zone['paths']:guide['points']=[shifted(point,origin) for point in guide['points']]
        zone['schema']=7
        zone['terrain']=dict(chunk=[int(origin[0]//128),int(origin[2]//128)],heights=[0]*(65*65),resolution='2m')
        source=output/f'{label}.source.json';source.write_text(json.dumps(zone));original_bytes=source.read_bytes()
        offset=[.125,.25,-.5];identifiers=[1,2,3,1000,1001,1002]
        command=dict(command='apply',expected_revision=0,edits=[dict(op='translate',id=i,offset=offset) for i in identifiers])
        commands=output/f'{label}.commands.jsonl';commands.write_text(json.dumps(command)+'\n');command_bytes=commands.read_bytes()
        target=output/f'{label}.translated.json'
        report=execute(f'{label}-move',['--headless','--zone',source,'--commands',commands,'--output',target])
        assert report['revision']==1 and report['history']==[1,0]
        expected=copy.deepcopy(zone)
        for obj in expected['objects']:obj['pose']['position']=shifted(obj['pose']['position'],offset)
        for entry in expected['plants']+expected['markers']:entry['position']=shifted(entry['position'],offset)
        for guide in expected['paths']:guide['points']=[shifted(point,offset) for point in guide['points']]
        actual=json.loads(target.read_text());assert actual==expected
        assert source.read_bytes()==original_bytes and commands.read_bytes()==command_bytes
        # A whole mixed selection remains one ordinary history transaction.
        undo=output/f'{label}.undo.jsonl';undo.write_text(json.dumps(command)+'\n'+json.dumps(dict(command='undo',expected_revision=1))+'\n')
        undone=output/f'{label}.undone.json'
        report=execute(f'{label}-undo',['--headless','--zone',source,'--commands',undo,'--output',undone])
        assert report['revision']==2 and report['history']==[0,1] and json.loads(undone.read_text())==zone
        redo=output/f'{label}.redo.jsonl';redo.write_text(undo.read_text()+json.dumps(dict(command='redo',expected_revision=2))+'\n')
        redone=output/f'{label}.redone.json'
        report=execute(f'{label}-redo',['--headless','--zone',source,'--commands',redo,'--output',redone])
        assert report['revision']==3 and report['history']==[1,0] and json.loads(redone.read_text())==expected
        report=execute(f'{label}-preflight',['--zone',source,'--commands',commands,'--check-playtest'])
        assert report['playtest']['ready'] and report['playtest']['requested_spawn']==zone['spawn']
        assert source.read_bytes()==original_bytes
        # Every failed file transaction retains a pre-existing output byte-for-byte.
        sentinel=output/f'{label}.retained.json';sentinel.write_text('retain destination exactly');sentinel_bytes=sentinel.read_bytes()
        bad_commands=[
            [dict(command='apply',expected_revision=0,edits=[dict(op='translate',id=1,offset=offset),dict(op='translate',id=9999,offset=offset)])],
            [dict(command='apply',expected_revision=0,edits=[dict(op='translate',id=1,offset=[1024,0,0])])],
            [dict(command='apply',expected_revision=0,edits=[dict(op='translate',id=1,offset=[1025,0,0])])],
            [dict(command='apply',expected_revision=1,edits=[dict(op='translate',id=1,offset=offset)])],
            [dict(command='apply',expected_revision=0,edits=[dict(op='translate',id=1,offset=offset,position=[0,0,0])])],
            [dict(command='apply',expected_revision=0,edits=[dict(op='translate',id=1,offset=offset)]*129)],
            [command,dict(command='undo',expected_revision=0)],
        ]
        for index,lines in enumerate(bad_commands):
            bad=output/f'{label}.bad-{index}.jsonl';bad.write_text(''.join(json.dumps(line)+'\n' for line in lines))
            execute(f'{label}-refused-{index}',['--headless','--zone',source,'--commands',bad,'--output',sentinel],code=1)
            assert sentinel.read_bytes()==sentinel_bytes and source.read_bytes()==original_bytes
        cases.append(dict(name=label,origin=origin,translated_ids=identifiers,offset=offset,full_document_equal=True,history_roundtrip=True,preflight_ready=True,refused_files=len(bad_commands)))
    result=dict(cases=cases,editor_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),checks=[
        'mixed primitive/plant/marker/guide translations preserve IDs, metadata, origin, spawn and terrain bytes',
        'one-entry undo/redo and read-only Avian preflight use the same command file',
        'unknown ID, invalid/boundary offsets, stale revisions, unknown fields, edit budget and late file errors preserve source/existing output'],
        scope='Linux CPU/headless only with display connections disabled; no native UI, Windows or multiplayer qualification')
    (output/'qualification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--binary',type=Path,default=Path('target/debug/theia-zone-editor'))
    args=parser.parse_args();run(args.output,args.binary)
