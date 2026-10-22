"""Check actual Editor sky geometry and source world-height lookup independently."""
import argparse
import hashlib
import io
import json
import math
from pathlib import Path
import numpy as np
from PIL import Image
from unity_scene import Sources,records
from unity_editor_reference import inspect
from verify_unity_foliage import reference_materials
from verify_unity_mixed import properties
from verify_unity_geometry import assembly
from merge_static_glb import read
from static_geometry import accessor,hierarchy


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def verify(source,shaders,stage,probe):
    source=Sources(source);shaders=Sources(shaders);stage=Path(stage);meta=json.loads((stage/'sky.json').read_text());project=Path(meta['project'])
    if meta['archive_sha256']!=source.manifest['archive_sha256'] or meta['archive_sha256']!=shaders.manifest['archive_sha256'] or sha(stage/'preview.glb')!=meta['prototype_sha256'] or sha(stage/'placements.glb')!=meta['placements_sha256']:raise ValueError('Sky source/geometry binding differs')
    if sha(project/'Reference/request.json')!=meta['request_sha256'] or sha(project/'Reference/imported.json')!=meta['editor_report_sha256']:raise ValueError('Sky Editor request/report changed')
    checked=inspect(source.root,project,probe,{meta['model']})
    if not checked['metadata_byte_identical'] or checked['recorded_frame_matches']!=checked['meshes_checked']:raise ValueError('Sky source Editor frame remains unqualified')
    rr=records(source.checked(meta['prefab']));source.metadata(meta['prefab']);renderer=rr[meta['renderer']]['fields'];go=renderer['m_GameObject']['fileID'];mesh_ref,=[v['fields'] for v in rr.values() if v['kind']==33 and v['fields']['m_GameObject']['fileID']==go]
    if renderer['m_CastShadows']!=0 or renderer['m_ReceiveShadows']!=0 or mesh_ref['m_Mesh']!=meta['mesh'] or source.by_guid[meta['mesh']['guid']]!=meta['model'] or meta['mesh']['fileID']!=meta['mesh_fileID']:raise ValueError('Raw sky renderer/mesh flags differ')
    refs=reference_materials(source,meta,renderer,mesh_ref)
    if refs!=meta['materials'] or len(refs)!=1 or source.by_guid[refs[0]['guid']]!=meta['material']:raise ValueError('Raw sky material binding differs')
    fields,=[v['fields'] for v in records(source.checked(meta['material'])).values() if v['kind']==21];shader=fields['m_Shader']
    if shader!={'fileID':4800000,'guid':'b79e425cce5c90642a8dd66b6852353e','type':3} or sha(shaders.checked(shaders.by_guid[shader['guid']]))!='24694dc81dc4d3f6e60bb9974b13397e1a6920d25d56ec11784a8b3bb2ac165c':raise ValueError('Sky reference shader differs')
    if fields.get('m_ShaderKeywords') or fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue',-1) not in (-1,2000):raise ValueError('Sky reference variant/pass differs')
    saved=fields['m_SavedProperties'];f=properties(saved['m_Floats']);c=properties(saved['m_Colors']);t=properties(saved['m_TexEnvs'])
    if type(f['_UV_Based_Toggle']) not in (int,float) or f['_UV_Based_Toggle']!=0 or any(v['m_Texture'].get('fileID') for v in t.values()):raise ValueError('Unqualified UV/textured sky source')
    for name,low,high in [('_Offset',-4096,4096),('_Distance',.001,10000),('_Falloff',.001,100)]:
        if type(f[name]) not in (int,float) or not math.isfinite(f[name]) or not low<=f[name]<=high:raise ValueError('Invalid source sky scalar')
    colours={name:[c[key][ch] for ch in 'rgb'] for name,key in [('bottom','_ColorBottom'),('top','_ColorTop')]}
    if any(type(v) not in (int,float) or not math.isfinite(v) or not 0<=v<=1 for values in colours.values() for v in values):raise ValueError('Invalid source sky colours')
    profile=meta['profile']
    for key,value in {**colours,'offset':f['_Offset'],'distance':f['_Distance'],'falloff':f['_Falloff'],'lut_height':1024,'profile':'synty-world-height-unlit-sky-v1'}.items():
        if profile[key]!=value:raise ValueError('Sky profile differs from source parameters')
    imported=json.loads((project/'Reference/imported.json').read_text());model,=[m for m in imported['models'] if m['path']==meta['model']];mesh,=[m for m in model['meshes'] if m['file_id']==str(meta['mesh_fileID'])]
    doc,binary=read(stage/'preview.glb');_,world=hierarchy(doc)
    if len(doc['nodes'])!=1 or len(doc['meshes'])!=1 or not np.array_equal(world[0],np.eye(4)) or len(mesh['submeshes'])!=1 or mesh['submeshes'][0]['topology']!=0:raise ValueError('Select one actual Editor sky prototype/submesh')
    p,=doc['meshes'][0]['primitives'];points=np.array(mesh['positions'],dtype=np.float32).reshape(-1,3);points[:,0]*=-1
    indices=np.array(mesh['submeshes'][0]['indices']).reshape(-1,3)[:,[0,2,1]].ravel()
    channel=profile.get('uv_channel',1)
    if type(channel) is not int or channel not in (0,1):raise ValueError('Unknown sky UV channel')
    uv_attribute='TEXCOORD_'+str(channel)
    if set(p['attributes'])!={'POSITION',uv_attribute} or not np.array_equal(accessor(doc,binary,p['attributes']['POSITION']),points) or not np.array_equal(accessor(doc,binary,p['indices']).ravel(),indices):raise ValueError('Actual Editor sky positions/topology changed')
    binding=meta['scene_binding'];plan_path=Path(meta['source_plan'])
    if sha(plan_path)!=meta['source_plan_sha256'] or binding['plan_sha256']!=meta['source_plan_sha256']:raise ValueError('Sky source plan changed')
    reflection=np.diag([-1,1,1,1]);matrix=reflection@np.array(binding['world_matrix'],dtype=float)@reflection
    world_points=points.astype(float)@matrix[:3,:3].T+matrix[:3,3]
    mapping=profile.get('lut_mapping','centre-grid-v1')
    if mapping not in ('centre-grid-v1','endpoint-centres-v1'):raise ValueError('Unknown sky lookup mapping')
    v=1-(world_points[:,1]+f['_Offset'])/f['_Distance']
    if mapping=='endpoint-centres-v1':v=.5/1024+v*1023/1024
    expected_uv=np.c_[np.full(len(points),.5),v].astype(np.float32)
    if not np.array_equal(accessor(doc,binary,p['attributes'][uv_attribute]),expected_uv):raise ValueError('Sky world-height UV coordinates differ')
    m,=doc['materials'];expected_material={'name':'source-world-height-sky','extensions':{'KHR_materials_unlit':{}},'extras':{'theia_render_flags':{'cast_shadows':False,'receive_shadows':False}},
        'pbrMetallicRoughness':{'baseColorFactor':[1,1,1,1],'baseColorTexture':{'index':0,'texCoord':channel},'metallicFactor':0,'roughnessFactor':1},'doubleSided':False,'alphaMode':'OPAQUE'}
    if m!=expected_material or doc.get('extensionsUsed')!=['KHR_materials_unlit'] or doc.get('extensionsRequired')!=['KHR_materials_unlit'] or doc['samplers']!=[{'wrapS':33071,'wrapT':33071,'magFilter':9729,'minFilter':9729}] or doc['textures']!=[{'source':0,'sampler':0}]:raise ValueError('Sky unlit/material/channel/render/sampler state differs')
    image,=doc['images'];view=doc['bufferViews'][image['bufferView']];start=view.get('byteOffset',0);data=binary[start:start+view['byteLength']]
    if hashlib.sha256(data).hexdigest()!=meta['lut_sha256']:raise ValueError('Sky lookup image binding differs')
    with Image.open(io.BytesIO(data)) as image:
        if image.size!=(2,1024) or image.format!='PNG':raise ValueError('Sky lookup image dimensions/format differ')
        pixels=np.array(image.convert('RGBA'))
    for row in range(1024):
        weight=((1023-row)/1023 if mapping=='endpoint-centres-v1' else (1024-row-.5)/1024)**f['_Falloff'];expected=[]
        for a,b in zip(colours['bottom'],colours['top']):
            linear=a+(b-a)*weight;encoded=linear*12.92 if linear<=.0031308 else 1.055*linear**(1/2.4)-.055;expected.append(round(encoded*255))
        expected.append(255)
        if not np.array_equal(pixels[row],np.array([expected,expected],dtype=np.uint8)):raise ValueError('Sky lookup source colour function differs')
    instance={'id':str(binding['instance'])+':'+str(binding['renderer']),'matrix':binding['world_matrix']};placed=assembly(stage/'preview.glb',[instance],stage/'placements.glb')
    if placed['asset_sha256']!=meta['placements_sha256']:raise ValueError('Sky exact source placement differs')
    key=hashlib.sha256(json.dumps([meta['mesh'],meta['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest()
    if key!=meta['key']:raise ValueError('Sky source binding key differs')
    return {'passed':True,'key':key,'draws':1,'source_plan_sha256':meta['source_plan_sha256'],'asset_sha256':meta['prototype_sha256'],'triangles':len(indices)//3,
            'expected_bounds':[points.min(0).tolist(),points.max(0).tolist()],'placements':placed,
            'scope':'actual Editor mesh positions/indices; source renderer/material/scene/world-height UVs; exact baked source gradient texels and source pose; native unlit/shadow behavior separate'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['source','shaders','stage','probe']:p.add_argument(name,type=Path)
    p.add_argument('--report',type=Path,required=True);a=p.parse_args()
    if a.report.exists():raise FileExistsError('Use a fresh report')
    r=verify(a.source,a.shaders,a.stage,a.probe);a.report.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
