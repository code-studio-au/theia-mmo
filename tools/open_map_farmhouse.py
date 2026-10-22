#!/usr/bin/env python3
"""Author ground-floor access to the actual farmhouse in the complete map tile.

This measured layout is pinned to one source OBJ/placement. Preserve the original
map BIN and all placements. Replace only the building envelope, retain its props,
add approved floor triangles and a visible fixed entrance ramp. No doors move.
"""
import argparse
import hashlib
import itertools
import json
from pathlib import Path

import numpy as np
import trimesh

from add_terrain_house import write_glb
from assemble_map import placement_plan
from merge_static_glb import merge_documents, read
from prepare_enterable_house import quad
from prepare_map_world import envelope

SOURCE_SHA = '090e535370366cc6016c234c3ce39014c5ad6d685c4531d36483717b46d82c52'
PLACEMENT = 'adt_65749577'


def subdivided_boxes(low, high, matrix, cell=.5):
    """Conservative short world AABBs preserve a rotated doorway's clearance."""
    low, high = np.asarray(low, dtype=float), np.asarray(high, dtype=float)
    xs = np.linspace(low[0], high[0], int(np.ceil((high[0]-low[0])/cell))+1)
    zs = np.linspace(low[2], high[2], int(np.ceil((high[2]-low[2])/cell))+1)
    boxes = []
    for x0, x1 in zip(xs[:-1], xs[1:]):
        for z0, z1 in zip(zs[:-1], zs[1:]):
            corners = np.asarray(list(itertools.product([x0,x1], [low[1],high[1]], [z0,z1])))
            boxes.append(envelope(corners @ matrix[:3,:3].T + matrix[:3,3]))
    return boxes


