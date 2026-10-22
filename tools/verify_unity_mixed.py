"""Independently check raw two-slot sources, geometry, PBR factors and PNGs."""
import argparse
import hashlib
import io
import json
import math
from pathlib import Path
import numpy as np
from PIL import Image
from unity_scene import Sources,records
from verify_unity_foliage import reference_materials,normal_error
from verify_unity_static import standard_source
from merge_static_glb import read
from static_geometry import accessor,hierarchy


def properties(rows):
    result={}
    for row in rows:
        if not isinstance(row,dict) or len(row)!=1:raise ValueError('Malformed mixed reference property')
        key,value=next(iter(row.items()))
        if key in result:raise ValueError('Duplicate mixed reference property')
        result[key]=value
    return result


def crystal_reference(fields,shaders):
    ref=fields['m_Shader']
    if ref!={'fileID':4800000,'guid':'006c4dbb387622342a1397597c7b3e66','type':3}:raise ValueError('Reference crystal shader differs')
    path=shaders.checked(shaders.by_guid[ref['guid']]);shaders.metadata(shaders.by_guid[ref['guid']])
    if hashlib.sha256(path.read_bytes()).hexdigest()!='ca667b1dc83a0f02e1059694197ff954ee2b065b0e8c78601f35a42aad2d6aa7':raise ValueError('Reference crystal shader bytes differ')
    if fields.get('m_ShaderKeywords') or fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue',-1) not in (-1,3000):raise ValueError('Reference crystal variant differs')
    saved=fields['m_SavedProperties'];floats=properties(saved['m_Floats']);colours=properties(saved['m_Colors']);properties(saved['m_TexEnvs'])
    for key,low,high in [('_ColourFalloff',.0001,100),('_Opacity',0,1),('_Metallic',0,1),('_Smoothness',0,1),('_Refraction',0,1),('_ChromaticAberration',0,.3)]:
        value=floats[key]
        if type(value) not in (int,float) or not math.isfinite(value) or not low<=value<=high:raise ValueError('Reference crystal scalar differs')
    if not 0<floats['_Opacity']<1:raise ValueError('Reference crystal opacity is not partial')
    for key in ['_ShallowColor','_DeepColor','_Emission']:
        if any(type(colours[key][c]) not in (int,float) or not math.isfinite(colours[key][c]) or not 0<=colours[key][c]<=1 for c in 'rgb'):raise ValueError('Reference crystal colour/HDR emission differs')
    t=max(0,min(1,1/floats['_ColourFalloff']))
    colour=[colours['_ShallowColor'][c]+(colours['_DeepColor'][c]-colours['_ShallowColor'][c])*t for c in 'rgb']
    return colour+[floats['_Opacity']],floats['_Metallic'],1-floats['_Smoothness'],[colours['_Emission'][c] for c in 'rgb']


