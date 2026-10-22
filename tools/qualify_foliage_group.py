"""Qualify a canonical static prototype and only its exact source binding.

Retain the foliage command name for existing qualification harnesses. Explicit
Standard profiles use their independent verifier through the same frame/group path.
"""
import argparse
import hashlib
import json
from pathlib import Path
from reframe_unity_static import reframe
from verify_unity_reframe import verify as verify_frame
from verify_unity_foliage import verify as verify_material
from verify_unity_static import verify as verify_static
from verify_unity_geometry import assembly
from unity_geometry import assemble


def source_group(draws, binding, prefab):
    witness,=[d for d in draws if (d['instance'],d['renderer'])==(binding['instance'],binding['renderer'])]
    key_data=[witness['mesh'],witness['materials']]
    key=hashlib.sha256(json.dumps(key_data,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    selected=[d for d in draws if [d['mesh'],d['materials']]==key_data]
    if not selected or witness['source']!=prefab:raise ValueError('Exact source witness differs')
    return key,witness,selected


def qualify(source, shaders, prototype, project, output, probe):
    prototype=Path(prototype).resolve(strict=True);output=Path(output).resolve()
    if output.exists() or not output.is_relative_to(Path(__file__).resolve().parents[1]/'build'):
        raise ValueError('Use a fresh ignored group output')
    conversion=json.loads((prototype/'conversion.json').read_text())
    binding=conversion['scene_binding'];plan_path=Path(binding['plan'])
    digest=hashlib.sha256(plan_path.read_bytes()).hexdigest()
    if digest!=binding['plan_sha256']:raise ValueError('Original scene binding changed')
    draws=json.loads(plan_path.read_text())['draws']
    key,witness,selected=source_group(draws,binding,conversion['prefab'])
    if conversion.get('material_plan',{}).get('profile') in ('unity-standard-opaque-v1','unity-standard-glass-blend-v1'):
        verified=verify_static(source,prototype)
    elif conversion.get('profile')=='unity-mixed-crystal-pbr-static-v1':
        from verify_unity_mixed import verify as verify_mixed
        verified=verify_mixed(source,shaders,prototype)
    else:
        verified=verify_material(source,shaders,prototype)
    reframe(source,prototype,project,output,probe)
    frame=verify_frame(source,output,probe,shaders)
    slots=None
    if conversion.get('profile')=='unity-mixed-crystal-pbr-static-v1':
        from verify_unity_submesh_slots import verify as verify_slots
        slots=verify_slots(output)
        (output/'editor-slot-reference.json').write_text(json.dumps(slots,indent=2)+'\n')
    (output/'independent-verification.json').write_text(json.dumps(frame,indent=2)+'\n')
    instances=[{'id':str(d['instance'])+':'+str(d['renderer']),'matrix':d['matrix']} for d in selected]
    (output/'instances.json').write_text(json.dumps({'source_plan_sha256':digest,'key':key,'instances':instances},indent=2)+'\n')
    placed=assemble(output/'preview.glb',instances,output/'placements.glb')
    checked=assembly(output/'preview.glb',instances,output/'placements.glb')
    (output/'assembly.json').write_text(json.dumps(placed,indent=2)+'\n')
    (output/'assembly-reference.json').write_text(json.dumps(checked,indent=2)+'\n')
    report={'passed':True,'key':key,'mesh':witness['mesh'],'materials':witness['materials'],
            'source_plan':str(plan_path),'source_plan_sha256':digest,'draws':len(selected),
            'source_material':verified,'frame':frame,'placements':checked}
    if slots is not None:report['editor_slots']=slots
    (output/'group.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['source','shaders','prototype','project','output','probe']:p.add_argument(name,type=Path)
    a=p.parse_args();r=qualify(a.source,a.shaders,a.prototype,a.project,a.output,a.probe)
    print(json.dumps({'key':r['key'],'draws':r['draws'],'frame':r['frame'],'placements':r['placements']}))
