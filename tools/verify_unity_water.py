"""Independent raw-source, Editor triangle and static water adaptation checks."""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
from unity_scene import Sources, records
from unity_editor_reference import inspect
from unity_builtin_plane_reference import inspect as inspect_plane
from verify_unity_mixed import properties
from verify_unity_prefabs import verify as verify_poses
from verify_unity_geometry import assembly
from merge_static_glb import read
from static_geometry import accessor, hierarchy


def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def verify(source, shaders, stage, probe):
    source=Sources(source); shaders=Sources(shaders); stage=Path(stage); meta=json.loads((stage/'water.json').read_text()); project=Path(meta['project'])
    if meta['schema']!=1 or meta['archive_sha256']!=source.manifest['archive_sha256'] or meta['archive_sha256']!=shaders.manifest['archive_sha256']:
        raise ValueError('Water source identity differs')
    if sha(stage/'preview.glb')!=meta['prototype_sha256'] or sha(stage/'placements.glb')!=meta['placements_sha256'] or sha(meta['source_plan'])!=meta['source_plan_sha256']:
        raise ValueError('Water artifact/source plan binding differs')
    if sha(project/'Reference/request.json')!=meta['request_sha256']: raise ValueError('Water Editor request differs')
    plan=json.loads(Path(meta['source_plan']).read_text()); subset=json.loads((stage/'source-subset.json').read_text())
    if plan['archive_sha256']!=source.manifest['archive_sha256']: raise ValueError('Water plan archive differs')
    witness,=[d for d in plan['draws'] if (d['instance'],d['renderer'])==(meta['instance'],meta['renderer'])]
    selected=[d for d in plan['draws'] if [d['mesh'],d['materials']]==[witness['mesh'],witness['materials']]]
    key=hashlib.sha256(json.dumps([witness['mesh'],witness['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest()
    if key!=meta['key'] or meta['mesh']!=witness['mesh'] or meta['materials']!=witness['materials'] or meta['draws']!=len(selected) or subset!=dict(plan,draws=selected):
        raise ValueError('Water exact source group differs')
    poses=verify_poses(source.root,stage/'source-subset.json')
    scene=records(source.checked(plan['scene'])); source.metadata(plan['scene'])
    flags=set()
    for d in selected:
        rr=scene if d['instance'] is None else records(source.checked(d['source'])); source.metadata(d['source']); f=rr[d['renderer']]['fields']; cast,receive=f['m_CastShadows'],f['m_ReceiveShadows']
        if d['instance'] is not None:
            guid=source.manifest['selected'][d['source']]['guid']
            modifications=scene[d['instance']]['fields']['m_Modification']
            if any(v for k,v in modifications.items() if k.startswith(('m_Removed','m_Added'))): raise ValueError('Water structural override unsupported')
            for m in modifications['m_Modifications']:
                if m['target'].get('guid')==guid and m['target']['fileID']==d['renderer']:
                    if m['propertyPath']=='m_CastShadows':cast=int(m['value'])
                    if m['propertyPath']=='m_ReceiveShadows':receive=int(m['value'])
        if cast not in (0,1) or receive not in (0,1): raise ValueError('Unqualified source shadow state')
        flags.add((bool(cast),bool(receive)))
    cast,receive=flags.pop()
    if flags or type(meta['cast_shadows']) is not bool or type(meta['receive_shadows']) is not bool or (meta['cast_shadows'],meta['receive_shadows'])!=(cast,receive):
        raise ValueError('Water group source render flags differ')
    name=source.by_guid[witness['materials'][0]['guid']]; source.metadata(name)
    fields,=[v['fields'] for v in records(source.checked(name)).values() if v['kind']==21]
    expected_shaders={'a14ab9faa19eaeb4b8b41c78635b23bd':'ab01bd729b9d7e76281573b27b72e22bfec1ca5e72047c554e7f7b3b4c4f702b',
                      '276a39a5a210b124885f0eff3e80f1f3':'d1798bf9cc2e6e4622cb1f6812c08f951778bcc380b8f0d0565fa37c0c7da6b8'}
    guid=fields['m_Shader']['guid']; shader_name=shaders.by_guid[guid]; shaders.metadata(shader_name)
    if guid not in expected_shaders or sha(shaders.checked(shader_name))!=expected_shaders[guid] or fields['m_Shader']!={'fileID':4800000,'guid':guid,'type':3}: raise ValueError('Water shader source differs')
    if fields.get('m_ShaderKeywords') not in (None,'','_GLOSSYREFLECTIONS_OFF _SPECULARHIGHLIGHTS_OFF') or fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue',-1)!=-1: raise ValueError('Water shader variant differs')
    saved=fields['m_SavedProperties']; f=properties(saved['m_Floats']); c=properties(saved['m_Colors']); profile=meta['profile']
    if f.get('_WaveAmplitude',0)!=0 or f['_OpacityFalloff']!=1: raise ValueError('Water geometry/opacity profile is unqualified')
    shallow=[c['_ShallowColour'][axis] for axis in 'rgb']; deep=[c['_DeepColour'][axis] for axis in 'rgb']
    # At normalized depth one: pow(1, overallFalloff)=1, very-deep weight=0.
    t=1+f['_ShallowFalloff']; tint=[(1-t)*a+t*b for a,b in zip(shallow,deep)] if t<1 else deep
    rgba=tint+[f['_Opacity']]; roughness=1-max(0,min(1,f['_Smoothness']*f['_ReflectionPower']))
    if profile['profile']!='synty-unit-depth-static-water-v1' or profile['shader_sha256']!=expected_shaders[guid] or profile['source_properties']!=saved or profile['unapplied_textures']!=properties(saved['m_TexEnvs']): raise ValueError('Water source property record differs')
    if not np.allclose(profile['base_color_factor'],rgba,atol=1e-15,rtol=0) or abs(profile['roughness']-roughness)>1e-15 or profile['metallic']!=0: raise ValueError('Water adaptation record differs')
    doc,binary=read(stage/'preview.glb'); hierarchy(doc)
    m,=doc['materials']; expected={'name':'source-static-water','pbrMetallicRoughness':{'baseColorFactor':profile['base_color_factor'],'metallicFactor':0,'roughnessFactor':roughness},'alphaMode':'BLEND','doubleSided':False,'extras':{'theia_render_flags':{'cast_shadows':cast,'receive_shadows':receive}}}
    if m!=expected or any(doc.get(k) for k in ['images','textures','samplers','extensionsUsed','extensionsRequired']): raise ValueError('Water material/dependencies differ')
    if doc['scenes']!=[{'nodes':[0]}] or doc['nodes']!=[{'mesh':0}] or len(doc['meshes'])!=1: raise ValueError('Water prototype hierarchy differs')
    p,=doc['meshes'][0]['primitives']
    if set(p['attributes'])!={'POSITION','NORMAL'} or p.get('material')!=0 or p.get('mode',4)!=4: raise ValueError('Water primitive attributes differ')
    if witness['mesh']['guid']=='0000000000000000e000000000000000':
        if witness['mesh']!={'fileID':10209,'guid':'0000000000000000e000000000000000','type':0} or meta['model'] is not None: raise ValueError('Water built-in identity differs')
        reference=inspect_plane(project); report_path=project/'Reference/plane.json'; mesh=json.loads(report_path.read_text()); triangles=mesh['indices']
    else:
        model=source.by_guid[witness['mesh']['guid']]
        if meta['model']!=model: raise ValueError('Water FBX identity differs')
        reference=inspect(source.root,project,probe,{model}); report_path=project/'Reference/imported.json'
        if not reference['metadata_byte_identical'] or reference['recorded_frame_matches']!=reference['meshes_checked']: raise ValueError('Water source Editor frame remains unqualified')
        imported=json.loads(report_path.read_text()); model,=[m for m in imported['models'] if m['path']==model]; mesh,=[m for m in model['meshes'] if m['file_id']==str(witness['mesh']['fileID'])]; sub,=mesh['submeshes']
        if sub['topology']!=0: raise ValueError('Water source topology differs')
        triangles=sub['indices']
    if sha(report_path)!=meta['editor_report_sha256']: raise ValueError('Water Editor report binding differs')
    positions=np.array(mesh['positions'],dtype=np.float32).reshape(-1,3);positions[:,0]*=-1
    triangles=np.array(triangles,dtype=int).reshape(-1,3)[:,[0,2,1]]; wanted=positions[triangles].reshape(-1,3)
    actual=accessor(doc,binary,p['attributes']['POSITION']); ii=accessor(doc,binary,p['indices']).ravel()
    if not np.array_equal(actual,wanted) or not np.array_equal(ii,np.arange(len(wanted))): raise ValueError('Water Editor corner topology/winding differs')
    normals=accessor(doc,binary,p['attributes']['NORMAL']).astype(float); tri=wanted.astype(float).reshape(-1,3,3)
    cross=np.cross(tri[:,1]-tri[:,0],tri[:,2]-tri[:,0]); lengths=np.linalg.norm(cross,axis=1)
    if (lengths<1e-12).any() or not np.allclose(normals,np.repeat(cross/lengths[:,None],3,axis=0),atol=1e-7,rtol=0): raise ValueError('Water derived flat normals differ')
    instances=[{'id':str(d['instance'])+':'+str(d['renderer']),'matrix':d['matrix']} for d in selected]
    placed=assembly(stage/'preview.glb',instances,stage/'placements.glb')
    return {'passed':True,'key':key,'draws':len(selected),'source_plan_sha256':meta['source_plan_sha256'],'asset_sha256':meta['prototype_sha256'],
        'triangles':len(tri),'expected_bounds':[wanted.min(0).tolist(),wanted.max(0).tolist()],'source_poses':poses,'placements':placed,
        'scope':'actual Editor corner topology/winding, independent raw source slots/poses/shadow flags and explicit static material factors; geometry-derived flat normals; no source water shader/normal/UV/collision parity'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['source','shaders','stage','probe']:p.add_argument(name,type=Path)
    p.add_argument('--report',type=Path,required=True);a=p.parse_args()
    if a.report.exists():raise FileExistsError('Use a fresh report')
    result=verify(a.source,a.shaders,a.stage,a.probe);a.report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
