"""Unity source poses to ordinary static glTF nodes, including affine shear.

The calibrated mesh-local convention mirrors X into Unity. Mirror X again for
Unity-to-glTF positions, so canonical RH prototype geometry stays unchanged.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import numpy as np
import trimesh
from merge_static_glb import read
from static_geometry import accessor,hierarchy,local_matrix,transformed_bounds

REFLECTION=np.diag([-1.,1.,1.,1.])


def converted_matrix(matrix):
    matrix=np.asarray(matrix,dtype=np.float64)
    if matrix.shape!=(4,4) or not np.isfinite(matrix).all() or not np.array_equal(matrix[3],[0,0,0,1]) or np.abs(matrix).max()>1e12:
        raise ValueError('Invalid bounded affine source matrix')
    return REFLECTION@matrix@REFLECTION


def normalized_quaternion(values):
    """Remove only floating-point unit-quaternion drift at the glTF boundary."""
    values=np.asarray(values,dtype=float)
    if values.shape!=(4,) or not np.isfinite(values).all() or abs(np.linalg.norm(values)-1)>2e-12:
        raise ValueError('Invalid unit quaternion')
    return (values/np.linalg.norm(values)).tolist()


def factor_pose(matrix):
    """Preserve shear as two ordinary TRS nodes, both with positive determinant."""
    matrix=converted_matrix(matrix);linear=matrix[:3,:3]
    left,singular,right=np.linalg.svd(linear)
    if singular.min()<1e-10 or singular.max()/singular.min()>1e8:
        raise ValueError('Singular/ill-conditioned source transform')
    mirrored=bool(np.linalg.det(linear)<0)
    if mirrored:matrix=matrix@REFLECTION
    left,singular,right=np.linalg.svd(matrix[:3,:3])
    if np.linalg.det(left)<0:
        left[:,-1]*=-1;right[-1,:]*=-1
    def quaternion(rotation):
        value=np.eye(4);value[:3,:3]=rotation
        return normalized_quaternion(trimesh.transformations.quaternion_from_matrix(value)[[1,2,3,0]])
    first={'translation':matrix[:3,3].tolist(),'rotation':quaternion(left),'scale':singular.tolist()}
    second={'rotation':quaternion(right)}
    if not np.allclose(local_matrix(first)@local_matrix(second),matrix,atol=1e-9,rtol=1e-12):
        raise ValueError('Affine factorization lost source pose')
    return first,second,mirrored


def assemble(prototype,instances,output):
    prototype=Path(prototype);output=Path(output)
    if output.exists():raise FileExistsError('Use a new output GLB')
    doc,binary=read(prototype);_,matrices=hierarchy(doc)
    if any(doc.get(key) for key in ('animations','skins')):
        raise ValueError('Prototype must be supported static and unskinned')
    if len(doc.get('buffers',[]))!=1 or 'uri' in doc['buffers'][0] or any('uri' in image for image in doc.get('images',[])):
        raise ValueError('Prototype must contain embedded geometry and images')
    if len(doc['nodes'])!=1 or len(doc['meshes'])!=1 or list(matrices)!=[0] or doc['nodes'][0].get('mesh')!=0 or not np.array_equal(matrices[0],np.eye(4)):
        raise ValueError('Prototype must be one identity canonical mesh node')
    if not 1<=len(instances)<=32768:raise ValueError('Static instance node budget exceeded')
    if any(p.get('mode',4)!=4 or p.get('targets') for p in doc['meshes'][0]['primitives']):
        raise ValueError('Only static triangle prototypes are supported')
    result=copy.deepcopy(doc);packed=bytearray(binary);mirrored_mesh=None
    def append(values,component,kind):
        packed.extend(b'\0'*(-len(packed)%4));offset=len(packed)
        array=np.asarray(values,dtype='<f4' if component==5126 else '<u4');packed.extend(array.tobytes())
        view=len(result['bufferViews']);result['bufferViews'].append({'buffer':0,'byteOffset':offset,'byteLength':array.nbytes,'target':34962 if component==5126 else 34963})
        item={'bufferView':view,'componentType':component,'count':len(array),'type':kind}
        if kind=='VEC3':item.update(min=array.min(0).tolist(),max=array.max(0).tolist())
        index=len(result['accessors']);result['accessors'].append(item);return index
    def mirror_mesh():
        nonlocal mirrored_mesh
        if mirrored_mesh is not None:return mirrored_mesh
        mesh=copy.deepcopy(doc['meshes'][0])
        for p in mesh['primitives']:
            if p.get('mode',4)!=4 or p.get('targets'):raise ValueError('Only static triangle prototypes are supported')
            for key in ['POSITION','NORMAL','TANGENT']:
                if key not in p['attributes']:continue
                source=doc['accessors'][p['attributes'][key]]
                if source['componentType']!=5126:raise ValueError('Mirroring requires float source vectors')
                values=accessor(doc,binary,p['attributes'][key]).copy();values[:,0]*=-1
                if key=='TANGENT':values[:,3]*=-1
                p['attributes'][key]=append(values,5126,'VEC4' if key=='TANGENT' else 'VEC3')
            if 'indices' in p:
                source=doc['accessors'][p['indices']]
                if source['componentType'] not in (5121,5123,5125) or source['type']!='SCALAR':raise ValueError('Invalid triangle indices')
                indices=accessor(doc,binary,p['indices']).ravel()
            else:indices=np.arange(result['accessors'][p['attributes']['POSITION']]['count'])
            if len(indices)%3 or indices.max()>=result['accessors'][p['attributes']['POSITION']]['count']:
                raise ValueError('Triangle indices exceed vertex data')
            p['indices']=append(indices.reshape(-1,3)[:,[0,2,1]].ravel(),5125,'SCALAR')
        mirrored_mesh=len(result['meshes']);result['meshes'].append(mesh);return mirrored_mesh
    nodes=[];roots=[];seen=set();bounds=[];parities=[]
    for instance in instances:
        identifier=instance['id']
        if type(identifier) not in (str,int) or identifier in seen:raise ValueError('Instance IDs must be unique strings/integers')
        seen.add(identifier);first,second,mirrored=factor_pose(instance['matrix']);mesh=mirror_mesh() if mirrored else 0
        root=len(nodes);first.update(name=f'instance-{identifier}',children=[root+1],extras={'source_instance':identifier})
        second.update(name=f'geometry-{identifier}',mesh=mesh);roots.append(root);nodes.extend([first,second]);parities.append(mirrored)
        world=local_matrix(first)@local_matrix(second)
        bound=transformed_bounds(result,packed,mesh,world)
        if np.abs(bound).max()>4096:raise ValueError('Assembly exceeds bounded local rendering frame')
        bounds.append(bound)
    result['nodes']=nodes;result['scenes']=[{'nodes':roots}];result['scene']=0
    result['asset']={'version':'2.0','generator':'Theia static source-pose assembly'}
    result['buffers'][0]['byteLength']=len(packed);packed.extend(b'\0'*(-len(packed)%4))
    text=json.dumps(result,separators=(',',':'),allow_nan=False).encode();text+=b' '*(-len(text)%4)
    output.parent.mkdir(parents=True,exist_ok=True)
    with output.open('xb') as destination:
        destination.write(struct.pack('<4sII',b'glTF',2,28+len(text)+len(packed)))
        destination.write(struct.pack('<I4s',len(text),b'JSON')+text+struct.pack('<I4s',len(packed),b'BIN\0')+packed)
    return {'instances':len(instances),'mirrored_instances':sum(parities),'mesh_variants':len(result['meshes']),
            'materials':len(result.get('materials',[])),'images':len(result.get('images',[])),
            'source_sha256':hashlib.sha256(prototype.read_bytes()).hexdigest(),
            'asset_sha256':hashlib.sha256(output.read_bytes()).hexdigest(),
            'bounds':[np.min([v[0] for v in bounds],axis=0).tolist(),np.max([v[1] for v in bounds],axis=0).tolist()],
            'scope':'static canonical mesh/source pose assembly; not complete Unity scene, material or collision equivalence'}


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('prototype',type=Path)
    parser.add_argument('instances',type=Path,help='JSON object containing an instances list of id/matrix records')
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--report',type=Path,required=True)
    args=parser.parse_args()
    if args.report.exists() or args.report.resolve()==args.output.resolve():
        raise FileExistsError('Use separate new output/report paths')
    report=assemble(args.prototype,json.loads(args.instances.read_text())['instances'],args.output)
    args.report.parent.mkdir(parents=True,exist_ok=True)
    with args.report.open('x') as destination:destination.write(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))
