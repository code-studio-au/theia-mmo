"""Append a colour-only UV channel; retain the normal/AO UV and tangent basis."""
import copy
import numpy as np
from static_geometry import accessor


def apply(document, binary, branches):
    doc=copy.deepcopy(document);packed=bytearray(binary)
    for mesh in doc['meshes']:
        for primitive in mesh['primitives']:
            material=doc['materials'][primitive['material']]
            transform=branches[material['name']].get('colour_uv')
            if transform is None:continue
            if 'TEXCOORD_1' in primitive['attributes']:raise ValueError('Existing second UV channel needs separate mapping')
            uv=accessor(document,binary,primitive['attributes']['TEXCOORD_0']).astype(float)
            scale=np.array(transform['scale'],dtype=float);offset=np.array(transform['offset'],dtype=float)
            if scale.shape!=(2,) or offset.shape!=(2,) or not np.isfinite(scale).all() or not np.isfinite(offset).all() or (scale<=0).any() or (scale>16).any() or (abs(offset)>16).any():raise ValueError('Invalid bounded colour UV transform')
            # Unity samples V upward; standard glTF stores the reflected V.
            uv[:,1]=1-uv[:,1];uv=uv*scale+offset;uv[:,1]=1-uv[:,1]
            values=uv.astype('<f4')
            if not np.isfinite(values).all() or abs(values).max()>65536:raise ValueError('Transformed colour UV exceeds asset bound')
            packed.extend(b'\0'*(-len(packed)%4));view=len(doc['bufferViews'])
            doc['bufferViews'].append({'buffer':0,'byteOffset':len(packed),'byteLength':values.nbytes,'target':34962});packed.extend(values.tobytes())
            index=len(doc['accessors']);doc['accessors'].append({'bufferView':view,'componentType':5126,'type':'VEC2','count':len(values)})
            primitive['attributes']['TEXCOORD_1']=index;material['pbrMetallicRoughness']['baseColorTexture']['texCoord']=1
    doc['buffers'][0]['byteLength']=len(packed)
    return doc,bytes(packed)
