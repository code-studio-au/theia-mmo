#!/usr/bin/env python3
"""Independently compare static GLB triangle winding, geometry, UVs and palette."""
import argparse
from collections import Counter
import hashlib
import io
import json
import math
from pathlib import Path
import numpy as np
from PIL import Image
from static_geometry import accessor, hierarchy
from unity_scene import Sources, records
from verify_synty_character import read
from verify_unity_foliage import reference_materials,normal_error
from unity_model_ids import identity,bound_candidates


def signatures(triangles):
    result = Counter()
    for triangle in triangles:
        rows = [tuple(row) for row in np.rint(np.asarray(triangle)*100000).astype(np.int64)]
        # Cyclic order is interchangeable; reversing winding is not.
        key = min(tuple(rows[n:]+rows[:n]) for n in range(3))
        result[key] += 1
    return result



def standard_source(fields):
    """Independent eligibility check; never trust a converter's profile label."""
    shader=fields['m_Shader'];saved=fields['m_SavedProperties']
    if shader.get('fileID')!=46 or shader.get('guid')!='0000000000000000f000000000000000':raise ValueError('Reference Standard shader differs')
    def entries(rows):
        result={}
        for row in rows:
            if not isinstance(row,dict) or len(row)!=1:raise ValueError('Malformed reference material property')
            key,value=next(iter(row.items()))
            if key in result:raise ValueError('Duplicate reference material property')
            result[key]=value
        return result
    floats=entries(saved['m_Floats']);colors=entries(saved['m_Colors']);textures=entries(saved['m_TexEnvs'])
    keywords=(fields.get('m_ShaderKeywords') or '').split()
    if set(keywords)-{'_EMISSION'} or fields.get('disabledShaderPasses') or fields.get('m_DoubleSidedGI',0) or fields.get('m_CustomRenderQueue',-1) not in (-1,2000):raise ValueError('Reference Standard variant/pass differs')
    for key,wanted in [('_Mode',0),('_SrcBlend',1),('_DstBlend',0),('_ZWrite',1),('_SpecularHighlights',1),('_GlossyReflections',1),('_SmoothnessTextureChannel',0)]:
        if floats.get(key,wanted)!=wanted:raise ValueError('Unsupported reference Standard state')
    if colors.get('_Color')!=dict(r=1,g=1,b=1,a=1):raise ValueError('Unsupported reference Standard tint')
    for key in ['_Metallic','_Glossiness']:
        value=floats[key]
        if isinstance(value,bool) or not isinstance(value,(int,float)) or not math.isfinite(value) or not 0<=value<=1:raise ValueError('Invalid reference Standard scalar')
    if '_EMISSION' in keywords and colors['_EmissionColor'] not in (dict(r=1,g=1,b=1,a=1),dict(r=0,g=0,b=0,a=1)):raise ValueError('Unsupported reference emission colour')
    if not textures.get('_MainTex',{}).get('m_Texture',{}).get('fileID'):raise ValueError('Missing reference Standard base texture')
    for key,value in textures.items():
        if not value['m_Texture'].get('fileID'):continue
        if key not in ('_MainTex','_EmissionMap') or value['m_Texture']['fileID']!=2800000 or value['m_Scale']!=dict(x=1,y=1) or value['m_Offset']!=dict(x=0,y=0):raise ValueError('Unsupported reference Standard texture')
    return True


