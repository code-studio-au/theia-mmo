"""Qualify exact two-slot crystal/Standard bindings with actual Editor slots."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
from unity_scene import Sources,records
from convert_unity_mixed import convert
from verify_unity_reframe import verify as verify_frame
from verify_unity_submesh_slots import verify as verify_slots
from qualify_foliage_group import qualify


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def batch(source,shaders,base,projects,output,probe,reuse=(),topology_references=()):
    source=Sources(source);base=Path(base).resolve(strict=True);output=Path(output).resolve()
    if output.exists() or not output.is_relative_to(Path(__file__).resolve().parents[1]/'build'):raise ValueError('Use a fresh ignored mixed batch')
    if output.is_relative_to(source.root) or source.root.is_relative_to(output) or output.is_relative_to(Path(shaders).resolve()) or Path(shaders).resolve().is_relative_to(output):raise ValueError('Mixed batch overlaps source inputs')
    previous=json.loads((base/'batch.json').read_text());base_hash=sha(base/'batch.json');plan_path=Path(previous['source_plan'])
    if sha(plan_path)!=previous['plan_sha256']:raise ValueError('Mixed source plan changed')
    plan=json.loads(plan_path.read_text())
    if plan['archive_sha256']!=source.manifest['archive_sha256']:raise ValueError('Mixed source archive differs')
    groups={}
    for d in plan['draws']:
        key=hashlib.sha256(json.dumps([d['mesh'],d['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest();groups.setdefault(key,[]).append(d)
    references={}
    for project in projects:
        project=Path(project).resolve(strict=True)
        for model in json.loads((project/'Reference/request.json').read_text())['models']:
            if model['path'] in references:raise ValueError('Duplicate Editor source model reference')
            references[model['path']]=project
    reused={}
    for path in reuse:
        path=Path(path).resolve(strict=True);r=json.loads((path/'group.json').read_text())
        if r['key'] in reused or r['source_plan_sha256']!=previous['plan_sha256']:raise ValueError('Duplicate/wrong-plan reused group')
        reused[r['key']]=path
    topology={}
    for path in topology_references:
        path=Path(path).resolve(strict=True);conversion=json.loads((path/'conversion.json').read_text());binding=conversion['scene_binding']
        witness,=[d for d in plan['draws'] if (d['instance'],d['renderer'])==(binding['instance'],binding['renderer'])]
        key=hashlib.sha256(json.dumps([witness['mesh'],witness['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest()
        if key in topology or binding['plan_sha256']!=previous['plan_sha256']:raise ValueError('Duplicate/wrong-plan topology reference')
        topology[key]=path
    selected=[]
    for r in previous['unresolved']:
        refs=groups[r['key']][0]['materials']
        if len(refs)!=2 or refs[0]==refs[1]:continue
        families=[]
        for ref in refs:
            name=source.by_guid[ref['guid']];f,=[v['fields'] for v in records(source.checked(name)).values() if v['kind']==21];families.append(f['m_Shader'].get('guid'))
        if sorted(families)==sorted(['006c4dbb387622342a1397597c7b3e66','0000000000000000f000000000000000']):selected.append(r)
    if not selected or not (set(reused)|set(topology))<={r['key'] for r in selected} or set(reused)&set(topology):raise ValueError('Select eligible exact mixed groups and matching reuse/topology')
    output.mkdir(parents=True);report={'schema':1,'source_plan':str(plan_path),'plan_sha256':previous['plan_sha256'],'base':str(base),'base_batch_sha256':base_hash,
                'selected_bindings':len(selected),'selected_draws':sum(len(groups[r['key']]) for r in selected),'converted':[],'unresolved':[]}
    for old in selected:
        key=old['key'];draws=groups[key];w=draws[0];target=output/'prototypes'/key
        try:
            if key in reused:
                prior=reused[key];checked=verify_frame(source.root,prior,probe,shaders);slots=verify_slots(prior);group=json.loads((prior/'group.json').read_text())
                if not group['passed'] or group['frame']!=checked:raise ValueError('Reused prototype frame proof changed')
                shutil.copytree(prior,target)
            else:
                raw=output/'source'/key;conversion=convert(source.root,shaders,plan_path,w['instance'],w['renderer'],raw,topology.get(key),probe)
                group=qualify(source.root,shaders,raw,references[conversion['model']],target,probe);slots=verify_slots(target)
            record={'key':key,'mesh':w['mesh'],'materials':w['materials'],'draws':len(draws),
                    'prototype_sha256':sha(target/'preview.glb'),'placements_sha256':sha(target/'placements.glb'),
                    'group_sha256':sha(target/'group.json'),'editor_slots':slots,'triangles':group['placements']['triangles']}
            if key in reused:record['reused_from']=str(reused[key])
            report['converted'].append(record);print('QUALIFIED',key,len(draws),slots['triangles_by_slot'],flush=True)
        except (ValueError,KeyError,IndexError,TypeError) as error:
            report['unresolved'].append({'key':key,'mesh':w['mesh'],'materials':w['materials'],'draws':len(draws),'reason':str(error),'exception':type(error).__name__});print('UNRESOLVED',key,str(error),flush=True)
        if sha(plan_path)!=report['plan_sha256'] or sha(base/'batch.json')!=base_hash:raise ValueError('Mixed plan/base changed during conversion')
        (output/'batch.json').write_text(json.dumps(report,indent=2)+'\n')
    report['converted_draws']=sum(r['draws'] for r in report['converted']);report['unresolved_draws']=sum(r['draws'] for r in report['unresolved'])
    (output/'batch.json').write_text(json.dumps(report,indent=2)+'\n');return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['source','shaders','base','output']:p.add_argument(name,type=Path)
    p.add_argument('--project',type=Path,action='append',required=True);p.add_argument('--probe',type=Path,required=True);p.add_argument('--reuse',type=Path,action='append',default=[])
    p.add_argument('--topology-reference',type=Path,action='append',default=[])
    a=p.parse_args();r=batch(a.source,a.shaders,a.base,a.project,a.output,a.probe,a.reuse,a.topology_reference);print(json.dumps({k:v for k,v in r.items() if k not in ['converted','unresolved']}))
