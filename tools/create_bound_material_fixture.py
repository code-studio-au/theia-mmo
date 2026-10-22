#!/usr/bin/env python3
"""Original three-provider projected-material fixture, with immutable PNG binding.

Source-local IDs differ across providers but identical bound profiles can share
materials/textures. A session is emitted only with --session for native runtime
qualification; no imported map or default is changed.
"""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
from PIL import Image
import trimesh
from add_terrain_house import write_glb
from externalize_world_images import extract_document
from index_chunk_contributors import author
from merge_static_glb import read
from package_world import package
from shared_images import filename


def cube():
    vertices=[];normals=[];faces=[];uv=[]
    for n,points in [([0,0,1],[[-1,-1,1],[1,-1,1],[1,1,1],[-1,1,1]]),
        ([0,0,-1],[[1,-1,-1],[-1,-1,-1],[-1,1,-1],[1,1,-1]]),
        ([1,0,0],[[1,-1,1],[1,-1,-1],[1,1,-1],[1,1,1]]),
        ([-1,0,0],[[-1,-1,-1],[-1,-1,1],[-1,1,1],[-1,1,-1]]),
        ([0,1,0],[[-1,1,1],[1,1,1],[1,1,-1],[-1,1,-1]]),
        ([0,-1,0],[[-1,-1,-1],[1,-1,-1],[1,-1,1],[-1,-1,1]])]:
        start=len(vertices);vertices.extend(points);normals.extend([n]*4)
        uv.extend([[0,0],[1,0],[1,1],[0,1]]);faces.extend([[start,start+1,start+2],[start,start+2,start+3]])
    vertices=np.array(vertices)*[2,1.5,1]+[65,1.5,44]
    mesh=trimesh.Trimesh(vertices=vertices,faces=faces,vertex_normals=normals,process=False)
    mesh.visual=trimesh.visual.TextureVisuals(uv=uv,material=trimesh.visual.material.PBRMaterial(
        name='bound_projected_cube',baseColorFactor=[.5,.5,.5,1],metallicFactor=0,roughnessFactor=1))
    return mesh


def create(output,base_x=0,base_z=0,session=False):
    output=Path(output).resolve()
    if output.exists():raise FileExistsError('Use a fresh bound material fixture directory')
    if any(type(v) is not int or abs(v)>7000000000 for v in [base_x,base_z]):raise ValueError('Fixture source frame exceeds global bounds')
    output.mkdir(parents=True);shared=output/'shared';shared.mkdir()
    y,x=np.mgrid[:32,:32]
    pictures={'top':np.stack([40+x,100+y*2,30+x//2],-1).astype('uint8'),
              'side':np.stack([90+x,65+y,35+x//2],-1).astype('uint8'),
              'normal':np.tile(np.array([128,128,255,255],dtype='uint8'),(32,32,1))}
    uris={}
    for role,pixels in pictures.items():
        path=output/(role+'.png');Image.fromarray(pixels).save(path)
        data=path.read_bytes();descriptor=dict(sha256=list(hashlib.sha256(data).digest()),bytes=len(data),
            color_space='linear' if role=='normal' else 'srgb',sampler=[10497,10497,9729,9729])
        name=filename(descriptor);path.rename(shared/name);uris[role]='../shared/'+name
    entries=[]
    for owner in [base_x-1,base_x,base_x+1]:
        directory=output/f'chunk_{owner}_{base_z}';directory.mkdir()
        floor=trimesh.Trimesh(vertices=[[-1,0,-1],[-1,0,129],[129,0,-1],[129,0,129]],faces=[[0,1,2],[2,1,3]],process=False)
        floor.visual=trimesh.visual.TextureVisuals(uv=[[0,0],[0,1],[1,0],[1,1]],
            material=trimesh.visual.material.PBRMaterial(name='standard_floor',baseColorTexture=Image.new('RGBA',(4,4),'white'),baseColorFactor=[.22,.65,.3,1],metallicFactor=0,roughnessFactor=1))
        scene=trimesh.Scene(floor);scene.add_geometry(cube())
        raw=directory/'raw.glb';raw.write_bytes(trimesh.exchange.gltf.export_glb(scene));document,binary=read(raw)
        binary,_=extract_document(document,binary,shared);raw.unlink()
        identifier=owner-base_x+2
        material=next(m for m in document['materials'] if m.get('name')=='bound_projected_cube')
        material['extras']={'theia_triplanar':identifier,'theia_triplanar_profile':{
            'schema':1,'profile':'synty-triplanar-world-v1','id':identifier,'tiling':.7,'falloff':4.1,
            'source_anchor':[base_x*128,0,base_z*128],
            'textures':dict(top=uris['top'],side=uris['side'],top_normal=uris['normal'],side_normal=uris['normal'])}}
        write_glb(document,binary,directory/'terrain.glb')
        profile=dict(schema=4,bounds=[[-1,-1],[129,129]],source_origin=[owner*128,0,base_z*128],
            visual_sha256=list(bytes.fromhex(hashlib.sha256((directory/'terrain.glb').read_bytes()).hexdigest())),
            triangles=[],spawns=[],static_shapes=dict(objects=[dict(kind='box',center=[65,1.5,44],size=[4,3,2],rotation=[0,0,0,1])],
                heightfields=[dict(rows=2,columns=2,heights_column_major=[0]*4,center=[64,0,64],scale=[130,1,130])]))
        (directory/'terrain.json').write_text(json.dumps(profile,separators=(',',':'))+'\n')
        published=package(directory,f'theia.fixture.projected.x{owner}-z{base_z}',1,shared_images=True)
        entries.append(dict(id=[owner,base_z],package=list(bytes.fromhex(published['identity']))))
    (output/'world.chunks.json').write_text(json.dumps(dict(schema=1,chunk_metres=128,query_origin=[base_x*128,0,base_z*128],chunks=entries))+'\n')
    derived=output/'world.contributors.json';proof=author(output,derived,resident_limit=3);derived.replace(output/'world.chunks.json')
    if session:
        document=dict(schema=2,resident_budget=3,initial_chunks=[e['id'] for e in entries],fall_plane=-18,
            world_bounds=[[(base_x-1)*128+1,base_z*128+1],[(base_x+2)*128-1,base_z*128+127]],
            spawns=[[base_x*128+60,0,base_z*128+50],[base_x*128+62,0,base_z*128+50]])
        (output/'world.session.json').write_text(json.dumps(document)+'\n')
    return dict(packages=len(entries),shared_files=len(list(shared.iterdir())),index=proof,session=session,
        scope='Original immutable material-profile qualification data; normal rendering remains a separate gate')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path)
    parser.add_argument('--base-x',type=int,default=0);parser.add_argument('--base-z',type=int,default=0);parser.add_argument('--session',action='store_true')
    args=parser.parse_args();print(json.dumps(create(args.output,args.base_x,args.base_z,args.session),indent=2))