def verify_glass(fields,doc,material):
    """Raw source eligibility and portable blend state, without author imports."""
    saved=fields['m_SavedProperties'];shader=fields['m_Shader']
    values=[]
    for key in ['m_Floats','m_Colors','m_TexEnvs']:
        result={}
        for row in saved[key]:
            if len(row)!=1:raise ValueError('Malformed reference glass property')
            name,value=next(iter(row.items()))
            if name in result:raise ValueError('Duplicate reference glass property')
            result[name]=value
        values.append(result)
    floats,colours,textures=values
    if shader!={'fileID':46,'guid':'0000000000000000f000000000000000','type':0} or fields.get('m_ShaderKeywords')!='_ALPHAPREMULTIPLY_ON' or fields.get('m_CustomRenderQueue')!=3000 or fields.get('disabledShaderPasses') or fields.get('m_DoubleSidedGI',0):raise ValueError('Reference glass source family/pass differs')
    for key,wanted in [('_Mode',3),('_SrcBlend',1),('_DstBlend',10),('_ZWrite',0),('_SpecularHighlights',1),('_GlossyReflections',1),('_SmoothnessTextureChannel',0)]:
        if floats.get(key)!=wanted:raise ValueError('Reference glass source state differs')
    colour=[colours['_Color'][k] for k in 'rgba'];surface=[floats['_Metallic'],floats['_Glossiness']]
    if any(type(v) not in (int,float) or not math.isfinite(v) or not 0<=v<=1 for v in colour+surface) or not 0<colour[3]<1:raise ValueError('Invalid reference glass scalar')
    if any(t['m_Texture'].get('fileID') for t in textures.values()) or colours.get('_EmissionColor')!=dict(r=0,g=0,b=0,a=1):raise ValueError('Unqualified reference glass texture/emission')
    pbr=material['pbrMetallicRoughness']
    expected=[colour,surface[0],1-surface[1]];actual=[pbr.get('baseColorFactor',[1,1,1,1]),pbr.get('metallicFactor',1),pbr.get('roughnessFactor',1)]
    if any(not np.allclose(a,b,atol=2e-7,rtol=0) for a,b in zip(actual,expected)):raise ValueError('Exported glass colour/surface differs')
    if material.get('alphaMode')!='BLEND' or material.get('doubleSided',False) or material.get('emissiveFactor',[0,0,0])!=[0,0,0] or any(k in material for k in ['alphaCutoff','normalTexture','occlusionTexture','emissiveTexture','extensions','extras']) or any(k in pbr for k in ['baseColorTexture','metallicRoughnessTexture','extensions']):raise ValueError('Exported glass blend/culling/features differ')
    if any(doc.get(k) for k in ['images','textures','samplers','extensionsUsed','extensionsRequired']):raise ValueError('Unexpected glass dependency graph')
    return {'base_color':{'source':None,'dimensions':None,'linear_factor':colour,'scope':'untextured glTF BLEND adaptation; no Unity premultiplied lighting equivalence'}}


def variant_renderer(sources,conversion,rr):
    """Derive a single FBX-backed renderer directly from raw variant references."""
    nested=[(i,v['fields']) for i,v in rr.items() if v['kind']==1001]
    if len(nested)!=1:raise ValueError('Reference needs one model variant')
    nested_id,fields=nested[0];ref=fields['m_SourcePrefab'];name=sources.by_guid[ref['guid']]
    if not name.lower().endswith('.fbx'):raise ValueError('Reference variant source is not FBX')
    model=identity(sources.checked(name));meta=sources.metadata(name)['ModelImporter']
    if len(model['models'])!=1 or len(model['meshes'])!=1 or meta.get('importAnimation') or meta['meshes'].get('importVisibility') or meta['meshes'].get('importBlendShapes') or meta['materials'].get('materialImportMode'):
        raise ValueError('Reference variant is outside static single-mesh profile')
    ids,_=bound_candidates(model,meta);selected={}
    for kind in ['Mesh','MeshRenderer','Transform']:
        selected[kind],=[i for i,value in ids.items() if value['kind']==kind]
    renderer=(selected['MeshRenderer']^nested_id)&((1<<63)-1)
    if renderer!=conversion['renderer']:raise ValueError('Reference inherited renderer differs')
    modification=fields['m_Modification']
    if modification['m_TransformParent']['fileID'] or any(v for k,v in modification.items() if k.startswith(('m_Removed','m_Added'))):raise ValueError('Reference structural model variant differs')
    materials=[m['objectReference'] for m in modification['m_Modifications'] if m['target'].get('guid')==ref['guid'] and m['target']['fileID']==selected['MeshRenderer'] and m['propertyPath']=='m_Materials.Array.data[0]']
    if len(materials)!=1:raise ValueError('Reference variant needs one explicit source material')
    return {'m_Materials':materials},{'m_Mesh':{'guid':ref['guid'],'fileID':selected['Mesh'],'type':3}}


