#!/usr/bin/env python3
"""Original multi-room static mesh fixture through the real authoring pipeline."""
import argparse
from pathlib import Path
import json
import numpy as np
import trimesh
from prepare_collision_world import prepare


def create(output):
    if output.exists():raise ValueError('Output directory must be new')
    scene=trimesh.Scene()
    for name,centre,size,colour in [
        ('floor',[40,-.25,40],[40,.5,40],[110,140,125,255]),
        ('west',[20,3,40],[.2,6,40],[140,110,85,255]),
        ('east',[60,3,40],[.2,6,40],[140,110,85,255]),
        ('north',[40,3,20],[40,6,.2],[140,110,85,255]),
        ('south',[40,3,60],[40,6,.2],[140,110,85,255]),
        ('ceiling',[40,6.1,40],[40,.2,40],[155,145,125,255]),
        ('partition',[48,2,50],[.2,4,20],[110,100,145,255]),
        ('step',[44,.125,40],[3,.25,4],[180,145,80,255]),
        ('overpass',[50,3,30],[10,.2,10],[80,140,180,255]),
    ]:
        mesh=trimesh.creation.box(extents=size);mesh.visual.vertex_colors=np.tile(colour,(len(mesh.vertices),1))
        scene.add_geometry(mesh,geom_name=name,node_name=name,transform=trimesh.transformations.translation_matrix(centre))
    output.mkdir(parents=True)
    source=output/'original.glb';source.write_bytes(trimesh.exchange.gltf.export_glb(scene))
    return prepare(source,source,output/'world',[[32,0,32],[36,0,32]],'theia.fixture.general-collision',1)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path)
    print(json.dumps(create(parser.parse_args().output),indent=2))
