"""Write original source corner streams directly after an Editor topology split.

Keep the existing material/image binary prefix. Triangle corners remain distinct
so Blender's custom-normal re-encoding cannot merge or alter source directions.
"""
import copy
import numpy as np


def write(document,binary,reference):
    doc=copy.deepcopy(document);packed=bytearray(binary)
    if len(doc['meshes'])!=1:raise ValueError('Select one static source mesh')
    positions=np.asarray(reference['vertices_z_up_metres'],dtype=float)[:,[0,2,1]];positions[:,2]*=-1
    faces=np.asarray(reference['triangles']);slots=np.asarray(reference['triangle_slots'])
    uv=np.asarray(reference['triangle_uv_blender'],dtype=float).copy();uv[:,:,1]=1-uv[:,:,1]
    normals=np.asarray(reference['triangle_normals_z_up'],dtype=float)[:,:,[0,2,1]];normals[:,:,2]*=-1
    lengths=np.linalg.norm(normals,axis=2)
    if not np.isfinite(normals).all() or (lengths<1e-12).any():raise ValueError('Invalid original source corner normals')
    normals/=lengths[:,:,None]
    if faces.dtype.kind not in 'iu' or slots.dtype.kind not in 'iu' or faces.ndim!=2 or faces.shape[1]!=3 or len(slots)!=len(faces) or faces.min()<0 or faces.max()>=len(positions):raise ValueError('Invalid original source corner topology')
    extra=None
    if 'triangle_uv1_blender' in reference:
        extra=np.asarray(reference['triangle_uv1_blender'],dtype=float).copy();extra[:,:,1]=1-extra[:,:,1]
    def append(values,kind):
        values=np.asarray(values,dtype='<u4' if kind=='SCALAR' else '<f4')
        if not np.isfinite(values).all() or not len(values):raise ValueError('Invalid finite source corner stream')
        packed.extend(b'\0'*(-len(packed)%4));view=len(doc['bufferViews']);doc['bufferViews'].append({'buffer':0,'byteOffset':len(packed),'byteLength':values.nbytes,'target':34963 if kind=='SCALAR' else 34962});packed.extend(values.tobytes())
        index=len(doc['accessors']);entry={'bufferView':view,'componentType':5125 if kind=='SCALAR' else 5126,'type':kind,'count':len(values)}
        if kind=='VEC3':entry.update(min=values.min(0).tolist(),max=values.max(0).tolist())
        doc['accessors'].append(entry);return index
    used=set();counts={}
    for primitive in doc['meshes'][0]['primitives']:
        slot=doc['materials'][primitive['material']]['extras']['theia_source_slot']
        if type(slot) is not int or slot in used or primitive.get('targets') or set(primitive['attributes'])-{'POSITION','NORMAL','TEXCOORD_0','TEXCOORD_1'}:raise ValueError('Unexpected static corner primitive/attributes')
        used.add(slot);mask=slots==slot
        if not mask.any():raise ValueError('Empty source material corner stream')
        vertices=positions[faces[mask]].reshape(-1,3)
        attrs={'POSITION':append(vertices,'VEC3'),'NORMAL':append(normals[mask].reshape(-1,3),'VEC3'),'TEXCOORD_0':append(uv[mask].reshape(-1,2),'VEC2')}
        if extra is not None:attrs['TEXCOORD_1']=append(extra[mask].reshape(-1,2),'VEC2')
        elif 'TEXCOORD_1' in primitive['attributes']:raise ValueError('Missing original secondary UV source')
        primitive['attributes']=attrs;primitive['indices']=append(np.arange(len(vertices),dtype='<u4').reshape(-1,1),'SCALAR');counts[str(slot)]=len(vertices)
    if used!=set(slots.tolist()):raise ValueError('Incomplete source material corner coverage')
    doc['buffers'][0]['byteLength']=len(packed)
    return doc,bytes(packed),{'vertices_by_slot':counts,'scope':'direct original source position/UV/normal corners; Editor triangle/material topology; unchanged material/image bytes'}
