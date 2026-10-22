"""Independent canonical FBX point and source-pose assembly checks."""
import argparse
from collections import defaultdict
import hashlib
import itertools
import json
from pathlib import Path
import subprocess
import numpy as np
from merge_static_glb import read
from static_geometry import accessor,hierarchy
from unity_scene import Sources
from unity_model_ids import identity,bound_candidates


def point_error(first,second,tolerance=.0001):
    """Bound both directions of a point-set match without quadratic allocation."""
    def direction(a,b):
        cells=defaultdict(list)
        for point in np.unique(b,axis=0):cells[tuple(np.floor(point/tolerance).astype(np.int64))].append(point)
        maximum=0.
        for point in np.unique(a,axis=0):
            key=np.floor(point/tolerance).astype(np.int64);candidates=[]
            for delta in itertools.product((-1,0,1),repeat=3):candidates.extend(cells.get(tuple(key+delta),()))
            if not candidates:raise ValueError('Canonical position has no independent FBX match')
            error=float(np.linalg.norm(np.asarray(candidates)-point,axis=1).min())
            if error>tolerance:raise ValueError('Canonical position exceeds independent FBX tolerance')
            maximum=max(maximum,error)
        return maximum
    return max(direction(np.asarray(first),np.asarray(second)),direction(np.asarray(second),np.asarray(first)))


def canonical(source,stage,probe):
    sources=Sources(source);stage=Path(stage);conversion=json.loads((stage/'conversion.json').read_text())
    if conversion['archive_sha256']!=sources.manifest['archive_sha256']:raise ValueError('Prototype source archive differs')
    model=sources.checked(conversion['model']);importer=sources.metadata(conversion['model'])['ModelImporter']
    ids,_=bound_candidates(identity(model),importer);binding=ids[conversion['mesh_fileID']]
    result=json.loads(subprocess.check_output([str(Path(probe).resolve()),str(model),'right'],timeout=30))
    if result['source_axes']!=[0,2,4] or result['ufbx_version']!=23001:raise ValueError('Unqualified source coordinate profile')
    node,=[n for n in result['nodes'] if n['name']==binding['name']]
    if node['deformed']:raise ValueError('Deformed prototype needs separate geometry qualification')
    points=np.array(node['vertices']);frame=np.array(node['geometry_to_node']);points=points@frame[:3,:3].T+frame[:3,3]
    used=np.unique(np.concatenate([face for face in node['faces'] if len(face)>=3]));points=points[used]
    doc,binary=read(stage/'preview.glb');_,world=hierarchy(doc);actual=[]
    for i,matrix in world.items():
        if 'mesh' not in doc['nodes'][i]:continue
        for p in doc['meshes'][doc['nodes'][i]['mesh']]['primitives']:
            v=accessor(doc,binary,p['attributes']['POSITION']);actual.extend(v@matrix[:3,:3].T+matrix[:3,3])
    error=point_error(points,actual)
    return {'passed':True,'max_position_error_metres':error,'tolerance_metres':.0001,
            'source_points':len(points),'prototype_vertices':len(actual),
            'scope':'canonical mesh-local positions/metres against independent ufbx; not topology, Unity Editor or material equivalence'}


