"""Hash-specific source triplanar profile and mip-zero reference calculations.

World coordinates remain Unity source coordinates. Bevy conversion reflects X;
PNG sampling flips V. Serialized texture ST and legacy surface fields are unused
by the pinned shader, and are recorded rather than applied accidentally.
"""
import math
import numpy as np
from unity_materials import properties

GUID='66f8401a4c881134f8803eceae302347'
SHA256='d478ba3c6b3476570529dee5f65dc725b07a5bc463ea05bec10757e1209fe118'
ROLES={'top':'_Top','side':'_Sides','top_normal':'_TopNormal','side_normal':'_SidesNormal'}


def material(fields,digest):
    shader=fields['m_Shader']
    if digest!=SHA256 or shader.get('guid')!=GUID or shader.get('fileID')!=4800000:raise ValueError('Unqualified triplanar shader')
    if fields.get('m_ShaderKeywords') or fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue',-1) not in (-1,2000):raise ValueError('Unsupported triplanar variant/pass')
    saved=fields['m_SavedProperties'];floats=properties(saved['m_Floats']);textures=properties(saved['m_TexEnvs'])
    falloff=floats['_FallOff'];tiling=floats['_Tiling']
    for value,low,high in [(falloff,0,100),(tiling,0,10)]:
        if isinstance(value,bool) or not isinstance(value,(int,float)) or not math.isfinite(value) or not low<value<=high or not float(np.float32(value))>0:raise ValueError('Invalid positive triplanar scalar')
    bindings={}
    for role,key in ROLES.items():
        ref=textures[key]['m_Texture']
        if ref.get('fileID')!=2800000 or not isinstance(ref.get('guid'),str):raise ValueError('Triplanar requires explicit texture references')
        bindings[role]=dict(ref)
    return {'profile':'synty-triplanar-world-v1','shader_sha256':digest,'textures':bindings,'falloff':falloff,'tiling':tiling,
            'unused_texture_st':{key:{k:v for k,v in textures[key].items() if k!='m_Texture'} for key in ROLES.values()},
            'base_color':bindings['side'],'metallic':0.,'roughness':1.,'emissive':None,'emissive_factor':[0.,0.,0.],
            'scope':'source world projection and epsilon weights; explicit desktop RG-alpha tangent normal reconstruction; mip/filter/Unity BRDF parity unproven'}


def projection(position,normal,falloff,tiling):
    p=np.asarray(position,dtype=float);n=np.asarray(normal,dtype=float)
    if p.shape!=n.shape or p.shape[-1]!=3 or not np.isfinite(p).all() or not np.isfinite(n).all() or (np.linalg.norm(n,axis=-1)<1e-12).any():raise ValueError('Invalid source projection vectors')
    if not math.isfinite(falloff) or not 0<falloff<=100 or not math.isfinite(tiling) or not 0<tiling<=10:raise ValueError('Invalid projection scalars')
    w=np.abs(n)**falloff;w/=w.sum(axis=-1,keepdims=True)+.00001;sign=np.sign(n)
    weights=np.stack([w[...,0],np.maximum(0,w[...,1]*sign[...,1]),np.maximum(0,-w[...,1]*sign[...,1]),w[...,2]],axis=-1)
    uv=np.stack([p[...,[2,1]]*np.stack([sign[...,0],np.ones_like(sign[...,0])],-1),
                 p[...,[0,2]]*np.stack([sign[...,1],np.ones_like(sign[...,1])],-1),
                 p[...,[0,2]]*np.stack([sign[...,1],np.ones_like(sign[...,1])],-1),
                 p[...,[0,1]]*np.stack([-sign[...,2],np.ones_like(sign[...,2])],-1)],axis=-2)*tiling
    return uv,weights


def repeat_bilinear(pixels,uv,srgb=False):
    a=np.asarray(pixels,dtype=float)/255.;uv=np.asarray(uv,dtype=float)
    if a.ndim!=3 or a.shape[2]!=4 or not np.isfinite(a).all() or not np.isfinite(uv).all():raise ValueError('Invalid triplanar sampling input')
    if srgb:a[...,:3]=np.where(a[...,:3]<=.04045,a[...,:3]/12.92,((a[...,:3]+.055)/1.055)**2.4)
    size=np.array([a.shape[1],a.shape[0]]);coord=np.stack([uv[...,0],1-uv[...,1]],axis=-1)*size-.5;lo=np.floor(coord).astype(np.int64);f=coord-lo
    result=np.zeros((*uv.shape[:-1],4))
    for x,y in [(0,0),(1,0),(0,1),(1,1)]:
        result+=a[(lo[...,1]+y)%size[1],(lo[...,0]+x)%size[0]]*((f[...,0] if x else 1-f[...,0])*(f[...,1] if y else 1-f[...,1]))[...,None]
    return result


def evaluate(images,position,normal,falloff,tiling):
    uv,w=projection(position,normal,falloff,tiling);colour=np.zeros((*w.shape[:-1],4));packed=np.zeros_like(colour)
    for i,role in enumerate(['side','top','side','side']):
        colour+=repeat_bilinear(images[role],uv[...,i,:],True)*w[...,i,None]
        packed+=repeat_bilinear(images[role+'_normal'],uv[...,i,:],False)*w[...,i,None]
    xy=np.stack([packed[...,0]*packed[...,3],packed[...,1]],-1)*2-1
    z=np.sqrt(1-np.clip(np.sum(xy*xy,-1),0,1));tangent=np.concatenate([xy,z[...,None]],-1)
    return colour[...,:3],tangent
