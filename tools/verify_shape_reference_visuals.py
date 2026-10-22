#!/usr/bin/env python3
"""Independently verify complete source draw streams/poses in shape packages."""
import argparse
import copy
import json
from pathlib import Path
import numpy as np
from merge_static_glb import read
from package_world import identity
from shared_images import verify as verify_images
from static_geometry import accessor, hierarchy
from unity_editor_reference import digest, read_json
from verify_world_partition import material


def require(condition,message):
    if not condition:raise ValueError(message)


def normalized_images(document):
    result=copy.deepcopy(document)
    for image in result.get('images',[]):
        image['uri']=image['uri'].removeprefix('../')
    return result


def verify(stage, profiles, output):
    stage,profiles,output=map(Path,(stage,profiles,output))
    receipt=read_json(output/'reference.json',4*1024*1024)
    require(receipt['profile']=='source-shape-visual-reference-v1' and not (output/'world.session.json').exists(),'Reference-only publication boundary differs')
    require(receipt['source_visual_sha256']==digest(stage/'preview.glb') and receipt['source_catalog_sha256']==digest(stage/'triplanar.json') and receipt['source_shapes_sha256']==digest(profiles/'reference.json'),'Source stage bindings differ')
    source,binary=read(stage/'preview.glb',allow_external_images=True)
    source_materials=normalized_images(source)
    _,source_world=hierarchy(source)
    expected={n for n in source_world if 'mesh' in source['nodes'][n]}
    collision=read_json(profiles/'reference.json',2*1024*1024)
    by_owner={tuple(p['owner']):p for p in collision['profiles']}
    index=read_json(output/'world.chunks.json',32*1024)
    require(index['schema']==2 and index['query_origin']==[0,0,0],'Candidate index frame differs')
    package_ids={tuple(p['id']):p['package'] for p in index['chunks']}
    require(len(package_ids)==len(index['chunks'])==len(receipt['packages'])==len(by_owner),'Candidate owner coverage differs')
    catalog=read_json(stage/'triplanar.json',64*1024)
    seen=set();streams=0;primitive_instances=0;maximum=0.
    for row in receipt['packages']:
        owner=tuple(row['owner']);directory=output/f'chunk_{owner[0]}_{owner[1]}'
        require(row['visual_sha256']==digest(directory/'terrain.glb'),'Candidate visual binding differs')
        manifest=read_json(directory/'world.manifest.json',256*1024)
        require(package_ids[owner]==list(bytes.fromhex(identity(manifest))),'Candidate package identity differs')
        require(manifest['visual_sha256']==list(bytes.fromhex(digest(directory/'terrain.glb'))) and manifest['collision_sha256']==list(bytes.fromhex(digest(directory/'terrain.json'))),'Candidate manifest bytes differ')
        after,data=read(directory/'terrain.glb',allow_external_images=True)
        verify_images(after,output,manifest['shared_images'])
        after_materials=normalized_images(after)
        _,after_world=hierarchy(after)
        actual={n for n in after_world if 'mesh' in after['nodes'][n]}
        mapping={int(a):b for a,b in row['node_mapping'].items()}
        require(set(mapping)==set(row['source_nodes']) and len(mapping)==len(set(mapping.values())) and set(mapping.values())==actual,'Draw node mapping is incomplete/repeated')
        checked=set()
        for before_node,after_node in mapping.items():
            require(before_node in expected and before_node not in seen,'Unexpected/repeated source draw')
            seen.add(before_node)
            before_fields={k:v for k,v in source['nodes'][before_node].items() if k not in ['mesh','children']}
            after_fields={k:v for k,v in after['nodes'][after_node].items() if k not in ['mesh','children']}
            require(before_fields==after_fields,'Source draw node fields differ')
            world=after_world[after_node].copy();world[:3,3]+=np.array([owner[0]*128,0,owner[1]*128])
            error=float(abs(world-source_world[before_node]).max());maximum=max(maximum,error)
            require(error<1e-9,'Rebased source draw pose differs')
            first=source['meshes'][source['nodes'][before_node]['mesh']]['primitives']
            second=after['meshes'][after['nodes'][after_node]['mesh']]['primitives']
            require(len(first)==len(second),'Source primitive coverage differs')
            primitive_instances+=len(first)
            for a,b in zip(first,second):
                require({k:v for k,v in a.items() if k not in ['attributes','indices','material']}=={k:v for k,v in b.items() if k not in ['attributes','indices','material']},'Primitive semantics differ')
                require(set(a['attributes'])==set(b['attributes']) and ('indices' in a)==('indices' in b),'Source streams omitted')
                pairs=list(zip((a['attributes'][k] for k in sorted(a['attributes'])),(b['attributes'][k] for k in sorted(b['attributes']))))
                if 'indices' in a:pairs.append((a['indices'],b['indices']))
                for old,new in pairs:
                    if (old,new) in checked:continue
                    checked.add((old,new));streams+=1
                    left,right=accessor(source,binary,old),accessor(after,data,new)
                    require(left.shape==right.shape and left.dtype==right.dtype and left.tobytes()==right.tobytes(),'Complete source vertex/index/normal/UV/tangent/colour stream differs')
                require(material(source_materials,a.get('material'))==material(after_materials,b.get('material')),'Source material/image/sampler/extension semantics differ')
        source_profile=profiles/by_owner[owner]['file']
        require(digest(source_profile)==row['source_profile_sha256']==by_owner[owner]['sha256'],'Source collision binding differs')
        old=read_json(source_profile,32*1024*1024);new=read_json(directory/'terrain.json',32*1024*1024)
        old['visual_sha256']=list(bytes.fromhex(row['visual_sha256']))
        require(new==old,'Complete collision profile differs beyond new visual binding')
        used={m.get('extras',{}).get('theia_triplanar') for m in after.get('materials',[])}-{None}
        if used:
            path=directory/'triplanar.json';expected_catalog=copy.deepcopy(catalog)
            expected_catalog['geometry_sha256']=row['visual_sha256']
            expected_catalog['profiles']=[p for p in catalog['profiles'] if p['id'] in used]
            require(row['catalog_sha256']==digest(path) and read_json(path,64*1024)==expected_catalog,'Retained custom material catalog differs')
            for p in expected_catalog['profiles']:
                for image in p['textures'].values():require(digest(directory/image['file'])==digest(stage/image['file'])==image['sha256'],'Custom image source bytes differ')
        else:require(row['catalog_sha256'] is None and not (directory/'triplanar.json').exists(),'Unexpected custom catalog')
    require(seen==expected,'Complete source draw coverage differs')
    return dict(passed=True,draw_nodes=len(seen),primitive_instances=primitive_instances,streams_checked=streams,
                maximum_pose_error=maximum,packages=len(receipt['packages']),reference_sha256=digest(output/'reference.json'),
                scope='Independent complete draw/stream/pose/material/custom-image/collision binding; no normal-runtime material, movement, water or adoption claim')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['stage','profiles','output']:parser.add_argument(name,type=Path)
    args=parser.parse_args();print(json.dumps(verify(args.stage,args.profiles,args.output),indent=2))
