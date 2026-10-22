"""Convert one exact water binding from actual Editor triangle geometry."""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
from add_terrain_house import write_glb
from unity_scene import Sources, records
from unity_draw_bindings import select_draw
from unity_editor_reference import inspect
from unity_builtin_plane_reference import inspect as inspect_plane, GUID as PLANE_GUID
from unity_water import material
from unity_geometry import assemble
from verify_unity_geometry import assembly


def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def convert(source, shaders, plan_path, instance, renderer, project, output, probe):
    source = Sources(source); shaders = Sources(shaders); project = Path(project).resolve(strict=True); output = Path(output).resolve(); plan_path = Path(plan_path).resolve(strict=True)
    if (output.exists() or not output.is_relative_to(Path(__file__).resolve().parents[1]/'build') or
            any(output.is_relative_to(p) or p.is_relative_to(output) for p in [source.root, shaders.root, project])):
        raise ValueError('Use a fresh ignored water stage outside inputs')
    plan = json.loads(plan_path.read_text())
    if plan['archive_sha256'] != source.manifest['archive_sha256'] or shaders.manifest['archive_sha256'] != source.manifest['archive_sha256']:
        raise ValueError('Water archive identities differ')
    witness, = [d for d in plan['draws'] if (d['instance'],d['renderer']) == (instance,renderer)]
    if instance is not None:
        draw, static, binding = select_draw(source, plan_path, instance, renderer, witness['source'])
        if set(static['unimplemented_components']) - {'136'}: raise ValueError('Unsupported water component')
    else:
        if witness['source'] != plan['scene'] or witness['mesh'] != {'fileID':10209,'guid':PLANE_GUID,'type':0}: raise ValueError('Only the built-in inline Plane is qualified')
    rr = records(source.checked(witness['source'])); source.metadata(witness['source']); flags = rr[renderer]['fields']
    cast, receive = flags['m_CastShadows'], flags['m_ReceiveShadows']
    if instance is not None:
        scene = records(source.checked(plan['scene'])); guid = source.manifest['selected'][witness['source']]['guid']
        for m in scene[instance]['fields']['m_Modification']['m_Modifications']:
            if m['target'].get('guid') == guid and m['target']['fileID'] == renderer:
                if m['propertyPath'] == 'm_CastShadows': cast = int(m['value'])
                if m['propertyPath'] == 'm_ReceiveShadows': receive = int(m['value'])
    if cast not in (0,1) or receive not in (0,1) or len(witness['materials']) != 1: raise ValueError('Unqualified water slots/shadow state')
    name = source.by_guid[witness['materials'][0]['guid']]; source.metadata(name)
    fields, = [v['fields'] for v in records(source.checked(name)).values() if v['kind'] == 21]
    shader_name = shaders.by_guid[fields['m_Shader']['guid']]; shaders.metadata(shader_name); profile = material(fields,sha(shaders.checked(shader_name)))
    if witness['mesh']['guid'] == PLANE_GUID:
        reference = inspect_plane(project); imported = json.loads((project/'Reference/plane.json').read_text()); model = None
        vertices = np.array(imported['positions'],dtype='<f4').reshape(-1,3); indices = np.array(imported['indices'],dtype='<u4').reshape(-1,3)
        report_path = project/'Reference/plane.json'
    else:
        model = source.by_guid[witness['mesh']['guid']]; reference = inspect(source.root,project,probe,{model})
        if not reference['metadata_byte_identical'] or reference['recorded_frame_matches'] != reference['meshes_checked']: raise ValueError('Water Editor frame remains unqualified')
        imported = json.loads((project/'Reference/imported.json').read_text()); record, = [m for m in imported['models'] if m['path'] == model]; mesh, = [m for m in record['meshes'] if m['file_id'] == str(witness['mesh']['fileID'])]
        submesh, = mesh['submeshes']
        if submesh['topology'] != 0: raise ValueError('Water requires triangle topology')
        vertices = np.array(mesh['positions'],dtype='<f4').reshape(-1,3); indices = np.array(submesh['indices'],dtype='<u4').reshape(-1,3)
        report_path = project/'Reference/imported.json'
    vertices[:,0] *= -1; indices = indices[:,[0,2,1]]
    # Flat corner normals are derived from exact geometry, not claimed as imported normals.
    corners = vertices[indices].reshape(-1,3); triangle = corners.astype(float).reshape(-1,3,3)
    normals = np.cross(triangle[:,1]-triangle[:,0],triangle[:,2]-triangle[:,0]); lengths = np.linalg.norm(normals,axis=1)
    if not np.isfinite(normals).all() or (lengths < 1e-12).any(): raise ValueError('Degenerate water triangles')
    normals = np.repeat(normals/lengths[:,None],3,axis=0).astype('<f4'); sequential = np.arange(len(corners),dtype='<u4').reshape(-1,1)
    binary = bytearray(); views = []; accessors = []
    for values,kind,component in [(corners,'VEC3',5126),(normals,'VEC3',5126),(sequential,'SCALAR',5125)]:
        views.append({'buffer':0,'byteOffset':len(binary),'byteLength':values.nbytes,'target':34963 if component == 5125 else 34962}); binary.extend(values.tobytes())
        item = {'bufferView':len(views)-1,'componentType':component,'type':kind,'count':len(values)}
        if len(accessors) == 0: item.update(min=values.min(0).tolist(),max=values.max(0).tolist())
        accessors.append(item)
    doc = {'asset':{'version':'2.0','generator':'Theia source Editor water geometry + explicit static PBR preview'},'scene':0,'scenes':[{'nodes':[0]}],'nodes':[{'mesh':0}],
        'buffers':[{'byteLength':len(binary)}],'bufferViews':views,'accessors':accessors,
        'materials':[{'name':'source-static-water','pbrMetallicRoughness':{'baseColorFactor':profile['base_color_factor'],'metallicFactor':0,'roughnessFactor':profile['roughness']},'alphaMode':'BLEND','doubleSided':False,
            'extras':{'theia_render_flags':{'cast_shadows':bool(cast),'receive_shadows':bool(receive)}}}],
        'meshes':[{'primitives':[{'attributes':{'POSITION':0,'NORMAL':1},'indices':2,'material':0}]}]}
    output.mkdir(parents=True); write_glb(doc,binary,output/'preview.glb')
    selected = [d for d in plan['draws'] if [d['mesh'],d['materials']] == [witness['mesh'],witness['materials']]]
    instances = [{'id':str(d['instance'])+':'+str(d['renderer']),'matrix':d['matrix']} for d in selected]
    placed = assemble(output/'preview.glb',instances,output/'placements.glb'); checked = assembly(output/'preview.glb',instances,output/'placements.glb')
    for filename,value in [('assembly.json',placed),('assembly-reference.json',checked),('editor-reference.json',reference),('instances.json',{'instances':instances})]: (output/filename).write_text(json.dumps(value,indent=2)+'\n')
    subset = dict(plan,draws=selected); (output/'source-subset.json').write_text(json.dumps(subset,indent=2)+'\n')
    key = hashlib.sha256(json.dumps([witness['mesh'],witness['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest()
    result = {'schema':1,'profile':profile,'key':key,'mesh':witness['mesh'],'materials':witness['materials'],'draws':len(selected),'model':model,'material':name,
        'source_plan':str(plan_path),'source_plan_sha256':sha(plan_path),'archive_sha256':source.manifest['archive_sha256'],'project':str(project),
        'request_sha256':sha(project/'Reference/request.json'),'editor_report_sha256':sha(report_path),'prototype_sha256':sha(output/'preview.glb'),'placements_sha256':sha(output/'placements.glb'),
        'instance':instance,'renderer':renderer,'prefab':witness['source'],'cast_shadows':bool(cast),'receive_shadows':bool(receive),
        'scope':profile['scope']+'; exact source geometry/placements, no collision or map adoption'}
    (output/'water.json').write_text(json.dumps(result,indent=2)+'\n'); return result


if __name__ == '__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['source','shaders','plan','project','output','probe']: p.add_argument(name,type=Path)
    p.add_argument('--instance',type=int); p.add_argument('--renderer',type=int,required=True); a=p.parse_args()
    print(json.dumps(convert(a.source,a.shaders,a.plan,a.instance,a.renderer,a.project,a.output,a.probe)))
