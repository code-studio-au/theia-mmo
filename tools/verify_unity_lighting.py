"""Independent raw source layer/light/fog and unchanged inspector geometry checks."""
import argparse
import collections
import copy
import hashlib
import json
import math
from pathlib import Path
import numpy as np
from PIL import Image
from unity_scene import Sources,records
from unity_lighting_reference import inspect
from unity_cookie_reference import inspect as inspect_cookie
from verify_forest_resources import verify as verify_resources
from merge_static_glb import read
from static_geometry import accessor,hierarchy
from shared_images import collect


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def verify(stage):
    import bpy  # Registers the bundled mathutils module; no source scripts run.
    from mathutils import Matrix,Vector,Quaternion
    stage=Path(stage).resolve(strict=True);meta=json.loads((stage/'lighting-source.json').read_text());source=Sources(meta['source']);plan_path=Path(meta['source_plan']);original=Path(meta['resource_stage'])
    prior=verify_resources(original);reference=inspect(source.root,meta['reference_project']);alpha_reference=inspect_cookie(source.root,meta['cookie_reference_project'])
    if meta['schema']!=1 or meta['archive_sha256']!=source.manifest['archive_sha256'] or sha(plan_path)!=meta['source_plan_sha256'] or prior['asset_sha256']!=meta['source_asset_sha256'] or reference['report_sha256']!=meta['reference_report_sha256'] or alpha_reference['report_sha256']!=meta['cookie_reference_report_sha256']:raise ValueError('Lighting source/reference binding differs')
    if sha(stage/'preview.glb')!=meta['asset_sha256'] or sha(stage/'lighting.json')!=meta['lighting_sha256'] or sha(stage/'triplanar.json')!=meta['triplanar_sha256']:raise ValueError('Lighting output binding differs')
    plan=json.loads(plan_path.read_text());scene=records(source.checked(plan['scene']));source.metadata(plan['scene'])
    if plan['archive_sha256']!=source.manifest['archive_sha256'] or plan['scene']!=meta['scene'] or sha(source.checked(plan['scene']))!=meta['scene_sha256']:raise ValueError('Lighting scene identity differs')
    variants={v['path']:v for v in reference['prefabs']};prefabs={};expected_layers={};rows=[]
    for d in plan['draws']:
        if d['source'] in variants:
            row,=[r for r in variants[d['source']]['renderers'] if r['renderer']['file_id']==str(d['renderer'])];owner=int(row['game_object']['file_id']);layer=row['layer']
        else:
            if d['source'] not in prefabs:prefabs[d['source']]=scene if d['source']==plan['scene'] else records(source.checked(d['source']));source.metadata(d['source'])
            rr=prefabs[d['source']];owner=rr[d['renderer']]['fields']['m_GameObject']['fileID'];layer=rr[owner]['fields']['m_Layer']
        if d['instance'] is not None:
            guid=source.manifest['selected'][d['source']]['guid'];changes=scene[d['instance']]['fields']['m_Modification']['m_Modifications']
            for m in changes:
                if (m['target'].get('guid'),m['target']['fileID'],m['propertyPath'])==(guid,owner,'m_Layer'):
                    if m['objectReference']['fileID']:raise ValueError('Reference layer override is not scalar')
                    layer=int(m['value'])
        if type(layer) is not int or not 0<=layer<=31:raise ValueError('Reference source layer is invalid')
        key=str(d['instance'])+':'+str(d['renderer']);expected_layers[key]=layer;rows.append({'source_instance':key,'source':d['source'],'game_object':owner,'layer':layer})
    if rows!=meta['source_renderers'] or len(expected_layers)!=len(plan['draws']):raise ValueError('Source renderer layer membership differs')
    before,old_binary=read(original/'preview.glb',allow_external_images=True);doc,binary=read(stage/'preview.glb',allow_external_images=True);parents,world=hierarchy(doc);restore=copy.deepcopy(doc);found={};unowned=[];roots=set()
    for i,node in enumerate(doc['nodes']):
        extra=node.get('extras',{})
        if 'source_instance' in extra:
            key=extra['source_instance']
            if key in found or extra.get('theia_render_layer')!=expected_layers.get(key) or type(extra.get('theia_render_layer')) is not int:raise ValueError('GLB source render layer differs')
            found[key]=i;roots.add(i)
    if set(found)!=set(expected_layers):raise ValueError('GLB source layer coverage differs')
    for n in world:
        if 'mesh' not in doc['nodes'][n]:continue
        owner=n
        while parents[owner] is not None and owner not in roots:owner=parents[owner]
        if owner not in roots:unowned.append(n)
    terrain,=[v['fields'] for v in scene.values() if v['kind']==218];terrain_layer=scene[terrain['m_GameObject']['fileID']]['fields']['m_Layer']
    if len(unowned)!=1 or meta['terrain_layer']!=terrain_layer or doc['nodes'][unowned[0]].get('name')!='source-terrain' or doc['nodes'][unowned[0]].get('extras',{}).get('theia_render_layer')!=terrain_layer:raise ValueError('Terrain source layer differs')
    marked=set(found.values())|set(unowned)
    for n,node in enumerate(restore['nodes']):
        if 'theia_render_layer' in node.get('extras',{}):
            if n not in marked:raise ValueError('Unexpected source layer marker')
            node['extras'].pop('theia_render_layer')
            if not node['extras']:node.pop('extras')
    if restore!=before or binary!=old_binary:raise ValueError('Lighting preparation changed geometry/materials/images beyond source layer extras')
    canonical=copy.deepcopy(doc)
    for image in canonical['images']:image['uri']='../'+image['uri']
    descriptors=collect(canonical,stage,allow_material_extensions=True)
    original_receipt=json.loads((original/'resources.json').read_text())
    if descriptors!=original_receipt['shared_images']:raise ValueError('Lighting shared image variants changed')
    triplanar=json.loads((original/'triplanar.json').read_text());triplanar['geometry_sha256']=meta['asset_sha256']
    if json.loads((stage/'triplanar.json').read_text())!=triplanar:raise ValueError('Lighting triplanar metadata changed')
    for p in triplanar['profiles']:
        for t in p['textures'].values():
            if (stage/t['file']).read_bytes()!=(original/t['file']).read_bytes():raise ValueError('Lighting triplanar image bytes changed')
    config=json.loads((stage/'lighting.json').read_text());settings,=[r['fields'] for r in scene.values() if r['kind']==104]
    if meta['source_settings']!=settings or config['schema']!=1 or config['profile']!='unity-static-lighting-preview-v1' or config['geometry_sha256']!=meta['asset_sha256'] or config['layer_roots']!=len(expected_layers)+1:raise ValueError('Lighting settings/geometry/schema binding differs')
    wanted=[.25*settings['m_AmbientSkyColor'][c]+.5*settings['m_AmbientEquatorColor'][c]+.25*settings['m_AmbientGroundColor'][c] for c in 'rgb']
    if settings['m_AmbientMode']!=1 or not np.allclose(config['ambient']['color'],wanted,atol=1e-15,rtol=0) or config['ambient']['brightness']!=3000*settings['m_AmbientIntensity'] or config['fog']!={'color':[settings['m_FogColor'][c] for c in 'rgb'],'density':settings['m_FogDensity']} or settings['m_FogMode']!=3 or settings['m_Fog']!=1:raise ValueError('Source ambient/fog adaptation differs')
    transforms={r['fields']['m_GameObject']['fileID']:i for i,r in scene.items() if r['kind']==4 and 'm_GameObject' in r['fields']};matrices={0:Matrix.Identity(4)};enabled={0:True}
    def world_matrix(i,visiting=()):
        if i in matrices:return matrices[i]
        if i in visiting or len(visiting)>128 or scene[i]['stripped']:raise ValueError('Unqualified reference light parent')
        f=scene[i]['fields'];p=f['m_LocalPosition'];s=f['m_LocalScale'];q=f['m_LocalRotation'];parent=f['m_Father']['fileID']
        local=Matrix.LocRotScale(Vector(tuple(p[c] for c in 'xyz')),Quaternion(tuple(q[c] for c in 'wxyz')).normalized(),Vector(tuple(s[c] for c in 'xyz')))
        matrices[i]=world_matrix(parent,visiting+(i,))@local;enabled[i]=enabled[parent] and bool(scene[f['m_GameObject']['fileID']]['fields']['m_IsActive']);return matrices[i]
    lights={v['id']:v for v in config['lights']};recorded={str(v['id']):v for v in meta['source_lights']};expected_ids=set();references={v['guid']:v for v in reference['textures']};cookie_sources={};max_pose=0.;skipped=[]
    for i,r in scene.items():
        if r['kind']!=108:continue
        f=r['fields'];transform=transforms[f['m_GameObject']['fileID']];matrix=np.asarray(world_matrix(transform),dtype=float)
        if not enabled[transform] or not f['m_Enabled']:skipped.append(i);continue
        key=str(i);expected_ids.add(key);l=lights[key]
        if key not in recorded or recorded[key]['fields']!=f or np.max(abs(matrix-np.asarray(recorded[key]['world_matrix'])))>.0001:raise ValueError('Recorded raw light provenance differs')
        kind={0:'spot',1:'directional',2:'point'}[f['m_Type']]
        position=matrix[:3,3]*[-1,1,1];direction=matrix[:3,2]*[-1,1,1];direction/=np.linalg.norm(direction);up=matrix[:3,1]*[-1,1,1];up/=np.linalg.norm(up)
        difference=max(float(abs(position-l['position']).max()),float(abs(direction-l['direction']).max()),float(abs(up-l['up']).max()));max_pose=max(max_pose,difference)
        if difference>.0001 or l['kind']!=kind or l['color']!=[f['m_Color'][c] for c in 'rgb'] or l['mask']!=f['m_CullingMask']['m_Bits'] or l['intensity']!=f['m_Intensity']*(10000 if kind=='directional' else 100000) or l['range']!=f['m_Range'] or l['shadows']!=(f['m_Shadows']['m_Type']!=0) or l['near_z']!=f['m_Shadows']['m_NearPlane']:raise ValueError('Source light pose/colour/mask/photometric adaptation differs')
        cone=[f['m_InnerSpotAngle']*math.pi/360,f['m_SpotAngle']*math.pi/360] if kind=='spot' else [0,0]
        if not np.allclose(l['cone'],cone,atol=1e-15,rtol=0):raise ValueError('Source spot cone differs')
        raw_cookie=f.get('m_Cookie',{'fileID':0})
        if raw_cookie.get('fileID'):
            if l['cookie'] in cookie_sources and cookie_sources[l['cookie']]!=raw_cookie['guid']:raise ValueError('Source cookie/light role alias differs')
            cookie_sources[l['cookie']]=raw_cookie['guid'];size=f['m_CookieSize'] if kind=='directional' else 1
            if l['cookie_size']!=size:raise ValueError('Source directional cookie scale differs')
        elif l['cookie'] is not None or l['cookie_size']!=1:raise ValueError('Unexpected source cookie')
    if set(lights)!=expected_ids or set(recorded)!=expected_ids or len(recorded)!=len(meta['source_lights']) or len(lights)!=len(config['lights']) or skipped!=meta['inactive_lights'] or set(cookie_sources)!=set(range(len(config['cookies']))):raise ValueError('Source active light/cookie coverage differs')
    for index,guid in cookie_sources.items():
        name=source.by_guid[guid];observed=references[guid];cookie=config['cookies'][index];path=stage/cookie['file']
        if cookie['file']!='lighting-cookie-'+cookie['sha256']+'.png' or sha(path)!=cookie['sha256'] or cookie['wrap']!=observed['wrap_u'] or cookie['dimensions']!=[observed['width'],observed['height']]:raise ValueError('Source cookie image/sampler binding differs')
        alpha,=[t for t in alpha_reference['textures'] if t['guid']==guid]
        values=np.frombuffer((Path(alpha_reference['project'])/'Reference'/alpha['alpha_file']).read_bytes(),dtype=np.uint8).reshape(alpha['height'],alpha['width'])
        with Image.open(path) as image:
            if image.mode!='L' or image.format!='PNG' or not np.array_equal(np.array(image),values):raise ValueError('Actual imported source cookie alpha pixels differ')
    return {'passed':True,'asset_sha256':meta['asset_sha256'],'source_asset_sha256':meta['source_asset_sha256'],'source_draws':len(expected_layers),'layer_roots':len(marked),'source_layer_counts':dict(collections.Counter(expected_layers.values())),'terrain_layer':terrain_layer,'lights':dict(collections.Counter(v['kind'] for v in lights.values())),'cookies':len(cookie_sources),'max_reference_pose_difference':max_pose,'mesh_instances':prior['mesh_instances'],'triangles':prior['triangles'],'expected_bounds':prior['expected_bounds'],'scope':'independent source hierarchy/layers/poses/colours/masks/static photometric preview and exact base-cookie pixels; unchanged geometry/material/shared/custom images; native application and projection parity separate'}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('stage',type=Path);p.add_argument('--report',type=Path,required=True);a=p.parse_args()
    if a.report.exists():raise FileExistsError('Use a fresh report')
    r=verify(a.stage);a.report.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))
