"""Independent source-material, derived-pixel and branch geometry GLB checks."""
import argparse
from collections import Counter
import hashlib
import io
import json
import math
from functools import lru_cache
from pathlib import Path
import numpy as np
from PIL import Image
from unity_scene import Sources, records
from static_geometry import accessor, hierarchy
from merge_static_glb import read


@lru_cache(maxsize=2)
def scene_reference(path,digest):
    if hashlib.sha256(Path(path).read_bytes()).hexdigest()!=digest:raise ValueError('Source scene changed before reference parsing')
    return records(Path(path))


def expected_colour_table(tint,flat_blend=0):
    result=[]
    for value in range(256):
        source=value/255.
        linear=source/12.92 if source<=.04045 else ((source+.055)/1.055)**2.4
        adjusted=(1-flat_blend)*max(linear,tint)+flat_blend*tint
        encoded=12.92*adjusted if adjusted<=.0031308 else 1.055*adjusted**(1/2.4)-.055
        result.append(round(min(1,max(0,encoded))*255))
    return result


def verify_auxiliary(sources,textures,floats,prefix,material,doc,binary):
    images=set();result={}
    for role,key,slot in [('normal','NormalMap','normalTexture'),('occlusion','AmbientOcclusion','occlusionTexture')]:
        ref=textures[prefix+key]['m_Texture'];enabled=bool(ref['fileID']) and (role!='normal' or floats[prefix+'NormalAmount']!=0)
        if (slot in material)!=enabled:raise ValueError('Source normal/AO texture binding differs')
        if not enabled:continue
        entry=material[slot]
        if entry.get('texCoord',0)!=0 or entry.get('extensions') or entry.get('scale',1)!=1 or entry.get('strength',1)!=1:raise ValueError('Unexpected auxiliary texture mapping/strength')
        texture=doc['textures'][entry['index']];sampler=doc['samplers'][texture['sampler']]
        if [sampler.get(k,v) for k,v in [('magFilter',9729),('minFilter',9987),('wrapS',10497),('wrapT',10497)]]!=[9729,9985,10497,10497]:raise ValueError('Incorrect normal/AO sampler')
        image=doc['images'][texture['source']];images.add(texture['source']);view=doc['bufferViews'][image['bufferView']];offset=view.get('byteOffset',0)
        name=sources.by_guid[ref['guid']];metadata=sources.metadata(name)['TextureImporter'];sample_srgb=bool(metadata['mipmaps']['sRGBTexture']) and metadata['textureType']!=1
        with Image.open(sources.checked(name)) as original,Image.open(io.BytesIO(binary[offset:offset+view['byteLength']])) as exported:
            source=np.array(original.convert('RGBA'),dtype=np.uint8);actual=np.array(exported.convert('RGB'),dtype=np.uint8)
        if source.shape[:2]!=actual.shape[:2]:raise ValueError('Normal/AO image dimensions differ')
        table=np.array([v/255/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in range(256)]) if sample_srgb else np.arange(256)/255
        if role=='normal':
            x=(table[source[:,:,0]]*source[:,:,3]/255*2-1)*floats[prefix+'NormalAmount'];y=(table[source[:,:,1]]*2-1)*floats[prefix+'NormalAmount']
            z=np.sqrt(np.clip(1-x*x-y*y,0,1));wanted=np.stack([x,y,z],axis=-1);wanted/=np.linalg.norm(wanted,axis=2)[...,None]
            decoded=actual.astype(float)/127.5-1;length=np.linalg.norm(decoded,axis=2)
            if (length<.9).any() or (length>1.1).any():raise ValueError('Invalid exported tangent-space normals')
            decoded/=length[...,None];error=float(np.linalg.norm(decoded-wanted,axis=2).max())
            if error>.007:raise ValueError('Exported normal map exceeds RGB8 quantization bound')
            result[role]={'source':name,'source_sample_srgb':sample_srgb,'max_vector_error':error,'vector_error_limit':.007}
        else:
            wanted=np.rint(table[source[:,:,0]]*255).astype(np.uint8)
            if not np.array_equal(actual,np.repeat(wanted[:,:,None],3,axis=2)):raise ValueError('Source occlusion channel differs')
            result[role]={'source':name,'pixels_exact':True}
    return images,result



