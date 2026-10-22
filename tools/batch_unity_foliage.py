"""Convert eligible foliage and opt-in Standard bindings with explicit omissions.

Runs in the pinned Blender environment. Validates ordinary source placements
independently before converting; every successful prototype gets geometry/pixel
and ufbx checks. Failed/unsupported groups remain in the report, never substituted.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
from unity_scene import Sources,records
from unity_draw_bindings import checked_records
from unity_foliage import BASIC_SHADER_GUID,vegetation_material
from unity_full_foliage import SHADER_GUID,full_material
from convert_unity_foliage import convert
from verify_unity_foliage import verify as verify_foliage
from verify_unity_prefabs import verify as verify_plan
from verify_unity_model_variants import verify as verify_variants
from verify_unity_geometry import canonical,assembly
from unity_geometry import assemble
from unity_materials import standard_material
from preview_unity_static import preview as convert_standard
from verify_unity_static import verify as verify_standard


def binding_key(draw):
    return json.dumps([draw['mesh'],draw['materials']],sort_keys=True,separators=(',',':'))


def batch(source,shader_source,plan_path,output,probe,include_standard=False):
    sources=Sources(source);shaders=Sources(shader_source);plan_path=Path(plan_path).resolve(strict=True);output=Path(output).resolve()
    if output.exists() or output.is_relative_to(sources.root) or output.is_relative_to(shaders.root):raise FileExistsError('Use a new batch directory outside sources')
    raw=plan_path.read_bytes()
    if len(raw)>64*1024*1024:raise ValueError('Source plan exceeds batch byte budget')
    plan=json.loads(raw);plan_hash=hashlib.sha256(raw).hexdigest()
    if plan['archive_sha256']!=sources.manifest['archive_sha256'] or sources.manifest['archive_sha256']!=shaders.manifest['archive_sha256']:raise ValueError('Batch source archives differ')
    groups={};ordinary=[];variants=[]
    for draw in plan['draws']:
        if draw['instance'] is None or not any(v['kind']==1001 for v in checked_records(sources,draw['source']).values()):ordinary.append(draw)
        else:variants.append(draw)
        groups.setdefault(binding_key(draw),[]).append(draw)
    output.mkdir(parents=True);(output/'prototypes').mkdir()
    ordinary_path=output/'ordinary-plan.json';ordinary_path.write_text(json.dumps(dict(plan,draws=ordinary),separators=(',',':'))+'\n')
    reference=verify_plan(source,ordinary_path);(output/'ordinary-reference.json').write_text(json.dumps(reference,indent=2)+'\n')
    variant_reference=verify_variants(source,plan_path);(output/'variant-reference.json').write_text(json.dumps(variant_reference,indent=2)+'\n')
    report={'schema':1,'plan_sha256':plan_hash,'source_plan':str(plan_path),'planned_draws':len(plan['draws']),'ordinary_draws':len(ordinary),'variant_draws':len(variants),'converted':[],'unresolved':[],
            'scope':'eligible static mesh/material bindings; explicit omissions and source motion/material limits; no full scene/collision/runtime adoption'}
    def save():
        if hashlib.sha256(plan_path.read_bytes()).hexdigest()!=plan_hash:raise ValueError('Source plan changed during conversion')
        (output/'batch.json').write_text(json.dumps(report,indent=2)+'\n')
    save()
    for index,(key,draws) in enumerate(sorted(groups.items())):
        identifier=hashlib.sha256(key.encode()).hexdigest();directory=output/'prototypes'/identifier
        record={'key':identifier,'mesh':draws[0]['mesh'],'materials':draws[0]['materials'],'draws':len(draws)}
        try:
            refs=draws[0]['materials']
            if not refs or any(r!=refs[0] for r in refs):raise ValueError('Distinct source material slots require separate conversion')
            material_path=sources.by_guid[refs[0]['guid']];material,=[v['fields'] for v in checked_records(sources,material_path).values() if v['kind']==21]
            shader_guid=material['m_Shader'].get('guid');shader_name=shaders.by_guid.get(shader_guid)
            standard=include_standard and shader_guid=='0000000000000000f000000000000000'
            if standard:
                standard_material(material);full=False
            else:
                if shader_name is None:raise ValueError('Shader source outside selected source stage')
                sha=hashlib.sha256(shaders.checked(shader_name).read_bytes()).hexdigest();shaders.metadata(shader_name)
                if shader_guid==SHADER_GUID:full_material(material,sha);full=True
                elif shader_guid==BASIC_SHADER_GUID:vegetation_material(material,sha);full=False
                else:raise ValueError('Source shader family requires its own adapter')
            # Choose an ordinary source witness; geometry/material bindings may
            # then be shared with every independently checked matching draw.
            candidates=[d for d in draws if d['instance'] is not None and not any(v['kind']==1001 for v in checked_records(sources,d['source']).values())]
            if not candidates and standard:candidates=[d for d in draws if d['instance'] is not None]
            if not candidates:raise ValueError('No qualified source witness for this binding')
            witness=candidates[0]
            print(f'CONVERT {index+1}/{len(groups)} {identifier} draws={len(draws)} source={witness["source"]}',flush=True)
            if standard:
                result=convert_standard(source,witness['source'],directory,False,True,
                           plan_path=plan_path,instance=witness['instance'],renderer=witness['renderer'])
                result=dict(result,asset_sha256=result['preview_sha256'],triangles=result['source_triangles'])
                checked=verify_standard(source,directory)
            else:
                result=convert(source,shader_source,witness['source'],directory,full_static=full,canonical_mesh=True,
                               plan_path=plan_path,instance=witness['instance'],renderer=witness['renderer'])
                checked=verify_foliage(source,shader_source,directory)
            (directory/'independent-verification.json').write_text(json.dumps(checked,indent=2)+'\n')
            geometry=canonical(source,directory,probe);(directory/'canonical-reference.json').write_text(json.dumps(geometry,indent=2)+'\n')
            instances=[{'id':str(d['instance'])+':'+str(d['renderer']),'matrix':d['matrix']} for d in draws]
            (directory/'instances.json').write_text(json.dumps({'instances':instances,'source_plan_sha256':plan_hash},separators=(',',':'))+'\n')
            placed=assemble(directory/'preview.glb',instances,directory/'placements.glb');oracle=assembly(directory/'preview.glb',instances,directory/'placements.glb')
            (directory/'assembly.json').write_text(json.dumps(placed,indent=2)+'\n');(directory/'assembly-reference.json').write_text(json.dumps(oracle,indent=2)+'\n')
            record.update(profile='standard-opaque' if standard else ('full-static' if full else 'basic-static'),source=witness['source'],instance=witness['instance'],renderer=witness['renderer'],triangles_per_instance=result['triangles'],
                          triangles=oracle['triangles'],prototype_sha256=result['asset_sha256'],placements_sha256=placed['asset_sha256'],expected_bounds=oracle['expected_bounds'])
            report['converted'].append(record)
        except (ValueError,KeyError,IndexError,TypeError) as error:
            record['reason']=str(error);record['exception']=type(error).__name__;report['unresolved'].append(record)
            if directory.exists():(directory/'rejected.json').write_text(json.dumps(record,indent=2)+'\n')
            print(f'UNRESOLVED {identifier} draws={len(draws)} {error}',flush=True)
        save()
    report['converted_draws']=sum(r['draws'] for r in report['converted']);report['unresolved_draws']=sum(r['draws'] for r in report['unresolved'])
    report['unresolved_reasons']=dict(Counter(r['reason'] for r in report['unresolved']));save();return report


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('source',type=Path);parser.add_argument('shaders',type=Path);parser.add_argument('plan',type=Path);parser.add_argument('output',type=Path);parser.add_argument('--probe',required=True,type=Path);parser.add_argument('--include-standard',action='store_true');args=parser.parse_args()
    result=batch(args.source,args.shaders,args.plan,args.output,args.probe,args.include_standard);print(json.dumps({k:v for k,v in result.items() if k not in ('converted','unresolved')}))
