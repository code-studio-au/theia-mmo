#!/usr/bin/env python3
"""Migrate a verified schema-2 world into a new schema-3 package/revision.

Spawns are explicit local capsule-bottom guesses. Geometry and visual payloads
are preserved; historical inputs are never overwritten. Runtime validates spawns.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

from package_world import file_hash, package, valid_id, MAX_COLLISION_BYTES, MAX_VISUAL_BYTES


def upgrade(source, output, spawns, revision):
    if output.exists(): raise ValueError('Output directory must be new')
    profile_path=source/'terrain.json';visual=source/'terrain.glb'
    collision_bytes,collision_hash=file_hash(profile_path,MAX_COLLISION_BYTES)
    visual_bytes,visual_hash=file_hash(visual,MAX_VISUAL_BYTES)
    manifest=json.loads((source/'world.manifest.json').read_text())
    if type(revision) is not int or not valid_id(manifest['content_id']) or manifest['schema']!=1 or manifest['collision_schema']!=2 or not manifest['revision']<revision<=0xffffffff:
        raise ValueError('Expected schema-2 package and strictly newer revision')
    if (manifest['collision_bytes'],manifest['collision_sha256'],manifest['visual_bytes'],manifest['visual_sha256'])!=(collision_bytes,list(collision_hash),visual_bytes,list(visual_hash)):
        raise ValueError('Source artifacts do not match manifest')
    profile=json.loads(profile_path.read_bytes())
    if profile['schema']!=2 or profile['visual_sha256']!=list(visual_hash):raise ValueError('Profile binding disagrees')
    if not 1<=len(spawns)<=8 or any(len(p)!=3 for p in spawns):raise ValueError('World requires one to eight explicit spawns')
    profile['schema']=3
    profile['triangles']+=profile.pop('walk_surfaces',[])
    profile.pop('step_height',None)
    profile['spawns']=spawns
    data=(json.dumps(profile,separators=(',',':'),allow_nan=False)+'\n').encode()
    if len(data)>MAX_COLLISION_BYTES:raise ValueError('Collision byte budget exceeded')
    if file_hash(profile_path,MAX_COLLISION_BYTES)[1]!=collision_hash or file_hash(visual,MAX_VISUAL_BYTES)[1]!=visual_hash:
        raise ValueError('Source changed during migration')
    output.mkdir(parents=True)
    shutil.copyfile(visual,output/'terrain.glb')
    (output/'terrain.json').write_bytes(data)
    # Preserve route geometry/evidence, updating only its new profile binding.
    for name in ['access.json','farmhouse-access.json']:
        if (source/name).exists():
            access=json.loads((source/name).read_text())
            access['world_sha256']=hashlib.sha256(data).hexdigest()
            (output/name).write_text(json.dumps(access,indent=2)+'\n')
    report=package(output,manifest['content_id'],revision)
    report.update(source_profile_sha256=collision_hash.hex(),source_revision=manifest['revision'],
                  scope='schema-2 geometry retained with explicit schema-3 spawns; runtime validates support/headroom')
    (output/'migration.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source',type=Path);parser.add_argument('output',type=Path)
    parser.add_argument('--spawn',type=float,nargs=3,action='append',required=True)
    parser.add_argument('--revision',type=int,required=True)
    args=parser.parse_args()
    print(json.dumps(upgrade(args.source,args.output,args.spawn,args.revision),indent=2))
