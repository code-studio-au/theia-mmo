"""Independent raw-source checks for a bounded triplanar material prototype."""
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
from merge_static_glb import read
from static_geometry import accessor,hierarchy


def verify(source,shader_source,stage):
    source=Sources(source);shaders=Sources(shader_source);stage=Path(stage);c=json.loads((stage/'conversion.json').read_text());settings=json.loads((stage/'triplanar.json').read_text())
    digest=hashlib.sha256((stage/'preview.glb').read_bytes()).hexdigest()
    if digest!=c['preview_sha256'] or digest!=settings['geometry_sha256'] or c['archive_sha256']!=source.manifest['archive_sha256'] or source.manifest['archive_sha256']!=shaders.manifest['archive_sha256']:raise ValueError('Triplanar source/geometry binding differs')
    rr=records(source.checked(c['prefab']));renderer=rr[c['renderer']]['fields'];go=renderer['m_GameObject']['fileID'];mesh,=[v['fields'] for v in rr.values() if v['kind']==33 and v['fields']['m_GameObject']['fileID']==go]
    refs=reference_materials(source,c,renderer,mesh)
    if any(r!=refs[0] for r in refs) or source.by_guid[refs[0]['guid']]!=c['material'] or mesh['m_Mesh']['fileID']!=c['mesh_fileID'] or source.by_guid[mesh['m_Mesh']['guid']]!=c['model']:raise ValueError('Triplanar raw draw binding differs')
    fields,=[v['fields'] for v in records(source.checked(c['material'])).values() if v['kind']==21];shader=fields['m_Shader']
    if shader!={'fileID':4800000,'guid':'66f8401a4c881134f8803eceae302347','type':3}:raise ValueError('Reference triplanar shader differs')
    shader_file=shaders.checked(shaders.by_guid[shader['guid']])
    if hashlib.sha256(shader_file.read_bytes()).hexdigest()!='d478ba3c6b3476570529dee5f65dc725b07a5bc463ea05bec10757e1209fe118':raise ValueError('Reference triplanar source hash differs')
    if fields.get('m_ShaderKeywords') or fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue',-1) not in (-1,2000):raise ValueError('Unqualified reference triplanar variant')
    saved=fields['m_SavedProperties'];floats={k:v for row in saved['m_Floats'] for k,v in row.items()};textures={k:v for row in saved['m_TexEnvs'] for k,v in row.items()}
    for key,source_key,limit in [('falloff','_FallOff',100),('tiling','_Tiling',10)]:
        value=settings[key]
        if not isinstance(value,(float,int)) or isinstance(value,bool) or not math.isfinite(value) or not 0<value<=limit or value!=floats[source_key]:raise ValueError('Reference triplanar scalar differs')
    doc,binary=read(stage/'preview.glb');_,world=hierarchy(doc);ref=json.loads((stage/'geometry-reference.json').read_text());points=np.array(ref['vertices_z_up_metres'])[:,[0,2,1]];points[:,2]*=-1
    uv=np.array(ref['triangle_uv_blender']);uv[:,:,1]=1-uv[:,:,1];normals=np.array(ref['triangle_normals_z_up'])[:,:,[0,2,1]];normals[:,:,2]*=-1;lengths=np.linalg.norm(normals,axis=2)
    if not np.isfinite(normals).all() or (lengths<1e-12).any():raise ValueError('Invalid triplanar source normal')
    normals/=lengths[:,:,None];expected=np.concatenate([points[np.array(ref['triangles'])],uv,normals],2);actual=[]
    for index,frame in world.items():
        if 'mesh' not in doc['nodes'][index]:continue
        for p in doc['meshes'][doc['nodes'][index]['mesh']]['primitives']:
            ids=accessor(doc,binary,p['indices']).reshape(-1,3);pos=accessor(doc,binary,p['attributes']['POSITION'])@frame[:3,:3].T+frame[:3,3];tex=accessor(doc,binary,p['attributes']['TEXCOORD_0']);n=accessor(doc,binary,p['attributes']['NORMAL'])@np.linalg.inv(frame[:3,:3]);actual.extend(np.concatenate([pos[ids],tex[ids],n[ids]],2))
            t=accessor(doc,binary,p['attributes']['TANGENT'])
            if not np.isfinite(t).all() or (np.abs(np.linalg.norm(t[:,:3],axis=1)-1)>2e-4).any() or not np.isin(t[:,3],[-1,1]).all():raise ValueError('Triplanar requires valid exported tangent basis')
    difference=normal_error(expected,actual);material,=doc['materials'];pbr=material['pbrMetallicRoughness']
    if material.get('extras')!={'theia_triplanar':1} or material.get('alphaMode','OPAQUE')!='OPAQUE' or material.get('doubleSided',False) or pbr.get('metallicFactor',1)!=0 or pbr.get('roughnessFactor',1)!=1 or material.get('normalTexture') or material.get('emissiveTexture'):raise ValueError('Triplanar geometry placeholder state differs')
    dependencies={}
    recorded=None
    if 'texture_reference' in settings:
        from unity_texture_reference import inspect as inspect_textures
        binding=settings['texture_reference'];recorded=inspect_textures(source.root,binding['project'])
        if binding!={key:recorded[key] for key in ['project','request_sha256','report_sha256','collector_sha256','unity_version']}:
            raise ValueError('Reference effective texture evidence changed')
    for role,key in [('top','_Top'),('side','_Sides'),('top_normal','_TopNormal'),('side_normal','_SidesNormal')]:
        name=source.by_guid[textures[key]['m_Texture']['guid']];original=source.checked(name);meta=source.metadata(name)['TextureImporter'];binding=settings['textures'][role]
        if binding['file']!='triplanar-'+role+'.png' or binding['source']!=name or binding['sha256']!=hashlib.sha256(original.read_bytes()).hexdigest() or (stage/binding['file']).read_bytes()!=original.read_bytes():raise ValueError('Triplanar external dependency differs')
        sampler=meta['textureSettings'];is_normal=role.endswith('normal')
        effective=[sampler.get(k) for k in ['filterMode','wrapU','wrapV','aniso','mipBias']]
        if sampler==dict(filterMode=-1,aniso=-1,mipBias=-1,wrapMode=-1) and recorded is not None:
            item,=[v for v in recorded['textures'] if v['path']==name]
            if item['importer_type']!=int(is_normal) or item['srgb']!=(not is_normal) or item['convert_normal']:
                raise ValueError('Reference actual texture type/colour policy differs')
            effective=[item[k] for k in ['filter','wrap_u','wrap_v','aniso','mip_bias']]
        if meta['textureType']!=(1 if is_normal else 0) or meta['mipmaps']['sRGBTexture']!=(0 if is_normal else 1) or effective!=[1,0,0,1,0] or meta.get('bumpmap',{}).get('convertToNormalMap'):raise ValueError('Reference triplanar importer/sampler differs')
        with Image.open(original) as image:
            if max(image.size)>4096:raise ValueError('Triplanar image budget exceeded')
            dependencies[role]={'source':name,'sha256':binding['sha256'],'dimensions':list(image.size),'srgb':not is_normal}
    index=pbr['baseColorTexture']['index'];image=doc['images'][doc['textures'][index]['source']];view=doc['bufferViews'][image['bufferView']];start=view.get('byteOffset',0)
    with Image.open(io.BytesIO(binary[start:start+view['byteLength']])) as embedded,Image.open(source.checked(dependencies['side']['source'])) as original:
        if embedded.convert('RGBA').tobytes()!=original.convert('RGBA').tobytes() or embedded.size!=original.size:raise ValueError('Fallback side texture changed')
    return {'passed':True,'triangles':len(actual),'mesh_normal_error':difference,'expected_bounds':[points.min(0).tolist(),points.max(0).tolist()],'asset_sha256':digest,'dependencies':dependencies,'scope':'raw source binding, geometry/UV/normal/tangent and exact texture bytes; shader sampling/GPU/Unity parity separate'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('source',type=Path);p.add_argument('shader_source',type=Path);p.add_argument('stage',type=Path);a=p.parse_args();result=verify(a.source,a.shader_source,a.stage);(a.stage/'independent-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
