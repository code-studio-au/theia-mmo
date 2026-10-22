#!/usr/bin/env python3
"""Open one measured blacksmith room while retaining the full map/farmhouse."""
import argparse
import copy
import hashlib
import itertools
import json
from pathlib import Path

import numpy as np
import trimesh

from add_terrain_house import write_glb
from assemble_map import placement_plan
from merge_static_glb import merge_documents, read
from open_map_farmhouse import subdivided_boxes
from prepare_enterable_house import quad
from prepare_map_world import envelope

PLACEMENT = 'adt_65748538'
SOURCE_SHA = 'f1e01da8e73ae9c72d93fae2459cb02697c13aec6c68f9033e2db2a51ed01678'


def rebase_profile(original, origin):
    profile = copy.deepcopy(original)
    delta = np.asarray(origin)-np.asarray(profile['source_origin'])
    for name in ['triangles','walk_surfaces']:
        profile[name] = np.asarray(np.asarray(profile.get(name,[]))-delta, dtype=np.float32).tolist() if profile.get(name) else []
    for name in ['solids','camera_solids']:
        for box in profile.get(name,[]):
            box['center'] = (np.asarray(box['center'])-delta).tolist()
    profile['bounds'] = (np.asarray(profile['bounds'])-delta[[0,2]]).tolist()
    profile['source_origin'] = np.asarray(origin).tolist()
    return profile, delta


def ground_height(triangles, x, z):
    a,b,c=triangles[:,0],triangles[:,1],triangles[:,2]
    area=(b[:,0]-a[:,0])*(c[:,2]-a[:,2])-(b[:,2]-a[:,2])*(c[:,0]-a[:,0])
    u=((b[:,0]-x)*(c[:,2]-z)-(b[:,2]-z)*(c[:,0]-x))/area
    v=((x-a[:,0])*(c[:,2]-a[:,2])-(z-a[:,2])*(c[:,0]-a[:,0]))/area
    w=1-u-v; mask=(u>=-1e-5)&(v>=-1e-5)&(w>=-1e-5)
    if not mask.any(): raise ValueError('Missing terrain at the blacksmith approach')
    return float((u*a[:,1]+v*b[:,1]+w*c[:,1])[mask].max())


