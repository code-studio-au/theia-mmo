"""Hash-specific static Synty vegetation adaptation to ordinary glTF PBR.

Preserves source branch colours/cutouts and scalar surfaces. Geometric normals
are an explicit approximation; no custom normal decoding or Unity BRDF parity.
"""
import math
import numpy as np
from unity_materials import properties

BASIC_SHADER_SHA256='f2f800520bffd6081ef699ded2f16e773e209b717cad6695b8786b97b499e9e0'
BASIC_SHADER_GUID='e5cf341aa7a312c4aa1dde88e8f1d2ad'


def vegetation_material(fields, shader_sha256):
    shader=fields['m_Shader']
    if shader_sha256!=BASIC_SHADER_SHA256 or shader.get('guid')!=BASIC_SHADER_GUID or shader.get('fileID')!=4800000:
        raise ValueError('Unqualified vegetation shader source')
    saved=fields['m_SavedProperties'];floats=properties(saved['m_Floats'])
    colors=properties(saved['m_Colors']);textures=properties(saved['m_TexEnvs'])
    if fields.get('m_ShaderKeywords') or fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue',-1) not in (-1,2450):
        raise ValueError('Unsupported vegetation shader variant/pass')
    if floats['_FrostingSwitch']!=0 or floats['_EmissiveAmount']!=0:
        raise ValueError('Frosting/emission needs explicit vegetation adaptation')
    def scalar(key):
        value=floats[key]
        if isinstance(value,bool) or not isinstance(value,(int,float)) or not math.isfinite(value) or not 0<=value<=1:
            raise ValueError('Invalid vegetation surface scalar')
        return value
    branches={}
    for branch,texture_key,prefix in [('leaf','_LeafTex','_Leaf'),('trunk','_TunkTex','_Trunk')]:
        texture=textures[texture_key]
        if texture['m_Scale']!=dict(x=1,y=1) or texture['m_Offset']!=dict(x=0,y=0) or texture['m_Texture'].get('fileID')!=2800000:
            raise ValueError('Unsupported vegetation base texture transform/reference')
        tint=[colors[prefix+'BaseColour'][k] for k in ('r','g','b')]
        if not all(math.isfinite(v) and 0<=v<=1 for v in tint):raise ValueError('Unsupported vegetation tint')
        branches[branch]={'texture':dict(texture['m_Texture']),'linear_rgb_floor':tint,
                          'metallic':scalar(prefix+'Metallic'),'roughness':1-scalar(prefix+'Smoothness')}
    return {'profile':'synty-basic-static-foliage-v1','shader_sha256':shader_sha256,
            'branches':branches,'alpha_cutoff':scalar('_Cutoff'),'double_sided':True,
            'approximations':['geometric normals replace source normal-map output',
                              'linear PBR colour workflow; no Unity project colour-space/BRDF reference',
                              'no runtime LOD crossfade'],
            'normal_maps_not_applied':{key:textures[key]['m_Texture'] for key in ('_LeafNormalMap','_TrunkNormalMap')}}


def colour_floor_rgba(pixels, floor):
    """Apply source max(RGB, tint) in the chosen linear workflow; keep alpha."""
    source=np.asarray(pixels)
    if source.dtype!=np.uint8 or source.ndim!=3 or source.shape[2]!=4 or source.shape[0]*source.shape[1]>4096**2:
        raise ValueError('Expected bounded uint8 RGBA source pixels')
    floor=np.asarray(floor,dtype=np.float64)
    if floor.shape!=(3,) or not np.isfinite(floor).all() or (floor<0).any() or (floor>1).any():
        raise ValueError('Invalid linear colour floor')
    srgb=source[:,:,:3].astype(np.float64)/255.
    linear=np.where(srgb<=.04045,srgb/12.92,((srgb+.055)/1.055)**2.4)
    linear=np.maximum(linear,floor)
    encoded=np.where(linear<=.0031308,12.92*linear,1.055*np.power(linear,1/2.4)-.055)
    result=source.copy();result[:,:,:3]=np.rint(np.clip(encoded,0,1)*255).astype(np.uint8)
    return result


def face_branch(blue):
    values=np.asarray(blue,dtype=np.float64)
    if values.size<3 or not np.isfinite(values).all() or not ((values==0)|(values==1)).all() or not (values==values[0]).all():
        raise ValueError('Nonbinary/mixed-face vertex blue requires fragment-level material selection')
    return 'leaf' if values[0]==1 else 'trunk'
