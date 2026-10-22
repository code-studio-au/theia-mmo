#!/usr/bin/env python3
"""Join verified shape references and whole static draw shards, without adoption.

Preserves original material/mesh streams and complete full-sample terrain. Emits
immutable candidate packages and a contributor index, but no session/spawns:
triplanar rebasing/resources, lighting/render flags and water remain separate
runtime gates. The complete source water plane stays reference-only here.
"""
import argparse
import copy
import json
import os
from pathlib import Path
import shutil
import numpy as np
from add_terrain_house import write_glb
from index_chunk_contributors import author
from merge_static_glb import read
from package_world import package
from shared_images import filename, verify as verify_images
from static_geometry import hierarchy, prune, transformed_bounds
from unity_editor_reference import digest, read_json
from verify_shape_reference_profiles import verify as verify_shapes


def copy_immutable(source, destination):
    try: os.link(source,destination)
    except OSError: shutil.copyfile(source,destination)
    if digest(source)!=digest(destination):raise ValueError('Immutable resource copy differs')


def prepare(stage, profiles, objects, terrain, output):
    stage,profiles=Path(stage).resolve(strict=True),Path(profiles).resolve(strict=True)
    output=Path(output).resolve();build=Path(__file__).resolve().parents[1]/'build'
    if output.exists() or output==build or not output.is_relative_to(build) or output.is_relative_to(stage) or output.is_relative_to(profiles):
        raise ValueError('Use a fresh ignored candidate output outside the source stages')
    shape_proof=verify_shapes(objects,terrain,profiles)
    source=stage/'preview.glb';resources=read_json(stage/'resources.json',1024*1024)
    catalog=read_json(stage/'triplanar.json',64*1024)
    if resources['asset_sha256']!=digest(source) or resources['metadata_sha256']!=digest(stage/'triplanar.json') or catalog['geometry_sha256']!=digest(source):
        raise ValueError('Qualified source visual/catalog bindings differ')
    document,binary=read(source,allow_external_images=True)
    # Inspector resources are local to their stage. Candidate chunk GLBs use
    # the established canonical sibling-shared package paths instead.
    for image in document.get('images',[]):
        if not image.get('uri','').startswith('shared/') or 'bufferView' in image:
            raise ValueError('Unexpected qualified inspector image path')
        image['uri']='../'+image['uri']
    verify_images(document,stage,resources['shared_images'])
    reference=read_json(profiles/'reference.json',2*1024*1024)
    physical={tuple(entry['owner']):entry for entry in reference['profiles']}
    _,matrices=hierarchy(document)
    selected={}
    for index,matrix in matrices.items():
        node=document['nodes'][index]
        if 'mesh' not in node:continue
        bounds=transformed_bounds(document,binary,node['mesh'],matrix)
        owner=tuple(int(v) for v in np.floor(bounds.mean(0)[[0,2]]/128))
        selected.setdefault(owner,set()).add(index)
    if set(selected)!=set(physical):raise ValueError('Visual and physical owners differ; no fabricated support is admitted')
    output.mkdir(parents=True)
    try:
        shared=output/'shared';shared.mkdir()
        for image in resources['shared_images']:
            name=filename(image);copy_immutable(stage/'shared'/name,shared/name)
        entries=[];reports=[]
        for owner,nodes in sorted(selected.items()):
            candidate=copy.deepcopy(document)
            wrapper=len(candidate['nodes'])
            candidate['nodes'].append({'name':'Chunk source rebase','translation':[-owner[0]*128,0,-owner[1]*128],
                                       'children':candidate['scenes'][0]['nodes']})
            candidate['scenes'][0]['nodes']=[wrapper]
            reduced,packed,mapping=prune(candidate,binary,nodes,with_mapping=True)
            directory=output/f'chunk_{owner[0]}_{owner[1]}';directory.mkdir()
            write_glb(reduced,packed,directory/'terrain.glb')
            source_profile=profiles/physical[owner]['file']
            if digest(source_profile)!=physical[owner]['sha256']:raise ValueError('Source profile changed during packaging')
            profile=read_json(source_profile,32*1024*1024)
            profile['visual_sha256']=list(bytes.fromhex(digest(directory/'terrain.glb')))
            (directory/'terrain.json').write_text(json.dumps(profile,separators=(',',':'),allow_nan=False)+'\n')
            used={material.get('extras',{}).get('theia_triplanar') for material in reduced.get('materials',[])}-{None}
            metadata=copy.deepcopy(catalog);metadata['geometry_sha256']=digest(directory/'terrain.glb')
            metadata['profiles']=[p for p in catalog['profiles'] if p['id'] in used]
            if {p['id'] for p in metadata['profiles']}!=used:raise ValueError('Unbound retained triplanar profile')
            if used:
                for name in {v['file'] for p in metadata['profiles'] for v in p['textures'].values()}:
                    if Path(name).name!=name or not name.startswith('triplanar-'):raise ValueError('Nonlocal source texture path')
                    descriptor=next(v for p in metadata['profiles'] for v in p['textures'].values() if v['file']==name)
                    if digest(stage/name)!=descriptor['sha256']:raise ValueError('Source custom image binding differs')
                    copy_immutable(stage/name,directory/name)
                (directory/'triplanar.json').write_text(json.dumps(metadata,indent=2,allow_nan=False)+'\n')
            published=package(directory,f'theia.reference.forest.x{owner[0]}-z{owner[1]}',1,shared_images=True)
            entries.append({'id':list(owner),'package':list(bytes.fromhex(published['identity']))})
            reports.append(dict(owner=list(owner),source_nodes=sorted(nodes),
                node_mapping={str(n):mapping['node'][n] for n in sorted(nodes)},
                source_profile_sha256=digest(source_profile),visual_sha256=digest(directory/'terrain.glb'),
                catalog_sha256=digest(directory/'triplanar.json') if used else None,
                primitives=sum(len(m['primitives']) for m in reduced['meshes']),
                instantiated_primitives=sum(len(reduced['meshes'][reduced['nodes'][mapping['node'][n]]['mesh']]['primitives']) for n in nodes)))
        index={'schema':1,'chunk_metres':128,'query_origin':[0,0,0],'chunks':entries}
        (output/'world.chunks.json').write_text(json.dumps(index)+'\n')
        derived=output/'world.contributors.json';index_proof=author(output,derived)
        derived.replace(output/'world.chunks.json')
        proof=dict(schema=1,profile='source-shape-visual-reference-v1',source=str(stage),
            source_visual_sha256=digest(source),source_catalog_sha256=digest(stage/'triplanar.json'),
            source_shapes_sha256=digest(profiles/'reference.json'),shape_verification=shape_proof,
            index=index_proof,packages=reports,
            scope='Reference-only complete source geometry packages; no session/spawns/default adoption. Water plane remains geometry evidence. Custom material metadata is receipt-bound, not admitted by normal runtime; lighting/render flags/rebase/resources remain separate gates.')
        (output/'reference.json').write_text(json.dumps(proof,indent=2)+'\n')
        return {'packages':len(entries),'draw_nodes':sum(len(v) for v in selected.values()),'maximum_cell_contributors':index_proof['maximum_cell_contributors']}
    except BaseException:
        shutil.rmtree(output)
        raise


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['stage','profiles','objects','terrain','output']:parser.add_argument(name,type=Path)
    args=parser.parse_args();print(json.dumps(prepare(args.stage,args.profiles,args.objects,args.terrain,args.output),indent=2))
