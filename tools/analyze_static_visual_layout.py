#!/usr/bin/env python3
"""Measure whole static draw ownership and conservative contributor envelopes.

Reads actual position/index bytes. Does not publish packages, split triangles,
admit visual-only providers or establish renderer/streaming capacity.
"""
import argparse
import itertools
import json
from pathlib import Path
import numpy as np
from index_chunk_contributors import overlaps
from merge_static_glb import read
from static_geometry import accessor, hierarchy
from unity_editor_reference import digest, read_json


def analyze(source, collision_reference=None):
    document,binary=read(source,allow_external_images=True)
    _,matrices=hierarchy(document)
    cache={};owners={}
    for node_index,matrix in matrices.items():
        node=document['nodes'][node_index]
        if 'mesh' not in node:continue
        mesh_index=node['mesh']
        if mesh_index not in cache:
            corners=[];triangles=0
            for primitive in document['meshes'][mesh_index]['primitives']:
                if primitive.get('mode',4)!=4:raise ValueError('Expected static triangle primitives')
                points=accessor(document,binary,primitive['attributes']['POSITION'])
                count=len(points)
                if 'indices' in primitive:
                    indices=accessor(document,binary,primitive['indices'])
                    if indices.dtype.kind!='u' or indices.shape[1]!=1 or indices.max()>=count:raise ValueError('Invalid static triangle indices')
                    count=len(indices)
                if count%3:raise ValueError('Incomplete static triangle')
                triangles+=count//3
                corners.extend(itertools.product(*zip(points.min(0),points.max(0))))
            cache[mesh_index]=(np.array(corners),triangles)
        corners,triangles=cache[mesh_index]
        points=corners@matrix[:3,:3].T+matrix[:3,3]
        bounds=np.array([points.min(0),points.max(0)])
        owner=tuple(np.floor(bounds.mean(0)[[0,2]]/128).astype(int))
        entry=owners.setdefault(owner,dict(nodes=0,primitives=0,triangles=0,meshes=set(),materials=set(),bounds=bounds.copy()))
        entry['nodes']+=1;entry['triangles']+=triangles
        primitives=document['meshes'][mesh_index]['primitives']
        entry['primitives']+=len(primitives);entry['meshes'].add(mesh_index)
        entry['materials'].update(p['material'] for p in primitives if 'material' in p)
        entry['bounds'][0]=np.minimum(entry['bounds'][0],bounds[0]);entry['bounds'][1]=np.maximum(entry['bounds'][1],bounds[1])
    collision={}
    if collision_reference:
        directory=Path(collision_reference)
        reference=read_json(directory/'reference.json',2*1024*1024)
        for entry in reference['profiles']:
            path=directory/entry['file']
            if digest(path)!=entry['sha256']:raise ValueError('Collision reference binding differs')
            owner=tuple(entry['owner']); boxes=np.array(entry['metrics']['bounds'])
            origin=np.array([owner[0]*128,0,owner[1]*128])
            collision[owner]=np.array([boxes[:,0].min(0),boxes[:,1].max(0)])+origin
    envelopes={owner:entry['bounds'].copy() for owner,entry in owners.items()}
    for owner,bounds in collision.items():
        if owner not in envelopes:envelopes[owner]=bounds.copy()
        else:envelopes[owner]=np.stack([np.minimum(envelopes[owner][0],bounds[0]),np.maximum(envelopes[owner][1],bounds[1])])
    reports=[]
    for owner,entry in sorted(owners.items()):
        reports.append(dict(owner=list(map(int,owner)),nodes=entry['nodes'],primitives=entry['primitives'],triangles=entry['triangles'],unique_meshes=len(entry['meshes']),materials=len(entry['materials']),bounds=entry['bounds'].tolist()))
    cells=[]
    for cell in [(-1,-1),(-1,0),(0,-1),(0,0)]:
        contributors=[]
        for owner,bounds in envelopes.items():
            origin=np.array([owner[0]*128,owner[1]*128])
            local=bounds[:,[0,2]]-origin
            if owner==cell or overlaps(owner,local,cell):contributors.append(list(map(int,owner)))
        cells.append(dict(cell=list(cell),contributors=sorted(contributors)))
    return dict(source_sha256=digest(source),draw_nodes=sum(v['nodes'] for v in reports),
                primitives=sum(v['primitives'] for v in reports),triangles=sum(v['triangles'] for v in reports),
                visual_providers=len(owners),collision_providers=len(collision),
                visual_only_owners=[list(map(int,o)) for o in sorted(set(owners)-set(collision))],
                combined_providers=len(envelopes),central_cells=cells,providers=reports,
                scope='Offline whole-draw centroid ownership/envelopes; terrain is not clipped; central cells are explicit [-128,128] XZ, not a world coverage/capacity claim')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('source',type=Path)
    parser.add_argument('--collision-reference',type=Path)
    args=parser.parse_args();print(json.dumps(analyze(args.source,args.collision_reference),indent=2))
