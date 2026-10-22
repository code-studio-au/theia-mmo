"""Repair zero derived tangents only where one source triangle defines a basis.

Positions/UVs/normals/indices remain byte-identical. Ambiguous or degenerate cases
reject. Missing tangent streams stay on the standard Bevy MikkTSpace path.
"""
import copy
import numpy as np
from static_geometry import accessor


def repair_tangents(document,binary):
    result=copy.deepcopy(document);packed=bytearray(binary);repairs=[]
    for mesh_index,mesh in enumerate(document['meshes']):
        for primitive_index,p in enumerate(mesh['primitives']):
            if 'TANGENT' not in p['attributes']:continue
            tangents=accessor(document,binary,p['attributes']['TANGENT']).astype(float)
            lengths=np.linalg.norm(tangents[:,:3],axis=1)
            if not np.isfinite(tangents).all() or not np.isin(tangents[:,3],[-1,1]).all():raise ValueError('Invalid exported tangent stream')
            zeros=np.flatnonzero(lengths<1e-12)
            if ((np.abs(lengths-1)>2e-4)&(lengths>=1e-12)).any():raise ValueError('Exported nonzero tangents exceed unit rounding bound')
            if not len(zeros):continue
            positions=accessor(document,binary,p['attributes']['POSITION']).astype(float);normals=accessor(document,binary,p['attributes']['NORMAL']).astype(float);uv=accessor(document,binary,p['attributes']['TEXCOORD_0']).astype(float)
            faces=accessor(document,binary,p['indices']).reshape(-1,3)
            for index in zeros:
                incident=faces[np.any(faces==index,axis=1)]
                if len(incident)!=1:raise ValueError('Zero tangent needs one unambiguous source triangle')
                a,b,c=incident[0];e1=positions[b]-positions[a];e2=positions[c]-positions[a];d1=uv[b]-uv[a];d2=uv[c]-uv[a];det=d1[0]*d2[1]-d1[1]*d2[0]
                if abs(det)<1e-12 or np.linalg.norm(np.cross(e1,e2))<1e-12:raise ValueError('Zero tangent has degenerate source geometry/UVs')
                normal=normals[index];normal_length=np.linalg.norm(normal)
                if not np.isfinite(normal).all() or normal_length<1e-12:raise ValueError('Zero tangent has invalid source normal')
                normal/=normal_length
                tangent=(e1*d2[1]-e2*d1[1])/det;bitangent=(e2*d1[0]-e1*d2[0])/det
                tangent-=normal*np.dot(normal,tangent);length=np.linalg.norm(tangent)
                if not np.isfinite(tangent).all() or length<1e-12:raise ValueError('Source triangle cannot define an orthogonal tangent')
                tangent/=length;sign=np.dot(np.cross(normal,tangent),bitangent)
                if abs(sign)<1e-12:raise ValueError('Source triangle tangent handedness is ambiguous')
                tangents[index]=[*tangent,1 if sign>0 else -1]
                repairs.append({'mesh':mesh_index,'primitive':primitive_index,'vertex':int(index),'triangle':incident[0].tolist(),
                                'reason':'zero derived tangent; UV derivative of sole incident source triangle'})
            packed.extend(b'\0'*(-len(packed)%4));values=tangents.astype('<f4');view=len(result['bufferViews']);result['bufferViews'].append({'buffer':0,'byteOffset':len(packed),'byteLength':values.nbytes,'target':34962});packed.extend(values.tobytes())
            index=len(result['accessors']);result['accessors'].append({'bufferView':view,'componentType':5126,'type':'VEC4','count':len(values)})
            result['meshes'][mesh_index]['primitives'][primitive_index]['attributes']['TANGENT']=index
    result['buffers'][0]['byteLength']=len(packed);return result,bytes(packed),repairs
