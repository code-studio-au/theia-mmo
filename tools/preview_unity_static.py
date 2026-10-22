#!/usr/bin/env python3
"""Convert a simple prefab or bound canonical scene draw through explicit PBR.

Blender handles FBX/glTF; scene overrides and mesh identities are source-bound.
Canonical scene geometry excludes imported model-node placement. No collision,
complete scene or Unity shader equivalence is implied.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

import bpy
from PIL import Image
from unity_scene import Sources, records
from unity_draw_bindings import select_draw
from unity_model_ids import identity, bound_candidates
from unity_fbx_static import require_identity_geometry
from unity_triplanar import material as triplanar_material,ROLES
from unity_materials import standard_material, standard_sampler_glb, glass_material


def only(rr, kind):
    values = [v['fields'] for v in rr.values() if v['kind'] == kind]
    if len(values) != 1:
        raise ValueError(f'Expected exactly one Unity class {kind}')
    return values[0]


def preview(source, prefab, output, base_color_preview, standard=False, *, plan_path=None, instance=None, renderer=None, triplanar_shader_source=None, triplanar_texture_reference=None, glass=False):
    if glass and (not standard or plan_path is None or triplanar_shader_source is not None):raise ValueError('Glass requires an explicit canonical Standard adaptation')
    if triplanar_shader_source is not None and (not standard or plan_path is None):raise ValueError('Triplanar requires an explicit canonical Standard rendering adapter')
    if base_color_preview == standard:
        raise ValueError('Select exactly one explicit material adaptation')
    sources = Sources(source)
    texture_reference = None
    if triplanar_texture_reference is not None:
        if triplanar_shader_source is None:raise ValueError('Texture reference requires explicit triplanar conversion')
        from unity_texture_reference import inspect as inspect_textures
        texture_reference = inspect_textures(source, triplanar_texture_reference)
    output = Path(output).resolve()
    if output.exists() or output.is_relative_to(sources.root) or sources.root.is_relative_to(output):
        raise FileExistsError('Preview output must be new and outside source')
    scene_binding=None
    if plan_path is not None:
        if not standard:raise ValueError('Canonical scene conversion requires the explicit Standard profile')
        draw,static,scene_binding=select_draw(sources,plan_path,instance,renderer,prefab,allow_model_variant=True)
        if set(static['unimplemented_components'])-{'136'}:raise ValueError('Unsupported source prefab component')
        mesh_ref=draw['mesh'];mats=draw['materials'];go={'m_Name':str(draw['renderer'])}
    else:
        rr = records(sources.checked(prefab));sources.metadata(prefab)
        if any(v['kind'] not in (1, 4, 23, 33, 64, 65) for v in rr.values()):
            raise ValueError('Expected a simple static prefab without nested models or behaviours')
        transform, mesh_filter, source_renderer = (only(rr, k) for k in (4, 33, 23))
        go = only(rr, 1)
        if not go.get('m_IsActive') or not source_renderer.get('m_Enabled'):
            raise ValueError('Cannot preview inactive/disabled source')
        for key, expected in [('m_LocalPosition', dict(x=0,y=0,z=0)),
                              ('m_LocalScale', dict(x=1,y=1,z=1)),
                              ('m_LocalRotation', dict(x=0,y=0,z=0,w=1))]:
            if transform[key] != expected:
                raise ValueError('Nonidentity prefab transform needs separately qualified coordinate mapping')
        if transform['m_Father']['fileID'] or transform.get('m_Children'):
            raise ValueError('Expected a single root transform')
        mesh_ref=mesh_filter['m_Mesh'];mats=source_renderer['m_Materials']
    model_name = sources.by_guid[mesh_ref['guid']]
    model = sources.checked(model_name);model_meta = sources.metadata(model_name)['ModelImporter']
    importer = model_meta['meshes']
    if importer['globalScale'] != 1 or importer['useFileUnits'] != 1 or importer['bakeAxisConversion'] != 0:
        raise ValueError('Unexpected source model import units/axis policy')
    if not mats or any(m != mats[0] for m in mats):
        raise ValueError('Preview currently requires one repeated material binding')
    material_name = sources.by_guid[mats[0]['guid']]
    material_fields = only(records(sources.checked(material_name)), 21);sources.metadata(material_name)
    if standard:
        if triplanar_shader_source is not None:
            if scene_binding is None:raise ValueError('Triplanar needs a canonical source draw')
            shaders=Sources(triplanar_shader_source)
            if shaders.manifest['archive_sha256']!=sources.manifest['archive_sha256']:raise ValueError('Triplanar shader archive differs')
            shader_name=shaders.by_guid[material_fields['m_Shader']['guid']];shaders.metadata(shader_name)
            material_plan=triplanar_material(material_fields,hashlib.sha256(shaders.checked(shader_name).read_bytes()).hexdigest())
        else:material_plan = glass_material(material_fields) if glass else standard_material(material_fields)
        base_ref = material_plan['base_color']
    else:
        saved = material_fields['m_SavedProperties']
        envs = {k:v for item in saved['m_TexEnvs'] for k,v in item.items()}
        color = {k:v for item in saved['m_Colors'] for k,v in item.items()}['_BaseColor']
        base = envs['_BaseMap']
        if base['m_Scale'] != dict(x=1,y=1) or base['m_Offset'] != dict(x=0,y=0) or color != dict(r=1,g=1,b=1,a=1):
            raise ValueError('Unexpected base-colour transform/tint')
        base_ref = base['m_Texture']
        material_plan = {'profile':'explicit-base-colour-preview', 'metallic':0., 'roughness':.8,
                         'emissive':None, 'emissive_factor':[0.,0.,0.]}
    def checked_texture(reference):
        name = sources.by_guid[reference['guid']]
        path = sources.checked(name);meta = sources.metadata(name)['TextureImporter']
        if path.suffix.lower() != '.png' or meta['mipmaps']['sRGBTexture'] != 1 or (standard and meta['mipmaps'].get('enableMipMap') != 1):
            raise ValueError('Expected sRGB source PNG')
        settings = meta['textureSettings']
        if settings['wrapU'] != 0 or settings['wrapV'] != 0 or settings['filterMode'] != 1 or (standard and (settings.get('aniso',1) != 1 or settings.get('mipBias',0) != 0)):
            raise ValueError('Unexpected source texture sampler')
        return name,path
    texture_name,texture = checked_texture(base_ref) if base_ref is not None else (None,None)
    emissive_name,emissive_path = (checked_texture(material_plan['emissive'])
                                  if material_plan['emissive'] else (None,None))
    if scene_binding:
        ids,_=bound_candidates(identity(model),model_meta);binding=ids.get(mesh_ref['fileID'])
        if not binding or binding['kind']!='Mesh':raise ValueError('Source mesh fileID is not bound')
        require_identity_geometry(model,binding['model'])
    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.fbx(filepath=str(model), use_anim=False, use_custom_props=False, bake_space_transform=bool(scene_binding))
    meshes = [o for o in bpy.data.objects if o.type == 'MESH']
    if scene_binding:
        ids,_=bound_candidates(identity(model),model_meta);binding=ids.get(mesh_ref['fileID'])
        if not binding or binding['kind']!='Mesh':raise ValueError('Source mesh fileID is not bound')
        meshes=[o for o in meshes if o.name==binding['name']]
        if len(meshes)!=1:raise ValueError('Imported mesh does not match bound source mesh')
    elif len(meshes) != 1 or any(o.type not in ('MESH','EMPTY') for o in bpy.data.objects):
        raise ValueError('Model mesh ID is ambiguous; refuse to guess a Unity hashed fileID')
    mesh = meshes[0]
    if mesh.modifiers or mesh.data.shape_keys:raise ValueError('Deformed source mesh needs separate conversion')
    if scene_binding:
        from mathutils import Matrix
        mesh.parent=None;mesh.matrix_world=Matrix.Identity(4)
    else:
        world = mesh.matrix_world.copy();mesh.parent=None;mesh.matrix_world=world
    bpy.ops.object.select_all(action='DESELECT');mesh.select_set(True);bpy.context.view_layer.objects.active=mesh
    bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
    for obj in list(bpy.data.objects):
        if obj != mesh:bpy.data.objects.remove(obj,do_unlink=True)
    mesh.name = go['m_Name']
    material = bpy.data.materials.new(material_fields['m_Name']+'AdaptedPBR');material.use_nodes=True
    if standard:material.use_backface_culling=True
    shader = next(n for n in material.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    shader.inputs['Roughness'].default_value=material_plan['roughness']
    shader.inputs['Metallic'].default_value=material_plan['metallic']
    if texture is not None:
        image = material.node_tree.nodes.new('ShaderNodeTexImage')
        image.image=bpy.data.images.load(str(texture));image.image.colorspace_settings.name='sRGB'
        material.node_tree.links.new(image.outputs['Color'],shader.inputs['Base Color'])
    else:
        shader.inputs['Base Color'].default_value=material_plan['base_color_factor']
        shader.inputs['Alpha'].default_value=material_plan['base_color_factor'][3]
    shader.inputs['Emission Color'].default_value=(*material_plan['emissive_factor'],1.)
    shader.inputs['Emission Strength'].default_value=1.
    if emissive_path:
        emissive_image=material.node_tree.nodes.new('ShaderNodeTexImage')
        emissive_image.image=bpy.data.images.load(str(emissive_path));emissive_image.image.colorspace_settings.name='sRGB'
        if material_plan['emissive_factor'] == [0.,0.,0.]:
            shader.inputs['Emission Strength'].default_value=0.
        else:
            material.node_tree.links.new(emissive_image.outputs['Color'],shader.inputs['Emission Color'])
    mesh.data.materials.clear();mesh.data.materials.append(material)
    for polygon in mesh.data.polygons:polygon.material_index=0
    mesh.data.calc_loop_triangles()
    reference = {'vertices_z_up_metres':[list(v.co) for v in mesh.data.vertices],
                 'triangles':[list(t.vertices) for t in mesh.data.loop_triangles],
                 'triangle_normals_z_up':[[list(mesh.data.corner_normals[i].vector) for i in t.loops] for t in mesh.data.loop_triangles],
                 'triangle_uv_blender':[[list(mesh.data.uv_layers.active.data[i].uv) for i in t.loops]
                                        for t in mesh.data.loop_triangles]}
    output.mkdir(parents=True)
    try:
        (output/'geometry-reference.json').write_text(json.dumps(reference)+'\n')
        bpy.ops.export_scene.gltf(filepath=str(output/'preview.glb'),export_format='GLB',
            export_yup=True,export_animations=False,export_skins=False,export_lights=False,
            export_cameras=False,export_vertex_color='NONE',export_materials='EXPORT',export_tangents=triplanar_shader_source is not None)
        if standard and not glass:standard_sampler_glb(output/'preview.glb')
        if glass:
            from add_terrain_house import write_glb
            from merge_static_glb import read
            doc,data=read(output/'preview.glb');m,=doc['materials'];m['alphaMode']='BLEND';m['doubleSided']=False
            m['pbrMetallicRoughness']['baseColorFactor']=material_plan['base_color_factor']
            (output/'preview.glb').unlink();write_glb(doc,data,output/'preview.glb')
        if triplanar_shader_source is not None:
            from add_terrain_house import write_glb
            from merge_static_glb import read
            doc,data=read(output/'preview.glb');doc['materials'][0]['extras']={'theia_triplanar':1}
            (output/'preview.glb').unlink();write_glb(doc,data,output/'preview.glb')
            dependencies={}
            for role,ref in material_plan['textures'].items():
                name=sources.by_guid[ref['guid']];original=sources.checked(name);meta=sources.metadata(name)['TextureImporter'];sampling=meta['textureSettings']
                effective=[sampling.get(k) for k in ('filterMode','wrapU','wrapV','aniso','mipBias')]
                if sampling=={'filterMode':-1,'aniso':-1,'mipBias':-1,'wrapMode':-1} and texture_reference is not None:
                    observed,=[v for v in texture_reference['textures'] if v['path']==name]
                    if observed['importer_type']!=meta['textureType'] or observed['srgb']!=bool(meta['mipmaps']['sRGBTexture']) or observed['convert_normal']:
                        raise ValueError('Editor texture importer differs from source profile')
                    effective=[observed[k] for k in ['filter','wrap_u','wrap_v','aniso','mip_bias']]
                if meta['textureType']!=(1 if role.endswith('normal') else 0) or meta['mipmaps']['sRGBTexture']!=(0 if role.endswith('normal') else 1) or meta.get('bumpmap',{}).get('convertToNormalMap') or effective!=[1,0,0,1,0]:raise ValueError('Triplanar source importer/sampler needs separate qualification')
                with Image.open(original) as image:
                    if image.format!='PNG' or max(image.size)>4096:raise ValueError('Triplanar texture exceeds PNG budget')
                filename='triplanar-'+role+'.png';shutil.copyfile(original,output/filename)
                dependencies[role]={'file':filename,'source':name,'sha256':hashlib.sha256(original.read_bytes()).hexdigest()}
            (output/'triplanar.json').write_text(json.dumps({'schema':1,'profile':material_plan['profile'],'tiling':material_plan['tiling'],'falloff':material_plan['falloff'],'geometry_sha256':hashlib.sha256((output/'preview.glb').read_bytes()).hexdigest(),'textures':dependencies,'scope':'bounded origin-zero source-world projection; no streaming/origin rebase or Unity shader parity'},indent=2)+'\n')
            if texture_reference is not None:
                config=json.loads((output/'triplanar.json').read_text())
                config['texture_reference']={k:texture_reference[k] for k in ['project','request_sha256','report_sha256','collector_sha256','unity_version']}
                (output/'triplanar.json').write_text(json.dumps(config,indent=2)+'\n')
        result = {'scope':'single source mesh, explicit standard PBR adaptation; no scene/collision/game adoption',
                  'scene_binding':scene_binding,'renderer':renderer if scene_binding else None,'canonical_mesh':bool(scene_binding),
                  'prefab':prefab,'model':model_name,'mesh_fileID':mesh_ref['fileID'],
                  'material':material_name,'base_color_texture':texture_name,'emissive_texture':emissive_name,'material_plan':material_plan,
                  'archive_sha256':sources.manifest['archive_sha256'],
                  'preview_sha256':hashlib.sha256((output/'preview.glb').read_bytes()).hexdigest(),
                  'source_vertices':len(mesh.data.vertices),'source_triangles':len(mesh.data.loop_triangles),
                  'blender':bpy.app.version_string,
                  'material_approximation':({'scope':material_plan['scope']} if standard else {'roughness':.8,'metallic':0,'omitted':'normal, metallic/smoothness, emission, shader-specific effects'}),
                  'full_scene_export':False,'game_adoption':False}
        (output/'conversion.json').write_text(json.dumps(result,indent=2)+'\n')
        return result
    except BaseException:
        shutil.rmtree(output);raise


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('--prefab',required=True)
    parser.add_argument('--output',required=True,type=Path)
    adaptation=parser.add_mutually_exclusive_group(required=True)
    adaptation.add_argument('--base-color-preview',action='store_true')
    adaptation.add_argument('--standard-material',action='store_true')
    parser.add_argument('--draw-plan',type=Path);parser.add_argument('--instance',type=int);parser.add_argument('--renderer',type=int)
    parser.add_argument('--triplanar-shader-source',type=Path)
    parser.add_argument('--triplanar-texture-reference',type=Path)
    parser.add_argument('--glass-blend',action='store_true',help='Explicit portable approximation of untextured source glass')
    args=parser.parse_args()
    if any(v is not None for v in [args.draw_plan,args.instance,args.renderer]) and any(v is None for v in [args.draw_plan,args.instance,args.renderer]):parser.error('Supply draw plan, instance and renderer together')
    print(json.dumps(preview(args.source,args.prefab,args.output,args.base_color_preview,args.standard_material,plan_path=args.draw_plan,instance=args.instance,renderer=args.renderer,triplanar_shader_source=args.triplanar_shader_source,triplanar_texture_reference=args.triplanar_texture_reference,glass=args.glass_blend),indent=2))
