"""Convert an identity static prefab's selected source mesh to masked PBR foliage.

Blender handles FBX/glTF. The source-specific adapter bakes colour floors and
consumes binary vertex-blue selectors. Source mesh normals are retained; custom
normal-map output is explicitly unqualified. No scene/world adoption.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import bpy
import numpy as np
from PIL import Image
from unity_scene import Sources, records
from unity_prefabs import static_prefab
from unity_model_ids import identity, bound_candidates
from unity_foliage import vegetation_material, colour_floor_rgba, face_branch
from unity_full_foliage import full_material,full_colour,normal_rg
from unity_materials import standard_sampler_glb
from merge_static_glb import read
from unity_draw_bindings import select_draw
from foliage_tangents import repair_tangents


def convert(source, shader_source, prefab, output, lod_level=0, full_static=False, canonical_mesh=False, plan_path=None, instance=None, renderer=None, colour_uv=False, emissive_preview=False):
    if emissive_preview and not (full_static and canonical_mesh and plan_path is not None):raise ValueError('Emissive preview requires a full static canonical source draw')
    if colour_uv and not (full_static and canonical_mesh and plan_path is not None):raise ValueError('Colour UV adaptation requires a full static canonical source draw')
    sources=Sources(source);shaders=Sources(shader_source);output=Path(output).resolve()
    if output.exists() or output.is_relative_to(sources.root) or output.is_relative_to(shaders.root):
        raise FileExistsError('Use a new preview directory outside source stages')
    if sources.manifest['archive_sha256']!=shaders.manifest['archive_sha256']:
        raise ValueError('Shader and asset source archives differ')
    scene_binding=None
    if plan_path is not None:
        if not canonical_mesh or lod_level!=0:raise ValueError('Scene draw conversion requires canonical highest-detail geometry')
        draw,static,scene_binding=select_draw(sources,plan_path,instance,renderer,prefab)
    else:
        rr=records(sources.checked(prefab));sources.metadata(prefab);static=static_prefab(rr,lod_level=lod_level)
    unsupported=set(static['unimplemented_components'])-({'136'} if canonical_mesh else set())
    if (plan_path is None and len(static['draws'])!=1) or unsupported:
        raise ValueError('Select one static renderer without unsupported components')
    if plan_path is None:draw,=static['draws']
    if not canonical_mesh and not np.allclose(draw['matrix'],np.eye(4),atol=1e-8,rtol=0):
        raise ValueError('Nonidentity prefab placement needs qualified FBX/Unity axis conversion')
    if any(m!=draw['materials'][0] for m in draw['materials']):
        raise ValueError('Distinct source submesh materials need separate conversion')
    material_path=sources.by_guid[draw['materials'][0]['guid']]
    material,=[v['fields'] for v in records(sources.checked(material_path)).values() if v['kind']==21]
    sources.metadata(material_path)
    shader_path=shaders.by_guid.get(material['m_Shader']['guid'])
    if shader_path is None:raise ValueError('Source shader is outside the qualified foliage family')
    shader_hash=hashlib.sha256(shaders.checked(shader_path).read_bytes()).hexdigest();shaders.metadata(shader_path)
    if emissive_preview:
        from unity_foliage_emission import material as emissive_material
        plan=emissive_material(material,shader_hash)
    else:plan=full_material(material,shader_hash,colour_uv=colour_uv) if full_static else vegetation_material(material,shader_hash)
    model_path=sources.by_guid[draw['mesh']['guid']];model=sources.checked(model_path)
    importer=sources.metadata(model_path)['ModelImporter'];settings=importer['meshes']
    if settings['globalScale']!=1 or settings['useFileUnits']!=1 or settings['bakeAxisConversion']!=0:
        raise ValueError('Unqualified FBX units/axis policy')
    ids,_=bound_candidates(identity(model),importer);binding=ids.get(draw['mesh']['fileID'])
    if not binding or binding['kind']!='Mesh':raise ValueError('Mesh fileID does not bind to source model')
    bpy.ops.wm.read_factory_settings(use_empty=True)
    if colour_uv or emissive_preview:
        from unity_fbx_static import require_identity_geometry
        require_identity_geometry(model,binding['model'])
    bpy.ops.import_scene.fbx(filepath=str(model),use_anim=False,use_custom_props=False,bake_space_transform=colour_uv or emissive_preview)
    selected=[o for o in bpy.data.objects if o.type=='MESH' and o.name==binding['name']]
    if len(selected)!=1:raise ValueError('Imported mesh name does not match bound source mesh')
    mesh,=selected
    if mesh.modifiers or mesh.data.shape_keys:raise ValueError('Deformed source mesh needs separate conversion')
    world=mesh.matrix_world.copy();mesh.parent=None;mesh.matrix_world=world
    if colour_uv or emissive_preview:
        from mathutils import Matrix
        mesh.matrix_world=Matrix.Identity(4)
    bpy.ops.object.select_all(action='DESELECT');mesh.select_set(True);bpy.context.view_layer.objects.active=mesh
    bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
    for o in list(bpy.data.objects):
        if o!=mesh:bpy.data.objects.remove(o,do_unlink=True)
    color=mesh.data.color_attributes.active_color
    if color is None or color.domain not in ('POINT','CORNER'):
        raise ValueError('Source mesh requires explicit point/corner vertex colour')
    branches=[]
    for polygon in mesh.data.polygons:
        values=[color.data[loop if color.domain=='CORNER' else mesh.data.loops[loop].vertex_index].color[2] for loop in polygon.loop_indices]
        branches.append(face_branch(values))
    used=sorted(set(branches));mesh.data.materials.clear();outputs={}
    output.mkdir(parents=True)
    try:
        for branch in used:
            parameters=plan['branches'][branch]
            if parameters['texture'].get('fileID')!=2800000:raise ValueError('Used foliage branch needs explicit default-texture adaptation')
            name=sources.by_guid[parameters['texture']['guid']]
            texture=sources.checked(name);meta=sources.metadata(name)['TextureImporter'];sampling=meta['textureSettings']
            if meta['mipmaps']['sRGBTexture']!=1 or meta['mipmaps'].get('enableMipMap')!=1 or [sampling[k] for k in ('filterMode','wrapU','wrapV','aniso','mipBias')]!=[1,0,0,1,0]:
                raise ValueError('Unsupported source foliage texture import/sampler')
            with Image.open(texture) as image:
                if image.width>4096 or image.height>4096:raise ValueError('Source foliage image exceeds dimension budget')
                pixels=np.array(image.convert('RGBA'))
            baked=(full_colour(pixels,parameters['linear_rgb_floor'],parameters['flat_blend']) if full_static else
                   colour_floor_rgba(pixels,parameters['linear_rgb_floor']));baked_path=output/(branch+'.png')
            Image.fromarray(baked).save(baked_path)
            m=bpy.data.materials.new(branch);m.use_nodes=True;m.use_backface_culling=False
            bsdf=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
            bsdf.inputs['Metallic'].default_value=parameters['metallic'];bsdf.inputs['Roughness'].default_value=parameters['roughness']
            image=m.node_tree.nodes.new('ShaderNodeTexImage');image.image=bpy.data.images.load(str(baked_path));image.image.colorspace_settings.name='sRGB'
            m.node_tree.links.new(image.outputs['Color'],bsdf.inputs['Base Color'])
            # Explicit glTF alpha mode/cutoff is finalized below; no Blender
            # surface-mode heuristic establishes source cutout semantics.
            m.node_tree.links.new(image.outputs['Alpha'],bsdf.inputs['Alpha'])
            mesh.data.materials.append(m)
            outputs[branch]={'source':name,'baked':baked_path.name,'dimensions':list(pixels.shape[1::-1]),
                             'source_sha256':hashlib.sha256(texture.read_bytes()).hexdigest()}
            if full_static:
                extras={}
                for role,key in [('normal','normal_texture'),('occlusion','occlusion_texture')]:
                    ref=parameters[key]
                    if not ref['fileID'] or (role=='normal' and parameters['normal_amount']==0):continue
                    source_name=sources.by_guid[ref['guid']];source_file=sources.checked(source_name);importer=sources.metadata(source_name)['TextureImporter'];sampling=importer['textureSettings']
                    if [sampling[k] for k in ('filterMode','wrapU','wrapV','aniso','mipBias')]!=[1,0,0,1,0] or importer['mipmaps']['enableMipMap']!=1 or importer.get('bumpmap',{}).get('convertToNormalMap',0):
                        raise ValueError('Unsupported foliage normal/AO importer or sampler')
                    if importer['textureType'] not in ((0,1) if role=='normal' else (0,)):raise ValueError('Unqualified foliage auxiliary texture type')
                    serialized_srgb=bool(importer['mipmaps']['sRGBTexture'])
                    is_srgb=serialized_srgb and importer['textureType']!=1
                    with Image.open(source_file) as original:
                        if max(original.size)>4096:raise ValueError('Auxiliary foliage image exceeds dimension budget')
                        values=np.array(original.convert('RGBA'))
                    if role=='normal':derived=normal_rg(values,parameters['normal_amount'],is_srgb)
                    else:
                        value=values[:,:,0].astype(float)/255
                        if is_srgb:value=np.where(value<=.04045,value/12.92,((value+.055)/1.055)**2.4)
                        derived=np.rint(value*255).astype(np.uint8)
                    target=output/(branch+'-'+role+'.png');Image.fromarray(derived).save(target)
                    extras[role]={'source':source_name,'source_sha256':hashlib.sha256(source_file.read_bytes()).hexdigest(),
                                  'baked':target.name,'source_sample_srgb':is_srgb,'serialized_srgb_flag':serialized_srgb,
                                  'source_texture_type':importer['textureType']}
                outputs[branch]['auxiliary']=extras
        for polygon,branch in zip(mesh.data.polygons,branches):polygon.material_index=used.index(branch)
        mesh.data.calc_loop_triangles()
        reference={'vertices_z_up_metres':[list(v.co) for v in mesh.data.vertices],
            'triangles':[list(t.vertices) for t in mesh.data.loop_triangles],
            'triangle_uv_blender':[[list(mesh.data.uv_layers.active.data[i].uv) for i in t.loops] for t in mesh.data.loop_triangles],
            'triangle_normals_z_up':[[list(mesh.data.corner_normals[i].vector) for i in t.loops] for t in mesh.data.loop_triangles],
            'triangle_branches':[branches[t.polygon_index] for t in mesh.data.loop_triangles],
            'triangle_vertex_blue':[[float(color.data[i if color.domain=='CORNER' else mesh.data.loops[i].vertex_index].color[2]) for i in t.loops] for t in mesh.data.loop_triangles]}
        (output/'geometry-reference.json').write_text(json.dumps(reference)+'\n')
        bpy.ops.export_scene.gltf(filepath=str(output/'preview.glb'),export_format='GLB',export_yup=True,
            export_animations=False,export_skins=False,export_lights=False,export_cameras=False,
            export_vertex_color='NONE',export_materials='EXPORT',export_tangents=full_static and not emissive_preview)
        standard_sampler_glb(output/'preview.glb')
        doc,binary=read(output/'preview.glb')
        packed=bytearray(binary)
        for m in doc['materials']:
            if m['name'] not in used:raise ValueError('Unexpected exported material')
            m['alphaMode']='MASK';m['alphaCutoff']=plan['alpha_cutoff'];m['doubleSided']=True
            for role,info in outputs[m['name']].get('auxiliary',{}).items():
                packed.extend(b'\0'*(-len(packed)%4));data=(output/info['baked']).read_bytes()
                view=len(doc['bufferViews']);doc['bufferViews'].append({'buffer':0,'byteOffset':len(packed),'byteLength':len(data)});packed.extend(data)
                image=len(doc['images']);doc['images'].append({'bufferView':view,'mimeType':'image/png','name':m['name']+'-'+role})
                texture=len(doc['textures']);sampler=doc['textures'][m['pbrMetallicRoughness']['baseColorTexture']['index']]['sampler'];doc['textures'].append({'source':image,'sampler':sampler})
                m['normalTexture' if role=='normal' else 'occlusionTexture']={'index':texture}
        if emissive_preview:
            from unity_foliage_emission import pixels as emission_pixels
            masks=[];mask_inputs=[]
            for key in ['_EmissiveMask','_Emissive2Mask']:
                mask_reference=plan['emission']['masks'][key];source_name=sources.by_guid[mask_reference['guid']];original=sources.checked(source_name);importer=sources.metadata(source_name)['TextureImporter'];sampling=importer['textureSettings']
                if importer['textureType']!=0 or importer['mipmaps']['sRGBTexture']!=1 or importer['mipmaps']['enableMipMap']!=1 or [sampling[k] for k in ('filterMode','wrapU','wrapV','aniso','mipBias')]!=[1,0,0,1,0]:raise ValueError('Unqualified emissive mask sampling')
                with Image.open(original) as image:
                    if max(image.size)>4096:raise ValueError('Emissive mask exceeds dimension budget')
                    masks.append(np.array(image.convert('RGBA')))
                mask_inputs.append({'source':source_name,'sha256':hashlib.sha256(original.read_bytes()).hexdigest()})
            emission=emission_pixels(*masks,plan['emission']['colours']);target=output/'leaf-emission.png';Image.fromarray(emission).save(target)
            leaf,=[m for m in doc['materials'] if m['name']=='leaf']
            packed.extend(b'\0'*(-len(packed)%4));data=target.read_bytes();view=len(doc['bufferViews']);doc['bufferViews'].append({'buffer':0,'byteOffset':len(packed),'byteLength':len(data)});packed.extend(data)
            image=len(doc['images']);doc['images'].append({'bufferView':view,'mimeType':'image/png','name':'leaf-emission'})
            texture=len(doc['textures']);sampler=doc['textures'][leaf['pbrMetallicRoughness']['baseColorTexture']['index']]['sampler'];doc['textures'].append({'source':image,'sampler':sampler})
            leaf['emissiveTexture']={'index':texture,'texCoord':0};leaf['emissiveFactor']=[1,1,1];leaf['extensions']={'KHR_materials_emissive_strength':{'emissiveStrength':plan['emission']['strength']}}
            doc['extensionsUsed']=['KHR_materials_emissive_strength'];doc['extensionsRequired']=['KHR_materials_emissive_strength'];outputs['leaf']['emission']={'baked':target.name,'masks':mask_inputs,'strength':plan['emission']['strength']}
        doc['buffers'][0]['byteLength']=len(packed);packed.extend(b'\0'*(-len(packed)%4));binary=bytes(packed)
        doc,binary,tangent_repairs=repair_tangents(doc,binary)
        if colour_uv:
            from foliage_colour_uv import apply as apply_colour_uv
            doc,binary=apply_colour_uv(doc,binary,plan['branches'])
        # Preserve embedded data while stating source alpha semantics explicitly.
        import struct
        text=json.dumps(doc,separators=(',',':')).encode();text+=b' '*(-len(text)%4)
        (output/'preview.glb').write_bytes(struct.pack('<4sII',b'glTF',2,28+len(text)+len(binary))+
            struct.pack('<I4s',len(text),b'JSON')+text+struct.pack('<I4s',len(binary),b'BIN\0')+binary)
        report={'profile':plan,'prefab':prefab,'model':model_path,'mesh_fileID':draw['mesh']['fileID'],
                'bound_mesh_name':binding['name'],'renderer':draw['renderer'],'material':material_path,'lod_level':lod_level,
                'textures':outputs,'branches':used,'triangles':len(reference['triangles']),
                'archive_sha256':sources.manifest['archive_sha256'],
                'canonical_mesh':canonical_mesh,'source_draw_matrix':draw['matrix'],
                'source_unimplemented_components':static['unimplemented_components'],
                'scene_binding':scene_binding,
                'tangent_repairs':tangent_repairs,
                'asset_sha256':hashlib.sha256((output/'preview.glb').read_bytes()).hexdigest(),
                'scope':'one selected source renderer, static masked PBR adaptation; no assembled scene or collision'}
        (output/'conversion.json').write_text(json.dumps(report,indent=2)+'\n');return report
    except BaseException:
        shutil.rmtree(output);raise


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('shader_source',type=Path)
    parser.add_argument('--prefab',required=True);parser.add_argument('--output',required=True,type=Path)
    parser.add_argument('--lod-level',type=int,default=0)
    parser.add_argument('--full-static',action='store_true',help='Explicit full-shader static PBR profile with motion/highlight limitations')
    parser.add_argument('--canonical-mesh',action='store_true',help='Export mesh prototype; retain source pose/collider metadata without applying it')
    parser.add_argument('--draw-plan',type=Path);parser.add_argument('--instance',type=int);parser.add_argument('--renderer',type=int)
    parser.add_argument('--colour-uv',action='store_true',help='Separate bounded source colour tiling from normal/AO UVs')
    parser.add_argument('--emissive-preview',action='store_true',help='Explicit HDR emission and base-floor preview; spatial colour noise omitted')
    args=parser.parse_args()
    if any(v is not None for v in [args.draw_plan,args.instance,args.renderer]) and any(v is None for v in [args.draw_plan,args.instance,args.renderer]):parser.error('Supply --draw-plan, --instance and --renderer together')
    print(json.dumps(convert(args.source,args.shader_source,args.prefab,args.output,args.lod_level,args.full_static,args.canonical_mesh,args.draw_plan,args.instance,args.renderer,args.colour_uv,args.emissive_preview),indent=2))
