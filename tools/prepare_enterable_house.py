#!/usr/bin/env python3
"""Prepare the qualified farmhouse's ground-floor interior and fixed access ramp.

This authored layout is specific to the checked farmhouse hash, not an automatic
mesh collider or general interior importer. Originals and embedded buffers survive.
"""
import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import trimesh
from add_terrain_house import clipped_heights, write_glb
from merge_static_glb import read, merge_documents

HOUSE_SHA = '7d60b5740f636f536b03695dcf75a6e18ab27e437b15da3c343afd655ee0d798'
SCALE = 0.6  # checked door clearance > 1.8m pawn + 0.15m margin


def quad(x0, x1, z0, z1, y0, y1):
    # z0 > z1; both triangles face upward.
    a, b, c, d = [x0,y0,z0], [x1,y0,z0], [x1,y1,z1], [x0,y1,z1]
    return [[a,b,c],[a,c,d]]


def layout(bounds, base, ground_at):
    bounds = np.asarray(bounds)
    offset = np.array([-8.-bounds[:,0].mean()*SCALE, base-bounds[0,1]*SCALE,
                       -8.-bounds[:,2].mean()*SCALE])
    transform = np.eye(4)
    transform[:3,:3] *= SCALE
    transform[:3,3] = offset
    def box(low, high):
        low, high = np.array(low)*SCALE+offset, np.array(high)*SCALE+offset
        return {'center': ((low+high)/2).tolist(), 'size': (high-low).tolist()}
    solids = [box(*b) for b in [
        ([-6.5,-.143,-6.8],[-5.4,10.2,7.1]),
        ([5.8,-.143,-6.8],[6.8,10.2,7.1]),
        ([-6.5,-.143,-6.8],[6.8,10.2,-5.65]),
        ([-6.5,-.143,6.],[-2.78,7.5,7.1]),
        ([-.11,-.143,6.],[6.8,7.5,7.1]),
        ([-1.,-.143,-8.4],[3.2,10.2,-4.7]),
        ([-5.4,4.,.4],[-3.4,10.2,6.]),  # exclude the unqualified loft stair area
    ]]
    camera_solids = [box([-6.5,8.1,-6.8],[6.8,17.1,7.1]),
                     box([-2.78,7.5,6.],[-.11,8.1,7.7])]
    x0, x1 = np.array([-2.78,-.11])*SCALE+offset[0]
    doorway_x = float((x0+x1)/2)
    start_z = 2.5
    landing_z, door_z = np.array([7.7,6.])*SCALE+offset[2]
    floor_y = 4.109*SCALE+offset[1]
    landing_y = 4.17*SCALE+offset[1]
    start_y = max(ground_at(float(x),start_z) for x in [x0,(x0+x1)/2,x1])
    if (landing_y-start_y)/(start_z-landing_z) > np.tan(np.radians(40)):
        raise ValueError('Authored entrance ramp exceeds walker slope limit')
    ramp = quad(x0,x1,start_z,landing_z,start_y,landing_y)
    landing = quad(x0,x1,landing_z,door_z,landing_y,landing_y)
    foundation = quad(-14.1,-1.9,-1.9,-14.1,base,base)
    return {'transform':transform, 'solids':solids, 'camera_solids':camera_solids,
            'ramp':ramp, 'landing':landing, 'foundation':foundation,
            'entry_x':doorway_x, 'entry_floor_y':float(floor_y),
            'door_clearance':float((7.5-4.17)*SCALE),
            'route':[[0.,3.],[doorway_x,3.],[doorway_x,-7.]],
            'step_height':.2}


def ground_sampler(triangles):
    def height(x,z):
        hits=[]
        for t in triangles:
            a,b,c=np.asarray(t)
            cross=lambda p,q,r:(q[0]-p[0])*(r[2]-p[2])-(q[2]-p[2])*(r[0]-p[0])
            area=cross(a,b,c);p=[x,0,z]
            u=cross(p,b,c)/area;v=cross(a,p,c)/area;w=1-u-v
            if min(u,v,w)>=-1e-5: hits.append(u*a[1]+v*b[1]+w*c[1])
        if not hits: raise ValueError('Missing terrain at authored ramp approach')
        return float(max(hits))
    return height


