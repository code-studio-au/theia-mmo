"""Pinned full-vegetation static PBR profile; motion/highlights remain explicit.

Constant noise colours collapse exactly; varying spatial colour fields reject.
Normal maps use the chosen desktop RG reconstruction and source import sampling.
"""
import math
import numpy as np
from unity_materials import properties
from unity_foliage import colour_floor_rgba

SHADER_GUID='6697e1f185bc8b840a2f02567abf15b5'
SHADER_SHA256='87a2b6f6206262674de9b2ff05d3743ee2a62e0d1bc7eff5b06575ab4c56abe4'


def full_material(fields,sha256,colour_uv=False):
    shader=fields['m_Shader']
    if shader.get('guid')!=SHADER_GUID or shader.get('fileID')!=4800000 or sha256!=SHADER_SHA256:raise ValueError('Unqualified full vegetation shader')
    keyword_text=fields.get('m_ShaderKeywords','')
    if keyword_text is None:keyword_text=''
    if not isinstance(keyword_text,str):raise ValueError('Invalid full vegetation keywords')
    keywords=keyword_text.split()
    if set(keywords)-{'_ALPHATEST_ON','_NORMALMAP','_EMISSION'} or len(set(keywords))!=len(keywords) or fields.get('disabledShaderPasses') or fields.get('m_CustomRenderQueue',-1) not in (-1,2450):raise ValueError('Unsupported full vegetation variant/pass')
    saved=fields['m_SavedProperties'];floats=properties(saved['m_Floats']);colours=properties(saved['m_Colors']);textures=properties(saved['m_TexEnvs'])
    def scalar(key):
        v=floats[key]
        if isinstance(v,bool) or not isinstance(v,(int,float)) or not math.isfinite(v) or not 0<=v<=1:raise ValueError('Invalid full vegetation scalar')
        return v
    if floats['_FrostingSwitch']!=0 or floats['_EmissiveAmount']!=0:raise ValueError('Frosting/emissive vegetation requires explicit adaptation')
    for key,value in floats.items():
        if isinstance(value,bool) or not isinstance(value,(int,float)) or not math.isfinite(value):raise ValueError('Nonfinite full vegetation property')
    for key,value in colours.items():
        if key=='_CameraFadeParams':continue  # Unused saved data, absent from the pinned shader.
        if not all(isinstance(v,(int,float)) and math.isfinite(v) for v in value.values()):raise ValueError('Nonfinite full vegetation colour')
    for key in ['_LeafBaseColour','_LeafNoiseColour','_LeafNoiseLargeColour','_TrunkBaseColour','_TrunkNoiseColour']:
        if not all(0<=colours[key][c]<=1 for c in 'rgb'):raise ValueError('Unbounded full vegetation colour')
    if any(colours['_LeafBaseColour'][c]!=colours[key][c] for c in 'rgb' for key in ['_LeafNoiseColour','_LeafNoiseLargeColour']) or any(colours['_TrunkBaseColour'][c]!=colours['_TrunkNoiseColour'][c] for c in 'rgb'):
        raise ValueError('Spatial vegetation colour noise needs explicit source evaluation')
    def texture(key):
        t=textures[key]
        if t['m_Scale']!={'x':1,'y':1} or t['m_Offset']!={'x':0,'y':0}:
            if not colour_uv or key not in ('_LeafTex','_TunkTex'):raise ValueError('Unsupported full vegetation texture transform')
            scale=[t['m_Scale'][k] for k in 'xy'];offset=[t['m_Offset'][k] for k in 'xy']
            if any(type(v) not in (int,float) or not math.isfinite(v) or not 0<v<=16 for v in scale) or any(type(v) not in (int,float) or not math.isfinite(v) or abs(v)>16 for v in offset):raise ValueError('Unbounded foliage colour UV transform')
        ref=t['m_Texture']
        if ref.get('fileID') not in (0,2800000):raise ValueError('Invalid full vegetation texture reference')
        return dict(ref)
    branches={}
    for name,prefix,key in [('leaf','_Leaf','_LeafTex'),('trunk','_Trunk','_TunkTex')]:
        base=texture(key)
        branches[name]={'texture':base,'linear_rgb_floor':[colours[prefix+'BaseColour'][c] for c in 'rgb'],
                        'flat_blend':scalar(prefix+'FlatColourSwitch'),'metallic':scalar(prefix+'Metallic'),
                        'roughness':1-scalar(prefix+'Smoothness'),'normal_texture':texture(prefix+'NormalMap'),
                        'normal_amount':scalar(prefix+'NormalAmount'),'occlusion_texture':texture(prefix+'AmbientOcclusion')}
        transform=textures[key]
        if transform['m_Scale']!={'x':1,'y':1} or transform['m_Offset']!={'x':0,'y':0}:
            branches[name]['colour_uv']={'scale':[transform['m_Scale'][k] for k in 'xy'],'offset':[transform['m_Offset'][k] for k in 'xy']}
    return {'profile':'synty-full-static-foliage-v1','shader_sha256':sha256,'source_keywords':sorted(keywords),'branches':branches,'alpha_cutoff':scalar('_Cutoff'),'double_sided':True,
            'unused_saved_colour_keys':[key for key in colours if key=='_CameraFadeParams'],
            'source_motion_properties':{k:v for k,v in floats.items() if any(s in k for s in ['Wind','Gust','Jitter','LeafBig','Y_multiplier'])},
            'source_gust_highlight':colours['_GustHighlight'],
            'approximations':['authored rest geometry; source wind/jitter and animated gust highlights not applied',
                              'desktop RG normal reconstruction and finite PNG quantization; no Unity platform compression/tangent/BRDF parity',
                              'base-texel normal reconstruction; filtered/mip samples may differ from reconstruction after sampling',
                              'linear PBR colour workflow; no Unity project colour-space reference','no runtime LOD crossfade']}