def normal_error(expected, actual):
    """Match cyclic triangles, then bound standard-export rounded unit normals."""
    groups={}
    def orient(packet):
        packet=np.asarray(packet,dtype=np.float64)
        if not np.isfinite(packet).all():raise ValueError('Nonfinite source/exported triangle')
        rows=[tuple(v) for v in packet[:,:3].astype(np.float32)]
        start=min(range(3),key=lambda n:tuple(rows[n:]+rows[:n]))
        key=tuple(rows[start:]+rows[:start])
        normal=np.roll(packet[:,5:],-start,axis=0)
        lengths=np.linalg.norm(normal,axis=1)
        if not np.isfinite(normal).all() or (np.abs(lengths-1.)>.00001).any():
            raise ValueError('Invalid source/exported unit normals')
        return key,(np.roll(packet[:,3:5],-start,axis=0),normal/lengths[:,None])
    for packet in expected:
        key,values=orient(packet);groups.setdefault(key,[]).append(values)
    maximum=0.
    for packet in actual:
        key,(uv,normals)=orient(packet);candidates=groups.get(key,[])
        if not candidates:raise ValueError('Normal triangle has no source geometry binding')
        errors=[float(np.linalg.norm(value-normals,axis=1).max()) if np.all(np.abs(source_uv-uv)<=np.maximum(1e-7,2*np.abs(np.spacing(source_uv.astype(np.float32))))) else float('inf') for source_uv,value in candidates]
        if not any(math.isfinite(value) for value in errors):raise ValueError('Source UVs exceed f32 export rounding bound')
        best=min(range(len(errors)),key=errors.__getitem__);difference=errors[best]
        # Blender 5.2 rounds each component by at most 5e-5 then normalizes.
        # 2*sqrt(3)*5e-5/(1-sqrt(3)*5e-5) < 0.00018 for unit inputs.
        if difference>.00018:raise ValueError('Source mesh normals exceed standard export rounding bound')
        maximum=max(maximum,difference);del candidates[best]
    if any(groups.values()):raise ValueError('Missing exported normal triangles')
    return maximum


def reference_materials(sources,conversion,renderer,mesh):
    material_refs=list(renderer['m_Materials']);binding=conversion.get('scene_binding')
    if binding:
        plan_bytes=Path(binding['plan']).read_bytes()
        if len(plan_bytes)>64*1024*1024 or hashlib.sha256(plan_bytes).hexdigest()!=binding['plan_sha256']:raise ValueError('Reference draw plan binding differs')
        source_plan=json.loads(plan_bytes)
        if source_plan['archive_sha256']!=sources.manifest['archive_sha256'] or source_plan['scene']!=binding['scene']:raise ValueError('Reference plan source differs')
        draws=[d for d in source_plan['draws'] if (d['instance'],d['renderer'])==(binding['instance'],binding['renderer'])]
        if len(draws)!=1 or binding['renderer']!=conversion['renderer']:raise ValueError('Reference draw instance/renderer differs')
        selected,=draws
        if selected['source']!=conversion['prefab'] or selected['mesh']!=mesh['m_Mesh'] or selected['matrix']!=binding['world_matrix']:raise ValueError('Reference source draw/provenance differs')
        scene_path=sources.checked(binding['scene']);sources.metadata(binding['scene'])
        if sources.manifest['selected'][binding['scene']]['sha256']!=binding['scene_sha256']:raise ValueError('Reference scene hash differs')
        scene=scene_reference(str(scene_path),binding['scene_sha256']);instance=scene[binding['instance']]
        guid=sources.manifest['selected'][conversion['prefab']]['guid']
        if instance['kind']!=1001 or instance['fields']['m_SourcePrefab'].get('guid')!=guid:raise ValueError('Reference instance source differs')
        for modification in instance['fields']['m_Modification']['m_Modifications']:
            if modification['target'].get('guid')!=guid or modification['target']['fileID']!=conversion['renderer']:continue
            property_path=modification['propertyPath']
            if property_path.startswith('m_Materials.Array.data[') and property_path.endswith(']'):
                slot=int(property_path[len('m_Materials.Array.data['):-1])
                if not 0<=slot<len(material_refs):raise ValueError('Reference material override slot invalid')
                material_refs[slot]=modification['objectReference']
        if material_refs!=selected['materials']:raise ValueError('Reference scene material slots differ')
    return material_refs