def prepare(world, placements, output):
    if output.exists():
        raise ValueError('Output directory must be new')
    plan, sidecars, _, _ = placement_plan(placements)
    row, = [row for row in plan if row[0] == PLACEMENT and row[1] == 'world']
    _, _, model, world_matrix = row
    paths = {world/'terrain.json', world/'terrain.glb', world/'preparation.json', placements, model, *sidecars}
    hashes = {str(p.resolve()):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
    if hashes[str(model.resolve())] != SOURCE_SHA:
        raise ValueError('Authored access requires the checked farmhouse source hash')
    profile = json.loads((world/'terrain.json').read_text())
    audit = json.loads((world/'preparation.json').read_text())
    if profile['schema'] != 2 or profile.get('walk_surfaces') or profile.get('camera_solids'):
        raise ValueError('Expected the original full-tile outdoor profile')
    for path in {placements,model,*sidecars}:
        key = str(path.resolve())
        if audit['source_hashes'].get(key) != hashes[key]:
            raise ValueError('Placement/source differs from the prepared visual world')
    if bytes(profile['visual_sha256']).hex() != hashes[str((world/'terrain.glb').resolve())]:
        raise ValueError('World visual/profile mismatch')
    building, = [r for r in audit['placements'] if r['placement'] == PLACEMENT]
    index = building['solid_index']
    # Put normal development spawns at the outdoor ramp approach, keeping source
    # orientation/scale intact. Only the coordinate origin changes.
    source_spawn = world_matrix[:3,:3] @ np.array([-1.445,0.,19.]) + world_matrix[:3,3]
    origin = np.array([source_spawn[0], profile['source_origin'][1], source_spawn[2]])
    delta = origin - np.asarray(profile['source_origin'])
    matrix = world_matrix.copy(); matrix[:3,3] -= origin
    terrain = np.asarray(profile['triangles']) - delta
    profile['triangles'] = np.asarray(terrain, dtype=np.float32).tolist()
    profile['bounds'] = (np.asarray(profile['bounds']) - delta[[0,2]]).tolist()
    solids = []
    for i, box in enumerate(profile['solids']):
        if i != index:
            solids.append(dict(center=(np.asarray(box['center'])-delta).tolist(), size=box['size']))
    wall_start = len(solids)
    for lo, hi in [
        ([-6.5,0.,-6.8],[-5.4,13.3,7.1]),
        ([5.8,0.,-6.8],[6.8,13.3,7.1]),
        ([-6.5,0.,-6.8],[6.8,13.3,-5.65]),
        ([-6.5,0.,6.],[-2.78,9.7,7.1]),
        ([-.11,0.,6.],[6.8,9.7,7.1]),
        ([-1.,0.,-8.4],[3.2,13.3,-4.7]),
        ([-5.4,4.,.4],[-3.4,13.3,6.]),
    ]:
        solids.extend(subdivided_boxes(lo,hi,matrix))
    camera_solids = []
    for lo, hi in [([-6.5,9.6,-6.8],[6.8,17.1,7.1]), ([-2.78,7.5,6.],[-.11,9.6,7.7])]:
        corners=np.asarray(list(itertools.product(*zip(lo,hi))))
        camera_solids.append(envelope(corners @ matrix[:3,:3].T + matrix[:3,3]))
    scene = trimesh.load_scene(model,process=False,group_material=False,split_groups=True)
    floors=[]
    for group in ['interior2','interior9']:
        mesh=scene.geometry[group]
        selected=(mesh.face_normals[:,1]>.95)&(mesh.triangles_center[:,1]>4.)&(mesh.triangles_center[:,1]<4.3)
        floors.extend(mesh.triangles[selected])
    if len(floors) != 133:
        raise ValueError('Unexpected measured source floor layout')
    # Sample terrain at three points across the ramp approach, using an
    # independent barycentric height interpolator.
    def ground(x,z):
        a,b,c=terrain[:,0],terrain[:,1],terrain[:,2]
        area=(b[:,0]-a[:,0])*(c[:,2]-a[:,2])-(b[:,2]-a[:,2])*(c[:,0]-a[:,0])
        u=((b[:,0]-x)*(c[:,2]-z)-(b[:,2]-z)*(c[:,0]-x))/area
        v=((x-a[:,0])*(c[:,2]-a[:,2])-(z-a[:,2])*(c[:,0]-a[:,0]))/area
        w=1-u-v;mask=(u>=-1e-5)&(v>=-1e-5)&(w>=-1e-5)
        if not mask.any(): raise ValueError('Missing ramp approach terrain')
        return float((u*a[:,1]+v*b[:,1]+w*c[:,1])[mask].max())
    x0,x1,z0,z1=-2.78,-.11,17.,7.7
    foot=[matrix[:3,:3] @ np.array([x,0,z0])+matrix[:3,3] for x in [x0,(x0+x1)/2,x1]]
    start_y=max(ground(p[0],p[2]) for p in foot)-matrix[1,3]
    if abs((4.17-start_y)/(z0-z1)) > np.tan(np.radians(40)):
        raise ValueError('Map entrance ramp exceeds walkable slope')
    access=quad(x0,x1,z0,z1,start_y,4.17)+quad(x0,x1,z1,5.8,4.17,4.17)
    support=np.asarray([*floors,*access]) @ matrix[:3,:3].T+matrix[:3,3]
    profile.update(source_origin=origin.tolist(), solids=solids, camera_solids=camera_solids,
                   walk_surfaces=np.asarray(support,dtype=np.float32).tolist(), step_height=.2)
    if len(solids)+len(camera_solids)>4096: raise ValueError('Static envelope budget exceeded')
    ramp=trimesh.Trimesh(vertices=support[-4:].reshape((-1,3)),faces=np.arange(12).reshape(-1,3),process=False)
    ramp.visual=trimesh.visual.TextureVisuals(material=trimesh.visual.material.PBRMaterial(
        baseColorFactor=[.35,.20,.10,1.],metallicFactor=0.,roughnessFactor=1.,doubleSided=True))
    ramp_scene=trimesh.Scene(ramp)
    visual_doc,binary=read(world/'terrain.glb')
    # The additional root shifts the existing complete scene, retaining its BIN.
    visual_doc['nodes'].append({'name':'farmhouse approach origin','translation':(-delta).tolist(),
                                'children':visual_doc['scenes'][0]['nodes']})
    visual_doc['scenes'][0]['nodes']=[len(visual_doc['nodes'])-1]
    if any(hashlib.sha256(Path(p).read_bytes()).hexdigest()!=h for p,h in hashes.items()):
        raise ValueError('Source changed during access preparation')
    output.mkdir(parents=True)
    (output/'access.glb').write_bytes(trimesh.exchange.gltf.export_glb(ramp_scene,include_normals=True))
    ramp_doc,ramp_bin=read(output/'access.glb')
    combined=merge_documents(visual_doc,ramp_doc,len(binary))
    combined['buffers'][0]['byteLength']=len(binary)+len(ramp_bin)
    write_glb(combined,binary+ramp_bin,output/'terrain.glb')
    profile['visual_sha256']=list(hashlib.sha256((output/'terrain.glb').read_bytes()).digest())
    data=(json.dumps(profile,separators=(',',':'),allow_nan=False)+'\n').encode()
    (output/'terrain.json').write_bytes(data)
    def point(x,z):
        p=matrix[:3,:3] @ np.array([x,0.,z])+matrix[:3,3]
        return p[[0,2]].tolist()
    approach,inside,wall=point(-1.445,18.),point(-1.445,2.),point(10.,2.)
    report={'source_hashes':hashes,'placement':PLACEMENT,'origin':origin.tolist(),
            'house_matrix':matrix.tolist(),'floor_triangles':133,'ramp_triangles':4,
            'wall_boxes':len(solids)-wall_start,'wall_start':wall_start,
            'door_clearance':7.5-4.17,'door_width':x1-x0,
            'route':[approach,inside,wall,inside,approach,inside],
            'wall_stage':2,'inside_stages':[1,3,5],'outside_stage':4,
            'world_sha256':hashlib.sha256(data).hexdigest(),
            'scope':'one original map farmhouse ground floor; measured walls and fixed ramp; loft blocked, no moving doors or general mesh physics'}
    (output/'access.json').write_text(json.dumps(report,indent=2)+'\n')
    return {k:v for k,v in report.items() if k not in ('source_hashes','house_matrix')}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('world',type=Path);parser.add_argument('placements',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args()
    print(json.dumps(prepare(args.world,args.placements,args.output),indent=2))