def verify_material(sources, conversion, doc, binary):
    """Independently bind source renderer/material properties to exported slots."""
    rr = records(sources.checked(conversion['prefab']))
    if conversion.get('scene_binding'):
        if any(v['kind']==1001 for v in rr.values()):
            renderer,mesh=variant_renderer(sources,conversion,rr)
        else:
            renderer=rr[conversion['renderer']]['fields'];go=renderer['m_GameObject']['fileID']
            mesh,=[v['fields'] for v in rr.values() if v['kind']==33 and v['fields']['m_GameObject']['fileID']==go]
        if mesh['m_Mesh']['fileID']!=conversion['mesh_fileID'] or sources.by_guid[mesh['m_Mesh']['guid']]!=conversion['model']:raise ValueError('Source mesh binding differs')
        refs=reference_materials(sources,conversion,renderer,mesh)
    else:
        renderers = [v['fields'] for v in rr.values() if v['kind'] == 23]
        if len(renderers) != 1:raise ValueError('Expected one source renderer')
        refs = renderers[0]['m_Materials']
    if not refs or any(ref != refs[0] for ref in refs):
        raise ValueError('Unexpected source material bindings')
    material_path = sources.by_guid[refs[0]['guid']]
    if conversion['material'] != material_path:
        raise ValueError('Exported material is not source renderer material')
    rows = records(sources.checked(material_path))
    fields, = [v['fields'] for v in rows.values() if v['kind'] == 21]
    saved = fields['m_SavedProperties']
    textures = {k:v for row in saved['m_TexEnvs'] for k,v in row.items()}
    colors = {k:v for row in saved['m_Colors'] for k,v in row.items()}
    floats = {k:v for row in saved['m_Floats'] for k,v in row.items()}
    material, = doc['materials']
    pbr = material['pbrMetallicRoughness']
    if conversion.get('material_plan',{}).get('profile')=='unity-standard-glass-blend-v1':
        return verify_glass(fields,doc,material)
    standard = conversion.get('material_plan',{}).get('profile') == 'unity-standard-opaque-v1'
    if standard:standard_source(fields)
    if standard and (fields['m_Shader'].get('fileID') != 46 or fields['m_Shader'].get('guid') != '0000000000000000f000000000000000'):
        raise ValueError('Source shader does not match Standard adaptation')
    base_key = '_MainTex' if standard else '_BaseMap'
    if list(pbr.get('baseColorFactor',[1,1,1,1])) != [1,1,1,1]:
        raise ValueError('Unexpected exported base colour factor')
    expected_metallic = floats['_Metallic'] if standard else 0.
    expected_roughness = 1. - floats['_Glossiness'] if standard else .8
    for actual,expected in [(pbr.get('metallicFactor',1.),expected_metallic),
                            (pbr.get('roughnessFactor',1.),expected_roughness)]:
        if not math.isfinite(actual) or abs(actual-expected)>2e-7:
            raise ValueError('Exported surface scalars differ from source adaptation')
    if standard and (material.get('doubleSided',False) or material.get('alphaMode','OPAQUE') != 'OPAQUE'):
        raise ValueError('Exported culling/alpha differs from opaque source')
    if any(k in material for k in ('normalTexture','occlusionTexture','extensions')) or 'metallicRoughnessTexture' in pbr:
        raise ValueError('Unexpected exported material feature')
    slots = [('base_color',base_key,pbr['baseColorTexture'])]
    expected_emission = [0,0,0]
    if standard and '_EMISSION' in (fields.get('m_ShaderKeywords') or '').split():
        expected_emission = [colors['_EmissionColor'][k] for k in ('r','g','b')]
        if any(expected_emission) and textures.get('_EmissionMap',{}).get('m_Texture',{}).get('fileID'):
            slots.append(('emissive','_EmissionMap',material['emissiveTexture']))
    if material.get('emissiveFactor',[0,0,0]) != expected_emission:
        raise ValueError('Exported emission differs from source')
    if 'emissiveTexture' in material and len(slots) == 1:
        raise ValueError('Unexpected exported emission texture')
    used = set(); result = {}
    for name,key,slot in slots:
        expected_path = sources.by_guid[textures[key]['m_Texture']['guid']]
        original = sources.checked(expected_path)
        if conversion['base_color_texture' if name=='base_color' else 'emissive_texture'] != expected_path:
            raise ValueError('Conversion texture does not match source material')
        if slot.get('texCoord',0) != 0 or slot.get('extensions'):
            raise ValueError('Unexpected exported texture coordinates')
        texture = doc['textures'][slot['index']]
        image_id = texture['source'];used.add(image_id)
        if standard:
            sampler = doc['samplers'][texture['sampler']]
            if [sampler.get(k,default) for k,default in [('magFilter',9729),('minFilter',9987),('wrapS',10497),('wrapT',10497)]] != [9729,9985,10497,10497]:
                raise ValueError('Exported sampler differs from source bilinear repeat')
        image = doc['images'][image_id]
        view = doc['bufferViews'][image['bufferView']];offset=view.get('byteOffset',0)
        embedded = binary[offset:offset+view['byteLength']]
        with Image.open(io.BytesIO(embedded)) as exported, Image.open(original) as palette:
            if exported.size!=palette.size or exported.convert('RGBA').tobytes()!=palette.convert('RGBA').tobytes():
                raise ValueError(f'{name} palette pixels changed or slots were swapped')
            result[name] = {'source':expected_path,'dimensions':list(palette.size),'pixels_exact':True}
    if used != set(range(len(doc['images']))):
        raise ValueError('Unexpected exported image graph')
    return result


