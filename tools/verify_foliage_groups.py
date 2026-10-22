"""Check exact new foliage groups, immutable predecessors and plan coverage."""
import argparse
import hashlib
import json
from pathlib import Path
from verify_unity_geometry import assembly


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def verify(base, triplanar, stages):
    base=Path(base);triplanar=Path(triplanar)
    old=json.loads((base/'batch.json').read_text());tri=json.loads((triplanar/'batch.json').read_text())
    if old['plan_sha256']!=tri['plan_sha256'] or sha(base/'batch.json')!=tri['base_batch_sha256']:
        raise ValueError('Base/triplanar source binding changed')
    plan_path=Path(old['source_plan'])
    if sha(plan_path)!=old['plan_sha256']:raise ValueError('Source plan changed')
    groups={}
    for draw in json.loads(plan_path.read_text())['draws']:
        key=hashlib.sha256(json.dumps([draw['mesh'],draw['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest()
        groups.setdefault(key,[]).append(draw)
    existing={r['key'] for r in old['converted']}|{r['key'] for r in tri['converted']}
    added=set();checks=[]
    for stage in stages:
        stage=Path(stage);report=json.loads((stage/'group.json').read_text());key=report['key']
        if key in existing or key in added or key not in groups:raise ValueError('Group duplicates/overlaps existing source draws')
        added.add(key);draws=groups[key]
        if report['source_plan_sha256']!=old['plan_sha256'] or report['draws']!=len(draws) or report['mesh']!=draws[0]['mesh'] or report['materials']!=draws[0]['materials']:
            raise ValueError('Foliage group differs from exact source binding')
        instances=[{'id':str(d['instance'])+':'+str(d['renderer']),'matrix':d['matrix']} for d in draws]
        observed=json.loads((stage/'instances.json').read_text())
        if observed!={'source_plan_sha256':old['plan_sha256'],'key':key,'instances':instances}:raise ValueError('Foliage source placement set differs')
        receipt=json.loads((stage/'unity-frame.json').read_text());source=Path(receipt['source_prototype'])
        frame=json.loads((stage/'independent-verification.json').read_text())
        if frame!=report['frame'] or not frame['passed'] or sha(stage/'preview.glb')!=frame['asset_sha256'] or sha(source/'preview.glb')!=receipt['source_prototype_sha256'] or sha(source/'conversion.json')!=receipt['source_conversion_sha256']:
            raise ValueError('Qualified foliage frame/predecessor changed')
        if not report['source_material']['passed'] or report['source_material']['asset_sha256']!=receipt['source_prototype_sha256']:
            raise ValueError('Qualified source material predecessor differs')
        conversion=json.loads((stage/'conversion.json').read_text())
        if conversion.get('profile')=='unity-mixed-crystal-pbr-static-v1':
            from verify_unity_submesh_slots import verify as verify_slots
            slots=verify_slots(stage)
            if slots!=json.loads((stage/'editor-slot-reference.json').read_text()) or slots!=report.get('editor_slots',slots):
                raise ValueError('Actual Editor source material slots changed')
        placed=assembly(stage/'preview.glb',instances,stage/'placements.glb')
        if placed!=report['placements']:raise ValueError('Foliage placement geometry changed')
        checks.append({'stage':str(stage),'group_sha256':sha(stage/'group.json'),'source_draws':len(draws),'placements':placed})
    converted=existing|added
    return {'passed':True,'base_batch_sha256':sha(base/'batch.json'),'triplanar_batch_sha256':sha(triplanar/'batch.json'),
            'source_plan_sha256':old['plan_sha256'],'new_groups':len(added),'new_draws':sum(len(groups[k]) for k in added),
            'qualified_source_draws':sum(len(groups[k]) for k in converted),'remaining_draws':sum(len(v) for k,v in groups.items() if k not in converted),
            'remaining_bindings':sorted(set(groups)-converted),'checks':checks,
            'scope':'exact source group coverage and fresh source-placement geometry; actual Editor/material source qualification remains in bound predecessors'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('base',type=Path);p.add_argument('triplanar',type=Path);p.add_argument('stages',nargs='+',type=Path);p.add_argument('--report',type=Path,required=True)
    a=p.parse_args()
    if a.report.exists():raise FileExistsError('Use a fresh report')
    result=verify(a.base,a.triplanar,a.stages);a.report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items() if k!='checks'}))
