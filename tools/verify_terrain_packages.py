"""Independent full-coverage terrain package oracle; no packager imports."""
import argparse
import hashlib
import json
from pathlib import Path
import numpy as np
from merge_static_glb import read
from package_world import identity
from shared_images import collect
from static_geometry import accessor,hierarchy


def oriented_rows(faces):
    start=faces.argmin(1);faces=np.take_along_axis(faces,(start[:,None]+np.arange(3))%3,axis=1)
    return faces[np.lexsort((faces[:,2],faces[:,1],faces[:,0]))]


def verify(source,output):
    source=Path(source);output=Path(output);original,binary=read(source/'terrain.glb')
    source_hash=hashlib.sha256((source/'terrain.glb').read_bytes()).hexdigest()
    if source_hash!=json.loads((source/'conversion.json').read_text())['outputs']['terrain.glb']:raise ValueError('Source conversion binding differs')
    primitive,=original['meshes'][0]['primitives'];positions=accessor(original,binary,primitive['attributes']['POSITION']).astype(float)
    attributes={k:accessor(original,binary,i) for k,i in primitive['attributes'].items() if k!='POSITION'}
    source_faces=accessor(original,binary,primitive['indices']).reshape(-1,3).astype(np.int64)
    lookup={(float(p[0]),float(p[2])):i for i,p in enumerate(positions)}
    if len(lookup)!=len(positions):raise ValueError('Ground source requires unique XZ samples')
    index=json.loads((output/'world.chunks.json').read_text());session=json.loads((output/'world.session.json').read_text())
    if index['schema']!=2 or index['chunk_metres']!=128 or index['query_origin']!=[0,0,0]:raise ValueError('Unexpected source terrain index/frame')
    source_owners=np.floor(positions[source_faces].mean(1)[:,[0,2]]/128).astype(np.int64)
    expected_owners={tuple(v) for v in np.unique(source_owners,axis=0)}
    actual_owners=[tuple(e['id']) for e in index['chunks']]
    if len(set(actual_owners))!=len(actual_owners) or set(actual_owners)!=expected_owners:raise ValueError('Missing/duplicate/extra terrain owner')
    def bind(points):
        points=np.asarray(points,dtype=float).reshape(-1,3)
        try:ids=np.array([lookup[(float(p[0]),float(p[2]))] for p in points],dtype=np.int64)
        except KeyError as error:raise ValueError('Package invented or moved source XZ samples') from error
        if not np.array_equal(points,positions[ids]):raise ValueError('Package changed source height samples')
        return ids
    image=original['images'][0];view=original['bufferViews'][image['bufferView']];start=view.get('byteOffset',0);image_hash=hashlib.sha256(binary[start:start+view['byteLength']]).digest()
    checked=0;seen=set();shared_paths=set();reports=[]
    for entry in index['chunks']:
        owner=np.array(entry['id']);directory=output/f'chunk_{owner[0]}_{owner[1]}';manifest=json.loads((directory/'world.manifest.json').read_text())
        if manifest['schema']!=2 or list(bytes.fromhex(identity(manifest)))!=entry['package']:raise ValueError('Package/index identity differs')
        for name,field in [('terrain.glb','visual'),('terrain.json','collision')]:
            data=(directory/name).read_bytes()
            if len(data)!=manifest[field+'_bytes'] or list(hashlib.sha256(data).digest())!=manifest[field+'_sha256']:raise ValueError('Package artifact binding differs')
        profile=json.loads((directory/'terrain.json').read_text())
        if profile['schema']!=3 or profile['spawns'] or profile.get('solids') or profile.get('walk_surfaces') or profile.get('camera_solids') or profile['visual_sha256']!=manifest['visual_sha256']:
            raise ValueError('Unexpected ground-only collision profile')
        doc,packed=read(directory/'terrain.glb',allow_external_images=True);_,world=hierarchy(doc)
        shared=collect(doc,output)
        if shared!=manifest['shared_images'] or len(shared)!=1 or bytes(shared[0]['sha256'])!=image_hash:raise ValueError('Terrain shared source image differs')
        if doc['materials']!=original['materials'] or doc['samplers']!=original['samplers']:raise ValueError('Terrain material/sampler changed')
        shared_paths.update(image['uri'] for image in doc['images'])
        mesh_nodes=[(n,matrix) for n,matrix in world.items() if 'mesh' in doc['nodes'][n]]
        if len(mesh_nodes)!=1:raise ValueError('Expected one compacted ground mesh per package')
        node,matrix=mesh_nodes[0];p,=doc['meshes'][doc['nodes'][node]['mesh']]['primitives']
        values=accessor(doc,packed,p['attributes']['POSITION']).astype(float);values=values@matrix[:3,:3].T+matrix[:3,3];values[:,[0,2]]+=owner*128
        ids=bind(values);seen.update(ids.tolist())
        for key,expected in attributes.items():
            if not np.array_equal(accessor(doc,packed,p['attributes'][key]),expected[ids]):raise ValueError('Terrain source UV/normal attribute changed')
        wanted=source_faces[np.all(source_owners==owner,axis=1)]
        faces=ids[accessor(doc,packed,p['indices']).reshape(-1,3)]
        if not np.array_equal(oriented_rows(faces),oriented_rows(wanted)):raise ValueError('Terrain visual coverage/winding differs')
        collision=np.asarray(profile['triangles'],dtype=float);collision[:,:,[0,2]]+=owner*128
        collision_ids=bind(collision).reshape(-1,3)
        if not np.array_equal(oriented_rows(collision_ids),oriented_rows(wanted)):raise ValueError('Terrain collision coverage/winding differs')
        if len(collision)>131072:raise ValueError('Terrain package exceeds current triangle budget')
        envelope=positions[wanted].reshape(-1,3)[:,[0,2]]-owner*128;bounds=[envelope.min(0).tolist(),envelope.max(0).tolist()]
        if profile['bounds']!=bounds or entry['bounds']!=bounds:raise ValueError('Terrain finite geometry bounds differ')
        checked+=len(wanted);reports.append({'id':owner.tolist(),'triangles':len(wanted),'vertices':len(ids)})
    if seen!=set(range(len(positions))) or checked!=len(source_faces) or len(shared_paths)!=1:raise ValueError('Terrain source sample/triangle/shared-image coverage incomplete')
    low=positions[:,[0,2]].min(0).tolist();high=positions[:,[0,2]].max(0).tolist()
    if session['schema']!=2 or session['world_bounds']!=[low,high] or set(map(tuple,session['initial_chunks']))!=expected_owners or session['resident_budget']!=len(expected_owners):
        raise ValueError('Ground startup/finite-world coverage differs')
    return {'passed':True,'source_visual_sha256':source_hash,'source_vertices':len(seen),'triangles':checked,'chunks':reports,'shared_image_variants':len(shared_paths),
            'scope':'exact source vertex/height/normal/UV and oriented triangle coverage for all ground visuals/collision; no Unity tessellation, object collision or runtime adoption'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('source',type=Path);parser.add_argument('output',type=Path);parser.add_argument('--report',required=True,type=Path);args=parser.parse_args()
    if args.report.exists():raise FileExistsError('Use a new report')
    result=verify(args.source,args.output);args.report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
