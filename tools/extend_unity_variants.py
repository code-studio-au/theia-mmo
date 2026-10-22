"""Extend a verified static scene batch with reviewed single-FBX variants.

Prior artifact bytes are copied unchanged after final-scene/hash checks. The full
source plan and variant category remain intact; each new binding gets fresh raw
source, pixel/geometry, ufbx and complete placement qualification. No collision or
runtime map adoption is implied.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import shutil
from unity_scene import Sources
from unity_draw_bindings import checked_records
from unity_materials import standard_material
from preview_unity_static import preview
from verify_unity_static import verify as verify_static
from verify_unity_model_variants import verify as verify_variants
from verify_foliage_batch_scene import verify as verify_scene
from verify_unity_geometry import canonical,assembly
from unity_geometry import assemble


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def extend(source,base,scene,output,probe):
    sources=Sources(source);base=Path(base).resolve(strict=True);scene=Path(scene).resolve(strict=True);output=Path(output).resolve()
    if output.exists() or any(output.is_relative_to(p) or p.is_relative_to(output) for p in [sources.root,base,scene]):raise FileExistsError('Use a new output outside all inputs')
    raw=(base/'batch.json').read_bytes();report=json.loads(raw);plan_path=Path(report['source_plan']);plan_raw=plan_path.read_bytes();plan=json.loads(plan_raw)
    if hashlib.sha256(plan_raw).hexdigest()!=report['plan_sha256'] or plan['archive_sha256']!=sources.manifest['archive_sha256']:raise ValueError('Source plan/archive differs')
    # Recheck raw inputs and all previously qualified artifact hashes before reuse.
    for name in sources.manifest['selected']:sources.checked(name);sources.metadata(name)
    prior=verify_scene(base,scene)
    for r in report['converted']:
        d=base/'prototypes'/r['key']
        if sha(d/'preview.glb')!=r['prototype_sha256'] or sha(d/'placements.glb')!=r['placements_sha256']:raise ValueError('Prior artifact changed')
    reference=verify_variants(source,plan_path)
    groups={}
    for draw in plan['draws']:
        key=hashlib.sha256(json.dumps([draw['mesh'],draw['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest();groups.setdefault(key,[]).append(draw)
    selected=[]
    for r in report['unresolved']:
        draws=groups[r['key']]
        if not all(d['instance'] is not None and any(v['kind']==1001 for v in checked_records(sources,d['source']).values()) for d in draws):continue
        refs=draws[0]['materials']
        if not refs or any(v!=refs[0] for v in refs):raise ValueError('Variant requires one repeated material')
        name=sources.by_guid[refs[0]['guid']];material,=[v['fields'] for v in checked_records(sources,name).values() if v['kind']==21];standard_material(material)
        selected.append((r,draws))
    if not selected:raise ValueError('No unresolved model-variant bindings')
    shutil.copytree(base,output)
    (output/'reused-scene-reference.json').write_text(json.dumps(prior,indent=2)+'\n');(output/'variant-reference.json').write_text(json.dumps(reference,indent=2)+'\n')
    for r,draws in selected:
        directory=output/'prototypes'/r['key']
        if directory.exists():raise FileExistsError('Variant output unexpectedly already exists')
        witness=draws[0];result=preview(source,witness['source'],directory,False,True,plan_path=plan_path,instance=witness['instance'],renderer=witness['renderer'])
        checked=verify_static(source,directory);(directory/'independent-verification.json').write_text(json.dumps(checked,indent=2)+'\n')
        geometry=canonical(source,directory,probe);(directory/'canonical-reference.json').write_text(json.dumps(geometry,indent=2)+'\n')
        instances=[{'id':str(d['instance'])+':'+str(d['renderer']),'matrix':d['matrix']} for d in draws]
        (directory/'instances.json').write_text(json.dumps({'instances':instances,'source_plan_sha256':report['plan_sha256']})+'\n')
        placed=assemble(directory/'preview.glb',instances,directory/'placements.glb');oracle=assembly(directory/'preview.glb',instances,directory/'placements.glb')
        (directory/'assembly.json').write_text(json.dumps(placed,indent=2)+'\n');(directory/'assembly-reference.json').write_text(json.dumps(oracle,indent=2)+'\n')
        record={k:v for k,v in r.items() if k not in ('reason','exception')}
        record.update(profile='standard-opaque-model-variant',source=witness['source'],instance=witness['instance'],renderer=witness['renderer'],triangles_per_instance=result['source_triangles'],triangles=oracle['triangles'],prototype_sha256=result['preview_sha256'],placements_sha256=placed['asset_sha256'],expected_bounds=oracle['expected_bounds'])
        report['converted'].append(record);report['unresolved']=[v for v in report['unresolved'] if v['key']!=r['key']]
        print(json.dumps({'key':r['key'],'draws':len(draws),'passed':True}),flush=True)
    if (base/'batch.json').read_bytes()!=raw or plan_path.read_bytes()!=plan_raw:raise ValueError('Input report/plan changed during extension')
    # A new Sources object performs a fresh end-of-run hash check.
    final_sources=Sources(source)
    if final_sources.manifest!=sources.manifest:raise ValueError('Source manifest changed during extension')
    for name in final_sources.manifest['selected']:final_sources.checked(name);final_sources.metadata(name)
    report['converted_draws']=sum(r['draws'] for r in report['converted']);report['unresolved_draws']=sum(r['draws'] for r in report['unresolved']);report['unresolved_reasons']=dict(Counter(r['reason'] for r in report['unresolved']))
    report['extension']={'base_batch':str(base),'base_batch_sha256':hashlib.sha256(raw).hexdigest(),'base_scene_sha256':prior['asset_sha256'],'new_bindings':len(selected),'source_variants_independently_checked':reference['model_placements'],'scope':'canonical single-model variants with source-authored complete outer TRS; copied qualified prior artifacts unchanged'}
    (output/'batch.json').write_text(json.dumps(report,indent=2)+'\n');return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('source',type=Path);p.add_argument('base',type=Path);p.add_argument('scene',type=Path);p.add_argument('output',type=Path);p.add_argument('--probe',required=True,type=Path);a=p.parse_args();result=extend(a.source,a.base,a.scene,a.output,a.probe);print(json.dumps({k:v for k,v in result.items() if k not in ('converted','unresolved')}))
