"""Independent source-pixel adapter for actual connected runtime diagnostics.

Uses immutable package geometry and the captured render-local camera. The adapter
only renumbers equivalent source-local profile IDs and adds checked root frame
translations for the existing geometric/PNG oracle; it does not render a proxy.
"""
import hashlib
import json
from pathlib import Path
import shutil
import numpy as np
from add_terrain_house import write_glb
from material_profiles import parse, ROLES
from merge_static_glb import read, merge_documents
from verify_triplanar_pixels import verify


def check(world,capture,output):
    world,capture=map(Path,(world,capture));output=Path(output).resolve()
    build=Path(__file__).resolve().parents[1]/'build'
    if output.exists() or not output.is_relative_to(build):raise ValueError('Use a fresh ignored pixel adapter directory')
    report=json.loads((capture/'network.json').read_text())
    bindings=report['chunk_world']['material_profiles']['bindings']
    modes={int(row['diagnostic']) for row in bindings}
    if len(modes)!=1 or not modes<={1,2}:raise ValueError('Use an actual runtime colour/normal diagnostic')
    mode,=modes;origin=np.array(report['coordinates']['render_origin'],dtype=float)
    combined=None;packed=bytearray();profiles={};entries=[];anchors=set()
    output.mkdir(parents=True)
    for owner in report['chunk_world']['loaded']:
        package=world/f'chunk_{owner[0]}_{owner[1]}'
        document,binary=read(package/'terrain.glb',allow_external_images=True)
        for material in document.get('materials',[]):
            profile=parse(material.get('extras'))
            if profile is None:continue
            key=(profile['tiling'],profile['falloff'],tuple(profile['source_anchor']),tuple(profile['textures'][role] for role,_ in ROLES))
            anchors.add(tuple(profile['source_anchor']))
            if key not in profiles:
                identifier=len(profiles)+1;profiles[key]=identifier
                textures={}
                for role,_ in ROLES:
                    uri=profile['textures'][role];name=Path(uri).name;source=world/'shared'/name
                    target=output/name
                    if not target.exists():shutil.copyfile(source,target)
                    textures[role]=dict(file=name,sha256=hashlib.sha256(source.read_bytes()).hexdigest())
                entries.append(dict(id=identifier,profile='synty-triplanar-world-v1',tiling=profile['tiling'],falloff=profile['falloff'],textures=textures))
            material['extras']['theia_triplanar']=profiles[key]
            material['extras']['theia_triplanar_profile']['id']=profiles[key]
        local=(np.array([owner[0]*128,0,owner[1]*128],dtype=float)-origin).astype(np.float32)
        wrapper=len(document['nodes']);document['nodes'].append(dict(name='Actual render frame root',translation=local.tolist(),children=document['scenes'][0]['nodes']))
        document['scenes'][0]['nodes']=[wrapper]
        if combined is None:combined=document;packed=bytearray(binary)
        else:
            combined=merge_documents(combined,document,len(packed),allow_external_images=True);packed.extend(binary)
    if len(anchors)!=1:raise ValueError('Diagnostic adapter requires one authored source frame')
    combined['buffers'][0]['byteLength']=len(packed)
    write_glb(combined,packed,output/'preview.glb')
    metadata=dict(schema=2,profile='synty-triplanar-catalog-v1',geometry_sha256=hashlib.sha256((output/'preview.glb').read_bytes()).hexdigest(),profiles=entries)
    (output/'triplanar.json').write_text(json.dumps(metadata)+'\n')
    adapted=output/'capture';adapted.mkdir();shutil.copyfile(capture/'scene.png',adapted/'scene.png')
    (adapted/'asset.json').write_text(json.dumps(dict(clip_from_world=report['clip_from_world'],stats=dict(triplanar=dict(diagnostic_unlit=mode))))+'\n')
    anchor,=anchors
    proof=verify(output,adapted,texture_origin=origin-np.array(anchor),sample_step=13)
    proof['scope']='Actual connected render-local camera, immutable package geometry and independently sampled source PNGs; diagnostic ID normalization only, no proxy render'
    (output/'verification.json').write_text(json.dumps(proof,indent=2)+'\n')
    return {k:v for k,v in proof.items() if k!='results'}
