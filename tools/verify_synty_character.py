#!/usr/bin/env python3
"""Independent glTF pose/skin comparison for the isolated Synty conversion."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import struct
import numpy as np
from static_geometry import local_matrix


def read(path):
    data = path.read_bytes()
    if struct.unpack_from('<4sII', data) != (b'glTF', 2, len(data)):
        raise ValueError('Invalid GLB header')
    size, kind = struct.unpack_from('<I4s', data, 12)
    if kind != b'JSON':raise ValueError('Expected JSON')
    document = json.loads(data[20:20 + size])
    size2, kind = struct.unpack_from('<I4s', data, 20 + size)
    if kind != b'BIN\0' or len(data) != 28 + size + size2:raise ValueError('Expected embedded binary')
    return document, data[28 + size:]


def accessor(document, binary, index):
    item = document['accessors'][index];view = document['bufferViews'][item['bufferView']]
    dtype = np.dtype({5121:'u1',5123:'<u2',5125:'<u4',5126:'<f4'}[item['componentType']])
    width = {'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4,'MAT4':16}[item['type']]
    stride = view.get('byteStride', dtype.itemsize * width)
    values = np.ndarray((item['count'],width), dtype=dtype, buffer=binary,
        offset=view.get('byteOffset',0)+item.get('byteOffset',0),strides=(stride,dtype.itemsize)).copy()
    if item.get('normalized'):
        values = values.astype(float) / np.iinfo(dtype).max
    assert np.isfinite(values).all()
    if width == 16:return values.reshape((-1,4,4)).transpose((0,2,1))
    return values


def sample(times, values, moment, rotation=False, step=False):
    if moment <= times[0]:return values[0]
    if moment >= times[-1]:return values[-1]
    right = int(np.searchsorted(times, moment, side='right'));left = right-1
    if step:return values[left]
    fraction = (moment-times[left])/(times[right]-times[left])
    a,b=values[left].astype(float),values[right].astype(float)
    if rotation:
        dot=float(a@b)
        if dot<0:b=-b;dot=-dot
        if dot<.9995:
            angle=np.arccos(np.clip(dot,-1,1));return (np.sin((1-fraction)*angle)*a+np.sin(fraction*angle)*b)/np.sin(angle)
        value=a+(b-a)*fraction;return value/np.linalg.norm(value)
    return a+(b-a)*fraction


def tracks(document,binary,animation):
    result=[]
    for channel in animation['channels']:
        sampler=animation['samplers'][channel['sampler']]
        assert sampler.get('interpolation','LINEAR') in ('LINEAR','STEP')
        result.append((channel['target']['node'],channel['target']['path'],accessor(document,binary,sampler['input']).ravel(),accessor(document,binary,sampler['output']),sampler.get('interpolation')=='STEP'))
    return result


def world(document, curves=(), moment=0.):
    nodes=copy.deepcopy(document['nodes'])
    for index,path,times,values,step in curves:
        nodes[index][path]=sample(times,values,moment,path=='rotation',step).tolist()
    matrices={}
    def visit(index,parent):
        assert index not in matrices
        matrices[index]=parent@local_matrix(nodes[index])
        for child in nodes[index].get('children',[]):visit(child,matrices[index])
    for index in document['scenes'][document.get('scene',0)]['nodes']:visit(index,np.eye(4))
    return matrices


def skin(document,binary):
    assert len(document['skins'])==1 and len(document['skins'][0]['joints'])==50
    node=next(n for n in document['nodes'] if 'skin' in n)
    primitive=document['meshes'][node['mesh']]['primitives'][0]
    attributes=primitive['attributes']
    points=accessor(document,binary,attributes['POSITION'])
    joints=[];weights=[]
    for index in range(2):
        if f'JOINTS_{index}' in attributes:
            joints.append(accessor(document,binary,attributes[f'JOINTS_{index}']))
            weights.append(accessor(document,binary,attributes[f'WEIGHTS_{index}']))
    joints=np.concatenate(joints,axis=1).astype(int);weights=np.concatenate(weights,axis=1)
    assert joints.max()<50 and np.all(weights>=0) and np.allclose(weights.sum(axis=1),1.,atol=1e-5)
    matrices=accessor(document,binary,document['skins'][0]['inverseBindMatrices'])
    return np.column_stack([points,np.ones(len(points))]),joints,weights,matrices


def deformed(document, data, matrices):
    points,joints,weights,inverse=data
    transforms=np.array([matrices[index] for index in document['skins'][0]['joints']])@inverse
    positions=np.einsum('nkij,nj->nki',transforms[joints],points)
    return np.einsum('nk,nki->ni',weights,positions)[:,:3]


def verify(directory):
    proof=directory/'character-verification.json';proof.unlink(missing_ok=True)
    master,master_binary=read(directory/'character-source-skin.glb')
    preview,preview_binary=read(directory/'character.glb')
    reference=json.loads((directory/'source-pose-reference.json').read_bytes())
    full=skin(master,master_binary);four=skin(preview,preview_binary)
    assert full[1].shape[1]>4 and four[1].shape[1]==4
    np.testing.assert_array_equal(full[0],four[0])
    assert master['nodes']==preview['nodes']
    by_name={node['name']:index for index,node in enumerate(master['nodes'])}
    assert len(by_name)==len(master['nodes'])
    # Bone local basis may differ after export. Joint positions and world-space
    # deformation deltas are independent of that fixed basis choice.
    conversion=np.array([[1,0,0,0],[0,0,1,0],[0,-1,0,0],[0,0,0,1]],dtype=float)
    results={}
    for animation in master['animations']:
        name=animation['name'];curves=tracks(master,master_binary,animation)
        preview_animation=next(a for a in preview['animations'] if a['name']==name)
        preview_curves=tracks(preview,preview_binary,preview_animation)
        first=world(master,curves,0.)
        baseline={bone:np.array(value) for bone,value in reference[name][0]['bones_blender_world'].items()}
        position_error=delta_error=skin_error=0.
        low=np.full(3,np.inf);high=np.full(3,-np.inf)
        for frame in reference[name]:
            actual=world(master,curves,frame['time'])
            for bone,expected in frame['bones_blender_world'].items():
                expected=np.array(expected);matrix=actual[by_name[bone]]
                position_error=max(position_error,float(np.linalg.norm(matrix[:3,3]-(conversion@expected)[:3,3])))
                source_delta=conversion@expected@np.linalg.inv(baseline[bone])@conversion.T
                exported_delta=matrix@np.linalg.inv(first[by_name[bone]])
                delta_error=max(delta_error,float(np.max(abs(source_delta-exported_delta))))
            preview_world=world(preview,preview_curves,frame['time'])
            source_vertices=deformed(master,full,actual)
            low=np.minimum(low,source_vertices.min(axis=0));high=np.maximum(high,source_vertices.max(axis=0))
            skin_error=max(skin_error,float(np.linalg.norm(source_vertices-deformed(preview,four,preview_world),axis=1).max()))
        assert position_error<.0001,(name,position_error)
        assert delta_error<.0005,(name,delta_error)
        results[name]={'source_joint_position_error_metres':position_error,'source_world_deformation_delta_error':delta_error,
                       'full_to_four_weight_skin_error_metres':skin_error,'frames':len(reference[name]),
                       'four_weight_skin_within_5mm':skin_error<=.005,
                       'animated_bounds_metres':[low.tolist(),high.tolist()]}
    assert set(results)=={'Idle','Walk','Run','Jump'}
    result={'passed_source_pose_comparison':True,'clips':results,'preview_vertices':len(four[0]),'bones':50,
            'preview_sha256':hashlib.sha256((directory/'character.glb').read_bytes()).hexdigest(),
            'master_sha256':hashlib.sha256((directory/'character-source-skin.glb').read_bytes()).hexdigest(),
            'scope':'independent source joint/animation and standard four-weight approximation comparison; not game adoption'}
    proof.write_text(json.dumps(result,indent=2)+'\n');return result


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('directory',type=Path)
    print(json.dumps(verify(parser.parse_args().directory),indent=2))
