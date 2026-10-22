"""Bind source renderer layers and static lights to a qualified forest inspector stage.

Uses standard Bevy light/fog concepts with explicit legacy-unit and flat-ambient
adaptations. No source scripts execute; normal gameplay adoption remains separate.
"""
import argparse
import collections
import hashlib
import json
import math
from pathlib import Path
import shutil
import numpy as np
from PIL import Image
from unity_scene import Sources, records
from unity_prefabs import local_matrix
from unity_lighting_reference import inspect
from unity_cookie_reference import inspect as inspect_cookie
from verify_forest_resources import verify as verify_resources
from merge_static_glb import read
from static_geometry import hierarchy
from add_terrain_house import write_glb


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def prepare(source, plan_path, resource_stage, reference, cookie_reference, output):
    sources=Sources(source);plan_path=Path(plan_path).resolve(strict=True);resource_stage=Path(resource_stage).resolve(strict=True);reference=Path(reference).resolve(strict=True);cookie_reference=Path(cookie_reference).resolve(strict=True);output=Path(output).resolve();build=Path(__file__).resolve().parents[1]/'build'
    if output.exists() or output==build or not output.is_relative_to(build) or any(output.is_relative_to(p) or p.is_relative_to(output) for p in [sources.root,resource_stage,reference,cookie_reference]):raise ValueError('Use a fresh ignored lighting stage outside inputs')
    original=verify_resources(resource_stage);checked=inspect(sources.root,reference);alpha_reference=inspect_cookie(sources.root,cookie_reference);plan=json.loads(plan_path.read_text())
    if plan['archive_sha256']!=sources.manifest['archive_sha256']:raise ValueError('Lighting plan source archive differs')
    scene=records(sources.checked(plan['scene']));sources.metadata(plan['scene']);variants={v['path']:v for v in checked['prefabs']};documents={};layers={};bindings=[]
    for draw in plan['draws']:
        prefab=draw['source'];renderer=draw['renderer'];instance=draw['instance']
        if prefab in variants:
            row,=[r for r in variants[prefab]['renderers'] if int(r['renderer']['file_id'])==renderer];go=int(row['game_object']['file_id']);layer=row['layer']
        else:
            if prefab not in documents:documents[prefab]=scene if prefab==plan['scene'] else records(sources.checked(prefab));sources.metadata(prefab)
            rr=documents[prefab];go=rr[renderer]['fields']['m_GameObject']['fileID'];layer=rr[go]['fields']['m_Layer']
        if instance is not None:
            guid=sources.manifest['selected'][prefab]['guid']
            for change in scene[instance]['fields']['m_Modification']['m_Modifications']:
                if change['target'].get('guid')==guid and change['target']['fileID']==go and change['propertyPath']=='m_Layer':
                    if change['objectReference']['fileID']:raise ValueError('Layer override must be scalar')
                    layer=int(change['value'])
        if type(layer) is not int or not 0<=layer<=31:raise ValueError('Source renderer layer exceeds 32-bit mask')
        identifier=str(instance)+':'+str(renderer)
        if identifier in layers:raise ValueError('Duplicate source renderer layer identity')
        layers[identifier]=layer;bindings.append({'source_instance':identifier,'source':prefab,'game_object':go,'layer':layer})
    document,binary=read(resource_stage/'preview.glb',allow_external_images=True);parents,world=hierarchy(document);roots={}
    for i,node in enumerate(document['nodes']):
        if 'source_instance' in node.get('extras',{}):
            identifier=node['extras']['source_instance']
            if identifier in roots:raise ValueError('Duplicate source instance root')
            roots[identifier]=i
    if set(roots)!=set(layers):raise ValueError('Lighting geometry/source layer coverage differs')
    for identifier,index in roots.items():document['nodes'][index]['extras']['theia_render_layer']=layers[identifier]
    root_nodes=set(roots.values());unowned=[]
    for node in world:
        if 'mesh' not in document['nodes'][node]:continue
        owner=node
        while parents[owner] is not None and owner not in root_nodes:owner=parents[owner]
        if owner not in root_nodes:unowned.append(node)
    terrain,=[v['fields'] for v in scene.values() if v['kind']==218];terrain_layer=scene[terrain['m_GameObject']['fileID']]['fields']['m_Layer']
    if len(unowned)!=1 or type(terrain_layer) is not int or not 0<=terrain_layer<=31:raise ValueError('Require exactly one qualified source terrain')
    node=unowned[0];mesh=document['meshes'][document['nodes'][node]['mesh']]
    if document['nodes'][node].get('name')!='source-terrain' or sum(document['accessors'][p['indices']]['count']//3 for p in mesh['primitives'])!=524288:raise ValueError('Unowned mesh is not the qualified terrain')
    document['nodes'][node].setdefault('extras',{})['theia_render_layer']=terrain_layer
    by_go={v['fields']['m_GameObject']['fileID']:i for i,v in scene.items() if v['kind']==4 and 'm_GameObject' in v['fields']};cache={0:(np.eye(4),True)}
    def transform(identifier,pending=()):
        if identifier in cache:return cache[identifier]
        if identifier in pending or len(pending)>128:raise ValueError('Light hierarchy cycle/depth')
        record=scene[identifier];f=record['fields']
        if record['stripped']:raise ValueError('Stripped light parents need separate instantiation')
        parent,enabled=transform(f['m_Father']['fileID'],pending+(identifier,));go=scene[f['m_GameObject']['fileID']]['fields'];result=parent@local_matrix(f)
        cache[identifier]=(result,enabled and bool(go['m_IsActive']));return cache[identifier]
    settings,=[v['fields'] for v in scene.values() if v['kind']==104]
    if settings['m_AmbientMode']!=1 or settings['m_FogMode']!=3 or settings['m_Fog']!=1:raise ValueError('Only the source trilight/exp-squared fog preview is qualified')
    ambient=[(settings['m_AmbientSkyColor'][c]+2*settings['m_AmbientEquatorColor'][c]+settings['m_AmbientGroundColor'][c])/4 for c in 'rgb']
    config={'schema':1,'profile':'unity-static-lighting-preview-v1','geometry_sha256':None,'layer_roots':len(roots)+1,
        'ambient':{'color':ambient,'brightness':3000*settings['m_AmbientIntensity']},
        'fog':{'color':[settings['m_FogColor'][c] for c in 'rgb'],'density':settings['m_FogDensity']},'cookies':[],'lights':[]}
    output.mkdir(parents=True)
    try:
        shutil.copytree(resource_stage/'shared',output/'shared')
        for file in resource_stage.glob('triplanar-*.png'):shutil.copyfile(file,output/file.name)
        cookies={};source_lights=[];skipped=[]
        for identifier,record in scene.items():
            if record['kind']!=108:continue
            f=record['fields'];frame,active=transform(by_go[f['m_GameObject']['fileID']]);active=active and bool(f['m_Enabled'])
            if not active:skipped.append(identifier);continue
            kind={0:'spot',1:'directional',2:'point'}.get(f['m_Type'])
            if kind is None or f.get('m_UseColorTemperature',0) or f.get('m_Lightmapping')!=4 or f.get('m_RenderingLayerMask',1)!=1 or f['m_Shadows'].get('m_UseCullingMatrixOverride',0):raise ValueError('Unsupported active source light feature')
            colour=[f['m_Color'][c] for c in 'rgb'];position=(frame[:3,3]*[-1,1,1]).tolist();direction=frame[:3,2]*[-1,1,1];up=frame[:3,1]*[-1,1,1];direction/=np.linalg.norm(direction);up/=np.linalg.norm(up)
            if abs(float(np.dot(direction,up)))>1e-8:raise ValueError('Sheared light orientation needs separate qualification')
            cookie=None;cookie_size=1.;cookie_ref=f.get('m_Cookie',{'fileID':0})
            if cookie_ref.get('fileID'):
                if kind=='point' or cookie_ref.get('fileID')!=2800000:raise ValueError('Only source 2D directional/spot cookies are qualified')
                guid=cookie_ref['guid']
                if guid not in cookies:
                    name=sources.by_guid[guid];observed,=[t for t in checked['textures'] if t['guid']==guid];metadata=sources.metadata(name)['TextureImporter'];raw=sources.checked(name)
                    if observed['format']!='Alpha8' or observed['alpha_source']!=2 or metadata['cookieLightType']!={'directional':1,'spot':0}[kind]:raise ValueError('Source cookie importer/type differs')
                    alpha,=[t for t in alpha_reference['textures'] if t['guid']==guid]
                    pixels=np.frombuffer((cookie_reference/'Reference'/alpha['alpha_file']).read_bytes(),dtype=np.uint8).reshape(alpha['height'],alpha['width'])
                    temporary=output/'cookie-temporary.png';Image.fromarray(pixels).save(temporary);digest=sha(temporary);filename='lighting-cookie-'+digest+'.png';temporary.rename(output/filename)
                    cookies[guid]=len(config['cookies']);config['cookies'].append({'file':filename,'sha256':digest,'dimensions':[observed['width'],observed['height']],'wrap':observed['wrap_u']})
                cookie=cookies[guid];cookie_size=float(f.get('m_CookieSize',1)) if kind=='directional' else 1.
            light={'id':str(identifier),'kind':kind,'position':position,'direction':direction.tolist(),'up':up.tolist(),'color':colour,
                'intensity':f['m_Intensity']*(10000 if kind=='directional' else 100000),'range':f['m_Range'],
                'mask':f['m_CullingMask']['m_Bits'],'shadows':f['m_Shadows']['m_Type']!=0,'near_z':f['m_Shadows']['m_NearPlane'],
                'cone':[math.radians(f['m_InnerSpotAngle']/2),math.radians(f['m_SpotAngle']/2)] if kind=='spot' else [0,0],
                'cookie':cookie,'cookie_size':cookie_size}
            config['lights'].append(light);source_lights.append({'id':identifier,'fields':f,'world_matrix':frame.tolist()})
        if not 1<=len(config['lights'])<=64 or len(config['cookies'])!=2:raise ValueError('Source light/cookie coverage exceeds bounded preview')
        write_glb(document,binary,output/'preview.glb');config['geometry_sha256']=sha(output/'preview.glb');metadata=json.loads((resource_stage/'triplanar.json').read_text());metadata['geometry_sha256']=config['geometry_sha256'];(output/'triplanar.json').write_text(json.dumps(metadata,indent=2)+'\n');(output/'lighting.json').write_text(json.dumps(config,indent=2,allow_nan=False)+'\n')
        result={'schema':1,'source':str(sources.root),'source_plan':str(plan_path),'source_plan_sha256':sha(plan_path),'archive_sha256':sources.manifest['archive_sha256'],'scene':plan['scene'],'scene_sha256':sha(sources.checked(plan['scene'])),
            'resource_stage':str(resource_stage),'source_asset_sha256':original['asset_sha256'],'reference_project':str(reference),'reference_report_sha256':checked['report_sha256'],'cookie_reference_project':str(cookie_reference),'cookie_reference_report_sha256':alpha_reference['report_sha256'],
            'asset_sha256':config['geometry_sha256'],'lighting_sha256':sha(output/'lighting.json'),'triplanar_sha256':sha(output/'triplanar.json'),
            'source_renderers':bindings,'terrain_layer':terrain_layer,'source_settings':settings,'source_lights':source_lights,'inactive_lights':skipped,
            'adaptations':{'ambient':'spherical mean of piecewise-linear source trilight, scaled by 3000 cd/m2 per source intensity unit','directional':'10000 lux per legacy source intensity unit','punctual':'100000 lumens per legacy source intensity unit','cookies':'actual imported Alpha8 base texels through linear Bevy R-channel masks; one preview mip','shadows':'source enabled state; Bevy default bias/filtering, no Unity shadow-strength/compression/BRDF parity','motion':'authored rest light poses; source scripts and animated light motion not applied'},
            'scope':'bounded source-layer/static-light/fog inspector preview; no Unity rendered parity, runtime rebase, collision or gameplay adoption'}
        (output/'lighting-source.json').write_text(json.dumps(result,indent=2,allow_nan=False)+'\n');return {'asset_sha256':config['geometry_sha256'],'renderer_layers':dict(collections.Counter(layers.values())),'terrain_layer':terrain_layer,'lights':dict(collections.Counter(l['kind'] for l in config['lights'])),'cookies':len(config['cookies'])}
    except BaseException:
        shutil.rmtree(output);raise


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    for name in ['source','plan','resources','reference','cookie_reference','output']:p.add_argument(name,type=Path)
    a=p.parse_args();print(json.dumps(prepare(a.source,a.plan,a.resources,a.reference,a.cookie_reference,a.output)))
