"""Pinned static world-height sky gradient as a standard unlit lookup texture."""
import math
import numpy as np
from unity_materials import properties

GUID='b79e425cce5c90642a8dd66b6852353e'
SHA256='24694dc81dc4d3f6e60bb9974b13397e1a6920d25d56ec11784a8b3bb2ac165c'


def material(fields,digest):
    if fields['m_Shader']!={'fileID':4800000,'guid':GUID,'type':3} or digest!=SHA256:raise ValueError('Unqualified sky shader')
    if fields.get('m_ShaderKeywords') or fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue',-1) not in (-1,2000):raise ValueError('Unsupported sky variant/pass')
    saved=fields['m_SavedProperties'];floats=properties(saved['m_Floats']);colours=properties(saved['m_Colors']);textures=properties(saved['m_TexEnvs'])
    if type(floats['_UV_Based_Toggle']) not in (int,float) or floats['_UV_Based_Toggle']!=0 or any(t['m_Texture'].get('fileID') for t in textures.values()):raise ValueError('Only untextured world-height sky is qualified')
    for name,low,high in [('_Offset',-4096,4096),('_Distance',.001,10000),('_Falloff',.001,100)]:
        v=floats[name]
        if type(v) not in (int,float) or not math.isfinite(v) or not low<=v<=high:raise ValueError('Invalid bounded sky scalar')
    rgb={name:[colours[key][c] for c in 'rgb'] for name,key in [('bottom','_ColorBottom'),('top','_ColorTop')]}
    if any(type(v) not in (int,float) or not math.isfinite(v) or not 0<=v<=1 for values in rgb.values() for v in values):raise ValueError('Unsupported sky HDR/colour scalar')
    return {'profile':'synty-world-height-unlit-sky-v1','shader_sha256':digest,**rgb,'offset':floats['_Offset'],'distance':floats['_Distance'],'falloff':floats['_Falloff'],'lut_height':1024,'lut_mapping':'endpoint-centres-v1','uv_channel':0,
            'scope':'static source world-height gradient through a quantized clamped standard unlit texture; no Unity project colour-space/tonemapping or runtime world adoption parity'}


def lut(profile):
    height=profile['lut_height'];t=1-np.arange(height,dtype=float)/(height-1)
    weight=np.clip(t,0,1)**profile['falloff'];bottom=np.array(profile['bottom']);top=np.array(profile['top']);linear=bottom+(top-bottom)*weight[:,None]
    encoded=np.where(linear<=.0031308,linear*12.92,1.055*np.maximum(linear,0)**(1/2.4)-.055)
    pixels=np.zeros((height,2,4),dtype=np.uint8);pixels[:,:,:3]=np.rint(np.clip(encoded,0,1)*255).astype(np.uint8)[:,None,:];pixels[:,:,3]=255
    return pixels