def prepare(terrain, house, output):
    if output.exists(): raise ValueError('Output directory must be new')
    paths=[terrain/'terrain.json',terrain/'terrain.glb',house]
    hashes={str(p.resolve()):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    if hashes[str(house.resolve())] != HOUSE_SHA: raise ValueError('Unqualified farmhouse; authored layout requires the checked source hash')
    profile=json.loads(paths[0].read_text())
    if profile['schema']!=1 or profile.get('solids') or profile.get('walk_surfaces'):
        raise ValueError('Expected original terrain-only fixture')
    if bytes(profile['visual_sha256']).hex()!=hashes[str(paths[1].resolve())]: raise ValueError('Terrain visual mismatch')
    low, base=clipped_heights(profile['triangles'],-14.1,-1.9)
    house_scene=trimesh.load_scene(house,process=False)
    if len(house_scene.geometry)!=23: raise ValueError('Unexpected farmhouse geometry layout')
    data=layout(house_scene.bounds,base,ground_sampler(profile['triangles']))
    floor_triangles=[]
    for index in [14,22]:
        mesh=list(house_scene.geometry.values())[index]
        selected=(mesh.face_normals[:,1]>.95)&(mesh.triangles_center[:,1]>4.)&(mesh.triangles_center[:,1]<4.3)
        floor_triangles.extend((mesh.triangles[selected]*SCALE+data['transform'][:3,3]).tolist())
    if len(floor_triangles)!=80: raise ValueError('Unexpected qualified ground-floor triangle count')
    profile.update(solids=data['solids'], camera_solids=data['camera_solids'],step_height=data['step_height'],
                   walk_surfaces=floor_triangles+data['ramp']+data['landing']+data['foundation'])
    house_doc,house_bin=read(house)
    roots=house_doc['scenes'][0]['nodes']
    house_doc['nodes'].append({'name':'enterable_farmhouse','children':roots,'matrix':data['transform'].T.reshape(-1).tolist()})
    house_doc['scenes'][0]['nodes']=[len(house_doc['nodes'])-1]
    visual=trimesh.Scene()
    for name,triangles,color in [('ramp',data['ramp']+data['landing'],[.35,.20,.10,1.])]:
        vertices=np.asarray(triangles).reshape((-1,3))
        faces=np.arange(len(vertices)).reshape((-1,3))
        normals=np.cross(np.asarray(triangles)[:,1]-np.asarray(triangles)[:,0],np.asarray(triangles)[:,2]-np.asarray(triangles)[:,0])
        normals/=np.linalg.norm(normals,axis=1)[:,None]
        mesh=trimesh.Trimesh(vertices=vertices,faces=faces,vertex_normals=np.repeat(normals,3,axis=0),process=False)
        mesh.visual=trimesh.visual.TextureVisuals(material=trimesh.visual.material.PBRMaterial(baseColorFactor=color,metallicFactor=0.,roughnessFactor=1.,doubleSided=True))
        visual.add_geometry(mesh,geom_name=name)
    foundation=trimesh.creation.box(extents=[12.2,base-low+.1,12.2])
    foundation.apply_translation([-8.,(base+low-.1)/2.,-8.])
    foundation=trimesh.Trimesh(vertices=foundation.triangles.reshape((-1,3)),faces=np.arange(36).reshape((-1,3)),
        vertex_normals=np.repeat(foundation.face_normals,3,axis=0),process=False)
    foundation.visual=trimesh.visual.TextureVisuals(material=trimesh.visual.material.PBRMaterial(baseColorFactor=[.22,.24,.26,1.],metallicFactor=0.,roughnessFactor=1.))
    visual.add_geometry(foundation,geom_name='foundation')
    output.mkdir(parents=True)
    (output/'access.glb').write_bytes(trimesh.exchange.gltf.export_glb(visual,include_normals=True))
    terrain_doc,terrain_bin=read(paths[1]);access_doc,access_bin=read(output/'access.glb')
    merged=merge_documents(terrain_doc,house_doc,len(terrain_bin))
    merged=merge_documents(merged,access_doc,len(terrain_bin)+len(house_bin))
    binary=terrain_bin+house_bin+access_bin;merged['buffers'][0]['byteLength']=len(binary)
    write_glb(merged,binary,output/'terrain.glb')
    profile['visual_sha256']=list(hashlib.sha256((output/'terrain.glb').read_bytes()).digest())
    (output/'terrain.json').write_text(json.dumps(profile,separators=(',',':'),allow_nan=False)+'\n')
    if any(hashlib.sha256(Path(p).read_bytes()).hexdigest()!=h for p,h in hashes.items()): raise ValueError('Source changed; discard output')
    report={k:v for k,v in data.items() if k not in ['transform','ramp','landing','foundation']}
    report.update(source_hashes=hashes,house_transform=data['transform'].tolist(),walk_surface_triangles=len(profile['walk_surfaces']),
        visual_sha256=bytes(profile['visual_sha256']).hex(),profile_sha256=hashlib.sha256((output/'terrain.json').read_bytes()).hexdigest(),
        terrain_binary_bytes=len(terrain_bin),house_binary_bytes=len(house_bin),visual_bytes=(output/'terrain.glb').stat().st_size,
        scope='authored ground-floor access with static ramp and doorway; no loft, jumping, falling, full mesh physics or NPC collision')
    (output/'interior.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('terrain',type=Path);parser.add_argument('house',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args()
    print(json.dumps(prepare(args.terrain,args.house,args.output),indent=2))