def prepare(world, placements, output):
    if output.exists(): raise ValueError('Output directory must be new')
    farm=json.loads((world/'access.json').read_text())
    if farm['placement']!='adt_65749577' or farm['floor_triangles']!=133:
        raise ValueError('Expected the qualified full-map farmhouse world')
    plan,sidecars,_,_=placement_plan(placements)
    _,_,model,world_matrix=next(row for row in plan if row[0]==PLACEMENT and row[1]=='world')
    audit_path,= [Path(p) for p in farm['source_hashes'] if Path(p).name=='preparation.json']
    paths={world/'access.json',world/'terrain.json',world/'terrain.glb',placements,model,audit_path,*sidecars}
    hashes={str(p.resolve()):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    if hashes[str(model.resolve())]!=SOURCE_SHA:
        raise ValueError('Authored access requires the checked blacksmith source hash')
    if farm['source_hashes'][str(placements.resolve())]!=hashes[str(placements.resolve())] or farm['source_hashes'][str(audit_path.resolve())]!=hashes[str(audit_path.resolve())]:
        raise ValueError('Placement/audit differs from the prepared farmhouse world')
    audit=json.loads(audit_path.read_text())
    for path in {placements,model,*sidecars}:
        if audit['source_hashes'].get(str(path.resolve()))!=hashes[str(path.resolve())]:
            raise ValueError('Source differs from the prepared visual map')
    original=json.loads((world/'terrain.json').read_text())
    if original['schema']!=2 or len(original.get('walk_surfaces',[]))!=137 or bytes(original['visual_sha256']).hex()!=hashes[str((world/'terrain.glb').resolve())]:
        raise ValueError('Expected the matching qualified farmhouse profile/visual')
    # Preserve the first open building, and account for its removed original
    # envelope when looking up the second envelope in the original audit.
    original_index=next(r['solid_index'] for r in audit['placements'] if r['placement']==PLACEMENT)
    farm_index=next(r['solid_index'] for r in audit['placements'] if r['placement']==farm['placement'])
    remove_index=original_index-int(original_index>farm_index)
    source_spawn=world_matrix[:3,:3] @ np.array([21.,0.,0.])+world_matrix[:3,3]
    origin=[source_spawn[0],original['source_origin'][1],source_spawn[2]]
    profile,delta=rebase_profile(original,origin)
    matrix=world_matrix.copy();matrix[:3,3]-=origin
    del profile['solids'][remove_index]
    wall_start=len(profile['solids'])
    # The main room is qualified; deep wall bays and the western forge/annex are
    # conservatively excluded. Split the eastern wall around its fixed doorway.
    for low,high in [
        ([-9.6,0.,-7.8],[-5.7,13.3,7.8]),
        ([-5.7,0.,-7.8],[11.2,13.3,-6.]),
        ([-5.7,0.,6.],[11.2,13.3,7.8]),
        ([10.1,0.,-6.],[11.2,8.3,-2.2]),
        ([10.1,0.,2.1],[11.2,8.3,6.]),
    ]:
        profile['solids'].extend(subdivided_boxes(low,high,matrix))
    # Keep the unqualified exterior bays/forge physically closed from outside,
    # too. Coarser pieces away from the entrance bound the original large WMO
    # silhouette without one rotated AABB spilling across the accessible room.
    excluded_start=len(profile['solids'])
    for low,high,cell in [
        ([-27.6,-2.83,-23.5],[-5.7,27.1,2.],2.),
        ([-12.,0.,-11.5],[13.,16.4,-7.8],1.),
        ([-12.,0.,7.8],[13.,16.4,11.5],1.),
        ([11.2,0.,-11.5],[16.3,16.4,-2.2],1.),
        ([11.2,0.,2.1],[16.3,16.4,11.5],1.),
    ]:
        profile['solids'].extend(subdivided_boxes(low,high,matrix,cell))
    for low,high in [([-5.7,8.26,-7.8],[11.2,16.4,7.8]), ([10.1,6.35,-2.2],[11.2,8.26,2.1])]:
        corners=np.asarray(list(itertools.product(*zip(low,high))))
        profile['camera_solids'].append(envelope(corners @ matrix[:3,:3].T+matrix[:3,3]))
    scene=trimesh.load_scene(model,process=False,group_material=False,split_groups=True)
    floors=[]
    for group in ['blacksmithinterior0114','blacksmithinterior017']:
        mesh=scene.geometry[group]
        selected=(mesh.face_normals[:,1]>.95)&(mesh.triangles_center[:,1]>1.7)&(mesh.triangles_center[:,1]<1.9)
        floors.extend(mesh.triangles[selected])
    if len(floors)!=226: raise ValueError('Unexpected measured blacksmith floor layout')
    terrain=np.asarray(profile['triangles'])
    feet=[matrix[:3,:3] @ [18.,0.,z]+matrix[:3,3] for z in [-1.8,0.,1.8]]
    start_y=max(ground_height(terrain,p[0],p[2]) for p in feet)-matrix[1,3]
    if abs((1.82-start_y)/(18-10.8))>np.tan(np.radians(40)):
        raise ValueError('Blacksmith ramp exceeds walkable slope')
    access=np.asarray(quad(-1.8,1.8,18.,10.8,start_y,1.82)+quad(-1.8,1.8,10.8,9.9,1.82,1.82))
    access=access[:,:,[2,1,0]];access[:,:,2]*=-1
    support=np.asarray([*floors,*access]) @ matrix[:3,:3].T+matrix[:3,3]
    profile['walk_surfaces'].extend(np.asarray(support,dtype=np.float32).tolist())
    if len(profile['solids'])+len(profile['camera_solids'])>4096:
        raise ValueError('Static envelope budget exceeded')
    ramp=trimesh.Trimesh(vertices=support[-4:].reshape(-1,3),faces=np.arange(12).reshape(-1,3),process=False)
    ramp.visual=trimesh.visual.TextureVisuals(material=trimesh.visual.material.PBRMaterial(
        baseColorFactor=[.35,.20,.10,1.],metallicFactor=0.,roughnessFactor=1.,doubleSided=True))
    ramp_scene=trimesh.Scene(ramp)
    doc,binary=read(world/'terrain.glb')
    doc['nodes'].append({'name':'blacksmith approach origin','translation':(-delta).tolist(),'children':doc['scenes'][0]['nodes']})
    doc['scenes'][0]['nodes']=[len(doc['nodes'])-1]
    if any(hashlib.sha256(Path(p).read_bytes()).hexdigest()!=h for p,h in hashes.items()):
        raise ValueError('Source changed during blacksmith preparation')
    output.mkdir(parents=True)
    (output/'blacksmith-access.glb').write_bytes(trimesh.exchange.gltf.export_glb(ramp_scene,include_normals=True))
    added,added_bin=read(output/'blacksmith-access.glb')
    combined=merge_documents(doc,added,len(binary));combined['buffers'][0]['byteLength']=len(binary)+len(added_bin)
    write_glb(combined,binary+added_bin,output/'terrain.glb')
    profile['visual_sha256']=list(hashlib.sha256((output/'terrain.glb').read_bytes()).digest())
    data=(json.dumps(profile,separators=(',',':'),allow_nan=False)+'\n').encode()
    if len(data)>32*1024*1024: raise ValueError('Profile exceeds byte budget')
    (output/'terrain.json').write_bytes(data)
    def point(x,z):
        p=matrix[:3,:3] @ [x,0.,z]+matrix[:3,3];return p[[0,2]].tolist()
    approach,inside,wall=point(20.,0.),point(4.,0.),point(4.,-12.)
    retained=copy.deepcopy(farm)
    retained['house_matrix']=(np.asarray(farm['house_matrix'])-np.array([[0,0,0,delta[0]],[0,0,0,delta[1]],[0,0,0,delta[2]],[0,0,0,0]])).tolist()
    retained['route']=(np.asarray(farm['route'])-delta[[0,2]]).tolist()
    retained['origin']=origin
    retained['wall_start']-=int(remove_index<retained['wall_start'])
    retained['world_sha256']=hashlib.sha256(data).hexdigest()
    (output/'farmhouse-access.json').write_text(json.dumps(retained,indent=2)+'\n')
    report={'placement':PLACEMENT,'source_hashes':hashes,'house_matrix':matrix.tolist(),'origin':origin,
        'floor_triangles':226,'ramp_triangles':4,'wall_boxes':excluded_start-wall_start,
        'exclusion_boxes':len(profile['solids'])-excluded_start,
        'door_width':4.3,'door_clearance':6.35-1.82,'route':[approach,inside,wall,inside,approach,inside],
        'qualification':{'inside':[4.,1.81,0.],'wall_axis':2,'wall_range':[-5.8,-4.5],'exit_axis':0,'exit_min':19.8},
        'retained_farmhouse':farm['placement'],'world_sha256':hashlib.sha256(data).hexdigest(),
        'scope':'original blacksmith main room plus retained farmhouse; forge/annex and roof remain excluded; no moving doors, stairs or general mesh physics'}
    (output/'access.json').write_text(json.dumps(report,indent=2)+'\n')
    return {k:v for k,v in report.items() if k not in ('source_hashes','house_matrix')}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('world',type=Path);parser.add_argument('placements',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args();print(json.dumps(prepare(args.world,args.placements,args.output),indent=2))