def verify_colour_uv(doc,binary,primitive,environment):
    scale=np.array([environment['m_Scale'][k] for k in 'xy'],dtype=float)
    offset=np.array([environment['m_Offset'][k] for k in 'xy'],dtype=float)
    if not np.isfinite(scale).all() or not np.isfinite(offset).all() or (scale<=0).any() or (scale>16).any() or (abs(offset)>16).any():raise ValueError('Reference colour UV transform exceeds bound')
    transformed=not np.array_equal(scale,[1,1]) or not np.array_equal(offset,[0,0])
    base=doc['materials'][primitive['material']]['pbrMetallicRoughness']['baseColorTexture']
    if base.get('extensions') or base.get('texCoord',0)!=(1 if transformed else 0):raise ValueError('Reference colour UV channel differs')
    if transformed:
        uv=accessor(doc,binary,primitive['attributes']['TEXCOORD_0']).astype(float)
        wanted=np.stack([uv[:,0]*scale[0]+offset[0],uv[:,1]*scale[1]+1-scale[1]-offset[1]],axis=1)
        if 'TEXCOORD_1' not in primitive['attributes']:raise ValueError('Missing source colour UV stream')
        actual=accessor(doc,binary,primitive['attributes']['TEXCOORD_1'])
        tolerance=np.maximum(1e-7,abs(np.spacing(wanted.astype(np.float32)))*2)
        if actual.shape!=wanted.shape or not np.isfinite(actual).all() or np.any(abs(actual-wanted)>tolerance):raise ValueError('Source colour UV transform differs')
    elif 'TEXCOORD_1' in primitive['attributes']:raise ValueError('Unexpected second colour UV stream')


