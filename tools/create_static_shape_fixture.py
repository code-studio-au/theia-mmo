#!/usr/bin/env python3
"""Original visible schema-4 fixture for normal encrypted client/server checks."""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
import trimesh
from package_world import package
from schema4_clearance import quaternion_matrix


def create(output):
    if output.exists(): raise FileExistsError('Use a fresh fixture directory')
    scene = trimesh.Scene()
    objects = [
        {'kind':'box','center':[5,1,5],'size':[1,2,3],'rotation':[0,.382683432365,0,.923879532511]},
        {'kind':'capsule','center':[-7,1.5,6],'radius':.4,'half_segment':[.4,.8,0]},
        {'kind':'mesh','mesh':0,'affine':[2,.3,0,6, 0,2,0,0, 0,0,2,-6, 0,0,0,1],'convex':True},
        {'kind':'mesh','mesh':0,'affine':[1,0,0,9, 0,1,0,0, 0,0,1,-6, 0,0,0,1],'convex':True},
        {'kind':'box','center':[-7,3,-6],'size':[4,.3,4],'rotation':[0,0,0,1]},
    ]
    pool = [{'vertices':[[0,0,0],[1,0,0],[0,1,0],[0,0,1]],'indices':[[0,2,1],[0,1,3],[0,3,2],[1,2,3]]}]
    # Rectangular 3 x 5 grid deliberately exposes transposed heightfield layouts.
    vertices = [[x,0,z] for x in np.linspace(-20,20,5) for z in np.linspace(-20,20,3)]
    faces = []
    for j in range(4):
        for i in range(2):
            a = j*3+i; b=a+1; c=a+3; d=c+1
            faces.extend([[a,b,c],[b,d,c]])
    floor = trimesh.Trimesh(vertices=vertices,faces=faces,process=False)
    floor.visual.vertex_colors = [90,145,110,255]
    scene.add_geometry(floor,node_name='full_sample_floor')
    for index,body in enumerate(objects):
        transform = np.eye(4)
        if body['kind'] == 'box':
            mesh = trimesh.creation.box(extents=body['size'])
            transform[:3,:3] = quaternion_matrix(body['rotation'])
            transform[:3,3] = body['center']
        elif body['kind'] == 'capsule':
            half = np.array(body['half_segment'])
            mesh = trimesh.creation.capsule(height=2*np.linalg.norm(half),radius=body['radius'],count=[16,16])
            # Installed trimesh capsules are centred on their Z-axis segment.
            transform = trimesh.geometry.align_vectors([0,0,1],half/np.linalg.norm(half))
            transform[:3,3] = body['center']
        else:
            source = pool[body['mesh']]
            mesh = trimesh.Trimesh(vertices=source['vertices'],faces=source['indices'],process=False)
            transform = np.array(body['affine']).reshape(4,4)
        mesh.visual.vertex_colors = [[185,130,80,255],[90,145,190,255],[170,100,155,255]][index%3]
        scene.add_geometry(mesh,node_name=f"shape_{index}_{body['kind']}",transform=transform)
    visual = trimesh.exchange.gltf.export_glb(scene)
    output.mkdir(parents=True)
    (output/'terrain.glb').write_bytes(visual)
    profile = {'schema':4,'bounds':[[-19,-19],[19,19]],'source_origin':[0,0,0],
               'visual_sha256':list(hashlib.sha256(visual).digest()),'triangles':[],
               'spawns':[[-3,0,-3],[0,0,-3]],'static_shapes':{'meshes':pool,'objects':objects,
               'heightfields':[{'rows':3,'columns':5,'heights_column_major':[0]*15,
                                'center':[0,0,0],'scale':[40,1,40]}]}}
    (output/'terrain.json').write_text(json.dumps(profile,allow_nan=False)+'\n')
    return package(output,'theia.fixture.static-shapes',1)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output',type=Path)
    print(json.dumps(create(parser.parse_args().output),indent=2))