def verify(source,shaders,stage):
    source=Sources(source);shaders=Sources(shaders);stage=Path(stage)
    c=json.loads((stage/'conversion.json').read_text());asset=stage/'preview.glb';digest=hashlib.sha256(asset.read_bytes()).hexdigest()
    if c['profile']!='unity-mixed-crystal-pbr-static-v1' or c['archive_sha256']!=source.manifest['archive_sha256'] or c['archive_sha256']!=shaders.manifest['archive_sha256'] or c['preview_sha256']!=digest:raise ValueError('Mixed source/profile/asset binding differs')
    rr=records(source.checked(c['prefab']));source.metadata(c['prefab']);renderer=rr[c['renderer']]['fields'];go=renderer['m_GameObject']['fileID']
    mesh,=[v['fields'] for v in rr.values() if v['kind']==33 and v['fields']['m_GameObject']['fileID']==go]
    if source.by_guid[mesh['m_Mesh']['guid']]!=c['model'] or mesh['m_Mesh']['fileID']!=c['mesh_fileID']:raise ValueError('Mixed raw model/mesh identity differs')
    refs=reference_materials(source,c,renderer,mesh)
    if len(refs)!=2 or refs[0]==refs[1] or refs!=c['material_refs']:raise ValueError('Mixed raw source slots differ')
    doc,binary=read(asset);_,world=hierarchy(doc);materials={}
    for m in doc['materials']:
        slot=m.get('extras',{}).get('theia_source_slot')
        if type(slot) is not int or slot not in (0,1) or slot in materials or m['extras']!={'theia_source_slot':slot}:raise ValueError('Mixed output material slot mapping differs')
        materials[slot]=m
    if set(materials)!={0,1}:raise ValueError('Missing mixed output material slot')
    used_images=set();families=[];dependencies=[]
    for slot,ref in enumerate(refs):
        name=source.by_guid[ref['guid']];source.metadata(name);fields,=[v['fields'] for v in records(source.checked(name)).values() if v['kind']==21]
        if c['material_plans'][slot]['source_material']!=name:raise ValueError('Mixed source material provenance differs')
        m=materials[slot];pbr=m['pbrMetallicRoughness'];shader=fields['m_Shader'];saved=fields['m_SavedProperties']
        if shader.get('guid')=='006c4dbb387622342a1397597c7b3e66':
            colour,metal,rough,emission=crystal_reference(fields,shaders);family='crystal'
            if 'baseColorTexture' in pbr or m.get('alphaMode')!='BLEND' or not m.get('doubleSided'):raise ValueError('Crystal preview blend/culling/texture differs')
        else:
            standard_source(fields);family='standard';floats=properties(saved['m_Floats']);colours=properties(saved['m_Colors']);textures=properties(saved['m_TexEnvs'])
            colour=[1,1,1,1];metal=floats['_Metallic'];rough=1-floats['_Glossiness'];emission=[colours['_EmissionColor'][c] for c in 'rgb'] if '_EMISSION' in (fields.get('m_ShaderKeywords') or '').split() else [0,0,0]
            if m.get('alphaMode','OPAQUE')!='OPAQUE' or m.get('doubleSided',False):raise ValueError('Mixed Standard alpha/culling differs')
            slots=[('_MainTex',pbr['baseColorTexture'])]
            if any(emission) and textures.get('_EmissionMap',{}).get('m_Texture',{}).get('fileID'):slots.append(('_EmissionMap',m['emissiveTexture']))
            elif 'emissiveTexture' in m:raise ValueError('Unexpected mixed Standard emission texture')
            for key,binding in slots:
                if binding.get('texCoord',0)!=0 or binding.get('extensions'):raise ValueError('Mixed Standard texture coordinates differ')
                texture=doc['textures'][binding['index']];sampler=doc['samplers'][texture['sampler']]
                if [sampler.get(k,v) for k,v in [('magFilter',9729),('minFilter',9987),('wrapS',10497),('wrapT',10497)]]!=[9729,9985,10497,10497]:raise ValueError('Mixed Standard sampler differs')
                image=doc['images'][texture['source']];used_images.add(texture['source']);view=doc['bufferViews'][image['bufferView']];offset=view.get('byteOffset',0)
                source_name=source.by_guid[textures[key]['m_Texture']['guid']];original=source.checked(source_name);meta=source.metadata(source_name)['TextureImporter']
                if meta['textureType']!=0 or meta['mipmaps']['sRGBTexture']!=1 or meta['mipmaps'].get('enableMipMap')!=1 or [meta['textureSettings'].get(k) for k in ['filterMode','wrapU','wrapV','aniso','mipBias']]!=[1,0,0,1,0]:raise ValueError('Mixed raw image importer/sampler differs')
                dependencies.append({'slot':slot,'role':'base_color' if key=='_MainTex' else 'emissive','source':source_name,'sha256':hashlib.sha256(original.read_bytes()).hexdigest()})
                with Image.open(original) as before,Image.open(io.BytesIO(binary[offset:offset+view['byteLength']])) as after:
                    if before.size!=after.size or before.convert('RGBA').tobytes()!=after.convert('RGBA').tobytes():raise ValueError('Mixed Standard source image pixels differ')
        families.append(family)
        if any(k in m for k in ['normalTexture','occlusionTexture','extensions','alphaCutoff']) or 'metallicRoughnessTexture' in pbr or (family=='crystal' and 'emissiveTexture' in m):raise ValueError('Unexpected mixed material feature')
        if any(not np.allclose(a,b,atol=2e-7,rtol=0) for a,b in [(pbr.get('baseColorFactor',[1,1,1,1]),colour),(pbr.get('metallicFactor',1),metal),(pbr.get('roughnessFactor',1),rough),(m.get('emissiveFactor',[0,0,0]),emission)]):raise ValueError('Mixed source material factors differ')
    if sorted(families)!=['crystal','standard'] or used_images!=set(range(len(doc.get('images',[])))):raise ValueError('Mixed family/image graph differs')
    if dependencies!=c['dependencies']:raise ValueError('Mixed source dependency provenance differs')
    reference=json.loads((stage/'geometry-reference.json').read_text());points=np.array(reference['vertices_z_up_metres'])[:,[0,2,1]];points[:,2]*=-1
    triangles=points[np.array(reference['triangles'])];uv=np.array(reference['triangle_uv_blender']);uv[:,:,1]=1-uv[:,:,1]
    normals=np.array(reference['triangle_normals_z_up'])[:,:,[0,2,1]];normals[:,:,2]*=-1;normals/=np.linalg.norm(normals,axis=2)[:,:,None]
    packets=np.concatenate([triangles,uv,normals],axis=2);expected={i:[] for i in (0,1)};actual={i:[] for i in (0,1)}
    if len(reference['triangle_slots'])!=len(packets):raise ValueError('Incomplete source mixed triangle slots')
    for packet,slot in zip(packets,reference['triangle_slots']):
        if type(slot) is not int or slot not in expected:raise ValueError('Invalid source triangle material slot')
        expected[slot].append(packet)
    for node,frame in world.items():
        if 'mesh' not in doc['nodes'][node]:continue
        for p in doc['meshes'][doc['nodes'][node]['mesh']]['primitives']:
            slot=doc['materials'][p['material']]['extras']['theia_source_slot'];ids=accessor(doc,binary,p['indices']).reshape(-1,3)
            pos=accessor(doc,binary,p['attributes']['POSITION'])@frame[:3,:3].T+frame[:3,3];uv0=accessor(doc,binary,p['attributes']['TEXCOORD_0']);normal=accessor(doc,binary,p['attributes']['NORMAL'])@np.linalg.inv(frame[:3,:3])
            actual[slot].extend(np.concatenate([pos[ids],uv0[ids],normal[ids]],axis=2))
    errors={i:normal_error(expected[i],actual[i]) for i in expected}
    return {'passed':True,'asset_sha256':digest,'triangles':len(triangles),'triangles_by_slot':{str(i):len(actual[i]) for i in actual},'normal_error_by_slot':errors,
            'expected_bounds':[points.min(0).tolist(),points.max(0).tolist()],
            'scope':'raw source renderer/slots and imported geometry/UV/normal/material/PBR/image adaptation; actual Editor submesh assignment is a separate gate; no source refraction/view-dependent shader parity'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['source','shaders','stage']:p.add_argument(name,type=Path)
    p.add_argument('--report',type=Path,required=True);a=p.parse_args()
    if a.report.exists():raise FileExistsError('Use a fresh report')
    r=verify(a.source,a.shaders,a.stage);a.report.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
