"""Use actual Editor sky mesh/indices and source world-height UVs in standard GLB."""
import argparse
import hashlib
import io
import json
from pathlib import Path
import numpy as np
from PIL import Image
from unity_scene import Sources,records
from unity_draw_bindings import select_draw
from unity_editor_reference import inspect
from unity_sky import material,lut
from add_terrain_house import write_glb
from unity_geometry import assemble
from verify_unity_geometry import assembly


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def convert(source,shaders,plan_path,instance,renderer,project,output,probe):
    sources=Sources(source);shaders=Sources(shaders);project=Path(project).resolve(strict=True);output=Path(output).resolve()
    if output.exists() or not output.is_relative_to(Path(__file__).resolve().parents[1]/'build') or any(output.is_relative_to(p) or p.is_relative_to(output) for p in [sources.root,shaders.root,project]):raise ValueError('Use a fresh ignored sky stage outside inputs')
    if sources.manifest['archive_sha256']!=shaders.manifest['archive_sha256']:raise ValueError('Sky source archives differ')
    plan_path=Path(plan_path).resolve(strict=True);plan=json.loads(plan_path.read_text());w,=[d for d in plan['draws'] if (d['instance'],d['renderer'])==(instance,renderer)]
    draw,static,binding=select_draw(sources,plan_path,instance,renderer,w['source'])
    if len(draw['materials'])!=1 or set(static['unimplemented_components'])-{'136'}:raise ValueError('Sky requires one static source material')
    rr=records(sources.checked(w['source']));flags=rr[renderer]['fields']
    if flags['m_CastShadows']!=0 or flags['m_ReceiveShadows']!=0:raise ValueError('Sky renderer must disable source shadows')
    name=sources.by_guid[draw['materials'][0]['guid']];sources.metadata(name);fields,=[v['fields'] for v in records(sources.checked(name)).values() if v['kind']==21]
    shader=shaders.by_guid[fields['m_Shader']['guid']];shaders.metadata(shader);profile=material(fields,sha(shaders.checked(shader)))
    model=sources.by_guid[draw['mesh']['guid']];checked=inspect(sources.root,project,probe,{model})
    if not checked['metadata_byte_identical'] or checked['recorded_frame_matches']!=checked['meshes_checked']:raise ValueError('Actual sky reference frame remains unqualified')
    imported=json.loads((project/'Reference/imported.json').read_text());record,=[m for m in imported['models'] if m['path']==model];mesh,=[m for m in record['meshes'] if m['file_id']==str(draw['mesh']['fileID'])]
    if len(mesh['submeshes'])!=1 or mesh['submeshes'][0]['topology']!=0:raise ValueError('Sky requires one triangulated Editor submesh')
    positions=np.array(mesh['positions'],dtype='<f4').reshape(-1,3);positions[:,0]*=-1
    indices=np.asarray(mesh['submeshes'][0]['indices'],dtype='<u4').reshape(-1,3)[:,[0,2,1]].reshape(-1,1)
    reflection=np.diag([-1,1,1,1]);world=reflection@np.asarray(w['matrix'],dtype=float)@reflection
    y=positions.astype(float)@world[1,:3]+world[1,3];height=profile['lut_height'];v=.5/height+(1-(profile['offset']+y)/profile['distance'])*(height-1)/height
    uv=np.stack([np.full(len(y),.5),v],axis=1).astype('<f4')
    packed=bytearray();views=[];accessors=[]
    for values,kind,component in [(positions,'VEC3',5126),(uv,'VEC2',5126),(indices,'SCALAR',5125)]:
        offset=len(packed);packed.extend(values.tobytes());views.append({'buffer':0,'byteOffset':offset,'byteLength':values.nbytes,'target':34963 if component==5125 else 34962});item={'bufferView':len(views)-1,'componentType':component,'type':kind,'count':len(values)}
        if kind=='VEC3':item.update(min=values.min(0).tolist(),max=values.max(0).tolist())
        accessors.append(item)
    image=io.BytesIO();Image.fromarray(lut(profile)).save(image,format='PNG');png=image.getvalue();view=len(views);views.append({'buffer':0,'byteOffset':len(packed),'byteLength':len(png)});packed.extend(png)
    doc={'asset':{'version':'2.0','generator':'Theia actual Editor sky + static unlit gradient'},'scene':0,'scenes':[{'nodes':[0]}],'nodes':[{'mesh':0}],
         'buffers':[{'byteLength':len(packed)}],'bufferViews':views,'accessors':accessors,'extensionsUsed':['KHR_materials_unlit'],'extensionsRequired':['KHR_materials_unlit'],
         'images':[{'bufferView':view,'mimeType':'image/png'}],'samplers':[{'wrapS':33071,'wrapT':33071,'magFilter':9729,'minFilter':9729}],
         'textures':[{'source':0,'sampler':0}],'materials':[{'name':'source-world-height-sky','extensions':{'KHR_materials_unlit':{}},'extras':{'theia_render_flags':{'cast_shadows':False,'receive_shadows':False}},
             'pbrMetallicRoughness':{'baseColorFactor':[1,1,1,1],'baseColorTexture':{'index':0,'texCoord':0},'metallicFactor':0,'roughnessFactor':1},'doubleSided':False,'alphaMode':'OPAQUE'}],
         'meshes':[{'primitives':[{'attributes':{'POSITION':0,'TEXCOORD_0':1},'indices':2,'material':0}]}]}
    packed.extend(b'\0'*(-len(packed)%4))
    output.mkdir(parents=True);write_glb(doc,packed,output/'preview.glb')
    instances=[{'id':str(instance)+':'+str(renderer),'matrix':w['matrix']}];placed=assemble(output/'preview.glb',instances,output/'placements.glb');reference=assembly(output/'preview.glb',instances,output/'placements.glb')
    (output/'assembly.json').write_text(json.dumps(placed,indent=2)+'\n');(output/'assembly-reference.json').write_text(json.dumps(reference,indent=2)+'\n')
    key=hashlib.sha256(json.dumps([w['mesh'],w['materials']],sort_keys=True,separators=(',',':')).encode()).hexdigest()
    result={'schema':1,'profile':profile,'key':key,'mesh':w['mesh'],'materials':w['materials'],'draws':1,'model':model,'mesh_fileID':w['mesh']['fileID'],
            'prefab':w['source'],'renderer':renderer,'material':name,'scene_binding':binding,'source_plan':str(plan_path),'source_plan_sha256':sha(plan_path),'archive_sha256':sources.manifest['archive_sha256'],
            'project':str(project),'request_sha256':sha(project/'Reference/request.json'),'editor_report_sha256':sha(project/'Reference/imported.json'),
            'prototype_sha256':sha(output/'preview.glb'),'placements_sha256':placed['asset_sha256'],'lut_sha256':hashlib.sha256(png).hexdigest(),
            'scope':'one source sky placement, actual Editor positions/indices, source world-height gradient and disabled shadows; inspector presentation, no collision/adoption'}
    (output/'sky.json').write_text(json.dumps(result,indent=2)+'\n');(output/'editor-reference.json').write_text(json.dumps(checked,indent=2)+'\n');return result


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['source','shaders','plan','project','output','probe']:p.add_argument(name,type=Path)
    p.add_argument('--instance',type=int,required=True);p.add_argument('--renderer',type=int,required=True);a=p.parse_args()
    print(json.dumps(convert(a.source,a.shaders,a.plan,a.instance,a.renderer,a.project,a.output,a.probe)))
