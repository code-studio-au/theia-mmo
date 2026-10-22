"""Publish full-sample, cell-aligned terrain as bounded shared-image packages.

The source GLB remains immutable. This profile rejects cross-cell triangles,
never clips/decimates them, and uses existing collision/package authoring tools.
It does not supply object collision or adopt the forest as the default map.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import numpy as np
from add_terrain_house import write_glb
from externalize_world_images import externalize
from index_chunk_contributors import package_bounds
from merge_static_glb import read
from package_world import valid_id,MAX_VISUAL_BYTES,file_hash
from prepare_collision_world import prepare
from static_geometry import accessor,hierarchy


def source_mesh(stage):
    stage=Path(stage);report=json.loads((stage/'conversion.json').read_text());path=stage/'terrain.glb'
    digest=file_hash(path,MAX_VISUAL_BYTES)[1].hex()
    if report['outputs']['terrain.glb']!=digest:raise ValueError('Terrain conversion source hash differs')
    document,binary=read(path);_,world=hierarchy(document)
    if len(document['nodes'])!=1 or len(document['meshes'])!=1 or len(document['images'])!=1 or not np.array_equal(world[0],np.eye(4)):
        raise ValueError('Expected one canonical identity terrain mesh/image')
    primitive,=document['meshes'][document['nodes'][0]['mesh']]['primitives']
    if set(primitive['attributes'])!={'POSITION','NORMAL','TEXCOORD_0'} or primitive.get('mode',4)!=4 or primitive.get('targets'):
        raise ValueError('Unsupported source terrain primitive')
    arrays={key:accessor(document,binary,index) for key,index in primitive['attributes'].items()}
    if any(document['accessors'][index]['componentType']!=5126 for index in primitive['attributes'].values()):raise ValueError('Terrain attributes must be float32')
    if any(len(value)!=len(arrays['POSITION']) or not np.isfinite(value).all() for value in arrays.values()):raise ValueError('Invalid terrain attribute layout')
    index=document['accessors'][primitive['indices']]
    if index['componentType'] not in (5121,5123,5125) or index['type']!='SCALAR':raise ValueError('Terrain indices must be unsigned integers')
    if any(arrays[key].shape[1]!=(2 if key=='TEXCOORD_0' else 3) for key in arrays):raise ValueError('Invalid terrain attribute dimensions')
    faces=accessor(document,binary,primitive['indices']).reshape(-1,3)
    if not len(faces) or faces.max()>=len(arrays['POSITION']) or report['vertices']!=len(arrays['POSITION']) or report['triangles']!=len(faces):
        raise ValueError('Terrain source counts/indices differ')
    return document,binary,arrays,faces,digest


def owners(positions,faces):
    triangles=positions[faces].astype(float);points=triangles.mean(1)[:,[0,2]]
    cells=np.floor(points/128).astype(np.int64)
    if np.abs(cells).max()>7_000_000_000:raise ValueError('Terrain owners exceed global chunk range')
    lower=cells[:,None,:]*128;upper=lower+128;xz=triangles[:,:,[0,2]]
    if (xz<lower).any() or (xz>upper).any():raise ValueError('Terrain triangle crosses a chunk plane; explicit clipping required')
    unique=np.unique(cells,axis=0)
    if not 1<=len(unique)<=32:raise ValueError('Terrain package count exceeds bounded startup budget')
    return cells,unique


def slice_mesh(document,binary,arrays,faces,output):
    chosen,remapped=np.unique(faces.ravel(),return_inverse=True);result=copy.deepcopy(document);packed=bytearray();views=[];accessors=[]
    def append(data,target=None):
        packed.extend(b'\0'*(-len(packed)%4));item={'buffer':0,'byteOffset':len(packed),'byteLength':len(data)}
        if target:item['target']=target
        views.append(item);packed.extend(data);return len(views)-1
    primitive=result['meshes'][0]['primitives'][0]
    for name in ['POSITION','NORMAL','TEXCOORD_0']:
        values=np.ascontiguousarray(arrays[name][chosen],dtype='<f4');entry={'bufferView':append(values.tobytes(),34962),'componentType':5126,'count':len(values),'type':'VEC2' if name=='TEXCOORD_0' else 'VEC3'}
        if name=='POSITION':entry.update(min=values.min(0).tolist(),max=values.max(0).tolist())
        primitive['attributes'][name]=len(accessors);accessors.append(entry)
    indices=remapped.astype('<u4');primitive['indices']=len(accessors);accessors.append({'bufferView':append(indices.tobytes(),34963),'componentType':5125,'count':len(indices),'type':'SCALAR'})
    original=document['bufferViews'][document['images'][0]['bufferView']];start=original.get('byteOffset',0)
    result['images'][0]['bufferView']=append(binary[start:start+original['byteLength']])
    result['bufferViews']=views;result['accessors']=accessors;result['buffers'][0]['byteLength']=len(packed)
    packed.extend(b'\0'*(-len(packed)%4));write_glb(result,packed,output)


def publish(stage,output,content_id,spawns):
    stage=Path(stage).resolve(strict=True);output=Path(output).resolve()
    if output.exists() or output.is_relative_to(stage):raise FileExistsError('Use a new output outside the terrain stage')
    if not valid_id(content_id):raise ValueError('Invalid terrain content ID')
    spawns=np.asarray(spawns,dtype=float)
    if spawns.ndim!=2 or spawns.shape[1]!=3 or not 1<=len(spawns)<=8 or not np.isfinite(spawns).all():raise ValueError('Supply one to eight explicit trusted spawn guesses')
    document,binary,arrays,faces,digest=source_mesh(stage);cells,unique=owners(arrays['POSITION'],faces)
    low=arrays['POSITION'][:,[0,2]].min(0).astype(float);high=arrays['POSITION'][:,[0,2]].max(0).astype(float)
    if (spawns[:,[0,2]]<low+.355).any() or (spawns[:,[0,2]]>high-.355).any():raise ValueError('Spawn footprint outside finite terrain')
    minimum_y=float(arrays['POSITION'][:,1].min());entries=[];reports=[];output.mkdir(parents=True)
    try:
        with tempfile.TemporaryDirectory(prefix='terrain-authoring-',dir=output.parent) as temporary:
            temporary=Path(temporary)
            for owner in unique:
                mask=np.all(cells==owner,axis=1);selected=faces[mask];x,z=map(int,owner)
                if len(selected)>131072:raise ValueError('Full-sample owner exceeds collision triangle budget')
                visual=temporary/f'owner_{x}_{z}.glb';slice_mesh(document,binary,arrays,selected,visual)
                name=f'{content_id}.x{"m" if x<0 else "p"}{abs(x)}.z{"m" if z<0 else "p"}{abs(z)}'
                if not valid_id(name):raise ValueError('Terrain owner ID exceeds content ID budget')
                embedded=temporary/f'embedded_{x}_{z}';prepare(visual,visual,embedded,[],name,1,origin=(x*128,0,z*128))
                directory=output/f'chunk_{x}_{z}';externalize(embedded,directory,name,2)
                manifest=json.loads((directory/'world.manifest.json').read_text())
                from package_world import identity
                binding=list(bytes.fromhex(identity(manifest)));bounds=package_bounds(directory,binding)
                entries.append({'id':[x,z],'package':binding,'bounds':bounds})
                reports.append({'id':[x,z],'triangles':len(selected),'vertices':len(np.unique(selected)),
                                'collision_bytes':manifest['collision_bytes'],'visual_bytes':manifest['visual_bytes']})
        index={'schema':2,'chunk_metres':128,'query_origin':[0.,0.,0.],'chunks':entries}
        session={'schema':2,'resident_budget':len(entries),'initial_chunks':[entry['id'] for entry in entries],
                 'spawns':spawns.tolist(),'fall_plane':minimum_y-32,'world_bounds':[low.tolist(),high.tolist()]}
        if file_hash(stage/'terrain.glb',MAX_VISUAL_BYTES)[1].hex()!=digest:raise ValueError('Source terrain changed during publishing')
        (output/'world.chunks.json').write_text(json.dumps(index,indent=2)+'\n');(output/'world.session.json').write_text(json.dumps(session,indent=2)+'\n')
        report={'schema':1,'source_visual_sha256':digest,'chunks':reports,'triangles':sum(v['triangles'] for v in reports),
                'shared_images':len(list((output/'shared').iterdir())),
                'scope':'full-sample ground-only packages; trusted spawn guesses require runtime support validation; no objects/full forest adoption'}
        (output/'terrain-packages.json').write_text(json.dumps(report,indent=2)+'\n');return report
    except BaseException:
        shutil.rmtree(output);raise


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('stage',type=Path);parser.add_argument('output',type=Path);parser.add_argument('--content-id',required=True);parser.add_argument('--spawn',type=float,nargs=3,action='append',required=True);args=parser.parse_args()
    print(json.dumps(publish(args.stage,args.output,args.content_id,args.spawn)))
