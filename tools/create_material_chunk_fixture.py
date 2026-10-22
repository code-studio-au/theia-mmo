#!/usr/bin/env python3
"""Original shared-image chunks with standard unlit/HDR-emissive materials."""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
from PIL import Image
import trimesh
from add_terrain_house import write_glb
from externalize_world_images import externalize
from index_chunk_contributors import author
from merge_static_glb import read
from package_world import package


def create(output):
    if output.exists():raise FileExistsError('Use a fresh material fixture directory')
    output.mkdir(parents=True)
    vertices=np.array([[-1,0,-1],[-1,0,129],[129,0,-1],[129,0,129]])
    faces=[[0,1,2],[2,1,3]]
    image=Image.new('RGBA',(4,4),(255,255,255,255))
    entries=[]
    for x in [-1,0,1]:
        source=output/'embedded'/f'chunk_{x}_0';source.mkdir(parents=True)
        floor=trimesh.Trimesh(vertices=vertices,faces=faces,process=False)
        floor.visual=trimesh.visual.TextureVisuals(uv=vertices[:,[0,2]]/8,
            material=trimesh.visual.material.PBRMaterial(name='qualified_unlit_floor',
                baseColorTexture=image,baseColorFactor=[.22,.65,.3,1],metallicFactor=0,roughnessFactor=1))
        scene=trimesh.Scene(floor)
        boxes=[]
        if x==0:
            wall=trimesh.creation.box(extents=[4,2,.2]);wall.apply_translation([65,1,44])
            wall.visual=trimesh.visual.TextureVisuals(uv=np.zeros((len(wall.vertices),2)),
                material=trimesh.visual.material.PBRMaterial(name='qualified_emissive_wall',
                    baseColorFactor=[.02,.02,.02,1],emissiveFactor=[.05,.6,.6],emissiveTexture=image,metallicFactor=0,roughnessFactor=1))
            scene.add_geometry(wall)
            boxes=[dict(kind='box',center=[65,1,44],size=[4,2,.2],rotation=[0,0,0,1])]
        raw=source/'raw.glb';raw.write_bytes(trimesh.exchange.gltf.export_glb(scene))
        document,binary=read(raw)
        names=[]
        for material in document['materials']:
            if material['name']=='qualified_unlit_floor':extension={'KHR_materials_unlit':{}}
            else:extension={'KHR_materials_emissive_strength':{'emissiveStrength':2.6}}
            material['extensions']=extension;names.extend(extension)
        document['extensionsUsed']=sorted(set(names));document['extensionsRequired']=document['extensionsUsed'][:]
        write_glb(document,binary,source/'terrain.glb')
        raw.unlink()
        visual=(source/'terrain.glb').read_bytes()
        profile={'schema':4,'spawns':[],'bounds':[[-1,-1],[129,129]],'source_origin':[0,0,0],
            'visual_sha256':list(hashlib.sha256(visual).digest()),'triangles':[],
            'static_shapes':{'objects':boxes,'heightfields':[{'rows':2,'columns':2,
                'heights_column_major':[0]*4,'center':[64,0,64],'scale':[130,1,130]}]}}
        (source/'terrain.json').write_text(json.dumps(profile)+'\n')
        content_id=f'theia.fixture.materials.x{x}'
        package(source,content_id,2)
        result=externalize(source,output/f'chunk_{x}_0',content_id,3)
        entries.append({'id':[x,0],'package':list(bytes.fromhex(result['identity']))})
    index={'schema':1,'chunk_metres':128,'query_origin':[0,0,0],'chunks':entries}
    (output/'world.chunks.json').write_text(json.dumps(index)+'\n')
    derived=output/'world.contributors.json';author(output,derived,resident_limit=3)
    derived.replace(output/'world.chunks.json')
    session={'schema':1,'resident_budget':3,'initial_chunks':[e['id'] for e in entries],
             'spawns':[[60,0,50],[62,0,50]],'fall_plane':-18}
    (output/'world.session.json').write_text(json.dumps(session)+'\n')
    return {'chunks':3,'shared_files':len(list((output/'shared').iterdir())),
            'scope':'Original bounded material/field package qualification; no forest map adoption'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path)
    print(json.dumps(create(parser.parse_args().output),indent=2))
