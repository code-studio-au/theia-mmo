"""Preserve two source material slots in canonical crystal/Standard prototypes."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import bpy
from mathutils import Matrix
from unity_scene import Sources,records
from unity_draw_bindings import select_draw
from unity_model_ids import identity,bound_candidates
from unity_fbx_static import require_identity_geometry
from unity_materials import standard_material,standard_sampler_glb
from unity_crystal import material as crystal_material,GUID
from merge_static_glb import read
from add_terrain_house import write_glb


def convert(source,shaders,plan_path,instance,renderer,output,topology_reference=None,probe=None):
    sources=Sources(source);shaders=Sources(shaders);output=Path(output).resolve()
    if output.exists() or not output.is_relative_to(Path(__file__).resolve().parents[1]/'build'):raise ValueError('Use a fresh ignored mixed-material output')
    if any(output.is_relative_to(root) or root.is_relative_to(output) for root in [sources.root,shaders.root]):raise ValueError('Mixed output overlaps immutable source inputs')
    if sources.manifest['archive_sha256']!=shaders.manifest['archive_sha256']:raise ValueError('Source/shader archives differ')
    planned=json.loads(Path(plan_path).read_text());witness,=[d for d in planned['draws'] if (d['instance'],d['renderer'])==(instance,renderer)]
    draw,static,binding=select_draw(sources,plan_path,instance,renderer,witness['source'])
    if set(static['unimplemented_components'])-{'136'} or len(draw['materials'])!=2 or draw['materials'][0]==draw['materials'][1]:raise ValueError('Select a two-slot static source renderer')
    plans=[];families=[]
    for ref in draw['materials']:
        if ref.get('fileID')!=2100000:raise ValueError('Invalid source material identity')
        name=sources.by_guid[ref['guid']];sources.metadata(name);fields,=[v['fields'] for v in records(sources.checked(name)).values() if v['kind']==21]
        guid=fields['m_Shader'].get('guid')
        if guid==GUID:
            shader=shaders.by_guid[guid];shaders.metadata(shader);plan=crystal_material(fields,hashlib.sha256(shaders.checked(shader).read_bytes()).hexdigest());family='crystal'
        else:plan=standard_material(fields);family='standard'
        plans.append(dict(plan,source_material=name));families.append(family)
    if sorted(families)!=['crystal','standard']:raise ValueError('Mixed profile requires one crystal and one opaque Standard slot')
    model_name=sources.by_guid[draw['mesh']['guid']];model=sources.checked(model_name);importer=sources.metadata(model_name)['ModelImporter'];settings=importer['meshes']
    if settings['globalScale']!=1 or settings['useFileUnits']!=1 or settings['bakeAxisConversion']!=0:raise ValueError('Unqualified mixed model units/axis policy')
    ids,_=bound_candidates(identity(model),importer);selected=ids[draw['mesh']['fileID']];require_identity_geometry(model,selected['model'])
    bpy.ops.wm.read_factory_settings(use_empty=True);bpy.ops.import_scene.fbx(filepath=str(model),use_anim=False,use_custom_props=False,bake_space_transform=True)
    mesh,=[o for o in bpy.data.objects if o.type=='MESH' and o.name==selected['name']]
    if mesh.modifiers or mesh.data.shape_keys:raise ValueError('Deformed mixed source mesh')
    mesh.parent=None;mesh.matrix_world=Matrix.Identity(4)
    bpy.ops.object.select_all(action='DESELECT');mesh.select_set(True);bpy.context.view_layer.objects.active=mesh;bpy.ops.object.transform_apply(location=True,rotation=True,scale=True)
    for o in list(bpy.data.objects):
        if o!=mesh:bpy.data.objects.remove(o,do_unlink=True)
    topology=None;source_normals=None
    if topology_reference is not None:
        if probe is None:raise ValueError('Editor topology requires independent frame validation')
        from verify_unity_reframe import verify as verify_frame
        from unity_editor_topology import triangulate
        prior=Path(topology_reference).resolve(strict=True);verified=verify_frame(sources.root,prior,probe,shaders.root)
        receipt=json.loads((prior/'unity-frame.json').read_text())
        if receipt['model']!=model_name or receipt['mesh_file_id']!=str(draw['mesh']['fileID']):raise ValueError('Editor topology reference selects another mesh')
        imported=json.loads((Path(receipt['project'])/'Reference/imported.json').read_text());model_ref,=[m for m in imported['models'] if m['path']==model_name];mesh_ref,=[m for m in model_ref['meshes'] if m['file_id']==str(draw['mesh']['fileID'])]
        old=mesh.data;mesh.data,topology,source_normals=triangulate(old,mesh_ref,receipt['canonical_to_render_mesh'])
        topology.update(frame_reference=str(prior),frame_asset_sha256=verified['asset_sha256'],editor_report_sha256=receipt['editor_report_sha256'])
        if old.users==0:bpy.data.meshes.remove(old)
    slots=[p.material_index for p in mesh.data.polygons]
    if set(slots)!={0,1}:raise ValueError('Source mesh does not use both expected material slots')
    mesh.data.materials.clear();dependencies=[]
    output.mkdir(parents=True)
    try:
        for index,(family,plan) in enumerate(zip(families,plans)):
            m=bpy.data.materials.new('source-slot-'+str(index));m.use_nodes=True;m.use_backface_culling=family=='standard'
            shader=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
            shader.inputs['Metallic'].default_value=plan['metallic'];shader.inputs['Roughness'].default_value=plan['roughness'];shader.inputs['Emission Color'].default_value=(*plan['emissive_factor'],1.)
            shader.inputs['Emission Strength'].default_value=1.
            if family=='crystal':shader.inputs['Base Color'].default_value=plan['base_color_factor'];shader.inputs['Alpha'].default_value=plan['base_color_factor'][3]
            else:
                for role,ref in [('base_color',plan['base_color']),('emissive',plan['emissive'])]:
                    if ref is None:continue
                    name=sources.by_guid[ref['guid']];path=sources.checked(name);meta=sources.metadata(name)['TextureImporter'];sampler=meta['textureSettings']
                    if path.suffix.lower()!='.png' or meta['mipmaps']['sRGBTexture']!=1 or meta['mipmaps'].get('enableMipMap')!=1 or [sampler.get(k) for k in ['filterMode','wrapU','wrapV','aniso','mipBias']]!=[1,0,0,1,0]:raise ValueError('Unsupported mixed Standard image/sampler')
                    image=m.node_tree.nodes.new('ShaderNodeTexImage');image.image=bpy.data.images.load(str(path));image.image.colorspace_settings.name='sRGB'
                    m.node_tree.links.new(image.outputs['Color'],shader.inputs['Base Color' if role=='base_color' else 'Emission Color'])
                    dependencies.append({'slot':index,'role':role,'source':name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()})
            mesh.data.materials.append(m)
        for polygon,slot in zip(mesh.data.polygons,slots):polygon.material_index=slot
        mesh.data.calc_loop_triangles()
        reference={'vertices_z_up_metres':[list(v.co) for v in mesh.data.vertices],
                   'triangles':[list(t.vertices) for t in mesh.data.loop_triangles],
                   'triangle_uv_blender':[[list(mesh.data.uv_layers.active.data[i].uv) for i in t.loops] for t in mesh.data.loop_triangles],
                   'triangle_normals_z_up':[[list(source_normals[i] if source_normals is not None else mesh.data.corner_normals[i].vector) for i in t.loops] for t in mesh.data.loop_triangles],
                   'triangle_slots':[slots[t.polygon_index] for t in mesh.data.loop_triangles]}
        if topology is not None:
            if len(mesh.data.uv_layers)>2:raise ValueError('Editor corner path supports two source UV channels')
            if len(mesh.data.uv_layers)==2:
                secondary,=[layer for layer in mesh.data.uv_layers if layer.name!=mesh.data.uv_layers.active.name]
                reference['triangle_uv1_blender']=[[list(secondary.data[i].uv) for i in t.loops] for t in mesh.data.loop_triangles]
        (output/'geometry-reference.json').write_text(json.dumps(reference)+'\n')
        bpy.ops.export_scene.gltf(filepath=str(output/'preview.glb'),export_format='GLB',export_yup=True,export_animations=False,export_skins=False,export_lights=False,export_cameras=False,export_vertex_color='NONE',export_materials='EXPORT')
        standard_sampler_glb(output/'preview.glb');doc,data=read(output/'preview.glb')
        for m in doc['materials']:
            if not m['name'].startswith('source-slot-'):raise ValueError('Unknown mixed output material')
            slot=int(m['name'][len('source-slot-'):]);plan=plans[slot];m['extras']={'theia_source_slot':slot}
            m['pbrMetallicRoughness']['metallicFactor']=plan['metallic'];m['pbrMetallicRoughness']['roughnessFactor']=plan['roughness']
            m['emissiveFactor']=plan['emissive_factor']
            if families[slot]=='crystal':m['pbrMetallicRoughness']['baseColorFactor']=plan['base_color_factor'];m['alphaMode']='BLEND';m['doubleSided']=True
            else:m['alphaMode']='OPAQUE';m['doubleSided']=False
        if topology is not None:
            from unity_corner_streams import write as write_corners
            doc,data,corners=write_corners(doc,data,reference);topology['corner_streams']=corners
        (output/'preview.glb').unlink();write_glb(doc,data,output/'preview.glb')
        result={'profile':'unity-mixed-crystal-pbr-static-v1','material_plans':plans,'material_refs':draw['materials'],
                'archive_sha256':sources.manifest['archive_sha256'],'prefab':witness['source'],'renderer':renderer,
                'model':model_name,'mesh_fileID':draw['mesh']['fileID'],'canonical_mesh':True,'scene_binding':binding,
                'dependencies':dependencies,'source_triangles':len(reference['triangles']),
                'preview_sha256':hashlib.sha256((output/'preview.glb').read_bytes()).hexdigest(),
                'scope':'two actual source slots; explicit Standard/crystal static PBR preview, not Unity refraction/view-dependent shader equivalence or collision'}
        if topology is not None:result['editor_triangulation']=topology
        (output/'conversion.json').write_text(json.dumps(result,indent=2)+'\n');return result
    except BaseException:shutil.rmtree(output);raise


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['source','shaders','plan','output']:p.add_argument(name,type=Path)
    p.add_argument('--instance',type=int,required=True);p.add_argument('--renderer',type=int,required=True)
    p.add_argument('--editor-topology-reference',type=Path);p.add_argument('--probe',type=Path)
    a=p.parse_args();print(json.dumps(convert(a.source,a.shaders,a.plan,a.instance,a.renderer,a.output,a.editor_topology_reference,a.probe)))