def full_colour(pixels,floor,flat_blend):
    if not isinstance(flat_blend,(int,float)) or not math.isfinite(flat_blend) or not 0<=flat_blend<=1:raise ValueError('Invalid flat colour blend')
    if flat_blend==0:return colour_floor_rgba(pixels,floor)
    source=np.asarray(pixels)
    if source.dtype!=np.uint8 or source.ndim!=3 or source.shape[2]!=4 or source.shape[0]*source.shape[1]>4096**2:raise ValueError('Expected bounded full foliage RGBA')
    floor=np.asarray(floor,dtype=float)
    if floor.shape!=(3,) or not np.isfinite(floor).all() or (floor<0).any() or (floor>1).any():raise ValueError('Invalid full foliage colour floor')
    rgb=source[:,:,:3].astype(float)/255;rgb=np.where(rgb<=.04045,rgb/12.92,((rgb+.055)/1.055)**2.4)
    rgb=(1-flat_blend)*np.maximum(rgb,floor)+flat_blend*floor;encoded=np.where(rgb<=.0031308,rgb*12.92,1.055*rgb**(1/2.4)-.055)
    result=source.copy();result[:,:,:3]=np.rint(np.clip(encoded,0,1)*255).astype(np.uint8);return result


def normal_rg(pixels,amount,srgb):
    pixels=np.asarray(pixels)
    if pixels.dtype!=np.uint8 or pixels.ndim!=3 or pixels.shape[2]!=4 or pixels.shape[0]*pixels.shape[1]>4096**2 or not math.isfinite(amount) or not 0<=amount<=1:
        raise ValueError('Invalid full foliage normal source')
    raw=pixels.astype(float)/255;rgb=raw[:,:,:3]
    if srgb:rgb=np.where(rgb<=.04045,rgb/12.92,((rgb+.055)/1.055)**2.4)
    xy=np.stack([rgb[:,:,0]*raw[:,:,3],rgb[:,:,1]],axis=-1)*2-1;xy*=amount
    vector=np.concatenate([xy,np.sqrt(np.maximum(0,1-np.sum(xy*xy,axis=2)))[...,None]],axis=2)
    vector/=np.linalg.norm(vector,axis=2)[...,None]
    return np.rint(np.clip(vector*.5+.5,0,1)*255).astype(np.uint8)