def assembly(prototype,instances,path):
    original,source_binary=read(Path(prototype));doc,binary=read(Path(path));parents,world=hierarchy(doc)
    expected={v['id']:v for v in instances};roots={}
    if len(expected)!=len(instances):raise ValueError('Duplicate source instance IDs')
    for i in doc['scenes'][0]['nodes']:
        identifier=doc['nodes'][i].get('extras',{}).get('source_instance')
        if identifier in roots:raise ValueError('Duplicate output instance ID')
        roots[identifier]=i
    if set(roots)!=set(expected):raise ValueError('Missing/extra assembled source instances')
    if doc.get('materials')!=original.get('materials') or doc.get('images')!=original.get('images') or binary[:len(source_binary)]!=source_binary:
        raise ValueError('Assembly changed prototype material/image/source bytes')
    maximum=0.;bounds=[];checked=0
    for identifier,root in roots.items():
        meshes=[]
        for i,frame in world.items():
            if 'mesh' not in doc['nodes'][i]:continue
            owner=i
            while parents[owner] is not None:owner=parents[owner]
            if owner==root:meshes.append((i,frame))
        if len(meshes)!=1:raise ValueError('Expected one shared source mesh per instance')
        node,frame=meshes[0]
        if np.linalg.det(frame[:3,:3])<=0:raise ValueError('Negative node determinant can reverse runtime culling')
        primitives=doc['meshes'][doc['nodes'][node]['mesh']]['primitives'];source_primitives=original['meshes'][0]['primitives']
        if len(primitives)!=len(source_primitives):raise ValueError('Primitive set changed')
        matrix=np.array(expected[identifier]['matrix'],dtype=np.float64)
        # Independent, explicit handedness operations on points/normals.
        target=matrix.copy();target[0,:]*=-1;target[:,0]*=-1
        for p,source in zip(primitives,source_primitives):
            source_points=accessor(original,source_binary,source['attributes']['POSITION']).astype(float)
            wanted=source_points.copy();wanted[:,0]*=-1;wanted=wanted@matrix[:3,:3].T+matrix[:3,3];wanted[:,0]*=-1
            actual=accessor(doc,binary,p['attributes']['POSITION']).astype(float);actual=actual@frame[:3,:3].T+frame[:3,3]
            if actual.shape!=wanted.shape:raise ValueError('Vertex set changed')
            error=float(np.abs(actual-wanted).max());maximum=max(maximum,error)
            if error>.000001:raise ValueError('Assembled source positions differ')
            if p.get('material')!=source.get('material'):raise ValueError('Material assignment changed')
            bounds.extend(wanted)
            if 'NORMAL' in source['attributes']:
                desired=accessor(original,source_binary,source['attributes']['NORMAL'])@np.linalg.inv(target[:3,:3])
                normals=accessor(doc,binary,p['attributes']['NORMAL'])@np.linalg.inv(frame[:3,:3])
                desired/=np.linalg.norm(desired,axis=1)[:,None];normals/=np.linalg.norm(normals,axis=1)[:,None]
                if not np.allclose(desired,normals,atol=1e-6):raise ValueError('Assembled normals differ')
            source_indices=accessor(original,source_binary,source['indices']).ravel().reshape(-1,3)
            actual_indices=accessor(doc,binary,p['indices']).ravel().reshape(-1,3)
            if np.linalg.det(target[:3,:3])<0:source_indices=source_indices[:,[0,2,1]]
            if not np.array_equal(actual_indices,source_indices):raise ValueError('Triangle winding changed incorrectly')
            checked+=len(source_indices)
    points=np.array(bounds)
    return {'passed':True,'instances':len(roots),'triangles':checked,'max_position_error_metres':maximum,
            'expected_bounds':[points.min(0).tolist(),points.max(0).tolist()],
            'asset_sha256':hashlib.sha256(Path(path).read_bytes()).hexdigest(),
            'scope':'source matrices/geometry/normals/winding and shared immutable prototype assets; no full scene/collision equivalence'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);sub=parser.add_subparsers(dest='mode',required=True)
    a=sub.add_parser('canonical');a.add_argument('source',type=Path);a.add_argument('stage',type=Path);a.add_argument('--probe',required=True,type=Path)
    b=sub.add_parser('assembly');b.add_argument('prototype',type=Path);b.add_argument('instances',type=Path);b.add_argument('asset',type=Path)
    parser.add_argument('--report',type=Path,required=True)
    args=parser.parse_args()
    result=canonical(args.source,args.stage,args.probe) if args.mode=='canonical' else assembly(args.prototype,json.loads(args.instances.read_text())['instances'],args.asset)
    if args.report.exists():raise FileExistsError('Use a new report')
    args.report.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