def verify(source, stage):
    stage=Path(stage);sources=Sources(source)
    conversion=json.loads((stage/'conversion.json').read_text())
    path=stage/'preview.glb'
    if conversion['archive_sha256']!=sources.manifest['archive_sha256']:raise ValueError('Source archive differs')
    if hashlib.sha256(path.read_bytes()).hexdigest()!=conversion['preview_sha256']:
        raise ValueError('Preview GLB hash mismatch')
    reference=json.loads((stage/'geometry-reference.json').read_text())
    vertices=np.array(reference['vertices_z_up_metres'])[:,[0,2,1]];vertices[:,2]*=-1
    expected=vertices[np.array(reference['triangles'])]
    uv=np.array(reference['triangle_uv_blender']);uv[:,:,1]=1-uv[:,:,1]
    doc,binary=read(path);_,world=hierarchy(doc)
    actual=[];normal_actual=[]
    for index,matrix in world.items():
        node=doc['nodes'][index]
        if 'mesh' not in node:continue
        for primitive in doc['meshes'][node['mesh']]['primitives']:
            if primitive.get('mode',4)!=4:raise ValueError('Expected triangle primitives')
            indices=accessor(doc,binary,primitive['indices']).ravel().reshape((-1,3))
            position=accessor(doc,binary,primitive['attributes']['POSITION'])
            position=position@matrix[:3,:3].T+matrix[:3,3]
            texcoord=accessor(doc,binary,primitive['attributes']['TEXCOORD_0'])
            actual.extend(np.concatenate([position[indices],texcoord[indices]],axis=2))
            if 'triangle_normals_z_up' in reference:
                normals=accessor(doc,binary,primitive['attributes']['NORMAL'])@np.linalg.inv(matrix[:3,:3])
                normal_actual.extend(np.concatenate([position[indices],texcoord[indices],normals[indices]],axis=2))
    normal_difference=None;normalized_source_normals=0;minimum_source_normal_length=None
    if 'triangle_normals_z_up' in reference:
        normals=np.array(reference['triangle_normals_z_up'],dtype=float)[:,:,[0,2,1]];normals[:,:,2]*=-1
        lengths=np.linalg.norm(normals,axis=2)
        if not np.isfinite(normals).all() or (lengths<1e-12).any():raise ValueError('Invalid source normal direction')
        normalized_source_normals=int((np.abs(lengths-1)>1e-5).sum());minimum_source_normal_length=float(lengths.min())
        normals/=lengths[:,:,None]
        normal_difference=normal_error(np.concatenate([expected,uv,normals],axis=2),normal_actual)
    elif signatures(actual)!=signatures(np.concatenate([expected,uv],axis=2)):
        raise ValueError('Export changed source geometry, winding or UVs beyond 10 micrometre/UV quantization')
    if len(doc['materials'])!=1:raise ValueError('Unexpected material graph')
    palettes = verify_material(sources, conversion, doc, binary)
    size = palettes['base_color']['dimensions']
    report={'passed':True,'triangles':len(actual),'geometry_winding_uv_quantization':None if normal_difference is not None else .00001,
            'geometry_matching':'exact f32 cyclic positions; bounded UVs and normalized source directions' if normal_difference is not None else 'legacy 1e-5 cyclic position/UV quantization',
            'bounds_y_up_metres':{'min':vertices.min(axis=0).tolist(),'max':vertices.max(axis=0).tolist()},
            'mesh_normal_vector_error':normal_difference,'normalized_source_normals':normalized_source_normals,'minimum_source_normal_length':minimum_source_normal_length,'source_palette_pixels_exact':True if size is not None else None,'palette_dimensions':size,'verified_material':palettes,
            'asset_sha256':conversion['preview_sha256'],
            'scope':'Blender-imported source geometry/UV/palette to GLB; no Unity scene, shader equivalence or collision claim'}
    return report


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('stage',type=Path)
    args=parser.parse_args();result=verify(args.source,args.stage)
    (args.stage/'independent-verification.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))