def verify(source, shader_source, stage):
    sources=Sources(source);shaders=Sources(shader_source);stage=Path(stage)
    conversion=json.loads((stage/'conversion.json').read_text());path=stage/'preview.glb'
    if hashlib.sha256(path.read_bytes()).hexdigest()!=conversion['asset_sha256']:raise ValueError('GLB hash differs from conversion')
    if conversion['archive_sha256']!=sources.manifest['archive_sha256'] or sources.manifest['archive_sha256']!=shaders.manifest['archive_sha256']:
        raise ValueError('Source archives differ')
    rr=records(sources.checked(conversion['prefab']));sources.metadata(conversion['prefab'])
    renderer=rr[conversion['renderer']]['fields'];go=renderer['m_GameObject']['fileID']
    filters=[v['fields'] for v in rr.values() if v['kind']==33 and v['fields']['m_GameObject']['fileID']==go]
    mesh,=filters
    if mesh['m_Mesh']['fileID']!=conversion['mesh_fileID'] or sources.by_guid[mesh['m_Mesh']['guid']]!=conversion['model']:
        raise ValueError('Selected source mesh differs')
    for record in rr.values():
        if record['kind']!=205:continue
        levels=[{r['renderer']['fileID'] for r in level['renderers']} for level in record['fields']['m_LODs']]
        if any(conversion['renderer'] in level for level in levels) and conversion['renderer'] not in levels[conversion['lod_level']]:
            raise ValueError('Selected renderer is outside requested source LOD')
    material_refs=reference_materials(sources,conversion,renderer,mesh)
    material_path=sources.by_guid[material_refs[0]['guid']]
    if any(m!=material_refs[0] for m in material_refs) or material_path!=conversion['material']:
        raise ValueError('Source material binding differs')
    fields,=[v['fields'] for v in records(sources.checked(material_path)).values() if v['kind']==21]
    shader_path=shaders.by_guid[fields['m_Shader']['guid']]
    shader_hash=hashlib.sha256(shaders.checked(shader_path).read_bytes()).hexdigest()
    full=shader_hash=='87a2b6f6206262674de9b2ff05d3743ee2a62e0d1bc7eff5b06575ab4c56abe4'
    if not full and shader_hash!='f2f800520bffd6081ef699ded2f16e773e209b717cad6695b8786b97b499e9e0':
        raise ValueError('Unsupported source shader')
    saved=fields['m_SavedProperties'];textures={k:v for row in saved['m_TexEnvs'] for k,v in row.items()}
    colors={k:v for row in saved['m_Colors'] for k,v in row.items()};floats={k:v for row in saved['m_Floats'] for k,v in row.items()}
    from verify_unity_foliage_emission import source as emission_source,material as verify_emission
    emissive_preview=emission_source(fields,conversion,shader_hash)
    if floats['_FrostingSwitch'] or (floats['_EmissiveAmount'] and not emissive_preview):raise ValueError('Unsupported source effects')
    if full:
        if set((fields.get('m_ShaderKeywords') or '').split())-{'_ALPHATEST_ON','_NORMALMAP','_EMISSION'}:raise ValueError('Unsupported source vegetation keywords')
        if not emissive_preview and (any(colors['_LeafBaseColour'][c]!=colors[key][c] for c in 'rgb' for key in ['_LeafNoiseColour','_LeafNoiseLargeColour']) or any(colors['_TrunkBaseColour'][c]!=colors['_TrunkNoiseColour'][c] for c in 'rgb')):
            raise ValueError('Source spatial colour requires further shader evaluation')
        for prefix in ['_Leaf','_Trunk']:
            for suffix in ['NormalMap','AmbientOcclusion']:
                entry=textures[prefix+suffix]
                if entry['m_Scale']!={'x':1,'y':1} or entry['m_Offset']!={'x':0,'y':0}:raise ValueError('Unqualified auxiliary source UV transform')
    doc,binary=read(path);reference=json.loads((stage/'geometry-reference.json').read_text())
    vertices=np.array(reference['vertices_z_up_metres'])[:,[0,2,1]];vertices[:,2]*=-1
    triangles=vertices[np.array(reference['triangles'])]
    uv=np.array(reference['triangle_uv_blender']);uv[:,:,1]=1-uv[:,:,1]
    expected={'leaf':[],'trunk':[]};actual={'leaf':[],'trunk':[]}
    normal_expected={'leaf':[],'trunk':[]};normal_actual={'leaf':[],'trunk':[]}
    normals=np.array(reference['triangle_normals_z_up'])[:,:,[0,2,1]];normals[:,:,2]*=-1
    for triangle,texcoord,blue,normal in zip(triangles,uv,reference['triangle_vertex_blue'],normals):
        if len(blue)!=3 or any(v not in (0,1) or v!=blue[0] for v in blue):raise ValueError('Invalid source branch selector')
        branch='leaf' if blue[0]==1 else 'trunk';expected[branch].append(np.concatenate([triangle,texcoord],axis=1))
        normal_expected[branch].append(np.concatenate([triangle,texcoord,normal],axis=1))
    if sum(map(len,expected.values()))!=len(triangles):raise ValueError('Incomplete branch reference')
    materials={m['name']:m for m in doc['materials']}
    if len(materials)!=len(doc['materials']) or set(materials)!={k for k,v in expected.items() if v}:raise ValueError('Incorrect branch material set')
    _,world=hierarchy(doc)
    for i,matrix in world.items():
        node=doc['nodes'][i]
        if 'mesh' not in node:continue
        for primitive in doc['meshes'][node['mesh']]['primitives']:
            if primitive.get('mode',4)!=4 or 'COLOR_0' in primitive['attributes']:raise ValueError('Unexpected primitive/vertex tint')
            branch=doc['materials'][primitive['material']]['name']
            if full:verify_colour_uv(doc,binary,primitive,textures['_LeafTex' if branch=='leaf' else '_TunkTex'])
            indices=accessor(doc,binary,primitive['indices']).ravel().reshape((-1,3))
            positions=accessor(doc,binary,primitive['attributes']['POSITION'])@matrix[:3,:3].T+matrix[:3,3]
            texcoord=accessor(doc,binary,primitive['attributes']['TEXCOORD_0'])
            actual[branch].extend(np.concatenate([positions[indices],texcoord[indices]],axis=2))
            normal=accessor(doc,binary,primitive['attributes']['NORMAL'])@np.linalg.inv(matrix[:3,:3])
            normal_actual[branch].extend(np.concatenate([positions[indices],texcoord[indices],normal[indices]],axis=2))
            if 'TANGENT' in primitive['attributes']:
                tangents=accessor(doc,binary,primitive['attributes']['TANGENT']);lengths=np.linalg.norm(tangents[:,:3],axis=1)
                if not np.isfinite(tangents).all() or (np.abs(lengths-1)>2e-4).any() or not np.isin(tangents[:,3],[-1,1]).all():raise ValueError('Invalid exported unit tangent/handedness')
    for repair in conversion.get('tangent_repairs',[]):
        primitive=doc['meshes'][repair['mesh']]['primitives'][repair['primitive']];index=repair['vertex'];faces=accessor(doc,binary,primitive['indices']).reshape(-1,3);incident=faces[np.any(faces==index,axis=1)]
        if len(incident)!=1 or incident[0].tolist()!=repair['triangle']:raise ValueError('Tangent repair source triangle differs')
        p=accessor(doc,binary,primitive['attributes']['POSITION']).astype(float);uv0=accessor(doc,binary,primitive['attributes']['TEXCOORD_0']).astype(float);ids=incident[0]
        basis=np.linalg.solve(uv0[ids[1:]]-uv0[ids[0]],p[ids[1:]]-p[ids[0]])
        n=accessor(doc,binary,primitive['attributes']['NORMAL'])[index].astype(float);n/=np.linalg.norm(n)
        t=basis[0]-n*np.dot(n,basis[0]);t/=np.linalg.norm(t);w=1 if np.dot(np.cross(n,t),basis[1])>0 else -1
        repaired_actual=accessor(doc,binary,primitive['attributes']['TANGENT'])[index]
        if not np.allclose(t,repaired_actual[:3],atol=1e-6,rtol=0) or repaired_actual[3]!=w:raise ValueError('Repaired tangent differs from independent source UV derivative')
    images=set();report={}
    for branch,material in materials.items():
        normal_difference=normal_error(normal_expected[branch],normal_actual[branch])
        prefix='_Leaf' if branch=='leaf' else '_Trunk';slot='_LeafTex' if branch=='leaf' else '_TunkTex'
        pbr=material['pbrMetallicRoughness']
        for exported,wanted in [(pbr.get('metallicFactor',1),floats[prefix+'Metallic']),
                                 (pbr.get('roughnessFactor',1),1-floats[prefix+'Smoothness'])]:
            if not math.isfinite(exported) or abs(exported-wanted)>2e-7:raise ValueError('Incorrect branch surface scalars')
        cutoff=material.get('alphaCutoff',.5)
        if material.get('alphaMode')!='MASK' or not math.isfinite(cutoff) or abs(cutoff-floats['_Cutoff'])>2e-7 or not material.get('doubleSided'):
            raise ValueError('Incorrect source cutout/culling')
        if (pbr.get('baseColorFactor',[1,1,1,1])!=[1,1,1,1] or 'metallicRoughnessTexture' in pbr
                or any(k in material for k in (() if emissive_preview else ('emissiveTexture','extensions') if full else ('normalTexture','emissiveTexture','occlusionTexture','extensions')))
                or (not emissive_preview and material.get('emissiveFactor',[0,0,0])!=[0,0,0])):raise ValueError('Unexpected material feature')
        base=pbr['baseColorTexture']
        expected_channel=1 if textures[slot]['m_Scale']!={'x':1,'y':1} or textures[slot]['m_Offset']!={'x':0,'y':0} else 0
        if base.get('texCoord',0)!=expected_channel or base.get('extensions'):raise ValueError('Unexpected texture coordinate mapping')
        texture=doc['textures'][base['index']];sampler=doc['samplers'][texture['sampler']]
        if [sampler.get(k,v) for k,v in [('magFilter',9729),('minFilter',9987),('wrapS',10497),('wrapT',10497)]]!=[9729,9985,10497,10497]:
            raise ValueError('Incorrect foliage sampler')
        image=doc['images'][texture['source']];images.add(texture['source']);view=doc['bufferViews'][image['bufferView']];offset=view.get('byteOffset',0)
        source_name=sources.by_guid[textures[slot]['m_Texture']['guid']]
        with Image.open(sources.checked(source_name)) as original,Image.open(io.BytesIO(binary[offset:offset+view['byteLength']])) as exported:
            original=original.convert('RGBA');exported=exported.convert('RGBA')
            channels=original.split();tint=colors[prefix+'BaseColour']
            wanted=Image.merge('RGBA',tuple(channels[n].point(expected_colour_table(tint[k],floats[prefix+'FlatColourSwitch'] if full else 0)) for n,k in enumerate('rgb'))+(channels[3],))
            if wanted.size!=exported.size or wanted.tobytes()!=exported.tobytes():raise ValueError('Derived source colour/alpha pixels differ')
            alpha=np.asarray(exported.getchannel('A'));cutoff=floats['_Cutoff']
            report[branch]={'triangles':len(actual[branch]),'dimensions':list(exported.size),'source':source_name,
                'discarded_texels':int(np.count_nonzero(alpha/255.<cutoff)),
                'retained_texels':int(np.count_nonzero(alpha/255.>=cutoff)),'derived_pixels_exact':True,'mesh_normal_vector_error':normal_difference,'mesh_normal_vector_error_limit':.00018}
        if full:
            auxiliary,details=verify_auxiliary(sources,textures,floats,prefix,material,doc,binary);images.update(auxiliary);report[branch]['auxiliary']=details
        if emissive_preview:
            emissive_images,details=verify_emission(sources,fields,branch,material,doc,binary);images.update(emissive_images);report[branch].update(details)
    if images!=set(range(len(doc['images']))):raise ValueError('Unexpected image graph')
    return {'passed':True,'branches':report,'triangles':len(triangles),'tangent_repairs_checked':len(conversion.get('tangent_repairs',[])),'geometry_winding_matching':'exact f32 cyclic positions; UV within max(1e-7, two f32 ULP)',
            'asset_sha256':conversion['asset_sha256'],
            'scope':('explicit base-floor preview omitting spatial colour noise, source mask HDR emission with default-white pulse; no Unity Editor, wind/highlights/BRDF/assembled scene parity' if emissive_preview else 'source reference geometry/branch/pixels and chosen normal reconstruction; no Unity Editor, motion/highlights, BRDF or assembled scene parity') if full else 'imported source reference geometry/branch bindings and derived raw-image pixels; no Unity Editor, custom normal or assembled scene parity'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('shader_source',type=Path);parser.add_argument('stage',type=Path)
    args=parser.parse_args();result=verify(args.source,args.shader_source,args.stage)
    (args.stage/'independent-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
