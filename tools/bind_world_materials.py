#!/usr/bin/env python3
"""Embed qualified source projection sidecars into immutable runtime packages.

Keeps authored world zero, all source geometry and complete reference collision.
No session, spawn or water gameplay policy is published by this conversion.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
from add_terrain_house import write_glb
from material_profiles import parse
from merge_static_glb import read
from package_world import package
from shared_images import filename
from unity_editor_reference import digest, read_json
from verify_shape_reference_visuals import verify


def link_checked(source,target):
    if target.exists():
        if digest(source)!=digest(target):raise ValueError('Conflicting immutable image bytes')
        return
    try:os.link(source,target)
    except OSError:shutil.copyfile(source,target)
    if digest(source)!=digest(target):raise ValueError('Immutable image copy differs')


def bind(stage,profiles,source,output):
    stage,profiles,source=map(lambda p:Path(p).resolve(strict=True),(stage,profiles,source))
    output=Path(output).resolve();build=Path(__file__).resolve().parents[1]/'build'
    if output.exists() or output==build or not output.is_relative_to(build) or output.is_relative_to(source):raise ValueError('Use a fresh ignored material-bound output')
    checked=verify(stage,profiles,source)
    index=read_json(source/'world.chunks.json',32*1024);receipt=read_json(source/'reference.json',4*1024*1024)
    rows={tuple(p['owner']):p for p in receipt['packages']}
    output.mkdir(parents=True);(output/'shared').mkdir()
    try:
        for path in (source/'shared').iterdir():
            if not path.is_file():raise ValueError('Unexpected source shared resource')
            link_checked(path,output/'shared'/path.name)
        entries=[];reports=[]
        for entry in index['chunks']:
            owner=tuple(entry['id']);original=source/f'chunk_{owner[0]}_{owner[1]}'
            target=output/original.name;target.mkdir()
            manifest=read_json(original/'world.manifest.json',256*1024)
            document,binary=read(original/'terrain.glb',allow_external_images=True)
            catalog={}
            if rows[owner]['catalog_sha256'] is not None:
                path=original/'triplanar.json'
                if digest(path)!=rows[owner]['catalog_sha256']:raise ValueError('Qualified source sidecar changed')
                metadata=read_json(path,64*1024)
                if metadata['geometry_sha256']!=digest(original/'terrain.glb'):raise ValueError('Source material geometry binding differs')
                catalog={p['id']:p for p in metadata['profiles']}
            used=set()
            for material in document.get('materials',[]):
                extras=material.get('extras',{});identifier=extras.get('theia_triplanar')
                if identifier is None:continue
                if identifier not in catalog or 'theia_triplanar_profile' in extras:raise ValueError('Unbound or already converted source mark')
                profile=catalog[identifier];textures={}
                for role,value in profile['textures'].items():
                    path=original/value['file']
                    if digest(path)!=value['sha256']:raise ValueError('Source projection PNG binding differs')
                    descriptor=dict(sha256=list(bytes.fromhex(value['sha256'])),bytes=path.stat().st_size,
                        color_space='linear' if role.endswith('normal') else 'srgb',sampler=[10497,10497,9729,9729])
                    name=filename(descriptor);link_checked(path,output/'shared'/name);textures[role]='../shared/'+name
                extras['theia_triplanar_profile']=dict(schema=1,profile='synty-triplanar-world-v1',id=identifier,
                    tiling=profile['tiling'],falloff=profile['falloff'],source_anchor=[0,0,0],textures=textures)
                parse(extras);used.add(identifier)
            if used!=set(catalog):raise ValueError('Source profile coverage differs')
            write_glb(document,binary,target/'terrain.glb')
            collision=read_json(original/'terrain.json',32*1024*1024)
            collision['visual_sha256']=list(bytes.fromhex(digest(target/'terrain.glb')))
            (target/'terrain.json').write_text(json.dumps(collision,separators=(',',':'),allow_nan=False)+'\n')
            published=package(target,manifest['content_id'],manifest['revision']+1,shared_images=True)
            entries.append(dict(id=list(owner),package=list(bytes.fromhex(published['identity'])),bounds=entry['bounds']))
            reports.append(dict(owner=list(owner),source_manifest_sha256=digest(original/'world.manifest.json'),
                manifest_sha256=digest(target/'world.manifest.json'),source_visual_sha256=digest(original/'terrain.glb'),
                visual_sha256=digest(target/'terrain.glb'),bound_profiles=sorted(used)))
        index['chunks']=entries;(output/'world.chunks.json').write_text(json.dumps(index,separators=(',',':'))+'\n')
        result=dict(schema=1,profile='bound-source-world-materials-v1',source=str(source),source_reference_sha256=digest(source/'reference.json'),
            source_verification=checked,packages=reports,index_sha256=digest(output/'world.chunks.json'),
            scope='Reference-only material-bound source geometry; no session/spawns, water policy, normal-map adoption or runtime resource/capacity claim')
        (output/'reference.json').write_text(json.dumps(result,indent=2)+'\n')
        return dict(packages=len(entries),shared_images=len(list((output/'shared').iterdir())),profile_packages=sum(bool(p['bound_profiles']) for p in reports))
    except BaseException:
        shutil.rmtree(output);raise


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['stage','profiles','source','output']:parser.add_argument(name,type=Path)
    args=parser.parse_args();print(json.dumps(bind(args.stage,args.profiles,args.source,args.output),indent=2))
